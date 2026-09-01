#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use golam_core::digest::sha256;
use golam_core::target_identity::{
    AuthorizedRoot, ObservedFileKind, PlatformFamily, ResolvedTargetIdentity, TargetIdentityError,
};
use golam_core::tool_request::{BindingDigest, RequestedOperationId, RequestedTarget, ResourceClassId};
use golam_core::{CanonicalEncoder, CoreError};

const IDENTITY_DOMAIN: &[u8] = b"golam:local-fs-identity:v1";
const METADATA_DOMAIN: &[u8] = b"golam:local-fs-metadata:v1";
const MISSING_DOMAIN: &[u8] = b"golam:local-fs-missing:v1";

#[derive(Debug)]
pub enum LocalFsResolutionError {
    Io(io::Error),
    Core(CoreError),
    Contract(TargetIdentityError),
    RootNotDirectory(PathBuf),
    RootAlias(PathBuf),
    InvalidRequestedPath,
    OperationNotAuthorized,
    NonUnicodePath(PathBuf),
    AliasBoundary {
        path: PathBuf,
        identity: BindingDigest,
    },
    MountBoundary {
        path: PathBuf,
        identity: BindingDigest,
    },
    EscapesAuthorizedRoot(PathBuf),
    ProtectedResource(PathBuf),
    ProtectedRootOverlap(PathBuf),
    MissingIntermediate(PathBuf),
}

impl fmt::Display for LocalFsResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "local filesystem resolution I/O error: {error}"),
            Self::Core(error) => write!(f, "local filesystem canonical encoding error: {error}"),
            Self::Contract(error) => write!(f, "local filesystem identity contract error: {error}"),
            Self::RootNotDirectory(path) => {
                write!(f, "authorized root is not a directory: {}", path.display())
            }
            Self::RootAlias(path) => {
                write!(f, "authorized root is an alias/reparse path: {}", path.display())
            }
            Self::InvalidRequestedPath => f.write_str(
                "requested filesystem path must be a bounded relative path without parent/root/prefix components",
            ),
            Self::OperationNotAuthorized => {
                f.write_str("requested filesystem operation is not authorized for this root")
            }
            Self::NonUnicodePath(path) => write!(
                f,
                "resolved filesystem path cannot be represented by the current bounded request contract: {}",
                path.display()
            ),
            Self::AliasBoundary { path, .. } => write!(
                f,
                "filesystem resolution encountered a denied symlink/reparse/junction boundary at {}",
                path.display()
            ),
            Self::MountBoundary { path, .. } => write!(
                f,
                "filesystem resolution encountered a denied mount/device boundary at {}",
                path.display()
            ),
            Self::EscapesAuthorizedRoot(path) => write!(
                f,
                "resolved filesystem target escapes the authorized root: {}",
                path.display()
            ),
            Self::ProtectedResource(path) => write!(
                f,
                "resolved filesystem target intersects protected Golam state: {}",
                path.display()
            ),
            Self::ProtectedRootOverlap(path) => write!(
                f,
                "authorized filesystem root overlaps protected Golam state: {}",
                path.display()
            ),
            Self::MissingIntermediate(path) => write!(
                f,
                "filesystem resolution has a missing intermediate component: {}",
                path.display()
            ),
        }
    }
}

