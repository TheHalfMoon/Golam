use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;
use golam_core::taint::{TaintLabel, TaintSet};
use golam_core::{EffectId, EffectTransitionId, EventId, SessionId};
use golam_ledger::approval_binding::{
    APPROVAL_ISSUE_ACTION, APPROVAL_MUTATION_RISK_CLASS, ApprovalStore, prepare_approval,
};
use golam_ledger::approval_revocation::{
    APPROVAL_REVOKE_ACTION, ApprovalRevocationStore, prepare_approval_revocation,
};
use golam_ledger::approvals::ApprovalScope;
use golam_ledger::authorization::{
    AppendAuthorizationDecision, AuthorizationAuditLog, AuthorizationDecisionEvidence,
    AuthorizationDecisionKind,
};
use golam_ledger::capability_leases::{
    CAPABILITY_LEASE_ISSUE_ACTION, CAPABILITY_LEASE_MUTATION_RISK_CLASS,
    CAPABILITY_LEASE_REVOKE_ACTION, CapabilityLeaseBinding, CapabilityLeaseRecord,
    CapabilityLeaseStore, prepare_capability_lease_issue, prepare_capability_lease_revocation,
};
use golam_ledger::dispatch::encode_effect_dependencies;
use golam_ledger::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
use golam_ledger::egress_permit::{
    EGRESS_PERMIT_ISSUE_ACTION, EGRESS_PERMIT_MUTATION_RISK_CLASS, EGRESS_PERMIT_REVOKE_ACTION,
    EgressParentLeaseBinding, EgressPermitRecord, EgressPermitStore, prepare_egress_permit_issue,
    prepare_egress_permit_revocation,
};
use golam_ledger::policy::{
    POLICY_ACTIVATE_ACTION, POLICY_MUTATION_RISK_CLASS, POLICY_STAGE_ACTION, PolicyStore,
    policy_bundle_resource, prepare_policy_bundle,
};
use golam_ledger::sandbox_profile::{
    SANDBOX_PROFILE_MUTATION_RISK_CLASS, SANDBOX_PROFILE_REGISTER_ACTION, SandboxNetworkRule,
    SandboxProfileClass, SandboxProfileDefinition, SandboxProfileStore, SandboxSpawnRule,
    prepare_sandbox_profile,
};
use golam_ledger::storage::{AuthorityStore, CreateSession};
use golam_ledger::taint_attestation::{
    TAINT_DOWNGRADE_ACTION, TaintAttestationStore, prepare_human_downgrade,
};
use golam_ledger::verifier_registry::{
    TAINT_AUTHORITY_MUTATION_RISK_CLASS, VERIFIER_RULE_REGISTER_ACTION, VerifierRuleKind,
    VerifierRuleStore, prepare_verifier_rule,
};
use rusqlite::{Connection, params};

static NEXT_ID: AtomicU64 = AtomicU64::new(10_000);

const OWNER: &str = "owner:owner";
const TEST_TIME: &str = "2026-08-30T12:00:00Z";
const TEST_TIME_LATER: &str = "2026-08-30T13:00:00Z";

struct Fixture {
    runtime: RuntimeLayout,
    authority: AuthorityLayout,
    session_id: SessionId,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let n = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-t003-084-{label}-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let session_id = SessionId(next_id());
        let mut store = AuthorityStore::open(authority.authority_db_path()).unwrap();
        store
            .create_session(CreateSession {
                session_id,
                event_id: EventId(next_id()),
                owner_principal: OWNER,
                actor_principal: OWNER,
                recorded_at: TEST_TIME,
                payload: b"t003-084-session",
                security_critical: false,
            })
            .unwrap();
        drop(store);
        Self {
            runtime,
            authority,
            session_id,
        }
    }

    fn cleanup(self) {
        fs::remove_dir_all(self.runtime.root).unwrap();
    }
}

fn next_id() -> u128 {
    u128::from(NEXT_ID.fetch_add(1, Ordering::Relaxed))
}

