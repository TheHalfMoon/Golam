#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{EffectAttemptId, EffectId, EffectTransitionId, EventId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::effects::StoredEffectTransition;
use crate::security_audit::{
    self, EffectTransitionAuditInput, RecoveryIncidentAuditInput,
};
use crate::storage::{AuthorityStore, StorageError};

const MANUAL_REVIEW_DOMAIN: &[u8] = b"golam:effect-manual-review:v1";
const INCIDENT_KIND: &str = "effect_manual_review";
const INCIDENT_SEVERITY: &str = "warning";
const RECOVERY_MODE: &str = "manual_review";
const AFFECTED_REFS_VERSION: u8 = 1;
const AFFECTED_REFS_LEN: usize = 50;
const MAX_EVIDENCE_BYTES: usize = 4096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManualReviewReason {
    UnreconcilableAmbiguity,
    ReconciliationUnsupported,
    ReconciliationExhausted,
}

impl ManualReviewReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnreconcilableAmbiguity => "unreconcilable_ambiguity",
            Self::ReconciliationUnsupported => "reconciliation_unsupported",
            Self::ReconciliationExhausted => "reconciliation_exhausted",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "unreconcilable_ambiguity" => Some(Self::UnreconcilableAmbiguity),
            "reconciliation_unsupported" => Some(Self::ReconciliationUnsupported),
            "reconciliation_exhausted" => Some(Self::ReconciliationExhausted),
            _ => None,
        }
    }
}

pub struct PlaceEffectInManualReview<'a> {
    pub effect_id: EffectId,
    pub transition_id: EffectTransitionId,
    pub attempt_id: Option<EffectAttemptId>,
    pub detected_at: &'a str,
    pub reason: ManualReviewReason,
    pub evidence_ref: Option<&'a [u8]>,
    pub event_id: EventId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManualReviewReport {
    pub incident_id: [u8; 16],
    pub effect_id: EffectId,
    pub transition_id: EffectTransitionId,
    pub attempt_id: Option<EffectAttemptId>,
    pub detected_at: String,
    pub from_state: String,
    pub reason: ManualReviewReason,
    pub evidence_ref: Option<Vec<u8>>,
    pub global_seq: u64,
}

#[derive(Debug)]
pub enum ManualReviewError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    SecurityAudit(String),
    InvalidMetadata,
    EvidenceTooLarge,
    EffectNotFound(EffectId),
    InvalidSourceState {
        effect_id: EffectId,
        actual: String,
    },
    AttemptNotFound(EffectAttemptId),
    AttemptEffectMismatch {
        attempt_id: EffectAttemptId,
        effect_id: EffectId,
    },
    SequenceOverflow,
    InvalidStoredRecord,
}

impl fmt::Display for ManualReviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "manual review authority error: {error}"),
            Self::Sqlite(error) => write!(f, "manual review sqlite error: {error}"),
            Self::SecurityAudit(error) => write!(f, "manual review integrity-chain error: {error}"),
            Self::InvalidMetadata => f.write_str("manual review detected-at metadata is required"),
            Self::EvidenceTooLarge => {
                f.write_str("manual review evidence exceeds the bounded limit")
            }
            Self::EffectNotFound(effect_id) => write!(f, "effect not found: {}", effect_id.0),
            Self::InvalidSourceState { effect_id, actual } => write!(
                f,
                "effect cannot enter manual review from current state: effect={} state={actual}",
                effect_id.0
            ),
            Self::AttemptNotFound(attempt_id) => {
                write!(f, "effect attempt not found: {}", attempt_id.0)
            }
            Self::AttemptEffectMismatch {
                attempt_id,
                effect_id,
            } => write!(
                f,
                "effect attempt does not belong to effect: attempt={} effect={}",
                attempt_id.0, effect_id.0
            ),
            Self::SequenceOverflow => f.write_str("manual review global sequence overflow"),
            Self::InvalidStoredRecord => f.write_str("stored manual review report is malformed"),
        }
    }
}

