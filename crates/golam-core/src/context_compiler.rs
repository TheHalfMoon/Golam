#![forbid(unsafe_code)]

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use crate::CanonicalEncoder;
use crate::context_evidence::{
    ContextCapsule, ContextEvidence, ContextValidationError, EvidenceRequirement,
    EvidenceSourceKind, RankingEvidence, SufficiencyState, missing_requirements,
};
use crate::digest::sha256;
use crate::tool_request::BindingDigest;
use crate::CoreError;

const MAX_L0_REQUIREMENTS: usize = 128;
const MAX_L0_RETRIEVED_ITEMS: usize = 128;
const MAX_L0_REPLANS: u8 = 4;
const CONTEXT_CAPSULE_DOMAIN: &[u8] = b"golam:spec005:l0-context-capsule:v1";

/// A bounded, in-process L0 source route. The route is mechanism metadata and
/// never carries authority by itself.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum L0SourceRoute {
    UserSelectedArtifact,
    FileRead,
    InProcessSearch,
    Git,
    CanonicalEvidence,
    ManagedMemory,
}

impl L0SourceRoute {
    pub const fn source_kind(self) -> EvidenceSourceKind {
        match self {
            Self::UserSelectedArtifact => EvidenceSourceKind::UserSelectedArtifact,
            Self::FileRead | Self::InProcessSearch => EvidenceSourceKind::File,
            Self::Git => EvidenceSourceKind::GitObject,
            Self::CanonicalEvidence => EvidenceSourceKind::CanonicalLedger,
            Self::ManagedMemory => EvidenceSourceKind::ManagedMemory,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextCompilerPlan {
    pub intent_ref: BindingDigest,
    pub requirements: Vec<EvidenceRequirement>,
    pub allowed_routes: Vec<L0SourceRoute>,
    pub max_evidence_items: usize,
    pub max_replans: u8,
    pub projection_policy_ref: BindingDigest,
    pub created_at_unix_ms: u64,
}

impl ContextCompilerPlan {
    pub fn validate(&self) -> Result<(), ContextCompilerError> {
        if self.requirements.is_empty() || self.requirements.len() > MAX_L0_REQUIREMENTS {
            return Err(ContextCompilerError::InvalidBounds);
        }
        if self.max_evidence_items == 0 || self.max_evidence_items > MAX_L0_RETRIEVED_ITEMS {
            return Err(ContextCompilerError::InvalidBounds);
        }
        if self.max_replans > MAX_L0_REPLANS {
            return Err(ContextCompilerError::InvalidBounds);
        }
        if self.allowed_routes.is_empty()
            || self
                .allowed_routes
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(ContextCompilerError::InvalidRoutePolicy);
        }
        if self
            .requirements
            .windows(2)
            .any(|pair| pair[0].requirement_id >= pair[1].requirement_id)
        {
            return Err(ContextCompilerError::InvalidRequirementOrder);
        }
        for requirement in &self.requirements {
            requirement.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct L0RoutePlan {
    pub route: L0SourceRoute,
    pub requirement_refs: Vec<BindingDigest>,
}

/// Routes requirements only to the explicitly admitted L0 mechanisms that can
/// emit the required source kind. It never performs I/O and never widens the
/// caller's route policy.
pub fn route_l0_requirements(
    plan: &ContextCompilerPlan,
) -> Result<Vec<L0RoutePlan>, ContextCompilerError> {
    plan.validate()?;
    let mut output = Vec::new();
    for route in &plan.allowed_routes {
        let source_kind = route.source_kind();
        let mut refs = Vec::new();
        for requirement in &plan.requirements {
            if requirement
                .allowed_source_kinds
                .binary_search(&source_kind)
                .is_ok()
            {
                refs.push(requirement.requirement_id);
            }
        }
        if !refs.is_empty() {
            output.push(L0RoutePlan {
                route: *route,
                requirement_refs: refs,
            });
        }
    }
    Ok(output)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct L0RetrievedEvidence {
    pub route: L0SourceRoute,
    pub evidence: ContextEvidence,
    pub bounded_score: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RejectedEvidenceReason {
    RouteNotAllowed,
    Stale,
    RequirementFilter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RejectedEvidence {
    pub evidence_ref: BindingDigest,
    pub reason: RejectedEvidenceReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedContextReplan {
    pub next_attempt: u8,
    pub remaining_attempts_after_next: u8,
    pub missing_requirements: Vec<BindingDigest>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledL0Context {
    pub capsule: ContextCapsule,
    pub rejected_evidence: Vec<RejectedEvidence>,
    pub replan: Option<BoundedContextReplan>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextCompilerError {
    Context(ContextValidationError),
    Core(CoreError),
    InvalidBounds,
    InvalidRoutePolicy,
    InvalidRequirementOrder,
    DuplicateEvidence(BindingDigest),
    RouteSourceMismatch(BindingDigest),
    ReplanLimitExceeded,
}

impl fmt::Display for ContextCompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Context(error) => write!(f, "context compiler validation failed: {error}"),
            Self::Core(error) => write!(f, "context compiler canonical encoding failed: {error}"),
            Self::InvalidBounds => f.write_str("context compiler bounds are invalid"),
            Self::InvalidRoutePolicy => {
                f.write_str("L0 source routes must be non-empty, sorted and unique")
            }
            Self::InvalidRequirementOrder => {
                f.write_str("context requirements must be sorted and unique by requirement id")
            }
            Self::DuplicateEvidence(_) => {
                f.write_str("retrieval returned duplicate context evidence identity")
            }
            Self::RouteSourceMismatch(_) => {
                f.write_str("retrieval route does not match the evidence source kind")
            }
            Self::ReplanLimitExceeded => {
                f.write_str("context compiler replan attempt exceeds the bounded plan")
            }
        }
    }
}

impl std::error::Error for ContextCompilerError {}

impl From<ContextValidationError> for ContextCompilerError {
    fn from(value: ContextValidationError) -> Self {
        Self::Context(value)
    }
}

impl From<CoreError> for ContextCompilerError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

/// Compiles one bounded L0 context attempt. Retrieval is performed by the
/// already-governed source boundaries; this function routes, validates,
/// filters, ranks, evaluates sufficiency and emits an explicit bounded replan
/// request without recursively retrieving anything itself.
pub fn compile_l0_context(
    plan: &ContextCompilerPlan,
    retrieved: &[L0RetrievedEvidence],
    replan_attempt: u8,
    now_unix_ms: u64,
) -> Result<CompiledL0Context, ContextCompilerError> {
    plan.validate()?;
    if retrieved.len() > MAX_L0_RETRIEVED_ITEMS {
        return Err(ContextCompilerError::InvalidBounds);
    }
    if replan_attempt > plan.max_replans {
        return Err(ContextCompilerError::ReplanLimitExceeded);
    }

    let allowed_routes: BTreeSet<L0SourceRoute> = plan.allowed_routes.iter().copied().collect();
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::new();
    let mut rejected = Vec::new();

    for item in retrieved {
        let evidence_ref = item.evidence.evidence_id;
        if !seen.insert(evidence_ref) {
            return Err(ContextCompilerError::DuplicateEvidence(evidence_ref));
        }
        if !allowed_routes.contains(&item.route) {
            rejected.push(RejectedEvidence {
                evidence_ref,
                reason: RejectedEvidenceReason::RouteNotAllowed,
            });
            continue;
        }
        if item.route.source_kind() != item.evidence.source_kind {
            return Err(ContextCompilerError::RouteSourceMismatch(evidence_ref));
        }
        match item.evidence.validate(now_unix_ms) {
            Ok(()) => {}
            Err(ContextValidationError::StaleEvidence(_)) => {
                rejected.push(RejectedEvidence {
                    evidence_ref,
                    reason: RejectedEvidenceReason::Stale,
                });
                continue;
            }
            Err(error) => return Err(error.into()),
        }

        let mut accepted = false;
        for requirement in &plan.requirements {
            if requirement.accepts(&item.evidence, now_unix_ms)? {
                accepted = true;
                break;
            }
        }
        if !accepted {
            rejected.push(RejectedEvidence {
                evidence_ref,
                reason: RejectedEvidenceReason::RequirementFilter,
            });
            continue;
        }
        candidates.push(item);
    }

    candidates.sort_by(|left, right| {
        right
            .bounded_score
            .cmp(&left.bounded_score)
            .then_with(|| left.evidence.evidence_id.cmp(&right.evidence.evidence_id))
    });

    let mut selected = BTreeSet::new();
    for requirement in &plan.requirements {
        if selected.len() >= plan.max_evidence_items {
            break;
        }
        for candidate in &candidates {
            if selected.contains(&candidate.evidence.evidence_id) {
                continue;
            }
            if requirement.accepts(&candidate.evidence, now_unix_ms)? {
                selected.insert(candidate.evidence.evidence_id);
                break;
            }
        }
    }
    for candidate in &candidates {
        if selected.len() >= plan.max_evidence_items {
            break;
        }
        selected.insert(candidate.evidence.evidence_id);
    }

    let candidate_by_id: BTreeMap<BindingDigest, &L0RetrievedEvidence> = candidates
        .iter()
        .map(|candidate| (candidate.evidence.evidence_id, *candidate))
        .collect();
    let selected_evidence: Vec<ContextEvidence> = selected
        .iter()
        .filter_map(|evidence_ref| candidate_by_id.get(evidence_ref))
        .map(|candidate| candidate.evidence.clone())
        .collect();

    let missing = missing_requirements(&plan.requirements, &selected_evidence, now_unix_ms)?;
    let sufficiency_state = if missing.is_empty() {
        SufficiencyState::Sufficient
    } else {
        SufficiencyState::Insufficient
    };

    let requirement_refs: Vec<BindingDigest> = plan
        .requirements
        .iter()
        .map(|requirement| requirement.requirement_id)
        .collect();
    let evidence_refs: Vec<BindingDigest> = selected.iter().copied().collect();
    let memory_refs: Vec<BindingDigest> = selected_evidence
        .iter()
        .filter(|evidence| evidence.source_kind == EvidenceSourceKind::ManagedMemory)
        .map(|evidence| evidence.evidence_id)
        .collect();
    let mut ranking_evidence: Vec<RankingEvidence> = evidence_refs
        .iter()
        .filter_map(|evidence_ref| candidate_by_id.get(evidence_ref))
        .map(|candidate| RankingEvidence {
            evidence_ref: candidate.evidence.evidence_id,
            bounded_score: candidate.bounded_score,
        })
        .collect();
    ranking_evidence.sort_unstable();

    rejected.sort_by_key(|item| item.evidence_ref);
    let capsule_id = capsule_digest(
        plan,
        &requirement_refs,
        &evidence_refs,
        &memory_refs,
        &ranking_evidence,
        sufficiency_state,
        &missing,
    )?;
    let capsule = ContextCapsule {
        capsule_id,
        intent_ref: plan.intent_ref,
        requirement_refs,
        evidence_refs,
        memory_refs,
        ranking_evidence,
        sufficiency_state,
        missing_requirements: missing.clone(),
        projection_policy_ref: plan.projection_policy_ref,
        created_at_unix_ms: plan.created_at_unix_ms,
    };
    capsule.validate()?;

    let replan = if !missing.is_empty() && replan_attempt < plan.max_replans {
        let next_attempt = replan_attempt + 1;
        Some(BoundedContextReplan {
            next_attempt,
            remaining_attempts_after_next: plan.max_replans - next_attempt,
            missing_requirements: missing,
        })
    } else {
        None
    };

    Ok(CompiledL0Context {
        capsule,
        rejected_evidence: rejected,
        replan,
    })
}

fn capsule_digest(
    plan: &ContextCompilerPlan,
    requirement_refs: &[BindingDigest],
    evidence_refs: &[BindingDigest],
    memory_refs: &[BindingDigest],
    ranking_evidence: &[RankingEvidence],
    sufficiency_state: SufficiencyState,
    missing_requirements: &[BindingDigest],
) -> Result<BindingDigest, ContextCompilerError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(CONTEXT_CAPSULE_DOMAIN)?;
    encoder.push_bytes(&plan.intent_ref.bytes())?;
    push_digest_slice(&mut encoder, requirement_refs)?;
    push_digest_slice(&mut encoder, evidence_refs)?;
    push_digest_slice(&mut encoder, memory_refs)?;
    encoder.push_u64(ranking_evidence.len() as u64);
    for ranking in ranking_evidence {
        encoder.push_bytes(&ranking.evidence_ref.bytes())?;
        encoder.push_bytes(&ranking.bounded_score.to_be_bytes())?;
    }
    encoder.push_u8(match sufficiency_state {
        SufficiencyState::Sufficient => 1,
        SufficiencyState::Insufficient => 2,
    });
    push_digest_slice(&mut encoder, missing_requirements)?;
    encoder.push_bytes(&plan.projection_policy_ref.bytes())?;
    encoder.push_u64(plan.created_at_unix_ms);
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn push_digest_slice(
    encoder: &mut CanonicalEncoder,
    values: &[BindingDigest],
) -> Result<(), CoreError> {
    encoder.push_u64(values.len() as u64);
    for value in values {
        encoder.push_bytes(&value.bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_evidence::{
        EvidenceAuthorityClass, EvidenceSourceId, FreshnessPolicy, PermissionScopeId,
    };
    use crate::taint::{TaintLabel, TaintSet};

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn requirement(
        id: u8,
        source_kind: EvidenceSourceKind,
        authority: EvidenceAuthorityClass,
    ) -> EvidenceRequirement {
        EvidenceRequirement {
            requirement_id: digest(id),
            allowed_source_kinds: vec![source_kind],
            allowed_authority_classes: vec![authority],
            forbidden_taint: TaintSet::from_labels([TaintLabel::SecretDerived]),
            required_permission_scope: Some(PermissionScopeId(digest(90))),
            minimum_observed_at_unix_ms: Some(900),
        }
    }

    fn evidence(
        id: u8,
        source_kind: EvidenceSourceKind,
        authority: EvidenceAuthorityClass,
    ) -> ContextEvidence {
        ContextEvidence {
            evidence_id: digest(id),
            source_id: EvidenceSourceId(digest(id.wrapping_add(20))),
            source_kind,
            source_version_or_observation: digest(id.wrapping_add(30)),
            content_ref: digest(id.wrapping_add(40)),
            content_digest: digest(id.wrapping_add(50)),
            authority_class: authority,
            taint_set: TaintSet::from_labels([TaintLabel::LocalTrusted]),
            permission_scope: PermissionScopeId(digest(90)),
            freshness_policy: FreshnessPolicy::MaxAgeMs(500),
            observed_at_unix_ms: 950,
            supersedes_or_conflicts_with: vec![],
        }
    }

    fn plan(requirements: Vec<EvidenceRequirement>) -> ContextCompilerPlan {
        ContextCompilerPlan {
            intent_ref: digest(1),
            requirements,
            allowed_routes: vec![
                L0SourceRoute::UserSelectedArtifact,
                L0SourceRoute::FileRead,
                L0SourceRoute::InProcessSearch,
                L0SourceRoute::Git,
                L0SourceRoute::CanonicalEvidence,
                L0SourceRoute::ManagedMemory,
            ],
            max_evidence_items: 16,
            max_replans: 2,
            projection_policy_ref: digest(2),
            created_at_unix_ms: 1_000,
        }
    }

    #[test]
    fn routes_every_required_l0_source_without_process_or_network_route() {
        let value = plan(vec![
            requirement(
                10,
                EvidenceSourceKind::UserSelectedArtifact,
                EvidenceAuthorityClass::UserAttributed,
            ),
            requirement(
                11,
                EvidenceSourceKind::File,
                EvidenceAuthorityClass::LocalObserved,
            ),
            requirement(
                12,
                EvidenceSourceKind::GitObject,
                EvidenceAuthorityClass::LocalObserved,
            ),
            requirement(
                13,
                EvidenceSourceKind::CanonicalLedger,
                EvidenceAuthorityClass::CanonicalGolam,
            ),
            requirement(
                14,
                EvidenceSourceKind::ManagedMemory,
                EvidenceAuthorityClass::CanonicalGolam,
            ),
        ]);
        let routed = route_l0_requirements(&value).unwrap();
        assert_eq!(routed.len(), 6);
        assert_eq!(routed[1].route, L0SourceRoute::FileRead);
        assert_eq!(routed[2].route, L0SourceRoute::InProcessSearch);
        assert_eq!(routed[3].route, L0SourceRoute::Git);
    }

    #[test]
    fn high_retrieval_score_cannot_raise_authority_or_clear_taint() {
        let value = plan(vec![requirement(
            10,
            EvidenceSourceKind::CanonicalLedger,
            EvidenceAuthorityClass::CanonicalGolam,
        )]);
        let mut untrusted = evidence(
            20,
            EvidenceSourceKind::CanonicalLedger,
            EvidenceAuthorityClass::UntrustedContent,
        );
        untrusted.taint_set = TaintSet::from_labels([TaintLabel::SecretDerived]);
        let trusted = evidence(
            21,
            EvidenceSourceKind::CanonicalLedger,
            EvidenceAuthorityClass::CanonicalGolam,
        );
        let output = compile_l0_context(
            &value,
            &[
                L0RetrievedEvidence {
                    route: L0SourceRoute::CanonicalEvidence,
                    evidence: untrusted,
                    bounded_score: i32::MAX,
                },
                L0RetrievedEvidence {
                    route: L0SourceRoute::CanonicalEvidence,
                    evidence: trusted,
                    bounded_score: -1,
                },
            ],
            0,
            1_000,
        )
        .unwrap();
        assert_eq!(output.capsule.sufficiency_state, SufficiencyState::Sufficient);
        assert_eq!(output.capsule.evidence_refs, vec![digest(21)]);
        assert_eq!(
            output.rejected_evidence,
            vec![RejectedEvidence {
                evidence_ref: digest(20),
                reason: RejectedEvidenceReason::RequirementFilter,
            }]
        );
    }

    #[test]
    fn stale_evidence_surfaces_missing_requirement_and_bounded_replan() {
        let value = plan(vec![requirement(
            10,
            EvidenceSourceKind::File,
            EvidenceAuthorityClass::LocalObserved,
        )]);
        let mut stale = evidence(
            20,
            EvidenceSourceKind::File,
            EvidenceAuthorityClass::LocalObserved,
        );
        stale.observed_at_unix_ms = 100;
        stale.freshness_policy = FreshnessPolicy::MaxAgeMs(10);
        let output = compile_l0_context(
            &value,
            &[L0RetrievedEvidence {
                route: L0SourceRoute::FileRead,
                evidence: stale,
                bounded_score: 99,
            }],
            0,
            1_000,
        )
        .unwrap();
        assert_eq!(output.capsule.sufficiency_state, SufficiencyState::Insufficient);
        assert_eq!(output.capsule.missing_requirements, vec![digest(10)]);
        assert_eq!(output.replan.as_ref().unwrap().next_attempt, 1);
        assert_eq!(output.replan.as_ref().unwrap().remaining_attempts_after_next, 1);
        assert_eq!(
            output.rejected_evidence[0].reason,
            RejectedEvidenceReason::Stale
        );

        let exhausted = compile_l0_context(&value, &[], 2, 1_000).unwrap();
        assert!(exhausted.replan.is_none());
        assert_eq!(
            compile_l0_context(&value, &[], 3, 1_000),
            Err(ContextCompilerError::ReplanLimitExceeded)
        );
    }

    #[test]
    fn route_source_mismatch_and_duplicate_evidence_fail_closed() {
        let value = plan(vec![requirement(
            10,
            EvidenceSourceKind::GitObject,
            EvidenceAuthorityClass::LocalObserved,
        )]);
        let item = L0RetrievedEvidence {
            route: L0SourceRoute::Git,
            evidence: evidence(
                20,
                EvidenceSourceKind::File,
                EvidenceAuthorityClass::LocalObserved,
            ),
            bounded_score: 1,
        };
        assert_eq!(
            compile_l0_context(&value, &[item], 0, 1_000),
            Err(ContextCompilerError::RouteSourceMismatch(digest(20)))
        );

        let valid = L0RetrievedEvidence {
            route: L0SourceRoute::Git,
            evidence: evidence(
                21,
                EvidenceSourceKind::GitObject,
                EvidenceAuthorityClass::LocalObserved,
            ),
            bounded_score: 1,
        };
        assert_eq!(
            compile_l0_context(&value, &[valid.clone(), valid], 0, 1_000),
            Err(ContextCompilerError::DuplicateEvidence(digest(21)))
        );
    }

    #[test]
    fn selected_capsule_is_deterministic_and_memory_refs_are_explicit() {
        let value = plan(vec![
            requirement(
                10,
                EvidenceSourceKind::File,
                EvidenceAuthorityClass::LocalObserved,
            ),
            requirement(
                11,
                EvidenceSourceKind::ManagedMemory,
                EvidenceAuthorityClass::CanonicalGolam,
            ),
        ]);
        let file = L0RetrievedEvidence {
            route: L0SourceRoute::InProcessSearch,
            evidence: evidence(
                20,
                EvidenceSourceKind::File,
                EvidenceAuthorityClass::LocalObserved,
            ),
            bounded_score: 5,
        };
        let memory = L0RetrievedEvidence {
            route: L0SourceRoute::ManagedMemory,
            evidence: evidence(
                21,
                EvidenceSourceKind::ManagedMemory,
                EvidenceAuthorityClass::CanonicalGolam,
            ),
            bounded_score: 4,
        };
        let first = compile_l0_context(&value, &[memory.clone(), file.clone()], 0, 1_000).unwrap();
        let second = compile_l0_context(&value, &[file, memory], 0, 1_000).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.capsule.memory_refs, vec![digest(21)]);
        assert_eq!(first.capsule.sufficiency_state, SufficiencyState::Sufficient);
        assert!(first.replan.is_none());
    }
}
