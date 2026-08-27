use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::paths::{ProtectedPathError, RuntimeLayout};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityLayout {
    root: PathBuf,
    credential_dir: PathBuf,
    db_path: PathBuf,
}

#[derive(Debug)]
pub enum AuthorityPathError {
    ProtectedPath(ProtectedPathError),
    Io(io::Error),
    Symlink(PathBuf),
    NotDirectory(PathBuf),
    NotRegularFile(PathBuf),
    PermissionsTooBroad { path: PathBuf, mode: u32 },
    WindowsAclMissing(PathBuf),
    WindowsAclMismatch(PathBuf),
    WindowsAclNotProtected(PathBuf),
    InvalidCredentialPath(PathBuf),
}

impl fmt::Display for AuthorityPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProtectedPath(error) => write!(f, "authority path protection error: {error}"),
            Self::Io(error) => write!(f, "authority path I/O error: {error}"),
            Self::Symlink(path) => write!(f, "authority path is a symlink: {}", path.display()),
            Self::NotDirectory(path) => {
                write!(f, "authority path is not a directory: {}", path.display())
            }
            Self::NotRegularFile(path) => write!(
                f,
                "authority credential is not a regular file: {}",
                path.display()
            ),
            Self::PermissionsTooBroad { path, mode } => write!(
                f,
                "authority path permissions are too broad: {} mode {mode:o}",
                path.display()
            ),
            Self::WindowsAclMissing(path) => {
                write!(f, "authority Windows path has no DACL: {}", path.display())
            }
            Self::WindowsAclMismatch(path) => write!(
                f,
                "authority Windows path DACL is not current-user-only: {}",
                path.display()
            ),
            Self::WindowsAclNotProtected(path) => write!(
                f,
                "authority Windows path DACL is not protected: {}",
                path.display()
            ),
            Self::InvalidCredentialPath(path) => write!(
                f,
                "credential path escapes the canonical credential directory: {}",
                path.display()
            ),
        }
    }
}

impl Error for AuthorityPathError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ProtectedPath(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ProtectedPathError> for AuthorityPathError {
    fn from(value: ProtectedPathError) -> Self {
        Self::ProtectedPath(value)
    }
}

impl From<io::Error> for AuthorityPathError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl AuthorityLayout {
    pub fn initialize(runtime: &RuntimeLayout) -> Result<Self, AuthorityPathError> {
        runtime.require_authority_ready()?;
        let root = runtime.data_dir.join("authority");
        let credential_dir = root.join("client-credentials");
        ensure_private_directory(&root)?;
        ensure_private_directory(&credential_dir)?;
        let db_path = root.join("golam.db");
        Ok(Self {
            root,
            credential_dir,
            db_path,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn credential_dir(&self) -> &Path {
        &self.credential_dir
    }
    pub fn authority_db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn credential_path(
        &self,
        client_id: u128,
        key_id: &[u8; 32],
    ) -> Result<PathBuf, AuthorityPathError> {
        let path = self
            .credential_dir
            .join(format!("{client_id:032x}-{}.gkey", encode_hex(key_id)));
        self.require_direct_credential_path(&path)?;
        Ok(path)
    }

    pub fn protect_credential_file(&self, path: &Path) -> Result<(), AuthorityPathError> {
        self.require_direct_credential_path(path)?;
        validate_regular_file(path)?;
        apply_private_file_permissions(path)?;
        verify_private_file_permissions(path)
    }

    pub fn verify_credential_file(&self, path: &Path) -> Result<(), AuthorityPathError> {
        self.require_direct_credential_path(path)?;
        validate_regular_file(path)?;
        verify_private_file_permissions(path)
    }

    pub fn is_authority_path(&self, path: &Path) -> bool {
        path == self.root || path.starts_with(&self.root)
    }

    fn require_direct_credential_path(&self, path: &Path) -> Result<(), AuthorityPathError> {
        if path.parent() == Some(self.credential_dir.as_path()) {
            Ok(())
        } else {
            Err(AuthorityPathError::InvalidCredentialPath(
                path.to_path_buf(),
            ))
        }
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(HEX[(byte >> 4) as usize]));
        out.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    out
}

fn validate_regular_file(path: &Path) -> Result<(), AuthorityPathError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(AuthorityPathError::Symlink(path.to_path_buf()));
    }
    if !metadata.is_file() {
        return Err(AuthorityPathError::NotRegularFile(path.to_path_buf()));
    }
    Ok(())
}

fn ensure_private_directory(path: &Path) -> Result<(), AuthorityPathError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err(AuthorityPathError::Symlink(path.to_path_buf()));
            }
            if !metadata.is_dir() {
                return Err(AuthorityPathError::NotDirectory(path.to_path_buf()));
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => fs::create_dir(path)?,
        Err(error) => return Err(error.into()),
    }
    apply_private_directory_permissions(path)?;
    verify_private_directory_permissions(path)
}