fn create_authorized_effect(
    fixture: &Fixture,
    action: &str,
    resource: &str,
    risk_class: &str,
    payload_hash: [u8; 32],
) -> EffectId {
    let effect_id = EffectId(next_id());
    let dependencies = encode_effect_dependencies(&[]).unwrap();
    let mut effects = EffectStore::open(&fixture.authority).unwrap();
    effects
        .propose(ProposeEffect {
            effect_id,
            session_id: fixture.session_id,
            requested_by: OWNER,
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
    effects
        .compare_and_swap(CompareAndSwapEffect {
            transition_id: EffectTransitionId(next_id()),
            effect_id,
            expected_state: "proposed",
            next_state: "authorized",
            attempt_id: None,
            reason_code: Some("t003_084_authorized"),
            evidence_ref: None,
            event_id: EventId(next_id()),
        })
        .unwrap();
    effect_id
}

fn append_allow(fixture: &Fixture, principal: &str, action: &str, resource: &str) -> [u8; 16] {
    let mut log = AuthorizationAuditLog::open(&fixture.authority).unwrap();
    log.append(AppendAuthorizationDecision {
        principal,
        action,
        resource,
        context: "local:t003-084",
        evidence: AuthorizationDecisionEvidence::hard_guard_only("pass"),
        decision: AuthorizationDecisionKind::Allow,
        reason_code: "t003_084_allow",
    })
    .unwrap()
    .decision_id
}

fn issue_once_approval(
    fixture: &Fixture,
    target_effect: EffectId,
    target_action: &str,
    target_resource: &str,
    target_risk: &str,
    taint_digest: [u8; 32],
) -> [u8; 16] {
    let scope = ApprovalScope::once(target_effect, target_action, target_resource).unwrap();
    let prepared =
        prepare_approval(OWNER, scope, target_risk, taint_digest, TEST_TIME, None, 1).unwrap();
    let issue_effect = create_authorized_effect(
        fixture,
        APPROVAL_ISSUE_ACTION,
        prepared.resource(),
        APPROVAL_MUTATION_RISK_CLASS,
        prepared.intent_digest(),
    );
    let decision = append_allow(fixture, OWNER, APPROVAL_ISSUE_ACTION, prepared.resource());
    ApprovalStore::open(&fixture.authority)
        .unwrap()
        .issue(prepared, decision, issue_effect)
        .unwrap()
        .approval_id()
}

fn install_commit_fault(authority: &AuthorityLayout) {
    let connection = Connection::open(authority.authority_db_path()).unwrap();
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE t003_084_fault_parent (id INTEGER PRIMARY KEY);
             CREATE TABLE t003_084_fault_child (
               audit_seq INTEGER PRIMARY KEY,
               parent_id INTEGER NOT NULL,
               FOREIGN KEY(parent_id) REFERENCES t003_084_fault_parent(id)
                 DEFERRABLE INITIALLY DEFERRED
             );
             CREATE TRIGGER t003_084_fail_authority_commit
             AFTER INSERT ON authority_security_audit_v2
             BEGIN
               INSERT INTO t003_084_fault_child(audit_seq, parent_id)
               VALUES (NEW.audit_seq, 1);
             END;",
        )
        .unwrap();
}

fn remove_commit_fault(authority: &AuthorityLayout) {
    let connection = Connection::open(authority.authority_db_path()).unwrap();
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;
             DROP TRIGGER IF EXISTS t003_084_fail_authority_commit;
             DROP TABLE IF EXISTS t003_084_fault_child;
             DROP TABLE IF EXISTS t003_084_fault_parent;",
        )
        .unwrap();
}

fn assert_commit_fault(error: &dyn std::fmt::Display) {
    let rendered = error.to_string().to_ascii_lowercase();
    assert!(
        rendered.contains("foreign key constraint failed"),
        "expected deferred commit failure, got: {rendered}"
    );
}

fn row_count(authority: &AuthorityLayout, sql: &str) -> i64 {
    Connection::open(authority.authority_db_path())
        .unwrap()
        .query_row(sql, [], |row| row.get(0))
        .unwrap()
}

fn approval_consumption_count(authority: &AuthorityLayout, approval_id: [u8; 16]) -> i64 {
    Connection::open(authority.authority_db_path())
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM approval_consumptions WHERE approval_id = ?1",
            params![&approval_id[..]],
            |row| row.get(0),
        )
        .unwrap()
}

fn assert_restart_integrity(authority: &AuthorityLayout) {
    let store = AuthorityStore::open(authority.authority_db_path()).unwrap();
    store.verify_integrity().unwrap();
}

