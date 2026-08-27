#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{EffectId, SessionId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::approval_runtime::{ApprovalUseError, ApprovalUseRequest, ApprovalUseStore};
use crate::approvals::ApprovalClass;
use crate::authority_security_write::append_approval_consumption_snapshot;
use crate::storage::{AuthorityStore, StorageError};

pub const MAX_UNATTENDED_IRREVERSIBLE_RUN_USES: u64 = 256;
const CONSUMPTION_ID_DOMAIN: &[u8] = b"golam:run-preauthorization-consumption:v1";

#[derive(Clone, Copy, Debug)]
pub struct UnattendedIrreversibleRequest<'a> {
    pub approval_id: [u8; 16],
    pub effect_id: EffectId,
    pub taint_digest: [u8; 32],
    pub observed_at: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RunPreauthorizationUse {
    consumption_id: [u8; 16],
    approval_id: [u8; 16],
    effect_id: EffectId,
    session_id: SessionId,
    use_number: u64,
    max_uses: u64,
    consumed_global_seq: u64,
}

impl RunPreauthorizationUse {
    pub const fn consumption_id(self) -> [u8; 16] {
        self.consumption_id
    }

    pub const fn approval_id(self) -> [u8; 16] {
        self.approval_id
    }

    pub const fn effect_id(self) -> EffectId {
        self.effect_id
    }

    pub const fn session_id(self) -> SessionId {
        self.session_id
    }

    pub const fn use_number(self) -> u64 {
        self.use_number
    }

    pub const fn max_uses(self) -> u64 {
        self.max_uses
    }

    pub const fn consumed_global_seq(self) -> u64 {
        self.consumed_global_seq
    }
}

#[derive(Debug)]
pub enum RunPreauthorizationError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    ApprovalUse(ApprovalUseError),
    Integrity(String),
    AuthoritySecurity(String),
    EffectNotFound,
    EffectNotAuthorized,
    NotIrreversible,
    WrongApprovalClass,
    UnboundRunScope,
    RunScopeMismatch,
    UsageLimitTooLarge,
    UsageLimitReached,
    Replay,
    InvalidStoredRecord(&'static str),
}

impl fmt::Display for RunPreauthorizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "run preauthorization authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "run preauthorization sqlite error: {error}"),
            Self::ApprovalUse(error) => {
                write!(f, "run preauthorization use validation failed: {error}")
            }
            Self::Integrity(error) => write!(f, "run preauthorization integrity error: {error}"),
            Self::AuthoritySecurity(error) => {
                write!(f, "run preauthorization authority-security error: {error}")
            }
            Self::EffectNotFound => f.write_str("unattended effect does not exist"),
            Self::EffectNotAuthorized => {
                f.write_str("unattended irreversible effect is not execution-ready")
            }
            Self::NotIrreversible => {
                f.write_str("RUN_PREAUTHORIZATION gate is reserved for irreversible effects")
            }
            Self::WrongApprovalClass => f.write_str(
                "unattended irreversible effects require RUN_PREAUTHORIZATION; other approval classes cannot substitute",
            ),
            Self::UnboundRunScope => f.write_str(
                "unattended irreversible RUN_PREAUTHORIZATION must bind one exact session/run",
            ),
            Self::RunScopeMismatch => {
                f.write_str("RUN_PREAUTHORIZATION does not match the protected effect run scope")
            }
            Self::UsageLimitTooLarge => write!(
                f,
                "RUN_PREAUTHORIZATION usage limit exceeds Spec 003 unattended irreversible ceiling of {MAX_UNATTENDED_IRREVERSIBLE_RUN_USES}",
            ),
            Self::UsageLimitReached => f.write_str("RUN_PREAUTHORIZATION usage limit is exhausted"),
            Self::Replay => f.write_str("this effect already claimed the RUN_PREAUTHORIZATION"),
            Self::InvalidStoredRecord(reason) => {
                write!(f, "stored run preauthorization record is invalid: {reason}")
            }
        }
    }
}

