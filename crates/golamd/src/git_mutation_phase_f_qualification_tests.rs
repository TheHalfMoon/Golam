#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::paths::RuntimeLayout;
use golam_core::tool_request::{RequestedOperationId, ResourceClassId};
use golam_core::{EffectId, EventId, SessionId};
use golam_kernel::{
    AuthorizationPolicy, AuthorizationRequest, CompleteToolEffect, KernelApi, KernelCreateSession,
    PolicyDecision, PrepareToolEffect, Principal, ToolExecutionCompletion,
};

use crate::deflate::compress_to_vec_zlib;
use crate::git_mutation::{
    GitMutationError, GitMutationExpectation, execute_git_branch_create, git_branch_payload_hash,
    git_branch_preconditions_hash, git_branch_resource,
};
use crate::git_read::GitObjectId;
use crate::git_sha1::GitObjectSha1;
use crate::git_status::{GitStatusBounds, observe_status};
use crate::local_fs::{LocalFsResolutionError, LocalFsResolver};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

struct AllowGit;

impl AuthorizationPolicy for AllowGit {
    fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
        PolicyDecision::allow("phase_f_git_stale_state_qualification")
    }
}

struct Fixture {
    base: PathBuf,
    repo: PathBuf,
    resolver: LocalFsResolver,
    kernel: KernelApi<AllowGit>,
}

impl Fixture {
    fn new() -> Self {
        let base = unique_root();
        let repo = base.join("repo");
        initialize_repo(&repo);
        let runtime = RuntimeLayout::initialize(base.join("runtime")).unwrap();
        let mut operations = vec![
            RequestedOperationId::new("git.add").unwrap(),
            RequestedOperationId::new("git.branch.create").unwrap(),
            RequestedOperationId::new("git.commit").unwrap(),
        ];
        operations.sort();
        let resolver = LocalFsResolver::new(
            &repo,
            ResourceClassId::new("project.git").unwrap(),
            operations,
            [runtime.root.clone()],
        )
        .unwrap();
        let mut kernel = KernelApi::open(&runtime, AllowGit).unwrap();
        kernel
            .create_session(
                Principal::test("phase-f-git-stale"),
                KernelCreateSession {
                    session_id: SessionId(64),
                    event_id: EventId(1),
                    recorded_at: "2026-09-05T09:10:00Z",
                    payload: b"phase-f-git-stale-state-qualification",
                },
                "phase-f-git-stale",
            )
            .unwrap();
        Self {
            base,
            repo,
            resolver,
            kernel,
        }
    }

    fn status(&self) -> crate::git_status::GitStatusObservation {
        observe_status(
            &self.resolver,
            &RequestedOperationId::new("git.branch.create").unwrap(),
            GitStatusBounds::default(),
            100,
        )
        .unwrap()
    }

    fn prepare_branch(
        &mut self,
        effect_id: u128,
        expectation: GitMutationExpectation,
        branch: &str,
    ) -> golam_kernel::PreparedToolEffect {
        self.kernel
            .prepare_tool_effect(
                Principal::test("phase-f-git-stale"),
                PrepareToolEffect {
                    effect_id: EffectId(effect_id),
                    session_id: SessionId(64),
                    action: "git.branch.create",
                    resource: &git_branch_resource(branch),
                    execution_semantics: "at_most_once",
                    handler_id: "golam-git-linux",
                    handler_version: "1",
                    idempotency_key: Some("phase-f-git-stale-state"),
                    preconditions_hash: git_branch_preconditions_hash(expectation, branch).unwrap(),
                    payload_hash: git_branch_payload_hash(branch),
                    started_at: "2026-09-05T09:10:01Z",
                },
                "phase-f-git-stale",
            )
            .unwrap()
    }

    fn complete_failed(&mut self, prepared: &golam_kernel::PreparedToolEffect) {
        self.kernel
            .complete_tool_effect(
                Principal::test("phase-f-git-stale"),
                CompleteToolEffect {
                    prepared,
                    finished_at: "2026-09-05T09:10:02Z",
                    completion: ToolExecutionCompletion::Failed,
                    reason_code: Some("stale_or_mismatched_precondition_denied"),
                    evidence_ref: None,
                    receipt: None,
                },
                "phase-f-git-stale",
            )
            .unwrap();
    }

    fn cleanup(self) {
        let base = self.base.clone();
        drop(self);
        fs::remove_dir_all(base).unwrap();
    }
}

#[test]
fn stale_head_after_prepare_is_rejected_before_branch_creation() {
    let mut fixture = Fixture::new();
    let status = fixture.status();
    let expectation = GitMutationExpectation::from_status(&status).unwrap();
    let prepared = fixture.prepare_branch(6401, expectation, "candidate");

    move_head_to_new_valid_commit(&fixture.repo);
    assert!(matches!(
        execute_git_branch_create(&fixture.resolver, &prepared, expectation, "candidate", 101,),
        Err(GitMutationError::StaleRepository)
    ));
    assert!(!fixture.repo.join(".git/refs/heads/candidate").exists());
    fixture.complete_failed(&prepared);
    fixture.cleanup();
}

