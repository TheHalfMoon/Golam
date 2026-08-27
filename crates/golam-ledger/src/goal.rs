use std::error::Error;
use std::fmt;
use std::path::Path;

use golam_core::{CanonicalEncoder, EventId, GoalId, GoalVersionId, SCHEMA_VERSION, SessionId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::{EventKind, EventRecord, audit_integrity_hash, event_integrity_hash, payload_hash};

const GOAL_DOMAIN: &[u8] = b"golam:goal-version:v1";
const SECURITY_AUDIT_CHAIN: &str = "security";
pub const MAX_GOAL_PAYLOAD_BYTES: usize = 256 * 1024;

#[derive(Clone, Copy)]
pub struct GoalDocument<'a> {
    pub goal: &'a str,
    pub acceptance_criteria: &'a [&'a str],
    pub constraints: &'a [&'a str],
    pub scope: &'a str,
    pub proven_facts: &'a [&'a str],
    pub blockers: &'a [&'a str],
    pub next_safe_action: Option<&'a str>,
}

pub struct CreateGoalVersion<'a> {
    pub goal_version_id: GoalVersionId,
    pub goal_id: GoalId,
    pub event_id: EventId,
    pub session_id: SessionId,
    pub expected_session_seq: u64,
    pub expected_goal_version: u64,
    pub actor_principal: &'a str,
    pub recorded_at: &'a str,
    pub document: GoalDocument<'a>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredGoalVersion {
    pub goal_version_id: GoalVersionId,
    pub goal_id: GoalId,
    pub session_id: SessionId,
    pub version: u64,
    pub payload_bytes: Vec<u8>,
    pub created_event_id: EventId,
    pub created_global_seq: u64,
}

#[derive(Debug)]
pub enum GoalError {
    Sqlite(rusqlite::Error),
    Core(golam_core::CoreError),
    SessionNotFound(SessionId),
    StaleSessionHead { expected: u64, actual: u64 },
    StaleGoalVersion { expected: u64, actual: u64 },
    PayloadTooLarge { actual: usize, maximum: usize },
    SequenceOverflow,
    InvalidStoredId,
    Verification(&'static str),
}

impl fmt::Display for GoalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "goal sqlite error: {error}"),
            Self::Core(error) => write!(f, "goal canonical encoding error: {error}"),
            Self::SessionNotFound(session_id) => {
                write!(f, "goal session not found: {}", session_id.0)
            }
            Self::StaleSessionHead { expected, actual } => {
                write!(
                    f,
                    "stale goal session head: expected {expected}, actual {actual}"
                )
            }
            Self::StaleGoalVersion { expected, actual } => {
                write!(
                    f,
                    "stale goal version: expected {expected}, actual {actual}"
                )
            }
            Self::PayloadTooLarge { actual, maximum } => {
                write!(f, "goal payload is {actual} bytes; maximum is {maximum}")
            }
            Self::SequenceOverflow => f.write_str("goal sequence exceeds SQLite integer range"),
            Self::InvalidStoredId => f.write_str("stored goal identifier is not 16 bytes"),
            Self::Verification(reason) => write!(f, "goal ledger verification failed: {reason}"),
        }
    }
}

impl Error for GoalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::SessionNotFound(_)
            | Self::StaleSessionHead { .. }
            | Self::StaleGoalVersion { .. }
            | Self::PayloadTooLarge { .. }
            | Self::SequenceOverflow
            | Self::InvalidStoredId
            | Self::Verification(_) => None,
        }
    }
}

impl From<rusqlite::Error> for GoalError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<golam_core::CoreError> for GoalError {
    fn from(value: golam_core::CoreError) -> Self {
        Self::Core(value)
    }
}

pub struct GoalManager {
    connection: Connection,
}

