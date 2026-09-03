#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::memory::{
    MemoryCandidate, MemoryCandidateId, MemoryScope, MemoryValidationError, PromotionRequirement,
};
use golam_core::tool_request::{BindingDigest, PrincipalId};
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::approval_runtime::{ApprovalUseError, ApprovalUseRequest, ApprovalUseStore};
use crate::approvals::ApprovalClass;
use crate::memory_promotion_operational::PromotionOperationalEvidence;
use crate::storage::{AuthorityStore, StorageError};

pub const MEMORY_PROMOTION_ACTION: &str = "memory.promote";
pub const MEMORY_PROMOTION_RISK_CLASS: &str = "memory_promotion";

const KERNEL_AUTHORIZATION_REF_DOMAIN: &[u8] = b"golam:memory-kernel-authorization-ref:v1";
const HUMAN_PROMOTION_REF_DOMAIN: &[u8] = b"golam:memory-human-promotion-ref:v1";
const VERIFIER_POLICY_REF_DOMAIN: &[u8] = b"golam:memory-verifier-policy-ref:v1";
const VERIFIER_PROMOTION_REF_DOMAIN: &[u8] = b"golam:memory-verifier-promotion-ref:v1";
const PROMOTION_EVIDENCE_ID_DOMAIN: &[u8] = b"golam:memory-promotion-evidence-id:v1";
const AUTHORITY_EVIDENCE_REF_DOMAIN: &[u8] = b"golam:memory-promotion-authority-evidence:v1";
const MAX_AUTHORITY_SOURCE_BINDING_BYTES: usize = 4096;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedMemoryPromotion {
    pub evidence_id: BindingDigest,
    pub candidate_id: MemoryCandidateId,
    pub kernel_authorization_ref: BindingDigest,
    pub promotion_authority_ref: BindingDigest,
    pub authority_evidence_ref: BindingDigest,
    pub approving_principal: Option<PrincipalId>,
    pub verifier_policy_ref: Option<BindingDigest>,
}

impl ValidatedMemoryPromotion {
    pub fn operational_evidence(&self, recorded_at_unix_ms: u64) -> PromotionOperationalEvidence<'_> {
        PromotionOperationalEvidence {
            evidence_id: self.evidence_id,
            candidate_id: self.candidate_id,
            promotion_authority_ref: self.promotion_authority_ref,
            approving_principal: self.approving_principal.as_ref(),
            verifier_policy_ref: self.verifier_policy_ref,
            authority_evidence_ref: self.authority_evidence_ref,
            recorded_at_unix_ms,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HumanPromotionRequest<'a> {
    pub candidate: &'a MemoryCandidate,
    pub initiating_principal: &'a PrincipalId,
    pub authorization_decision_id: [u8; 16],
    pub approval_id: [u8; 16],
    pub effect_id: EffectId,
    pub observed_at: &'a str,
}

#[derive(Clone, Copy, Debug)]
pub struct DeterministicPromotionRequest<'a> {
    pub candidate: &'a MemoryCandidate,
    pub initiating_principal: &'a PrincipalId,
    pub authorization_decision_id: [u8; 16],
    pub rule_id: [u8; 16],
    pub rule_version: u64,
    pub authority_source_binding: &'a [u8],
    pub evidence_hash: [u8; 32],
}

#[derive(Debug)]
pub enum MemoryPromotionAuthorityError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Candidate(MemoryValidationError),
    Approval(ApprovalUseError),
    Integrity(String),
    AuthoritySecurity(String),
    WrongPromotionRequirement,
    MissingAuthorityDecision,
    AuthorityDecisionMismatch,
    StaleAuthorityDecision,
    ApprovalPrincipalMismatch,
    RunPreauthorizationNotEligible,
    VerifierRuleNotFound,
    VerifierRuleInactive,
    VerifierRuleKindMismatch,
    VerifierRuleVersionMismatch,
    VerifierSourceBindingMismatch,
    VerifierPolicyMismatch,
    InvalidVerifierEvidence,
    InvalidStoredRecord(&'static str),
}

