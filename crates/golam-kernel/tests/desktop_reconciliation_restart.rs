#![forbid(unsafe_code)]

use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;
use golam_core::tool_request::BindingDigest;
use golam_core::{EffectId, EventId, SessionId, ToolReconciliationResolution};
use golam_kernel::{
    AuthorizationPolicy, AuthorizationRequest, CompleteToolEffect, DesktopReconciliationError,
    DesktopReconciliationRepair, KernelApi, KernelCreateSession, PolicyDecision, PrepareToolEffect,
    Principal, ToolEffectError, ToolExecutionCompletion, ToolMutationEvidenceKernelError,
    ToolMutationVerifiedStatus,
};
use golam_ledger::desktop_control_evidence::{
    DesktopControlEvidenceStore, DesktopEffectEvidence, DesktopEvidenceOperation,
    DesktopEvidenceStatus,
};
use golam_ledger::effects::EffectStore;

static N: AtomicU64 = AtomicU64::new(0);
const PROVIDER_ID: &str = "golam-desktop-action-v1";

struct AllowDesktop;

impl AuthorizationPolicy for AllowDesktop {
    fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
        PolicyDecision::allow("desktop_reconciliation_restart_qualification")
    }
}

fn runtime(label: &str) -> RuntimeLayout {
    let n = N.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    RuntimeLayout::initialize(std::env::temp_dir().join(format!(
        "golam-desktop-reconciliation-{label}-{}-{t}-{n}",
        std::process::id()
    )))
    .unwrap()
}

fn digest(value: u8) -> BindingDigest {
    BindingDigest::new([value; 32])
}

fn kernel_with_session(runtime: &RuntimeLayout) -> KernelApi<AllowDesktop> {
    let mut kernel = KernelApi::open(runtime, AllowDesktop).unwrap();
    kernel
        .create_session(
            Principal::test("desktop-restart"),
            KernelCreateSession {
                session_id: SessionId(90_000),
                event_id: EventId(90_001),
                recorded_at: "2026-09-07T00:00:00Z",
                payload: b"desktop-reconciliation-restart-qualification",
            },
            "desktop-restart",
        )
        .unwrap();
    kernel
}

fn prepare(kernel: &mut KernelApi<AllowDesktop>, effect_id: EffectId) -> golam_kernel::PreparedToolEffect {
    kernel
        .prepare_tool_effect(
            Principal::test("desktop-restart"),
            PrepareToolEffect {
                effect_id,
                session_id: SessionId(90_000),
                action: "desktop.focus",
                resource: "desktop-target:restart-proof",
                execution_semantics: "at_most_once",
                handler_id: "golam-desktop-kernel",
                handler_version: "1",
                idempotency_key: Some("desktop-restart-proof"),
                preconditions_hash: [31; 32],
                payload_hash: [32; 32],
                started_at: "2026-09-07T00:00:01Z",
            },
            "desktop-restart",
        )
        .unwrap()
}

fn prepared_evidence(effect_id: EffectId) -> DesktopEffectEvidence {
    DesktopEffectEvidence {
        effect_id,
        session_id: SessionId(90_000),
        operation: DesktopEvidenceOperation::Focus,
        request_digest: digest(1),
        effect_digest: digest(2),
        intent_digest: digest(3),
        fallback_eligibility_digest: None,
        control_lease_digest: Some(digest(4)),
        visible_channel_digest: Some(digest(5)),
        permission_session_digest: digest(6),
        target_or_source_digest: digest(7),
        status: DesktopEvidenceStatus::Prepared,
        reconciliation_ref: None,
        recorded_at_unix_ms: 1_000,
    }
}

fn seed_desktop_prepared(runtime: &RuntimeLayout, effect_id: EffectId) {
    let authority = AuthorityLayout::initialize(runtime).unwrap();
    let mut store = DesktopControlEvidenceStore::open(&authority).unwrap();
    store
        .append_effect_evidence(prepared_evidence(effect_id))
        .unwrap();
}

fn latest_desktop_status(runtime: &RuntimeLayout, effect_id: EffectId) -> DesktopEvidenceStatus {
    let authority = AuthorityLayout::initialize(runtime).unwrap();
    DesktopControlEvidenceStore::open(&authority)
        .unwrap()
        .latest_effect_status(effect_id)
        .unwrap()
        .unwrap()
}

