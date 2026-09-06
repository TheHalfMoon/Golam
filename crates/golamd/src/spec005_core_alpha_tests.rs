#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use golam_core::context_compiler::{
    ContextCompilerPlan, L0RetrievedEvidence, L0SourceRoute, compile_l0_context,
};
use golam_core::context_evidence::{
    ContextEvidence, EvidenceAuthorityClass, EvidenceRequirement, EvidenceSourceId,
    EvidenceSourceKind, FreshnessPolicy, PermissionScopeId, SufficiencyState,
};
use golam_core::digest::sha256;
use golam_core::paths::RuntimeLayout;
use golam_core::taint::TaintSet;
use golam_core::target_identity::{FileMutationExpectation, ObservedFileKind};
use golam_core::tool_request::{
    BindingDigest, RequestedOperationId, RequestedTarget, ResourceClassId,
};
use golam_core::{EffectId, EventId, SessionId};
use golam_kernel::{
    AuthorizationPolicy, AuthorizationRequest, CompleteToolEffect, KernelApi, KernelCreateSession,
    PolicyDecision, PrepareToolEffect, Principal, ToolExecutionCompletion,
};

use crate::file_mutation::{
    FileWriteMode, execute_file_write, file_mutation_resource, file_preconditions_hash,
};
use crate::local_fs::LocalFsResolver;
use crate::local_read::{LocalFileReadBounds, read_regular_file};
use crate::local_search::{LocalTextSearchBounds, search_literal_text};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

struct AllowCoreAlpha;

impl AuthorizationPolicy for AllowCoreAlpha {
    fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
        PolicyDecision::allow("spec005_core_alpha")
    }
}

fn digest(value: u8) -> BindingDigest {
    BindingDigest::new([value; 32])
}

fn unique_root() -> PathBuf {
    let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "golam-spec005-core-alpha-{}-{nanos}-{counter}",
        std::process::id()
    ))
}

