#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::{CheckpointId, ClientId, EffectId, EventId, GoalId, GoalVersionId, SessionId};
use golam_ipc::command::{AuthorityQualificationKind, Command, SyntheticSemantics};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliError {
    Usage(String),
    InvalidInteger { field: &'static str, value: String },
    InvalidSemantics(String),
    InvalidHex { field: &'static str, value: String },
    InvalidQualificationKind(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(f, "{message}"),
            Self::InvalidInteger { field, value } => {
                write!(f, "invalid {field} integer: {value}")
            }
            Self::InvalidSemantics(value) => write!(
                f,
                "invalid synthetic semantics {value}; expected read-only, idempotent-at-least-once, at-most-once, compensatable, or irreversible"
            ),
            Self::InvalidHex { field, value } => {
                write!(
                    f,
                    "invalid {field} hex value: {value}; expected exactly 32 lowercase or uppercase hex digits"
                )
            }
            Self::InvalidQualificationKind(value) => write!(
                f,
                "invalid authority qualification kind {value}; expected lease, approval, secret-canary, or sandbox-profile"
            ),
        }
    }
}

impl Error for CliError {}

pub fn parse_args<I, S>(args: I) -> Result<Command, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    let values = args.iter().map(String::as_str).collect::<Vec<_>>();
    parse_values(&values)
}

pub fn usage() -> &'static str {
    "Golam CLI\n\
usage:\n\
  golam client enroll <client-id>\n\
  golam sessions\n\
  golam session open <session-id>\n\
  golam session create <session-id> <event-id> <recorded-at> <payload>\n\
  golam session fork <child-session-id> <event-id> <parent-session-id> <through-session-seq> <recorded-at>\n\
  golam goal append <goal-version-id> <goal-id> <event-id> <session-id> <expected-session-seq> <expected-goal-version> <recorded-at> <goal>\n\
  golam checkpoint create <checkpoint-id> <event-id> <session-id> <through-session-seq> <recorded-at>\n\
  golam checkpoint verify <checkpoint-id> <session-id> <through-session-seq>\n\
  golam replay <session-id> <through-session-seq>\n\
  golam effect simulate <effect-id> <session-id> <semantics>\n\
  golam effect reconcile <effect-id>\n\
  golam policy validate <policy-source> <schema-source>\n\
  golam authority qualify <lease|approval|secret-canary|sandbox-profile>\n\
  golam authority explain <decision-id-hex>\n\
  golam doctor"
}

