#![forbid(unsafe_code)]

use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::authority::AuthorityLayout;
use golam_core::memory::{
    ExpectedMemoryVersion, MemoryCandidateId, MemoryItemId, MemoryMutationIntent, MemoryOperation,
    MemoryScope, MemoryStoreId, MemoryVersion, MemoryVersionId, MemoryVersionStatus,
    MemoryWriterId, PreparedMemoryMutationIntent,
};
use golam_core::memory_storage::MemoryLayout;
use golam_core::paths::RuntimeLayout;
use golam_core::taint::{TaintLabel, TaintSet};
use golam_core::tool_request::{BindingDigest, PrincipalId};
use golam_core::{EffectId, EffectTransitionId, EventId, SessionId};
use rusqlite::{Connection, params};

use crate::dispatch::encode_effect_dependencies;
use crate::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
use crate::memory_operational::MemoryOperationalStore;
use crate::memory_restart::{
    MemoryRestartError, MemoryRestartObservation, MemoryRestartResolution, MemoryRestartStore,
};
use crate::memory_writer_authority::{
    MEMORY_MUTATION_ACTION, MEMORY_MUTATION_RISK_CLASS, MemoryWriterAuthorityStore,
    memory_mutation_resource,
};

static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(0);
static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_id() -> u128 {
    27_000_000 + u128::from(ID_COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn digest(value: u8) -> BindingDigest {
    BindingDigest::new([value; 32])
}

struct Fixture {
    runtime: RuntimeLayout,
    authority: AuthorityLayout,
    memory: MemoryLayout,
    prepared: PreparedMemoryMutationIntent,
    effect_id: EffectId,
    target_identity: BindingDigest,
    original_content: BindingDigest,
}

impl Fixture {
    fn cleanup(self) {
        fs::remove_dir_all(self.runtime.root).expect("test runtime cleanup must succeed");
    }
}

fn fixture(store_override: Option<MemoryStoreId>) -> Fixture {
    let counter = TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after UNIX epoch")
        .as_nanos();
    let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
        "golam-memory-restart-adversarial-{}-{nanos}-{counter}",
        std::process::id()
    )))
    .expect("test runtime must initialize");
    let authority =
        AuthorityLayout::initialize(&runtime).expect("test authority layout must initialize");
    let memory = MemoryLayout::initialize(&runtime).expect("test memory layout must initialize");

    let effect_id = EffectId(next_id());
    let item_id = MemoryItemId(digest(1));
    let old_version = MemoryVersionId(digest(2));
    let new_version = MemoryVersionId(digest(3));
    let target_identity = digest(4);
    let original_content = digest(5);
    let principal = PrincipalId::new("owner:owner").expect("test principal must be canonical");
    let prepared = MemoryMutationIntent {
        operation: MemoryOperation::Update,
        item_ids: vec![item_id],
        expected_current_versions: vec![ExpectedMemoryVersion {
            item_id,
            expected_version: Some(old_version),
        }],
        expected_markdown_target_identity_ref: target_identity,
        expected_markdown_content_digest: original_content,
        expected_markdown_version: old_version,
        memory_operational_store_ref: store_override.unwrap_or(memory.store_id()),
        candidate_ref: Some(MemoryCandidateId(digest(6))),
        kernel_authorization_ref: digest(7),
        promotion_authority_ref: digest(8),
        effect_id,
        reason_ref: digest(9),
        initiating_principal: principal.clone(),
        created_at_unix_ms: 10,
    }
    .prepare()
    .expect("test memory intent must prepare");
    let version = MemoryVersion {
        item_id,
        version_id: new_version,
        scope: MemoryScope::Project,
        canonical_markdown_ref: digest(11),
        content_digest: digest(12),
        provenance_refs: vec![digest(13)],
        taint_set: TaintSet::from_labels([TaintLabel::UserTrusted]),
        status: MemoryVersionStatus::Active,
        predecessor_versions: vec![old_version],
        conflict_refs: Vec::new(),
        promotion_evidence_ref: digest(14),
        created_by_principal: principal,
        committed_by_writer_identity: MemoryWriterId(digest(15)),
        mutation_effect_ref: effect_id,
        created_at_unix_ms: 16,
    };
    let markdown_path = memory.vault_dir().join("restart-qualified.md");

    let dependencies =
        encode_effect_dependencies(&[]).expect("empty effect dependencies must encode");
    let resource = memory_mutation_resource(&prepared);
    let mut effects = EffectStore::open(&authority).expect("effect store must open");
    effects
        .propose(ProposeEffect {
            effect_id,
            session_id: SessionId(1),
            requested_by: "owner:owner",
            action: MEMORY_MUTATION_ACTION,
            resource: &resource,
            risk_class: MEMORY_MUTATION_RISK_CLASS,
            execution_semantics: "at_most_once",
            idempotency_key: None,
            preconditions: b"[]",
            dependencies: &dependencies,
            payload_hash: prepared.binding_digest(),
            proposed_event_id: EventId(next_id()),
            transition_id: EffectTransitionId(next_id()),
        })
        .expect("test memory effect must be proposed");
    effects
        .compare_and_swap(CompareAndSwapEffect {
            transition_id: EffectTransitionId(next_id()),
            effect_id,
            expected_state: "proposed",
            next_state: "authorized",
            attempt_id: None,
            reason_code: Some("memory_restart_adversarial_authorized"),
            evidence_ref: None,
            event_id: EventId(next_id()),
        })
        .expect("test memory effect must be authorized");
    drop(effects);

    MemoryWriterAuthorityStore::open(&authority)
        .expect("writer authority store must open")
        .prepare(&prepared, &version, &markdown_path)
        .expect("test PREPARED writer authority must persist");

    Fixture {
        runtime,
        authority,
        memory,
        prepared,
        effect_id,
        target_identity,
        original_content,
    }
}