fn issue_committed_lease(
    fixture: &Fixture,
    principal: &str,
    action: &str,
    resource: &str,
) -> CapabilityLeaseRecord {
    let actions = vec![action.to_owned()];
    let resources = vec![resource.to_owned()];
    let prepared =
        prepare_capability_lease_issue(principal, None, &actions, &resources, &[], None, None)
            .unwrap();
    let effect = create_authorized_effect(
        fixture,
        CAPABILITY_LEASE_ISSUE_ACTION,
        prepared.resource(),
        CAPABILITY_LEASE_MUTATION_RISK_CLASS,
        prepared.intent_digest(),
    );
    let approval = issue_once_approval(
        fixture,
        effect,
        CAPABILITY_LEASE_ISSUE_ACTION,
        prepared.resource(),
        CAPABILITY_LEASE_MUTATION_RISK_CLASS,
        [0; 32],
    );
    let decision = append_allow(
        fixture,
        OWNER,
        CAPABILITY_LEASE_ISSUE_ACTION,
        prepared.resource(),
    );
    CapabilityLeaseStore::open(&fixture.authority)
        .unwrap()
        .issue(prepared, decision, approval, effect)
        .unwrap()
}

fn issue_committed_egress_permit(
    fixture: &Fixture,
    lease: &CapabilityLeaseRecord,
) -> EgressPermitRecord {
    let parent =
        EgressParentLeaseBinding::new(lease.lease_id, lease.generation, lease.authority_digest);
    let prepared = prepare_egress_permit_issue(
        &lease.principal_id,
        "network.egress",
        "t003-084",
        "https://example.invalid",
        "tcp:443",
        [0; 32],
        None,
        parent,
        TEST_TIME,
        None,
        Some(2),
    )
    .unwrap();
    let effect = create_authorized_effect(
        fixture,
        EGRESS_PERMIT_ISSUE_ACTION,
        prepared.resource(),
        EGRESS_PERMIT_MUTATION_RISK_CLASS,
        prepared.intent_digest(),
    );
    let approval = issue_once_approval(
        fixture,
        effect,
        EGRESS_PERMIT_ISSUE_ACTION,
        prepared.resource(),
        EGRESS_PERMIT_MUTATION_RISK_CLASS,
        [0; 32],
    );
    let decision = append_allow(
        fixture,
        OWNER,
        EGRESS_PERMIT_ISSUE_ACTION,
        prepared.resource(),
    );
    EgressPermitStore::open(&fixture.authority)
        .unwrap()
        .issue(prepared, decision, approval, effect)
        .unwrap()
}

#[test]
fn authorization_audit_commit_failure_rolls_back_all_coupled_evidence() {
    let fixture = Fixture::new("authorization");
    let mut log = AuthorizationAuditLog::open(&fixture.authority).unwrap();
    install_commit_fault(&fixture.authority);
    let result = log.append(AppendAuthorizationDecision {
        principal: OWNER,
        action: "test.read",
        resource: "test:resource",
        context: "local:t003-084",
        evidence: AuthorizationDecisionEvidence::hard_guard_only("pass"),
        decision: AuthorizationDecisionKind::Allow,
        reason_code: "t003_084_allow",
    });
    let error = result.unwrap_err();
    assert_commit_fault(&error);
    drop(log);
    remove_commit_fault(&fixture.authority);
    assert_eq!(
        row_count(
            &fixture.authority,
            "SELECT COUNT(*) FROM authorization_decisions"
        ),
        0
    );
    assert_restart_integrity(&fixture.authority);
    fixture.cleanup();
}

