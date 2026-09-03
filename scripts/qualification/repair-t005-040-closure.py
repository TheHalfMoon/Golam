from pathlib import Path


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected exactly one match, found {count}: {old[:160]!r}")
    path.write_text(text.replace(old, new, 1))


# ---------------------------------------------------------------------------
# Bind every public Git observation to immutable repository evidence.
# ---------------------------------------------------------------------------
observe = Path("crates/golamd/src/git_observe.rs")

for struct_name, first_field in (
    ("GitObjectObservation", "    pub object: GitObject,\n"),
    ("GitBlobObservation", "    pub object_id: GitObjectId,\n"),
    ("GitCommitObservation", "    pub object_id: GitObjectId,\n"),
    ("GitTreeObjectObservation", "    pub object_id: GitObjectId,\n"),
    ("GitTreeWalkObservation", "    pub root_tree: GitObjectId,\n"),
    ("GitLogObservation", "    pub commits: Vec<GitCommitObservation>,\n"),
):
    replace_once(
        observe,
        f"pub struct {struct_name} {{\n{first_field}",
        f"pub struct {struct_name} {{\n    pub repository_evidence: GitRepositoryEvidence,\n{first_field}",
    )

replace_once(
    observe,
    """                return Ok(GitObjectObservation {
                    object,
                    source: GitObjectSource::Loose,
                });
""",
    """                return Ok(GitObjectObservation {
                    repository_evidence: self.repository.evidence().clone(),
                    object,
                    source: GitObjectSource::Loose,
                });
""",
)

replace_once(
    observe,
    """        Ok(GitObjectObservation {
            object: GitObject {
""",
    """        Ok(GitObjectObservation {
            repository_evidence: self.repository.evidence().clone(),
            object: GitObject {
""",
)

replace_once(
    observe,
    """        Ok(GitBlobObservation {
            object_id,
            source: observation.source,
            bytes: observation.object.bytes,
        })
""",
    """        Ok(GitBlobObservation {
            repository_evidence: observation.repository_evidence,
            object_id,
            source: observation.source,
            bytes: observation.object.bytes,
        })
""",
)

# Keep parsing pure: parse_commit returns a private parsed value, while the
# public observation is constructed only after repository evidence is attached.
replace_once(
    observe,
    """#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitCommitObservation {
    pub repository_evidence: GitRepositoryEvidence,
    pub object_id: GitObjectId,
    pub source: GitObjectSource,
    pub tree: GitObjectId,
    pub parents: Vec<GitObjectId>,
    pub author: Vec<u8>,
    pub committer: Vec<u8>,
    pub message: Vec<u8>,
}
""",
    """#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitCommitObservation {
    pub repository_evidence: GitRepositoryEvidence,
    pub object_id: GitObjectId,
    pub source: GitObjectSource,
    pub tree: GitObjectId,
    pub parents: Vec<GitObjectId>,
    pub author: Vec<u8>,
    pub committer: Vec<u8>,
    pub message: Vec<u8>,
}

struct ParsedGitCommit {
    tree: GitObjectId,
    parents: Vec<GitObjectId>,
    author: Vec<u8>,
    committer: Vec<u8>,
    message: Vec<u8>,
}
""",
)

replace_once(
    observe,
    """        self.deadline.run_step(|| {
            parse_commit(
                object_id,
                observation.source,
                &observation.object.bytes,
                self.bounds,
            )
        })?
""",
    """        let parsed = self
            .deadline
            .run_step(|| parse_commit(&observation.object.bytes, self.bounds))??;
        Ok(GitCommitObservation {
            repository_evidence: observation.repository_evidence,
            object_id,
            source: observation.source,
            tree: parsed.tree,
            parents: parsed.parents,
            author: parsed.author,
            committer: parsed.committer,
            message: parsed.message,
        })
""",
)