impl Error for RunPreauthorizationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::ApprovalUse(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for RunPreauthorizationError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for RunPreauthorizationError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<ApprovalUseError> for RunPreauthorizationError {
    fn from(value: ApprovalUseError) -> Self {
        Self::ApprovalUse(value)
    }
}

pub struct RunPreauthorizationStore {
    layout: AuthorityLayout,
    connection: Connection,
}

impl RunPreauthorizationStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, RunPreauthorizationError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self {
            layout: layout.clone(),
            connection,
        })
    }

    /// Claims one bounded RUN_PREAUTHORIZATION use for an unattended
    /// irreversible effect. Protected effect state supplies action, resource,
    /// risk and session; caller input cannot widen those authority dimensions.
    pub fn claim_unattended_irreversible(
        &mut self,
        request: UnattendedIrreversibleRequest<'_>,
    ) -> Result<RunPreauthorizationUse, RunPreauthorizationError> {
        let effect = self.load_effect(request.effect_id)?;
        if effect.execution_semantics != "irreversible" {
            return Err(RunPreauthorizationError::NotIrreversible);
        }
        if effect.state != "authorized" {
            return Err(RunPreauthorizationError::EffectNotAuthorized);
        }

        let evidence = ApprovalUseStore::open(&self.layout)?.validate(ApprovalUseRequest {
            approval_id: request.approval_id,
            action: &effect.action,
            resource: &effect.resource,
            effect_id: Some(request.effect_id),
            session_id: Some(effect.session_id),
            risk_class: &effect.risk_class,
            taint_digest: request.taint_digest,
            observed_at: request.observed_at,
        })?;
        if evidence.class() != ApprovalClass::RunPreauthorization {
            return Err(RunPreauthorizationError::WrongApprovalClass);
        }
        if evidence.max_uses() > MAX_UNATTENDED_IRREVERSIBLE_RUN_USES {
            return Err(RunPreauthorizationError::UsageLimitTooLarge);
        }

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        if load_effect_in_transaction(&transaction, request.effect_id)? != effect {
            return Err(RunPreauthorizationError::EffectNotAuthorized);
        }
        verify_run_approval(
            &transaction,
            request.approval_id,
            &effect,
            request.taint_digest,
            request.observed_at,
            evidence.max_uses(),
        )?;

        let consumption_id = consumption_id(request.approval_id, request.effect_id);
        if transaction
            .query_row(
                "SELECT 1 FROM approval_consumptions WHERE consumption_id = ?1 OR (approval_id = ?2 AND effect_or_operation_id = ?3) LIMIT 1",
                params![
                    &consumption_id[..],
                    &request.approval_id[..],
                    &request.effect_id.0.to_be_bytes()[..],
                ],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some()
        {
            return Err(RunPreauthorizationError::Replay);
        }

        let current_uses = current_usage_count(&transaction, request.approval_id)?;
        if current_uses >= evidence.max_uses() {
            return Err(RunPreauthorizationError::UsageLimitReached);
        }
        let use_number = current_uses.checked_add(1).ok_or(
            RunPreauthorizationError::InvalidStoredRecord(
                "run preauthorization use count overflow",
            ),
        )?;
        transaction.execute(
            "INSERT INTO approval_consumptions (consumption_id, approval_id, effect_or_operation_id, reserved_global_seq, consumed_global_seq, state) VALUES (?1, ?2, ?3, ?4, ?5, 'consumed')",
            params![
                &consumption_id[..],
                &request.approval_id[..],
                &request.effect_id.0.to_be_bytes()[..],
                to_i64(effect.authorized_global_seq)?,
                to_i64(effect.authorized_global_seq)?,
            ],
        )?;
        append_approval_consumption_snapshot(&transaction, &consumption_id)
            .map_err(|error| RunPreauthorizationError::AuthoritySecurity(error.to_string()))?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| RunPreauthorizationError::AuthoritySecurity(error.to_string()))?;
        transaction.commit()?;

        Ok(RunPreauthorizationUse {
            consumption_id,
            approval_id: request.approval_id,
            effect_id: request.effect_id,
            session_id: effect.session_id,
            use_number,
            max_uses: evidence.max_uses(),
            consumed_global_seq: effect.authorized_global_seq,
        })
    }

    fn load_effect(
        &self,
        effect_id: EffectId,
    ) -> Result<ProtectedEffect, RunPreauthorizationError> {
        load_effect_from_connection(&self.connection, effect_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProtectedEffect {
    session_id: SessionId,
    action: String,
    resource: String,
    risk_class: String,
    execution_semantics: String,
    state: String,
    authorized_global_seq: u64,
}

fn load_effect_from_connection(
    connection: &Connection,
    effect_id: EffectId,
) -> Result<ProtectedEffect, RunPreauthorizationError> {
    let row = connection
        .query_row(
            "SELECT i.session_id, i.action, i.resource, i.risk_class, i.execution_semantics, t.to_state, t.global_seq FROM effect_intents i JOIN effect_transitions t ON t.effect_id = i.effect_id WHERE i.effect_id = ?1 AND t.global_seq = (SELECT MAX(t2.global_seq) FROM effect_transitions t2 WHERE t2.effect_id = i.effect_id)",
            params![&effect_id.0.to_be_bytes()[..]],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            },
        )
        .optional()?
        .ok_or(RunPreauthorizationError::EffectNotFound)?;
    protected_effect(row)
}

fn load_effect_in_transaction(
    transaction: &Transaction<'_>,
    effect_id: EffectId,
) -> Result<ProtectedEffect, RunPreauthorizationError> {
    let row = transaction
        .query_row(
            "SELECT i.session_id, i.action, i.resource, i.risk_class, i.execution_semantics, t.to_state, t.global_seq FROM effect_intents i JOIN effect_transitions t ON t.effect_id = i.effect_id WHERE i.effect_id = ?1 AND t.global_seq = (SELECT MAX(t2.global_seq) FROM effect_transitions t2 WHERE t2.effect_id = i.effect_id)",
            params![&effect_id.0.to_be_bytes()[..]],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            },
        )
        .optional()?
        .ok_or(RunPreauthorizationError::EffectNotFound)?;
    protected_effect(row)
}

