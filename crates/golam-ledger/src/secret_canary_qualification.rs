#![forbid(unsafe_code)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;
use golam_core::{EffectId, EventId, SessionId};
use rusqlite::{Connection, TransactionBehavior, params};
use zeroize::Zeroizing;

use crate::authority_security_write::{
    append_approval_snapshot, append_authorization_decision_v2_snapshot,
};
use crate::secret_detection::{RecognizedSecretKind, recognized_secret_kind};
use crate::secret_entry::{
    PrepareDesignatedSecretEntryRequest, PreparedDesignatedSecretEntry, SecretEntryStore,
    prepare_designated_secret_entry,
};
use crate::secret_mutation::{SECRET_CREATE_ACTION, SECRET_MUTATION_RISK_CLASS};
use crate::secret_vault::{KeyProtectionError, KeyProtector};
use crate::security_audit::{
    AuthorizationAuditInput, EffectIntentAuditInput, EffectTransitionAuditInput,
    append_authorization_decision, append_effect_intent, append_effect_transition,
};
use crate::storage::{AuthorityStore, CreateSession};

const RECOGNIZED_CANARY: &[u8] = b"ghp_T003056ABCDEFGHIJKLMNOPQRSTUVWXYZ123456";
const UNKNOWN_CANARY: &[u8] = b"orchid::seven-moons::unknown-secret-shape::T003-056";
const PROBE_ROOT_ENV: &str = "GOLAM_T003_056_PROBE_ROOT";
static N: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
struct TestProtector {
    key: [u8; 32],
}

impl KeyProtector for TestProtector {
    fn load_master_key(&self) -> Result<Zeroizing<Vec<u8>>, KeyProtectionError> {
        Ok(Zeroizing::new(self.key.to_vec()))
    }

    fn store_master_key(&self, _key: &[u8]) -> Result<(), KeyProtectionError> {
        Err(KeyProtectionError::Unsupported)
    }
}

struct WorkIds {
    effect: EffectId,
    decision: [u8; 16],
    approval: [u8; 16],
}

fn authority(label: &str) -> (RuntimeLayout, AuthorityLayout) {
    let n = N.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
        "golam-secret-canary-{label}-{}-{t}-{n}",
        std::process::id()
    )))
    .unwrap();
    let authority = AuthorityLayout::initialize(&runtime).unwrap();
    (runtime, authority)
}

fn create_session(authority: &AuthorityLayout) -> SessionId {
    let session_id = SessionId(56_001);
    let mut store = AuthorityStore::open(authority.authority_db_path()).unwrap();
    store
        .create_session(CreateSession {
            session_id,
            event_id: EventId(56_002),
            owner_principal: "owner:owner",
            actor_principal: "owner:owner",
            recorded_at: "2026-08-28T00:00:00Z",
            payload: b"t003-056-designated-secret-session",
            security_critical: false,
        })
        .unwrap();
    session_id
}

