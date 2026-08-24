use std::error::Error;
use std::fmt;
use std::path::Path;

use golam_core::{CanonicalEncoder, EventId, SCHEMA_VERSION, SessionId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::{
    EventKind, EventRecord, audit_integrity_hash, event_integrity_hash, payload_hash,
};

const FORK_EVENT_DOMAIN: &[u8] = b"golam:session-fork:v1";
const SECURITY_AUDIT_CHAIN: &str = "security";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForkAnchor {
    pub parent_session_id: SessionId,
    pub parent_session_seq: u64,
    pub parent_global_seq: u64,
    pub parent_event_hash: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForkRecord {
    pub child_session_id: SessionId,
    pub event_id: EventId,
    pub child_global_seq: u64,
    pub child_event_hash: [u8; 32],
    pub anchor: ForkAnchor,
}

pub struct CreateFork<'a> {
    pub child_session_id: SessionId,
    pub event_id: EventId,
    pub parent_session_id: SessionId,
    pub through_session_seq: u64,
    pub actor_principal: &'a str,
    pub recorded_at: &'a str,
}

#[derive(Debug)]
pub enum ForkError {
    Sqlite(rusqlite::Error),
    Core(golam_core::CoreError),
    ParentAnchorNotFound {
        session_id: SessionId,
        through_session_seq: u64,
    },
    ChildAlreadyExists(SessionId),
    InvalidStoredId,
    InvalidStoredHash,
    InvalidStoredEventKind,
    SequenceOverflow,
    AnchorViolation(&'static str),
}

impl fmt::Display for ForkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "fork sqlite error: {error}"),
            Self::Core(error) => write!(f, "fork canonical encoding error: {error}"),
            Self::ParentAnchorNotFound {
                session_id,
                through_session_seq,
            } => write!(
                f,
                "fork parent anchor not found for session {} at sequence {through_session_seq}",
                session_id.0
            ),
            Self::ChildAlreadyExists(session_id) => {
                write!(f, "fork child session already exists: {}", session_id.0)
            }
            Self::InvalidStoredId => f.write_str("stored fork identifier is not 16 bytes"),
            Self::InvalidStoredHash => f.write_str("stored fork hash is not 32 bytes"),
            Self::InvalidStoredEventKind => f.write_str("stored fork event kind is invalid"),
            Self::SequenceOverflow => f.write_str("fork sequence exceeds SQLite integer range"),
            Self::AnchorViolation(reason) => write!(f, "fork anchor integrity violation: {reason}"),
        }
    }
}

impl Error for ForkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::ParentAnchorNotFound { .. }
            | Self::ChildAlreadyExists(_)
            | Self::InvalidStoredId
            | Self::InvalidStoredHash
            | Self::InvalidStoredEventKind
            | Self::SequenceOverflow
            | Self::AnchorViolation(_) => None,
        }
    }
}

impl From<rusqlite::Error> for ForkError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<golam_core::CoreError> for ForkError {
    fn from(value: golam_core::CoreError) -> Self {
        Self::Core(value)
    }
}

pub struct ForkManager {
    connection: Connection,
}

