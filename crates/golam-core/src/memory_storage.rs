#![forbid(unsafe_code)]

use core::fmt;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::digest::sha256;
use crate::memory::{MemoryItemId, MemoryScope, MemoryStoreId};
use crate::paths::RuntimeLayout;
use crate::tool_request::BindingDigest;
use crate::{CanonicalEncoder, CoreError};

pub const MEMORY_OPERATIONAL_SCHEMA_VERSION: u16 = 1;
const MEMORY_SCHEMA_DOMAIN: &[u8] = b"golam:memory-operational-schema:v1";
const MEMORY_STORE_DOMAIN: &[u8] = b"golam:memory-operational-store:v1";

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MemoryVaultScope {
    User,
    Project(BindingDigest),
}

impl MemoryVaultScope {
    pub const fn memory_scope(self) -> MemoryScope {
        match self {
            Self::User => MemoryScope::User,
            Self::Project(_) => MemoryScope::Project,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryLayout {
    root: PathBuf,
    vault_dir: PathBuf,
    user_dir: PathBuf,
    projects_dir: PathBuf,
    operational_dir: PathBuf,
    operational_db_path: PathBuf,
    schema_ref: BindingDigest,
    store_id: MemoryStoreId,
}

#[derive(Debug)]
pub enum MemoryLayoutError {
    Io(io::Error),
    Core(CoreError),
    Symlink(PathBuf),
    NotDirectory(PathBuf),
    NonUnicodePath(PathBuf),
    AuthorityDatabaseAlias,
}

impl fmt::Display for MemoryLayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "memory layout I/O error: {error}"),
            Self::Core(error) => write!(f, "memory layout canonical encoding error: {error}"),
            Self::Symlink(path) => {
                write!(f, "memory layout directory is a symlink: {}", path.display())
            }
            Self::NotDirectory(path) => {
                write!(f, "memory layout path is not a directory: {}", path.display())
            }
            Self::NonUnicodePath(path) => write!(
                f,
                "memory operational path is not representable by the canonical store binding: {}",
                path.display()
            ),
            Self::AuthorityDatabaseAlias => {
                f.write_str("memory operational SQLite must not alias the authority database")
            }
        }
    }
}

