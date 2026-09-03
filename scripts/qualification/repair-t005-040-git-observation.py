from pathlib import Path


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected exactly one match, found {count}: {old[:120]!r}")
    path.write_text(text.replace(old, new, 1))


main = Path("crates/golamd/src/main.rs")
replace_once(main, "pub mod local_fs;\n", "pub mod local_dir;\npub mod local_fs;\n")

read = Path("crates/golamd/src/git_read.rs")
replace_once(
    read,
    '''    pub fn open(
        resolver: &'a LocalFsResolver,
        operation: &RequestedOperationId,
        bounds: GitReadBounds,
        observed_at_unix_ms: u64,
    ) -> Result<Self, GitReadError> {
        bounds.validate()?;
        let deadline = GitOperationDeadline::start(bounds.max_duration)?;
        let repository_root =
''',
    '''    pub fn open(
        resolver: &'a LocalFsResolver,
        operation: &RequestedOperationId,
        bounds: GitReadBounds,
        observed_at_unix_ms: u64,
    ) -> Result<Self, GitReadError> {
        bounds.validate()?;
        let deadline = GitOperationDeadline::start(bounds.max_duration)?;
        Self::open_with_deadline(resolver, operation, bounds, deadline, observed_at_unix_ms)
    }

    pub(crate) fn open_with_deadline(
        resolver: &'a LocalFsResolver,
        operation: &RequestedOperationId,
        bounds: GitReadBounds,
        deadline: GitOperationDeadline,
        observed_at_unix_ms: u64,
    ) -> Result<Self, GitReadError> {
        bounds.validate()?;
        deadline.require_active()?;
        let repository_root =
''',
)
replace_once(
    read,
    '''    pub fn evidence(&self) -> &GitRepositoryEvidence {
        &self.evidence
    }

    pub fn resolve_ref(
''',
    '''    pub fn evidence(&self) -> &GitRepositoryEvidence {
        &self.evidence
    }

    pub(crate) const fn operation_deadline(&self) -> GitOperationDeadline {
        self.deadline
    }

    pub fn resolve_ref(
''',
)

pack = Path("crates/golamd/src/git_pack.rs")
replace_once(
    pack,
    '''use crate::git_read_budget::{
    DECOMPRESSION_INPUT_QUANTUM_BYTES, DECOMPRESSION_OUTPUT_QUANTUM_BYTES,
    DecompressionBudgetError, DecompressionDeadline,
};
''',
    '''use crate::git_read_budget::{
    DECOMPRESSION_INPUT_QUANTUM_BYTES, DECOMPRESSION_OUTPUT_QUANTUM_BYTES,
    DecompressionBudgetError, DecompressionDeadline, GitOperationBudgetError,
    GitOperationDeadline,
};
''',
)
replace_once(
    pack,
    '''pub fn read_packed_object(
    pack: &[u8],
    index: &GitPackIndex,
    wanted: PackObjectId,
    bounds: GitPackBounds,
) -> Result<PackedGitObject, GitPackError> {
    bounds.validate()?;
    validate_pack(pack, index, bounds)?;

    let wanted_index = index
        .entries
        .binary_search_by_key(&wanted, |entry| entry.object_id)
        .map_err(|_| GitPackError::MissingPackedObject(wanted))?;
    let lookup = PackLookup::new(pack, index)?;
    let mut active_offsets = HashSet::new();
    resolve_entry(
        pack,
        index,
        &lookup,
        wanted_index,
        bounds,
        0,
        &mut active_offsets,
    )
}
''',
    '''pub fn read_packed_object(
    pack: &[u8],
    index: &GitPackIndex,
    wanted: PackObjectId,
    bounds: GitPackBounds,
) -> Result<PackedGitObject, GitPackError> {
    bounds.validate()?;
    let deadline = GitOperationDeadline::start(bounds.max_duration)?;
    read_packed_object_with_deadline(pack, index, wanted, bounds, deadline)
}

pub(crate) fn read_packed_object_with_deadline(
    pack: &[u8],
    index: &GitPackIndex,
    wanted: PackObjectId,
    bounds: GitPackBounds,
    deadline: GitOperationDeadline,
) -> Result<PackedGitObject, GitPackError> {
    bounds.validate()?;
    deadline.require_active()?;
    deadline.run_step(|| validate_pack(pack, index, bounds))??;

    let wanted_index = deadline
        .run_step(|| {
            index
                .entries
                .binary_search_by_key(&wanted, |entry| entry.object_id)
        })?
        .map_err(|_| GitPackError::MissingPackedObject(wanted))?;
    let lookup = deadline.run_step(|| PackLookup::new(pack, index))??;
    let mut active_offsets = HashSet::new();
    resolve_entry(
        pack,
        index,
        &lookup,
        wanted_index,
        bounds,
        deadline,
        0,
        &mut active_offsets,
    )
}
''',
)
replace_once(
    pack,
    '''    bounds: GitPackBounds,
    depth: usize,
    active_offsets: &mut HashSet<u64>,
) -> Result<PackedGitObject, GitPackError> {
    if depth >= bounds.max_delta_depth {
''',
    '''    bounds: GitPackBounds,
    deadline: GitOperationDeadline,
    depth: usize,
    active_offsets: &mut HashSet<u64>,
) -> Result<PackedGitObject, GitPackError> {
    deadline.require_active()?;
    if depth >= bounds.max_delta_depth {
''',
)
# Both recursive delta branches have the same argument sequence.
text = pack.read_text()
old = '''                    bounds,
                    depth + 1,
                    active_offsets,
'''
if text.count(old) != 2:
    raise SystemExit(f"{pack}: expected two recursive resolve_entry calls, found {text.count(old)}")
