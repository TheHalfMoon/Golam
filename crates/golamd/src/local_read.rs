#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::time::{Duration, Instant};

use golam_core::digest::sha256;
use golam_core::target_identity::{ObservedFileKind, ResolvedTargetIdentity};
use golam_core::tool_request::{BindingDigest, RequestedOperationId, RequestedTarget};

use crate::local_fs::{
    LocalFsResolutionError, LocalFsResolver, metadata_matches_resolved_identity,
};

const READ_CHUNK_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalFileReadBounds {
    pub max_bytes: u64,
    pub max_duration: Duration,
}

impl LocalFileReadBounds {
    pub fn validate(self) -> Result<(), LocalFileReadError> {
        if self.max_bytes == 0
            || self.max_duration.is_zero()
            || usize::try_from(self.max_bytes).is_err()
        {
            return Err(LocalFileReadError::InvalidBounds);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedFileStat {
    pub identity: ResolvedTargetIdentity,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedFileRead {
    pub identity: ResolvedTargetIdentity,
    pub content_digest: BindingDigest,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub enum LocalFileReadError {
    Resolution(LocalFsResolutionError),
    Io(io::Error),
    InvalidBounds,
    UnsupportedFileKind(ObservedFileKind),
    SizeLimitExceeded {
        identity: ResolvedTargetIdentity,
        observed: u64,
        limit: u64,
    },
    DurationLimitExceeded,
    TargetChangedDuringRead,
}

impl fmt::Display for LocalFileReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resolution(error) => write!(f, "local filesystem resolution failed: {error}"),
            Self::Io(error) => write!(f, "bounded local file read I/O error: {error}"),
            Self::InvalidBounds => f.write_str(
                "bounded local file read requires positive finite byte and duration limits",
            ),
            Self::UnsupportedFileKind(kind) => {
                write!(f, "bounded local file read denies file kind: {kind:?}")
            }
            Self::SizeLimitExceeded {
                observed, limit, ..
            } => write!(
                f,
                "bounded local file read size limit exceeded: observed={observed} limit={limit}"
            ),
            Self::DurationLimitExceeded => {
                f.write_str("bounded local file read duration limit exceeded")
            }
            Self::TargetChangedDuringRead => {
                f.write_str("bounded local file read target changed during observation")
            }
        }
    }
}

impl Error for LocalFileReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Resolution(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<LocalFsResolutionError> for LocalFileReadError {
    fn from(value: LocalFsResolutionError) -> Self {
        Self::Resolution(value)
    }
}

impl From<io::Error> for LocalFileReadError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn stat_regular_file(
    resolver: &LocalFsResolver,
    requested: &RequestedTarget,
    operation: &RequestedOperationId,
    bounds: LocalFileReadBounds,
    observed_at_unix_ms: u64,
) -> Result<BoundedFileStat, LocalFileReadError> {
    bounds.validate()?;
    let identity = resolver.resolve_read_target(requested, operation, observed_at_unix_ms)?;
    require_regular_file(&identity)?;

    let path = Path::new(identity.normalized_path.as_str());
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file()
        || !metadata_matches_resolved_identity(&identity, &metadata)?
    {
        return Err(LocalFileReadError::TargetChangedDuringRead);
    }
    if metadata.len() > bounds.max_bytes {
        return Err(LocalFileReadError::SizeLimitExceeded {
            identity,
            observed: metadata.len(),
            limit: bounds.max_bytes,
        });
    }

    Ok(BoundedFileStat {
        identity,
        size_bytes: metadata.len(),
    })
}

pub fn read_regular_file(
    resolver: &LocalFsResolver,
    requested: &RequestedTarget,
    operation: &RequestedOperationId,
    bounds: LocalFileReadBounds,
    observed_at_start_unix_ms: u64,
    observed_at_end_unix_ms: u64,
) -> Result<BoundedFileRead, LocalFileReadError> {
    read_regular_file_with_pre_open(
        resolver,
        requested,
        operation,
        bounds,
        observed_at_start_unix_ms,
        observed_at_end_unix_ms,
        || {},
    )
}

fn read_regular_file_with_pre_open<F>(
    resolver: &LocalFsResolver,
    requested: &RequestedTarget,
    operation: &RequestedOperationId,
    bounds: LocalFileReadBounds,
    observed_at_start_unix_ms: u64,
    observed_at_end_unix_ms: u64,
    pre_open: F,
) -> Result<BoundedFileRead, LocalFileReadError>
where
    F: FnOnce(),
{
    let initial = stat_regular_file(
        resolver,
        requested,
        operation,
        bounds,
        observed_at_start_unix_ms,
    )?;
    let started = Instant::now();
    let path = Path::new(initial.identity.normalized_path.as_str());

    pre_open();

    let mut file = open_read_only_no_follow(path)?;
    let opened_metadata = file.metadata()?;
    if !opened_metadata.file_type().is_file()
        || !metadata_matches_resolved_identity(&initial.identity, &opened_metadata)?
    {
        return Err(LocalFileReadError::TargetChangedDuringRead);
    }
    if opened_metadata.len() > bounds.max_bytes {
        return Err(LocalFileReadError::SizeLimitExceeded {
            identity: initial.identity.clone(),
            observed: opened_metadata.len(),
            limit: bounds.max_bytes,
        });
    }

    let max_bytes =
        usize::try_from(bounds.max_bytes).map_err(|_| LocalFileReadError::InvalidBounds)?;
    let mut bytes = Vec::with_capacity(max_bytes.min(READ_CHUNK_BYTES));
    let mut chunk = [0_u8; READ_CHUNK_BYTES];

    while bytes.len() < max_bytes {
        require_within_duration(started, bounds.max_duration)?;
        let remaining = max_bytes - bytes.len();
        let read = file.read(&mut chunk[..remaining.min(READ_CHUNK_BYTES)])?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);
    }

    if bytes.len() == max_bytes {
        require_within_duration(started, bounds.max_duration)?;
        let mut probe = [0_u8; 1];
        if file.read(&mut probe)? != 0 {
            return Err(LocalFileReadError::TargetChangedDuringRead);
        }
    }
    require_within_duration(started, bounds.max_duration)?;

    let verified = resolver.resolve_read_target(requested, operation, observed_at_end_unix_ms)?;
    require_regular_file(&verified)?;
    if initial.identity.resolved_target_identity != verified.resolved_target_identity
        || initial.identity.observed_metadata_digest != verified.observed_metadata_digest
    {
        return Err(LocalFileReadError::TargetChangedDuringRead);
    }

    Ok(BoundedFileRead {
        identity: verified,
        content_digest: BindingDigest::new(sha256(&bytes)),
        bytes,
    })
}

fn require_regular_file(identity: &ResolvedTargetIdentity) -> Result<(), LocalFileReadError> {
    if identity.file_kind != ObservedFileKind::RegularFile {
        return Err(LocalFileReadError::UnsupportedFileKind(identity.file_kind));
    }
    Ok(())
}

fn require_within_duration(
    started: Instant,
    max_duration: Duration,
) -> Result<(), LocalFileReadError> {
    if started.elapsed() > max_duration {
        return Err(LocalFileReadError::DurationLimitExceeded);
    }
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "android"))]
const O_NOFOLLOW_FLAG: i32 = 0x0002_0000;

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
const O_NOFOLLOW_FLAG: i32 = 0x0000_0100;

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd"
))]
fn open_read_only_no_follow(path: &Path) -> io::Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;

    fs::OpenOptions::new()
        .read(true)
        .custom_flags(O_NOFOLLOW_FLAG)
        .open(path)
}

