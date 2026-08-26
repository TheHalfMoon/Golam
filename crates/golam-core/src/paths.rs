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
    PermissionsTooBroad {
        path: PathBuf,
        mode: u32,
    },
    OwnershipMismatch {
        path: PathBuf,
        expected: u32,
        actual: u32,
    },
    WindowsAclMissing(PathBuf),
    WindowsAclMismatch(PathBuf),
    WindowsAclNotProtected(PathBuf),
    InvalidWindowsSid,
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
            Self::OwnershipMismatch {
                path,
                expected,
                actual,
            } => write!(
                f,
                "protected Unix path owner mismatch at {}: expected uid {expected}, actual uid {actual}",
                path.display()
            ),
            Self::WindowsAclMissing(path) => {
                write!(
                    f,
                    "protected Windows directory has no DACL: {}",
                    path.display()
                )
            }
            Self::WindowsAclMismatch(path) => write!(
                f,
                "protected Windows directory DACL is not current-user-only: {}",
                path.display()
            ),
            Self::WindowsAclNotProtected(path) => write!(
                f,
                "protected Windows directory DACL still permits inheritance: {}",
                path.display()
            ),
            Self::InvalidWindowsSid => f.write_str("current Windows process SID is invalid"),
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
            | Self::OwnershipMismatch { .. }
            | Self::WindowsAclMissing(_)
            | Self::WindowsAclMismatch(_)
            | Self::WindowsAclNotProtected(_)
            | Self::InvalidWindowsSid
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
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let expected = nix::unistd::Uid::effective().as_raw();
    let actual = metadata.uid();
    if actual != expected {
        return Err(ProtectedPathError::OwnershipMismatch {
            path: path.to_path_buf(),
            expected,
            actual,
        });
    }
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
pub fn windows_current_process_sid_string() -> Result<String, ProtectedPathError> {
    let sid = windows_permissions::utilities::current_process_sid()?;
    let sid_string = windows_permissions::wrappers::ConvertSidToStringSid(&sid)?;
    let sid_string = sid_string.to_string_lossy().into_owned();
    if !sid_string.starts_with("S-1-") || sid_string.contains([';', '(', ')']) {
        return Err(ProtectedPathError::InvalidWindowsSid);
    }
    Ok(sid_string)
}

#[cfg(windows)]
fn apply_platform_permissions(path: &Path) -> Result<(), ProtectedPathError> {
    use windows_permissions::constants::{SeObjectType, SecurityInformation};
    use windows_permissions::{LocalBox, SecurityDescriptor};

    let sid = windows_current_process_sid_string()?;
    let descriptor: LocalBox<SecurityDescriptor> = format!("D:P(A;OICI;FA;;;{sid})").parse()?;
    let dacl = descriptor
        .dacl()
        .ok_or_else(|| ProtectedPathError::WindowsAclMissing(path.to_path_buf()))?;
    windows_permissions::wrappers::SetNamedSecurityInfo(
        path.as_os_str(),
        SeObjectType::SE_FILE_OBJECT,
        SecurityInformation::Dacl | SecurityInformation::ProtectedDacl,
        None,
        None,
        Some(dacl),
        None,
    )?;
    Ok(())
}

#[cfg(windows)]
fn verify_platform_permissions(
    path: &Path,
    _metadata: &fs::Metadata,
) -> Result<(), ProtectedPathError> {
    use windows_permissions::constants::{
        AccessRights, AceType, SeObjectType, SecurityInformation,
    };

    let expected_sid = windows_permissions::utilities::current_process_sid()?;
    let descriptor = windows_permissions::wrappers::GetNamedSecurityInfo(
        path.as_os_str(),
        SeObjectType::SE_FILE_OBJECT,
        SecurityInformation::Dacl,
    )?;
    let dacl = descriptor
        .dacl()
        .ok_or_else(|| ProtectedPathError::WindowsAclMissing(path.to_path_buf()))?;
    if dacl.len() != 1 {
        return Err(ProtectedPathError::WindowsAclMismatch(path.to_path_buf()));
    }
    let ace = dacl
        .get_ace(0)
        .ok_or_else(|| ProtectedPathError::WindowsAclMismatch(path.to_path_buf()))?;
    if ace.ace_type() != AceType::ACCESS_ALLOWED_ACE_TYPE
        || ace.mask() != AccessRights::FileAllAccess
        || ace.sid() != Some(&*expected_sid)
    {
        return Err(ProtectedPathError::WindowsAclMismatch(path.to_path_buf()));
    }

    let sddl = windows_permissions::wrappers::ConvertSecurityDescriptorToStringSecurityDescriptor(
        &descriptor,
        SecurityInformation::Dacl,
    )?;
    if !sddl.to_string_lossy().starts_with("D:P") {
        return Err(ProtectedPathError::WindowsAclNotProtected(
            path.to_path_buf(),
        ));
    }
    Ok(())
}

#[cfg(windows)]
const fn platform_protection_level() -> ProtectionLevel {
    ProtectionLevel::UserOnlyVerified
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
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_root() -> PathBuf {
        let counter = TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "golam-paths-{}-{nanos}-{counter}",
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
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        let root = unique_root();
        let layout = RuntimeLayout::initialize(&root).unwrap();
        layout.require_authority_ready().unwrap();
        let metadata = fs::metadata(&layout.runtime_dir).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, 0o700);
        assert_eq!(metadata.uid(), nix::unistd::Uid::effective().as_raw());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn windows_layout_is_current_user_only_and_authority_ready() {
        let root = unique_root();
        let layout = RuntimeLayout::initialize(&root).unwrap();
        layout.require_authority_ready().unwrap();
        for path in [
            &layout.root,
            &layout.data_dir,
            &layout.runtime_dir,
            &layout.artifact_dir,
            &layout.artifact_tmp_dir,
        ] {
            let metadata = fs::symlink_metadata(path).unwrap();
            verify_platform_permissions(path, &metadata).unwrap();
        }
        assert!(
            windows_current_process_sid_string()
                .unwrap()
                .starts_with("S-1-")
        );
        fs::remove_dir_all(root).unwrap();
    }
}
