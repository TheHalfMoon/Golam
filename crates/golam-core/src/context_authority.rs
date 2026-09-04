#![forbid(unsafe_code)]

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use crate::context_compiler::{
    CompiledL0Context, ContextCompilerError, ContextCompilerPlan, L0RetrievedEvidence,
    L0SourceRoute, compile_l0_context,
};
use crate::context_evidence::{
    ContextEvidence, EvidenceAuthorityClass, EvidenceSourceKind, PermissionScopeId,
};
use crate::digest::sha256;
use crate::tool_request::BindingDigest;
use crate::{CanonicalEncoder, CoreError};

const MAX_RECONCILIATION_ITEMS: usize = 128;
const LIVE_MEMORY_CONFLICT_DOMAIN: &[u8] = b"golam:spec005:live-memory-conflict:v1";

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiveAuthorityResolution {
    LiveRepositoryOrFilesystem,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveMemoryConflictEvidence {
    pub conflict_id: BindingDigest,
    pub subject_ref: BindingDigest,
    pub permission_scope: PermissionScopeId,
    pub live_evidence_ref: BindingDigest,
    pub memory_evidence_ref: BindingDigest,
    pub live_source_kind: EvidenceSourceKind,
    pub live_source_version_or_observation: BindingDigest,
    pub memory_source_version_or_observation: BindingDigest,
    pub live_content_digest: BindingDigest,
    pub memory_content_digest: BindingDigest,
    pub live_observed_at_unix_ms: u64,
    pub memory_observed_at_unix_ms: u64,
    pub resolution: LiveAuthorityResolution,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityReconciledL0Context {
    pub compiled: CompiledL0Context,
    pub conflicts: Vec<LiveMemoryConflictEvidence>,
    pub suppressed_memory_refs: Vec<BindingDigest>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextAuthorityError {
    Compiler(ContextCompilerError),
    Core(CoreError),
    TooManyItems,
    ConflictingLiveAuthority(BindingDigest),
    LiveObservationPredatesMemory {
        live_evidence_ref: BindingDigest,
        memory_evidence_ref: BindingDigest,
    },
}

impl fmt::Display for ContextAuthorityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compiler(error) => write!(f, "context authority reconciliation failed: {error}"),
            Self::Core(error) => write!(f, "context authority evidence encoding failed: {error}"),
            Self::TooManyItems => {
                f.write_str("context authority reconciliation item bound exceeded")
            }
            Self::ConflictingLiveAuthority(subject_ref) => write!(
                f,
                "conflicting live repository/filesystem observations exist for subject {subject_ref:?}"
            ),
            Self::LiveObservationPredatesMemory {
                live_evidence_ref,
                memory_evidence_ref,
            } => write!(
                f,
                "live evidence {live_evidence_ref:?} predates conflicting memory evidence {memory_evidence_ref:?}"
            ),
        }
    }
}

impl std::error::Error for ContextAuthorityError {}

impl From<ContextCompilerError> for ContextAuthorityError {
    fn from(value: ContextCompilerError) -> Self {
        Self::Compiler(value)
    }
}

impl From<CoreError> for ContextAuthorityError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

/// Reconciles managed-memory claims against fresh repository/filesystem
/// observations before L0 ranking. Conflicting memory is suppressed before
/// score-based selection and the displacement is returned as deterministic
/// conflict evidence. Ambiguous live state fails closed.
pub fn compile_l0_context_with_live_precedence(
    plan: &ContextCompilerPlan,
    retrieved: &[L0RetrievedEvidence],
    replan_attempt: u8,
    now_unix_ms: u64,
) -> Result<AuthorityReconciledL0Context, ContextAuthorityError> {
    let reconciliation = reconcile_live_authority(retrieved, now_unix_ms)?;
    let compiled = compile_l0_context(
        plan,
        &reconciliation.filtered,
        replan_attempt,
        now_unix_ms,
    )?;
    Ok(AuthorityReconciledL0Context {
        compiled,
        conflicts: reconciliation.conflicts,
        suppressed_memory_refs: reconciliation.suppressed_memory_refs,
    })
}