replace_once(
    observe,
    """        Ok(GitTreeObjectObservation {
            object_id,
            source: observation.source,
            entries,
        })
""",
    """        Ok(GitTreeObjectObservation {
            repository_evidence: observation.repository_evidence,
            object_id,
            source: observation.source,
            entries,
        })
""",
)

replace_once(
    observe,
    """    pub fn observe_head_log(&self) -> Result<GitLogObservation, GitObservationError> {
        let mut pending = VecDeque::new();
""",
    """    pub fn observe_head_log(&self) -> Result<GitLogObservation, GitObservationError> {
        let repository_evidence = self.repository.evidence().clone();
        let mut pending = VecDeque::new();
""",
)

replace_once(
    observe,
    """                return Ok(GitLogObservation {
                    commits,
                    truncated: true,
                });
""",
    """                return Ok(GitLogObservation {
                    repository_evidence: repository_evidence.clone(),
                    commits,
                    truncated: true,
                });
""",
)

replace_once(
    observe,
    """        Ok(GitLogObservation {
            commits,
            truncated: false,
        })
""",
    """        Ok(GitLogObservation {
            repository_evidence,
            commits,
            truncated: false,
        })
""",
)

replace_once(
    observe,
    """    pub fn walk_tree(
        &self,
        root_tree: GitObjectId,
    ) -> Result<GitTreeWalkObservation, GitObservationError> {
        let mut pending = VecDeque::new();
""",
    """    pub fn walk_tree(
        &self,
        root_tree: GitObjectId,
    ) -> Result<GitTreeWalkObservation, GitObservationError> {
        let repository_evidence = self.repository.evidence().clone();
        let mut pending = VecDeque::new();
""",
)

replace_once(
    observe,
    """                    return Ok(GitTreeWalkObservation {
                        root_tree,
                        entries: paths.into_values().collect(),
                        truncated: true,
                    });
""",
    """                    return Ok(GitTreeWalkObservation {
                        repository_evidence: repository_evidence.clone(),
                        root_tree,
                        entries: paths.into_values().collect(),
                        truncated: true,
                    });
""",
)

replace_once(
    observe,
    """        Ok(GitTreeWalkObservation {
            root_tree,
            entries: paths.into_values().collect(),
            truncated: false,
        })
""",
    """        Ok(GitTreeWalkObservation {
            repository_evidence,
            root_tree,
            entries: paths.into_values().collect(),
            truncated: false,
        })
""",
)

replace_once(
    observe,
    """fn parse_commit(
    object_id: GitObjectId,
    source: GitObjectSource,
    bytes: &[u8],
    bounds: GitObservationBounds,
) -> Result<GitCommitObservation, GitObservationError> {
""",
    """fn parse_commit(
    bytes: &[u8],
    bounds: GitObservationBounds,
) -> Result<ParsedGitCommit, GitObservationError> {
""",
)

replace_once(
    observe,
    """    Ok(GitCommitObservation {
        object_id,
        source,
        tree: tree.ok_or(GitObservationError::InvalidCommit)?,
        parents,
        author: author.ok_or(GitObservationError::InvalidCommit)?,
        committer: committer.ok_or(GitObservationError::InvalidCommit)?,
        message,
    })
""",
    """    Ok(ParsedGitCommit {
        tree: tree.ok_or(GitObservationError::InvalidCommit)?,
        parents,
        author: author.ok_or(GitObservationError::InvalidCommit)?,
        committer: committer.ok_or(GitObservationError::InvalidCommit)?,
        message,
    })
""",
)

replace_once(
    observe,
    """            parse_commit(
                id,
                GitObjectSource::Loose,
                b"tree 2222222222222222222222222222222222222222\n\nmissing identities",
                GitObservationBounds::default()
            ),
""",
    """            parse_commit(
                b"tree 2222222222222222222222222222222222222222\n\nmissing identities",
                GitObservationBounds::default()
            ),
""",
)