fn install_authorized_create(
    authority: &AuthorityLayout,
    prepared: &PreparedDesignatedSecretEntry,
) -> WorkIds {
    let effect = EffectId(56_100);
    let effect_bytes = effect.0.to_be_bytes();
    let transition_id = [61_u8; 16];
    let decision = [62_u8; 16];
    let approval = [63_u8; 16];
    let session_id = [64_u8; 16];
    let proposed_event_id = [65_u8; 16];
    let transition_event_id = [66_u8; 16];
    let mut connection = Connection::open(authority.authority_db_path()).unwrap();
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    transaction
        .execute(
            "INSERT INTO effect_intents (effect_id, session_id, requested_by, action, resource, risk_class, execution_semantics, idempotency_key, preconditions, dependencies, payload_hash, proposed_event_id) VALUES (?1, ?2, 'owner:owner', ?3, ?4, ?5, 'at_most_once', NULL, X'', X'', ?6, ?7)",
            params![
                &effect_bytes[..],
                &session_id[..],
                SECRET_CREATE_ACTION,
                prepared.resource(),
                SECRET_MUTATION_RISK_CLASS,
                &prepared.intent_digest()[..],
                &proposed_event_id[..],
            ],
        )
        .unwrap();
    append_effect_intent(
        &transaction,
        EffectIntentAuditInput {
            effect_id: &effect_bytes,
            session_id: &session_id,
            requested_by: "owner:owner",
            action: SECRET_CREATE_ACTION,
            resource: prepared.resource(),
            risk_class: SECRET_MUTATION_RISK_CLASS,
            execution_semantics: "at_most_once",
            idempotency_key: None,
            preconditions: b"",
            dependencies: b"",
            payload_hash: &prepared.intent_digest(),
            proposed_event_id: &proposed_event_id,
        },
    )
    .unwrap();
    transaction
        .execute(
            "INSERT INTO effect_transitions (transition_id, effect_id, global_seq, from_state, to_state, attempt_id, reason_code, evidence_ref, event_id) VALUES (?1, ?2, 2, NULL, 'authorized', NULL, NULL, NULL, ?3)",
            params![&transition_id[..], &effect_bytes[..], &transition_event_id[..]],
        )
        .unwrap();
    append_effect_transition(
        &transaction,
        EffectTransitionAuditInput {
            transition_id: &transition_id,
            effect_id: &effect_bytes,
            global_seq: 2,
            from_state: None,
            to_state: "authorized",
            attempt_id: None,
            reason_code: None,
            evidence_ref: None,
            event_id: &transition_event_id,
        },
    )
    .unwrap();
    transaction
        .execute(
            "INSERT INTO authorization_decisions (decision_id, principal, action, resource, context_hash, decision, reason_code, global_seq, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, matched_rule_ids, approval_id, authority_evidence_version) VALUES (?1, 'owner:owner', ?2, ?3, ?4, 'allow', 'test_t003_056_designated_secret', 3, 'allow', NULL, NULL, NULL, NULL, X'', ?5, 2)",
            params![
                &decision[..],
                SECRET_CREATE_ACTION,
                prepared.resource(),
                &[0_u8; 32][..],
                &approval[..],
            ],
        )
        .unwrap();
    append_authorization_decision(
        &transaction,
        AuthorizationAuditInput {
            decision_id: &decision,
            principal: "owner:owner",
            action: SECRET_CREATE_ACTION,
            resource: prepared.resource(),
            context_hash: &[0_u8; 32],
            decision: "allow",
            reason_code: "test_t003_056_designated_secret",
            global_seq: 3,
        },
    )
    .unwrap();
    append_authorization_decision_v2_snapshot(&transaction, &decision).unwrap();
    transaction
        .execute(
            "INSERT INTO approvals (approval_id, class, approver_principal, scope_digest, action_scope, resource_scope, effect_id, session_id, risk_class, taint_digest, parent_decision_id, issued_at, expires_at, max_uses, revoked_at) VALUES (?1, 'ONCE', 'owner:owner', ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8, '2026-08-28T00:00:00Z', NULL, 1, NULL)",
            params![
                &approval[..],
                &[1_u8; 32][..],
                SECRET_CREATE_ACTION.as_bytes(),
                prepared.resource().as_bytes(),
                &effect_bytes[..],
                SECRET_MUTATION_RISK_CLASS,
                &[0_u8; 32][..],
                &decision[..],
            ],
        )
        .unwrap();
    append_approval_snapshot(&transaction, &approval).unwrap();
    crate::integrity::verify(&transaction).unwrap();
    crate::authority_security_v2::verify(&transaction).unwrap();
    transaction.commit().unwrap();
    WorkIds {
        effect,
        decision,
        approval,
    }
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn durable_paths(root: &Path) -> Vec<PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let file_type = entry.file_type().unwrap();
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if file_type.is_file() {
                files.push(entry.path());
            }
        }
    }
    files.sort();
    files
}

fn assert_no_canary_in_durable_authority(authority: &AuthorityLayout, canary: &[u8]) {
    let checkpoint = Connection::open(authority.authority_db_path()).unwrap();
    checkpoint
        .execute_batch("PRAGMA wal_checkpoint(FULL);")
        .unwrap();
    drop(checkpoint);

    for path in durable_paths(authority.root()) {
        let bytes = fs::read(&path).unwrap();
        assert!(
            !contains(&bytes, canary),
            "plaintext canary appeared in durable authority file {}",
            path.display()
        );
    }
}