struct ReconciliationResult {
    filtered: Vec<L0RetrievedEvidence>,
    conflicts: Vec<LiveMemoryConflictEvidence>,
    suppressed_memory_refs: Vec<BindingDigest>,
}

fn reconcile_live_authority(
    retrieved: &[L0RetrievedEvidence],
    now_unix_ms: u64,
) -> Result<ReconciliationResult, ContextAuthorityError> {
    if retrieved.len() > MAX_RECONCILIATION_ITEMS {
        return Err(ContextAuthorityError::TooManyItems);
    }

    let mut live_by_subject: BTreeMap<
        (BindingDigest, PermissionScopeId),
        &L0RetrievedEvidence,
    > = BTreeMap::new();
    for candidate in retrieved.iter().filter(|candidate| is_live(candidate)) {
        candidate
            .evidence
            .validate(now_unix_ms)
            .map_err(ContextCompilerError::from)?;
        let key = (
            candidate.evidence.content_ref,
            candidate.evidence.permission_scope,
        );
        match live_by_subject.get(&key).copied() {
            None => {
                live_by_subject.insert(key, candidate);
            }
            Some(existing)
                if existing.evidence.content_digest != candidate.evidence.content_digest =>
            {
                return Err(ContextAuthorityError::ConflictingLiveAuthority(key.0));
            }
            Some(existing) => {
                let candidate_order = (
                    candidate.evidence.observed_at_unix_ms,
                    candidate.evidence.evidence_id,
                );
                let existing_order = (
                    existing.evidence.observed_at_unix_ms,
                    existing.evidence.evidence_id,
                );
                if candidate_order > existing_order {
                    live_by_subject.insert(key, candidate);
                }
            }
        }
    }

    let mut suppressed = BTreeSet::new();
    let mut conflicts = Vec::new();
    for memory in retrieved.iter().filter(|candidate| is_managed_memory(candidate)) {
        let key = (memory.evidence.content_ref, memory.evidence.permission_scope);
        let Some(live) = live_by_subject.get(&key).copied() else {
            continue;
        };
        if live.evidence.content_digest == memory.evidence.content_digest {
            continue;
        }
        if live.evidence.observed_at_unix_ms < memory.evidence.observed_at_unix_ms {
            return Err(ContextAuthorityError::LiveObservationPredatesMemory {
                live_evidence_ref: live.evidence.evidence_id,
                memory_evidence_ref: memory.evidence.evidence_id,
            });
        }
        suppressed.insert(memory.evidence.evidence_id);
        conflicts.push(conflict_evidence(&live.evidence, &memory.evidence)?);
    }

    conflicts.sort_by_key(|conflict| conflict.conflict_id);
    let suppressed_memory_refs = suppressed.iter().copied().collect();
    let filtered = retrieved
        .iter()
        .filter(|candidate| !suppressed.contains(&candidate.evidence.evidence_id))
        .cloned()
        .collect();
    Ok(ReconciliationResult {
        filtered,
        conflicts,
        suppressed_memory_refs,
    })
}

fn is_live(candidate: &L0RetrievedEvidence) -> bool {
    matches!(
        (candidate.route, candidate.evidence.source_kind),
        (L0SourceRoute::FileRead, EvidenceSourceKind::File)
            | (L0SourceRoute::Git, EvidenceSourceKind::GitObject)
    ) && candidate.evidence.authority_class == EvidenceAuthorityClass::LocalObserved
}

fn is_managed_memory(candidate: &L0RetrievedEvidence) -> bool {
    candidate.route == L0SourceRoute::ManagedMemory
        && candidate.evidence.source_kind == EvidenceSourceKind::ManagedMemory
        && candidate.evidence.authority_class == EvidenceAuthorityClass::CanonicalGolam
}

