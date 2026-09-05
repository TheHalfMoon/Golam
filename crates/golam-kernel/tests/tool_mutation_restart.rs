#![forbid(unsafe_code)]

use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;
use golam_core::{EffectId, EventId, SessionId, ToolReconciliationResolution, ToolReconciliationResult};
use golam_kernel::{
    AuthorizationPolicy, AuthorizationRequest, KernelApi, KernelCreateSession, PolicyDecision,
    PrepareToolEffect, Principal, ToolEffectError, ToolMutationEvidenceKernelError,
    ToolMutationVerifiedStatus,
};
use golam_ledger::effects::EffectStore;

static N: AtomicU64 = AtomicU64::new(0);

struct AllowTools;

impl AuthorizationPolicy for AllowTools {
    fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
        PolicyDecision::allow("tool_mutation_restart_qualification")
    }
}

fn runtime(label: &str) -> RuntimeLayout {
    let n = N.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    RuntimeLayout::initialize(std::env::temp_dir().join(format!(
        "golam-tool-mutation-restart-{label}-{}-{t}-{n}",
        std::process::id()
    )))
    .unwrap()
}

fn kernel_with_session(runtime: &RuntimeLayout, session_id: SessionId) -> KernelApi<AllowTools> {
    let mut kernel = KernelApi::open(runtime, AllowTools).unwrap();
    kernel
        .create_session(
            Principal::test("restart"),
            KernelCreateSession {
                session_id,
                event_id: EventId(session_id.0 + 1),
                recorded_at: "2026-09-05T10:20:00Z",
                payload: b"tool-mutation-restart-qualification",
            },
            "restart",
        )
        .unwrap();
    kernel
}

fn prepare(
    kernel: &mut KernelApi<AllowTools>,
    effect_id: EffectId,
    session_id: SessionId,
) -> golam_kernel::PreparedToolEffect {
    kernel
        .prepare_tool_effect(
            Principal::test("restart"),
            PrepareToolEffect {
                effect_id,
                session_id,
                action: "git.branch.create",
                resource: "git-branch-create:restart-proof",
                execution_semantics: "at_most_once",
                handler_id: "golam-git-linux",
                handler_version: "1",
                idempotency_key: Some("restart-proof"),
                preconditions_hash: [21; 32],
                payload_hash: [22; 32],
                started_at: "2026-09-05T10:20:01Z",
            },
            "restart",
        )
        .unwrap()
}

#[test]
fn verified_mutation_receipt_survives_restart_without_redispatch() {
    let runtime = runtime("verified");
    let session_id = SessionId(91_000);
    let effect_id = EffectId(91_100);
    let mut kernel = kernel_with_session(&runtime, session_id);
    let prepared = prepare(&mut kernel, effect_id, session_id);
    let attempt_id = prepared.attempt_id();
    kernel
        .record_tool_mutation_intent(
            Principal::test("restart"),
            &prepared,
            "golam-git-linux-v1",
            b"git.branch.create:restart-proof",
            "restart",
        )
        .unwrap();
    kernel
        .record_tool_mutation_verified_receipt(
            Principal::test("restart"),
            &prepared,
            "golam-git-linux-v1",
            ToolMutationVerifiedStatus::Succeeded,
            b"verified-ref-readback:restart-proof",
            "restart",
        )
        .unwrap();
    drop(kernel);

    let mut restarted = KernelApi::open(&runtime, AllowTools).unwrap();
    let context = restarted
        .begin_tool_reconciliation(
            Principal::test("restart"),
            effect_id,
            "2026-09-05T10:20:02Z",
            "restart",
        )
        .unwrap();
    assert_eq!(context.attempt_id, attempt_id);
    assert_eq!(context.preconditions_hash, [21; 32]);
    assert_eq!(context.payload_hash, [22; 32]);
    let result = restarted
        .resolve_tool_reconciliation(
            Principal::test("restart"),
            effect_id,
            ToolReconciliationResolution::Succeeded,
            Some("protected_receipt_verified_after_restart"),
            None,
            "2026-09-05T10:20:03Z",
            "restart",
        )
        .unwrap();
    assert_eq!(
        result,
        ToolReconciliationResult::Resolved {
            effect_id,
            state: "succeeded".to_owned(),
        }
    );
    drop(restarted);

    let authority = AuthorityLayout::initialize(&runtime).unwrap();
    let effects = EffectStore::open(&authority).unwrap();
    assert_eq!(effects.attempt_count(effect_id).unwrap(), 1);
    assert_eq!(
        effects.current_state(effect_id).unwrap().as_deref(),
        Some("succeeded")
    );
    drop(effects);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn restart_without_verified_receipt_cannot_claim_success() {
    let runtime = runtime("missing-receipt");
    let session_id = SessionId(92_000);
    let effect_id = EffectId(92_100);
    let mut kernel = kernel_with_session(&runtime, session_id);
    let prepared = prepare(&mut kernel, effect_id, session_id);
    kernel
        .record_tool_mutation_intent(
            Principal::test("restart"),
            &prepared,
            "golam-git-linux-v1",
            b"git.branch.create:restart-proof",
            "restart",
        )
        .unwrap();
    drop(kernel);

    let mut restarted = KernelApi::open(&runtime, AllowTools).unwrap();
    restarted
        .begin_tool_reconciliation(
            Principal::test("restart"),
            effect_id,
            "2026-09-05T10:21:02Z",
            "restart",
        )
        .unwrap();
    assert!(matches!(
        restarted.resolve_tool_reconciliation(
            Principal::test("restart"),
            effect_id,
            ToolReconciliationResolution::Succeeded,
            Some("caller_claimed_success"),
            Some(b"caller-bytes-must-not-be-terminal-proof"),
            "2026-09-05T10:21:03Z",
            "restart",
        ),
        Err(ToolEffectError::MutationEvidence(
            ToolMutationEvidenceKernelError::MissingVerifiedReceipt(id)
        )) if id == effect_id
    ));
    drop(restarted);

    let authority = AuthorityLayout::initialize(&runtime).unwrap();
    let effects = EffectStore::open(&authority).unwrap();
    assert_eq!(effects.attempt_count(effect_id).unwrap(), 1);
    assert_eq!(
        effects.current_state(effect_id).unwrap().as_deref(),
        Some("reconciling")
    );
    drop(effects);
    fs::remove_dir_all(runtime.root).unwrap();
}
