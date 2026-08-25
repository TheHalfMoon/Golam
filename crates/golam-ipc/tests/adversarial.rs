use ed25519_dalek::Signer;
use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;
use golam_core::{ClientId, PROTOCOL_VERSION, ResourceLimits};
use golam_ipc::credentials::ClientCredentialStore;
use golam_ipc::enrollment::{EnrollmentError, LocalClientEnrollment};
use golam_ipc::lifecycle::{
    AuthTranscript, Authenticate, Challenge, ConnectionId, Hello, LifecycleError, LifecycleMessage,
    LifecyclePhase, NONCE_LEN, ServerLifecycle,
};
use golam_ipc::request::{
    ClientAction, MethodId, PendingState, ReplyMessage, ReplyStatus, RequestMessage,
    RequestProtocolError, ServerAction, ServerRequestTracker, Settlement, encode_reply,
    encode_request,
};
use golam_ipc::{FRAME_HEADER_LEN, FrameHeader, FrameKind, IpcError};
use golam_ledger::clients::{ClientKind, ClientRegistryError};
use golam_ledger::protocol_audit::ProtocolRejectionReason;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static N: AtomicU64 = AtomicU64::new(0);

fn authority() -> (RuntimeLayout, AuthorityLayout) {
    let n = N.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
        "golam-adversarial-ipc-{}-{t}-{n}",
        std::process::id()
    )))
    .unwrap();
    let authority = AuthorityLayout::initialize(&runtime).unwrap();
    (runtime, authority)
}

fn header(kind: FrameKind, request_id: Option<u64>, payload_len: usize) -> FrameHeader {
    FrameHeader {
        protocol_version: PROTOCOL_VERSION,
        kind,
        request_id,
        payload_len: u32::try_from(payload_len).unwrap(),
    }
}

fn request(method: u16, body: &[u8]) -> Vec<u8> {
    encode_request(&RequestMessage {
        method: MethodId(method),
        body: body.to_vec(),
    })
    .unwrap()
}

fn authenticate(
    signing: &ed25519_dalek::SigningKey,
    client_id: ClientId,
    key_id: golam_ipc::lifecycle::ClientKeyId,
    client_nonce: [u8; NONCE_LEN],
    server_nonce: [u8; NONCE_LEN],
    server_epoch: u64,
    limits: ResourceLimits,
) -> Authenticate {
    let hello = Hello {
        protocol_version: PROTOCOL_VERSION,
        client_id,
        client_nonce,
    };
    let challenge = Challenge {
        protocol_version: PROTOCOL_VERSION,
        server_epoch,
        server_nonce,
        limits,
    };
    let transcript = AuthTranscript::from_messages(hello, challenge).unwrap();
    let signature = signing
        .sign(&transcript.canonical_bytes(key_id).unwrap())
        .to_bytes();
    Authenticate {
        key_id,
        client_nonce,
        signature,
    }
}

