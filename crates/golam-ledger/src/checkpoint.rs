use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use golam_core::{CanonicalEncoder, CheckpointId, EventId, SCHEMA_VERSION, SessionId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::artifacts::{ArtifactError, ArtifactReceipt, ArtifactStore};
use crate::storage::{AuthorityStore, StorageError};
use crate::{EventKind, EventRecord, audit_integrity_hash, event_integrity_hash, payload_hash};

const PROJECTION_DOMAIN: &[u8] = b"golam:checkpoint-projection:v1";
const CHECKPOINT_EVENT_DOMAIN: &[u8] = b"golam:checkpoint-event:v1";
const SECURITY_AUDIT_CHAIN: &str = "security";
pub const CHECKPOINT_PROJECTION_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionSource {
    Checkpoint,
    ReplayFallback,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedProjection {
    pub bytes: Vec<u8>,
    pub source: ProjectionSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckpointRecord {
    pub checkpoint_id: CheckpointId,
    pub session_id: SessionId,
    pub through_session_seq: u64,
    pub through_global_seq: u64,
    pub through_event_hash: [u8; 32],
    pub artifact: ArtifactReceipt,
    pub created_event_id: EventId,
}

pub struct CreateCheckpoint<'a> {
    pub checkpoint_id: CheckpointId,
    pub created_event_id: EventId,
    pub session_id: SessionId,
    pub through_session_seq: u64,
    pub actor_principal: &'a str,
    pub recorded_at: &'a str,
}

#[derive(Debug)]
pub enum CheckpointError {
    Sqlite(rusqlite::Error),
    Storage(StorageError),
    Artifact(ArtifactError),
    Core(golam_core::CoreError),
    PrefixNotFound {
        session_id: SessionId,
        through_session_seq: u64,
    },
    InvalidStoredHash,
    SequenceOverflow,
}

impl fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "checkpoint sqlite error: {error}"),
            Self::Storage(error) => write!(f, "checkpoint ledger error: {error}"),
            Self::Artifact(error) => write!(f, "checkpoint artifact error: {error}"),
            Self::Core(error) => write!(f, "checkpoint canonical encoding error: {error}"),
            Self::PrefixNotFound {
                session_id,
                through_session_seq,
            } => write!(
                f,
                "checkpoint prefix not found for session {} through sequence {through_session_seq}",
                session_id.0
            ),
            Self::InvalidStoredHash => f.write_str("checkpoint stored hash is not 32 bytes"),
            Self::SequenceOverflow => {
                f.write_str("checkpoint sequence exceeds SQLite integer range")
            }
        }
    }
}

impl Error for CheckpointError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Storage(error) => Some(error),
            Self::Artifact(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::PrefixNotFound { .. } | Self::InvalidStoredHash | Self::SequenceOverflow => None,
        }
    }
}

impl From<rusqlite::Error> for CheckpointError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<StorageError> for CheckpointError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<ArtifactError> for CheckpointError {
    fn from(value: ArtifactError) -> Self {
        Self::Artifact(value)
    }
}

impl From<golam_core::CoreError> for CheckpointError {
    fn from(value: golam_core::CoreError) -> Self {
        Self::Core(value)
    }
}

#[derive(Clone, Debug)]
struct ProjectionEvent {
    global_seq: u64,
    session_seq: u64,
    event_hash: [u8; 32],
    payload_hash: [u8; 32],
}

#[derive(Clone, Debug)]
struct ProjectionBuild {
    bytes: Vec<u8>,
    through_global_seq: u64,
    through_event_hash: [u8; 32],
}

#[derive(Clone, Copy, Debug)]
struct CreatedCheckpointEvent {
    global_seq: u64,
    session_seq: u64,
}

pub struct CheckpointManager {
    connection: Connection,
    artifacts: ArtifactStore,
}