#[test]
fn repository_read_search_context_authorized_edit_and_readback_converge() {
    let base = unique_root();
    let workspace = base.join("repository");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir(workspace.join(".git")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        b"pub const TODO_ALPHA: &str = \"pending\";\n",
    )
    .unwrap();

    let runtime = RuntimeLayout::initialize(base.join("runtime")).unwrap();
    let mut operations = vec![
        RequestedOperationId::new("file.write").unwrap(),
        RequestedOperationId::new("list").unwrap(),
        RequestedOperationId::new("read").unwrap(),
    ];
    operations.sort();
    let resolver = LocalFsResolver::new(
        &workspace,
        ResourceClassId::new("workspace.core-alpha").unwrap(),
        operations,
        [runtime.root.clone()],
    )
    .unwrap();

    let read_operation = RequestedOperationId::new("read").unwrap();
    let list_operation = RequestedOperationId::new("list").unwrap();
    let write_operation = RequestedOperationId::new("file.write").unwrap();
    let target = RequestedTarget::new("src/lib.rs").unwrap();

    let initial = read_regular_file(
        &resolver,
        &target,
        &read_operation,
        LocalFileReadBounds {
            max_bytes: 4096,
            max_duration: Duration::from_secs(2),
        },
        10,
        10,
    )
    .unwrap();
    assert_eq!(
        initial.bytes,
        b"pub const TODO_ALPHA: &str = \"pending\";\n"
    );

    let search = search_literal_text(
        &resolver,
        &RequestedTarget::new(".").unwrap(),
        &list_operation,
        &read_operation,
        "TODO_ALPHA",
        LocalTextSearchBounds {
            max_walk_entries: 64,
            max_files: 32,
            max_matches: 8,
            max_file_bytes: 4096,
            max_total_bytes: 16 * 1024,
            max_line_bytes: 4096,
            max_depth: 8,
            max_duration: Duration::from_secs(2),
        },
        11,
    )
    .unwrap();
    assert_eq!(search.matches.len(), 1);
    assert_eq!(search.matches[0].content_digest, initial.content_digest);

    let scope = PermissionScopeId(digest(50));
    let requirement = EvidenceRequirement {
        requirement_id: digest(51),
        allowed_source_kinds: vec![EvidenceSourceKind::File],
        allowed_authority_classes: vec![EvidenceAuthorityClass::LocalObserved],
        forbidden_taint: TaintSet::empty(),
        required_permission_scope: Some(scope),
        minimum_observed_at_unix_ms: Some(10),
    };
    let plan = ContextCompilerPlan {
        intent_ref: digest(52),
        requirements: vec![requirement],
        allowed_routes: vec![L0SourceRoute::FileRead, L0SourceRoute::InProcessSearch],
        max_evidence_items: 2,
        max_replans: 1,
        projection_policy_ref: digest(53),
        created_at_unix_ms: 10,
    };
    let read_evidence = ContextEvidence {
        evidence_id: digest(54),
        source_id: EvidenceSourceId(digest(55)),
        source_kind: EvidenceSourceKind::File,
        source_version_or_observation: initial.content_digest,
        content_ref: initial.content_digest,
        content_digest: initial.content_digest,
        authority_class: EvidenceAuthorityClass::LocalObserved,
        taint_set: TaintSet::empty(),
        permission_scope: scope,
        freshness_policy: FreshnessPolicy::Immutable,
        observed_at_unix_ms: 10,
        supersedes_or_conflicts_with: Vec::new(),
    };
    let search_evidence = ContextEvidence {
        evidence_id: digest(56),
        source_id: EvidenceSourceId(digest(57)),
        source_kind: EvidenceSourceKind::File,
        source_version_or_observation: search.matches[0].content_digest,
        content_ref: search.matches[0].content_digest,
        content_digest: search.matches[0].content_digest,
        authority_class: EvidenceAuthorityClass::LocalObserved,
        taint_set: TaintSet::empty(),
        permission_scope: scope,
        freshness_policy: FreshnessPolicy::Immutable,
        observed_at_unix_ms: 11,
        supersedes_or_conflicts_with: Vec::new(),
    };
    let compiled = compile_l0_context(
        &plan,
        &[
            L0RetrievedEvidence {
                route: L0SourceRoute::FileRead,
                evidence: read_evidence,
                bounded_score: 1,
            },
            L0RetrievedEvidence {
                route: L0SourceRoute::InProcessSearch,
                evidence: search_evidence,
                bounded_score: 2,
            },
        ],
        0,
        11,
    )
    .unwrap();
    assert_eq!(
        compiled.capsule.sufficiency_state,
        SufficiencyState::Sufficient
    );
    assert!(compiled.replan.is_none());

    let target_identity = resolver
        .resolve_read_target(&target, &write_operation, 12)
        .unwrap();
    let parent_identity = resolver
        .resolve_read_target(&RequestedTarget::new("src").unwrap(), &write_operation, 12)
        .unwrap();
    let expectation = FileMutationExpectation {
        expected_exists: true,
        expected_kind: Some(ObservedFileKind::RegularFile),
        expected_identity: target_identity.resolved_target_identity,
        expected_content_digest: Some(initial.content_digest),
        expected_size: Some(initial.bytes.len() as u64),
        expected_parent_identity: parent_identity.resolved_target_identity,
    };

    let mut kernel = KernelApi::open(&runtime, AllowCoreAlpha).unwrap();
    kernel
        .create_session(
            Principal::test("spec005-core-alpha"),
            KernelCreateSession {
                session_id: SessionId(700),
                event_id: EventId(1),
                recorded_at: "2026-09-06T15:20:00Z",
                payload: b"spec005-core-alpha",
            },
            "spec005-core-alpha",
        )
        .unwrap();

    let new_bytes = b"pub const TODO_ALPHA: &str = \"done\";\n";
    let resource = file_mutation_resource(&target);
    let prepared = kernel
        .prepare_tool_effect(
            Principal::test("spec005-core-alpha"),
            PrepareToolEffect {
                effect_id: EffectId(701),
                session_id: SessionId(700),
                action: FileWriteMode::Write.action(),
                resource: &resource,
                execution_semantics: "at_most_once",
                handler_id: "golam-fs-unix",
                handler_version: "1",
                idempotency_key: Some("spec005-core-alpha"),
                preconditions_hash: file_preconditions_hash(
                    FileWriteMode::Write,
                    &target,
                    expectation,
                )
                .unwrap(),
                payload_hash: sha256(new_bytes),
                started_at: "2026-09-06T15:20:01Z",
            },
            "spec005-core-alpha",
        )
        .unwrap();

    let receipt = execute_file_write(
        &resolver,
        &prepared,
        FileWriteMode::Write,
        &target,
        expectation,
        new_bytes,
        13,
    )
    .unwrap();
    assert_eq!(receipt.effect_id, EffectId(701));

    let readback = read_regular_file(
        &resolver,
        &target,
        &read_operation,
        LocalFileReadBounds {
            max_bytes: 4096,
            max_duration: Duration::from_secs(2),
        },
        14,
        14,
    )
    .unwrap();
    assert_eq!(readback.bytes, new_bytes);
    assert_eq!(
        readback.content_digest,
        BindingDigest::new(sha256(new_bytes))
    );

    kernel
        .complete_tool_effect(
            Principal::test("spec005-core-alpha"),
            CompleteToolEffect {
                prepared: &prepared,
                finished_at: "2026-09-06T15:20:02Z",
                completion: ToolExecutionCompletion::Succeeded,
                reason_code: Some("spec005_core_alpha"),
                evidence_ref: Some(&compiled.capsule.capsule_id.bytes()),
                receipt: None,
            },
            "spec005-core-alpha",
        )
        .unwrap();

    fs::remove_dir_all(base).unwrap();
}
