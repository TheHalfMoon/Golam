#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;

use golam_core::digest::sha256;
use golam_core::target_identity::{FileMutationExpectation, ObservedFileKind};
use golam_core::tool_request::{BindingDigest, RequestedOperationId, RequestedTarget};
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use golam_kernel::PreparedToolEffect;
use miniz_oxide::deflate::compress_to_vec_zlib;

use crate::git_index::{
    GitIndex, GitIndexBounds, GitIndexEntry, GitIndexError, GitIndexMode, GitIndexVersion,
    parse_git_index,
};
use crate::git_observe::GitTreeMode;
use crate::git_read::{GitHeadRepresentation, GitObjectId};
use crate::git_sha1::{GitObjectSha1, GitObjectSha1Error};
use crate::git_status::{GitChangeKind, GitStatusBounds, GitStatusError, GitStatusObservation, observe_status};
use crate::local_fs::{LocalFsResolutionError, LocalFsResolver};
use crate::local_read::{LocalFileReadBounds, LocalFileReadError, read_regular_file};

const EXPECTATION_DOMAIN: &[u8] = b"golam:git-mutation-expectation:v1";
const STATUS_DOMAIN: &[u8] = b"golam:git-status-state:v1";
const ADD_PRECONDITION_DOMAIN: &[u8] = b"golam:git-add-preconditions:v1";
const ADD_PAYLOAD_DOMAIN: &[u8] = b"golam:git-add-payload:v1";
const COMMIT_PRECONDITION_DOMAIN: &[u8] = b"golam:git-commit-preconditions:v1";
const COMMIT_PAYLOAD_DOMAIN: &[u8] = b"golam:git-commit-payload:v1";
const BRANCH_PRECONDITION_DOMAIN: &[u8] = b"golam:git-branch-preconditions:v1";
const BRANCH_PAYLOAD_DOMAIN: &[u8] = b"golam:git-branch-payload:v1";
const MAX_ADD_BYTES: u64 = 32 * 1024 * 1024;
const MAX_COMMIT_MESSAGE_BYTES: usize = 64 * 1024;
const MAX_IDENTITY_BYTES: usize = 256;
const MAX_BRANCH_BYTES: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GitMutationExpectation {
    pub repository_binding: BindingDigest,
    pub head: GitObjectId,
    pub index_checksum: [u8; 20],
    pub status_digest: BindingDigest,
}