replace_once(
    observe,
    """        assert_eq!(reader.evidence().head.object_id, commit);
        let log = reader.observe_head_log().unwrap();
        assert!(!log.truncated);
        assert_eq!(log.commits.len(), 1);
        assert_eq!(log.commits[0].tree, tree);
        let walk = reader.observe_head_tree().unwrap();
        assert!(!walk.truncated);
        assert_eq!(walk.entries.len(), 1);
        assert_eq!(walk.entries[0].path, "hello.txt");
        assert_eq!(walk.entries[0].object_id, blob);
        assert_eq!(reader.read_blob(blob).unwrap().bytes, b"hello\n");
""",
    """        assert_eq!(reader.evidence().head.object_id, commit);
        let expected_evidence = reader.evidence().clone();
        let log = reader.observe_head_log().unwrap();
        assert_eq!(log.repository_evidence, expected_evidence);
        assert!(!log.truncated);
        assert_eq!(log.commits.len(), 1);
        assert_eq!(log.commits[0].repository_evidence, expected_evidence);
        assert_eq!(log.commits[0].tree, tree);
        let walk = reader.observe_head_tree().unwrap();
        assert_eq!(walk.repository_evidence, expected_evidence);
        assert!(!walk.truncated);
        assert_eq!(walk.entries.len(), 1);
        assert_eq!(walk.entries[0].path, "hello.txt");
        assert_eq!(walk.entries[0].object_id, blob);
        let blob_observation = reader.read_blob(blob).unwrap();
        assert_eq!(blob_observation.repository_evidence, expected_evidence);
        assert_eq!(blob_observation.bytes, b"hello\n");
""",
)


# ---------------------------------------------------------------------------
# Make status/worktree observation use the same operation deadline and the
# retained-handle directory primitive, and bind status to repository evidence.
# ---------------------------------------------------------------------------
status = Path("crates/golamd/src/git_status.rs")
replace_once(status, "use std::fs;\n", "")
replace_once(status, "use std::path::Path;\n", "")
replace_once(status, "use std::time::{Duration, Instant};\n", "use std::time::Duration;\n")
replace_once(
    status,
    """use crate::git_read::{GitObjectId, GitReadError};
""",
    """use crate::git_read::{GitObjectId, GitReadError, GitRepositoryEvidence};
use crate::git_read_budget::{GitOperationBudgetError, GitOperationDeadline};
""",
)
replace_once(
    status,
    """use crate::local_fs::{LocalFsResolutionError, LocalFsResolver};
""",
    """use crate::local_dir::{
    LocalDirectorySnapshotBounds, LocalDirectorySnapshotError, snapshot_directory,
};
use crate::local_fs::{LocalFsResolutionError, LocalFsResolver};
""",
)
replace_once(
    status,
    """pub struct GitStatusObservation {
    pub head: GitObjectId,
""",
    """pub struct GitStatusObservation {
    pub repository_evidence: GitRepositoryEvidence,
    pub head: GitObjectId,
""",
)

replace_once(
    status,
    """    bounds.validate()?;
    let started = Instant::now();
    let observation =
        GitObservationReader::open(resolver, operation, bounds.observation, observed_at_unix_ms)?;
    require_time(started, bounds.max_duration)?;

    let index_bytes = read_file(
        resolver,
        operation,
        ".git/index",
        u64::try_from(bounds.index.max_bytes).map_err(|_| GitStatusError::SizeOverflow)?,
        bounds.max_duration,
        observed_at_unix_ms,
    )?;
    let index = parse_git_index(&index_bytes, bounds.index)?;
""",
    """    bounds.validate()?;
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
""",
)

replace_once(
    status,
    """    let grouped = group_index_entries(&index.entries, bounds.max_entries)?;
    let stage_zero = stage_zero_entries(&grouped);
    let staged = staged_diff(&head, &grouped, bounds.max_entries)?;
    require_time(started, bounds.max_duration)?;
""",
    """    let grouped = deadline.run_step(|| group_index_entries(&index.entries, bounds.max_entries))??;
    let stage_zero = deadline.run_step(|| stage_zero_entries(&grouped))?;
    let staged = deadline.run_step(|| staged_diff(&head, &grouped, bounds.max_entries))??;
    deadline.require_active()?;
""",
)