impl fmt::Display for MemoryPromotionAuthorityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "memory promotion authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "memory promotion sqlite error: {error}"),
            Self::Core(error) => write!(f, "memory promotion canonical encoding error: {error}"),
            Self::Candidate(error) => write!(f, "memory promotion candidate is ineligible: {error}"),
            Self::Approval(error) => write!(f, "memory promotion approval validation failed: {error}"),
            Self::Integrity(error) => write!(f, "memory promotion integrity verification failed: {error}"),
            Self::AuthoritySecurity(error) => write!(f, "memory promotion authority-security verification failed: {error}"),
            Self::WrongPromotionRequirement => f.write_str("memory promotion authority mode does not match the candidate requirement"),
            Self::MissingAuthorityDecision => f.write_str("memory promotion has no durable Kernel authorization decision"),
            Self::AuthorityDecisionMismatch => f.write_str("memory promotion Kernel authorization does not match the exact principal/action/resource"),
            Self::StaleAuthorityDecision => f.write_str("memory promotion Kernel authorization decision is stale"),
            Self::ApprovalPrincipalMismatch => f.write_str("memory promotion approval is not attributable to the currently authorized initiating principal"),
            Self::RunPreauthorizationNotEligible => f.write_str("RUN_PREAUTHORIZATION cannot substitute for memory promotion authority"),
            Self::VerifierRuleNotFound => f.write_str("memory promotion verifier rule is not pre-registered"),
            Self::VerifierRuleInactive => f.write_str("memory promotion verifier rule is not active"),
            Self::VerifierRuleKindMismatch => f.write_str("memory promotion requires a deterministic verifier rule"),
            Self::VerifierRuleVersionMismatch => f.write_str("memory promotion verifier rule version does not match the governed requirement"),
            Self::VerifierSourceBindingMismatch => f.write_str("memory promotion verifier authority-source binding does not match the governed requirement"),
            Self::VerifierPolicyMismatch => f.write_str("memory promotion verifier policy was not independently selected by current Kernel authority"),
            Self::InvalidVerifierEvidence => f.write_str("memory promotion deterministic verifier evidence is missing or invalid"),
            Self::InvalidStoredRecord(reason) => write!(f, "stored memory promotion authority record is invalid: {reason}"),
        }
    }
}