impl GitMutationExpectation {
    pub fn from_status(status: &GitStatusObservation) -> Result<Self, GitMutationError> {
        status.repository_evidence.verify_binding()?;
        Ok(Self {
            repository_binding: status.repository_evidence.binding_digest(),
            head: status.head,
            index_checksum: status.index_checksum,
            status_digest: git_status_digest(status)?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitCommitMetadata {
    pub author_name: String,
    pub author_email: String,
    pub timestamp_seconds: i64,
    pub message: String,
}

impl GitCommitMetadata {
    pub fn validate(&self) -> Result<(), GitMutationError> {
        validate_identity_component(&self.author_name)?;
        validate_identity_component(&self.author_email)?;
        if self.message.is_empty()
            || self.message.len() > MAX_COMMIT_MESSAGE_BYTES
            || self.message.contains('\0')
            || self.message.contains('\r')
        {
            return Err(GitMutationError::InvalidCommitMetadata);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitMutationReceipt {
    pub effect_id: EffectId,
    pub action: &'static str,
    pub previous_head: GitObjectId,
    pub current_head: GitObjectId,
    pub previous_index_checksum: [u8; 20],
    pub current_index_checksum: [u8; 20],
    pub object_id: Option<GitObjectId>,
    pub verified_at_unix_ms: u64,
}

#[derive(Debug)]
pub enum GitMutationError {
    Io(io::Error),
    Core(CoreError),
    Sha1(GitObjectSha1Error),
    Index(GitIndexError),
    Status(GitStatusError),
    Resolution(LocalFsResolutionError),
    LocalRead(LocalFileReadError),
    UnsupportedPlatform,
    InvalidEffectBinding,
    StaleRepository,
    StaleWorktree,
    InvalidTarget,
    UnsupportedIndexProfile,
    ConflictedIndex,
    NothingToCommit,
    DetachedHead,
    InvalidBranchName,
    BranchExists,
    InvalidCommitMetadata,
    MutationTooLarge,
    StagingCollision(PathBuf),
    UnknownOutcome(PathBuf),
    #[cfg(unix)]
    Unix(nix::errno::Errno),
}

impl fmt::Display for GitMutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "Git mutation I/O failed: {error}"),
            Self::Core(error) => write!(f, "Git mutation canonical encoding failed: {error}"),
            Self::Sha1(error) => write!(f, "Git mutation SHA-1 failed: {error}"),
            Self::Index(error) => write!(f, "Git mutation index validation failed: {error}"),
            Self::Status(error) => write!(f, "Git mutation status observation failed: {error}"),
            Self::Resolution(error) => write!(f, "Git mutation target resolution failed: {error}"),
            Self::LocalRead(error) => write!(f, "Git mutation bounded read failed: {error}"),
            Self::UnsupportedPlatform => f.write_str("governed Git mutation is not qualified on this platform"),
            Self::InvalidEffectBinding => f.write_str("prepared tool effect does not bind this exact Git mutation"),
            Self::StaleRepository => f.write_str("Git repository HEAD/index/status preconditions are stale"),
            Self::StaleWorktree => f.write_str("Git worktree target precondition is stale"),
            Self::InvalidTarget => f.write_str("Git mutation target is invalid for the first profile"),
            Self::UnsupportedIndexProfile => f.write_str("Git mutation requires a canonical extension-free v2 stage-zero index"),
            Self::ConflictedIndex => f.write_str("Git mutation refuses conflicted or special index entries"),
            Self::NothingToCommit => f.write_str("Git commit requires at least one staged change and a clean worktree"),
            Self::DetachedHead => f.write_str("Git commit requires a symbolic local branch HEAD"),
            Self::InvalidBranchName => f.write_str("Git branch name is outside the bounded ordinary-authority profile"),
            Self::BranchExists => f.write_str("Git branch already exists and ordinary authority cannot move it"),
            Self::InvalidCommitMetadata => f.write_str("Git commit metadata is empty, unbounded, or non-canonical"),
            Self::MutationTooLarge => f.write_str("Git mutation input exceeds the bounded first-profile limit"),
            Self::StagingCollision(path) => write!(f, "Git mutation staging/guard entry already exists at {}", path.display()),
            Self::UnknownOutcome(path) => write!(f, "Git mutation completion is ambiguous and requires reconciliation at {}", path.display()),
            #[cfg(unix)]
            Self::Unix(error) => write!(f, "Git mutation Unix primitive failed: {error}"),
        }
    }
}

impl Error for GitMutationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::Sha1(error) => Some(error),
            Self::Index(error) => Some(error),
            Self::Status(error) => Some(error),
            Self::Resolution(error) => Some(error),
            Self::LocalRead(error) => Some(error),
            #[cfg(unix)]
            Self::Unix(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for GitMutationError {
    fn from(value: io::Error) -> Self { Self::Io(value) }
}
impl From<CoreError> for GitMutationError {
    fn from(value: CoreError) -> Self { Self::Core(value) }
}
impl From<GitObjectSha1Error> for GitMutationError {
    fn from(value: GitObjectSha1Error) -> Self { Self::Sha1(value) }
}
impl From<GitIndexError> for GitMutationError {
    fn from(value: GitIndexError) -> Self { Self::Index(value) }
}
impl From<GitStatusError> for GitMutationError {
    fn from(value: GitStatusError) -> Self { Self::Status(value) }
}
impl From<LocalFsResolutionError> for GitMutationError {
    fn from(value: LocalFsResolutionError) -> Self { Self::Resolution(value) }
}
impl From<LocalFileReadError> for GitMutationError {
    fn from(value: LocalFileReadError) -> Self { Self::LocalRead(value) }
}
#[cfg(unix)]
impl From<nix::errno::Errno> for GitMutationError {
    fn from(value: nix::errno::Errno) -> Self { Self::Unix(value) }
}

pub fn git_add_resource(path: &RequestedTarget) -> String {
    format!("git-add:{}", path.as_str())
}

pub const fn git_commit_resource() -> &'static str { "git-commit:HEAD" }

pub fn git_branch_resource(branch: &str) -> String { format!("git-branch:{branch}") }

pub fn git_add_preconditions_hash(
    expectation: GitMutationExpectation,
    path: &RequestedTarget,
    target: FileMutationExpectation,
) -> Result<[u8; 32], CoreError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(ADD_PRECONDITION_DOMAIN)?;
    push_expectation(&mut encoder, expectation)?;
    encoder.push_bytes(path.as_str().as_bytes())?;
    push_file_expectation(&mut encoder, target)?;
    Ok(sha256(&encoder.finish()))
}

pub fn git_add_payload_hash(path: &RequestedTarget, content: BindingDigest) -> [u8; 32] {
    let mut bytes = Vec::with_capacity(ADD_PAYLOAD_DOMAIN.len() + path.as_str().len() + 32);
    bytes.extend_from_slice(ADD_PAYLOAD_DOMAIN);
    bytes.extend_from_slice(path.as_str().as_bytes());
    bytes.extend_from_slice(&content.bytes());
    sha256(&bytes)
}

pub fn git_commit_preconditions_hash(expectation: GitMutationExpectation) -> Result<[u8; 32], CoreError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(COMMIT_PRECONDITION_DOMAIN)?;
    push_expectation(&mut encoder, expectation)?;
    Ok(sha256(&encoder.finish()))
}

pub fn git_commit_payload_hash(metadata: &GitCommitMetadata) -> Result<[u8; 32], GitMutationError> {
    metadata.validate()?;
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(COMMIT_PAYLOAD_DOMAIN)?;
    encoder.push_bytes(metadata.author_name.as_bytes())?;
    encoder.push_bytes(metadata.author_email.as_bytes())?;
    encoder.push_u64(u64::from_be_bytes(metadata.timestamp_seconds.to_be_bytes()));
    encoder.push_bytes(metadata.message.as_bytes())?;
    Ok(sha256(&encoder.finish()))
}

pub fn git_branch_preconditions_hash(expectation: GitMutationExpectation, branch: &str) -> Result<[u8; 32], CoreError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(BRANCH_PRECONDITION_DOMAIN)?;
    push_expectation(&mut encoder, expectation)?;
    encoder.push_bytes(branch.as_bytes())?;
    Ok(sha256(&encoder.finish()))
}

pub fn git_branch_payload_hash(branch: &str) -> [u8; 32] {
    let mut bytes = Vec::with_capacity(BRANCH_PAYLOAD_DOMAIN.len() + branch.len());
    bytes.extend_from_slice(BRANCH_PAYLOAD_DOMAIN);
    bytes.extend_from_slice(branch.as_bytes());
    sha256(&bytes)
}

pub fn execute_git_add(
    resolver: &LocalFsResolver,
    prepared: &PreparedToolEffect,
    expectation: GitMutationExpectation,
    path: &RequestedTarget,
    target_expectation: FileMutationExpectation,
    observed_at_unix_ms: u64,
) -> Result<GitMutationReceipt, GitMutationError> {
    #[cfg(unix)]
    {
        execute_git_add_unix(resolver, prepared, expectation, path, target_expectation, observed_at_unix_ms)
    }
    #[cfg(not(unix))]
    {
        let _ = (resolver, prepared, expectation, path, target_expectation, observed_at_unix_ms);
        Err(GitMutationError::UnsupportedPlatform)
    }
}

pub fn execute_git_commit(
    resolver: &LocalFsResolver,
    prepared: &PreparedToolEffect,
    expectation: GitMutationExpectation,
    metadata: &GitCommitMetadata,
    observed_at_unix_ms: u64,
) -> Result<GitMutationReceipt, GitMutationError> {
    #[cfg(unix)]
    {
        execute_git_commit_unix(resolver, prepared, expectation, metadata, observed_at_unix_ms)
    }
    #[cfg(not(unix))]
    {
        let _ = (resolver, prepared, expectation, metadata, observed_at_unix_ms);
        Err(GitMutationError::UnsupportedPlatform)
    }
}

pub fn execute_git_branch_create(
    resolver: &LocalFsResolver,
    prepared: &PreparedToolEffect,
    expectation: GitMutationExpectation,
    branch: &str,
    observed_at_unix_ms: u64,
) -> Result<GitMutationReceipt, GitMutationError> {
    #[cfg(unix)]
    {
        execute_git_branch_create_unix(resolver, prepared, expectation, branch, observed_at_unix_ms)
    }
    #[cfg(not(unix))]
    {
        let _ = (resolver, prepared, expectation, branch, observed_at_unix_ms);
        Err(GitMutationError::UnsupportedPlatform)
    }
}

#[cfg(unix)]
fn execute_git_add_unix(
    resolver: &LocalFsResolver,
    prepared: &PreparedToolEffect,
    expectation: GitMutationExpectation,
    path: &RequestedTarget,
    target_expectation: FileMutationExpectation,
    observed_at_unix_ms: u64,
) -> Result<GitMutationReceipt, GitMutationError> {
    use std::os::unix::fs::MetadataExt;

    target_expectation.validate().map_err(|_| GitMutationError::InvalidTarget)?;
    if !target_expectation.expected_exists
        || target_expectation.expected_kind != Some(ObservedFileKind::RegularFile)
        || target_expectation.expected_identity.is_none()
        || target_expectation.expected_content_digest.is_none()
    {
        return Err(GitMutationError::InvalidTarget);
    }
    let expected_content = target_expectation.expected_content_digest.ok_or(GitMutationError::InvalidTarget)?;
    if prepared.action() != "git.add"
        || prepared.resource() != git_add_resource(path)
        || prepared.preconditions_hash() != git_add_preconditions_hash(expectation, path, target_expectation)?
        || prepared.payload_hash() != git_add_payload_hash(path, expected_content)
    {
        return Err(GitMutationError::InvalidEffectBinding);
    }

    let operation = RequestedOperationId::new("git.add").map_err(|_| GitMutationError::InvalidEffectBinding)?;
    let before = observe_status(resolver, &operation, GitStatusBounds::default(), observed_at_unix_ms)?;
    verify_expectation(&before, expectation)?;
    if before.untracked.iter().any(|candidate| candidate != path.as_str()) {
        return Err(GitMutationError::StaleWorktree);
    }

    let identity = resolver.resolve_read_target(path, &operation, observed_at_unix_ms)?;
    if identity.file_kind != ObservedFileKind::RegularFile
        || identity.resolved_target_identity != target_expectation.expected_identity
    {
        return Err(GitMutationError::StaleWorktree);
    }
    let read = read_regular_file(
        resolver,
        path,
        &operation,
        LocalFileReadBounds { max_bytes: MAX_ADD_BYTES, max_duration: GitStatusBounds::default().max_duration },
        observed_at_unix_ms,
        observed_at_unix_ms,
    )?;
    if BindingDigest::new(sha256(&read.bytes)) != expected_content
        || target_expectation.expected_size.is_some_and(|size| size != read.bytes.len() as u64)
    {
        return Err(GitMutationError::StaleWorktree);
    }

    let blob_id = write_loose_object(resolver, &operation, "blob", &read.bytes, observed_at_unix_ms)?;
    let (mut index, index_path) = read_index(resolver, &operation, observed_at_unix_ms)?;
    require_mutable_index_profile(&index)?;
    let metadata = fs::metadata(Path::new(identity.normalized_path.as_str()))?;
    let file_size = u32::try_from(metadata.len()).map_err(|_| GitMutationError::MutationTooLarge)?;
    let path_bytes = path.as_str().as_bytes().to_vec();
    if path_bytes.len() > 4096 || path_bytes.contains(&0) {
        return Err(GitMutationError::InvalidTarget);
    }
    let entry = GitIndexEntry {
        ctime_seconds: clamp_i64_u32(metadata.ctime()),
        ctime_nanoseconds: clamp_i64_u32(metadata.ctime_nsec()),
        mtime_seconds: clamp_i64_u32(metadata.mtime()),
        mtime_nanoseconds: clamp_i64_u32(metadata.mtime_nsec()),
        dev: u32::try_from(metadata.dev()).unwrap_or(0),
        ino: u32::try_from(metadata.ino()).unwrap_or(0),
        mode: GitIndexMode::RegularFile { executable: metadata.mode() & 0o111 != 0 },
        uid: metadata.uid(),
        gid: metadata.gid(),
        file_size,
        object_id: blob_id.bytes(),
        assume_valid: false,
        stage: 0,
        skip_worktree: false,
        intent_to_add: false,
        path: path_bytes,
    };
    match index.entries.binary_search_by(|candidate| candidate.path.cmp(&entry.path)) {
        Ok(position) => index.entries[position] = entry,
        Err(position) => index.entries.insert(position, entry),
    }
    let index_bytes = serialize_index_v2(&index)?;
    replace_expected_file(&index_path, &index_bytes, expectation.index_checksum, prepared.effect_id())?;

    let after = observe_status(resolver, &operation, GitStatusBounds::default(), observed_at_unix_ms)?;
    if after.head != before.head || after.index_checksum == before.index_checksum {
        return Err(GitMutationError::UnknownOutcome(index_path));
    }
    let staged_matches = after.staged.iter().any(|change| {
        change.path == path.as_str() && change.after == Some(blob_id)
    });
    if !staged_matches || after.worktree.iter().any(|change| change.path == path.as_str()) || after.untracked.iter().any(|candidate| candidate == path.as_str()) {
        return Err(GitMutationError::UnknownOutcome(index_path));
    }
    Ok(GitMutationReceipt {
        effect_id: prepared.effect_id(),
        action: "git.add",
        previous_head: before.head,
        current_head: after.head,
        previous_index_checksum: before.index_checksum,
        current_index_checksum: after.index_checksum,
        object_id: Some(blob_id),
        verified_at_unix_ms: observed_at_unix_ms,
    })
}

#[cfg(unix)]
fn execute_git_commit_unix(
    resolver: &LocalFsResolver,
    prepared: &PreparedToolEffect,
    expectation: GitMutationExpectation,
    metadata: &GitCommitMetadata,
    observed_at_unix_ms: u64,
) -> Result<GitMutationReceipt, GitMutationError> {
    metadata.validate()?;
    if prepared.action() != "git.commit"
        || prepared.resource() != git_commit_resource()
        || prepared.preconditions_hash() != git_commit_preconditions_hash(expectation)?
        || prepared.payload_hash() != git_commit_payload_hash(metadata)?
    {
        return Err(GitMutationError::InvalidEffectBinding);
    }
    let operation = RequestedOperationId::new("git.commit").map_err(|_| GitMutationError::InvalidEffectBinding)?;
    let before = observe_status(resolver, &operation, GitStatusBounds::default(), observed_at_unix_ms)?;
    verify_expectation(&before, expectation)?;
    if before.staged.is_empty() || !before.worktree.is_empty() || !before.untracked.is_empty() {
        return Err(GitMutationError::NothingToCommit);
    }
    let head_ref = match &before.repository_evidence.head.representation {
        GitHeadRepresentation::Symbolic(name) if name.starts_with("refs/heads/") => name.clone(),
        _ => return Err(GitMutationError::DetachedHead),
    };
    validate_local_ref(&head_ref)?;

    let (index, _) = read_index(resolver, &operation, observed_at_unix_ms)?;
    require_mutable_index_profile(&index)?;
    let tree_id = write_index_tree(resolver, &operation, &index, observed_at_unix_ms)?;
    let commit_bytes = build_commit_bytes(tree_id, before.head, metadata)?;
    let commit_id = write_loose_object(resolver, &operation, "commit", &commit_bytes, observed_at_unix_ms)?;

    let ref_request = RequestedTarget::new(format!(".git/{head_ref}"))
        .map_err(|_| GitMutationError::InvalidTarget)?;
    let ref_identity = resolver.resolve_read_target(&ref_request, &operation, observed_at_unix_ms)?;
    if ref_identity.file_kind != ObservedFileKind::RegularFile {
        return Err(GitMutationError::StaleRepository);
    }
    let ref_path = PathBuf::from(ref_identity.normalized_path.as_str());
    let expected_ref = format!("{}\n", before.head.to_hex());
    let next_ref = format!("{}\n", commit_id.to_hex());
    replace_expected_bytes(&ref_path, expected_ref.as_bytes(), next_ref.as_bytes(), prepared.effect_id())?;

    let after = observe_status(resolver, &operation, GitStatusBounds::default(), observed_at_unix_ms)?;
    if after.head != commit_id || !after.staged.is_empty() || !after.worktree.is_empty() || !after.untracked.is_empty() || after.index_checksum != before.index_checksum {
        return Err(GitMutationError::UnknownOutcome(ref_path));
    }
    Ok(GitMutationReceipt {
        effect_id: prepared.effect_id(),
        action: "git.commit",
        previous_head: before.head,
        current_head: after.head,
        previous_index_checksum: before.index_checksum,
        current_index_checksum: after.index_checksum,
        object_id: Some(commit_id),
        verified_at_unix_ms: observed_at_unix_ms,
    })
}

#[cfg(unix)]
fn execute_git_branch_create_unix(
    resolver: &LocalFsResolver,
    prepared: &PreparedToolEffect,
    expectation: GitMutationExpectation,
    branch: &str,
    observed_at_unix_ms: u64,
) -> Result<GitMutationReceipt, GitMutationError> {
    validate_branch_name(branch)?;
    if prepared.action() != "git.branch.create"
        || prepared.resource() != git_branch_resource(branch)
        || prepared.preconditions_hash() != git_branch_preconditions_hash(expectation, branch)?
        || prepared.payload_hash() != git_branch_payload_hash(branch)
    {
        return Err(GitMutationError::InvalidEffectBinding);
    }
    let operation = RequestedOperationId::new("git.branch.create").map_err(|_| GitMutationError::InvalidEffectBinding)?;
    let before = observe_status(resolver, &operation, GitStatusBounds::default(), observed_at_unix_ms)?;
    verify_expectation(&before, expectation)?;
    if !before.staged.is_empty() || !before.worktree.is_empty() || !before.untracked.is_empty() {
        return Err(GitMutationError::StaleWorktree);
    }
    let heads_request = RequestedTarget::new(".git/refs/heads").map_err(|_| GitMutationError::InvalidTarget)?;
    let heads = resolver.resolve_read_target(&heads_request, &operation, observed_at_unix_ms)?;
    if heads.file_kind != ObservedFileKind::Directory {
        return Err(GitMutationError::StaleRepository);
    }
    let branch_path = Path::new(heads.normalized_path.as_str()).join(branch);
    match fs::symlink_metadata(&branch_path) {
        Ok(_) => return Err(GitMutationError::BranchExists),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    write_new_file(&branch_path, format!("{}\n", before.head.to_hex()).as_bytes())?;
    let readback = fs::read(&branch_path)?;
    if readback != format!("{}\n", before.head.to_hex()).as_bytes() {
        return Err(GitMutationError::UnknownOutcome(branch_path));
    }
    let after = observe_status(resolver, &operation, GitStatusBounds::default(), observed_at_unix_ms)?;
    if after.head != before.head || after.index_checksum != before.index_checksum || git_status_digest(&after)? != expectation.status_digest {
        return Err(GitMutationError::UnknownOutcome(branch_path));
    }
    Ok(GitMutationReceipt {
        effect_id: prepared.effect_id(),
        action: "git.branch.create",
        previous_head: before.head,
        current_head: after.head,
        previous_index_checksum: before.index_checksum,
        current_index_checksum: after.index_checksum,
        object_id: Some(before.head),
        verified_at_unix_ms: observed_at_unix_ms,
    })
}

fn verify_expectation(status: &GitStatusObservation, expected: GitMutationExpectation) -> Result<(), GitMutationError> {
    status.repository_evidence.verify_binding()?;
    if status.repository_evidence.binding_digest() != expected.repository_binding
        || status.head != expected.head
        || status.index_checksum != expected.index_checksum
        || git_status_digest(status)? != expected.status_digest
    {
        return Err(GitMutationError::StaleRepository);
    }
    Ok(())
}

pub fn git_status_digest(status: &GitStatusObservation) -> Result<BindingDigest, GitMutationError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(STATUS_DOMAIN)?;
    encoder.push_bytes(&status.head.bytes())?;
    encoder.push_bytes(&status.index_checksum)?;
    push_changes(&mut encoder, &status.staged)?;
    push_changes(&mut encoder, &status.worktree)?;
    encoder.push_u64(u64::try_from(status.untracked.len()).map_err(|_| CoreError::CanonicalLengthOverflow)?);
    for path in &status.untracked { encoder.push_bytes(path.as_bytes())?; }
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn push_changes(encoder: &mut CanonicalEncoder, changes: &[crate::git_status::GitDiffEvidence]) -> Result<(), CoreError> {
    encoder.push_u64(u64::try_from(changes.len()).map_err(|_| CoreError::CanonicalLengthOverflow)?);
    for change in changes {
        encoder.push_bytes(change.path.as_bytes())?;
        encoder.push_u8(match change.kind {
            GitChangeKind::Added => 1,
            GitChangeKind::Modified => 2,
            GitChangeKind::Deleted => 3,
            GitChangeKind::TypeChanged => 4,
            GitChangeKind::Conflicted => 5,
            GitChangeKind::IntentToAdd => 6,
        });
        push_object_id(encoder, change.before)?;
        push_object_id(encoder, change.after)?;
    }
    Ok(())
}

fn push_object_id(encoder: &mut CanonicalEncoder, value: Option<GitObjectId>) -> Result<(), CoreError> {
    match value {
        Some(id) => { encoder.push_u8(1); encoder.push_bytes(&id.bytes())?; }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn push_expectation(encoder: &mut CanonicalEncoder, value: GitMutationExpectation) -> Result<(), CoreError> {
    encoder.push_bytes(EXPECTATION_DOMAIN)?;
    encoder.push_bytes(&value.repository_binding.bytes())?;
    encoder.push_bytes(&value.head.bytes())?;
    encoder.push_bytes(&value.index_checksum)?;
    encoder.push_bytes(&value.status_digest.bytes())?;
    Ok(())
}

fn push_file_expectation(encoder: &mut CanonicalEncoder, value: FileMutationExpectation) -> Result<(), CoreError> {
    encoder.push_u8(u8::from(value.expected_exists));
    encoder.push_u8(match value.expected_kind {
        None => 0,
        Some(ObservedFileKind::Missing) => 1,
        Some(ObservedFileKind::RegularFile) => 2,
        Some(ObservedFileKind::Directory) => 3,
        Some(ObservedFileKind::SymlinkOrReparsePoint) => 4,
        Some(ObservedFileKind::Special) => 5,
    });
    push_digest(encoder, value.expected_identity)?;
    push_digest(encoder, value.expected_content_digest)?;
    match value.expected_size { Some(size) => { encoder.push_u8(1); encoder.push_u64(size); }, None => encoder.push_u8(0) }
    push_digest(encoder, value.expected_parent_identity)?;
    Ok(())
}

fn push_digest(encoder: &mut CanonicalEncoder, value: Option<BindingDigest>) -> Result<(), CoreError> {
    match value { Some(digest) => { encoder.push_u8(1); encoder.push_bytes(&digest.bytes())?; }, None => encoder.push_u8(0) }
    Ok(())
}

#[cfg(unix)]
fn read_index(resolver: &LocalFsResolver, operation: &RequestedOperationId, observed_at_unix_ms: u64) -> Result<(GitIndex, PathBuf), GitMutationError> {
    let request = RequestedTarget::new(".git/index").map_err(|_| GitMutationError::InvalidTarget)?;
    let identity = resolver.resolve_read_target(&request, operation, observed_at_unix_ms)?;
    if identity.file_kind != ObservedFileKind::RegularFile { return Err(GitMutationError::StaleRepository); }
    let path = PathBuf::from(identity.normalized_path.as_str());
    let bytes = fs::read(&path)?;
    Ok((parse_git_index(&bytes, GitIndexBounds::default())?, path))
}

fn require_mutable_index_profile(index: &GitIndex) -> Result<(), GitMutationError> {
    if index.version != GitIndexVersion::V2 || !index.extensions.is_empty() { return Err(GitMutationError::UnsupportedIndexProfile); }
    if index.entries.iter().any(|entry| entry.stage != 0 || entry.skip_worktree || entry.intent_to_add || !matches!(entry.mode, GitIndexMode::RegularFile { .. })) {
        return Err(GitMutationError::ConflictedIndex);
    }
    Ok(())
}

fn serialize_index_v2(index: &GitIndex) -> Result<Vec<u8>, GitMutationError> {
    require_mutable_index_profile(index)?;
    let mut output = Vec::new();
    output.extend_from_slice(b"DIRC");
    output.extend_from_slice(&2_u32.to_be_bytes());
    output.extend_from_slice(&u32::try_from(index.entries.len()).map_err(|_| GitMutationError::MutationTooLarge)?.to_be_bytes());
    let mut previous: Option<&[u8]> = None;
    for entry in &index.entries {
        if previous.is_some_and(|path| path >= entry.path.as_slice()) || entry.path.is_empty() || entry.path.len() > 4096 { return Err(GitMutationError::UnsupportedIndexProfile); }
        previous = Some(&entry.path);
        let start = output.len();
        for value in [entry.ctime_seconds, entry.ctime_nanoseconds, entry.mtime_seconds, entry.mtime_nanoseconds, entry.dev, entry.ino] { output.extend_from_slice(&value.to_be_bytes()); }
        output.extend_from_slice(&index_mode_bits(entry.mode).to_be_bytes());
        for value in [entry.uid, entry.gid, entry.file_size] { output.extend_from_slice(&value.to_be_bytes()); }
        output.extend_from_slice(&entry.object_id);
        let mut flags = u16::try_from(entry.path.len().min(0x0fff)).map_err(|_| GitMutationError::MutationTooLarge)?;
        if entry.assume_valid { flags |= 0x8000; }
        output.extend_from_slice(&flags.to_be_bytes());
        output.extend_from_slice(&entry.path);
        output.push(0);
        let consumed = output.len() - start;
        let padding = (8 - (consumed % 8)) % 8;
        output.resize(output.len() + padding, 0);
    }
    let checksum = GitObjectSha1::digest(&output)?;
    output.extend_from_slice(&checksum);
    let parsed = parse_git_index(&output, GitIndexBounds::default())?;
    if parsed.entries.len() != index.entries.len() { return Err(GitMutationError::UnsupportedIndexProfile); }
    Ok(output)
}

fn index_mode_bits(mode: GitIndexMode) -> u32 {
    match mode { GitIndexMode::RegularFile { executable: false } => 0o100644, GitIndexMode::RegularFile { executable: true } => 0o100755, GitIndexMode::SymbolicLink => 0o120000, GitIndexMode::Gitlink => 0o160000 }
}

#[cfg(unix)]
fn write_index_tree(resolver: &LocalFsResolver, operation: &RequestedOperationId, index: &GitIndex, observed_at_unix_ms: u64) -> Result<GitObjectId, GitMutationError> {
    let mut root = TreeNode::default();
    for entry in &index.entries {
        let path = std::str::from_utf8(&entry.path).map_err(|_| GitMutationError::InvalidTarget)?;
        root.insert(path, entry.mode, GitObjectId::parse(&hex20(entry.object_id)).map_err(|_| GitMutationError::InvalidTarget)?)?;
    }
    write_tree_node(resolver, operation, &root, observed_at_unix_ms)
}

#[derive(Default)]
struct TreeNode { files: BTreeMap<Vec<u8>, (GitIndexMode, GitObjectId)>, dirs: BTreeMap<Vec<u8>, TreeNode> }

impl TreeNode {
    fn insert(&mut self, path: &str, mode: GitIndexMode, id: GitObjectId) -> Result<(), GitMutationError> {
        let parts = path.as_bytes().split(|byte| *byte == b'/').collect::<Vec<_>>();
        if parts.is_empty() || parts.iter().any(|part| part.is_empty()) { return Err(GitMutationError::InvalidTarget); }
        self.insert_parts(&parts, mode, id)
    }
    fn insert_parts(&mut self, parts: &[&[u8]], mode: GitIndexMode, id: GitObjectId) -> Result<(), GitMutationError> {
        if parts.len() == 1 {
            if self.dirs.contains_key(parts[0]) || self.files.insert(parts[0].to_vec(), (mode, id)).is_some() { return Err(GitMutationError::InvalidTarget); }
            return Ok(());
        }
        if self.files.contains_key(parts[0]) { return Err(GitMutationError::InvalidTarget); }
        self.dirs.entry(parts[0].to_vec()).or_default().insert_parts(&parts[1..], mode, id)
    }
}

#[cfg(unix)]
fn write_tree_node(resolver: &LocalFsResolver, operation: &RequestedOperationId, node: &TreeNode, observed_at_unix_ms: u64) -> Result<GitObjectId, GitMutationError> {
    let mut entries = Vec::<(Vec<u8>, GitTreeMode, GitObjectId)>::new();
    for (name, child) in &node.dirs {
        let id = write_tree_node(resolver, operation, child, observed_at_unix_ms)?;
        entries.push((name.clone(), GitTreeMode::Directory, id));
    }
    for (name, (mode, id)) in &node.files {
        let tree_mode = match mode { GitIndexMode::RegularFile { executable } => GitTreeMode::RegularFile { executable: *executable }, _ => return Err(GitMutationError::ConflictedIndex) };
        entries.push((name.clone(), tree_mode, *id));
    }
    entries.sort_by(|left, right| tree_sort_key(&left.0, left.1).cmp(&tree_sort_key(&right.0, right.1)));
    let mut bytes = Vec::new();
    for (name, mode, id) in entries {
        bytes.extend_from_slice(tree_mode_text(mode));
        bytes.push(b' ');
        bytes.extend_from_slice(&name);
        bytes.push(0);
        bytes.extend_from_slice(&id.bytes());
    }
    write_loose_object(resolver, operation, "tree", &bytes, observed_at_unix_ms)
}

fn tree_sort_key(name: &[u8], mode: GitTreeMode) -> Vec<u8> {
    let mut key = name.to_vec();
    if mode == GitTreeMode::Directory { key.push(b'/'); }
    key
}

fn tree_mode_text(mode: GitTreeMode) -> &'static [u8] {
    match mode { GitTreeMode::RegularFile { executable: false } => b"100644", GitTreeMode::RegularFile { executable: true } => b"100755", GitTreeMode::SymbolicLink => b"120000", GitTreeMode::Directory => b"40000", GitTreeMode::Gitlink => b"160000" }
}

fn build_commit_bytes(tree: GitObjectId, parent: GitObjectId, metadata: &GitCommitMetadata) -> Result<Vec<u8>, GitMutationError> {
    metadata.validate()?;
    let identity = format!("{} <{}> {} +0000", metadata.author_name, metadata.author_email, metadata.timestamp_seconds);
    let message = metadata.message.trim_end_matches('\n');
    Ok(format!("tree {}\nparent {}\nauthor {identity}\ncommitter {identity}\n\n{message}\n", tree.to_hex(), parent.to_hex()).into_bytes())
}

#[cfg(unix)]
fn write_loose_object(resolver: &LocalFsResolver, operation: &RequestedOperationId, kind: &str, body: &[u8], observed_at_unix_ms: u64) -> Result<GitObjectId, GitMutationError> {
    if body.len() > 32 * 1024 * 1024 { return Err(GitMutationError::MutationTooLarge); }
    let mut canonical = format!("{kind} {}\0", body.len()).into_bytes();
    canonical.extend_from_slice(body);
    let digest = GitObjectSha1::digest(&canonical)?;
    let id = GitObjectId::parse(&hex20(digest)).map_err(|_| GitMutationError::InvalidTarget)?;
    let objects_request = RequestedTarget::new(".git/objects").map_err(|_| GitMutationError::InvalidTarget)?;
    let objects = resolver.resolve_read_target(&objects_request, operation, observed_at_unix_ms)?;
    if objects.file_kind != ObservedFileKind::Directory { return Err(GitMutationError::StaleRepository); }
    let hex = id.to_hex();
    let directory = Path::new(objects.normalized_path.as_str()).join(&hex[..2]);
    match fs::create_dir(&directory) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            let metadata = fs::symlink_metadata(&directory)?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() { return Err(GitMutationError::StaleRepository); }
        }
        Err(error) => return Err(error.into()),
    }
    let object_path = directory.join(&hex[2..]);
    let compressed = compress_to_vec_zlib(&canonical, 6);
    match fs::OpenOptions::new().write(true).create_new(true).open(&object_path) {
        Ok(mut file) => {
            use std::io::Write;
            file.write_all(&compressed)?;
            file.sync_all()?;
            fs::File::open(&directory)?.sync_all()?;
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error.into()),
    }
    Ok(id)
}

#[cfg(unix)]
fn replace_expected_file(path: &Path, next: &[u8], expected_checksum: [u8; 20], effect_id: EffectId) -> Result<(), GitMutationError> {
    let current = fs::read(path)?;
    if current.len() < 20 || current[current.len() - 20..] != expected_checksum { return Err(GitMutationError::StaleRepository); }
    replace_expected_bytes(path, &current, next, effect_id)
}

#[cfg(unix)]
fn replace_expected_bytes(path: &Path, expected: &[u8], next: &[u8], effect_id: EffectId) -> Result<(), GitMutationError> {
    use std::fs::File;
    use std::io::{Read, Write};
    use nix::fcntl::{AtFlags, OFlag, open, openat};
    use nix::sys::stat::Mode;
    use nix::unistd::{UnlinkatFlags, linkat, unlinkat};

    let parent_path = path.parent().ok_or(GitMutationError::InvalidTarget)?;
    let name = path.file_name().and_then(|value| value.to_str()).ok_or(GitMutationError::InvalidTarget)?;
    let parent_fd = open(parent_path, OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW, Mode::empty())?;
    let parent = File::from(parent_fd);
    let token = format!("{:032x}", effect_id.0);
    let temp = format!(".golam-{token}.next");
    let guard = format!(".golam-{token}.previous");
    require_missing_at(&parent, &temp)?;
    require_missing_at(&parent, &guard)?;
    let temp_fd = openat(&parent, temp.as_str(), OFlag::O_RDWR | OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW, Mode::from_bits_truncate(0o600))?;
    let mut temp_file = File::from(temp_fd);
    temp_file.write_all(next)?;
    temp_file.sync_all()?;

    let current_fd = openat(&parent, name, OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW, Mode::empty())?;
    let mut current_file = File::from(current_fd);
    let mut current = Vec::new();
    current_file.read_to_end(&mut current)?;
    if current != expected { cleanup_at(&parent, &temp); return Err(GitMutationError::StaleRepository); }
    linkat(&parent, name, &parent, guard.as_str(), AtFlags::empty())?;
    let guard_fd = openat(&parent, guard.as_str(), OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW, Mode::empty())?;
    let guard_file = File::from(guard_fd);
    if !same_unix_object(&current_file.metadata()?, &guard_file.metadata()?) { cleanup_at(&parent, &temp); cleanup_at(&parent, &guard); return Err(GitMutationError::StaleRepository); }
    unlinkat(&parent, name, UnlinkatFlags::NoRemoveDir)?;
    if let Err(_) = linkat(&parent, temp.as_str(), &parent, name, AtFlags::empty()) {
        let restored = linkat(&parent, guard.as_str(), &parent, name, AtFlags::empty()).is_ok();
        cleanup_at(&parent, &temp);
        if restored { cleanup_at(&parent, &guard); return Err(GitMutationError::StaleRepository); }
        return Err(GitMutationError::UnknownOutcome(path.to_path_buf()));
    }
    let installed_fd = openat(&parent, name, OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW, Mode::empty())?;
    let mut installed = File::from(installed_fd);
    let mut readback = Vec::new();
    installed.read_to_end(&mut readback)?;
    if readback != next { return Err(GitMutationError::UnknownOutcome(path.to_path_buf())); }
    cleanup_at(&parent, &temp);
    cleanup_at(&parent, &guard);
    parent.sync_all()?;
    Ok(())
}

#[cfg(unix)]
fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), GitMutationError> {
    use std::io::Write;
    let mut file = match fs::OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => return Err(GitMutationError::BranchExists),
        Err(error) => return Err(error.into()),
    };
    file.write_all(bytes)?;
    file.sync_all()?;
    if let Some(parent) = path.parent() { fs::File::open(parent)?.sync_all()?; }
    Ok(())
}