replace_once(status, "        started,\n    )?;\n\n    Ok(GitStatusObservation {\n", "        deadline,\n    )?;\n\n    Ok(GitStatusObservation {\n        repository_evidence: observation.evidence().clone(),\n")

replace_once(
    status,
    """    observed_at_unix_ms: u64,
    started: Instant,
) -> Result<(Vec<GitDiffEvidence>, Vec<String>), GitStatusError> {
""",
    """    observed_at_unix_ms: u64,
    deadline: GitOperationDeadline,
) -> Result<(Vec<GitDiffEvidence>, Vec<String>), GitStatusError> {
""",
)
replace_once(status, "        require_time(started, bounds.max_duration)?;\n", "        deadline.require_active()?;\n")
replace_once(
    status,
    """        let identity = resolver.resolve_read_target(&requested, operation, observed_at_unix_ms)?;
""",
    """        let identity = deadline.run_step(|| {
            resolver.resolve_read_target(&requested, operation, observed_at_unix_ms)
        })??;
""",
)
replace_once(status, "                        max_duration: bounds.max_duration,\n", "                        max_duration: deadline.remaining()?,\n")
replace_once(status, "                let worktree_id = blob_id(&read.bytes)?;\n", "                let worktree_id = deadline.run_step(|| blob_id(&read.bytes))??;\n")
replace_once(status, "        started,\n    )?;\n    Ok((output, untracked))\n", "        deadline,\n    )?;\n    Ok((output, untracked))\n")

replace_once(
    status,
    """    observed_at_unix_ms: u64,
    started: Instant,
) -> Result<Vec<String>, GitStatusError> {
""",
    """    observed_at_unix_ms: u64,
    deadline: GitOperationDeadline,
) -> Result<Vec<String>, GitStatusError> {
""",
)
replace_once(
    status,
    """    let root_identity = resolver.resolve_read_target(&root, operation, observed_at_unix_ms)?;
""",
    """    let root_identity = deadline.run_step(|| {
        resolver.resolve_read_target(&root, operation, observed_at_unix_ms)
    })??;
""",
)
replace_once(status, "        require_time(started, bounds.max_duration)?;\n", "        deadline.require_active()?;\n")

old_directory_block = """        let current =
            resolver.resolve_read_target(&requested_dir, operation, observed_at_unix_ms)?;
        if current.file_kind != ObservedFileKind::Directory
            || current.resolved_target_identity != expected.resolved_target_identity
            || current.observed_metadata_digest != expected.observed_metadata_digest
        {
            return Err(GitStatusError::RepositoryRootChanged);
        }

        let mut names = Vec::new();
        for entry in fs::read_dir(Path::new(current.normalized_path.as_str()))? {
            require_time(started, bounds.max_duration)?;
            observed_entries = observed_entries
                .checked_add(1)
                .ok_or(GitStatusError::EntryLimitExceeded)?;
            if observed_entries > bounds.max_entries {
                return Err(GitStatusError::EntryLimitExceeded);
            }
            let entry = entry?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| GitStatusError::NonUnicodeWorktreePath)?;
            names.push(name);
        }
        names.sort_unstable();

        for name in names {
"""
new_directory_block = """        let remaining_entries = bounds
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
"""
replace_once(status, old_directory_block, new_directory_block)

replace_once(
    status,
    """            let identity =
                resolver.resolve_read_target(&requested, operation, observed_at_unix_ms)?;
""",
    """            let identity = deadline.run_step(|| {
                resolver.resolve_read_target(&requested, operation, observed_at_unix_ms)
            })??;
""",
)

