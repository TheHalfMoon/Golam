#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use golam_core::target_identity::{ObservedFileKind, ResolvedTargetIdentity};
use golam_core::tool_request::{BindingDigest, RequestedOperationId, RequestedTarget};

use crate::local_fs::{LocalFsResolutionError, LocalFsResolver};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalDirectoryWalkBounds {
    pub max_entries: u64,
    pub max_depth: u32,
    pub max_duration: Duration,
}

impl LocalDirectoryWalkBounds {
    pub fn validate(self) -> Result<(), LocalDirectoryWalkError> {
        if self.max_entries == 0
            || self.max_duration.is_zero()
            || usize::try_from(self.max_entries).is_err()
        {
            return Err(LocalDirectoryWalkError::InvalidBounds);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedDirectoryEntry {
    pub requested_path: RequestedTarget,
    pub identity: ResolvedTargetIdentity,
    pub depth: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedDirectoryWalk {
    pub root: ResolvedTargetIdentity,
    pub entries: Vec<BoundedDirectoryEntry>,
}

#[derive(Debug)]
pub enum LocalDirectoryWalkError {
    Resolution(LocalFsResolutionError),
    Io(io::Error),
    InvalidBounds,
    InvalidChildPath(PathBuf),
    NonUnicodeName(PathBuf),
    NotDirectory(ObservedFileKind),
    UnsupportedFileKind {
        path: RequestedTarget,
        kind: ObservedFileKind,
    },
    EntryLimitExceeded {
        limit: u64,
    },
    DurationLimitExceeded,
    DirectoryChangedDuringWalk(RequestedTarget),
}

impl fmt::Display for LocalDirectoryWalkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resolution(error) => write!(f, "local filesystem resolution failed: {error}"),
            Self::Io(error) => write!(f, "bounded local directory walk I/O error: {error}"),
            Self::InvalidBounds => f.write_str(
                "bounded local directory walk requires a positive finite entry limit and duration",
            ),
            Self::InvalidChildPath(path) => write!(
                f,
                "bounded local directory walk child path is invalid: {}",
                path.display()
            ),
            Self::NonUnicodeName(path) => write!(
                f,
                "bounded local directory walk denies non-Unicode path under the current request contract: {}",
                path.display()
            ),
            Self::NotDirectory(kind) => {
                write!(f, "bounded local directory walk requires a directory, observed: {kind:?}")
            }
            Self::UnsupportedFileKind { path, kind } => write!(
                f,
                "bounded local directory walk denies file kind {kind:?} at {}",
                path.as_str()
            ),
            Self::EntryLimitExceeded { limit } => {
                write!(f, "bounded local directory walk entry limit exceeded: limit={limit}")
            }
            Self::DurationLimitExceeded => {
                f.write_str("bounded local directory walk duration limit exceeded")
            }
            Self::DirectoryChangedDuringWalk(path) => write!(
                f,
                "bounded local directory identity changed during walk: {}",
                path.as_str()
            ),
        }
    }
}

impl Error for LocalDirectoryWalkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Resolution(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<LocalFsResolutionError> for LocalDirectoryWalkError {
    fn from(value: LocalFsResolutionError) -> Self {
        Self::Resolution(value)
    }
}

impl From<io::Error> for LocalDirectoryWalkError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Clone, Debug)]
struct PendingDirectory {
    requested: RequestedTarget,
    depth: u32,
    expected_target_identity: Option<BindingDigest>,
    expected_metadata_digest: BindingDigest,
}

pub fn walk_directory(
    resolver: &LocalFsResolver,
    requested: &RequestedTarget,
    operation: &RequestedOperationId,
    bounds: LocalDirectoryWalkBounds,
    observed_at_unix_ms: u64,
) -> Result<BoundedDirectoryWalk, LocalDirectoryWalkError> {
    bounds.validate()?;
    let started = Instant::now();
    let root = resolver.resolve_read_target(requested, operation, observed_at_unix_ms)?;
    require_directory(&root)?;

    let mut pending = VecDeque::new();
    pending.push_back(PendingDirectory {
        requested: requested.clone(),
        depth: 0,
        expected_target_identity: root.resolved_target_identity,
        expected_metadata_digest: root.observed_metadata_digest,
    });
    let mut entries = Vec::new();

    while let Some(directory) = pending.pop_front() {
        require_within_duration(started, bounds.max_duration)?;
        let current = resolver.resolve_read_target(
            &directory.requested,
            operation,
            observed_at_unix_ms,
        )?;
        require_directory(&current)?;
        if current.resolved_target_identity != directory.expected_target_identity
            || current.observed_metadata_digest != directory.expected_metadata_digest
        {
            return Err(LocalDirectoryWalkError::DirectoryChangedDuringWalk(
                directory.requested,
            ));
        }

        let absolute = Path::new(current.normalized_path.as_str());
        let remaining = bounds
            .max_entries
            .saturating_sub(u64::try_from(entries.len()).unwrap_or(u64::MAX));
        let mut names = Vec::new();
        for entry in fs::read_dir(absolute)? {
            require_within_duration(started, bounds.max_duration)?;
            if u64::try_from(names.len()).unwrap_or(u64::MAX) >= remaining {
                return Err(LocalDirectoryWalkError::EntryLimitExceeded {
                    limit: bounds.max_entries,
                });
            }
            let entry = entry?;
            let path = entry.path();
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| LocalDirectoryWalkError::NonUnicodeName(path))?;
            names.push(name);
        }
        names.sort_unstable();

        for name in names {
            require_within_duration(started, bounds.max_duration)?;
            let child = child_request(&directory.requested, &name)?;
            let identity = resolver.resolve_read_target(&child, operation, observed_at_unix_ms)?;
            match identity.file_kind {
                ObservedFileKind::RegularFile | ObservedFileKind::Directory => {}
                kind => {
                    return Err(LocalDirectoryWalkError::UnsupportedFileKind {
                        path: child,
                        kind,
                    });
                }
            }

            if identity.file_kind == ObservedFileKind::Directory
                && directory.depth < bounds.max_depth
            {
                pending.push_back(PendingDirectory {
                    requested: child.clone(),
                    depth: directory.depth + 1,
                    expected_target_identity: identity.resolved_target_identity,
                    expected_metadata_digest: identity.observed_metadata_digest,
                });
            }

            entries.push(BoundedDirectoryEntry {
                requested_path: child,
                identity,
                depth: directory.depth,
            });
        }
    }

    require_within_duration(started, bounds.max_duration)?;
    Ok(BoundedDirectoryWalk { root, entries })
}

