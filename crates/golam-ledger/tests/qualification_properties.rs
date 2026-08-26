#![forbid(unsafe_code)]

use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;
use golam_core::{CheckpointId, EventId, SCHEMA_VERSION, SessionId};
use golam_ledger::checkpoint::{CheckpointManager, CreateCheckpoint, ProjectionSource};
use golam_ledger::fork::{CreateFork, ForkManager};
use golam_ledger::storage::{AppendEvent, AuthorityStore, CreateSession};
use golam_ledger::{
    EventKind, EventRecord, audit_integrity_hash, event_integrity_hash, payload_hash,
};

static N: AtomicU64 = AtomicU64::new(0);

fn runtime(label: &str) -> RuntimeLayout {
    let n = N.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    RuntimeLayout::initialize(std::env::temp_dir().join(format!(
        "golam-qualification-property-{label}-{}-{t}-{n}",
        std::process::id()
    )))
    .unwrap()
}

fn append_payload_events(
    store: &mut AuthorityStore,
    session_id: SessionId,
    starting_seq: u64,
    count: u64,
    seed: u128,
) {
    for offset in 0..count {
        let expected = starting_seq + offset;
        let payload = format!("qualification-payload-{seed}-{offset}");
        store
            .append_event(AppendEvent {
                event_id: EventId(seed + 100 + u128::from(offset)),
                session_id,
                expected_session_seq: expected,
                kind: EventKind::GoalVersioned,
                actor_principal: "qualification",
                recorded_at: "2026-08-26T09:00:00Z",
                payload: payload.as_bytes(),
                security_critical: true,
            })
            .unwrap();
    }
}

