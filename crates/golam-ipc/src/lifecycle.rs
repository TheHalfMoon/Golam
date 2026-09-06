use std::error::Error;
use std::fmt;

use ed25519_dalek::{Signature, VerifyingKey};
use golam_core::{CanonicalEncoder, ClientId, CoreError, PROTOCOL_VERSION, ResourceLimits};

use crate::FrameKind;

const AUTH_TRANSCRIPT_DOMAIN: &[u8] = b"golam:ipc-auth-transcript:v1";
pub const NONCE_LEN: usize = 32;
pub const CLIENT_KEY_ID_LEN: usize = 32;
pub const SIGNATURE_LEN: usize = 64;
const HELLO_PAYLOAD_LEN: usize = 50;
const CHALLENGE_PAYLOAD_LEN: usize = 52;
const AUTHENTICATE_PAYLOAD_LEN: usize = 128;
const READY_PAYLOAD_LEN: usize = 34;
const SHUTDOWN_PAYLOAD_LEN: usize = 1;

pub type ClientNonce = [u8; NONCE_LEN];
pub type ServerNonce = [u8; NONCE_LEN];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientKeyId(pub [u8; CLIENT_KEY_ID_LEN]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConnectionId(pub u128);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecyclePhase {
    AwaitHello,
    ChallengeSent,
    Ready,
    Closed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Hello {
    pub protocol_version: u16,
    pub client_id: ClientId,
    pub client_nonce: ClientNonce,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Challenge {
    pub protocol_version: u16,
    pub server_epoch: u64,
    pub server_nonce: ServerNonce,
    pub limits: ResourceLimits,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Authenticate {
    pub key_id: ClientKeyId,
    pub client_nonce: ClientNonce,
    pub signature: [u8; SIGNATURE_LEN],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ready {
    pub connection_id: ConnectionId,
    pub server_epoch: u64,
    pub limits: ResourceLimits,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthenticatedLocalClient {
    client_id: ClientId,
    connection_id: ConnectionId,
    server_epoch: u64,
    limits: ResourceLimits,
}

impl AuthenticatedLocalClient {
    pub const fn client_id(self) -> ClientId {
        self.client_id
    }

    pub const fn connection_id(self) -> ConnectionId {
        self.connection_id
    }

    pub const fn server_epoch(self) -> u64 {
        self.server_epoch
    }

    pub const fn limits(self) -> ResourceLimits {
        self.limits
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShutdownReason {
    Normal,
    ProtocolViolation,
    AuthenticationFailed,
    ServerStopping,
}

impl ShutdownReason {
    const fn code(self) -> u8 {
        match self {
            Self::Normal => 1,
            Self::ProtocolViolation => 2,
            Self::AuthenticationFailed => 3,
            Self::ServerStopping => 4,
        }
    }

    const fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Normal),
            2 => Some(Self::ProtocolViolation),
            3 => Some(Self::AuthenticationFailed),
            4 => Some(Self::ServerStopping),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleMessage {
    Hello(Hello),
    Challenge(Challenge),
    Authenticate(Authenticate),
    Ready(Ready),
    Shutdown(ShutdownReason),
}

impl LifecycleMessage {
    pub const fn frame_kind(self) -> FrameKind {
        match self {
            Self::Hello(_) => FrameKind::Hello,
            Self::Challenge(_) => FrameKind::Challenge,
            Self::Authenticate(_) => FrameKind::Authenticate,
            Self::Ready(_) => FrameKind::Ready,
            Self::Shutdown(_) => FrameKind::Shutdown,
        }
    }

    pub fn encode_payload(self) -> Vec<u8> {
        match self {
            Self::Hello(message) => encode_hello(message),
            Self::Challenge(message) => encode_challenge(message),
            Self::Authenticate(message) => encode_authenticate(message),
            Self::Ready(message) => encode_ready(message),
            Self::Shutdown(reason) => vec![reason.code()],
        }
    }

    pub fn decode(frame_kind: FrameKind, payload: &[u8]) -> Result<Self, LifecycleError> {
        match frame_kind {
            FrameKind::Hello => Ok(Self::Hello(decode_hello(payload)?)),
            FrameKind::Challenge => Ok(Self::Challenge(decode_challenge(payload)?)),
            FrameKind::Authenticate => Ok(Self::Authenticate(decode_authenticate(payload)?)),
            FrameKind::Ready => Ok(Self::Ready(decode_ready(payload)?)),
            FrameKind::Shutdown => Ok(Self::Shutdown(decode_shutdown(payload)?)),
            kind => Err(LifecycleError::NotLifecycleFrame { kind }),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthTranscript {
    pub protocol_version: u16,
    pub client_id: ClientId,
    pub client_nonce: ClientNonce,
    pub server_nonce: ServerNonce,
    pub server_epoch: u64,
    pub limits: ResourceLimits,
}

impl AuthTranscript {
    pub fn from_messages(hello: Hello, challenge: Challenge) -> Result<Self, LifecycleError> {
        validate_protocol_version(hello.protocol_version)?;
        validate_protocol_version(challenge.protocol_version)?;
        if hello.protocol_version != challenge.protocol_version {
            return Err(LifecycleError::TranscriptProtocolMismatch);
        }
        validate_nonce(hello.client_nonce)?;
        validate_nonce(challenge.server_nonce)?;
        validate_server_epoch(challenge.server_epoch)?;
        Ok(Self {
            protocol_version: hello.protocol_version,
            client_id: hello.client_id,
            client_nonce: hello.client_nonce,
            server_nonce: challenge.server_nonce,
            server_epoch: challenge.server_epoch,
            limits: challenge.limits,
        })
    }

    pub fn canonical_bytes(self, key_id: ClientKeyId) -> Result<Vec<u8>, LifecycleError> {
        validate_key_id(key_id)?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(AUTH_TRANSCRIPT_DOMAIN)?;
        encoder.push_u16(self.protocol_version);
        encoder.push_u128(self.client_id.0);
        encoder.push_bytes(&self.client_nonce)?;
        encoder.push_bytes(&self.server_nonce)?;
        encoder.push_u64(self.server_epoch);
        encoder.push_u64(u64::from(self.limits.max_frame_bytes));
        encoder.push_u64(u64::from(self.limits.max_pending_requests));
        encoder.push_u64(u64::from(self.limits.max_concurrent_clients));
        encoder.push_bytes(&key_id.0)?;
        Ok(encoder.finish())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EnrolledClientKey {
    pub key_id: ClientKeyId,
    pub verifying_key: VerifyingKey,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LifecycleError {
    Core(CoreError),
    InvalidPayloadLength {
        kind: FrameKind,
        expected: usize,
        actual: usize,
    },
    InvalidShutdownReason {
        code: u8,
    },
    NotLifecycleFrame {
        kind: FrameKind,
    },
    InvalidPhase {
        expected: LifecyclePhase,
        actual: LifecyclePhase,
    },
    UnsupportedProtocolVersion {
        found: u16,
    },
    TranscriptProtocolMismatch,
    InvalidNonce,
    InvalidServerEpoch,
    InvalidConnectionId,
    InvalidKeyId,
    ClientNonceMismatch,
    KeyIdMismatch,
    AuthenticationFailed,
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core(error) => write!(f, "IPC lifecycle encoding error: {error}"),
            Self::InvalidPayloadLength {
                kind,
                expected,
                actual,
            } => write!(
                f,
                "IPC lifecycle payload {kind:?} has {actual} bytes; expected {expected}"
            ),
            Self::InvalidShutdownReason { code } => {
                write!(f, "invalid IPC shutdown reason code {code}")
            }
            Self::NotLifecycleFrame { kind } => write!(f, "IPC frame {kind:?} is not lifecycle"),
            Self::InvalidPhase { expected, actual } => write!(
                f,
                "IPC lifecycle frame is out of order: expected {expected:?}, actual {actual:?}"
            ),
            Self::UnsupportedProtocolVersion { found } => write!(
                f,
                "unsupported IPC lifecycle protocol version {found}; expected {PROTOCOL_VERSION}"
            ),
            Self::TranscriptProtocolMismatch => {
                f.write_str("IPC hello/challenge protocol versions differ")
            }
            Self::InvalidNonce => f.write_str("IPC authentication nonce must not be all zero"),
            Self::InvalidServerEpoch => {
                f.write_str("IPC server epoch must be non-zero and fresh per daemon epoch")
            }
            Self::InvalidConnectionId => f.write_str("IPC connection id must be non-zero"),
            Self::InvalidKeyId => f.write_str("IPC client key id must not be all zero"),
            Self::ClientNonceMismatch => {
                f.write_str("IPC authenticate client nonce does not match hello")
            }
            Self::KeyIdMismatch => f.write_str("IPC client key id does not match enrolled key"),
            Self::AuthenticationFailed => {
                f.write_str("IPC transcript signature verification failed")
            }
        }
    }
}

impl Error for LifecycleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CoreError> for LifecycleError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

pub struct ServerLifecycle {
    phase: LifecyclePhase,
    server_epoch: u64,
    server_nonce: ServerNonce,
    limits: ResourceLimits,
    connection_id: ConnectionId,
    transcript: Option<AuthTranscript>,
    authenticated_client: Option<ClientId>,
}

impl ServerLifecycle {
    pub fn new(
        server_epoch: u64,
        server_nonce: ServerNonce,
        limits: ResourceLimits,
        connection_id: ConnectionId,
    ) -> Result<Self, LifecycleError> {
        validate_server_epoch(server_epoch)?;
        validate_nonce(server_nonce)?;
        if connection_id.0 == 0 {
            return Err(LifecycleError::InvalidConnectionId);
        }
        Ok(Self {
            phase: LifecyclePhase::AwaitHello,
            server_epoch,
            server_nonce,
            limits,
            connection_id,
            transcript: None,
            authenticated_client: None,
        })
    }

    pub const fn phase(&self) -> LifecyclePhase {
        self.phase
    }

    pub const fn authenticated_client(&self) -> Option<ClientId> {
        self.authenticated_client
    }

    pub fn authenticated_local_client(&self) -> Option<AuthenticatedLocalClient> {
        if self.phase != LifecyclePhase::Ready {
            return None;
        }
        let client_id = self.authenticated_client?;
        Some(AuthenticatedLocalClient {
            client_id,
            connection_id: self.connection_id,
            server_epoch: self.server_epoch,
            limits: self.limits,
        })
    }

    pub fn receive_hello(&mut self, hello: Hello) -> Result<Challenge, LifecycleError> {
        if self.phase != LifecyclePhase::AwaitHello {
            return self.close_with(LifecycleError::InvalidPhase {
                expected: LifecyclePhase::AwaitHello,
                actual: self.phase,
            });
        }
        if let Err(error) = validate_protocol_version(hello.protocol_version) {
            return self.close_with(error);
        }
        if let Err(error) = validate_nonce(hello.client_nonce) {
            return self.close_with(error);
        }
        let challenge = Challenge {
            protocol_version: PROTOCOL_VERSION,
            server_epoch: self.server_epoch,
            server_nonce: self.server_nonce,
            limits: self.limits,
        };
        let transcript = match AuthTranscript::from_messages(hello, challenge) {
            Ok(transcript) => transcript,
            Err(error) => return self.close_with(error),
        };
        self.transcript = Some(transcript);
        self.phase = LifecyclePhase::ChallengeSent;
        Ok(challenge)
    }

    pub fn authenticate(
        &mut self,
        authenticate: Authenticate,
        enrolled_key: &EnrolledClientKey,
    ) -> Result<Ready, LifecycleError> {
        if self.phase != LifecyclePhase::ChallengeSent {
            return self.close_with(LifecycleError::InvalidPhase {
                expected: LifecyclePhase::ChallengeSent,
                actual: self.phase,
            });
        }
        let transcript = self
            .transcript
            .expect("challenge-sent lifecycle always carries transcript");
        if authenticate.client_nonce != transcript.client_nonce {
            return self.close_with(LifecycleError::ClientNonceMismatch);
        }
        if authenticate.key_id != enrolled_key.key_id {
            return self.close_with(LifecycleError::KeyIdMismatch);
        }
        let message = match transcript.canonical_bytes(authenticate.key_id) {
            Ok(message) => message,
            Err(error) => return self.close_with(error),
        };
        let signature = Signature::from_bytes(&authenticate.signature);
        if enrolled_key
            .verifying_key
            .verify_strict(&message, &signature)
            .is_err()
        {
            return self.close_with(LifecycleError::AuthenticationFailed);
        }

        self.phase = LifecyclePhase::Ready;
        self.authenticated_client = Some(transcript.client_id);
        Ok(Ready {
            connection_id: self.connection_id,
            server_epoch: self.server_epoch,
            limits: self.limits,
        })
    }

    pub fn receive_shutdown(&mut self, _reason: ShutdownReason) -> Result<(), LifecycleError> {
        if self.phase == LifecyclePhase::Closed {
            return Err(LifecycleError::InvalidPhase {
                expected: LifecyclePhase::Ready,
                actual: LifecyclePhase::Closed,
            });
        }
        self.phase = LifecyclePhase::Closed;
        Ok(())
    }

    fn close_with<T>(&mut self, error: LifecycleError) -> Result<T, LifecycleError> {
        self.phase = LifecyclePhase::Closed;
        Err(error)
    }
}

fn validate_protocol_version(protocol_version: u16) -> Result<(), LifecycleError> {
    if protocol_version == PROTOCOL_VERSION {
        Ok(())
    } else {
        Err(LifecycleError::UnsupportedProtocolVersion {
            found: protocol_version,
        })
    }
}

fn validate_nonce(nonce: [u8; NONCE_LEN]) -> Result<(), LifecycleError> {
    if nonce.iter().all(|byte| *byte == 0) {
        Err(LifecycleError::InvalidNonce)
    } else {
        Ok(())
    }
}

fn validate_server_epoch(server_epoch: u64) -> Result<(), LifecycleError> {
    if server_epoch == 0 {
        Err(LifecycleError::InvalidServerEpoch)
    } else {
        Ok(())
    }
}

fn validate_key_id(key_id: ClientKeyId) -> Result<(), LifecycleError> {
    if key_id.0.iter().all(|byte| *byte == 0) {
        Err(LifecycleError::InvalidKeyId)
    } else {
        Ok(())
    }
}

fn encode_hello(message: Hello) -> Vec<u8> {
    let mut payload = Vec::with_capacity(HELLO_PAYLOAD_LEN);
    payload.extend_from_slice(&message.protocol_version.to_be_bytes());
    payload.extend_from_slice(&message.client_id.0.to_be_bytes());
    payload.extend_from_slice(&message.client_nonce);
    payload
}

fn decode_hello(payload: &[u8]) -> Result<Hello, LifecycleError> {
    expect_payload_len(FrameKind::Hello, payload, HELLO_PAYLOAD_LEN)?;
    Ok(Hello {
        protocol_version: read_u16(&payload[0..2]),
        client_id: ClientId(read_u128(&payload[2..18])),
        client_nonce: read_array(&payload[18..50]),
    })
}

fn encode_challenge(message: Challenge) -> Vec<u8> {
    let mut payload = Vec::with_capacity(CHALLENGE_PAYLOAD_LEN);
    payload.extend_from_slice(&message.protocol_version.to_be_bytes());
    payload.extend_from_slice(&message.server_epoch.to_be_bytes());
    payload.extend_from_slice(&message.server_nonce);
    payload.extend_from_slice(&message.limits.max_frame_bytes.to_be_bytes());
    payload.extend_from_slice(&message.limits.max_pending_requests.to_be_bytes());
    payload.extend_from_slice(&message.limits.max_concurrent_clients.to_be_bytes());
    payload
}

fn decode_challenge(payload: &[u8]) -> Result<Challenge, LifecycleError> {
    expect_payload_len(FrameKind::Challenge, payload, CHALLENGE_PAYLOAD_LEN)?;
    Ok(Challenge {
        protocol_version: read_u16(&payload[0..2]),
        server_epoch: read_u64(&payload[2..10]),
        server_nonce: read_array(&payload[10..42]),
        limits: ResourceLimits {
            max_frame_bytes: read_u32(&payload[42..46]),
            max_pending_requests: read_u32(&payload[46..50]),
            max_concurrent_clients: read_u16(&payload[50..52]),
        },
    })
}

fn encode_authenticate(message: Authenticate) -> Vec<u8> {
    let mut payload = Vec::with_capacity(AUTHENTICATE_PAYLOAD_LEN);
    payload.extend_from_slice(&message.key_id.0);
    payload.extend_from_slice(&message.client_nonce);
    payload.extend_from_slice(&message.signature);
    payload
}

fn decode_authenticate(payload: &[u8]) -> Result<Authenticate, LifecycleError> {
    expect_payload_len(FrameKind::Authenticate, payload, AUTHENTICATE_PAYLOAD_LEN)?;
    Ok(Authenticate {
        key_id: ClientKeyId(read_array(&payload[0..32])),
        client_nonce: read_array(&payload[32..64]),
        signature: read_array(&payload[64..128]),
    })
}

fn encode_ready(message: Ready) -> Vec<u8> {
    let mut payload = Vec::with_capacity(READY_PAYLOAD_LEN);
    payload.extend_from_slice(&message.connection_id.0.to_be_bytes());
    payload.extend_from_slice(&message.server_epoch.to_be_bytes());
    payload.extend_from_slice(&message.limits.max_frame_bytes.to_be_bytes());
    payload.extend_from_slice(&message.limits.max_pending_requests.to_be_bytes());
    payload.extend_from_slice(&message.limits.max_concurrent_clients.to_be_bytes());
    payload
}

fn decode_ready(payload: &[u8]) -> Result<Ready, LifecycleError> {
    expect_payload_len(FrameKind::Ready, payload, READY_PAYLOAD_LEN)?;
    Ok(Ready {
        connection_id: ConnectionId(read_u128(&payload[0..16])),
        server_epoch: read_u64(&payload[16..24]),
        limits: ResourceLimits {
            max_frame_bytes: read_u32(&payload[24..28]),
            max_pending_requests: read_u32(&payload[28..32]),
            max_concurrent_clients: read_u16(&payload[32..34]),
        },
    })
}

fn decode_shutdown(payload: &[u8]) -> Result<ShutdownReason, LifecycleError> {
    expect_payload_len(FrameKind::Shutdown, payload, SHUTDOWN_PAYLOAD_LEN)?;
    ShutdownReason::from_code(payload[0])
        .ok_or(LifecycleError::InvalidShutdownReason { code: payload[0] })
}

fn expect_payload_len(
    kind: FrameKind,
    payload: &[u8],
    expected: usize,
) -> Result<(), LifecycleError> {
    if payload.len() == expected {
        Ok(())
    } else {
        Err(LifecycleError::InvalidPayloadLength {
            kind,
            expected,
            actual: payload.len(),
        })
    }
}

fn read_array<const N: usize>(bytes: &[u8]) -> [u8; N] {
    bytes.try_into().expect("validated fixed lifecycle range")
}

fn read_u16(bytes: &[u8]) -> u16 {
    u16::from_be_bytes(read_array(bytes))
}

fn read_u32(bytes: &[u8]) -> u32 {
    u32::from_be_bytes(read_array(bytes))
}

fn read_u64(bytes: &[u8]) -> u64 {
    u64::from_be_bytes(read_array(bytes))
}

fn read_u128(bytes: &[u8]) -> u128 {
    u128::from_be_bytes(read_array(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn signing_key() -> SigningKey {
        SigningKey::from_bytes(&[7_u8; 32])
    }

    fn key_id() -> ClientKeyId {
        ClientKeyId([9_u8; CLIENT_KEY_ID_LEN])
    }

    fn hello() -> Hello {
        Hello {
            protocol_version: PROTOCOL_VERSION,
            client_id: ClientId(42),
            client_nonce: [1_u8; NONCE_LEN],
        }
    }

    fn fresh_server() -> ServerLifecycle {
        ServerLifecycle::new(
            7,
            [2_u8; NONCE_LEN],
            ResourceLimits::default(),
            ConnectionId(99),
        )
        .unwrap()
    }

    fn authenticate_message(
        signing_key: &SigningKey,
        hello: Hello,
        challenge: Challenge,
    ) -> Authenticate {
        let transcript = AuthTranscript::from_messages(hello, challenge).unwrap();
        let signature = signing_key
            .sign(&transcript.canonical_bytes(key_id()).unwrap())
            .to_bytes();
        Authenticate {
            key_id: key_id(),
            client_nonce: hello.client_nonce,
            signature,
        }
    }

    fn enrolled(signing_key: &SigningKey) -> EnrolledClientKey {
        EnrolledClientKey {
            key_id: key_id(),
            verifying_key: signing_key.verifying_key(),
        }
    }

    #[test]
    fn lifecycle_payloads_round_trip_exactly() {
        let messages = [
            LifecycleMessage::Hello(hello()),
            LifecycleMessage::Challenge(Challenge {
                protocol_version: PROTOCOL_VERSION,
                server_epoch: 7,
                server_nonce: [2_u8; NONCE_LEN],
                limits: ResourceLimits::default(),
            }),
            LifecycleMessage::Authenticate(Authenticate {
                key_id: key_id(),
                client_nonce: [1_u8; NONCE_LEN],
                signature: [3_u8; SIGNATURE_LEN],
            }),
            LifecycleMessage::Ready(Ready {
                connection_id: ConnectionId(99),
                server_epoch: 7,
                limits: ResourceLimits::default(),
            }),
            LifecycleMessage::Shutdown(ShutdownReason::Normal),
        ];

        for message in messages {
            let kind = message.frame_kind();
            let payload = message.encode_payload();
            assert_eq!(LifecycleMessage::decode(kind, &payload).unwrap(), message);
        }
        assert!(matches!(
            LifecycleMessage::decode(FrameKind::Request, b""),
            Err(LifecycleError::NotLifecycleFrame {
                kind: FrameKind::Request
            })
        ));
    }

    #[test]
    fn correct_transcript_signature_reaches_ready_and_shutdown() {
        let signing_key = signing_key();
        let hello = hello();
        let mut server = fresh_server();
        let challenge = server.receive_hello(hello).unwrap();
        let authenticate = authenticate_message(&signing_key, hello, challenge);
        let ready = server
            .authenticate(authenticate, &enrolled(&signing_key))
            .unwrap();

        assert_eq!(server.phase(), LifecyclePhase::Ready);
        assert_eq!(server.authenticated_client(), Some(hello.client_id));
        assert_eq!(ready.connection_id, ConnectionId(99));
        assert_eq!(ready.server_epoch, challenge.server_epoch);

        server.receive_shutdown(ShutdownReason::Normal).unwrap();
        assert_eq!(server.phase(), LifecyclePhase::Closed);
    }

    #[test]
    fn wrong_signature_fails_closed() {
        let correct_key = signing_key();
        let wrong_key = SigningKey::from_bytes(&[8_u8; 32]);
        let hello = hello();
        let mut server = fresh_server();
        let challenge = server.receive_hello(hello).unwrap();
        let authenticate = authenticate_message(&wrong_key, hello, challenge);

        assert_eq!(
            server.authenticate(authenticate, &enrolled(&correct_key)),
            Err(LifecycleError::AuthenticationFailed)
        );
        assert_eq!(server.phase(), LifecyclePhase::Closed);
    }

    #[test]
    fn stale_epoch_signature_fails_closed() {
        let signing_key = signing_key();
        let hello = hello();
        let mut server = fresh_server();
        let challenge = server.receive_hello(hello).unwrap();
        let stale_challenge = Challenge {
            server_epoch: challenge.server_epoch - 1,
            ..challenge
        };
        let authenticate = authenticate_message(&signing_key, hello, stale_challenge);

        assert_eq!(
            server.authenticate(authenticate, &enrolled(&signing_key)),
            Err(LifecycleError::AuthenticationFailed)
        );
        assert_eq!(server.phase(), LifecyclePhase::Closed);
    }

    #[test]
    fn changed_client_nonce_and_key_id_fail_closed() {
        let signing_key = signing_key();
        let hello = hello();
        let mut server = fresh_server();
        let challenge = server.receive_hello(hello).unwrap();
        let mut authenticate = authenticate_message(&signing_key, hello, challenge);
        authenticate.client_nonce = [4_u8; NONCE_LEN];
        assert_eq!(
            server.authenticate(authenticate, &enrolled(&signing_key)),
            Err(LifecycleError::ClientNonceMismatch)
        );

        let mut server = fresh_server();
        let challenge = server.receive_hello(hello).unwrap();
        let mut authenticate = authenticate_message(&signing_key, hello, challenge);
        authenticate.key_id = ClientKeyId([5_u8; CLIENT_KEY_ID_LEN]);
        assert_eq!(
            server.authenticate(authenticate, &enrolled(&signing_key)),
            Err(LifecycleError::KeyIdMismatch)
        );
        assert_eq!(server.phase(), LifecyclePhase::Closed);
    }

    #[test]
    fn repeated_or_out_of_order_lifecycle_fails_closed() {
        let mut server = fresh_server();
        let hello = hello();
        server.receive_hello(hello).unwrap();
        assert!(matches!(
            server.receive_hello(hello),
            Err(LifecycleError::InvalidPhase { .. })
        ));
        assert_eq!(server.phase(), LifecyclePhase::Closed);

        let signing_key = signing_key();
        let mut server = fresh_server();
        let authenticate = Authenticate {
            key_id: key_id(),
            client_nonce: hello.client_nonce,
            signature: [0_u8; SIGNATURE_LEN],
        };
        assert!(matches!(
            server.authenticate(authenticate, &enrolled(&signing_key)),
            Err(LifecycleError::InvalidPhase { .. })
        ));
        assert_eq!(server.phase(), LifecyclePhase::Closed);
    }

    #[test]
    fn transcript_binds_limits_nonces_epoch_and_key_id() {
        let hello = hello();
        let challenge = Challenge {
            protocol_version: PROTOCOL_VERSION,
            server_epoch: 7,
            server_nonce: [2_u8; NONCE_LEN],
            limits: ResourceLimits::default(),
        };
        let base = AuthTranscript::from_messages(hello, challenge)
            .unwrap()
            .canonical_bytes(key_id())
            .unwrap();

        let changed_nonce = AuthTranscript::from_messages(
            Hello {
                client_nonce: [6_u8; NONCE_LEN],
                ..hello
            },
            challenge,
        )
        .unwrap()
        .canonical_bytes(key_id())
        .unwrap();
        assert_ne!(base, changed_nonce);

        let changed_epoch = AuthTranscript::from_messages(
            hello,
            Challenge {
                server_epoch: 8,
                ..challenge
            },
        )
        .unwrap()
        .canonical_bytes(key_id())
        .unwrap();
        assert_ne!(base, changed_epoch);

        let changed_limits = AuthTranscript::from_messages(
            hello,
            Challenge {
                limits: ResourceLimits {
                    max_pending_requests: 64,
                    ..challenge.limits
                },
                ..challenge
            },
        )
        .unwrap()
        .canonical_bytes(key_id())
        .unwrap();
        assert_ne!(base, changed_limits);

        let changed_key = AuthTranscript::from_messages(hello, challenge)
            .unwrap()
            .canonical_bytes(ClientKeyId([10_u8; CLIENT_KEY_ID_LEN]))
            .unwrap();
        assert_ne!(base, changed_key);
    }

    #[test]
    fn invalid_payload_lengths_and_codes_fail_closed() {
        assert!(matches!(
            LifecycleMessage::decode(FrameKind::Hello, &[0_u8; 1]),
            Err(LifecycleError::InvalidPayloadLength { .. })
        ));
        assert_eq!(
            LifecycleMessage::decode(FrameKind::Shutdown, &[255]),
            Err(LifecycleError::InvalidShutdownReason { code: 255 })
        );
    }
}