fn pending_case(
    fixture: &Fixture,
) -> (MemoryRestartStore, crate::memory_restart::MemoryRestartCase) {
    let restart = MemoryRestartStore::open(&fixture.authority, &fixture.memory)
        .expect("restart store must open");
    let cases = restart.pending_cases().expect("restart scan must succeed");
    assert_eq!(cases.len(), 1);
    (restart, cases[0].clone())
}

fn operational_effect_rows(fixture: &Fixture) -> i64 {
    let connection = Connection::open(fixture.memory.operational_db_path())
        .expect("operational sqlite must open");
    connection
        .query_row("SELECT COUNT(*) FROM memory_effect_state", [], |row| {
            row.get(0)
        })
        .expect("operational effect count must read")
}

#[test]
fn authority_prepared_without_operational_row_can_only_resolve_as_no_mutation() {
    let fixture = fixture(None);
    let (restart, case) = pending_case(&fixture);
    assert_eq!(operational_effect_rows(&fixture), 0);

    let resolution = restart
        .reconcile(
            &case,
            &MemoryRestartObservation::Regular {
                target_identity_ref: fixture.target_identity,
                content_digest: fixture.original_content,
                markdown_readback_ref: digest(40),
            },
            "2026-09-03T20:50:00Z",
            50,
        )
        .expect("unchanged target must reconcile deterministically");
    assert_eq!(resolution, MemoryRestartResolution::ReconciledNoMutation);
    assert_eq!(operational_effect_rows(&fixture), 1);
    assert_eq!(
        EffectStore::open(&fixture.authority)
            .expect("effect store must reopen")
            .current_state(fixture.effect_id)
            .expect("effect state must read")
            .as_deref(),
        Some("failed")
    );
    fixture.cleanup();
}

#[test]
fn operational_row_without_markdown_is_blocked_unknown_outcome() {
    let fixture = fixture(None);
    MemoryOperationalStore::open(&fixture.memory)
        .expect("operational store must open")
        .record_prepared(&fixture.prepared)
        .expect("PREPARED operational row must persist");
    let (restart, case) = pending_case(&fixture);

    let resolution = restart
        .reconcile(
            &case,
            &MemoryRestartObservation::Missing,
            "2026-09-03T20:51:00Z",
            51,
        )
        .expect("missing Markdown must become explicit unknown outcome");
    assert_eq!(resolution, MemoryRestartResolution::BlockedUnknownOutcome);
    assert!(
        MemoryOperationalStore::open(&fixture.memory)
            .expect("operational store must reopen")
            .has_blocking_unknown_outcome()
            .expect("unknown-outcome gate must read")
    );
    assert_eq!(
        EffectStore::open(&fixture.authority)
            .expect("effect store must reopen")
            .current_state(fixture.effect_id)
            .expect("effect state must read")
            .as_deref(),
        Some("unknown_outcome")
    );
    fixture.cleanup();
}

