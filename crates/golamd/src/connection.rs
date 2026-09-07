#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::desktop_control::{DesktopControlMode, HumanInterruptOperation};
use golam_core::paths::RuntimeLayout;
use golam_core::{ClientId, PROTOCOL_VERSION, ResourceLimits};
use golam_ipc::desktop_control::{
    DesktopControlRequest, DesktopControlSurfaceReply, DesktopControlSurfaceState,
    decode_desktop_control_request, encode_desktop_control_reply, is_desktop_control_method,
};
use golam_ipc::lifecycle::{
    Authenticate, ClientKeyId, ConnectionId, Hello, LifecycleError, LifecycleMessage,
    LifecyclePhase, ServerLifecycle,
};
use golam_ipc::request::{
    ClientAction, ReplyMessage, ReplyStatus, RequestMessage, RequestProtocolError,
    ServerRequestTracker, decode_request, encode_reply,
};
use golam_ipc::wire::{WireError, read_frame, write_frame};
use golam_ipc::{FrameHeader, FrameKind};
use golam_kernel::{
    AuthorizationPolicy, BootstrapPolicy, ClientEnrollmentError, ClientKind, DesktopAuthorityError,
    KernelApi, KernelError, Principal,
};
use golamd::CommandRouter;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConnectionMaterial {
    pub server_epoch: u64,
    pub server_nonce: [u8; 32],
    pub connection_id: ConnectionId,
    pub limits: ResourceLimits,
}

pub trait BootstrapApprover {
    fn approve(&mut self, client_id: ClientId, key_id: ClientKeyId) -> Option<ClientKind>;
}

#[derive(Debug)]
pub enum ConnectionError {
    Wire(WireError),
    Lifecycle(LifecycleError),
    Kernel(KernelError),
    Enrollment(ClientEnrollmentError),
    Request(RequestProtocolError),
    UnexpectedFrame {
        expected: FrameKind,
        actual: FrameKind,
    },
    BootstrapDenied {
        client_id: ClientId,
    },
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wire(error) => write!(f, "daemon IPC wire error: {error}"),
            Self::Lifecycle(error) => write!(f, "daemon IPC lifecycle error: {error}"),
            Self::Kernel(error) => write!(f, "daemon kernel error: {error}"),
            Self::Enrollment(error) => write!(f, "daemon client enrollment error: {error}"),
            Self::Request(error) => write!(f, "daemon request protocol error: {error}"),
            Self::UnexpectedFrame { expected, actual } => {
                write!(f, "daemon expected {expected:?} frame, received {actual:?}")
            }
            Self::BootstrapDenied { client_id } => write!(
                f,
                "local bootstrap enrollment was not approved for client {}",
                client_id.0
            ),
        }
    }
}

impl Error for ConnectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Wire(error) => Some(error),
            Self::Lifecycle(error) => Some(error),
            Self::Kernel(error) => Some(error),
            Self::Enrollment(error) => Some(error),
            Self::Request(error) => Some(error),
            Self::UnexpectedFrame { .. } | Self::BootstrapDenied { .. } => None,
        }
    }
}

impl From<WireError> for ConnectionError {
    fn from(value: WireError) -> Self {
        Self::Wire(value)
    }
}

impl From<LifecycleError> for ConnectionError {
    fn from(value: LifecycleError) -> Self {
        Self::Lifecycle(value)
    }
}

impl From<KernelError> for ConnectionError {
    fn from(value: KernelError) -> Self {
        Self::Kernel(value)
    }
}

impl From<ClientEnrollmentError> for ConnectionError {
    fn from(value: ClientEnrollmentError) -> Self {
        Self::Enrollment(value)
    }
}

impl From<RequestProtocolError> for ConnectionError {
    fn from(value: RequestProtocolError) -> Self {
        Self::Request(value)
    }
}