impl Error for LocalFsResolutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::Contract(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for LocalFsResolutionError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<CoreError> for LocalFsResolutionError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<TargetIdentityError> for LocalFsResolutionError {
    fn from(value: TargetIdentityError) -> Self {
        Self::Contract(value)
    }
}

#[derive(Clone, Debug)]
pub struct LocalFsResolver {
    root_path: PathBuf,
    contract: AuthorizedRoot,
    protected_paths: Vec<PathBuf>,
    #[cfg(unix)]
    root_device: u64,
}

impl LocalFsResolver {
    pub fn new(
        root: impl AsRef<Path>,
        policy_resource_class: ResourceClassId,
        allowed_operations: Vec<RequestedOperationId>,
        protected_paths: impl IntoIterator<Item = PathBuf>,
    ) -> Result<Self, LocalFsResolutionError> {
        let root = root.as_ref();
        let root_metadata = fs::symlink_metadata(root)?;
        if is_alias(&root_metadata) {
            return Err(LocalFsResolutionError::RootAlias(root.to_path_buf()));
        }
        if !root_metadata.is_dir() {
            return Err(LocalFsResolutionError::RootNotDirectory(root.to_path_buf()));
        }
        let root_path = fs::canonicalize(root)?;
        let root_metadata = fs::metadata(&root_path)?;
        let protected_paths = protected_paths
            .into_iter()
            .map(fs::canonicalize)
            .collect::<Result<Vec<_>, _>>()?;
        for protected in &protected_paths {
            if root_path.starts_with(protected) || protected.starts_with(&root_path) {
                return Err(LocalFsResolutionError::ProtectedRootOverlap(
                    protected.clone(),
                ));
            }
        }

        let contract = AuthorizedRoot {
            platform: platform_family(),
            policy_resource_class,
            resolved_root_identity: identity_digest(&root_path, &root_metadata)?,
            allowed_operations,
        };
        contract.validate()?;

        Ok(Self {
            root_path,
            contract,
            protected_paths,
            #[cfg(unix)]
            root_device: unix_device(&root_metadata),
        })
    }

    pub fn authorized_root(&self) -> &AuthorizedRoot {
        &self.contract
    }

    pub fn resolve_read_target(
        &self,
        requested: &RequestedTarget,
        operation: &RequestedOperationId,
        observed_at_unix_ms: u64,
    ) -> Result<ResolvedTargetIdentity, LocalFsResolutionError> {
        if self
            .contract
            .allowed_operations
            .binary_search(operation)
            .is_err()
        {
            return Err(LocalFsResolutionError::OperationNotAuthorized);
        }

        let relative = normalize_relative(requested.as_str())?;
        let components = relative.components().collect::<Vec<_>>();
        let mut current = self.root_path.clone();
        let mut final_missing = false;

        for (index, component) in components.iter().enumerate() {
            let Component::Normal(segment) = component else {
                return Err(LocalFsResolutionError::InvalidRequestedPath);
            };
            current.push(segment);
            match fs::symlink_metadata(&current) {
                Ok(metadata) => {
                    let boundary_identity = identity_digest(&current, &metadata)?;
                    if is_alias(&metadata) {
                        return Err(LocalFsResolutionError::AliasBoundary {
                            path: current,
                            identity: boundary_identity,
                        });
                    }
                    #[cfg(unix)]
                    if unix_device(&metadata) != self.root_device {
                        return Err(LocalFsResolutionError::MountBoundary {
                            path: current,
                            identity: boundary_identity,
                        });
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    if index + 1 != components.len() {
                        return Err(LocalFsResolutionError::MissingIntermediate(current));
                    }
                    final_missing = true;
                    break;
                }
                Err(error) => return Err(error.into()),
            }
        }

        if final_missing {
            self.resolve_missing(requested, current, observed_at_unix_ms)
        } else {
            self.resolve_existing(requested, current, observed_at_unix_ms)
        }
    }

    fn resolve_existing(
        &self,
        requested: &RequestedTarget,
        candidate: PathBuf,
        observed_at_unix_ms: u64,
    ) -> Result<ResolvedTargetIdentity, LocalFsResolutionError> {
        let canonical = fs::canonicalize(&candidate)?;
        self.require_authorized_path(&canonical)?;
        let metadata = fs::metadata(&canonical)?;
        let normalized_path = requested_from_path(&canonical)?;
        let parent_identity = canonical
            .parent()
            .filter(|parent| *parent != self.root_path.as_path())
            .map(|parent| {
                let metadata = fs::metadata(parent)?;
                identity_digest(parent, &metadata)
            })
            .transpose()?;
        let identity = ResolvedTargetIdentity {
            platform: platform_family(),
            requested_path: requested.clone(),
            normalized_path,
            resolved_parent_identity: parent_identity,
            resolved_target_identity: Some(identity_digest(&canonical, &metadata)?),
            file_kind: classify(&metadata),
            symlink_or_reparse_chain: vec![],
            observed_metadata_digest: metadata_digest(&canonical, &metadata)?,
            observed_at_unix_ms,
        };
        identity.validate()?;
        Ok(identity)
    }

    fn resolve_missing(
        &self,
        requested: &RequestedTarget,
        candidate: PathBuf,
        observed_at_unix_ms: u64,
    ) -> Result<ResolvedTargetIdentity, LocalFsResolutionError> {
        let parent = candidate
            .parent()
            .ok_or(LocalFsResolutionError::InvalidRequestedPath)?;
        let parent = fs::canonicalize(parent)?;
        self.require_authorized_path(&parent)?;
        let parent_metadata = fs::metadata(&parent)?;
        if !parent_metadata.is_dir() {
            return Err(LocalFsResolutionError::RootNotDirectory(parent));
        }
        let normalized_path = requested_from_path(&candidate)?;
        let parent_identity = identity_digest(&parent, &parent_metadata)?;
        let identity = ResolvedTargetIdentity {
            platform: platform_family(),
            requested_path: requested.clone(),
            normalized_path,
            resolved_parent_identity: Some(parent_identity),
            resolved_target_identity: None,
            file_kind: ObservedFileKind::Missing,
            symlink_or_reparse_chain: vec![],
            observed_metadata_digest: missing_digest(&candidate, parent_identity)?,
            observed_at_unix_ms,
        };
        identity.validate()?;
        Ok(identity)
    }

    fn require_authorized_path(&self, path: &Path) -> Result<(), LocalFsResolutionError> {
        if !path.starts_with(&self.root_path) {
            return Err(LocalFsResolutionError::EscapesAuthorizedRoot(
                path.to_path_buf(),
            ));
        }
        if self
            .protected_paths
            .iter()
            .any(|protected| path.starts_with(protected) || protected.starts_with(path))
        {
            return Err(LocalFsResolutionError::ProtectedResource(
                path.to_path_buf(),
            ));
        }
        Ok(())
    }
}

fn normalize_relative(value: &str) -> Result<PathBuf, LocalFsResolutionError> {
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(LocalFsResolutionError::InvalidRequestedPath);
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(segment) => normalized.push(segment),
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(LocalFsResolutionError::InvalidRequestedPath);
            }
        }
    }
    Ok(normalized)
}