text = text.replace(old, '''                    bounds,
                    deadline,
                    depth + 1,
                    active_offsets,
''')
pack.write_text(text)
text = pack.read_text()
old = '''                    bounds.max_object_bytes,
                    bounds.max_duration,
                )?;'''
if text.count(old) != 3:
    raise SystemExit(f"{pack}: expected three inflate call tails, found {text.count(old)}")
text = text.replace(old, '''                    bounds.max_object_bytes,
                    deadline,
                )?;''')
pack.write_text(text)
replace_once(
    pack,
    '''                let body = apply_delta(&base.bytes, &delta, bounds.max_object_bytes)?;
                (base.kind, body)
''',
    '''                let body = deadline
                    .run_step(|| apply_delta(&base.bytes, &delta, bounds.max_object_bytes))??;
                (base.kind, body)
''',
)
# The REF_DELTA branch has the same body after the first replacement; patch the remaining copy.
replace_once(
    pack,
    '''                let body = apply_delta(&base.bytes, &delta, bounds.max_object_bytes)?;
                (base.kind, body)
''',
    '''                let body = deadline
                    .run_step(|| apply_delta(&base.bytes, &delta, bounds.max_object_bytes))??;
                (base.kind, body)
''',
)
replace_once(
    pack,
    '''        let actual_id = canonical_object_id(kind, &body)?;
''',
    '''        let actual_id = deadline.run_step(|| canonical_object_id(kind, &body))??;
''',
)
replace_once(
    pack,
    '''fn inflate_one_zlib(
    compressed: &[u8],
    max_output_bytes: usize,
    max_duration: Duration,
) -> Result<(Vec<u8>, usize), GitPackError> {
    let deadline = DecompressionDeadline::start(max_duration)?;
''',
    '''fn inflate_one_zlib(
    compressed: &[u8],
    max_output_bytes: usize,
    operation_deadline: GitOperationDeadline,
) -> Result<(Vec<u8>, usize), GitPackError> {
    let deadline = DecompressionDeadline::from_operation(operation_deadline);
''',
)
replace_once(
    pack,
    '''    DeltaCycle,
    Decompression(DecompressionBudgetError),
''',
    '''    DeltaCycle,
    OperationBudget(GitOperationBudgetError),
    Decompression(DecompressionBudgetError),
''',
)
replace_once(
    pack,
    '''            Self::DeltaCycle => f.write_str("Git delta cycle detected"),
            Self::Decompression(error) => {
''',
    '''            Self::DeltaCycle => f.write_str("Git delta cycle detected"),
            Self::OperationBudget(error) => {
                write!(f, "bounded Git pack operation budget failed: {error}")
            }
            Self::Decompression(error) => {
''',
)
replace_once(
    pack,
    '''        match self {
            Self::Decompression(error) => Some(error),
''',
    '''        match self {
            Self::OperationBudget(error) => Some(error),
            Self::Decompression(error) => Some(error),
''',
)
replace_once(
    pack,
    '''impl From<DecompressionBudgetError> for GitPackError {
''',
    '''impl From<GitOperationBudgetError> for GitPackError {
    fn from(value: GitOperationBudgetError) -> Self {
        Self::OperationBudget(value)
    }
}

impl From<DecompressionBudgetError> for GitPackError {
''',
)

