#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{EffectAttemptId, EffectId, SessionId};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};

use crate::effects::StoredEffectAttempt;
use crate::storage::{AuthorityStore, StorageError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectSnapshot {
    pub effect_id: EffectId,
    pub session_id: SessionId,
    pub requested_by: String,
    pub action: String,
    pub resource: String,
    pub execution_semantics: String,
    pub idempotency_key: Option<String>,
    pub payload_hash: [u8; 32],
    pub current_state: String,
    pub latest_attempt: Option<StoredEffectAttempt>,
}

#[derive(Debug)]
pub enum EffectReadError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    InvalidStoredId,
    InvalidStoredHash,
    InvalidStoredSequence,
    InvalidStoredAttempt,
}

impl fmt::Display for EffectReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "effect reader authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "effect reader sqlite error: {error}"),
            Self::InvalidStoredId => f.write_str("stored effect identifier is malformed"),
            Self::InvalidStoredHash => f.write_str("stored effect hash is malformed"),
            Self::InvalidStoredSequence => f.write_str("stored effect sequence is invalid"),
            Self::InvalidStoredAttempt => f.write_str("stored effect attempt is malformed"),
        }
    }
}

impl Error for EffectReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for EffectReadError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for EffectReadError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub struct EffectReader {
    connection: Connection,
}

impl EffectReader {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, EffectReadError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open_with_flags(
            layout.authority_db_path(),
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA query_only = ON; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn snapshot(&self, effect_id: EffectId) -> Result<Option<EffectSnapshot>, EffectReadError> {
        let effect_blob = effect_id.0.to_be_bytes().to_vec();
        let raw = self
            .connection
            .query_row(
                "SELECT i.session_id, i.requested_by, i.action, i.resource, i.execution_semantics, \
                 i.idempotency_key, i.payload_hash, t.to_state, t.attempt_id \
                 FROM effect_intents i JOIN effect_transitions t ON t.effect_id = i.effect_id \
                 WHERE i.effect_id = ?1 AND t.global_seq = (\
                   SELECT MAX(t2.global_seq) FROM effect_transitions t2 WHERE t2.effect_id = i.effect_id\
                 )",
                params![&effect_blob],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, Vec<u8>>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, Option<Vec<u8>>>(8)?,
                    ))
                },
            )
            .optional()?;
        let Some(raw) = raw else {
            return Ok(None);
        };
        let latest_attempt = raw
            .8
            .map(|value| self.attempt(EffectAttemptId(id_from_vec(value)?)))
            .transpose()?
            .flatten();
        Ok(Some(EffectSnapshot {
            effect_id,
            session_id: SessionId(id_from_vec(raw.0)?),
            requested_by: raw.1,
            action: raw.2,
            resource: raw.3,
            execution_semantics: raw.4,
            idempotency_key: raw.5,
            payload_hash: hash_from_vec(raw.6)?,
            current_state: raw.7,
            latest_attempt,
        }))
    }

    fn attempt(
        &self,
        attempt_id: EffectAttemptId,
    ) -> Result<Option<StoredEffectAttempt>, EffectReadError> {
        let row = self
            .connection
            .query_row(
                "SELECT effect_id, started_global_seq, handler_id, handler_version, dispatch_token, \
                 started_at, finished_at, outcome, receipt FROM effect_attempts WHERE attempt_id = ?1",
                params![attempt_id.0.to_be_bytes().to_vec()],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Vec<u8>>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, Option<Vec<u8>>>(8)?,
                    ))
                },
            )
            .optional()?;
        row.map(|raw| {
            if !matches!(raw.7.as_str(), "success" | "failure" | "unknown") {
                return Err(EffectReadError::InvalidStoredAttempt);
            }
            Ok(StoredEffectAttempt {
                attempt_id,
                effect_id: EffectId(id_from_vec(raw.0)?),
                started_global_seq: seq_from_i64(raw.1)?,
                handler_id: raw.2,
                handler_version: raw.3,
                dispatch_token: raw.4,
                started_at: raw.5,
                finished_at: raw.6,
                outcome: raw.7,
                receipt: raw.8,
            })
        })
        .transpose()
    }
}

fn id_from_vec(value: Vec<u8>) -> Result<u128, EffectReadError> {
    let bytes: [u8; 16] = value
        .try_into()
        .map_err(|_| EffectReadError::InvalidStoredId)?;
    Ok(u128::from_be_bytes(bytes))
}

fn hash_from_vec(value: Vec<u8>) -> Result<[u8; 32], EffectReadError> {
    value
        .try_into()
        .map_err(|_| EffectReadError::InvalidStoredHash)
}

fn seq_from_i64(value: i64) -> Result<u64, EffectReadError> {
    u64::try_from(value).map_err(|_| EffectReadError::InvalidStoredSequence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::{EffectDispatchStore, PrepareEffectDispatch, encode_effect_dependencies};
    use crate::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
    use golam_core::paths::RuntimeLayout;
    use golam_core::{EffectTransitionId, EventId};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    fn runtime() -> RuntimeLayout {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-effect-reader-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    #[test]
    fn reads_durable_intent_state_and_latest_attempt() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let dependencies = encode_effect_dependencies(&[]).unwrap();
        let effect_id = EffectId(1);
        let mut effects = EffectStore::open(&authority).unwrap();
        effects
            .propose(ProposeEffect {
                effect_id,
                session_id: SessionId(2),
                requested_by: "owner",
                action: "sim.write",
                resource: "sim:effect:1",
                risk_class: "synthetic",
                execution_semantics: "at_most_once",
                idempotency_key: None,
                preconditions: b"[]",
                dependencies: &dependencies,
                payload_hash: [3; 32],
                proposed_event_id: EventId(4),
                transition_id: EffectTransitionId(5),
            })
            .unwrap();
        effects
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(6),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: Some("test"),
                evidence_ref: None,
                event_id: EventId(7),
            })
            .unwrap();
        drop(effects);
        let mut dispatch = EffectDispatchStore::open(&authority).unwrap();
        dispatch
            .prepare_dispatch(PrepareEffectDispatch {
                effect_id,
                attempt_id: EffectAttemptId(8),
                transition_id: EffectTransitionId(9),
                handler_id: "sim-at-most-once-write",
                handler_version: "1",
                dispatch_token: b"token",
                started_at: "2026-08-25T12:10:00Z",
                event_id: EventId(10),
            })
            .unwrap();
        drop(dispatch);

        let reader = EffectReader::open(&authority).unwrap();
        let snapshot = reader.snapshot(effect_id).unwrap().unwrap();
        assert_eq!(snapshot.session_id, SessionId(2));
        assert_eq!(snapshot.current_state, "executing");
        assert_eq!(snapshot.latest_attempt.unwrap().attempt_id, EffectAttemptId(8));
        drop(reader);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
