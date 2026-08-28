#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::taint::{TaintLabel, TaintSet};
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::authority_security_write::{
    append_approval_consumption_snapshot, append_verifier_rule_snapshot,
};
use crate::storage::{AuthorityStore, StorageError};

pub const VERIFIER_RULE_REGISTER_ACTION: &str = "verifier.register";
pub const TAINT_AUTHORITY_MUTATION_RISK_CLASS: &str = "taint_authority_mutation";

const MAX_PRINCIPAL_BYTES: usize = 512;
const MAX_AUTHORITY_SOURCE_BINDING_BYTES: usize = 4096;
const RULE_ID_DOMAIN: &[u8] = b"golam:verifier-rule-id:v1";
const REGISTER_INTENT_DOMAIN: &[u8] = b"golam:verifier-rule-register-intent:v1";
const APPROVAL_CONSUMPTION_DOMAIN: &[u8] = b"golam:verifier-rule-approval-consumption:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerifierRuleKind {
    DeterministicVerifier,
    SecretEliminationSanitizer,
}

impl VerifierRuleKind {
    const fn code(self) -> u8 {
        match self {
            Self::DeterministicVerifier => 1,
            Self::SecretEliminationSanitizer => 2,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::DeterministicVerifier => "deterministic_verifier",
            Self::SecretEliminationSanitizer => "secret_elimination_sanitizer",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedVerifierRule {
    rule_id: [u8; 16],
    kind: VerifierRuleKind,
    version: u64,
    authority_source_binding: Vec<u8>,
    allowed_downgrades: TaintSet,
    allowed_downgrades_bytes: Vec<u8>,
    registered_by_principal: String,
    registration_taint_digest: [u8; 32],
    intent_digest: [u8; 32],
    resource: String,
}

impl PreparedVerifierRule {
    pub const fn rule_id(&self) -> [u8; 16] {
        self.rule_id
    }

    pub fn resource(&self) -> &str {
        &self.resource
    }

    pub const fn intent_digest(&self) -> [u8; 32] {
        self.intent_digest
    }

    pub const fn registration_taint_digest(&self) -> [u8; 32] {
        self.registration_taint_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifierRuleRecord {
    pub rule_id: [u8; 16],
    pub kind: VerifierRuleKind,
    pub version: u64,
    pub authority_source_binding: Vec<u8>,
    pub allowed_downgrades: TaintSet,
    pub registered_by: String,
    pub created_global_seq: u64,
}

#[derive(Debug)]
pub enum VerifierRegistryError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Integrity(String),
    AuthoritySecurity(String),
    InvalidVersion,
    InvalidPrincipal,
    InvalidAuthoritySourceBinding,
    EmptyAllowedDowngrades,
    UntrustedRegistrationSource,
    IntegerOverflow,
    MissingAuthorityDecision,
    AuthorityDecisionMismatch,
    StaleAuthorityDecision,
    EffectNotFound,
    EffectMismatch,
    ApprovalNotFound,
    ApprovalMismatch,
    ApprovalAlreadyUsed,
    DuplicateRule,
    InvalidStoredRecord(&'static str),
}

impl fmt::Display for VerifierRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "verifier registry authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "verifier registry sqlite error: {error}"),
            Self::Core(error) => write!(f, "verifier registry canonical encoding error: {error}"),
            Self::Integrity(error) => write!(f, "verifier registry integrity error: {error}"),
            Self::AuthoritySecurity(error) => {
                write!(f, "verifier registry authority-security error: {error}")
            }
            Self::InvalidVersion => f.write_str("verifier rule version must be non-zero"),
            Self::InvalidPrincipal => f.write_str("verifier rule principal is not canonical"),
            Self::InvalidAuthoritySourceBinding => {
                f.write_str("verifier rule authority-source binding is invalid or too large")
            }
            Self::EmptyAllowedDowngrades => {
                f.write_str("verifier rule must declare at least one allowed downgrade label")
            }
            Self::UntrustedRegistrationSource => f.write_str(
                "untrusted or generated provenance cannot register verifier downgrade authority",
            ),
            Self::IntegerOverflow => f.write_str("verifier registry integer conversion overflow"),
            Self::MissingAuthorityDecision => {
                f.write_str("verifier registration has no durable authorization decision")
            }
            Self::AuthorityDecisionMismatch => f.write_str(
                "verifier registration decision does not match exact principal/action/resource",
            ),
            Self::StaleAuthorityDecision => {
                f.write_str("verifier registration authorization decision is stale")
            }
            Self::EffectNotFound => f.write_str("verifier registration effect does not exist"),
            Self::EffectMismatch => f.write_str(
                "verifier registration effect is not exact authorized at-most-once elevated work",
            ),
            Self::ApprovalNotFound => f.write_str("verifier registration approval does not exist"),
            Self::ApprovalMismatch => f.write_str(
                "verifier registration approval does not match exact effect/risk/provenance",
            ),
            Self::ApprovalAlreadyUsed => {
                f.write_str("verifier registration one-shot approval was already consumed")
            }
            Self::DuplicateRule => f.write_str("verifier rule already exists"),
            Self::InvalidStoredRecord(reason) => {
                write!(f, "stored verifier registry record is invalid: {reason}")
            }
        }
    }
}

impl Error for VerifierRegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for VerifierRegistryError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for VerifierRegistryError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for VerifierRegistryError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

pub fn prepare_verifier_rule(
    kind: VerifierRuleKind,
    version: u64,
    authority_source_binding: &[u8],
    allowed_downgrades: TaintSet,
    registered_by_principal: &str,
    registration_taint: TaintSet,
) -> Result<PreparedVerifierRule, VerifierRegistryError> {
    if version == 0 {
        return Err(VerifierRegistryError::InvalidVersion);
    }
    validate_principal(registered_by_principal)?;
    if authority_source_binding.is_empty()
        || authority_source_binding.len() > MAX_AUTHORITY_SOURCE_BINDING_BYTES
    {
        return Err(VerifierRegistryError::InvalidAuthoritySourceBinding);
    }
    if allowed_downgrades.is_empty() {
        return Err(VerifierRegistryError::EmptyAllowedDowngrades);
    }
    if !trusted_registration_source(registration_taint) {
        return Err(VerifierRegistryError::UntrustedRegistrationSource);
    }

    let allowed_downgrades_bytes = allowed_downgrades.canonical_bytes()?;
    let registration_taint_bytes = registration_taint.canonical_bytes()?;
    let registration_taint_digest = *blake3::hash(&registration_taint_bytes).as_bytes();

    let mut definition = CanonicalEncoder::new();
    definition.push_bytes(RULE_ID_DOMAIN)?;
    definition.push_u8(kind.code());
    definition.push_u64(version);
    definition.push_bytes(authority_source_binding)?;
    definition.push_bytes(&allowed_downgrades_bytes)?;
    let definition_hash = *blake3::hash(&definition.finish()).as_bytes();
    let mut rule_id = [0_u8; 16];
    rule_id.copy_from_slice(&definition_hash[..16]);
    let resource = verifier_rule_resource(rule_id);

    let mut intent = CanonicalEncoder::new();
    intent.push_bytes(REGISTER_INTENT_DOMAIN)?;
    intent.push_bytes(&rule_id)?;
    intent.push_bytes(registered_by_principal.as_bytes())?;
    intent.push_bytes(&registration_taint_digest)?;
    let intent_digest = *blake3::hash(&intent.finish()).as_bytes();

    Ok(PreparedVerifierRule {
        rule_id,
        kind,
        version,
        authority_source_binding: authority_source_binding.to_vec(),
        allowed_downgrades,
        allowed_downgrades_bytes,
        registered_by_principal: registered_by_principal.to_owned(),
        registration_taint_digest,
        intent_digest,
        resource,
    })
}

pub fn verifier_rule_resource(rule_id: [u8; 16]) -> String {
    format!("verifier-rule:{}", hex_bytes(&rule_id))
}

pub struct VerifierRuleStore {
    connection: Connection,
}

impl VerifierRuleStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, VerifierRegistryError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn register(
        &mut self,
        prepared: PreparedVerifierRule,
        authority_decision_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
    ) -> Result<VerifierRuleRecord, VerifierRegistryError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        let authority = verify_current_authority(
            &transaction,
            authority_decision_id,
            &prepared.registered_by_principal,
            &prepared.resource,
        )?;
        verify_registration_effect(
            &transaction,
            effect_id,
            &prepared.resource,
            prepared.intent_digest,
        )?;
        verify_once_approval(
            &transaction,
            approval_id,
            effect_id,
            &prepared.resource,
            prepared.registration_taint_digest,
        )?;

        let duplicate = transaction
            .query_row(
                "SELECT 1 FROM verifier_rules WHERE rule_id = ?1 LIMIT 1",
                params![&prepared.rule_id[..]],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if duplicate {
            return Err(VerifierRegistryError::DuplicateRule);
        }

        transaction.execute(
            "INSERT INTO verifier_rules (rule_id, kind, version, authority_source_binding, allowed_downgrades, registered_by, status, created_global_seq) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'active', ?7)",
            params![
                &prepared.rule_id[..],
                prepared.kind.as_str(),
                to_i64(prepared.version)?,
                &prepared.authority_source_binding,
                &prepared.allowed_downgrades_bytes,
                &authority.principal,
                to_i64(authority.global_seq)?,
            ],
        )?;
        append_verifier_rule_snapshot(&transaction, &prepared.rule_id)
            .map_err(|error| VerifierRegistryError::AuthoritySecurity(error.to_string()))?;
        consume_once_approval(&transaction, approval_id, effect_id, authority.global_seq)?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| VerifierRegistryError::AuthoritySecurity(error.to_string()))?;
        transaction.commit()?;

        Ok(VerifierRuleRecord {
            rule_id: prepared.rule_id,
            kind: prepared.kind,
            version: prepared.version,
            authority_source_binding: prepared.authority_source_binding,
            allowed_downgrades: prepared.allowed_downgrades,
            registered_by: authority.principal,
            created_global_seq: authority.global_seq,
        })
    }
}