#[test]
fn replay_checkpoint_and_fallback_are_equivalent_across_prefix_lengths() {
    for extra_events in 0_u64..8 {
        let runtime = runtime("replay");
        let authority_layout = AuthorityLayout::initialize(&runtime).unwrap();
        let session_id = SessionId(10_000 + u128::from(extra_events));
        let mut authority = AuthorityStore::open(authority_layout.authority_db_path()).unwrap();
        authority
            .create_session(CreateSession {
                session_id,
                event_id: EventId(20_000 + u128::from(extra_events)),
                owner_principal: "owner",
                actor_principal: "owner",
                recorded_at: "2026-08-26T09:00:00Z",
                payload: b"session-created",
                security_critical: true,
            })
            .unwrap();
        append_payload_events(
            &mut authority,
            session_id,
            1,
            extra_events,
            30_000 + u128::from(extra_events) * 100,
        );
        authority.verify_integrity().unwrap();

        let through = 1 + extra_events;
        let checkpoint_id = CheckpointId(40_000 + u128::from(extra_events));
        let mut checkpoints = CheckpointManager::open(
            authority_layout.authority_db_path(),
            &runtime.artifact_dir,
        )
        .unwrap();
        let replay_before = checkpoints.replay_projection(session_id, through).unwrap();
        let created = checkpoints
            .create(
                &mut authority,
                CreateCheckpoint {
                    checkpoint_id,
                    created_event_id: EventId(50_000 + u128::from(extra_events)),
                    session_id,
                    through_session_seq: through,
                    actor_principal: "qualification",
                    recorded_at: "2026-08-26T09:01:00Z",
                },
            )
            .unwrap();

        let loaded = checkpoints
            .load_or_replay(checkpoint_id, session_id, through)
            .unwrap();
        assert_eq!(loaded.source, ProjectionSource::Checkpoint);
        assert_eq!(loaded.bytes, replay_before);

        let artifact_path = runtime.artifact_dir.join(&created.artifact.relative_path);
        fs::remove_file(artifact_path).unwrap();
        let fallback = checkpoints
            .load_or_replay(checkpoint_id, session_id, through)
            .unwrap();
        assert_eq!(fallback.source, ProjectionSource::ReplayFallback);
        assert_eq!(fallback.bytes, replay_before);

        drop(checkpoints);
        drop(authority);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}

#[test]
fn fork_anchor_is_immutable_across_multiple_parent_prefixes() {
    for through in 1_u64..=4 {
        let runtime = runtime("fork");
        let authority_layout = AuthorityLayout::initialize(&runtime).unwrap();
        let parent = SessionId(60_000 + u128::from(through));
        let child = SessionId(70_000 + u128::from(through));
        let mut authority = AuthorityStore::open(authority_layout.authority_db_path()).unwrap();
        authority
            .create_session(CreateSession {
                session_id: parent,
                event_id: EventId(80_000 + u128::from(through)),
                owner_principal: "owner",
                actor_principal: "owner",
                recorded_at: "2026-08-26T09:10:00Z",
                payload: b"parent-created",
                security_critical: true,
            })
            .unwrap();
        append_payload_events(&mut authority, parent, 1, 3, 90_000 + u128::from(through) * 100);

        let mut forks = ForkManager::open(authority_layout.authority_db_path()).unwrap();
        let created = forks
            .create(CreateFork {
                child_session_id: child,
                event_id: EventId(100_000 + u128::from(through)),
                parent_session_id: parent,
                through_session_seq: through,
                actor_principal: "qualification",
                recorded_at: "2026-08-26T09:11:00Z",
            })
            .unwrap();
        let anchor_before = forks.anchor(child).unwrap().unwrap();
        assert_eq!(anchor_before, created.anchor);
        assert_eq!(anchor_before.parent_session_id, parent);
        assert_eq!(anchor_before.parent_session_seq, through);
        forks.verify_all().unwrap();

        authority
            .append_event(AppendEvent {
                event_id: EventId(110_000 + u128::from(through)),
                session_id: parent,
                expected_session_seq: 4,
                kind: EventKind::GoalVersioned,
                actor_principal: "owner",
                recorded_at: "2026-08-26T09:12:00Z",
                payload: b"parent-continues-after-fork",
                security_critical: true,
            })
            .unwrap();
        assert_eq!(forks.anchor(child).unwrap().unwrap(), anchor_before);
        forks.verify_all().unwrap();
        authority.verify_integrity().unwrap();

        drop(forks);
        drop(authority);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}

#[test]
fn hash_chain_is_deterministic_and_parent_sensitive_across_corpus() {
    let mut previous_event_hash = None;
    let mut previous_audit_hash = None;

    for seed in 0_u8..96 {
        let record = EventRecord {
            event_id: EventId(120_000 + u128::from(seed)),
            session_id: SessionId(121_000),
            global_seq: u64::from(seed) + 1,
            session_seq: u64::from(seed) + 1,
            schema_version: SCHEMA_VERSION,
            kind: EventKind::GoalVersioned,
            actor_principal: "qualification".to_owned(),
            recorded_at: format!("2026-08-26T09:{:02}:00Z", seed % 60),
            payload_hash: payload_hash(&[seed; 17]),
            previous_session_event_hash: previous_event_hash,
            security_critical: true,
            previous_audit_hash,
        };
        let event_hash = event_integrity_hash(&record).unwrap();
        let audit_hash = audit_integrity_hash(&record, event_hash).unwrap();
        assert_eq!(event_hash, event_integrity_hash(&record).unwrap());
        assert_eq!(audit_hash, audit_integrity_hash(&record, event_hash).unwrap());

        let mut changed_parent = record.clone();
        changed_parent.previous_session_event_hash = Some([seed.wrapping_add(1); 32]);
        assert_ne!(event_hash, event_integrity_hash(&changed_parent).unwrap());

        let mut changed_audit_parent = record.clone();
        changed_audit_parent.previous_audit_hash = Some([seed.wrapping_add(2); 32]);
        assert_ne!(
            audit_hash,
            audit_integrity_hash(&changed_audit_parent, event_hash).unwrap()
        );

        previous_event_hash = Some(event_hash);
        previous_audit_hash = Some(audit_hash);
    }
}
