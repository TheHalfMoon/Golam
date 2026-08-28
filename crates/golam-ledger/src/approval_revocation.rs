#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::approval_binding::APPROVAL_MUTATION_RISK_CLASS;
use crate::authority_security_write::append_approval_snapshot;
use crate::storage::{AuthorityStore, StorageError};

pub const APPROVAL_REVOKE_ACTION: &str = "approval.revoke";

const MAX_PRINCIPAL_BYTES: usize = 512;
const REVOKE_INTENT_DOMAIN: &[u8] = b"golam:approval-revoke-intent:v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedApprovalRevocation {
    approval_id: [u8; 16],
    revoked_by_principal: String,
    revoked_at: String,
    intent_digest: [u8; 32],
    resource: String,
}

impl PreparedApprovalRevocation {
    pub const fn approval_id(&self) -> [u8; 16] {
        self.approval_id
    }

    pub fn resource(&self) -> &str {
        &self.resource
    }

    pub const fn intent_digest(&self) -> [u8; 32] {
        self.intent_digest
    }
}

#[derive(Debug)]
pub enum ApprovalRevocationError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Integrity(String),
    AuthoritySecurity(String),
    InvalidPrincipal,
    InvalidTime,
    MissingAuthorityDecision,
    AuthorityDecisionMismatch,
    StaleAuthorityDecision,
    EffectNotFound,
    EffectMismatch,
    ApprovalNotFound,
    ApprovalAlreadyRevoked,
    RevocationPredatesApproval,
    InvalidStoredRecord(&'static str),
}

impl fmt::Display for ApprovalRevocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "approval revocation authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "approval revocation sqlite error: {error}"),
            Self::Core(error) => write!(f, "approval revocation canonical encoding error: {error}"),
            Self::Integrity(error) => write!(f, "approval revocation integrity error: {error}"),
            Self::AuthoritySecurity(error) => {
                write!(f, "approval revocation authority-security error: {error}")
            }
            Self::InvalidPrincipal => {
                f.write_str("approval revocation principal is not canonical")
            }
            Self::InvalidTime => f.write_str("approval revocation time is not canonical UTC-second"),
            Self::MissingAuthorityDecision => {
                f.write_str("approval revocation has no durable authorization decision")
            }
            Self::AuthorityDecisionMismatch => f.write_str(
                "approval revocation authorization decision does not match exact principal/action/resource",
            ),
            Self::StaleAuthorityDecision => {
                f.write_str("approval revocation authorization decision is stale")
            }
            Self::EffectNotFound => f.write_str("approval revocation effect does not exist"),
            Self::EffectMismatch => f.write_str(
                "approval revocation effect is not exact authorized at-most-once elevated work",
            ),
            Self::ApprovalNotFound => f.write_str("approval revocation target does not exist"),
            Self::ApprovalAlreadyRevoked => f.write_str("approval is already revoked"),
            Self::RevocationPredatesApproval => {
                f.write_str("approval revocation cannot predate approval issuance")
            }
            Self::InvalidStoredRecord(reason) => {
                write!(f, "stored approval revocation record is invalid: {reason}")
            }
        }
    }
}

impl Error for ApprovalRevocationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for ApprovalRevocationError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for ApprovalRevocationError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for ApprovalRevocationError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

pub fn prepare_approval_revocation(
    approval_id: [u8; 16],
    revoked_by_principal: &str,
    revoked_at: &str,
) -> Result<PreparedApprovalRevocation, ApprovalRevocationError> {
    validate_principal(revoked_by_principal)?;
    if !valid_utc_second(revoked_at) {
        return Err(ApprovalRevocationError::InvalidTime);
    }

    let resource = approval_resource(approval_id);
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(REVOKE_INTENT_DOMAIN)?;
    encoder.push_bytes(&approval_id)?;
    encoder.push_bytes(revoked_by_principal.as_bytes())?;
    encoder.push_bytes(revoked_at.as_bytes())?;
    let intent_digest = *blake3::hash(&encoder.finish()).as_bytes();

    Ok(PreparedApprovalRevocation {
        approval_id,
        revoked_by_principal: revoked_by_principal.to_owned(),
        revoked_at: revoked_at.to_owned(),
        intent_digest,
        resource,
    })
}

