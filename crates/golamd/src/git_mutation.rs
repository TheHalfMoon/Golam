#![forbid(unsafe_code)]

#[cfg(unix)]
use std::collections::BTreeMap;
use std::error::Error;
#[cfg(unix)]
use std::fs;
use std::fmt;
use std::io;
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;

use golam_core::digest::sha256;
use golam_core::target_identity::{FileMutationExpectation, ObservedFileKind};
use golam_core::tool_request::{BindingDigest, RequestedOperationId, RequestedTarget};
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use golam_kernel::PreparedToolEffect;
#[cfg(unix)]
use miniz_oxide::deflate::compress_to_vec_zlib;

#[cfg(unix)]
use crate::git_index::{GitIndex, GitIndexBounds, GitIndexEntry, GitIndexMode, GitIndexVersion, parse_git_index};
use crate::git_index::GitIndexError;
#[cfg(unix)]
use crate::git_observe::GitTreeMode;
use crate::git_read::{GitHeadRepresentation, GitObjectId, GitReadError};
#[cfg(unix)]
use crate::git_read::{GitObjectKind, GitReadBounds, GitRefSource, GitRepositoryReader};
use crate::git_sha1::{GitObjectSha1, GitObjectSha1Error};
use crate::git_status::{GitChangeKind, GitDiffEvidence, GitStatusBounds, GitStatusError, GitStatusObservation, observe_status};
use crate::local_fs::{LocalFsResolutionError, LocalFsResolver};
#[cfg(unix)]
use crate::local_fs::metadata_matches_resolved_identity;
use crate::local_read::LocalFileReadError;
#[cfg(unix)]
use crate::local_read::{LocalFileReadBounds, read_regular_file};

const EXPECTATION_DOMAIN: &[u8] = b"golam:git-mutation-expectation:v2";
const STATUS_DOMAIN: &[u8] = b"golam:git-status-state:v2";
const ADD_PRECONDITION_DOMAIN: &[u8] = b"golam:git-add-preconditions:v2";
const ADD_PAYLOAD_DOMAIN: &[u8] = b"golam:git-add-payload:v2";
const COMMIT_PRECONDITION_DOMAIN: &[u8] = b"golam:git-commit-preconditions:v2";
const COMMIT_PAYLOAD_DOMAIN: &[u8] = b"golam:git-commit-payload:v2";
const BRANCH_PRECONDITION_DOMAIN: &[u8] = b"golam:git-branch-preconditions:v2";
const BRANCH_PAYLOAD_DOMAIN: &[u8] = b"golam:git-branch-payload:v2";
const MAX_ADD_BYTES: u64 = 32 * 1024 * 1024;
const MAX_OBJECT_BYTES: usize = 32 * 1024 * 1024;
const MAX_COMMIT_MESSAGE_BYTES: usize = 64 * 1024;
const MAX_IDENTITY_BYTES: usize = 256;
const MAX_BRANCH_BYTES: usize = 128;
const MAX_INDEX_PATH_BYTES: usize = 4096;

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
        validate_author_name(&self.author_name)?;
        validate_author_email(&self.author_email)?;
        if self.timestamp_seconds < 0
            || self.message.is_empty()
            || self.message.len() > MAX_COMMIT_MESSAGE_BYTES
            || self.message.ends_with('\n')
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
    GitRead(GitReadError),
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
    NothingToAdd,
    NothingToCommit,
    DetachedHead,
    PackedOrSymbolicHeadRef,
    InvalidBranchName,
    BranchExists,
    InvalidCommitMetadata,
    MutationTooLarge,
    ObjectCollision(PathBuf),
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
            Self::GitRead(error) => write!(f, "Git mutation object verification failed: {error}"),
            Self::Index(error) => write!(f, "Git mutation index validation failed: {error}"),
            Self::Status(error) => write!(f, "Git mutation status observation failed: {error}"),
            Self::Resolution(error) => write!(f, "Git mutation target resolution failed: {error}"),
            Self::LocalRead(error) => write!(f, "Git mutation bounded read failed: {error}"),
            Self::UnsupportedPlatform => {
                f.write_str("governed Git mutation is not qualified on this platform")
            }
            Self::InvalidEffectBinding => {
                f.write_str("prepared tool effect does not bind this exact Git mutation")
            }
            Self::StaleRepository => {
                f.write_str("Git repository HEAD/index/status preconditions are stale")
            }
            Self::StaleWorktree => f.write_str("Git worktree target precondition is stale"),
            Self::InvalidTarget => {
                f.write_str("Git mutation target is invalid for the first profile")
            }
            Self::UnsupportedIndexProfile => {
                f.write_str("Git mutation requires a canonical extension-free v2 stage-zero index")
            }
            Self::ConflictedIndex => {
                f.write_str("Git mutation refuses conflicted or special index entries")
            }
            Self::NothingToAdd => f.write_str("Git add requires the bound target to change index state"),
            Self::NothingToCommit => {
                f.write_str("Git commit requires staged changes and a clean worktree")
            }
            Self::DetachedHead => f.write_str("Git commit requires a symbolic local branch HEAD"),
            Self::PackedOrSymbolicHeadRef => f.write_str(
                "Git commit requires one direct loose local branch ref in the first mutation profile",
            ),
            Self::InvalidBranchName => {
                f.write_str("Git branch name is outside the bounded ordinary-authority profile")
            }
            Self::BranchExists => {
                f.write_str("Git branch already exists and ordinary authority cannot move it")
            }
            Self::InvalidCommitMetadata => {
                f.write_str("Git commit metadata is empty, unbounded, or non-canonical")
            }
            Self::MutationTooLarge => {
                f.write_str("Git mutation input exceeds the bounded first-profile limit")
            }
            Self::ObjectCollision(path) => write!(
                f,
                "existing loose Git object failed exact content-addressed verification at {}",
                path.display()
            ),
            Self::StagingCollision(path) => write!(
                f,
                "Git mutation staging/guard entry already exists at {}",
                path.display()
            ),
            Self::UnknownOutcome(path) => write!(
                f,
                "Git mutation completion is ambiguous and requires reconciliation at {}",
                path.display()
            ),
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
            Self::GitRead(error) => Some(error),
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
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<CoreError> for GitMutationError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<GitObjectSha1Error> for GitMutationError {
    fn from(value: GitObjectSha1Error) -> Self {
        Self::Sha1(value)
    }
}

impl From<GitReadError> for GitMutationError {
    fn from(value: GitReadError) -> Self {
        Self::GitRead(value)
    }
}

impl From<GitIndexError> for GitMutationError {
    fn from(value: GitIndexError) -> Self {
        Self::Index(value)
    }
}

impl From<GitStatusError> for GitMutationError {
    fn from(value: GitStatusError) -> Self {
        Self::Status(value)
    }
}

impl From<LocalFsResolutionError> for GitMutationError {
    fn from(value: LocalFsResolutionError) -> Self {
        Self::Resolution(value)
    }
}

impl From<LocalFileReadError> for GitMutationError {
    fn from(value: LocalFileReadError) -> Self {
        Self::LocalRead(value)
    }
}

#[cfg(unix)]
impl From<nix::errno::Errno> for GitMutationError {
    fn from(value: nix::errno::Errno) -> Self {
        Self::Unix(value)
    }
}

pub fn git_add_resource(path: &RequestedTarget) -> String {
    format!("git-add:{}", path.as_str())
}

pub const fn git_commit_resource() -> &'static str {
    "git-commit:HEAD"
}

pub fn git_branch_resource(branch: &str) -> String {
    format!("git-branch-create:{branch}")
}

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