fn parse_values(args: &[&str]) -> Result<Command, CliError> {
    match args {
        ["client", "enroll", client_id] => Ok(Command::ClientEnroll {
            client_id: ClientId(parse_u128("client-id", client_id)?),
        }),
        ["sessions"] => Ok(Command::SessionsList),
        ["session", "open", session_id] => Ok(Command::SessionOpen {
            session_id: SessionId(parse_u128("session-id", session_id)?),
        }),
        [
            "session",
            "create",
            session_id,
            event_id,
            recorded_at,
            payload,
        ] => Ok(Command::SessionCreate {
            session_id: SessionId(parse_u128("session-id", session_id)?),
            event_id: EventId(parse_u128("event-id", event_id)?),
            recorded_at: (*recorded_at).to_owned(),
            payload: payload.as_bytes().to_vec(),
        }),
        [
            "session",
            "fork",
            child_session_id,
            event_id,
            parent_session_id,
            through_session_seq,
            recorded_at,
        ] => Ok(Command::SessionFork {
            child_session_id: SessionId(parse_u128("child-session-id", child_session_id)?),
            event_id: EventId(parse_u128("event-id", event_id)?),
            parent_session_id: SessionId(parse_u128("parent-session-id", parent_session_id)?),
            through_session_seq: parse_u64("through-session-seq", through_session_seq)?,
            recorded_at: (*recorded_at).to_owned(),
        }),
        [
            "goal",
            "append",
            goal_version_id,
            goal_id,
            event_id,
            session_id,
            expected_session_seq,
            expected_goal_version,
            recorded_at,
            goal,
        ] => Ok(Command::GoalAppend {
            goal_version_id: GoalVersionId(parse_u128("goal-version-id", goal_version_id)?),
            goal_id: GoalId(parse_u128("goal-id", goal_id)?),
            event_id: EventId(parse_u128("event-id", event_id)?),
            session_id: SessionId(parse_u128("session-id", session_id)?),
            expected_session_seq: parse_u64("expected-session-seq", expected_session_seq)?,
            expected_goal_version: parse_u64("expected-goal-version", expected_goal_version)?,
            recorded_at: (*recorded_at).to_owned(),
            goal: (*goal).to_owned(),
        }),
        [
            "checkpoint",
            "create",
            checkpoint_id,
            event_id,
            session_id,
            through_session_seq,
            recorded_at,
        ] => Ok(Command::CheckpointCreate {
            checkpoint_id: CheckpointId(parse_u128("checkpoint-id", checkpoint_id)?),
            created_event_id: EventId(parse_u128("event-id", event_id)?),
            session_id: SessionId(parse_u128("session-id", session_id)?),
            through_session_seq: parse_u64("through-session-seq", through_session_seq)?,
            recorded_at: (*recorded_at).to_owned(),
        }),
        [
            "checkpoint",
            "verify",
            checkpoint_id,
            session_id,
            through_session_seq,
        ] => Ok(Command::CheckpointVerify {
            checkpoint_id: CheckpointId(parse_u128("checkpoint-id", checkpoint_id)?),
            session_id: SessionId(parse_u128("session-id", session_id)?),
            through_session_seq: parse_u64("through-session-seq", through_session_seq)?,
        }),
        ["replay", session_id, through_session_seq] => Ok(Command::Replay {
            session_id: SessionId(parse_u128("session-id", session_id)?),
            through_session_seq: parse_u64("through-session-seq", through_session_seq)?,
        }),
        ["effect", "simulate", effect_id, session_id, semantics] => Ok(Command::EffectSimulate {
            effect_id: EffectId(parse_u128("effect-id", effect_id)?),
            session_id: SessionId(parse_u128("session-id", session_id)?),
            semantics: parse_semantics(semantics)?,
        }),
        ["effect", "reconcile", effect_id] => Ok(Command::EffectReconcile {
            effect_id: EffectId(parse_u128("effect-id", effect_id)?),
        }),
        ["policy", "validate", policy_source, schema_source] => Ok(Command::PolicyValidate {
            policy_source: (*policy_source).to_owned(),
            schema_source: (*schema_source).to_owned(),
        }),
        ["authority", "qualify", kind] => Ok(Command::AuthorityQualify {
            kind: parse_qualification_kind(kind)?,
        }),
        ["authority", "explain", decision_id] => Ok(Command::AuthorityExplain {
            decision_id: parse_hex_16("decision-id", decision_id)?,
        }),
        ["doctor"] => Ok(Command::Doctor),
        _ => Err(CliError::Usage(usage().to_owned())),
    }
}

fn parse_u128(field: &'static str, value: &str) -> Result<u128, CliError> {
    value.parse().map_err(|_| CliError::InvalidInteger {
        field,
        value: value.to_owned(),
    })
}

fn parse_u64(field: &'static str, value: &str) -> Result<u64, CliError> {
    value.parse().map_err(|_| CliError::InvalidInteger {
        field,
        value: value.to_owned(),
    })
}

fn parse_hex_16(field: &'static str, value: &str) -> Result<[u8; 16], CliError> {
    if value.len() != 32 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(CliError::InvalidHex {
            field,
            value: value.to_owned(),
        });
    }
    u128::from_str_radix(value, 16)
        .map(u128::to_be_bytes)
        .map_err(|_| CliError::InvalidHex {
            field,
            value: value.to_owned(),
        })
}

fn parse_qualification_kind(value: &str) -> Result<AuthorityQualificationKind, CliError> {
    match value {
        "lease" => Ok(AuthorityQualificationKind::Lease),
        "approval" => Ok(AuthorityQualificationKind::Approval),
        "secret-canary" => Ok(AuthorityQualificationKind::SecretCanary),
        "sandbox-profile" => Ok(AuthorityQualificationKind::SandboxProfile),
        _ => Err(CliError::InvalidQualificationKind(value.to_owned())),
    }
}

