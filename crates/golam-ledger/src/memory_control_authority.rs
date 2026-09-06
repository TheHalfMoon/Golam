#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::memory::{MemoryItemId, MemoryOperation, MemoryScope, MemoryVersionId};
use golam_core::taint::TaintSet;
use golam_core::tool_request::{BindingDigest, PrincipalId};
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::approval_runtime::{ApprovalUseError, ApprovalUseRequest, ApprovalUseStore};
use crate::approvals::ApprovalClass;
use crate::memory_promotion_authority::verifier_policy_ref;
use crate::memory_writer_authority::{MEMORY_MUTATION_ACTION, MEMORY_MUTATION_RISK_CLASS};
use crate::storage::{AuthorityStore, StorageError};

const KERNEL_AUTHORIZATION_REF_DOMAIN: &[u8] = b"golam:memory-control-kernel-authorization-ref:v1";
const HUMAN_CONTROL_REF_DOMAIN: &[u8] = b"golam:memory-human-control-ref:v1";
const VERIFIER_CONTROL_REF_DOMAIN: &[u8] = b"golam:memory-verifier-control-ref:v1";
const CONTROL_EVIDENCE_REF_DOMAIN: &[u8] = b"golam:memory-control-evidence-ref:v1";
const QUALIFIED_CONTROL_RECORD_DOMAIN: &[u8] = b"golam:qualified-memory-control:v1";
const MAX_AUTHORITY_SOURCE_BINDING_BYTES: usize = 4096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryControlTarget {
    pub operation: MemoryOperation,
    pub item_id: MemoryItemId,
    pub scope: MemoryScope,
    pub expected_version: MemoryVersionId,
    pub taint_set: TaintSet,
}

