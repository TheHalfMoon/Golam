#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{CheckpointId, SessionId};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};

use crate::storage::{AuthorityStore, StorageError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionSummary {
    pub session_id: SessionId,
    pub owner_principal: String,
    pub status: String,
    pub latest_session_seq: u64,
    pub latest_event_hash: [u8; 32],
    pub parent_session_id: Option<SessionId>,
    pub parent_session_seq: Option<u64>,
    pub parent_event_hash: Option<[u8; 32]>,
    pub latest_checkpoint_id: Option<CheckpointId>,
}

#[derive(Debug)]
pub enum SessionReadError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    InvalidStoredId,
    InvalidStoredHash,
    InvalidStoredSequence,
    InvalidForkAnchor,
}

impl fmt::Display for SessionReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "session reader authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "session reader sqlite error: {error}"),
            Self::InvalidStoredId => f.write_str("stored session identifier is malformed"),
            Self::InvalidStoredHash => f.write_str("stored session hash is malformed"),
            Self::InvalidStoredSequence => f.write_str("stored session sequence is invalid"),
            Self::InvalidForkAnchor => f.write_str("stored session fork anchor is incomplete"),
        }
    }
}

impl Error for SessionReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::InvalidStoredId
            | Self::InvalidStoredHash
            | Self::InvalidStoredSequence
            | Self::InvalidForkAnchor => None,
        }
    }
}

impl From<StorageError> for SessionReadError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for SessionReadError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub struct SessionReader {
    connection: Connection,
}

impl SessionReader {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, SessionReadError> {
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

    pub fn list(&self) -> Result<Vec<SessionSummary>, SessionReadError> {
        let mut statement = self.connection.prepare(
            "SELECT session_id, owner_principal, status, latest_session_seq, latest_event_hash, \
             parent_session_id, parent_session_seq, parent_event_hash, latest_checkpoint_id \
             FROM sessions ORDER BY created_global_seq ASC, session_id ASC",
        )?;
        let rows = statement.query_map([], raw_session)?;
        rows.map(|row| parse_session(row?)).collect()
    }

    pub fn get(&self, session_id: SessionId) -> Result<Option<SessionSummary>, SessionReadError> {
        let raw = self
            .connection
            .query_row(
                "SELECT session_id, owner_principal, status, latest_session_seq, latest_event_hash, \
                 parent_session_id, parent_session_seq, parent_event_hash, latest_checkpoint_id \
                 FROM sessions WHERE session_id = ?1",
                params![session_id.0.to_be_bytes().to_vec()],
                raw_session,
            )
            .optional()?;
        raw.map(parse_session).transpose()
    }
}

type RawSession = (
    Vec<u8>,
    String,
    String,
    i64,
    Vec<u8>,
    Option<Vec<u8>>,
    Option<i64>,
    Option<Vec<u8>>,
    Option<Vec<u8>>,
);

fn raw_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawSession> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
        row.get(7)?,
        row.get(8)?,
    ))
}

fn parse_session(raw: RawSession) -> Result<SessionSummary, SessionReadError> {
    let parent = match (raw.5, raw.6, raw.7) {
        (None, None, None) => (None, None, None),
        (Some(id), Some(seq), Some(hash)) => (
            Some(SessionId(id_from_blob(id)?)),
            Some(seq_from_i64(seq)?),
            Some(hash_from_vec(hash)?),
        ),
        _ => return Err(SessionReadError::InvalidForkAnchor),
    };
    Ok(SessionSummary {
        session_id: SessionId(id_from_blob(raw.0)?),
        owner_principal: raw.1,
        status: raw.2,
        latest_session_seq: seq_from_i64(raw.3)?,
        latest_event_hash: hash_from_vec(raw.4)?,
        parent_session_id: parent.0,
        parent_session_seq: parent.1,
        parent_event_hash: parent.2,
        latest_checkpoint_id: raw
            .8
            .map(id_from_blob)
            .transpose()?
            .map(CheckpointId),
    })
}

fn id_from_blob(value: Vec<u8>) -> Result<u128, SessionReadError> {
    let bytes: [u8; 16] = value
        .try_into()
        .map_err(|_| SessionReadError::InvalidStoredId)?;
    Ok(u128::from_be_bytes(bytes))
}

fn hash_from_vec(value: Vec<u8>) -> Result<[u8; 32], SessionReadError> {
    value
        .try_into()
        .map_err(|_| SessionReadError::InvalidStoredHash)
}

fn seq_from_i64(value: i64) -> Result<u64, SessionReadError> {
    u64::try_from(value).map_err(|_| SessionReadError::InvalidStoredSequence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fork::{CreateFork, ForkManager};
    use crate::storage::CreateSession;
    use golam_core::paths::RuntimeLayout;
    use golam_core::EventId;
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
            "golam-session-reader-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    #[test]
    fn lists_roots_and_forks_without_mutation() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let mut store = AuthorityStore::open(authority.authority_db_path()).unwrap();
        store
            .create_session(CreateSession {
                session_id: SessionId(1),
                event_id: EventId(2),
                owner_principal: "owner",
                actor_principal: "owner",
                recorded_at: "2026-08-25T11:30:00Z",
                payload: b"root",
                security_critical: true,
            })
            .unwrap();
        drop(store);
        let mut forks = ForkManager::open(authority.authority_db_path()).unwrap();
        forks
            .create(CreateFork {
                child_session_id: SessionId(3),
                event_id: EventId(4),
                parent_session_id: SessionId(1),
                through_session_seq: 1,
                actor_principal: "owner",
                recorded_at: "2026-08-25T11:31:00Z",
            })
            .unwrap();
        drop(forks);

        let reader = SessionReader::open(&authority).unwrap();
        let sessions = reader.list().unwrap();
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].session_id, SessionId(1));
        let child = reader.get(SessionId(3)).unwrap().unwrap();
        assert_eq!(child.parent_session_id, Some(SessionId(1)));
        assert_eq!(child.parent_session_seq, Some(1));
        assert!(reader.get(SessionId(999)).unwrap().is_none());
        drop(reader);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
