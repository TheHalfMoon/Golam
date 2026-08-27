#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::Path;

use golam_core::{EventId, SCHEMA_VERSION, SessionId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::{EventKind, EventRecord, audit_integrity_hash, event_integrity_hash, payload_hash};

pub const AUTHORITY_SCHEMA_VERSION: i64 = 3;
const SECURITY_AUDIT_CHAIN: &str = "security";

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
    "principal_records",
    "policy_bundles",
    "active_policy",
    "capability_leases",
    "capability_revocations",
    "approvals",
    "approval_consumptions",
    "taint_attestations",
    "verifier_rules",
    "secret_records",
    "secret_versions",
    "secret_handles",
    "secret_use_records",
    "egress_permits",
    "sandbox_profiles",
    "sandbox_admissions",
    "authority_security_audit_v2",
];

#[derive(Debug)]
pub enum StorageError {
    Sqlite(rusqlite::Error),
    Core(golam_core::CoreError),
    FutureSchema { found: i64, supported: i64 },
    IntegrityCheckFailed(String),
    SessionAlreadyExists(SessionId),
    SessionNotFound(SessionId),
    StaleSessionHead { expected: u64, actual: u64 },
    SequenceOverflow,
    InvalidStoredHash,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "sqlite error: {error}"),
            Self::Core(error) => write!(f, "core encoding error: {error}"),
            Self::FutureSchema { found, supported } => {
                write!(
                    f,
                    "authority schema {found} is newer than supported {supported}"
                )
            }
            Self::IntegrityCheckFailed(result) => {
                write!(f, "authority database integrity check failed: {result}")
            }
            Self::SessionAlreadyExists(session_id) => {
                write!(f, "session already exists: {}", session_id.0)
            }
            Self::SessionNotFound(session_id) => {
                write!(f, "session not found: {}", session_id.0)
            }
            Self::StaleSessionHead { expected, actual } => {
                write!(
                    f,
                    "stale session head: expected {expected}, actual {actual}"
                )
            }
            Self::SequenceOverflow => f.write_str("canonical sequence overflow"),
            Self::InvalidStoredHash => f.write_str("stored integrity hash is not 32 bytes"),
        }
    }
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::FutureSchema { .. }
            | Self::IntegrityCheckFailed(_)
            | Self::SessionAlreadyExists(_)
            | Self::SessionNotFound(_)
            | Self::StaleSessionHead { .. }
            | Self::SequenceOverflow
            | Self::InvalidStoredHash => None,
        }
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<golam_core::CoreError> for StorageError {
    fn from(value: golam_core::CoreError) -> Self {
        Self::Core(value)
    }
}

pub struct CreateSession<'a> {
    pub session_id: SessionId,
    pub event_id: EventId,
    pub owner_principal: &'a str,
    pub actor_principal: &'a str,
    pub recorded_at: &'a str,
    pub payload: &'a [u8],
    pub security_critical: bool,
}