#[test]
fn policy_stage_and_activation_commit_failures_roll_back_source_and_approval_state() {
    let fixture = Fixture::new("policy");
    let prepared =
        prepare_policy_bundle(1, "permit(principal, action, resource);", "entity User;").unwrap();
    let bundle_id = prepared.policy_bundle_id();
    let resource = policy_bundle_resource(bundle_id);
    let stage_decision = append_allow(&fixture, OWNER, POLICY_STAGE_ACTION, &resource);

    let mut policies = PolicyStore::open(&fixture.authority).unwrap();
    install_commit_fault(&fixture.authority);
    let error = policies
        .stage_prepared(prepared, stage_decision)
        .unwrap_err();
    assert_commit_fault(&error);
    drop(policies);
    remove_commit_fault(&fixture.authority);
    assert_eq!(
        row_count(&fixture.authority, "SELECT COUNT(*) FROM policy_bundles"),
        0
    );
    assert_restart_integrity(&fixture.authority);

    let prepared =
        prepare_policy_bundle(1, "permit(principal, action, resource);", "entity User;").unwrap();
    PolicyStore::open(&fixture.authority)
        .unwrap()
        .stage_prepared(prepared, stage_decision)
        .unwrap();

    let activation_effect = create_authorized_effect(
        &fixture,
        POLICY_ACTIVATE_ACTION,
        &resource,
        POLICY_MUTATION_RISK_CLASS,
        [0; 32],
    );
    let approval = issue_once_approval(
        &fixture,
        activation_effect,
        POLICY_ACTIVATE_ACTION,
        &resource,
        POLICY_MUTATION_RISK_CLASS,
        [0; 32],
    );
    let activation_decision = append_allow(&fixture, OWNER, POLICY_ACTIVATE_ACTION, &resource);
    let mut policies = PolicyStore::open(&fixture.authority).unwrap();
    install_commit_fault(&fixture.authority);
    let error = policies
        .activate(bundle_id, activation_decision, approval, activation_effect)
        .unwrap_err();
    assert_commit_fault(&error);
    drop(policies);
    remove_commit_fault(&fixture.authority);
    assert_eq!(
        row_count(&fixture.authority, "SELECT COUNT(*) FROM active_policy"),
        0
    );
    assert_eq!(approval_consumption_count(&fixture.authority, approval), 0);
    assert_restart_integrity(&fixture.authority);
    fixture.cleanup();
}

#[test]
fn approval_issue_and_revocation_commit_failures_preserve_prior_authority() {
    let fixture = Fixture::new("approval");
    let target_effect = EffectId(next_id());
    let scope = ApprovalScope::once(target_effect, "test.effect", "test:resource").unwrap();
    let prepared =
        prepare_approval(OWNER, scope, "test_risk", [0; 32], TEST_TIME, None, 1).unwrap();
    let issue_resource = prepared.resource().to_owned();
    let issue_effect = create_authorized_effect(
        &fixture,
        APPROVAL_ISSUE_ACTION,
        &issue_resource,
        APPROVAL_MUTATION_RISK_CLASS,
        prepared.intent_digest(),
    );
    let issue_decision = append_allow(&fixture, OWNER, APPROVAL_ISSUE_ACTION, &issue_resource);
    let mut approvals = ApprovalStore::open(&fixture.authority).unwrap();
    install_commit_fault(&fixture.authority);
    let error = approvals
        .issue(prepared, issue_decision, issue_effect)
        .unwrap_err();
    assert_commit_fault(&error);
    drop(approvals);
    remove_commit_fault(&fixture.authority);
    assert_eq!(
        row_count(&fixture.authority, "SELECT COUNT(*) FROM approvals"),
        0
    );
    assert_restart_integrity(&fixture.authority);

    let scope = ApprovalScope::once(target_effect, "test.effect", "test:resource").unwrap();
    let prepared =
        prepare_approval(OWNER, scope, "test_risk", [0; 32], TEST_TIME, None, 1).unwrap();
    let approval = ApprovalStore::open(&fixture.authority)
        .unwrap()
        .issue(prepared, issue_decision, issue_effect)
        .unwrap();
    let revocation =
        prepare_approval_revocation(approval.approval_id(), OWNER, TEST_TIME_LATER).unwrap();
    let revoke_effect = create_authorized_effect(
        &fixture,
        APPROVAL_REVOKE_ACTION,
        revocation.resource(),
        APPROVAL_MUTATION_RISK_CLASS,
        revocation.intent_digest(),
    );
    let revoke_decision = append_allow(
        &fixture,
        OWNER,
        APPROVAL_REVOKE_ACTION,
        revocation.resource(),
    );
    let mut revocations = ApprovalRevocationStore::open(&fixture.authority).unwrap();
    install_commit_fault(&fixture.authority);
    let error = revocations
        .revoke(revocation, revoke_decision, revoke_effect)
        .unwrap_err();
    assert_commit_fault(&error);
    drop(revocations);
    remove_commit_fault(&fixture.authority);
    let revoked_at: Option<String> = Connection::open(fixture.authority.authority_db_path())
        .unwrap()
        .query_row(
            "SELECT revoked_at FROM approvals WHERE approval_id = ?1",
            params![&approval.approval_id()[..]],
            |row| row.get(0),
        )
        .unwrap();
    assert!(revoked_at.is_none());
    assert_restart_integrity(&fixture.authority);
    fixture.cleanup();
}