pub fn git_commit_preconditions_hash(
    expectation: GitMutationExpectation,
) -> Result<[u8; 32], CoreError> {
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
    encoder.push_u64(u64::try_from(metadata.timestamp_seconds).map_err(|_| GitMutationError::InvalidCommitMetadata)?);
    encoder.push_bytes(metadata.message.as_bytes())?;
    Ok(sha256(&encoder.finish()))
}

pub fn git_branch_preconditions_hash(
    expectation: GitMutationExpectation,
    branch: &str,
) -> Result<[u8; 32], CoreError> {
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
        execute_git_add_unix(
            resolver,
            prepared,
            expectation,
            path,
            target_expectation,
            observed_at_unix_ms,
        )
    }
    #[cfg(not(unix))]
    {
        let _ = (
            resolver,
            prepared,
            expectation,
            path,
            target_expectation,
            observed_at_unix_ms,
        );
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
        execute_git_commit_unix(
            resolver,
            prepared,
            expectation,
            metadata,
            observed_at_unix_ms,
        )
    }
    #[cfg(not(unix))]
    {
        let _ = (
            resolver,
            prepared,
            expectation,
            metadata,
            observed_at_unix_ms,
        );
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
        execute_git_branch_create_unix(
            resolver,
            prepared,
            expectation,
            branch,
            observed_at_unix_ms,
        )
    }
    #[cfg(not(unix))]
    {
        let _ = (
            resolver,
            prepared,
            expectation,
            branch,
            observed_at_unix_ms,
        );
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

    target_expectation
        .validate()
        .map_err(|_| GitMutationError::InvalidTarget)?;
    let expected_content = target_expectation
        .expected_content_digest
        .ok_or(GitMutationError::InvalidTarget)?;
    if !target_expectation.expected_exists
        || target_expectation.expected_kind != Some(ObservedFileKind::RegularFile)
        || target_expectation.expected_identity.is_none()
        || target_expectation.expected_parent_identity.is_none()
    {
        return Err(GitMutationError::InvalidTarget);
    }
    if prepared.action() != "git.add"
        || prepared.resource() != git_add_resource(path)
        || prepared.preconditions_hash()
            != git_add_preconditions_hash(expectation, path, target_expectation)?
        || prepared.payload_hash() != git_add_payload_hash(path, expected_content)
    {
        return Err(GitMutationError::InvalidEffectBinding);
    }

    let operation =
        RequestedOperationId::new("git.add").map_err(|_| GitMutationError::InvalidEffectBinding)?;
    let before = observe_status(
        resolver,
        &operation,
        GitStatusBounds::default(),
        observed_at_unix_ms,
    )?;
    verify_expectation(&before, expectation)?;

    let read = read_regular_file(
        resolver,
        path,
        &operation,
        LocalFileReadBounds {
            max_bytes: MAX_ADD_BYTES,
            max_duration: GitStatusBounds::default().max_duration,
        },
        observed_at_unix_ms,
        observed_at_unix_ms,
    )?;
    if read.identity.resolved_target_identity != target_expectation.expected_identity
        || read.identity.file_kind != ObservedFileKind::RegularFile
        || read.content_digest != expected_content
        || target_expectation
            .expected_size
            .is_some_and(|size| size != read.bytes.len() as u64)
    {
        return Err(GitMutationError::StaleWorktree);
    }
    let root = resolver.resolve_read_target(
        &RequestedTarget::new(".").map_err(|_| GitMutationError::InvalidTarget)?,
        &operation,
        observed_at_unix_ms,
    )?;
    if root.resolved_target_identity != target_expectation.expected_parent_identity {
        return Err(GitMutationError::StaleWorktree);
    }
    let metadata = fs::symlink_metadata(Path::new(read.identity.normalized_path.as_str()))?;
    if !metadata.file_type().is_file()
        || !metadata_matches_resolved_identity(&read.identity, &metadata)?
    {
        return Err(GitMutationError::StaleWorktree);
    }

    let (mut index, index_bytes_before) = read_index(resolver, &operation, observed_at_unix_ms)?;
    require_mutable_index_profile(&index)?;
    let path_bytes = path.as_str().as_bytes().to_vec();
    validate_index_path(&path_bytes)?;

    let before_entry = index
        .entries
        .binary_search_by(|candidate| candidate.path.cmp(&path_bytes))
        .ok()
        .and_then(|position| index.entries.get(position))
        .map(|entry| entry.object_id);
    let blob_digest = git_object_digest("blob", &read.bytes)?;
    if before_entry == Some(blob_digest) {
        return Err(GitMutationError::NothingToAdd);
    }

    let final_precommit = observe_status(
        resolver,
        &operation,
        GitStatusBounds::default(),
        observed_at_unix_ms,
    )?;
    verify_expectation(&final_precommit, expectation)?;

    let blob_id = write_or_verify_loose_object(
        resolver,
        &operation,
        "blob",
        &read.bytes,
        observed_at_unix_ms,
    )?;

    let file_size =
        u32::try_from(metadata.len()).map_err(|_| GitMutationError::MutationTooLarge)?;
    let entry = GitIndexEntry {
        ctime_seconds: clamp_i64_u32(metadata.ctime()),
        ctime_nanoseconds: clamp_i64_u32(metadata.ctime_nsec()),
        mtime_seconds: clamp_i64_u32(metadata.mtime()),
        mtime_nanoseconds: clamp_i64_u32(metadata.mtime_nsec()),
        dev: truncate_u64_u32(metadata.dev()),
        ino: truncate_u64_u32(metadata.ino()),
        mode: GitIndexMode::RegularFile {
            executable: metadata.mode() & 0o111 != 0,
        },
        uid: metadata.uid(),
        gid: metadata.gid(),
        file_size,
        object_id: blob_id.bytes(),
        assume_valid: false,
        stage: 0,
        skip_worktree: false,
        intent_to_add: false,
        path: path_bytes.clone(),
    };
    match index
        .entries
        .binary_search_by(|candidate| candidate.path.cmp(&path_bytes))
    {
        Ok(position) => index.entries[position] = entry,
        Err(position) => index.entries.insert(position, entry),
    }
    let index_bytes_after = serialize_index_v2(&index)?;

    let state_before_index = observe_status(
        resolver,
        &operation,
        GitStatusBounds::default(),
        observed_at_unix_ms,
    )?;
    verify_logical_state_unchanged_after_object_write(&state_before_index, &before, expectation)?;

    replace_expected_relative(
        resolver,
        &operation,
        ".git",
        "index",
        &index_bytes_before,
        &index_bytes_after,
        prepared.effect_id(),
        observed_at_unix_ms,
    )?;

    let after = observe_status(
        resolver,
        &operation,
        GitStatusBounds::default(),
        observed_at_unix_ms,
    )?;
    verify_add_result(&before, &after, path.as_str(), blob_id)?;
    verify_loose_object(
        resolver,
        &operation,
        blob_id,
        GitObjectKind::Blob,
        &read.bytes,
        observed_at_unix_ms,
    )?;

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

    let operation = RequestedOperationId::new("git.commit")
        .map_err(|_| GitMutationError::InvalidEffectBinding)?;
    let before = observe_status(
        resolver,
        &operation,
        GitStatusBounds::default(),
        observed_at_unix_ms,
    )?;
    verify_expectation(&before, expectation)?;
    if before.staged.is_empty() || !before.worktree.is_empty() || !before.untracked.is_empty() {
        return Err(GitMutationError::NothingToCommit);
    }

    let head_ref = match &before.repository_evidence.head.representation {
        GitHeadRepresentation::Symbolic(name) if name.starts_with("refs/heads/") => name.clone(),
        _ => return Err(GitMutationError::DetachedHead),
    };
    let branch = validate_local_ref(&head_ref)?;
    let resolved_ref = before
        .repository_evidence
        .head
        .resolved_ref
        .as_ref()
        .ok_or(GitMutationError::PackedOrSymbolicHeadRef)?;
    if resolved_ref.source != GitRefSource::Loose
        || resolved_ref.symbolic_chain.len() != 1
        || resolved_ref.symbolic_chain.first().map(String::as_str) != Some(head_ref.as_str())
    {
        return Err(GitMutationError::PackedOrSymbolicHeadRef);
    }

    let (index, index_bytes) = read_index(resolver, &operation, observed_at_unix_ms)?;
    require_mutable_index_profile(&index)?;
    if GitObjectSha1::digest(&index_bytes[..index_bytes.len().saturating_sub(20)])?
        != before.index_checksum
    {
        return Err(GitMutationError::StaleRepository);
    }

    let final_precommit = observe_status(
        resolver,
        &operation,
        GitStatusBounds::default(),
        observed_at_unix_ms,
    )?;
    verify_expectation(&final_precommit, expectation)?;

    let tree_id = write_index_tree(resolver, &operation, &index, observed_at_unix_ms)?;
    let commit_bytes = build_commit_bytes(tree_id, before.head, metadata)?;
    let commit_id = write_or_verify_loose_object(
        resolver,
        &operation,
        "commit",
        &commit_bytes,
        observed_at_unix_ms,
    )?;

    let state_before_ref = observe_status(
        resolver,
        &operation,
        GitStatusBounds::default(),
        observed_at_unix_ms,
    )?;
    verify_logical_state_unchanged_after_object_write(&state_before_ref, &before, expectation)?;
    verify_loose_object(
        resolver,
        &operation,
        commit_id,
        GitObjectKind::Commit,
        &commit_bytes,
        observed_at_unix_ms,
    )?;

    let expected_ref = format!("{}\n", before.head.to_hex());
    let next_ref = format!("{}\n", commit_id.to_hex());
    replace_expected_relative(
        resolver,
        &operation,
        ".git/refs/heads",
        branch,
        expected_ref.as_bytes(),
        next_ref.as_bytes(),
        prepared.effect_id(),
        observed_at_unix_ms,
    )?;

    let after = observe_status(
        resolver,
        &operation,
        GitStatusBounds::default(),
        observed_at_unix_ms,
    )?;
    if after.head != commit_id
        || after.index_checksum != before.index_checksum
        || !after.staged.is_empty()
        || !after.worktree.is_empty()
        || !after.untracked.is_empty()
    {
        return Err(GitMutationError::UnknownOutcome(PathBuf::from(format!(
            ".git/refs/heads/{branch}"
        ))));
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

    let operation = RequestedOperationId::new("git.branch.create")
        .map_err(|_| GitMutationError::InvalidEffectBinding)?;
    let before = observe_status(
        resolver,
        &operation,
        GitStatusBounds::default(),
        observed_at_unix_ms,
    )?;
    verify_expectation(&before, expectation)?;
    if !before.staged.is_empty() || !before.worktree.is_empty() || !before.untracked.is_empty() {
        return Err(GitMutationError::StaleWorktree);
    }

    let final_precommit = observe_status(
        resolver,
        &operation,
        GitStatusBounds::default(),
        observed_at_unix_ms,
    )?;
    verify_expectation(&final_precommit, expectation)?;

    let bytes = format!("{}\n", before.head.to_hex());
    create_new_relative(
        resolver,
        &operation,
        ".git/refs/heads",
        branch,
        bytes.as_bytes(),
        observed_at_unix_ms,
    )?;

    let after = observe_status(
        resolver,
        &operation,
        GitStatusBounds::default(),
        observed_at_unix_ms,
    )?;
    if after.head != before.head
        || after.index_checksum != before.index_checksum
        || git_status_digest(&after)? != expectation.status_digest
    {
        return Err(GitMutationError::UnknownOutcome(PathBuf::from(format!(
            ".git/refs/heads/{branch}"
        ))));
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

fn verify_expectation(
    status: &GitStatusObservation,
    expected: GitMutationExpectation,
) -> Result<(), GitMutationError> {
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

fn verify_logical_state_unchanged_after_object_write(
    current: &GitStatusObservation,
    before: &GitStatusObservation,
    expected: GitMutationExpectation,
) -> Result<(), GitMutationError> {
    current.repository_evidence.verify_binding()?;
    if current.head != expected.head
        || current.index_checksum != expected.index_checksum
        || git_status_digest(current)? != expected.status_digest
        || current.staged != before.staged
        || current.worktree != before.worktree
        || current.untracked != before.untracked
        || current.repository_evidence.repository_root.normalized_path
            != before.repository_evidence.repository_root.normalized_path
        || current.repository_evidence.git_directory.normalized_path
            != before.repository_evidence.git_directory.normalized_path
        || current.repository_evidence.object_store_directory.normalized_path
            != before.repository_evidence.object_store_directory.normalized_path
    {
        return Err(GitMutationError::StaleRepository);
    }
    Ok(())
}

#[cfg(unix)]
fn verify_add_result(
    before: &GitStatusObservation,
    after: &GitStatusObservation,
    path: &str,
    blob_id: GitObjectId,
) -> Result<(), GitMutationError> {
    if after.head != before.head || after.index_checksum == before.index_checksum {
        return Err(GitMutationError::UnknownOutcome(PathBuf::from(".git/index")));
    }
    let staged_target = after
        .staged
        .iter()
        .find(|change| change.path == path)
        .ok_or_else(|| GitMutationError::UnknownOutcome(PathBuf::from(".git/index")))?;
    if staged_target.after != Some(blob_id)
        || after.worktree.iter().any(|change| change.path == path)
        || after.untracked.iter().any(|candidate| candidate == path)
        || changes_without_path(&after.staged, path) != changes_without_path(&before.staged, path)
        || changes_without_path(&after.worktree, path)
            != changes_without_path(&before.worktree, path)
        || untracked_without_path(&after.untracked, path)
            != untracked_without_path(&before.untracked, path)
    {
        return Err(GitMutationError::UnknownOutcome(PathBuf::from(".git/index")));
    }
    Ok(())
}

fn changes_without_path(changes: &[GitDiffEvidence], path: &str) -> Vec<GitDiffEvidence> {
    changes
        .iter()
        .filter(|change| change.path != path)
        .cloned()
        .collect()
}

fn untracked_without_path(paths: &[String], path: &str) -> Vec<String> {
    paths
        .iter()
        .filter(|candidate| candidate.as_str() != path)
        .cloned()
        .collect()
}

pub fn git_status_digest(status: &GitStatusObservation) -> Result<BindingDigest, GitMutationError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(STATUS_DOMAIN)?;
    encoder.push_bytes(&status.head.bytes())?;
    encoder.push_bytes(&status.index_checksum)?;
    push_changes(&mut encoder, &status.staged)?;
    push_changes(&mut encoder, &status.worktree)?;
    encoder.push_u64(
        u64::try_from(status.untracked.len()).map_err(|_| CoreError::CanonicalLengthOverflow)?,
    );
    for path in &status.untracked {
        encoder.push_bytes(path.as_bytes())?;
    }
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn push_changes(
    encoder: &mut CanonicalEncoder,
    changes: &[GitDiffEvidence],
) -> Result<(), CoreError> {
    encoder.push_u64(
        u64::try_from(changes.len()).map_err(|_| CoreError::CanonicalLengthOverflow)?,
    );
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

fn push_object_id(
    encoder: &mut CanonicalEncoder,
    value: Option<GitObjectId>,
) -> Result<(), CoreError> {
    match value {
        Some(id) => {
            encoder.push_u8(1);
            encoder.push_bytes(&id.bytes())?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn push_expectation(
    encoder: &mut CanonicalEncoder,
    value: GitMutationExpectation,
) -> Result<(), CoreError> {
    encoder.push_bytes(EXPECTATION_DOMAIN)?;
    encoder.push_bytes(&value.repository_binding.bytes())?;
    encoder.push_bytes(&value.head.bytes())?;
    encoder.push_bytes(&value.index_checksum)?;
    encoder.push_bytes(&value.status_digest.bytes())?;
    Ok(())
}

fn push_file_expectation(
    encoder: &mut CanonicalEncoder,
    value: FileMutationExpectation,
) -> Result<(), CoreError> {
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
    match value.expected_size {
        Some(size) => {
            encoder.push_u8(1);
            encoder.push_u64(size);
        }
        None => encoder.push_u8(0),
    }
    push_digest(encoder, value.expected_parent_identity)?;
    Ok(())
}

fn push_digest(
    encoder: &mut CanonicalEncoder,
    value: Option<BindingDigest>,
) -> Result<(), CoreError> {
    match value {
        Some(digest) => {
            encoder.push_u8(1);
            encoder.push_bytes(&digest.bytes())?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

#[cfg(unix)]
fn read_index(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    observed_at_unix_ms: u64,
) -> Result<(GitIndex, Vec<u8>), GitMutationError> {
    let request =
        RequestedTarget::new(".git/index").map_err(|_| GitMutationError::InvalidTarget)?;
    let read = read_regular_file(
        resolver,
        &request,
        operation,
        LocalFileReadBounds {
            max_bytes: u64::try_from(crate::git_index::MAX_INDEX_BYTES)
                .map_err(|_| GitMutationError::MutationTooLarge)?,
            max_duration: GitStatusBounds::default().max_duration,
        },
        observed_at_unix_ms,
        observed_at_unix_ms,
    )?;
    let index = parse_git_index(&read.bytes, GitIndexBounds::default())?;
    Ok((index, read.bytes))
}

#[cfg(unix)]
fn require_mutable_index_profile(index: &GitIndex) -> Result<(), GitMutationError> {
    if index.version != GitIndexVersion::V2 || !index.extensions.is_empty() {
        return Err(GitMutationError::UnsupportedIndexProfile);
    }
    if index.entries.iter().any(|entry| {
        entry.stage != 0
            || entry.skip_worktree
            || entry.intent_to_add
            || !matches!(entry.mode, GitIndexMode::RegularFile { .. })
    }) {
        return Err(GitMutationError::ConflictedIndex);
    }
    Ok(())
}

#[cfg(unix)]
fn serialize_index_v2(index: &GitIndex) -> Result<Vec<u8>, GitMutationError> {
    require_mutable_index_profile(index)?;
    let mut output = Vec::new();
    output.extend_from_slice(b"DIRC");
    output.extend_from_slice(&2_u32.to_be_bytes());
    output.extend_from_slice(
        &u32::try_from(index.entries.len())
            .map_err(|_| GitMutationError::MutationTooLarge)?
            .to_be_bytes(),
    );
    let mut previous: Option<&[u8]> = None;
    for entry in &index.entries {
        validate_index_path(&entry.path)?;
        if previous.is_some_and(|path| path >= entry.path.as_slice()) {
            return Err(GitMutationError::UnsupportedIndexProfile);
        }
        previous = Some(&entry.path);
        let start = output.len();
        for value in [
            entry.ctime_seconds,
            entry.ctime_nanoseconds,
            entry.mtime_seconds,
            entry.mtime_nanoseconds,
            entry.dev,
            entry.ino,
        ] {
            output.extend_from_slice(&value.to_be_bytes());
        }
        output.extend_from_slice(&index_mode_bits(entry.mode).to_be_bytes());
        for value in [entry.uid, entry.gid, entry.file_size] {
            output.extend_from_slice(&value.to_be_bytes());
        }
        output.extend_from_slice(&entry.object_id);
        let mut flags = u16::try_from(entry.path.len().min(0x0fff))
            .map_err(|_| GitMutationError::MutationTooLarge)?;
        if entry.assume_valid {
            flags |= 0x8000;
        }
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
    if parsed.entries != index.entries {
        return Err(GitMutationError::UnsupportedIndexProfile);
    }
    Ok(output)
}

#[cfg(unix)]
fn validate_index_path(path: &[u8]) -> Result<(), GitMutationError> {
    if path.is_empty()
        || path.len() > MAX_INDEX_PATH_BYTES
        || path.first() == Some(&b'/')
        || path.last() == Some(&b'/')
        || path.contains(&0)
        || path.split(|byte| *byte == b'/').any(|component| {
            component.is_empty()
                || component == b"."
                || component == b".."
                || component.eq_ignore_ascii_case(b".git")
        })
    {
        return Err(GitMutationError::InvalidTarget);
    }
    Ok(())
}

#[cfg(unix)]
fn index_mode_bits(mode: GitIndexMode) -> u32 {
    match mode {
        GitIndexMode::RegularFile { executable: false } => 0o100644,
        GitIndexMode::RegularFile { executable: true } => 0o100755,
        GitIndexMode::SymbolicLink => 0o120000,
        GitIndexMode::Gitlink => 0o160000,
    }
}

#[cfg(unix)]
fn write_index_tree(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    index: &GitIndex,
    observed_at_unix_ms: u64,
) -> Result<GitObjectId, GitMutationError> {
    let mut root = TreeNode::default();
    for entry in &index.entries {
        let path = std::str::from_utf8(&entry.path).map_err(|_| GitMutationError::InvalidTarget)?;
        root.insert(
            path,
            entry.mode,
            GitObjectId::parse(&hex20(entry.object_id))
                .map_err(|_| GitMutationError::InvalidTarget)?,
        )?;
    }
    write_tree_node(resolver, operation, &root, observed_at_unix_ms)
}

#[cfg(unix)]
#[derive(Default)]
struct TreeNode {
    files: BTreeMap<Vec<u8>, (GitIndexMode, GitObjectId)>,
    dirs: BTreeMap<Vec<u8>, TreeNode>,
}

#[cfg(unix)]
impl TreeNode {
    fn insert(
        &mut self,
        path: &str,
        mode: GitIndexMode,
        id: GitObjectId,
    ) -> Result<(), GitMutationError> {
        let parts = path
            .as_bytes()
            .split(|byte| *byte == b'/')
            .collect::<Vec<_>>();
        if parts.is_empty() || parts.iter().any(|part| part.is_empty()) {
            return Err(GitMutationError::InvalidTarget);
        }
        self.insert_parts(&parts, mode, id)
    }

    fn insert_parts(
        &mut self,
        parts: &[&[u8]],
        mode: GitIndexMode,
        id: GitObjectId,
    ) -> Result<(), GitMutationError> {
        if parts.len() == 1 {
            if self.dirs.contains_key(parts[0])
                || self.files.insert(parts[0].to_vec(), (mode, id)).is_some()
            {
                return Err(GitMutationError::InvalidTarget);
            }
            return Ok(());
        }
        if self.files.contains_key(parts[0]) {
            return Err(GitMutationError::InvalidTarget);
        }
        self.dirs
            .entry(parts[0].to_vec())
            .or_default()
            .insert_parts(&parts[1..], mode, id)
    }
}

#[cfg(unix)]
fn write_tree_node(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    node: &TreeNode,
    observed_at_unix_ms: u64,
) -> Result<GitObjectId, GitMutationError> {
    let mut entries = Vec::<(Vec<u8>, GitTreeMode, GitObjectId)>::new();
    for (name, child) in &node.dirs {
        let id = write_tree_node(resolver, operation, child, observed_at_unix_ms)?;
        entries.push((name.clone(), GitTreeMode::Directory, id));
    }
    for (name, (mode, id)) in &node.files {
        let tree_mode = match mode {
            GitIndexMode::RegularFile { executable } => GitTreeMode::RegularFile {
                executable: *executable,
            },
            _ => return Err(GitMutationError::ConflictedIndex),
        };
        entries.push((name.clone(), tree_mode, *id));
    }
    entries.sort_by(|left, right| {
        tree_sort_key(&left.0, left.1).cmp(&tree_sort_key(&right.0, right.1))
    });
    let mut bytes = Vec::new();
    for (name, mode, id) in entries {
        bytes.extend_from_slice(tree_mode_text(mode));
        bytes.push(b' ');
        bytes.extend_from_slice(&name);
        bytes.push(0);
        bytes.extend_from_slice(&id.bytes());
    }
    write_or_verify_loose_object(resolver, operation, "tree", &bytes, observed_at_unix_ms)
}

#[cfg(unix)]
fn tree_sort_key(name: &[u8], mode: GitTreeMode) -> Vec<u8> {
    let mut key = name.to_vec();
    if mode == GitTreeMode::Directory {
        key.push(b'/');
    }
    key
}

#[cfg(unix)]
fn tree_mode_text(mode: GitTreeMode) -> &'static [u8] {
    match mode {
        GitTreeMode::RegularFile { executable: false } => b"100644",
        GitTreeMode::RegularFile { executable: true } => b"100755",
        GitTreeMode::SymbolicLink => b"120000",
        GitTreeMode::Directory => b"40000",
        GitTreeMode::Gitlink => b"160000",
    }
}

#[cfg(unix)]
fn build_commit_bytes(
    tree: GitObjectId,
    parent: GitObjectId,
    metadata: &GitCommitMetadata,
) -> Result<Vec<u8>, GitMutationError> {
    metadata.validate()?;
    let identity = format!(
        "{} <{}> {} +0000",
        metadata.author_name, metadata.author_email, metadata.timestamp_seconds
    );
    Ok(format!(
        "tree {}\nparent {}\nauthor {identity}\ncommitter {identity}\n\n{}\n",
        tree.to_hex(),
        parent.to_hex(),
        metadata.message
    )
    .into_bytes())
}

#[cfg(unix)]
fn git_object_digest(kind: &str, body: &[u8]) -> Result<[u8; 20], GitMutationError> {
    if body.len() > MAX_OBJECT_BYTES || !matches!(kind, "blob" | "tree" | "commit") {
        return Err(GitMutationError::MutationTooLarge);
    }
    let mut canonical = format!("{kind} {}\0", body.len()).into_bytes();
    canonical.extend_from_slice(body);
    Ok(GitObjectSha1::digest(&canonical)?)
}

#[cfg(unix)]
fn write_or_verify_loose_object(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    kind: &str,
    body: &[u8],
    observed_at_unix_ms: u64,
) -> Result<GitObjectId, GitMutationError> {
    use std::fs::File;
    use std::io::Write;
    use std::os::unix::fs::MetadataExt;

    use nix::fcntl::{OFlag, open, openat};
    use nix::sys::stat::{Mode, mkdirat};

    let digest = git_object_digest(kind, body)?;
    let id = GitObjectId::parse(&hex20(digest)).map_err(|_| GitMutationError::InvalidTarget)?;
    let objects_request =
        RequestedTarget::new(".git/objects").map_err(|_| GitMutationError::InvalidTarget)?;
    let objects_identity =
        resolver.resolve_read_target(&objects_request, operation, observed_at_unix_ms)?;
    if objects_identity.file_kind != ObservedFileKind::Directory {
        return Err(GitMutationError::StaleRepository);
    }
    let objects_fd = open(
        Path::new(objects_identity.normalized_path.as_str()),
        OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )?;
    let objects = File::from(objects_fd);
    let objects_metadata = objects.metadata()?;
    if !metadata_matches_resolved_identity(&objects_identity, &objects_metadata)? {
        return Err(GitMutationError::StaleRepository);
    }

    let hex = id.to_hex();
    let fanout_name = &hex[..2];
    match mkdirat(
        &objects,
        fanout_name,
        Mode::from_bits_truncate(0o700),
    ) {
        Ok(()) => objects.sync_all()?,
        Err(nix::errno::Errno::EEXIST) => {}
        Err(error) => return Err(GitMutationError::Unix(error)),
    }
    let fanout_fd = openat(
        &objects,
        fanout_name,
        OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )?;
    let fanout = File::from(fanout_fd);
    let fanout_metadata = fanout.metadata()?;
    if !fanout_metadata.is_dir() || fanout_metadata.dev() != objects_metadata.dev() {
        return Err(GitMutationError::StaleRepository);
    }

    let object_name = &hex[2..];
    let mut canonical = format!("{kind} {}\0", body.len()).into_bytes();
    canonical.extend_from_slice(body);
    let compressed = compress_to_vec_zlib(&canonical, 6);
    match openat(
        &fanout,
        object_name,
        OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::from_bits_truncate(0o600),
    ) {
        Ok(fd) => {
            let mut file = File::from(fd);
            file.write_all(&compressed)?;
            file.sync_all()?;
            fanout.sync_all()?;
        }
        Err(nix::errno::Errno::EEXIST) => {}
        Err(error) => return Err(GitMutationError::Unix(error)),
    }

    let expected_kind = match kind {
        "blob" => GitObjectKind::Blob,
        "tree" => GitObjectKind::Tree,
        "commit" => GitObjectKind::Commit,
        _ => return Err(GitMutationError::InvalidTarget),
    };
    verify_loose_object(
        resolver,
        operation,
        id,
        expected_kind,
        body,
        observed_at_unix_ms,
    )
    .map_err(|error| match error {
        GitMutationError::GitRead(_) => GitMutationError::ObjectCollision(PathBuf::from(format!(
            ".git/objects/{fanout_name}/{object_name}"
        ))),
        other => other,
    })?;
    Ok(id)
}

#[cfg(unix)]
fn verify_loose_object(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    id: GitObjectId,
    expected_kind: GitObjectKind,
    expected_body: &[u8],
    observed_at_unix_ms: u64,
) -> Result<(), GitMutationError> {
    let reader = GitRepositoryReader::open(
        resolver,
        operation,
        GitReadBounds::default(),
        observed_at_unix_ms,
    )?;
    let object = reader.read_loose_object(id, observed_at_unix_ms)?;
    if object.id != id || object.kind != expected_kind || object.bytes != expected_body {
        return Err(GitMutationError::ObjectCollision(PathBuf::from(format!(
            ".git/objects/{}/{}",
            &id.to_hex()[..2],
            &id.to_hex()[2..]
        ))));
    }
    Ok(())
}

#[cfg(unix)]
#[allow(
    clippy::too_many_arguments,
    reason = "conditional Git replacement keeps authority, parent, prior bytes, next bytes, and effect identity explicit"
)]
fn replace_expected_relative(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    parent_request: &str,
    target_name: &str,
    expected: &[u8],
    next: &[u8],
    effect_id: EffectId,
    observed_at_unix_ms: u64,
) -> Result<(), GitMutationError> {
    use std::fs::File;
    use std::io::{Read, Write};

    use nix::fcntl::{AtFlags, OFlag, open, openat, renameat};
    use nix::sys::stat::Mode;
    use nix::unistd::{UnlinkatFlags, linkat, unlinkat};

    let parent_requested =
        RequestedTarget::new(parent_request).map_err(|_| GitMutationError::InvalidTarget)?;
    let parent_identity =
        resolver.resolve_read_target(&parent_requested, operation, observed_at_unix_ms)?;
    if parent_identity.file_kind != ObservedFileKind::Directory {
        return Err(GitMutationError::StaleRepository);
    }
    let parent_fd = open(
        Path::new(parent_identity.normalized_path.as_str()),
        OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )?;
    let parent = File::from(parent_fd);
    if !metadata_matches_resolved_identity(&parent_identity, &parent.metadata()?)? {
        return Err(GitMutationError::StaleRepository);
    }

    let token = format!("{:032x}", effect_id.0);
    let temp_name = format!(".golam-{token}.next");
    let guard_name = format!(".golam-{token}.previous");
    require_missing_at(&parent, &temp_name)?;
    require_missing_at(&parent, &guard_name)?;

    let temp_fd = openat(
        &parent,
        temp_name.as_str(),
        OFlag::O_RDWR | OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::from_bits_truncate(0o600),
    )?;
    let mut temp = File::from(temp_fd);
    temp.write_all(next)?;
    temp.sync_all()?;
    let temp_metadata = temp.metadata()?;

    let current_fd = openat(
        &parent,
        target_name,
        OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )?;
    let mut current = File::from(current_fd);
    let mut current_bytes = Vec::new();
    current.read_to_end(&mut current_bytes)?;
    if current_bytes != expected {
        cleanup_at(&parent, &temp_name);
        return Err(GitMutationError::StaleRepository);
    }

    renameat(&parent, target_name, &parent, guard_name.as_str())?;
    let guard_fd = match openat(
        &parent,
        guard_name.as_str(),
        OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    ) {
        Ok(fd) => fd,
        Err(_) => {
            return Err(GitMutationError::UnknownOutcome(PathBuf::from(format!(
                "{parent_request}/{guard_name}"
            ))));
        }
    };
    let mut guard = File::from(guard_fd);
    let guard_metadata = guard.metadata()?;
    let mut guard_bytes = Vec::new();
    guard.read_to_end(&mut guard_bytes)?;
    if !same_unix_object(&current.metadata()?, &guard_metadata) || guard_bytes != expected {
        return restore_or_preserve_conflict(
            &parent,
            target_name,
            &guard_name,
            &temp_name,
            parent_request,
        );
    }

    if linkat(
        &parent,
        temp_name.as_str(),
        &parent,
        target_name,
        AtFlags::empty(),
    )
    .is_err()
    {
        return restore_or_preserve_conflict(
            &parent,
            target_name,
            &guard_name,
            &temp_name,
            parent_request,
        );
    }

    let installed_fd = openat(
        &parent,
        target_name,
        OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )?;
    let mut installed = File::from(installed_fd);
    let installed_metadata = installed.metadata()?;
    let mut installed_bytes = Vec::new();
    installed.read_to_end(&mut installed_bytes)?;
    if !same_unix_object(&temp_metadata, &installed_metadata) || installed_bytes != next {
        return Err(GitMutationError::UnknownOutcome(PathBuf::from(format!(
            "{parent_request}/{guard_name}"
        ))));
    }

    unlinkat(
        &parent,
        temp_name.as_str(),
        UnlinkatFlags::NoRemoveDir,
    )?;
    if unlinkat(
        &parent,
        guard_name.as_str(),
        UnlinkatFlags::NoRemoveDir,
    )
    .is_err()
    {
        return Err(GitMutationError::UnknownOutcome(PathBuf::from(format!(
            "{parent_request}/{guard_name}"
        ))));
    }
    parent.sync_all()?;
    Ok(())
}

#[cfg(unix)]
fn restore_or_preserve_conflict(
    parent: &fs::File,
    target_name: &str,
    guard_name: &str,
    temp_name: &str,
    parent_request: &str,
) -> Result<(), GitMutationError> {
    use nix::fcntl::AtFlags;
    use nix::unistd::linkat;

    let restored = linkat(
        parent,
        guard_name,
        parent,
        target_name,
        AtFlags::empty(),
    )
    .is_ok();
    cleanup_at(parent, temp_name);
    if restored {
        cleanup_at(parent, guard_name);
        parent.sync_all()?;
        Err(GitMutationError::StaleRepository)
    } else {
        Err(GitMutationError::UnknownOutcome(PathBuf::from(format!(
            "{parent_request}/{guard_name}"
        ))))
    }
}

#[cfg(unix)]
fn create_new_relative(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    parent_request: &str,
    target_name: &str,
    bytes: &[u8],
    observed_at_unix_ms: u64,
) -> Result<(), GitMutationError> {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom, Write};

    use nix::fcntl::{OFlag, open, openat};
    use nix::sys::stat::Mode;

    let parent_requested =
        RequestedTarget::new(parent_request).map_err(|_| GitMutationError::InvalidTarget)?;
    let parent_identity =
        resolver.resolve_read_target(&parent_requested, operation, observed_at_unix_ms)?;
    if parent_identity.file_kind != ObservedFileKind::Directory {
        return Err(GitMutationError::StaleRepository);
    }
    let parent_fd = open(
        Path::new(parent_identity.normalized_path.as_str()),
        OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )?;
    let parent = File::from(parent_fd);
    if !metadata_matches_resolved_identity(&parent_identity, &parent.metadata()?)? {
        return Err(GitMutationError::StaleRepository);
    }

    let created_fd = match openat(
        &parent,
        target_name,
        OFlag::O_RDWR | OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::from_bits_truncate(0o600),
    ) {
        Ok(fd) => fd,
        Err(nix::errno::Errno::EEXIST) => return Err(GitMutationError::BranchExists),
        Err(error) => return Err(GitMutationError::Unix(error)),
    };
    let mut created = File::from(created_fd);
    created.write_all(bytes)?;
    created.sync_all()?;
    let created_metadata = created.metadata()?;
    created.seek(SeekFrom::Start(0))?;
    let mut created_bytes = Vec::new();
    created.read_to_end(&mut created_bytes)?;
    if created_bytes != bytes {
        return Err(GitMutationError::UnknownOutcome(PathBuf::from(format!(
            "{parent_request}/{target_name}"
        ))));
    }

    let verify_fd = openat(
        &parent,
        target_name,
        OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )?;
    let mut verify = File::from(verify_fd);
    let verify_metadata = verify.metadata()?;
    let mut verify_bytes = Vec::new();
    verify.read_to_end(&mut verify_bytes)?;
    if !same_unix_object(&created_metadata, &verify_metadata) || verify_bytes != bytes {
        return Err(GitMutationError::UnknownOutcome(PathBuf::from(format!(
            "{parent_request}/{target_name}"
        ))));
    }
    parent.sync_all()?;
    Ok(())
}

#[cfg(unix)]
fn require_missing_at(parent: &fs::File, name: &str) -> Result<(), GitMutationError> {
    use nix::fcntl::{OFlag, openat};
    use nix::sys::stat::Mode;

    match openat(
        parent,
        name,
        OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    ) {
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

fn validate_author_name(value: &str) -> Result<(), GitMutationError> {
    if value.is_empty()
        || value.len() > MAX_IDENTITY_BYTES
        || value.contains(['\n', '\r', '\0', '<', '>'])
    {
        return Err(GitMutationError::InvalidCommitMetadata);
    }
    Ok(())
}

fn validate_author_email(value: &str) -> Result<(), GitMutationError> {
    if value.is_empty()
        || value.len() > MAX_IDENTITY_BYTES
        || !value.contains('@')
        || value.chars().any(char::is_whitespace)
        || value.contains(['\n', '\r', '\0', '<', '>'])
    {
        return Err(GitMutationError::InvalidCommitMetadata);
    }
    Ok(())
}

fn validate_branch_name(branch: &str) -> Result<(), GitMutationError> {
    if branch.is_empty()
        || branch.len() > MAX_BRANCH_BYTES
        || branch.starts_with(['.', '-'])
        || branch.ends_with('.')
        || branch.ends_with(".lock")
        || branch.contains("..")
        || branch.bytes().any(|byte| {
            !(byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        })
    {
        return Err(GitMutationError::InvalidBranchName);
    }
    Ok(())
}

fn validate_local_ref(reference: &str) -> Result<&str, GitMutationError> {
    let branch = reference
        .strip_prefix("refs/heads/")
        .ok_or(GitMutationError::DetachedHead)?;
    validate_branch_name(branch)?;
    Ok(branch)
}

#[cfg(unix)]
fn clamp_i64_u32(value: i64) -> u32 {
    u32::try_from(value).unwrap_or(0)
}

#[cfg(unix)]
fn truncate_u64_u32(value: u64) -> u32 {
    u32::try_from(value & u64::from(u32::MAX)).expect("masked Unix metadata fits u32")
}

#[cfg(unix)]
fn hex20(bytes: [u8; 20]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(40);
    for byte in bytes {
        out.push(char::from(DIGITS[(byte >> 4) as usize]));
        out.push(char::from(DIGITS[(byte & 0x0f) as usize]));
    }
    out
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use golam_core::paths::RuntimeLayout;
    use golam_core::tool_request::ResourceClassId;
    use golam_core::{EventId, SessionId};
    use golam_kernel::{
        AuthorizationPolicy, AuthorizationRequest, CompleteToolEffect, KernelApi,
        KernelCreateSession, PolicyDecision, PrepareToolEffect, Principal, ToolExecutionCompletion,
    };

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct AllowGit;

    impl AuthorizationPolicy for AllowGit {
        fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
            PolicyDecision::allow("phase_f_git_mutation_qualification")
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
                    Principal::test("phase-f-git"),
                    KernelCreateSession {
                        session_id: SessionId(63),
                        event_id: EventId(1),
                        recorded_at: "2026-09-04T13:00:00Z",
                        payload: b"phase-f-git-mutation-qualification",
                    },
                    "phase-f-git",
                )
                .unwrap();
            Self {
                base,
                repo,
                resolver,
                kernel,
            }
        }

        fn status(&self, operation: &str) -> GitStatusObservation {
            observe_status(
                &self.resolver,
                &RequestedOperationId::new(operation).unwrap(),
                GitStatusBounds::default(),
                100,
            )
            .unwrap()
        }

        fn file_expectation(
            &self,
            target: &RequestedTarget,
            operation: &str,
            bytes: &[u8],
        ) -> FileMutationExpectation {
            let operation = RequestedOperationId::new(operation).unwrap();
            let identity = self
                .resolver
                .resolve_read_target(target, &operation, 100)
                .unwrap();
            let root = self
                .resolver
                .resolve_read_target(&RequestedTarget::new(".").unwrap(), &operation, 100)
                .unwrap();
            FileMutationExpectation {
                expected_exists: true,
                expected_kind: Some(ObservedFileKind::RegularFile),
                expected_identity: identity.resolved_target_identity,
                expected_content_digest: Some(BindingDigest::new(sha256(bytes))),
                expected_size: Some(bytes.len() as u64),
                expected_parent_identity: root.resolved_target_identity,
            }
        }

        fn prepare_add(
            &mut self,
            effect: u128,
            expectation: GitMutationExpectation,
            target: &RequestedTarget,
            file: FileMutationExpectation,
        ) -> PreparedToolEffect {
            let resource = git_add_resource(target);
            self.kernel
                .prepare_tool_effect(
                    Principal::test("phase-f-git"),
                    PrepareToolEffect {
                        effect_id: EffectId(effect),
                        session_id: SessionId(63),
                        action: "git.add",
                        resource: &resource,
                        execution_semantics: "at_most_once",
                        handler_id: "golam-git-unix",
                        handler_version: "1",
                        idempotency_key: Some("phase-f-git-add"),
                        preconditions_hash: git_add_preconditions_hash(expectation, target, file)
                            .unwrap(),
                        payload_hash: git_add_payload_hash(
                            target,
                            file.expected_content_digest.unwrap(),
                        ),
                        started_at: "2026-09-04T13:00:01Z",
                    },
                    "phase-f-git",
                )
                .unwrap()
        }

        fn prepare_commit(
            &mut self,
            effect: u128,
            expectation: GitMutationExpectation,
            metadata: &GitCommitMetadata,
        ) -> PreparedToolEffect {
            self.kernel
                .prepare_tool_effect(
                    Principal::test("phase-f-git"),
                    PrepareToolEffect {
                        effect_id: EffectId(effect),
                        session_id: SessionId(63),
                        action: "git.commit",
                        resource: git_commit_resource(),
                        execution_semantics: "at_most_once",
                        handler_id: "golam-git-unix",
                        handler_version: "1",
                        idempotency_key: Some("phase-f-git-commit"),
                        preconditions_hash: git_commit_preconditions_hash(expectation).unwrap(),
                        payload_hash: git_commit_payload_hash(metadata).unwrap(),
                        started_at: "2026-09-04T13:00:02Z",
                    },
                    "phase-f-git",
                )
                .unwrap()
        }

        fn prepare_branch(
            &mut self,
            effect: u128,
            expectation: GitMutationExpectation,
            branch: &str,
        ) -> PreparedToolEffect {
            let resource = git_branch_resource(branch);
            self.kernel
                .prepare_tool_effect(
                    Principal::test("phase-f-git"),
                    PrepareToolEffect {
                        effect_id: EffectId(effect),
                        session_id: SessionId(63),
                        action: "git.branch.create",
                        resource: &resource,
                        execution_semantics: "at_most_once",
                        handler_id: "golam-git-unix",
                        handler_version: "1",
                        idempotency_key: Some("phase-f-git-branch"),
                        preconditions_hash: git_branch_preconditions_hash(expectation, branch)
                            .unwrap(),
                        payload_hash: git_branch_payload_hash(branch),
                        started_at: "2026-09-04T13:00:03Z",
                    },
                    "phase-f-git",
                )
                .unwrap()
        }

        fn complete(&mut self, prepared: &PreparedToolEffect, completion: ToolExecutionCompletion) {
            self.kernel
                .complete_tool_effect(
                    Principal::test("phase-f-git"),
                    CompleteToolEffect {
                        prepared,
                        finished_at: "2026-09-04T13:00:04Z",
                        completion,
                        reason_code: Some("phase_f_git_mutation_qualification"),
                        evidence_ref: None,
                        receipt: None,
                    },
                    "phase-f-git",
                )
                .unwrap();
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.base);
        }
    }

    #[test]
    fn add_commit_and_branch_create_are_effect_bound_and_readback_verified() {
        let mut fixture = Fixture::new();
        fs::write(fixture.repo.join("note.txt"), b"qualified\n").unwrap();
        let target = RequestedTarget::new("note.txt").unwrap();

        let status = fixture.status("git.add");
        let expectation = GitMutationExpectation::from_status(&status).unwrap();
        let file = fixture.file_expectation(&target, "git.add", b"qualified\n");
        let add = fixture.prepare_add(6301, expectation, &target, file);
        let add_receipt = execute_git_add(
            &fixture.resolver,
            &add,
            expectation,
            &target,
            file,
            101,
        )
        .unwrap();
        assert_eq!(add_receipt.action, "git.add");
        fixture.complete(&add, ToolExecutionCompletion::Succeeded);

        let status = fixture.status("git.commit");
        assert_eq!(status.staged.len(), 1);
        let expectation = GitMutationExpectation::from_status(&status).unwrap();
        let metadata = GitCommitMetadata {
            author_name: "Golam Qualification".into(),
            author_email: "golam@example.invalid".into(),
            timestamp_seconds: 1_788_527_000,
            message: "qualify governed Git mutation".into(),
        };
        let commit = fixture.prepare_commit(6302, expectation, &metadata);
        let commit_receipt = execute_git_commit(
            &fixture.resolver,
            &commit,
            expectation,
            &metadata,
            102,
        )
        .unwrap();
        assert_ne!(commit_receipt.current_head, commit_receipt.previous_head);
        fixture.complete(&commit, ToolExecutionCompletion::Succeeded);

        let status = fixture.status("git.branch.create");
        assert!(status.staged.is_empty());
        assert!(status.worktree.is_empty());
        assert!(status.untracked.is_empty());
        let expectation = GitMutationExpectation::from_status(&status).unwrap();
        let branch = fixture.prepare_branch(6303, expectation, "candidate");
        let branch_receipt = execute_git_branch_create(
            &fixture.resolver,
            &branch,
            expectation,
            "candidate",
            103,
        )
        .unwrap();
        assert_eq!(branch_receipt.current_head, commit_receipt.current_head);
        assert_eq!(
            fs::read_to_string(fixture.repo.join(".git/refs/heads/candidate")).unwrap(),
            format!("{}\n", commit_receipt.current_head.to_hex())
        );
        fixture.complete(&branch, ToolExecutionCompletion::Succeeded);
    }

    #[test]
    fn stale_worktree_after_prepared_add_cannot_mutate_index() {
        let mut fixture = Fixture::new();
        fs::write(fixture.repo.join("note.txt"), b"expected\n").unwrap();
        let target = RequestedTarget::new("note.txt").unwrap();
        let status = fixture.status("git.add");
        let expectation = GitMutationExpectation::from_status(&status).unwrap();
        let file = fixture.file_expectation(&target, "git.add", b"expected\n");
        let prepared = fixture.prepare_add(6310, expectation, &target, file);
        let index_before = fs::read(fixture.repo.join(".git/index")).unwrap();

        fs::write(fixture.repo.join("note.txt"), b"swapped\n").unwrap();
        assert!(matches!(
            execute_git_add(
                &fixture.resolver,
                &prepared,
                expectation,
                &target,
                file,
                110,
            ),
            Err(GitMutationError::StaleRepository | GitMutationError::StaleWorktree)
        ));
        assert_eq!(
            fs::read(fixture.repo.join(".git/index")).unwrap(),
            index_before
        );
        fixture.complete(&prepared, ToolExecutionCompletion::Failed);
    }

    #[test]
    fn corrupt_existing_loose_object_is_never_accepted_as_git_add_success() {
        let mut fixture = Fixture::new();
        let bytes = b"collision\n";
        fs::write(fixture.repo.join("note.txt"), bytes).unwrap();
        let target = RequestedTarget::new("note.txt").unwrap();
        let status = fixture.status("git.add");
        let expectation = GitMutationExpectation::from_status(&status).unwrap();
        let file = fixture.file_expectation(&target, "git.add", bytes);
        let prepared = fixture.prepare_add(6320, expectation, &target, file);
        let id = GitObjectId::parse(&hex20(git_object_digest("blob", bytes).unwrap())).unwrap();
        let hex = id.to_hex();
        let dir = fixture.repo.join(".git/objects").join(&hex[..2]);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(&hex[2..]), b"not-zlib-and-not-the-object").unwrap();
        let index_before = fs::read(fixture.repo.join(".git/index")).unwrap();

        assert!(matches!(
            execute_git_add(
                &fixture.resolver,
                &prepared,
                expectation,
                &target,
                file,
                120,
            ),
            Err(GitMutationError::ObjectCollision(_) | GitMutationError::GitRead(_))
        ));
        assert_eq!(
            fs::read(fixture.repo.join(".git/index")).unwrap(),
            index_before
        );
        fixture.complete(&prepared, ToolExecutionCompletion::Failed);
    }

    #[test]
    fn existing_branch_is_never_moved_and_destructive_branch_names_are_rejected() {
        let mut fixture = Fixture::new();
        let status = fixture.status("git.branch.create");
        let expectation = GitMutationExpectation::from_status(&status).unwrap();
        let existing = fs::read(fixture.repo.join(".git/refs/heads/main")).unwrap();
        let prepared = fixture.prepare_branch(6330, expectation, "main");
        assert!(matches!(
            execute_git_branch_create(
                &fixture.resolver,
                &prepared,
                expectation,
                "main",
                130,
            ),
            Err(GitMutationError::BranchExists)
        ));
        assert_eq!(
            fs::read(fixture.repo.join(".git/refs/heads/main")).unwrap(),
            existing
        );
        fixture.complete(&prepared, ToolExecutionCompletion::Failed);

        for denied in [
            "",
            ".hidden",
            "-force",
            "a/b",
            "a..b",
            "main.lock",
            "refs-heads-main",
            "bad name",
        ] {
            assert!(matches!(
                validate_branch_name(denied),
                Err(GitMutationError::InvalidBranchName)
            ));
        }
    }

    #[test]
    fn commit_metadata_rejects_ambiguous_or_noncanonical_identity_and_message() {
        for invalid in [
            GitCommitMetadata {
                author_name: "owner\nforged".into(),
                author_email: "local@example.invalid".into(),
                timestamp_seconds: 1,
                message: "message".into(),
            },
            GitCommitMetadata {
                author_name: "owner".into(),
                author_email: "not-an-email".into(),
                timestamp_seconds: 1,
                message: "message".into(),
            },
            GitCommitMetadata {
                author_name: "owner".into(),
                author_email: "local@example.invalid".into(),
                timestamp_seconds: -1,
                message: "message".into(),
            },
            GitCommitMetadata {
                author_name: "owner".into(),
                author_email: "local@example.invalid".into(),
                timestamp_seconds: 1,
                message: "message\n".into(),
            },
        ] {
            assert!(matches!(
                invalid.validate(),
                Err(GitMutationError::InvalidCommitMetadata)
            ));
        }
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

        let tree = write_fixture_object(root, "tree", b"");
        let identity = "Fixture <fixture@example.invalid> 1 +0000";
        let commit = format!(
            "tree {}\nauthor {identity}\ncommitter {identity}\n\ninitial\n",
            tree.to_hex()
        );
        let commit_id = write_fixture_object(root, "commit", commit.as_bytes());
        fs::write(
            root.join(".git/refs/heads/main"),
            format!("{}\n", commit_id.to_hex()),
        )
        .unwrap();

        let index = GitIndex {
            version: GitIndexVersion::V2,
            entries: vec![],
            extensions: vec![],
            checksum: [0; 20],
        };
        fs::write(root.join(".git/index"), serialize_index_v2(&index).unwrap()).unwrap();
    }

    fn write_fixture_object(root: &Path, kind: &str, body: &[u8]) -> GitObjectId {
        let digest = git_object_digest(kind, body).unwrap();
        let id = GitObjectId::parse(&hex20(digest)).unwrap();
        let hex = id.to_hex();
        let directory = root.join(".git/objects").join(&hex[..2]);
        fs::create_dir_all(&directory).unwrap();
        let mut canonical = format!("{kind} {}\0", body.len()).into_bytes();
        canonical.extend_from_slice(body);
        fs::write(
            directory.join(&hex[2..]),
            compress_to_vec_zlib(&canonical, 6),
        )
        .unwrap();
        id
    }

    fn unique_root() -> PathBuf {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "golam-phase-f-git-{}-{nanos}-{counter}",
            std::process::id()
        ))
    }
}