impl Error for ManualReviewError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for ManualReviewError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for ManualReviewError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub struct ManualReviewStore {
    connection: Connection,
}

impl ManualReviewStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, ManualReviewError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn place(
        &mut self,
        input: PlaceEffectInManualReview<'_>,
    ) -> Result<ManualReviewReport, ManualReviewError> {
        validate_input(&input)?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let effect_blob = id_blob(input.effect_id.0);
        let from_state = transaction
            .query_row(
                "SELECT to_state FROM effect_transitions WHERE effect_id = ?1 \
                 ORDER BY global_seq DESC LIMIT 1",
                params![&effect_blob],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or(ManualReviewError::EffectNotFound(input.effect_id))?;
        if from_state != "unknown_outcome" && from_state != "reconciling" {
            return Err(ManualReviewError::InvalidSourceState {
                effect_id: input.effect_id,
                actual: from_state,
            });
        }
        validate_attempt(&transaction, input.effect_id, input.attempt_id)?;

        let global_seq = next_global_seq(&transaction)?;
        let transition = StoredEffectTransition {
            transition_id: input.transition_id,
            effect_id: input.effect_id,
            global_seq,
            from_state: Some(from_state.clone()),
            to_state: "manual_review".to_owned(),
            attempt_id: input.attempt_id,
            reason_code: Some(input.reason.as_str().to_owned()),
            evidence_ref: input.evidence_ref.map(<[u8]>::to_vec),
            event_id: input.event_id,
        };
        insert_transition(&transaction, &transition)?;

        let incident_id = manual_review_incident_id(input.effect_id, input.transition_id);
        let affected_refs =
            encode_affected_refs(input.effect_id, input.transition_id, input.attempt_id);
        let resolution = input.reason.as_str().as_bytes();
        transaction.execute(
            "INSERT INTO recovery_incidents \
             (incident_id, detected_at, kind, severity, affected_refs, recovery_mode, resolution) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &incident_id[..],
                input.detected_at,
                INCIDENT_KIND,
                INCIDENT_SEVERITY,
                &affected_refs,
                RECOVERY_MODE,
                resolution,
            ],
        )?;
        security_audit::append_recovery_incident(
            &transaction,
            RecoveryIncidentAuditInput {
                incident_id: &incident_id,
                detected_at: input.detected_at,
                kind: INCIDENT_KIND,
                severity: INCIDENT_SEVERITY,
                affected_refs: &affected_refs,
                recovery_mode: RECOVERY_MODE,
                resolution: Some(resolution),
            },
        )
        .map_err(|error| ManualReviewError::SecurityAudit(error.to_string()))?;
        transaction.commit()?;

        Ok(ManualReviewReport {
            incident_id,
            effect_id: input.effect_id,
            transition_id: input.transition_id,
            attempt_id: input.attempt_id,
            detected_at: input.detected_at.to_owned(),
            from_state,
            reason: input.reason,
            evidence_ref: transition.evidence_ref,
            global_seq,
        })
    }

    pub fn reports(&self) -> Result<Vec<ManualReviewReport>, ManualReviewError> {
        let raw = {
            let mut statement = self.connection.prepare(
                "SELECT incident_id, detected_at, affected_refs, resolution \
                 FROM recovery_incidents WHERE kind = ?1 ORDER BY rowid ASC",
            )?;
            let rows = statement.query_map(params![INCIDENT_KIND], |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, Option<Vec<u8>>>(3)?,
                ))
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let mut reports = Vec::with_capacity(raw.len());
        for (incident_id, detected_at, affected_refs, resolution) in raw {
            let (effect_id, transition_id, attempt_id) = decode_affected_refs(&affected_refs)?;
            let reason_bytes = resolution.ok_or(ManualReviewError::InvalidStoredRecord)?;
            let reason_text = std::str::from_utf8(&reason_bytes)
                .map_err(|_| ManualReviewError::InvalidStoredRecord)?;
            let reason = ManualReviewReason::parse(reason_text)
                .ok_or(ManualReviewError::InvalidStoredRecord)?;
            let (from_state, evidence_ref, global_seq) = self
                .connection
                .query_row(
                    "SELECT from_state, evidence_ref, global_seq FROM effect_transitions \
                     WHERE transition_id = ?1 AND effect_id = ?2 AND to_state = 'manual_review'",
                    params![id_blob(transition_id.0), id_blob(effect_id.0)],
                    |row| {
                        Ok((
                            row.get::<_, Option<String>>(0)?,
                            row.get::<_, Option<Vec<u8>>>(1)?,
                            row.get::<_, i64>(2)?,
                        ))
                    },
                )
                .optional()?
                .ok_or(ManualReviewError::InvalidStoredRecord)?;
            let incident_id: [u8; 16] = incident_id
                .try_into()
                .map_err(|_| ManualReviewError::InvalidStoredRecord)?;
            reports.push(ManualReviewReport {
                incident_id,
                effect_id,
                transition_id,
                attempt_id,
                detected_at,
                from_state: from_state.ok_or(ManualReviewError::InvalidStoredRecord)?,
                reason,
                evidence_ref,
                global_seq: i64_to_seq(global_seq)?,
            });
        }
        Ok(reports)
    }
}

