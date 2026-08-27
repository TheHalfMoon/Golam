#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;
use golam_core::{EffectAttemptId, EffectId, EffectTransitionId, EventId, SessionId};
use golam_kernel::{
    DenyByDefault, EffectDispatchError, KernelApi, KernelError, PrepareEffectDispatch,
    encode_effect_dependencies,
};
use golam_ledger::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};

const CHILD_FLAG: &str = "GOLAM_PROCESS_KILL_CHILD";
const ROOT_ENV: &str = "GOLAM_PROCESS_KILL_ROOT";
const MARKER_FILE: &str = "dispatch-durable.marker";
const EFFECT_ID: EffectId = EffectId(91_000);
const FIRST_ATTEMPT: EffectAttemptId = EffectAttemptId(91_001);
const SECOND_ATTEMPT: EffectAttemptId = EffectAttemptId(91_002);

static N: AtomicU64 = AtomicU64::new(0);

fn runtime() -> RuntimeLayout {
    let n = N.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    RuntimeLayout::initialize(std::env::temp_dir().join(format!(
        "golam-process-kill-recovery-{}-{t}-{n}",
        std::process::id()
    )))
    .unwrap()
}

fn seed_authorized_effect(runtime: &RuntimeLayout) {
    let authority = AuthorityLayout::initialize(runtime).unwrap();
    let dependencies = encode_effect_dependencies(&[]).unwrap();
    let mut effects = EffectStore::open(&authority).unwrap();
    effects
        .propose(ProposeEffect {
            effect_id: EFFECT_ID,
            session_id: SessionId(91_010),
            requested_by: "owner",
            action: "sim.write",
            resource: "sim:process-kill",
            risk_class: "synthetic",
            execution_semantics: "at_most_once",
            idempotency_key: None,
            preconditions: b"[]",
            dependencies: &dependencies,
            payload_hash: [9; 32],
            proposed_event_id: EventId(91_020),
            transition_id: EffectTransitionId(91_030),
        })
        .unwrap();
    effects
        .compare_and_swap(CompareAndSwapEffect {
            transition_id: EffectTransitionId(91_031),
            effect_id: EFFECT_ID,
            expected_state: "proposed",
            next_state: "authorized",
            attempt_id: None,
            reason_code: Some("process_kill_test_authorized"),
            evidence_ref: None,
            event_id: EventId(91_021),
        })
        .unwrap();
}

#[test]
fn process_kill_child_after_durable_dispatch() {
    if env::var_os(CHILD_FLAG).is_none() {
        return;
    }

    let root = PathBuf::from(env::var_os(ROOT_ENV).expect("child runtime root is set"));
    let runtime = RuntimeLayout::initialize(&root).unwrap();
    let mut kernel = KernelApi::open(&runtime, DenyByDefault).unwrap();
    kernel
        .prepare_effect_dispatch(PrepareEffectDispatch {
            effect_id: EFFECT_ID,
            attempt_id: FIRST_ATTEMPT,
            transition_id: EffectTransitionId(91_032),
            handler_id: "sim-at-most-once-write",
            handler_version: "1",
            dispatch_token: b"process-kill-first-dispatch",
            started_at: "2026-08-26T08:50:00Z",
            event_id: EventId(91_022),
        })
        .unwrap();

    let marker = runtime.runtime_dir.join(MARKER_FILE);
    fs::write(&marker, b"durable-before-kill").unwrap();

    loop {
        thread::park_timeout(Duration::from_secs(60));
    }
}

#[test]
fn os_process_kill_after_durable_dispatch_blocks_blind_redispatch() {
    if env::var_os(CHILD_FLAG).is_some() {
        return;
    }

    let runtime = runtime();
    seed_authorized_effect(&runtime);
    let marker = runtime.runtime_dir.join(MARKER_FILE);

    let mut child = Command::new(env::current_exe().unwrap())
        .arg("--exact")
        .arg("process_kill_child_after_durable_dispatch")
        .arg("--nocapture")
        .env(CHILD_FLAG, "1")
        .env(ROOT_ENV, &runtime.root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(15);
    while !marker.exists() {
        if let Some(status) = child.try_wait().unwrap() {
            panic!("process-kill child exited before durable marker: {status}");
        }
        assert!(
            Instant::now() < deadline,
            "process-kill child did not produce durable marker before deadline"
        );
        thread::sleep(Duration::from_millis(20));
    }

    child.kill().unwrap();
    let _ = child.wait().unwrap();

    let mut restarted = KernelApi::open(&runtime, DenyByDefault).unwrap();
    assert!(matches!(
        restarted.prepare_effect_dispatch(PrepareEffectDispatch {
            effect_id: EFFECT_ID,
            attempt_id: SECOND_ATTEMPT,
            transition_id: EffectTransitionId(91_033),
            handler_id: "sim-at-most-once-write",
            handler_version: "1",
            dispatch_token: b"process-kill-second-dispatch",
            started_at: "2026-08-26T08:51:00Z",
            event_id: EventId(91_023),
        }),
        Err(KernelError::EffectDispatch(EffectDispatchError::NotAuthorized {
            effect_id: blocked,
            ref actual,
        })) if blocked == EFFECT_ID && actual == "executing"
    ));
    drop(restarted);

    let authority = AuthorityLayout::initialize(&runtime).unwrap();
    let effects = EffectStore::open(&authority).unwrap();
    assert_eq!(effects.attempt_count(EFFECT_ID).unwrap(), 1);
    assert!(effects.attempt(FIRST_ATTEMPT).unwrap().is_some());
    assert!(effects.attempt(SECOND_ATTEMPT).unwrap().is_none());
    assert_eq!(
        effects.current_state(EFFECT_ID).unwrap().as_deref(),
        Some("executing")
    );
    drop(effects);

    fs::remove_file(marker).unwrap();
    fs::remove_dir_all(runtime.root).unwrap();
}
