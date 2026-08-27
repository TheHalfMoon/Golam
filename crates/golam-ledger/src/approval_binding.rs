#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::approvals::{ApprovalClass, ApprovalRecord, ApprovalScope, ApprovalScopeError};
use crate::authority_security_write::append_approval_snapshot;
use crate::storage::{AuthorityStore, StorageError};

pub const APPROVAL_ISSUE_ACTION: &str = "approval.issue";
pub const APPROVAL_MUTATION_RISK_CLASS: &str = "approval_mutation";

const MAX_PRINCIPAL_BYTES: usize = 512;
const MAX_RISK_CLASS_BYTES: usize = 128;
const APPROVAL_INTENT_DOMAIN: &[u8] = b"golam:approval-issue-intent:v1";
const APPROVAL_BINDING_DOMAIN: &[u8] = b"golam:approval-binding:v1";
const APPROVAL_ID_DOMAIN: &[u8] = b"golam:approval-id:v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedApproval {
    approver_principal: String,
    scope: ApprovalScope,
    risk_class: String,
    taint_digest: [u8; 32],
    issued_at: String,
    expires_at: Option<String>,
    max_uses: u64,
    intent_digest: [u8; 32],
    resource: String,
}

impl PreparedApproval {
    pub fn resource(&self) -> &str {
        &self.resource
    }

    pub const fn intent_digest(&self) -> [u8; 32] {
        self.intent_digest
    }

    pub const fn class(&self) -> ApprovalClass {
        self.scope.class()
    }
}

#[derive(Debug)]
pub enum ApprovalBindingError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Scope(ApprovalScopeError),
    Integrity(String),
    AuthoritySecurity(String),
    InvalidApprover,
    InvalidRiskClass,
    InvalidTime,
    InvalidUsageLimit,
    MissingExpiry,
    MissingAuthorityDecision,
    AuthorityDecisionMismatch,
    StaleAuthorityDecision,
    EffectNotFound,
    EffectMismatch,
    DuplicateApproval,
    InvalidStoredRecord(&'static str),
}

impl fmt::Display for ApprovalBindingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "approval authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "approval sqlite error: {error}"),
            Self::Core(error) => write!(f, "approval canonical encoding error: {error}"),
            Self::Scope(error) => write!(f, "approval scope error: {error}"),
            Self::Integrity(error) => write!(f, "approval integrity error: {error}"),
            Self::AuthoritySecurity(error) => {
                write!(f, "approval authority-security error: {error}")
            }
            Self::InvalidApprover => f.write_str("approval approver principal is not canonical"),
            Self::InvalidRiskClass => f.write_str("approval risk class is not canonical"),
            Self::InvalidTime => f.write_str("approval time bound is invalid"),
            Self::InvalidUsageLimit => f.write_str("approval usage limit is invalid"),
            Self::MissingExpiry => f.write_str("approval class requires a finite expiry"),
            Self::MissingAuthorityDecision => {
                f.write_str("approval issuance has no durable authorization decision")
            }
            Self::AuthorityDecisionMismatch => f.write_str(
                "approval issuance authorization decision does not match exact current authority",
            ),
            Self::StaleAuthorityDecision => {
                f.write_str("approval issuance authorization decision is stale")
            }
            Self::EffectNotFound => f.write_str("approval issuance effect does not exist"),
            Self::EffectMismatch => f.write_str(
                "approval issuance effect is not exact authorized at-most-once elevated work",
            ),
            Self::DuplicateApproval => f.write_str("approval already exists"),
            Self::InvalidStoredRecord(reason) => {
                write!(f, "stored approval authority record is invalid: {reason}")
            }
        }
    }
}