#[cfg(all(
    unix,
    not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd"
    ))
))]
fn open_read_only_no_follow(_path: &Path) -> io::Result<fs::File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "bounded no-follow local file reads are unqualified on this Unix target",
    ))
}

#[cfg(windows)]
fn open_read_only_no_follow(path: &Path) -> io::Result<fs::File> {
    use std::os::windows::fs::OpenOptionsExt;

    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    fs::OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
}

#[cfg(not(any(unix, windows)))]
fn open_read_only_no_follow(_path: &Path) -> io::Result<fs::File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "bounded local file reads are unsupported on this platform",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::tool_request::ResourceClassId;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_root() -> std::path::PathBuf {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "golam-local-read-{}-{nanos}-{counter}",
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

    fn bounds(max_bytes: u64) -> LocalFileReadBounds {
        LocalFileReadBounds {
            max_bytes,
            max_duration: Duration::from_secs(1),
        }
    }

    #[test]
    fn reads_regular_file_with_bounded_attributable_result() {
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        fs::write(root.join("note.txt"), b"hello").unwrap();
        let resolver = resolver(&root);
        let result = read_regular_file(
            &resolver,
            &RequestedTarget::new("note.txt").unwrap(),
            &RequestedOperationId::new("read").unwrap(),
            bounds(16),
            10,
            11,
        )
        .unwrap();

        assert_eq!(result.bytes, b"hello");
        assert_eq!(result.identity.file_kind, ObservedFileKind::RegularFile);
        assert_eq!(result.content_digest, BindingDigest::new(sha256(b"hello")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn oversized_file_fails_without_returning_partial_content() {
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        fs::write(root.join("large.txt"), b"0123456789").unwrap();
        let resolver = resolver(&root);
        let error = read_regular_file(
            &resolver,
            &RequestedTarget::new("large.txt").unwrap(),
            &RequestedOperationId::new("read").unwrap(),
            bounds(4),
            10,
            11,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            LocalFileReadError::SizeLimitExceeded {
                identity: _,
                observed: 10,
                limit: 4
            }
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn parent_directory_replacement_cannot_return_outside_bytes() {
        use std::os::unix::fs::symlink;

        let root = unique_root();
        let outside = unique_root();
        fs::create_dir_all(root.join("nested")).unwrap();
        fs::create_dir(&outside).unwrap();
        fs::write(root.join("nested/note.txt"), b"inside").unwrap();
        fs::write(outside.join("note.txt"), b"outside-secret").unwrap();
        let resolver = resolver(&root);

        let result = read_regular_file_with_pre_open(
            &resolver,
            &RequestedTarget::new("nested/note.txt").unwrap(),
            &RequestedOperationId::new("read").unwrap(),
            bounds(64),
            10,
            11,
            || {
                fs::rename(root.join("nested"), root.join("original-nested")).unwrap();
                symlink(&outside, root.join("nested")).unwrap();
            },
        );

        assert!(matches!(result, Err(LocalFileReadError::TargetChangedDuringRead)));
        fs::remove_file(root.join("nested")).unwrap();
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[test]
    fn directories_and_zero_bounds_fail_closed() {
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("nested")).unwrap();
        let resolver = resolver(&root);
        let directory_error = stat_regular_file(
            &resolver,
            &RequestedTarget::new("nested").unwrap(),
            &RequestedOperationId::new("read").unwrap(),
            bounds(16),
            10,
        )
        .unwrap_err();
        assert!(matches!(
            directory_error,
            LocalFileReadError::UnsupportedFileKind(ObservedFileKind::Directory)
        ));

        let invalid = LocalFileReadBounds {
            max_bytes: 0,
            max_duration: Duration::from_secs(1),
        };
        assert!(matches!(
            invalid.validate(),
            Err(LocalFileReadError::InvalidBounds)
        ));
        fs::remove_dir_all(root).unwrap();
    }
}