impl Error for MemoryPromotionAuthorityError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::Candidate(error) => Some(error),
            Self::Approval(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for MemoryPromotionAuthorityError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for MemoryPromotionAuthorityError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for MemoryPromotionAuthorityError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<MemoryValidationError> for MemoryPromotionAuthorityError {
    fn from(value: MemoryValidationError) -> Self {
        Self::Candidate(value)
    }
}

impl From<ApprovalUseError> for MemoryPromotionAuthorityError {
    fn from(value: ApprovalUseError) -> Self {
        Self::Approval(value)
    }
}

pub struct MemoryPromotionAuthorityValidator {
    layout: AuthorityLayout,
    connection: Connection,
}

impl MemoryPromotionAuthorityValidator {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, MemoryPromotionAuthorityError> {
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

    pub fn validate_human(
        &mut self,
        request: HumanPromotionRequest<'_>,
    ) -> Result<ValidatedMemoryPromotion, MemoryPromotionAuthorityError> {
        request.candidate.validate_for_canonical_promotion()?;
        let approval_policy_ref = match request.candidate.promotion_requirement {
            PromotionRequirement::AttributableHumanApproval { approval_policy_ref } => {
                approval_policy_ref
            }
            PromotionRequirement::DeterministicPreregisteredVerifier { .. } => {
                return Err(MemoryPromotionAuthorityError::WrongPromotionRequirement);
            }
        };
        let resource = promotion_resource(request.candidate.scope, "human", approval_policy_ref);
        let kernel = self.validate_current_authorization(
            request.authorization_decision_id,
            request.initiating_principal,
            &resource,
        )?;
        let taint_digest = candidate_taint_digest(request.candidate)?;
        let approval = ApprovalUseStore::open(&self.layout)?.validate(ApprovalUseRequest {
            approval_id: request.approval_id,
            action: MEMORY_PROMOTION_ACTION,
            resource: &resource,
            effect_id: Some(request.effect_id),
            session_id: None,
            risk_class: MEMORY_PROMOTION_RISK_CLASS,
            taint_digest,
            observed_at: request.observed_at,
        })?;
        if approval.class() == ApprovalClass::RunPreauthorization {
            return Err(MemoryPromotionAuthorityError::RunPreauthorizationNotEligible);
        }
        let approver = self.approver_principal(request.approval_id)?;
        if approver != request.initiating_principal.as_str() {
            return Err(MemoryPromotionAuthorityError::ApprovalPrincipalMismatch);
        }
        let approving_principal = PrincipalId::new(approver)
            .map_err(|_| MemoryPromotionAuthorityError::InvalidStoredRecord("approval principal"))?;
        let promotion_authority_ref = human_promotion_ref(
            request.candidate.candidate_id,
            approval_policy_ref,
            request.approval_id,
            approval.class(),
            approval.scope_digest(),
            approval.parent_decision_id(),
            approval.current_uses(),
            approval.max_uses(),
        )?;
        let authority_evidence_ref = authority_evidence_ref(
            request.candidate.candidate_id,
            promotion_authority_ref,
            &taint_digest,
        )?;
        let evidence_id = promotion_evidence_id(
            request.candidate.candidate_id,
            kernel,
            promotion_authority_ref,
            authority_evidence_ref,
        )?;
        Ok(ValidatedMemoryPromotion {
            evidence_id,
            candidate_id: request.candidate.candidate_id,
            kernel_authorization_ref: kernel,
            promotion_authority_ref,
            authority_evidence_ref,
            approving_principal: Some(approving_principal),
            verifier_policy_ref: None,
        })
    }

    pub fn validate_deterministic(
        &mut self,
        request: DeterministicPromotionRequest<'_>,
    ) -> Result<ValidatedMemoryPromotion, MemoryPromotionAuthorityError> {
        request.candidate.validate_for_canonical_promotion()?;
        let governed_policy_ref = match request.candidate.promotion_requirement {
            PromotionRequirement::DeterministicPreregisteredVerifier { verifier_policy_ref } => {
                verifier_policy_ref
            }
            PromotionRequirement::AttributableHumanApproval { .. } => {
                return Err(MemoryPromotionAuthorityError::WrongPromotionRequirement);
            }
        };
        if request.rule_version == 0
            || request.authority_source_binding.is_empty()
            || request.authority_source_binding.len() > MAX_AUTHORITY_SOURCE_BINDING_BYTES
            || request.evidence_hash == [0; 32]
        {
            return Err(MemoryPromotionAuthorityError::InvalidVerifierEvidence);
        }
        let derived_policy_ref = verifier_policy_ref(
            request.rule_id,
            request.rule_version,
            request.authority_source_binding,
        )?;
        if derived_policy_ref != governed_policy_ref {
            return Err(MemoryPromotionAuthorityError::VerifierPolicyMismatch);
        }
        let resource = promotion_resource(request.candidate.scope, "verifier", governed_policy_ref);
        let kernel = self.validate_current_authorization(
            request.authorization_decision_id,
            request.initiating_principal,
            &resource,
        )?;
        let rule = self.active_verifier_rule(request.rule_id)?;
        if rule.kind != "deterministic_verifier" {
            return Err(MemoryPromotionAuthorityError::VerifierRuleKindMismatch);
        }
        if rule.version != request.rule_version {
            return Err(MemoryPromotionAuthorityError::VerifierRuleVersionMismatch);
        }
        if rule.authority_source_binding.as_slice() != request.authority_source_binding {
            return Err(MemoryPromotionAuthorityError::VerifierSourceBindingMismatch);
        }
        let promotion_authority_ref = verifier_promotion_ref(
            request.candidate.candidate_id,
            governed_policy_ref,
            request.rule_id,
            request.rule_version,
            request.authority_source_binding,
            &rule.registered_by,
            rule.created_global_seq,
        )?;
        let authority_evidence_ref = authority_evidence_ref(
            request.candidate.candidate_id,
            promotion_authority_ref,
            &request.evidence_hash,
        )?;
        let evidence_id = promotion_evidence_id(
            request.candidate.candidate_id,
            kernel,
            promotion_authority_ref,
            authority_evidence_ref,
        )?;
        Ok(ValidatedMemoryPromotion {
            evidence_id,
            candidate_id: request.candidate.candidate_id,
            kernel_authorization_ref: kernel,
            promotion_authority_ref,
            authority_evidence_ref,
            approving_principal: None,
            verifier_policy_ref: Some(governed_policy_ref),
        })
    }

    fn validate_current_authorization(
        &mut self,
        decision_id: [u8; 16],
        expected_principal: &PrincipalId,
        expected_resource: &str,
    ) -> Result<BindingDigest, MemoryPromotionAuthorityError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Deferred)?;
        verify_transaction_integrity(&transaction)?;
        let row = transaction
            .query_row(
                "SELECT principal, action, resource, context_hash, hard_guard_result, decision, global_seq \
                 FROM authorization_decisions WHERE decision_id = ?1",
                params![&decision_id[..]],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Vec<u8>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, i64>(6)?,
                    ))
                },
            )
            .optional()?
            .ok_or(MemoryPromotionAuthorityError::MissingAuthorityDecision)?;
        if row.0 != expected_principal.as_str()
            || row.1 != MEMORY_PROMOTION_ACTION
            || row.2 != expected_resource
            || row.4 != "pass"
            || row.5 != "allow"
        {
            return Err(MemoryPromotionAuthorityError::AuthorityDecisionMismatch);
        }
        let context_hash = hash32(row.3, "authorization context hash")?;
        let global_seq = from_i64(row.6, "authorization sequence")?;
        let latest: i64 = transaction.query_row(
            "SELECT COALESCE(MAX(global_seq), 0) FROM (\
               SELECT global_seq FROM session_events \
               UNION ALL SELECT global_seq FROM effect_transitions \
               UNION ALL SELECT global_seq FROM authorization_decisions\
             )",
            [],
            |row| row.get(0),
        )?;
        if global_seq != from_i64(latest, "latest authority sequence")? {
            return Err(MemoryPromotionAuthorityError::StaleAuthorityDecision);
        }
        let reference = kernel_authorization_ref(
            decision_id,
            expected_principal,
            expected_resource,
            context_hash,
            global_seq,
        )?;
        transaction.commit()?;
        Ok(reference)
    }

    fn approver_principal(
        &self,
        approval_id: [u8; 16],
    ) -> Result<String, MemoryPromotionAuthorityError> {
        self.connection
            .query_row(
                "SELECT approver_principal FROM approvals WHERE approval_id = ?1",
                params![&approval_id[..]],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or(MemoryPromotionAuthorityError::InvalidStoredRecord(
                "validated approval disappeared",
            ))
    }

    fn active_verifier_rule(
        &mut self,
        rule_id: [u8; 16],
    ) -> Result<StoredVerifierRule, MemoryPromotionAuthorityError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Deferred)?;
        verify_transaction_integrity(&transaction)?;
        let row = transaction
            .query_row(
                "SELECT kind, version, authority_source_binding, registered_by, status, created_global_seq \
                 FROM verifier_rules WHERE rule_id = ?1",
                params![&rule_id[..]],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                    ))
                },
            )
            .optional()?
            .ok_or(MemoryPromotionAuthorityError::VerifierRuleNotFound)?;
        if row.4 != "active" {
            return Err(MemoryPromotionAuthorityError::VerifierRuleInactive);
        }
        let record = StoredVerifierRule {
            kind: row.0,
            version: from_i64(row.1, "verifier rule version")?,
            authority_source_binding: row.2,
            registered_by: row.3,
            created_global_seq: from_i64(row.5, "verifier rule global sequence")?,
        };
        transaction.commit()?;
        Ok(record)
    }
}

