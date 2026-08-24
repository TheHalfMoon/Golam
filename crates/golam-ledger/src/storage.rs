#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

pub const AUTHORITY_SCHEMA_VERSION: i64 = 1;

pub const REQUIRED_TABLES: &[&str] = &[
    "clients",
    "sessions",
    "session_events",
    "goal_versions",
    "artifacts",
    "checkpoints",
    "effect_intents",
    "effect_transitions",
    "effect_attempts",
    "authorization_decisions",
    "audit_chain_heads",
    "recovery_incidents",
];

#[derive(Debug)]
pub enum StorageError {
    Sqlite(rusqlite::Error),
    FutureSchema { found: i64, supported: i64 },
    IntegrityCheckFailed(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "sqlite error: {error}"),
            Self::FutureSchema { found, supported } => {
                write!(
                    f,
                    "authority schema {found} is newer than supported {supported}"
                )
            }
            Self::IntegrityCheckFailed(result) => {
                write!(f, "authority database integrity check failed: {result}")
            }
        }
    }
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::FutureSchema { .. } | Self::IntegrityCheckFailed(_) => None,
        }
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub struct AuthorityStore {
    connection: Connection,
}

impl AuthorityStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let connection = Connection::open(path)?;
        Self::initialize(connection)
    }

    pub fn open_in_memory() -> Result<Self, StorageError> {
        let connection = Connection::open_in_memory()?;
        Self::initialize(connection)
    }

    fn initialize(connection: Connection) -> Result<Self, StorageError> {
        configure_connection(&connection)?;
        migrate(&connection)?;
        verify_quick_check(&connection)?;
        Ok(Self { connection })
    }

    pub fn schema_version(&self) -> Result<i64, StorageError> {
        Ok(self
            .connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))?)
    }

    pub fn has_table(&self, table: &str) -> Result<bool, StorageError> {
        let exists = self
            .connection
            .query_row(
                "SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?1 LIMIT 1",
                params![table],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;
        Ok(exists.is_some())
    }

    pub fn verify_integrity(&self) -> Result<(), StorageError> {
        verify_quick_check(&self.connection)
    }
}

fn configure_connection(connection: &Connection) -> Result<(), StorageError> {
    connection.execute_batch(
        "PRAGMA foreign_keys = ON;\n\
         PRAGMA journal_mode = WAL;\n\
         PRAGMA synchronous = FULL;\n\
         PRAGMA busy_timeout = 5000;",
    )?;
    Ok(())
}