#[test]
fn request_tracker_rejects_resource_direction_length_and_race_attacks() {
    let payload = request(1, b"");

    let tight = ResourceLimits {
        max_frame_bytes: u32::try_from(FRAME_HEADER_LEN + 1).unwrap(),
        max_pending_requests: 2,
        ..ResourceLimits::default()
    };
    let mut oversized = ServerRequestTracker::new(LifecyclePhase::Ready, tight).unwrap();
    assert!(matches!(
        oversized.receive_client_frame(
            header(FrameKind::Request, Some(1), payload.len()),
            &payload
        ),
        Err(RequestProtocolError::Ipc(IpcError::FrameTooLarge { .. }))
    ));
    assert!(oversized.is_closed());

    let single = ResourceLimits {
        max_pending_requests: 1,
        ..ResourceLimits::default()
    };
    let mut exhausted = ServerRequestTracker::new(LifecyclePhase::Ready, single).unwrap();
    exhausted
        .receive_client_frame(
            header(FrameKind::Request, Some(1), payload.len()),
            &payload,
        )
        .unwrap();
    assert!(matches!(
        exhausted.receive_client_frame(
            header(FrameKind::Request, Some(2), payload.len()),
            &payload
        ),
        Err(RequestProtocolError::PendingLimitExceeded { maximum: 1 })
    ));
    assert!(exhausted.is_closed());

    let mut duplicate = ServerRequestTracker::new(LifecyclePhase::Ready, single).unwrap();
    duplicate
        .receive_client_frame(
            header(FrameKind::Request, Some(7), payload.len()),
            &payload,
        )
        .unwrap();
    assert!(matches!(
        duplicate.receive_client_frame(
            header(FrameKind::Request, Some(7), payload.len()),
            &payload
        ),
        Err(RequestProtocolError::DuplicateRequestId { .. })
    ));

    let mut malformed =
        ServerRequestTracker::new(LifecyclePhase::Ready, ResourceLimits::default()).unwrap();
    assert!(matches!(
        malformed.receive_client_frame(
            header(FrameKind::Request, Some(3), payload.len() + 1),
            &payload
        ),
        Err(RequestProtocolError::PayloadLengthMismatch { .. })
    ));

    let mut client_direction =
        ServerRequestTracker::new(LifecyclePhase::Ready, ResourceLimits::default()).unwrap();
    assert!(matches!(
        client_direction.receive_client_frame(header(FrameKind::Reply, Some(4), 2), &[0, 0]),
        Err(RequestProtocolError::ImpossibleClientDirection {
            kind: FrameKind::Reply
        })
    ));

    let mut server_direction =
        ServerRequestTracker::new(LifecyclePhase::Ready, ResourceLimits::default()).unwrap();
    assert!(matches!(
        server_direction.settle_server_frame(
            header(FrameKind::Request, Some(5), payload.len()),
            &payload
        ),
        Err(RequestProtocolError::ImpossibleServerDirection {
            kind: FrameKind::Request
        })
    ));

    let mut race =
        ServerRequestTracker::new(LifecyclePhase::Ready, ResourceLimits::default()).unwrap();
    assert_eq!(
        race.receive_client_frame(
            header(FrameKind::Request, Some(9), payload.len()),
            &payload
        )
        .unwrap(),
        ClientAction::Begin {
            request_id: golam_ipc::request::RequestId(9),
            method: MethodId(1),
        }
    );
    let reply = encode_reply(&ReplyMessage {
        status: ReplyStatus::Ok,
        body: Vec::new(),
    });
    assert_eq!(
        race.settle_server_frame(header(FrameKind::Reply, Some(9), reply.len()), &reply)
            .unwrap(),
        ServerAction::Reply {
            settlement: Settlement {
                request_id: golam_ipc::request::RequestId(9),
                state: PendingState::Active,
            },
            message: ReplyMessage {
                status: ReplyStatus::Ok,
                body: Vec::new(),
            },
        }
    );
    assert!(matches!(
        race.receive_client_frame(header(FrameKind::Cancel, Some(9), 0), b""),
        Err(RequestProtocolError::UnknownRequestId { .. })
    ));
    assert!(race.is_closed());

    assert!(matches!(
        ServerRequestTracker::new(LifecyclePhase::ChallengeSent, ResourceLimits::default()),
        Err(RequestProtocolError::NotReady {
            phase: LifecyclePhase::ChallengeSent
        })
    ));
}