#[cfg(unix)]
fn require_missing_at(parent: &fs::File, name: &str) -> Result<(), GitMutationError> {
    use nix::fcntl::{OFlag, openat};
    use nix::sys::stat::Mode;
    match openat(parent, name, OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW, Mode::empty()) {
        Ok(_) => Err(GitMutationError::StagingCollision(PathBuf::from(name))),
        Err(nix::errno::Errno::ENOENT) => Ok(()),
        Err(error) => Err(GitMutationError::Unix(error)),
    }
}

#[cfg(unix)]
fn cleanup_at(parent: &fs::File, name: &str) {
    let _ = nix::unistd::unlinkat(parent, name, nix::unistd::UnlinkatFlags::NoRemoveDir);
}

#[cfg(unix)]
fn same_unix_object(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    left.dev() == right.dev() && left.ino() == right.ino()
}

fn validate_identity_component(value: &str) -> Result<(), GitMutationError> {
    if value.is_empty() || value.len() > MAX_IDENTITY_BYTES || value.contains(['\n', '\r', '\0', '<', '>']) { return Err(GitMutationError::InvalidCommitMetadata); }
    Ok(())
}

fn validate_branch_name(branch: &str) -> Result<(), GitMutationError> {
    if branch.is_empty() || branch.len() > MAX_BRANCH_BYTES || branch.starts_with('.') || branch.ends_with('.') || branch.ends_with(".lock") || branch.contains("..") || branch.bytes().any(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))) {
        return Err(GitMutationError::InvalidBranchName);
    }
    Ok(())
}

