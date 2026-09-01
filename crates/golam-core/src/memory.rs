#![forbid(unsafe_code)]

use core::fmt;

use crate::digest::sha256;
use crate::taint::{
    validate_canonical_long_term_memory_admission, CanonicalMemoryAdmissionError, TaintSet,
};
use crate::tool_request::{BindingDigest, PrincipalId};
use crate::{CanonicalEncoder, CoreError, EffectId};

const MAX_MEMORY_REFS: usize = 128;
const MEMORY_MUTATION_INTENT_DOMAIN: &[u8] = b"golam:memory-mutation-intent:v1";

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MemoryCandidateId(pub BindingDigest);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MemoryItemId(pub BindingDigest);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MemoryVersionId(pub BindingDigest);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MemoryStoreId(pub BindingDigest);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MemoryWriterId(pub BindingDigest);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MemoryDerivativeGenerationId(pub BindingDigest);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MemoryScope {
    User,
    Project,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MemoryAuthorityClass {
    UntrustedContent,
    UserAttributed,
    LocalObserved,
    CanonicalGolam,
    ExternalAuthoritative,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromotionRequirement {
    AttributableHumanApproval {
        approval_policy_ref: BindingDigest,
    },
    DeterministicPreregisteredVerifier {
        verifier_policy_ref: BindingDigest,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryCandidate {
    pub candidate_id: MemoryCandidateId,
    pub scope: MemoryScope,
    pub proposed_content_ref: BindingDigest,
    pub provenance_refs: Vec<BindingDigest>,
    pub taint_set: TaintSet,
    pub authority_class: MemoryAuthorityClass,
    pub created_by_principal: PrincipalId,
    pub created_at_unix_ms: u64,
    pub promotion_requirement: PromotionRequirement,
}

impl MemoryCandidate {
    pub fn validate(&self) -> Result<(), MemoryValidationError> {
        validate_ordered_unique(&self.provenance_refs, "provenance_refs")?;
        if self.provenance_refs.is_empty() {
            return Err(MemoryValidationError::MissingRequirement(
                "candidate provenance_refs",
            ));
        }
        Ok(())
    }

    pub fn validate_for_canonical_promotion(&self) -> Result<(), MemoryValidationError> {
        self.validate()?;
        validate_canonical_long_term_memory_admission(self.taint_set)
            .map_err(MemoryValidationError::MemoryAdmission)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MemoryOperation {
    Add,
    Update,
    Supersede,
    Contradict,
    Merge,
    Expire,
    Forget,
    Redact,
}

impl MemoryOperation {
    const fn requires_candidate(self) -> bool {
        matches!(
            self,
            Self::Add | Self::Update | Self::Supersede | Self::Contradict | Self::Merge
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExpectedMemoryVersion {
    pub item_id: MemoryItemId,
    pub expected_version: Option<MemoryVersionId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryMutationIntent {
    pub operation: MemoryOperation,
    pub item_ids: Vec<MemoryItemId>,
    pub expected_current_versions: Vec<ExpectedMemoryVersion>,
    pub expected_markdown_target_identity_ref: BindingDigest,
    pub expected_markdown_content_digest: BindingDigest,
    pub expected_markdown_version: MemoryVersionId,
    pub memory_operational_store_ref: MemoryStoreId,
    pub candidate_ref: Option<MemoryCandidateId>,
    pub kernel_authorization_ref: BindingDigest,
    pub promotion_authority_ref: BindingDigest,
    pub effect_id: EffectId,
    pub reason_ref: BindingDigest,
    pub initiating_principal: PrincipalId,
    pub created_at_unix_ms: u64,
}

impl MemoryMutationIntent {
    pub fn validate(&self) -> Result<(), MemoryValidationError> {
        validate_ordered_unique(&self.item_ids, "item_ids")?;
        validate_ordered_unique(
            &self.expected_current_versions,
            "expected_current_versions",
        )?;
        if self.item_ids.is_empty() {
            return Err(MemoryValidationError::MissingRequirement("item_ids"));
        }
        if self.expected_current_versions.len() != self.item_ids.len() {
            return Err(MemoryValidationError::VersionBindingMismatch);
        }
        if self
            .item_ids
            .iter()
            .zip(&self.expected_current_versions)
            .any(|(item, expected)| *item != expected.item_id)
        {
            return Err(MemoryValidationError::VersionBindingMismatch);
        }
        if self.operation.requires_candidate() && self.candidate_ref.is_none() {
            return Err(MemoryValidationError::MissingRequirement("candidate_ref"));
        }
        if !self.operation.requires_candidate() && self.candidate_ref.is_some() {
            return Err(MemoryValidationError::UnexpectedCandidateRef);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, MemoryValidationError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(MEMORY_MUTATION_INTENT_DOMAIN)?;
        encoder.push_u8(memory_operation_code(self.operation));
        push_memory_item_ids(&mut encoder, &self.item_ids)?;
        encoder.push_u64(self.expected_current_versions.len() as u64);
        for expected in &self.expected_current_versions {
            push_digest(&mut encoder, expected.item_id.0)?;
            push_optional_digest(
                &mut encoder,
                expected.expected_version.map(|version| version.0),
            )?;
        }
        push_digest(
            &mut encoder,
            self.expected_markdown_target_identity_ref,
        )?;
        push_digest(&mut encoder, self.expected_markdown_content_digest)?;
        push_digest(&mut encoder, self.expected_markdown_version.0)?;
        push_digest(&mut encoder, self.memory_operational_store_ref.0)?;
        push_optional_digest(&mut encoder, self.candidate_ref.map(|candidate| candidate.0))?;
        push_digest(&mut encoder, self.kernel_authorization_ref)?;
        push_digest(&mut encoder, self.promotion_authority_ref)?;
        encoder.push_u128(self.effect_id.0);
        push_digest(&mut encoder, self.reason_ref)?;
        encoder.push_bytes(self.initiating_principal.as_str().as_bytes())?;
        encoder.push_u64(self.created_at_unix_ms);
        Ok(encoder.finish())
    }

    pub fn binding_digest(&self) -> Result<[u8; 32], MemoryValidationError> {
        Ok(sha256(&self.canonical_bytes()?))
    }

    pub fn prepare(self) -> Result<PreparedMemoryMutationIntent, MemoryValidationError> {
        let binding_digest = self.binding_digest()?;
        Ok(PreparedMemoryMutationIntent {
            intent: self,
            binding_digest,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedMemoryMutationIntent {
    intent: MemoryMutationIntent,
    binding_digest: [u8; 32],
}

impl PreparedMemoryMutationIntent {
    pub fn intent(&self) -> &MemoryMutationIntent {
        &self.intent
    }

    pub const fn binding_digest(&self) -> [u8; 32] {
        self.binding_digest
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MemoryMutationStatus {
    Committed,
    Rejected,
    Failed,
    UnknownOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryMutationOutcome {
    pub effect_id: EffectId,
    pub mutation_intent_digest: BindingDigest,
    pub status: MemoryMutationStatus,
    pub canonical_version_refs: Vec<MemoryVersionId>,
    pub authority_journal_readback_ref: Option<BindingDigest>,
    pub markdown_readback_ref: Option<BindingDigest>,
    pub memory_sqlite_readback_ref: Option<BindingDigest>,
    pub reconciliation_ref: Option<BindingDigest>,
    pub verification_refs: Vec<BindingDigest>,
    pub integrity_evidence_refs: Vec<BindingDigest>,
    pub terminal_at_unix_ms: u64,
}

impl MemoryMutationOutcome {
    pub fn validate(&self) -> Result<(), MemoryValidationError> {
        validate_ordered_unique(&self.canonical_version_refs, "canonical_version_refs")?;
        validate_ordered_unique(&self.verification_refs, "verification_refs")?;
        validate_ordered_unique(&self.integrity_evidence_refs, "integrity_evidence_refs")?;
        if self.status == MemoryMutationStatus::Committed
            && (self.authority_journal_readback_ref.is_none()
                || self.markdown_readback_ref.is_none()
                || self.memory_sqlite_readback_ref.is_none()
                || self.reconciliation_ref.is_none()
                || self.verification_refs.is_empty()
                || self.integrity_evidence_refs.is_empty())
        {
            return Err(MemoryValidationError::CommittedWithoutCrossStoreEvidence);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MemoryVersionStatus {
    Active,
    Superseded,
    Contradicted,
    Expired,
    Forgotten,
    Redacted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryVersion {
    pub item_id: MemoryItemId,
    pub version_id: MemoryVersionId,
    pub scope: MemoryScope,
    pub canonical_markdown_ref: BindingDigest,
    pub content_digest: BindingDigest,
    pub provenance_refs: Vec<BindingDigest>,
    pub taint_set: TaintSet,
    pub status: MemoryVersionStatus,
    pub predecessor_versions: Vec<MemoryVersionId>,
    pub conflict_refs: Vec<BindingDigest>,
    pub promotion_evidence_ref: BindingDigest,
    pub created_by_principal: PrincipalId,
    pub committed_by_writer_identity: MemoryWriterId,
    pub mutation_effect_ref: EffectId,
    pub created_at_unix_ms: u64,
}

impl MemoryVersion {
    pub fn validate(&self) -> Result<(), MemoryValidationError> {
        validate_canonical_long_term_memory_admission(self.taint_set)
            .map_err(MemoryValidationError::MemoryAdmission)?;
        validate_ordered_unique(&self.provenance_refs, "version provenance_refs")?;
        validate_ordered_unique(&self.predecessor_versions, "predecessor_versions")?;
        validate_ordered_unique(&self.conflict_refs, "conflict_refs")?;
        if self.provenance_refs.is_empty() {
            return Err(MemoryValidationError::MissingRequirement(
                "version provenance_refs",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MemoryReconciliationState {
    InSync,
    UserEditDetected,
    Conflict,
    Reconciled,
    Blocked,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DerivativeIndexStatus {
    Current,
    Stale,
    Rebuilding,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivativeIndexGeneration {
    pub index_kind_ref: BindingDigest,
    pub generation_id: MemoryDerivativeGenerationId,
    pub canonical_cut_digest: BindingDigest,
    pub implementation_identity: BindingDigest,
    pub status: DerivativeIndexStatus,
    pub built_at_unix_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryValidationError {
    TooManyReferences(&'static str),
    UnsortedOrDuplicate(&'static str),
    MissingRequirement(&'static str),
    VersionBindingMismatch,
    UnexpectedCandidateRef,
    CommittedWithoutCrossStoreEvidence,
    MemoryAdmission(CanonicalMemoryAdmissionError),
    CanonicalEncoding(CoreError),
}

impl fmt::Display for MemoryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyReferences(field) => {
                write!(f, "memory reference bound exceeded: {field}")
            }
            Self::UnsortedOrDuplicate(field) => {
                write!(f, "memory references must be sorted and unique: {field}")
            }
            Self::MissingRequirement(field) => {
                write!(f, "required memory binding is missing: {field}")
            }
            Self::VersionBindingMismatch => {
                f.write_str("expected memory versions do not bind the exact ordered item set")
            }
            Self::UnexpectedCandidateRef => {
                f.write_str("non-content memory operation cannot carry a candidate reference")
            }
            Self::CommittedWithoutCrossStoreEvidence => f.write_str(
                "committed memory outcome requires authority, Markdown, SQLite, reconciliation, verification, and integrity evidence",
            ),
            Self::MemoryAdmission(error) => write!(f, "memory admission denied: {error}"),
            Self::CanonicalEncoding(error) => write!(f, "canonical encoding error: {error}"),
        }
    }
}

impl std::error::Error for MemoryValidationError {}

impl From<CoreError> for MemoryValidationError {
    fn from(value: CoreError) -> Self {
        Self::CanonicalEncoding(value)
    }
}

fn validate_ordered_unique<T: Ord>(
    values: &[T],
    field: &'static str,
) -> Result<(), MemoryValidationError> {
    if values.len() > MAX_MEMORY_REFS {
        return Err(MemoryValidationError::TooManyReferences(field));
    }
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(MemoryValidationError::UnsortedOrDuplicate(field));
    }
    Ok(())
}

fn push_digest(
    encoder: &mut CanonicalEncoder,
    digest: BindingDigest,
) -> Result<(), MemoryValidationError> {
    encoder.push_bytes(&digest.bytes())?;
    Ok(())
}

fn push_optional_digest(
    encoder: &mut CanonicalEncoder,
    digest: Option<BindingDigest>,
) -> Result<(), MemoryValidationError> {
    match digest {
        Some(value) => {
            encoder.push_u8(1);
            push_digest(encoder, value)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn push_memory_item_ids(
    encoder: &mut CanonicalEncoder,
    values: &[MemoryItemId],
) -> Result<(), MemoryValidationError> {
    encoder.push_u64(values.len() as u64);
    for value in values {
        push_digest(encoder, value.0)?;
    }
    Ok(())
}

const fn memory_operation_code(operation: MemoryOperation) -> u8 {
    match operation {
        MemoryOperation::Add => 1,
        MemoryOperation::Update => 2,
        MemoryOperation::Supersede => 3,
        MemoryOperation::Contradict => 4,
        MemoryOperation::Merge => 5,
        MemoryOperation::Expire => 6,
        MemoryOperation::Forget => 7,
        MemoryOperation::Redact => 8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::taint::TaintLabel;

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn item(value: u8) -> MemoryItemId {
        MemoryItemId(digest(value))
    }

    fn version(value: u8) -> MemoryVersionId {
        MemoryVersionId(digest(value))
    }

    fn intent() -> MemoryMutationIntent {
        MemoryMutationIntent {
            operation: MemoryOperation::Update,
            item_ids: vec![item(1)],
            expected_current_versions: vec![ExpectedMemoryVersion {
                item_id: item(1),
                expected_version: Some(version(2)),
            }],
            expected_markdown_target_identity_ref: digest(3),
            expected_markdown_content_digest: digest(4),
            expected_markdown_version: version(5),
            memory_operational_store_ref: MemoryStoreId(digest(6)),
            candidate_ref: Some(MemoryCandidateId(digest(7))),
            kernel_authorization_ref: digest(8),
            promotion_authority_ref: digest(9),
            effect_id: EffectId(10),
            reason_ref: digest(11),
            initiating_principal: PrincipalId::new("principal.local").unwrap(),
            created_at_unix_ms: 12,
        }
    }

    #[test]
    fn mutation_intent_digest_binds_every_protected_field_and_prepare_freezes_it() {
        let original = intent();
        let digest_before = original.binding_digest().unwrap();
        let prepared = original.clone().prepare().unwrap();
        assert_eq!(prepared.binding_digest(), digest_before);
        assert_eq!(prepared.intent(), &original);

        let mut changed = original;
        changed.memory_operational_store_ref = MemoryStoreId(digest(99));
        assert_ne!(changed.binding_digest().unwrap(), digest_before);
    }

    #[test]
    fn version_bindings_must_match_exact_item_order() {
        let mut value = intent();
        value.expected_current_versions[0].item_id = item(2);
        assert_eq!(
            value.validate(),
            Err(MemoryValidationError::VersionBindingMismatch)
        );
    }

    #[test]
    fn secret_derived_candidate_cannot_be_promoted() {
        let candidate = MemoryCandidate {
            candidate_id: MemoryCandidateId(digest(1)),
            scope: MemoryScope::Project,
            proposed_content_ref: digest(2),
            provenance_refs: vec![digest(3)],
            taint_set: TaintSet::from_labels([
                TaintLabel::UserTrusted,
                TaintLabel::SecretDerived,
            ]),
            authority_class: MemoryAuthorityClass::UserAttributed,
            created_by_principal: PrincipalId::new("principal.local").unwrap(),
            created_at_unix_ms: 4,
            promotion_requirement: PromotionRequirement::AttributableHumanApproval {
                approval_policy_ref: digest(5),
            },
        };
        assert!(matches!(
            candidate.validate_for_canonical_promotion(),
            Err(MemoryValidationError::MemoryAdmission(
                CanonicalMemoryAdmissionError::SecretDerived
            ))
        ));
    }

    #[test]
    fn committed_outcome_requires_cross_store_readback_and_integrity_evidence() {
        let outcome = MemoryMutationOutcome {
            effect_id: EffectId(1),
            mutation_intent_digest: digest(2),
            status: MemoryMutationStatus::Committed,
            canonical_version_refs: vec![version(3)],
            authority_journal_readback_ref: Some(digest(4)),
            markdown_readback_ref: Some(digest(5)),
            memory_sqlite_readback_ref: Some(digest(6)),
            reconciliation_ref: Some(digest(7)),
            verification_refs: vec![digest(8)],
            integrity_evidence_refs: vec![digest(9)],
            terminal_at_unix_ms: 10,
        };
        assert_eq!(outcome.validate(), Ok(()));

        let mut invalid = outcome;
        invalid.memory_sqlite_readback_ref = None;
        assert_eq!(
            invalid.validate(),
            Err(MemoryValidationError::CommittedWithoutCrossStoreEvidence)
        );
    }

    #[test]
    fn memory_version_preserves_creator_writer_and_effect_as_distinct_attribution() {
        let value = MemoryVersion {
            item_id: item(1),
            version_id: version(2),
            scope: MemoryScope::User,
            canonical_markdown_ref: digest(3),
            content_digest: digest(4),
            provenance_refs: vec![digest(5)],
            taint_set: TaintSet::from_labels([TaintLabel::UserTrusted]),
            status: MemoryVersionStatus::Active,
            predecessor_versions: vec![],
            conflict_refs: vec![],
            promotion_evidence_ref: digest(6),
            created_by_principal: PrincipalId::new("principal.creator").unwrap(),
            committed_by_writer_identity: MemoryWriterId(digest(7)),
            mutation_effect_ref: EffectId(8),
            created_at_unix_ms: 9,
        };
        assert_eq!(value.validate(), Ok(()));
        assert_eq!(value.created_by_principal.as_str(), "principal.creator");
        assert_eq!(value.committed_by_writer_identity, MemoryWriterId(digest(7)));
        assert_eq!(value.mutation_effect_ref, EffectId(8));
    }
}