#[test]
fn stale_index_after_prepare_is_rejected_before_branch_creation() {
    let mut fixture = Fixture::new();
    let status = fixture.status();
    let expectation = GitMutationExpectation::from_status(&status).unwrap();
    let prepared = fixture.prepare_branch(6402, expectation, "candidate");

    write_empty_index(&fixture.repo, true);
    assert!(matches!(
        execute_git_branch_create(&fixture.resolver, &prepared, expectation, "candidate", 102,),
        Err(GitMutationError::StaleRepository)
    ));
    assert!(!fixture.repo.join(".git/refs/heads/candidate").exists());
    fixture.complete_failed(&prepared);
    fixture.cleanup();
}

#[test]
fn prepared_branch_effect_cannot_be_rebound_to_another_branch() {
    let mut fixture = Fixture::new();
    let status = fixture.status();
    let expectation = GitMutationExpectation::from_status(&status).unwrap();
    let prepared = fixture.prepare_branch(6403, expectation, "candidate");

    assert!(matches!(
        execute_git_branch_create(&fixture.resolver, &prepared, expectation, "other", 103),
        Err(GitMutationError::InvalidEffectBinding)
    ));
    assert!(!fixture.repo.join(".git/refs/heads/candidate").exists());
    assert!(!fixture.repo.join(".git/refs/heads/other").exists());
    fixture.complete_failed(&prepared);
    fixture.cleanup();
}

#[test]
fn generic_git_authorized_root_cannot_overlap_protected_golam_state() {
    let base = unique_root();
    fs::create_dir_all(&base).unwrap();
    let runtime = RuntimeLayout::initialize(base.join("runtime")).unwrap();
    let mut operations = vec![
        RequestedOperationId::new("git.add").unwrap(),
        RequestedOperationId::new("git.branch.create").unwrap(),
        RequestedOperationId::new("git.commit").unwrap(),
    ];
    operations.sort();

    let result = LocalFsResolver::new(
        &runtime.root,
        ResourceClassId::new("project.git").unwrap(),
        operations,
        [runtime.root.clone()],
    );
    assert!(matches!(
        result,
        Err(LocalFsResolutionError::ProtectedRootOverlap(_))
    ));
    let _ = fs::remove_dir_all(base);
}

fn initialize_repo(root: &Path) {
    fs::create_dir_all(root.join(".git/objects/info")).unwrap();
    fs::create_dir_all(root.join(".git/objects/pack")).unwrap();
    fs::create_dir_all(root.join(".git/refs/heads")).unwrap();
    fs::write(
        root.join(".git/config"),
        b"[core]\nrepositoryformatversion = 0\n",
    )
    .unwrap();
    fs::write(root.join(".git/HEAD"), b"ref: refs/heads/main\n").unwrap();

    let tree = write_object(root, "tree", b"");
    let identity = "Fixture <fixture@example.invalid> 1 +0000";
    let commit = format!(
        "tree {}\nauthor {identity}\ncommitter {identity}\n\ninitial\n",
        tree.to_hex()
    );
    let commit_id = write_object(root, "commit", commit.as_bytes());
    fs::write(
        root.join(".git/refs/heads/main"),
        format!("{}\n", commit_id.to_hex()),
    )
    .unwrap();
    write_empty_index(root, false);
}

fn move_head_to_new_valid_commit(root: &Path) {
    let parent = fs::read_to_string(root.join(".git/refs/heads/main"))
        .unwrap()
        .trim()
        .to_owned();
    let tree = write_object(root, "tree", b"");
    let identity = "Fixture <fixture@example.invalid> 2 +0000";
    let commit = format!(
        "tree {}\nparent {parent}\nauthor {identity}\ncommitter {identity}\n\nconcurrent\n",
        tree.to_hex()
    );
    let commit_id = write_object(root, "commit", commit.as_bytes());
    fs::write(
        root.join(".git/refs/heads/main"),
        format!("{}\n", commit_id.to_hex()),
    )
    .unwrap();
}

fn write_empty_index(root: &Path, optional_tree_extension: bool) {
    let mut content = Vec::new();
    content.extend_from_slice(b"DIRC");
    content.extend_from_slice(&2_u32.to_be_bytes());
    content.extend_from_slice(&0_u32.to_be_bytes());
    if optional_tree_extension {
        content.extend_from_slice(b"TREE");
        content.extend_from_slice(&0_u32.to_be_bytes());
    }
    let checksum = GitObjectSha1::digest(&content).unwrap();
    content.extend_from_slice(&checksum);
    fs::write(root.join(".git/index"), content).unwrap();
}

fn write_object(root: &Path, kind: &str, body: &[u8]) -> GitObjectId {
    let mut canonical = format!("{kind} {}\0", body.len()).into_bytes();
    canonical.extend_from_slice(body);
    let digest = GitObjectSha1::digest(&canonical).unwrap();
    let id = GitObjectId::parse(&hex20(digest)).unwrap();
    let hex = id.to_hex();
    let directory = root.join(".git/objects").join(&hex[..2]);
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join(&hex[2..]),
        compress_to_vec_zlib(&canonical, 6),
    )
    .unwrap();
    id
}

fn hex20(bytes: [u8; 20]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(40);
    for byte in bytes {
        out.push(char::from(DIGITS[(byte >> 4) as usize]));
        out.push(char::from(DIGITS[(byte & 0x0f) as usize]));
    }
    out
}

fn unique_root() -> PathBuf {
    let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "golam-phase-f-git-stale-{}-{nanos}-{counter}",
        std::process::id()
    ))
}