struct AuthorityEvidence {
    principal: String,
    global_seq: u64,
}

fn verify_transaction_integrity(
    transaction: &Transaction<'_>,
) -> Result<(), VerifierRegistryError> {
    crate::integrity::verify(transaction)
        .map_err(|error| VerifierRegistryError::Integrity(error.to_string()))?;
    crate::authority_security_v2::verify(transaction)
        .map_err(|error| VerifierRegistryError::AuthoritySecurity(error.to_string()))
}

fn verify_current_authority(
    transaction: &Transaction<'_>,
    decision_id: [u8; 16],
    expected_principal: &str,
    expected_resource: &str,
) -> Result<AuthorityEvidence, VerifierRegistryError> {
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
        .ok_or(VerifierRegistryError::MissingAuthorityDecision)?;
    if row.0 != expected_principal
        || row.1 != VERIFIER_RULE_REGISTER_ACTION
        || row.2 != expected_resource
        || row.3 != "allow"
    {
        return Err(VerifierRegistryError::AuthorityDecisionMismatch);
    }
    let global_seq = from_i64(row.4, "verifier decision sequence is negative")?;
    let latest: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(global_seq), 0) FROM (SELECT global_seq FROM session_events UNION ALL SELECT global_seq FROM effect_transitions UNION ALL SELECT global_seq FROM authorization_decisions)",
        [],
        |row| row.get(0),
    )?;
    if global_seq != from_i64(latest, "latest authority sequence is negative")? {
        return Err(VerifierRegistryError::StaleAuthorityDecision);
    }
    Ok(AuthorityEvidence {
        principal: row.0,
        global_seq,
    })
}

