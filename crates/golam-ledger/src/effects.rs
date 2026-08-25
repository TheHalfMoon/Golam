#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{EffectAttemptId, EffectId, EffectTransitionId, EventId, SessionId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

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

#[derive(Debug)]
pub enum EffectStoreError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    InvalidMetadata,
    InvalidState(String),
    EffectAlreadyExists(EffectId),
    EffectNotFound(EffectId),
    MissingCurrentState(EffectId),
    StaleState { expected: String, actual: String },
    SequenceOverflow,
    InvalidStoredRecord,
}

impl fmt::Display for EffectStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "effect store authority error: {error}"),
            Self::Sqlite(error) => write!(f, "effect store sqlite error: {error}"),
            Self::InvalidMetadata => f.write_str("effect request metadata must be non-empty"),
            Self::InvalidState(state) => write!(f, "invalid effect state: {state}"),
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
            Self::SequenceOverflow => f.write_str("effect global sequence overflow"),
            Self::InvalidStoredRecord => f.write_str("stored effect transition is malformed"),
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

        transaction.execute(
            "INSERT INTO effect_intents (effect_id, session_id, requested_by, action, resource, \
             risk_class, execution_semantics, idempotency_key, preconditions, dependencies, \
             payload_hash, proposed_event_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                &effect_blob,
                id_blob(input.session_id.0),
                input.requested_by,
                input.action,
                input.resource,
                input.risk_class,
                input.execution_semantics,
                input.idempotency_key,
                input.preconditions,
                input.dependencies,
                &input.payload_hash[..],
                id_blob(input.proposed_event_id.0),
            ],
        )?;

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

    pub fn transition_count(&self, effect_id: EffectId) -> Result<usize, EffectStoreError> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM effect_transitions WHERE effect_id = ?1",
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

fn validate_state(state: &str) -> Result<(), EffectStoreError> {
    if EFFECT_STATES.contains(&state) {
        Ok(())
    } else {
        Err(EffectStoreError::InvalidState(state.to_owned()))
    }
}

fn insert_transition(
    transaction: &Transaction<'_>,
    stored: &StoredEffectTransition,
) -> Result<(), EffectStoreError> {
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
    u64::try_from(current)
        .map_err(|_| EffectStoreError::InvalidStoredRecord)?
        .checked_add(1)
        .ok_or(EffectStoreError::SequenceOverflow)
}

fn id_blob(value: u128) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

fn seq_to_i64(value: u64) -> Result<i64, EffectStoreError> {
    i64::try_from(value).map_err(|_| EffectStoreError::SequenceOverflow)
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
}
