#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{EffectAttemptId, EffectId, EffectTransitionId, EventId, SessionId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::security_audit::{
    self, EffectAttemptFinishedAuditInput, EffectAttemptStartedAuditInput, EffectIntentAuditInput,
    EffectTransitionAuditInput,
};
use crate::storage::{AuthorityStore, StorageError};

const EFFECT_STATES: &[&str] = &[
    "proposed",
    "denied",
    "authorized",
    "approval_required",
    "executing",
    "succeeded",
    "failed",
    "unknown_outcome",
    "reconciling",
    "manual_review",
];

const ATTEMPT_OUTCOMES: &[&str] = &["success", "failure", "unknown"];

pub struct ProposeEffect<'a> {
    pub effect_id: EffectId,
    pub session_id: SessionId,
    pub requested_by: &'a str,
    pub action: &'a str,
    pub resource: &'a str,
    pub risk_class: &'a str,
    pub execution_semantics: &'a str,
    pub idempotency_key: Option<&'a str>,
    pub preconditions: &'a [u8],
    pub dependencies: &'a [u8],
    pub payload_hash: [u8; 32],
    pub proposed_event_id: EventId,
    pub transition_id: EffectTransitionId,
}

pub struct CompareAndSwapEffect<'a> {
    pub transition_id: EffectTransitionId,
    pub effect_id: EffectId,
    pub expected_state: &'a str,
    pub next_state: &'a str,
    pub attempt_id: Option<EffectAttemptId>,
    pub reason_code: Option<&'a str>,
    pub evidence_ref: Option<&'a [u8]>,
    pub event_id: EventId,
}

pub struct StartEffectAttempt<'a> {
    pub attempt_id: EffectAttemptId,
    pub effect_id: EffectId,
    pub handler_id: &'a str,
    pub handler_version: &'a str,
    pub dispatch_token: &'a [u8],
    pub started_at: &'a str,
}

pub struct FinishEffectAttempt<'a> {
    pub attempt_id: EffectAttemptId,
    pub finished_at: &'a str,
    pub outcome: &'a str,
    pub receipt: Option<&'a [u8]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredEffectTransition {
    pub transition_id: EffectTransitionId,
    pub effect_id: EffectId,
    pub global_seq: u64,
    pub from_state: Option<String>,
    pub to_state: String,
    pub attempt_id: Option<EffectAttemptId>,
    pub reason_code: Option<String>,
    pub evidence_ref: Option<Vec<u8>>,
    pub event_id: EventId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredEffectAttempt {
    pub attempt_id: EffectAttemptId,
    pub effect_id: EffectId,
    pub started_global_seq: u64,
    pub handler_id: String,
    pub handler_version: String,
    pub dispatch_token: Vec<u8>,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub outcome: String,
    pub receipt: Option<Vec<u8>>,
}

#[derive(Debug)]
pub enum EffectStoreError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    SecurityAudit(String),
    InvalidMetadata,
    InvalidState(String),
    InvalidTransition { from: String, to: String },
    InvalidAttemptOutcome(String),
    EffectAlreadyExists(EffectId),
    EffectNotFound(EffectId),
    MissingCurrentState(EffectId),
    StaleState { expected: String, actual: String },
    AttemptAlreadyExists(EffectAttemptId),
    AttemptNotFound(EffectAttemptId),
    AttemptAlreadyFinished(EffectAttemptId),
    SequenceOverflow,
    InvalidStoredRecord,
}

impl fmt::Display for EffectStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "effect store authority error: {error}"),
            Self::Sqlite(error) => write!(f, "effect store sqlite error: {error}"),
            Self::SecurityAudit(error) => write!(f, "effect integrity-chain error: {error}"),
            Self::InvalidMetadata => f.write_str("effect request metadata must be non-empty"),
            Self::InvalidState(state) => write!(f, "invalid effect state: {state}"),
            Self::InvalidTransition { from, to } => {
                write!(f, "invalid effect state transition: {from} -> {to}")
            }
            Self::InvalidAttemptOutcome(outcome) => {
                write!(f, "invalid effect attempt outcome: {outcome}")
            }
            Self::EffectAlreadyExists(effect_id) => {
                write!(f, "effect already exists: {}", effect_id.0)
            }
            Self::EffectNotFound(effect_id) => write!(f, "effect not found: {}", effect_id.0),
            Self::MissingCurrentState(effect_id) => {
                write!(f, "effect has no current transition: {}", effect_id.0)
            }
            Self::StaleState { expected, actual } => {
                write!(
                    f,
                    "stale effect state: expected {expected}, actual {actual}"
                )
            }
            Self::AttemptAlreadyExists(attempt_id) => {
                write!(f, "effect attempt already exists: {}", attempt_id.0)
            }
            Self::AttemptNotFound(attempt_id) => {
                write!(f, "effect attempt not found: {}", attempt_id.0)
            }
            Self::AttemptAlreadyFinished(attempt_id) => {
                write!(f, "effect attempt already finished: {}", attempt_id.0)
            }
            Self::SequenceOverflow => f.write_str("effect global sequence overflow"),
            Self::InvalidStoredRecord => f.write_str("stored effect record is malformed"),
        }
    }
}

