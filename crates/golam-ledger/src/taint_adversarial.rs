#![forbid(unsafe_code)]

use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;
use golam_core::taint::{
    CanonicalMemoryAdmissionError, Provenanced, TaintLabel, TaintSet,
    validate_canonical_long_term_memory_admission,
};
use golam_core::{EffectId, EffectTransitionId, EventId, SessionId};
use rusqlite::{Connection, TransactionBehavior, params};

use crate::authority_security_write::append_verifier_rule_snapshot;
use crate::authorization::{
    AppendAuthorizationDecision, AuthorizationAuditLog, AuthorizationDecisionEvidence,
    AuthorizationDecisionKind,
};
use crate::dispatch::encode_effect_dependencies;
use crate::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
use crate::storage::AuthorityStore;
use crate::taint_attestation::{
    DeterministicVerifierEvidence, SecretEliminationSanitizerEvidence, TAINT_DOWNGRADE_ACTION,
    TAINT_SECRET_ELIMINATION_ACTION, TaintAttestationError, TaintAttestationStore,
    prepare_deterministic_verifier_downgrade, prepare_human_downgrade,
    prepare_secret_elimination_sanitizer,
};
use crate::verifier_registry::TAINT_AUTHORITY_MUTATION_RISK_CLASS;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);
static RECORD_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_id() -> u128 {
    8_000_000 + u128::from(RECORD_COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn authority() -> (RuntimeLayout, AuthorityLayout) {
    let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
        "golam-taint-adversarial-{}-{t}-{n}",
        std::process::id()
    )))
    .unwrap();
    let authority = AuthorityLayout::initialize(&runtime).unwrap();
    (runtime, authority)
}

#[allow(clippy::too_many_arguments)]
fn create_authorized_effect(
    authority: &AuthorityLayout,
    effect_id: EffectId,
    action: &str,
    resource: &str,
    payload_hash: [u8; 32],
    reason: &str,
) {
    let dependencies = encode_effect_dependencies(&[]).unwrap();
    let mut store = EffectStore::open(authority).unwrap();
    store
        .propose(ProposeEffect {
            effect_id,
            session_id: SessionId(1),
            requested_by: "owner:owner",
            action,
            resource,
            risk_class: TAINT_AUTHORITY_MUTATION_RISK_CLASS,
            execution_semantics: "at_most_once",
            idempotency_key: None,
            preconditions: b"[]",
            dependencies: &dependencies,
            payload_hash,
            proposed_event_id: EventId(next_id()),
            transition_id: EffectTransitionId(next_id()),
        })
        .unwrap();
    store
        .compare_and_swap(CompareAndSwapEffect {
            transition_id: EffectTransitionId(next_id()),
            effect_id,
            expected_state: "proposed",
            next_state: "authorized",
            attempt_id: None,
            reason_code: Some(reason),
            evidence_ref: None,
            event_id: EventId(next_id()),
        })
        .unwrap();
}

fn append_allow(authority: &AuthorityLayout, action: &str, resource: &str) -> [u8; 16] {
    AuthorizationAuditLog::open(authority)
        .unwrap()
        .append(AppendAuthorizationDecision {
            principal: "owner:owner",
            action,
            resource,
            context: "scope=local-owner",
            evidence: AuthorizationDecisionEvidence::hard_guard_only("pass"),
            decision: AuthorizationDecisionKind::Allow,
            reason_code: "test_taint_adversarial_authority",
        })
        .unwrap()
        .decision_id
}

fn install_active_sanitizer(
    authority: &AuthorityLayout,
    rule_id: [u8; 16],
    binding: &[u8],
    allowed: TaintSet,
) {
    let store = AuthorityStore::open(authority.authority_db_path()).unwrap();
    drop(store);
    let mut connection = Connection::open(authority.authority_db_path()).unwrap();
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    transaction
        .execute(
            "INSERT INTO verifier_rules (rule_id, kind, version, authority_source_binding, allowed_downgrades, registered_by, status, created_global_seq) VALUES (?1, 'secret_elimination_sanitizer', 1, ?2, ?3, 'owner:owner', 'active', 1)",
            params![&rule_id[..], binding, allowed.canonical_bytes().unwrap()],
        )
        .unwrap();
    append_verifier_rule_snapshot(&transaction, &rule_id).unwrap();
    crate::authority_security_v2::verify(&transaction).unwrap();
    transaction.commit().unwrap();
}