fn conflict_evidence(
    live: &ContextEvidence,
    memory: &ContextEvidence,
) -> Result<LiveMemoryConflictEvidence, ContextAuthorityError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(LIVE_MEMORY_CONFLICT_DOMAIN)?;
    encoder.push_bytes(&live.content_ref.bytes())?;
    encoder.push_bytes(&live.permission_scope.0.bytes())?;
    encoder.push_bytes(&live.evidence_id.bytes())?;
    encoder.push_bytes(&memory.evidence_id.bytes())?;
    encoder.push_u8(source_kind_code(live.source_kind));
    encoder.push_bytes(&live.source_version_or_observation.bytes())?;
    encoder.push_bytes(&memory.source_version_or_observation.bytes())?;
    encoder.push_bytes(&live.content_digest.bytes())?;
    encoder.push_bytes(&memory.content_digest.bytes())?;
    encoder.push_u64(live.observed_at_unix_ms);
    encoder.push_u64(memory.observed_at_unix_ms);
    encoder.push_u8(1);
    let conflict_id = BindingDigest::new(sha256(&encoder.finish()));

    Ok(LiveMemoryConflictEvidence {
        conflict_id,
        subject_ref: live.content_ref,
        permission_scope: live.permission_scope,
        live_evidence_ref: live.evidence_id,
        memory_evidence_ref: memory.evidence_id,
        live_source_kind: live.source_kind,
        live_source_version_or_observation: live.source_version_or_observation,
        memory_source_version_or_observation: memory.source_version_or_observation,
        live_content_digest: live.content_digest,
        memory_content_digest: memory.content_digest,
        live_observed_at_unix_ms: live.observed_at_unix_ms,
        memory_observed_at_unix_ms: memory.observed_at_unix_ms,
        resolution: LiveAuthorityResolution::LiveRepositoryOrFilesystem,
    })
}

