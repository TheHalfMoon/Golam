#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::Path;

use golam_core::SessionId;
use golam_core::harness::{
    ExecutionProfileId, HardwareProfileId, RequestAttemptId, RequestSeriesId,
};
use golam_core::harness_state::{
    BenchmarkRecord, CalibrationRun, CompactionArtifact, CompactionAttempt, ModelEvent,
    ModelEventAcceptance, ModelEventKind, RequestAttempt, RequestAttemptState,
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
    FutureSchema {
        found: i64,
        supported: i64,
    },
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
                write!(
                    f,
                    "harness schema {found} is newer than supported {supported}"
                )
            }
            Self::InvalidRecord(reason) => write!(f, "invalid harness evidence record: {reason}"),
            Self::MissingAttempt(attempt_id) => {
                write!(f, "request attempt not found: {attempt_id}")
            }
            Self::ImmutableAttemptMismatch(attempt_id) => {
                write!(
                    f,
                    "immutable request attempt identity mismatch: {attempt_id}"
                )
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
            _ => None,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredAcceptedModelEvent {
    pub sequence: u64,
    pub kind: ModelEventKind,
    pub payload: Vec<u8>,
    pub canonical_evidence_ref: String,
}

impl HarnessEvidenceStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, HarnessEvidenceError> {
        Self::initialize(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self, HarnessEvidenceError> {
        Self::initialize(Connection::open_in_memory()?)
    }

    fn initialize(connection: Connection) -> Result<Self, HarnessEvidenceError> {
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        migrate(&connection)?;
        verify_required_tables(&connection)?;
        Ok(Self { connection })
    }

    pub fn schema_version(&self) -> Result<i64, HarnessEvidenceError> {
        Ok(self.connection.query_row(
            "SELECT schema_version FROM harness_schema_meta WHERE singleton = 1",
            [],
            |row| row.get(0),
        )?)
    }

    pub fn has_table(&self, table: &str) -> Result<bool, HarnessEvidenceError> {
        Ok(self
            .connection
            .query_row(
                "SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?1 LIMIT 1",
                params![table],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some())
    }

    pub fn record_execution_profile(
        &mut self,
        evidence: ExecutionProfileEvidence<'_>,
    ) -> Result<(), HarnessEvidenceError> {
        if evidence.schema_version == 0 || evidence.semantic_bytes.is_empty() {
            return Err(HarnessEvidenceError::InvalidRecord("execution profile"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed = tx.execute(
            r#"INSERT INTO harness_execution_profiles
               (profile_id, schema_version, content_digest, semantic_bytes, benchmark_metadata_bytes)
               VALUES (?1, ?2, ?3, ?4, ?5)
               ON CONFLICT(profile_id) DO UPDATE SET
                 benchmark_metadata_bytes = excluded.benchmark_metadata_bytes
               WHERE harness_execution_profiles.schema_version = excluded.schema_version
                 AND harness_execution_profiles.content_digest = excluded.content_digest
                 AND harness_execution_profiles.semantic_bytes = excluded.semantic_bytes"#,
            params![
                id_blob(evidence.profile_id.as_u128()),
                i64::from(evidence.schema_version),
                &evidence.content_digest[..],
                evidence.semantic_bytes,
                evidence.benchmark_metadata_bytes,
            ],
        )?;
        if changed == 0 {
            return Err(HarnessEvidenceError::InvalidRecord(
                "execution profile identity collision",
            ));
        }
        tx.commit()?;
        Ok(())
    }

    pub fn record_hardware_profile(
        &mut self,
        evidence: HardwareProfileEvidence<'_>,
    ) -> Result<(), HarnessEvidenceError> {
        if evidence.record_bytes.is_empty() {
            return Err(HarnessEvidenceError::InvalidRecord("hardware profile"));
        }
        let changed = self.connection.execute(
            r#"INSERT INTO harness_hardware_profiles
               (hardware_profile_id, observed_at_unix_ms, content_digest, record_bytes)
               VALUES (?1, ?2, ?3, ?4)
               ON CONFLICT(hardware_profile_id) DO UPDATE SET
                 hardware_profile_id = excluded.hardware_profile_id
               WHERE harness_hardware_profiles.observed_at_unix_ms = excluded.observed_at_unix_ms
                 AND harness_hardware_profiles.content_digest = excluded.content_digest
                 AND harness_hardware_profiles.record_bytes = excluded.record_bytes"#,
            params![
                id_blob(evidence.profile_id.as_u128()),
                u64_to_i64(evidence.observed_at_unix_ms)?,
                &evidence.content_digest[..],
                evidence.record_bytes,
            ],
        )?;
        if changed != 1 {
            return Err(HarnessEvidenceError::InvalidRecord(
                "hardware profile identity collision",
            ));
        }
        Ok(())
    }

    pub fn record_profile_selection(
        &mut self,
        evidence: ProfileSelectionEvidence<'_>,
    ) -> Result<(), HarnessEvidenceError> {
        if evidence.reason_bytes.is_empty() {
            return Err(HarnessEvidenceError::InvalidRecord(
                "profile selection reason",
            ));
        }
        self.connection.execute(
            r#"INSERT INTO harness_profile_selections
               (session_id, request_attempt_id, profile_id, selected_at_unix_ms, reason_bytes)
               VALUES (?1, ?2, ?3, ?4, ?5)"#,
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
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(selected) = selected_profile_id(&tx, attempt.request_attempt_id)?
            && selected != attempt.execution_profile_id
        {
            return Err(HarnessEvidenceError::ImmutableAttemptMismatch(
                attempt.request_attempt_id,
            ));
        }
        tx.execute(
            r#"INSERT INTO harness_request_attempts
               (request_attempt_id, request_series_id, session_id, initiator_principal_ref,
                execution_profile_id, request_digest, state, prepared_at_unix_ms,
                terminal_at_unix_ms, backend_instance_ref, failure_class,
                accepted_output_digest, record_bytes)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?9, NULL, NULL, ?10)"#,
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
        tx.commit()?;
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
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let immutable = read_attempt_identity(&tx, attempt.request_attempt_id)?.ok_or(
            HarnessEvidenceError::MissingAttempt(attempt.request_attempt_id),
        )?;
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
        tx.execute(
            r#"UPDATE harness_request_attempts SET
               state = ?1, terminal_at_unix_ms = ?2, backend_instance_ref = ?3,
               failure_class = ?4, accepted_output_digest = ?5, record_bytes = ?6
               WHERE request_attempt_id = ?7"#,
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
        tx.commit()?;
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
        if event.acceptance == ModelEventAcceptance::Accepted
            && event.canonical_evidence_ref.is_none()
        {
            return Err(HarnessEvidenceError::InvalidRecord(
                "accepted model event requires canonical evidence reference",
            ));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if read_attempt_identity(&tx, event.request_attempt_id)?.is_none() {
            return Err(HarnessEvidenceError::MissingAttempt(
                event.request_attempt_id,
            ));
        }
        let inserted = tx.execute(
            r#"INSERT OR IGNORE INTO harness_model_events
               (request_attempt_id, sequence, event_kind, acceptance, payload,
                canonical_evidence_ref, record_bytes)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![
                id_blob(event.request_attempt_id.as_u128()),
                u64_to_i64(event.sequence)?,
                model_event_kind_code(event.kind),
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
        tx.commit()?;
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
            return Err(HarnessEvidenceError::InvalidRecord(
                "compaction attempt bytes",
            ));
        }
        if attempt
            .terminal_at_unix_ms
            .is_some_and(|terminal| terminal < attempt.started_at_unix_ms)
        {
            return Err(HarnessEvidenceError::InvalidRecord(
                "compaction terminal precedes start",
            ));
        }

        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let next_state = compaction_state_code(attempt);
        let existing = tx
            .query_row(
                r#"SELECT state, session_id, deterministic, producing_request_attempt_id,
                          started_at_unix_ms
                   FROM harness_compaction_attempts
                   WHERE compaction_id = ?1"#,
                params![id_blob(attempt.compaction_id.as_u128())],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, Vec<u8>>(1)?,
                        row.get::<_, bool>(2)?,
                        row.get::<_, Option<Vec<u8>>>(3)?,
                        row.get::<_, i64>(4)?,
                    ))
                },
            )
            .optional()?;

        match existing {
            None if next_state != 0 => {
                return Err(HarnessEvidenceError::InvalidRecord(
                    "compaction lifecycle must begin with STARTED",
                ));
            }
            Some((current_state, session_id, deterministic, producing_attempt_id, started_at)) => {
                let expected_producing_attempt = attempt
                    .producing_request_attempt_id
                    .map(|id| id_blob(id.as_u128()));
                if session_id != id_blob(attempt.session_id.0)
                    || deterministic != attempt.deterministic
                    || producing_attempt_id != expected_producing_attempt
                    || started_at != u64_to_i64(attempt.started_at_unix_ms)?
                {
                    return Err(HarnessEvidenceError::InvalidRecord(
                        "immutable compaction identity mismatch",
                    ));
                }
                if !compaction_state_transition_allowed(current_state, next_state) {
                    return Err(HarnessEvidenceError::InvalidRecord(
                        "invalid durable compaction state transition",
                    ));
                }
            }
            None => {}
        }

        let changed = tx.execute(
            r#"INSERT INTO harness_compaction_attempts
               (compaction_id, session_id, state, deterministic, producing_request_attempt_id,
                started_at_unix_ms, terminal_at_unix_ms, failure_class, record_bytes)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
               ON CONFLICT(compaction_id) DO UPDATE SET
                 state = excluded.state,
                 terminal_at_unix_ms = excluded.terminal_at_unix_ms,
                 failure_class = excluded.failure_class,
                 record_bytes = excluded.record_bytes
               WHERE harness_compaction_attempts.session_id = excluded.session_id
                 AND harness_compaction_attempts.deterministic = excluded.deterministic
                 AND harness_compaction_attempts.producing_request_attempt_id
                     IS excluded.producing_request_attempt_id
                 AND harness_compaction_attempts.started_at_unix_ms = excluded.started_at_unix_ms"#,
            params![
                id_blob(attempt.compaction_id.as_u128()),
                id_blob(attempt.session_id.0),
                next_state,
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
        if changed != 1 {
            return Err(HarnessEvidenceError::InvalidRecord(
                "compaction lifecycle persistence conflict",
            ));
        }
        tx.commit()?;
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
            return Err(HarnessEvidenceError::InvalidRecord(
                "compaction artifact bytes",
            ));
        }
        let changed = self.connection.execute(
            r#"INSERT INTO harness_compaction_artifacts
               (compaction_id, deterministic, producing_request_attempt_id,
                accepted_output_ref, artifact_digest, record_bytes)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6)
               ON CONFLICT(compaction_id) DO UPDATE SET
                 compaction_id = excluded.compaction_id
               WHERE harness_compaction_artifacts.deterministic = excluded.deterministic
                 AND harness_compaction_artifacts.producing_request_attempt_id
                     IS excluded.producing_request_attempt_id
                 AND harness_compaction_artifacts.accepted_output_ref IS excluded.accepted_output_ref
                 AND harness_compaction_artifacts.artifact_digest = excluded.artifact_digest
                 AND harness_compaction_artifacts.record_bytes = excluded.record_bytes"#,
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
        if changed != 1 {
            return Err(HarnessEvidenceError::InvalidRecord(
                "compaction artifact identity collision",
            ));
        }
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
        let changed = self.connection.execute(
            r#"INSERT INTO harness_benchmark_records
               (benchmark_id, execution_profile_id, hardware_profile_id, workload_fixture_id,
                started_at_unix_ms, finished_at_unix_ms, record_bytes)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
               ON CONFLICT(benchmark_id) DO UPDATE SET
                 benchmark_id = excluded.benchmark_id
               WHERE harness_benchmark_records.execution_profile_id = excluded.execution_profile_id
                 AND harness_benchmark_records.hardware_profile_id = excluded.hardware_profile_id
                 AND harness_benchmark_records.workload_fixture_id = excluded.workload_fixture_id
                 AND harness_benchmark_records.started_at_unix_ms = excluded.started_at_unix_ms
                 AND harness_benchmark_records.finished_at_unix_ms = excluded.finished_at_unix_ms
                 AND harness_benchmark_records.record_bytes = excluded.record_bytes"#,
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
        if changed != 1 {
            return Err(HarnessEvidenceError::InvalidRecord(
                "benchmark identity collision",
            ));
        }
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
        let changed = self.connection.execute(
            r#"INSERT INTO harness_calibration_runs
               (calibration_id, hardware_profile_id, backend_identity_ref, workload_fixture_id,
                started_at_unix_ms, finished_at_unix_ms, record_bytes)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
               ON CONFLICT(calibration_id) DO UPDATE SET
                 calibration_id = excluded.calibration_id
               WHERE harness_calibration_runs.hardware_profile_id = excluded.hardware_profile_id
                 AND harness_calibration_runs.backend_identity_ref = excluded.backend_identity_ref
                 AND harness_calibration_runs.workload_fixture_id = excluded.workload_fixture_id
                 AND harness_calibration_runs.started_at_unix_ms = excluded.started_at_unix_ms
                 AND harness_calibration_runs.finished_at_unix_ms IS excluded.finished_at_unix_ms
                 AND harness_calibration_runs.record_bytes = excluded.record_bytes"#,
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
        if changed != 1 {
            return Err(HarnessEvidenceError::InvalidRecord(
                "calibration identity collision",
            ));
        }
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
            "SELECT COUNT(*) FROM harness_model_events WHERE request_attempt_id = ?1 AND acceptance = ?2",
            params![
                id_blob(attempt_id.as_u128()),
                model_event_acceptance_code(ModelEventAcceptance::Accepted),
            ],
            |row| row.get(0),
        )?;
        i64_to_u64(count)
    }

    pub fn accepted_events(
        &self,
        attempt_id: RequestAttemptId,
    ) -> Result<Vec<StoredAcceptedModelEvent>, HarnessEvidenceError> {
        let mut statement = self.connection.prepare(
            r#"SELECT sequence, event_kind, payload, canonical_evidence_ref
               FROM harness_model_events
               WHERE request_attempt_id = ?1 AND acceptance = ?2
               ORDER BY sequence ASC"#,
        )?;
        let rows = statement.query_map(
            params![
                id_blob(attempt_id.as_u128()),
                model_event_acceptance_code(ModelEventAcceptance::Accepted),
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            },
        )?;
        let mut accepted = Vec::new();
        for row in rows {
            let (sequence, kind, payload, evidence_ref) = row?;
            accepted.push(StoredAcceptedModelEvent {
                sequence: i64_to_u64(sequence)?,
                kind: model_event_kind_from_code(kind)?,
                payload,
                canonical_evidence_ref: evidence_ref.ok_or(HarnessEvidenceError::InvalidRecord(
                    "accepted event missing canonical evidence reference",
                ))?,
            });
        }
        Ok(accepted)
    }
}

fn migrate(connection: &Connection) -> Result<(), HarnessEvidenceError> {
    let version = match connection
        .query_row(
            "SELECT schema_version FROM harness_schema_meta WHERE singleton = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()
    {
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
        r#"BEGIN IMMEDIATE;
        CREATE TABLE IF NOT EXISTS harness_schema_meta (
          singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
          schema_version INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS harness_execution_profiles (
          profile_id BLOB PRIMARY KEY NOT NULL,
          schema_version INTEGER NOT NULL,
          content_digest BLOB NOT NULL,
          semantic_bytes BLOB NOT NULL,
          benchmark_metadata_bytes BLOB NOT NULL
        );
        CREATE TABLE IF NOT EXISTS harness_hardware_profiles (
          hardware_profile_id BLOB PRIMARY KEY NOT NULL,
          observed_at_unix_ms INTEGER NOT NULL,
          content_digest BLOB NOT NULL,
          record_bytes BLOB NOT NULL
        );
        CREATE TABLE IF NOT EXISTS harness_profile_selections (
          selection_seq INTEGER PRIMARY KEY AUTOINCREMENT,
          session_id BLOB NOT NULL,
          request_attempt_id BLOB NOT NULL,
          profile_id BLOB NOT NULL,
          selected_at_unix_ms INTEGER NOT NULL,
          reason_bytes BLOB NOT NULL
        );
        CREATE TABLE IF NOT EXISTS harness_request_attempts (
          request_attempt_id BLOB PRIMARY KEY NOT NULL,
          request_series_id BLOB NOT NULL,
          session_id BLOB NOT NULL,
          initiator_principal_ref TEXT NOT NULL,
          execution_profile_id BLOB NOT NULL,
          request_digest BLOB NOT NULL,
          state INTEGER NOT NULL,
          prepared_at_unix_ms INTEGER NOT NULL,
          terminal_at_unix_ms INTEGER,
          backend_instance_ref TEXT,
          failure_class TEXT,
          accepted_output_digest BLOB,
          record_bytes BLOB NOT NULL
        );
        CREATE TABLE IF NOT EXISTS harness_model_events (
          request_attempt_id BLOB NOT NULL,
          sequence INTEGER NOT NULL,
          event_kind INTEGER NOT NULL,
          acceptance INTEGER NOT NULL,
          payload BLOB NOT NULL,
          canonical_evidence_ref TEXT,
          record_bytes BLOB NOT NULL,
          PRIMARY KEY(request_attempt_id, sequence)
        );
        CREATE TABLE IF NOT EXISTS harness_compaction_attempts (
          compaction_id BLOB PRIMARY KEY NOT NULL,
          session_id BLOB NOT NULL,
          state INTEGER NOT NULL,
          deterministic INTEGER NOT NULL,
          producing_request_attempt_id BLOB,
          started_at_unix_ms INTEGER NOT NULL,
          terminal_at_unix_ms INTEGER,
          failure_class TEXT,
          record_bytes BLOB NOT NULL
        );
        CREATE TABLE IF NOT EXISTS harness_compaction_artifacts (
          compaction_id BLOB PRIMARY KEY NOT NULL,
          deterministic INTEGER NOT NULL,
          producing_request_attempt_id BLOB,
          accepted_output_ref TEXT,
          artifact_digest BLOB NOT NULL,
          record_bytes BLOB NOT NULL
        );
        CREATE TABLE IF NOT EXISTS harness_benchmark_records (
          benchmark_id BLOB PRIMARY KEY NOT NULL,
          execution_profile_id BLOB NOT NULL,
          hardware_profile_id BLOB NOT NULL,
          workload_fixture_id TEXT NOT NULL,
          started_at_unix_ms INTEGER NOT NULL,
          finished_at_unix_ms INTEGER NOT NULL,
          record_bytes BLOB NOT NULL
        );
        CREATE TABLE IF NOT EXISTS harness_calibration_runs (
          calibration_id BLOB PRIMARY KEY NOT NULL,
          hardware_profile_id BLOB NOT NULL,
          backend_identity_ref TEXT NOT NULL,
          workload_fixture_id TEXT NOT NULL,
          started_at_unix_ms INTEGER NOT NULL,
          finished_at_unix_ms INTEGER,
          record_bytes BLOB NOT NULL
        );
        INSERT INTO harness_schema_meta(singleton, schema_version) VALUES (1, 1)
          ON CONFLICT(singleton) DO UPDATE SET schema_version = excluded.schema_version;
        COMMIT;"#,
    )?;
    Ok(())
}

fn verify_required_tables(connection: &Connection) -> Result<(), HarnessEvidenceError> {
    for table in REQUIRED_HARNESS_TABLES {
        if connection
            .query_row(
                "SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?1 LIMIT 1",
                params![table],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_none()
        {
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
            "SELECT request_series_id, request_attempt_id, session_id, initiator_principal_ref, execution_profile_id, request_digest, state FROM harness_request_attempts WHERE request_attempt_id = ?1",
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

fn selected_profile_id(
    connection: &Connection,
    attempt_id: RequestAttemptId,
) -> Result<Option<ExecutionProfileId>, HarnessEvidenceError> {
    let value = connection
        .query_row(
            "SELECT profile_id FROM harness_profile_selections WHERE request_attempt_id = ?1 ORDER BY selection_seq DESC LIMIT 1",
            params![id_blob(attempt_id.as_u128())],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .optional()?;
    value
        .map(blob_to_u128)
        .transpose()
        .map(|id| id.map(ExecutionProfileId::from_u128))
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

fn model_event_kind_code(kind: ModelEventKind) -> i64 {
    match kind {
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

fn model_event_kind_from_code(code: i64) -> Result<ModelEventKind, HarnessEvidenceError> {
    match code {
        0 => Ok(ModelEventKind::TextDelta),
        1 => Ok(ModelEventKind::ReasoningDelta),
        2 => Ok(ModelEventKind::ToolCallFragment),
        3 => Ok(ModelEventKind::ToolCallComplete),
        4 => Ok(ModelEventKind::Usage),
        5 => Ok(ModelEventKind::Stop),
        6 => Ok(ModelEventKind::BackendWarning),
        7 => Ok(ModelEventKind::BackendError),
        _ => Err(HarnessEvidenceError::InvalidRecord("model event kind code")),
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

fn compaction_state_transition_allowed(current: i64, next: i64) -> bool {
    match current {
        0 => matches!(next, 1 | 4 | 5 | 8),
        1 => matches!(next, 2 | 4 | 5 | 6 | 7 | 8),
        2 => matches!(next, 3 | 4 | 5 | 7 | 8),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::harness::CompactionId;
    use golam_core::harness_state::CompactionState;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

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

    fn accepted_event(attempt_id: RequestAttemptId, sequence: u64, payload: &[u8]) -> ModelEvent {
        ModelEvent {
            request_attempt_id: attempt_id,
            sequence,
            kind: ModelEventKind::TextDelta,
            payload: payload.to_vec(),
            acceptance: ModelEventAcceptance::Accepted,
            canonical_evidence_ref: Some(format!("event:model:{sequence}")),
        }
    }

    fn started_compaction() -> CompactionAttempt {
        CompactionAttempt {
            compaction_id: CompactionId::from_u128(21),
            session_id: SessionId(9),
            source_projection_ref: "projection:source:1".into(),
            state: CompactionState::Started,
            deterministic: true,
            producing_request_attempt_id: None,
            started_at_unix_ms: 100,
            terminal_at_unix_ms: None,
            failure_class: None,
        }
    }

    fn temp_db_path(label: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "golam-spec004-{label}-{}-{nonce}.sqlite3",
            std::process::id()
        ))
    }

    #[test]
    fn schema_is_forward_only_and_complete() {
        let store = HarnessEvidenceStore::open_in_memory().unwrap();
        assert_eq!(
            store.schema_version().unwrap(),
            HARNESS_EVIDENCE_SCHEMA_VERSION
        );
        for table in REQUIRED_HARNESS_TABLES {
            assert!(store.has_table(table).unwrap(), "missing {table}");
        }
    }

    #[test]
    fn future_schema_is_rejected_without_rewrite() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE harness_schema_meta (singleton INTEGER PRIMARY KEY, schema_version INTEGER NOT NULL); INSERT INTO harness_schema_meta VALUES (1, 2);",
            )
            .unwrap();
        assert!(matches!(
            HarnessEvidenceStore::initialize(connection),
            Err(HarnessEvidenceError::FutureSchema {
                found: 2,
                supported: 1
            })
        ));
    }

    #[test]
    fn prepared_attempt_must_exist_before_stream_event() {
        let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
        let attempt = prepared_attempt();
        let event = accepted_event(attempt.request_attempt_id, 0, b"hello");
        assert!(matches!(
            store.append_model_event(&event, b"event-record"),
            Err(HarnessEvidenceError::MissingAttempt(_))
        ));
        store
            .persist_prepared_attempt(SessionId(9), &attempt, b"prepared")
            .unwrap();
        store.append_model_event(&event, b"event-record").unwrap();
        assert_eq!(
            store
                .accepted_event_count(attempt.request_attempt_id)
                .unwrap(),
            1
        );
    }

    #[test]
    fn immutable_attempt_identity_and_profile_cannot_change_with_state() {
        let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
        let mut attempt = prepared_attempt();
        store
            .record_profile_selection(ProfileSelectionEvidence {
                session_id: SessionId(9),
                request_attempt_id: attempt.request_attempt_id,
                profile_id: attempt.execution_profile_id,
                selected_at_unix_ms: 9,
                reason_bytes: b"fixture pin",
            })
            .unwrap();
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
    fn prepared_attempt_rejects_mismatched_selected_profile() {
        let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
        let attempt = prepared_attempt();
        store
            .record_profile_selection(ProfileSelectionEvidence {
                session_id: SessionId(9),
                request_attempt_id: attempt.request_attempt_id,
                profile_id: ExecutionProfileId::from_u128(999),
                selected_at_unix_ms: 9,
                reason_bytes: b"stale selection",
            })
            .unwrap();
        assert!(matches!(
            store.persist_prepared_attempt(SessionId(9), &attempt, b"prepared"),
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
        let event = accepted_event(attempt.request_attempt_id, 0, b"hello");
        store.append_model_event(&event, b"one").unwrap();
        assert!(matches!(
            store.append_model_event(&event, b"two"),
            Err(HarnessEvidenceError::SequenceConflict { .. })
        ));
    }

    #[test]
    fn accepted_partial_output_survives_reopen_and_replays_in_order() {
        let path = temp_db_path("partial-reopen");
        let attempt = prepared_attempt();
        {
            let mut store = HarnessEvidenceStore::open(&path).unwrap();
            store
                .persist_prepared_attempt(SessionId(9), &attempt, b"prepared")
                .unwrap();
            store
                .append_model_event(
                    &accepted_event(attempt.request_attempt_id, 0, b"hel"),
                    b"event-0",
                )
                .unwrap();
            store
                .append_model_event(
                    &accepted_event(attempt.request_attempt_id, 1, b"lo"),
                    b"event-1",
                )
                .unwrap();
        }
        let reopened = HarnessEvidenceStore::open(&path).unwrap();
        let identity = reopened
            .attempt_identity(attempt.request_attempt_id)
            .unwrap()
            .unwrap();
        assert_eq!(identity.state, RequestAttemptState::Prepared);
        assert_eq!(identity.execution_profile_id, attempt.execution_profile_id);
        let events = reopened
            .accepted_events(attempt.request_attempt_id)
            .unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].sequence, 0);
        assert_eq!(events[0].payload, b"hel");
        assert_eq!(events[1].sequence, 1);
        assert_eq!(events[1].payload, b"lo");
        drop(reopened);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn compaction_lifecycle_must_begin_with_started() {
        let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
        let mut attempt = started_compaction();
        attempt.state = CompactionState::Deriving;
        assert!(matches!(
            store.record_compaction_attempt(&attempt, b"deriving"),
            Err(HarnessEvidenceError::InvalidRecord(
                "compaction lifecycle must begin with STARTED"
            ))
        ));
    }

    #[test]
    fn compaction_lifecycle_rejects_regression_and_terminal_rewrite() {
        let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
        let mut attempt = started_compaction();
        store
            .record_compaction_attempt(&attempt, b"started")
            .unwrap();

        attempt.transition(CompactionState::Deriving).unwrap();
        store
            .record_compaction_attempt(&attempt, b"deriving")
            .unwrap();

        let mut regression = attempt.clone();
        regression.state = CompactionState::Started;
        assert!(matches!(
            store.record_compaction_attempt(&regression, b"regression"),
            Err(HarnessEvidenceError::InvalidRecord(
                "invalid durable compaction state transition"
            ))
        ));

        attempt.transition(CompactionState::Validating).unwrap();
        store
            .record_compaction_attempt(&attempt, b"validating")
            .unwrap();
        attempt.transition(CompactionState::Committed).unwrap();
        attempt.terminal_at_unix_ms = Some(120);
        store
            .record_compaction_attempt(&attempt, b"committed")
            .unwrap();

        let mut rewrite = attempt.clone();
        rewrite.state = CompactionState::Cancelled;
        rewrite.terminal_at_unix_ms = Some(121);
        assert!(matches!(
            store.record_compaction_attempt(&rewrite, b"rewrite"),
            Err(HarnessEvidenceError::InvalidRecord(
                "invalid durable compaction state transition"
            ))
        ));
    }

    #[test]
    fn incomplete_compaction_start_survives_reopen() {
        let path = temp_db_path("compaction-start-reopen");
        let attempt = started_compaction();
        {
            let mut store = HarnessEvidenceStore::open(&path).unwrap();
            store
                .record_compaction_attempt(&attempt, b"started")
                .unwrap();
        }
        let reopened = HarnessEvidenceStore::open(&path).unwrap();
        let stored_state: i64 = reopened
            .connection
            .query_row(
                "SELECT state FROM harness_compaction_attempts WHERE compaction_id = ?1",
                params![id_blob(attempt.compaction_id.as_u128())],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored_state, 0);
        drop(reopened);
        let _ = fs::remove_file(path);
    }
}