pub fn serve_connection<S: Read + Write, A: BootstrapApprover, P: AuthorizationPolicy>(
    stream: &mut S,
    runtime: &RuntimeLayout,
    router: &mut CommandRouter<P>,
    material: ConnectionMaterial,
    approver: &mut A,
) -> Result<(), ConnectionError> {
    let mut auth_kernel = KernelApi::open(runtime, BootstrapPolicy::default())?;
    let mut lifecycle = ServerLifecycle::new(
        material.server_epoch,
        material.server_nonce,
        material.limits,
        material.connection_id,
    )?;

    let hello_frame = read_frame(stream, material.limits)?;
    if hello_frame.header.kind != FrameKind::Hello {
        return Err(ConnectionError::UnexpectedFrame {
            expected: FrameKind::Hello,
            actual: hello_frame.header.kind,
        });
    }
    let hello = match LifecycleMessage::decode(hello_frame.header.kind, &hello_frame.payload)? {
        LifecycleMessage::Hello(hello) => hello,
        _ => unreachable!("hello frame decodes to hello lifecycle message"),
    };
    let challenge = lifecycle.receive_hello(hello)?;
    write_lifecycle(
        stream,
        LifecycleMessage::Challenge(challenge),
        material.limits,
    )?;

    let authenticate_frame = read_frame(stream, challenge.limits)?;
    if matches!(
        authenticate_frame.header.kind,
        FrameKind::Request | FrameKind::Cancel
    ) {
        auth_kernel.reject_unauthenticated_request(
            &mut lifecycle,
            material.connection_id,
            hello.client_id,
            None,
            &timestamp_now(),
        )?;
        return Err(ConnectionError::UnexpectedFrame {
            expected: FrameKind::Authenticate,
            actual: authenticate_frame.header.kind,
        });
    }
    if authenticate_frame.header.kind != FrameKind::Authenticate {
        return Err(ConnectionError::UnexpectedFrame {
            expected: FrameKind::Authenticate,
            actual: authenticate_frame.header.kind,
        });
    }
    let authenticate = match LifecycleMessage::decode(
        authenticate_frame.header.kind,
        &authenticate_frame.payload,
    )? {
        LifecycleMessage::Authenticate(authenticate) => authenticate,
        _ => unreachable!("authenticate frame decodes to authenticate lifecycle message"),
    };

    authenticate_with_optional_bootstrap(
        &mut auth_kernel,
        &mut lifecycle,
        material.connection_id,
        hello,
        authenticate,
        approver,
    )?;

    let ready = auth_kernel.authenticate_registered_client(
        &mut lifecycle,
        material.connection_id,
        hello.client_id,
        authenticate,
        &timestamp_now(),
    )?;
    let client_record = auth_kernel.active_client_record(hello.client_id, authenticate.key_id)?;
    write_lifecycle(stream, LifecycleMessage::Ready(ready), ready.limits)?;

    let mut tracker = ServerRequestTracker::new(LifecyclePhase::Ready, ready.limits)?;
    let principal = Principal::enrolled_client(client_subject(client_record.kind), hello.client_id);

    loop {
        let frame = read_frame(stream, ready.limits)?;
        if frame.header.kind == FrameKind::Shutdown {
            let reason = match LifecycleMessage::decode(frame.header.kind, &frame.payload)? {
                LifecycleMessage::Shutdown(reason) => reason,
                _ => unreachable!("shutdown frame decodes to shutdown lifecycle message"),
            };
            lifecycle.receive_shutdown(reason)?;
            tracker.close();
            return Ok(());
        }

        match tracker.receive_client_frame(frame.header, &frame.payload)? {
            ClientAction::Begin { request_id, method } => {
                let request = decode_request(&frame.payload)?;
                debug_assert_eq!(request.method, method);
                let reply = if is_desktop_control_method(request.method) {
                    match unix_time_ms() {
                        Some(now_unix_ms) => route_desktop_control(
                            &auth_kernel,
                            client_record.kind,
                            principal,
                            &request,
                            now_unix_ms,
                        ),
                        None => desktop_error_reply(
                            ReplyStatus::Failed,
                            "desktop_clock_unavailable",
                        ),
                    }
                } else {
                    router.route(principal, &request, &timestamp_now(), "local-ipc")
                };
                write_reply(stream, request_id.0, &reply, ready.limits, &mut tracker)?;
            }
            ClientAction::Cancel { request_id }
            | ClientAction::CancelAlreadyRequested { request_id } => {
                let reply = ReplyMessage {
                    status: ReplyStatus::Cancelled,
                    body: Vec::new(),
                };
                write_reply(stream, request_id.0, &reply, ready.limits, &mut tracker)?;
            }
        }
    }
}