pub struct AppendEvent<'a> {
    pub event_id: EventId,
    pub session_id: SessionId,
    pub expected_session_seq: u64,
    pub kind: EventKind,
    pub actor_principal: &'a str,
    pub recorded_at: &'a str,
    pub payload: &'a [u8],
    pub security_critical: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredEvent {
    pub record: EventRecord,
    pub event_hash: [u8; 32],
    pub audit_hash: Option<[u8; 32]>,
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
        verify_canonical_integrity(&connection)?;
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
        verify_quick_check(&self.connection)?;
        verify_canonical_integrity(&self.connection)
    }

    pub fn create_session(
        &mut self,
        input: CreateSession<'_>,
    ) -> Result<StoredEvent, StorageError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let session_blob = id_blob(input.session_id.0);
        let exists = transaction
            .query_row(
                "SELECT 1 FROM sessions WHERE session_id = ?1 LIMIT 1",
                params![&session_blob],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;
        if exists.is_some() {
            return Err(StorageError::SessionAlreadyExists(input.session_id));
        }

        let global_seq = next_global_seq(&transaction)?;
        let previous_audit_hash = if input.security_critical {
            audit_head(&transaction)?
        } else {
            None
        };
        let record = EventRecord {
            event_id: input.event_id,
            session_id: input.session_id,
            global_seq,
            session_seq: 1,
            schema_version: SCHEMA_VERSION,
            kind: EventKind::SessionCreated,
            actor_principal: input.actor_principal.to_owned(),
            recorded_at: input.recorded_at.to_owned(),
            payload_hash: payload_hash(input.payload),
            previous_session_event_hash: None,
            security_critical: input.security_critical,
            previous_audit_hash,
        };
        let stored = build_stored_event(record)?;

        transaction.execute(
            "INSERT INTO sessions (session_id, owner_principal, created_global_seq, status, \
             latest_session_seq, latest_event_hash) VALUES (?1, ?2, ?3, 'active', 1, ?4)",
            params![
                &session_blob,
                input.owner_principal,
                seq_to_i64(global_seq)?,
                &stored.event_hash[..]
            ],
        )?;
        insert_event(&transaction, &stored, input.payload)?;
        update_audit_head(&transaction, &stored)?;
        transaction.commit()?;
        Ok(stored)
    }

    pub fn append_event(&mut self, input: AppendEvent<'_>) -> Result<StoredEvent, StorageError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let session_blob = id_blob(input.session_id.0);
        let head = transaction
            .query_row(
                "SELECT latest_session_seq, latest_event_hash FROM sessions WHERE session_id = ?1",
                params![&session_blob],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
            )
            .optional()?
            .ok_or(StorageError::SessionNotFound(input.session_id))?;
        let actual_session_seq = i64_to_seq(head.0)?;
        if actual_session_seq != input.expected_session_seq {
            return Err(StorageError::StaleSessionHead {
                expected: input.expected_session_seq,
                actual: actual_session_seq,
            });
        }

        let previous_session_event_hash = Some(hash_from_vec(head.1)?);
        let session_seq = actual_session_seq
            .checked_add(1)
            .ok_or(StorageError::SequenceOverflow)?;
        let global_seq = next_global_seq(&transaction)?;
        let previous_audit_hash = if input.security_critical {
            audit_head(&transaction)?
        } else {
            None
        };
        let record = EventRecord {
            event_id: input.event_id,
            session_id: input.session_id,
            global_seq,
            session_seq,
            schema_version: SCHEMA_VERSION,
            kind: input.kind,
            actor_principal: input.actor_principal.to_owned(),
            recorded_at: input.recorded_at.to_owned(),
            payload_hash: payload_hash(input.payload),
            previous_session_event_hash,
            security_critical: input.security_critical,
            previous_audit_hash,
        };
        let stored = build_stored_event(record)?;

        insert_event(&transaction, &stored, input.payload)?;
        let updated = transaction.execute(
            "UPDATE sessions SET latest_session_seq = ?1, latest_event_hash = ?2 \
             WHERE session_id = ?3 AND latest_session_seq = ?4",
            params![
                seq_to_i64(session_seq)?,
                &stored.event_hash[..],
                &session_blob,
                seq_to_i64(input.expected_session_seq)?
            ],
        )?;
        if updated != 1 {
            return Err(StorageError::StaleSessionHead {
                expected: input.expected_session_seq,
                actual: actual_session_seq,
            });
        }
        update_audit_head(&transaction, &stored)?;
        transaction.commit()?;
        Ok(stored)
    }
}

fn verify_canonical_integrity(connection: &Connection) -> Result<(), StorageError> {
    crate::integrity::verify(connection)
        .map_err(|error| StorageError::IntegrityCheckFailed(error.to_string()))?;
    crate::authority_security_v2::verify(connection)
        .map_err(|error| StorageError::IntegrityCheckFailed(error.to_string()))
}

fn build_stored_event(record: EventRecord) -> Result<StoredEvent, StorageError> {
    let event_hash = event_integrity_hash(&record)?;
    let audit_hash = if record.security_critical {
        Some(audit_integrity_hash(&record, event_hash)?)
    } else {
        None
    };
    Ok(StoredEvent {
        record,
        event_hash,
        audit_hash,
    })
}

fn insert_event(
    transaction: &Transaction<'_>,
    stored: &StoredEvent,
    payload: &[u8],
) -> Result<(), StorageError> {
    let record = &stored.record;
    let previous_session_hash = record.previous_session_event_hash.map(|hash| hash.to_vec());
    let previous_audit_hash = record.previous_audit_hash.map(|hash| hash.to_vec());
    let audit_hash = stored.audit_hash.map(|hash| hash.to_vec());
    transaction.execute(
        "INSERT INTO session_events (event_id, global_seq, session_id, session_seq, event_type, \
         schema_version, actor_principal, recorded_at, payload_bytes, payload_hash, \
         previous_session_event_hash, event_hash, security_critical, previous_audit_hash, audit_hash) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
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
            previous_session_hash,
            &stored.event_hash[..],
            record.security_critical,
            previous_audit_hash,
            audit_hash,
        ],
    )?;
    Ok(())
}