impl MemoryControlTarget {
    pub fn validate(self) -> Result<(), MemoryControlAuthorityError> {
        if !matches!(
            self.operation,
            MemoryOperation::Expire | MemoryOperation::Forget | MemoryOperation::Redact
        ) {
            return Err(MemoryControlAuthorityError::CandidateOperation);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HumanMemoryControlRequest<'a> {
    pub target: MemoryControlTarget,
    pub initiating_principal: &'a PrincipalId,
    pub authorization_decision_id: [u8; 16],
    pub approval_id: [u8; 16],
    pub effect_id: EffectId,
    pub observed_at: &'a str,
}

#[derive(Clone, Copy, Debug)]
pub struct DeterministicMemoryControlRequest<'a> {
    pub target: MemoryControlTarget,
    pub initiating_principal: &'a PrincipalId,
    pub authorization_decision_id: [u8; 16],
    pub rule_id: [u8; 16],
    pub rule_version: u64,
    pub authority_source_binding: &'a [u8],
    pub evidence_hash: [u8; 32],
    pub effect_id: EffectId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedMemoryControlAuthority {
    target: MemoryControlTarget,
    effect_id: EffectId,
    evidence_id: BindingDigest,
    kernel_authorization_ref: BindingDigest,
    mutation_authority_ref: BindingDigest,
    authority_evidence_ref: BindingDigest,
    approving_principal: Option<PrincipalId>,
    verifier_policy_ref: Option<BindingDigest>,
}

impl ValidatedMemoryControlAuthority {
    pub const fn target(&self) -> MemoryControlTarget {
        self.target
    }

    pub const fn effect_id(&self) -> EffectId {
        self.effect_id
    }

    pub const fn evidence_id(&self) -> BindingDigest {
        self.evidence_id
    }

    pub const fn kernel_authorization_ref(&self) -> BindingDigest {
        self.kernel_authorization_ref
    }

    pub const fn mutation_authority_ref(&self) -> BindingDigest {
        self.mutation_authority_ref
    }

    pub const fn authority_evidence_ref(&self) -> BindingDigest {
        self.authority_evidence_ref
    }

    pub fn approving_principal(&self) -> Option<&PrincipalId> {
        self.approving_principal.as_ref()
    }

    pub const fn verifier_policy_ref(&self) -> Option<BindingDigest> {
        self.verifier_policy_ref
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum MemoryControlRevalidation {
    Human {
        target: MemoryControlTarget,
        initiating_principal: PrincipalId,
        authorization_decision_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
    },
    Deterministic {
        target: MemoryControlTarget,
        initiating_principal: PrincipalId,
        authorization_decision_id: [u8; 16],
        rule_id: [u8; 16],
        rule_version: u64,
        authority_source_binding: Vec<u8>,
        evidence_hash: [u8; 32],
        effect_id: EffectId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualifiedMemoryControlAuthority {
    validated: ValidatedMemoryControlAuthority,
    revalidation: MemoryControlRevalidation,
    record_bytes: Vec<u8>,
}

impl QualifiedMemoryControlAuthority {
    pub const fn target(&self) -> MemoryControlTarget {
        self.validated.target()
    }

    pub const fn effect_id(&self) -> EffectId {
        self.validated.effect_id()
    }

    pub const fn evidence_id(&self) -> BindingDigest {
        self.validated.evidence_id()
    }

    pub const fn kernel_authorization_ref(&self) -> BindingDigest {
        self.validated.kernel_authorization_ref()
    }

    pub const fn mutation_authority_ref(&self) -> BindingDigest {
        self.validated.mutation_authority_ref()
    }

    pub const fn authority_evidence_ref(&self) -> BindingDigest {
        self.validated.authority_evidence_ref()
    }

    pub fn approving_principal(&self) -> Option<&PrincipalId> {
        self.validated.approving_principal()
    }

    pub const fn verifier_policy_ref(&self) -> Option<BindingDigest> {
        self.validated.verifier_policy_ref()
    }

    pub fn record_bytes(&self) -> &[u8] {
        &self.record_bytes
    }
}

#[derive(Debug)]
pub enum MemoryControlAuthorityError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Approval(ApprovalUseError),
    Integrity(String),
    AuthoritySecurity(String),
    CandidateOperation,
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
    InvalidVerifierEvidence,
    InvalidStoredRecord(&'static str),
    RevalidationMismatch,
}

impl fmt::Display for MemoryControlAuthorityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "memory control authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "memory control sqlite error: {error}"),
            Self::Core(error) => write!(f, "memory control canonical encoding error: {error}"),
            Self::Approval(error) => write!(f, "memory control approval validation failed: {error}"),
            Self::Integrity(error) => write!(f, "memory control integrity verification failed: {error}"),
            Self::AuthoritySecurity(error) => {
                write!(f, "memory control authority-security verification failed: {error}")
            }
            Self::CandidateOperation => f.write_str(
                "candidate-backed memory operation must use the promotion authority path",
            ),
            Self::MissingAuthorityDecision => {
                f.write_str("memory control has no durable Kernel authorization decision")
            }
            Self::AuthorityDecisionMismatch => f.write_str(
                "memory control Kernel authorization does not match the exact principal/action/resource",
            ),
            Self::StaleAuthorityDecision => {
                f.write_str("memory control Kernel authorization decision is stale")
            }
            Self::ApprovalPrincipalMismatch => f.write_str(
                "memory control approval is not attributable to the initiating principal",
            ),
            Self::RunPreauthorizationNotEligible => f.write_str(
                "RUN_PREAUTHORIZATION cannot substitute for memory control authority",
            ),
            Self::VerifierRuleNotFound => {
                f.write_str("memory control verifier rule is not pre-registered")
            }
            Self::VerifierRuleInactive => {
                f.write_str("memory control verifier rule is not active")
            }
            Self::VerifierRuleKindMismatch => {
                f.write_str("memory control requires a deterministic verifier rule")
            }
            Self::VerifierRuleVersionMismatch => f.write_str(
                "memory control verifier rule version does not match the requested authority",
            ),
            Self::VerifierSourceBindingMismatch => f.write_str(
                "memory control verifier source binding does not match the registered authority",
            ),
            Self::InvalidVerifierEvidence => {
                f.write_str("memory control deterministic verifier evidence is missing or invalid")
            }
            Self::InvalidStoredRecord(reason) => {
                write!(f, "stored memory control authority record is invalid: {reason}")
            }
            Self::RevalidationMismatch => f.write_str(
                "memory control authority changed between qualification and PREPARED",
            ),
        }
    }
}

impl Error for MemoryControlAuthorityError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::Approval(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for MemoryControlAuthorityError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for MemoryControlAuthorityError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for MemoryControlAuthorityError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<ApprovalUseError> for MemoryControlAuthorityError {
    fn from(value: ApprovalUseError) -> Self {
        Self::Approval(value)
    }
}

pub struct MemoryControlAuthorityGate {
    connection: Connection,
}

impl MemoryControlAuthorityGate {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, MemoryControlAuthorityError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn validate_human(
        &mut self,
        layout: &AuthorityLayout,
        request: HumanMemoryControlRequest<'_>,
    ) -> Result<QualifiedMemoryControlAuthority, MemoryControlAuthorityError> {
        request.target.validate()?;
        let resource = memory_control_resource(request.target);
        let kernel_authorization_ref = self.validate_current_authorization(
            request.authorization_decision_id,
            request.initiating_principal,
            &resource,
        )?;
        let taint_digest = taint_digest(request.target.taint_set)?;
        let approval = ApprovalUseStore::open(layout)?.validate(ApprovalUseRequest {
            approval_id: request.approval_id,
            action: MEMORY_MUTATION_ACTION,
            resource: &resource,
            effect_id: Some(request.effect_id),
            session_id: None,
            risk_class: MEMORY_MUTATION_RISK_CLASS,
            taint_digest,
            observed_at: request.observed_at,
        })?;
        if approval.class() == ApprovalClass::RunPreauthorization {
            return Err(MemoryControlAuthorityError::RunPreauthorizationNotEligible);
        }
        let approver = self.approver_principal(request.approval_id)?;
        if approver != request.initiating_principal.as_str() {
            return Err(MemoryControlAuthorityError::ApprovalPrincipalMismatch);
        }
        let approving_principal = PrincipalId::new(approver)
            .map_err(|_| MemoryControlAuthorityError::InvalidStoredRecord("approval principal"))?;
        let mutation_authority_ref = human_control_ref(
            request.target,
            request.approval_id,
            approval.class(),
            approval.scope_digest(),
            approval.parent_decision_id(),
            approval.current_uses(),
            approval.max_uses(),
        )?;
        let authority_evidence_ref = authority_evidence_ref(
            request.target,
            request.effect_id,
            mutation_authority_ref,
            &taint_digest,
        )?;
        let evidence_id = control_evidence_id(
            request.target,
            request.effect_id,
            kernel_authorization_ref,
            mutation_authority_ref,
            authority_evidence_ref,
        )?;
        let validated = ValidatedMemoryControlAuthority {
            target: request.target,
            effect_id: request.effect_id,
            evidence_id,
            kernel_authorization_ref,
            mutation_authority_ref,
            authority_evidence_ref,
            approving_principal: Some(approving_principal),
            verifier_policy_ref: None,
        };
        qualify(
            validated,
            MemoryControlRevalidation::Human {
                target: request.target,
                initiating_principal: request.initiating_principal.clone(),
                authorization_decision_id: request.authorization_decision_id,
                approval_id: request.approval_id,
                effect_id: request.effect_id,
            },
        )
    }

    pub fn validate_deterministic(
        &mut self,
        request: DeterministicMemoryControlRequest<'_>,
    ) -> Result<QualifiedMemoryControlAuthority, MemoryControlAuthorityError> {
        request.target.validate()?;
        if request.rule_version == 0
            || request.authority_source_binding.is_empty()
            || request.authority_source_binding.len() > MAX_AUTHORITY_SOURCE_BINDING_BYTES
            || request.evidence_hash == [0; 32]
        {
            return Err(MemoryControlAuthorityError::InvalidVerifierEvidence);
        }
        let resource = memory_control_resource(request.target);
        let kernel_authorization_ref = self.validate_current_authorization(
            request.authorization_decision_id,
            request.initiating_principal,
            &resource,
        )?;
        let rule = self.active_verifier_rule(request.rule_id)?;
        if rule.kind != "deterministic_verifier" {
            return Err(MemoryControlAuthorityError::VerifierRuleKindMismatch);
        }
        if rule.version != request.rule_version {
            return Err(MemoryControlAuthorityError::VerifierRuleVersionMismatch);
        }
        if rule.authority_source_binding.as_slice() != request.authority_source_binding {
            return Err(MemoryControlAuthorityError::VerifierSourceBindingMismatch);
        }
        let policy_ref = verifier_policy_ref(
            request.rule_id,
            request.rule_version,
            request.authority_source_binding,
        )
        .map_err(|_| MemoryControlAuthorityError::InvalidVerifierEvidence)?;
        let mutation_authority_ref = verifier_control_ref(
            request.target,
            request.rule_id,
            request.rule_version,
            request.authority_source_binding,
            &rule.registered_by,
            rule.created_global_seq,
        )?;
        let authority_evidence_ref = authority_evidence_ref(
            request.target,
            request.effect_id,
            mutation_authority_ref,
            &request.evidence_hash,
        )?;
        let evidence_id = control_evidence_id(
            request.target,
            request.effect_id,
            kernel_authorization_ref,
            mutation_authority_ref,
            authority_evidence_ref,
        )?;
        let validated = ValidatedMemoryControlAuthority {
            target: request.target,
            effect_id: request.effect_id,
            evidence_id,
            kernel_authorization_ref,
            mutation_authority_ref,
            authority_evidence_ref,
            approving_principal: None,
            verifier_policy_ref: Some(policy_ref),
        };
        qualify(
            validated,
            MemoryControlRevalidation::Deterministic {
                target: request.target,
                initiating_principal: request.initiating_principal.clone(),
                authorization_decision_id: request.authorization_decision_id,
                rule_id: request.rule_id,
                rule_version: request.rule_version,
                authority_source_binding: request.authority_source_binding.to_vec(),
                evidence_hash: request.evidence_hash,
                effect_id: request.effect_id,
            },
        )
    }

    pub fn revalidate(
        &mut self,
        layout: &AuthorityLayout,
        authority: &QualifiedMemoryControlAuthority,
        observed_at: &str,
    ) -> Result<(), MemoryControlAuthorityError> {
        let refreshed = match &authority.revalidation {
            MemoryControlRevalidation::Human {
                target,
                initiating_principal,
                authorization_decision_id,
                approval_id,
                effect_id,
            } => self.validate_human(
                layout,
                HumanMemoryControlRequest {
                    target: *target,
                    initiating_principal,
                    authorization_decision_id: *authorization_decision_id,
                    approval_id: *approval_id,
                    effect_id: *effect_id,
                    observed_at,
                },
            )?,
            MemoryControlRevalidation::Deterministic {
                target,
                initiating_principal,
                authorization_decision_id,
                rule_id,
                rule_version,
                authority_source_binding,
                evidence_hash,
                effect_id,
            } => self.validate_deterministic(DeterministicMemoryControlRequest {
                target: *target,
                initiating_principal,
                authorization_decision_id: *authorization_decision_id,
                rule_id: *rule_id,
                rule_version: *rule_version,
                authority_source_binding,
                evidence_hash: *evidence_hash,
                effect_id: *effect_id,
            })?,
        };
        if refreshed.validated != authority.validated {
            return Err(MemoryControlAuthorityError::RevalidationMismatch);
        }
        Ok(())
    }

    fn validate_current_authorization(
        &mut self,
        decision_id: [u8; 16],
        expected_principal: &PrincipalId,
        expected_resource: &str,
    ) -> Result<BindingDigest, MemoryControlAuthorityError> {
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
            .ok_or(MemoryControlAuthorityError::MissingAuthorityDecision)?;
        if row.0 != expected_principal.as_str()
            || row.1 != MEMORY_MUTATION_ACTION
            || row.2 != expected_resource
            || row.4 != "pass"
            || row.5 != "allow"
        {
            return Err(MemoryControlAuthorityError::AuthorityDecisionMismatch);
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
            return Err(MemoryControlAuthorityError::StaleAuthorityDecision);
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
    ) -> Result<String, MemoryControlAuthorityError> {
        self.connection
            .query_row(
                "SELECT approver_principal FROM approvals WHERE approval_id = ?1",
                params![&approval_id[..]],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or(MemoryControlAuthorityError::InvalidStoredRecord(
                "validated approval disappeared",
            ))
    }

    fn active_verifier_rule(
        &mut self,
        rule_id: [u8; 16],
    ) -> Result<StoredVerifierRule, MemoryControlAuthorityError> {
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
            .ok_or(MemoryControlAuthorityError::VerifierRuleNotFound)?;
        if row.4 != "active" {
            return Err(MemoryControlAuthorityError::VerifierRuleInactive);
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

pub fn memory_control_resource(target: MemoryControlTarget) -> String {
    let scope = match target.scope {
        MemoryScope::User => "user",
        MemoryScope::Project => "project",
    };
    let operation = match target.operation {
        MemoryOperation::Expire => "expire",
        MemoryOperation::Forget => "forget",
        MemoryOperation::Redact => "redact",
        _ => "invalid",
    };
    format!(
        "memory-control:{scope}:{operation}:{}:{}",
        hex_bytes(&target.item_id.0.bytes()),
        hex_bytes(&target.expected_version.0.bytes())
    )
}

fn verify_transaction_integrity(
    transaction: &Transaction<'_>,
) -> Result<(), MemoryControlAuthorityError> {
    crate::integrity::verify(transaction)
        .map_err(|error| MemoryControlAuthorityError::Integrity(error.to_string()))?;
    crate::authority_security_v2::verify(transaction)
        .map_err(|error| MemoryControlAuthorityError::AuthoritySecurity(error.to_string()))
}

fn taint_digest(taint: TaintSet) -> Result<[u8; 32], MemoryControlAuthorityError> {
    Ok(*blake3::hash(&taint.canonical_bytes()?).as_bytes())
}

fn kernel_authorization_ref(
    decision_id: [u8; 16],
    principal: &PrincipalId,
    resource: &str,
    context_hash: [u8; 32],
    global_seq: u64,
) -> Result<BindingDigest, MemoryControlAuthorityError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(KERNEL_AUTHORIZATION_REF_DOMAIN)?;
    encoder.push_bytes(&decision_id)?;
    encoder.push_bytes(principal.as_str().as_bytes())?;
    encoder.push_bytes(MEMORY_MUTATION_ACTION.as_bytes())?;
    encoder.push_bytes(resource.as_bytes())?;
    encoder.push_bytes(&context_hash)?;
    encoder.push_u64(global_seq);
    Ok(BindingDigest::new(
        *blake3::hash(&encoder.finish()).as_bytes(),
    ))
}

fn human_control_ref(
    target: MemoryControlTarget,
    approval_id: [u8; 16],
    class: ApprovalClass,
    scope_digest: [u8; 32],
    parent_decision_id: [u8; 16],
    current_uses: u64,
    max_uses: u64,
) -> Result<BindingDigest, MemoryControlAuthorityError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(HUMAN_CONTROL_REF_DOMAIN)?;
    push_target(&mut encoder, target)?;
    encoder.push_bytes(&approval_id)?;
    encoder.push_bytes(class.as_str().as_bytes())?;
    encoder.push_bytes(&scope_digest)?;
    encoder.push_bytes(&parent_decision_id)?;
    encoder.push_u64(current_uses);
    encoder.push_u64(max_uses);
    Ok(BindingDigest::new(
        *blake3::hash(&encoder.finish()).as_bytes(),
    ))
}

fn verifier_control_ref(
    target: MemoryControlTarget,
    rule_id: [u8; 16],
    version: u64,
    authority_source_binding: &[u8],
    registered_by: &str,
    created_global_seq: u64,
) -> Result<BindingDigest, MemoryControlAuthorityError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(VERIFIER_CONTROL_REF_DOMAIN)?;
    push_target(&mut encoder, target)?;
    encoder.push_bytes(&rule_id)?;
    encoder.push_u64(version);
    encoder.push_bytes(authority_source_binding)?;
    encoder.push_bytes(registered_by.as_bytes())?;
    encoder.push_u64(created_global_seq);
    Ok(BindingDigest::new(
        *blake3::hash(&encoder.finish()).as_bytes(),
    ))
}

fn authority_evidence_ref(
    target: MemoryControlTarget,
    effect_id: EffectId,
    mutation_authority_ref: BindingDigest,
    evidence: &[u8; 32],
) -> Result<BindingDigest, MemoryControlAuthorityError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(CONTROL_EVIDENCE_REF_DOMAIN)?;
    push_target(&mut encoder, target)?;
    encoder.push_u128(effect_id.0);
    encoder.push_bytes(&mutation_authority_ref.bytes())?;
    encoder.push_bytes(evidence)?;
    Ok(BindingDigest::new(
        *blake3::hash(&encoder.finish()).as_bytes(),
    ))
}

fn control_evidence_id(
    target: MemoryControlTarget,
    effect_id: EffectId,
    kernel_authorization_ref: BindingDigest,
    mutation_authority_ref: BindingDigest,
    authority_evidence_ref: BindingDigest,
) -> Result<BindingDigest, MemoryControlAuthorityError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(b"golam:memory-control-evidence-id:v1")?;
    push_target(&mut encoder, target)?;
    encoder.push_u128(effect_id.0);
    encoder.push_bytes(&kernel_authorization_ref.bytes())?;
    encoder.push_bytes(&mutation_authority_ref.bytes())?;
    encoder.push_bytes(&authority_evidence_ref.bytes())?;
    Ok(BindingDigest::new(
        *blake3::hash(&encoder.finish()).as_bytes(),
    ))
}

fn qualify(
    validated: ValidatedMemoryControlAuthority,
    revalidation: MemoryControlRevalidation,
) -> Result<QualifiedMemoryControlAuthority, MemoryControlAuthorityError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(QUALIFIED_CONTROL_RECORD_DOMAIN)?;
    push_target(&mut encoder, validated.target)?;
    encoder.push_u128(validated.effect_id.0);
    encoder.push_bytes(&validated.evidence_id.bytes())?;
    encoder.push_bytes(&validated.kernel_authorization_ref.bytes())?;
    encoder.push_bytes(&validated.mutation_authority_ref.bytes())?;
    encoder.push_bytes(&validated.authority_evidence_ref.bytes())?;
    match (
        validated.approving_principal.as_ref(),
        validated.verifier_policy_ref,
    ) {
        (Some(principal), None) => {
            encoder.push_u8(1);
            encoder.push_bytes(principal.as_str().as_bytes())?;
        }
        (None, Some(verifier)) => {
            encoder.push_u8(2);
            encoder.push_bytes(&verifier.bytes())?;
        }
        _ => unreachable!("validated memory control admits exactly one authority mode"),
    }
    Ok(QualifiedMemoryControlAuthority {
        validated,
        revalidation,
        record_bytes: encoder.finish(),
    })
}

fn push_target(
    encoder: &mut CanonicalEncoder,
    target: MemoryControlTarget,
) -> Result<(), MemoryControlAuthorityError> {
    target.validate()?;
    encoder.push_u8(match target.operation {
        MemoryOperation::Expire => 1,
        MemoryOperation::Forget => 2,
        MemoryOperation::Redact => 3,
        _ => unreachable!("validated target admits only candidate-less control operations"),
    });
    encoder.push_u8(match target.scope {
        MemoryScope::User => 1,
        MemoryScope::Project => 2,
    });
    encoder.push_bytes(&target.item_id.0.bytes())?;
    encoder.push_bytes(&target.expected_version.0.bytes())?;
    encoder.push_bytes(&target.taint_set.canonical_bytes()?)?;
    Ok(())
}

fn hash32(value: Vec<u8>, reason: &'static str) -> Result<[u8; 32], MemoryControlAuthorityError> {
    value
        .try_into()
        .map_err(|_| MemoryControlAuthorityError::InvalidStoredRecord(reason))
}

fn from_i64(value: i64, reason: &'static str) -> Result<u64, MemoryControlAuthorityError> {
    u64::try_from(value).map_err(|_| MemoryControlAuthorityError::InvalidStoredRecord(reason))
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
    use golam_core::taint::TaintLabel;

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn target(operation: MemoryOperation) -> MemoryControlTarget {
        MemoryControlTarget {
            operation,
            item_id: MemoryItemId(digest(1)),
            scope: MemoryScope::Project,
            expected_version: MemoryVersionId(digest(2)),
            taint_set: TaintSet::from_labels([TaintLabel::UserTrusted]),
        }
    }

    #[test]
    fn only_candidate_less_control_operations_are_admitted() {
        for operation in [
            MemoryOperation::Expire,
            MemoryOperation::Forget,
            MemoryOperation::Redact,
        ] {
            target(operation).validate().unwrap();
        }
        assert!(matches!(
            target(MemoryOperation::Update).validate(),
            Err(MemoryControlAuthorityError::CandidateOperation)
        ));
    }

    #[test]
    fn control_resource_binds_scope_operation_item_and_version() {
        let expire = memory_control_resource(target(MemoryOperation::Expire));
        let redact = memory_control_resource(target(MemoryOperation::Redact));
        assert_ne!(expire, redact);
        assert!(expire.starts_with("memory-control:project:expire:"));
        assert!(expire.ends_with(&hex_bytes(&digest(2).bytes())));
    }
}
