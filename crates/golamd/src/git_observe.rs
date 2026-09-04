#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashSet, VecDeque};
use std::error::Error;
use std::fmt;
#[cfg(test)]
use std::fs;
use std::io;
#[cfg(test)]
use std::path::Path;

use golam_core::digest::sha256;
use golam_core::target_identity::ObservedFileKind;
use golam_core::tool_request::{BindingDigest, RequestedOperationId, RequestedTarget};
use golam_core::{CanonicalEncoder, CoreError};

use crate::git_pack::{
    GitPackBounds, GitPackError, GitPackIndex, PackObjectId, PackedObjectKind, ValidatedPack,
    parse_pack_index_v2, read_validated_packed_object_with_deadline,
    validate_pack_for_reuse_with_deadline,
};
use crate::git_read::{
    GitObject, GitObjectId, GitObjectKind, GitReadBounds, GitReadError, GitRepositoryEvidence,
    GitRepositoryReader,
};
use crate::git_read_budget::{GitOperationBudgetError, GitOperationDeadline};
use crate::local_dir::{
    LocalDirectorySnapshotBounds, LocalDirectorySnapshotError, snapshot_directory,
};
use crate::local_fs::{LocalFsResolutionError, LocalFsResolver};
use crate::local_read::{LocalFileReadBounds, LocalFileReadError, read_regular_file};

pub const MAX_PACK_FILES: usize = 16;
const MAX_PACK_DIRECTORY_ENTRIES: usize = MAX_PACK_FILES * 4 + 16;
pub const MAX_TOTAL_PACK_INDEX_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_TOTAL_PACK_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_LOG_COMMITS: usize = 512;
pub const MAX_COMMIT_PARENTS: usize = 64;
pub const MAX_TREE_ENTRIES: usize = 250_000;
pub const MAX_TREE_DEPTH: usize = 64;
pub const MAX_TREE_NAME_BYTES: usize = 4096;
pub const MAX_COMMIT_HEADER_BYTES: usize = 1024 * 1024;
const OBJECT_OBSERVATION_DOMAIN: &[u8] = b"golam:git-object-observation:v1";
const BLOB_OBSERVATION_DOMAIN: &[u8] = b"golam:git-blob-observation:v1";
const COMMIT_OBSERVATION_DOMAIN: &[u8] = b"golam:git-commit-observation:v1";
const TREE_OBJECT_OBSERVATION_DOMAIN: &[u8] = b"golam:git-tree-object-observation:v1";
const TREE_WALK_OBSERVATION_DOMAIN: &[u8] = b"golam:git-tree-walk-observation:v1";
const LOG_OBSERVATION_DOMAIN: &[u8] = b"golam:git-log-observation:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GitObservationBounds {
    pub git: GitReadBounds,
    pub pack: GitPackBounds,
    pub max_pack_files: usize,
    pub max_total_pack_index_bytes: u64,
    pub max_total_pack_bytes: u64,
    pub max_log_commits: usize,
    pub max_commit_parents: usize,
    pub max_tree_entries: usize,
    pub max_tree_depth: usize,
    pub max_tree_name_bytes: usize,
    pub max_commit_header_bytes: usize,
}

impl Default for GitObservationBounds {
    fn default() -> Self {
        Self {
            git: GitReadBounds::default(),
            pack: GitPackBounds::default(),
            max_pack_files: MAX_PACK_FILES,
            max_total_pack_index_bytes: MAX_TOTAL_PACK_INDEX_BYTES,
            max_total_pack_bytes: MAX_TOTAL_PACK_BYTES,
            max_log_commits: MAX_LOG_COMMITS,
            max_commit_parents: MAX_COMMIT_PARENTS,
            max_tree_entries: MAX_TREE_ENTRIES,
            max_tree_depth: MAX_TREE_DEPTH,
            max_tree_name_bytes: MAX_TREE_NAME_BYTES,
            max_commit_header_bytes: MAX_COMMIT_HEADER_BYTES,
        }
    }
}

