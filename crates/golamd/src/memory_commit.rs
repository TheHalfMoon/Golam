#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use golam_core::digest::sha256;
use golam_core::memory_storage::MemoryLayout;
use golam_core::target_identity::ObservedFileKind;
use golam_core::tool_request::{BindingDigest, RequestedOperationId, RequestedTarget};
use golam_core::{CanonicalEncoder, CoreError, EffectId};

use crate::local_fs::{
    LocalFsResolutionError, LocalFsResolver, metadata_matches_resolved_identity,
};

const MARKDOWN_READBACK_DOMAIN: &[u8] = b"golam:managed-markdown-readback:v1";
const MAX_COMMIT_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkdownCommitReceipt {
    pub readback_ref: BindingDigest,
    pub target_identity_ref: BindingDigest,
    pub content_digest: BindingDigest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreservedConflict {
    pub preserved_path: PathBuf,
    pub observed_content_digest: Option<BindingDigest>,
}

#[derive(Debug)]
pub enum MemoryCommitError {
    Io(io::Error),
    Core(CoreError),
    Resolution(LocalFsResolutionError),
    UnsupportedPlatform,
    InvalidTargetKind(ObservedFileKind),
    TargetIdentityMismatch,
    ContentDigestMismatch,
    CommitTooLarge,
    StagingCollision(PathBuf),
    StagingBoundary(PathBuf),
    UserEditDetected(PreservedConflict),
    UnknownOutcome(PreservedConflict),
}

impl fmt::Display for MemoryCommitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "managed Markdown commit I/O failed: {error}"),
            Self::Core(error) => write!(f, "managed Markdown commit encoding failed: {error}"),
            Self::Resolution(error) => {
                write!(f, "managed Markdown target resolution failed: {error}")
            }
            Self::UnsupportedPlatform => f.write_str(
                "managed Markdown conditional replacement is not qualified on this platform",
            ),
            Self::InvalidTargetKind(kind) => write!(
                f,
                "managed Markdown conditional replacement requires a regular file, observed {kind:?}"
            ),
            Self::TargetIdentityMismatch => {
                f.write_str("managed Markdown target identity no longer matches PREPARED authority")
            }
            Self::ContentDigestMismatch => {
                f.write_str("managed Markdown content no longer matches PREPARED authority")
            }
            Self::CommitTooLarge => f.write_str("managed Markdown commit exceeds the byte bound"),
            Self::StagingCollision(path) => write!(
                f,
                "managed Markdown staging path already exists and requires reconciliation: {}",
                path.display()
            ),
            Self::StagingBoundary(path) => write!(
                f,
                "managed Markdown staging path is not a protected ordinary directory: {}",
                path.display()
            ),
            Self::UserEditDetected(conflict) => write!(
                f,
                "managed Markdown user edit was preserved for reconciliation at {}",
                conflict.preserved_path.display()
            ),
            Self::UnknownOutcome(conflict) => write!(
                f,
                "managed Markdown commit outcome is ambiguous; preserved artifact: {}",
                conflict.preserved_path.display()
            ),
        }
    }
}