#[test]
fn lifecycle_rejects_malformed_repeated_and_nonce_mismatch_probes() {
    assert!(LifecycleMessage::decode(FrameKind::Hello, &[0]).is_err());

    let limits = ResourceLimits::default();
    let hello = Hello {
        protocol_version: PROTOCOL_VERSION,
        client_id: ClientId(600),
        client_nonce: [1; NONCE_LEN],
    };
    let mut repeated =
        ServerLifecycle::new(40, [2; NONCE_LEN], limits, ConnectionId(200)).unwrap();
    repeated.receive_hello(hello).unwrap();
    assert!(matches!(
        repeated.receive_hello(hello),
        Err(LifecycleError::InvalidPhase { .. })
    ));
    assert_eq!(repeated.phase(), LifecyclePhase::Closed);

    let signing = ed25519_dalek::SigningKey::from_bytes(&[7; 32]);
    let key_id = golam_ipc::lifecycle::ClientKeyId([9; 32]);
    let mut nonce_mismatch =
        ServerLifecycle::new(41, [3; NONCE_LEN], limits, ConnectionId(201)).unwrap();
    nonce_mismatch.receive_hello(hello).unwrap();
    let mut auth = authenticate(
        &signing,
        hello.client_id,
        key_id,
        hello.client_nonce,
        [3; NONCE_LEN],
        41,
        limits,
    );
    auth.client_nonce = [4; NONCE_LEN];
    let enrolled = golam_ipc::lifecycle::EnrolledClientKey {
        key_id,
        verifying_key: signing.verifying_key(),
    };
    assert_eq!(
        nonce_mismatch.authenticate(auth, &enrolled),
        Err(LifecycleError::ClientNonceMismatch)
    );
    assert_eq!(nonce_mismatch.phase(), LifecyclePhase::Closed);
}