#[test]
fn target_identity_swap_without_full_cross_store_proof_never_claims_success() {
    let fixture = fixture(None);
    let (restart, case) = pending_case(&fixture);
    let resolution = restart
        .reconcile(
            &case,
            &MemoryRestartObservation::Regular {
                target_identity_ref: digest(99),
                content_digest: digest(98),
                markdown_readback_ref: digest(97),
            },
            "2026-09-03T20:52:00Z",
            52,
        )
        .expect("identity swap must remain an attributable unknown outcome");
    assert_eq!(resolution, MemoryRestartResolution::BlockedUnknownOutcome);
    fixture.cleanup();
}

#[test]
fn stale_scanned_effect_state_is_rejected_before_reconciliation_writes() {
    let fixture = fixture(None);
    let (restart, case) = pending_case(&fixture);
    EffectStore::open(&fixture.authority)
        .expect("effect store must reopen")
        .compare_and_swap(CompareAndSwapEffect {
            transition_id: EffectTransitionId(next_id()),
            effect_id: fixture.effect_id,
            expected_state: "authorized",
            next_state: "approval_required",
            attempt_id: None,
            reason_code: Some("memory_restart_stale_case"),
            evidence_ref: None,
            event_id: EventId(next_id()),
        })
        .expect("test effect state must change after scan");

    assert!(matches!(
        restart.reconcile(
            &case,
            &MemoryRestartObservation::Missing,
            "2026-09-03T20:53:00Z",
            53,
        ),
        Err(MemoryRestartError::StaleCase)
    ));
    assert_eq!(operational_effect_rows(&fixture), 0);
    fixture.cleanup();
}

#[test]
fn wrong_prepared_store_binding_fails_closed_without_projecting_into_current_store() {
    let fixture = fixture(Some(MemoryStoreId(digest(88))));
    let (restart, case) = pending_case(&fixture);
    assert!(matches!(
        restart.reconcile(
            &case,
            &MemoryRestartObservation::Missing,
            "2026-09-03T20:54:00Z",
            54,
        ),
        Err(MemoryRestartError::StoreBindingMismatch)
    ));
    assert_eq!(operational_effect_rows(&fixture), 0);
    fixture.cleanup();
}

#[test]
fn split_operational_intent_digest_is_detected_before_terminal_resolution() {
    let fixture = fixture(None);
    let mut operational =
        MemoryOperationalStore::open(&fixture.memory).expect("operational store must open");
    operational
        .record_prepared(&fixture.prepared)
        .expect("PREPARED operational row must persist");
    drop(operational);
    let connection = Connection::open(fixture.memory.operational_db_path())
        .expect("operational sqlite must reopen");
    connection
        .execute(
            "UPDATE memory_effect_state SET intent_digest = ?1 WHERE effect_id = ?2",
            params![
                digest(77).bytes().to_vec(),
                fixture.effect_id.0.to_be_bytes().to_vec()
            ],
        )
        .expect("test must create a split-store digest disagreement");
    drop(connection);
    let (restart, case) = pending_case(&fixture);

    assert!(matches!(
        restart.reconcile(
            &case,
            &MemoryRestartObservation::Missing,
            "2026-09-03T20:55:00Z",
            55,
        ),
        Err(MemoryRestartError::StoreBindingMismatch)
    ));
    fixture.cleanup();
}

#[test]
fn unreadable_operational_store_never_becomes_success_by_absence_of_terminal_evidence() {
    let fixture = fixture(None);
    let (restart, case) = pending_case(&fixture);
    fs::write(
        fixture.memory.operational_db_path(),
        b"not-a-sqlite-database",
    )
    .expect("test must corrupt operational store bytes");

    assert!(matches!(
        restart.reconcile(
            &case,
            &MemoryRestartObservation::Missing,
            "2026-09-03T20:56:00Z",
            56,
        ),
        Err(MemoryRestartError::Operational(_)) | Err(MemoryRestartError::Sqlite(_))
    ));
    fixture.cleanup();
}
