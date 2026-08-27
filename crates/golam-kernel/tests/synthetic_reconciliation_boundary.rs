#![forbid(unsafe_code)]

use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;
use golam_core::{EffectAttemptId, EffectId, EffectTransitionId, EventId, SessionId};
use golam_kernel::{
    BootstrapPolicy, KernelApi, Principal, ResolveSyntheticReconciliation, SyntheticEffectError,
    SyntheticExecutionCompletion,
};
use golam_ledger::dispatch::{
    EffectDispatchStore, PrepareEffectDispatch, encode_effect_dependencies,
};
use golam_ledger::effect_completion::{
    CompleteEffectExecution, EffectCompletionStore, ExecutionCompletion,
};
use golam_ledger::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};

static N: AtomicU64 = AtomicU64::new(0);
const EFFECT_ID: EffectId = EffectId(250);
const ATTEMPT_ID: EffectAttemptId = EffectAttemptId(8_005);

fn runtime() -> RuntimeLayout {
    let n = N.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    RuntimeLayout::initialize(std::env::temp_dir().join(format!(
        "golam-synthetic-reconciliation-boundary-{}-{t}-{n}",
        std::process::id()
    )))
    .unwrap()
}

fn synthetic_dispatch_token(effect_id: EffectId) -> [u8; 17] {
    let mut token = [0_u8; 17];
    token[..16].copy_from_slice(&effect_id.0.to_be_bytes());
    token[16] = 5;
    token
}

fn seed_ordinary_executing_effect(authority: &AuthorityLayout) {
    let dependencies = encode_effect_dependencies(&[]).unwrap();
    let resource = format!("sim:effect:{}", EFFECT_ID.0);
    let mut effects = EffectStore::open(authority).unwrap();
    effects
        .propose(ProposeEffect {
            effect_id: EFFECT_ID,
            session_id: SessionId(251),
            requested_by: "owner",
            action: "sim.write",
            resource: &resource,
            risk_class: "ordinary",
            execution_semantics: "at_most_once",
            idempotency_key: None,
            preconditions: b"ordinary-effect",
            dependencies: &dependencies,
            payload_hash: [25; 32],
            proposed_event_id: EventId(1),
            transition_id: EffectTransitionId(2),
        })
        .unwrap();
    effects
        .compare_and_swap(CompareAndSwapEffect {
            transition_id: EffectTransitionId(3),
            effect_id: EFFECT_ID,
            expected_state: "proposed",
            next_state: "authorized",
            attempt_id: None,
            reason_code: Some("ordinary_authorized"),
            evidence_ref: None,
            event_id: EventId(4),
        })
        .unwrap();
    drop(effects);

    let token = synthetic_dispatch_token(EFFECT_ID);
    let mut dispatch = EffectDispatchStore::open(authority).unwrap();
    dispatch
        .prepare_dispatch(PrepareEffectDispatch {
            effect_id: EFFECT_ID,
            attempt_id: ATTEMPT_ID,
            transition_id: EffectTransitionId(5),
            handler_id: "ordinary-handler",
            handler_version: "1",
            dispatch_token: &token,
            started_at: "2026-08-27T01:30:00Z",
            event_id: EventId(6),
        })
        .unwrap();
}

#[test]
fn ordinary_executing_effect_cannot_enter_synthetic_reconciliation() {
    let runtime = runtime();
    let mut kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
    let authority = AuthorityLayout::initialize(&runtime).unwrap();
    seed_ordinary_executing_effect(&authority);

    let error = kernel
        .begin_synthetic_reconciliation(
            Principal::local_owner("owner"),
            EFFECT_ID,
            "2026-08-27T01:30:01Z",
            "local-owner",
        )
        .unwrap_err();
    assert!(matches!(
        error,
        SyntheticEffectError::NotSyntheticEffect(effect_id) if effect_id == EFFECT_ID
    ));

    let effects = EffectStore::open(&authority).unwrap();
    assert_eq!(
        effects.current_state(EFFECT_ID).unwrap().as_deref(),
        Some("executing")
    );
    assert_eq!(effects.attempt_count(EFFECT_ID).unwrap(), 1);
    drop(effects);
    drop(kernel);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn ordinary_reconciling_effect_cannot_be_resolved_as_synthetic() {
    let runtime = runtime();
    let mut kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
    let authority = AuthorityLayout::initialize(&runtime).unwrap();
    seed_ordinary_executing_effect(&authority);

    let mut completion = EffectCompletionStore::open(&authority).unwrap();
    completion
        .complete(CompleteEffectExecution {
            effect_id: EFFECT_ID,
            attempt_id: ATTEMPT_ID,
            transition_id: EffectTransitionId(7),
            event_id: EventId(8),
            finished_at: "2026-08-27T01:30:01Z",
            completion: ExecutionCompletion::UnknownOutcome,
            reason_code: Some("ordinary_unknown"),
            evidence_ref: None,
            receipt: None,
        })
        .unwrap();
    drop(completion);

    let mut effects = EffectStore::open(&authority).unwrap();
    effects
        .compare_and_swap(CompareAndSwapEffect {
            transition_id: EffectTransitionId(9),
            effect_id: EFFECT_ID,
            expected_state: "unknown_outcome",
            next_state: "reconciling",
            attempt_id: Some(ATTEMPT_ID),
            reason_code: Some("ordinary_reconciling"),
            evidence_ref: None,
            event_id: EventId(10),
        })
        .unwrap();
    drop(effects);

    let error = kernel
        .resolve_synthetic_reconciliation(
            Principal::local_owner("owner"),
            ResolveSyntheticReconciliation {
                effect_id: EFFECT_ID,
                resolution: SyntheticExecutionCompletion::Succeeded,
                reason_code: Some("must_not_apply"),
                evidence_ref: None,
                detected_at: "2026-08-27T01:30:02Z",
            },
            "local-owner",
        )
        .unwrap_err();
    assert!(matches!(
        error,
        SyntheticEffectError::NotSyntheticEffect(effect_id) if effect_id == EFFECT_ID
    ));

    let effects = EffectStore::open(&authority).unwrap();
    assert_eq!(
        effects.current_state(EFFECT_ID).unwrap().as_deref(),
        Some("reconciling")
    );
    drop(effects);
    drop(kernel);
    fs::remove_dir_all(runtime.root).unwrap();
}