fn validate_input(input: &PlaceEffectInManualReview<'_>) -> Result<(), ManualReviewError> {
    if input.detected_at.is_empty() {
        return Err(ManualReviewError::InvalidMetadata);
    }
    if input
        .evidence_ref
        .is_some_and(|evidence| evidence.len() > MAX_EVIDENCE_BYTES)
    {
        return Err(ManualReviewError::EvidenceTooLarge);
    }
    Ok(())
}

fn validate_attempt(
    transaction: &Transaction<'_>,
    effect_id: EffectId,
    attempt_id: Option<EffectAttemptId>,
) -> Result<(), ManualReviewError> {
    let Some(attempt_id) = attempt_id else {
        return Ok(());
    };
    let stored_effect = transaction
        .query_row(
            "SELECT effect_id FROM effect_attempts WHERE attempt_id = ?1",
            params![id_blob(attempt_id.0)],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .optional()?
        .ok_or(ManualReviewError::AttemptNotFound(attempt_id))?;
    if stored_effect != id_blob(effect_id.0) {
        return Err(ManualReviewError::AttemptEffectMismatch {
            attempt_id,
            effect_id,
        });
    }
    Ok(())
}

fn insert_transition(
    transaction: &Transaction<'_>,
    stored: &StoredEffectTransition,
) -> Result<(), ManualReviewError> {
    let transition_blob = id_blob(stored.transition_id.0);
    let effect_blob = id_blob(stored.effect_id.0);
    let attempt_blob = stored.attempt_id.map(|attempt_id| id_blob(attempt_id.0));
    let event_blob = id_blob(stored.event_id.0);
    transaction.execute(
        "INSERT INTO effect_transitions (transition_id, effect_id, global_seq, from_state, to_state, \
         attempt_id, reason_code, evidence_ref, event_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            &transition_blob,
            &effect_blob,
            seq_to_i64(stored.global_seq)?,
            stored.from_state.as_deref(),
            stored.to_state.as_str(),
            attempt_blob.as_deref(),
            stored.reason_code.as_deref(),
            stored.evidence_ref.as_deref(),
            &event_blob,
        ],
    )?;
    security_audit::append_effect_transition(
        transaction,
        EffectTransitionAuditInput {
            transition_id: &transition_blob,
            effect_id: &effect_blob,
            global_seq: stored.global_seq,
            from_state: stored.from_state.as_deref(),
            to_state: stored.to_state.as_str(),
            attempt_id: attempt_blob.as_deref(),
            reason_code: stored.reason_code.as_deref(),
            evidence_ref: stored.evidence_ref.as_deref(),
            event_id: &event_blob,
        },
    )
    .map_err(|error| ManualReviewError::SecurityAudit(error.to_string()))?;
    Ok(())
}

