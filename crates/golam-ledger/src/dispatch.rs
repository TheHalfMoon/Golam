#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{EffectAttemptId, EffectId, EffectTransitionId, EventId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::effects::{StoredEffectAttempt, StoredEffectTransition};
use crate::storage::{AuthorityStore, StorageError};

const DEPENDENCY_MAGIC: &[u8; 4] = b"GDEP";
const DEPENDENCY_VERSION: u8 = 1;
const MAX_DEPENDENCIES: usize = 256;

pub struct PrepareEffectDispatch<'a> {
    pub effect_id: EffectId,
    pub attempt_id: EffectAttemptId,
    pub transition_id: EffectTransitionId,
    pub handler_id: &'a str,
    pub handler_version: &'a str,
    pub dispatch_token: &'a [u8],
    pub started_at: &'a str,
    pub event_id: EventId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedEffectDispatchRecord {
    pub attempt: StoredEffectAttempt,
    pub transition: StoredEffectTransition,
}

#[derive(Debug)]
pub enum EffectDispatchStoreError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    InvalidMetadata,
    InvalidDependencyEncoding,
    TooManyDependencies,
    DuplicateDependency(EffectId),
    SelfDependency(EffectId),
    EffectNotFound(EffectId),
    NotAuthorized { effect_id: EffectId, actual: String },
    DependencyBlocked {
        dependency_id: EffectId,
        state: Option<String>,
    },
    AttemptAlreadyExists(EffectAttemptId),
    SequenceOverflow,
    InvalidStoredRecord,
}

impl fmt::Display for EffectDispatchStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "effect dispatch authority error: {error}"),
            Self::Sqlite(error) => write!(f, "effect dispatch sqlite error: {error}"),
            Self::InvalidMetadata => f.write_str("effect dispatch metadata must be non-empty"),
            Self::InvalidDependencyEncoding => {
                f.write_str("effect dependency encoding is invalid or non-canonical")
            }
            Self::TooManyDependencies => f.write_str("effect dependency list exceeds the limit"),
            Self::DuplicateDependency(effect_id) => {
                write!(f, "duplicate effect dependency: {}", effect_id.0)
            }
            Self::SelfDependency(effect_id) => {
                write!(f, "effect cannot depend on itself: {}", effect_id.0)
            }
            Self::EffectNotFound(effect_id) => write!(f, "effect not found: {}", effect_id.0),
            Self::NotAuthorized { effect_id, actual } => write!(
                f,
                "effect is not dispatchable: effect={} state={actual}",
                effect_id.0
            ),
            Self::DependencyBlocked {
                dependency_id,
                state,
            } => write!(
                f,
                "effect dependency blocks dispatch: dependency={} state={}",
                dependency_id.0,
                state.as_deref().unwrap_or("missing")
            ),
            Self::AttemptAlreadyExists(attempt_id) => {
                write!(f, "effect attempt already exists: {}", attempt_id.0)
            }
            Self::SequenceOverflow => f.write_str("effect dispatch global sequence overflow"),
            Self::InvalidStoredRecord => f.write_str("stored effect dispatch record is malformed"),
        }
    }
}

impl Error for EffectDispatchStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for EffectDispatchStoreError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for EffectDispatchStoreError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub fn encode_effect_dependencies(
    dependencies: &[EffectId],
) -> Result<Vec<u8>, EffectDispatchStoreError> {
    if dependencies.len() > MAX_DEPENDENCIES {
        return Err(EffectDispatchStoreError::TooManyDependencies);
    }

    let mut ids = dependencies.iter().map(|dependency| dependency.0).collect::<Vec<_>>();
    ids.sort_unstable();
    for pair in ids.windows(2) {
        if pair[0] == pair[1] {
            return Err(EffectDispatchStoreError::DuplicateDependency(EffectId(pair[0])));
        }
    }

    let count = u16::try_from(ids.len()).map_err(|_| EffectDispatchStoreError::TooManyDependencies)?;
    let mut encoded = Vec::with_capacity(7 + ids.len() * 16);
    encoded.extend_from_slice(DEPENDENCY_MAGIC);
    encoded.push(DEPENDENCY_VERSION);
    encoded.extend_from_slice(&count.to_be_bytes());
    for id in ids {
        encoded.extend_from_slice(&id.to_be_bytes());
    }
    Ok(encoded)
}