impl Error for MemoryCommitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::Resolution(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for MemoryCommitError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<CoreError> for MemoryCommitError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<LocalFsResolutionError> for MemoryCommitError {
    fn from(value: LocalFsResolutionError) -> Self {
        Self::Resolution(value)
    }
}

pub fn commit_existing_markdown(
    resolver: &LocalFsResolver,
    requested: &RequestedTarget,
    layout: &MemoryLayout,
    effect_id: EffectId,
    expected_target_identity_ref: BindingDigest,
    expected_content_digest: BindingDigest,
    new_bytes: &[u8],
    observed_at_unix_ms: u64,
) -> Result<MarkdownCommitReceipt, MemoryCommitError> {
    commit_existing_with_hook(
        resolver,
        requested,
        layout,
        effect_id,
        expected_target_identity_ref,
        expected_content_digest,
        new_bytes,
        observed_at_unix_ms,
        |_| {},
    )
}

#[allow(clippy::too_many_arguments)]
fn commit_existing_with_hook<F>(
    resolver: &LocalFsResolver,
    requested: &RequestedTarget,
    layout: &MemoryLayout,
    effect_id: EffectId,
    expected_target_identity_ref: BindingDigest,
    expected_content_digest: BindingDigest,
    new_bytes: &[u8],
    observed_at_unix_ms: u64,
    pre_quarantine: F,
) -> Result<MarkdownCommitReceipt, MemoryCommitError>
where
    F: FnOnce(&Path),
{
    require_supported_platform()?;
    if new_bytes.len() > MAX_COMMIT_BYTES {
        return Err(MemoryCommitError::CommitTooLarge);
    }
    let staging_dir = ensure_staging_dir(layout)?;
    let token = format!("{:032x}", effect_id.0);
    let staged = staging_dir.join(format!("{token}.next"));
    let previous = staging_dir.join(format!("{token}.previous"));
    let displaced = staging_dir.join(format!("{token}.displaced"));
    for path in [&staged, &previous, &displaced] {
        require_missing(path)?;
    }

    let mut staged_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staged)?;
    staged_file.write_all(new_bytes)?;
    staged_file.sync_all()?;
    drop(staged_file);

    let operation = RequestedOperationId::new("memory.write")
        .map_err(|_| MemoryCommitError::TargetIdentityMismatch)?;
    let initial = resolver.resolve_read_target(requested, &operation, observed_at_unix_ms)?;
    if initial.file_kind != ObservedFileKind::RegularFile {
        cleanup_best_effort(&staged);
        return Err(MemoryCommitError::InvalidTargetKind(initial.file_kind));
    }
    if initial.resolved_target_identity != Some(expected_target_identity_ref) {
        cleanup_best_effort(&staged);
        return Err(MemoryCommitError::TargetIdentityMismatch);
    }
    let target = Path::new(initial.normalized_path.as_str());
    let initial_bytes = fs::read(target)?;
    if digest(&initial_bytes) != expected_content_digest {
        cleanup_best_effort(&staged);
        return Err(MemoryCommitError::ContentDigestMismatch);
    }

    pre_quarantine(target);
    if let Err(error) = fs::rename(target, &previous) {
        cleanup_best_effort(&staged);
        return Err(MemoryCommitError::Io(error));
    }

    let moved_matches = fs::metadata(&previous)
        .ok()
        .and_then(|metadata| metadata_matches_resolved_identity(&initial, &metadata).ok())
        == Some(true);
    let moved_digest = fs::read(&previous).ok().map(|bytes| digest(&bytes));
    if !moved_matches || moved_digest != Some(expected_content_digest) {
        return rollback_before_install(
            target,
            &previous,
            &staged,
            moved_digest,
            MemoryCommitError::UserEditDetected,
        );
    }

    if let Err(_error) = fs::hard_link(&staged, target) {
        return rollback_before_install(
            target,
            &previous,
            &staged,
            moved_digest,
            MemoryCommitError::UserEditDetected,
        );
    }

    let target_bytes = match fs::read(target) {
        Ok(bytes) => bytes,
        Err(_) => {
            return rollback_after_install(target, &previous, &staged, &displaced, None, true);
        }
    };
    let new_digest = digest(&target_bytes);
    let previous_digest = fs::read(&previous).ok().map(|bytes| digest(&bytes));
    if new_digest != digest(new_bytes) || previous_digest != Some(expected_content_digest) {
        return rollback_after_install(
            target,
            &previous,
            &staged,
            &displaced,
            Some(new_digest),
            false,
        );
    }

    let verified = resolver.resolve_read_target(requested, &operation, observed_at_unix_ms)?;
    if verified.file_kind != ObservedFileKind::RegularFile
        || fs::read(target).ok().map(|bytes| digest(&bytes)) != Some(new_digest)
    {
        return rollback_after_install(
            target,
            &previous,
            &staged,
            &displaced,
            Some(new_digest),
            false,
        );
    }

    sync_file_and_parent(target)?;
    fs::remove_file(&staged)?;
    fs::remove_file(&previous)?;
    let target_identity_ref = verified
        .resolved_target_identity
        .ok_or(MemoryCommitError::TargetIdentityMismatch)?;
    let readback_ref = markdown_readback_ref(target_identity_ref, new_digest)?;
    Ok(MarkdownCommitReceipt {
        readback_ref,
        target_identity_ref,
        content_digest: new_digest,
    })
}