fn verify_registration_effect(
    transaction: &Transaction<'_>,
    effect_id: EffectId,
    expected_resource: &str,
    expected_payload_hash: [u8; 32],
) -> Result<(), VerifierRegistryError> {
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
        .ok_or(VerifierRegistryError::EffectNotFound)?;
    if row.0 != VERIFIER_RULE_REGISTER_ACTION
        || row.1 != expected_resource
        || row.2 != TAINT_AUTHORITY_MUTATION_RISK_CLASS
        || row.3 != "at_most_once"
        || row.4.as_slice() != expected_payload_hash
        || row.5 != "authorized"
    {
        return Err(VerifierRegistryError::EffectMismatch);
    }
    Ok(())
}

fn verify_once_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    effect_id: EffectId,
    expected_resource: &str,
    expected_taint_digest: [u8; 32],
) -> Result<(), VerifierRegistryError> {
    let row = transaction
        .query_row(
            "SELECT class, action_scope, resource_scope, effect_id, session_id, risk_class, taint_digest, expires_at, max_uses, revoked_at FROM approvals WHERE approval_id = ?1",
            params![&approval_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, Option<Vec<u8>>>(3)?,
                    row.get::<_, Option<Vec<u8>>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Vec<u8>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<i64>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                ))
            },
        )
        .optional()?
        .ok_or(VerifierRegistryError::ApprovalNotFound)?;
    if row.0 != "ONCE"
        || row.1.as_slice() != VERIFIER_RULE_REGISTER_ACTION.as_bytes()
        || row.2.as_slice() != expected_resource.as_bytes()
        || row.3.as_deref() != Some(effect_id.0.to_be_bytes().as_slice())
        || row.4.is_some()
        || row.5 != TAINT_AUTHORITY_MUTATION_RISK_CLASS
        || row.6.as_slice() != expected_taint_digest
        || row.7.is_some()
        || row.8 != Some(1)
        || row.9.is_some()
    {
        return Err(VerifierRegistryError::ApprovalMismatch);
    }
    let already_used = transaction
        .query_row(
            "SELECT 1 FROM approval_consumptions WHERE approval_id = ?1 LIMIT 1",
            params![&approval_id[..]],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    if already_used {
        return Err(VerifierRegistryError::ApprovalAlreadyUsed);
    }
    Ok(())
}

