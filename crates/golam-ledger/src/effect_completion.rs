#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{EffectAttemptId, EffectId, EffectTransitionId, EventId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::effects::{StoredEffectAttempt, StoredEffectTransition};
use crate::storage::{AuthorityStore, StorageError};

const MAX_RECEIPT_BYTES: usize = 64 * 1024;
const MAX_EVIDENCE_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionCompletion {
    Succeeded,
    Failed,
    UnknownOutcome,
}

impl ExecutionCompletion {
    const fn attempt_outcome(self) -> &'static str {
        match self {
            Self::Succeeded => "success",
            Self::Failed => "failure",
            Self::UnknownOutcome => "unknown",
        }
    }

    const fn target_state(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::UnknownOutcome => "unknown_outcome",
        }
    }
}

pub struct CompleteEffectExecution<'a> {
    pub effect_id: EffectId,
    pub attempt_id: EffectAttemptId,
    pub transition_id: EffectTransitionId,
    pub event_id: EventId,
    pub finished_at: &'a str,
    pub completion: ExecutionCompletion,
    pub reason_code: Option<&'a str>,
    pub evidence_ref: Option<&'a [u8]>,
    pub receipt: Option<&'a [u8]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletedEffectExecution {
    pub attempt: StoredEffectAttempt,
    pub transition: StoredEffectTransition,
}

#[derive(Debug)]
pub enum EffectCompletionError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    InvalidMetadata,
    ReceiptTooLarge,
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
    AttemptAlreadyFinished(EffectAttemptId),
    SequenceOverflow,
    InvalidStoredRecord,
}

impl fmt::Display for EffectCompletionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "effect completion authority error: {error}"),
            Self::Sqlite(error) => write!(f, "effect completion sqlite error: {error}"),
            Self::InvalidMetadata => {
                f.write_str("effect completion finished-at metadata is required")
            }
            Self::ReceiptTooLarge => {
                f.write_str("effect completion receipt exceeds the bounded limit")
            }
            Self::EvidenceTooLarge => {
                f.write_str("effect completion evidence exceeds the bounded limit")
            }
            Self::EffectNotFound(effect_id) => write!(f, "effect not found: {}", effect_id.0),
            Self::InvalidSourceState { effect_id, actual } => write!(
                f,
                "effect cannot complete from current state: effect={} state={actual}",
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
            Self::AttemptAlreadyFinished(attempt_id) => {
                write!(f, "effect attempt already finished: {}", attempt_id.0)
            }
            Self::SequenceOverflow => f.write_str("effect completion global sequence overflow"),
            Self::InvalidStoredRecord => {
                f.write_str("stored effect completion record is malformed")
            }
        }
    }
}

impl Error for EffectCompletionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for EffectCompletionError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for EffectCompletionError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub struct EffectCompletionStore {
    connection: Connection,
}

