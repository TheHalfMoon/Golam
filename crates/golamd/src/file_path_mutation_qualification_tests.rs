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

use crate::file_path_mutation::{
    PathMutationError, execute_file_delete, execute_file_rename, file_delete_payload_hash,
    file_delete_preconditions_hash, file_delete_resource, file_rename_payload_hash,
    file_rename_preconditions_hash, file_rename_resource,
};
use crate::local_fs::{LocalFsResolutionError, LocalFsResolver};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

struct AllowPhaseF;

impl AuthorizationPolicy for AllowPhaseF {
    fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
        PolicyDecision::allow("phase_f_path_mutation_qualification")
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
            RequestedOperationId::new("file.delete").unwrap(),
            RequestedOperationId::new("file.rename").unwrap(),
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
                Principal::test("phase-f-path"),
                KernelCreateSession {
                    session_id: SessionId(17),
                    event_id: EventId(1),
                    recorded_at: "2026-09-04T12:15:00Z",
                    payload: b"phase-f-path-mutation-qualification",
                },
                "phase-f-path",
            )
            .unwrap();
        Self {
            base,
            workspace,
            resolver,
            kernel,
        }
    }

    fn source_expectation(
        &self,
        requested: &RequestedTarget,
        content: &[u8],
        operation: &str,
    ) -> FileMutationExpectation {
        let operation = RequestedOperationId::new(operation).unwrap();
        let target = self
            .resolver
            .resolve_read_target(requested, &operation, 10)
            .unwrap();
        assert_eq!(target.file_kind, ObservedFileKind::RegularFile);
        let parent = self
            .resolver
            .resolve_read_target(&RequestedTarget::new(".").unwrap(), &operation, 10)
            .unwrap();
        FileMutationExpectation {
            expected_exists: true,
            expected_kind: Some(ObservedFileKind::RegularFile),
            expected_identity: target.resolved_target_identity,
            expected_content_digest: Some(BindingDigest::new(sha256(content))),
            expected_size: Some(content.len() as u64),
            expected_parent_identity: parent.resolved_target_identity,
        }
    }

    fn root_identity(&self, operation: &str) -> BindingDigest {
        self.resolver
            .resolve_read_target(
                &RequestedTarget::new(".").unwrap(),
                &RequestedOperationId::new(operation).unwrap(),
                10,
            )
            .unwrap()
            .resolved_target_identity
            .unwrap()
    }

    fn prepare_rename(
        &mut self,
        effect_id: u128,
        source: &RequestedTarget,
        destination: &RequestedTarget,
        expectation: FileMutationExpectation,
        destination_parent_identity: BindingDigest,
    ) -> PreparedToolEffect {
        let resource = file_rename_resource(source, destination);
        self.kernel
            .prepare_tool_effect(
                Principal::test("phase-f-path"),
                PrepareToolEffect {
                    effect_id: EffectId(effect_id),
                    session_id: SessionId(17),
                    action: "file.rename",
                    resource: &resource,
                    execution_semantics: "at_most_once",
                    handler_id: "golam-fs-unix",
                    handler_version: "1",
                    idempotency_key: Some("phase-f-rename-qualification"),
                    preconditions_hash: file_rename_preconditions_hash(
                        source,
                        destination,
                        expectation,
                        destination_parent_identity,
                    )
                    .unwrap(),
                    payload_hash: file_rename_payload_hash(destination),
                    started_at: "2026-09-04T12:15:01Z",
                },
                "phase-f-path",
            )
            .unwrap()
    }

    fn prepare_delete(
        &mut self,
        effect_id: u128,
        source: &RequestedTarget,
        expectation: FileMutationExpectation,
    ) -> PreparedToolEffect {
        let resource = file_delete_resource(source);
        self.kernel
            .prepare_tool_effect(
                Principal::test("phase-f-path"),
                PrepareToolEffect {
                    effect_id: EffectId(effect_id),
                    session_id: SessionId(17),
                    action: "file.delete",
                    resource: &resource,
                    execution_semantics: "at_most_once",
                    handler_id: "golam-fs-unix",
                    handler_version: "1",
                    idempotency_key: Some("phase-f-delete-qualification"),
                    preconditions_hash: file_delete_preconditions_hash(source, expectation)
                        .unwrap(),
                    payload_hash: file_delete_payload_hash(),
                    started_at: "2026-09-04T12:15:01Z",
                },
                "phase-f-path",
            )
            .unwrap()
    }

    fn complete(&mut self, prepared: &PreparedToolEffect, completion: ToolExecutionCompletion) {
        self.kernel
            .complete_tool_effect(
                Principal::test("phase-f-path"),
                CompleteToolEffect {
                    prepared,
                    finished_at: "2026-09-04T12:15:02Z",
                    completion,
                    reason_code: Some("phase_f_path_mutation_qualification"),
                    evidence_ref: None,
                    receipt: None,
                },
                "phase-f-path",
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
        "golam-phase-f-path-{}-{nanos}-{counter}",
        std::process::id()
    ))
}