struct StoredVerifierRule {
    kind: String,
    version: u64,
    authority_source_binding: Vec<u8>,
    registered_by: String,
    created_global_seq: u64,
}

fn verify_transaction_integrity(
    transaction: &Transaction<'_>,
) -> Result<(), MemoryPromotionAuthorityError> {
    crate::integrity::verify(transaction)
        .map_err(|error| MemoryPromotionAuthorityError::Integrity(error.to_string()))?;
    crate::authority_security_v2::verify(transaction)
        .map_err(|error| MemoryPromotionAuthorityError::AuthoritySecurity(error.to_string()))
}

pub fn verifier_policy_ref(
    rule_id: [u8; 16],
    version: u64,
    authority_source_binding: &[u8],
) -> Result<BindingDigest, MemoryPromotionAuthorityError> {
    if version == 0
        || authority_source_binding.is_empty()
        || authority_source_binding.len() > MAX_AUTHORITY_SOURCE_BINDING_BYTES
    {
        return Err(MemoryPromotionAuthorityError::InvalidVerifierEvidence);
    }
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(VERIFIER_POLICY_REF_DOMAIN)?;
    encoder.push_bytes(&rule_id)?;
    encoder.push_u64(version);
    encoder.push_bytes(authority_source_binding)?;
    Ok(BindingDigest::new(*blake3::hash(&encoder.finish()).as_bytes()))
}