pub fn approval_resource(approval_id: [u8; 16]) -> String {
    format!("approval:{}", hex_bytes(&approval_id))
}

pub struct ApprovalRevocationStore {
    connection: Connection,
}

impl ApprovalRevocationStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, ApprovalRevocationError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn revoke(
        &mut self,
        prepared: PreparedApprovalRevocation,
        authority_decision_id: [u8; 16],
        effect_id: EffectId,
    ) -> Result<(), ApprovalRevocationError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        verify_current_authority(
            &transaction,
            authority_decision_id,
            &prepared.revoked_by_principal,
            &prepared.resource,
        )?;
        verify_revoke_effect(
            &transaction,
            effect_id,
            &prepared.resource,
            prepared.intent_digest,
        )?;
        verify_target(&transaction, prepared.approval_id, &prepared.revoked_at)?;

        let updated = transaction.execute(
            "UPDATE approvals SET revoked_at = ?1 WHERE approval_id = ?2 AND revoked_at IS NULL",
            params![&prepared.revoked_at, &prepared.approval_id[..]],
        )?;
        if updated != 1 {
            return Err(ApprovalRevocationError::ApprovalAlreadyRevoked);
        }
        append_approval_snapshot(&transaction, &prepared.approval_id)
            .map_err(|error| ApprovalRevocationError::AuthoritySecurity(error.to_string()))?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| ApprovalRevocationError::AuthoritySecurity(error.to_string()))?;
        transaction.commit()?;
        Ok(())
    }
}

fn verify_transaction_integrity(
    transaction: &Transaction<'_>,
) -> Result<(), ApprovalRevocationError> {
    crate::integrity::verify(transaction)
        .map_err(|error| ApprovalRevocationError::Integrity(error.to_string()))?;
    crate::authority_security_v2::verify(transaction)
        .map_err(|error| ApprovalRevocationError::AuthoritySecurity(error.to_string()))
}

fn verify_current_authority(
    transaction: &Transaction<'_>,
    decision_id: [u8; 16],
    expected_principal: &str,
    expected_resource: &str,
) -> Result<(), ApprovalRevocationError> {
    let row = transaction
        .query_row(
            "SELECT principal, action, resource, decision, global_seq FROM authorization_decisions WHERE decision_id = ?1",
            params![&decision_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            },
        )
        .optional()?
        .ok_or(ApprovalRevocationError::MissingAuthorityDecision)?;
    if row.0 != expected_principal
        || row.1 != APPROVAL_REVOKE_ACTION
        || row.2 != expected_resource
        || row.3 != "allow"
    {
        return Err(ApprovalRevocationError::AuthorityDecisionMismatch);
    }
    let global_seq = from_i64(row.4, "approval revocation decision sequence is negative")?;
    let latest: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(global_seq), 0) FROM (SELECT global_seq FROM session_events UNION ALL SELECT global_seq FROM effect_transitions UNION ALL SELECT global_seq FROM authorization_decisions)",
        [],
        |row| row.get(0),
    )?;
    if global_seq
        != from_i64(
            latest,
            "approval revocation latest authority sequence is negative",
        )?
    {
        return Err(ApprovalRevocationError::StaleAuthorityDecision);
    }
    Ok(())
}

fn verify_revoke_effect(
    transaction: &Transaction<'_>,
    effect_id: EffectId,
    expected_resource: &str,
    expected_payload_hash: [u8; 32],
) -> Result<(), ApprovalRevocationError> {
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
        .ok_or(ApprovalRevocationError::EffectNotFound)?;
    if row.0 != APPROVAL_REVOKE_ACTION
        || row.1 != expected_resource
        || row.2 != APPROVAL_MUTATION_RISK_CLASS
        || row.3 != "at_most_once"
        || row.4.as_slice() != expected_payload_hash
        || row.5 != "authorized"
    {
        return Err(ApprovalRevocationError::EffectMismatch);
    }
    Ok(())
}