fn route_desktop_control<P: AuthorizationPolicy>(
    kernel: &KernelApi<P>,
    client_kind: ClientKind,
    principal: Principal<'_>,
    request: &RequestMessage,
    now_unix_ms: u64,
) -> ReplyMessage {
    if client_kind != ClientKind::DesktopFuture {
        return desktop_error_reply(ReplyStatus::Denied, "desktop_host_unauthorized");
    }

    let request = match decode_desktop_control_request(request) {
        Ok(request) => request,
        Err(_) => {
            return desktop_error_reply(
                ReplyStatus::InvalidRequest,
                "desktop_control_request_invalid",
            );
        }
    };

    let result = match request {
        DesktopControlRequest::Status => kernel
            .desktop_control_surface_snapshot(now_unix_ms)
            .map(|snapshot| {
                desktop_surface_reply(snapshot.mode, snapshot.visible_channel_qualified)
            }),
        DesktopControlRequest::Heartbeat => kernel
            .observe_desktop_native_visible_channel(principal, true, now_unix_ms)
            .and_then(|_| kernel.desktop_control_surface_snapshot(now_unix_ms))
            .map(|snapshot| {
                desktop_surface_reply(snapshot.mode, snapshot.visible_channel_qualified)
            }),
        DesktopControlRequest::Pause => kernel
            .apply_native_desktop_human_interrupt(
                principal,
                HumanInterruptOperation::Pause,
                now_unix_ms,
                now_unix_ms,
            )
            .map(|snapshot| {
                desktop_surface_reply(snapshot.mode, snapshot.visible_channel_qualified)
            }),
        DesktopControlRequest::Stop => kernel
            .apply_native_desktop_human_interrupt(
                principal,
                HumanInterruptOperation::Stop,
                now_unix_ms,
                now_unix_ms,
            )
            .map(|snapshot| {
                desktop_surface_reply(snapshot.mode, snapshot.visible_channel_qualified)
            }),
        DesktopControlRequest::Takeover => kernel
            .apply_native_desktop_human_interrupt(
                principal,
                HumanInterruptOperation::Takeover,
                now_unix_ms,
                now_unix_ms,
            )
            .map(|snapshot| {
                desktop_surface_reply(snapshot.mode, snapshot.visible_channel_qualified)
            }),
    };

    match result {
        Ok(reply) => ReplyMessage {
            status: ReplyStatus::Ok,
            body: encode_desktop_control_reply(reply).to_vec(),
        },
        Err(error) => desktop_authority_error_reply(error),
    }
}

fn desktop_surface_reply(
    mode: Option<DesktopControlMode>,
    visible_channel_qualified: bool,
) -> DesktopControlSurfaceReply {
    let state = match mode {
        None => DesktopControlSurfaceState::NoLease,
        Some(DesktopControlMode::AgentAllowed) => DesktopControlSurfaceState::AgentAllowed,
        Some(DesktopControlMode::Paused) => DesktopControlSurfaceState::Paused,
        Some(DesktopControlMode::HumanExclusive) => DesktopControlSurfaceState::HumanExclusive,
        Some(DesktopControlMode::Revoked) => DesktopControlSurfaceState::Revoked,
    };
    DesktopControlSurfaceReply {
        state,
        visible_channel_qualified,
    }
}

fn desktop_authority_error_reply(error: DesktopAuthorityError) -> ReplyMessage {
    let (status, code) = match error {
        DesktopAuthorityError::UnauthorizedDesktopHost => {
            (ReplyStatus::Denied, "desktop_host_unauthorized")
        }
        DesktopAuthorityError::NoQualifiedVisibleChannel => {
            (ReplyStatus::Denied, "desktop_visible_channel_required")
        }
        DesktopAuthorityError::InvalidInterruptTransition
        | DesktopAuthorityError::LeaseExpiredBeforeInterrupt => {
            (ReplyStatus::Denied, "desktop_interrupt_denied")
        }
        DesktopAuthorityError::MissingDesktopControlLease => {
            (ReplyStatus::Failed, "desktop_control_lease_unavailable")
        }
        DesktopAuthorityError::AmbiguousDesktopControlLease => {
            (ReplyStatus::Failed, "desktop_control_state_ambiguous")
        }
        _ => (ReplyStatus::Failed, "desktop_control_failed"),
    };
    desktop_error_reply(status, code)
}