fn migrate(connection: &Connection) -> Result<(), StorageError> {
    let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version > AUTHORITY_SCHEMA_VERSION {
        return Err(StorageError::FutureSchema {
            found: version,
            supported: AUTHORITY_SCHEMA_VERSION,
        });
    }
    if version == AUTHORITY_SCHEMA_VERSION {
        return Ok(());
    }

    connection.execute_batch(
        "BEGIN IMMEDIATE;\n\
         CREATE TABLE clients (\n\
           client_id BLOB PRIMARY KEY NOT NULL,\n\
           key_id TEXT NOT NULL UNIQUE,\n\
           public_key BLOB NOT NULL,\n\
           kind TEXT NOT NULL,\n\
           owner_principal TEXT NOT NULL,\n\
           enrolled_at TEXT NOT NULL,\n\
           last_authenticated_at TEXT,\n\
           revoked_at TEXT,\n\
           assurance_class TEXT NOT NULL\n\
         );\n\
         CREATE TABLE sessions (\n\
           session_id BLOB PRIMARY KEY NOT NULL,\n\
           owner_principal TEXT NOT NULL,\n\
           created_global_seq INTEGER NOT NULL,\n\
           status TEXT NOT NULL,\n\
           parent_session_id BLOB,\n\
           parent_session_seq INTEGER,\n\
           parent_event_hash BLOB,\n\
           latest_session_seq INTEGER NOT NULL,\n\
           latest_event_hash BLOB NOT NULL,\n\
           latest_checkpoint_id BLOB\n\
         );\n\
         CREATE TABLE session_events (\n\
           event_id BLOB NOT NULL UNIQUE,\n\
           global_seq INTEGER PRIMARY KEY,\n\
           session_id BLOB NOT NULL,\n\
           session_seq INTEGER NOT NULL,\n\
           event_type INTEGER NOT NULL,\n\
           schema_version INTEGER NOT NULL,\n\
           actor_principal TEXT NOT NULL,\n\
           recorded_at TEXT NOT NULL,\n\
           payload_bytes BLOB NOT NULL,\n\
           payload_hash BLOB NOT NULL,\n\
           previous_session_event_hash BLOB,\n\
           event_hash BLOB NOT NULL,\n\
           security_critical INTEGER NOT NULL,\n\
           previous_audit_hash BLOB,\n\
           audit_hash BLOB,\n\
           UNIQUE(session_id, session_seq)\n\
         );\n\
         CREATE TABLE goal_versions (\n\
           goal_version_id BLOB PRIMARY KEY NOT NULL,\n\
           goal_id BLOB NOT NULL,\n\
           session_id BLOB NOT NULL,\n\
           version INTEGER NOT NULL,\n\
           payload_bytes BLOB NOT NULL,\n\
           created_event_id BLOB NOT NULL,\n\
           created_global_seq INTEGER NOT NULL,\n\
           UNIQUE(goal_id, version)\n\
         );\n\
         CREATE TABLE artifacts (\n\
           artifact_hash BLOB PRIMARY KEY NOT NULL,\n\
           size_bytes INTEGER NOT NULL,\n\
           media_type TEXT NOT NULL,\n\
           created_global_seq INTEGER NOT NULL,\n\
           retention_class TEXT NOT NULL,\n\
           relative_path TEXT NOT NULL UNIQUE\n\
         );\n\
         CREATE TABLE checkpoints (\n\
           checkpoint_id BLOB PRIMARY KEY NOT NULL,\n\
           session_id BLOB NOT NULL,\n\
           through_session_seq INTEGER NOT NULL,\n\
           through_global_seq INTEGER NOT NULL,\n\
           through_event_hash BLOB NOT NULL,\n\
           projection_schema_version INTEGER NOT NULL,\n\
           artifact_hash BLOB NOT NULL,\n\
           created_event_id BLOB NOT NULL,\n\
           verified_at TEXT\n\
         );\n\
         CREATE TABLE effect_intents (\n\
           effect_id BLOB PRIMARY KEY NOT NULL,\n\
           session_id BLOB NOT NULL,\n\
           requested_by TEXT NOT NULL,\n\
           action TEXT NOT NULL,\n\
           resource TEXT NOT NULL,\n\
           risk_class TEXT NOT NULL,\n\
           execution_semantics TEXT NOT NULL,\n\
           idempotency_key TEXT,\n\
           preconditions BLOB NOT NULL,\n\
           dependencies BLOB NOT NULL,\n\
           payload_hash BLOB NOT NULL,\n\
           proposed_event_id BLOB NOT NULL\n\
         );\n\
         CREATE TABLE effect_transitions (\n\
           transition_id BLOB PRIMARY KEY NOT NULL,\n\
           effect_id BLOB NOT NULL,\n\
           global_seq INTEGER NOT NULL UNIQUE,\n\
           from_state TEXT,\n\
           to_state TEXT NOT NULL,\n\
           attempt_id BLOB,\n\
           reason_code TEXT,\n\
           evidence_ref BLOB,\n\
           event_id BLOB NOT NULL\n\
         );\n\
         CREATE TABLE effect_attempts (\n\
           attempt_id BLOB PRIMARY KEY NOT NULL,\n\
           effect_id BLOB NOT NULL,\n\
           started_global_seq INTEGER NOT NULL,\n\
           handler_id TEXT NOT NULL,\n\
           handler_version TEXT NOT NULL,\n\
           dispatch_token BLOB NOT NULL,\n\
           started_at TEXT NOT NULL,\n\
           finished_at TEXT,\n\
           outcome TEXT NOT NULL,\n\
           receipt BLOB\n\
         );\n\
         CREATE TABLE authorization_decisions (\n\
           decision_id BLOB PRIMARY KEY NOT NULL,\n\
           principal TEXT NOT NULL,\n\
           action TEXT NOT NULL,\n\
           resource TEXT NOT NULL,\n\
           context_hash BLOB NOT NULL,\n\
           decision TEXT NOT NULL,\n\
           reason_code TEXT NOT NULL,\n\
           global_seq INTEGER NOT NULL UNIQUE\n\
         );\n\
         CREATE TABLE audit_chain_heads (\n\
           chain_name TEXT PRIMARY KEY NOT NULL,\n\
           last_global_seq INTEGER NOT NULL,\n\
           last_hash BLOB NOT NULL\n\
         );\n\
         CREATE TABLE recovery_incidents (\n\
           incident_id BLOB PRIMARY KEY NOT NULL,\n\
           detected_at TEXT NOT NULL,\n\
           kind TEXT NOT NULL,\n\
           severity TEXT NOT NULL,\n\
           affected_refs BLOB NOT NULL,\n\
           recovery_mode TEXT NOT NULL,\n\
           resolution BLOB\n\
         );\n\
         PRAGMA user_version = 1;\n\
         COMMIT;",
    )?;
    Ok(())
}

fn verify_quick_check(connection: &Connection) -> Result<(), StorageError> {
    let result: String = connection.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if result == "ok" {
        Ok(())
    } else {
        Err(StorageError::IntegrityCheckFailed(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_creates_required_authority_tables() {
        let store = AuthorityStore::open_in_memory().unwrap();
        assert_eq!(store.schema_version().unwrap(), AUTHORITY_SCHEMA_VERSION);
        for table in REQUIRED_TABLES {
            assert!(store.has_table(table).unwrap(), "missing table {table}");
        }
        store.verify_integrity().unwrap();
    }
}
