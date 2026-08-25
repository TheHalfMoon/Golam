#![forbid(unsafe_code)]

use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;
use golam_core::{EffectAttemptId, EffectId, EffectTransitionId, EventId, SessionId};
use golam_kernel::{
    DenyByDefault, EffectDispatchError, KernelApi, KernelError, PrepareEffectDispatch,
    encode_effect_dependencies,
};
use golam_ledger::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};

static N: AtomicU64 = AtomicU64::new(0);

fn runtime(label: &str) -> RuntimeLayout {
    let n = N.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    RuntimeLayout::initialize(std::env::temp_dir().join(format!(
        "golam-restart-safety-{label}-{}-{t}-{n}",
        std::process::id()
    )))
    .unwrap()
}

fn prove_no_blind_duplicate(execution_semantics: &'static str, seed: u128) {
    let runtime = runtime(execution_semantics);
    let authority = AuthorityLayout::initialize(&runtime).unwrap();
    let dependencies = encode_effect_dependencies(&[]).unwrap();
    let effect_id = EffectId(seed);
    let first_attempt_id = EffectAttemptId(seed + 1);
    let second_attempt_id = EffectAttemptId(seed + 2);

    let mut effects = EffectStore::open(&authority).unwrap();
    effects
        .propose(ProposeEffect {
            effect_id,
            session_id: SessionId(seed + 10),
            requested_by: "owner",
            action: "sim.write",
            resource: "sim:restart-target",
            risk_class: "synthetic",
            execution_semantics,
            idempotency_key: None,
            preconditions: b"[]",
            dependencies: &dependencies,
            payload_hash: [9; 32],
            proposed_event_id: EventId(seed + 20),
            transition_id: EffectTransitionId(seed + 30),
        })
        .unwrap();
    effects
        .compare_and_swap(CompareAndSwapEffect {
            transition_id: EffectTransitionId(seed + 31),
            effect_id,
            expected_state: "proposed",
            next_state: "authorized",
            attempt_id: None,
            reason_code: Some("test_authorized"),
            evidence_ref: None,
            event_id: EventId(seed + 21),
        })
        .unwrap();
    drop(effects);

    let mut kernel = KernelApi::open(&runtime, DenyByDefault).unwrap();
    kernel
        .prepare_effect_dispatch(PrepareEffectDispatch {
            effect_id,
            attempt_id: first_attempt_id,
            transition_id: EffectTransitionId(seed + 32),
            handler_id: if execution_semantics == "at_most_once" {
                "sim-at-most-once-write"
            } else {
                "sim-irreversible-write"
            },
            handler_version: "1",
            dispatch_token: b"first-dispatch",
            started_at: "2026-08-25T10:30:00Z",
            event_id: EventId(seed + 22),
        })
        .unwrap();

    // This counter represents the external target accepting the first dispatch.
    // The daemon may die immediately afterward, before any acknowledgement or
    // terminal transition is durably recorded.
    let remote_accept_count = 1usize;
    drop(kernel);

    let mut restarted = KernelApi::open(&runtime, DenyByDefault).unwrap();
    assert!(matches!(
        restarted.prepare_effect_dispatch(PrepareEffectDispatch {
            effect_id,
            attempt_id: second_attempt_id,
            transition_id: EffectTransitionId(seed + 33),
            handler_id: if execution_semantics == "at_most_once" {
                "sim-at-most-once-write"
            } else {
                "sim-irreversible-write"
            },
            handler_version: "1",
            dispatch_token: b"second-dispatch",
            started_at: "2026-08-25T10:31:00Z",
            event_id: EventId(seed + 23),
        }),
        Err(KernelError::EffectDispatch(EffectDispatchError::NotAuthorized {
            effect_id: blocked,
            ref actual,
        })) if blocked == effect_id && actual == "executing"
    ));
    drop(restarted);

    let effects = EffectStore::open(&authority).unwrap();
    assert_eq!(effects.attempt_count(effect_id).unwrap(), 1);
    assert!(effects.attempt(first_attempt_id).unwrap().is_some());
    assert!(effects.attempt(second_attempt_id).unwrap().is_none());
    assert_eq!(
        effects.current_state(effect_id).unwrap().as_deref(),
        Some("executing")
    );
    assert_eq!(remote_accept_count, 1);
    drop(effects);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn at_most_once_does_not_blind_redispatch_after_daemon_restart() {
    prove_no_blind_duplicate("at_most_once", 70_000);
}

#[test]
fn irreversible_does_not_blind_redispatch_after_daemon_restart() {
    prove_no_blind_duplicate("irreversible", 80_000);
}