fn requested_from_path(path: &Path) -> Result<RequestedTarget, LocalFsResolutionError> {
    let value = path
        .to_str()
        .ok_or_else(|| LocalFsResolutionError::NonUnicodePath(path.to_path_buf()))?;
    RequestedTarget::new(value).map_err(|_| LocalFsResolutionError::InvalidRequestedPath)
}

fn identity_digest(path: &Path, metadata: &fs::Metadata) -> Result<BindingDigest, LocalFsResolutionError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(IDENTITY_DOMAIN)?;
    encoder.push_bytes(&path_bytes(path))?;
    push_platform_metadata(&mut encoder, metadata)?;
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn metadata_digest(path: &Path, metadata: &fs::Metadata) -> Result<BindingDigest, LocalFsResolutionError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(METADATA_DOMAIN)?;
    encoder.push_bytes(&path_bytes(path))?;
    encoder.push_u8(file_kind_code(classify(metadata)));
    encoder.push_u64(metadata.len());
    push_platform_metadata(&mut encoder, metadata)?;
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn missing_digest(path: &Path, parent_identity: BindingDigest) -> Result<BindingDigest, LocalFsResolutionError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(MISSING_DOMAIN)?;
    encoder.push_bytes(&path_bytes(path))?;
    encoder.push_bytes(&parent_identity.bytes())?;
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn classify(metadata: &fs::Metadata) -> ObservedFileKind {
    let kind = metadata.file_type();
    if is_alias(metadata) {
        ObservedFileKind::SymlinkOrReparsePoint
    } else if kind.is_file() {
        ObservedFileKind::RegularFile
    } else if kind.is_dir() {
        ObservedFileKind::Directory
    } else {
        ObservedFileKind::Special
    }
}

const fn file_kind_code(kind: ObservedFileKind) -> u8 {
    match kind {
        ObservedFileKind::Missing => 1,
        ObservedFileKind::RegularFile => 2,
        ObservedFileKind::Directory => 3,
        ObservedFileKind::SymlinkOrReparsePoint => 4,
        ObservedFileKind::Special => 5,
    }
}