fn protected_effect(
    row: (Vec<u8>, String, String, String, String, String, i64),
) -> Result<ProtectedEffect, RunPreauthorizationError> {
    let session_id = id16(row.0, "effect session id is not 16 bytes")?;
    Ok(ProtectedEffect {
        session_id: SessionId(u128::from_be_bytes(session_id)),
        action: row.1,
        resource: row.2,
        risk_class: row.3,
        execution_semantics: row.4,
        state: row.5,
        authorized_global_seq: from_i64(row.6, "effect global sequence is invalid")?,
    })
}

fn verify_run_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    effect: &ProtectedEffect,
    expected_taint: [u8; 32],
    observed_at: &str,
    expected_max_uses: u64,
) -> Result<(), RunPreauthorizationError> {
    let row = transaction
        .query_row(
            "SELECT class, action_scope, resource_scope, session_id, risk_class, taint_digest, issued_at, expires_at, max_uses, revoked_at FROM approvals WHERE approval_id = ?1",
            params![&approval_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, Option<Vec<u8>>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Vec<u8>>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<i64>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                ))
            },
        )
        .optional()?
        .ok_or(RunPreauthorizationError::InvalidStoredRecord(
            "RUN_PREAUTHORIZATION approval disappeared after validation",
        ))?;
    if row.0 != ApprovalClass::RunPreauthorization.as_str() {
        return Err(RunPreauthorizationError::WrongApprovalClass);
    }
    let session_id = row
        .3
        .ok_or(RunPreauthorizationError::UnboundRunScope)
        .and_then(|value| id16(value, "RUN_PREAUTHORIZATION session id is not 16 bytes"))?;
    if session_id != effect.session_id.0.to_be_bytes() {
        return Err(RunPreauthorizationError::RunScopeMismatch);
    }
    if row.4 != effect.risk_class
        || hash32(row.5, "RUN_PREAUTHORIZATION taint digest is not 32 bytes")? != expected_taint
        || !exact_set_contains(&row.1, &effect.action)?
        || !exact_set_contains(&row.2, &effect.resource)?
    {
        return Err(RunPreauthorizationError::RunScopeMismatch);
    }
    if observed_at < row.6.as_str() {
        return Err(RunPreauthorizationError::RunScopeMismatch);
    }
    let expires_at = row
        .7
        .as_deref()
        .ok_or(RunPreauthorizationError::InvalidStoredRecord(
            "RUN_PREAUTHORIZATION is missing finite expiry",
        ))?;
    if row.6.as_str() >= expires_at || observed_at >= expires_at || row.9.is_some() {
        return Err(RunPreauthorizationError::RunScopeMismatch);
    }
    let max_uses = row
        .8
        .ok_or(RunPreauthorizationError::InvalidStoredRecord(
            "RUN_PREAUTHORIZATION max_uses is missing",
        ))
        .and_then(|value| from_i64(value, "RUN_PREAUTHORIZATION max_uses is invalid"))?;
    if max_uses != expected_max_uses {
        return Err(RunPreauthorizationError::InvalidStoredRecord(
            "RUN_PREAUTHORIZATION max_uses changed during claim",
        ));
    }
    if max_uses == 0 || max_uses > MAX_UNATTENDED_IRREVERSIBLE_RUN_USES {
        return Err(RunPreauthorizationError::UsageLimitTooLarge);
    }
    Ok(())
}

