#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::digest::sha256;
use golam_core::paths::RuntimeLayout;
use golam_core::target_identity::{FileMutationExpectation, ObservedFileKind};
use golam_core::tool_request::{
    BindingDigest, RequestedOperationId, RequestedTarget, ResourceClassId,
};
use golam_core::{EffectId, EventId, SessionId};
use golam_kernel::{
    AuthorizationPolicy, AuthorizationRequest, CompleteToolEffect, KernelApi, KernelCreateSession,
    PolicyDecision, PrepareToolEffect, PreparedToolEffect, Principal, ToolExecutionCompletion,
};

use crate::file_mutation::{
    FileMutationError, FileWriteMode, execute_file_write, file_mutation_resource,
    file_preconditions_hash,
};
use crate::local_fs::{LocalFsResolutionError, LocalFsResolver};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

struct AllowPhaseF;

impl AuthorizationPolicy for AllowPhaseF {
    fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
        PolicyDecision::allow("phase_f_qualification")
    }
}

struct Fixture {
    base: PathBuf,
    workspace: PathBuf,
    resolver: LocalFsResolver,
    kernel: KernelApi<AllowPhaseF>,
}

impl Fixture {
    fn new() -> Self {
        let base = unique_root();
        let workspace = base.join("workspace");
        fs::create_dir_all(&workspace).unwrap();
        let runtime = RuntimeLayout::initialize(base.join("runtime")).unwrap();
        let mut operations = vec![
            RequestedOperationId::new("file.create").unwrap(),
            RequestedOperationId::new("file.replace").unwrap(),
            RequestedOperationId::new("file.write").unwrap(),
        ];
        operations.sort();
        let resolver = LocalFsResolver::new(
            &workspace,
            ResourceClassId::new("workspace.write").unwrap(),
            operations,
            [runtime.root.clone()],
        )
        .unwrap();
        let mut kernel = KernelApi::open(&runtime, AllowPhaseF).unwrap();
        kernel
            .create_session(
                Principal::test("phase-f"),
                KernelCreateSession {
                    session_id: SessionId(7),
                    event_id: EventId(1),
                    recorded_at: "2026-09-04T11:55:00Z",
                    payload: b"phase-f-qualification",
                },
                "phase-f",
            )
            .unwrap();
        Self {
            base,
            workspace,
            resolver,
            kernel,
        }
    }

    fn expectation(
        &self,
        mode: FileWriteMode,
        requested: &RequestedTarget,
        content: Option<&[u8]>,
    ) -> FileMutationExpectation {
        let operation = RequestedOperationId::new(mode.action()).unwrap();
        let target = self
            .resolver
            .resolve_read_target(requested, &operation, 10)
            .unwrap();
        let parent = self
            .resolver
            .resolve_read_target(&RequestedTarget::new(".").unwrap(), &operation, 10)
            .unwrap();
        let parent_identity = parent.resolved_target_identity.unwrap();
        if target.file_kind == ObservedFileKind::Missing {
            FileMutationExpectation {
                expected_exists: false,
                expected_kind: None,
                expected_identity: None,
                expected_content_digest: None,
                expected_size: None,
                expected_parent_identity: Some(parent_identity),
            }
        } else {
            let content = content.expect("existing target requires expected content");
            FileMutationExpectation {
                expected_exists: true,
                expected_kind: Some(ObservedFileKind::RegularFile),
                expected_identity: target.resolved_target_identity,
                expected_content_digest: Some(BindingDigest::new(sha256(content))),
                expected_size: Some(content.len() as u64),
                expected_parent_identity: Some(parent_identity),
            }
        }
    }

    fn prepare(
        &mut self,
        effect_id: u128,
        mode: FileWriteMode,
        requested: &RequestedTarget,
        expectation: FileMutationExpectation,
        payload: &[u8],
    ) -> PreparedToolEffect {
        let resource = file_mutation_resource(requested);
        self.kernel
            .prepare_tool_effect(
                Principal::test("phase-f"),
                PrepareToolEffect {
                    effect_id: EffectId(effect_id),
                    session_id: SessionId(7),
                    action: mode.action(),
                    resource: &resource,
                    execution_semantics: "at_most_once",
                    handler_id: "golam-fs-unix",
                    handler_version: "1",
                    idempotency_key: Some("phase-f-qualification"),
                    preconditions_hash: file_preconditions_hash(mode, requested, expectation)
                        .unwrap(),
                    payload_hash: sha256(payload),
                    started_at: "2026-09-04T11:55:01Z",
                },
                "phase-f",
            )
            .unwrap()
    }

    fn complete(&mut self, prepared: &PreparedToolEffect, completion: ToolExecutionCompletion) {
        self.kernel
            .complete_tool_effect(
                Principal::test("phase-f"),
                CompleteToolEffect {
                    prepared,
                    finished_at: "2026-09-04T11:55:02Z",
                    completion,
                    reason_code: Some("phase_f_qualification"),
                    evidence_ref: None,
                    receipt: None,
                },
                "phase-f",
            )
            .unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

fn unique_root() -> PathBuf {
    let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "golam-phase-f-mutation-{}-{nanos}-{counter}",
        std::process::id()
    ))
}

