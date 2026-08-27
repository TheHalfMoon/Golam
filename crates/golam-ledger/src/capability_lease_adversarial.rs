#![forbid(unsafe_code)]

use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;
use golam_core::{EffectId, EffectTransitionId, EventId, SessionId};
use rusqlite::{Connection, TransactionBehavior, params};

use crate::authority_security_write::append_approval_snapshot;
use crate::authorization::{
    AppendAuthorizationDecision, AuthorizationAuditLog, AuthorizationDecisionEvidence,
    AuthorizationDecisionKind, StoredAuthorizationDecision,
};
use crate::capability_lease_mutation::{
    CAPABILITY_LEASE_ISSUE_ACTION, CAPABILITY_LEASE_MUTATION_RISK_CLASS,
    CAPABILITY_LEASE_REVOKE_ACTION, CapabilityLeaseBinding, CapabilityLeaseMutationError,
    CapabilityLeaseRecord, CapabilityLeaseStore, PreparedCapabilityLeaseIssue,
    prepare_capability_lease_issue, prepare_capability_lease_revocation,
};
use crate::capability_lease_runtime::load_capability_lease_runtime_chain;
use crate::dispatch::encode_effect_dependencies;
use crate::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};

static N: AtomicU64 = AtomicU64::new(0);

fn authority(label: &str) -> (RuntimeLayout, AuthorityLayout) {
    let n = N.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
        "golam-capability-lease-adversarial-{label}-{}-{t}-{n}",
        std::process::id()
    )))
    .unwrap();
    let authority = AuthorityLayout::initialize(&runtime).unwrap();
    (runtime, authority)
}

fn authorize_effect(
    authority: &AuthorityLayout,
    effect_id: EffectId,
    action: &str,
    resource: &str,
    payload_hash: [u8; 32],
    id_base: u128,
) {
    let dependencies = encode_effect_dependencies(&[]).unwrap();
    let mut effects = EffectStore::open(authority).unwrap();
    effects
        .propose(ProposeEffect {
            effect_id,
            session_id: SessionId(1),
            requested_by: "owner:owner",
            action,
            resource,
            risk_class: CAPABILITY_LEASE_MUTATION_RISK_CLASS,
            execution_semantics: "at_most_once",
            idempotency_key: None,
            preconditions: b"[]",
            dependencies: &dependencies,
            payload_hash,
            proposed_event_id: EventId(id_base),
            transition_id: EffectTransitionId(id_base + 1),
        })
        .unwrap();
    effects
        .compare_and_swap(CompareAndSwapEffect {
            transition_id: EffectTransitionId(id_base + 2),
            effect_id,
            expected_state: "proposed",
            next_state: "authorized",
            attempt_id: None,
            reason_code: Some("lease_mutation_approved"),
            evidence_ref: None,
            event_id: EventId(id_base + 3),
        })
        .unwrap();
}

fn append_allow(
    authority: &AuthorityLayout,
    action: &str,
    resource: &str,
) -> StoredAuthorizationDecision {
    let mut log = AuthorizationAuditLog::open(authority).unwrap();
    log.append(AppendAuthorizationDecision {
        principal: "owner:owner",
        action,
        resource,
        context: "scope=local-owner",
        evidence: AuthorizationDecisionEvidence::hard_guard_only("pass"),
        decision: AuthorizationDecisionKind::Allow,
        reason_code: "lease_adversarial_current_authority",
    })
    .unwrap()
}