fn require_directory(identity: &ResolvedTargetIdentity) -> Result<(), LocalDirectoryWalkError> {
    if identity.file_kind != ObservedFileKind::Directory {
        return Err(LocalDirectoryWalkError::NotDirectory(identity.file_kind));
    }
    Ok(())
}

fn child_request(
    parent: &RequestedTarget,
    name: &str,
) -> Result<RequestedTarget, LocalDirectoryWalkError> {
    let path = Path::new(parent.as_str()).join(name);
    let value = path
        .to_str()
        .ok_or_else(|| LocalDirectoryWalkError::NonUnicodeName(path.clone()))?;
    RequestedTarget::new(value)
        .map_err(|_| LocalDirectoryWalkError::InvalidChildPath(path.to_path_buf()))
}

fn require_within_duration(
    started: Instant,
    max_duration: Duration,
) -> Result<(), LocalDirectoryWalkError> {
    if started.elapsed() > max_duration {
        return Err(LocalDirectoryWalkError::DurationLimitExceeded);
    }
    Ok(())
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
            "golam-local-walk-{}-{nanos}-{counter}",
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

    fn bounds(max_entries: u64, max_depth: u32) -> LocalDirectoryWalkBounds {
        LocalDirectoryWalkBounds {
            max_entries,
            max_depth,
            max_duration: Duration::from_secs(1),
        }
    }

    fn basename(entry: &BoundedDirectoryEntry) -> String {
        Path::new(entry.requested_path.as_str())
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn walk_is_deterministic_and_depth_bounded() {
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        fs::write(root.join("b.txt"), b"b").unwrap();
        fs::write(root.join("a.txt"), b"a").unwrap();
        fs::create_dir(root.join("nested")).unwrap();
        fs::write(root.join("nested").join("c.txt"), b"c").unwrap();
        let resolver = resolver(&root);

        let shallow = walk_directory(
            &resolver,
            &RequestedTarget::new(".").unwrap(),
            &RequestedOperationId::new("list").unwrap(),
            bounds(10, 0),
            10,
        )
        .unwrap();
        let shallow_names = shallow.entries.iter().map(basename).collect::<Vec<_>>();
        assert_eq!(shallow_names, vec!["a.txt", "b.txt", "nested"]);
        assert!(shallow.entries.iter().all(|entry| entry.depth == 0));

        let deep = walk_directory(
            &resolver,
            &RequestedTarget::new(".").unwrap(),
            &RequestedOperationId::new("list").unwrap(),
            bounds(10, 1),
            10,
        )
        .unwrap();
        assert_eq!(deep.entries.len(), 4);
        assert_eq!(basename(deep.entries.last().unwrap()), "c.txt");
        assert_eq!(deep.entries.last().unwrap().depth, 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn entry_limit_fails_without_returning_partial_walk() {
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        fs::write(root.join("a.txt"), b"a").unwrap();
        fs::write(root.join("b.txt"), b"b").unwrap();
        let resolver = resolver(&root);
        let error = walk_directory(
            &resolver,
            &RequestedTarget::new(".").unwrap(),
            &RequestedOperationId::new("list").unwrap(),
            bounds(1, 0),
            10,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            LocalDirectoryWalkError::EntryLimitExceeded { limit: 1 }
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn zero_entry_or_duration_bounds_fail_closed() {
        let zero_entries = LocalDirectoryWalkBounds {
            max_entries: 0,
            max_depth: 0,
            max_duration: Duration::from_secs(1),
        };
        assert!(matches!(
            zero_entries.validate(),
            Err(LocalDirectoryWalkError::InvalidBounds)
        ));
        let zero_duration = LocalDirectoryWalkBounds {
            max_entries: 1,
            max_depth: 0,
            max_duration: Duration::ZERO,
        };
        assert!(matches!(
            zero_duration.validate(),
            Err(LocalDirectoryWalkError::InvalidBounds)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_entry_is_denied_by_the_authorized_resolver() {
        use std::os::unix::fs::symlink;

        let root = unique_root();
        fs::create_dir(&root).unwrap();
        fs::write(root.join("real.txt"), b"hello").unwrap();
        symlink(root.join("real.txt"), root.join("alias.txt")).unwrap();
        let resolver = resolver(&root);
        let error = walk_directory(
            &resolver,
            &RequestedTarget::new(".").unwrap(),
            &RequestedOperationId::new("list").unwrap(),
            bounds(10, 0),
            10,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            LocalDirectoryWalkError::Resolution(LocalFsResolutionError::AliasBoundary { .. })
        ));
        fs::remove_dir_all(root).unwrap();
    }
}