impl ForkManager {
    pub fn open(authority_db: impl AsRef<Path>) -> Result<Self, ForkError> {
        let connection = Connection::open(authority_db)?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;\n\
             PRAGMA busy_timeout = 5000;\n\
             CREATE TRIGGER IF NOT EXISTS sessions_fork_anchor_immutable\n\
             BEFORE UPDATE OF parent_session_id, parent_session_seq, parent_event_hash ON sessions\n\
             WHEN OLD.parent_session_id IS NOT NEW.parent_session_id\n\
               OR OLD.parent_session_seq IS NOT NEW.parent_session_seq\n\
               OR OLD.parent_event_hash IS NOT NEW.parent_event_hash\n\
             BEGIN\n\
               SELECT RAISE(ABORT, 'fork anchor is immutable');\n\
             END;",
        )?;
        Ok(Self { connection })
    }

    pub fn create(&mut self, input: CreateFork<'_>) -> Result<ForkRecord, ForkError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let child_blob = id_blob(input.child_session_id.0);
        let child_exists = transaction
            .query_row(
                "SELECT 1 FROM sessions WHERE session_id = ?1 LIMIT 1",
                params![&child_blob],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;
        if child_exists.is_some() {
            return Err(ForkError::ChildAlreadyExists(input.child_session_id));
        }

        let parent_blob = id_blob(input.parent_session_id.0);
        let parent = transaction
            .query_row(
                "SELECT s.owner_principal, e.global_seq, e.event_hash \
                 FROM sessions s JOIN session_events e ON e.session_id = s.session_id \
                 WHERE s.session_id = ?1 AND e.session_seq = ?2",
                params![&parent_blob, seq_to_i64(input.through_session_seq)?],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                    ))
                },
            )
            .optional()?
            .ok_or(ForkError::ParentAnchorNotFound {
                session_id: input.parent_session_id,
                through_session_seq: input.through_session_seq,
            })?;
        let anchor = ForkAnchor {
            parent_session_id: input.parent_session_id,
            parent_session_seq: input.through_session_seq,
            parent_global_seq: seq_from_i64(parent.1)?,
            parent_event_hash: hash_from_vec(parent.2)?,
        };

        let global_seq = next_global_seq(&transaction)?;
        let previous_audit_hash = audit_head(&transaction)?;
        let event_payload = fork_event_payload(input.child_session_id, anchor)?;
        let record = EventRecord {
            event_id: input.event_id,
            session_id: input.child_session_id,
            global_seq,
            session_seq: 1,
            schema_version: SCHEMA_VERSION,
            kind: EventKind::SessionForked,
            actor_principal: input.actor_principal.to_owned(),
            recorded_at: input.recorded_at.to_owned(),
            payload_hash: payload_hash(&event_payload),
            previous_session_event_hash: None,
            security_critical: true,
            previous_audit_hash,
        };
        let event_hash = event_integrity_hash(&record)?;
        let audit_hash = audit_integrity_hash(&record, event_hash)?;

        transaction.execute(
            "INSERT INTO sessions (session_id, owner_principal, created_global_seq, status, \
             parent_session_id, parent_session_seq, parent_event_hash, latest_session_seq, \
             latest_event_hash) VALUES (?1, ?2, ?3, 'active', ?4, ?5, ?6, 1, ?7)",
            params![
                &child_blob,
                parent.0,
                seq_to_i64(global_seq)?,
                &parent_blob,
                seq_to_i64(anchor.parent_session_seq)?,
                &anchor.parent_event_hash[..],
                &event_hash[..],
            ],
        )?;
        insert_event(
            &transaction,
            &record,
            &event_payload,
            event_hash,
            audit_hash,
        )?;
        update_audit_head(&transaction, global_seq, audit_hash)?;
        transaction.commit()?;

        Ok(ForkRecord {
            child_session_id: input.child_session_id,
            event_id: input.event_id,
            child_global_seq: global_seq,
            child_event_hash: event_hash,
            anchor,
        })
    }

    pub fn anchor(&self, child_session_id: SessionId) -> Result<Option<ForkAnchor>, ForkError> {
        let row = self
            .connection
            .query_row(
                "SELECT parent_session_id, parent_session_seq, parent_event_hash \
                 FROM sessions WHERE session_id = ?1 AND parent_session_id IS NOT NULL",
                params![id_blob(child_session_id.0)],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                    ))
                },
            )
            .optional()?;
        let Some((parent_id, parent_seq, parent_hash)) = row else {
            return Ok(None);
        };
        let parent_session_id = SessionId(id_from_vec(parent_id)?);
        let parent_session_seq = seq_from_i64(parent_seq)?;
        let parent_event_hash = hash_from_vec(parent_hash)?;
        let parent_global_seq = self
            .connection
            .query_row(
                "SELECT global_seq FROM session_events WHERE session_id = ?1 AND session_seq = ?2",
                params![id_blob(parent_session_id.0), seq_to_i64(parent_session_seq)?],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .ok_or(ForkError::AnchorViolation("parent event is missing"))?;
        Ok(Some(ForkAnchor {
            parent_session_id,
            parent_session_seq,
            parent_global_seq: seq_from_i64(parent_global_seq)?,
            parent_event_hash,
        }))
    }

    pub fn verify_all(&self) -> Result<(), ForkError> {
        let mut statement = self.connection.prepare(
            "SELECT session_id FROM sessions WHERE parent_session_id IS NOT NULL ORDER BY session_id",
        )?;
        let mut rows = statement.query([])?;
        let mut children = Vec::new();
        while let Some(row) = rows.next()? {
            children.push(SessionId(id_from_vec(row.get(0)?)?));
        }
        drop(rows);
        drop(statement);

        for child in children {
            let anchor = self
                .anchor(child)?
                .ok_or(ForkError::AnchorViolation("child fork anchor is incomplete"))?;
            let canonical_parent_hash = self
                .connection
                .query_row(
                    "SELECT event_hash FROM session_events WHERE session_id = ?1 AND session_seq = ?2",
                    params![
                        id_blob(anchor.parent_session_id.0),
                        seq_to_i64(anchor.parent_session_seq)?
                    ],
                    |row| row.get::<_, Vec<u8>>(0),
                )
                .optional()?
                .ok_or(ForkError::AnchorViolation("parent anchor event is missing"))?;
            if hash_from_vec(canonical_parent_hash)? != anchor.parent_event_hash {
                return Err(ForkError::AnchorViolation("parent anchor hash mismatch"));
            }

            let expected_payload = fork_event_payload(child, anchor)?;
            let stored = self
                .connection
                .query_row(
                    "SELECT event_id, event_type, payload_bytes FROM session_events \
                     WHERE session_id = ?1 AND session_seq = 1",
                    params![id_blob(child.0)],
                    |row| {
                        Ok((
                            row.get::<_, Vec<u8>>(0)?,
                            row.get::<_, i64>(1)?,
                            row.get::<_, Vec<u8>>(2)?,
                        ))
                    },
                )
                .optional()?
                .ok_or(ForkError::AnchorViolation("fork event is missing"))?;
            let _event_id = EventId(id_from_vec(stored.0)?);
            let event_code = u8::try_from(stored.1).map_err(|_| ForkError::InvalidStoredEventKind)?;
            if EventKind::from_code(event_code) != Some(EventKind::SessionForked) {
                return Err(ForkError::AnchorViolation("first child event is not SessionForked"));
            }
            if stored.2 != expected_payload {
                return Err(ForkError::AnchorViolation("fork event payload does not match anchor"));
            }
        }
        Ok(())
    }
}