#[test]
fn rename_then_delete_require_effect_bound_identity_and_verify_terminal_state() {
    let mut fixture = Fixture::new();
    fs::write(fixture.workspace.join("source.txt"), b"alpha").unwrap();
    let source = RequestedTarget::new("source.txt").unwrap();
    let destination = RequestedTarget::new("destination.txt").unwrap();

    let rename_expectation = fixture.source_expectation(&source, b"alpha", "file.rename");
    let destination_parent = fixture.root_identity("file.rename");
    let rename = fixture.prepare_rename(
        610,
        &source,
        &destination,
        rename_expectation,
        destination_parent,
    );
    let receipt = execute_file_rename(
        &fixture.resolver,
        &rename,
        &source,
        &destination,
        rename_expectation,
        destination_parent,
        11,
    )
    .unwrap();
    assert_eq!(receipt.effect_id, EffectId(610));
    assert!(!fixture.workspace.join("source.txt").exists());
    assert_eq!(
        fs::read(fixture.workspace.join("destination.txt")).unwrap(),
        b"alpha"
    );
    fixture.complete(&rename, ToolExecutionCompletion::Succeeded);

    let delete_expectation = fixture.source_expectation(&destination, b"alpha", "file.delete");
    let delete = fixture.prepare_delete(620, &destination, delete_expectation);
    let receipt = execute_file_delete(
        &fixture.resolver,
        &delete,
        &destination,
        delete_expectation,
        12,
    )
    .unwrap();
    assert_eq!(receipt.effect_id, EffectId(620));
    assert!(!fixture.workspace.join("destination.txt").exists());
    fixture.complete(&delete, ToolExecutionCompletion::Succeeded);
}

#[test]
fn rename_denies_existing_destination_and_stale_parent_without_mutating_source() {
    let mut fixture = Fixture::new();
    fs::write(fixture.workspace.join("source.txt"), b"stable").unwrap();
    fs::write(fixture.workspace.join("occupied.txt"), b"occupied").unwrap();
    let source = RequestedTarget::new("source.txt").unwrap();
    let occupied = RequestedTarget::new("occupied.txt").unwrap();
    let expectation = fixture.source_expectation(&source, b"stable", "file.rename");
    let parent = fixture.root_identity("file.rename");

    let prepared = fixture.prepare_rename(630, &source, &occupied, expectation, parent);
    assert!(matches!(
        execute_file_rename(
            &fixture.resolver,
            &prepared,
            &source,
            &occupied,
            expectation,
            parent,
            20,
        ),
        Err(PathMutationError::DestinationExists)
    ));
    assert_eq!(
        fs::read(fixture.workspace.join("source.txt")).unwrap(),
        b"stable"
    );
    fixture.complete(&prepared, ToolExecutionCompletion::Failed);

    let destination = RequestedTarget::new("fresh.txt").unwrap();
    let stale_parent = BindingDigest::new([0x77; 32]);
    let prepared = fixture.prepare_rename(640, &source, &destination, expectation, stale_parent);
    assert!(matches!(
        execute_file_rename(
            &fixture.resolver,
            &prepared,
            &source,
            &destination,
            expectation,
            stale_parent,
            21,
        ),
        Err(PathMutationError::StaleParent)
    ));
    assert_eq!(
        fs::read(fixture.workspace.join("source.txt")).unwrap(),
        b"stable"
    );
    assert!(!fixture.workspace.join("fresh.txt").exists());
    fixture.complete(&prepared, ToolExecutionCompletion::Failed);
}

#[test]
fn prepared_rename_rejects_symlink_substitution_without_touching_preserved_source() {
    use std::os::unix::fs::symlink;

    let mut fixture = Fixture::new();
    let source_path = fixture.workspace.join("source.txt");
    let preserved_path = fixture.workspace.join("preserved.txt");
    fs::write(&source_path, b"authority-bound").unwrap();
    let source = RequestedTarget::new("source.txt").unwrap();
    let destination = RequestedTarget::new("destination.txt").unwrap();
    let expectation = fixture.source_expectation(&source, b"authority-bound", "file.rename");
    let parent = fixture.root_identity("file.rename");
    let prepared = fixture.prepare_rename(650, &source, &destination, expectation, parent);

    fs::rename(&source_path, &preserved_path).unwrap();
    symlink("preserved.txt", &source_path).unwrap();

    assert!(matches!(
        execute_file_rename(
            &fixture.resolver,
            &prepared,
            &source,
            &destination,
            expectation,
            parent,
            30,
        ),
        Err(PathMutationError::Resolution(
            LocalFsResolutionError::AliasBoundary { .. }
        ))
    ));
    assert_eq!(fs::read(&preserved_path).unwrap(), b"authority-bound");
    assert!(!fixture.workspace.join("destination.txt").exists());
    fixture.complete(&prepared, ToolExecutionCompletion::Failed);
}

#[test]
fn prepared_rename_rejects_source_inode_swap_without_committing_destination() {
    let mut fixture = Fixture::new();
    let source_path = fixture.workspace.join("source.txt");
    let preserved_path = fixture.workspace.join("preserved.txt");
    fs::write(&source_path, b"original").unwrap();
    let source = RequestedTarget::new("source.txt").unwrap();
    let destination = RequestedTarget::new("destination.txt").unwrap();
    let expectation = fixture.source_expectation(&source, b"original", "file.rename");
    let parent = fixture.root_identity("file.rename");
    let prepared = fixture.prepare_rename(660, &source, &destination, expectation, parent);

    fs::rename(&source_path, &preserved_path).unwrap();
    fs::write(&source_path, b"substituted").unwrap();

    assert!(matches!(
        execute_file_rename(
            &fixture.resolver,
            &prepared,
            &source,
            &destination,
            expectation,
            parent,
            31,
        ),
        Err(PathMutationError::StaleSource)
    ));
    assert_eq!(fs::read(&preserved_path).unwrap(), b"original");
    assert_eq!(fs::read(&source_path).unwrap(), b"substituted");
    assert!(!fixture.workspace.join("destination.txt").exists());
    fixture.complete(&prepared, ToolExecutionCompletion::Failed);
}