impl Error for ApprovalBindingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::Scope(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for ApprovalBindingError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for ApprovalBindingError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for ApprovalBindingError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<ApprovalScopeError> for ApprovalBindingError {
    fn from(value: ApprovalScopeError) -> Self {
        Self::Scope(value)
    }
}

pub fn prepare_approval(
    approver_principal: &str,
    scope: ApprovalScope,
    risk_class: &str,
    taint_digest: [u8; 32],
    issued_at: &str,
    expires_at: Option<&str>,
    max_uses: u64,
) -> Result<PreparedApproval, ApprovalBindingError> {
    validate_text(approver_principal, MAX_PRINCIPAL_BYTES)
        .map_err(|_| ApprovalBindingError::InvalidApprover)?;
    validate_risk_class(risk_class)?;
    if !valid_utc_second(issued_at) {
        return Err(ApprovalBindingError::InvalidTime);
    }
    let expires_at = match expires_at {
        Some(value) if valid_utc_second(value) && issued_at < value => Some(value.to_owned()),
        Some(_) => return Err(ApprovalBindingError::InvalidTime),
        None => None,
    };
    if max_uses == 0 {
        return Err(ApprovalBindingError::InvalidUsageLimit);
    }
    validate_class_bounds(&scope, expires_at.as_deref(), max_uses)?;

    let scope_bytes = scope.canonical_bytes()?;
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(APPROVAL_INTENT_DOMAIN)?;
    encoder.push_bytes(approver_principal.as_bytes())?;
    encoder.push_bytes(&scope_bytes)?;
    encoder.push_bytes(risk_class.as_bytes())?;
    encoder.push_bytes(&taint_digest)?;
    encoder.push_bytes(issued_at.as_bytes())?;
    encode_optional_text(&mut encoder, expires_at.as_deref())?;
    encoder.push_u64(max_uses);
    let intent_digest = *blake3::hash(&encoder.finish()).as_bytes();
    let resource = format!("approval-issue:{}", hex_bytes(&intent_digest));

    Ok(PreparedApproval {
        approver_principal: approver_principal.to_owned(),
        scope,
        risk_class: risk_class.to_owned(),
        taint_digest,
        issued_at: issued_at.to_owned(),
        expires_at,
        max_uses,
        intent_digest,
        resource,
    })
}

pub struct ApprovalStore {
    connection: Connection,
}

impl ApprovalStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, ApprovalBindingError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn issue(
        &mut self,
        prepared: PreparedApproval,
        parent_decision_id: [u8; 16],
        issue_effect_id: EffectId,
    ) -> Result<ApprovalRecord, ApprovalBindingError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        let authority = verify_current_authority(
            &transaction,
            parent_decision_id,
            &prepared.approver_principal,
            &prepared.resource,
        )?;
        verify_issue_effect(
            &transaction,
            issue_effect_id,
            &prepared.resource,
            prepared.intent_digest,
        )?;
        let scope_digest =
            bound_scope_digest(&prepared, parent_decision_id, authority.context_hash)?;
        let approval_id = approval_id(scope_digest, issue_effect_id);
        let exists = transaction
            .query_row(
                "SELECT 1 FROM approvals WHERE approval_id = ?1 LIMIT 1",
                params![&approval_id[..]],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if exists {
            return Err(ApprovalBindingError::DuplicateApproval);
        }
        let stored_scope = stored_scope(&prepared.scope);
        transaction.execute(
            "INSERT INTO approvals (approval_id, class, approver_principal, scope_digest, action_scope, resource_scope, effect_id, session_id, risk_class, taint_digest, parent_decision_id, issued_at, expires_at, max_uses, revoked_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, NULL)",
            params![
                &approval_id[..],
                prepared.scope.class().as_str(),
                &prepared.approver_principal,
                &scope_digest[..],
                &stored_scope.action_scope,
                &stored_scope.resource_scope,
                stored_scope.effect_id.map(|id| id.to_vec()),
                stored_scope.session_id.map(|id| id.to_vec()),
                &prepared.risk_class,
                &prepared.taint_digest[..],
                &parent_decision_id[..],
                &prepared.issued_at,
                prepared.expires_at.as_deref(),
                to_i64(prepared.max_uses)?,
            ],
        )?;
        append_approval_snapshot(&transaction, &approval_id)
            .map_err(|error| ApprovalBindingError::AuthoritySecurity(error.to_string()))?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| ApprovalBindingError::AuthoritySecurity(error.to_string()))?;
        transaction.commit()?;

        Ok(ApprovalRecord {
            approval_id,
            approver_principal: prepared.approver_principal,
            scope: prepared.scope,
            scope_digest,
            risk_class: prepared.risk_class,
            taint_digest: prepared.taint_digest,
            parent_decision_id,
            issued_at: prepared.issued_at,
            expires_at: prepared.expires_at,
            max_uses: Some(prepared.max_uses),
            revoked_at: None,
        })
    }
}