observe = Path("crates/golamd/src/git_observe.rs")
replace_once(
    observe,
    '''    GitPackBounds, GitPackError, GitPackIndex, PackObjectId, PackedObjectKind, parse_pack_index_v2,
    read_packed_object,
''',
    '''    GitPackBounds, GitPackError, GitPackIndex, PackObjectId, PackedObjectKind, parse_pack_index_v2,
    read_packed_object_with_deadline,
''',
)
replace_once(
    observe,
    '''use crate::git_read::{
    GitObject, GitObjectId, GitObjectKind, GitReadBounds, GitReadError, GitRepositoryEvidence,
    GitRepositoryReader,
};
use crate::local_fs::{LocalFsResolutionError, LocalFsResolver};
''',
    '''use crate::git_read::{
    GitObject, GitObjectId, GitObjectKind, GitReadBounds, GitReadError, GitRepositoryEvidence,
    GitRepositoryReader,
};
use crate::git_read_budget::{GitOperationBudgetError, GitOperationDeadline};
use crate::local_dir::{
    LocalDirectorySnapshotBounds, LocalDirectorySnapshotError, snapshot_directory,
};
use crate::local_fs::{LocalFsResolutionError, LocalFsResolver};
''',
)
replace_once(
    observe,
    '''pub const MAX_PACK_FILES: usize = 16;
''',
    '''pub const MAX_PACK_FILES: usize = 16;
const MAX_PACK_DIRECTORY_ENTRIES: usize = MAX_PACK_FILES * 4 + 16;
''',
)
replace_once(
    observe,
    '''pub struct GitObservationReader<'a> {
    repository: GitRepositoryReader<'a>,
    packs: Vec<LoadedPack>,
    bounds: GitObservationBounds,
    observed_at_unix_ms: u64,
}
''',
    '''pub struct GitObservationReader<'a> {
    repository: GitRepositoryReader<'a>,
    packs: Vec<LoadedPack>,
    bounds: GitObservationBounds,
    deadline: GitOperationDeadline,
    observed_at_unix_ms: u64,
}
''',
)
replace_once(
    observe,
    '''    ) -> Result<Self, GitObservationError> {
        bounds.validate()?;
        let repository =
            GitRepositoryReader::open(resolver, operation, bounds.git, observed_at_unix_ms)?;
        reject_alternate_object_store(resolver, operation, observed_at_unix_ms)?;
        let packs = load_packs(resolver, operation, bounds, observed_at_unix_ms)?;
        Ok(Self {
            repository,
            packs,
            bounds,
            observed_at_unix_ms,
        })
    }

    pub fn evidence(&self) -> &GitRepositoryEvidence {
''',
    '''    ) -> Result<Self, GitObservationError> {
        bounds.validate()?;
        let deadline = GitOperationDeadline::start(bounds.git.max_duration)?;
        Self::open_with_deadline(
            resolver,
            operation,
            bounds,
            deadline,
            observed_at_unix_ms,
        )
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
''',
)
replace_once(
    observe,
    '''        let packed = read_packed_object(&pack.bytes, &pack.index, wanted, self.bounds.pack)?;
''',
    '''        let packed = read_packed_object_with_deadline(
            &pack.bytes,
            &pack.index,
            wanted,
            self.bounds.pack,
            self.deadline,
        )?;
''',
)
replace_once(
    observe,
    '''        parse_commit(
            object_id,
            observation.source,
            &observation.object.bytes,
            self.bounds,
        )
''',
    '''        self.deadline.run_step(|| {
            parse_commit(
                object_id,
                observation.source,
                &observation.object.bytes,
                self.bounds,
            )
        })?
''',
)
replace_once(
    observe,
    '''        let entries = parse_tree(&observation.object.bytes, self.bounds)?;
''',
    '''        let entries = self
            .deadline
            .run_step(|| parse_tree(&observation.object.bytes, self.bounds))??;
''',
)
replace_once(
    observe,
    '''        while let Some(object_id) = pending.pop_front() {
            if !seen.insert(object_id) {
''',
    '''        while let Some(object_id) = pending.pop_front() {
            self.deadline.require_active()?;
            if !seen.insert(object_id) {
''',
)
replace_once(
    observe,
    '''        while let Some((prefix, tree_id, depth)) = pending.pop_front() {
            if depth > self.bounds.max_tree_depth {
''',
    '''        while let Some((prefix, tree_id, depth)) = pending.pop_front() {
            self.deadline.require_active()?;
            if depth > self.bounds.max_tree_depth {
''',
)
replace_once(
    observe,
    '''fn reject_alternate_object_store(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    observed_at_unix_ms: u64,
) -> Result<(), GitObservationError> {
''',
    '''fn reject_alternate_object_store(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    deadline: GitOperationDeadline,
    observed_at_unix_ms: u64,
) -> Result<(), GitObservationError> {
    deadline.require_active()?;
''',
)
replace_once(
    observe,
    '''    let alternates = requested(".git/objects/info/alternates")?;
''',
    '''    deadline.require_active()?;
    let alternates = requested(".git/objects/info/alternates")?;
''',
)
replace_once(
    observe,
    '''fn load_packs(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    bounds: GitObservationBounds,
    observed_at_unix_ms: u64,
) -> Result<Vec<LoadedPack>, GitObservationError> {
''',
    '''fn load_packs(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    bounds: GitObservationBounds,
    deadline: GitOperationDeadline,
    observed_at_unix_ms: u64,
) -> Result<Vec<LoadedPack>, GitObservationError> {
    deadline.require_active()?;
''',
)
replace_once(
    observe,
    '''    let absolute = Path::new(initial.normalized_path.as_str());
    let mut index_names = Vec::new();
    let mut pack_names = HashSet::new();
    for entry in fs::read_dir(absolute)? {
        let entry = entry?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| GitObservationError::NonUnicodePackName)?;
''',
    '''    let snapshot = snapshot_directory(
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
''',
)
# File reads use the shared remaining duration.
text = observe.read_text()
old = '''            bounds.pack.max_duration,
            observed_at_unix_ms,
        )?;'''