#[test]
fn lease_issue_and_revocation_commit_failures_roll_back_consumption_and_revocation() {
    let fixture = Fixture::new("lease");
    let actions = vec!["network.egress".to_owned()];
    let resources = vec!["https://example.invalid".to_owned()];
    let prepared = prepare_capability_lease_issue(
        "client:42:worker",
        None,
        &actions,
        &resources,
        &[],
        None,
        None,
    )
    .unwrap();
    let effect = create_authorized_effect(
        &fixture,
        CAPABILITY_LEASE_ISSUE_ACTION,
        prepared.resource(),
        CAPABILITY_LEASE_MUTATION_RISK_CLASS,
        prepared.intent_digest(),
    );
    let approval = issue_once_approval(
        &fixture,
        effect,
        CAPABILITY_LEASE_ISSUE_ACTION,
        prepared.resource(),
        CAPABILITY_LEASE_MUTATION_RISK_CLASS,
        [0; 32],
    );
    let decision = append_allow(
        &fixture,
        OWNER,
        CAPABILITY_LEASE_ISSUE_ACTION,
        prepared.resource(),
    );
    let mut leases = CapabilityLeaseStore::open(&fixture.authority).unwrap();
    install_commit_fault(&fixture.authority);
    let error = leases
        .issue(prepared, decision, approval, effect)
        .unwrap_err();
    assert_commit_fault(&error);
    drop(leases);
    remove_commit_fault(&fixture.authority);
    assert_eq!(
        row_count(&fixture.authority, "SELECT COUNT(*) FROM capability_leases"),
        0
    );
    assert_eq!(approval_consumption_count(&fixture.authority, approval), 0);
    assert_restart_integrity(&fixture.authority);

    let lease = issue_committed_lease(
        &fixture,
        "client:42:worker",
        "network.egress",
        "https://example.invalid",
    );
    let binding =
        CapabilityLeaseBinding::new(lease.lease_id, lease.generation, lease.authority_digest);
    let prepared =
        prepare_capability_lease_revocation(binding, "t003_084", TEST_TIME_LATER).unwrap();
    let effect = create_authorized_effect(
        &fixture,
        CAPABILITY_LEASE_REVOKE_ACTION,
        prepared.resource(),
        CAPABILITY_LEASE_MUTATION_RISK_CLASS,
        prepared.intent_digest(),
    );
    let approval = issue_once_approval(
        &fixture,
        effect,
        CAPABILITY_LEASE_REVOKE_ACTION,
        prepared.resource(),
        CAPABILITY_LEASE_MUTATION_RISK_CLASS,
        [0; 32],
    );
    let decision = append_allow(
        &fixture,
        OWNER,
        CAPABILITY_LEASE_REVOKE_ACTION,
        prepared.resource(),
    );
    let mut leases = CapabilityLeaseStore::open(&fixture.authority).unwrap();
    install_commit_fault(&fixture.authority);
    let error = leases
        .revoke(prepared, decision, approval, effect)
        .unwrap_err();
    assert_commit_fault(&error);
    drop(leases);
    remove_commit_fault(&fixture.authority);
    assert_eq!(
        row_count(
            &fixture.authority,
            "SELECT COUNT(*) FROM capability_revocations"
        ),
        0
    );
    assert_eq!(approval_consumption_count(&fixture.authority, approval), 0);
    assert_restart_integrity(&fixture.authority);
    fixture.cleanup();
}

