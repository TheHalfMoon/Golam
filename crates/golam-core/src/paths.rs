use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtectionLevel {
    UserOnlyVerified,
    PathIsolationOnly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeLayout {
    pub root: PathBuf,
    pub data_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub artifact_dir: PathBuf,
    pub artifact_tmp_dir: PathBuf,
    pub protection: ProtectionLevel,
}

#[derive(Debug)]
pub enum ProtectedPathError {
    Io(io::Error),
    Symlink(PathBuf),
    NotDirectory(PathBuf),
    PermissionsTooBroad { path: PathBuf, mode: u32 },
    AuthorityProtectionUnverified,
}

impl fmt::Display for ProtectedPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "protected path I/O error: {error}"),
            Self::Symlink(path) => write!(f, "protected path is a symlink: {}", path.display()),
            Self::NotDirectory(path) => {
                write!(f, "protected path is not a directory: {}", path.display())
            }
            Self::PermissionsTooBroad { path, mode } => write!(
                f,
                "protected directory permissions are too broad: {} mode {mode:o}",
                path.display()
            ),
            Self::AuthorityProtectionUnverified => {
                f.write_str("authority directory protection is not verified on this platform")
            }
        }
    }
}

impl Error for ProtectedPathError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Symlink(_)
            | Self::NotDirectory(_)
            | Self::PermissionsTooBroad { .. }
            | Self::AuthorityProtectionUnverified => None,
        }
    }
}

impl From<io::Error> for ProtectedPathError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl RuntimeLayout {
    pub fn initialize(root: impl AsRef<Path>) -> Result<Self, ProtectedPathError> {
        let root = root.as_ref().to_path_buf();
        let data_dir = root.join("data");
        let runtime_dir = root.join("runtime");
        let artifact_dir = data_dir.join("artifacts");
        let artifact_tmp_dir = artifact_dir.join(".tmp");

        for path in [
            &root,
            &data_dir,
            &runtime_dir,
            &artifact_dir,
            &artifact_tmp_dir,
        ] {
            create_private_directory(path)?;
        }

        Ok(Self {
            root,
            data_dir,
            runtime_dir,
            artifact_dir,
            artifact_tmp_dir,
            protection: platform_protection_level(),
        })
    }

    pub fn require_authority_ready(&self) -> Result<(), ProtectedPathError> {
        match self.protection {
            ProtectionLevel::UserOnlyVerified => Ok(()),
            ProtectionLevel::PathIsolationOnly => {
                Err(ProtectedPathError::AuthorityProtectionUnverified)
            }
        }
    }
}

fn create_private_directory(path: &Path) -> Result<(), ProtectedPathError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => validate_directory(path, &metadata)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            fs::create_dir(path)?;
        }
        Err(error) => return Err(error.into()),
    }

    apply_platform_permissions(path)?;
    let metadata = fs::symlink_metadata(path)?;
    validate_directory(path, &metadata)?;
    verify_platform_permissions(path, &metadata)
}

fn validate_directory(path: &Path, metadata: &fs::Metadata) -> Result<(), ProtectedPathError> {
    if metadata.file_type().is_symlink() {
        return Err(ProtectedPathError::Symlink(path.to_path_buf()));
    }
    if !metadata.is_dir() {
        return Err(ProtectedPathError::NotDirectory(path.to_path_buf()));
    }
    Ok(())
}

#[cfg(unix)]
fn apply_platform_permissions(path: &Path) -> Result<(), ProtectedPathError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

#[cfg(unix)]
fn verify_platform_permissions(
    path: &Path,
    metadata: &fs::Metadata,
) -> Result<(), ProtectedPathError> {
    use std::os::unix::fs::PermissionsExt;

    let mode = metadata.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(ProtectedPathError::PermissionsTooBroad {
            path: path.to_path_buf(),
            mode,
        });
    }
    Ok(())
}

#[cfg(unix)]
const fn platform_protection_level() -> ProtectionLevel {
    ProtectionLevel::UserOnlyVerified
}

#[cfg(windows)]
fn apply_platform_permissions(_path: &Path) -> Result<(), ProtectedPathError> {
    Ok(())
}

#[cfg(windows)]
fn verify_platform_permissions(
    _path: &Path,
    _metadata: &fs::Metadata,
) -> Result<(), ProtectedPathError> {
    Ok(())
}

#[cfg(windows)]
const fn platform_protection_level() -> ProtectionLevel {
    ProtectionLevel::PathIsolationOnly
}

#[cfg(not(any(unix, windows)))]
fn apply_platform_permissions(_path: &Path) -> Result<(), ProtectedPathError> {
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn verify_platform_permissions(
    _path: &Path,
    _metadata: &fs::Metadata,
) -> Result<(), ProtectedPathError> {
    Ok(())
}

#[cfg(not(any(unix, windows)))]
const fn platform_protection_level() -> ProtectionLevel {
    ProtectionLevel::PathIsolationOnly
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_root() -> PathBuf {
        let counter = TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "golam-paths-{}-{counter}",
            std::process::id()
        ))
    }

    #[test]
    fn layout_creates_only_expected_directories() {
        let root = unique_root();
        let layout = RuntimeLayout::initialize(&root).unwrap();
        assert!(layout.data_dir.is_dir());
        assert!(layout.runtime_dir.is_dir());
        assert!(layout.artifact_dir.is_dir());
        assert!(layout.artifact_tmp_dir.is_dir());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn unix_layout_is_user_only() {
        use std::os::unix::fs::PermissionsExt;

        let root = unique_root();
        let layout = RuntimeLayout::initialize(&root).unwrap();
        layout.require_authority_ready().unwrap();
        let mode = fs::metadata(&layout.runtime_dir)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o700);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn windows_authority_use_remains_fail_closed_until_acl_verification() {
        let root = unique_root();
        let layout = RuntimeLayout::initialize(&root).unwrap();
        assert!(matches!(
            layout.require_authority_ready(),
            Err(ProtectedPathError::AuthorityProtectionUnverified)
        ));
        fs::remove_dir_all(root).unwrap();
    }
}
