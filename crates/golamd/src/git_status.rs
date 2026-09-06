#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;
use std::io;
use std::time::Duration;

use golam_core::digest::sha256;
use golam_core::target_identity::ObservedFileKind;
use golam_core::tool_request::{BindingDigest, RequestedOperationId, RequestedTarget};
use golam_core::{CanonicalEncoder, CoreError};

use crate::git_index::{
    GitIndexBounds, GitIndexEntry, GitIndexError, GitIndexMode, parse_git_index,
};
use crate::git_observe::{
    GitObservationBounds, GitObservationError, GitObservationReader, GitTreeMode,
};
use crate::git_read::{GitObjectId, GitReadError, GitRepositoryEvidence};
use crate::git_read_budget::{GitOperationBudgetError, GitOperationDeadline};
use crate::git_sha1::{GitObjectSha1, GitObjectSha1Error};
use crate::local_dir::{
    LocalDirectorySnapshotBounds, LocalDirectorySnapshotError, snapshot_directory,
};
use crate::local_fs::{LocalFsResolutionError, LocalFsResolver};
use crate::local_read::{LocalFileReadBounds, LocalFileReadError, read_regular_file};

pub const MAX_STATUS_ENTRIES: usize = 250_000;
pub const MAX_WORKTREE_FILE_BYTES: u64 = 32 * 1024 * 1024;
pub const MAX_TOTAL_WORKTREE_BYTES: u64 = 128 * 1024 * 1024;
pub const MAX_WORKTREE_DEPTH: usize = 64;
pub const DEFAULT_STATUS_TIME_BUDGET: Duration = Duration::from_secs(10);
pub const MAX_STATUS_TIME_BUDGET: Duration = Duration::from_secs(60);
const STATUS_OBSERVATION_DOMAIN: &[u8] = b"golam:git-status-observation:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GitStatusBounds {
    pub observation: GitObservationBounds,
    pub index: GitIndexBounds,
    pub max_entries: usize,
    pub max_worktree_file_bytes: u64,
    pub max_total_worktree_bytes: u64,
    pub max_worktree_depth: usize,
    pub max_duration: Duration,
}

impl Default for GitStatusBounds {
    fn default() -> Self {
        Self {
            observation: GitObservationBounds::default(),
            index: GitIndexBounds::default(),
            max_entries: MAX_STATUS_ENTRIES,
            max_worktree_file_bytes: MAX_WORKTREE_FILE_BYTES,
            max_total_worktree_bytes: MAX_TOTAL_WORKTREE_BYTES,
            max_worktree_depth: MAX_WORKTREE_DEPTH,
            max_duration: DEFAULT_STATUS_TIME_BUDGET,
        }
    }
}

