#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use golam_core::authority::{AuthorityLayout, AuthorityPathError};
use golam_core::paths::RuntimeLayout;
use golam_core::{CheckpointId, EffectId};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};

use crate::storage::{AuthorityStore, StorageError};

const MAX_DETAIL_CHARS: usize = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryMode {
    Normal,
    RecoveryOnly,
    Quarantine,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryIssueKind {
    AuthorityIntegrity,
    EffectNeedsRecovery,
    EffectStateIncoherent,
    CheckpointPrefixMissing,
    CheckpointPrefixMismatch,
    CheckpointArtifactMissing,
    CheckpointArtifactInvalid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryIssue {
    pub kind: RecoveryIssueKind,
    pub reference: String,
    pub detail: String,
    pub blocking: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryReport {
    pub mode: RecoveryMode,
    pub authority_db: PathBuf,
    pub issues: Vec<RecoveryIssue>,
}

impl RecoveryReport {
    pub fn privileged_service_allowed(&self) -> bool {
        self.mode == RecoveryMode::Normal
    }

    pub fn requires_attention(&self) -> bool {
        !self.issues.is_empty()
    }
}

#[derive(Debug)]
pub enum RecoveryError {
    AuthorityPath(AuthorityPathError),
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Io(io::Error),
    InvalidStoredRecord,
}

impl fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthorityPath(error) => write!(f, "recovery authority path error: {error}"),
            Self::Storage(error) => write!(f, "recovery authority store error: {error}"),
            Self::Sqlite(error) => write!(f, "recovery sqlite error: {error}"),
            Self::Io(error) => write!(f, "recovery I/O error: {error}"),
            Self::InvalidStoredRecord => f.write_str("recovery scan found malformed stored data"),
        }
    }
}

impl Error for RecoveryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AuthorityPath(error) => Some(error),
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::InvalidStoredRecord => None,
        }
    }
}

impl From<AuthorityPathError> for RecoveryError {
    fn from(value: AuthorityPathError) -> Self {
        Self::AuthorityPath(value)
    }
}

impl From<StorageError> for RecoveryError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for RecoveryError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<io::Error> for RecoveryError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub struct RecoveryScanner;

impl RecoveryScanner {
    pub fn scan(runtime: &RuntimeLayout) -> Result<RecoveryReport, RecoveryError> {
        let authority = AuthorityLayout::initialize(runtime)?;
        let authority_db = authority.authority_db_path().to_path_buf();

        if !authority_db.exists() {
            let store = AuthorityStore::open(&authority_db)?;
            drop(store);
        } else if let Err(error) = AuthorityStore::open(&authority_db) {
            return Ok(RecoveryReport {
                mode: RecoveryMode::Quarantine,
                authority_db,
                issues: vec![RecoveryIssue {
                    kind: RecoveryIssueKind::AuthorityIntegrity,
                    reference: "authority:golam.db".to_owned(),
                    detail: bounded_detail(&error.to_string()),
                    blocking: true,
                }],
            });
        }

        let connection = Connection::open_with_flags(
            &authority_db,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA query_only = ON; PRAGMA busy_timeout = 5000;",
        )?;

        let mut issues = Vec::new();
        scan_effects(&connection, &mut issues)?;
        scan_checkpoints(&connection, &runtime.artifact_dir, &mut issues)?;
        let mode = if issues.iter().any(|issue| issue.blocking) {
            RecoveryMode::RecoveryOnly
        } else {
            RecoveryMode::Normal
        };

        Ok(RecoveryReport {
            mode,
            authority_db,
            issues,
        })
    }
}