pub fn promotion_resource(scope: MemoryScope, mode: &str, policy_ref: BindingDigest) -> String {
    let scope = match scope {
        MemoryScope::User => "user",
        MemoryScope::Project => "project",
    };
    format!(
        "memory-promotion:{scope}:{mode}:{}",
        hex_bytes(&policy_ref.bytes())
    )
}

fn candidate_taint_digest(
    candidate: &MemoryCandidate,
) -> Result<[u8; 32], MemoryPromotionAuthorityError> {
    Ok(*blake3::hash(&candidate.taint_set.canonical_bytes()?).as_bytes())
}

fn kernel_authorization_ref(
    decision_id: [u8; 16],
    principal: &PrincipalId,
    resource: &str,
    context_hash: [u8; 32],
    global_seq: u64,
) -> Result<BindingDigest, MemoryPromotionAuthorityError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(KERNEL_AUTHORIZATION_REF_DOMAIN)?;
    encoder.push_bytes(&decision_id)?;
    encoder.push_bytes(principal.as_str().as_bytes())?;
    encoder.push_bytes(MEMORY_PROMOTION_ACTION.as_bytes())?;
    encoder.push_bytes(resource.as_bytes())?;
    encoder.push_bytes(&context_hash)?;
    encoder.push_u64(global_seq);
    Ok(BindingDigest::new(*blake3::hash(&encoder.finish()).as_bytes()))
}

#[allow(clippy::too_many_arguments)]
fn human_promotion_ref(
    candidate_id: MemoryCandidateId,
    policy_ref: BindingDigest,
    approval_id: [u8; 16],
    class: ApprovalClass,
    scope_digest: [u8; 32],
    parent_decision_id: [u8; 16],
    current_uses: u64,
    max_uses: u64,
) -> Result<BindingDigest, MemoryPromotionAuthorityError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(HUMAN_PROMOTION_REF_DOMAIN)?;
    encoder.push_bytes(&candidate_id.0.bytes())?;
    encoder.push_bytes(&policy_ref.bytes())?;
    encoder.push_bytes(&approval_id)?;
    encoder.push_bytes(class.as_str().as_bytes())?;
    encoder.push_bytes(&scope_digest)?;
    encoder.push_bytes(&parent_decision_id)?;
    encoder.push_u64(current_uses);
    encoder.push_u64(max_uses);
    Ok(BindingDigest::new(*blake3::hash(&encoder.finish()).as_bytes()))
}

fn verifier_promotion_ref(
    candidate_id: MemoryCandidateId,
    policy_ref: BindingDigest,
    rule_id: [u8; 16],
    version: u64,
    authority_source_binding: &[u8],
    registered_by: &str,
    created_global_seq: u64,
) -> Result<BindingDigest, MemoryPromotionAuthorityError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(VERIFIER_PROMOTION_REF_DOMAIN)?;
    encoder.push_bytes(&candidate_id.0.bytes())?;
    encoder.push_bytes(&policy_ref.bytes())?;
    encoder.push_bytes(&rule_id)?;
    encoder.push_u64(version);
    encoder.push_bytes(authority_source_binding)?;
    encoder.push_bytes(registered_by.as_bytes())?;
    encoder.push_u64(created_global_seq);
    Ok(BindingDigest::new(*blake3::hash(&encoder.finish()).as_bytes()))
}

fn authority_evidence_ref(
    candidate_id: MemoryCandidateId,
    promotion_authority_ref: BindingDigest,
    evidence: &[u8; 32],
) -> Result<BindingDigest, MemoryPromotionAuthorityError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(AUTHORITY_EVIDENCE_REF_DOMAIN)?;
    encoder.push_bytes(&candidate_id.0.bytes())?;
    encoder.push_bytes(&promotion_authority_ref.bytes())?;
    encoder.push_bytes(evidence)?;
    Ok(BindingDigest::new(*blake3::hash(&encoder.finish()).as_bytes()))
}