fn exact_set_contains(encoded: &[u8], expected: &str) -> Result<bool, RunPreauthorizationError> {
    let text = std::str::from_utf8(encoded).map_err(|_| {
        RunPreauthorizationError::InvalidStoredRecord("RUN_PREAUTHORIZATION set scope is not UTF-8")
    })?;
    if text.is_empty() {
        return Err(RunPreauthorizationError::InvalidStoredRecord(
            "RUN_PREAUTHORIZATION set scope is empty",
        ));
    }
    let mut previous = None;
    let mut found = false;
    for value in text.split('\n') {
        if value.is_empty() || previous.is_some_and(|prior| prior >= value) {
            return Err(RunPreauthorizationError::InvalidStoredRecord(
                "RUN_PREAUTHORIZATION set scope is not strictly sorted and unique",
            ));
        }
        found |= value == expected;
        previous = Some(value);
    }
    Ok(found)
}

fn current_usage_count(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
) -> Result<u64, RunPreauthorizationError> {
    let mut statement = transaction.prepare(
        "SELECT state FROM approval_consumptions WHERE approval_id = ?1 ORDER BY reserved_global_seq ASC, consumption_id ASC",
    )?;
    let states = statement.query_map(params![&approval_id[..]], |row| row.get::<_, String>(0))?;
    let mut count = 0_u64;
    for state in states {
        match state?.as_str() {
            "reserved" | "consumed" => {
                count = count.checked_add(1).ok_or(
                    RunPreauthorizationError::InvalidStoredRecord(
                        "RUN_PREAUTHORIZATION usage count overflow",
                    ),
                )?;
            }
            "released" => {
                return Err(RunPreauthorizationError::InvalidStoredRecord(
                    "released state cannot reopen unattended irreversible authority",
                ));
            }
            _ => {
                return Err(RunPreauthorizationError::InvalidStoredRecord(
                    "RUN_PREAUTHORIZATION consumption state is unknown",
                ));
            }
        }
    }
    Ok(count)
}

fn verify_transaction_integrity(
    transaction: &Transaction<'_>,
) -> Result<(), RunPreauthorizationError> {
    crate::integrity::verify(transaction)
        .map_err(|error| RunPreauthorizationError::Integrity(error.to_string()))?;
    crate::authority_security_v2::verify(transaction)
        .map_err(|error| RunPreauthorizationError::AuthoritySecurity(error.to_string()))
}

fn consumption_id(approval_id: [u8; 16], effect_id: EffectId) -> [u8; 16] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(CONSUMPTION_ID_DOMAIN);
    hasher.update(&approval_id);
    hasher.update(&effect_id.0.to_be_bytes());
    let digest = hasher.finalize();
    let mut id = [0_u8; 16];
    id.copy_from_slice(&digest.as_bytes()[..16]);
    id
}

fn hash32(value: Vec<u8>, reason: &'static str) -> Result<[u8; 32], RunPreauthorizationError> {
    value
        .try_into()
        .map_err(|_| RunPreauthorizationError::InvalidStoredRecord(reason))
}

fn id16(value: Vec<u8>, reason: &'static str) -> Result<[u8; 16], RunPreauthorizationError> {
    value
        .try_into()
        .map_err(|_| RunPreauthorizationError::InvalidStoredRecord(reason))
}

fn from_i64(value: i64, reason: &'static str) -> Result<u64, RunPreauthorizationError> {
    u64::try_from(value).map_err(|_| RunPreauthorizationError::InvalidStoredRecord(reason))
}

