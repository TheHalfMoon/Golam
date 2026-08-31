#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::Path;

use golam_core::SessionId;
use golam_core::harness::{
    CompactionId, ExecutionProfileId, HardwareProfileId, RequestAttemptId, RequestSeriesId,
};
use golam_core::harness_state::{
    BenchmarkRecord, CalibrationRun, CompactionArtifact, CompactionAttempt, ModelEvent,
    ModelEventAcceptance, RequestAttempt, RequestAttemptState,
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

pub const HARNESS_EVIDENCE_SCHEMA_VERSION: i64 = 1;

pub const REQUIRED_HARNESS_TABLES: &[&str] = &[
    "harness_schema_meta",
    "harness_execution_profiles",
    "harness_hardware_profiles",
    "harness_profile_selections",
    "harness_request_attempts",
    "harness_model_events",
    "harness_compaction_attempts",
    "harness_compaction_artifacts",
    "harness_benchmark_records",
    "harness_calibration_runs",
];

#[derive(Debug)]
pub enum HarnessEvidenceError {
    Sqlite(rusqlite::Error),
    FutureSchema { found: i64, supported: i64 },
    InvalidRecord(&'static str),
    MissingAttempt(RequestAttemptId),
    ImmutableAttemptMismatch(RequestAttemptId),
    SequenceConflict {
        attempt_id: RequestAttemptId,
        sequence: u64,
    },
    IntegerOverflow,
}

impl fmt::Display for HarnessEvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "sqlite error: {error}"),
            Self::FutureSchema { found, supported } => {
                write!(f, "harness schema {found} is newer than supported {supported}")
            }
            Self::InvalidRecord(reason) => write!(f, "invalid harness evidence record: {reason}"),
            Self::MissingAttempt(attempt_id) => {
                write!(f, "request attempt not found: {attempt_id}")
            }
            Self::ImmutableAttemptMismatch(attempt_id) => {
                write!(f, "immutable request attempt identity mismatch: {attempt_id}")
            }
            Self::SequenceConflict {
                attempt_id,
                sequence,
            } => write!(
                f,
                "model event sequence conflict for request attempt {attempt_id} at {sequence}"
            ),
            Self::IntegerOverflow => f.write_str("integer conversion overflow"),
        }
    }
}

impl Error for HarnessEvidenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::FutureSchema { .. }
            | Self::InvalidRecord(_)
            | Self::MissingAttempt(_)
            | Self::ImmutableAttemptMismatch(_)
            | Self::SequenceConflict { .. }
            | Self::IntegerOverflow => None,
        }
    }
}

impl From<rusqlite::Error> for HarnessEvidenceError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub struct HarnessEvidenceStore {
    connection: Connection,
}

pub struct ExecutionProfileEvidence<'a> {
    pub profile_id: ExecutionProfileId,
    pub schema_version: u16,
    pub content_digest: [u8; 32],
    pub semantic_bytes: &'a [u8],
    pub benchmark_metadata_bytes: &'a [u8],
}

pub struct HardwareProfileEvidence<'a> {
    pub profile_id: HardwareProfileId,
    pub observed_at_unix_ms: u64,
    pub content_digest: [u8; 32],
    pub record_bytes: &'a [u8],
}

pub struct ProfileSelectionEvidence<'a> {
    pub session_id: SessionId,
    pub request_attempt_id: RequestAttemptId,
    pub profile_id: ExecutionProfileId,
    pub selected_at_unix_ms: u64,
    pub reason_bytes: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredAttemptIdentity {
    pub request_series_id: RequestSeriesId,
    pub request_attempt_id: RequestAttemptId,
    pub session_id: SessionId,
    pub initiator_principal_ref: String,
    pub execution_profile_id: ExecutionProfileId,
    pub request_digest: [u8; 32],
    pub state: RequestAttemptState,
}