fn scan_effects(
    connection: &Connection,
    issues: &mut Vec<RecoveryIssue>,
) -> Result<(), RecoveryError> {
    let mut statement = connection.prepare(
        "SELECT i.effect_id, t.to_state, \
         (SELECT COUNT(*) FROM effect_attempts a WHERE a.effect_id = i.effect_id) \
         FROM effect_intents i \
         JOIN effect_transitions t ON t.effect_id = i.effect_id \
         WHERE t.global_seq = (SELECT MAX(t2.global_seq) FROM effect_transitions t2 \
                               WHERE t2.effect_id = i.effect_id) \
         ORDER BY t.global_seq ASC",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, Vec<u8>>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;

    for row in rows {
        let (effect_blob, state, attempt_count) = row?;
        let effect_id = EffectId(blob_u128(&effect_blob)?);
        let reference = format!("effect:{}", effect_id.0);
        match state.as_str() {
            "denied" | "succeeded" | "failed" => {}
            "executing" | "unknown_outcome" | "reconciling" if attempt_count <= 0 => {
                issues.push(RecoveryIssue {
                    kind: RecoveryIssueKind::EffectStateIncoherent,
                    reference,
                    detail: format!("state={state}; durable_attempts=0"),
                    blocking: true,
                });
            }
            "proposed" | "authorized" | "approval_required" | "executing"
            | "unknown_outcome" | "reconciling" | "manual_review" => {
                issues.push(RecoveryIssue {
                    kind: RecoveryIssueKind::EffectNeedsRecovery,
                    reference,
                    detail: format!("state={state}; durable_attempts={attempt_count}"),
                    blocking: false,
                });
            }
            _ => {
                issues.push(RecoveryIssue {
                    kind: RecoveryIssueKind::EffectStateIncoherent,
                    reference,
                    detail: format!("unknown_effect_state={state}"),
                    blocking: true,
                });
            }
        }
    }
    Ok(())
}

fn scan_checkpoints(
    connection: &Connection,
    artifact_root: &Path,
    issues: &mut Vec<RecoveryIssue>,
) -> Result<(), RecoveryError> {
    let mut statement = connection.prepare(
        "SELECT c.checkpoint_id, c.session_id, c.through_session_seq, c.through_global_seq, \
         c.through_event_hash, c.artifact_hash, a.size_bytes, a.relative_path \
         FROM checkpoints c LEFT JOIN artifacts a ON a.artifact_hash = c.artifact_hash \
         ORDER BY c.through_global_seq ASC",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, Vec<u8>>(0)?,
            row.get::<_, Vec<u8>>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, Vec<u8>>(4)?,
            row.get::<_, Vec<u8>>(5)?,
            row.get::<_, Option<i64>>(6)?,
            row.get::<_, Option<String>>(7)?,
        ))
    })?;

    for row in rows {
        let (
            checkpoint_blob,
            session_blob,
            through_session_seq,
            through_global_seq,
            through_event_hash,
            artifact_hash,
            size_bytes,
            relative_path,
        ) = row?;
        let checkpoint_id = CheckpointId(blob_u128(&checkpoint_blob)?);
        let reference = format!("checkpoint:{}", checkpoint_id.0);

        let prefix = connection
            .query_row(
                "SELECT global_seq, event_hash FROM session_events \
                 WHERE session_id = ?1 AND session_seq = ?2",
                params![&session_blob, through_session_seq],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
            )
            .optional()?;
        match prefix {
            None => issues.push(RecoveryIssue {
                kind: RecoveryIssueKind::CheckpointPrefixMissing,
                reference: reference.clone(),
                detail: format!("through_session_seq={through_session_seq}"),
                blocking: true,
            }),
            Some((actual_global_seq, actual_event_hash))
                if actual_global_seq != through_global_seq
                    || actual_event_hash != through_event_hash =>
            {
                issues.push(RecoveryIssue {
                    kind: RecoveryIssueKind::CheckpointPrefixMismatch,
                    reference: reference.clone(),
                    detail: format!(
                        "stored_global_seq={through_global_seq}; actual_global_seq={actual_global_seq}"
                    ),
                    blocking: true,
                });
            }
            Some(_) => {}
        }

        let (Some(size_bytes), Some(relative_path)) = (size_bytes, relative_path) else {
            issues.push(RecoveryIssue {
                kind: RecoveryIssueKind::CheckpointArtifactMissing,
                reference,
                detail: "artifact_metadata_missing".to_owned(),
                blocking: true,
            });
            continue;
        };
        let expected_size = u64::try_from(size_bytes).map_err(|_| RecoveryError::InvalidStoredRecord)?;
        let expected_hash = blob_hash(&artifact_hash)?;
        let relative_path = PathBuf::from(relative_path);
        if !safe_relative_path(&relative_path) {
            issues.push(RecoveryIssue {
                kind: RecoveryIssueKind::CheckpointArtifactInvalid,
                reference,
                detail: "artifact_relative_path_escapes_root".to_owned(),
                blocking: true,
            });
            continue;
        }
        let path = artifact_root.join(&relative_path);
        match verify_artifact(&path, expected_size, expected_hash) {
            Ok(()) => {}
            Err(detail) => issues.push(RecoveryIssue {
                kind: if detail == "artifact_missing" {
                    RecoveryIssueKind::CheckpointArtifactMissing
                } else {
                    RecoveryIssueKind::CheckpointArtifactInvalid
                },
                reference,
                detail,
                blocking: true,
            }),
        }
    }
    Ok(())
}

