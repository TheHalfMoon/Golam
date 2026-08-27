#![forbid(unsafe_code)]

use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;
use golam_core::{EffectAttemptId, EffectId, EffectTransitionId, EventId, SessionId};
use golam_ledger::dispatch::encode_effect_dependencies;
use golam_ledger::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
use rusqlite::{Connection, TransactionBehavior, params};

static N: AtomicU64 = AtomicU64::new(0);

fn runtime() -> RuntimeLayout {
    let n = N.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    RuntimeLayout::initialize(std::env::temp_dir().join(format!(
        "golam-disk-full-fail-closed-{}-{t}-{n}",
        std::process::id()
    )))
    .unwrap()
}

fn id_blob(value: u128) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

#[test]
fn sqlite_full_before_durable_dispatch_rolls_back_attempt_and_state() {
    let runtime = runtime();
    let authority = AuthorityLayout::initialize(&runtime).unwrap();
    let effect_id = EffectId(92_000);
    let attempt_id = EffectAttemptId(92_001);
    let dependencies = encode_effect_dependencies(&[]).unwrap();

    let mut effects = EffectStore::open(&authority).unwrap();
    effects
        .propose(ProposeEffect {
            effect_id,
            session_id: SessionId(92_010),
            requested_by: "owner",
            action: "sim.write",
            resource: "sim:disk-full",
            risk_class: "synthetic",
            execution_semantics: "at_most_once",
            idempotency_key: None,
            preconditions: b"[]",
            dependencies: &dependencies,
            payload_hash: [7; 32],
            proposed_event_id: EventId(92_020),
            transition_id: EffectTransitionId(92_030),
        })
        .unwrap();
    let authorized = effects
        .compare_and_swap(CompareAndSwapEffect {
            transition_id: EffectTransitionId(92_031),
            effect_id,
            expected_state: "proposed",
            next_state: "authorized",
            attempt_id: None,
            reason_code: Some("disk_full_test_authorized"),
            evidence_ref: None,
            event_id: EventId(92_021),
        })
        .unwrap();
    drop(effects);

    let mut connection = Connection::open(authority.authority_db_path()).unwrap();
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )
        .unwrap();
    let page_count: i64 = connection
        .query_row("PRAGMA page_count", [], |row| row.get(0))
        .unwrap();
    connection
        .execute_batch(&format!("PRAGMA max_page_count = {page_count};"))
        .unwrap();

    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    let oversized_dispatch_token = vec![0x5a_u8; 4 * 1024 * 1024];
    let error = transaction
        .execute(
            "INSERT INTO effect_attempts (attempt_id, effect_id, started_global_seq, handler_id, \
             handler_version, dispatch_token, started_at, finished_at, outcome, receipt) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, 'unknown', NULL)",
            params![
                id_blob(attempt_id.0),
                id_blob(effect_id.0),
                i64::try_from(authorized.global_seq).unwrap(),
                "sim-at-most-once-write",
                "1",
                &oversized_dispatch_token,
                "2026-08-26T08:52:00Z",
            ],
        )
        .expect_err("bounded database must report SQLITE_FULL before attempt commit");
    assert!(matches!(
        error,
        rusqlite::Error::SqliteFailure(ref code, _)
            if code.extended_code == rusqlite::ffi::SQLITE_FULL
    ));
    drop(transaction);
    drop(connection);

    let effects = EffectStore::open(&authority).unwrap();
    assert_eq!(effects.attempt_count(effect_id).unwrap(), 0);
    assert!(effects.attempt(attempt_id).unwrap().is_none());
    assert_eq!(
        effects.current_state(effect_id).unwrap().as_deref(),
        Some("authorized")
    );
    drop(effects);

    fs::remove_dir_all(runtime.root).unwrap();
}