fn to_i64(value: u64) -> Result<i64, RunPreauthorizationError> {
    i64::try_from(value).map_err(|_| {
        RunPreauthorizationError::InvalidStoredRecord(
            "RUN_PREAUTHORIZATION global sequence overflows i64",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approval_binding::{
        APPROVAL_ISSUE_ACTION, APPROVAL_MUTATION_RISK_CLASS, ApprovalStore, PreparedApproval,
        prepare_approval,
    };
    use crate::approvals::ApprovalScope;
    use crate::authorization::{
        AppendAuthorizationDecision, AuthorizationAuditLog, AuthorizationDecisionEvidence,
        AuthorizationDecisionKind,
    };
    use crate::dispatch::encode_effect_dependencies;
    use crate::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
    use golam_core::paths::RuntimeLayout;
    use golam_core::{EffectTransitionId, EventId};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);
    static RECORD_N: AtomicU64 = AtomicU64::new(0);
    static ISSUE_EFFECT_N: AtomicU64 = AtomicU64::new(0);

    fn next_record_id() -> u128 {
        1_000_000 + u128::from(RECORD_N.fetch_add(1, Ordering::Relaxed))
    }

    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-run-preauthorization-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    fn create_effect(
        authority: &AuthorityLayout,
        effect_id: EffectId,
        session_id: SessionId,
        action: &str,
        resource: &str,
        risk_class: &str,
        execution_semantics: &str,
        payload_hash: [u8; 32],
    ) {
        let dependencies = encode_effect_dependencies(&[]).unwrap();
        let mut effects = EffectStore::open(authority).unwrap();
        effects
            .propose(ProposeEffect {
                effect_id,
                session_id,
                requested_by: "owner:owner",
                action,
                resource,
                risk_class,
                execution_semantics,
                idempotency_key: None,
                preconditions: b"[]",
                dependencies: &dependencies,
                payload_hash,
                proposed_event_id: EventId(next_record_id()),
                transition_id: EffectTransitionId(next_record_id()),
            })
            .unwrap();
        effects
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(next_record_id()),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: Some("test_effect_authorized"),
                evidence_ref: None,
                event_id: EventId(next_record_id()),
            })
            .unwrap();
    }

    fn authorize_issue_effect(
        authority: &AuthorityLayout,
        prepared: &PreparedApproval,
        effect_id: EffectId,
    ) {
        create_effect(
            authority,
            effect_id,
            SessionId(1),
            APPROVAL_ISSUE_ACTION,
            prepared.resource(),
            APPROVAL_MUTATION_RISK_CLASS,
            "at_most_once",
            prepared.intent_digest(),
        );
    }

    fn issue_approval(
        authority: &AuthorityLayout,
        scope: ApprovalScope,
        max_uses: u64,
    ) -> [u8; 16] {
        let prepared = prepare_approval(
            "owner:owner",
            scope,
            "irreversible_effect",
            [9; 32],
            "2026-08-27T00:00:00Z",
            Some("2026-08-27T01:00:00Z"),
            max_uses,
        )
        .unwrap();
        let issue_effect = EffectId(
            50_000 + u128::from(ISSUE_EFFECT_N.fetch_add(1, Ordering::Relaxed)) * 1_000,
        );
        authorize_issue_effect(authority, &prepared, issue_effect);
        let mut log = AuthorizationAuditLog::open(authority).unwrap();
        let decision = log
            .append(AppendAuthorizationDecision {
                principal: "owner:owner",
                action: APPROVAL_ISSUE_ACTION,
                resource: prepared.resource(),
                context: "scope=local-owner",
                evidence: AuthorizationDecisionEvidence::hard_guard_only("pass"),
                decision: AuthorizationDecisionKind::Allow,
                reason_code: "test_run_preauthorization_parent_authority",
            })
            .unwrap();
        drop(log);
        ApprovalStore::open(authority)
            .unwrap()
            .issue(prepared, decision.decision_id, issue_effect)
            .unwrap()
            .approval_id()
    }

    fn create_target(authority: &AuthorityLayout, effect_id: EffectId) {
        create_effect(
            authority,
            effect_id,
            SessionId(7),
            "effect.simulate",
            "session:7",
            "irreversible_effect",
            "irreversible",
            [7; 32],
        );
    }

    fn request(
        approval_id: [u8; 16],
        effect_id: EffectId,
        observed_at: &'static str,
    ) -> UnattendedIrreversibleRequest<'static> {
        UnattendedIrreversibleRequest {
            approval_id,
            effect_id,
            taint_digest: [9; 32],
            observed_at,
        }
    }

    #[test]
    fn bounded_run_preauthorization_claims_exact_irreversible_effects_until_limit() {
        let (runtime, authority) = authority();
        let actions = vec!["effect.simulate".to_owned()];
        let resources = vec!["session:7".to_owned()];
        let approval_id = issue_approval(
            &authority,
            ApprovalScope::run_preauthorization(Some(SessionId(7)), &actions, &resources).unwrap(),
            2,
        );
        for effect_id in [EffectId(700), EffectId(701), EffectId(702)] {
            create_target(&authority, effect_id);
        }

        let mut store = RunPreauthorizationStore::open(&authority).unwrap();
        let first = store
            .claim_unattended_irreversible(request(
                approval_id,
                EffectId(700),
                "2026-08-27T00:30:00Z",
            ))
            .unwrap();
        let second = store
            .claim_unattended_irreversible(request(
                approval_id,
                EffectId(701),
                "2026-08-27T00:31:00Z",
            ))
            .unwrap();
        assert_eq!(first.use_number(), 1);
        assert_eq!(second.use_number(), 2);
        assert_eq!(second.max_uses(), 2);
        assert!(matches!(
            store.claim_unattended_irreversible(request(
                approval_id,
                EffectId(702),
                "2026-08-27T00:32:00Z",
            )),
            Err(RunPreauthorizationError::ApprovalUse(
                ApprovalUseError::UsageLimitReached
            )) | Err(RunPreauthorizationError::UsageLimitReached)
        ));
        drop(store);
        drop(AuthorityStore::open(authority.authority_db_path()).unwrap());
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn unattended_irreversible_rejects_other_classes_and_unbound_run_scope() {
        let (runtime, authority) = authority();
        create_target(&authority, EffectId(710));
        let actions = vec!["effect.simulate".to_owned()];
        let resources = vec!["session:7".to_owned()];
        let time_boxed = issue_approval(
            &authority,
            ApprovalScope::time_boxed(&actions, &resources).unwrap(),
            2,
        );
        let unbound_run = issue_approval(
            &authority,
            ApprovalScope::run_preauthorization(None, &actions, &resources).unwrap(),
            2,
        );
        let mut store = RunPreauthorizationStore::open(&authority).unwrap();
        assert!(matches!(
            store.claim_unattended_irreversible(request(
                time_boxed,
                EffectId(710),
                "2026-08-27T00:30:00Z",
            )),
            Err(RunPreauthorizationError::WrongApprovalClass)
        ));
        assert!(matches!(
            store.claim_unattended_irreversible(request(
                unbound_run,
                EffectId(710),
                "2026-08-27T00:30:00Z",
            )),
            Err(RunPreauthorizationError::UnboundRunScope)
        ));
        drop(store);
        drop(AuthorityStore::open(authority.authority_db_path()).unwrap());
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn unattended_irreversible_denies_effect_replay_and_generic_huge_allowance() {
        let (runtime, authority) = authority();
        let actions = vec!["effect.simulate".to_owned()];
        let resources = vec!["session:7".to_owned()];
        let normal = issue_approval(
            &authority,
            ApprovalScope::run_preauthorization(Some(SessionId(7)), &actions, &resources).unwrap(),
            2,
        );
        let huge = issue_approval(
            &authority,
            ApprovalScope::run_preauthorization(Some(SessionId(7)), &actions, &resources).unwrap(),
            MAX_UNATTENDED_IRREVERSIBLE_RUN_USES + 1,
        );
        create_target(&authority, EffectId(720));

        let mut store = RunPreauthorizationStore::open(&authority).unwrap();
        store
            .claim_unattended_irreversible(request(
                normal,
                EffectId(720),
                "2026-08-27T00:30:00Z",
            ))
            .unwrap();
        assert!(matches!(
            store.claim_unattended_irreversible(request(
                normal,
                EffectId(720),
                "2026-08-27T00:31:00Z",
            )),
            Err(RunPreauthorizationError::Replay)
                | Err(RunPreauthorizationError::ApprovalUse(
                    ApprovalUseError::UsageLimitReached
                ))
        ));
        assert!(matches!(
            store.claim_unattended_irreversible(request(
                huge,
                EffectId(720),
                "2026-08-27T00:30:00Z",
            )),
            Err(RunPreauthorizationError::UsageLimitTooLarge)
        ));
        drop(store);
        drop(AuthorityStore::open(authority.authority_db_path()).unwrap());
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