impl GoalManager {
    pub fn open(authority_db: impl AsRef<Path>) -> Result<Self, GoalError> {
        let connection = Connection::open(authority_db)?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;\n\
             PRAGMA busy_timeout = 5000;\n\
             CREATE TRIGGER IF NOT EXISTS goal_versions_append_only_update\n\
             BEFORE UPDATE ON goal_versions\n\
             BEGIN SELECT RAISE(ABORT, 'goal versions are append-only'); END;\n\
             CREATE TRIGGER IF NOT EXISTS goal_versions_append_only_delete\n\
             BEFORE DELETE ON goal_versions\n\
             BEGIN SELECT RAISE(ABORT, 'goal versions are append-only'); END;",
        )?;
        Ok(Self { connection })
    }

    pub fn append_version(
        &mut self,
        input: CreateGoalVersion<'_>,
    ) -> Result<StoredGoalVersion, GoalError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let session_blob = id_blob(input.session_id.0);
        let session_head = transaction
            .query_row(
                "SELECT latest_session_seq, latest_event_hash FROM sessions WHERE session_id = ?1",
                params![&session_blob],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
            )
            .optional()?
            .ok_or(GoalError::SessionNotFound(input.session_id))?;
        let actual_session_seq = seq_from_i64(session_head.0)?;
        if actual_session_seq != input.expected_session_seq {
            return Err(GoalError::StaleSessionHead {
                expected: input.expected_session_seq,
                actual: actual_session_seq,
            });
        }
        let previous_session_event_hash = hash_from_vec(session_head.1)?;

        let goal_blob = id_blob(input.goal_id.0);
        let current_version: i64 = transaction.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM goal_versions WHERE goal_id = ?1",
            params![&goal_blob],
            |row| row.get(0),
        )?;
        let current_version = seq_from_i64(current_version)?;
        if current_version != input.expected_goal_version {
            return Err(GoalError::StaleGoalVersion {
                expected: input.expected_goal_version,
                actual: current_version,
            });
        }
        let version = current_version
            .checked_add(1)
            .ok_or(GoalError::SequenceOverflow)?;
        let payload =
            encode_goal_payload(input.goal_id, input.session_id, version, input.document)?;
        if payload.len() > MAX_GOAL_PAYLOAD_BYTES {
            return Err(GoalError::PayloadTooLarge {
                actual: payload.len(),
                maximum: MAX_GOAL_PAYLOAD_BYTES,
            });
        }

        let global_seq = next_global_seq(&transaction)?;
        let session_seq = actual_session_seq
            .checked_add(1)
            .ok_or(GoalError::SequenceOverflow)?;
        let previous_audit_hash = audit_head(&transaction)?;
        let record = EventRecord {
            event_id: input.event_id,
            session_id: input.session_id,
            global_seq,
            session_seq,
            schema_version: SCHEMA_VERSION,
            kind: EventKind::GoalVersioned,
            actor_principal: input.actor_principal.to_owned(),
            recorded_at: input.recorded_at.to_owned(),
            payload_hash: payload_hash(&payload),
            previous_session_event_hash: Some(previous_session_event_hash),
            security_critical: true,
            previous_audit_hash,
        };
        let event_hash = event_integrity_hash(&record)?;
        let audit_hash = audit_integrity_hash(&record, event_hash)?;

        insert_event(&transaction, &record, &payload, event_hash, audit_hash)?;
        transaction.execute(
            "INSERT INTO goal_versions (goal_version_id, goal_id, session_id, version, payload_bytes, \
             created_event_id, created_global_seq) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id_blob(input.goal_version_id.0),
                &goal_blob,
                &session_blob,
                seq_to_i64(version)?,
                &payload,
                id_blob(input.event_id.0),
                seq_to_i64(global_seq)?,
            ],
        )?;
        let updated = transaction.execute(
            "UPDATE sessions SET latest_session_seq = ?1, latest_event_hash = ?2 \
             WHERE session_id = ?3 AND latest_session_seq = ?4",
            params![
                seq_to_i64(session_seq)?,
                &event_hash[..],
                &session_blob,
                seq_to_i64(input.expected_session_seq)?,
            ],
        )?;
        if updated != 1 {
            return Err(GoalError::StaleSessionHead {
                expected: input.expected_session_seq,
                actual: actual_session_seq,
            });
        }
        update_audit_head(&transaction, global_seq, audit_hash)?;
        transaction.commit()?;

        Ok(StoredGoalVersion {
            goal_version_id: input.goal_version_id,
            goal_id: input.goal_id,
            session_id: input.session_id,
            version,
            payload_bytes: payload,
            created_event_id: input.event_id,
            created_global_seq: global_seq,
        })
    }

    pub fn current(&self, goal_id: GoalId) -> Result<Option<StoredGoalVersion>, GoalError> {
        let row = self
            .connection
            .query_row(
                "SELECT goal_version_id, session_id, version, payload_bytes, created_event_id, \
                 created_global_seq FROM goal_versions WHERE goal_id = ?1 ORDER BY version DESC LIMIT 1",
                params![id_blob(goal_id.0)],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, Vec<u8>>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, Vec<u8>>(3)?,
                        row.get::<_, Vec<u8>>(4)?,
                        row.get::<_, i64>(5)?,
                    ))
                },
            )
            .optional()?;
        row.map(|row| {
            Ok(StoredGoalVersion {
                goal_version_id: GoalVersionId(id_from_vec(row.0)?),
                goal_id,
                session_id: SessionId(id_from_vec(row.1)?),
                version: seq_from_i64(row.2)?,
                payload_bytes: row.3,
                created_event_id: EventId(id_from_vec(row.4)?),
                created_global_seq: seq_from_i64(row.5)?,
            })
        })
        .transpose()
    }

    pub fn verify_all(&self) -> Result<(), GoalError> {
        let mut statement = self.connection.prepare(
            "SELECT goal_id, version, payload_bytes, created_event_id, created_global_seq \
             FROM goal_versions ORDER BY goal_id, version",
        )?;
        let mut rows = statement.query([])?;
        let mut previous_goal: Option<GoalId> = None;
        let mut expected_version = 1_u64;
        while let Some(row) = rows.next()? {
            let goal_id = GoalId(id_from_vec(row.get(0)?)?);
            let version = seq_from_i64(row.get(1)?)?;
            if previous_goal != Some(goal_id) {
                previous_goal = Some(goal_id);
                expected_version = 1;
            }
            if version != expected_version {
                return Err(GoalError::Verification("goal versions are not contiguous"));
            }
            expected_version = expected_version
                .checked_add(1)
                .ok_or(GoalError::SequenceOverflow)?;

            let payload: Vec<u8> = row.get(2)?;
            let event_id = id_from_vec(row.get(3)?)?;
            let created_global_seq = seq_from_i64(row.get(4)?)?;
            let event = self
                .connection
                .query_row(
                    "SELECT global_seq, event_type, payload_bytes FROM session_events WHERE event_id = ?1",
                    params![id_blob(event_id)],
                    |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, i64>(1)?,
                            row.get::<_, Vec<u8>>(2)?,
                        ))
                    },
                )
                .optional()?
                .ok_or(GoalError::Verification("goal event is missing"))?;
            if seq_from_i64(event.0)? != created_global_seq {
                return Err(GoalError::Verification(
                    "goal event global sequence mismatch",
                ));
            }
            let event_code = u8::try_from(event.1)
                .map_err(|_| GoalError::Verification("invalid goal event type"))?;
            if EventKind::from_code(event_code) != Some(EventKind::GoalVersioned) {
                return Err(GoalError::Verification(
                    "goal row is not linked to GoalVersioned event",
                ));
            }
            if event.2 != payload {
                return Err(GoalError::Verification(
                    "goal row payload differs from canonical event",
                ));
            }
        }
        Ok(())
    }
}