replace_once(
    status,
    """    max_bytes: u64,
    max_duration: Duration,
    observed_at_unix_ms: u64,
) -> Result<Vec<u8>, GitStatusError> {
""",
    """    max_bytes: u64,
    deadline: GitOperationDeadline,
    observed_at_unix_ms: u64,
) -> Result<Vec<u8>, GitStatusError> {
    deadline.require_active()?;
""",
)
replace_once(status, "            max_duration,\n", "            max_duration: deadline.remaining()?,\n")

# Remove obsolete private clock helper; the public error variant may remain for
# backward-compatible diagnostics, but no implementation path uses a second clock.
replace_once(
    status,
    """fn require_time(started: Instant, max_duration: Duration) -> Result<(), GitStatusError> {
    if started.elapsed() > max_duration {
        return Err(GitStatusError::DurationLimitExceeded);
    }
    Ok(())
}

""",
    "",
)

replace_once(
    status,
    """    LocalRead(LocalFileReadError),
    Sha1(GitObjectSha1Error),
""",
    """    LocalRead(LocalFileReadError),
    DirectorySnapshot(LocalDirectorySnapshotError),
    OperationBudget(GitOperationBudgetError),
    Sha1(GitObjectSha1Error),
""",
)
replace_once(
    status,
    """            Self::LocalRead(error) => write!(f, "Git status bounded file read failed: {error}"),
            Self::Sha1(error) => write!(f, "Git status SHA-1 failed: {error}"),
""",
    """            Self::LocalRead(error) => write!(f, "Git status bounded file read failed: {error}"),
            Self::DirectorySnapshot(error) => {
                write!(f, "Git status directory snapshot failed: {error}")
            }
            Self::OperationBudget(error) => {
                write!(f, "Git status operation budget failed: {error}")
            }
            Self::Sha1(error) => write!(f, "Git status SHA-1 failed: {error}"),
""",
)
replace_once(
    status,
    """            Self::LocalRead(error) => Some(error),
            Self::Sha1(error) => Some(error),
""",
    """            Self::LocalRead(error) => Some(error),
            Self::DirectorySnapshot(error) => Some(error),
            Self::OperationBudget(error) => Some(error),
            Self::Sha1(error) => Some(error),
""",
)
replace_once(
    status,
    """impl From<GitObjectSha1Error> for GitStatusError {
""",
    """impl From<LocalDirectorySnapshotError> for GitStatusError {
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
""",
)


# ---------------------------------------------------------------------------
# Add admitted REF_DELTA and shared-deadline qualification fixtures.
# ---------------------------------------------------------------------------
pack = Path("crates/golamd/src/git_pack.rs")
replace_once(
    pack,
    """    #[test]
    fn rejects_index_checksum_fanout_and_pack_checksum_corruption() {
""",
    """    #[test]
    fn resolves_bounded_in_pack_ref_delta_and_verifies_reconstructed_identity() {
        let fixture = PackFixture::base_and_ref_delta(b"hello", b"hello!");
        let index = parse_pack_index_v2(&fixture.index, GitPackBounds::default()).unwrap();
        let object = read_packed_object(
            &fixture.pack,
            &index,
            fixture.object_ids[1],
            GitPackBounds::default(),
        )
        .unwrap();
        assert_eq!(object.kind, PackedObjectKind::Blob);
        assert_eq!(object.bytes, b"hello!");
    }

    #[test]
    fn shared_operation_deadline_covers_index_parse_and_ref_delta_read() {
        let fixture = PackFixture::base_and_ref_delta(b"hello", b"hello!");
        let deadline = GitOperationDeadline::start(Duration::from_millis(20)).unwrap();
        let index = deadline
            .run_step(|| parse_pack_index_v2(&fixture.index, GitPackBounds::default()))
            .unwrap()
            .unwrap();
        std::thread::sleep(Duration::from_millis(30));
        assert!(matches!(
            read_packed_object_with_deadline(
                &fixture.pack,
                &index,
                fixture.object_ids[1],
                GitPackBounds::default(),
                deadline,
            ),
            Err(GitPackError::OperationBudget(
                GitOperationBudgetError::DeadlineExceeded
            ))
        ));
    }

    #[test]
    fn rejects_index_checksum_fanout_and_pack_checksum_corruption() {
""",
)