impl GitObservationBounds {
    pub fn validate(self) -> Result<(), GitObservationError> {
        self.git.validate()?;
        self.pack.validate()?;
        if self.max_pack_files == 0
            || self.max_pack_files > MAX_PACK_FILES
            || self.max_total_pack_index_bytes == 0
            || self.max_total_pack_index_bytes > MAX_TOTAL_PACK_INDEX_BYTES
            || self.max_total_pack_bytes == 0
            || self.max_total_pack_bytes > MAX_TOTAL_PACK_BYTES
            || self.max_log_commits == 0
            || self.max_log_commits > MAX_LOG_COMMITS
            || self.max_commit_parents == 0
            || self.max_commit_parents > MAX_COMMIT_PARENTS
            || self.max_tree_entries == 0
            || self.max_tree_entries > MAX_TREE_ENTRIES
            || self.max_tree_depth == 0
            || self.max_tree_depth > MAX_TREE_DEPTH
            || self.max_tree_name_bytes == 0
            || self.max_tree_name_bytes > MAX_TREE_NAME_BYTES
            || self.max_commit_header_bytes == 0
            || self.max_commit_header_bytes > MAX_COMMIT_HEADER_BYTES
            || self.pack.max_object_bytes > self.git.max_single_object_decompressed_bytes
            || self.pack.max_duration > self.git.max_duration
        {
            return Err(GitObservationError::InvalidBounds);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GitObjectSource {
    Loose,
    Pack { pack_name: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitObjectObservation {
    pub(crate) repository_evidence: GitRepositoryEvidence,
    pub(crate) object: GitObject,
    pub(crate) source: GitObjectSource,
    binding_digest: BindingDigest,
}

impl GitObjectObservation {
    fn new(
        repository_evidence: GitRepositoryEvidence,
        object: GitObject,
        source: GitObjectSource,
    ) -> Result<Self, GitObservationError> {
        let binding_digest = object_observation_binding(&repository_evidence, &object, &source)?;
        Ok(Self {
            repository_evidence,
            object,
            source,
            binding_digest,
        })
    }

    pub fn repository_evidence(&self) -> &GitRepositoryEvidence {
        &self.repository_evidence
    }

    pub fn object(&self) -> &GitObject {
        &self.object
    }

    pub fn source(&self) -> &GitObjectSource {
        &self.source
    }

    pub const fn binding_digest(&self) -> BindingDigest {
        self.binding_digest
    }

    pub fn verify_binding(&self) -> Result<(), GitObservationError> {
        let expected = object_observation_binding(
            &self.repository_evidence,
            &self.object,
            &self.source,
        )?;
        verify_observation_digest(expected, self.binding_digest)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitBlobObservation {
    pub(crate) repository_evidence: GitRepositoryEvidence,
    pub(crate) object_id: GitObjectId,
    pub(crate) source: GitObjectSource,
    pub(crate) bytes: Vec<u8>,
    binding_digest: BindingDigest,
}

impl GitBlobObservation {
    fn new(
        repository_evidence: GitRepositoryEvidence,
        object_id: GitObjectId,
        source: GitObjectSource,
        bytes: Vec<u8>,
    ) -> Result<Self, GitObservationError> {
        let binding_digest =
            blob_observation_binding(&repository_evidence, object_id, &source, &bytes)?;
        Ok(Self {
            repository_evidence,
            object_id,
            source,
            bytes,
            binding_digest,
        })
    }

    pub fn repository_evidence(&self) -> &GitRepositoryEvidence {
        &self.repository_evidence
    }

    pub const fn object_id(&self) -> GitObjectId {
        self.object_id
    }

    pub fn source(&self) -> &GitObjectSource {
        &self.source
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub const fn binding_digest(&self) -> BindingDigest {
        self.binding_digest
    }

    pub fn verify_binding(&self) -> Result<(), GitObservationError> {
        let expected = blob_observation_binding(
            &self.repository_evidence,
            self.object_id,
            &self.source,
            &self.bytes,
        )?;
        verify_observation_digest(expected, self.binding_digest)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitCommitObservation {
    pub(crate) repository_evidence: GitRepositoryEvidence,
    pub(crate) object_id: GitObjectId,
    pub(crate) source: GitObjectSource,
    pub(crate) tree: GitObjectId,
    pub(crate) parents: Vec<GitObjectId>,
    pub(crate) author: Vec<u8>,
    pub(crate) committer: Vec<u8>,
    pub(crate) message: Vec<u8>,
    binding_digest: BindingDigest,
}

impl GitCommitObservation {
    fn new(
        repository_evidence: GitRepositoryEvidence,
        object_id: GitObjectId,
        source: GitObjectSource,
        tree: GitObjectId,
        parents: Vec<GitObjectId>,
        author: Vec<u8>,
        committer: Vec<u8>,
        message: Vec<u8>,
    ) -> Result<Self, GitObservationError> {
        let binding_digest = commit_observation_binding(
            &repository_evidence,
            object_id,
            &source,
            tree,
            &parents,
            &author,
            &committer,
            &message,
        )?;
        Ok(Self {
            repository_evidence,
            object_id,
            source,
            tree,
            parents,
            author,
            committer,
            message,
            binding_digest,
        })
    }

    pub fn repository_evidence(&self) -> &GitRepositoryEvidence {
        &self.repository_evidence
    }

    pub const fn object_id(&self) -> GitObjectId {
        self.object_id
    }

    pub fn source(&self) -> &GitObjectSource {
        &self.source
    }

    pub const fn tree(&self) -> GitObjectId {
        self.tree
    }

    pub fn parents(&self) -> &[GitObjectId] {
        &self.parents
    }

    pub fn author(&self) -> &[u8] {
        &self.author
    }

    pub fn committer(&self) -> &[u8] {
        &self.committer
    }

    pub fn message(&self) -> &[u8] {
        &self.message
    }

    pub const fn binding_digest(&self) -> BindingDigest {
        self.binding_digest
    }

    pub fn verify_binding(&self) -> Result<(), GitObservationError> {
        let expected = commit_observation_binding(
            &self.repository_evidence,
            self.object_id,
            &self.source,
            self.tree,
            &self.parents,
            &self.author,
            &self.committer,
            &self.message,
        )?;
        verify_observation_digest(expected, self.binding_digest)
    }
}

struct ParsedGitCommit {
    tree: GitObjectId,
    parents: Vec<GitObjectId>,
    author: Vec<u8>,
    committer: Vec<u8>,
    message: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitTreeMode {
    RegularFile { executable: bool },
    SymbolicLink,
    Directory,
    Gitlink,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitTreeEntryObservation {
    pub mode: GitTreeMode,
    pub name: Vec<u8>,
    pub object_id: GitObjectId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitTreeObjectObservation {
    pub(crate) repository_evidence: GitRepositoryEvidence,
    pub(crate) object_id: GitObjectId,
    pub(crate) source: GitObjectSource,
    pub(crate) entries: Vec<GitTreeEntryObservation>,
    binding_digest: BindingDigest,
}

impl GitTreeObjectObservation {
    fn new(
        repository_evidence: GitRepositoryEvidence,
        object_id: GitObjectId,
        source: GitObjectSource,
        entries: Vec<GitTreeEntryObservation>,
    ) -> Result<Self, GitObservationError> {
        let binding_digest = tree_object_observation_binding(
            &repository_evidence,
            object_id,
            &source,
            &entries,
        )?;
        Ok(Self {
            repository_evidence,
            object_id,
            source,
            entries,
            binding_digest,
        })
    }

    pub fn repository_evidence(&self) -> &GitRepositoryEvidence {
        &self.repository_evidence
    }

    pub const fn object_id(&self) -> GitObjectId {
        self.object_id
    }

    pub fn source(&self) -> &GitObjectSource {
        &self.source
    }

    pub fn entries(&self) -> &[GitTreeEntryObservation] {
        &self.entries
    }

    pub const fn binding_digest(&self) -> BindingDigest {
        self.binding_digest
    }

    pub fn verify_binding(&self) -> Result<(), GitObservationError> {
        let expected = tree_object_observation_binding(
            &self.repository_evidence,
            self.object_id,
            &self.source,
            &self.entries,
        )?;
        verify_observation_digest(expected, self.binding_digest)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitTreePathObservation {
    pub path: String,
    pub mode: GitTreeMode,
    pub object_id: GitObjectId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitTreeWalkObservation {
    pub(crate) repository_evidence: GitRepositoryEvidence,
    pub(crate) root_tree: GitObjectId,
    pub(crate) entries: Vec<GitTreePathObservation>,
    pub(crate) truncated: bool,
    binding_digest: BindingDigest,
}

impl GitTreeWalkObservation {
    fn new(
        repository_evidence: GitRepositoryEvidence,
        root_tree: GitObjectId,
        entries: Vec<GitTreePathObservation>,
        truncated: bool,
    ) -> Result<Self, GitObservationError> {
        let binding_digest = tree_walk_observation_binding(
            &repository_evidence,
            root_tree,
            &entries,
            truncated,
        )?;
        Ok(Self {
            repository_evidence,
            root_tree,
            entries,
            truncated,
            binding_digest,
        })
    }

    pub fn repository_evidence(&self) -> &GitRepositoryEvidence {
        &self.repository_evidence
    }

    pub const fn root_tree(&self) -> GitObjectId {
        self.root_tree
    }

    pub fn entries(&self) -> &[GitTreePathObservation] {
        &self.entries
    }

    pub const fn truncated(&self) -> bool {
        self.truncated
    }

    pub const fn binding_digest(&self) -> BindingDigest {
        self.binding_digest
    }

    pub fn verify_binding(&self) -> Result<(), GitObservationError> {
        let expected = tree_walk_observation_binding(
            &self.repository_evidence,
            self.root_tree,
            &self.entries,
            self.truncated,
        )?;
        verify_observation_digest(expected, self.binding_digest)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitLogObservation {
    pub(crate) repository_evidence: GitRepositoryEvidence,
    pub(crate) commits: Vec<GitCommitObservation>,
    pub(crate) truncated: bool,
    binding_digest: BindingDigest,
}

impl GitLogObservation {
    fn new(
        repository_evidence: GitRepositoryEvidence,
        commits: Vec<GitCommitObservation>,
        truncated: bool,
    ) -> Result<Self, GitObservationError> {
        for commit in &commits {
            commit.verify_binding()?;
        }
        let binding_digest = log_observation_binding(&repository_evidence, &commits, truncated)?;
        Ok(Self {
            repository_evidence,
            commits,
            truncated,
            binding_digest,
        })
    }

    pub fn repository_evidence(&self) -> &GitRepositoryEvidence {
        &self.repository_evidence
    }

    pub fn commits(&self) -> &[GitCommitObservation] {
        &self.commits
    }

    pub const fn truncated(&self) -> bool {
        self.truncated
    }

    pub const fn binding_digest(&self) -> BindingDigest {
        self.binding_digest
    }

    pub fn verify_binding(&self) -> Result<(), GitObservationError> {
        for commit in &self.commits {
            commit.verify_binding()?;
        }
        let expected =
            log_observation_binding(&self.repository_evidence, &self.commits, self.truncated)?;
        verify_observation_digest(expected, self.binding_digest)
    }
}

fn object_observation_binding(
    repository_evidence: &GitRepositoryEvidence,
    object: &GitObject,
    source: &GitObjectSource,
) -> Result<BindingDigest, GitObservationError> {
    repository_evidence.verify_binding()?;
    let mut encoder = observation_encoder(OBJECT_OBSERVATION_DOMAIN, repository_evidence)?;
    push_object_id(&mut encoder, object.id)?;
    encoder.push_u8(object_kind_tag(object.kind));
    encoder.push_u64(object.declared_size);
    encoder.push_bytes(&object.bytes)?;
    push_object_source(&mut encoder, source)?;
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn blob_observation_binding(
    repository_evidence: &GitRepositoryEvidence,
    object_id: GitObjectId,
    source: &GitObjectSource,
    bytes: &[u8],
) -> Result<BindingDigest, GitObservationError> {
    repository_evidence.verify_binding()?;
    let mut encoder = observation_encoder(BLOB_OBSERVATION_DOMAIN, repository_evidence)?;
    push_object_id(&mut encoder, object_id)?;
    push_object_source(&mut encoder, source)?;
    encoder.push_bytes(bytes)?;
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

#[allow(clippy::too_many_arguments)]
fn commit_observation_binding(
    repository_evidence: &GitRepositoryEvidence,
    object_id: GitObjectId,
    source: &GitObjectSource,
    tree: GitObjectId,
    parents: &[GitObjectId],
    author: &[u8],
    committer: &[u8],
    message: &[u8],
) -> Result<BindingDigest, GitObservationError> {
    repository_evidence.verify_binding()?;
    let mut encoder = observation_encoder(COMMIT_OBSERVATION_DOMAIN, repository_evidence)?;
    push_object_id(&mut encoder, object_id)?;
    push_object_source(&mut encoder, source)?;
    push_object_id(&mut encoder, tree)?;
    encoder.push_u64(observation_len(parents.len())?);
    for parent in parents {
        push_object_id(&mut encoder, *parent)?;
    }
    encoder.push_bytes(author)?;
    encoder.push_bytes(committer)?;
    encoder.push_bytes(message)?;
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn tree_object_observation_binding(
    repository_evidence: &GitRepositoryEvidence,
    object_id: GitObjectId,
    source: &GitObjectSource,
    entries: &[GitTreeEntryObservation],
) -> Result<BindingDigest, GitObservationError> {
    repository_evidence.verify_binding()?;
    let mut encoder = observation_encoder(TREE_OBJECT_OBSERVATION_DOMAIN, repository_evidence)?;
    push_object_id(&mut encoder, object_id)?;
    push_object_source(&mut encoder, source)?;
    encoder.push_u64(observation_len(entries.len())?);
    for entry in entries {
        push_tree_mode(&mut encoder, entry.mode);
        encoder.push_bytes(&entry.name)?;
        push_object_id(&mut encoder, entry.object_id)?;
    }
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn tree_walk_observation_binding(
    repository_evidence: &GitRepositoryEvidence,
    root_tree: GitObjectId,
    entries: &[GitTreePathObservation],
    truncated: bool,
) -> Result<BindingDigest, GitObservationError> {
    repository_evidence.verify_binding()?;
    let mut encoder = observation_encoder(TREE_WALK_OBSERVATION_DOMAIN, repository_evidence)?;
    push_object_id(&mut encoder, root_tree)?;
    encoder.push_u64(observation_len(entries.len())?);
    for entry in entries {
        encoder.push_bytes(entry.path.as_bytes())?;
        push_tree_mode(&mut encoder, entry.mode);
        push_object_id(&mut encoder, entry.object_id)?;
    }
    encoder.push_u8(u8::from(truncated));
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn log_observation_binding(
    repository_evidence: &GitRepositoryEvidence,
    commits: &[GitCommitObservation],
    truncated: bool,
) -> Result<BindingDigest, GitObservationError> {
    repository_evidence.verify_binding()?;
    let mut encoder = observation_encoder(LOG_OBSERVATION_DOMAIN, repository_evidence)?;
    encoder.push_u64(observation_len(commits.len())?);
    for commit in commits {
        encoder.push_bytes(&commit.binding_digest().bytes())?;
    }
    encoder.push_u8(u8::from(truncated));
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn observation_encoder(
    domain: &[u8],
    repository_evidence: &GitRepositoryEvidence,
) -> Result<CanonicalEncoder, GitObservationError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(domain)?;
    encoder.push_bytes(&repository_evidence.binding_digest().bytes())?;
    Ok(encoder)
}

fn push_object_source(
    encoder: &mut CanonicalEncoder,
    source: &GitObjectSource,
) -> Result<(), GitObservationError> {
    match source {
        GitObjectSource::Loose => encoder.push_u8(0),
        GitObjectSource::Pack { pack_name } => {
            encoder.push_u8(1);
            encoder.push_bytes(pack_name.as_bytes())?;
        }
    }
    Ok(())
}

fn push_tree_mode(encoder: &mut CanonicalEncoder, mode: GitTreeMode) {
    match mode {
        GitTreeMode::RegularFile { executable } => {
            encoder.push_u8(0);
            encoder.push_u8(u8::from(executable));
        }
        GitTreeMode::SymbolicLink => encoder.push_u8(1),
        GitTreeMode::Directory => encoder.push_u8(2),
        GitTreeMode::Gitlink => encoder.push_u8(3),
    }
}

fn push_object_id(
    encoder: &mut CanonicalEncoder,
    object_id: GitObjectId,
) -> Result<(), GitObservationError> {
    encoder.push_bytes(&object_id.bytes())?;
    Ok(())
}

fn observation_len(value: usize) -> Result<u64, GitObservationError> {
    u64::try_from(value).map_err(|_| GitObservationError::ObjectSizeOverflow)
}

fn object_kind_tag(kind: GitObjectKind) -> u8 {
    match kind {
        GitObjectKind::Blob => 0,
        GitObjectKind::Tree => 1,
        GitObjectKind::Commit => 2,
        GitObjectKind::Tag => 3,
    }
}

fn verify_observation_digest(
    expected: BindingDigest,
    observed: BindingDigest,
) -> Result<(), GitObservationError> {
    if expected != observed {
        return Err(GitObservationError::ObservationBindingMismatch);
    }
    Ok(())
}

struct LoadedPack {
    name: String,
    index: GitPackIndex,
    bytes: Vec<u8>,
    validated: ValidatedPack,
}

pub struct GitObservationReader<'a> {
    repository: GitRepositoryReader<'a>,
    packs: Vec<LoadedPack>,
    bounds: GitObservationBounds,
    deadline: GitOperationDeadline,
    observed_at_unix_ms: u64,
}

impl<'a> GitObservationReader<'a> {
    pub fn open(
        resolver: &'a LocalFsResolver,
        operation: &RequestedOperationId,
        bounds: GitObservationBounds,
        observed_at_unix_ms: u64,
    ) -> Result<Self, GitObservationError> {
        bounds.validate()?;
        let deadline = GitOperationDeadline::start(bounds.git.max_duration)?;
        Self::open_with_deadline(resolver, operation, bounds, deadline, observed_at_unix_ms)
    }

    pub(crate) fn open_with_deadline(
        resolver: &'a LocalFsResolver,
        operation: &RequestedOperationId,
        bounds: GitObservationBounds,
        deadline: GitOperationDeadline,
        observed_at_unix_ms: u64,
    ) -> Result<Self, GitObservationError> {
        bounds.validate()?;
        deadline.require_active()?;
        let repository = GitRepositoryReader::open_with_deadline(
            resolver,
            operation,
            bounds.git,
            deadline,
            observed_at_unix_ms,
        )?;
        reject_alternate_object_store(resolver, operation, deadline, observed_at_unix_ms)?;
        let packs = load_packs(resolver, operation, bounds, deadline, observed_at_unix_ms)?;
        deadline.require_active()?;
        Ok(Self {
            repository,
            packs,
            bounds,
            deadline,
            observed_at_unix_ms,
        })
    }

    pub fn evidence(&self) -> &GitRepositoryEvidence {
        self.repository.evidence()
    }

    pub fn read_object(
        &self,
        object_id: GitObjectId,
    ) -> Result<GitObjectObservation, GitObservationError> {
        match self
            .repository
            .read_loose_object(object_id, self.observed_at_unix_ms)
        {
            Ok(object) => {
                return GitObjectObservation::new(
                    self.repository.evidence().clone(),
                    object,
                    GitObjectSource::Loose,
                );
            }
            Err(GitReadError::MissingObject(_)) => {}
            Err(error) => return Err(error.into()),
        }

        let wanted = PackObjectId::from_bytes(object_id.bytes());
        let mut match_index = None;
        for (index, pack) in self.packs.iter().enumerate() {
            if pack
                .index
                .entries
                .binary_search_by_key(&wanted, |entry| entry.object_id)
                .is_ok()
            {
                if match_index.is_some() {
                    return Err(GitObservationError::DuplicatePackedObject(object_id));
                }
                match_index = Some(index);
            }
        }

        let pack = match_index
            .and_then(|index| self.packs.get(index))
            .ok_or(GitObservationError::MissingObject(object_id))?;
        let packed = read_validated_packed_object_with_deadline(
            &pack.bytes,
            &pack.index,
            &pack.validated,
            wanted,
            self.bounds.pack,
            self.deadline,
        )?;
        let kind = match packed.kind {
            PackedObjectKind::Commit => GitObjectKind::Commit,
            PackedObjectKind::Tree => GitObjectKind::Tree,
            PackedObjectKind::Blob => GitObjectKind::Blob,
            PackedObjectKind::Tag => GitObjectKind::Tag,
        };
        let declared_size = u64::try_from(packed.bytes.len())
            .map_err(|_| GitObservationError::ObjectSizeOverflow)?;
        GitObjectObservation::new(
            self.repository.evidence().clone(),
            GitObject {
                id: object_id,
                kind,
                declared_size,
                bytes: packed.bytes,
            },
            GitObjectSource::Pack {
                pack_name: pack.name.clone(),
            },
        )
    }

    pub fn read_blob(
        &self,
        object_id: GitObjectId,
    ) -> Result<GitBlobObservation, GitObservationError> {
        let observation = self.read_object(object_id)?;
        if observation.object.kind != GitObjectKind::Blob {
            return Err(GitObservationError::UnexpectedObjectKind {
                expected: GitObjectKind::Blob,
                observed: observation.object.kind,
            });
        }
        GitBlobObservation::new(
            observation.repository_evidence,
            object_id,
            observation.source,
            observation.object.bytes,
        )
    }

    pub fn read_commit(
        &self,
        object_id: GitObjectId,
    ) -> Result<GitCommitObservation, GitObservationError> {
        let observation = self.read_object(object_id)?;
        if observation.object.kind != GitObjectKind::Commit {
            return Err(GitObservationError::UnexpectedObjectKind {
                expected: GitObjectKind::Commit,
                observed: observation.object.kind,
            });
        }
        let parsed = self
            .deadline
            .run_step(|| parse_commit(&observation.object.bytes, self.bounds))??;
        GitCommitObservation::new(
            observation.repository_evidence,
            object_id,
            observation.source,
            parsed.tree,
            parsed.parents,
            parsed.author,
            parsed.committer,
            parsed.message,
        )
    }

    pub fn read_tree(
        &self,
        object_id: GitObjectId,
    ) -> Result<GitTreeObjectObservation, GitObservationError> {
        let observation = self.read_object(object_id)?;
        if observation.object.kind != GitObjectKind::Tree {
            return Err(GitObservationError::UnexpectedObjectKind {
                expected: GitObjectKind::Tree,
                observed: observation.object.kind,
            });
        }
        let entries = self
            .deadline
            .run_step(|| parse_tree(&observation.object.bytes, self.bounds))??;
        GitTreeObjectObservation::new(
            observation.repository_evidence,
            object_id,
            observation.source,
            entries,
        )
    }

    pub fn observe_head_log(&self) -> Result<GitLogObservation, GitObservationError> {
        let repository_evidence = self.repository.evidence().clone();
        let mut pending = VecDeque::new();
        let mut seen = HashSet::new();
        let mut commits = Vec::new();
        pending.push_back(self.repository.evidence().head.object_id);

        while let Some(object_id) = pending.pop_front() {
            self.deadline.require_active()?;
            if !seen.insert(object_id) {
                continue;
            }
            if commits.len() >= self.bounds.max_log_commits {
                return GitLogObservation::new(repository_evidence.clone(), commits, true);
            }
            let commit = self.read_commit(object_id)?;
            for parent in &commit.parents {
                if !seen.contains(parent) {
                    pending.push_back(*parent);
                }
            }
            commits.push(commit);
        }

        GitLogObservation::new(repository_evidence, commits, false)
    }

    pub fn observe_head_tree(&self) -> Result<GitTreeWalkObservation, GitObservationError> {
        let head = self.read_commit(self.repository.evidence().head.object_id)?;
        self.walk_tree(head.tree)
    }

    pub fn walk_tree(
        &self,
        root_tree: GitObjectId,
    ) -> Result<GitTreeWalkObservation, GitObservationError> {
        let repository_evidence = self.repository.evidence().clone();
        let mut pending = VecDeque::new();
        let mut paths = BTreeMap::new();
        pending.push_back((String::new(), root_tree, 0_usize));

        while let Some((prefix, tree_id, depth)) = pending.pop_front() {
            self.deadline.require_active()?;
            if depth > self.bounds.max_tree_depth {
                return Err(GitObservationError::TreeDepthExceeded);
            }
            let tree = self.read_tree(tree_id)?;
            for entry in tree.entries {
                if paths.len() >= self.bounds.max_tree_entries {
                    return GitTreeWalkObservation::new(
                        repository_evidence.clone(),
                        root_tree,
                        paths.into_values().collect(),
                        true,
                    );
                }
                let name = std::str::from_utf8(&entry.name)
                    .map_err(|_| GitObservationError::NonUnicodeTreeName)?;
                let path = if prefix.is_empty() {
                    name.to_owned()
                } else {
                    format!("{prefix}/{name}")
                };
                match entry.mode {
                    GitTreeMode::Directory => {
                        pending.push_back((path, entry.object_id, depth + 1));
                    }
                    _ => {
                        let observation = GitTreePathObservation {
                            path: path.clone(),
                            mode: entry.mode,
                            object_id: entry.object_id,
                        };
                        if paths.insert(path, observation).is_some() {
                            return Err(GitObservationError::DuplicateTreePath);
                        }
                    }
                }
            }
        }

        GitTreeWalkObservation::new(
            repository_evidence,
            root_tree,
            paths.into_values().collect(),
            false,
        )
    }
}

fn reject_alternate_object_store(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    deadline: GitOperationDeadline,
    observed_at_unix_ms: u64,
) -> Result<(), GitObservationError> {
    deadline.require_active()?;
    let info = requested(".git/objects/info")?;
    let info_identity = resolver.resolve_read_target(&info, operation, observed_at_unix_ms)?;
    match info_identity.file_kind {
        ObservedFileKind::Missing => return Ok(()),
        ObservedFileKind::Directory => {}
        kind => return Err(GitObservationError::UnexpectedObjectStoreKind(kind)),
    }

    deadline.require_active()?;
    let alternates = requested(".git/objects/info/alternates")?;
    let identity = resolver.resolve_read_target(&alternates, operation, observed_at_unix_ms)?;
    if identity.file_kind != ObservedFileKind::Missing {
        return Err(GitObservationError::AlternateObjectStoreUnsupported);
    }
    Ok(())
}

fn load_packs(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    bounds: GitObservationBounds,
    deadline: GitOperationDeadline,
    observed_at_unix_ms: u64,
) -> Result<Vec<LoadedPack>, GitObservationError> {
    deadline.require_active()?;
    let requested_pack_dir = requested(".git/objects/pack")?;
    let initial =
        resolver.resolve_read_target(&requested_pack_dir, operation, observed_at_unix_ms)?;
    match initial.file_kind {
        ObservedFileKind::Missing => return Ok(Vec::new()),
        ObservedFileKind::Directory => {}
        kind => return Err(GitObservationError::UnexpectedObjectStoreKind(kind)),
    }

    let multi_pack_index = requested(".git/objects/pack/multi-pack-index")?;
    let midx = resolver.resolve_read_target(&multi_pack_index, operation, observed_at_unix_ms)?;
    if midx.file_kind != ObservedFileKind::Missing {
        return Err(GitObservationError::MultiPackIndexUnsupported);
    }

    let snapshot = snapshot_directory(
        resolver,
        &requested_pack_dir,
        operation,
        LocalDirectorySnapshotBounds {
            max_entries: MAX_PACK_DIRECTORY_ENTRIES,
            max_duration: deadline.remaining()?,
        },
        observed_at_unix_ms,
    )?;
    let mut index_names = Vec::new();
    let mut pack_names = HashSet::new();
    for name in snapshot.names {
        deadline.require_active()?;
        if name.ends_with(".idx") {
            validate_pack_file_name(&name, ".idx")?;
            index_names.push(name);
        } else if name.ends_with(".pack") {
            validate_pack_file_name(&name, ".pack")?;
            pack_names.insert(name);
        }
    }
    index_names.sort_unstable();
    if index_names.len() > bounds.max_pack_files || pack_names.len() > bounds.max_pack_files {
        return Err(GitObservationError::PackFileLimitExceeded);
    }
    if index_names.len() != pack_names.len() {
        return Err(GitObservationError::PackIndexPairMismatch);
    }

    let mut total_index_bytes = 0_u64;
    let mut total_pack_bytes = 0_u64;
    let mut packs = Vec::with_capacity(index_names.len());
    for index_name in index_names {
        let stem = index_name
            .strip_suffix(".idx")
            .ok_or(GitObservationError::InvalidPackFileName)?;
        let pack_name = format!("{stem}.pack");
        if !pack_names.remove(&pack_name) {
            return Err(GitObservationError::PackIndexPairMismatch);
        }
        let index_path = format!(".git/objects/pack/{index_name}");
        let pack_path = format!(".git/objects/pack/{pack_name}");
        let index_bytes = read_required_file(
            resolver,
            operation,
            &index_path,
            u64::try_from(bounds.pack.max_index_bytes)
                .map_err(|_| GitObservationError::ObjectSizeOverflow)?,
            deadline,
            observed_at_unix_ms,
        )?;
        total_index_bytes = total_index_bytes
            .checked_add(
                u64::try_from(index_bytes.len())
                    .map_err(|_| GitObservationError::ObjectSizeOverflow)?,
            )
            .ok_or(GitObservationError::PackIndexByteLimitExceeded)?;
        if total_index_bytes > bounds.max_total_pack_index_bytes {
            return Err(GitObservationError::PackIndexByteLimitExceeded);
        }
        let index = deadline.run_step(|| parse_pack_index_v2(&index_bytes, bounds.pack))??;
        let expected_checksum = parse_pack_name_checksum(stem)?;
        if index.pack_checksum != expected_checksum {
            return Err(GitObservationError::PackNameChecksumMismatch);
        }

        let pack_bytes = read_required_file(
            resolver,
            operation,
            &pack_path,
            u64::try_from(bounds.pack.max_pack_bytes)
                .map_err(|_| GitObservationError::ObjectSizeOverflow)?,
            deadline,
            observed_at_unix_ms,
        )?;
        total_pack_bytes = total_pack_bytes
            .checked_add(
                u64::try_from(pack_bytes.len())
                    .map_err(|_| GitObservationError::ObjectSizeOverflow)?,
            )
            .ok_or(GitObservationError::PackByteLimitExceeded)?;
        if total_pack_bytes > bounds.max_total_pack_bytes {
            return Err(GitObservationError::PackByteLimitExceeded);
        }
        let validated =
            validate_pack_for_reuse_with_deadline(&pack_bytes, &index, bounds.pack, deadline)?;
        packs.push(LoadedPack {
            name: pack_name,
            index,
            bytes: pack_bytes,
            validated,
        });
    }
    if !pack_names.is_empty() {
        return Err(GitObservationError::PackIndexPairMismatch);
    }

    let verified =
        resolver.resolve_read_target(&requested_pack_dir, operation, observed_at_unix_ms)?;
    if initial.resolved_target_identity != verified.resolved_target_identity
        || initial.observed_metadata_digest != verified.observed_metadata_digest
    {
        return Err(GitObservationError::PackDirectoryChangedDuringObservation);
    }
    Ok(packs)
}

fn validate_pack_file_name(name: &str, suffix: &str) -> Result<(), GitObservationError> {
    let Some(stem) = name.strip_suffix(suffix) else {
        return Err(GitObservationError::InvalidPackFileName);
    };
    parse_pack_name_checksum(stem).map(|_| ())
}

fn parse_pack_name_checksum(stem: &str) -> Result<[u8; 20], GitObservationError> {
    let hex = stem
        .strip_prefix("pack-")
        .ok_or(GitObservationError::InvalidPackFileName)?;
    if hex.len() != 40
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(GitObservationError::InvalidPackFileName);
    }
    let mut bytes = [0_u8; 20];
    for (index, pair) in hex.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        bytes[index] = (hex_nibble(pair[0])? << 4) | hex_nibble(pair[1])?;
    }
    Ok(bytes)
}

fn hex_nibble(byte: u8) -> Result<u8, GitObservationError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(GitObservationError::InvalidPackFileName),
    }
}

fn read_required_file(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    path: &str,
    max_bytes: u64,
    deadline: GitOperationDeadline,
    observed_at_unix_ms: u64,
) -> Result<Vec<u8>, GitObservationError> {
    deadline.require_active()?;
    let requested = requested(path)?;
    let identity = resolver.resolve_read_target(&requested, operation, observed_at_unix_ms)?;
    if identity.file_kind == ObservedFileKind::Missing {
        return Err(GitObservationError::MissingInternalFile(path.to_owned()));
    }
    if identity.file_kind != ObservedFileKind::RegularFile {
        return Err(GitObservationError::UnexpectedObjectStoreKind(
            identity.file_kind,
        ));
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

fn requested(path: &str) -> Result<RequestedTarget, GitObservationError> {
    RequestedTarget::new(path).map_err(|_| GitObservationError::InvalidInternalPath)
}

fn parse_commit(
    bytes: &[u8],
    bounds: GitObservationBounds,
) -> Result<ParsedGitCommit, GitObservationError> {
    let separator = bytes
        .windows(2)
        .position(|window| window == b"\n\n")
        .ok_or(GitObservationError::InvalidCommit)?;
    let header_end = separator + 1;
    if header_end > bounds.max_commit_header_bytes {
        return Err(GitObservationError::CommitHeaderLimitExceeded);
    }
    let headers = &bytes[..header_end];
    let message = bytes[separator + 2..].to_vec();
    let mut tree = None;
    let mut parents = Vec::new();
    let mut author = None;
    let mut committer = None;
    let mut previous_header = false;

    for line in headers.split(|byte| *byte == b'\n') {
        if line.is_empty() {
            continue;
        }
        if line[0] == b' ' {
            if !previous_header {
                return Err(GitObservationError::InvalidCommit);
            }
            continue;
        }
        previous_header = true;
        let Some(space) = line.iter().position(|byte| *byte == b' ') else {
            return Err(GitObservationError::InvalidCommit);
        };
        let key = &line[..space];
        let value = &line[space + 1..];
        match key {
            b"tree" => {
                if tree.is_some() {
                    return Err(GitObservationError::InvalidCommit);
                }
                tree = Some(parse_object_id_bytes(value)?);
            }
            b"parent" => {
                if parents.len() >= bounds.max_commit_parents {
                    return Err(GitObservationError::CommitParentLimitExceeded);
                }
                parents.push(parse_object_id_bytes(value)?);
            }
            b"author" if author.is_some() => {
                return Err(GitObservationError::InvalidCommit);
            }
            b"author" => author = Some(value.to_vec()),
            b"committer" if committer.is_some() => {
                return Err(GitObservationError::InvalidCommit);
            }
            b"committer" => committer = Some(value.to_vec()),
            _ => {}
        }
    }

    Ok(ParsedGitCommit {
        tree: tree.ok_or(GitObservationError::InvalidCommit)?,
        parents,
        author: author.ok_or(GitObservationError::InvalidCommit)?,
        committer: committer.ok_or(GitObservationError::InvalidCommit)?,
        message,
    })
}

fn parse_tree(
    bytes: &[u8],
    bounds: GitObservationBounds,
) -> Result<Vec<GitTreeEntryObservation>, GitObservationError> {
    let mut cursor = 0_usize;
    let mut entries = Vec::new();
    while cursor < bytes.len() {
        if entries.len() >= bounds.max_tree_entries {
            return Err(GitObservationError::TreeEntryLimitExceeded);
        }
        let mode_end = bytes[cursor..]
            .iter()
            .position(|byte| *byte == b' ')
            .map(|relative| cursor + relative)
            .ok_or(GitObservationError::InvalidTree)?;
        if mode_end == cursor || mode_end - cursor > 6 {
            return Err(GitObservationError::InvalidTree);
        }
        let mode = parse_tree_mode(&bytes[cursor..mode_end])?;
        cursor = mode_end + 1;
        let name_end = bytes[cursor..]
            .iter()
            .take(bounds.max_tree_name_bytes.saturating_add(1))
            .position(|byte| *byte == 0)
            .map(|relative| cursor + relative)
            .ok_or(GitObservationError::TreeNameLimitExceeded)?;
        let name = bytes[cursor..name_end].to_vec();
        validate_tree_name(&name)?;
        cursor = name_end + 1;
        let end = cursor
            .checked_add(20)
            .ok_or(GitObservationError::InvalidTree)?;
        let raw_id = bytes
            .get(cursor..end)
            .ok_or(GitObservationError::InvalidTree)?;
        let object_id = object_id_from_raw(raw_id)?;
        cursor = end;
        entries.push(GitTreeEntryObservation {
            mode,
            name,
            object_id,
        });
    }
    Ok(entries)
}

fn parse_tree_mode(bytes: &[u8]) -> Result<GitTreeMode, GitObservationError> {
    match bytes {
        b"100644" => Ok(GitTreeMode::RegularFile { executable: false }),
        b"100755" => Ok(GitTreeMode::RegularFile { executable: true }),
        b"120000" => Ok(GitTreeMode::SymbolicLink),
        b"40000" | b"040000" => Ok(GitTreeMode::Directory),
        b"160000" => Ok(GitTreeMode::Gitlink),
        _ => Err(GitObservationError::UnsupportedTreeMode(
            String::from_utf8_lossy(bytes).into_owned(),
        )),
    }
}

fn validate_tree_name(name: &[u8]) -> Result<(), GitObservationError> {
    if name.is_empty() || name == b"." || name == b".." || name.contains(&b'/') || name.contains(&0)
    {
        return Err(GitObservationError::InvalidTreeName);
    }
    Ok(())
}

fn parse_object_id_bytes(bytes: &[u8]) -> Result<GitObjectId, GitObservationError> {
    let value = std::str::from_utf8(bytes).map_err(|_| GitObservationError::InvalidCommit)?;
    GitObjectId::parse(value).map_err(|_| GitObservationError::InvalidCommit)
}

fn object_id_from_raw(raw: &[u8]) -> Result<GitObjectId, GitObservationError> {
    let array: [u8; 20] = raw
        .try_into()
        .map_err(|_| GitObservationError::InvalidTree)?;
    let mut hex = String::with_capacity(40);
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    for byte in array {
        hex.push(char::from(DIGITS[(byte >> 4) as usize]));
        hex.push(char::from(DIGITS[(byte & 0x0f) as usize]));
    }
    GitObjectId::parse(&hex).map_err(|_| GitObservationError::InvalidTree)
}

#[derive(Debug)]
pub enum GitObservationError {
    InvalidBounds,
    InvalidInternalPath,
    Core(CoreError),
    Git(GitReadError),
    Pack(GitPackError),
    Resolution(LocalFsResolutionError),
    LocalRead(LocalFileReadError),
    DirectorySnapshot(LocalDirectorySnapshotError),
    OperationBudget(GitOperationBudgetError),
    Io(io::Error),
    UnexpectedObjectStoreKind(ObservedFileKind),
    AlternateObjectStoreUnsupported,
    MultiPackIndexUnsupported,
    NonUnicodePackName,
    InvalidPackFileName,
    PackFileLimitExceeded,
    PackIndexPairMismatch,
    PackNameChecksumMismatch,
    PackIndexByteLimitExceeded,
    PackByteLimitExceeded,
    PackDirectoryChangedDuringObservation,
    MissingInternalFile(String),
    DuplicatePackedObject(GitObjectId),
    MissingObject(GitObjectId),
    ObjectSizeOverflow,
    ObservationBindingMismatch,
    UnexpectedObjectKind {
        expected: GitObjectKind,
        observed: GitObjectKind,
    },
    InvalidCommit,
    CommitHeaderLimitExceeded,
    CommitParentLimitExceeded,
    InvalidTree,
    TreeEntryLimitExceeded,
    TreeDepthExceeded,
    TreeNameLimitExceeded,
    InvalidTreeName,
    NonUnicodeTreeName,
    UnsupportedTreeMode(String),
    DuplicateTreePath,
}

impl fmt::Display for GitObservationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBounds => {
                f.write_str("Git observation bounds exceed the first-profile limits")
            }
            Self::InvalidInternalPath => {
                f.write_str("Git observation constructed an invalid internal path")
            }
            Self::Core(error) => write!(f, "Git observation canonical binding failed: {error}"),
            Self::Git(error) => write!(f, "bounded Git repository read failed: {error}"),
            Self::Pack(error) => write!(f, "bounded Git pack read failed: {error}"),
            Self::Resolution(error) => write!(f, "Git observation path resolution failed: {error}"),
            Self::LocalRead(error) => {
                write!(f, "Git observation bounded file read failed: {error}")
            }
            Self::DirectorySnapshot(error) => {
                write!(f, "Git observation directory snapshot failed: {error}")
            }
            Self::OperationBudget(error) => {
                write!(f, "Git observation operation budget failed: {error}")
            }
            Self::Io(error) => write!(f, "Git observation filesystem I/O failed: {error}"),
            Self::UnexpectedObjectStoreKind(kind) => {
                write!(f, "Git object-store path has unsupported kind: {kind:?}")
            }
            Self::AlternateObjectStoreUnsupported => f.write_str(
                "Git alternate object stores are unsupported by the first observation profile",
            ),
            Self::MultiPackIndexUnsupported => {
                f.write_str("Git multi-pack-index is unsupported by the first observation profile")
            }
            Self::NonUnicodePackName => {
                f.write_str("Git pack directory contains a non-Unicode file name")
            }
            Self::InvalidPackFileName => {
                f.write_str("Git pack/index file name is not canonical pack-<sha1> form")
            }
            Self::PackFileLimitExceeded => f.write_str("Git pack file-count limit exceeded"),
            Self::PackIndexPairMismatch => {
                f.write_str("Git pack directory has unmatched .idx/.pack files")
            }
            Self::PackNameChecksumMismatch => {
                f.write_str("Git pack file name does not match the bound pack checksum")
            }
            Self::PackIndexByteLimitExceeded => {
                f.write_str("aggregate Git pack-index byte limit exceeded")
            }
            Self::PackByteLimitExceeded => f.write_str("aggregate Git pack byte limit exceeded"),
            Self::PackDirectoryChangedDuringObservation => {
                f.write_str("Git pack directory identity changed during observation")
            }
            Self::MissingInternalFile(path) => {
                write!(f, "required Git internal file is missing: {path}")
            }
            Self::DuplicatePackedObject(id) => write!(
                f,
                "Git object appears in multiple admitted pack indexes: {}",
                id.to_hex()
            ),
            Self::MissingObject(id) => write!(
                f,
                "Git object is unavailable in admitted loose/pack storage: {}",
                id.to_hex()
            ),
            Self::ObjectSizeOverflow => f.write_str(
                "Git object or storage size cannot be represented by the bounded profile",
            ),
            Self::ObservationBindingMismatch => {
                f.write_str("Git observation payload is detached from repository evidence")
            }
            Self::UnexpectedObjectKind { expected, observed } => write!(
                f,
                "Git object kind mismatch: expected {expected:?}, observed {observed:?}"
            ),
            Self::InvalidCommit => {
                f.write_str("Git commit object is malformed for the bounded parser")
            }
            Self::CommitHeaderLimitExceeded => f.write_str("Git commit header byte limit exceeded"),
            Self::CommitParentLimitExceeded => {
                f.write_str("Git commit parent-count limit exceeded")
            }
            Self::InvalidTree => f.write_str("Git tree object is malformed for the bounded parser"),
            Self::TreeEntryLimitExceeded => f.write_str("Git tree entry limit exceeded"),
            Self::TreeDepthExceeded => f.write_str("Git tree recursion depth limit exceeded"),
            Self::TreeNameLimitExceeded => f.write_str("Git tree entry name byte limit exceeded"),
            Self::InvalidTreeName => f.write_str("Git tree entry name is invalid"),
            Self::NonUnicodeTreeName => {
                f.write_str("Git tree path is non-Unicode and outside the first request profile")
            }
            Self::UnsupportedTreeMode(mode) => write!(f, "unsupported Git tree mode: {mode}"),
            Self::DuplicateTreePath => f.write_str("Git tree walk produced a duplicate path"),
        }
    }
}

impl Error for GitObservationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Core(error) => Some(error),
            Self::Git(error) => Some(error),
            Self::Pack(error) => Some(error),
            Self::Resolution(error) => Some(error),
            Self::LocalRead(error) => Some(error),
            Self::DirectorySnapshot(error) => Some(error),
            Self::OperationBudget(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CoreError> for GitObservationError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<GitReadError> for GitObservationError {
    fn from(value: GitReadError) -> Self {
        Self::Git(value)
    }
}

impl From<GitPackError> for GitObservationError {
    fn from(value: GitPackError) -> Self {
        Self::Pack(value)
    }
}

impl From<LocalFsResolutionError> for GitObservationError {
    fn from(value: LocalFsResolutionError) -> Self {
        Self::Resolution(value)
    }
}

impl From<LocalFileReadError> for GitObservationError {
    fn from(value: LocalFileReadError) -> Self {
        Self::LocalRead(value)
    }
}

impl From<LocalDirectorySnapshotError> for GitObservationError {
    fn from(value: LocalDirectorySnapshotError) -> Self {
        Self::DirectorySnapshot(value)
    }
}

impl From<GitOperationBudgetError> for GitObservationError {
    fn from(value: GitOperationBudgetError) -> Self {
        Self::OperationBudget(value)
    }
}

impl From<io::Error> for GitObservationError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use golam_core::tool_request::ResourceClassId;

    use crate::git_sha1::GitObjectSha1;

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_root() -> std::path::PathBuf {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "golam-git-observe-{}-{nanos}-{counter}",
            std::process::id()
        ))
    }

    fn object_id(kind: &str, body: &[u8]) -> GitObjectId {
        let mut canonical = format!("{kind} {}\0", body.len()).into_bytes();
        canonical.extend_from_slice(body);
        let digest = GitObjectSha1::digest(&canonical).unwrap();
        object_id_from_raw(&digest).unwrap()
    }

    fn write_loose(root: &Path, kind: &str, body: &[u8]) -> GitObjectId {
        let id = object_id(kind, body);
        let mut canonical = format!("{kind} {}\0", body.len()).into_bytes();
        canonical.extend_from_slice(body);
        let hex = id.to_hex();
        let directory = root.join(".git/objects").join(&hex[..2]);
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join(&hex[2..]), zlib_store(&canonical)).unwrap();
        id
    }

    fn fixture_repo() -> (std::path::PathBuf, GitObjectId, GitObjectId, GitObjectId) {
        let root = unique_root();
        fs::create_dir_all(root.join(".git/objects")).unwrap();
        fs::write(
            root.join(".git/config"),
            b"[extensions]\n\tobjectFormat = sha1\n",
        )
        .unwrap();

        let blob = write_loose(&root, "blob", b"hello\n");
        let mut tree = b"100644 hello.txt\0".to_vec();
        tree.extend_from_slice(&blob.bytes());
        let tree_id = write_loose(&root, "tree", &tree);
        let commit_body = format!(
            "tree {}\nauthor A <a@example.test> 1 +0000\ncommitter A <a@example.test> 1 +0000\n\ninitial\n",
            tree_id.to_hex()
        );
        let commit = write_loose(&root, "commit", commit_body.as_bytes());
        fs::write(root.join(".git/HEAD"), format!("{}\n", commit.to_hex())).unwrap();
        (root, commit, tree_id, blob)
    }

    fn zlib_store(data: &[u8]) -> Vec<u8> {
        assert!(data.len() <= u16::MAX as usize);
        let len = data.len() as u16;
        let mut output = vec![0x78, 0x01, 0x01];
        output.extend_from_slice(&len.to_le_bytes());
        output.extend_from_slice(&(!len).to_le_bytes());
        output.extend_from_slice(data);
        output.extend_from_slice(&adler32(data).to_be_bytes());
        output
    }

    fn adler32(data: &[u8]) -> u32 {
        const MOD: u32 = 65_521;
        let mut a = 1_u32;
        let mut b = 0_u32;
        for byte in data {
            a = (a + u32::from(*byte)) % MOD;
            b = (b + a) % MOD;
        }
        b << 16 | a
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
    #[test]
    fn observes_loose_head_log_tree_and_blob_without_mutation() {
        let (root, commit, tree, blob) = fixture_repo();
        let resolver = resolver(&root);
        let reader = GitObservationReader::open(
            &resolver,
            &RequestedOperationId::new("read").unwrap(),
            GitObservationBounds::default(),
            10,
        )
        .unwrap();

        assert_eq!(reader.evidence().head.object_id, commit);
        let expected_evidence = reader.evidence().clone();
        let log = reader.observe_head_log().unwrap();
        assert!(log.verify_binding().is_ok());
        assert_eq!(log.repository_evidence, expected_evidence);
        assert!(!log.truncated);
        assert_eq!(log.commits.len(), 1);
        assert!(log.commits[0].verify_binding().is_ok());
        assert_eq!(log.commits[0].repository_evidence, expected_evidence);
        assert_eq!(log.commits[0].tree, tree);
        let walk = reader.observe_head_tree().unwrap();
        assert!(walk.verify_binding().is_ok());
        assert_eq!(walk.repository_evidence, expected_evidence);
        assert!(!walk.truncated);
        assert_eq!(walk.entries.len(), 1);
        assert_eq!(walk.entries[0].path, "hello.txt");
        assert_eq!(walk.entries[0].object_id, blob);
        let blob_observation = reader.read_blob(blob).unwrap();
        assert!(blob_observation.verify_binding().is_ok());
        assert_eq!(blob_observation.repository_evidence, expected_evidence);
        assert_eq!(blob_observation.bytes, b"hello\n");
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd"
    ))]
    #[test]
    fn observation_binding_rejects_repository_substitution_and_payload_mutation() {
        let (root_a, _, _, blob_a) = fixture_repo();
        let (root_b, _, _, _) = fixture_repo();
        let resolver_a = resolver(&root_a);
        let resolver_b = resolver(&root_b);
        let reader_a = GitObservationReader::open(
            &resolver_a,
            &RequestedOperationId::new("read").unwrap(),
            GitObservationBounds::default(),
            10,
        )
        .unwrap();
        let reader_b = GitObservationReader::open(
            &resolver_b,
            &RequestedOperationId::new("read").unwrap(),
            GitObservationBounds::default(),
            10,
        )
        .unwrap();

        let mut substituted = reader_a.read_blob(blob_a).unwrap();
        substituted.repository_evidence = reader_b.evidence().clone();
        assert!(matches!(
            substituted.verify_binding(),
            Err(GitObservationError::ObservationBindingMismatch)
        ));

        let mut mutated_blob = reader_a.read_blob(blob_a).unwrap();
        mutated_blob.bytes.push(b'!');
        assert!(matches!(
            mutated_blob.verify_binding(),
            Err(GitObservationError::ObservationBindingMismatch)
        ));

        let mut mutated_log = reader_a.observe_head_log().unwrap();
        mutated_log.commits[0].message.push(b'!');
        assert!(matches!(
            mutated_log.verify_binding(),
            Err(GitObservationError::ObservationBindingMismatch)
        ));

        fs::remove_dir_all(root_a).unwrap();
        fs::remove_dir_all(root_b).unwrap();
    }

    #[test]
    fn pure_commit_and_tree_parsers_reject_malformed_or_unbounded_inputs() {
        assert!(matches!(
            parse_commit(
                b"tree 2222222222222222222222222222222222222222\n\nmissing identities",
                GitObservationBounds::default()
            ),
            Err(GitObservationError::InvalidCommit)
        ));

        let mut tree = b"100644 bad/name\0".to_vec();
        tree.extend_from_slice(&[0_u8; 20]);
        assert!(matches!(
            parse_tree(&tree, GitObservationBounds::default()),
            Err(GitObservationError::InvalidTreeName)
        ));
    }

    #[cfg(windows)]
    #[test]
    fn windows_observation_fails_closed_when_content_read_is_unqualified() {
        let (root, _, _, _) = fixture_repo();
        let resolver = resolver(&root);
        let result = GitObservationReader::open(
            &resolver,
            &RequestedOperationId::new("read").unwrap(),
            GitObservationBounds::default(),
            10,
        );
        assert!(matches!(
            result,
            Err(GitObservationError::Git(GitReadError::LocalRead(
                LocalFileReadError::UnsupportedPlatform
            )))
        ));
        fs::remove_dir_all(root).unwrap();
    }
}