fn rollback_before_install<F>(
    target: &Path,
    previous: &Path,
    staged: &Path,
    observed_content_digest: Option<BindingDigest>,
    classify: F,
) -> Result<MarkdownCommitReceipt, MemoryCommitError>
where
    F: FnOnce(PreservedConflict) -> MemoryCommitError,
{
    match fs::hard_link(previous, target) {
        Ok(()) => {
            cleanup_best_effort(previous);
            cleanup_best_effort(staged);
            Err(classify(PreservedConflict {
                preserved_path: target.to_path_buf(),
                observed_content_digest,
            }))
        }
        Err(_) => Err(MemoryCommitError::UnknownOutcome(PreservedConflict {
            preserved_path: previous.to_path_buf(),
            observed_content_digest,
        })),
    }
}

fn rollback_after_install(
    target: &Path,
    previous: &Path,
    staged: &Path,
    displaced: &Path,
    observed_content_digest: Option<BindingDigest>,
    unreadable_target: bool,
) -> Result<MarkdownCommitReceipt, MemoryCommitError> {
    if fs::rename(target, displaced).is_err() {
        return Err(MemoryCommitError::UnknownOutcome(PreservedConflict {
            preserved_path: previous.to_path_buf(),
            observed_content_digest,
        }));
    }
    if fs::hard_link(previous, target).is_err() {
        return Err(MemoryCommitError::UnknownOutcome(PreservedConflict {
            preserved_path: previous.to_path_buf(),
            observed_content_digest,
        }));
    }
    cleanup_best_effort(previous);
    cleanup_best_effort(staged);
    let conflict = PreservedConflict {
        preserved_path: displaced.to_path_buf(),
        observed_content_digest,
    };
    if unreadable_target {
        Err(MemoryCommitError::UnknownOutcome(conflict))
    } else {
        Err(MemoryCommitError::UserEditDetected(conflict))
    }
}

fn ensure_staging_dir(layout: &MemoryLayout) -> Result<PathBuf, MemoryCommitError> {
    let staging = layout.operational_dir().join("staging");
    match fs::symlink_metadata(&staging) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(MemoryCommitError::StagingBoundary(staging));
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            fs::create_dir(&staging)?;
        }
        Err(error) => return Err(error.into()),
    }
    Ok(staging)
}

fn require_missing(path: &Path) -> Result<(), MemoryCommitError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(MemoryCommitError::StagingCollision(path.to_path_buf())),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn digest(bytes: &[u8]) -> BindingDigest {
    BindingDigest::new(sha256(bytes))
}

fn markdown_readback_ref(
    target_identity_ref: BindingDigest,
    content_digest: BindingDigest,
) -> Result<BindingDigest, MemoryCommitError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(MARKDOWN_READBACK_DOMAIN)?;
    encoder.push_bytes(&target_identity_ref.bytes())?;
    encoder.push_bytes(&content_digest.bytes())?;
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

#[cfg(unix)]
fn require_supported_platform() -> Result<(), MemoryCommitError> {
    Ok(())
}

#[cfg(not(unix))]
fn require_supported_platform() -> Result<(), MemoryCommitError> {
    Err(MemoryCommitError::UnsupportedPlatform)
}

#[cfg(unix)]
fn sync_file_and_parent(path: &Path) -> Result<(), MemoryCommitError> {
    OpenOptions::new().read(true).open(path)?.sync_all()?;
    let parent = path
        .parent()
        .ok_or_else(|| MemoryCommitError::StagingBoundary(path.to_path_buf()))?;
    OpenOptions::new().read(true).open(parent)?.sync_all()?;
    Ok(())
}

#[cfg(not(unix))]
fn sync_file_and_parent(_path: &Path) -> Result<(), MemoryCommitError> {
    Err(MemoryCommitError::UnsupportedPlatform)
}

