#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::str;

use golam_core::{CheckpointId, ClientId, EffectId, EventId, GoalId, GoalVersionId, SessionId};

use crate::request::{MethodId, RequestMessage};

pub const METHOD_SESSIONS_LIST: MethodId = MethodId(100);
pub const METHOD_SESSION_OPEN: MethodId = MethodId(101);
pub const METHOD_SESSION_CREATE: MethodId = MethodId(102);
pub const METHOD_SESSION_FORK: MethodId = MethodId(103);
pub const METHOD_GOAL_APPEND: MethodId = MethodId(104);
pub const METHOD_CHECKPOINT_CREATE: MethodId = MethodId(105);
pub const METHOD_CHECKPOINT_VERIFY: MethodId = MethodId(106);
pub const METHOD_REPLAY: MethodId = MethodId(107);
pub const METHOD_EFFECT_SIMULATE: MethodId = MethodId(108);
pub const METHOD_EFFECT_RECONCILE: MethodId = MethodId(109);
pub const METHOD_DOCTOR: MethodId = MethodId(110);
pub const METHOD_CLIENT_ENROLL: MethodId = MethodId(111);

pub const MAX_COMMAND_BODY_BYTES: usize = 256 * 1024;
pub const MAX_TEXT_BYTES: usize = 64 * 1024;
pub const MAX_EVENT_PAYLOAD_BYTES: usize = 128 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyntheticSemantics {
    ReadOnly,
    IdempotentAtLeastOnce,
    AtMostOnce,
    Compensatable,
    Irreversible,
}

impl SyntheticSemantics {
    const fn code(self) -> u8 {
        match self {
            Self::ReadOnly => 1,
            Self::IdempotentAtLeastOnce => 2,
            Self::AtMostOnce => 3,
            Self::Compensatable => 4,
            Self::Irreversible => 5,
        }
    }

    const fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::ReadOnly),
            2 => Some(Self::IdempotentAtLeastOnce),
            3 => Some(Self::AtMostOnce),
            4 => Some(Self::Compensatable),
            5 => Some(Self::Irreversible),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    ClientEnroll {
        client_id: ClientId,
    },
    SessionsList,
    SessionOpen {
        session_id: SessionId,
    },
    SessionCreate {
        session_id: SessionId,
        event_id: EventId,
        recorded_at: String,
        payload: Vec<u8>,
    },
    SessionFork {
        child_session_id: SessionId,
        event_id: EventId,
        parent_session_id: SessionId,
        through_session_seq: u64,
        recorded_at: String,
    },
    GoalAppend {
        goal_version_id: GoalVersionId,
        goal_id: GoalId,
        event_id: EventId,
        session_id: SessionId,
        expected_session_seq: u64,
        expected_goal_version: u64,
        recorded_at: String,
        goal: String,
    },
    CheckpointCreate {
        checkpoint_id: CheckpointId,
        created_event_id: EventId,
        session_id: SessionId,
        through_session_seq: u64,
        recorded_at: String,
    },
    CheckpointVerify {
        checkpoint_id: CheckpointId,
        session_id: SessionId,
        through_session_seq: u64,
    },
    Replay {
        session_id: SessionId,
        through_session_seq: u64,
    },
    EffectSimulate {
        effect_id: EffectId,
        session_id: SessionId,
        semantics: SyntheticSemantics,
    },
    EffectReconcile {
        effect_id: EffectId,
    },
    Doctor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandCodecError {
    UnknownMethod(u16),
    BodyTooLarge { actual: usize, maximum: usize },
    FieldTooLarge { actual: usize, maximum: usize },
    Truncated,
    TrailingBytes { actual: usize },
    InvalidUtf8,
    InvalidSyntheticSemantics(u8),
    LengthOverflow,
}

impl fmt::Display for CommandCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownMethod(method) => write!(f, "unknown Golam command method {method}"),
            Self::BodyTooLarge { actual, maximum } => {
                write!(
                    f,
                    "Golam command body is {actual} bytes; maximum is {maximum}"
                )
            }
            Self::FieldTooLarge { actual, maximum } => {
                write!(
                    f,
                    "Golam command field is {actual} bytes; maximum is {maximum}"
                )
            }
            Self::Truncated => f.write_str("Golam command body is truncated"),
            Self::TrailingBytes { actual } => {
                write!(f, "Golam command body has {actual} trailing bytes")
            }
            Self::InvalidUtf8 => f.write_str("Golam command text is not valid UTF-8"),
            Self::InvalidSyntheticSemantics(code) => {
                write!(f, "unknown synthetic effect semantics code {code}")
            }
            Self::LengthOverflow => f.write_str("Golam command field length exceeds u32"),
        }
    }
}

