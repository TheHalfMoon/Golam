use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use golam_core::{CanonicalEncoder, CheckpointId, EventId, SessionId};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use crate::EventKind;
use crate::artifacts::{ArtifactError, ArtifactReceipt, ArtifactStore};
use crate::storage::{AppendEvent, AuthorityStore, StorageError};

const PROJECTION_DOMAIN: &[u8] = b"golam:checkpoint-projection:v1";
const CHECKPOINT_EVENT_DOMAIN: &[u8] = b"golam:checkpoint-event:v1";
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
        authority: &mut AuthorityStore,
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

        let created_event = authority.append_event(AppendEvent {
            event_id: input.created_event_id,
            session_id: input.session_id,
            expected_session_seq: input.through_session_seq,
            kind: EventKind::CheckpointCreated,
            actor_principal: input.actor_principal,
            recorded_at: input.recorded_at,
            payload: &event_payload,
            security_critical: true,
        })?;

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute(
            "INSERT OR IGNORE INTO artifacts (artifact_hash, size_bytes, media_type, \
             created_global_seq, retention_class, relative_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &artifact.hash[..],
                seq_to_i64(artifact.size_bytes)?,
                "application/x-golam-checkpoint",
                seq_to_i64(created_event.record.global_seq)?,
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
                seq_to_i64(created_event.record.session_seq)?,
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
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("golam-checkpoint-{}-{nonce}", std::process::id()))
    }

    #[test]
    fn checkpoint_matches_replay_and_missing_artifact_falls_back() {
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
}