#[cfg(unix)]
fn apply_private_directory_permissions(path: &Path) -> Result<(), AuthorityPathError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}
#[cfg(unix)]
fn verify_private_directory_permissions(path: &Path) -> Result<(), AuthorityPathError> {
    use std::os::unix::fs::PermissionsExt;
    let mode = fs::symlink_metadata(path)?.permissions().mode() & 0o777;
    if mode != 0o700 {
        return Err(AuthorityPathError::PermissionsTooBroad {
            path: path.to_path_buf(),
            mode,
        });
    }
    Ok(())
}
#[cfg(unix)]
fn apply_private_file_permissions(path: &Path) -> Result<(), AuthorityPathError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}
#[cfg(unix)]
fn verify_private_file_permissions(path: &Path) -> Result<(), AuthorityPathError> {
    use std::os::unix::fs::PermissionsExt;
    let mode = fs::symlink_metadata(path)?.permissions().mode() & 0o777;
    if mode != 0o600 {
        return Err(AuthorityPathError::PermissionsTooBroad {
            path: path.to_path_buf(),
            mode,
        });
    }
    Ok(())
}

#[cfg(windows)]
fn apply_private_directory_permissions(path: &Path) -> Result<(), AuthorityPathError> {
    apply_windows_user_only_acl(path, true)
}
#[cfg(windows)]
fn verify_private_directory_permissions(path: &Path) -> Result<(), AuthorityPathError> {
    verify_windows_user_only_acl(path)
}
#[cfg(windows)]
fn apply_private_file_permissions(path: &Path) -> Result<(), AuthorityPathError> {
    apply_windows_user_only_acl(path, false)
}
#[cfg(windows)]
fn verify_private_file_permissions(path: &Path) -> Result<(), AuthorityPathError> {
    verify_windows_user_only_acl(path)
}

#[cfg(windows)]
fn apply_windows_user_only_acl(path: &Path, inheritable: bool) -> Result<(), AuthorityPathError> {
    use windows_permissions::constants::{SeObjectType, SecurityInformation};
    use windows_permissions::{LocalBox, SecurityDescriptor};
    let sid = crate::paths::windows_current_process_sid_string()?;
    let flags = if inheritable { "OICI" } else { "" };
    let descriptor: LocalBox<SecurityDescriptor> = format!("D:P(A;{flags};FA;;;{sid})").parse()?;
    let dacl = descriptor
        .dacl()
        .ok_or_else(|| AuthorityPathError::WindowsAclMissing(path.to_path_buf()))?;
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
fn verify_windows_user_only_acl(path: &Path) -> Result<(), AuthorityPathError> {
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
        .ok_or_else(|| AuthorityPathError::WindowsAclMissing(path.to_path_buf()))?;
    if dacl.len() != 1 {
        return Err(AuthorityPathError::WindowsAclMismatch(path.to_path_buf()));
    }
    let ace = dacl
        .get_ace(0)
        .ok_or_else(|| AuthorityPathError::WindowsAclMismatch(path.to_path_buf()))?;
    if ace.ace_type() != AceType::ACCESS_ALLOWED_ACE_TYPE
        || ace.mask() != AccessRights::FileAllAccess
        || ace.sid() != Some(&*expected_sid)
    {
        return Err(AuthorityPathError::WindowsAclMismatch(path.to_path_buf()));
    }
    let sddl = windows_permissions::wrappers::ConvertSecurityDescriptorToStringSecurityDescriptor(
        &descriptor,
        SecurityInformation::Dacl,
    )?;
    if !sddl.to_string_lossy().starts_with("D:P") {
        return Err(AuthorityPathError::WindowsAclNotProtected(
            path.to_path_buf(),
        ));
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn apply_private_directory_permissions(_path: &Path) -> Result<(), AuthorityPathError> {
    Err(ProtectedPathError::AuthorityProtectionUnverified.into())
}
#[cfg(not(any(unix, windows)))]
fn verify_private_directory_permissions(_path: &Path) -> Result<(), AuthorityPathError> {
    Err(ProtectedPathError::AuthorityProtectionUnverified.into())
}
#[cfg(not(any(unix, windows)))]
fn apply_private_file_permissions(_path: &Path) -> Result<(), AuthorityPathError> {
    Err(ProtectedPathError::AuthorityProtectionUnverified.into())
}
#[cfg(not(any(unix, windows)))]
fn verify_private_file_permissions(_path: &Path) -> Result<(), AuthorityPathError> {
    Err(ProtectedPathError::AuthorityProtectionUnverified.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn runtime() -> RuntimeLayout {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        RuntimeLayout::initialize(
            std::env::temp_dir().join(format!("golam-authority-{}-{t}-{n}", std::process::id())),
        )
        .unwrap()
    }

    #[test]
    fn authority_layout_is_canonical_and_private() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        assert_eq!(authority.root(), runtime.data_dir.join("authority"));
        assert_eq!(
            authority.authority_db_path(),
            runtime.data_dir.join("authority").join("golam.db")
        );
        assert!(authority.credential_dir().is_dir());
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn credential_files_are_direct_children_and_private() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let path = authority.credential_path(7, &[9; 32]).unwrap();
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        authority.protect_credential_file(&path).unwrap();
        authority.verify_credential_file(&path).unwrap();
        assert!(matches!(
            authority.protect_credential_file(&authority.root().join("escape.gkey")),
            Err(AuthorityPathError::InvalidCredentialPath(_))
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