impl Error for MemoryLayoutError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for MemoryLayoutError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<CoreError> for MemoryLayoutError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl MemoryLayout {
    pub fn initialize(runtime: &RuntimeLayout) -> Result<Self, MemoryLayoutError> {
        let root = runtime.data_dir.join("memory");
        let vault_dir = root.join("vault");
        let user_dir = vault_dir.join("user");
        let projects_dir = vault_dir.join("projects");
        let operational_dir = root.join(".operational");
        for path in [
            &root,
            &vault_dir,
            &user_dir,
            &projects_dir,
            &operational_dir,
        ] {
            ensure_directory(path)?;
        }
        let operational_db_path = operational_dir.join("memory.sqlite3");
        let authority_db_path = runtime.data_dir.join("authority").join("golam.db");
        if operational_db_path == authority_db_path {
            return Err(MemoryLayoutError::AuthorityDatabaseAlias);
        }
        let schema_ref = schema_ref()?;
        let store_id = MemoryStoreId(store_binding(&operational_db_path, schema_ref)?);
        Ok(Self {
            root,
            vault_dir,
            user_dir,
            projects_dir,
            operational_dir,
            operational_db_path,
            schema_ref,
            store_id,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn vault_dir(&self) -> &Path {
        &self.vault_dir
    }

    pub fn operational_dir(&self) -> &Path {
        &self.operational_dir
    }

    pub fn operational_db_path(&self) -> &Path {
        &self.operational_db_path
    }

    pub const fn schema_ref(&self) -> BindingDigest {
        self.schema_ref
    }

    pub const fn store_id(&self) -> MemoryStoreId {
        self.store_id
    }

    pub fn item_path(
        &self,
        scope: MemoryVaultScope,
        item_id: MemoryItemId,
    ) -> Result<PathBuf, MemoryLayoutError> {
        let file_name = format!("{}.md", encode_hex(&item_id.0.bytes()));
        match scope {
            MemoryVaultScope::User => Ok(self.user_dir.join(file_name)),
            MemoryVaultScope::Project(project_ref) => {
                let project_dir = self.projects_dir.join(encode_hex(&project_ref.bytes()));
                ensure_directory(&project_dir)?;
                Ok(project_dir.join(file_name))
            }
        }
    }

    pub fn is_operational_path(&self, path: &Path) -> bool {
        path == self.operational_dir || path.starts_with(&self.operational_dir)
    }
}

pub fn schema_ref() -> Result<BindingDigest, MemoryLayoutError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(MEMORY_SCHEMA_DOMAIN)?;
    encoder.push_u16(MEMORY_OPERATIONAL_SCHEMA_VERSION);
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn store_binding(
    operational_db_path: &Path,
    schema_ref: BindingDigest,
) -> Result<BindingDigest, MemoryLayoutError> {
    let path = operational_db_path
        .to_str()
        .ok_or_else(|| MemoryLayoutError::NonUnicodePath(operational_db_path.to_path_buf()))?;
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(MEMORY_STORE_DOMAIN)?;
    encoder.push_bytes(path.as_bytes())?;
    encoder.push_bytes(&schema_ref.bytes())?;
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn ensure_directory(path: &Path) -> Result<(), MemoryLayoutError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err(MemoryLayoutError::Symlink(path.to_path_buf()));
            }
            if !metadata.is_dir() {
                return Err(MemoryLayoutError::NotDirectory(path.to_path_buf()));
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => fs::create_dir(path)?,
        Err(error) => return Err(error.into()),
    }
    apply_private_permissions(path)?;
    Ok(())
}

#[cfg(unix)]
fn apply_private_permissions(path: &Path) -> Result<(), MemoryLayoutError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

#[cfg(not(unix))]
fn apply_private_permissions(_path: &Path) -> Result<(), MemoryLayoutError> {
    Ok(())
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[(byte >> 4) as usize]));
        output.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
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
            "golam-memory-layout-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    #[test]
    fn layout_separates_human_vault_from_operational_and_authority_state() {
        let runtime = runtime();
        let layout = MemoryLayout::initialize(&runtime).unwrap();
        assert_eq!(layout.root(), runtime.data_dir.join("memory"));
        assert_eq!(layout.vault_dir(), runtime.data_dir.join("memory/vault"));
        assert_eq!(
            layout.operational_db_path(),
            runtime.data_dir.join("memory/.operational/memory.sqlite3")
        );
        assert_ne!(
            layout.operational_db_path(),
            runtime.data_dir.join("authority/golam.db")
        );
        assert!(layout.is_operational_path(layout.operational_db_path()));
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn item_paths_are_stable_and_scope_separated() {
        let runtime = runtime();
        let layout = MemoryLayout::initialize(&runtime).unwrap();
        let item = MemoryItemId(BindingDigest::new([7; 32]));
        let user = layout.item_path(MemoryVaultScope::User, item).unwrap();
        let project = layout
            .item_path(
                MemoryVaultScope::Project(BindingDigest::new([8; 32])),
                item,
            )
            .unwrap();
        assert!(user.starts_with(runtime.data_dir.join("memory/vault/user")));
        assert!(project.starts_with(runtime.data_dir.join("memory/vault/projects")));
        assert_ne!(user, project);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn store_identity_binds_exact_path_and_schema() {
        let first_runtime = runtime();
        let second_runtime = runtime();
        let first = MemoryLayout::initialize(&first_runtime).unwrap();
        let same = MemoryLayout::initialize(&first_runtime).unwrap();
        let second = MemoryLayout::initialize(&second_runtime).unwrap();
        assert_eq!(first.store_id(), same.store_id());
        assert_eq!(first.schema_ref(), same.schema_ref());
        assert_ne!(first.store_id(), second.store_id());
        fs::remove_dir_all(first_runtime.root).unwrap();
        fs::remove_dir_all(second_runtime.root).unwrap();
    }
}