fn assert_no_canary_in_model_visible_history(authority: &AuthorityLayout, canary: &[u8]) {
    let connection = Connection::open(authority.authority_db_path()).unwrap();
    let mut statement = connection
        .prepare("SELECT payload_bytes FROM session_events ORDER BY global_seq")
        .unwrap();
    let rows = statement
        .query_map([], |row| row.get::<_, Vec<u8>>(0))
        .unwrap();
    for payload in rows {
        assert!(!contains(&payload.unwrap(), canary));
    }
}

fn assert_error_surface_redacts_by_construction(canary: &[u8]) {
    let (_runtime, authority) = authority("error");
    let session_id = create_session(&authority);
    let result = prepare_designated_secret_entry(PrepareDesignatedSecretEntryRequest {
        session_id,
        expected_session_seq: 1,
        event_id: EventId(56_010),
        actor_principal: "owner:\ninvalid",
        owner_principal: "owner:owner",
        recorded_at: "2026-08-28T00:01:00Z",
        classification: "api_credential",
        purpose_scope: "local.canary",
        expires_at: None,
        value: canary.to_vec(),
    });
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("invalid actor unexpectedly accepted"),
    };
    assert!(!contains(error.to_string().as_bytes(), canary));
}

fn assert_unauthorized_subprocess_cannot_observe_plaintext(
    authority: &AuthorityLayout,
    canary: &[u8],
) {
    let output = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("secret_canary_qualification::unauthorized_subprocess_probe")
        .arg("--nocapture")
        .env_clear()
        .env(PROBE_ROOT_ENV, authority.root())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(!contains(&output.stdout, canary));
    assert!(!contains(&output.stderr, canary));
}

fn qualify_canary(label: &str, canary: &[u8]) {
    let (runtime, authority) = authority(label);
    let session_id = create_session(&authority);
    let prepared = prepare_designated_secret_entry(PrepareDesignatedSecretEntryRequest {
        session_id,
        expected_session_seq: 1,
        event_id: EventId(56_003),
        actor_principal: "owner:owner",
        owner_principal: "owner:owner",
        recorded_at: "2026-08-28T00:01:00Z",
        classification: "api_credential",
        purpose_scope: "local.canary",
        expires_at: Some("2026-08-29T00:00:00Z"),
        value: canary.to_vec(),
    })
    .unwrap();
    let work = install_authorized_create(&authority, &prepared);
    let receipt = SecretEntryStore::new(&authority)
        .commit_with_protector(
            prepared,
            work.decision,
            work.approval,
            work.effect,
            TestProtector { key: [56_u8; 32] },
        )
        .unwrap();
    assert_eq!(receipt.version(), 1);

    assert_no_canary_in_model_visible_history(&authority, canary);
    assert_no_canary_in_durable_authority(&authority, canary);
    assert_error_surface_redacts_by_construction(canary);
    assert_unauthorized_subprocess_cannot_observe_plaintext(&authority, canary);

    let connection = Connection::open(authority.authority_db_path()).unwrap();
    crate::integrity::verify(&connection).unwrap();
    crate::authority_security_v2::verify(&connection).unwrap();
    drop(connection);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn recognized_and_unknown_explicit_entry_canaries_do_not_leak() {
    qualify_canary("recognized", RECOGNIZED_CANARY);
    qualify_canary("unknown", UNKNOWN_CANARY);
}

#[test]
fn free_text_recognition_is_separate_bounded_defense_in_depth() {
    let mut recognized_text = b"please use credential ".to_vec();
    recognized_text.extend_from_slice(RECOGNIZED_CANARY);
    recognized_text.extend_from_slice(b" only for this local operation");
    assert_eq!(
        recognized_secret_kind(&recognized_text).unwrap(),
        Some(RecognizedSecretKind::GitHubToken)
    );

    let mut unknown_text = b"please use credential ".to_vec();
    unknown_text.extend_from_slice(UNKNOWN_CANARY);
    assert_eq!(recognized_secret_kind(&unknown_text).unwrap(), None);
}

#[test]
fn unauthorized_subprocess_probe() {
    let Some(root) = std::env::var_os(PROBE_ROOT_ENV) else {
        return;
    };
    let mut stdout = std::io::stdout().lock();
    for path in durable_paths(Path::new(&root)) {
        stdout.write_all(&fs::read(path).unwrap()).unwrap();
    }
    stdout.flush().unwrap();
}
