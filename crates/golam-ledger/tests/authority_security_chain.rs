#![forbid(unsafe_code)]

use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::authority::{AuthorityLayout, AuthorityPath};
use golam_core::paths::RuntimeLayout;
use golam_core::{ClientId, EffectId, EffectTransitionId, EventId, SessionId};
use golam_ledger::authorization::{
    AppendAuthorizationDecision, AuthorizationAuditLog, AuthorizationDecisionKind,
};
use golam_ledger::clients::{AssuranceClass, ClientKind, ClientRegistry, EnrollClient};
use golam_ledger::dispatch::encode_effect_dependencies;
use golam_ledger::effects::{EffectStore, ProposeEffect};
use golam_ledger::protocol_audit::{
    AppendProtocolRejection, ProtocolAuditLog, ProtocolRejectionReason,
};
use golam_ledger::storage::{AuthorityStore, StorageError};
use rusqlite::{Connection, params};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn test_authority(prefix: &str) -> (RuntimeLayout, AuthorityLayout) {
    let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
        "golam-authority-security-{prefix}-{}-{nanos}-{counter}",
        std::process::id()
    )))
    .unwrap();
    let authority = AuthorityLayout::initialize(&runtime).unwrap();
    (runtime, authority)
}

fn assert_integrity_rejects_tamper(authority: &AuthorityLayout) {
    assert!(matches!(
        AuthorityStore::open(authority.authority_db_path()),
        Err(StorageError::IntegrityCheckFailed(_))
    ));
}

#[test]
fn authorization_row_tamper_is_detected_on_reopen() {
    let (runtime, authority) = test_authority("authorization");
    let mut log = AuthorizationAuditLog::open(&authority).unwrap();
    let decision = log
        .append(AppendAuthorizationDecision {
            principal: "owner:local",
            action: "session.create",
            resource: "session:new",
            context: "authenticated-local",
            decision: AuthorizationDecisionKind::Allow,
            reason_code: "bootstrap_owner_session",
        })
        .unwrap();
    drop(log);

    let connection = Connection::open(authority.authority_db_path()).unwrap();
    connection
        .execute(
            "UPDATE authorization_decisions SET reason_code = 'tampered' WHERE decision_id = ?1",
            params![&decision.decision_id[..]],
        )
        .unwrap();
    drop(connection);

    assert_integrity_rejects_tamper(&authority);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn effect_intent_tamper_is_detected_on_reopen() {
    let (runtime, authority) = test_authority("effect");
    let dependencies = encode_effect_dependencies(&[]).unwrap();
    let effect_id = EffectId(41);
    let mut effects = EffectStore::open(&authority).unwrap();
    effects
        .propose(ProposeEffect {
            effect_id,
            session_id: SessionId(7),
            requested_by: "owner:local",
            action: "sim.write",
            resource: "sim:target",
            risk_class: "synthetic",
            execution_semantics: "at_most_once",
            idempotency_key: Some("stable-key"),
            preconditions: b"[]",
            dependencies: &dependencies,
            payload_hash: [4; 32],
            proposed_event_id: EventId(800),
            transition_id: EffectTransitionId(900),
        })
        .unwrap();
    drop(effects);

    let connection = Connection::open(authority.authority_db_path()).unwrap();
    connection
        .execute(
            "UPDATE effect_intents SET resource = 'sim:tampered' WHERE effect_id = ?1",
            params![&effect_id.0.to_be_bytes()[..]],
        )
        .unwrap();
    drop(connection);

    assert_integrity_rejects_tamper(&authority);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn client_enrollment_tamper_is_detected_on_reopen() {
    let (runtime, authority) = test_authority("client");
    let client_id = ClientId(51);
    let mut clients = ClientRegistry::open(&authority).unwrap();
    clients
        .enroll(EnrollClient {
            client_id,
            key_id: [7; 32],
            public_key: [9; 32],
            kind: ClientKind::Test,
            owner_principal: "owner:local",
            enrolled_at: "2026-08-26T10:00:00Z",
            assurance_class: AssuranceClass::FilesystemUserPrivateV1,
        })
        .unwrap();
    drop(clients);

    let connection = Connection::open(authority.authority_db_path()).unwrap();
    connection
        .execute(
            "UPDATE clients SET owner_principal = 'tampered' WHERE client_id = ?1",
            params![&client_id.0.to_be_bytes()[..]],
        )
        .unwrap();
    drop(connection);

    assert_integrity_rejects_tamper(&authority);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn recovery_incident_tamper_is_detected_on_reopen() {
    let (runtime, authority) = test_authority("recovery");
    let mut audit = ProtocolAuditLog::open(&authority).unwrap();
    audit
        .append_rejection(AppendProtocolRejection {
            connection_id: 77,
            client_id: ClientId(61),
            key_id: [11; 32],
            detected_at: "2026-08-26T10:01:00Z",
            reason: ProtocolRejectionReason::AuthenticationFailed,
        })
        .unwrap();
    drop(audit);

    let connection = Connection::open(authority.authority_db_path()).unwrap();
    connection
        .execute(
            "UPDATE recovery_incidents SET severity = 'tampered' WHERE kind = 'protocol'",
            [],
        )
        .unwrap();
    drop(connection);

    assert_integrity_rejects_tamper(&authority);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn missing_authority_security_record_is_detected_on_reopen() {
    let (runtime, authority) = test_authority("coverage");
    let mut log = AuthorizationAuditLog::open(&authority).unwrap();
    let decision = log
        .append(AppendAuthorizationDecision {
            principal: "owner:local",
            action: "session.create",
            resource: "session:new",
            context: "authenticated-local",
            decision: AuthorizationDecisionKind::Allow,
            reason_code: "bootstrap_owner_session",
        })
        .unwrap();
    drop(log);

    let connection = Connection::open(authority.authority_db_path()).unwrap();
    connection
        .execute(
            "DELETE FROM authority_security_audit WHERE record_kind = 'authorization_decision' AND record_id = ?1",
            params![&decision.decision_id[..]],
        )
        .unwrap();
    drop(connection);

    assert_integrity_rejects_tamper(&authority);
    fs::remove_dir_all(runtime.root).unwrap();
}