fn cleanup_best_effort(path: &Path) {
    let _ = fs::remove_file(path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::memory_storage::MemoryVaultScope;
    use golam_core::paths::RuntimeLayout;
    use golam_core::tool_request::{RequestedOperationId, ResourceClassId};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    fn runtime() -> RuntimeLayout {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-memory-commit-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    #[cfg(unix)]
    fn fixture() -> (
        RuntimeLayout,
        MemoryLayout,
        LocalFsResolver,
        RequestedTarget,
        BindingDigest,
        BindingDigest,
    ) {
        let runtime = runtime();
        let layout = MemoryLayout::initialize(&runtime).unwrap();
        let item = golam_core::memory::MemoryItemId(BindingDigest::new([1; 32]));
        let target = layout.item_path(MemoryVaultScope::User, item).unwrap();
        fs::write(&target, b"old\n").unwrap();
        let resolver = LocalFsResolver::new(
            layout.vault_dir(),
            ResourceClassId::new("memory.vault").unwrap(),
            vec![RequestedOperationId::new("memory.write").unwrap()],
            [layout.operational_dir().to_path_buf()],
        )
        .unwrap();
        let relative = target.strip_prefix(layout.vault_dir()).unwrap();
        let requested = RequestedTarget::new(relative.to_str().unwrap()).unwrap();
        let initial = resolver
            .resolve_read_target(
                &requested,
                &RequestedOperationId::new("memory.write").unwrap(),
                10,
            )
            .unwrap();
        (
            runtime,
            layout,
            resolver,
            requested,
            initial.resolved_target_identity.unwrap(),
            digest(b"old\n"),
        )
    }

    #[cfg(unix)]
    #[test]
    fn conditional_commit_replaces_only_the_exact_observed_file() {
        let (runtime, layout, resolver, requested, identity, content) = fixture();
        let receipt = commit_existing_markdown(
            &resolver,
            &requested,
            &layout,
            EffectId(1),
            identity,
            content,
            b"new\n",
            11,
        )
        .unwrap();
        let target = Path::new(
            resolver
                .resolve_read_target(
                    &requested,
                    &RequestedOperationId::new("memory.write").unwrap(),
                    12,
                )
                .unwrap()
                .normalized_path
                .as_str(),
        )
        .to_path_buf();
        assert_eq!(fs::read(target).unwrap(), b"new\n");
        assert_eq!(receipt.content_digest, digest(b"new\n"));
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn in_place_user_edit_after_check_is_preserved_not_overwritten() {
        let (runtime, layout, resolver, requested, identity, content) = fixture();
        let result = commit_existing_with_hook(
            &resolver,
            &requested,
            &layout,
            EffectId(2),
            identity,
            content,
            b"new\n",
            11,
            |target| fs::write(target, b"user edit\n").unwrap(),
        );
        assert!(matches!(
            result,
            Err(MemoryCommitError::UserEditDetected(_))
        ));
        let target = resolver
            .resolve_read_target(
                &requested,
                &RequestedOperationId::new("memory.write").unwrap(),
                12,
            )
            .unwrap();
        assert_eq!(
            fs::read(Path::new(target.normalized_path.as_str())).unwrap(),
            b"user edit\n"
        );
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn path_swap_after_check_is_preserved_and_never_receives_new_content() {
        let (runtime, layout, resolver, requested, identity, content) = fixture();
        let result = commit_existing_with_hook(
            &resolver,
            &requested,
            &layout,
            EffectId(3),
            identity,
            content,
            b"new\n",
            11,
            |target| {
                let original = target.with_extension("original");
                fs::rename(target, &original).unwrap();
                fs::write(target, b"replacement\n").unwrap();
            },
        );
        assert!(matches!(
            result,
            Err(MemoryCommitError::UserEditDetected(_))
        ));
        let target = resolver
            .resolve_read_target(
                &requested,
                &RequestedOperationId::new("memory.write").unwrap(),
                12,
            )
            .unwrap();
        assert_eq!(
            fs::read(Path::new(target.normalized_path.as_str())).unwrap(),
            b"replacement\n"
        );
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn windows_commit_fails_closed_until_strong_opened_handle_identity_is_admitted() {
        let runtime = runtime();
        let layout = MemoryLayout::initialize(&runtime).unwrap();
        let resolver = LocalFsResolver::new(
            layout.vault_dir(),
            ResourceClassId::new("memory.vault").unwrap(),
            vec![RequestedOperationId::new("memory.write").unwrap()],
            [layout.operational_dir().to_path_buf()],
        )
        .unwrap();
        let result = commit_existing_markdown(
            &resolver,
            &RequestedTarget::new("user/missing.md").unwrap(),
            &layout,
            EffectId(4),
            BindingDigest::new([1; 32]),
            BindingDigest::new([2; 32]),
            b"new\n",
            10,
        );
        assert!(matches!(
            result,
            Err(MemoryCommitError::UnsupportedPlatform)
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