fn encode_goal_payload(
    goal_id: GoalId,
    session_id: SessionId,
    version: u64,
    document: GoalDocument<'_>,
) -> Result<Vec<u8>, GoalError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(GOAL_DOMAIN)?;
    encoder.push_u128(goal_id.0);
    encoder.push_u128(session_id.0);
    encoder.push_u64(version);
    encoder.push_bytes(document.goal.as_bytes())?;
    encode_text_list(&mut encoder, document.acceptance_criteria)?;
    encode_text_list(&mut encoder, document.constraints)?;
    encoder.push_bytes(document.scope.as_bytes())?;
    encode_text_list(&mut encoder, document.proven_facts)?;
    encode_text_list(&mut encoder, document.blockers)?;
    match document.next_safe_action {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(value.as_bytes())?;
        }
        None => encoder.push_u8(0),
    }
    Ok(encoder.finish())
}

fn encode_text_list(encoder: &mut CanonicalEncoder, values: &[&str]) -> Result<(), GoalError> {
    let len = u64::try_from(values.len()).map_err(|_| GoalError::SequenceOverflow)?;
    encoder.push_u64(len);
    for value in values {
        encoder.push_bytes(value.as_bytes())?;
    }
    Ok(())
}

fn insert_event(
    transaction: &Transaction<'_>,
    record: &EventRecord,
    payload: &[u8],
    event_hash: [u8; 32],
    audit_hash: [u8; 32],
) -> Result<(), GoalError> {
    transaction.execute(
        "INSERT INTO session_events (event_id, global_seq, session_id, session_seq, event_type, \
         schema_version, actor_principal, recorded_at, payload_bytes, payload_hash, \
         previous_session_event_hash, event_hash, security_critical, previous_audit_hash, audit_hash) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 1, ?13, ?14)",
        params![
            id_blob(record.event_id.0),
            seq_to_i64(record.global_seq)?,
            id_blob(record.session_id.0),
            seq_to_i64(record.session_seq)?,
            i64::from(record.kind.code()),
            i64::from(record.schema_version),
            record.actor_principal,
            record.recorded_at,
            payload,
            &record.payload_hash[..],
            record.previous_session_event_hash.map(|hash| hash.to_vec()),
            &event_hash[..],
            record.previous_audit_hash.map(|hash| hash.to_vec()),
            &audit_hash[..],
        ],
    )?;
    Ok(())
}

