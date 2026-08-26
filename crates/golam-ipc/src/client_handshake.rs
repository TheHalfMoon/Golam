#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::io::{Read, Write};

use ed25519_dalek::{Signer, SigningKey};
use golam_core::{ClientId, PROTOCOL_VERSION, ResourceLimits};

use crate::lifecycle::{
    AuthTranscript, Authenticate, Challenge, ClientKeyId, Hello, LifecycleError, LifecycleMessage,
    Ready,
};
use crate::wire::{WireError, read_frame, write_frame};
use crate::{FrameHeader, FrameKind};

#[derive(Debug)]
pub enum ClientHandshakeError {
    Wire(WireError),
    Lifecycle(LifecycleError),
    Random(getrandom::Error),
    InvalidClientId,
    InvalidClientNonce,
    UnexpectedFrame {
        expected: FrameKind,
        actual: FrameKind,
    },
    ServerEpochMismatch {
        challenge: u64,
        ready: u64,
    },
    LimitsMismatch,
}

impl fmt::Display for ClientHandshakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wire(error) => write!(f, "IPC client handshake wire error: {error}"),
            Self::Lifecycle(error) => write!(f, "IPC client handshake lifecycle error: {error}"),
            Self::Random(error) => write!(f, "IPC client nonce generation failed: {error}"),
            Self::InvalidClientId => f.write_str("IPC client id must be non-zero"),
            Self::InvalidClientNonce => f.write_str("IPC client nonce must not be all zero"),
            Self::UnexpectedFrame { expected, actual } => write!(
                f,
                "IPC client handshake expected {expected:?} frame, received {actual:?}"
            ),
            Self::ServerEpochMismatch { challenge, ready } => write!(
                f,
                "IPC server epoch changed during handshake: challenge={challenge}, ready={ready}"
            ),
            Self::LimitsMismatch => {
                f.write_str("IPC server resource limits changed during handshake")
            }
        }
    }
}

impl Error for ClientHandshakeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Wire(error) => Some(error),
            Self::Lifecycle(error) => Some(error),
            Self::Random(error) => Some(error),
            Self::InvalidClientId
            | Self::InvalidClientNonce
            | Self::UnexpectedFrame { .. }
            | Self::ServerEpochMismatch { .. }
            | Self::LimitsMismatch => None,
        }
    }
}

impl From<WireError> for ClientHandshakeError {
    fn from(value: WireError) -> Self {
        Self::Wire(value)
    }
}

impl From<LifecycleError> for ClientHandshakeError {
    fn from(value: LifecycleError) -> Self {
        Self::Lifecycle(value)
    }
}

impl From<getrandom::Error> for ClientHandshakeError {
    fn from(value: getrandom::Error) -> Self {
        Self::Random(value)
    }
}