impl Error for EffectStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for EffectStoreError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for EffectStoreError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub struct EffectStore {
    connection: Connection,
}

impl EffectStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, EffectStoreError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn propose(
        &mut self,
        input: ProposeEffect<'_>,
    ) -> Result<StoredEffectTransition, EffectStoreError> {
        validate_proposal(&input)?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let effect_blob = id_blob(input.effect_id.0);
        if transaction
            .query_row(
                "SELECT 1 FROM effect_intents WHERE effect_id = ?1 LIMIT 1",
                params![&effect_blob],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some()
        {
            return Err(EffectStoreError::EffectAlreadyExists(input.effect_id));
        }

        let session_blob = id_blob(input.session_id.0);
        let proposed_event_blob = id_blob(input.proposed_event_id.0);
        transaction.execute(
            "INSERT INTO effect_intents (effect_id, session_id, requested_by, action, resource, \
             risk_class, execution_semantics, idempotency_key, preconditions, dependencies, \
             payload_hash, proposed_event_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                &effect_blob,
                &session_blob,
                input.requested_by,
                input.action,
                input.resource,
                input.risk_class,
                input.execution_semantics,
                input.idempotency_key,
                input.preconditions,
                input.dependencies,
                &input.payload_hash[..],
                &proposed_event_blob,
            ],
        )?;
        security_audit::append_effect_intent(
            &transaction,
            EffectIntentAuditInput {
                effect_id: &effect_blob,
                session_id: &session_blob,
                requested_by: input.requested_by,
                action: input.action,
                resource: input.resource,
                risk_class: input.risk_class,
                execution_semantics: input.execution_semantics,
                idempotency_key: input.idempotency_key,
                preconditions: input.preconditions,
                dependencies: input.dependencies,
                payload_hash: &input.payload_hash,
                proposed_event_id: &proposed_event_blob,
            },
        )
        .map_err(|error| EffectStoreError::SecurityAudit(error.to_string()))?;

        let stored = StoredEffectTransition {
            transition_id: input.transition_id,
            effect_id: input.effect_id,
            global_seq: next_global_seq(&transaction)?,
            from_state: None,
            to_state: "proposed".to_owned(),
            attempt_id: None,
            reason_code: None,
            evidence_ref: None,
            event_id: input.proposed_event_id,
        };
        insert_transition(&transaction, &stored)?;
        transaction.commit()?;
        Ok(stored)
    }

    pub fn compare_and_swap(
        &mut self,
        input: CompareAndSwapEffect<'_>,
    ) -> Result<StoredEffectTransition, EffectStoreError> {
        validate_state(input.expected_state)?;
        validate_state(input.next_state)?;
        validate_transition(input.expected_state, input.next_state)?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let effect_blob = id_blob(input.effect_id.0);
        if transaction
            .query_row(
                "SELECT 1 FROM effect_intents WHERE effect_id = ?1 LIMIT 1",
                params![&effect_blob],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_none()
        {
            return Err(EffectStoreError::EffectNotFound(input.effect_id));
        }

        let actual = transaction
            .query_row(
                "SELECT to_state FROM effect_transitions WHERE effect_id = ?1 \
                 ORDER BY global_seq DESC LIMIT 1",
                params![&effect_blob],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or(EffectStoreError::MissingCurrentState(input.effect_id))?;
        validate_state(&actual)?;
        if actual != input.expected_state {
            return Err(EffectStoreError::StaleState {
                expected: input.expected_state.to_owned(),
                actual,
            });
        }

        let stored = StoredEffectTransition {
            transition_id: input.transition_id,
            effect_id: input.effect_id,
            global_seq: next_global_seq(&transaction)?,
            from_state: Some(input.expected_state.to_owned()),
            to_state: input.next_state.to_owned(),
            attempt_id: input.attempt_id,
            reason_code: input.reason_code.map(str::to_owned),
            evidence_ref: input.evidence_ref.map(<[u8]>::to_vec),
            event_id: input.event_id,
        };
        insert_transition(&transaction, &stored)?;
        transaction.commit()?;
        Ok(stored)
    }

    pub fn start_attempt(
        &mut self,
        input: StartEffectAttempt<'_>,
    ) -> Result<StoredEffectAttempt, EffectStoreError> {
        validate_start_attempt(&input)?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let attempt_blob = id_blob(input.attempt_id.0);
        if transaction
            .query_row(
                "SELECT 1 FROM effect_attempts WHERE attempt_id = ?1 LIMIT 1",
                params![&attempt_blob],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some()
        {
            return Err(EffectStoreError::AttemptAlreadyExists(input.attempt_id));
        }

        let effect_blob = id_blob(input.effect_id.0);
        let started_global_seq = transaction
            .query_row(
                "SELECT global_seq FROM effect_transitions WHERE effect_id = ?1 \
                 ORDER BY global_seq DESC LIMIT 1",
                params![&effect_blob],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .ok_or(EffectStoreError::MissingCurrentState(input.effect_id))?;
        let started_global_seq = i64_to_seq(started_global_seq)?;

        transaction.execute(
            "INSERT INTO effect_attempts (attempt_id, effect_id, started_global_seq, handler_id, \
             handler_version, dispatch_token, started_at, finished_at, outcome, receipt) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, 'unknown', NULL)",
            params![
                &attempt_blob,
                &effect_blob,
                seq_to_i64(started_global_seq)?,
                input.handler_id,
                input.handler_version,
                input.dispatch_token,
                input.started_at,
            ],
        )?;
        security_audit::append_effect_attempt_started(
            &transaction,
            EffectAttemptStartedAuditInput {
                attempt_id: &attempt_blob,
                effect_id: &effect_blob,
                started_global_seq,
                handler_id: input.handler_id,
                handler_version: input.handler_version,
                dispatch_token: input.dispatch_token,
                started_at: input.started_at,
            },
        )
        .map_err(|error| EffectStoreError::SecurityAudit(error.to_string()))?;
        transaction.commit()?;

        Ok(StoredEffectAttempt {
            attempt_id: input.attempt_id,
            effect_id: input.effect_id,
            started_global_seq,
            handler_id: input.handler_id.to_owned(),
            handler_version: input.handler_version.to_owned(),
            dispatch_token: input.dispatch_token.to_vec(),
            started_at: input.started_at.to_owned(),
            finished_at: None,
            outcome: "unknown".to_owned(),
            receipt: None,
        })
    }

    pub fn finish_attempt(
        &mut self,
        input: FinishEffectAttempt<'_>,
    ) -> Result<StoredEffectAttempt, EffectStoreError> {
        validate_finish_attempt(&input)?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let attempt_blob = id_blob(input.attempt_id.0);
        let updated = transaction.execute(
            "UPDATE effect_attempts SET finished_at = ?1, outcome = ?2, receipt = ?3 \
             WHERE attempt_id = ?4 AND finished_at IS NULL",
            params![
                input.finished_at,
                input.outcome,
                input.receipt,
                &attempt_blob,
            ],
        )?;
        if updated != 1 {
            let existing = transaction
                .query_row(
                    "SELECT finished_at FROM effect_attempts WHERE attempt_id = ?1",
                    params![&attempt_blob],
                    |row| row.get::<_, Option<String>>(0),
                )
                .optional()?;
            return match existing {
                None => Err(EffectStoreError::AttemptNotFound(input.attempt_id)),
                Some(Some(_)) => Err(EffectStoreError::AttemptAlreadyFinished(input.attempt_id)),
                Some(None) => Err(EffectStoreError::InvalidStoredRecord),
            };
        }
        security_audit::append_effect_attempt_finished(
            &transaction,
            EffectAttemptFinishedAuditInput {
                attempt_id: &attempt_blob,
                finished_at: input.finished_at,
                outcome: input.outcome,
                receipt: input.receipt,
            },
        )
        .map_err(|error| EffectStoreError::SecurityAudit(error.to_string()))?;
        transaction.commit()?;
        self.attempt(input.attempt_id)?
            .ok_or(EffectStoreError::AttemptNotFound(input.attempt_id))
    }

    pub fn current_state(&self, effect_id: EffectId) -> Result<Option<String>, EffectStoreError> {
        let state = self
            .connection
            .query_row(
                "SELECT to_state FROM effect_transitions WHERE effect_id = ?1 \
                 ORDER BY global_seq DESC LIMIT 1",
                params![id_blob(effect_id.0)],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        if let Some(state) = &state {
            validate_state(state)?;
        }
        Ok(state)
    }

    pub fn attempt(
        &self,
        attempt_id: EffectAttemptId,
    ) -> Result<Option<StoredEffectAttempt>, EffectStoreError> {
        let row = self
            .connection
            .query_row(
                "SELECT effect_id, started_global_seq, handler_id, handler_version, dispatch_token, \
                 started_at, finished_at, outcome, receipt FROM effect_attempts WHERE attempt_id = ?1",
                params![id_blob(attempt_id.0)],
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

        let Some((
            effect_id,
            started_global_seq,
            handler_id,
            handler_version,
            dispatch_token,
            started_at,
            finished_at,
            outcome,
            receipt,
        )) = row
        else {
            return Ok(None);
        };
        validate_attempt_outcome(&outcome)?;
        Ok(Some(StoredEffectAttempt {
            attempt_id,
            effect_id: EffectId(id_from_blob(effect_id)?),
            started_global_seq: i64_to_seq(started_global_seq)?,
            handler_id,
            handler_version,
            dispatch_token,
            started_at,
            finished_at,
            outcome,
            receipt,
        }))
    }

    pub fn transition_count(&self, effect_id: EffectId) -> Result<usize, EffectStoreError> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM effect_transitions WHERE effect_id = ?1",
            params![id_blob(effect_id.0)],
            |row| row.get(0),
        )?;
        usize::try_from(count).map_err(|_| EffectStoreError::InvalidStoredRecord)
    }

    pub fn attempt_count(&self, effect_id: EffectId) -> Result<usize, EffectStoreError> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM effect_attempts WHERE effect_id = ?1",
            params![id_blob(effect_id.0)],
            |row| row.get(0),
        )?;
        usize::try_from(count).map_err(|_| EffectStoreError::InvalidStoredRecord)
    }
}

fn validate_proposal(input: &ProposeEffect<'_>) -> Result<(), EffectStoreError> {
    if input.requested_by.is_empty()
        || input.action.is_empty()
        || input.resource.is_empty()
        || input.risk_class.is_empty()
        || input.execution_semantics.is_empty()
    {
        return Err(EffectStoreError::InvalidMetadata);
    }
    Ok(())
}

fn validate_start_attempt(input: &StartEffectAttempt<'_>) -> Result<(), EffectStoreError> {
    if input.handler_id.is_empty()
        || input.handler_version.is_empty()
        || input.dispatch_token.is_empty()
        || input.started_at.is_empty()
    {
        return Err(EffectStoreError::InvalidMetadata);
    }
    Ok(())
}

fn validate_finish_attempt(input: &FinishEffectAttempt<'_>) -> Result<(), EffectStoreError> {
    if input.finished_at.is_empty() {
        return Err(EffectStoreError::InvalidMetadata);
    }
    validate_attempt_outcome(input.outcome)
}

fn validate_state(state: &str) -> Result<(), EffectStoreError> {
    if EFFECT_STATES.contains(&state) {
        Ok(())
    } else {
        Err(EffectStoreError::InvalidState(state.to_owned()))
    }
}

fn validate_transition(from: &str, to: &str) -> Result<(), EffectStoreError> {
    if matches!(
        (from, to),
        ("proposed", "denied")
            | ("proposed", "authorized")
            | ("authorized", "approval_required")
            | ("authorized", "executing")
            | ("approval_required", "authorized")
            | ("approval_required", "denied")
            | ("executing", "succeeded")
            | ("executing", "failed")
            | ("executing", "unknown_outcome")
            | ("unknown_outcome", "reconciling")
            | ("reconciling", "succeeded")
            | ("reconciling", "failed")
            | ("reconciling", "manual_review")
    ) {
        Ok(())
    } else {
        Err(EffectStoreError::InvalidTransition {
            from: from.to_owned(),
            to: to.to_owned(),
        })
    }
}

fn validate_attempt_outcome(outcome: &str) -> Result<(), EffectStoreError> {
    if ATTEMPT_OUTCOMES.contains(&outcome) {
        Ok(())
    } else {
        Err(EffectStoreError::InvalidAttemptOutcome(outcome.to_owned()))
    }
}

fn insert_transition(
    transaction: &Transaction<'_>,
    stored: &StoredEffectTransition,
) -> Result<(), EffectStoreError> {
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
    .map_err(|error| EffectStoreError::SecurityAudit(error.to_string()))?;
    Ok(())
}

fn next_global_seq(transaction: &Transaction<'_>) -> Result<u64, EffectStoreError> {
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
        .ok_or(EffectStoreError::SequenceOverflow)
}

fn id_blob(value: u128) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

fn id_from_blob(value: Vec<u8>) -> Result<u128, EffectStoreError> {
    let bytes: [u8; 16] = value
        .try_into()
        .map_err(|_| EffectStoreError::InvalidStoredRecord)?;
    Ok(u128::from_be_bytes(bytes))
}

fn seq_to_i64(value: u64) -> Result<i64, EffectStoreError> {
    i64::try_from(value).map_err(|_| EffectStoreError::SequenceOverflow)
}

fn i64_to_seq(value: i64) -> Result<u64, EffectStoreError> {
    u64::try_from(value).map_err(|_| EffectStoreError::InvalidStoredRecord)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let runtime = RuntimeLayout::initialize(
            std::env::temp_dir().join(format!("golam-effect-store-{}-{t}-{n}", std::process::id())),
        )
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    fn proposal(effect_id: EffectId) -> ProposeEffect<'static> {
        ProposeEffect {
            effect_id,
            session_id: SessionId(7),
            requested_by: "owner",
            action: "sim.write",
            resource: "sim:target",
            risk_class: "synthetic",
            execution_semantics: "at_most_once",
            idempotency_key: Some("stable-key"),
            preconditions: b"[]",
            dependencies: b"[]",
            payload_hash: [4; 32],
            proposed_event_id: EventId(800),
            transition_id: EffectTransitionId(900),
        }
    }

    #[test]
    fn proposal_and_compare_and_swap_are_durable() {
        let (runtime, authority) = authority();
        let effect_id = EffectId(42);
        let mut store = EffectStore::open(&authority).unwrap();
        let proposed = store.propose(proposal(effect_id)).unwrap();
        assert_eq!(proposed.global_seq, 1);
        assert_eq!(proposed.to_state, "proposed");

        let authorized = store
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(901),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: Some("bootstrap_explicit_allow"),
                evidence_ref: None,
                event_id: EventId(801),
            })
            .unwrap();
        assert_eq!(authorized.global_seq, 2);
        drop(store);

        let reopened = EffectStore::open(&authority).unwrap();
        assert_eq!(
            reopened.current_state(effect_id).unwrap().as_deref(),
            Some("authorized")
        );
        assert_eq!(reopened.transition_count(effect_id).unwrap(), 2);
        drop(reopened);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn invalid_fsm_transition_is_rejected_without_mutation() {
        let (runtime, authority) = authority();
        let effect_id = EffectId(46);
        let mut store = EffectStore::open(&authority).unwrap();
        store.propose(proposal(effect_id)).unwrap();
        assert!(matches!(
            store.compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(930),
                effect_id,
                expected_state: "proposed",
                next_state: "succeeded",
                attempt_id: None,
                reason_code: Some("illegal_skip"),
                evidence_ref: None,
                event_id: EventId(830),
            }),
            Err(EffectStoreError::InvalidTransition { ref from, ref to })
                if from == "proposed" && to == "succeeded"
        ));
        assert_eq!(store.transition_count(effect_id).unwrap(), 1);
        assert_eq!(
            store.current_state(effect_id).unwrap().as_deref(),
            Some("proposed")
        );
        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn stale_compare_and_swap_does_not_consume_sequence() {
        let (runtime, authority) = authority();
        let effect_id = EffectId(43);
        let mut store = EffectStore::open(&authority).unwrap();
        store.propose(proposal(effect_id)).unwrap();
        store
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(910),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: None,
                evidence_ref: None,
                event_id: EventId(810),
            })
            .unwrap();
        assert!(matches!(
            store.compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(911),
                effect_id,
                expected_state: "proposed",
                next_state: "denied",
                attempt_id: None,
                reason_code: Some("stale-writer"),
                evidence_ref: None,
                event_id: EventId(811),
            }),
            Err(EffectStoreError::StaleState { .. })
        ));
        let executing = store
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(912),
                effect_id,
                expected_state: "authorized",
                next_state: "executing",
                attempt_id: Some(EffectAttemptId(700)),
                reason_code: None,
                evidence_ref: None,
                event_id: EventId(812),
            })
            .unwrap();
        assert_eq!(executing.global_seq, 3);
        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn attempts_are_durable_and_anchor_to_latest_canonical_transition() {
        let (runtime, authority) = authority();
        let effect_id = EffectId(44);
        let attempt_id = EffectAttemptId(720);
        let mut store = EffectStore::open(&authority).unwrap();
        store.propose(proposal(effect_id)).unwrap();
        let authorized = store
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(920),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: Some("authorized"),
                evidence_ref: None,
                event_id: EventId(820),
            })
            .unwrap();
        assert_eq!(authorized.global_seq, 2);

        let started = store
            .start_attempt(StartEffectAttempt {
                attempt_id,
                effect_id,
                handler_id: "sim-at-most-once",
                handler_version: "1",
                dispatch_token: b"dispatch-720",
                started_at: "2026-08-25T09:40:00Z",
            })
            .unwrap();
        assert_eq!(started.started_global_seq, authorized.global_seq);
        assert_eq!(started.outcome, "unknown");
        assert_eq!(started.finished_at, None);

        let executing = store
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(921),
                effect_id,
                expected_state: "authorized",
                next_state: "executing",
                attempt_id: Some(attempt_id),
                reason_code: None,
                evidence_ref: None,
                event_id: EventId(821),
            })
            .unwrap();
        assert_eq!(executing.global_seq, 3);

        let finished = store
            .finish_attempt(FinishEffectAttempt {
                attempt_id,
                finished_at: "2026-08-25T09:40:01Z",
                outcome: "success",
                receipt: Some(b"receipt-720"),
            })
            .unwrap();
        assert_eq!(finished.started_global_seq, 2);
        assert_eq!(finished.outcome, "success");
        assert_eq!(finished.receipt.as_deref(), Some(b"receipt-720".as_slice()));
        drop(store);

        let reopened = EffectStore::open(&authority).unwrap();
        assert_eq!(reopened.attempt_count(effect_id).unwrap(), 1);
        assert_eq!(reopened.attempt(attempt_id).unwrap(), Some(finished));
        drop(reopened);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn duplicate_or_refinished_attempts_fail_closed() {
        let (runtime, authority) = authority();
        let effect_id = EffectId(45);
        let attempt_id = EffectAttemptId(730);
        let mut store = EffectStore::open(&authority).unwrap();
        store.propose(proposal(effect_id)).unwrap();
        store
            .start_attempt(StartEffectAttempt {
                attempt_id,
                effect_id,
                handler_id: "sim-read",
                handler_version: "1",
                dispatch_token: b"dispatch-730",
                started_at: "2026-08-25T09:41:00Z",
            })
            .unwrap();
        assert!(matches!(
            store.start_attempt(StartEffectAttempt {
                attempt_id,
                effect_id,
                handler_id: "sim-read",
                handler_version: "1",
                dispatch_token: b"dispatch-730-duplicate",
                started_at: "2026-08-25T09:41:01Z",
            }),
            Err(EffectStoreError::AttemptAlreadyExists(id)) if id == attempt_id
        ));
        store
            .finish_attempt(FinishEffectAttempt {
                attempt_id,
                finished_at: "2026-08-25T09:41:02Z",
                outcome: "unknown",
                receipt: None,
            })
            .unwrap();
        assert!(matches!(
            store.finish_attempt(FinishEffectAttempt {
                attempt_id,
                finished_at: "2026-08-25T09:41:03Z",
                outcome: "failure",
                receipt: None,
            }),
            Err(EffectStoreError::AttemptAlreadyFinished(id)) if id == attempt_id
        ));
        assert!(matches!(
            store.finish_attempt(FinishEffectAttempt {
                attempt_id: EffectAttemptId(999),
                finished_at: "2026-08-25T09:41:04Z",
                outcome: "failure",
                receipt: None,
            }),
            Err(EffectStoreError::AttemptNotFound(EffectAttemptId(999)))
        ));
        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