#[test]
fn property_memory_sink_rejects_exactly_every_secret_derived_combination() {
    for mask in 0_u16..(1_u16 << TaintLabel::ALL.len()) {
        let labels = TaintLabel::ALL
            .into_iter()
            .enumerate()
            .filter_map(|(index, label)| ((mask & (1_u16 << index)) != 0).then_some(label));
        let taint = TaintSet::from_labels(labels);
        let result = validate_canonical_long_term_memory_admission(taint);
        if taint.contains(TaintLabel::SecretDerived) {
            assert_eq!(result, Err(CanonicalMemoryAdmissionError::SecretDerived));
        } else {
            assert_eq!(result, Ok(()));
        }
    }
}

#[test]
fn multi_hop_derivation_preserves_all_sources_and_secret_dominance() {
    let web_secret = TaintSet::from_labels([TaintLabel::WebUntrusted, TaintLabel::SecretDerived]);
    let local = TaintSet::from_labels([TaintLabel::LocalTrusted]);
    let generated = TaintSet::from_labels([TaintLabel::ModelGenerated]);
    let plugin = TaintSet::from_labels([TaintLabel::PluginUnverified]);

    let hop_one = Provenanced::derive("hop-one", [web_secret], generated);
    let hop_two = Provenanced::derive("hop-two", [hop_one.taint(), local], plugin);
    let required = web_secret.union(generated).union(local).union(plugin);

    assert!(hop_two.taint().contains_all(required));
    assert_eq!(
        validate_canonical_long_term_memory_admission(hop_two.taint()),
        Err(CanonicalMemoryAdmissionError::SecretDerived)
    );
}

#[test]
fn normal_human_and_verifier_paths_cannot_self_clear_secret_derived() {
    let source = TaintSet::from_labels([TaintLabel::SecretDerived, TaintLabel::ModelGenerated]);
    let result = TaintSet::from_labels([TaintLabel::ModelGenerated]);

    assert!(matches!(
        prepare_human_downgrade([[1; 32]], source, [2; 32], result, "owner:owner", [3; 32],),
        Err(TaintAttestationError::SecretDerivedRequiresSanitizer)
    ));
    assert!(matches!(
        prepare_deterministic_verifier_downgrade(
            [[4; 32]],
            source,
            [5; 32],
            result,
            "owner:owner",
            DeterministicVerifierEvidence {
                rule_id: [6; 16],
                authority_source_binding: b"self-asserted-model-proof",
                evidence_hash: [7; 32],
            },
        ),
        Err(TaintAttestationError::SecretDerivedRequiresSanitizer)
    ));
}