fn fork_event_payload(
    child_session_id: SessionId,
    anchor: ForkAnchor,
) -> Result<Vec<u8>, ForkError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(FORK_EVENT_DOMAIN)?;
    encoder.push_u128(child_session_id.0);
    encoder.push_u128(anchor.parent_session_id.0);
    encoder.push_u64(anchor.parent_session_seq);
    encoder.push_u64(anchor.parent_global_seq);
    encoder.push_bytes(&anchor.parent_event_hash)?;
    Ok(encoder.finish())
}

fn insert_event(
    transaction: &Transaction<'_>,
    record: &EventRecord,
    payload: &[u8],
    event_hash: [u8; 32],
    audit_hash: [u8; 32],
) -> Result<(), ForkError> {
    transaction.execute(
        "INSERT INTO session_events (event_id, global_seq, session_id, session_seq, event_type, \
         schema_version, actor_principal, recorded_at, payload_bytes, payload_hash, \
         previous_session_event_hash, event_hash, security_critical, previous_audit_hash, audit_hash) \
         VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, ?7, ?8, ?9, NULL, ?10, 1, ?11, ?12)",
        params![
            id_blob(record.event_id.0),
            seq_to_i64(record.global_seq)?,
            id_blob(record.session_id.0),
            i64::from(record.kind.code()),
            i64::from(record.schema_version),
            record.actor_principal,
            record.recorded_at,
            payload,
            &record.payload_hash[..],
            &event_hash[..],
            record.previous_audit_hash.map(|hash| hash.to_vec()),
            &audit_hash[..],
        ],
    )?;
    Ok(())
}