fn consume_once_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    effect_id: EffectId,
    global_seq: u64,
) -> Result<(), VerifierRegistryError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(APPROVAL_CONSUMPTION_DOMAIN);
    hasher.update(&approval_id);
    hasher.update(&effect_id.0.to_be_bytes());
    let digest = hasher.finalize();
    let mut consumption_id = [0_u8; 16];
    consumption_id.copy_from_slice(&digest.as_bytes()[..16]);
    transaction.execute(
        "INSERT INTO approval_consumptions (consumption_id, approval_id, effect_or_operation_id, reserved_global_seq, consumed_global_seq, state) VALUES (?1, ?2, ?3, ?4, ?5, 'consumed')",
        params![
            &consumption_id[..],
            &approval_id[..],
            &effect_id.0.to_be_bytes()[..],
            to_i64(global_seq)?,
            to_i64(global_seq)?,
        ],
    )?;
    append_approval_consumption_snapshot(transaction, &consumption_id)
        .map_err(|error| VerifierRegistryError::AuthoritySecurity(error.to_string()))
}

fn trusted_registration_source(taint: TaintSet) -> bool {
    if taint.is_empty() {
        return false;
    }
    let forbidden = [
        TaintLabel::LocalUnverified,
        TaintLabel::WebUntrusted,
        TaintLabel::ChannelUntrusted,
        TaintLabel::McpUntrusted,
        TaintLabel::PluginUnverified,
        TaintLabel::ModelGenerated,
        TaintLabel::SecretDerived,
    ];
    let has_forbidden = forbidden.into_iter().any(|label| taint.contains(label));
    let has_trusted_origin =
        taint.contains(TaintLabel::UserTrusted) || taint.contains(TaintLabel::LocalTrusted);
    has_trusted_origin && !has_forbidden
}

fn validate_principal(value: &str) -> Result<(), VerifierRegistryError> {
    if value.is_empty()
        || value.len() > MAX_PRINCIPAL_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(VerifierRegistryError::InvalidPrincipal);
    }
    Ok(())
}

fn from_i64(value: i64, reason: &'static str) -> Result<u64, VerifierRegistryError> {
    u64::try_from(value).map_err(|_| VerifierRegistryError::InvalidStoredRecord(reason))
}