#[test]
fn verifier_taint_and_sandbox_commit_failures_leave_no_half_authority() {
    let verifier_fixture = Fixture::new("verifier");
    let prepared = prepare_verifier_rule(
        VerifierRuleKind::DeterministicVerifier,
        1,
        b"t003-084-authoritative-source",
        TaintSet::from_labels([TaintLabel::WebUntrusted]),
        OWNER,
        TaintSet::from_labels([TaintLabel::UserTrusted]),
    )
    .unwrap();
    let effect = create_authorized_effect(
        &verifier_fixture,
        VERIFIER_RULE_REGISTER_ACTION,
        prepared.resource(),
        TAINT_AUTHORITY_MUTATION_RISK_CLASS,
        prepared.intent_digest(),
    );
    let approval = issue_once_approval(
        &verifier_fixture,
        effect,
        VERIFIER_RULE_REGISTER_ACTION,
        prepared.resource(),
        TAINT_AUTHORITY_MUTATION_RISK_CLASS,
        prepared.registration_taint_digest(),
    );
    let decision = append_allow(
        &verifier_fixture,
        OWNER,
        VERIFIER_RULE_REGISTER_ACTION,
        prepared.resource(),
    );
    let mut store = VerifierRuleStore::open(&verifier_fixture.authority).unwrap();
    install_commit_fault(&verifier_fixture.authority);
    let error = store
        .register(prepared, decision, approval, effect)
        .unwrap_err();
    assert_commit_fault(&error);
    drop(store);
    remove_commit_fault(&verifier_fixture.authority);
    assert_eq!(
        row_count(
            &verifier_fixture.authority,
            "SELECT COUNT(*) FROM verifier_rules"
        ),
        0
    );
    assert_eq!(
        approval_consumption_count(&verifier_fixture.authority, approval),
        0
    );
    assert_restart_integrity(&verifier_fixture.authority);
    verifier_fixture.cleanup();

    let taint_fixture = Fixture::new("taint");
    let source_labels = TaintSet::from_labels([TaintLabel::UserTrusted, TaintLabel::WebUntrusted]);
    let result_labels = TaintSet::from_labels([TaintLabel::UserTrusted]);
    let prepared = prepare_human_downgrade(
        [[1; 32]],
        source_labels,
        [2; 32],
        result_labels,
        OWNER,
        [3; 32],
    )
    .unwrap();
    let effect = create_authorized_effect(
        &taint_fixture,
        TAINT_DOWNGRADE_ACTION,
        prepared.resource(),
        TAINT_AUTHORITY_MUTATION_RISK_CLASS,
        prepared.intent_digest(),
    );
    let approval = issue_once_approval(
        &taint_fixture,
        effect,
        TAINT_DOWNGRADE_ACTION,
        prepared.resource(),
        TAINT_AUTHORITY_MUTATION_RISK_CLASS,
        prepared.source_taint_digest(),
    );
    let decision = append_allow(
        &taint_fixture,
        OWNER,
        TAINT_DOWNGRADE_ACTION,
        prepared.resource(),
    );
    let mut store = TaintAttestationStore::open(&taint_fixture.authority).unwrap();
    install_commit_fault(&taint_fixture.authority);
    let error = store
        .attest_human(prepared, decision, approval, effect)
        .unwrap_err();
    assert_commit_fault(&error);
    drop(store);
    remove_commit_fault(&taint_fixture.authority);
    assert_eq!(
        row_count(
            &taint_fixture.authority,
            "SELECT COUNT(*) FROM taint_attestations"
        ),
        0
    );
    assert_eq!(
        approval_consumption_count(&taint_fixture.authority, approval),
        0
    );
    assert_restart_integrity(&taint_fixture.authority);
    taint_fixture.cleanup();

    let sandbox_fixture = Fixture::new("sandbox");
    let empty: [&str; 0] = [];
    let prepared = prepare_sandbox_profile(
        SandboxProfileDefinition {
            profile_id: [9; 16],
            version: 1,
            class: SandboxProfileClass::NativeUntrustedSubprocess,
            filesystem_read_roots: &empty,
            filesystem_write_roots: &empty,
            network_rule: SandboxNetworkRule::DenyAll,
            environment_allowlist: &empty,
            spawn_rule: SandboxSpawnRule::Deny,
            cpu_limit: Some(1),
            memory_limit: Some(1),
            time_limit: Some(1),
            output_limit: Some(1),
            device_allowlist: &empty,
            ipc_allowlist: &empty,
            inherited_handle_rules: &empty,
            platform_requirements: &empty,
        },
        OWNER,
        [7; 32],
    )
    .unwrap();
    let effect = create_authorized_effect(
        &sandbox_fixture,
        SANDBOX_PROFILE_REGISTER_ACTION,
        prepared.resource(),
        SANDBOX_PROFILE_MUTATION_RISK_CLASS,
        prepared.intent_digest(),
    );
    let approval = issue_once_approval(
        &sandbox_fixture,
        effect,
        SANDBOX_PROFILE_REGISTER_ACTION,
        prepared.resource(),
        SANDBOX_PROFILE_MUTATION_RISK_CLASS,
        prepared.mutation_taint_digest(),
    );
    let decision = append_allow(
        &sandbox_fixture,
        OWNER,
        SANDBOX_PROFILE_REGISTER_ACTION,
        prepared.resource(),
    );
    let mut store = SandboxProfileStore::open(&sandbox_fixture.authority).unwrap();
    install_commit_fault(&sandbox_fixture.authority);
    let error = store
        .register(prepared, decision, approval, effect)
        .unwrap_err();
    assert_commit_fault(&error);
    drop(store);
    remove_commit_fault(&sandbox_fixture.authority);
    assert_eq!(
        row_count(
            &sandbox_fixture.authority,
            "SELECT COUNT(*) FROM sandbox_profiles"
        ),
        0
    );
    assert_eq!(
        approval_consumption_count(&sandbox_fixture.authority, approval),
        0
    );
    assert_restart_integrity(&sandbox_fixture.authority);
    sandbox_fixture.cleanup();
}