fn next_global_seq(transaction: &Transaction<'_>) -> Result<u64, GoalError> {
    let current: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(global_seq), 0) FROM (\
           SELECT global_seq FROM session_events \
           UNION ALL SELECT global_seq FROM effect_transitions \
           UNION ALL SELECT global_seq FROM authorization_decisions\
         )",
        [],
        |row| row.get(0),
    )?;
    seq_from_i64(current)?
        .checked_add(1)
        .ok_or(GoalError::SequenceOverflow)
}

fn audit_head(transaction: &Transaction<'_>) -> Result<Option<[u8; 32]>, GoalError> {
    transaction
        .query_row(
            "SELECT last_hash FROM audit_chain_heads WHERE chain_name = ?1",
            params![SECURITY_AUDIT_CHAIN],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .optional()?
        .map(hash_from_vec)
        .transpose()
}

fn update_audit_head(
    transaction: &Transaction<'_>,
    global_seq: u64,
    audit_hash: [u8; 32],
) -> Result<(), GoalError> {
    transaction.execute(
        "INSERT INTO audit_chain_heads (chain_name, last_global_seq, last_hash) VALUES (?1, ?2, ?3) \
         ON CONFLICT(chain_name) DO UPDATE SET last_global_seq = excluded.last_global_seq, \
         last_hash = excluded.last_hash",
        params![SECURITY_AUDIT_CHAIN, seq_to_i64(global_seq)?, &audit_hash[..]],
    )?;
    Ok(())
}

fn id_blob(value: u128) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

fn id_from_vec(value: Vec<u8>) -> Result<u128, GoalError> {
    let bytes: [u8; 16] = value.try_into().map_err(|_| GoalError::InvalidStoredId)?;
    Ok(u128::from_be_bytes(bytes))
}

fn hash_from_vec(value: Vec<u8>) -> Result<[u8; 32], GoalError> {
    value
        .try_into()
        .map_err(|_| GoalError::Verification("stored goal hash is not 32 bytes"))
}

fn seq_to_i64(value: u64) -> Result<i64, GoalError> {
    i64::try_from(value).map_err(|_| GoalError::SequenceOverflow)
}

fn seq_from_i64(value: i64) -> Result<u64, GoalError> {
    u64::try_from(value).map_err(|_| GoalError::SequenceOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{AuthorityStore, CreateSession};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_root() -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("golam-goal-{}-{nonce}", std::process::id()))
    }

    fn doc<'a>(goal: &'a str, next: Option<&'a str>) -> GoalDocument<'a> {
        GoalDocument {
            goal,
            acceptance_criteria: &["tests pass"],
            constraints: &["local only"],
            scope: "spec-002",
            proven_facts: &[],
            blockers: &[],
            next_safe_action: next,
        }
    }

    #[test]
    fn goal_versions_append_atomically_and_stale_writes_do_not_advance_history() {
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        let db_path = root.join("authority.db");
        let mut authority = AuthorityStore::open(&db_path).unwrap();
        authority
            .create_session(CreateSession {
                session_id: SessionId(1),
                event_id: EventId(1),
                owner_principal: "owner",
                actor_principal: "owner",
                recorded_at: "2026-08-24T00:00:00Z",
                payload: b"create",
                security_critical: true,
            })
            .unwrap();

        let mut goals = GoalManager::open(&db_path).unwrap();
        let first = goals
            .append_version(CreateGoalVersion {
                goal_version_id: GoalVersionId(10),
                goal_id: GoalId(9),
                event_id: EventId(2),
                session_id: SessionId(1),
                expected_session_seq: 1,
                expected_goal_version: 0,
                actor_principal: "owner",
                recorded_at: "2026-08-24T00:00:01Z",
                document: doc("build durable spine", Some("add fork")),
            })
            .unwrap();
        assert_eq!(first.version, 1);
        assert_eq!(first.created_global_seq, 2);

        let second = goals
            .append_version(CreateGoalVersion {
                goal_version_id: GoalVersionId(11),
                goal_id: GoalId(9),
                event_id: EventId(3),
                session_id: SessionId(1),
                expected_session_seq: 2,
                expected_goal_version: 1,
                actor_principal: "owner",
                recorded_at: "2026-08-24T00:00:02Z",
                document: doc("build durable spine", Some("add IPC")),
            })
            .unwrap();
        assert_eq!(second.version, 2);
        assert_eq!(second.created_global_seq, 3);

        let stale = goals.append_version(CreateGoalVersion {
            goal_version_id: GoalVersionId(12),
            goal_id: GoalId(9),
            event_id: EventId(4),
            session_id: SessionId(1),
            expected_session_seq: 3,
            expected_goal_version: 1,
            actor_principal: "owner",
            recorded_at: "2026-08-24T00:00:03Z",
            document: doc("stale", None),
        });
        assert!(matches!(
            stale,
            Err(GoalError::StaleGoalVersion {
                expected: 1,
                actual: 2
            })
        ));

        let third = goals
            .append_version(CreateGoalVersion {
                goal_version_id: GoalVersionId(13),
                goal_id: GoalId(9),
                event_id: EventId(5),
                session_id: SessionId(1),
                expected_session_seq: 3,
                expected_goal_version: 2,
                actor_principal: "owner",
                recorded_at: "2026-08-24T00:00:04Z",
                document: doc("build durable spine", Some("qualify")),
            })
            .unwrap();
        assert_eq!(third.version, 3);
        assert_eq!(third.created_global_seq, 4);
        assert_eq!(goals.current(GoalId(9)).unwrap(), Some(third));
        goals.verify_all().unwrap();

        let mutation = goals.connection.execute(
            "UPDATE goal_versions SET version = 99 WHERE goal_id = ?1 AND version = 1",
            params![id_blob(GoalId(9).0)],
        );
        assert!(mutation.is_err());

        authority.verify_integrity().unwrap();
        drop(goals);
        drop(authority);
        fs::remove_dir_all(root).unwrap();
    }
}