fn to_i64(value: u64) -> Result<i64, VerifierRegistryError> {
    i64::try_from(value).map_err(|_| VerifierRegistryError::IntegerOverflow)
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
        APPROVAL_ISSUE_ACTION, APPROVAL_MUTATION_RISK_CLASS, ApprovalStore, prepare_approval,
    };
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

    fn next_id() -> u128 {
        3_000_000 + u128::from(RECORD_N.fetch_add(1, Ordering::Relaxed))
    }

    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-verifier-registry-{}-{t}-{n}",
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
        requested_by: &str,
        reason: &str,
    ) {
        let dependencies = encode_effect_dependencies(&[]).unwrap();
        let mut store = EffectStore::open(authority).unwrap();
        store
            .propose(ProposeEffect {
                effect_id,
                session_id: SessionId(1),
                requested_by,
                action,
                resource,
                risk_class,
                execution_semantics: "at_most_once",
                idempotency_key: None,
                preconditions: b"[]",
                dependencies: &dependencies,
                payload_hash,
                proposed_event_id: EventId(next_id()),
                transition_id: EffectTransitionId(next_id()),
            })
            .unwrap();
        store
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(next_id()),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: Some(reason),
                evidence_ref: None,
                event_id: EventId(next_id()),
            })
            .unwrap();
    }

    fn append_allow(
        authority: &AuthorityLayout,
        action: &str,
        resource: &str,
        reason: &str,
    ) -> [u8; 16] {
        AuthorizationAuditLog::open(authority)
            .unwrap()
            .append(AppendAuthorizationDecision {
                principal: "owner:owner",
                action,
                resource,
                context: "scope=local-owner",
                evidence: AuthorizationDecisionEvidence::hard_guard_only("pass"),
                decision: AuthorizationDecisionKind::Allow,
                reason_code: reason,
            })
            .unwrap()
            .decision_id
    }

    fn issue_registration_approval(
        authority: &AuthorityLayout,
        prepared: &PreparedVerifierRule,
        effect_id: EffectId,
    ) -> [u8; 16] {
        let approval = prepare_approval(
            "owner:owner",
            ApprovalScope::once(effect_id, VERIFIER_RULE_REGISTER_ACTION, prepared.resource())
                .unwrap(),
            TAINT_AUTHORITY_MUTATION_RISK_CLASS,
            prepared.registration_taint_digest(),
            "2026-08-28T00:00:00Z",
            None,
            1,
        )
        .unwrap();
        let approval_effect_id = EffectId(next_id());
        create_authorized_effect(
            authority,
            approval_effect_id,
            APPROVAL_ISSUE_ACTION,
            approval.resource(),
            APPROVAL_MUTATION_RISK_CLASS,
            approval.intent_digest(),
            "owner:owner",
            "test_verifier_approval_issue",
        );
        let decision = append_allow(
            authority,
            APPROVAL_ISSUE_ACTION,
            approval.resource(),
            "test_verifier_approval_authority",
        );
        ApprovalStore::open(authority)
            .unwrap()
            .issue(approval, decision, approval_effect_id)
            .unwrap()
            .approval_id
    }

    #[test]
    fn tainted_or_generated_source_cannot_prepare_verifier_authority() {
        for label in [
            TaintLabel::LocalUnverified,
            TaintLabel::WebUntrusted,
            TaintLabel::ChannelUntrusted,
            TaintLabel::McpUntrusted,
            TaintLabel::PluginUnverified,
            TaintLabel::ModelGenerated,
            TaintLabel::SecretDerived,
        ] {
            assert!(matches!(
                prepare_verifier_rule(
                    VerifierRuleKind::DeterministicVerifier,
                    1,
                    b"authority-source:v1",
                    TaintSet::from_labels([TaintLabel::WebUntrusted]),
                    "owner:owner",
                    TaintSet::from_labels([TaintLabel::UserTrusted, label]),
                ),
                Err(VerifierRegistryError::UntrustedRegistrationSource)
            ));
        }
        assert!(matches!(
            prepare_verifier_rule(
                VerifierRuleKind::DeterministicVerifier,
                1,
                b"authority-source:v1",
                TaintSet::from_labels([TaintLabel::WebUntrusted]),
                "owner:owner",
                TaintSet::empty(),
            ),
            Err(VerifierRegistryError::UntrustedRegistrationSource)
        ));
    }

    #[test]
    fn trusted_registration_is_exact_effect_approval_and_integrity_bound() {
        let (runtime, authority) = authority();
        let registration_taint = TaintSet::from_labels([TaintLabel::UserTrusted]);
        let prepared = prepare_verifier_rule(
            VerifierRuleKind::DeterministicVerifier,
            1,
            b"signed-authoritative-source:v1",
            TaintSet::from_labels([TaintLabel::WebUntrusted, TaintLabel::ModelGenerated]),
            "owner:owner",
            registration_taint,
        )
        .unwrap();
        let rule_id = prepared.rule_id();
        let resource = prepared.resource().to_owned();
        let intent_digest = prepared.intent_digest();
        let effect_id = EffectId(next_id());
        let approval_id = issue_registration_approval(&authority, &prepared, effect_id);
        create_authorized_effect(
            &authority,
            effect_id,
            VERIFIER_RULE_REGISTER_ACTION,
            &resource,
            TAINT_AUTHORITY_MUTATION_RISK_CLASS,
            intent_digest,
            "owner:owner",
            "test_verifier_registration_effect",
        );
        let decision = append_allow(
            &authority,
            VERIFIER_RULE_REGISTER_ACTION,
            &resource,
            "test_verifier_registration_authority",
        );

        let record = VerifierRuleStore::open(&authority)
            .unwrap()
            .register(prepared, decision, approval_id, effect_id)
            .unwrap();
        assert_eq!(record.rule_id, rule_id);
        assert_eq!(record.registered_by, "owner:owner");
        assert!(record.allowed_downgrades.contains(TaintLabel::WebUntrusted));
        assert!(record.allowed_downgrades.contains(TaintLabel::ModelGenerated));
        AuthorityStore::open(authority.authority_db_path()).unwrap();

        fs::remove_dir_all(runtime.root).unwrap();
    }
}