fn seed_approval(
    authority: &AuthorityLayout,
    approval_id: [u8; 16],
    effect_id: EffectId,
    action: &str,
    resource: &str,
    parent_decision_id: [u8; 16],
) {
    let mut connection = Connection::open(authority.authority_db_path()).unwrap();
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )
        .unwrap();
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    transaction
        .execute(
            "INSERT INTO approvals (approval_id, class, approver_principal, scope_digest, action_scope, resource_scope, effect_id, session_id, risk_class, taint_digest, parent_decision_id, issued_at, expires_at, max_uses, revoked_at) VALUES (?1, 'ONCE', 'owner:owner', ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8, '2026-08-27T00:00:00Z', NULL, 1, NULL)",
            params![
                &approval_id[..],
                &[0_u8; 32][..],
                action.as_bytes(),
                resource.as_bytes(),
                &effect_id.0.to_be_bytes()[..],
                CAPABILITY_LEASE_MUTATION_RISK_CLASS,
                &[0_u8; 32][..],
                &parent_decision_id[..],
            ],
        )
        .unwrap();
    append_approval_snapshot(&transaction, &approval_id).unwrap();
    crate::authority_security_v2::verify(&transaction).unwrap();
    transaction.commit().unwrap();
}

fn authorize_mutation(
    authority: &AuthorityLayout,
    effect_id: EffectId,
    approval_id: [u8; 16],
    action: &str,
    resource: &str,
    payload_hash: [u8; 32],
    id_base: u128,
) -> StoredAuthorizationDecision {
    authorize_effect(authority, effect_id, action, resource, payload_hash, id_base);
    let decision = append_allow(authority, action, resource);
    seed_approval(
        authority,
        approval_id,
        effect_id,
        action,
        resource,
        decision.decision_id,
    );
    decision
}

fn binding(record: &CapabilityLeaseRecord) -> CapabilityLeaseBinding {
    CapabilityLeaseBinding::new(record.lease_id, record.generation, record.authority_digest)
}

fn basic_issue(
    principal: &str,
    parent: Option<CapabilityLeaseBinding>,
    actions: &[&str],
    resources: &[&str],
    context: &[&str],
    not_before: Option<&str>,
    expires_at: Option<&str>,
) -> PreparedCapabilityLeaseIssue {
    prepare_capability_lease_issue(
        principal,
        parent,
        &actions.iter().map(|value| (*value).to_owned()).collect::<Vec<_>>(),
        &resources
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>(),
        &context
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>(),
        not_before,
        expires_at,
    )
    .unwrap()
}

fn issue_authorized(
    authority: &AuthorityLayout,
    prepared: PreparedCapabilityLeaseIssue,
    effect_number: u128,
) -> CapabilityLeaseRecord {
    let effect_id = EffectId(effect_number);
    let approval_id = effect_number.to_be_bytes();
    let decision = authorize_mutation(
        authority,
        effect_id,
        approval_id,
        CAPABILITY_LEASE_ISSUE_ACTION,
        prepared.resource(),
        prepared.intent_digest(),
        effect_number + 10_000,
    );
    CapabilityLeaseStore::open(authority)
        .unwrap()
        .issue(prepared, decision.decision_id, approval_id, effect_id)
        .unwrap()
}