#[cfg(unix)]
fn platform_family() -> PlatformFamily {
    PlatformFamily::Unix
}

#[cfg(windows)]
fn platform_family() -> PlatformFamily {
    PlatformFamily::Windows
}

#[cfg(unix)]
fn is_alias(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(windows)]
fn is_alias(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    metadata.file_type().is_symlink()
        || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(unix)]
fn unix_device(metadata: &fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    metadata.dev()
}

#[cfg(unix)]
fn push_platform_metadata(
    encoder: &mut CanonicalEncoder,
    metadata: &fs::Metadata,
) -> Result<(), CoreError> {
    use std::os::unix::fs::MetadataExt;
    encoder.push_u64(metadata.dev());
    encoder.push_u64(metadata.ino());
    encoder.push_u64(u64::from(metadata.mode()));
    encoder.push_u64(u64::from(metadata.uid()));
    encoder.push_u64(u64::from(metadata.gid()));
    encoder.push_u64(metadata.len());
    Ok(())
}

#[cfg(windows)]
fn push_platform_metadata(
    encoder: &mut CanonicalEncoder,
    metadata: &fs::Metadata,
) -> Result<(), CoreError> {
    use std::os::windows::fs::MetadataExt;
    encoder.push_u64(u64::from(metadata.file_attributes()));
    encoder.push_u64(metadata.file_size());
    encoder.push_u64(metadata.creation_time());
    encoder.push_u64(metadata.last_write_time());
    Ok(())
}

#[cfg(unix)]
fn path_bytes(path: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    path.as_os_str().as_bytes().to_vec()
}

#[cfg(windows)]
fn path_bytes(path: &Path) -> Vec<u8> {
    use std::os::windows::ffi::OsStrExt;
    path.as_os_str()
        .encode_wide()
        .flat_map(u16::to_le_bytes)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
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
            "golam-local-fs-{}-{nanos}-{counter}",
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

    #[test]
    fn resolves_regular_file_under_exact_root() {
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        fs::write(root.join("note.txt"), b"hello").unwrap();
        let resolver = resolver(&root);
        let identity = resolver
            .resolve_read_target(
                &RequestedTarget::new("note.txt").unwrap(),
                &RequestedOperationId::new("read").unwrap(),
                10,
            )
            .unwrap();
        assert_eq!(identity.file_kind, ObservedFileKind::RegularFile);
        assert!(identity.resolved_target_identity.is_some());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn parent_escape_and_unauthorized_operation_fail_closed() {
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        let resolver = resolver(&root);
        assert!(matches!(
            resolver.resolve_read_target(
                &RequestedTarget::new("../escape").unwrap(),
                &RequestedOperationId::new("read").unwrap(),
                10,
            ),
            Err(LocalFsResolutionError::InvalidRequestedPath)
        ));
        assert!(matches!(
            resolver.resolve_read_target(
                &RequestedTarget::new("missing.txt").unwrap(),
                &RequestedOperationId::new("write").unwrap(),
                10,
            ),
            Err(LocalFsResolutionError::OperationNotAuthorized)
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn root_cannot_overlap_protected_golam_state() {
        let root = unique_root();
        let protected = root.join(".golam-protected");
        fs::create_dir_all(&protected).unwrap();
        assert!(matches!(
            LocalFsResolver::new(
                &root,
                ResourceClassId::new("workspace.read").unwrap(),
                vec![RequestedOperationId::new("read").unwrap()],
                [protected],
            ),
            Err(LocalFsResolutionError::ProtectedRootOverlap(_))
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn unix_symlink_boundary_is_denied_with_identity_evidence() {
        use std::os::unix::fs::symlink;

        let root = unique_root();
        fs::create_dir(&root).unwrap();
        fs::write(root.join("real.txt"), b"hello").unwrap();
        symlink(root.join("real.txt"), root.join("alias.txt")).unwrap();
        let resolver = resolver(&root);
        assert!(matches!(
            resolver.resolve_read_target(
                &RequestedTarget::new("alias.txt").unwrap(),
                &RequestedOperationId::new("read").unwrap(),
                10,
            ),
            Err(LocalFsResolutionError::AliasBoundary { .. })
        ));
        fs::remove_dir_all(root).unwrap();
    }
}