const fn source_kind_code(kind: EvidenceSourceKind) -> u8 {
    match kind {
        EvidenceSourceKind::File => 1,
        EvidenceSourceKind::GitObject => 2,
        EvidenceSourceKind::UserSelectedArtifact => 3,
        EvidenceSourceKind::CanonicalLedger => 4,
        EvidenceSourceKind::ManagedMemory => 5,
        EvidenceSourceKind::ProtocolResource => 6,
        EvidenceSourceKind::ExternalDocument => 7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_evidence::{
        EvidenceRequirement, EvidenceSourceId, FreshnessPolicy, SufficiencyState,
    };
    use crate::taint::{TaintLabel, TaintSet};

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn evidence(
        id: u8,
        source_kind: EvidenceSourceKind,
        authority_class: EvidenceAuthorityClass,
        content_digest: u8,
        observed_at_unix_ms: u64,
    ) -> ContextEvidence {
        ContextEvidence {
            evidence_id: digest(id),
            source_id: EvidenceSourceId(digest(id.wrapping_add(20))),
            source_kind,
            source_version_or_observation: digest(id.wrapping_add(30)),
            content_ref: digest(9),
            content_digest: digest(content_digest),
            authority_class,
            taint_set: TaintSet::from_labels([TaintLabel::LocalTrusted]),
            permission_scope: PermissionScopeId(digest(90)),
            freshness_policy: FreshnessPolicy::MaxAgeMs(10_000),
            observed_at_unix_ms,
            supersedes_or_conflicts_with: vec![],
        }
    }

    fn plan(
        source_kinds: Vec<EvidenceSourceKind>,
        routes: Vec<L0SourceRoute>,
    ) -> ContextCompilerPlan {
        ContextCompilerPlan {
            intent_ref: digest(60),
            requirements: vec![EvidenceRequirement {
                requirement_id: digest(61),
                allowed_source_kinds: source_kinds,
                allowed_authority_classes: vec![
                    EvidenceAuthorityClass::LocalObserved,
                    EvidenceAuthorityClass::CanonicalGolam,
                ],
                forbidden_taint: TaintSet::from_labels([TaintLabel::SecretDerived]),
                required_permission_scope: Some(PermissionScopeId(digest(90))),
                minimum_observed_at_unix_ms: Some(800),
            }],
            allowed_routes: routes,
            max_evidence_items: 8,
            max_replans: 1,
            projection_policy_ref: digest(62),
            created_at_unix_ms: 1_000,
        }
    }

    #[test]
    fn live_file_out_ranks_conflicting_memory_even_with_lower_score() {
        let live = L0RetrievedEvidence {
            route: L0SourceRoute::FileRead,
            evidence: evidence(
                1,
                EvidenceSourceKind::File,
                EvidenceAuthorityClass::LocalObserved,
                11,
                1_000,
            ),
            bounded_score: -1_000,
        };
        let memory = L0RetrievedEvidence {
            route: L0SourceRoute::ManagedMemory,
            evidence: evidence(
                2,
                EvidenceSourceKind::ManagedMemory,
                EvidenceAuthorityClass::CanonicalGolam,
                12,
                900,
            ),
            bounded_score: 1_000,
        };
        let output = compile_l0_context_with_live_precedence(
            &plan(
                vec![EvidenceSourceKind::File, EvidenceSourceKind::ManagedMemory],
                vec![L0SourceRoute::FileRead, L0SourceRoute::ManagedMemory],
            ),
            &[live.clone(), memory.clone()],
            0,
            1_000,
        )
        .unwrap();

        assert_eq!(
            output.compiled.capsule.sufficiency_state,
            SufficiencyState::Sufficient
        );
        assert_eq!(
            output.compiled.capsule.evidence_refs,
            vec![live.evidence.evidence_id]
        );
        assert!(output.compiled.capsule.memory_refs.is_empty());
        assert_eq!(
            output.suppressed_memory_refs,
            vec![memory.evidence.evidence_id]
        );
        assert_eq!(output.conflicts.len(), 1);
        assert_eq!(output.conflicts[0].live_evidence_ref, live.evidence.evidence_id);
        assert_eq!(
            output.conflicts[0].memory_evidence_ref,
            memory.evidence.evidence_id
        );
    }

    #[test]
    fn live_git_out_ranks_conflicting_memory() {
        let live = L0RetrievedEvidence {
            route: L0SourceRoute::Git,
            evidence: evidence(
                3,
                EvidenceSourceKind::GitObject,
                EvidenceAuthorityClass::LocalObserved,
                21,
                1_000,
            ),
            bounded_score: 0,
        };
        let memory = L0RetrievedEvidence {
            route: L0SourceRoute::ManagedMemory,
            evidence: evidence(
                4,
                EvidenceSourceKind::ManagedMemory,
                EvidenceAuthorityClass::CanonicalGolam,
                22,
                850,
            ),
            bounded_score: 100,
        };
        let output = compile_l0_context_with_live_precedence(
            &plan(
                vec![EvidenceSourceKind::GitObject, EvidenceSourceKind::ManagedMemory],
                vec![L0SourceRoute::Git, L0SourceRoute::ManagedMemory],
            ),
            &[live.clone(), memory],
            0,
            1_000,
        )
        .unwrap();
        assert_eq!(
            output.compiled.capsule.evidence_refs,
            vec![live.evidence.evidence_id]
        );
        assert_eq!(
            output.conflicts[0].live_source_kind,
            EvidenceSourceKind::GitObject
        );
    }

    #[test]
    fn divergent_live_observations_fail_closed() {
        let first = L0RetrievedEvidence {
            route: L0SourceRoute::FileRead,
            evidence: evidence(
                7,
                EvidenceSourceKind::File,
                EvidenceAuthorityClass::LocalObserved,
                41,
                1_000,
            ),
            bounded_score: 0,
        };
        let second = L0RetrievedEvidence {
            route: L0SourceRoute::FileRead,
            evidence: evidence(
                8,
                EvidenceSourceKind::File,
                EvidenceAuthorityClass::LocalObserved,
                42,
                1_000,
            ),
            bounded_score: 0,
        };
        assert!(matches!(
            reconcile_live_authority(&[first, second], 1_000),
            Err(ContextAuthorityError::ConflictingLiveAuthority(_))
        ));
    }

    #[test]
    fn conflict_digest_binds_substituted_memory_state() {
        let live = evidence(
            11,
            EvidenceSourceKind::File,
            EvidenceAuthorityClass::LocalObserved,
            61,
            1_000,
        );
        let first_memory = evidence(
            12,
            EvidenceSourceKind::ManagedMemory,
            EvidenceAuthorityClass::CanonicalGolam,
            62,
            900,
        );
        let mut substituted_memory = first_memory.clone();
        substituted_memory.content_digest = digest(63);
        assert_ne!(
            conflict_evidence(&live, &first_memory).unwrap().conflict_id,
            conflict_evidence(&live, &substituted_memory)
                .unwrap()
                .conflict_id
        );
    }
}
