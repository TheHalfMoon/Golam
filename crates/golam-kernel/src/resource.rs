#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtectedResourceError {
    OutsideRuntime(PathBuf),
    ParentTraversal(PathBuf),
    AuthorityReserved(PathBuf),
}

impl fmt::Display for ProtectedResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
        }
    }
}

impl Error for ProtectedResourceError {}

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
    if !path.starts_with(&runtime.root) {
        return Err(ProtectedResourceError::OutsideRuntime(path.to_path_buf()));
    }
    if authority.is_authority_path(path) {
        return Err(ProtectedResourceError::AuthorityReserved(
            path.to_path_buf(),
        ));
    }
    Ok(UnprivilegedPath {
        path: path.to_path_buf(),
    })
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
}