fn validate_local_ref(reference: &str) -> Result<(), GitMutationError> {
    let branch = reference.strip_prefix("refs/heads/").ok_or(GitMutationError::DetachedHead)?;
    validate_branch_name(branch)
}

fn clamp_i64_u32(value: i64) -> u32 { u32::try_from(value).unwrap_or(0) }

fn hex20(bytes: [u8; 20]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(40);
    for byte in bytes { out.push(char::from(DIGITS[(byte >> 4) as usize])); out.push(char::from(DIGITS[(byte & 0x0f) as usize])); }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_branch_profile_excludes_ref_paths_and_destructive_syntax() {
        for denied in ["", ".hidden", "a/b", "a..b", "main.lock", "refs-heads-main", "bad name"] {
            assert!(matches!(validate_branch_name(denied), Err(GitMutationError::InvalidBranchName)));
        }
        for allowed in ["main", "feature-1", "release_2.0"] { validate_branch_name(allowed).unwrap(); }
    }

    #[test]
    fn commit_metadata_rejects_authority_ambiguous_newlines() {
        let invalid = GitCommitMetadata { author_name: "owner\nforged".into(), author_email: "local@example.invalid".into(), timestamp_seconds: 1, message: "message".into() };
        assert!(matches!(invalid.validate(), Err(GitMutationError::InvalidCommitMetadata)));
    }
}