fn generic_state(runtime: &RuntimeLayout, effect_id: EffectId) -> String {
    let authority = AuthorityLayout::initialize(runtime).unwrap();
    EffectStore::open(&authority)
        .unwrap()
        .current_state(effect_id)
        .unwrap()
        .unwrap()
}

#[test]
fn prepared_desktop_effect_is_durable_unresolved_state() {
    let runtime = runtime("prepared-blocks");
    let effect_id = EffectId(91_100);
    let mut kernel = kernel_with_session(&runtime);
    let _prepared = prepare(&mut kernel, effect_id);
    seed_desktop_prepared(&runtime, effect_id);

    let authority = AuthorityLayout::initialize(&runtime).unwrap();
    let store = DesktopControlEvidenceStore::open(&authority).unwrap();
    assert!(
        store
            .has_unresolved_unknown_outcome_for_effect(effect_id)
            .unwrap()
    );
    drop(store);
    drop(kernel);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn restart_moves_prepared_desktop_effect_to_reconciling_without_redispatch() {
    let runtime = runtime("begin");
    let effect_id = EffectId(91_200);
    let mut kernel = kernel_with_session(&runtime);
    let prepared = prepare(&mut kernel, effect_id);
    let attempt_id = prepared.attempt_id();
    seed_desktop_prepared(&runtime, effect_id);
    drop(kernel);

    let mut restarted = KernelApi::open(&runtime, AllowDesktop).unwrap();
    let context = restarted
        .begin_desktop_reconciliation(
            Principal::test("desktop-restart"),
            effect_id,
            "2026-09-07T00:00:02Z",
            1_001,
            "desktop-restart",
        )
        .unwrap();
    assert_eq!(context.attempt_id, attempt_id);
    assert_eq!(generic_state(&runtime, effect_id), "reconciling");
    assert_eq!(
        latest_desktop_status(&runtime, effect_id),
        DesktopEvidenceStatus::Reconciling
    );

    let authority = AuthorityLayout::initialize(&runtime).unwrap();
    let effects = EffectStore::open(&authority).unwrap();
    assert_eq!(effects.attempt_count(effect_id).unwrap(), 1);
    drop(effects);
    drop(restarted);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn verified_restart_receipt_allows_reconciled_success_without_new_attempt() {
    let runtime = runtime("verified-success");
    let effect_id = EffectId(91_300);
    let mut kernel = kernel_with_session(&runtime);
    let prepared = prepare(&mut kernel, effect_id);
    seed_desktop_prepared(&runtime, effect_id);
    kernel
        .record_tool_mutation_intent(
            Principal::test("desktop-restart"),
            &prepared,
            PROVIDER_ID,
            b"canonical-desktop-focus-intent",
            "desktop-restart",
        )
        .unwrap();
    drop(kernel);

    let mut restarted = KernelApi::open(&runtime, AllowDesktop).unwrap();
    restarted
        .begin_desktop_reconciliation(
            Principal::test("desktop-restart"),
            effect_id,
            "2026-09-07T00:00:02Z",
            1_001,
            "desktop-restart",
        )
        .unwrap();
    restarted
        .record_tool_reconciliation_verified_receipt(
            Principal::test("desktop-restart"),
            effect_id,
            PROVIDER_ID,
            ToolMutationVerifiedStatus::Succeeded,
            b"verified-focus-readback",
            "desktop-restart",
        )
        .unwrap();
    restarted
        .resolve_desktop_reconciliation(
            Principal::test("desktop-restart"),
            effect_id,
            ToolReconciliationResolution::Succeeded,
            Some("verified_focus_readback_after_restart"),
            None,
            "2026-09-07T00:00:03Z",
            1_002,
            "desktop-restart",
        )
        .unwrap();

    assert_eq!(generic_state(&runtime, effect_id), "succeeded");
    assert_eq!(
        latest_desktop_status(&runtime, effect_id),
        DesktopEvidenceStatus::ReconciledSucceeded
    );
    let authority = AuthorityLayout::initialize(&runtime).unwrap();
    let effects = EffectStore::open(&authority).unwrap();
    assert_eq!(effects.attempt_count(effect_id).unwrap(), 1);
    drop(effects);
    drop(restarted);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn missing_verified_receipt_cannot_claim_reconciled_success() {
    let runtime = runtime("missing-receipt");
    let effect_id = EffectId(91_400);
    let mut kernel = kernel_with_session(&runtime);
    let prepared = prepare(&mut kernel, effect_id);
    seed_desktop_prepared(&runtime, effect_id);
    kernel
        .record_tool_mutation_intent(
            Principal::test("desktop-restart"),
            &prepared,
            PROVIDER_ID,
            b"canonical-desktop-focus-intent",
            "desktop-restart",
        )
        .unwrap();
    drop(kernel);

    let mut restarted = KernelApi::open(&runtime, AllowDesktop).unwrap();
    restarted
        .begin_desktop_reconciliation(
            Principal::test("desktop-restart"),
            effect_id,
            "2026-09-07T00:00:02Z",
            1_001,
            "desktop-restart",
        )
        .unwrap();
    assert!(matches!(
        restarted.resolve_desktop_reconciliation(
            Principal::test("desktop-restart"),
            effect_id,
            ToolReconciliationResolution::Succeeded,
            Some("caller_claimed_success"),
            None,
            "2026-09-07T00:00:03Z",
            1_002,
            "desktop-restart",
        ),
        Err(DesktopReconciliationError::Tool(ToolEffectError::MutationEvidence(
            ToolMutationEvidenceKernelError::MissingVerifiedReceipt(id)
        ))) if id == effect_id
    ));
    assert_eq!(generic_state(&runtime, effect_id), "reconciling");
    assert_eq!(
        latest_desktop_status(&runtime, effect_id),
        DesktopEvidenceStatus::Reconciling
    );
    drop(restarted);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn direct_terminal_crash_window_repairs_desktop_evidence_without_receipt() {
    let runtime = runtime("direct-terminal-repair");
    let effect_id = EffectId(91_500);
    let mut kernel = kernel_with_session(&runtime);
    let prepared = prepare(&mut kernel, effect_id);
    seed_desktop_prepared(&runtime, effect_id);
    kernel
        .complete_tool_effect(
            Principal::test("desktop-restart"),
            CompleteToolEffect {
                prepared: &prepared,
                finished_at: "2026-09-07T00:00:02Z",
                completion: ToolExecutionCompletion::Succeeded,
                reason_code: Some("provider_committed_before_desktop_evidence_write"),
                evidence_ref: Some(b"verified-direct-receipt"),
                receipt: Some(b"verified-direct-receipt"),
            },
            "desktop-restart",
        )
        .unwrap();

    assert_eq!(generic_state(&runtime, effect_id), "succeeded");
    assert_eq!(
        latest_desktop_status(&runtime, effect_id),
        DesktopEvidenceStatus::Prepared
    );
    let repair = kernel
        .repair_desktop_reconciliation_evidence(
            Principal::test("desktop-restart"),
            effect_id,
            1_001,
            "desktop-restart",
        )
        .unwrap();
    assert_eq!(
        repair,
        DesktopReconciliationRepair::Repaired(DesktopEvidenceStatus::Succeeded)
    );
    assert_eq!(
        latest_desktop_status(&runtime, effect_id),
        DesktopEvidenceStatus::Succeeded
    );
    drop(kernel);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn substituted_reconciliation_reference_fails_closed() {
    let runtime = runtime("binding-mismatch");
    let effect_id = EffectId(91_600);
    let mut kernel = kernel_with_session(&runtime);
    let _prepared = prepare(&mut kernel, effect_id);
    seed_desktop_prepared(&runtime, effect_id);

    let authority = AuthorityLayout::initialize(&runtime).unwrap();
    let mut store = DesktopControlEvidenceStore::open(&authority).unwrap();
    let unknown = store.recovered_unknown_evidence(effect_id, 1_001).unwrap();
    store.append_effect_evidence(unknown).unwrap();
    let substituted = store
        .reconciliation_evidence(
            effect_id,
            DesktopEvidenceStatus::Reconciling,
            digest(99),
            1_002,
        )
        .unwrap();
    store.append_effect_evidence(substituted).unwrap();
    drop(store);
    drop(kernel);

    let mut restarted = KernelApi::open(&runtime, AllowDesktop).unwrap();
    assert!(matches!(
        restarted.begin_desktop_reconciliation(
            Principal::test("desktop-restart"),
            effect_id,
            "2026-09-07T00:00:03Z",
            1_003,
            "desktop-restart",
        ),
        Err(DesktopReconciliationError::ReconciliationBindingMismatch(id)) if id == effect_id
    ));
    assert_eq!(generic_state(&runtime, effect_id), "reconciling");
    assert_eq!(
        latest_desktop_status(&runtime, effect_id),
        DesktopEvidenceStatus::Reconciling
    );
    drop(restarted);
    fs::remove_dir_all(runtime.root).unwrap();
}