pub fn decode_effect_dependencies(
    encoded: &[u8],
) -> Result<Vec<EffectId>, EffectDispatchStoreError> {
    if encoded.len() < 7
        || &encoded[..4] != DEPENDENCY_MAGIC
        || encoded[4] != DEPENDENCY_VERSION
    {
        return Err(EffectDispatchStoreError::InvalidDependencyEncoding);
    }

    let count = usize::from(u16::from_be_bytes([encoded[5], encoded[6]]));
    if count > MAX_DEPENDENCIES || encoded.len() != 7 + count * 16 {
        return Err(EffectDispatchStoreError::InvalidDependencyEncoding);
    }

    let mut dependencies = Vec::with_capacity(count);
    let mut previous = None;
    for chunk in encoded[7..].chunks_exact(16) {
        let bytes: [u8; 16] = chunk
            .try_into()
            .map_err(|_| EffectDispatchStoreError::InvalidDependencyEncoding)?;
        let id = u128::from_be_bytes(bytes);
        if previous.is_some_and(|prior| prior >= id) {
            return Err(EffectDispatchStoreError::InvalidDependencyEncoding);
        }
        previous = Some(id);
        dependencies.push(EffectId(id));
    }
    Ok(dependencies)
}

pub struct EffectDispatchStore {
    connection: Connection,
}

impl EffectDispatchStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, EffectDispatchStoreError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn prepare_dispatch(
        &mut self,
        input: PrepareEffectDispatch<'_>,
    ) -> Result<PreparedEffectDispatchRecord, EffectDispatchStoreError> {
        validate_prepare(&input)?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let effect_blob = id_blob(input.effect_id.0);

        let effect = transaction
            .query_row(
                "SELECT t.to_state, t.global_seq, i.dependencies FROM effect_intents i \
                 JOIN effect_transitions t ON t.effect_id = i.effect_id \
                 WHERE i.effect_id = ?1 ORDER BY t.global_seq DESC LIMIT 1",
                params![&effect_blob],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                    ))
                },
            )
            .optional()?
            .ok_or(EffectDispatchStoreError::EffectNotFound(input.effect_id))?;

        if effect.0 != "authorized" {
            return Err(EffectDispatchStoreError::NotAuthorized {
                effect_id: input.effect_id,
                actual: effect.0,
            });
        }
        let authorized_global_seq = i64_to_seq(effect.1)?;
        let dependencies = decode_effect_dependencies(&effect.2)?;
        for dependency_id in dependencies {
            if dependency_id == input.effect_id {
                return Err(EffectDispatchStoreError::SelfDependency(input.effect_id));
            }
            let state = latest_state(&transaction, dependency_id)?;
            if state.as_deref() != Some("succeeded") {
                return Err(EffectDispatchStoreError::DependencyBlocked {
                    dependency_id,
                    state,
                });
            }
        }

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
            return Err(EffectDispatchStoreError::AttemptAlreadyExists(input.attempt_id));
        }

        transaction.execute(
            "INSERT INTO effect_attempts (attempt_id, effect_id, started_global_seq, handler_id, \
             handler_version, dispatch_token, started_at, finished_at, outcome, receipt) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, 'unknown', NULL)",
            params![
                &attempt_blob,
                &effect_blob,
                seq_to_i64(authorized_global_seq)?,
                input.handler_id,
                input.handler_version,
                input.dispatch_token,
                input.started_at,
            ],
        )?;

        let transition = StoredEffectTransition {
            transition_id: input.transition_id,
            effect_id: input.effect_id,
            global_seq: next_global_seq(&transaction)?,
            from_state: Some("authorized".to_owned()),
            to_state: "executing".to_owned(),
            attempt_id: Some(input.attempt_id),
            reason_code: Some("durable_attempt_before_dispatch".to_owned()),
            evidence_ref: None,
            event_id: input.event_id,
        };
        insert_transition(&transaction, &transition)?;
        transaction.commit()?;

        Ok(PreparedEffectDispatchRecord {
            attempt: StoredEffectAttempt {
                attempt_id: input.attempt_id,
                effect_id: input.effect_id,
                started_global_seq: authorized_global_seq,
                handler_id: input.handler_id.to_owned(),
                handler_version: input.handler_version.to_owned(),
                dispatch_token: input.dispatch_token.to_vec(),
                started_at: input.started_at.to_owned(),
                finished_at: None,
                outcome: "unknown".to_owned(),
                receipt: None,
            },
            transition,
        })
    }
}