fn next_global_seq(transaction: &Transaction<'_>) -> Result<u64, ManualReviewError> {
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
        .ok_or(ManualReviewError::SequenceOverflow)
}

fn manual_review_incident_id(effect_id: EffectId, transition_id: EffectTransitionId) -> [u8; 16] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(MANUAL_REVIEW_DOMAIN);
    hasher.update(&effect_id.0.to_be_bytes());
    hasher.update(&transition_id.0.to_be_bytes());
    let digest = hasher.finalize();
    let mut incident_id = [0_u8; 16];
    incident_id.copy_from_slice(&digest.as_bytes()[..16]);
    incident_id
}

fn encode_affected_refs(
    effect_id: EffectId,
    transition_id: EffectTransitionId,
    attempt_id: Option<EffectAttemptId>,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(AFFECTED_REFS_LEN);
    bytes.push(AFFECTED_REFS_VERSION);
    bytes.extend_from_slice(&effect_id.0.to_be_bytes());
    bytes.extend_from_slice(&transition_id.0.to_be_bytes());
    match attempt_id {
        Some(attempt_id) => {
            bytes.push(1);
            bytes.extend_from_slice(&attempt_id.0.to_be_bytes());
        }
        None => {
            bytes.push(0);
            bytes.extend_from_slice(&[0_u8; 16]);
        }
    }
    bytes
}

fn decode_affected_refs(
    bytes: &[u8],
) -> Result<(EffectId, EffectTransitionId, Option<EffectAttemptId>), ManualReviewError> {
    if bytes.len() != AFFECTED_REFS_LEN || bytes[0] != AFFECTED_REFS_VERSION {
        return Err(ManualReviewError::InvalidStoredRecord);
    }
    let effect_id = EffectId(u128::from_be_bytes(
        bytes[1..17]
            .try_into()
            .map_err(|_| ManualReviewError::InvalidStoredRecord)?,
    ));
    let transition_id = EffectTransitionId(u128::from_be_bytes(
        bytes[17..33]
            .try_into()
            .map_err(|_| ManualReviewError::InvalidStoredRecord)?,
    ));
    let attempt_value = u128::from_be_bytes(
        bytes[34..50]
            .try_into()
            .map_err(|_| ManualReviewError::InvalidStoredRecord)?,
    );
    let attempt_id = match bytes[33] {
        0 if attempt_value == 0 => None,
        1 => Some(EffectAttemptId(attempt_value)),
        _ => return Err(ManualReviewError::InvalidStoredRecord),
    };
    Ok((effect_id, transition_id, attempt_id))
}

fn id_blob(value: u128) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

fn seq_to_i64(value: u64) -> Result<i64, ManualReviewError> {
    i64::try_from(value).map_err(|_| ManualReviewError::SequenceOverflow)
}

