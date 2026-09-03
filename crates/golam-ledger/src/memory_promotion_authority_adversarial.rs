#![forbid(unsafe_code)]

use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::authority::AuthorityLayout;
use golam_core::memory::{
    MemoryAuthorityClass, MemoryCandidate, MemoryCandidateId, MemoryScope, PromotionRequirement,
};
use golam_core::paths::RuntimeLayout;
use golam_core::taint::{TaintLabel, TaintSet};
use golam_core::tool_request::{BindingDigest, PrincipalId};
use golam_core::{EffectId, EffectTransitionId, EventId, SessionId};

use crate::approval_binding::{
    APPROVAL_ISSUE_ACTION, APPROVAL_MUTATION_RISK_CLASS, ApprovalStore, prepare_approval,
};
use crate::approval_revocation::{
    APPROVAL_REVOKE_ACTION, ApprovalRevocationStore, prepare_approval_revocation,
};
use crate::approvals::ApprovalScope;
use crate::authorization::{
    AppendAuthorizationDecision, AuthorizationAuditLog, AuthorizationDecisionEvidence,
    AuthorizationDecisionKind,
};
use crate::dispatch::encode_effect_dependencies;
use crate::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
use crate::memory_promotion_authority::{
    HumanPromotionRequest, MEMORY_PROMOTION_ACTION, MEMORY_PROMOTION_RISK_CLASS,
    MemoryPromotionAuthorityError, MemoryPromotionAuthorityValidator, promotion_resource,
};

static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(0);
static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_id() -> u128 {
    19_000_000 + u128::from(ID_COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn digest(value: u8) -> BindingDigest {
    BindingDigest::new([value; 32])
}

fn authority() -> (RuntimeLayout, AuthorityLayout) {
    let counter = TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after UNIX epoch")
        .as_nanos();
    let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
        "golam-memory-promotion-adversarial-{}-{nanos}-{counter}",
        std::process::id()
    )))
    .expect("test runtime layout must initialize");
    let authority =
        AuthorityLayout::initialize(&runtime).expect("test authority layout must initialize");
    (runtime, authority)
}