fn next_global_seq(transaction: &Transaction<'_>) -> Result<u64, StorageError> {
    let current: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(global_seq), 0) FROM (\
           SELECT global_seq FROM session_events \
           UNION ALL SELECT global_seq FROM effect_transitions \
           UNION ALL SELECT global_seq FROM authorization_decisions\
         )",
        [],
        |row| row.get(0),
    )?;
    i64_to_seq(current)?
        .checked_add(1)
        .ok_or(StorageError::SequenceOverflow)
}

fn audit_head(transaction: &Transaction<'_>) -> Result<Option<[u8; 32]>, StorageError> {
    let value = transaction
        .query_row(
            "SELECT last_hash FROM audit_chain_heads WHERE chain_name = ?1",
            params![SECURITY_AUDIT_CHAIN],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .optional()?;
    value.map(hash_from_vec).transpose()
}

fn update_audit_head(
    transaction: &Transaction<'_>,
    stored: &StoredEvent,
) -> Result<(), StorageError> {
    let Some(audit_hash) = stored.audit_hash else {
        return Ok(());
    };
    transaction.execute(
        "INSERT INTO audit_chain_heads (chain_name, last_global_seq, last_hash) VALUES (?1, ?2, ?3) \
         ON CONFLICT(chain_name) DO UPDATE SET last_global_seq = excluded.last_global_seq, \
         last_hash = excluded.last_hash",
        params![
            SECURITY_AUDIT_CHAIN,
            seq_to_i64(stored.record.global_seq)?,
            &audit_hash[..]
        ],
    )?;
    Ok(())
}

fn id_blob(value: u128) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

fn seq_to_i64(value: u64) -> Result<i64, StorageError> {
    i64::try_from(value).map_err(|_| StorageError::SequenceOverflow)
}

fn i64_to_seq(value: i64) -> Result<u64, StorageError> {
    u64::try_from(value).map_err(|_| StorageError::SequenceOverflow)
}

fn hash_from_vec(value: Vec<u8>) -> Result<[u8; 32], StorageError> {
    value
        .try_into()
        .map_err(|_| StorageError::InvalidStoredHash)
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
    let mut version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version > AUTHORITY_SCHEMA_VERSION {
        return Err(StorageError::FutureSchema {
            found: version,
            supported: AUTHORITY_SCHEMA_VERSION,
        });
    }
    if version == 0 {
        migrate_v1(connection)?;
        version = 1;
    }
    if version == 1 {
        migrate_v2(connection)?;
        version = 2;
    }
    if version == 2 {
        migrate_v3(connection)?;
        version = 3;
    }
    debug_assert_eq!(version, AUTHORITY_SCHEMA_VERSION);
    Ok(())
}

fn migrate_v1(connection: &Connection) -> Result<(), StorageError> {
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

fn migrate_v2(connection: &Connection) -> Result<(), StorageError> {
    connection.execute_batch(
        "BEGIN IMMEDIATE;\n\
         CREATE TABLE principal_records (\n\
           principal_id TEXT PRIMARY KEY NOT NULL,\n\
           principal_kind TEXT NOT NULL,\n\
           owner_principal TEXT,\n\
           status TEXT NOT NULL,\n\
           attributes_version INTEGER NOT NULL,\n\
           created_global_seq INTEGER NOT NULL,\n\
           revoked_at TEXT\n\
         );\n\
         CREATE TABLE policy_bundles (\n\
           policy_bundle_id BLOB PRIMARY KEY NOT NULL,\n\
           version INTEGER NOT NULL,\n\
           schema_version INTEGER NOT NULL,\n\
           canonical_policy_bytes BLOB NOT NULL,\n\
           bundle_hash BLOB NOT NULL UNIQUE,\n\
           created_by TEXT NOT NULL,\n\
           created_global_seq INTEGER NOT NULL,\n\
           validation_status TEXT NOT NULL\n\
         );\n\
         CREATE TABLE active_policy (\n\
           singleton_id INTEGER PRIMARY KEY NOT NULL CHECK(singleton_id = 1),\n\
           policy_bundle_id BLOB NOT NULL,\n\
           bundle_hash BLOB NOT NULL,\n\
           activated_by TEXT NOT NULL,\n\
           activation_effect_id BLOB NOT NULL,\n\
           activated_global_seq INTEGER NOT NULL\n\
         );\n\
         CREATE TABLE capability_leases (\n\
           lease_id BLOB PRIMARY KEY NOT NULL,\n\
           principal_id TEXT NOT NULL,\n\
           parent_lease_id BLOB,\n\
           actions_scope BLOB NOT NULL,\n\
           resources_scope BLOB NOT NULL,\n\
           context_constraints BLOB NOT NULL,\n\
           issued_by TEXT NOT NULL,\n\
           issued_global_seq INTEGER NOT NULL,\n\
           not_before TEXT,\n\
           expires_at TEXT,\n\
           generation INTEGER NOT NULL,\n\
           status TEXT NOT NULL,\n\
           authority_digest BLOB NOT NULL\n\
         );\n\
         CREATE INDEX capability_leases_principal_status_idx ON capability_leases(principal_id, status);\n\
         CREATE TABLE capability_revocations (\n\
           revocation_id BLOB PRIMARY KEY NOT NULL,\n\
           lease_id BLOB NOT NULL,\n\
           revoked_by TEXT NOT NULL,\n\
           reason_code TEXT NOT NULL,\n\
           revoked_global_seq INTEGER NOT NULL,\n\
           revoked_at TEXT NOT NULL\n\
         );\n\
         CREATE INDEX capability_revocations_lease_idx ON capability_revocations(lease_id);\n\
         CREATE TABLE approvals (\n\
           approval_id BLOB PRIMARY KEY NOT NULL,\n\
           class TEXT NOT NULL,\n\
           approver_principal TEXT NOT NULL,\n\
           scope_digest BLOB NOT NULL,\n\
           action_scope BLOB NOT NULL,\n\
           resource_scope BLOB NOT NULL,\n\
           effect_id BLOB,\n\
           session_id BLOB,\n\
           risk_class TEXT NOT NULL,\n\
           taint_digest BLOB NOT NULL,\n\
           parent_decision_id BLOB NOT NULL,\n\
           issued_at TEXT NOT NULL,\n\
           expires_at TEXT,\n\
           max_uses INTEGER,\n\
           revoked_at TEXT\n\
         );\n\
         CREATE TABLE approval_consumptions (\n\
           consumption_id BLOB PRIMARY KEY NOT NULL,\n\
           approval_id BLOB NOT NULL,\n\
           effect_or_operation_id BLOB NOT NULL,\n\
           reserved_global_seq INTEGER NOT NULL,\n\
           consumed_global_seq INTEGER,\n\
           state TEXT NOT NULL\n\
         );\n\
         CREATE INDEX approval_consumptions_approval_idx ON approval_consumptions(approval_id);\n\
         CREATE TABLE taint_attestations (\n\
           attestation_id BLOB PRIMARY KEY NOT NULL,\n\
           source_artifact_ids BLOB NOT NULL,\n\
           source_labels BLOB NOT NULL,\n\
           result_artifact_id BLOB NOT NULL,\n\
           result_labels BLOB NOT NULL,\n\
           mechanism TEXT NOT NULL,\n\
           rule_id BLOB NOT NULL,\n\
           principal TEXT,\n\
           evidence_hash BLOB NOT NULL,\n\
           created_global_seq INTEGER NOT NULL\n\
         );\n\
         CREATE TABLE verifier_rules (\n\
           rule_id BLOB PRIMARY KEY NOT NULL,\n\
           kind TEXT NOT NULL,\n\
           version INTEGER NOT NULL,\n\
           authority_source_binding BLOB NOT NULL,\n\
           allowed_downgrades BLOB NOT NULL,\n\
           registered_by TEXT NOT NULL,\n\
           status TEXT NOT NULL,\n\
           created_global_seq INTEGER NOT NULL\n\
         );\n\
         CREATE TABLE secret_records (\n\
           secret_id BLOB PRIMARY KEY NOT NULL,\n\
           classification TEXT NOT NULL,\n\
           owner_principal TEXT NOT NULL,\n\
           current_version INTEGER NOT NULL,\n\
           status TEXT NOT NULL,\n\
           created_global_seq INTEGER NOT NULL,\n\
           revoked_at TEXT\n\
         );\n\
         CREATE TABLE secret_versions (\n\
           secret_id BLOB NOT NULL,\n\
           version INTEGER NOT NULL,\n\
           ciphertext BLOB NOT NULL,\n\
           nonce_or_algorithm_metadata BLOB NOT NULL,\n\
           associated_data_hash BLOB NOT NULL,\n\
           created_global_seq INTEGER NOT NULL,\n\
           rotated_from INTEGER,\n\
           retired_at TEXT,\n\
           PRIMARY KEY(secret_id, version)\n\
         );\n\
         CREATE TABLE secret_handles (\n\
           handle_id BLOB PRIMARY KEY NOT NULL,\n\
           secret_id BLOB NOT NULL,\n\
           version_constraint INTEGER,\n\
           purpose_scope BLOB NOT NULL,\n\
           expires_at TEXT\n\
         );\n\
         CREATE TABLE secret_use_records (\n\
           use_id BLOB PRIMARY KEY NOT NULL,\n\
           handle_id BLOB NOT NULL,\n\
           principal TEXT NOT NULL,\n\
           purpose TEXT NOT NULL,\n\
           destination_or_process TEXT NOT NULL,\n\
           mode TEXT NOT NULL,\n\
           approval_id BLOB,\n\
           decision_id BLOB NOT NULL,\n\
           created_global_seq INTEGER NOT NULL\n\
         );\n\
         CREATE TABLE egress_permits (\n\
           permit_id BLOB PRIMARY KEY NOT NULL,\n\
           principal_or_process TEXT NOT NULL,\n\
           action TEXT NOT NULL,\n\
           purpose TEXT NOT NULL,\n\
           destination_scope BLOB NOT NULL,\n\
           protocol_port_scope BLOB NOT NULL,\n\
           taint_digest BLOB NOT NULL,\n\
           secret_handle_id BLOB,\n\
           parent_lease_id BLOB NOT NULL,\n\
           issued_at TEXT NOT NULL,\n\
           expires_at TEXT,\n\
           usage_limit INTEGER,\n\
           status TEXT NOT NULL\n\
         );\n\
         CREATE TABLE sandbox_profiles (\n\
           profile_id BLOB NOT NULL,\n\
           version INTEGER NOT NULL,\n\
           class TEXT NOT NULL,\n\
           filesystem_read_roots BLOB NOT NULL,\n\
           filesystem_write_roots BLOB NOT NULL,\n\
           network_rule BLOB NOT NULL,\n\
           environment_allowlist BLOB NOT NULL,\n\
           spawn_rule BLOB NOT NULL,\n\
           cpu_limit INTEGER,\n\
           memory_limit INTEGER,\n\
           time_limit INTEGER,\n\
           output_limit INTEGER,\n\
           device_allowlist BLOB NOT NULL,\n\
           ipc_allowlist BLOB NOT NULL,\n\
           inherited_handle_rules BLOB NOT NULL,\n\
           platform_requirements BLOB NOT NULL,\n\
           status TEXT NOT NULL,\n\
           PRIMARY KEY(profile_id, version)\n\
         );\n\
         CREATE TABLE sandbox_admissions (\n\
           admission_id BLOB PRIMARY KEY NOT NULL,\n\
           profile_id BLOB NOT NULL,\n\
           profile_version INTEGER NOT NULL,\n\
           principal_or_process TEXT NOT NULL,\n\
           lease_id BLOB NOT NULL,\n\
           decision_id BLOB NOT NULL,\n\
           egress_permit_id BLOB,\n\
           resolved_launch_plan_hash BLOB NOT NULL,\n\
           platform_executor TEXT NOT NULL,\n\
           created_global_seq INTEGER NOT NULL\n\
         );\n\
         ALTER TABLE authorization_decisions ADD COLUMN hard_guard_result TEXT NOT NULL DEFAULT 'spec002_legacy';\n\
         ALTER TABLE authorization_decisions ADD COLUMN lease_id BLOB;\n\
         ALTER TABLE authorization_decisions ADD COLUMN lease_generation INTEGER;\n\
         ALTER TABLE authorization_decisions ADD COLUMN policy_bundle_id BLOB;\n\
         ALTER TABLE authorization_decisions ADD COLUMN policy_bundle_hash BLOB;\n\
         ALTER TABLE authorization_decisions ADD COLUMN matched_rule_ids BLOB NOT NULL DEFAULT X'';\n\
         ALTER TABLE authorization_decisions ADD COLUMN approval_id BLOB;\n\
         PRAGMA user_version = 2;\n\
         COMMIT;",
    )?;
    Ok(())
}

fn migrate_v3(connection: &Connection) -> Result<(), StorageError> {
    connection.execute_batch(
        "BEGIN IMMEDIATE;\n\
         CREATE TABLE authority_security_audit_v2 (\n\
           audit_seq INTEGER PRIMARY KEY NOT NULL,\n\
           record_kind TEXT NOT NULL,\n\
           record_id BLOB NOT NULL,\n\
           payload_bytes BLOB NOT NULL,\n\
           payload_hash BLOB NOT NULL,\n\
           previous_hash BLOB,\n\
           record_hash BLOB NOT NULL\n\
         );\n\
         CREATE INDEX authority_security_audit_v2_record_idx \
           ON authority_security_audit_v2(record_kind, record_id, audit_seq);\n\
         ALTER TABLE authorization_decisions \
           ADD COLUMN authority_evidence_version INTEGER NOT NULL DEFAULT 1;\n\
         PRAGMA user_version = 3;\n\
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

    fn has_column(connection: &Connection, table: &str, column: &str) -> bool {
        let query = format!("SELECT COUNT(*) FROM pragma_table_info('{table}') WHERE name = ?1");
        let count: i64 = connection
            .query_row(&query, params![column], |row| row.get(0))
            .unwrap();
        count == 1
    }

    #[test]
    fn migration_creates_required_authority_tables() {
        let store = AuthorityStore::open_in_memory().unwrap();
        assert_eq!(store.schema_version().unwrap(), AUTHORITY_SCHEMA_VERSION);
        for table in REQUIRED_TABLES {
            assert!(store.has_table(table).unwrap(), "missing table {table}");
        }
        for column in [
            "hard_guard_result",
            "lease_id",
            "lease_generation",
            "policy_bundle_id",
            "policy_bundle_hash",
            "matched_rule_ids",
            "approval_id",
            "authority_evidence_version",
        ] {
            assert!(
                has_column(&store.connection, "authorization_decisions", column),
                "missing authorization v2 column {column}"
            );
        }
        store.verify_integrity().unwrap();
    }

    #[test]
    fn v1_database_migrates_forward_without_rewriting_existing_rows() {
        let connection = Connection::open_in_memory().unwrap();
        configure_connection(&connection).unwrap();
        migrate_v1(&connection).unwrap();
        assert_eq!(
            connection
                .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            1
        );
        connection
            .execute(
                "INSERT INTO clients \
                 (client_id, key_id, public_key, kind, owner_principal, enrolled_at, revoked_at, assurance_class) \
                 VALUES (?1, 'legacy-key', ?2, 'local', 'owner', '2026-08-24T00:00:00Z', NULL, 'owner')",
                params![vec![7_u8; 16], vec![9_u8; 32]],
            )
            .unwrap();

        migrate(&connection).unwrap();
        assert_eq!(
            connection
                .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            AUTHORITY_SCHEMA_VERSION
        );
        let key_id: String = connection
            .query_row(
                "SELECT key_id FROM clients WHERE client_id = ?1",
                params![vec![7_u8; 16]],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(key_id, "legacy-key");
        let legacy_guard: String = connection
            .query_row(
                "SELECT dflt_value FROM pragma_table_info('authorization_decisions') WHERE name = 'hard_guard_result'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(legacy_guard, "'spec002_legacy'");
        let legacy_evidence_version: String = connection
            .query_row(
                "SELECT dflt_value FROM pragma_table_info('authorization_decisions') WHERE name = 'authority_evidence_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(legacy_evidence_version, "1");
    }

    #[test]
    fn future_authority_schema_still_fails_closed() {
        let connection = Connection::open_in_memory().unwrap();
        configure_connection(&connection).unwrap();
        connection.execute_batch("PRAGMA user_version = 99;").unwrap();
        assert!(matches!(
            migrate(&connection),
            Err(StorageError::FutureSchema {
                found: 99,
                supported: AUTHORITY_SCHEMA_VERSION
            })
        ));
    }

    #[test]
    fn session_creation_and_append_are_transactionally_ordered() {
        let mut store = AuthorityStore::open_in_memory().unwrap();
        let created = store
            .create_session(CreateSession {
                session_id: SessionId(1),
                event_id: EventId(10),
                owner_principal: "owner",
                actor_principal: "owner",
                recorded_at: "2026-08-24T00:00:00Z",
                payload: b"create",
                security_critical: true,
            })
            .unwrap();
        assert_eq!(created.record.global_seq, 1);
        assert_eq!(created.record.session_seq, 1);
        assert_eq!(created.record.previous_session_event_hash, None);
        assert_eq!(created.record.previous_audit_hash, None);
        assert!(created.audit_hash.is_some());

        let appended = store
            .append_event(AppendEvent {
                event_id: EventId(11),
                session_id: SessionId(1),
                expected_session_seq: 1,
                kind: EventKind::GoalVersioned,
                actor_principal: "owner",
                recorded_at: "2026-08-24T00:00:01Z",
                payload: b"goal",
                security_critical: true,
            })
            .unwrap();
        assert_eq!(appended.record.global_seq, 2);
        assert_eq!(appended.record.session_seq, 2);
        assert_eq!(
            appended.record.previous_session_event_hash,
            Some(created.event_hash)
        );
        assert_eq!(appended.record.previous_audit_hash, created.audit_hash);
        store.verify_integrity().unwrap();
    }

    #[test]
    fn session_allocator_advances_past_authorization_decisions() {
        let mut store = AuthorityStore::open_in_memory().unwrap();
        store
            .connection
            .execute(
                "INSERT INTO authorization_decisions \
                 (decision_id, principal, action, resource, context_hash, decision, reason_code, global_seq) \
                 VALUES (?1, ?2, ?3, ?4, ?5, 'allow', 'test', 1)",
                params![
                    vec![1_u8; 16],
                    "owner:owner",
                    "session.create",
                    "session:new",
                    vec![0_u8; 32]
                ],
            )
            .unwrap();

        let created = store
            .create_session(CreateSession {
                session_id: SessionId(99),
                event_id: EventId(990),
                owner_principal: "owner",
                actor_principal: "owner",
                recorded_at: "2026-08-25T07:58:00Z",
                payload: b"create-after-auth",
                security_critical: true,
            })
            .unwrap();
        assert_eq!(created.record.global_seq, 2);
    }

    #[test]
    fn stale_session_head_does_not_consume_global_sequence() {
        let mut store = AuthorityStore::open_in_memory().unwrap();
        store
            .create_session(CreateSession {
                session_id: SessionId(2),
                event_id: EventId(20),
                owner_principal: "owner",
                actor_principal: "owner",
                recorded_at: "2026-08-24T00:00:00Z",
                payload: b"create",
                security_critical: true,
            })
            .unwrap();

        let stale = store.append_event(AppendEvent {
            event_id: EventId(21),
            session_id: SessionId(2),
            expected_session_seq: 0,
            kind: EventKind::GoalVersioned,
            actor_principal: "owner",
            recorded_at: "2026-08-24T00:00:01Z",
            payload: b"stale",
            security_critical: true,
        });
        assert!(matches!(
            stale,
            Err(StorageError::StaleSessionHead {
                expected: 0,
                actual: 1
            })
        ));

        let appended = store
            .append_event(AppendEvent {
                event_id: EventId(22),
                session_id: SessionId(2),
                expected_session_seq: 1,
                kind: EventKind::GoalVersioned,
                actor_principal: "owner",
                recorded_at: "2026-08-24T00:00:02Z",
                payload: b"valid",
                security_critical: true,
            })
            .unwrap();
        assert_eq!(appended.record.global_seq, 2);
    }

    #[test]
    fn canonical_integrity_detects_payload_tampering() {
        let mut store = AuthorityStore::open_in_memory().unwrap();
        store
            .create_session(CreateSession {
                session_id: SessionId(3),
                event_id: EventId(30),
                owner_principal: "owner",
                actor_principal: "owner",
                recorded_at: "2026-08-24T00:00:00Z",
                payload: b"original",
                security_critical: true,
            })
            .unwrap();
        store
            .connection
            .execute(
                "UPDATE session_events SET payload_bytes = ?1 WHERE global_seq = 1",
                params![b"tampered".as_slice()],
            )
            .unwrap();
        assert!(matches!(
            store.verify_integrity(),
            Err(StorageError::IntegrityCheckFailed(_))
        ));
    }
}