fn verify_artifact(path: &Path, expected_size: u64, expected_hash: [u8; 32]) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err("artifact_missing".to_owned());
        }
        Err(error) => return Err(bounded_detail(&format!("artifact_metadata_error:{error}"))),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("artifact_not_regular_file".to_owned());
    }
    if metadata.len() != expected_size {
        return Err(format!(
            "artifact_size_mismatch:expected={expected_size};actual={}",
            metadata.len()
        ));
    }
    let actual_hash = match hash_file(path) {
        Ok(hash) => hash,
        Err(error) => return Err(bounded_detail(&format!("artifact_read_error:{error}"))),
    };
    if actual_hash != expected_hash {
        return Err("artifact_hash_mismatch".to_owned());
    }
    Ok(())
}

fn hash_file(path: &Path) -> io::Result<[u8; 32]> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(*hasher.finalize().as_bytes())
}

fn safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path.components().all(|component| matches!(component, Component::Normal(_)))
}

fn blob_u128(bytes: &[u8]) -> Result<u128, RecoveryError> {
    let bytes: [u8; 16] = bytes
        .try_into()
        .map_err(|_| RecoveryError::InvalidStoredRecord)?;
    Ok(u128::from_be_bytes(bytes))
}

fn blob_hash(bytes: &[u8]) -> Result<[u8; 32], RecoveryError> {
    bytes
        .try_into()
        .map_err(|_| RecoveryError::InvalidStoredRecord)
}