fn candidate(policy_ref: BindingDigest) -> MemoryCandidate {
    MemoryCandidate {
        candidate_id: MemoryCandidateId(digest(1)),
        scope: MemoryScope::Project,
        proposed_content_ref: digest(2),
        provenance_refs: vec![digest(3)],
        taint_set: TaintSet::from_labels([TaintLabel::UserTrusted]),
        authority_class: MemoryAuthorityClass::UserAttributed,
        created_by_principal: PrincipalId::new("owner:owner")
            .expect("test principal must be canonical"),
        created_at_unix_ms: 4,
        promotion_requirement: PromotionRequirement::AttributableHumanApproval {
            approval_policy_ref: policy_ref,
        },
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
    let dependencies =
        encode_effect_dependencies(&[]).expect("empty effect dependencies must encode");
    let mut store = EffectStore::open(authority).expect("effect store must open");
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
        .expect("test effect proposal must persist");
    store
        .compare_and_swap(CompareAndSwapEffect {
            transition_id: EffectTransitionId(next_id()),
            effect_id,
            expected_state: "proposed",
            next_state: "authorized",
            attempt_id: None,
            reason_code: Some("memory_promotion_adversarial_test"),
            evidence_ref: None,
            event_id: EventId(next_id()),
        })
        .expect("test effect must become authorized");
}

fn append_decision(
    authority: &AuthorityLayout,
    action: &str,
    resource: &str,
    decision: AuthorizationDecisionKind,
    reason_code: &str,
) -> [u8; 16] {
    AuthorizationAuditLog::open(authority)
        .expect("authorization log must open")
        .append(AppendAuthorizationDecision {
            principal: "owner:owner",
            action,
            resource,
            context: "scope=memory-promotion",
            evidence: AuthorizationDecisionEvidence::hard_guard_only("pass"),
            decision,
            reason_code,
        })
        .expect("test authorization decision must persist")
        .decision_id
}

fn append_allow(authority: &AuthorityLayout, action: &str, resource: &str) -> [u8; 16] {
    append_decision(
        authority,
        action,
        resource,
        AuthorizationDecisionKind::Allow,
        "memory_promotion_adversarial_allow",
    )
}

fn issue_once_approval(
    authority: &AuthorityLayout,
    effect_id: EffectId,
    resource: &str,
    taint_digest: [u8; 32],
) -> [u8; 16] {
    let approval = prepare_approval(
        "owner:owner",
        ApprovalScope::once(effect_id, MEMORY_PROMOTION_ACTION, resource)
            .expect("test approval scope must be valid"),
        MEMORY_PROMOTION_RISK_CLASS,
        taint_digest,
        "2026-09-03T00:00:00Z",
        None,
        1,
    )
    .expect("test approval must prepare");
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
        .expect("approval store must open")
        .issue(approval, decision, issue_effect)
        .expect("test approval must issue")
        .approval_id()
}

#[test]
fn revoked_human_approval_is_rejected_even_with_fresh_kernel_authorization() {
    let (runtime, authority) = authority();
    let policy_ref = digest(80);
    let candidate = candidate(policy_ref);
    let principal = PrincipalId::new("owner:owner").expect("test principal must be canonical");
    let resource = promotion_resource(candidate.scope, "human", policy_ref);
    let effect_id = EffectId(next_id());
    let taint_digest = *blake3::hash(
        &candidate
            .taint_set
            .canonical_bytes()
            .expect("candidate taint must encode"),
    )
    .as_bytes();
    let approval_id = issue_once_approval(&authority, effect_id, &resource, taint_digest);

    let revocation =
        prepare_approval_revocation(approval_id, "owner:owner", "2026-09-03T00:30:00Z")
            .expect("revocation must prepare");
    let revocation_effect = EffectId(next_id());
    create_authorized_effect(
        &authority,
        revocation_effect,
        APPROVAL_REVOKE_ACTION,
        revocation.resource(),
        APPROVAL_MUTATION_RISK_CLASS,
        revocation.intent_digest(),
        "owner:owner",
    );
    let revocation_decision =
        append_allow(&authority, APPROVAL_REVOKE_ACTION, revocation.resource());
    ApprovalRevocationStore::open(&authority)
        .expect("revocation store must open")
        .revoke(revocation, revocation_decision, revocation_effect)
        .expect("test approval must revoke");

    let fresh_decision = append_allow(&authority, MEMORY_PROMOTION_ACTION, &resource);
    let result = MemoryPromotionAuthorityValidator::open(&authority)
        .expect("promotion validator must open")
        .validate_human(HumanPromotionRequest {
            candidate: &candidate,
            initiating_principal: &principal,
            authorization_decision_id: fresh_decision,
            approval_id,
            effect_id,
            observed_at: "2026-09-03T01:00:00Z",
        });
    assert!(matches!(
        result,
        Err(MemoryPromotionAuthorityError::Approval(_))
    ));
    fs::remove_dir_all(runtime.root).expect("test runtime cleanup must succeed");
}

#[test]
fn current_kernel_deny_cannot_be_used_as_promotion_authority() {
    let (runtime, authority) = authority();
    let policy_ref = digest(90);
    let candidate = candidate(policy_ref);
    let principal = PrincipalId::new("owner:owner").expect("test principal must be canonical");
    let resource = promotion_resource(candidate.scope, "human", policy_ref);
    let denied = append_decision(
        &authority,
        MEMORY_PROMOTION_ACTION,
        &resource,
        AuthorizationDecisionKind::Deny,
        "memory_promotion_adversarial_deny",
    );

    let result = MemoryPromotionAuthorityValidator::open(&authority)
        .expect("promotion validator must open")
        .validate_human(HumanPromotionRequest {
            candidate: &candidate,
            initiating_principal: &principal,
            authorization_decision_id: denied,
            approval_id: [0; 16],
            effect_id: EffectId(1),
            observed_at: "2026-09-03T01:00:00Z",
        });
    assert!(matches!(
        result,
        Err(MemoryPromotionAuthorityError::AuthorityDecisionMismatch)
    ));
    fs::remove_dir_all(runtime.root).expect("test runtime cleanup must succeed");
}
