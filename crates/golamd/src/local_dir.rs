#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::fs::{self, File, ReadDir};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use golam_core::target_identity::{ObservedFileKind, ResolvedTargetIdentity};
use golam_core::tool_request::{RequestedOperationId, RequestedTarget};

use crate::local_fs::{
    LocalFsResolutionError, LocalFsResolver, metadata_matches_resolved_identity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalDirectorySnapshotBounds {
    pub max_entries: usize,
    pub max_duration: Duration,
}

impl LocalDirectorySnapshotBounds {
    pub fn validate(self) -> Result<(), LocalDirectorySnapshotError> {
        if self.max_entries == 0 || self.max_duration.is_zero() {
            return Err(LocalDirectorySnapshotError::InvalidBounds);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalDirectorySnapshot {
    pub identity: ResolvedTargetIdentity,
    pub names: Vec<String>,
}

#[derive(Debug)]
pub enum LocalDirectorySnapshotError {
    Resolution(LocalFsResolutionError),
    Io(io::Error),
    InvalidBounds,
    UnsupportedPlatform,
    NotDirectory(ObservedFileKind),
    TargetChangedDuringObservation,
    NonUnicodeName(PathBuf),
    EntryLimitExceeded,
    DurationLimitExceeded,
}

impl fmt::Display for LocalDirectorySnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resolution(error) => write!(f, "local directory resolution failed: {error}"),
            Self::Io(error) => write!(f, "retained-handle directory observation failed: {error}"),
            Self::InvalidBounds => {
                f.write_str("retained-handle directory observation requires positive bounds")
            }
            Self::UnsupportedPlatform => f.write_str(
                "retained-handle directory observation is unqualified on this platform",
            ),
            Self::NotDirectory(kind) => {
                write!(f, "retained-handle directory observation requires a directory, observed {kind:?}")
            }
            Self::TargetChangedDuringObservation => {
                f.write_str("directory identity changed during retained-handle observation")
            }
            Self::NonUnicodeName(path) => write!(
                f,
                "directory entry name is non-Unicode under the bounded request profile: {}",
                path.display()
            ),
            Self::EntryLimitExceeded => {
                f.write_str("retained-handle directory observation entry limit exceeded")
            }
            Self::DurationLimitExceeded => {
                f.write_str("retained-handle directory observation duration limit exceeded")
            }
        }
    }
}

impl Error for LocalDirectorySnapshotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Resolution(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<LocalFsResolutionError> for LocalDirectorySnapshotError {
    fn from(value: LocalFsResolutionError) -> Self {
        Self::Resolution(value)
    }
}

impl From<io::Error> for LocalDirectorySnapshotError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn snapshot_directory(
    resolver: &LocalFsResolver,
    requested: &RequestedTarget,
    operation: &RequestedOperationId,
    bounds: LocalDirectorySnapshotBounds,
    observed_at_unix_ms: u64,
) -> Result<LocalDirectorySnapshot, LocalDirectorySnapshotError> {
    snapshot_directory_with_pre_enumeration(
        resolver,
        requested,
        operation,
        bounds,
        observed_at_unix_ms,
        || {},
    )
}

fn snapshot_directory_with_pre_enumeration<F>(
    resolver: &LocalFsResolver,
    requested: &RequestedTarget,
    operation: &RequestedOperationId,
    bounds: LocalDirectorySnapshotBounds,
    observed_at_unix_ms: u64,
    pre_enumeration: F,
) -> Result<LocalDirectorySnapshot, LocalDirectorySnapshotError>
where
    F: FnOnce(),
{
    bounds.validate()?;
    require_platform()?;
    let started = Instant::now();
    let initial = resolver.resolve_read_target(requested, operation, observed_at_unix_ms)?;
    require_directory(&initial)?;
    require_within_duration(started, bounds.max_duration)?;

    let path = Path::new(initial.normalized_path.as_str());
    let directory = File::open(path)?;
    let opened_metadata = directory.metadata()?;
    if !opened_metadata.is_dir()
        || !metadata_matches_resolved_identity(&initial, &opened_metadata)?
    {
        return Err(LocalDirectorySnapshotError::TargetChangedDuringObservation);
    }

    pre_enumeration();
    require_within_duration(started, bounds.max_duration)?;

    let mut names = Vec::new();
    for entry in read_dir_from_handle(&directory)? {
        require_within_duration(started, bounds.max_duration)?;
        if names.len() >= bounds.max_entries {
            return Err(LocalDirectorySnapshotError::EntryLimitExceeded);
        }
        let entry = entry?;
        let path = entry.path();
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| LocalDirectorySnapshotError::NonUnicodeName(path))?;
        names.push(name);
    }
    names.sort_unstable();

    let terminal_metadata = directory.metadata()?;
    if !terminal_metadata.is_dir()
        || !metadata_matches_resolved_identity(&initial, &terminal_metadata)?
    {
        return Err(LocalDirectorySnapshotError::TargetChangedDuringObservation);
    }
    let verified = resolver.resolve_read_target(requested, operation, observed_at_unix_ms)?;
    require_directory(&verified)?;
    if verified.resolved_target_identity != initial.resolved_target_identity
        || verified.observed_metadata_digest != initial.observed_metadata_digest
    {
        return Err(LocalDirectorySnapshotError::TargetChangedDuringObservation);
    }
    require_within_duration(started, bounds.max_duration)?;

    Ok(LocalDirectorySnapshot {
        identity: verified,
        names,
    })
}

fn require_directory(
    identity: &ResolvedTargetIdentity,
) -> Result<(), LocalDirectorySnapshotError> {
    if identity.file_kind != ObservedFileKind::Directory {
        return Err(LocalDirectorySnapshotError::NotDirectory(identity.file_kind));
    }
    Ok(())
}

fn require_within_duration(
    started: Instant,
    max_duration: Duration,
) -> Result<(), LocalDirectorySnapshotError> {
    if started.elapsed() >= max_duration {
        return Err(LocalDirectorySnapshotError::DurationLimitExceeded);
    }
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn require_platform() -> Result<(), LocalDirectorySnapshotError> {
    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
fn require_platform() -> Result<(), LocalDirectorySnapshotError> {
    Ok(())
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd"
)))]
fn require_platform() -> Result<(), LocalDirectorySnapshotError> {
    Err(LocalDirectorySnapshotError::UnsupportedPlatform)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn read_dir_from_handle(directory: &File) -> io::Result<ReadDir> {
    use std::os::fd::AsRawFd;

    fs::read_dir(format!("/proc/self/fd/{}", directory.as_raw_fd()))
}

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
fn read_dir_from_handle(directory: &File) -> io::Result<ReadDir> {
    use std::os::fd::AsRawFd;

    fs::read_dir(format!("/dev/fd/{}", directory.as_raw_fd()))
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd"
)))]
fn read_dir_from_handle(_directory: &File) -> io::Result<ReadDir> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "retained-handle directory enumeration is unqualified on this platform",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::tool_request::ResourceClassId;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_root() -> PathBuf {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "golam-local-dir-{}-{nanos}-{counter}",
            std::process::id()
        ))
    }

    fn resolver(root: &Path) -> LocalFsResolver {
        LocalFsResolver::new(
            root,
            ResourceClassId::new("workspace.list").unwrap(),
            vec![RequestedOperationId::new("list").unwrap()],
            [],
        )
        .unwrap()
    }

    fn bounds() -> LocalDirectorySnapshotBounds {
        LocalDirectorySnapshotBounds {
            max_entries: 32,
            max_duration: Duration::from_secs(1),
        }
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd"
    ))]
    #[test]
    fn retained_handle_snapshot_is_sorted_and_identity_bound() {
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        fs::write(root.join("b.txt"), b"b").unwrap();
        fs::write(root.join("a.txt"), b"a").unwrap();
        let resolver = resolver(&root);
        let snapshot = snapshot_directory(
            &resolver,
            &RequestedTarget::new(".").unwrap(),
            &RequestedOperationId::new("list").unwrap(),
            bounds(),
            10,
        )
        .unwrap();
        assert_eq!(snapshot.names, vec!["a.txt", "b.txt"]);
        assert_eq!(snapshot.identity.file_kind, ObservedFileKind::Directory);
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
    fn path_replacement_after_handle_open_fails_closed_before_returning_names() {
        use std::os::unix::fs::symlink;

        let root = unique_root();
        let outside = unique_root();
        fs::create_dir_all(root.join("nested")).unwrap();
        fs::create_dir(&outside).unwrap();
        fs::write(root.join("nested/inside.txt"), b"inside").unwrap();
        fs::write(outside.join("outside.txt"), b"outside").unwrap();
        let resolver = resolver(&root);

        let result = snapshot_directory_with_pre_enumeration(
            &resolver,
            &RequestedTarget::new("nested").unwrap(),
            &RequestedOperationId::new("list").unwrap(),
            bounds(),
            10,
            || {
                fs::rename(root.join("nested"), root.join("original-nested")).unwrap();
                symlink(&outside, root.join("nested")).unwrap();
            },
        );

        assert!(matches!(
            result,
            Err(LocalDirectorySnapshotError::Resolution(
                LocalFsResolutionError::AliasBoundary { .. }
            ))
        ));
        fs::remove_file(root.join("nested")).unwrap();
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn windows_directory_snapshot_fails_closed_until_handle_enumeration_is_admitted() {
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        let resolver = resolver(&root);
        let result = snapshot_directory(
            &resolver,
            &RequestedTarget::new(".").unwrap(),
            &RequestedOperationId::new("list").unwrap(),
            bounds(),
            10,
        );
        assert!(matches!(
            result,
            Err(LocalDirectorySnapshotError::UnsupportedPlatform)
        ));
        fs::remove_dir_all(root).unwrap();
    }
}