if text.count(old) != 2:
    raise SystemExit(f"{observe}: expected two pack file read budgets, found {text.count(old)}")
text = text.replace(old, '''            deadline,
            observed_at_unix_ms,
        )?;''')
observe.write_text(text)
replace_once(
    observe,
    '''        let index = parse_pack_index_v2(&index_bytes, bounds.pack)?;
''',
    '''        let index = deadline.run_step(|| parse_pack_index_v2(&index_bytes, bounds.pack))??;
''',
)
replace_once(
    observe,
    '''fn read_required_file(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    path: &str,
    max_bytes: u64,
    max_duration: Duration,
    observed_at_unix_ms: u64,
) -> Result<Vec<u8>, GitObservationError> {
''',
    '''fn read_required_file(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    path: &str,
    max_bytes: u64,
    deadline: GitOperationDeadline,
    observed_at_unix_ms: u64,
) -> Result<Vec<u8>, GitObservationError> {
    deadline.require_active()?;
''',
)
replace_once(
    observe,
    '''        LocalFileReadBounds {
            max_bytes,
            max_duration,
        },
''',
    '''        LocalFileReadBounds {
            max_bytes,
            max_duration: deadline.remaining()?,
        },
''',
)
replace_once(
    observe,
    '''    LocalRead(LocalFileReadError),
    Io(io::Error),
''',
    '''    LocalRead(LocalFileReadError),
    DirectorySnapshot(LocalDirectorySnapshotError),
    OperationBudget(GitOperationBudgetError),
    Io(io::Error),
''',
)
replace_once(
    observe,
    '''            Self::LocalRead(error) => {
                write!(f, "Git observation bounded file read failed: {error}")
            }
            Self::Io(error) => write!(f, "Git observation filesystem I/O failed: {error}"),
''',
    '''            Self::LocalRead(error) => {
                write!(f, "Git observation bounded file read failed: {error}")
            }
            Self::DirectorySnapshot(error) => {
                write!(f, "Git observation directory snapshot failed: {error}")
            }
            Self::OperationBudget(error) => {
                write!(f, "Git observation operation budget failed: {error}")
            }
            Self::Io(error) => write!(f, "Git observation filesystem I/O failed: {error}"),
''',
)
replace_once(
    observe,
    '''            Self::LocalRead(error) => Some(error),
            Self::Io(error) => Some(error),
''',
    '''            Self::LocalRead(error) => Some(error),
            Self::DirectorySnapshot(error) => Some(error),
            Self::OperationBudget(error) => Some(error),
            Self::Io(error) => Some(error),
''',
)
replace_once(
    observe,
    '''impl From<io::Error> for GitObservationError {
''',
    '''impl From<LocalDirectorySnapshotError> for GitObservationError {
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
''',
)

print("T005-040 Git observation repair staged")
