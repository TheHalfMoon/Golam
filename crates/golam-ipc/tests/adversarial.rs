use ed25519_dalek::Signer;
use golam_core::{ClientId, PROTOCOL_VERSION, ResourceLimits};
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
    Authenticate {
        key_id,
        client_nonce,
        signature: signing
            .sign(&transcript.canonical_bytes(key_id).unwrap())
            .to_bytes(),
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
        oversized
            .receive_client_frame(header(FrameKind::Request, Some(1), payload.len()), &payload),
        Err(RequestProtocolError::Ipc(IpcError::FrameTooLarge { .. }))
    ));
    assert!(oversized.is_closed());

    let single = ResourceLimits {
        max_pending_requests: 1,
        ..ResourceLimits::default()
    };
    let mut exhausted = ServerRequestTracker::new(LifecyclePhase::Ready, single).unwrap();
    exhausted
        .receive_client_frame(header(FrameKind::Request, Some(1), payload.len()), &payload)
        .unwrap();
    assert!(matches!(
        exhausted
            .receive_client_frame(header(FrameKind::Request, Some(2), payload.len()), &payload),
        Err(RequestProtocolError::PendingLimitExceeded { maximum: 1 })
    ));
    assert!(exhausted.is_closed());

    let mut duplicate = ServerRequestTracker::new(LifecyclePhase::Ready, single).unwrap();
    duplicate
        .receive_client_frame(header(FrameKind::Request, Some(7), payload.len()), &payload)
        .unwrap();
    assert!(matches!(
        duplicate
            .receive_client_frame(header(FrameKind::Request, Some(7), payload.len()), &payload),
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
        server_direction
            .settle_server_frame(header(FrameKind::Request, Some(5), payload.len()), &payload),
        Err(RequestProtocolError::ImpossibleServerDirection {
            kind: FrameKind::Request
        })
    ));

    let mut race =
        ServerRequestTracker::new(LifecyclePhase::Ready, ResourceLimits::default()).unwrap();
    assert_eq!(
        race.receive_client_frame(header(FrameKind::Request, Some(9), payload.len()), &payload)
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
    let mut repeated = ServerLifecycle::new(40, [2; NONCE_LEN], limits, ConnectionId(200)).unwrap();
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