pub fn authenticate_client<S: Read + Write>(
    stream: &mut S,
    client_id: ClientId,
    key_id: ClientKeyId,
    signing_key: &SigningKey,
    local_limits: ResourceLimits,
) -> Result<Ready, ClientHandshakeError> {
    if client_id.0 == 0 {
        return Err(ClientHandshakeError::InvalidClientId);
    }

    let mut client_nonce = [0_u8; 32];
    getrandom::fill(&mut client_nonce)?;
    if client_nonce.iter().all(|byte| *byte == 0) {
        return Err(ClientHandshakeError::InvalidClientNonce);
    }

    let hello = Hello {
        protocol_version: PROTOCOL_VERSION,
        client_id,
        client_nonce,
    };
    write_lifecycle(stream, LifecycleMessage::Hello(hello), local_limits)?;

    let challenge_frame = read_frame(stream, local_limits)?;
    if challenge_frame.header.kind != FrameKind::Challenge {
        return Err(ClientHandshakeError::UnexpectedFrame {
            expected: FrameKind::Challenge,
            actual: challenge_frame.header.kind,
        });
    }
    let challenge = match LifecycleMessage::decode(
        challenge_frame.header.kind,
        &challenge_frame.payload,
    )? {
        LifecycleMessage::Challenge(challenge) => challenge,
        _ => unreachable!("challenge frame decodes to challenge lifecycle message"),
    };

    let transcript = AuthTranscript::from_messages(hello, challenge)?;
    let signature = signing_key
        .sign(&transcript.canonical_bytes(key_id)?)
        .to_bytes();
    let authenticate = Authenticate {
        key_id,
        client_nonce,
        signature,
    };
    write_lifecycle(
        stream,
        LifecycleMessage::Authenticate(authenticate),
        challenge.limits,
    )?;

    let ready_frame = read_frame(stream, challenge.limits)?;
    if ready_frame.header.kind != FrameKind::Ready {
        return Err(ClientHandshakeError::UnexpectedFrame {
            expected: FrameKind::Ready,
            actual: ready_frame.header.kind,
        });
    }
    let ready = match LifecycleMessage::decode(ready_frame.header.kind, &ready_frame.payload)? {
        LifecycleMessage::Ready(ready) => ready,
        _ => unreachable!("ready frame decodes to ready lifecycle message"),
    };
    if ready.server_epoch != challenge.server_epoch {
        return Err(ClientHandshakeError::ServerEpochMismatch {
            challenge: challenge.server_epoch,
            ready: ready.server_epoch,
        });
    }
    if ready.limits != challenge.limits {
        return Err(ClientHandshakeError::LimitsMismatch);
    }
    Ok(ready)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::key_id_for_public_key;
    use crate::lifecycle::{ConnectionId, EnrolledClientKey};
    use crate::wire::read_frame;
    use crate::{FrameKind, encode_frame};
    use ed25519_dalek::VerifyingKey;
    use std::io::{self, Cursor};

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

    fn server_script(challenge: Challenge, ready: Ready, local_limits: ResourceLimits) -> Vec<u8> {
        let challenge_payload = LifecycleMessage::Challenge(challenge).encode_payload();
        let mut bytes = encode_frame(FrameKind::Challenge, None, &challenge_payload, local_limits)
            .unwrap();
        let ready_payload = LifecycleMessage::Ready(ready).encode_payload();
        bytes.extend_from_slice(
            &encode_frame(FrameKind::Ready, None, &ready_payload, challenge.limits).unwrap(),
        );
        bytes
    }

    #[test]
    fn client_handshake_emits_signed_hello_and_authenticate_and_accepts_ready() {
        let local_limits = ResourceLimits::default();
        let challenge = Challenge {
            protocol_version: PROTOCOL_VERSION,
            server_epoch: 17,
            server_nonce: [9; 32],
            limits: local_limits,
        };
        let ready = Ready {
            connection_id: ConnectionId(29),
            server_epoch: challenge.server_epoch,
            limits: challenge.limits,
        };
        let signing_key = SigningKey::from_bytes(&[7; 32]);
        let key_id = key_id_for_public_key(signing_key.verifying_key().to_bytes());
        let mut io = ScriptedIo::new(server_script(challenge, ready, local_limits));

        let observed = authenticate_client(
            &mut io,
            ClientId(41),
            key_id,
            &signing_key,
            local_limits,
        )
        .unwrap();
        assert_eq!(observed, ready);

        let mut output = Cursor::new(io.output);
        let hello_frame = read_frame(&mut output, local_limits).unwrap();
        assert_eq!(hello_frame.header.kind, FrameKind::Hello);
        let hello = match LifecycleMessage::decode(hello_frame.header.kind, &hello_frame.payload)
            .unwrap()
        {
            LifecycleMessage::Hello(hello) => hello,
            _ => panic!("expected hello"),
        };
        assert_eq!(hello.client_id, ClientId(41));
        assert!(hello.client_nonce.iter().any(|byte| *byte != 0));

        let authenticate_frame = read_frame(&mut output, challenge.limits).unwrap();
        assert_eq!(authenticate_frame.header.kind, FrameKind::Authenticate);
        let authenticate = match LifecycleMessage::decode(
            authenticate_frame.header.kind,
            &authenticate_frame.payload,
        )
        .unwrap()
        {
            LifecycleMessage::Authenticate(authenticate) => authenticate,
            _ => panic!("expected authenticate"),
        };
        let transcript = AuthTranscript::from_messages(hello, challenge).unwrap();
        let enrolled = EnrolledClientKey {
            key_id,
            verifying_key: VerifyingKey::from_bytes(&signing_key.verifying_key().to_bytes()).unwrap(),
        };
        let mut server = crate::lifecycle::ServerLifecycle::new(
            challenge.server_epoch,
            challenge.server_nonce,
            challenge.limits,
            ready.connection_id,
        )
        .unwrap();
        server.receive_hello(hello).unwrap();
        let server_ready = server.authenticate(authenticate, &enrolled).unwrap();
        assert_eq!(server_ready, ready);
        assert_eq!(
            transcript.canonical_bytes(key_id).unwrap(),
            AuthTranscript::from_messages(hello, challenge)
                .unwrap()
                .canonical_bytes(key_id)
                .unwrap()
        );
    }

    #[test]
    fn server_epoch_or_limit_changes_fail_closed() {
        let local_limits = ResourceLimits::default();
        let challenge = Challenge {
            protocol_version: PROTOCOL_VERSION,
            server_epoch: 5,
            server_nonce: [3; 32],
            limits: local_limits,
        };
        let signing_key = SigningKey::from_bytes(&[4; 32]);
        let key_id = key_id_for_public_key(signing_key.verifying_key().to_bytes());

        let wrong_epoch = Ready {
            connection_id: ConnectionId(6),
            server_epoch: 7,
            limits: challenge.limits,
        };
        let mut epoch_io = ScriptedIo::new(server_script(challenge, wrong_epoch, local_limits));
        assert!(matches!(
            authenticate_client(
                &mut epoch_io,
                ClientId(8),
                key_id,
                &signing_key,
                local_limits
            ),
            Err(ClientHandshakeError::ServerEpochMismatch { .. })
        ));

        let changed_limits = ResourceLimits {
            max_pending_requests: challenge.limits.max_pending_requests.saturating_add(1),
            ..challenge.limits
        };
        let wrong_limits = Ready {
            connection_id: ConnectionId(9),
            server_epoch: challenge.server_epoch,
            limits: changed_limits,
        };
        let mut limits_io = ScriptedIo::new(server_script(challenge, wrong_limits, local_limits));
        assert!(matches!(
            authenticate_client(
                &mut limits_io,
                ClientId(10),
                key_id,
                &signing_key,
                local_limits
            ),
            Err(ClientHandshakeError::LimitsMismatch)
        ));
    }
}
