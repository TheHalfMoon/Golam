#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnprivilegedPath {
    path: PathBuf,
}

impl UnprivilegedPath {
    pub fn as_path(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug)]
pub enum ProtectedResourceError {
    Io(io::Error),
    OutsideRuntime(PathBuf),
    ParentTraversal(PathBuf),
    AuthorityReserved(PathBuf),
    Symlink(PathBuf),
}

impl fmt::Display for ProtectedResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "unprivileged storage path I/O error: {error}"),
            Self::OutsideRuntime(path) => write!(
                f,
                "unprivileged storage path is outside the Golam runtime root: {}",
                path.display()
            ),
            Self::ParentTraversal(path) => write!(
                f,
                "unprivileged storage path contains parent traversal: {}",
                path.display()
            ),
            Self::AuthorityReserved(path) => write!(
                f,
                "unprivileged storage path targets kernel-reserved authority state: {}",
                path.display()
            ),
            Self::Symlink(path) => write!(
                f,
                "unprivileged storage path traverses a symlink: {}",
                path.display()
            ),
        }
    }
}

impl Error for ProtectedResourceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::OutsideRuntime(_)
            | Self::ParentTraversal(_)
            | Self::AuthorityReserved(_)
            | Self::Symlink(_) => None,
        }
    }
}

impl From<io::Error> for ProtectedResourceError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub(crate) fn admit_unprivileged_path(
    runtime: &RuntimeLayout,
    authority: &AuthorityLayout,
    path: impl AsRef<Path>,
) -> Result<UnprivilegedPath, ProtectedResourceError> {
    let path = path.as_ref();
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(ProtectedResourceError::ParentTraversal(path.to_path_buf()));
    }
    let relative = path
        .strip_prefix(&runtime.root)
        .map_err(|_| ProtectedResourceError::OutsideRuntime(path.to_path_buf()))?;
    if authority.is_authority_path(path) {
        return Err(ProtectedResourceError::AuthorityReserved(
            path.to_path_buf(),
        ));
    }
    reject_symlink_components(&runtime.root, relative)?;
    Ok(UnprivilegedPath {
        path: path.to_path_buf(),
    })
}

fn reject_symlink_components(root: &Path, relative: &Path) -> Result<(), ProtectedResourceError> {
    let root_metadata = fs::symlink_metadata(root)?;
    if root_metadata.file_type().is_symlink() {
        return Err(ProtectedResourceError::Symlink(root.to_path_buf()));
    }

    let mut current = root.to_path_buf();
    for component in relative.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(ProtectedResourceError::OutsideRuntime(
                root.join(relative),
            ));
        }
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(ProtectedResourceError::Symlink(current));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => break,
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::authority::AuthorityLayout;
    use golam_core::paths::RuntimeLayout;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    fn layouts() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-kernel-resource-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    #[test]
    fn ordinary_artifact_path_is_admitted() {
        let (runtime, authority) = layouts();
        let path = runtime.artifact_dir.join("example.txt");
        assert_eq!(
            admit_unprivileged_path(&runtime, &authority, &path)
                .unwrap()
                .as_path(),
            path
        );
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn authority_database_credentials_and_subtree_are_rejected() {
        let (runtime, authority) = layouts();
        for path in [
            authority.root().to_path_buf(),
            authority.authority_db_path().to_path_buf(),
            authority.credential_dir().to_path_buf(),
            authority.root().join("audit-state"),
            authority.root().join("policy"),
        ] {
            assert!(matches!(
                admit_unprivileged_path(&runtime, &authority, path),
                Err(ProtectedResourceError::AuthorityReserved(_))
            ));
        }
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn traversal_and_external_paths_fail_closed() {
        let (runtime, authority) = layouts();
        let traversal = runtime.artifact_dir.join("..").join("authority");
        assert!(matches!(
            admit_unprivileged_path(&runtime, &authority, traversal),
            Err(ProtectedResourceError::ParentTraversal(_))
        ));
        assert!(matches!(
            admit_unprivileged_path(&runtime, &authority, std::env::temp_dir().join("outside")),
            Err(ProtectedResourceError::OutsideRuntime(_))
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn symlink_component_cannot_escape_to_authority() {
        use std::os::unix::fs::symlink;

        let (runtime, authority) = layouts();
        let link = runtime.artifact_dir.join("authority-link");
        symlink(authority.root(), &link).unwrap();
        let candidate = link.join("golam.db");
        assert!(matches!(
            admit_unprivileged_path(&runtime, &authority, candidate),
            Err(ProtectedResourceError::Symlink(path)) if path == link
        ));
        fs::remove_file(link).unwrap();
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