impl CheckpointManager {
    pub fn open(
        authority_db: impl AsRef<Path>,
        artifact_root: impl AsRef<Path>,
    ) -> Result<Self, CheckpointError> {
        let connection = Connection::open(authority_db)?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;\n\
             PRAGMA journal_mode = WAL;\n\
             PRAGMA synchronous = FULL;\n\
             PRAGMA busy_timeout = 5000;",
        )?;
        let artifacts = ArtifactStore::open(artifact_root)?;
        Ok(Self {
            connection,
            artifacts,
        })
    }

    pub fn create(
        &mut self,
        _authority: &mut AuthorityStore,
        input: CreateCheckpoint<'_>,
    ) -> Result<CheckpointRecord, CheckpointError> {
        let projection = self.build_projection(input.session_id, input.through_session_seq)?;
        let artifact = self.artifacts.install_bytes(&projection.bytes)?;
        let event_payload = checkpoint_event_payload(
            input.checkpoint_id,
            input.session_id,
            input.through_session_seq,
            projection.through_global_seq,
            projection.through_event_hash,
            artifact.hash,
        )?;

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let created_event = append_checkpoint_event(&transaction, &input, &event_payload)?;
        #[cfg(test)]
        checkpoint_process_kill_test_hook();

        transaction.execute(
            "INSERT OR IGNORE INTO artifacts (artifact_hash, size_bytes, media_type, \
             created_global_seq, retention_class, relative_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &artifact.hash[..],
                seq_to_i64(artifact.size_bytes)?,
                "application/x-golam-checkpoint",
                seq_to_i64(created_event.global_seq)?,
                "checkpoint",
                relative_path_string(&artifact.relative_path),
            ],
        )?;
        transaction.execute(
            "INSERT INTO checkpoints (checkpoint_id, session_id, through_session_seq, \
             through_global_seq, through_event_hash, projection_schema_version, artifact_hash, \
             created_event_id, verified_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id_blob(input.checkpoint_id.0),
                id_blob(input.session_id.0),
                seq_to_i64(input.through_session_seq)?,
                seq_to_i64(projection.through_global_seq)?,
                &projection.through_event_hash[..],
                i64::from(CHECKPOINT_PROJECTION_SCHEMA_VERSION),
                &artifact.hash[..],
                id_blob(input.created_event_id.0),
                input.recorded_at,
            ],
        )?;
        transaction.execute(
            "UPDATE sessions SET latest_checkpoint_id = ?1 \
             WHERE session_id = ?2 AND latest_session_seq = ?3",
            params![
                id_blob(input.checkpoint_id.0),
                id_blob(input.session_id.0),
                seq_to_i64(created_event.session_seq)?,
            ],
        )?;
        transaction.commit()?;

        Ok(CheckpointRecord {
            checkpoint_id: input.checkpoint_id,
            session_id: input.session_id,
            through_session_seq: input.through_session_seq,
            through_global_seq: projection.through_global_seq,
            through_event_hash: projection.through_event_hash,
            artifact,
            created_event_id: input.created_event_id,
        })
    }

    pub fn replay_projection(
        &self,
        session_id: SessionId,
        through_session_seq: u64,
    ) -> Result<Vec<u8>, CheckpointError> {
        Ok(self
            .build_projection(session_id, through_session_seq)?
            .bytes)
    }

    pub fn load_or_replay(
        &self,
        checkpoint_id: CheckpointId,
        session_id: SessionId,
        through_session_seq: u64,
    ) -> Result<LoadedProjection, CheckpointError> {
        if let Some(receipt) =
            self.load_verified_checkpoint_receipt(checkpoint_id, session_id, through_session_seq)?
            && let Ok(bytes) = self.artifacts.read_verified(&receipt)
        {
            return Ok(LoadedProjection {
                bytes,
                source: ProjectionSource::Checkpoint,
            });
        }

        Ok(LoadedProjection {
            bytes: self.replay_projection(session_id, through_session_seq)?,
            source: ProjectionSource::ReplayFallback,
        })
    }

    fn build_projection(
        &self,
        session_id: SessionId,
        through_session_seq: u64,
    ) -> Result<ProjectionBuild, CheckpointError> {
        let mut statement = self.connection.prepare(
            "SELECT global_seq, session_seq, event_hash, payload_hash FROM session_events \
             WHERE session_id = ?1 AND session_seq <= ?2 ORDER BY session_seq ASC",
        )?;
        let mut rows = statement.query(params![
            id_blob(session_id.0),
            seq_to_i64(through_session_seq)?,
        ])?;
        let mut events = Vec::new();
        while let Some(row) = rows.next()? {
            events.push(ProjectionEvent {
                global_seq: seq_from_i64(row.get(0)?)?,
                session_seq: seq_from_i64(row.get(1)?)?,
                event_hash: hash_from_vec(row.get(2)?)?,
                payload_hash: hash_from_vec(row.get(3)?)?,
            });
        }

        let Some(last) = events.last() else {
            return Err(CheckpointError::PrefixNotFound {
                session_id,
                through_session_seq,
            });
        };
        if last.session_seq != through_session_seq {
            return Err(CheckpointError::PrefixNotFound {
                session_id,
                through_session_seq,
            });
        }

        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(PROJECTION_DOMAIN)?;
        encoder.push_u16(CHECKPOINT_PROJECTION_SCHEMA_VERSION);
        encoder.push_u128(session_id.0);
        encoder.push_u64(through_session_seq);
        encoder
            .push_u64(u64::try_from(events.len()).map_err(|_| CheckpointError::SequenceOverflow)?);
        for event in &events {
            encoder.push_u64(event.global_seq);
            encoder.push_u64(event.session_seq);
            encoder.push_bytes(&event.event_hash)?;
            encoder.push_bytes(&event.payload_hash)?;
        }

        Ok(ProjectionBuild {
            bytes: encoder.finish(),
            through_global_seq: last.global_seq,
            through_event_hash: last.event_hash,
        })
    }

    fn load_verified_checkpoint_receipt(
        &self,
        checkpoint_id: CheckpointId,
        session_id: SessionId,
        through_session_seq: u64,
    ) -> Result<Option<ArtifactReceipt>, CheckpointError> {
        let row = self
            .connection
            .query_row(
                "SELECT c.through_global_seq, c.through_event_hash, c.artifact_hash, \
                 c.created_event_id, a.size_bytes, a.relative_path \
                 FROM checkpoints c JOIN artifacts a ON a.artifact_hash = c.artifact_hash \
                 WHERE c.checkpoint_id = ?1 AND c.session_id = ?2 AND c.through_session_seq = ?3",
                params![
                    id_blob(checkpoint_id.0),
                    id_blob(session_id.0),
                    seq_to_i64(through_session_seq)?,
                ],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, Vec<u8>>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                        row.get::<_, Vec<u8>>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, String>(5)?,
                    ))
                },
            )
            .optional()?;

        let Some((through_global, through_hash, artifact_hash, created_event_id, size, relative)) =
            row
        else {
            return Ok(None);
        };
        let through_global = seq_from_i64(through_global)?;
        let through_hash = hash_from_vec(through_hash)?;
        let artifact_hash = hash_from_vec(artifact_hash)?;
        let created_event_id = id_from_vec(created_event_id)?;
        let size_bytes = seq_from_i64(size)?;

        let canonical_prefix = self
            .connection
            .query_row(
                "SELECT global_seq, event_hash FROM session_events \
                 WHERE session_id = ?1 AND session_seq = ?2",
                params![id_blob(session_id.0), seq_to_i64(through_session_seq)?],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
            )
            .optional()?;
        let Some((canonical_global, canonical_hash)) = canonical_prefix else {
            return Ok(None);
        };
        if seq_from_i64(canonical_global)? != through_global
            || hash_from_vec(canonical_hash)? != through_hash
        {
            return Ok(None);
        }

        let expected_event_payload = checkpoint_event_payload(
            checkpoint_id,
            session_id,
            through_session_seq,
            through_global,
            through_hash,
            artifact_hash,
        )?;
        let stored_event_payload = self
            .connection
            .query_row(
                "SELECT payload_bytes FROM session_events WHERE event_id = ?1 AND event_type = ?2",
                params![
                    id_blob(created_event_id),
                    i64::from(EventKind::CheckpointCreated.code())
                ],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()?;
        if stored_event_payload.as_deref() != Some(expected_event_payload.as_slice()) {
            return Ok(None);
        }

        Ok(Some(ArtifactReceipt {
            hash: artifact_hash,
            size_bytes,
            relative_path: path_from_stored_relative(&relative),
        }))
    }
}