impl Error for CommandCodecError {}

pub fn encode_command(command: &Command) -> Result<RequestMessage, CommandCodecError> {
    let (method, body) = match command {
        Command::ClientEnroll { client_id } => {
            let mut body = Writer::new();
            body.u128(client_id.0);
            (METHOD_CLIENT_ENROLL, body.finish())
        }
        Command::SessionsList => (METHOD_SESSIONS_LIST, Vec::new()),
        Command::SessionOpen { session_id } => {
            let mut body = Writer::new();
            body.u128(session_id.0);
            (METHOD_SESSION_OPEN, body.finish())
        }
        Command::SessionCreate {
            session_id,
            event_id,
            recorded_at,
            payload,
        } => {
            check_field(recorded_at.len(), MAX_TEXT_BYTES)?;
            check_field(payload.len(), MAX_EVENT_PAYLOAD_BYTES)?;
            let mut body = Writer::new();
            body.u128(session_id.0);
            body.u128(event_id.0);
            body.bytes(recorded_at.as_bytes())?;
            body.bytes(payload)?;
            (METHOD_SESSION_CREATE, body.finish())
        }
        Command::SessionFork {
            child_session_id,
            event_id,
            parent_session_id,
            through_session_seq,
            recorded_at,
        } => {
            check_field(recorded_at.len(), MAX_TEXT_BYTES)?;
            let mut body = Writer::new();
            body.u128(child_session_id.0);
            body.u128(event_id.0);
            body.u128(parent_session_id.0);
            body.u64(*through_session_seq);
            body.bytes(recorded_at.as_bytes())?;
            (METHOD_SESSION_FORK, body.finish())
        }
        Command::GoalAppend {
            goal_version_id,
            goal_id,
            event_id,
            session_id,
            expected_session_seq,
            expected_goal_version,
            recorded_at,
            goal,
        } => {
            check_field(recorded_at.len(), MAX_TEXT_BYTES)?;
            check_field(goal.len(), MAX_TEXT_BYTES)?;
            let mut body = Writer::new();
            body.u128(goal_version_id.0);
            body.u128(goal_id.0);
            body.u128(event_id.0);
            body.u128(session_id.0);
            body.u64(*expected_session_seq);
            body.u64(*expected_goal_version);
            body.bytes(recorded_at.as_bytes())?;
            body.bytes(goal.as_bytes())?;
            (METHOD_GOAL_APPEND, body.finish())
        }
        Command::CheckpointCreate {
            checkpoint_id,
            created_event_id,
            session_id,
            through_session_seq,
            recorded_at,
        } => {
            check_field(recorded_at.len(), MAX_TEXT_BYTES)?;
            let mut body = Writer::new();
            body.u128(checkpoint_id.0);
            body.u128(created_event_id.0);
            body.u128(session_id.0);
            body.u64(*through_session_seq);
            body.bytes(recorded_at.as_bytes())?;
            (METHOD_CHECKPOINT_CREATE, body.finish())
        }
        Command::CheckpointVerify {
            checkpoint_id,
            session_id,
            through_session_seq,
        } => {
            let mut body = Writer::new();
            body.u128(checkpoint_id.0);
            body.u128(session_id.0);
            body.u64(*through_session_seq);
            (METHOD_CHECKPOINT_VERIFY, body.finish())
        }
        Command::Replay {
            session_id,
            through_session_seq,
        } => {
            let mut body = Writer::new();
            body.u128(session_id.0);
            body.u64(*through_session_seq);
            (METHOD_REPLAY, body.finish())
        }
        Command::EffectSimulate {
            effect_id,
            session_id,
            semantics,
        } => {
            let mut body = Writer::new();
            body.u128(effect_id.0);
            body.u128(session_id.0);
            body.u8(semantics.code());
            (METHOD_EFFECT_SIMULATE, body.finish())
        }
        Command::EffectReconcile { effect_id } => {
            let mut body = Writer::new();
            body.u128(effect_id.0);
            (METHOD_EFFECT_RECONCILE, body.finish())
        }
        Command::Doctor => (METHOD_DOCTOR, Vec::new()),
    };
    check_body(body.len())?;
    Ok(RequestMessage { method, body })
}