struct AuthorityEvidence {
    context_hash: [u8; 32],
}

struct StoredScope {
    action_scope: Vec<u8>,
    resource_scope: Vec<u8>,
    effect_id: Option<[u8; 16]>,
    session_id: Option<[u8; 16]>,
}

fn verify_transaction_integrity(transaction: &Transaction<'_>) -> Result<(), ApprovalBindingError> {
    crate::integrity::verify(transaction)
        .map_err(|error| ApprovalBindingError::Integrity(error.to_string()))?;
    crate::authority_security_v2::verify(transaction)
        .map_err(|error| ApprovalBindingError::AuthoritySecurity(error.to_string()))
}

fn verify_current_authority(
    transaction: &Transaction<'_>,
    decision_id: [u8; 16],
    approver_principal: &str,
    expected_resource: &str,
) -> Result<AuthorityEvidence, ApprovalBindingError> {
    let row = transaction
        .query_row(
            "SELECT principal, action, resource, context_hash, decision, global_seq FROM authorization_decisions WHERE decision_id = ?1",
            params![&decision_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            },
        )
        .optional()?
        .ok_or(ApprovalBindingError::MissingAuthorityDecision)?;
    if row.0 != approver_principal
        || row.1 != APPROVAL_ISSUE_ACTION
        || row.2 != expected_resource
        || row.4 != "allow"
    {
        return Err(ApprovalBindingError::AuthorityDecisionMismatch);
    }
    let global_seq = seq_from_i64(row.5)?;
    let latest: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(global_seq), 0) FROM (SELECT global_seq FROM session_events UNION ALL SELECT global_seq FROM effect_transitions UNION ALL SELECT global_seq FROM authorization_decisions)",
        [],
        |row| row.get(0),
    )?;
    if global_seq != seq_from_i64(latest)? {
        return Err(ApprovalBindingError::StaleAuthorityDecision);
    }
    let context_hash = row.3.try_into().map_err(|_| {
        ApprovalBindingError::InvalidStoredRecord("authorization context hash is not 32 bytes")
    })?;
    Ok(AuthorityEvidence { context_hash })
}