#[cfg(test)]
fn checkpoint_process_kill_test_hook() {
    const CHILD_FLAG: &str = "GOLAM_CHECKPOINT_KILL_CHILD";
    const MARKER_ENV: &str = "GOLAM_CHECKPOINT_KILL_MARKER";

    if std::env::var_os(CHILD_FLAG).is_none() {
        return;
    }
    let marker = std::path::PathBuf::from(
        std::env::var_os(MARKER_ENV).expect("checkpoint kill marker path is configured"),
    );
    std::fs::write(marker, b"checkpoint-event-uncommitted").unwrap();
    loop {
        std::thread::park_timeout(std::time::Duration::from_secs(60));
    }
}

fn append_checkpoint_event(
    transaction: &Transaction<'_>,
    input: &CreateCheckpoint<'_>,
    payload: &[u8],
) -> Result<CreatedCheckpointEvent, CheckpointError> {
    let session_blob = id_blob(input.session_id.0);
    let head = transaction
        .query_row(
            "SELECT latest_session_seq, latest_event_hash FROM sessions WHERE session_id = ?1",
            params![&session_blob],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?
        .ok_or(StorageError::SessionNotFound(input.session_id))?;
    let actual_session_seq = seq_from_i64(head.0)?;
    if actual_session_seq != input.through_session_seq {
        return Err(StorageError::StaleSessionHead {
            expected: input.through_session_seq,
            actual: actual_session_seq,
        }
        .into());
    }

    let previous_session_event_hash = Some(hash_from_vec(head.1)?);
    let session_seq = actual_session_seq
        .checked_add(1)
        .ok_or(CheckpointError::SequenceOverflow)?;
    let global_seq = next_global_seq(transaction)?;
    let previous_audit_hash = transaction
        .query_row(
            "SELECT last_hash FROM audit_chain_heads WHERE chain_name = ?1",
            params![SECURITY_AUDIT_CHAIN],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .optional()?
        .map(hash_from_vec)
        .transpose()?;
    let record = EventRecord {
        event_id: input.created_event_id,
        session_id: input.session_id,
        global_seq,
        session_seq,
        schema_version: SCHEMA_VERSION,
        kind: EventKind::CheckpointCreated,
        actor_principal: input.actor_principal.to_owned(),
        recorded_at: input.recorded_at.to_owned(),
        payload_hash: payload_hash(payload),
        previous_session_event_hash,
        security_critical: true,
        previous_audit_hash,
    };
    let event_hash = event_integrity_hash(&record)?;
    let audit_hash = audit_integrity_hash(&record, event_hash)?;

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
    let updated = transaction.execute(
        "UPDATE sessions SET latest_session_seq = ?1, latest_event_hash = ?2 \
         WHERE session_id = ?3 AND latest_session_seq = ?4",
        params![
            seq_to_i64(session_seq)?,
            &event_hash[..],
            &session_blob,
            seq_to_i64(input.through_session_seq)?,
        ],
    )?;
    if updated != 1 {
        return Err(StorageError::StaleSessionHead {
            expected: input.through_session_seq,
            actual: actual_session_seq,
        }
        .into());
    }
    transaction.execute(
        "INSERT INTO audit_chain_heads (chain_name, last_global_seq, last_hash) VALUES (?1, ?2, ?3) \
         ON CONFLICT(chain_name) DO UPDATE SET last_global_seq = excluded.last_global_seq, \
         last_hash = excluded.last_hash",
        params![SECURITY_AUDIT_CHAIN, seq_to_i64(global_seq)?, &audit_hash[..]],
    )?;

    Ok(CreatedCheckpointEvent {
        global_seq,
        session_seq,
    })
}

fn next_global_seq(transaction: &Transaction<'_>) -> Result<u64, CheckpointError> {
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
        .ok_or(CheckpointError::SequenceOverflow)
}

fn checkpoint_event_payload(
    checkpoint_id: CheckpointId,
    session_id: SessionId,
    through_session_seq: u64,
    through_global_seq: u64,
    through_event_hash: [u8; 32],
    artifact_hash: [u8; 32],
) -> Result<Vec<u8>, CheckpointError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(CHECKPOINT_EVENT_DOMAIN)?;
    encoder.push_u128(checkpoint_id.0);
    encoder.push_u128(session_id.0);
    encoder.push_u64(through_session_seq);
    encoder.push_u64(through_global_seq);
    encoder.push_bytes(&through_event_hash)?;
    encoder.push_bytes(&artifact_hash)?;
    Ok(encoder.finish())
}

fn relative_path_string(path: &Path) -> String {
    path.iter()
        .map(|component| component.to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn path_from_stored_relative(value: &str) -> PathBuf {
    value.split('/').collect()
}

fn id_blob(value: u128) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

fn id_from_vec(value: Vec<u8>) -> Result<u128, CheckpointError> {
    let bytes: [u8; 16] = value
        .try_into()
        .map_err(|_| CheckpointError::InvalidStoredHash)?;
    Ok(u128::from_be_bytes(bytes))
}

fn hash_from_vec(value: Vec<u8>) -> Result<[u8; 32], CheckpointError> {
    value
        .try_into()
        .map_err(|_| CheckpointError::InvalidStoredHash)
}

fn seq_to_i64(value: u64) -> Result<i64, CheckpointError> {
    i64::try_from(value).map_err(|_| CheckpointError::SequenceOverflow)
}

fn seq_from_i64(value: i64) -> Result<u64, CheckpointError> {
    u64::try_from(value).map_err(|_| CheckpointError::SequenceOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{AppendEvent, CreateSession};
    use std::env;
    use std::fs;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    const CHILD_FLAG: &str = "GOLAM_CHECKPOINT_KILL_CHILD";
    const ROOT_ENV: &str = "GOLAM_CHECKPOINT_KILL_ROOT";
    const MARKER_ENV: &str = "GOLAM_CHECKPOINT_KILL_MARKER";

    fn unique_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("golam-checkpoint-{}-{nonce}", std::process::id()))
    }

    #[test]
    fn checkpoint_matches_replay_and_missing_artifact_falls_back() {
        if env::var_os(CHILD_FLAG).is_some() {
            return;
        }
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        let db_path = root.join("authority.db");
        let artifact_root = root.join("artifacts");
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

        let mut manager = CheckpointManager::open(&db_path, &artifact_root).unwrap();
        let record = manager
            .create(
                &mut authority,
                CreateCheckpoint {
                    checkpoint_id: CheckpointId(100),
                    created_event_id: EventId(3),
                    session_id: SessionId(1),
                    through_session_seq: 2,
                    actor_principal: "owner",
                    recorded_at: "2026-08-24T00:00:02Z",
                },
            )
            .unwrap();

        let replay = manager.replay_projection(SessionId(1), 2).unwrap();
        let loaded = manager
            .load_or_replay(CheckpointId(100), SessionId(1), 2)
            .unwrap();
        assert_eq!(loaded.source, ProjectionSource::Checkpoint);
        assert_eq!(loaded.bytes, replay);

        fs::remove_file(artifact_root.join(&record.artifact.relative_path)).unwrap();
        let fallback = manager
            .load_or_replay(CheckpointId(100), SessionId(1), 2)
            .unwrap();
        assert_eq!(fallback.source, ProjectionSource::ReplayFallback);
        assert_eq!(fallback.bytes, replay);

        drop(manager);
        drop(authority);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_checkpoint_metadata_write_rolls_back_event_and_session_head() {
        if env::var_os(CHILD_FLAG).is_some() {
            return;
        }
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        let db_path = root.join("authority.db");
        let artifact_root = root.join("artifacts");
        let mut authority = AuthorityStore::open(&db_path).unwrap();
        for (session_id, event_id) in [(SessionId(10), EventId(10)), (SessionId(20), EventId(20))] {
            authority
                .create_session(CreateSession {
                    session_id,
                    event_id,
                    owner_principal: "owner",
                    actor_principal: "owner",
                    recorded_at: "2026-08-26T11:55:00Z",
                    payload: b"create",
                    security_critical: true,
                })
                .unwrap();
        }

        let mut manager = CheckpointManager::open(&db_path, &artifact_root).unwrap();
        manager
            .create(
                &mut authority,
                CreateCheckpoint {
                    checkpoint_id: CheckpointId(900),
                    created_event_id: EventId(11),
                    session_id: SessionId(10),
                    through_session_seq: 1,
                    actor_principal: "owner",
                    recorded_at: "2026-08-26T11:55:01Z",
                },
            )
            .unwrap();

        let failed = manager.create(
            &mut authority,
            CreateCheckpoint {
                checkpoint_id: CheckpointId(900),
                created_event_id: EventId(21),
                session_id: SessionId(20),
                through_session_seq: 1,
                actor_principal: "owner",
                recorded_at: "2026-08-26T11:55:02Z",
            },
        );
        assert!(matches!(failed, Err(CheckpointError::Sqlite(_))));

        let latest_session_seq: i64 = manager
            .connection
            .query_row(
                "SELECT latest_session_seq FROM sessions WHERE session_id = ?1",
                params![id_blob(SessionId(20).0)],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(latest_session_seq, 1);
        let rolled_back_event_count: i64 = manager
            .connection
            .query_row(
                "SELECT COUNT(*) FROM session_events WHERE event_id = ?1",
                params![id_blob(EventId(21).0)],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(rolled_back_event_count, 0);
        authority.verify_integrity().unwrap();

        drop(manager);
        drop(authority);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn checkpoint_process_kill_child() {
        if env::var_os(CHILD_FLAG).is_none() {
            return;
        }
        let root =
            PathBuf::from(env::var_os(ROOT_ENV).expect("checkpoint kill root is configured"));
        let db_path = root.join("authority.db");
        let artifact_root = root.join("artifacts");
        let mut authority = AuthorityStore::open(&db_path).unwrap();
        let mut manager = CheckpointManager::open(&db_path, &artifact_root).unwrap();
        let _ = manager.create(
            &mut authority,
            CreateCheckpoint {
                checkpoint_id: CheckpointId(930),
                created_event_id: EventId(931),
                session_id: SessionId(932),
                through_session_seq: 1,
                actor_principal: "owner",
                recorded_at: "2026-08-26T11:56:00Z",
            },
        );
        unreachable!("checkpoint kill hook must stop the child before create returns");
    }

    #[test]
    fn os_process_kill_during_checkpoint_transaction_rolls_back_canonical_state() {
        if env::var_os(CHILD_FLAG).is_some() {
            return;
        }
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        let db_path = root.join("authority.db");
        let artifact_root = root.join("artifacts");
        let marker = root.join("checkpoint-uncommitted.marker");

        let mut authority = AuthorityStore::open(&db_path).unwrap();
        authority
            .create_session(CreateSession {
                session_id: SessionId(932),
                event_id: EventId(933),
                owner_principal: "owner",
                actor_principal: "owner",
                recorded_at: "2026-08-26T11:55:59Z",
                payload: b"create",
                security_critical: true,
            })
            .unwrap();
        drop(authority);

        let mut child = Command::new(env::current_exe().unwrap())
            .arg("checkpoint_process_kill_child")
            .arg("--nocapture")
            .env(CHILD_FLAG, "1")
            .env(ROOT_ENV, &root)
            .env(MARKER_ENV, &marker)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();

        let deadline = Instant::now() + Duration::from_secs(15);
        while !marker.exists() {
            if let Some(status) = child.try_wait().unwrap() {
                panic!("checkpoint kill child exited before transaction marker: {status}");
            }
            assert!(
                Instant::now() < deadline,
                "checkpoint kill child did not reach uncommitted transaction marker"
            );
            thread::sleep(Duration::from_millis(20));
        }

        child.kill().unwrap();
        let _ = child.wait().unwrap();

        let restarted = AuthorityStore::open(&db_path).unwrap();
        restarted.verify_integrity().unwrap();
        drop(restarted);

        let connection = Connection::open(&db_path).unwrap();
        let latest_session_seq: i64 = connection
            .query_row(
                "SELECT latest_session_seq FROM sessions WHERE session_id = ?1",
                params![id_blob(SessionId(932).0)],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(latest_session_seq, 1);
        let event_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM session_events WHERE event_id = ?1",
                params![id_blob(EventId(931).0)],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(event_count, 0);
        let checkpoint_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM checkpoints WHERE checkpoint_id = ?1",
                params![id_blob(CheckpointId(930).0)],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(checkpoint_count, 0);
        drop(connection);

        fs::remove_dir_all(root).unwrap();
    }
}