fn validate_prepare(input: &PrepareEffectDispatch<'_>) -> Result<(), EffectDispatchStoreError> {
    if input.handler_id.is_empty()
        || input.handler_version.is_empty()
        || input.dispatch_token.is_empty()
        || input.started_at.is_empty()
    {
        return Err(EffectDispatchStoreError::InvalidMetadata);
    }
    Ok(())
}

fn latest_state(
    transaction: &Transaction<'_>,
    effect_id: EffectId,
) -> Result<Option<String>, EffectDispatchStoreError> {
    Ok(transaction
        .query_row(
            "SELECT to_state FROM effect_transitions WHERE effect_id = ?1 \
             ORDER BY global_seq DESC LIMIT 1",
            params![id_blob(effect_id.0)],
            |row| row.get::<_, String>(0),
        )
        .optional()?)
}

fn insert_transition(
    transaction: &Transaction<'_>,
    stored: &StoredEffectTransition,
) -> Result<(), EffectDispatchStoreError> {
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

fn next_global_seq(transaction: &Transaction<'_>) -> Result<u64, EffectDispatchStoreError> {
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
        .ok_or(EffectDispatchStoreError::SequenceOverflow)
}

fn id_blob(value: u128) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

fn seq_to_i64(value: u64) -> Result<i64, EffectDispatchStoreError> {
    i64::try_from(value).map_err(|_| EffectDispatchStoreError::SequenceOverflow)
}

fn i64_to_seq(value: i64) -> Result<u64, EffectDispatchStoreError> {
    u64::try_from(value).map_err(|_| EffectDispatchStoreError::InvalidStoredRecord)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
    use golam_core::paths::RuntimeLayout;
    use golam_core::SessionId;
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
            std::env::temp_dir().join(format!("golam-dispatch-store-{}-{t}-{n}", std::process::id())),
        )
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    fn proposal<'a>(effect_id: EffectId, dependencies: &'a [u8]) -> ProposeEffect<'a> {
        ProposeEffect {
            effect_id,
            session_id: SessionId(7),
            requested_by: "owner",
            action: "sim.write",
            resource: "sim:target",
            risk_class: "synthetic",
            execution_semantics: "at_most_once",
            idempotency_key: None,
            preconditions: b"[]",
            dependencies,
            payload_hash: [4; 32],
            proposed_event_id: EventId(10_000 + effect_id.0),
            transition_id: EffectTransitionId(20_000 + effect_id.0),
        }
    }

    fn authorize(store: &mut EffectStore, effect_id: EffectId, ordinal: u128) -> u64 {
        store
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(30_000 + ordinal),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: Some("test_authorized"),
                evidence_ref: None,
                event_id: EventId(40_000 + ordinal),
            })
            .unwrap()
            .global_seq
    }

    fn transition(
        store: &mut EffectStore,
        effect_id: EffectId,
        from: &'static str,
        to: &'static str,
        ordinal: u128,
    ) {
        store
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(50_000 + ordinal),
                effect_id,
                expected_state: from,
                next_state: to,
                attempt_id: None,
                reason_code: Some("test_transition"),
                evidence_ref: None,
                event_id: EventId(60_000 + ordinal),
            })
            .unwrap();
    }

    #[test]
    fn dependency_encoding_is_sorted_bounded_and_canonical() {
        let encoded = encode_effect_dependencies(&[EffectId(3), EffectId(1), EffectId(2)]).unwrap();
        assert_eq!(
            decode_effect_dependencies(&encoded).unwrap(),
            vec![EffectId(1), EffectId(2), EffectId(3)]
        );
        assert!(matches!(
            encode_effect_dependencies(&[EffectId(1), EffectId(1)]),
            Err(EffectDispatchStoreError::DuplicateDependency(EffectId(1)))
        ));
        assert!(decode_effect_dependencies(b"[]").is_err());
    }

    #[test]
    fn prepare_dispatch_commits_attempt_and_executing_before_return() {
        let (runtime, authority) = authority();
        let dependencies = encode_effect_dependencies(&[]).unwrap();
        let effect_id = EffectId(100);
        let attempt_id = EffectAttemptId(200);
        let mut effects = EffectStore::open(&authority).unwrap();
        effects.propose(proposal(effect_id, &dependencies)).unwrap();
        let authorized_global_seq = authorize(&mut effects, effect_id, 1);
        drop(effects);

        let mut dispatch = EffectDispatchStore::open(&authority).unwrap();
        let prepared = dispatch
            .prepare_dispatch(PrepareEffectDispatch {
                effect_id,
                attempt_id,
                transition_id: EffectTransitionId(301),
                handler_id: "sim-at-most-once-write",
                handler_version: "1",
                dispatch_token: b"dispatch-200",
                started_at: "2026-08-25T10:00:00Z",
                event_id: EventId(401),
            })
            .unwrap();
        assert_eq!(prepared.attempt.started_global_seq, authorized_global_seq);
        assert_eq!(prepared.transition.from_state.as_deref(), Some("authorized"));
        assert_eq!(prepared.transition.to_state, "executing");
        assert_eq!(prepared.transition.attempt_id, Some(attempt_id));
        drop(dispatch);

        let effects = EffectStore::open(&authority).unwrap();
        assert_eq!(effects.attempt(attempt_id).unwrap(), Some(prepared.attempt));
        assert_eq!(
            effects.current_state(effect_id).unwrap().as_deref(),
            Some("executing")
        );
        drop(effects);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn unknown_outcome_dependency_blocks_dispatch_without_attempt() {
        let (runtime, authority) = authority();
        let empty = encode_effect_dependencies(&[]).unwrap();
        let dependency_id = EffectId(110);
        let dependent_id = EffectId(111);
        let dependent_dependencies = encode_effect_dependencies(&[dependency_id]).unwrap();
        let mut effects = EffectStore::open(&authority).unwrap();
        effects.propose(proposal(dependency_id, &empty)).unwrap();
        authorize(&mut effects, dependency_id, 10);
        transition(&mut effects, dependency_id, "authorized", "executing", 11);
        transition(
            &mut effects,
            dependency_id,
            "executing",
            "unknown_outcome",
            12,
        );
        effects
            .propose(proposal(dependent_id, &dependent_dependencies))
            .unwrap();
        authorize(&mut effects, dependent_id, 13);
        drop(effects);

        let mut dispatch = EffectDispatchStore::open(&authority).unwrap();
        assert!(matches!(
            dispatch.prepare_dispatch(PrepareEffectDispatch {
                effect_id: dependent_id,
                attempt_id: EffectAttemptId(211),
                transition_id: EffectTransitionId(311),
                handler_id: "sim-at-most-once-write",
                handler_version: "1",
                dispatch_token: b"dispatch-211",
                started_at: "2026-08-25T10:01:00Z",
                event_id: EventId(411),
            }),
            Err(EffectDispatchStoreError::DependencyBlocked {
                dependency_id: blocked,
                state: Some(ref state),
            }) if blocked == dependency_id && state == "unknown_outcome"
        ));
        drop(dispatch);

        let effects = EffectStore::open(&authority).unwrap();
        assert_eq!(effects.attempt_count(dependent_id).unwrap(), 0);
        assert_eq!(
            effects.current_state(dependent_id).unwrap().as_deref(),
            Some("authorized")
        );
        drop(effects);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn succeeded_dependency_allows_dispatch() {
        let (runtime, authority) = authority();
        let empty = encode_effect_dependencies(&[]).unwrap();
        let dependency_id = EffectId(120);
        let dependent_id = EffectId(121);
        let dependent_dependencies = encode_effect_dependencies(&[dependency_id]).unwrap();
        let mut effects = EffectStore::open(&authority).unwrap();
        effects.propose(proposal(dependency_id, &empty)).unwrap();
        authorize(&mut effects, dependency_id, 20);
        transition(&mut effects, dependency_id, "authorized", "executing", 21);
        transition(&mut effects, dependency_id, "executing", "succeeded", 22);
        effects
            .propose(proposal(dependent_id, &dependent_dependencies))
            .unwrap();
        authorize(&mut effects, dependent_id, 23);
        drop(effects);

        let mut dispatch = EffectDispatchStore::open(&authority).unwrap();
        let prepared = dispatch
            .prepare_dispatch(PrepareEffectDispatch {
                effect_id: dependent_id,
                attempt_id: EffectAttemptId(221),
                transition_id: EffectTransitionId(321),
                handler_id: "sim-at-most-once-write",
                handler_version: "1",
                dispatch_token: b"dispatch-221",
                started_at: "2026-08-25T10:02:00Z",
                event_id: EventId(421),
            })
            .unwrap();
        assert_eq!(prepared.transition.to_state, "executing");
        drop(dispatch);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