pub fn decode_command(message: &RequestMessage) -> Result<Command, CommandCodecError> {
    check_body(message.body.len())?;
    let mut reader = Reader::new(&message.body);
    let command = match message.method {
        METHOD_CLIENT_ENROLL => Command::ClientEnroll {
            client_id: ClientId(reader.u128()?),
        },
        METHOD_SESSIONS_LIST => Command::SessionsList,
        METHOD_SESSION_OPEN => Command::SessionOpen {
            session_id: SessionId(reader.u128()?),
        },
        METHOD_SESSION_CREATE => Command::SessionCreate {
            session_id: SessionId(reader.u128()?),
            event_id: EventId(reader.u128()?),
            recorded_at: reader.text(MAX_TEXT_BYTES)?,
            payload: reader.bytes(MAX_EVENT_PAYLOAD_BYTES)?.to_vec(),
        },
        METHOD_SESSION_FORK => Command::SessionFork {
            child_session_id: SessionId(reader.u128()?),
            event_id: EventId(reader.u128()?),
            parent_session_id: SessionId(reader.u128()?),
            through_session_seq: reader.u64()?,
            recorded_at: reader.text(MAX_TEXT_BYTES)?,
        },
        METHOD_GOAL_APPEND => Command::GoalAppend {
            goal_version_id: GoalVersionId(reader.u128()?),
            goal_id: GoalId(reader.u128()?),
            event_id: EventId(reader.u128()?),
            session_id: SessionId(reader.u128()?),
            expected_session_seq: reader.u64()?,
            expected_goal_version: reader.u64()?,
            recorded_at: reader.text(MAX_TEXT_BYTES)?,
            goal: reader.text(MAX_TEXT_BYTES)?,
        },
        METHOD_CHECKPOINT_CREATE => Command::CheckpointCreate {
            checkpoint_id: CheckpointId(reader.u128()?),
            created_event_id: EventId(reader.u128()?),
            session_id: SessionId(reader.u128()?),
            through_session_seq: reader.u64()?,
            recorded_at: reader.text(MAX_TEXT_BYTES)?,
        },
        METHOD_CHECKPOINT_VERIFY => Command::CheckpointVerify {
            checkpoint_id: CheckpointId(reader.u128()?),
            session_id: SessionId(reader.u128()?),
            through_session_seq: reader.u64()?,
        },
        METHOD_REPLAY => Command::Replay {
            session_id: SessionId(reader.u128()?),
            through_session_seq: reader.u64()?,
        },
        METHOD_EFFECT_SIMULATE => {
            let effect_id = EffectId(reader.u128()?);
            let session_id = SessionId(reader.u128()?);
            let code = reader.u8()?;
            let semantics = SyntheticSemantics::from_code(code)
                .ok_or(CommandCodecError::InvalidSyntheticSemantics(code))?;
            Command::EffectSimulate {
                effect_id,
                session_id,
                semantics,
            }
        }
        METHOD_EFFECT_RECONCILE => Command::EffectReconcile {
            effect_id: EffectId(reader.u128()?),
        },
        METHOD_DOCTOR => Command::Doctor,
        other => return Err(CommandCodecError::UnknownMethod(other.0)),
    };
    reader.finish()?;
    Ok(command)
}

fn check_body(actual: usize) -> Result<(), CommandCodecError> {
    if actual > MAX_COMMAND_BODY_BYTES {
        Err(CommandCodecError::BodyTooLarge {
            actual,
            maximum: MAX_COMMAND_BODY_BYTES,
        })
    } else {
        Ok(())
    }
}

fn check_field(actual: usize, maximum: usize) -> Result<(), CommandCodecError> {
    if actual > maximum {
        Err(CommandCodecError::FieldTooLarge { actual, maximum })
    } else {
        Ok(())
    }
}

struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn u128(&mut self, value: u128) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn bytes(&mut self, value: &[u8]) -> Result<(), CommandCodecError> {
        let len = u32::try_from(value.len()).map_err(|_| CommandCodecError::LengthOverflow)?;
        self.bytes.extend_from_slice(&len.to_be_bytes());
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn u8(&mut self) -> Result<u8, CommandCodecError> {
        Ok(self.take(1)?[0])
    }

    fn u64(&mut self) -> Result<u64, CommandCodecError> {
        Ok(u64::from_be_bytes(
            self.take(8)?.try_into().expect("fixed u64 field length"),
        ))
    }

    fn u128(&mut self) -> Result<u128, CommandCodecError> {
        Ok(u128::from_be_bytes(
            self.take(16)?.try_into().expect("fixed u128 field length"),
        ))
    }

    fn bytes(&mut self, maximum: usize) -> Result<&'a [u8], CommandCodecError> {
        let len = u32::from_be_bytes(
            self.take(4)?
                .try_into()
                .expect("fixed command length field"),
        ) as usize;
        check_field(len, maximum)?;
        self.take(len)
    }

    fn text(&mut self, maximum: usize) -> Result<String, CommandCodecError> {
        let bytes = self.bytes(maximum)?;
        str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| CommandCodecError::InvalidUtf8)
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], CommandCodecError> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or(CommandCodecError::Truncated)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(CommandCodecError::Truncated)?;
        self.offset = end;
        Ok(value)
    }

    fn finish(self) -> Result<(), CommandCodecError> {
        let trailing = self.bytes.len().saturating_sub(self.offset);
        if trailing == 0 {
            Ok(())
        } else {
            Err(CommandCodecError::TrailingBytes { actual: trailing })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn samples() -> Vec<Command> {
        vec![
            Command::ClientEnroll {
                client_id: ClientId(1),
            },
            Command::SessionsList,
            Command::SessionOpen {
                session_id: SessionId(1),
            },
            Command::SessionCreate {
                session_id: SessionId(2),
                event_id: EventId(3),
                recorded_at: "2026-08-25T12:00:00Z".to_owned(),
                payload: b"hello".to_vec(),
            },
            Command::SessionFork {
                child_session_id: SessionId(4),
                event_id: EventId(5),
                parent_session_id: SessionId(2),
                through_session_seq: 1,
                recorded_at: "2026-08-25T12:01:00Z".to_owned(),
            },
            Command::GoalAppend {
                goal_version_id: GoalVersionId(6),
                goal_id: GoalId(7),
                event_id: EventId(8),
                session_id: SessionId(2),
                expected_session_seq: 1,
                expected_goal_version: 0,
                recorded_at: "2026-08-25T12:02:00Z".to_owned(),
                goal: "qualify cli".to_owned(),
            },
            Command::CheckpointCreate {
                checkpoint_id: CheckpointId(9),
                created_event_id: EventId(10),
                session_id: SessionId(2),
                through_session_seq: 2,
                recorded_at: "2026-08-25T12:03:00Z".to_owned(),
            },
            Command::CheckpointVerify {
                checkpoint_id: CheckpointId(9),
                session_id: SessionId(2),
                through_session_seq: 2,
            },
            Command::Replay {
                session_id: SessionId(2),
                through_session_seq: 2,
            },
            Command::EffectSimulate {
                effect_id: EffectId(11),
                session_id: SessionId(2),
                semantics: SyntheticSemantics::Irreversible,
            },
            Command::EffectReconcile {
                effect_id: EffectId(11),
            },
            Command::Doctor,
        ]
    }

    #[test]
    fn every_minimal_cli_command_round_trips() {
        for command in samples() {
            let message = encode_command(&command).unwrap();
            assert_eq!(decode_command(&message).unwrap(), command);
        }
    }

    #[test]
    fn unknown_truncated_trailing_and_invalid_semantics_fail_closed() {
        assert_eq!(
            decode_command(&RequestMessage {
                method: MethodId(999),
                body: Vec::new(),
            }),
            Err(CommandCodecError::UnknownMethod(999))
        );
        assert_eq!(
            decode_command(&RequestMessage {
                method: METHOD_SESSION_OPEN,
                body: vec![0; 15],
            }),
            Err(CommandCodecError::Truncated)
        );
        let mut message = encode_command(&Command::Doctor).unwrap();
        message.body.push(1);
        assert_eq!(
            decode_command(&message),
            Err(CommandCodecError::TrailingBytes { actual: 1 })
        );
        let mut body = vec![0; 32];
        body.push(255);
        assert_eq!(
            decode_command(&RequestMessage {
                method: METHOD_EFFECT_SIMULATE,
                body,
            }),
            Err(CommandCodecError::InvalidSyntheticSemantics(255))
        );
    }

    #[test]
    fn fields_and_total_body_are_bounded() {
        let too_large = vec![0; MAX_EVENT_PAYLOAD_BYTES + 1];
        assert!(matches!(
            encode_command(&Command::SessionCreate {
                session_id: SessionId(1),
                event_id: EventId(2),
                recorded_at: "time".to_owned(),
                payload: too_large,
            }),
            Err(CommandCodecError::FieldTooLarge { .. })
        ));
        assert!(matches!(
            decode_command(&RequestMessage {
                method: METHOD_DOCTOR,
                body: vec![0; MAX_COMMAND_BODY_BYTES + 1],
            }),
            Err(CommandCodecError::BodyTooLarge { .. })
        ));
    }
}