fn verify_issue_effect(
    transaction: &Transaction<'_>,
    effect_id: EffectId,
    expected_resource: &str,
    expected_payload_hash: [u8; 32],
) -> Result<(), ApprovalBindingError> {
    let row = transaction
        .query_row(
            "SELECT i.action, i.resource, i.risk_class, i.execution_semantics, i.payload_hash, t.to_state FROM effect_intents i JOIN effect_transitions t ON t.effect_id = i.effect_id WHERE i.effect_id = ?1 AND t.global_seq = (SELECT MAX(t2.global_seq) FROM effect_transitions t2 WHERE t2.effect_id = i.effect_id)",
            params![&effect_id.0.to_be_bytes()[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .optional()?
        .ok_or(ApprovalBindingError::EffectNotFound)?;
    if row.0 != APPROVAL_ISSUE_ACTION
        || row.1 != expected_resource
        || row.2 != APPROVAL_MUTATION_RISK_CLASS
        || row.3 != "at_most_once"
        || row.4.as_slice() != expected_payload_hash
        || row.5 != "authorized"
    {
        return Err(ApprovalBindingError::EffectMismatch);
    }
    Ok(())
}

fn bound_scope_digest(
    prepared: &PreparedApproval,
    parent_decision_id: [u8; 16],
    context_hash: [u8; 32],
) -> Result<[u8; 32], ApprovalBindingError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(APPROVAL_BINDING_DOMAIN)?;
    encoder.push_bytes(&prepared.intent_digest)?;
    encoder.push_bytes(&parent_decision_id)?;
    encoder.push_bytes(&context_hash)?;
    Ok(*blake3::hash(&encoder.finish()).as_bytes())
}

fn approval_id(scope_digest: [u8; 32], effect_id: EffectId) -> [u8; 16] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(APPROVAL_ID_DOMAIN);
    hasher.update(&scope_digest);
    hasher.update(&effect_id.0.to_be_bytes());
    let digest = hasher.finalize();
    let mut id = [0_u8; 16];
    id.copy_from_slice(&digest.as_bytes()[..16]);
    id
}

fn stored_scope(scope: &ApprovalScope) -> StoredScope {
    match scope {
        ApprovalScope::Once {
            effect_id,
            action,
            resource,
        } => StoredScope {
            action_scope: action.as_bytes().to_vec(),
            resource_scope: resource.as_bytes().to_vec(),
            effect_id: Some(effect_id.0.to_be_bytes()),
            session_id: None,
        },
        ApprovalScope::SessionScoped {
            session_id,
            actions,
            resources,
        } => StoredScope {
            action_scope: encode_values(actions),
            resource_scope: encode_values(resources),
            effect_id: None,
            session_id: Some(session_id.0.to_be_bytes()),
        },
        ApprovalScope::TimeBoxed { actions, resources } => StoredScope {
            action_scope: encode_values(actions),
            resource_scope: encode_values(resources),
            effect_id: None,
            session_id: None,
        },
        ApprovalScope::OperationPattern {
            action_pattern,
            resource_pattern,
        } => StoredScope {
            action_scope: action_pattern.as_bytes().to_vec(),
            resource_scope: resource_pattern.as_bytes().to_vec(),
            effect_id: None,
            session_id: None,
        },
        ApprovalScope::RunPreauthorization {
            session_id,
            actions,
            resources,
        } => StoredScope {
            action_scope: encode_values(actions),
            resource_scope: encode_values(resources),
            effect_id: None,
            session_id: session_id.map(|id| id.0.to_be_bytes()),
        },
    }
}

fn validate_class_bounds(
    scope: &ApprovalScope,
    expires_at: Option<&str>,
    max_uses: u64,
) -> Result<(), ApprovalBindingError> {
    match scope {
        ApprovalScope::Once { .. } if max_uses != 1 => Err(ApprovalBindingError::InvalidUsageLimit),
        ApprovalScope::SessionScoped { .. }
        | ApprovalScope::TimeBoxed { .. }
        | ApprovalScope::RunPreauthorization { .. }
            if expires_at.is_none() =>
        {
            Err(ApprovalBindingError::MissingExpiry)
        }
        _ => Ok(()),
    }
}

fn validate_risk_class(value: &str) -> Result<(), ApprovalBindingError> {
    if value.is_empty() || value.len() > MAX_RISK_CLASS_BYTES {
        return Err(ApprovalBindingError::InvalidRiskClass);
    }
    let bytes = value.as_bytes();
    if !bytes.first().is_some_and(u8::is_ascii_lowercase)
        || !bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        || bytes.iter().any(|byte| {
            !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(*byte, b'_' | b'-'))
        })
    {
        return Err(ApprovalBindingError::InvalidRiskClass);
    }
    Ok(())
}

fn validate_text(value: &str, max_bytes: usize) -> Result<(), ()> {
    if value.is_empty()
        || value.len() > max_bytes
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        Err(())
    } else {
        Ok(())
    }
}