fn parse_semantics(value: &str) -> Result<SyntheticSemantics, CliError> {
    match value {
        "read-only" | "read_only" => Ok(SyntheticSemantics::ReadOnly),
        "idempotent-at-least-once" | "idempotent_at_least_once" => {
            Ok(SyntheticSemantics::IdempotentAtLeastOnce)
        }
        "at-most-once" | "at_most_once" => Ok(SyntheticSemantics::AtMostOnce),
        "compensatable" => Ok(SyntheticSemantics::Compensatable),
        "irreversible" => Ok(SyntheticSemantics::Irreversible),
        _ => Err(CliError::InvalidSemantics(value.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_command_surface_parses_to_typed_ipc_commands() {
        let cases = [
            (
                vec!["client", "enroll", "1"],
                Command::ClientEnroll {
                    client_id: ClientId(1),
                },
            ),
            (vec!["sessions"], Command::SessionsList),
            (
                vec!["session", "open", "2"],
                Command::SessionOpen {
                    session_id: SessionId(2),
                },
            ),
            (
                vec![
                    "session",
                    "create",
                    "3",
                    "4",
                    "2026-08-26T01:40:00Z",
                    "root",
                ],
                Command::SessionCreate {
                    session_id: SessionId(3),
                    event_id: EventId(4),
                    recorded_at: "2026-08-26T01:40:00Z".to_owned(),
                    payload: b"root".to_vec(),
                },
            ),
            (
                vec!["checkpoint", "verify", "5", "3", "1"],
                Command::CheckpointVerify {
                    checkpoint_id: CheckpointId(5),
                    session_id: SessionId(3),
                    through_session_seq: 1,
                },
            ),
            (
                vec!["replay", "3", "1"],
                Command::Replay {
                    session_id: SessionId(3),
                    through_session_seq: 1,
                },
            ),
            (
                vec!["effect", "simulate", "6", "3", "irreversible"],
                Command::EffectSimulate {
                    effect_id: EffectId(6),
                    session_id: SessionId(3),
                    semantics: SyntheticSemantics::Irreversible,
                },
            ),
            (
                vec!["effect", "reconcile", "6"],
                Command::EffectReconcile {
                    effect_id: EffectId(6),
                },
            ),
            (
                vec![
                    "policy",
                    "validate",
                    "permit(principal, action, resource);",
                    "entity User;",
                ],
                Command::PolicyValidate {
                    policy_source: "permit(principal, action, resource);".to_owned(),
                    schema_source: "entity User;".to_owned(),
                },
            ),
            (
                vec!["authority", "qualify", "secret-canary"],
                Command::AuthorityQualify {
                    kind: AuthorityQualificationKind::SecretCanary,
                },
            ),
            (
                vec!["authority", "explain", "07070707070707070707070707070707"],
                Command::AuthorityExplain {
                    decision_id: [7; 16],
                },
            ),
            (vec!["doctor"], Command::Doctor),
        ];
        for (args, expected) in cases {
            assert_eq!(parse_args(args).unwrap(), expected);
        }
    }

    #[test]
    fn fork_goal_and_checkpoint_create_are_covered() {
        assert!(matches!(
            parse_args([
                "session",
                "fork",
                "10",
                "11",
                "3",
                "1",
                "2026-08-26T01:41:00Z"
            ]),
            Ok(Command::SessionFork {
                child_session_id: SessionId(10),
                ..
            })
        ));
        assert!(matches!(
            parse_args([
                "goal",
                "append",
                "12",
                "13",
                "14",
                "3",
                "1",
                "0",
                "2026-08-26T01:42:00Z",
                "finish-spec-002"
            ]),
            Ok(Command::GoalAppend {
                goal_id: GoalId(13),
                ..
            })
        ));
        assert!(matches!(
            parse_args([
                "checkpoint",
                "create",
                "15",
                "16",
                "3",
                "1",
                "2026-08-26T01:43:00Z"
            ]),
            Ok(Command::CheckpointCreate {
                checkpoint_id: CheckpointId(15),
                ..
            })
        ));
    }

    #[test]
    fn invalid_numbers_semantics_and_shapes_fail_closed() {
        assert!(matches!(
            parse_args(["session", "open", "not-a-number"]),
            Err(CliError::InvalidInteger {
                field: "session-id",
                ..
            })
        ));
        assert_eq!(
            parse_args(["effect", "simulate", "1", "2", "unsafe"]),
            Err(CliError::InvalidSemantics("unsafe".to_owned()))
        );
        assert!(matches!(parse_args(["session"]), Err(CliError::Usage(_))));
        assert!(matches!(
            parse_args(["authority", "explain", "xyz"]),
            Err(CliError::InvalidHex { .. })
        ));
        assert_eq!(
            parse_args(["authority", "qualify", "unknown"]),
            Err(CliError::InvalidQualificationKind("unknown".to_owned()))
        );
    }
}