fn promotion_evidence_id(
    candidate_id: MemoryCandidateId,
    kernel_authorization_ref: BindingDigest,
    promotion_authority_ref: BindingDigest,
    authority_evidence_ref: BindingDigest,
) -> Result<BindingDigest, MemoryPromotionAuthorityError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(PROMOTION_EVIDENCE_ID_DOMAIN)?;
    encoder.push_bytes(&candidate_id.0.bytes())?;
    encoder.push_bytes(&kernel_authorization_ref.bytes())?;
    encoder.push_bytes(&promotion_authority_ref.bytes())?;
    encoder.push_bytes(&authority_evidence_ref.bytes())?;
    Ok(BindingDigest::new(*blake3::hash(&encoder.finish()).as_bytes()))
}

fn hash32(
    value: Vec<u8>,
    reason: &'static str,
) -> Result<[u8; 32], MemoryPromotionAuthorityError> {
    value
        .try_into()
        .map_err(|_| MemoryPromotionAuthorityError::InvalidStoredRecord(reason))
}

fn from_i64(value: i64, reason: &'static str) -> Result<u64, MemoryPromotionAuthorityError> {
    u64::try_from(value).map_err(|_| MemoryPromotionAuthorityError::InvalidStoredRecord(reason))
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
    use crate::verifier_registry::{
        TAINT_AUTHORITY_MUTATION_RISK_CLASS, VERIFIER_RULE_REGISTER_ACTION, VerifierRuleKind,
        VerifierRuleStore, prepare_verifier_rule,
    };
    use golam_core::memory::{MemoryAuthorityClass, PromotionRequirement};
    use golam_core::paths::RuntimeLayout;
    use golam_core::taint::{TaintLabel, TaintSet};
    use golam_core::{EffectTransitionId, EventId, SessionId};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);
    static ID: AtomicU64 = AtomicU64::new(0);

    fn next_id() -> u128 {
        9_000_000 + u128::from(ID.fetch_add(1, Ordering::Relaxed))
    }

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-memory-promotion-authority-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    fn candidate(requirement: PromotionRequirement) -> MemoryCandidate {
        MemoryCandidate {
            candidate_id: MemoryCandidateId(digest(1)),
            scope: MemoryScope::Project,
            proposed_content_ref: digest(2),
            provenance_refs: vec![digest(3)],
            taint_set: TaintSet::from_labels([TaintLabel::UserTrusted]),
            authority_class: MemoryAuthorityClass::UserAttributed,
            created_by_principal: PrincipalId::new("owner:owner").unwrap(),
            created_at_unix_ms: 4,
            promotion_requirement: requirement,
        }
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
                reason_code: Some("memory_promotion_test"),
                evidence_ref: None,
                event_id: EventId(next_id()),
            })
            .unwrap();
    }

    fn append_allow(authority: &AuthorityLayout, action: &str, resource: &str) -> [u8; 16] {
        AuthorizationAuditLog::open(authority)
            .unwrap()
            .append(AppendAuthorizationDecision {
                principal: "owner:owner",
                action,
                resource,
                context: "scope=memory-promotion",
                evidence: AuthorizationDecisionEvidence::hard_guard_only("pass"),
                decision: AuthorizationDecisionKind::Allow,
                reason_code: "memory_promotion_test",
            })
            .unwrap()
            .decision_id
    }

    fn issue_once_approval(
        authority: &AuthorityLayout,
        effect_id: EffectId,
        resource: &str,
        taint_digest: [u8; 32],
    ) -> [u8; 16] {
        let approval = prepare_approval(
            "owner:owner",
            ApprovalScope::once(effect_id, MEMORY_PROMOTION_ACTION, resource).unwrap(),
            MEMORY_PROMOTION_RISK_CLASS,
            taint_digest,
            "2026-09-03T00:00:00Z",
            None,
            1,
        )
        .unwrap();
        let issue_effect = EffectId(next_id());
        create_authorized_effect(
            authority,
            issue_effect,
            APPROVAL_ISSUE_ACTION,
            approval.resource(),
            APPROVAL_MUTATION_RISK_CLASS,
            approval.intent_digest(),
            "owner:owner",
        );
        let decision = append_allow(authority, APPROVAL_ISSUE_ACTION, approval.resource());
        ApprovalStore::open(authority)
            .unwrap()
            .issue(approval, decision, issue_effect)
            .unwrap()
            .approval_id()
    }

    fn register_verifier(
        authority: &AuthorityLayout,
        source_binding: &[u8],
    ) -> ([u8; 16], u64) {
        let prepared = prepare_verifier_rule(
            VerifierRuleKind::DeterministicVerifier,
            1,
            source_binding,
            TaintSet::from_labels([TaintLabel::WebUntrusted]),
            "owner:owner",
            TaintSet::from_labels([TaintLabel::UserTrusted]),
        )
        .unwrap();
        let rule_id = prepared.rule_id();
        let registration_effect = EffectId(next_id());
        let approval = prepare_approval(
            "owner:owner",
            ApprovalScope::once(
                registration_effect,
                VERIFIER_RULE_REGISTER_ACTION,
                prepared.resource(),
            )
            .unwrap(),
            TAINT_AUTHORITY_MUTATION_RISK_CLASS,
            *blake3::hash(&TaintSet::from_labels([TaintLabel::UserTrusted]).canonical_bytes().unwrap()).as_bytes(),
            "2026-09-03T00:00:00Z",
            None,
            1,
        )
        .unwrap();
        let approval_effect = EffectId(next_id());
        create_authorized_effect(
            authority,
            approval_effect,
            APPROVAL_ISSUE_ACTION,
            approval.resource(),
            APPROVAL_MUTATION_RISK_CLASS,
            approval.intent_digest(),
            "owner:owner",
        );
        let approval_decision = append_allow(authority, APPROVAL_ISSUE_ACTION, approval.resource());
        let approval_id = ApprovalStore::open(authority)
            .unwrap()
            .issue(approval, approval_decision, approval_effect)
            .unwrap()
            .approval_id();
        create_authorized_effect(
            authority,
            registration_effect,
            VERIFIER_RULE_REGISTER_ACTION,
            prepared.resource(),
            TAINT_AUTHORITY_MUTATION_RISK_CLASS,
            prepared.intent_digest(),
            "owner:owner",
        );
        let decision = append_allow(authority, VERIFIER_RULE_REGISTER_ACTION, prepared.resource());
        VerifierRuleStore::open(authority)
            .unwrap()
            .register(prepared, decision, approval_id, registration_effect)
            .unwrap();
        (rule_id, 1)
    }

    #[test]
    fn human_promotion_requires_current_authorization_and_attributable_approval() {
        let (runtime, authority) = authority();
        let policy_ref = digest(10);
        let candidate = candidate(PromotionRequirement::AttributableHumanApproval {
            approval_policy_ref: policy_ref,
        });
        let principal = PrincipalId::new("owner:owner").unwrap();
        let resource = promotion_resource(candidate.scope, "human", policy_ref);
        let effect_id = EffectId(next_id());
        let taint_digest = candidate_taint_digest(&candidate).unwrap();
        let approval_id = issue_once_approval(&authority, effect_id, &resource, taint_digest);
        let decision = append_allow(&authority, MEMORY_PROMOTION_ACTION, &resource);
        let validated = MemoryPromotionAuthorityValidator::open(&authority)
            .unwrap()
            .validate_human(HumanPromotionRequest {
                candidate: &candidate,
                initiating_principal: &principal,
                authorization_decision_id: decision,
                approval_id,
                effect_id,
                observed_at: "2026-09-03T01:00:00Z",
            })
            .unwrap();
        assert_eq!(validated.candidate_id, candidate.candidate_id);
        assert_eq!(validated.approving_principal, Some(principal));
        assert_eq!(validated.verifier_policy_ref, None);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn stale_kernel_authorization_is_rejected() {
        let (runtime, authority) = authority();
        let policy_ref = digest(20);
        let candidate = candidate(PromotionRequirement::AttributableHumanApproval {
            approval_policy_ref: policy_ref,
        });
        let principal = PrincipalId::new("owner:owner").unwrap();
        let resource = promotion_resource(candidate.scope, "human", policy_ref);
        let effect_id = EffectId(next_id());
        let approval_id = issue_once_approval(
            &authority,
            effect_id,
            &resource,
            candidate_taint_digest(&candidate).unwrap(),
        );
        let decision = append_allow(&authority, MEMORY_PROMOTION_ACTION, &resource);
        let _newer = append_allow(&authority, "unrelated.action", "unrelated:resource");
        assert!(matches!(
            MemoryPromotionAuthorityValidator::open(&authority)
                .unwrap()
                .validate_human(HumanPromotionRequest {
                    candidate: &candidate,
                    initiating_principal: &principal,
                    authorization_decision_id: decision,
                    approval_id,
                    effect_id,
                    observed_at: "2026-09-03T01:00:00Z",
                }),
            Err(MemoryPromotionAuthorityError::StaleAuthorityDecision)
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn deterministic_promotion_requires_kernel_selected_active_registered_rule() {
        let (runtime, authority) = authority();
        let source_binding = b"authoritative-memory-source:v1";
        let (rule_id, version) = register_verifier(&authority, source_binding);
        let policy_ref = verifier_policy_ref(rule_id, version, source_binding).unwrap();
        let candidate = candidate(PromotionRequirement::DeterministicPreregisteredVerifier {
            verifier_policy_ref: policy_ref,
        });
        let principal = PrincipalId::new("owner:owner").unwrap();
        let resource = promotion_resource(candidate.scope, "verifier", policy_ref);
        let decision = append_allow(&authority, MEMORY_PROMOTION_ACTION, &resource);
        let validated = MemoryPromotionAuthorityValidator::open(&authority)
            .unwrap()
            .validate_deterministic(DeterministicPromotionRequest {
                candidate: &candidate,
                initiating_principal: &principal,
                authorization_decision_id: decision,
                rule_id,
                rule_version: version,
                authority_source_binding: source_binding,
                evidence_hash: [0x55; 32],
            })
            .unwrap();
        assert_eq!(validated.verifier_policy_ref, Some(policy_ref));
        assert_eq!(validated.approving_principal, None);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn candidate_selected_or_missing_verifier_evidence_fails_closed() {
        let (runtime, authority) = authority();
        let source_binding = b"authoritative-memory-source:v1";
        let (rule_id, version) = register_verifier(&authority, source_binding);
        let actual = verifier_policy_ref(rule_id, version, source_binding).unwrap();
        let candidate = candidate(PromotionRequirement::DeterministicPreregisteredVerifier {
            verifier_policy_ref: digest(99),
        });
        let principal = PrincipalId::new("owner:owner").unwrap();
        let resource = promotion_resource(candidate.scope, "verifier", digest(99));
        let decision = append_allow(&authority, MEMORY_PROMOTION_ACTION, &resource);
        assert_ne!(actual, digest(99));
        assert!(matches!(
            MemoryPromotionAuthorityValidator::open(&authority)
                .unwrap()
                .validate_deterministic(DeterministicPromotionRequest {
                    candidate: &candidate,
                    initiating_principal: &principal,
                    authorization_decision_id: decision,
                    rule_id,
                    rule_version: version,
                    authority_source_binding: source_binding,
                    evidence_hash: [0x66; 32],
                }),
            Err(MemoryPromotionAuthorityError::VerifierPolicyMismatch)
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn secret_derived_candidate_is_rejected_before_authority_lookup() {
        let (runtime, authority) = authority();
        let policy_ref = digest(70);
        let mut candidate = candidate(PromotionRequirement::AttributableHumanApproval {
            approval_policy_ref: policy_ref,
        });
        candidate.taint_set = TaintSet::from_labels([TaintLabel::SecretDerived]);
        let principal = PrincipalId::new("owner:owner").unwrap();
        assert!(matches!(
            MemoryPromotionAuthorityValidator::open(&authority)
                .unwrap()
                .validate_human(HumanPromotionRequest {
                    candidate: &candidate,
                    initiating_principal: &principal,
                    authorization_decision_id: [0; 16],
                    approval_id: [0; 16],
                    effect_id: EffectId(1),
                    observed_at: "2026-09-03T01:00:00Z",
                }),
            Err(MemoryPromotionAuthorityError::Candidate(_))
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