fn next_global_seq(transaction: &Transaction<'_>) -> Result<u64, ForkError> {
    let current: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(global_seq), 0) FROM session_events",
        [],
        |row| row.get(0),
    )?;
    seq_from_i64(current)?
        .checked_add(1)
        .ok_or(ForkError::SequenceOverflow)
}

fn audit_head(transaction: &Transaction<'_>) -> Result<Option<[u8; 32]>, ForkError> {
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
) -> Result<(), ForkError> {
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

fn id_from_vec(value: Vec<u8>) -> Result<u128, ForkError> {
    let bytes: [u8; 16] = value.try_into().map_err(|_| ForkError::InvalidStoredId)?;
    Ok(u128::from_be_bytes(bytes))
}

fn hash_from_vec(value: Vec<u8>) -> Result<[u8; 32], ForkError> {
    value
        .try_into()
        .map_err(|_| ForkError::InvalidStoredHash)
}

fn seq_to_i64(value: u64) -> Result<i64, ForkError> {
    i64::try_from(value).map_err(|_| ForkError::SequenceOverflow)
}

fn seq_from_i64(value: i64) -> Result<u64, ForkError> {
    u64::try_from(value).map_err(|_| ForkError::SequenceOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{AppendEvent, AuthorityStore, CreateSession};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_root() -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("golam-fork-{}-{nonce}", std::process::id()))
    }

    #[test]
    fn fork_anchor_is_exact_immutable_and_parent_can_continue() {
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        let db_path = root.join("authority.db");
        let mut authority = AuthorityStore::open(&db_path).unwrap();
        let created = authority
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
        authority
            .append_event(AppendEvent {
                event_id: EventId(2),
                session_id: SessionId(1),
                expected_session_seq: 1,
                kind: EventKind::GoalVersioned,
                actor_principal: "owner",
                recorded_at: "2026-08-24T00:00:01Z",
                payload: b"goal",
                security_critical: true,
            })
            .unwrap();

        let mut forks = ForkManager::open(&db_path).unwrap();
        let fork = forks
            .create(CreateFork {
                child_session_id: SessionId(2),
                event_id: EventId(3),
                parent_session_id: SessionId(1),
                through_session_seq: 1,
                actor_principal: "owner",
                recorded_at: "2026-08-24T00:00:02Z",
            })
            .unwrap();
        assert_eq!(fork.anchor.parent_event_hash, created.event_hash);
        assert_eq!(fork.anchor.parent_session_seq, 1);
        forks.verify_all().unwrap();

        authority
            .append_event(AppendEvent {
                event_id: EventId(4),
                session_id: SessionId(1),
                expected_session_seq: 2,
                kind: EventKind::GoalVersioned,
                actor_principal: "owner",
                recorded_at: "2026-08-24T00:00:03Z",
                payload: b"parent continues",
                security_critical: true,
            })
            .unwrap();
        assert_eq!(forks.anchor(SessionId(2)).unwrap(), Some(fork.anchor));
        forks.verify_all().unwrap();

        let mutation = forks.connection.execute(
            "UPDATE sessions SET parent_session_seq = 2 WHERE session_id = ?1",
            params![id_blob(SessionId(2).0)],
        );
        assert!(mutation.is_err());
        assert_eq!(forks.anchor(SessionId(2)).unwrap(), Some(fork.anchor));

        authority.verify_integrity().unwrap();
        drop(forks);
        drop(authority);
        fs::remove_dir_all(root).unwrap();
    }
}
