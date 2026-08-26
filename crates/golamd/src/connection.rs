#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::paths::RuntimeLayout;
use golam_core::{ClientId, PROTOCOL_VERSION, ResourceLimits};
use golam_ipc::lifecycle::{
    Authenticate, ClientKeyId, ConnectionId, Hello, LifecycleError, LifecycleMessage,
    LifecyclePhase, ServerLifecycle,
};
use golam_ipc::request::{
    ClientAction, ReplyMessage, ReplyStatus, RequestProtocolError, ServerRequestTracker,
    decode_request, encode_reply,
};
use golam_ipc::wire::{WireError, read_frame, write_frame};
use golam_ipc::{FrameHeader, FrameKind};
use golam_kernel::{
    BootstrapPolicy, ClientEnrollmentError, ClientKind, KernelApi, KernelError, Principal,
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
    fn approve(&mut self, client_id: ClientId, key_id: ClientKeyId) -> bool;
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

pub fn serve_connection<S: Read + Write, A: BootstrapApprover>(
    stream: &mut S,
    runtime: &RuntimeLayout,
    router: &mut CommandRouter<BootstrapPolicy>,
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
    write_lifecycle(stream, LifecycleMessage::Ready(ready), ready.limits)?;

    let mut tracker = ServerRequestTracker::new(LifecyclePhase::Ready, ready.limits)?;
    let principal = Principal::enrolled_client("local-cli", hello.client_id);

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
                let reply = router.route(principal, &request, &timestamp_now(), "local-ipc");
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

    if !approver.approve(hello.client_id, authenticate.key_id) {
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
    }

    kernel.enroll_precreated_client(
        Principal::local_owner("local-owner"),
        hello.client_id,
        authenticate.key_id,
        ClientKind::Cli,
        &timestamp_now(),
        "local-bootstrap",
    )?;
    Ok(())
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

fn timestamp_now() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => format!("unix:{}.{:09}", duration.as_secs(), duration.subsec_nanos()),
        Err(_) => "unix:0.000000000".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;
    use golam_core::authority::AuthorityLayout;
    use golam_core::{ClientId, PROTOCOL_VERSION};
    use golam_ipc::command::{Command, encode_command};
    use golam_ipc::credentials::ClientCredentialStore;
    use golam_ipc::lifecycle::{AuthTranscript, Challenge, ShutdownReason};
    use golam_ipc::request::{ReplyStatus, decode_reply, encode_request};
    use golam_ipc::wire::read_frame;
    use golam_ipc::{FrameKind, encode_frame};
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
        allow: bool,
        calls: usize,
    }

    impl BootstrapApprover for Approval {
        fn approve(&mut self, _client_id: ClientId, _key_id: ClientKeyId) -> bool {
            self.calls += 1;
            self.allow
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
        client_id: ClientId,
        key_id: ClientKeyId,
        signing_key: &ed25519_dalek::SigningKey,
        command: Option<Command>,
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
        let transcript = AuthTranscript::from_messages(hello, challenge).unwrap();
        let authenticate = Authenticate {
            key_id,
            client_nonce,
            signature: signing_key
                .sign(&transcript.canonical_bytes(key_id).unwrap())
                .to_bytes(),
        };

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
        if let Some(command) = command {
            let message = encode_command(&command).unwrap();
            let payload = encode_request(&message).unwrap();
            bytes.extend_from_slice(
                &golam_ipc::encode_frame(FrameKind::Request, Some(1), &payload, material.limits)
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

    #[test]
    fn unknown_precreated_client_requires_approval_then_authenticates_same_transcript() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let store = ClientCredentialStore::new(&authority);
        let generated = store.generate(ClientId(2001)).unwrap();
        let signing_key = store.load(generated.client_id, generated.key_id).unwrap();
        let input = authenticated_input(
            generated.client_id,
            generated.key_id,
            &signing_key,
            Some(Command::SessionsList),
        );
        let mut io = ScriptedIo::new(input);
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let mut router = CommandRouter::new(kernel);
        let mut approval = Approval {
            allow: true,
            calls: 0,
        };

        serve_connection(&mut io, &runtime, &mut router, material(), &mut approval).unwrap();
        assert_eq!(approval.calls, 1);

        let mut output = Cursor::new(io.output);
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
        let reply = read_frame(&mut output, material().limits).unwrap();
        assert_eq!(reply.header.kind, FrameKind::Reply);
        assert_eq!(
            decode_reply(&reply.payload).unwrap().status,
            ReplyStatus::Ok
        );

        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        assert!(
            !kernel
                .client_requires_bootstrap_enrollment(generated.client_id, generated.key_id)
                .unwrap()
        );
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn denied_bootstrap_does_not_register_unknown_client() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let store = ClientCredentialStore::new(&authority);
        let generated = store.generate(ClientId(2002)).unwrap();
        let signing_key = store.load(generated.client_id, generated.key_id).unwrap();
        let mut io = ScriptedIo::new(authenticated_input(
            generated.client_id,
            generated.key_id,
            &signing_key,
            None,
        ));
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let mut router = CommandRouter::new(kernel);
        let mut approval = Approval {
            allow: false,
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
            allow: true,
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
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