impl GitStatusBounds {
    pub fn validate(self) -> Result<(), GitStatusError> {
        self.observation.validate()?;
        self.index.validate()?;
        if self.max_entries == 0
            || self.max_entries > MAX_STATUS_ENTRIES
            || self.max_worktree_file_bytes == 0
            || self.max_worktree_file_bytes > MAX_WORKTREE_FILE_BYTES
            || self.max_total_worktree_bytes == 0
            || self.max_total_worktree_bytes > MAX_TOTAL_WORKTREE_BYTES
            || self.max_worktree_depth == 0
            || self.max_worktree_depth > MAX_WORKTREE_DEPTH
            || self.max_duration.is_zero()
            || self.max_duration > MAX_STATUS_TIME_BUDGET
            || self.max_worktree_file_bytes > self.max_total_worktree_bytes
            || self.max_duration > self.observation.git.max_duration
        {
            return Err(GitStatusError::InvalidBounds);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitChangeKind {
    Added,
    Modified,
    Deleted,
    TypeChanged,
    Conflicted,
    IntentToAdd,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitDiffEvidence {
    pub path: String,
    pub kind: GitChangeKind,
    pub before: Option<GitObjectId>,
    pub after: Option<GitObjectId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitStatusObservation {
    pub(crate) repository_evidence: GitRepositoryEvidence,
    pub(crate) head: GitObjectId,
    pub(crate) index_checksum: [u8; 20],
    pub(crate) staged: Vec<GitDiffEvidence>,
    pub(crate) worktree: Vec<GitDiffEvidence>,
    pub(crate) untracked: Vec<String>,
    pub(crate) observed_at_unix_ms: u64,
    binding_digest: BindingDigest,
}

impl GitStatusObservation {
    #[allow(clippy::too_many_arguments)]
    fn new(
        repository_evidence: GitRepositoryEvidence,
        head: GitObjectId,
        index_checksum: [u8; 20],
        staged: Vec<GitDiffEvidence>,
        worktree: Vec<GitDiffEvidence>,
        untracked: Vec<String>,
        observed_at_unix_ms: u64,
    ) -> Result<Self, GitStatusError> {
        let binding_digest = status_observation_binding(
            &repository_evidence,
            head,
            &index_checksum,
            &staged,
            &worktree,
            &untracked,
            observed_at_unix_ms,
        )?;
        Ok(Self {
            repository_evidence,
            head,
            index_checksum,
            staged,
            worktree,
            untracked,
            observed_at_unix_ms,
            binding_digest,
        })
    }

    pub fn repository_evidence(&self) -> &GitRepositoryEvidence {
        &self.repository_evidence
    }

    pub const fn head(&self) -> GitObjectId {
        self.head
    }

    pub const fn index_checksum(&self) -> [u8; 20] {
        self.index_checksum
    }

    pub fn staged(&self) -> &[GitDiffEvidence] {
        &self.staged
    }

    pub fn worktree(&self) -> &[GitDiffEvidence] {
        &self.worktree
    }

    pub fn untracked(&self) -> &[String] {
        &self.untracked
    }

    pub const fn observed_at_unix_ms(&self) -> u64 {
        self.observed_at_unix_ms
    }

    pub const fn binding_digest(&self) -> BindingDigest {
        self.binding_digest
    }

    pub fn verify_binding(&self) -> Result<(), GitStatusError> {
        let expected = status_observation_binding(
            &self.repository_evidence,
            self.head,
            &self.index_checksum,
            &self.staged,
            &self.worktree,
            &self.untracked,
            self.observed_at_unix_ms,
        )?;
        if expected != self.binding_digest {
            return Err(GitStatusError::ObservationBindingMismatch);
        }
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn status_observation_binding(
    repository_evidence: &GitRepositoryEvidence,
    head: GitObjectId,
    index_checksum: &[u8; 20],
    staged: &[GitDiffEvidence],
    worktree: &[GitDiffEvidence],
    untracked: &[String],
    observed_at_unix_ms: u64,
) -> Result<BindingDigest, GitStatusError> {
    repository_evidence.verify_binding()?;
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(STATUS_OBSERVATION_DOMAIN)?;
    encoder.push_bytes(&repository_evidence.binding_digest().bytes())?;
    push_object_id(&mut encoder, head)?;
    encoder.push_bytes(index_checksum)?;
    push_diff_evidence(&mut encoder, staged)?;
    push_diff_evidence(&mut encoder, worktree)?;
    encoder.push_u64(status_len(untracked.len())?);
    for path in untracked {
        encoder.push_bytes(path.as_bytes())?;
    }
    encoder.push_u64(observed_at_unix_ms);
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn push_diff_evidence(
    encoder: &mut CanonicalEncoder,
    evidence: &[GitDiffEvidence],
) -> Result<(), GitStatusError> {
    encoder.push_u64(status_len(evidence.len())?);
    for entry in evidence {
        encoder.push_bytes(entry.path.as_bytes())?;
        encoder.push_u8(change_kind_tag(entry.kind));
        push_optional_object_id(encoder, entry.before)?;
        push_optional_object_id(encoder, entry.after)?;
    }
    Ok(())
}

fn push_optional_object_id(
    encoder: &mut CanonicalEncoder,
    object_id: Option<GitObjectId>,
) -> Result<(), GitStatusError> {
    match object_id {
        None => encoder.push_u8(0),
        Some(object_id) => {
            encoder.push_u8(1);
            push_object_id(encoder, object_id)?;
        }
    }
    Ok(())
}

fn push_object_id(
    encoder: &mut CanonicalEncoder,
    object_id: GitObjectId,
) -> Result<(), GitStatusError> {
    encoder.push_bytes(&object_id.bytes())?;
    Ok(())
}

fn status_len(value: usize) -> Result<u64, GitStatusError> {
    u64::try_from(value).map_err(|_| GitStatusError::SizeOverflow)
}

const fn change_kind_tag(kind: GitChangeKind) -> u8 {
    match kind {
        GitChangeKind::Added => 0,
        GitChangeKind::Modified => 1,
        GitChangeKind::Deleted => 2,
        GitChangeKind::TypeChanged => 3,
        GitChangeKind::Conflicted => 4,
        GitChangeKind::IntentToAdd => 5,
    }
}

pub fn observe_status(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    bounds: GitStatusBounds,
    observed_at_unix_ms: u64,
) -> Result<GitStatusObservation, GitStatusError> {
    bounds.validate()?;
    let deadline = GitOperationDeadline::start(bounds.max_duration)?;
    let observation = GitObservationReader::open_with_deadline(
        resolver,
        operation,
        bounds.observation,
        deadline,
        observed_at_unix_ms,
    )?;
    deadline.require_active()?;

    let index_bytes = read_file(
        resolver,
        operation,
        ".git/index",
        u64::try_from(bounds.index.max_bytes).map_err(|_| GitStatusError::SizeOverflow)?,
        deadline,
        observed_at_unix_ms,
    )?;
    let index = deadline.run_step(|| parse_git_index(&index_bytes, bounds.index))??;
    let head_tree = observation.observe_head_tree()?;
    if head_tree.truncated {
        return Err(GitStatusError::IncompleteHeadTree);
    }

    let mut head = BTreeMap::new();
    for entry in head_tree.entries {
        let previous = head.insert(entry.path, (entry.mode, entry.object_id));
        if previous.is_some() {
            return Err(GitStatusError::DuplicatePath);
        }
    }

    let grouped =
        deadline.run_step(|| group_index_entries(&index.entries, bounds.max_entries))??;
    let stage_zero = deadline.run_step(|| stage_zero_entries(&grouped))?;
    let staged = deadline.run_step(|| staged_diff(&head, &grouped, bounds.max_entries))??;
    deadline.require_active()?;
    let (worktree, untracked) = worktree_diff(
        resolver,
        operation,
        &stage_zero,
        bounds,
        observed_at_unix_ms,
        deadline,
    )?;

    GitStatusObservation::new(
        observation.evidence().clone(),
        observation.evidence().head.object_id,
        index.checksum,
        staged,
        worktree,
        untracked,
        observed_at_unix_ms,
    )
}

fn group_index_entries(
    entries: &[GitIndexEntry],
    max_entries: usize,
) -> Result<BTreeMap<String, Vec<&GitIndexEntry>>, GitStatusError> {
    if entries.len() > max_entries {
        return Err(GitStatusError::EntryLimitExceeded);
    }
    let mut grouped = BTreeMap::<String, Vec<&GitIndexEntry>>::new();
    for entry in entries {
        let path = std::str::from_utf8(&entry.path)
            .map_err(|_| GitStatusError::NonUnicodeIndexPath)?
            .to_owned();
        grouped.entry(path).or_default().push(entry);
    }
    Ok(grouped)
}

fn stage_zero_entries<'a>(
    grouped: &'a BTreeMap<String, Vec<&'a GitIndexEntry>>,
) -> BTreeMap<String, &'a GitIndexEntry> {
    grouped
        .iter()
        .filter_map(|(path, entries)| {
            if entries.len() == 1 && entries[0].stage == 0 {
                Some((path.clone(), entries[0]))
            } else {
                None
            }
        })
        .collect()
}

fn staged_diff(
    head: &BTreeMap<String, (GitTreeMode, GitObjectId)>,
    index: &BTreeMap<String, Vec<&GitIndexEntry>>,
    max_entries: usize,
) -> Result<Vec<GitDiffEvidence>, GitStatusError> {
    let mut paths = BTreeSet::new();
    paths.extend(head.keys().cloned());
    paths.extend(index.keys().cloned());
    if paths.len() > max_entries {
        return Err(GitStatusError::EntryLimitExceeded);
    }

    let mut output = Vec::new();
    for path in paths {
        let head_entry = head.get(&path);
        let index_entries = index.get(&path);
        if index_entries.is_some_and(|entries| entries.len() != 1 || entries[0].stage != 0) {
            output.push(GitDiffEvidence {
                path,
                kind: GitChangeKind::Conflicted,
                before: head_entry.map(|(_, id)| *id),
                after: None,
            });
            continue;
        }
        let index_entry = index_entries.and_then(|entries| entries.first().copied());
        match (head_entry, index_entry) {
            (None, Some(entry)) => output.push(GitDiffEvidence {
                path,
                kind: if entry.intent_to_add {
                    GitChangeKind::IntentToAdd
                } else {
                    GitChangeKind::Added
                },
                before: None,
                after: Some(index_object_id(entry)?),
            }),
            (Some((_, before)), None) => output.push(GitDiffEvidence {
                path,
                kind: GitChangeKind::Deleted,
                before: Some(*before),
                after: None,
            }),
            (Some((head_mode, before)), Some(entry)) => {
                let after = index_object_id(entry)?;
                let type_changed = !modes_equivalent(*head_mode, entry.mode);
                if type_changed || *before != after || entry.intent_to_add {
                    output.push(GitDiffEvidence {
                        path,
                        kind: if entry.intent_to_add {
                            GitChangeKind::IntentToAdd
                        } else if type_changed {
                            GitChangeKind::TypeChanged
                        } else {
                            GitChangeKind::Modified
                        },
                        before: Some(*before),
                        after: Some(after),
                    });
                }
            }
            (None, None) => {}
        }
    }
    Ok(output)
}

fn worktree_diff(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    index: &BTreeMap<String, &GitIndexEntry>,
    bounds: GitStatusBounds,
    observed_at_unix_ms: u64,
    deadline: GitOperationDeadline,
) -> Result<(Vec<GitDiffEvidence>, Vec<String>), GitStatusError> {
    let mut output = Vec::new();
    let mut total_bytes = 0_u64;

    for (path, entry) in index {
        deadline.require_active()?;
        if entry.skip_worktree || entry.intent_to_add {
            continue;
        }
        match entry.mode {
            GitIndexMode::RegularFile { .. } => {}
            GitIndexMode::SymbolicLink | GitIndexMode::Gitlink => {
                return Err(GitStatusError::UnsupportedWorktreeMode(path.clone()));
            }
        }
        let requested = RequestedTarget::new(path)
            .map_err(|_| GitStatusError::InvalidWorktreePath(path.clone()))?;
        let identity = deadline.run_step(|| {
            resolver.resolve_read_target(&requested, operation, observed_at_unix_ms)
        })??;
        match identity.file_kind {
            ObservedFileKind::Missing => {
                output.push(GitDiffEvidence {
                    path: path.clone(),
                    kind: GitChangeKind::Deleted,
                    before: Some(index_object_id(entry)?),
                    after: None,
                });
            }
            ObservedFileKind::RegularFile => {
                let read = read_regular_file(
                    resolver,
                    &requested,
                    operation,
                    LocalFileReadBounds {
                        max_bytes: bounds.max_worktree_file_bytes,
                        max_duration: deadline.remaining()?,
                    },
                    observed_at_unix_ms,
                    observed_at_unix_ms,
                )?;
                total_bytes = total_bytes
                    .checked_add(
                        u64::try_from(read.bytes.len())
                            .map_err(|_| GitStatusError::SizeOverflow)?,
                    )
                    .ok_or(GitStatusError::WorktreeByteLimitExceeded)?;
                if total_bytes > bounds.max_total_worktree_bytes {
                    return Err(GitStatusError::WorktreeByteLimitExceeded);
                }
                let worktree_id = deadline.run_step(|| blob_id(&read.bytes))??;
                let index_id = index_object_id(entry)?;
                if worktree_id != index_id {
                    output.push(GitDiffEvidence {
                        path: path.clone(),
                        kind: GitChangeKind::Modified,
                        before: Some(index_id),
                        after: Some(worktree_id),
                    });
                }
            }
            _ => {
                output.push(GitDiffEvidence {
                    path: path.clone(),
                    kind: GitChangeKind::TypeChanged,
                    before: Some(index_object_id(entry)?),
                    after: None,
                });
            }
        }
        if output.len() > bounds.max_entries {
            return Err(GitStatusError::EntryLimitExceeded);
        }
    }

    let untracked = discover_untracked(
        resolver,
        operation,
        index.keys().map(String::as_str).collect(),
        bounds,
        observed_at_unix_ms,
        deadline,
    )?;
    Ok((output, untracked))
}

fn discover_untracked(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    tracked: BTreeSet<&str>,
    bounds: GitStatusBounds,
    observed_at_unix_ms: u64,
    deadline: GitOperationDeadline,
) -> Result<Vec<String>, GitStatusError> {
    let root = RequestedTarget::new(".").map_err(|_| GitStatusError::InvalidInternalPath)?;
    let root_identity = deadline
        .run_step(|| resolver.resolve_read_target(&root, operation, observed_at_unix_ms))??;
    if root_identity.file_kind != ObservedFileKind::Directory {
        return Err(GitStatusError::RepositoryRootChanged);
    }
    let mut pending = VecDeque::new();
    pending.push_back((String::new(), root_identity, 0_usize));
    let mut output = Vec::new();
    let mut observed_entries = 0_usize;

    while let Some((prefix, expected, depth)) = pending.pop_front() {
        deadline.require_active()?;
        if depth > bounds.max_worktree_depth {
            return Err(GitStatusError::WorktreeDepthExceeded);
        }
        let requested_dir = if prefix.is_empty() {
            RequestedTarget::new(".").map_err(|_| GitStatusError::InvalidInternalPath)?
        } else {
            RequestedTarget::new(&prefix)
                .map_err(|_| GitStatusError::InvalidWorktreePath(prefix.clone()))?
        };
        let remaining_entries = bounds
            .max_entries
            .checked_sub(observed_entries)
            .filter(|remaining| *remaining > 0)
            .ok_or(GitStatusError::EntryLimitExceeded)?;
        let snapshot = snapshot_directory(
            resolver,
            &requested_dir,
            operation,
            LocalDirectorySnapshotBounds {
                max_entries: remaining_entries,
                max_duration: deadline.remaining()?,
            },
            observed_at_unix_ms,
        )?;
        if snapshot.identity.file_kind != ObservedFileKind::Directory
            || snapshot.identity.resolved_target_identity != expected.resolved_target_identity
            || snapshot.identity.observed_metadata_digest != expected.observed_metadata_digest
        {
            return Err(GitStatusError::RepositoryRootChanged);
        }
        observed_entries = observed_entries
            .checked_add(snapshot.names.len())
            .ok_or(GitStatusError::EntryLimitExceeded)?;
        if observed_entries > bounds.max_entries {
            return Err(GitStatusError::EntryLimitExceeded);
        }

        for name in snapshot.names {
            deadline.require_active()?;
            if name == ".git" {
                continue;
            }
            let path = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            let requested = RequestedTarget::new(&path)
                .map_err(|_| GitStatusError::InvalidWorktreePath(path.clone()))?;
            let identity = deadline.run_step(|| {
                resolver.resolve_read_target(&requested, operation, observed_at_unix_ms)
            })??;
            match identity.file_kind {
                ObservedFileKind::Directory => {
                    pending.push_back((path, identity, depth + 1));
                }
                ObservedFileKind::RegularFile => {
                    if !tracked.contains(path.as_str()) {
                        output.push(path);
                        if output.len() > bounds.max_entries {
                            return Err(GitStatusError::EntryLimitExceeded);
                        }
                    }
                }
                ObservedFileKind::Missing => {}
                _ => return Err(GitStatusError::UnsupportedWorktreeKind(path)),
            }
        }
    }
    Ok(output)
}

fn read_file(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    path: &str,
    max_bytes: u64,
    deadline: GitOperationDeadline,
    observed_at_unix_ms: u64,
) -> Result<Vec<u8>, GitStatusError> {
    deadline.require_active()?;
    let requested = RequestedTarget::new(path).map_err(|_| GitStatusError::InvalidInternalPath)?;
    let identity = resolver.resolve_read_target(&requested, operation, observed_at_unix_ms)?;
    if identity.file_kind != ObservedFileKind::RegularFile {
        return Err(GitStatusError::MissingIndex);
    }
    let read = read_regular_file(
        resolver,
        &requested,
        operation,
        LocalFileReadBounds {
            max_bytes,
            max_duration: deadline.remaining()?,
        },
        observed_at_unix_ms,
        observed_at_unix_ms,
    )?;
    Ok(read.bytes)
}

fn index_object_id(entry: &GitIndexEntry) -> Result<GitObjectId, GitStatusError> {
    object_id_from_digest(entry.object_id)
}

fn blob_id(bytes: &[u8]) -> Result<GitObjectId, GitStatusError> {
    let header = format!("blob {}\0", bytes.len());
    let mut sha1 = GitObjectSha1::new();
    sha1.update(header.as_bytes())?;
    sha1.update(bytes)?;
    object_id_from_digest(sha1.finalize()?)
}

fn object_id_from_digest(digest: [u8; 20]) -> Result<GitObjectId, GitStatusError> {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut hex = String::with_capacity(40);
    for byte in digest {
        hex.push(char::from(DIGITS[(byte >> 4) as usize]));
        hex.push(char::from(DIGITS[(byte & 0x0f) as usize]));
    }
    GitObjectId::parse(&hex).map_err(GitStatusError::Git)
}

const fn modes_equivalent(tree: GitTreeMode, index: GitIndexMode) -> bool {
    matches!(
        (tree, index),
        (
            GitTreeMode::RegularFile { executable: false },
            GitIndexMode::RegularFile { executable: false }
        ) | (
            GitTreeMode::RegularFile { executable: true },
            GitIndexMode::RegularFile { executable: true }
        ) | (GitTreeMode::SymbolicLink, GitIndexMode::SymbolicLink)
            | (GitTreeMode::Gitlink, GitIndexMode::Gitlink)
    )
}

#[derive(Debug)]
pub enum GitStatusError {
    InvalidBounds,
    InvalidInternalPath,
    InvalidWorktreePath(String),
    Core(CoreError),
    Git(GitReadError),
    Observation(GitObservationError),
    Index(GitIndexError),
    Resolution(LocalFsResolutionError),
    LocalRead(LocalFileReadError),
    DirectorySnapshot(LocalDirectorySnapshotError),
    OperationBudget(GitOperationBudgetError),
    Sha1(GitObjectSha1Error),
    Io(io::Error),
    MissingIndex,
    IncompleteHeadTree,
    DuplicatePath,
    NonUnicodeIndexPath,
    NonUnicodeWorktreePath,
    UnsupportedWorktreeMode(String),
    UnsupportedWorktreeKind(String),
    EntryLimitExceeded,
    WorktreeByteLimitExceeded,
    WorktreeDepthExceeded,
    DurationLimitExceeded,
    RepositoryRootChanged,
    SizeOverflow,
    ObservationBindingMismatch,
}

impl fmt::Display for GitStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBounds => f.write_str("Git status bounds exceed the first-profile limits"),
            Self::InvalidInternalPath => {
                f.write_str("Git status constructed an invalid internal path")
            }
            Self::InvalidWorktreePath(path) => write!(f, "Git worktree path is invalid: {path}"),
            Self::Core(error) => write!(f, "Git status canonical binding failed: {error}"),
            Self::Git(error) => write!(f, "Git status repository read failed: {error}"),
            Self::Observation(error) => write!(f, "Git status object observation failed: {error}"),
            Self::Index(error) => write!(f, "Git status index parse failed: {error}"),
            Self::Resolution(error) => write!(f, "Git status target resolution failed: {error}"),
            Self::LocalRead(error) => write!(f, "Git status bounded file read failed: {error}"),
            Self::DirectorySnapshot(error) => {
                write!(f, "Git status directory snapshot failed: {error}")
            }
            Self::OperationBudget(error) => {
                write!(f, "Git status operation budget failed: {error}")
            }
            Self::Sha1(error) => write!(f, "Git status SHA-1 failed: {error}"),
            Self::Io(error) => write!(f, "Git status filesystem I/O failed: {error}"),
            Self::MissingIndex => f.write_str("Git index is missing or not a regular file"),
            Self::IncompleteHeadTree => f.write_str(
                "Git HEAD tree observation was truncated; status refuses incomplete evidence",
            ),
            Self::DuplicatePath => f.write_str("Git status input contains a duplicate path"),
            Self::NonUnicodeIndexPath => {
                f.write_str("Git index path is non-Unicode and outside the first request profile")
            }
            Self::NonUnicodeWorktreePath => f.write_str(
                "Git worktree contains a non-Unicode path outside the first request profile",
            ),
            Self::UnsupportedWorktreeMode(path) => write!(
                f,
                "Git worktree verification for symlink/gitlink is not admitted: {path}"
            ),
            Self::UnsupportedWorktreeKind(path) => {
                write!(f, "Git worktree contains unsupported file kind: {path}")
            }
            Self::EntryLimitExceeded => f.write_str("Git status entry limit exceeded"),
            Self::WorktreeByteLimitExceeded => {
                f.write_str("Git status aggregate worktree byte limit exceeded")
            }
            Self::WorktreeDepthExceeded => f.write_str("Git status worktree depth limit exceeded"),
            Self::DurationLimitExceeded => f.write_str("Git status duration limit exceeded"),
            Self::RepositoryRootChanged => {
                f.write_str("Git worktree directory identity changed during status observation")
            }
            Self::SizeOverflow => {
                f.write_str("Git status size cannot be represented by the bounded profile")
            }
            Self::ObservationBindingMismatch => {
                f.write_str("Git status payload is detached from repository evidence")
            }
        }
    }
}

impl Error for GitStatusError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Core(error) => Some(error),
            Self::Git(error) => Some(error),
            Self::Observation(error) => Some(error),
            Self::Index(error) => Some(error),
            Self::Resolution(error) => Some(error),
            Self::LocalRead(error) => Some(error),
            Self::DirectorySnapshot(error) => Some(error),
            Self::OperationBudget(error) => Some(error),
            Self::Sha1(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CoreError> for GitStatusError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<GitReadError> for GitStatusError {
    fn from(value: GitReadError) -> Self {
        Self::Git(value)
    }
}

impl From<GitObservationError> for GitStatusError {
    fn from(value: GitObservationError) -> Self {
        Self::Observation(value)
    }
}

impl From<GitIndexError> for GitStatusError {
    fn from(value: GitIndexError) -> Self {
        Self::Index(value)
    }
}

impl From<LocalFsResolutionError> for GitStatusError {
    fn from(value: LocalFsResolutionError) -> Self {
        Self::Resolution(value)
    }
}

impl From<LocalFileReadError> for GitStatusError {
    fn from(value: LocalFileReadError) -> Self {
        Self::LocalRead(value)
    }
}

impl From<LocalDirectorySnapshotError> for GitStatusError {
    fn from(value: LocalDirectorySnapshotError) -> Self {
        Self::DirectorySnapshot(value)
    }
}

impl From<GitOperationBudgetError> for GitStatusError {
    fn from(value: GitOperationBudgetError) -> Self {
        Self::OperationBudget(value)
    }
}

impl From<GitObjectSha1Error> for GitStatusError {
    fn from(value: GitObjectSha1Error) -> Self {
        Self::Sha1(value)
    }
}

impl From<io::Error> for GitStatusError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use golam_core::tool_request::ResourceClassId;

    use crate::git_read::{GitReadBounds, GitRepositoryReader};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_root() -> PathBuf {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "golam-git-status-binding-{}-{nanos}-{counter}",
            std::process::id()
        ))
    }

    fn resolver(root: &Path) -> LocalFsResolver {
        LocalFsResolver::new(
            root,
            ResourceClassId::new("workspace.read").unwrap(),
            vec![RequestedOperationId::new("read").unwrap()],
            [],
        )
        .unwrap()
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd"
    ))]
    fn fixture_repository_evidence() -> (PathBuf, GitRepositoryEvidence) {
        let root = unique_root();
        fs::create_dir_all(root.join(".git/objects")).unwrap();
        fs::write(
            root.join(".git/config"),
            b"[extensions]\n\tobjectFormat = sha1\n",
        )
        .unwrap();
        fs::write(
            root.join(".git/HEAD"),
            b"1111111111111111111111111111111111111111\n",
        )
        .unwrap();
        let resolver = resolver(&root);
        let reader = GitRepositoryReader::open(
            &resolver,
            &RequestedOperationId::new("read").unwrap(),
            GitReadBounds::default(),
            10,
        )
        .unwrap();
        (root, reader.evidence().clone())
    }

    #[test]
    fn staged_diff_reports_identity_changes_without_patch_text() {
        let old = GitObjectId::parse("1111111111111111111111111111111111111111").unwrap();
        let new = GitObjectId::parse("2222222222222222222222222222222222222222").unwrap();
        let mut head = BTreeMap::new();
        head.insert(
            "a.txt".to_owned(),
            (GitTreeMode::RegularFile { executable: false }, old),
        );
        let entry = GitIndexEntry {
            ctime_seconds: 0,
            ctime_nanoseconds: 0,
            mtime_seconds: 0,
            mtime_nanoseconds: 0,
            dev: 0,
            ino: 0,
            mode: GitIndexMode::RegularFile { executable: false },
            uid: 0,
            gid: 0,
            file_size: 1,
            object_id: new.bytes(),
            assume_valid: false,
            stage: 0,
            skip_worktree: false,
            intent_to_add: false,
            path: b"a.txt".to_vec(),
        };
        let mut index = BTreeMap::new();
        index.insert("a.txt".to_owned(), vec![&entry]);
        let diff = staged_diff(&head, &index, 10).unwrap();
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].kind, GitChangeKind::Modified);
        assert_eq!(diff[0].before, Some(old));
        assert_eq!(diff[0].after, Some(new));
    }

    #[test]
    fn conflict_stage_never_collapses_into_normal_change() {
        let id = GitObjectId::parse("1111111111111111111111111111111111111111").unwrap();
        let base = GitIndexEntry {
            ctime_seconds: 0,
            ctime_nanoseconds: 0,
            mtime_seconds: 0,
            mtime_nanoseconds: 0,
            dev: 0,
            ino: 0,
            mode: GitIndexMode::RegularFile { executable: false },
            uid: 0,
            gid: 0,
            file_size: 1,
            object_id: id.bytes(),
            assume_valid: false,
            stage: 1,
            skip_worktree: false,
            intent_to_add: false,
            path: b"conflict.txt".to_vec(),
        };
        let ours = GitIndexEntry {
            stage: 2,
            ..base.clone()
        };
        let mut index = BTreeMap::new();
        index.insert("conflict.txt".to_owned(), vec![&base, &ours]);
        let diff = staged_diff(&BTreeMap::new(), &index, 10).unwrap();
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].kind, GitChangeKind::Conflicted);
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd"
    ))]
    #[test]
    fn status_binding_rejects_repository_substitution_and_payload_mutation() {
        let (root_a, evidence_a) = fixture_repository_evidence();
        let (root_b, evidence_b) = fixture_repository_evidence();
        let head = evidence_a.head.object_id;
        let tracked = GitObjectId::parse("2222222222222222222222222222222222222222").unwrap();
        let status = GitStatusObservation::new(
            evidence_a,
            head,
            [7_u8; 20],
            vec![GitDiffEvidence {
                path: "tracked.txt".to_owned(),
                kind: GitChangeKind::Modified,
                before: Some(tracked),
                after: Some(head),
            }],
            Vec::new(),
            vec!["untracked.txt".to_owned()],
            10,
        )
        .unwrap();
        assert!(status.verify_binding().is_ok());

        let mut substituted = status.clone();
        substituted.repository_evidence = evidence_b;
        assert!(matches!(
            substituted.verify_binding(),
            Err(GitStatusError::ObservationBindingMismatch)
        ));

        let mut mutated = status.clone();
        mutated.untracked.push("injected.txt".to_owned());
        assert!(matches!(
            mutated.verify_binding(),
            Err(GitStatusError::ObservationBindingMismatch)
        ));

        let mut staged_mutation = status;
        staged_mutation.staged[0].path = "other.txt".to_owned();
        assert!(matches!(
            staged_mutation.verify_binding(),
            Err(GitStatusError::ObservationBindingMismatch)
        ));

        fs::remove_dir_all(root_a).unwrap();
        fs::remove_dir_all(root_b).unwrap();
    }
}