fn i64_to_seq(value: i64) -> Result<u64, ManualReviewError> {
    u64::try_from(value).map_err(|_| ManualReviewError::InvalidStoredRecord)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::encode_effect_dependencies;
    use crate::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
    use golam_core::SessionId;
    use golam_core::paths::RuntimeLayout;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-manual-review-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    fn create_effect(
        store: &mut EffectStore,
        effect_id: EffectId,
        through_state: &'static str,
        seed: u128,
    ) {
        let dependencies = encode_effect_dependencies(&[]).unwrap();
        store
            .propose(ProposeEffect {
                effect_id,
                session_id: SessionId(seed + 1),
                requested_by: "owner",
                action: "sim.write",
                resource: "sim:manual-review",
                risk_class: "synthetic",
                execution_semantics: "irreversible",
                idempotency_key: None,
                preconditions: b"[]",
                dependencies: &dependencies,
                payload_hash: [3; 32],
                proposed_event_id: EventId(seed + 2),
                transition_id: EffectTransitionId(seed + 3),
            })
            .unwrap();
        if through_state == "proposed" {
            return;
        }
        transition(store, effect_id, "proposed", "authorized", seed + 4);
        if through_state == "authorized" {
            return;
        }
        transition(store, effect_id, "authorized", "executing", seed + 5);
        if through_state == "executing" {
            return;
        }
        transition(store, effect_id, "executing", "unknown_outcome", seed + 6);
        if through_state == "unknown_outcome" {
            return;
        }
        transition(store, effect_id, "unknown_outcome", "reconciling", seed + 7);
    }

    fn transition(
        store: &mut EffectStore,
        effect_id: EffectId,
        from: &'static str,
        to: &'static str,
        seed: u128,
    ) {
        store
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(seed),
                effect_id,
                expected_state: from,
                next_state: to,
                attempt_id: None,
                reason_code: Some("test_transition"),
                evidence_ref: None,
                event_id: EventId(seed + 1000),
            })
            .unwrap();
    }

    #[test]
    fn unknown_outcome_enters_manual_review_with_atomic_durable_report() {
        let (runtime, authority) = authority();
        let effect_id = EffectId(9000);
        let mut effects = EffectStore::open(&authority).unwrap();
        create_effect(&mut effects, effect_id, "unknown_outcome", 10_000);
        drop(effects);

        let mut reviews = ManualReviewStore::open(&authority).unwrap();
        let report = reviews
            .place(PlaceEffectInManualReview {
                effect_id,
                transition_id: EffectTransitionId(20_000),
                attempt_id: None,
                detected_at: "2026-08-25T10:40:00Z",
                reason: ManualReviewReason::UnreconcilableAmbiguity,
                evidence_ref: Some(b"ambiguous-ack"),
                event_id: EventId(20_001),
            })
            .unwrap();
        assert_eq!(report.from_state, "unknown_outcome");
        drop(reviews);

        let effects = EffectStore::open(&authority).unwrap();
        assert_eq!(
            effects.current_state(effect_id).unwrap().as_deref(),
            Some("manual_review")
        );
        drop(effects);
        let reopened = ManualReviewStore::open(&authority).unwrap();
        assert_eq!(reopened.reports().unwrap(), vec![report]);
        drop(reopened);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn reconciling_can_escalate_to_manual_review() {
        let (runtime, authority) = authority();
        let effect_id = EffectId(9100);
        let mut effects = EffectStore::open(&authority).unwrap();
        create_effect(&mut effects, effect_id, "reconciling", 11_000);
        drop(effects);

        let mut reviews = ManualReviewStore::open(&authority).unwrap();
        let report = reviews
            .place(PlaceEffectInManualReview {
                effect_id,
                transition_id: EffectTransitionId(21_000),
                attempt_id: None,
                detected_at: "2026-08-25T10:41:00Z",
                reason: ManualReviewReason::ReconciliationExhausted,
                evidence_ref: None,
                event_id: EventId(21_001),
            })
            .unwrap();
        assert_eq!(report.from_state, "reconciling");
        assert_eq!(reviews.reports().unwrap(), vec![report]);
        drop(reviews);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn non_ambiguous_state_is_rejected_without_report() {
        let (runtime, authority) = authority();
        let effect_id = EffectId(9200);
        let mut effects = EffectStore::open(&authority).unwrap();
        create_effect(&mut effects, effect_id, "executing", 12_000);
        drop(effects);

        let mut reviews = ManualReviewStore::open(&authority).unwrap();
        assert!(matches!(
            reviews.place(PlaceEffectInManualReview {
                effect_id,
                transition_id: EffectTransitionId(22_000),
                attempt_id: None,
                detected_at: "2026-08-25T10:42:00Z",
                reason: ManualReviewReason::UnreconcilableAmbiguity,
                evidence_ref: None,
                event_id: EventId(22_001),
            }),
            Err(ManualReviewError::InvalidSourceState { actual, .. }) if actual == "executing"
        ));
        assert!(reviews.reports().unwrap().is_empty());
        drop(reviews);

        let effects = EffectStore::open(&authority).unwrap();
        assert_eq!(
            effects.current_state(effect_id).unwrap().as_deref(),
            Some("executing")
        );
        drop(effects);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