impl EffectCompletionStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, EffectCompletionError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn complete(
        &mut self,
        input: CompleteEffectExecution<'_>,
    ) -> Result<CompletedEffectExecution, EffectCompletionError> {
        validate_input(&input)?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let effect_blob = id_blob(input.effect_id.0);
        let current_state = transaction
            .query_row(
                "SELECT to_state FROM effect_transitions WHERE effect_id = ?1 \
                 ORDER BY global_seq DESC LIMIT 1",
                params![&effect_blob],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or(EffectCompletionError::EffectNotFound(input.effect_id))?;
        if current_state != "executing" {
            return Err(EffectCompletionError::InvalidSourceState {
                effect_id: input.effect_id,
                actual: current_state,
            });
        }

        let attempt_blob = id_blob(input.attempt_id.0);
        let attempt = transaction
            .query_row(
                "SELECT effect_id, started_global_seq, handler_id, handler_version, dispatch_token, \
                 started_at, finished_at, outcome, receipt FROM effect_attempts WHERE attempt_id = ?1",
                params![&attempt_blob],
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
            .optional()?
            .ok_or(EffectCompletionError::AttemptNotFound(input.attempt_id))?;
        if EffectId(id_from_vec(attempt.0.clone())?) != input.effect_id {
            return Err(EffectCompletionError::AttemptEffectMismatch {
                attempt_id: input.attempt_id,
                effect_id: input.effect_id,
            });
        }
        if attempt.6.is_some() {
            return Err(EffectCompletionError::AttemptAlreadyFinished(
                input.attempt_id,
            ));
        }

        transaction.execute(
            "UPDATE effect_attempts SET finished_at = ?1, outcome = ?2, receipt = ?3 \
             WHERE attempt_id = ?4 AND finished_at IS NULL",
            params![
                input.finished_at,
                input.completion.attempt_outcome(),
                input.receipt,
                &attempt_blob,
            ],
        )?;

        let transition = StoredEffectTransition {
            transition_id: input.transition_id,
            effect_id: input.effect_id,
            global_seq: next_global_seq(&transaction)?,
            from_state: Some("executing".to_owned()),
            to_state: input.completion.target_state().to_owned(),
            attempt_id: Some(input.attempt_id),
            reason_code: input.reason_code.map(str::to_owned),
            evidence_ref: input.evidence_ref.map(<[u8]>::to_vec),
            event_id: input.event_id,
        };
        insert_transition(&transaction, &transition)?;
        transaction.commit()?;

        Ok(CompletedEffectExecution {
            attempt: StoredEffectAttempt {
                attempt_id: input.attempt_id,
                effect_id: input.effect_id,
                started_global_seq: seq_from_i64(attempt.1)?,
                handler_id: attempt.2,
                handler_version: attempt.3,
                dispatch_token: attempt.4,
                started_at: attempt.5,
                finished_at: Some(input.finished_at.to_owned()),
                outcome: input.completion.attempt_outcome().to_owned(),
                receipt: input.receipt.map(<[u8]>::to_vec),
            },
            transition,
        })
    }
}

fn validate_input(input: &CompleteEffectExecution<'_>) -> Result<(), EffectCompletionError> {
    if input.finished_at.is_empty() {
        return Err(EffectCompletionError::InvalidMetadata);
    }
    if input
        .receipt
        .is_some_and(|value| value.len() > MAX_RECEIPT_BYTES)
    {
        return Err(EffectCompletionError::ReceiptTooLarge);
    }
    if input
        .evidence_ref
        .is_some_and(|value| value.len() > MAX_EVIDENCE_BYTES)
    {
        return Err(EffectCompletionError::EvidenceTooLarge);
    }
    Ok(())
}

fn insert_transition(
    transaction: &Transaction<'_>,
    stored: &StoredEffectTransition,
) -> Result<(), EffectCompletionError> {
    transaction.execute(
        "INSERT INTO effect_transitions (transition_id, effect_id, global_seq, from_state, to_state, \
         attempt_id, reason_code, evidence_ref, event_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id_blob(stored.transition_id.0),
            id_blob(stored.effect_id.0),
            seq_to_i64(stored.global_seq)?,
            stored.from_state.as_deref(),
            stored.to_state.as_str(),
            stored.attempt_id.map(|attempt_id| id_blob(attempt_id.0)),
            stored.reason_code.as_deref(),
            stored.evidence_ref.as_deref(),
            id_blob(stored.event_id.0),
        ],
    )?;
    Ok(())
}

fn next_global_seq(transaction: &Transaction<'_>) -> Result<u64, EffectCompletionError> {
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
        .ok_or(EffectCompletionError::SequenceOverflow)
}

fn id_blob(value: u128) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

fn id_from_vec(value: Vec<u8>) -> Result<u128, EffectCompletionError> {
    let bytes: [u8; 16] = value
        .try_into()
        .map_err(|_| EffectCompletionError::InvalidStoredRecord)?;
    Ok(u128::from_be_bytes(bytes))
}

fn seq_to_i64(value: u64) -> Result<i64, EffectCompletionError> {
    i64::try_from(value).map_err(|_| EffectCompletionError::SequenceOverflow)
}