fn verify_target(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    revoked_at: &str,
) -> Result<(), ApprovalRevocationError> {
    let row = transaction
        .query_row(
            "SELECT issued_at, revoked_at FROM approvals WHERE approval_id = ?1",
            params![&approval_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                ))
            },
        )
        .optional()?
        .ok_or(ApprovalRevocationError::ApprovalNotFound)?;
    if !valid_utc_second(&row.0) {
        return Err(ApprovalRevocationError::InvalidStoredRecord(
            "approval issued_at is malformed",
        ));
    }
    if row.1.is_some() {
        return Err(ApprovalRevocationError::ApprovalAlreadyRevoked);
    }
    if revoked_at < row.0.as_str() {
        return Err(ApprovalRevocationError::RevocationPredatesApproval);
    }
    Ok(())
}

fn validate_principal(value: &str) -> Result<(), ApprovalRevocationError> {
    if value.is_empty()
        || value.len() > MAX_PRINCIPAL_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(ApprovalRevocationError::InvalidPrincipal);
    }
    Ok(())
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
    true
}

fn from_i64(value: i64, reason: &'static str) -> Result<u64, ApprovalRevocationError> {
    u64::try_from(value).map_err(|_| ApprovalRevocationError::InvalidStoredRecord(reason))
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[(byte >> 4) as usize]));
        output.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approval_binding::{
        APPROVAL_ISSUE_ACTION, ApprovalStore, prepare_approval,
    };
    use crate::approval_runtime::{ApprovalUseError, ApprovalUseRequest, ApprovalUseStore};
    use crate::approvals::ApprovalScope;
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
    static RECORD_N: AtomicU64 = AtomicU64::new(0);

    fn next_record_id() -> u128 {
        2_000_000 + u128::from(RECORD_N.fetch_add(1, Ordering::Relaxed))
    }

    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-approval-revocation-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    #[allow(clippy::too_many_arguments)]
    fn create_authorized_effect(
        authority: &AuthorityLayout,
        effect_id: EffectId,
        action: &str,
        resource: &str,
        risk_class: &str,
        payload_hash: [u8; 32],
        reason_code: &str,
        session_id: SessionId,
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
                execution_semantics: "at_most_once",
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
                reason_code: Some(reason_code),
                evidence_ref: None,
                event_id: EventId(next_record_id()),
            })
            .unwrap();
    }

    fn append_allow_decision(
        authority: &AuthorityLayout,
        action: &str,
        resource: &str,
        reason_code: &str,
    ) -> [u8; 16] {
        let mut log = AuthorizationAuditLog::open(authority).unwrap();
        log.append(AppendAuthorizationDecision {
            principal: "owner:owner",
            action,
            resource,
            context: "scope=local-owner",
            evidence: AuthorizationDecisionEvidence::hard_guard_only("pass"),
            decision: AuthorizationDecisionKind::Allow,
            reason_code,
        })
        .unwrap()
        .decision_id
    }

    fn issue_time_boxed_approval(authority: &AuthorityLayout) -> [u8; 16] {
        let actions = vec!["effect.simulate".to_owned()];
        let resources = vec!["resource:item".to_owned()];
        let prepared = prepare_approval(
            "owner:owner",
            ApprovalScope::time_boxed(&actions, &resources).unwrap(),
            "dangerous_effect",
            [9; 32],
            "2026-08-27T00:00:00Z",
            Some("2026-08-27T01:00:00Z"),
            4,
        )
        .unwrap();
        let issue_effect_id = EffectId(10_000);
        create_authorized_effect(
            authority,
            issue_effect_id,
            APPROVAL_ISSUE_ACTION,
            prepared.resource(),
            APPROVAL_MUTATION_RISK_CLASS,
            prepared.intent_digest(),
            "test_approval_issue",
            SessionId(1),
        );
        let decision_id = append_allow_decision(
            authority,
            APPROVAL_ISSUE_ACTION,
            prepared.resource(),
            "test_approval_issue_authority",
        );
        ApprovalStore::open(authority)
            .unwrap()
            .issue(prepared, decision_id, issue_effect_id)
            .unwrap()
            .approval_id
    }

    #[test]
    fn protected_revocation_immediately_denies_exact_approval_use() {
        let (runtime, authority) = authority();
        let approval_id = issue_time_boxed_approval(&authority);
        let request = ApprovalUseRequest {
            approval_id,
            action: "effect.simulate",
            resource: "resource:item",
            effect_id: None,
            session_id: None,
            risk_class: "dangerous_effect",
            taint_digest: [9; 32],
            observed_at: "2026-08-27T00:10:00Z",
        };
        ApprovalUseStore::open(&authority)
            .unwrap()
            .validate(request)
            .unwrap();

        let prepared = prepare_approval_revocation(
            approval_id,
            "owner:owner",
            "2026-08-27T00:11:00Z",
        )
        .unwrap();
        let revoke_effect_id = EffectId(10_001);
        create_authorized_effect(
            &authority,
            revoke_effect_id,
            APPROVAL_REVOKE_ACTION,
            prepared.resource(),
            APPROVAL_MUTATION_RISK_CLASS,
            prepared.intent_digest(),
            "test_approval_revoke",
            SessionId(1),
        );
        let decision_id = append_allow_decision(
            &authority,
            APPROVAL_REVOKE_ACTION,
            prepared.resource(),
            "test_approval_revoke_authority",
        );
        ApprovalRevocationStore::open(&authority)
            .unwrap()
            .revoke(prepared, decision_id, revoke_effect_id)
            .unwrap();

        let denied = ApprovalUseStore::open(&authority)
            .unwrap()
            .validate(ApprovalUseRequest {
                observed_at: "2026-08-27T00:12:00Z",
                ..request
            });
        assert!(matches!(denied, Err(ApprovalUseError::Revoked)));

        drop(AuthorityStore::open(authority.authority_db_path()).unwrap());
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn revocation_is_monotonic_and_rejects_stale_or_mismatched_effects() {
        let (runtime, authority) = authority();
        let approval_id = issue_time_boxed_approval(&authority);

        let first = prepare_approval_revocation(
            approval_id,
            "owner:owner",
            "2026-08-27T00:11:00Z",
        )
        .unwrap();
        let first_effect_id = EffectId(10_010);
        create_authorized_effect(
            &authority,
            first_effect_id,
            APPROVAL_REVOKE_ACTION,
            first.resource(),
            APPROVAL_MUTATION_RISK_CLASS,
            first.intent_digest(),
            "test_first_revoke",
            SessionId(1),
        );
        let first_decision = append_allow_decision(
            &authority,
            APPROVAL_REVOKE_ACTION,
            first.resource(),
            "test_first_revoke_authority",
        );
        ApprovalRevocationStore::open(&authority)
            .unwrap()
            .revoke(first, first_decision, first_effect_id)
            .unwrap();

        let second = prepare_approval_revocation(
            approval_id,
            "owner:owner",
            "2026-08-27T00:12:00Z",
        )
        .unwrap();
        let second_effect_id = EffectId(10_011);
        create_authorized_effect(
            &authority,
            second_effect_id,
            APPROVAL_REVOKE_ACTION,
            second.resource(),
            APPROVAL_MUTATION_RISK_CLASS,
            second.intent_digest(),
            "test_second_revoke",
            SessionId(1),
        );
        let second_decision = append_allow_decision(
            &authority,
            APPROVAL_REVOKE_ACTION,
            second.resource(),
            "test_second_revoke_authority",
        );
        assert!(matches!(
            ApprovalRevocationStore::open(&authority)
                .unwrap()
                .revoke(second, second_decision, second_effect_id),
            Err(ApprovalRevocationError::ApprovalAlreadyRevoked)
        ));

        drop(AuthorityStore::open(authority.authority_db_path()).unwrap());
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