#[test]
fn unregistered_sanitizer_cannot_commit_even_with_exact_effect_and_allow() {
    let (runtime, authority) = authority();
    let source = TaintSet::from_labels([TaintLabel::SecretDerived, TaintLabel::WebUntrusted]);
    let prepared = prepare_secret_elimination_sanitizer(
        [[10; 32]],
        source,
        [11; 32],
        TaintSet::from_labels([TaintLabel::WebUntrusted]),
        "owner:owner",
        SecretEliminationSanitizerEvidence {
            rule_id: [12; 16],
            authority_source_binding: b"unregistered-secret-schema:v1",
            evidence_hash: [13; 32],
        },
    )
    .unwrap();
    let effect_id = EffectId(next_id());
    create_authorized_effect(
        &authority,
        effect_id,
        TAINT_SECRET_ELIMINATION_ACTION,
        prepared.resource(),
        prepared.intent_digest(),
        "test_unregistered_sanitizer",
    );
    let decision = append_allow(
        &authority,
        TAINT_SECRET_ELIMINATION_ACTION,
        prepared.resource(),
    );

    assert!(matches!(
        TaintAttestationStore::open(&authority)
            .unwrap()
            .attest_secret_elimination_sanitizer(prepared, decision, effect_id),
        Err(TaintAttestationError::VerifierRuleNotFound)
    ));
    AuthorityStore::open(authority.authority_db_path()).unwrap();
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn successful_sanitizer_never_rewrites_or_admits_the_original_source() {
    let (runtime, authority) = authority();
    let rule_id = [20; 16];
    let binding = b"registered-secret-schema:v1";
    install_active_sanitizer(
        &authority,
        rule_id,
        binding,
        TaintSet::from_labels([TaintLabel::SecretDerived]),
    );

    let source_artifact = [21; 32];
    let result_artifact = [22; 32];
    let source_labels =
        TaintSet::from_labels([TaintLabel::SecretDerived, TaintLabel::ModelGenerated]);
    let result_labels = TaintSet::from_labels([TaintLabel::ModelGenerated]);
    let prepared = prepare_secret_elimination_sanitizer(
        [source_artifact],
        source_labels,
        result_artifact,
        result_labels,
        "owner:owner",
        SecretEliminationSanitizerEvidence {
            rule_id,
            authority_source_binding: binding,
            evidence_hash: [23; 32],
        },
    )
    .unwrap();
    let effect_id = EffectId(next_id());
    create_authorized_effect(
        &authority,
        effect_id,
        TAINT_SECRET_ELIMINATION_ACTION,
        prepared.resource(),
        prepared.intent_digest(),
        "test_registered_sanitizer",
    );
    let decision = append_allow(
        &authority,
        TAINT_SECRET_ELIMINATION_ACTION,
        prepared.resource(),
    );
    let record = TaintAttestationStore::open(&authority)
        .unwrap()
        .attest_secret_elimination_sanitizer(prepared, decision, effect_id)
        .unwrap();

    assert_eq!(record.source_artifact_ids, vec![source_artifact]);
    assert_eq!(record.result_artifact_id, result_artifact);
    assert_ne!(record.source_artifact_ids[0], record.result_artifact_id);
    assert_eq!(record.source_labels, source_labels);
    assert_eq!(record.result_labels, result_labels);
    assert_eq!(
        validate_canonical_long_term_memory_admission(record.source_labels),
        Err(CanonicalMemoryAdmissionError::SecretDerived)
    );
    assert_eq!(
        validate_canonical_long_term_memory_admission(record.result_labels),
        Ok(())
    );
    assert!(matches!(
        prepare_secret_elimination_sanitizer(
            [source_artifact],
            source_labels,
            source_artifact,
            result_labels,
            "owner:owner",
            SecretEliminationSanitizerEvidence {
                rule_id,
                authority_source_binding: binding,
                evidence_hash: [24; 32],
            },
        ),
        Err(TaintAttestationError::ResultArtifactMustBeNew)
    ));

    AuthorityStore::open(authority.authority_db_path()).unwrap();
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn normal_downgrade_action_cannot_be_substituted_for_sanitizer_action() {
    let (runtime, authority) = authority();
    let rule_id = [30; 16];
    let binding = b"action-bound-secret-schema:v1";
    install_active_sanitizer(
        &authority,
        rule_id,
        binding,
        TaintSet::from_labels([TaintLabel::SecretDerived]),
    );
    let prepared = prepare_secret_elimination_sanitizer(
        [[31; 32]],
        TaintSet::from_labels([TaintLabel::SecretDerived]),
        [32; 32],
        TaintSet::empty(),
        "owner:owner",
        SecretEliminationSanitizerEvidence {
            rule_id,
            authority_source_binding: binding,
            evidence_hash: [33; 32],
        },
    )
    .unwrap();
    let effect_id = EffectId(next_id());
    create_authorized_effect(
        &authority,
        effect_id,
        TAINT_DOWNGRADE_ACTION,
        prepared.resource(),
        prepared.intent_digest(),
        "test_wrong_sanitizer_action",
    );
    let decision = append_allow(&authority, TAINT_DOWNGRADE_ACTION, prepared.resource());

    assert!(matches!(
        TaintAttestationStore::open(&authority)
            .unwrap()
            .attest_secret_elimination_sanitizer(prepared, decision, effect_id),
        Err(TaintAttestationError::AuthorityDecisionMismatch)
    ));
    AuthorityStore::open(authority.authority_db_path()).unwrap();
    fs::remove_dir_all(runtime.root).unwrap();
}