fn bounded_detail(value: &str) -> String {
    value.chars().take(MAX_DETAIL_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::{EffectDispatchStore, PrepareEffectDispatch, encode_effect_dependencies};
    use crate::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
    use crate::storage::CreateSession;
    use golam_core::{EffectAttemptId, EffectTransitionId, EventId, SessionId};
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
            "golam-recovery-scan-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    fn propose_authorized(
        authority: &AuthorityLayout,
        effect_id: EffectId,
        seed: u128,
    ) -> EffectStore {
        let dependencies = encode_effect_dependencies(&[]).unwrap();
        let mut effects = EffectStore::open(authority).unwrap();
        effects
            .propose(ProposeEffect {
                effect_id,
                session_id: SessionId(seed + 1),
                requested_by: "owner",
                action: "sim.write",
                resource: "sim:recovery",
                risk_class: "synthetic",
                execution_semantics: "at_most_once",
                idempotency_key: None,
                preconditions: b"[]",
                dependencies: &dependencies,
                payload_hash: [5; 32],
                proposed_event_id: EventId(seed + 2),
                transition_id: EffectTransitionId(seed + 3),
            })
            .unwrap();
        effects
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(seed + 4),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: Some("test_authorized"),
                evidence_ref: None,
                event_id: EventId(seed + 5),
            })
            .unwrap();
        effects
    }

    #[test]
    fn clean_startup_is_normal_and_privileged_service_allowed() {
        let runtime = runtime();
        let report = RecoveryScanner::scan(&runtime).unwrap();
        assert_eq!(report.mode, RecoveryMode::Normal);
        assert!(report.privileged_service_allowed());
        assert!(!report.requires_attention());
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn coherent_executing_effect_is_attention_not_quarantine() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let effect_id = EffectId(100);
        let effects = propose_authorized(&authority, effect_id, 1_000);
        drop(effects);
        let mut dispatch = EffectDispatchStore::open(&authority).unwrap();
        dispatch
            .prepare_dispatch(PrepareEffectDispatch {
                effect_id,
                attempt_id: EffectAttemptId(101),
                transition_id: EffectTransitionId(1_010),
                handler_id: "sim-at-most-once-write",
                handler_version: "1",
                dispatch_token: b"dispatch-101",
                started_at: "2026-08-25T11:00:00Z",
                event_id: EventId(1_011),
            })
            .unwrap();
        drop(dispatch);

        let report = RecoveryScanner::scan(&runtime).unwrap();
        assert_eq!(report.mode, RecoveryMode::Normal);
        assert!(report.privileged_service_allowed());
        assert!(report.issues.iter().any(|issue| {
            issue.kind == RecoveryIssueKind::EffectNeedsRecovery
                && issue.reference == "effect:100"
                && issue.detail.contains("state=executing")
        }));
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn executing_effect_without_attempt_enters_recovery_only() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let effect_id = EffectId(200);
        let mut effects = propose_authorized(&authority, effect_id, 2_000);
        effects
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(2_010),
                effect_id,
                expected_state: "authorized",
                next_state: "executing",
                attempt_id: None,
                reason_code: Some("incoherent_test"),
                evidence_ref: None,
                event_id: EventId(2_011),
            })
            .unwrap();
        drop(effects);

        let report = RecoveryScanner::scan(&runtime).unwrap();
        assert_eq!(report.mode, RecoveryMode::RecoveryOnly);
        assert!(!report.privileged_service_allowed());
        assert!(report.issues.iter().any(|issue| {
            issue.kind == RecoveryIssueKind::EffectStateIncoherent
                && issue.reference == "effect:200"
        }));
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn checkpoint_without_artifact_enters_recovery_only() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let mut store = AuthorityStore::open(authority.authority_db_path()).unwrap();
        let created = store
            .create_session(CreateSession {
                session_id: SessionId(300),
                event_id: EventId(301),
                owner_principal: "owner",
                actor_principal: "owner",
                recorded_at: "2026-08-25T11:01:00Z",
                payload: b"session",
                security_critical: true,
            })
            .unwrap();
        drop(store);
        let connection = Connection::open(authority.authority_db_path()).unwrap();
        connection
            .execute(
                "INSERT INTO checkpoints (checkpoint_id, session_id, through_session_seq, \
                 through_global_seq, through_event_hash, projection_schema_version, artifact_hash, \
                 created_event_id, verified_at) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?8)",
                params![
                    302_u128.to_be_bytes().to_vec(),
                    300_u128.to_be_bytes().to_vec(),
                    1_i64,
                    i64::try_from(created.record.global_seq).unwrap(),
                    created.event_hash.to_vec(),
                    vec![9_u8; 32],
                    303_u128.to_be_bytes().to_vec(),
                    "2026-08-25T11:01:01Z",
                ],
            )
            .unwrap();
        drop(connection);

        let report = RecoveryScanner::scan(&runtime).unwrap();
        assert_eq!(report.mode, RecoveryMode::RecoveryOnly);
        assert!(report.issues.iter().any(|issue| {
            issue.kind == RecoveryIssueKind::CheckpointArtifactMissing
                && issue.reference == "checkpoint:302"
        }));
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn canonical_corruption_enters_quarantine_without_reset() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let mut store = AuthorityStore::open(authority.authority_db_path()).unwrap();
        store
            .create_session(CreateSession {
                session_id: SessionId(400),
                event_id: EventId(401),
                owner_principal: "owner",
                actor_principal: "owner",
                recorded_at: "2026-08-25T11:02:00Z",
                payload: b"session",
                security_critical: true,
            })
            .unwrap();
        drop(store);
        let connection = Connection::open(authority.authority_db_path()).unwrap();
        connection
            .execute(
                "UPDATE session_events SET event_hash = ?1 WHERE event_id = ?2",
                params![vec![0xAA_u8; 32], 401_u128.to_be_bytes().to_vec()],
            )
            .unwrap();
        drop(connection);

        let report = RecoveryScanner::scan(&runtime).unwrap();
        assert_eq!(report.mode, RecoveryMode::Quarantine);
        assert!(!report.privileged_service_allowed());
        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.issues[0].kind, RecoveryIssueKind::AuthorityIntegrity);

        let connection = Connection::open(authority.authority_db_path()).unwrap();
        let persisted: Vec<u8> = connection
            .query_row(
                "SELECT event_hash FROM session_events WHERE event_id = ?1",
                params![401_u128.to_be_bytes().to_vec()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(persisted, vec![0xAA_u8; 32]);
        drop(connection);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