fn valid_utc_second(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    for index in [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18] {
        if !bytes[index].is_ascii_digit() {
            return false;
        }
    }
    let year = decimal(bytes, 0, 4);
    let month = decimal(bytes, 5, 7);
    let day = decimal(bytes, 8, 10);
    let hour = decimal(bytes, 11, 13);
    let minute = decimal(bytes, 14, 16);
    let second = decimal(bytes, 17, 19);
    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    let max_day = match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    (1..=max_day).contains(&day)
}

fn decimal(bytes: &[u8], start: usize, end: usize) -> u32 {
    bytes[start..end]
        .iter()
        .fold(0_u32, |value, byte| value * 10 + u32::from(*byte - b'0'))
}

fn is_leap_year(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

fn encode_optional_text(
    encoder: &mut CanonicalEncoder,
    value: Option<&str>,
) -> Result<(), ApprovalBindingError> {
    match value {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(value.as_bytes())?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn encode_values(values: &[String]) -> Vec<u8> {
    values.join("\n").into_bytes()
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(char::from(HEX[(byte >> 4) as usize]));
        value.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    value
}

fn seq_from_i64(value: i64) -> Result<u64, ApprovalBindingError> {
    u64::try_from(value).map_err(|_| ApprovalBindingError::InvalidStoredRecord("negative sequence"))
}

fn to_i64(value: u64) -> Result<i64, ApprovalBindingError> {
    i64::try_from(value).map_err(|_| ApprovalBindingError::InvalidUsageLimit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorization::{
        AppendAuthorizationDecision, AuthorizationAuditLog, AuthorizationDecisionEvidence,
        AuthorizationDecisionKind,
    };
    use crate::dispatch::encode_effect_dependencies;
    use crate::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
    use golam_core::paths::RuntimeLayout;
    use golam_core::{EffectTransitionId, EventId, SessionId};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    fn authority(label: &str) -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-approval-binding-{label}-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    fn authorize_effect(
        authority: &AuthorityLayout,
        prepared: &PreparedApproval,
        effect_id: EffectId,
    ) {
        let dependencies = encode_effect_dependencies(&[]).unwrap();
        let mut effects = EffectStore::open(authority).unwrap();
        effects
            .propose(ProposeEffect {
                effect_id,
                session_id: SessionId(1),
                requested_by: "owner:owner",
                action: APPROVAL_ISSUE_ACTION,
                resource: prepared.resource(),
                risk_class: APPROVAL_MUTATION_RISK_CLASS,
                execution_semantics: "at_most_once",
                idempotency_key: None,
                preconditions: b"[]",
                dependencies: &dependencies,
                payload_hash: prepared.intent_digest(),
                proposed_event_id: EventId(effect_id.0 + 100),
                transition_id: EffectTransitionId(effect_id.0 + 101),
            })
            .unwrap();
        effects
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(effect_id.0 + 102),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: Some("approval_issue_authorized"),
                evidence_ref: None,
                event_id: EventId(effect_id.0 + 103),
            })
            .unwrap();
    }

    fn authorize_issue(
        authority: &AuthorityLayout,
        prepared: &PreparedApproval,
        context: &str,
    ) -> [u8; 16] {
        let mut log = AuthorizationAuditLog::open(authority).unwrap();
        log.append(AppendAuthorizationDecision {
            principal: "owner:owner",
            action: APPROVAL_ISSUE_ACTION,
            resource: prepared.resource(),
            context,
            evidence: AuthorizationDecisionEvidence::hard_guard_only("pass"),
            decision: AuthorizationDecisionKind::Allow,
            reason_code: "test_approval_issue_current_authority",
        })
        .unwrap()
        .decision_id
    }

    #[test]
    fn protected_once_approval_binds_scope_authority_context_and_integrity() {
        let (runtime, authority) = authority("once");
        let prepared = prepare_approval(
            "owner:owner",
            ApprovalScope::once(EffectId(700), "effect.simulate", "session:7").unwrap(),
            "irreversible_effect",
            [9; 32],
            "2026-08-27T00:00:00Z",
            Some("2026-08-27T01:00:00Z"),
            1,
        )
        .unwrap();
        let issue_effect = EffectId(1_000);
        authorize_effect(&authority, &prepared, issue_effect);
        let decision = authorize_issue(&authority, &prepared, "scope=local-owner");
        let record = ApprovalStore::open(&authority)
            .unwrap()
            .issue(prepared, decision, issue_effect)
            .unwrap();
        assert_eq!(record.class(), ApprovalClass::Once);
        assert_eq!(record.parent_decision_id(), decision);
        assert_eq!(record.max_uses(), Some(1));
        drop(ApprovalStore::open(&authority).unwrap());
        AuthorityStore::open(authority.authority_db_path()).unwrap();
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn authority_context_is_part_of_bound_scope_digest() {
        fn issue(context: &str, label: &str, effect_number: u128) -> [u8; 32] {
            let (runtime, authority) = authority(label);
            let prepared = prepare_approval(
                "owner:owner",
                ApprovalScope::time_boxed(&["session.read".to_owned()], &["session:7".to_owned()])
                    .unwrap(),
                "sensitive_read",
                [3; 32],
                "2026-08-27T00:00:00Z",
                Some("2026-08-27T01:00:00Z"),
                4,
            )
            .unwrap();
            let effect = EffectId(effect_number);
            authorize_effect(&authority, &prepared, effect);
            let decision = authorize_issue(&authority, &prepared, context);
            let digest = ApprovalStore::open(&authority)
                .unwrap()
                .issue(prepared, decision, effect)
                .unwrap()
                .scope_digest();
            fs::remove_dir_all(runtime.root).unwrap();
            digest
        }
        assert_ne!(
            issue("scope=local-owner", "context-a", 2_000),
            issue("scope=local-session:7", "context-b", 3_000)
        );
    }

    #[test]
    fn bounded_classes_reject_missing_expiry_or_invalid_usage_limit() {
        let scope = ApprovalScope::session_scoped(
            SessionId(7),
            &["session.read".to_owned()],
            &["session:7".to_owned()],
        )
        .unwrap();
        assert!(matches!(
            prepare_approval(
                "owner:owner",
                scope,
                "sensitive_read",
                [0; 32],
                "2026-08-27T00:00:00Z",
                None,
                3,
            ),
            Err(ApprovalBindingError::MissingExpiry)
        ));
        let once = ApprovalScope::once(EffectId(7), "effect.simulate", "session:7").unwrap();
        assert!(matches!(
            prepare_approval(
                "owner:owner",
                once,
                "irreversible_effect",
                [0; 32],
                "2026-08-27T00:00:00Z",
                None,
                2,
            ),
            Err(ApprovalBindingError::InvalidUsageLimit)
        ));
    }

    #[test]
    fn mismatched_approver_cannot_reuse_an_allow_decision() {
        let (runtime, authority) = authority("mismatch");
        let prepared = prepare_approval(
            "owner:other",
            ApprovalScope::once(EffectId(9), "effect.simulate", "session:9").unwrap(),
            "irreversible_effect",
            [0; 32],
            "2026-08-27T00:00:00Z",
            None,
            1,
        )
        .unwrap();
        let issue_effect = EffectId(4_000);
        authorize_effect(&authority, &prepared, issue_effect);
        let mut log = AuthorizationAuditLog::open(&authority).unwrap();
        let decision = log
            .append(AppendAuthorizationDecision {
                principal: "owner:owner",
                action: APPROVAL_ISSUE_ACTION,
                resource: prepared.resource(),
                context: "scope=local-owner",
                evidence: AuthorizationDecisionEvidence::hard_guard_only("pass"),
                decision: AuthorizationDecisionKind::Allow,
                reason_code: "test_wrong_approver",
            })
            .unwrap()
            .decision_id;
        assert!(matches!(
            ApprovalStore::open(&authority)
                .unwrap()
                .issue(prepared, decision, issue_effect),
            Err(ApprovalBindingError::AuthorityDecisionMismatch)
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