fn seq_from_i64(value: i64) -> Result<u64, EffectCompletionError> {
    u64::try_from(value).map_err(|_| EffectCompletionError::InvalidStoredRecord)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::{EffectDispatchStore, PrepareEffectDispatch, encode_effect_dependencies};
    use crate::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
    use golam_core::SessionId;
    use golam_core::paths::RuntimeLayout;
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
            "golam-effect-completion-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    fn executing(runtime: &RuntimeLayout, effect_id: EffectId, attempt_id: EffectAttemptId) {
        let authority = AuthorityLayout::initialize(runtime).unwrap();
        let dependencies = encode_effect_dependencies(&[]).unwrap();
        let mut effects = EffectStore::open(&authority).unwrap();
        effects
            .propose(ProposeEffect {
                effect_id,
                session_id: SessionId(2),
                requested_by: "owner",
                action: "sim.write",
                resource: "sim:effect",
                risk_class: "synthetic",
                execution_semantics: "at_most_once",
                idempotency_key: None,
                preconditions: b"[]",
                dependencies: &dependencies,
                payload_hash: [3; 32],
                proposed_event_id: EventId(10),
                transition_id: EffectTransitionId(11),
            })
            .unwrap();
        effects
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(12),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: Some("test"),
                evidence_ref: None,
                event_id: EventId(13),
            })
            .unwrap();
        drop(effects);
        let mut dispatch = EffectDispatchStore::open(&authority).unwrap();
        dispatch
            .prepare_dispatch(PrepareEffectDispatch {
                effect_id,
                attempt_id,
                transition_id: EffectTransitionId(14),
                handler_id: "sim-at-most-once-write",
                handler_version: "1",
                dispatch_token: b"token",
                started_at: "2026-08-25T13:00:00Z",
                event_id: EventId(15),
            })
            .unwrap();
    }

    #[test]
    fn attempt_finish_and_terminal_transition_commit_atomically() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let effect_id = EffectId(1);
        let attempt_id = EffectAttemptId(2);
        executing(&runtime, effect_id, attempt_id);

        let mut completion = EffectCompletionStore::open(&authority).unwrap();
        let completed = completion
            .complete(CompleteEffectExecution {
                effect_id,
                attempt_id,
                transition_id: EffectTransitionId(16),
                event_id: EventId(17),
                finished_at: "2026-08-25T13:01:00Z",
                completion: ExecutionCompletion::Succeeded,
                reason_code: Some("simulated_success"),
                evidence_ref: None,
                receipt: Some(b"receipt"),
            })
            .unwrap();
        assert_eq!(completed.attempt.outcome, "success");
        assert_eq!(
            completed.transition.from_state.as_deref(),
            Some("executing")
        );
        assert_eq!(completed.transition.to_state, "succeeded");

        let effects = EffectStore::open(&authority).unwrap();
        assert_eq!(
            effects.current_state(effect_id).unwrap().as_deref(),
            Some("succeeded")
        );
        assert_eq!(
            effects
                .attempt(attempt_id)
                .unwrap()
                .unwrap()
                .finished_at
                .as_deref(),
            Some("2026-08-25T13:01:00Z")
        );
        drop(effects);
        drop(completion);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn wrong_effect_rolls_back_without_finishing_attempt() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let effect_id = EffectId(21);
        let attempt_id = EffectAttemptId(22);
        executing(&runtime, effect_id, attempt_id);

        let mut completion = EffectCompletionStore::open(&authority).unwrap();
        assert!(matches!(
            completion.complete(CompleteEffectExecution {
                effect_id: EffectId(99),
                attempt_id,
                transition_id: EffectTransitionId(26),
                event_id: EventId(27),
                finished_at: "2026-08-25T13:02:00Z",
                completion: ExecutionCompletion::Failed,
                reason_code: Some("wrong"),
                evidence_ref: None,
                receipt: None,
            }),
            Err(EffectCompletionError::EffectNotFound(EffectId(99)))
        ));
        let effects = EffectStore::open(&authority).unwrap();
        let attempt = effects.attempt(attempt_id).unwrap().unwrap();
        assert!(attempt.finished_at.is_none());
        assert_eq!(
            effects.current_state(effect_id).unwrap().as_deref(),
            Some("executing")
        );
        drop(effects);
        drop(completion);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
