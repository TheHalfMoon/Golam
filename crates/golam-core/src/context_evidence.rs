#![forbid(unsafe_code)]

use core::fmt;

use crate::taint::{TaintLabel, TaintSet};
use crate::tool_request::BindingDigest;

const MAX_REQUIREMENT_ITEMS: usize = 128;
const MAX_CONTEXT_REFS: usize = 512;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EvidenceSourceId(pub BindingDigest);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EvidenceSourceKind {
    UserSelectedArtifact,
    File,
    GitObject,
    CanonicalLedger,
    ManagedMemory,
    ProtocolResource,
    ExternalDocument,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EvidenceAuthorityClass {
    UntrustedContent,
    UserAttributed,
    LocalObserved,
    CanonicalGolam,
    ExternalAuthoritative,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PermissionScopeId(pub BindingDigest);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FreshnessPolicy {
    Snapshot,
    MaxAgeMs(u64),
    LiveObservation,
}

impl FreshnessPolicy {
    pub fn validate(self) -> Result<(), ContextValidationError> {
        match self {
            Self::MaxAgeMs(0) => Err(ContextValidationError::InvalidFreshnessPolicy),
            _ => Ok(()),
        }
    }

    pub fn is_fresh(self, observed_at_unix_ms: u64, now_unix_ms: u64) -> bool {
        if observed_at_unix_ms > now_unix_ms {
            return false;
        }
        match self {
            Self::Snapshot => true,
            Self::MaxAgeMs(max_age_ms) => now_unix_ms - observed_at_unix_ms <= max_age_ms,
            Self::LiveObservation => observed_at_unix_ms == now_unix_ms,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextEvidence {
    pub evidence_id: BindingDigest,
    pub source_id: EvidenceSourceId,
    pub source_kind: EvidenceSourceKind,
    pub source_version_or_observation: BindingDigest,
    pub content_ref: BindingDigest,
    pub content_digest: BindingDigest,
    pub authority_class: EvidenceAuthorityClass,
    pub taint_set: TaintSet,
    pub permission_scope: PermissionScopeId,
    pub freshness_policy: FreshnessPolicy,
    pub observed_at_unix_ms: u64,
    pub supersedes_or_conflicts_with: Vec<BindingDigest>,
}

impl ContextEvidence {
    pub fn validate(&self, now_unix_ms: u64) -> Result<(), ContextValidationError> {
        self.freshness_policy.validate()?;
        validate_ordered_unique(
            &self.supersedes_or_conflicts_with,
            "supersedes_or_conflicts_with",
            MAX_CONTEXT_REFS,
        )?;
        if !self
            .freshness_policy
            .is_fresh(self.observed_at_unix_ms, now_unix_ms)
        {
            return Err(ContextValidationError::StaleEvidence(self.evidence_id));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceRequirement {
    pub requirement_id: BindingDigest,
    pub allowed_source_kinds: Vec<EvidenceSourceKind>,
    pub allowed_authority_classes: Vec<EvidenceAuthorityClass>,
    pub forbidden_taint: TaintSet,
    pub required_permission_scope: Option<PermissionScopeId>,
    pub minimum_observed_at_unix_ms: Option<u64>,
}

impl EvidenceRequirement {
    pub fn validate(&self) -> Result<(), ContextValidationError> {
        if self.allowed_source_kinds.is_empty() || self.allowed_authority_classes.is_empty() {
            return Err(ContextValidationError::EmptyRequirementPolicy);
        }
        validate_ordered_unique(
            &self.allowed_source_kinds,
            "allowed_source_kinds",
            MAX_REQUIREMENT_ITEMS,
        )?;
        validate_ordered_unique(
            &self.allowed_authority_classes,
            "allowed_authority_classes",
            MAX_REQUIREMENT_ITEMS,
        )?;
        Ok(())
    }

    pub fn accepts(
        &self,
        evidence: &ContextEvidence,
        now_unix_ms: u64,
    ) -> Result<bool, ContextValidationError> {
        self.validate()?;
        evidence.validate(now_unix_ms)?;
        if self
            .allowed_source_kinds
            .binary_search(&evidence.source_kind)
            .is_err()
            || self
                .allowed_authority_classes
                .binary_search(&evidence.authority_class)
                .is_err()
        {
            return Ok(false);
        }
        if self
            .forbidden_taint
            .labels()
            .any(|label| evidence.taint_set.contains(label))
        {
            return Ok(false);
        }
        if self
            .required_permission_scope
            .is_some_and(|scope| scope != evidence.permission_scope)
        {
            return Ok(false);
        }
        if self
            .minimum_observed_at_unix_ms
            .is_some_and(|minimum| evidence.observed_at_unix_ms < minimum)
        {
            return Ok(false);
        }
        Ok(true)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RankingEvidence {
    pub evidence_ref: BindingDigest,
    pub bounded_score: i32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SufficiencyState {
    Sufficient,
    Insufficient,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextCapsule {
    pub capsule_id: BindingDigest,
    pub intent_ref: BindingDigest,
    pub requirement_refs: Vec<BindingDigest>,
    pub evidence_refs: Vec<BindingDigest>,
    pub memory_refs: Vec<BindingDigest>,
    pub ranking_evidence: Vec<RankingEvidence>,
    pub sufficiency_state: SufficiencyState,
    pub missing_requirements: Vec<BindingDigest>,
    pub projection_policy_ref: BindingDigest,
    pub created_at_unix_ms: u64,
}

impl ContextCapsule {
    pub fn validate(&self) -> Result<(), ContextValidationError> {
        validate_ordered_unique(&self.requirement_refs, "requirement_refs", MAX_CONTEXT_REFS)?;
        validate_ordered_unique(&self.evidence_refs, "evidence_refs", MAX_CONTEXT_REFS)?;
        validate_ordered_unique(&self.memory_refs, "memory_refs", MAX_CONTEXT_REFS)?;
        validate_ordered_unique(&self.ranking_evidence, "ranking_evidence", MAX_CONTEXT_REFS)?;
        validate_ordered_unique(
            &self.missing_requirements,
            "missing_requirements",
            MAX_CONTEXT_REFS,
        )?;

        if self.ranking_evidence.iter().any(|ranking| {
            self.evidence_refs
                .binary_search(&ranking.evidence_ref)
                .is_err()
        }) {
            return Err(ContextValidationError::RankingReferencesUnknownEvidence);
        }

        match self.sufficiency_state {
            SufficiencyState::Sufficient if !self.missing_requirements.is_empty() => {
                Err(ContextValidationError::SufficientCapsuleHasMissingRequirement)
            }
            SufficiencyState::Insufficient if self.missing_requirements.is_empty() => {
                Err(ContextValidationError::InsufficientCapsuleNeedsMissingRequirement)
            }
            _ => Ok(()),
        }
    }
}

pub fn missing_requirements(
    requirements: &[EvidenceRequirement],
    evidence: &[ContextEvidence],
    now_unix_ms: u64,
) -> Result<Vec<BindingDigest>, ContextValidationError> {
    if requirements.len() > MAX_REQUIREMENT_ITEMS || evidence.len() > MAX_CONTEXT_REFS {
        return Err(ContextValidationError::TooManyItems("context evaluation"));
    }
    let mut missing = Vec::new();
    for requirement in requirements {
        requirement.validate()?;
        let mut satisfied = false;
        for item in evidence {
            match requirement.accepts(item, now_unix_ms) {
                Ok(true) => {
                    satisfied = true;
                    break;
                }
                Ok(false) => {}
                Err(ContextValidationError::StaleEvidence(_)) => {}
                Err(error) => return Err(error),
            }
        }
        if !satisfied {
            missing.push(requirement.requirement_id);
        }
    }
    missing.sort_unstable();
    missing.dedup();
    Ok(missing)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextValidationError {
    InvalidFreshnessPolicy,
    StaleEvidence(BindingDigest),
    EmptyRequirementPolicy,
    TooManyItems(&'static str),
    UnsortedOrDuplicate(&'static str),
    RankingReferencesUnknownEvidence,
    SufficientCapsuleHasMissingRequirement,
    InsufficientCapsuleNeedsMissingRequirement,
}

impl fmt::Display for ContextValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFreshnessPolicy => f.write_str("freshness policy must be finite"),
            Self::StaleEvidence(_) => f.write_str("context evidence is stale or from the future"),
            Self::EmptyRequirementPolicy => {
                f.write_str("evidence requirement must allow source and authority classes")
            }
            Self::TooManyItems(field) => write!(f, "context item bound exceeded: {field}"),
            Self::UnsortedOrDuplicate(field) => {
                write!(
                    f,
                    "context references must be strictly sorted and unique: {field}"
                )
            }
            Self::RankingReferencesUnknownEvidence => {
                f.write_str("ranking evidence references an item outside the capsule")
            }
            Self::SufficientCapsuleHasMissingRequirement => {
                f.write_str("sufficient context capsule cannot list missing requirements")
            }
            Self::InsufficientCapsuleNeedsMissingRequirement => {
                f.write_str("insufficient context capsule must list missing requirements")
            }
        }
    }
}

impl std::error::Error for ContextValidationError {}

fn validate_ordered_unique<T: Ord>(
    values: &[T],
    field: &'static str,
    max_items: usize,
) -> Result<(), ContextValidationError> {
    if values.len() > max_items {
        return Err(ContextValidationError::TooManyItems(field));
    }
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(ContextValidationError::UnsortedOrDuplicate(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn evidence() -> ContextEvidence {
        ContextEvidence {
            evidence_id: digest(1),
            source_id: EvidenceSourceId(digest(2)),
            source_kind: EvidenceSourceKind::File,
            source_version_or_observation: digest(3),
            content_ref: digest(4),
            content_digest: digest(5),
            authority_class: EvidenceAuthorityClass::LocalObserved,
            taint_set: TaintSet::from_labels([TaintLabel::LocalTrusted]),
            permission_scope: PermissionScopeId(digest(6)),
            freshness_policy: FreshnessPolicy::MaxAgeMs(100),
            observed_at_unix_ms: 950,
            supersedes_or_conflicts_with: vec![],
        }
    }

    #[test]
    fn requirement_filters_authority_taint_permission_and_freshness() {
        let requirement = EvidenceRequirement {
            requirement_id: digest(10),
            allowed_source_kinds: vec![EvidenceSourceKind::File],
            allowed_authority_classes: vec![EvidenceAuthorityClass::LocalObserved],
            forbidden_taint: TaintSet::from_labels([TaintLabel::SecretDerived]),
            required_permission_scope: Some(PermissionScopeId(digest(6))),
            minimum_observed_at_unix_ms: Some(900),
        };
        assert_eq!(requirement.accepts(&evidence(), 1_000), Ok(true));

        let mut tainted = evidence();
        tainted.taint_set = tainted
            .taint_set
            .union(TaintSet::from_labels([TaintLabel::SecretDerived]));
        assert_eq!(requirement.accepts(&tainted, 1_000), Ok(false));
        assert!(matches!(
            requirement.accepts(&evidence(), 1_051),
            Err(ContextValidationError::StaleEvidence(_))
        ));
    }

    #[test]
    fn ranking_cannot_satisfy_a_missing_requirement() {
        let requirement = EvidenceRequirement {
            requirement_id: digest(10),
            allowed_source_kinds: vec![EvidenceSourceKind::CanonicalLedger],
            allowed_authority_classes: vec![EvidenceAuthorityClass::CanonicalGolam],
            forbidden_taint: TaintSet::empty(),
            required_permission_scope: None,
            minimum_observed_at_unix_ms: None,
        };
        assert_eq!(
            missing_requirements(&[requirement], &[evidence()], 1_000).unwrap(),
            vec![digest(10)]
        );
    }

    #[test]
    fn capsule_sufficiency_and_ranking_refs_are_consistent() {
        let capsule = ContextCapsule {
            capsule_id: digest(20),
            intent_ref: digest(21),
            requirement_refs: vec![digest(10)],
            evidence_refs: vec![digest(1)],
            memory_refs: vec![],
            ranking_evidence: vec![RankingEvidence {
                evidence_ref: digest(1),
                bounded_score: 100,
            }],
            sufficiency_state: SufficiencyState::Sufficient,
            missing_requirements: vec![],
            projection_policy_ref: digest(22),
            created_at_unix_ms: 1_000,
        };
        assert_eq!(capsule.validate(), Ok(()));

        let mut invalid = capsule;
        invalid.ranking_evidence[0].evidence_ref = digest(99);
        assert_eq!(
            invalid.validate(),
            Err(ContextValidationError::RankingReferencesUnknownEvidence)
        );
    }
}