#[test]
fn unknown_wrong_revoked_replay_and_pre_ready_probes_are_audited() {
    let (runtime, authority) = authority();
    let store = ClientCredentialStore::new(&authority);
    let enrolled = store.generate(ClientId(701)).unwrap();
    let enrolled_signing = store.load(enrolled.client_id, enrolled.key_id).unwrap();
    let unknown = store.generate(ClientId(702)).unwrap();
    let unknown_signing = store.load(unknown.client_id, unknown.key_id).unwrap();
    let mut enrollment = LocalClientEnrollment::open(&authority).unwrap();
    enrollment
        .enroll_generated(
            &enrolled,
            ClientKind::Test,
            "owner",
            "2026-08-25T02:00:00Z",
        )
        .unwrap();
    let limits = ResourceLimits::default();

    let mut unknown_server =
        ServerLifecycle::new(50, [10; NONCE_LEN], limits, ConnectionId(300)).unwrap();
    let unknown_hello = Hello {
        protocol_version: PROTOCOL_VERSION,
        client_id: unknown.client_id,
        client_nonce: [11; NONCE_LEN],
    };
    unknown_server.receive_hello(unknown_hello).unwrap();
    assert!(matches!(
        enrollment.authenticate_registered(
            &mut unknown_server,
            ConnectionId(300),
            unknown.client_id,
            authenticate(
                &unknown_signing,
                unknown.client_id,
                unknown.key_id,
                unknown_hello.client_nonce,
                [10; NONCE_LEN],
                50,
                limits,
            ),
            "2026-08-25T02:01:00Z",
        ),
        Err(EnrollmentError::Registry(
            ClientRegistryError::UnknownClient
        ))
    ));
    assert_eq!(unknown_server.phase(), LifecyclePhase::Closed);

    let mut wrong_key_server =
        ServerLifecycle::new(51, [12; NONCE_LEN], limits, ConnectionId(301)).unwrap();
    let enrolled_hello = Hello {
        protocol_version: PROTOCOL_VERSION,
        client_id: enrolled.client_id,
        client_nonce: [13; NONCE_LEN],
    };
    wrong_key_server.receive_hello(enrolled_hello).unwrap();
    assert!(matches!(
        enrollment.authenticate_registered(
            &mut wrong_key_server,
            ConnectionId(301),
            enrolled.client_id,
            authenticate(
                &unknown_signing,
                enrolled.client_id,
                unknown.key_id,
                enrolled_hello.client_nonce,
                [12; NONCE_LEN],
                51,
                limits,
            ),
            "2026-08-25T02:02:00Z",
        ),
        Err(EnrollmentError::Registry(
            ClientRegistryError::ClientKeyMismatch
        ))
    ));
    assert_eq!(wrong_key_server.phase(), LifecyclePhase::Closed);

    let captured_hello = Hello {
        protocol_version: PROTOCOL_VERSION,
        client_id: enrolled.client_id,
        client_nonce: [14; NONCE_LEN],
    };
    let captured = authenticate(
        &enrolled_signing,
        enrolled.client_id,
        enrolled.key_id,
        captured_hello.client_nonce,
        [15; NONCE_LEN],
        52,
        limits,
    );
    let mut valid =
        ServerLifecycle::new(52, [15; NONCE_LEN], limits, ConnectionId(302)).unwrap();
    valid.receive_hello(captured_hello).unwrap();
    enrollment
        .authenticate_registered(
            &mut valid,
            ConnectionId(302),
            enrolled.client_id,
            captured,
            "2026-08-25T02:03:00Z",
        )
        .unwrap();

    let mut replay =
        ServerLifecycle::new(53, [16; NONCE_LEN], limits, ConnectionId(303)).unwrap();
    replay.receive_hello(captured_hello).unwrap();
    assert!(matches!(
        enrollment.authenticate_registered(
            &mut replay,
            ConnectionId(303),
            enrolled.client_id,
            captured,
            "2026-08-25T02:04:00Z",
        ),
        Err(EnrollmentError::Lifecycle(
            LifecycleError::AuthenticationFailed
        ))
    ));
    assert_eq!(replay.phase(), LifecyclePhase::Closed);

    enrollment
        .revoke(enrolled.client_id, "2026-08-25T02:05:00Z")
        .unwrap();
    let mut revoked =
        ServerLifecycle::new(54, [17; NONCE_LEN], limits, ConnectionId(304)).unwrap();
    let revoked_hello = Hello {
        client_nonce: [18; NONCE_LEN],
        ..captured_hello
    };
    revoked.receive_hello(revoked_hello).unwrap();
    assert!(matches!(
        enrollment.authenticate_registered(
            &mut revoked,
            ConnectionId(304),
            enrolled.client_id,
            authenticate(
                &enrolled_signing,
                enrolled.client_id,
                enrolled.key_id,
                revoked_hello.client_nonce,
                [17; NONCE_LEN],
                54,
                limits,
            ),
            "2026-08-25T02:06:00Z",
        ),
        Err(EnrollmentError::Registry(
            ClientRegistryError::RevokedClient
        ))
    ));
    assert_eq!(revoked.phase(), LifecyclePhase::Closed);

    let mut pre_ready =
        ServerLifecycle::new(55, [19; NONCE_LEN], limits, ConnectionId(305)).unwrap();
    pre_ready
        .receive_hello(Hello {
            client_nonce: [20; NONCE_LEN],
            ..captured_hello
        })
        .unwrap();
    enrollment
        .reject_unauthenticated_request(
            &mut pre_ready,
            ConnectionId(305),
            enrolled.client_id,
            None,
            "2026-08-25T02:07:00Z",
        )
        .unwrap();
    assert_eq!(pre_ready.phase(), LifecyclePhase::Closed);

    let records = enrollment.protocol_audit_records().unwrap();
    let reasons: Vec<_> = records.iter().map(|record| record.reason).collect();
    assert_eq!(
        reasons,
        vec![
            ProtocolRejectionReason::UnknownClient,
            ProtocolRejectionReason::ClientKeyMismatch,
            ProtocolRejectionReason::AuthenticationFailed,
            ProtocolRejectionReason::RevokedClient,
            ProtocolRejectionReason::UnauthenticatedRequest,
        ]
    );
    drop(enrollment);
    fs::remove_dir_all(runtime.root).unwrap();
}