replace_once(
    pack,
    """        fn thin_ref_delta() -> Self {
""",
    """        fn base_and_ref_delta(base: &[u8], target: &[u8]) -> Self {
            assert_eq!(target, b"hello!");
            assert_eq!(base, b"hello");
            let mut pack = pack_header(2);

            let base_offset = pack.len() as u32;
            let mut base_entry = encode_pack_header(3, base.len());
            base_entry.extend_from_slice(&zlib_store(base));
            let base_crc = crc32(&base_entry);
            pack.extend_from_slice(&base_entry);
            let base_id = canonical_object_id(PackedObjectKind::Blob, base).unwrap();

            let delta_offset = pack.len() as u32;
            let delta = [
                base.len() as u8,
                target.len() as u8,
                0x90,
                base.len() as u8,
                1,
                b'!',
            ];
            let mut delta_entry = encode_pack_header(7, delta.len());
            delta_entry.extend_from_slice(&base_id.bytes());
            delta_entry.extend_from_slice(&zlib_store(&delta));
            let delta_crc = crc32(&delta_entry);
            pack.extend_from_slice(&delta_entry);

            let target_id = canonical_object_id(PackedObjectKind::Blob, target).unwrap();
            let pack_checksum = GitObjectSha1::digest(&pack).unwrap();
            pack.extend_from_slice(&pack_checksum);
            let index = build_index(
                &[
                    Record {
                        id: base_id,
                        offset: base_offset,
                        crc: base_crc,
                    },
                    Record {
                        id: target_id,
                        offset: delta_offset,
                        crc: delta_crc,
                    },
                ],
                pack_checksum,
            );
            Self {
                pack,
                index,
                object_ids: vec![base_id, target_id],
            }
        }

        fn thin_ref_delta() -> Self {
""",
)


# ---------------------------------------------------------------------------
# Add context-specific retained-handle race fixtures for pack/worktree use.
# ---------------------------------------------------------------------------
local_dir = Path("crates/golamd/src/local_dir.rs")
replace_once(
    local_dir,
    """    #[cfg(windows)]
    #[test]
    fn windows_directory_snapshot_fails_closed_until_handle_enumeration_is_admitted() {
""",
    """    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd"
    ))]
    fn assert_context_directory_replacement_fails_closed(relative: &str) {
        use std::os::unix::fs::symlink;

        let root = unique_root();
        let outside = unique_root();
        let target = root.join(relative);
        fs::create_dir_all(&target).unwrap();
        fs::create_dir(&outside).unwrap();
        fs::write(target.join("inside.txt"), b"inside").unwrap();
        fs::write(outside.join("outside.txt"), b"outside").unwrap();
        let resolver = resolver(&root);
        let original = target.with_file_name(format!(
            "{}-original",
            target.file_name().unwrap().to_string_lossy()
        ));

        let result = snapshot_directory_with_pre_enumeration(
            &resolver,
            &RequestedTarget::new(relative).unwrap(),
            &RequestedOperationId::new("list").unwrap(),
            bounds(),
            10,
            || {
                fs::rename(&target, &original).unwrap();
                symlink(&outside, &target).unwrap();
            },
        );

        assert!(result.is_err());
        fs::remove_file(&target).unwrap();
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd"
    ))]
    #[test]
    fn pack_directory_replacement_fails_closed_before_enumeration_returns() {
        assert_context_directory_replacement_fails_closed(".git/objects/pack");
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd"
    ))]
    #[test]
    fn worktree_directory_replacement_fails_closed_before_enumeration_returns() {
        assert_context_directory_replacement_fails_closed("nested/worktree");
    }

    #[cfg(windows)]
    #[test]
    fn windows_directory_snapshot_fails_closed_until_handle_enumeration_is_admitted() {
""",
)

print("T005-040 closure repair staged")