fn desktop_error_reply(status: ReplyStatus, code: &str) -> ReplyMessage {
    ReplyMessage {
        status,
        body: format!("error={code}\n").into_bytes(),
    }
}

fn authenticate_with_optional_bootstrap<A: BootstrapApprover>(
    kernel: &mut KernelApi<BootstrapPolicy>,
    lifecycle: &mut ServerLifecycle,
    connection_id: ConnectionId,
    hello: Hello,
    authenticate: Authenticate,
    approver: &mut A,
) -> Result<(), ConnectionError> {
    if !kernel.client_requires_bootstrap_enrollment(hello.client_id, authenticate.key_id)? {
        return Ok(());
    }

    let Some(kind) = approver.approve(hello.client_id, authenticate.key_id) else {
        let _ = kernel.authenticate_registered_client(
            lifecycle,
            connection_id,
            hello.client_id,
            authenticate,
            &timestamp_now(),
        );
        return Err(ConnectionError::BootstrapDenied {
            client_id: hello.client_id,
        });
    };

    kernel.enroll_precreated_client(
        Principal::local_owner("local-owner"),
        hello.client_id,
        authenticate.key_id,
        kind,
        &timestamp_now(),
        "local-bootstrap",
    )?;
    Ok(())
}

const fn client_subject(kind: ClientKind) -> &'static str {
    match kind {
        ClientKind::Cli => "local-cli",
        ClientKind::DesktopFuture => "local-desktop",
        ClientKind::IdeFuture => "local-ide",
        ClientKind::Test => "local-test",
    }
}

fn write_lifecycle<S: Write>(
    stream: &mut S,
    message: LifecycleMessage,
    limits: ResourceLimits,
) -> Result<(), WireError> {
    let kind = message.frame_kind();
    let payload = message.encode_payload();
    let header = FrameHeader {
        protocol_version: PROTOCOL_VERSION,
        kind,
        request_id: None,
        payload_len: u32::try_from(payload.len()).expect("lifecycle payload length fits u32"),
    };
    write_frame(stream, header, &payload, limits)
}

fn write_reply<S: Write>(
    stream: &mut S,
    request_id: u64,
    reply: &ReplyMessage,
    limits: ResourceLimits,
    tracker: &mut ServerRequestTracker,
) -> Result<(), ConnectionError> {
    let payload = encode_reply(reply);
    let header = FrameHeader {
        protocol_version: PROTOCOL_VERSION,
        kind: FrameKind::Reply,
        request_id: Some(request_id),
        payload_len: u32::try_from(payload.len()).expect("reply payload length fits u32"),
    };
    write_frame(stream, header, &payload, limits)?;
    let _ = tracker.settle_server_frame(header, &payload)?;
    Ok(())
}

fn unix_time_ms() -> Option<u64> {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_millis();
    u64::try_from(millis).ok().filter(|value| *value != 0)
}