#[test]
fn egress_issue_and_revocation_commit_failures_preserve_permit_and_approval_state() {
    let fixture = Fixture::new("egress");
    let lease = issue_committed_lease(
        &fixture,
        "client:42:worker",
        "network.egress",
        "https://example.invalid",
    );
    let parent =
        EgressParentLeaseBinding::new(lease.lease_id, lease.generation, lease.authority_digest);
    let prepared = prepare_egress_permit_issue(
        &lease.principal_id,
        "network.egress",
        "t003-084",
        "https://example.invalid",
        "tcp:443",
        [0; 32],
        None,
        parent,
        TEST_TIME,
        None,
        Some(2),
    )
    .unwrap();
    let effect = create_authorized_effect(
        &fixture,
        EGRESS_PERMIT_ISSUE_ACTION,
        prepared.resource(),
        EGRESS_PERMIT_MUTATION_RISK_CLASS,
        prepared.intent_digest(),
    );
    let approval = issue_once_approval(
        &fixture,
        effect,
        EGRESS_PERMIT_ISSUE_ACTION,
        prepared.resource(),
        EGRESS_PERMIT_MUTATION_RISK_CLASS,
        [0; 32],
    );
    let decision = append_allow(
        &fixture,
        OWNER,
        EGRESS_PERMIT_ISSUE_ACTION,
        prepared.resource(),
    );
    let mut store = EgressPermitStore::open(&fixture.authority).unwrap();
    install_commit_fault(&fixture.authority);
    let error = store
        .issue(prepared, decision, approval, effect)
        .unwrap_err();
    assert_commit_fault(&error);
    drop(store);
    remove_commit_fault(&fixture.authority);
    assert_eq!(
        row_count(&fixture.authority, "SELECT COUNT(*) FROM egress_permits"),
        0
    );
    assert_eq!(approval_consumption_count(&fixture.authority, approval), 0);
    assert_restart_integrity(&fixture.authority);

    let permit = issue_committed_egress_permit(&fixture, &lease);
    let prepared = prepare_egress_permit_revocation(permit.permit_id, "t003_084").unwrap();
    let effect = create_authorized_effect(
        &fixture,
        EGRESS_PERMIT_REVOKE_ACTION,
        prepared.resource(),
        EGRESS_PERMIT_MUTATION_RISK_CLASS,
        prepared.intent_digest(),
    );
    let approval = issue_once_approval(
        &fixture,
        effect,
        EGRESS_PERMIT_REVOKE_ACTION,
        prepared.resource(),
        EGRESS_PERMIT_MUTATION_RISK_CLASS,
        [0; 32],
    );
    let decision = append_allow(
        &fixture,
        OWNER,
        EGRESS_PERMIT_REVOKE_ACTION,
        prepared.resource(),
    );
    let mut store = EgressPermitStore::open(&fixture.authority).unwrap();
    install_commit_fault(&fixture.authority);
    let error = store
        .revoke(prepared, decision, approval, effect)
        .unwrap_err();
    assert_commit_fault(&error);
    drop(store);
    remove_commit_fault(&fixture.authority);
    let status: String = Connection::open(fixture.authority.authority_db_path())
        .unwrap()
        .query_row(
            "SELECT status FROM egress_permits WHERE permit_id = ?1",
            params![&permit.permit_id[..]],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(status, "active");
    assert_eq!(approval_consumption_count(&fixture.authority, approval), 0);
    assert_restart_integrity(&fixture.authority);
    fixture.cleanup();
}