#[test]
fn property_child_authority_never_widens_parent_scope_or_expiry() {
    let (runtime, authority) = authority("widening");
    let parent = issue_authorized(
        &authority,
        basic_issue(
            "client:9:alice",
            None,
            &["session.read"],
            &["session:1"],
            &["local-session:1"],
            Some("2026-08-27T00:00:00Z"),
            Some("2026-08-28T00:00:00Z"),
        ),
        1_000,
    );

    let cases = [
        (
            vec!["session.read", "session.create"],
            vec!["session:1"],
            vec!["local-session:1"],
            Some("2026-08-27T00:00:00Z"),
            Some("2026-08-28T00:00:00Z"),
            false,
        ),
        (
            vec!["session.read"],
            vec!["session:1", "session:2"],
            vec!["local-session:1"],
            Some("2026-08-27T00:00:00Z"),
            Some("2026-08-28T00:00:00Z"),
            false,
        ),
        (
            vec!["session.read"],
            vec!["session:1"],
            vec!["local-session:1", "local-owner"],
            Some("2026-08-27T00:00:00Z"),
            Some("2026-08-28T00:00:00Z"),
            false,
        ),
        (
            vec!["session.read"],
            vec!["session:1"],
            vec!["local-session:1"],
            Some("2026-08-26T23:59:59Z"),
            Some("2026-08-28T00:00:00Z"),
            true,
        ),
        (
            vec!["session.read"],
            vec!["session:1"],
            vec!["local-session:1"],
            Some("2026-08-27T00:00:00Z"),
            Some("2026-08-28T00:00:01Z"),
            true,
        ),
        (
            vec!["session.read"],
            vec!["session:1"],
            vec!["local-session:1"],
            Some("2026-08-27T00:00:00Z"),
            None,
            true,
        ),
    ];

    for (index, (actions, resources, context, not_before, expires_at, temporal)) in
        cases.into_iter().enumerate()
    {
        let prepared = basic_issue(
            "client:9:alice",
            Some(binding(&parent)),
            &actions,
            &resources,
            &context,
            not_before,
            expires_at,
        );
        let effect_number = 2_000 + index as u128;
        let effect_id = EffectId(effect_number);
        let approval_id = effect_number.to_be_bytes();
        let decision = authorize_mutation(
            &authority,
            effect_id,
            approval_id,
            CAPABILITY_LEASE_ISSUE_ACTION,
            prepared.resource(),
            prepared.intent_digest(),
            20_000 + index as u128 * 10,
        );
        let error = CapabilityLeaseStore::open(&authority)
            .unwrap()
            .issue(prepared, decision.decision_id, approval_id, effect_id)
            .unwrap_err();
        if temporal {
            assert!(matches!(
                error,
                CapabilityLeaseMutationError::ParentTemporalWidening
            ));
        } else {
            assert!(matches!(
                error,
                CapabilityLeaseMutationError::ParentScopeWidening
            ));
        }
    }

    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn adversarial_self_grant_fails_despite_exact_allow_effect_and_once_approval() {
    let (runtime, authority) = authority("self-grant");
    for (index, action) in ["session.read", "session.create", "effect.simulate"]
        .into_iter()
        .enumerate()
    {
        let prepared = basic_issue(
            "owner:owner",
            None,
            &[action],
            &["session:1"],
            &["local-owner"],
            None,
            None,
        );
        let effect_number = 30_000 + index as u128;
        let effect_id = EffectId(effect_number);
        let approval_id = effect_number.to_be_bytes();
        let decision = authorize_mutation(
            &authority,
            effect_id,
            approval_id,
            CAPABILITY_LEASE_ISSUE_ACTION,
            prepared.resource(),
            prepared.intent_digest(),
            31_000 + index as u128 * 10,
        );
        assert!(matches!(
            CapabilityLeaseStore::open(&authority).unwrap().issue(
                prepared,
                decision.decision_id,
                approval_id,
                effect_id,
            ),
            Err(CapabilityLeaseMutationError::SelfGrantForbidden)
        ));
    }
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn adversarial_stale_generation_cannot_revoke_current_lease() {
    let (runtime, authority) = authority("stale-generation");
    let issued = issue_authorized(
        &authority,
        basic_issue(
            "client:9:alice",
            None,
            &["session.read"],
            &["session:1"],
            &["local-session:1"],
            None,
            None,
        ),
        40_000,
    );
    let stale = CapabilityLeaseBinding::new(
        issued.lease_id,
        issued.generation + 1,
        issued.authority_digest,
    );
    let prepared = prepare_capability_lease_revocation(
        stale,
        "stale_generation_probe",
        "2026-08-27T12:00:00Z",
    )
    .unwrap();
    let effect_id = EffectId(41_000);
    let approval_id = 41_000_u128.to_be_bytes();
    let decision = authorize_mutation(
        &authority,
        effect_id,
        approval_id,
        CAPABILITY_LEASE_REVOKE_ACTION,
        prepared.resource(),
        prepared.intent_digest(),
        42_000,
    );
    assert!(matches!(
        CapabilityLeaseStore::open(&authority).unwrap().revoke(
            prepared,
            decision.decision_id,
            approval_id,
            effect_id,
        ),
        Err(CapabilityLeaseMutationError::LeaseEvidenceMismatch)
    ));

    let chain = load_capability_lease_runtime_chain(&authority, issued.lease_id).unwrap();
    assert_eq!(chain.len(), 1);
    assert!(!chain[0].revoked);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn adversarial_issue_and_revocation_replay_are_fail_closed_and_monotonic() {
    let (runtime, authority) = authority("replay");
    let prepared = basic_issue(
        "client:9:alice",
        None,
        &["session.read"],
        &["session:1"],
        &["local-session:1"],
        Some("2026-08-27T00:00:00Z"),
        Some("2026-08-28T00:00:00Z"),
    );
    let issue_effect = EffectId(50_000);
    let issue_approval = 50_000_u128.to_be_bytes();
    let issue_decision = authorize_mutation(
        &authority,
        issue_effect,
        issue_approval,
        CAPABILITY_LEASE_ISSUE_ACTION,
        prepared.resource(),
        prepared.intent_digest(),
        51_000,
    );
    let replay_prepared = prepared.clone();
    let issued = CapabilityLeaseStore::open(&authority)
        .unwrap()
        .issue(
            prepared,
            issue_decision.decision_id,
            issue_approval,
            issue_effect,
        )
        .unwrap();
    assert!(matches!(
        CapabilityLeaseStore::open(&authority).unwrap().issue(
            replay_prepared,
            issue_decision.decision_id,
            issue_approval,
            issue_effect,
        ),
        Err(CapabilityLeaseMutationError::ApprovalAlreadyUsed)
    ));

    let revoke_prepared = prepare_capability_lease_revocation(
        binding(&issued),
        "manual_revoke",
        "2026-08-27T12:00:00Z",
    )
    .unwrap();
    let revoke_replay = revoke_prepared.clone();
    let revoke_effect = EffectId(52_000);
    let revoke_approval = 52_000_u128.to_be_bytes();
    let revoke_decision = authorize_mutation(
        &authority,
        revoke_effect,
        revoke_approval,
        CAPABILITY_LEASE_REVOKE_ACTION,
        revoke_prepared.resource(),
        revoke_prepared.intent_digest(),
        53_000,
    );
    CapabilityLeaseStore::open(&authority)
        .unwrap()
        .revoke(
            revoke_prepared,
            revoke_decision.decision_id,
            revoke_approval,
            revoke_effect,
        )
        .unwrap();
    assert!(matches!(
        CapabilityLeaseStore::open(&authority).unwrap().revoke(
            revoke_replay,
            revoke_decision.decision_id,
            revoke_approval,
            revoke_effect,
        ),
        Err(CapabilityLeaseMutationError::ApprovalAlreadyUsed)
    ));

    let fresh_revoke = prepare_capability_lease_revocation(
        binding(&issued),
        "manual_revoke_again",
        "2026-08-27T12:00:01Z",
    )
    .unwrap();
    let fresh_effect = EffectId(54_000);
    let fresh_approval = 54_000_u128.to_be_bytes();
    let fresh_decision = authorize_mutation(
        &authority,
        fresh_effect,
        fresh_approval,
        CAPABILITY_LEASE_REVOKE_ACTION,
        fresh_revoke.resource(),
        fresh_revoke.intent_digest(),
        55_000,
    );
    assert!(matches!(
        CapabilityLeaseStore::open(&authority).unwrap().revoke(
            fresh_revoke,
            fresh_decision.decision_id,
            fresh_approval,
            fresh_effect,
        ),
        Err(CapabilityLeaseMutationError::LeaseAlreadyRevoked)
    ));

    let chain = load_capability_lease_runtime_chain(&authority, issued.lease_id).unwrap();
    assert_eq!(chain.len(), 1);
    assert!(chain[0].revoked);

    drop(CapabilityLeaseStore::open(&authority).unwrap());
    fs::remove_dir_all(runtime.root).unwrap();
}