fn timestamp_now() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => format!("unix:{}.{:09}", duration.as_secs(), duration.subsec_nanos()),
        Err(_) => "unix:0.000000000".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::authority::AuthorityLayout;
    use golam_core::desktop_control::{
        DESKTOP_CONTROL_SCHEMA_VERSION, DesktopControlLeaseId, DesktopControlLeaseState,
    };
    use golam_core::tool_request::BindingDigest;
    use golam_core::{ClientId, PROTOCOL_VERSION};
    use golam_ipc::FrameKind;
    use golam_ipc::client_handshake::sign_authenticate;
    use golam_ipc::command::{Command, encode_command};
    use golam_ipc::credentials::ClientCredentialStore;
    use golam_ipc::desktop_control::{decode_desktop_control_reply, encode_desktop_control_request};
    use golam_ipc::lifecycle::{Challenge, ShutdownReason};
    use golam_ipc::request::{ReplyStatus, decode_reply, encode_request};
    use golam_ipc::wire::read_frame;
    use golam_kernel::ProtectedDesktopControlState;
    use std::fs;
    use std::io::{self, Cursor};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    struct ScriptedIo {
        input: Cursor<Vec<u8>>,
        output: Vec<u8>,
    }

    impl ScriptedIo {
        fn new(input: Vec<u8>) -> Self {
            Self {
                input: Cursor::new(input),
                output: Vec::new(),
            }
        }
    }

    impl Read for ScriptedIo {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            self.input.read(buffer)
        }
    }

    impl Write for ScriptedIo {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.output.extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct Approval {
        kind: Option<ClientKind>,
        calls: usize,
    }

    impl BootstrapApprover for Approval {
        fn approve(&mut self, _client_id: ClientId, _key_id: ClientKeyId) -> Option<ClientKind> {
            self.calls += 1;
            self.kind
        }
    }

    fn runtime() -> RuntimeLayout {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        RuntimeLayout::initialize(
            std::env::temp_dir().join(format!("golamd-connection-{}-{t}-{n}", std::process::id())),
        )
        .unwrap()
    }

    fn material() -> ConnectionMaterial {
        ConnectionMaterial {
            server_epoch: 91,
            server_nonce: [7; 32],
            connection_id: ConnectionId(92),
            limits: ResourceLimits::default(),
        }
    }

    fn authenticated_input(
        store: &ClientCredentialStore<'_>,
        client_id: ClientId,
        key_id: ClientKeyId,
        command: Option<Command>,
    ) -> Vec<u8> {
        let requests = command
            .map(|command| vec![encode_command(&command).unwrap()])
            .unwrap_or_default();
        authenticated_requests_input(store, client_id, key_id, &requests)
    }

    fn authenticated_requests_input(
        store: &ClientCredentialStore<'_>,
        client_id: ClientId,
        key_id: ClientKeyId,
        requests: &[RequestMessage],
    ) -> Vec<u8> {
        let material = material();
        let client_nonce = [5; 32];
        let hello = Hello {
            protocol_version: PROTOCOL_VERSION,
            client_id,
            client_nonce,
        };
        let challenge = Challenge {
            protocol_version: PROTOCOL_VERSION,
            server_epoch: material.server_epoch,
            server_nonce: material.server_nonce,
            limits: material.limits,
        };
        let signing_key = store.load(client_id, key_id).unwrap();
        let authenticate = sign_authenticate(hello, challenge, key_id, &signing_key).unwrap();

        let hello_payload = LifecycleMessage::Hello(hello).encode_payload();
        let mut bytes =
            golam_ipc::encode_frame(FrameKind::Hello, None, &hello_payload, material.limits)
                .unwrap();
        let authenticate_payload = LifecycleMessage::Authenticate(authenticate).encode_payload();
        bytes.extend_from_slice(
            &golam_ipc::encode_frame(
                FrameKind::Authenticate,
                None,
                &authenticate_payload,
                material.limits,
            )
            .unwrap(),
        );
        for (index, request) in requests.iter().enumerate() {
            let payload = encode_request(request).unwrap();
            bytes.extend_from_slice(
                &golam_ipc::encode_frame(
                    FrameKind::Request,
                    Some((index + 1) as u64),
                    &payload,
                    material.limits,
                )
                .unwrap(),
            );
        }
        let shutdown_payload = LifecycleMessage::Shutdown(ShutdownReason::Normal).encode_payload();
        bytes.extend_from_slice(
            &golam_ipc::encode_frame(
                FrameKind::Shutdown,
                None,
                &shutdown_payload,
                material.limits,
            )
            .unwrap(),
        );
        bytes
    }

    fn decoded_replies(output: Vec<u8>, count: usize) -> Vec<ReplyMessage> {
        let mut output = Cursor::new(output);
        assert_eq!(
            read_frame(&mut output, material().limits)
                .unwrap()
                .header
                .kind,
            FrameKind::Challenge
        );
        assert_eq!(
            read_frame(&mut output, material().limits)
                .unwrap()
                .header
                .kind,
            FrameKind::Ready
        );
        (0..count)
            .map(|_| {
                let reply = read_frame(&mut output, material().limits).unwrap();
                assert_eq!(reply.header.kind, FrameKind::Reply);
                decode_reply(&reply.payload).unwrap()
            })
            .collect()
    }

    fn persist_desktop_lease(runtime: &RuntimeLayout, lease_id: u128) -> DesktopControlLeaseId {
        let now = unix_time_ms().unwrap();
        let lease_id = DesktopControlLeaseId::from_u128(lease_id);
        let state = ProtectedDesktopControlState::new(
            DesktopControlLeaseState {
                schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
                lease_id,
                generation: 1,
                controlling_principal_ref: BindingDigest::new([1; 32]),
                mode: DesktopControlMode::AgentAllowed,
                issued_at_unix_ms: now,
                updated_at_unix_ms: now,
                expires_at_unix_ms: now.checked_add(120_000).unwrap(),
                capability_ref: BindingDigest::new([2; 32]),
                policy_ref: BindingDigest::new([3; 32]),
                interrupt_cause_ref: None,
            },
            Vec::new(),
        )
        .unwrap();
        let kernel = KernelApi::open(runtime, BootstrapPolicy::default()).unwrap();
        kernel.persist_desktop_control_state(&state).unwrap();
        drop(kernel);
        lease_id
    }

    #[test]
    fn unknown_precreated_client_requires_approval_then_authenticates_same_transcript() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let store = ClientCredentialStore::new(&authority);
        let generated = store.generate(ClientId(2001)).unwrap();
        let input = authenticated_input(
            &store,
            generated.client_id,
            generated.key_id,
            Some(Command::SessionsList),
        );
        let mut io = ScriptedIo::new(input);
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let mut router = CommandRouter::new(kernel);
        let mut approval = Approval {
            kind: Some(ClientKind::Cli),
            calls: 0,
        };

        serve_connection(&mut io, &runtime, &mut router, material(), &mut approval).unwrap();
        assert_eq!(approval.calls, 1);

        let replies = decoded_replies(io.output, 1);
        assert_eq!(replies[0].status, ReplyStatus::Ok);

        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        assert!(
            !kernel
                .client_requires_bootstrap_enrollment(generated.client_id, generated.key_id)
                .unwrap()
        );
        assert_eq!(
            kernel
                .active_client_record(generated.client_id, generated.key_id)
                .unwrap()
                .kind,
            ClientKind::Cli
        );
        drop(kernel);
        drop(router);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn desktop_bootstrap_kind_is_trusted_approval_output() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let store = ClientCredentialStore::new(&authority);
        let generated = store.generate(ClientId(2010)).unwrap();
        let mut io = ScriptedIo::new(authenticated_input(
            &store,
            generated.client_id,
            generated.key_id,
            None,
        ));
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let mut router = CommandRouter::new(kernel);
        let mut approval = Approval {
            kind: Some(ClientKind::DesktopFuture),
            calls: 0,
        };

        serve_connection(&mut io, &runtime, &mut router, material(), &mut approval).unwrap();
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let record = kernel
            .active_client_record(generated.client_id, generated.key_id)
            .unwrap();
        assert_eq!(record.kind, ClientKind::DesktopFuture);
        assert_eq!(client_subject(record.kind), "local-desktop");
        drop(kernel);
        drop(router);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn desktop_future_control_requests_route_through_protected_kernel_state() {
        let runtime = runtime();
        let lease_id = persist_desktop_lease(&runtime, 55);
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let store = ClientCredentialStore::new(&authority);
        let generated = store.generate(ClientId(2011)).unwrap();
        let requests = [
            encode_desktop_control_request(DesktopControlRequest::Heartbeat),
            encode_desktop_control_request(DesktopControlRequest::Pause),
            encode_desktop_control_request(DesktopControlRequest::Takeover),
            encode_desktop_control_request(DesktopControlRequest::Stop),
            encode_desktop_control_request(DesktopControlRequest::Status),
        ];
        let mut io = ScriptedIo::new(authenticated_requests_input(
            &store,
            generated.client_id,
            generated.key_id,
            &requests,
        ));
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let mut router = CommandRouter::new(kernel);
        let mut approval = Approval {
            kind: Some(ClientKind::DesktopFuture),
            calls: 0,
        };

        serve_connection(&mut io, &runtime, &mut router, material(), &mut approval).unwrap();
        let replies = decoded_replies(io.output, requests.len());
        assert!(replies.iter().all(|reply| reply.status == ReplyStatus::Ok));
        let states = replies
            .iter()
            .map(|reply| decode_desktop_control_reply(&reply.body).unwrap().state)
            .collect::<Vec<_>>();
        assert_eq!(
            states,
            vec![
                DesktopControlSurfaceState::AgentAllowed,
                DesktopControlSurfaceState::Paused,
                DesktopControlSurfaceState::HumanExclusive,
                DesktopControlSurfaceState::Revoked,
                DesktopControlSurfaceState::Revoked,
            ]
        );
        assert!(
            replies
                .iter()
                .all(|reply| decode_desktop_control_reply(&reply.body)
                    .unwrap()
                    .visible_channel_qualified)
        );

        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let state = kernel
            .restore_desktop_control_state(lease_id)
            .unwrap()
            .unwrap();
        assert_eq!(state.current_lease().generation, 4);
        assert_eq!(state.current_lease().mode, DesktopControlMode::Revoked);
        assert!(!state.autonomous_actuation_allowed(unix_time_ms().unwrap()));
        drop(kernel);
        drop(router);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn stale_desktop_heartbeat_blocks_agent_actuation_while_lease_remains_agent_allowed() {
        let runtime = runtime();
        let lease_id = persist_desktop_lease(&runtime, 56);
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let store = ClientCredentialStore::new(&authority);
        let generated = store.generate(ClientId(2012)).unwrap();
        let requests = [encode_desktop_control_request(
            DesktopControlRequest::Heartbeat,
        )];
        let mut io = ScriptedIo::new(authenticated_requests_input(
            &store,
            generated.client_id,
            generated.key_id,
            &requests,
        ));
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let mut router = CommandRouter::new(kernel);
        let mut approval = Approval {
            kind: Some(ClientKind::DesktopFuture),
            calls: 0,
        };

        serve_connection(&mut io, &runtime, &mut router, material(), &mut approval).unwrap();
        let replies = decoded_replies(io.output, 1);
        let heartbeat = decode_desktop_control_reply(&replies[0].body).unwrap();
        assert_eq!(heartbeat.state, DesktopControlSurfaceState::AgentAllowed);
        assert!(heartbeat.visible_channel_qualified);

        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let state = kernel
            .restore_desktop_control_state(lease_id)
            .unwrap()
            .unwrap();
        assert_eq!(
            state.current_lease().mode,
            DesktopControlMode::AgentAllowed
        );
        let stale_at = unix_time_ms().unwrap().checked_add(10_000).unwrap();
        assert!(!state.autonomous_actuation_allowed(stale_at));
        assert!(
            !kernel
                .desktop_control_surface_snapshot(stale_at)
                .unwrap()
                .visible_channel_qualified
        );
        drop(kernel);
        drop(router);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn cli_principal_cannot_invoke_desktop_human_control_authority() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let store = ClientCredentialStore::new(&authority);
        let generated = store.generate(ClientId(2013)).unwrap();
        let requests = [encode_desktop_control_request(DesktopControlRequest::Status)];
        let mut io = ScriptedIo::new(authenticated_requests_input(
            &store,
            generated.client_id,
            generated.key_id,
            &requests,
        ));
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let mut router = CommandRouter::new(kernel);
        let mut approval = Approval {
            kind: Some(ClientKind::Cli),
            calls: 0,
        };

        serve_connection(&mut io, &runtime, &mut router, material(), &mut approval).unwrap();
        let replies = decoded_replies(io.output, 1);
        assert_eq!(replies[0].status, ReplyStatus::Denied);
        assert_eq!(replies[0].body, b"error=desktop_host_unauthorized\n");
        drop(router);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn desktop_control_request_body_cannot_supply_authority_identifiers() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let store = ClientCredentialStore::new(&authority);
        let generated = store.generate(ClientId(2014)).unwrap();
        let mut request = encode_desktop_control_request(DesktopControlRequest::Pause);
        request.body.extend_from_slice(b"lease=renderer-supplied");
        let mut io = ScriptedIo::new(authenticated_requests_input(
            &store,
            generated.client_id,
            generated.key_id,
            &[request],
        ));
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let mut router = CommandRouter::new(kernel);
        let mut approval = Approval {
            kind: Some(ClientKind::DesktopFuture),
            calls: 0,
        };

        serve_connection(&mut io, &runtime, &mut router, material(), &mut approval).unwrap();
        let replies = decoded_replies(io.output, 1);
        assert_eq!(replies[0].status, ReplyStatus::InvalidRequest);
        assert_eq!(
            replies[0].body,
            b"error=desktop_control_request_invalid\n"
        );
        drop(router);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn denied_bootstrap_does_not_register_unknown_client() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let store = ClientCredentialStore::new(&authority);
        let generated = store.generate(ClientId(2002)).unwrap();
        let mut io = ScriptedIo::new(authenticated_input(
            &store,
            generated.client_id,
            generated.key_id,
            None,
        ));
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let mut router = CommandRouter::new(kernel);
        let mut approval = Approval {
            kind: None,
            calls: 0,
        };

        assert!(matches!(
            serve_connection(&mut io, &runtime, &mut router, material(), &mut approval,),
            Err(ConnectionError::BootstrapDenied {
                client_id: ClientId(2002)
            })
        ));
        assert_eq!(approval.calls, 1);
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        assert!(
            kernel
                .client_requires_bootstrap_enrollment(generated.client_id, generated.key_id)
                .unwrap()
        );
        drop(kernel);
        drop(router);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn request_before_authenticate_is_rejected_before_router_dispatch() {
        let runtime = runtime();
        let client_id = ClientId(2003);
        let hello = Hello {
            protocol_version: PROTOCOL_VERSION,
            client_id,
            client_nonce: [4; 32],
        };
        let hello_payload = LifecycleMessage::Hello(hello).encode_payload();
        let mut input =
            golam_ipc::encode_frame(FrameKind::Hello, None, &hello_payload, material().limits)
                .unwrap();
        let request = encode_command(&Command::SessionsList).unwrap();
        let request_payload = encode_request(&request).unwrap();
        input.extend_from_slice(
            &golam_ipc::encode_frame(
                FrameKind::Request,
                Some(1),
                &request_payload,
                material().limits,
            )
            .unwrap(),
        );
        let mut io = ScriptedIo::new(input);
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let mut router = CommandRouter::new(kernel);
        let mut approval = Approval {
            kind: Some(ClientKind::Cli),
            calls: 0,
        };

        assert!(matches!(
            serve_connection(&mut io, &runtime, &mut router, material(), &mut approval,),
            Err(ConnectionError::UnexpectedFrame {
                expected: FrameKind::Authenticate,
                actual: FrameKind::Request
            })
        ));
        assert_eq!(approval.calls, 0);
        drop(router);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn desktop_request_before_authenticate_is_rejected_before_ready() {
        let runtime = runtime();
        let client_id = ClientId(2004);
        let hello = Hello {
            protocol_version: PROTOCOL_VERSION,
            client_id,
            client_nonce: [6; 32],
        };
        let hello_payload = LifecycleMessage::Hello(hello).encode_payload();
        let mut input =
            golam_ipc::encode_frame(FrameKind::Hello, None, &hello_payload, material().limits)
                .unwrap();
        let request = encode_desktop_control_request(DesktopControlRequest::Status);
        let request_payload = encode_request(&request).unwrap();
        input.extend_from_slice(
            &golam_ipc::encode_frame(
                FrameKind::Request,
                Some(1),
                &request_payload,
                material().limits,
            )
            .unwrap(),
        );
        let mut io = ScriptedIo::new(input);
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let mut router = CommandRouter::new(kernel);
        let mut approval = Approval {
            kind: Some(ClientKind::DesktopFuture),
            calls: 0,
        };

        assert!(matches!(
            serve_connection(&mut io, &runtime, &mut router, material(), &mut approval,),
            Err(ConnectionError::UnexpectedFrame {
                expected: FrameKind::Authenticate,
                actual: FrameKind::Request
            })
        ));
        assert_eq!(approval.calls, 0);
        drop(router);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