#[test]
fn create_write_and_replace_require_fresh_effect_bound_state_and_verify_readback() {
    let mut fixture = Fixture::new();
    let target = RequestedTarget::new("note.txt").unwrap();

    let create_expectation = fixture.expectation(FileWriteMode::Create, &target, None);
    let create = fixture.prepare(
        100,
        FileWriteMode::Create,
        &target,
        create_expectation,
        b"alpha",
    );
    let create_receipt = execute_file_write(
        &fixture.resolver,
        &create,
        FileWriteMode::Create,
        &target,
        create_expectation,
        b"alpha",
        11,
    )
    .unwrap();
    assert_eq!(create_receipt.effect_id, EffectId(100));
    assert_eq!(
        fs::read(fixture.workspace.join("note.txt")).unwrap(),
        b"alpha"
    );
    fixture.complete(&create, ToolExecutionCompletion::Succeeded);

    let write_expectation = fixture.expectation(FileWriteMode::Write, &target, Some(b"alpha"));
    let write = fixture.prepare(
        200,
        FileWriteMode::Write,
        &target,
        write_expectation,
        b"beta",
    );
    execute_file_write(
        &fixture.resolver,
        &write,
        FileWriteMode::Write,
        &target,
        write_expectation,
        b"beta",
        12,
    )
    .unwrap();
    assert_eq!(
        fs::read(fixture.workspace.join("note.txt")).unwrap(),
        b"beta"
    );
    fixture.complete(&write, ToolExecutionCompletion::Succeeded);

    let replace_expectation = fixture.expectation(FileWriteMode::Replace, &target, Some(b"beta"));
    let replace = fixture.prepare(
        300,
        FileWriteMode::Replace,
        &target,
        replace_expectation,
        b"gamma",
    );
    let replace_receipt = execute_file_write(
        &fixture.resolver,
        &replace,
        FileWriteMode::Replace,
        &target,
        replace_expectation,
        b"gamma",
        13,
    )
    .unwrap();
    assert_eq!(replace_receipt.effect_id, EffectId(300));
    assert_eq!(
        fs::read(fixture.workspace.join("note.txt")).unwrap(),
        b"gamma"
    );
    fixture.complete(&replace, ToolExecutionCompletion::Succeeded);
}

#[test]
fn stale_content_and_parent_preconditions_deny_before_commit() {
    let mut fixture = Fixture::new();
    let path = fixture.workspace.join("note.txt");
    fs::write(&path, b"stable").unwrap();
    let target = RequestedTarget::new("note.txt").unwrap();

    let mut stale_content = fixture.expectation(FileWriteMode::Replace, &target, Some(b"stable"));
    stale_content.expected_content_digest = Some(BindingDigest::new(sha256(b"other!")));
    let prepared = fixture.prepare(
        400,
        FileWriteMode::Replace,
        &target,
        stale_content,
        b"replacement",
    );
    assert!(matches!(
        execute_file_write(
            &fixture.resolver,
            &prepared,
            FileWriteMode::Replace,
            &target,
            stale_content,
            b"replacement",
            20,
        ),
        Err(FileMutationError::StaleContent)
    ));
    assert_eq!(fs::read(&path).unwrap(), b"stable");
    fixture.complete(&prepared, ToolExecutionCompletion::Failed);

    let mut stale_parent = fixture.expectation(FileWriteMode::Write, &target, Some(b"stable"));
    stale_parent.expected_parent_identity = Some(BindingDigest::new([0x55; 32]));
    let prepared = fixture.prepare(500, FileWriteMode::Write, &target, stale_parent, b"mutated");
    assert!(matches!(
        execute_file_write(
            &fixture.resolver,
            &prepared,
            FileWriteMode::Write,
            &target,
            stale_parent,
            b"mutated",
            21,
        ),
        Err(FileMutationError::StaleParent)
    ));
    assert_eq!(fs::read(&path).unwrap(), b"stable");
    fixture.complete(&prepared, ToolExecutionCompletion::Failed);
}

#[test]
fn generic_mutation_root_cannot_overlap_protected_golam_state() {
    let base = unique_root();
    let workspace = base.join("workspace");
    let protected = workspace.join("golam-protected");
    fs::create_dir_all(&protected).unwrap();
    let operation = RequestedOperationId::new("file.write").unwrap();
    let result = LocalFsResolver::new(
        &workspace,
        ResourceClassId::new("workspace.write").unwrap(),
        vec![operation],
        [protected],
    );
    assert!(matches!(
        result,
        Err(LocalFsResolutionError::ProtectedRootOverlap(_))
    ));
    fs::remove_dir_all(base).unwrap();
}