impl HarnessEvidenceStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, HarnessEvidenceError> {
        let connection = Connection::open(path)?;
        Self::initialize(connection)
    }

    pub fn open_in_memory() -> Result<Self, HarnessEvidenceError> {
        let connection = Connection::open_in_memory()?;
        Self::initialize(connection)
    }

    fn initialize(connection: Connection) -> Result<Self, HarnessEvidenceError> {
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;\n\
             PRAGMA synchronous = FULL;\n\
             PRAGMA busy_timeout = 5000;",
        )?;
        migrate(&connection)?;
        verify_required_tables(&connection)?;
        Ok(Self { connection })
    }

    pub fn schema_version(&self) -> Result<i64, HarnessEvidenceError> {
        let version = self.connection.query_row(
            "SELECT schema_version FROM harness_schema_meta WHERE singleton = 1",
            [],
            |row| row.get(0),
        )?;
        Ok(version)
    }

    pub fn has_table(&self, table: &str) -> Result<bool, HarnessEvidenceError> {
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

    pub fn record_execution_profile(
        &mut self,
        evidence: ExecutionProfileEvidence<'_>,
    ) -> Result<(), HarnessEvidenceError> {
        if evidence.schema_version == 0 || evidence.semantic_bytes.is_empty() {
            return Err(HarnessEvidenceError::InvalidRecord("execution profile"));
        }
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute(
            "INSERT INTO harness_execution_profiles (\
               profile_id, schema_version, content_digest, semantic_bytes, benchmark_metadata_bytes\
             ) VALUES (?1, ?2, ?3, ?4, ?5)\
             ON CONFLICT(profile_id) DO UPDATE SET\
               benchmark_metadata_bytes = excluded.benchmark_metadata_bytes\
             WHERE harness_execution_profiles.schema_version = excluded.schema_version\
               AND harness_execution_profiles.content_digest = excluded.content_digest\
               AND harness_execution_profiles.semantic_bytes = excluded.semantic_bytes",
            params![
                id_blob(evidence.profile_id.as_u128()),
                i64::from(evidence.schema_version),
                &evidence.content_digest[..],
                evidence.semantic_bytes,
                evidence.benchmark_metadata_bytes,
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn record_hardware_profile(
        &mut self,
        evidence: HardwareProfileEvidence<'_>,
    ) -> Result<(), HarnessEvidenceError> {
        if evidence.record_bytes.is_empty() {
            return Err(HarnessEvidenceError::InvalidRecord("hardware profile"));
        }
        self.connection.execute(
            "INSERT OR IGNORE INTO harness_hardware_profiles (\
               hardware_profile_id, observed_at_unix_ms, content_digest, record_bytes\
             ) VALUES (?1, ?2, ?3, ?4)",
            params![
                id_blob(evidence.profile_id.as_u128()),
                u64_to_i64(evidence.observed_at_unix_ms)?,
                &evidence.content_digest[..],
                evidence.record_bytes,
            ],
        )?;
        Ok(())
    }

    pub fn record_profile_selection(
        &mut self,
        evidence: ProfileSelectionEvidence<'_>,
    ) -> Result<(), HarnessEvidenceError> {
        if evidence.reason_bytes.is_empty() {
            return Err(HarnessEvidenceError::InvalidRecord("profile selection reason"));
        }
        self.connection.execute(
            "INSERT INTO harness_profile_selections (\
               session_id, request_attempt_id, profile_id, selected_at_unix_ms, reason_bytes\
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id_blob(evidence.session_id.0),
                id_blob(evidence.request_attempt_id.as_u128()),
                id_blob(evidence.profile_id.as_u128()),
                u64_to_i64(evidence.selected_at_unix_ms)?,
                evidence.reason_bytes,
            ],
        )?;
        Ok(())
    }

    pub fn persist_prepared_attempt(
        &mut self,
        session_id: SessionId,
        attempt: &RequestAttempt,
        record_bytes: &[u8],
    ) -> Result<(), HarnessEvidenceError> {
        attempt
            .validate()
            .map_err(|_| HarnessEvidenceError::InvalidRecord("request attempt validation"))?;
        if attempt.state != RequestAttemptState::Prepared
            || attempt.terminal_at_unix_ms.is_some()
            || record_bytes.is_empty()
        {
            return Err(HarnessEvidenceError::InvalidRecord(
                "attempt must be PREPARED before dispatch",
            ));
        }
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute(
            "INSERT INTO harness_request_attempts (\
               request_attempt_id, request_series_id, session_id, initiator_principal_ref,\
               execution_profile_id, request_digest, state, prepared_at_unix_ms, terminal_at_unix_ms,\
               backend_instance_ref, failure_class, accepted_output_digest, record_bytes\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?9, NULL, NULL, ?10)",
            params![
                id_blob(attempt.request_attempt_id.as_u128()),
                id_blob(attempt.request_series_id.as_u128()),
                id_blob(session_id.0),
                attempt.initiator_principal_ref,
                id_blob(attempt.execution_profile_id.as_u128()),
                &attempt.request_digest[..],
                request_state_code(attempt.state),
                u64_to_i64(attempt.prepared_at_unix_ms)?,
                attempt.backend_instance_ref,
                record_bytes,
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn persist_attempt_state(
        &mut self,
        session_id: SessionId,
        attempt: &RequestAttempt,
        record_bytes: &[u8],
    ) -> Result<(), HarnessEvidenceError> {
        attempt
            .validate()
            .map_err(|_| HarnessEvidenceError::InvalidRecord("request attempt validation"))?;
        if record_bytes.is_empty() {
            return Err(HarnessEvidenceError::InvalidRecord("request attempt bytes"));
        }
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let immutable = read_attempt_identity(&transaction, attempt.request_attempt_id)?
            .ok_or(HarnessEvidenceError::MissingAttempt(attempt.request_attempt_id))?;
        if immutable.request_series_id != attempt.request_series_id
            || immutable.session_id != session_id
            || immutable.initiator_principal_ref != attempt.initiator_principal_ref
            || immutable.execution_profile_id != attempt.execution_profile_id
            || immutable.request_digest != attempt.request_digest
        {
            return Err(HarnessEvidenceError::ImmutableAttemptMismatch(
                attempt.request_attempt_id,
            ));
        }
        transaction.execute(
            "UPDATE harness_request_attempts SET\
               state = ?1, terminal_at_unix_ms = ?2, backend_instance_ref = ?3,\
               failure_class = ?4, accepted_output_digest = ?5, record_bytes = ?6\
             WHERE request_attempt_id = ?7",
            params![
                request_state_code(attempt.state),
                optional_u64_to_i64(attempt.terminal_at_unix_ms)?,
                attempt.backend_instance_ref,
                attempt.failure_class,
                attempt.accepted_output_digest.map(|digest| digest.to_vec()),
                record_bytes,
                id_blob(attempt.request_attempt_id.as_u128()),
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn append_model_event(
        &mut self,
        event: &ModelEvent,
        record_bytes: &[u8],
    ) -> Result<(), HarnessEvidenceError> {
        event
            .validate()
            .map_err(|_| HarnessEvidenceError::InvalidRecord("model event validation"))?;
        if record_bytes.is_empty() {
            return Err(HarnessEvidenceError::InvalidRecord("model event bytes"));
        }
        if event.acceptance == ModelEventAcceptance::Accepted && event.canonical_evidence_ref.is_none()
        {
            return Err(HarnessEvidenceError::InvalidRecord(
                "accepted model event requires canonical evidence reference",
            ));
        }
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if read_attempt_identity(&transaction, event.request_attempt_id)?.is_none() {
            return Err(HarnessEvidenceError::MissingAttempt(event.request_attempt_id));
        }
        let inserted = transaction.execute(
            "INSERT OR IGNORE INTO harness_model_events (\
               request_attempt_id, sequence, event_kind, acceptance, payload, canonical_evidence_ref, record_bytes\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id_blob(event.request_attempt_id.as_u128()),
                u64_to_i64(event.sequence)?,
                model_event_kind_code(event),
                model_event_acceptance_code(event.acceptance),
                event.payload,
                event.canonical_evidence_ref,
                record_bytes,
            ],
        )?;
        if inserted != 1 {
            return Err(HarnessEvidenceError::SequenceConflict {
                attempt_id: event.request_attempt_id,
                sequence: event.sequence,
            });
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn record_compaction_attempt(
        &mut self,
        attempt: &CompactionAttempt,
        record_bytes: &[u8],
    ) -> Result<(), HarnessEvidenceError> {
        attempt
            .validate()
            .map_err(|_| HarnessEvidenceError::InvalidRecord("compaction attempt validation"))?;
        if record_bytes.is_empty() {
            return Err(HarnessEvidenceError::InvalidRecord("compaction attempt bytes"));
        }
        self.connection.execute(
            "INSERT INTO harness_compaction_attempts (\
               compaction_id, session_id, state, deterministic, producing_request_attempt_id,\
               started_at_unix_ms, terminal_at_unix_ms, failure_class, record_bytes\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)\
             ON CONFLICT(compaction_id) DO UPDATE SET\
               state = excluded.state, terminal_at_unix_ms = excluded.terminal_at_unix_ms,\
               failure_class = excluded.failure_class, record_bytes = excluded.record_bytes\
             WHERE harness_compaction_attempts.session_id = excluded.session_id\
               AND harness_compaction_attempts.deterministic = excluded.deterministic\
               AND harness_compaction_attempts.producing_request_attempt_id IS excluded.producing_request_attempt_id",
            params![
                id_blob(attempt.compaction_id.as_u128()),
                id_blob(attempt.session_id.0),
                compaction_state_code(attempt),
                attempt.deterministic,
                attempt
                    .producing_request_attempt_id
                    .map(|id| id_blob(id.as_u128())),
                u64_to_i64(attempt.started_at_unix_ms)?,
                optional_u64_to_i64(attempt.terminal_at_unix_ms)?,
                attempt.failure_class,
                record_bytes,
            ],
        )?;
        Ok(())
    }

    pub fn record_compaction_artifact(
        &mut self,
        artifact: &CompactionArtifact,
        record_bytes: &[u8],
    ) -> Result<(), HarnessEvidenceError> {
        artifact
            .validate()
            .map_err(|_| HarnessEvidenceError::InvalidRecord("compaction artifact validation"))?;
        if record_bytes.is_empty() {
            return Err(HarnessEvidenceError::InvalidRecord("compaction artifact bytes"));
        }
        self.connection.execute(
            "INSERT OR IGNORE INTO harness_compaction_artifacts (\
               compaction_id, deterministic, producing_request_attempt_id, accepted_output_ref,\
               artifact_digest, record_bytes\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id_blob(artifact.compaction_id.as_u128()),
                artifact.deterministic,
                artifact
                    .producing_request_attempt_id
                    .map(|id| id_blob(id.as_u128())),
                artifact.accepted_output_ref,
                &artifact.artifact_digest[..],
                record_bytes,
            ],
        )?;
        Ok(())
    }

    pub fn record_benchmark(
        &mut self,
        record: &BenchmarkRecord,
        record_bytes: &[u8],
    ) -> Result<(), HarnessEvidenceError> {
        record
            .validate()
            .map_err(|_| HarnessEvidenceError::InvalidRecord("benchmark validation"))?;
        if record_bytes.is_empty() {
            return Err(HarnessEvidenceError::InvalidRecord("benchmark bytes"));
        }
        self.connection.execute(
            "INSERT OR IGNORE INTO harness_benchmark_records (\
               benchmark_id, execution_profile_id, hardware_profile_id, workload_fixture_id,\
               started_at_unix_ms, finished_at_unix_ms, record_bytes\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id_blob(record.benchmark_id),
                id_blob(record.execution_profile_id.as_u128()),
                id_blob(record.hardware_profile_id.as_u128()),
                record.workload_fixture_id,
                u64_to_i64(record.started_at_unix_ms)?,
                u64_to_i64(record.finished_at_unix_ms)?,
                record_bytes,
            ],
        )?;
        Ok(())
    }

    pub fn record_calibration(
        &mut self,
        run: &CalibrationRun,
        record_bytes: &[u8],
    ) -> Result<(), HarnessEvidenceError> {
        run.validate()
            .map_err(|_| HarnessEvidenceError::InvalidRecord("calibration validation"))?;
        if record_bytes.is_empty() {
            return Err(HarnessEvidenceError::InvalidRecord("calibration bytes"));
        }
        self.connection.execute(
            "INSERT OR IGNORE INTO harness_calibration_runs (\
               calibration_id, hardware_profile_id, backend_identity_ref, workload_fixture_id,\
               started_at_unix_ms, finished_at_unix_ms, record_bytes\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id_blob(run.calibration_id),
                id_blob(run.hardware_profile_id.as_u128()),
                run.backend_identity_ref,
                run.workload_fixture_id,
                u64_to_i64(run.started_at_unix_ms)?,
                optional_u64_to_i64(run.finished_at_unix_ms)?,
                record_bytes,
            ],
        )?;
        Ok(())
    }

    pub fn attempt_identity(
        &self,
        attempt_id: RequestAttemptId,
    ) -> Result<Option<StoredAttemptIdentity>, HarnessEvidenceError> {
        read_attempt_identity(&self.connection, attempt_id)
    }

    pub fn accepted_event_count(
        &self,
        attempt_id: RequestAttemptId,
    ) -> Result<u64, HarnessEvidenceError> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM harness_model_events\
             WHERE request_attempt_id = ?1 AND acceptance = ?2",
            params![
                id_blob(attempt_id.as_u128()),
                model_event_acceptance_code(ModelEventAcceptance::Accepted),
            ],
            |row| row.get(0),
        )?;
        i64_to_u64(count)
    }
}

fn migrate(connection: &Connection) -> Result<(), HarnessEvidenceError> {
    let existing = connection
        .query_row(
            "SELECT schema_version FROM harness_schema_meta WHERE singleton = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional();

    let version = match existing {
        Ok(value) => value,
        Err(error) if is_missing_table(&error) => None,
        Err(error) => return Err(error.into()),
    };

    if let Some(found) = version {
        if found > HARNESS_EVIDENCE_SCHEMA_VERSION {
            return Err(HarnessEvidenceError::FutureSchema {
                found,
                supported: HARNESS_EVIDENCE_SCHEMA_VERSION,
            });
        }
        if found == HARNESS_EVIDENCE_SCHEMA_VERSION {
            return Ok(());
        }
    }

    connection.execute_batch(
        "BEGIN IMMEDIATE;\n\
         CREATE TABLE IF NOT EXISTS harness_schema_meta (\n\
           singleton INTEGER PRIMARY KEY CHECK(singleton = 1),\n\
           schema_version INTEGER NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS harness_execution_profiles (\n\
           profile_id BLOB PRIMARY KEY NOT NULL,\n\
           schema_version INTEGER NOT NULL,\n\
           content_digest BLOB NOT NULL,\n\
           semantic_bytes BLOB NOT NULL,\n\
           benchmark_metadata_bytes BLOB NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS harness_hardware_profiles (\n\
           hardware_profile_id BLOB PRIMARY KEY NOT NULL,\n\
           observed_at_unix_ms INTEGER NOT NULL,\n\
           content_digest BLOB NOT NULL,\n\
           record_bytes BLOB NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS harness_profile_selections (\n\
           selection_seq INTEGER PRIMARY KEY AUTOINCREMENT,\n\
           session_id BLOB NOT NULL,\n\
           request_attempt_id BLOB NOT NULL,\n\
           profile_id BLOB NOT NULL,\n\
           selected_at_unix_ms INTEGER NOT NULL,\n\
           reason_bytes BLOB NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS harness_request_attempts (\n\
           request_attempt_id BLOB PRIMARY KEY NOT NULL,\n\
           request_series_id BLOB NOT NULL,\n\
           session_id BLOB NOT NULL,\n\
           initiator_principal_ref TEXT NOT NULL,\n\
           execution_profile_id BLOB NOT NULL,\n\
           request_digest BLOB NOT NULL,\n\
           state INTEGER NOT NULL,\n\
           prepared_at_unix_ms INTEGER NOT NULL,\n\
           terminal_at_unix_ms INTEGER,\n\
           backend_instance_ref TEXT,\n\
           failure_class TEXT,\n\
           accepted_output_digest BLOB,\n\
           record_bytes BLOB NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS harness_model_events (\n\
           request_attempt_id BLOB NOT NULL,\n\
           sequence INTEGER NOT NULL,\n\
           event_kind INTEGER NOT NULL,\n\
           acceptance INTEGER NOT NULL,\n\
           payload BLOB NOT NULL,\n\
           canonical_evidence_ref TEXT,\n\
           record_bytes BLOB NOT NULL,\n\
           PRIMARY KEY(request_attempt_id, sequence)\n\
         );\n\
         CREATE TABLE IF NOT EXISTS harness_compaction_attempts (\n\
           compaction_id BLOB PRIMARY KEY NOT NULL,\n\
           session_id BLOB NOT NULL,\n\
           state INTEGER NOT NULL,\n\
           deterministic INTEGER NOT NULL,\n\
           producing_request_attempt_id BLOB,\n\
           started_at_unix_ms INTEGER NOT NULL,\n\
           terminal_at_unix_ms INTEGER,\n\
           failure_class TEXT,\n\
           record_bytes BLOB NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS harness_compaction_artifacts (\n\
           compaction_id BLOB PRIMARY KEY NOT NULL,\n\
           deterministic INTEGER NOT NULL,\n\
           producing_request_attempt_id BLOB,\n\
           accepted_output_ref TEXT,\n\
           artifact_digest BLOB NOT NULL,\n\
           record_bytes BLOB NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS harness_benchmark_records (\n\
           benchmark_id BLOB PRIMARY KEY NOT NULL,\n\
           execution_profile_id BLOB NOT NULL,\n\
           hardware_profile_id BLOB NOT NULL,\n\
           workload_fixture_id TEXT NOT NULL,\n\
           started_at_unix_ms INTEGER NOT NULL,\n\
           finished_at_unix_ms INTEGER NOT NULL,\n\
           record_bytes BLOB NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS harness_calibration_runs (\n\
           calibration_id BLOB PRIMARY KEY NOT NULL,\n\
           hardware_profile_id BLOB NOT NULL,\n\
           backend_identity_ref TEXT NOT NULL,\n\
           workload_fixture_id TEXT NOT NULL,\n\
           started_at_unix_ms INTEGER NOT NULL,\n\
           finished_at_unix_ms INTEGER,\n\
           record_bytes BLOB NOT NULL\n\
         );\n\
         INSERT INTO harness_schema_meta(singleton, schema_version) VALUES (1, 1)\n\
           ON CONFLICT(singleton) DO UPDATE SET schema_version = excluded.schema_version;\n\
         COMMIT;",
    )?;
    Ok(())
}

fn verify_required_tables(connection: &Connection) -> Result<(), HarnessEvidenceError> {
    for table in REQUIRED_HARNESS_TABLES {
        let exists = connection
            .query_row(
                "SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?1 LIMIT 1",
                params![table],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;
        if exists.is_none() {
            return Err(HarnessEvidenceError::InvalidRecord(
                "required harness table missing",
            ));
        }
    }
    Ok(())
}

fn read_attempt_identity(
    connection: &Connection,
    attempt_id: RequestAttemptId,
) -> Result<Option<StoredAttemptIdentity>, HarnessEvidenceError> {
    let row = connection
        .query_row(
            "SELECT request_series_id, request_attempt_id, session_id, initiator_principal_ref,\
             execution_profile_id, request_digest, state\
             FROM harness_request_attempts WHERE request_attempt_id = ?1",
            params![id_blob(attempt_id.as_u128())],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                    row.get::<_, Vec<u8>>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            },
        )
        .optional()?;
    let Some((series, attempt, session, initiator, profile, digest, state)) = row else {
        return Ok(None);
    };
    Ok(Some(StoredAttemptIdentity {
        request_series_id: RequestSeriesId::from_u128(blob_to_u128(series)?),
        request_attempt_id: RequestAttemptId::from_u128(blob_to_u128(attempt)?),
        session_id: SessionId(blob_to_u128(session)?),
        initiator_principal_ref: initiator,
        execution_profile_id: ExecutionProfileId::from_u128(blob_to_u128(profile)?),
        request_digest: digest
            .try_into()
            .map_err(|_| HarnessEvidenceError::InvalidRecord("request digest width"))?,
        state: request_state_from_code(state)?,
    }))
}

fn is_missing_table(error: &rusqlite::Error) -> bool {
    matches!(
        error,
        rusqlite::Error::SqliteFailure(_, Some(message)) if message.contains("no such table")
    )
}

fn id_blob(value: u128) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

fn blob_to_u128(value: Vec<u8>) -> Result<u128, HarnessEvidenceError> {
    let bytes: [u8; 16] = value
        .try_into()
        .map_err(|_| HarnessEvidenceError::InvalidRecord("identifier width"))?;
    Ok(u128::from_be_bytes(bytes))
}

fn u64_to_i64(value: u64) -> Result<i64, HarnessEvidenceError> {
    i64::try_from(value).map_err(|_| HarnessEvidenceError::IntegerOverflow)
}

fn optional_u64_to_i64(value: Option<u64>) -> Result<Option<i64>, HarnessEvidenceError> {
    value.map(u64_to_i64).transpose()
}

fn i64_to_u64(value: i64) -> Result<u64, HarnessEvidenceError> {
    u64::try_from(value).map_err(|_| HarnessEvidenceError::IntegerOverflow)
}

fn request_state_code(state: RequestAttemptState) -> i64 {
    match state {
        RequestAttemptState::Prepared => 0,
        RequestAttemptState::Dispatched => 1,
        RequestAttemptState::Streaming => 2,
        RequestAttemptState::CancelRequested => 3,
        RequestAttemptState::Completed => 4,
        RequestAttemptState::Cancelled => 5,
        RequestAttemptState::TimedOut => 6,
        RequestAttemptState::FailedTransient => 7,
        RequestAttemptState::FailedDeterministic => 8,
        RequestAttemptState::FailedContextOverflow => 9,
    }
}

fn request_state_from_code(code: i64) -> Result<RequestAttemptState, HarnessEvidenceError> {
    match code {
        0 => Ok(RequestAttemptState::Prepared),
        1 => Ok(RequestAttemptState::Dispatched),
        2 => Ok(RequestAttemptState::Streaming),
        3 => Ok(RequestAttemptState::CancelRequested),
        4 => Ok(RequestAttemptState::Completed),
        5 => Ok(RequestAttemptState::Cancelled),
        6 => Ok(RequestAttemptState::TimedOut),
        7 => Ok(RequestAttemptState::FailedTransient),
        8 => Ok(RequestAttemptState::FailedDeterministic),
        9 => Ok(RequestAttemptState::FailedContextOverflow),
        _ => Err(HarnessEvidenceError::InvalidRecord("request state code")),
    }
}

fn model_event_kind_code(event: &ModelEvent) -> i64 {
    use golam_core::harness_state::ModelEventKind;
    match event.kind {
        ModelEventKind::TextDelta => 0,
        ModelEventKind::ReasoningDelta => 1,
        ModelEventKind::ToolCallFragment => 2,
        ModelEventKind::ToolCallComplete => 3,
        ModelEventKind::Usage => 4,
        ModelEventKind::Stop => 5,
        ModelEventKind::BackendWarning => 6,
        ModelEventKind::BackendError => 7,
    }
}

fn model_event_acceptance_code(acceptance: ModelEventAcceptance) -> i64 {
    match acceptance {
        ModelEventAcceptance::Accepted => 0,
        ModelEventAcceptance::RejectedMalformed => 1,
        ModelEventAcceptance::RejectedOversized => 2,
        ModelEventAcceptance::RejectedOutOfOrder => 3,
        ModelEventAcceptance::RejectedAfterTerminal => 4,
    }
}

fn compaction_state_code(attempt: &CompactionAttempt) -> i64 {
    use golam_core::harness_state::CompactionState;
    match attempt.state {
        CompactionState::Started => 0,
        CompactionState::Deriving => 1,
        CompactionState::Validating => 2,
        CompactionState::Committed => 3,
        CompactionState::Cancelled => 4,
        CompactionState::FailedChangedSource => 5,
        CompactionState::FailedTransient => 6,
        CompactionState::FailedDeterministic => 7,
        CompactionState::FailedPersistence => 8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::harness_state::{ModelEventKind, RequestAttemptState};

    fn prepared_attempt() -> RequestAttempt {
        RequestAttempt {
            request_series_id: RequestSeriesId::from_u128(1),
            request_attempt_id: RequestAttemptId::from_u128(2),
            initiator_principal_ref: "principal:owner".into(),
            state: RequestAttemptState::Prepared,
            execution_profile_id: ExecutionProfileId::from_u128(3),
            request_digest: [4; 32],
            backend_instance_ref: None,
            accepted_event_refs: Vec::new(),
            accepted_output_digest: None,
            failure_class: None,
            prepared_at_unix_ms: 10,
            terminal_at_unix_ms: None,
        }
    }

    #[test]
    fn schema_is_forward_only_and_complete() {
        let store = HarnessEvidenceStore::open_in_memory().unwrap();
        assert_eq!(store.schema_version().unwrap(), HARNESS_EVIDENCE_SCHEMA_VERSION);
        for table in REQUIRED_HARNESS_TABLES {
            assert!(store.has_table(table).unwrap(), "missing {table}");
        }
    }

    #[test]
    fn prepared_attempt_must_exist_before_stream_event() {
        let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
        let event = ModelEvent {
            request_attempt_id: RequestAttemptId::from_u128(2),
            sequence: 0,
            kind: ModelEventKind::TextDelta,
            payload: b"hello".to_vec(),
            acceptance: ModelEventAcceptance::Accepted,
            canonical_evidence_ref: Some("event:model:1".into()),
        };
        assert!(matches!(
            store.append_model_event(&event, b"event-record"),
            Err(HarnessEvidenceError::MissingAttempt(_))
        ));

        let attempt = prepared_attempt();
        store
            .persist_prepared_attempt(SessionId(9), &attempt, b"prepared")
            .unwrap();
        store.append_model_event(&event, b"event-record").unwrap();
        assert_eq!(store.accepted_event_count(attempt.request_attempt_id).unwrap(), 1);
    }

    #[test]
    fn immutable_attempt_identity_cannot_change_with_state() {
        let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
        let mut attempt = prepared_attempt();
        store
            .persist_prepared_attempt(SessionId(9), &attempt, b"prepared")
            .unwrap();
        attempt.state = RequestAttemptState::Dispatched;
        attempt.backend_instance_ref = Some("scripted:1".into());
        store
            .persist_attempt_state(SessionId(9), &attempt, b"dispatched")
            .unwrap();

        attempt.execution_profile_id = ExecutionProfileId::from_u128(999);
        assert!(matches!(
            store.persist_attempt_state(SessionId(9), &attempt, b"invalid"),
            Err(HarnessEvidenceError::ImmutableAttemptMismatch(_))
        ));
    }

    #[test]
    fn duplicate_event_sequence_is_rejected() {
        let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
        let attempt = prepared_attempt();
        store
            .persist_prepared_attempt(SessionId(9), &attempt, b"prepared")
            .unwrap();
        let event = ModelEvent {
            request_attempt_id: attempt.request_attempt_id,
            sequence: 0,
            kind: ModelEventKind::TextDelta,
            payload: b"hello".to_vec(),
            acceptance: ModelEventAcceptance::Accepted,
            canonical_evidence_ref: Some("event:model:1".into()),
        };
        store.append_model_event(&event, b"one").unwrap();
        assert!(matches!(
            store.append_model_event(&event, b"two"),
            Err(HarnessEvidenceError::SequenceConflict { .. })
        ));
    }
}
