#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use golam_core::digest::sha256;
use golam_core::target_identity::{FileMutationExpectation, ObservedFileKind};
use golam_core::tool_request::{BindingDigest, RequestedOperationId, RequestedTarget};
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use golam_kernel::PreparedToolEffect;

use crate::local_fs::{LocalFsResolutionError, LocalFsResolver};

const FILE_PRECONDITION_DOMAIN: &[u8] = b"golam:file-mutation-preconditions:v1";
const MAX_FILE_MUTATION_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileWriteMode {
    Create,
    Write,
    Replace,
}

impl FileWriteMode {
    pub const fn action(self) -> &'static str {
        match self {
            Self::Create => "file.create",
            Self::Write => "file.write",
            Self::Replace => "file.replace",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileMutationReceipt {
    pub effect_id: EffectId,
    pub action: &'static str,
    pub target_identity_ref: BindingDigest,
    pub content_digest: BindingDigest,
    pub verified_at_unix_ms: u64,
}

#[derive(Debug)]
pub enum FileMutationError {
    Io(io::Error),
    Core(CoreError),
    Resolution(LocalFsResolutionError),
    UnsupportedPlatform,
    InvalidEffectBinding,
    InvalidExpectation,
    MutationTooLarge,
    StaleParent,
    StaleTarget,
    StaleContent,
    TargetExists,
    InvalidTargetKind(ObservedFileKind),
    InvalidTargetName,
    StagingCollision(PathBuf),
    ConflictPreserved(PathBuf),
    UnknownOutcome(PathBuf),
    #[cfg(unix)]
    Unix(nix::errno::Errno),
}

impl fmt::Display for FileMutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "filesystem mutation I/O failed: {error}"),
            Self::Core(error) => write!(f, "filesystem mutation canonical encoding failed: {error}"),
            Self::Resolution(error) => write!(f, "filesystem mutation resolution failed: {error}"),
            Self::UnsupportedPlatform => f.write_str(
                "identity-preserving filesystem mutation is not qualified on this platform",
            ),
            Self::InvalidEffectBinding => {
                f.write_str("prepared tool effect does not bind this exact filesystem mutation")
            }
            Self::InvalidExpectation => {
                f.write_str("filesystem mutation expectation is incomplete or inconsistent")
            }
            Self::MutationTooLarge => f.write_str("filesystem mutation exceeds the byte bound"),
            Self::StaleParent => f.write_str("filesystem parent identity is stale"),
            Self::StaleTarget => f.write_str("filesystem target identity is stale"),
            Self::StaleContent => f.write_str("filesystem target content precondition is stale"),
            Self::TargetExists => f.write_str("filesystem create target already exists"),
            Self::InvalidTargetKind(kind) => {
                write!(f, "filesystem mutation requires a regular file, observed {kind:?}")
            }
            Self::InvalidTargetName => {
                f.write_str("filesystem mutation target name is invalid for descriptor-relative use")
            }
            Self::StagingCollision(path) => write!(
                f,
                "filesystem mutation staging entry already exists and requires reconciliation: {}",
                path.display()
            ),
            Self::ConflictPreserved(path) => write!(
                f,
                "filesystem mutation detected a concurrent conflict and preserved data at {}",
                path.display()
            ),
            Self::UnknownOutcome(path) => write!(
                f,
                "filesystem mutation completion is ambiguous and requires reconciliation at {}",
                path.display()
            ),
            #[cfg(unix)]
            Self::Unix(error) => write!(f, "filesystem mutation Unix primitive failed: {error}"),
        }
    }
}

impl Error for FileMutationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::Resolution(error) => Some(error),
            #[cfg(unix)]
            Self::Unix(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for FileMutationError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<CoreError> for FileMutationError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<LocalFsResolutionError> for FileMutationError {
    fn from(value: LocalFsResolutionError) -> Self {
        Self::Resolution(value)
    }
}

#[cfg(unix)]
impl From<nix::errno::Errno> for FileMutationError {
    fn from(value: nix::errno::Errno) -> Self {
        Self::Unix(value)
    }
}

pub fn file_mutation_resource(requested: &RequestedTarget) -> String {
    format!("file:{}", requested.as_str())
}

pub fn file_preconditions_hash(
    mode: FileWriteMode,
    requested: &RequestedTarget,
    expectation: FileMutationExpectation,
) -> Result<[u8; 32], CoreError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(FILE_PRECONDITION_DOMAIN)?;
    encoder.push_bytes(mode.action().as_bytes())?;
    encoder.push_bytes(requested.as_str().as_bytes())?;
    encoder.push_u8(u8::from(expectation.expected_exists));
    push_optional_kind(&mut encoder, expectation.expected_kind);
    push_optional_digest(&mut encoder, expectation.expected_identity)?;
    push_optional_digest(&mut encoder, expectation.expected_content_digest)?;
    match expectation.expected_size {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_u64(value);
        }
        None => encoder.push_u8(0),
    }
    push_optional_digest(&mut encoder, expectation.expected_parent_identity)?;
    Ok(sha256(&encoder.finish()))
}

pub fn execute_file_write(
    resolver: &LocalFsResolver,
    prepared: &PreparedToolEffect,
    mode: FileWriteMode,
    requested: &RequestedTarget,
    expectation: FileMutationExpectation,
    new_bytes: &[u8],
    observed_at_unix_ms: u64,
) -> Result<FileMutationReceipt, FileMutationError> {
    #[cfg(unix)]
    {
        execute_file_write_unix(
            resolver,
            prepared,
            mode,
            requested,
            expectation,
            new_bytes,
            observed_at_unix_ms,
        )
    }
    #[cfg(not(unix))]
    {
        let _ = (
            resolver,
            prepared,
            mode,
            requested,
            expectation,
            new_bytes,
            observed_at_unix_ms,
        );
        Err(FileMutationError::UnsupportedPlatform)
    }
}

#[cfg(unix)]
#[expect(
    clippy::too_many_arguments,
    reason = "mutation boundary keeps exact authority and stale-state bindings explicit"
)]
fn execute_file_write_unix(
    resolver: &LocalFsResolver,
    prepared: &PreparedToolEffect,
    mode: FileWriteMode,
    requested: &RequestedTarget,
    expectation: FileMutationExpectation,
    new_bytes: &[u8],
    observed_at_unix_ms: u64,
) -> Result<FileMutationReceipt, FileMutationError> {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::os::unix::fs::MetadataExt;

    use nix::fcntl::{AtFlags, OFlag, open, openat, renameat};
    use nix::sys::stat::Mode;
    use nix::unistd::{UnlinkatFlags, linkat, unlinkat};

    use crate::local_fs::metadata_matches_resolved_identity;

    expectation
        .validate()
        .map_err(|_| FileMutationError::InvalidExpectation)?;
    if new_bytes.len() > MAX_FILE_MUTATION_BYTES {
        return Err(FileMutationError::MutationTooLarge);
    }
    if expectation.expected_parent_identity.is_none() {
        return Err(FileMutationError::InvalidExpectation);
    }
    match mode {
        FileWriteMode::Create if expectation.expected_exists => {
            return Err(FileMutationError::InvalidExpectation);
        }
        FileWriteMode::Write | FileWriteMode::Replace if !expectation.expected_exists => {
            return Err(FileMutationError::InvalidExpectation);
        }
        _ => {}
    }
    if matches!(mode, FileWriteMode::Write | FileWriteMode::Replace)
        && expectation.expected_content_digest.is_none()
    {
        return Err(FileMutationError::InvalidExpectation);
    }

    let expected_preconditions = file_preconditions_hash(mode, requested, expectation)?;
    let expected_payload = sha256(new_bytes);
    if prepared.action() != mode.action()
        || prepared.resource() != file_mutation_resource(requested)
        || prepared.preconditions_hash() != expected_preconditions
        || prepared.payload_hash() != expected_payload
    {
        return Err(FileMutationError::InvalidEffectBinding);
    }

    let operation = RequestedOperationId::new(mode.action())
        .map_err(|_| FileMutationError::InvalidEffectBinding)?;
    let resolved = resolver.resolve_read_target(requested, &operation, observed_at_unix_ms)?;
    validate_observed_target(&resolved, expectation)?;

    let parent_request = parent_request(requested)?;
    let parent = resolver.resolve_read_target(&parent_request, &operation, observed_at_unix_ms)?;
    if parent.file_kind != ObservedFileKind::Directory
        || parent.resolved_target_identity != expectation.expected_parent_identity
    {
        return Err(FileMutationError::StaleParent);
    }
    let parent_path = Path::new(parent.normalized_path.as_str());
    let parent_fd = open(
        parent_path,
        OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )?;
    let parent_file = File::from(parent_fd);
    if !metadata_matches_resolved_identity(&parent, &parent_file.metadata()?)? {
        return Err(FileMutationError::StaleParent);
    }

    let target_name = target_name(requested)?;
    match mode {
        FileWriteMode::Create => {
            if resolved.file_kind != ObservedFileKind::Missing {
                return Err(FileMutationError::TargetExists);
            }
            let created_fd = openat(
                &parent_file,
                target_name,
                OFlag::O_RDWR
                    | OFlag::O_CREAT
                    | OFlag::O_EXCL
                    | OFlag::O_CLOEXEC
                    | OFlag::O_NOFOLLOW,
                Mode::from_bits_truncate(0o600),
            )?;
            let mut created = File::from(created_fd);
            created.write_all(new_bytes)?;
            created.sync_all()?;
            created.seek(SeekFrom::Start(0))?;
            let mut readback = Vec::with_capacity(new_bytes.len());
            created.read_to_end(&mut readback)?;
            if sha256(&readback) != expected_payload {
                return Err(FileMutationError::UnknownOutcome(PathBuf::from(
                    resolved.normalized_path.as_str(),
                )));
            }

            let observed_fd = openat(
                &parent_file,
                target_name,
                OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
                Mode::empty(),
            )?;
            let observed_file = File::from(observed_fd);
            if !same_unix_object(&created.metadata()?, &observed_file.metadata()?) {
                return Err(FileMutationError::UnknownOutcome(PathBuf::from(
                    resolved.normalized_path.as_str(),
                )));
            }
            parent_file.sync_all()?;
            verified_receipt(
                resolver,
                prepared,
                mode,
                requested,
                expected_payload,
                &created.metadata()?,
                observed_at_unix_ms,
            )
        }
        FileWriteMode::Write | FileWriteMode::Replace => {
            if resolved.file_kind != ObservedFileKind::RegularFile {
                return Err(FileMutationError::InvalidTargetKind(resolved.file_kind));
            }
            let current_fd = openat(
                &parent_file,
                target_name,
                OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
                Mode::empty(),
            )?;
            let mut current = File::from(current_fd);
            let current_metadata = current.metadata()?;
            if !metadata_matches_resolved_identity(&resolved, &current_metadata)? {
                return Err(FileMutationError::StaleTarget);
            }
            if current_metadata.len() > MAX_FILE_MUTATION_BYTES as u64 {
                return Err(FileMutationError::MutationTooLarge);
            }
            let mut current_bytes = Vec::with_capacity(current_metadata.len() as usize);
            current.read_to_end(&mut current_bytes)?;
            if expectation.expected_content_digest != Some(BindingDigest::new(sha256(&current_bytes))) {
                return Err(FileMutationError::StaleContent);
            }
            if let Some(size) = expectation.expected_size
                && size != current_metadata.len()
            {
                return Err(FileMutationError::StaleContent);
            }

            let token = format!("{:032x}", prepared.effect_id().0);
            let stage_name = format!(".golam-{token}.next");
            let guard_name = format!(".golam-{token}.previous");
            require_missing_at(&parent_file, &stage_name)?;
            require_missing_at(&parent_file, &guard_name)?;

            let staged_fd = openat(
                &parent_file,
                stage_name.as_str(),
                OFlag::O_RDWR
                    | OFlag::O_CREAT
                    | OFlag::O_EXCL
                    | OFlag::O_CLOEXEC
                    | OFlag::O_NOFOLLOW,
                Mode::from_bits_truncate(0o600),
            )?;
            let mut staged = File::from(staged_fd);
            staged.write_all(new_bytes)?;
            staged.sync_all()?;
            staged.seek(SeekFrom::Start(0))?;
            let mut staged_bytes = Vec::with_capacity(new_bytes.len());
            staged.read_to_end(&mut staged_bytes)?;
            if sha256(&staged_bytes) != expected_payload {
                cleanup_at(&parent_file, &stage_name);
                return Err(FileMutationError::StaleContent);
            }
            let staged_metadata = staged.metadata()?;

            renameat(&parent_file, target_name, &parent_file, guard_name.as_str())?;
            let guard_fd = match openat(
                &parent_file,
                guard_name.as_str(),
                OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
                Mode::empty(),
            ) {
                Ok(fd) => fd,
                Err(_) => {
                    return Err(FileMutationError::UnknownOutcome(parent_path.join(&guard_name)));
                }
            };
            let guard_file = File::from(guard_fd);
            if !same_unix_object(&current_metadata, &guard_file.metadata()?) {
                let _ = linkat(
                    &parent_file,
                    guard_name.as_str(),
                    &parent_file,
                    target_name,
                    AtFlags::empty(),
                );
                cleanup_at(&parent_file, &stage_name);
                return Err(FileMutationError::ConflictPreserved(parent_path.join(&guard_name)));
            }

            if let Err(_error) = linkat(
                &parent_file,
                stage_name.as_str(),
                &parent_file,
                target_name,
                AtFlags::empty(),
            ) {
                let restored = linkat(
                    &parent_file,
                    guard_name.as_str(),
                    &parent_file,
                    target_name,
                    AtFlags::empty(),
                )
                .is_ok();
                cleanup_at(&parent_file, &stage_name);
                if restored {
                    cleanup_at(&parent_file, &guard_name);
                    return Err(FileMutationError::StaleTarget);
                }
                return Err(FileMutationError::ConflictPreserved(parent_path.join(&guard_name)));
            }

            let installed_fd = openat(
                &parent_file,
                target_name,
                OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
                Mode::empty(),
            )?;
            let installed = File::from(installed_fd);
            let installed_metadata = installed.metadata()?;
            let mut installed_bytes = Vec::with_capacity(new_bytes.len());
            let mut installed_reader = installed;
            installed_reader.read_to_end(&mut installed_bytes)?;
            if !same_unix_object(&staged_metadata, &installed_metadata)
                || sha256(&installed_bytes) != expected_payload
            {
                return Err(FileMutationError::UnknownOutcome(parent_path.join(&guard_name)));
            }

            unlinkat(&parent_file, stage_name.as_str(), UnlinkatFlags::NoRemoveDir)?;
            if let Err(_error) = unlinkat(
                &parent_file,
                guard_name.as_str(),
                UnlinkatFlags::NoRemoveDir,
            ) {
                return Err(FileMutationError::UnknownOutcome(parent_path.join(&guard_name)));
            }
            parent_file.sync_all()?;
            verified_receipt(
                resolver,
                prepared,
                mode,
                requested,
                expected_payload,
                &installed_metadata,
                observed_at_unix_ms,
            )
        }
    }
}

#[cfg(unix)]
fn validate_observed_target(
    resolved: &golam_core::target_identity::ResolvedTargetIdentity,
    expectation: FileMutationExpectation,
) -> Result<(), FileMutationError> {
    if expectation.expected_exists == (resolved.file_kind == ObservedFileKind::Missing) {
        return Err(if expectation.expected_exists {
            FileMutationError::StaleTarget
        } else {
            FileMutationError::TargetExists
        });
    }
    if let Some(kind) = expectation.expected_kind
        && kind != resolved.file_kind
    {
        return Err(FileMutationError::StaleTarget);
    }
    if expectation.expected_identity != resolved.resolved_target_identity {
        return Err(FileMutationError::StaleTarget);
    }
    Ok(())
}

#[cfg(unix)]
fn parent_request(requested: &RequestedTarget) -> Result<RequestedTarget, FileMutationError> {
    let parent = Path::new(requested.as_str()).parent().unwrap_or_else(|| Path::new("."));
    let value = if parent.as_os_str().is_empty() {
        "."
    } else {
        parent.to_str().ok_or(FileMutationError::InvalidTargetName)?
    };
    RequestedTarget::new(value).map_err(|_| FileMutationError::InvalidTargetName)
}

#[cfg(unix)]
fn target_name(requested: &RequestedTarget) -> Result<&str, FileMutationError> {
    Path::new(requested.as_str())
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty() && *value != "." && *value != "..")
        .ok_or(FileMutationError::InvalidTargetName)
}

#[cfg(unix)]
fn require_missing_at(
    parent: &std::fs::File,
    name: &str,
) -> Result<(), FileMutationError> {
    use nix::fcntl::{OFlag, openat};
    use nix::sys::stat::Mode;

    match openat(
        parent,
        name,
        OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    ) {
        Ok(_) => Err(FileMutationError::StagingCollision(PathBuf::from(name))),
        Err(nix::errno::Errno::ENOENT) => Ok(()),
        Err(error) => Err(FileMutationError::Unix(error)),
    }
}

#[cfg(unix)]
fn cleanup_at(parent: &std::fs::File, name: &str) {
    let _ = nix::unistd::unlinkat(parent, name, nix::unistd::UnlinkatFlags::NoRemoveDir);
}

#[cfg(unix)]
fn same_unix_object(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(unix)]
fn verified_receipt(
    resolver: &LocalFsResolver,
    prepared: &PreparedToolEffect,
    mode: FileWriteMode,
    requested: &RequestedTarget,
    expected_payload: [u8; 32],
    expected_metadata: &std::fs::Metadata,
    observed_at_unix_ms: u64,
) -> Result<FileMutationReceipt, FileMutationError> {
    use std::fs::File;
    use std::io::Read;

    use nix::fcntl::{OFlag, openat};
    use nix::sys::stat::Mode;

    let operation = RequestedOperationId::new(mode.action())
        .map_err(|_| FileMutationError::InvalidEffectBinding)?;
    let verified = resolver.resolve_read_target(requested, &operation, observed_at_unix_ms)?;
    if verified.file_kind != ObservedFileKind::RegularFile {
        return Err(FileMutationError::UnknownOutcome(PathBuf::from(
            verified.normalized_path.as_str(),
        )));
    }
    let parent_request = parent_request(requested)?;
    let parent = resolver.resolve_read_target(&parent_request, &operation, observed_at_unix_ms)?;
    let parent_file = File::open(parent.normalized_path.as_str())?;
    let name = target_name(requested)?;
    let fd = openat(
        &parent_file,
        name,
        OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )?;
    let mut file = File::from(fd);
    let metadata = file.metadata()?;
    if !same_unix_object(expected_metadata, &metadata) {
        return Err(FileMutationError::UnknownOutcome(PathBuf::from(
            verified.normalized_path.as_str(),
        )));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut bytes)?;
    if sha256(&bytes) != expected_payload {
        return Err(FileMutationError::UnknownOutcome(PathBuf::from(
            verified.normalized_path.as_str(),
        )));
    }
    let target_identity_ref = verified
        .resolved_target_identity
        .ok_or(FileMutationError::StaleTarget)?;
    Ok(FileMutationReceipt {
        effect_id: prepared.effect_id(),
        action: mode.action(),
        target_identity_ref,
        content_digest: BindingDigest::new(expected_payload),
        verified_at_unix_ms: observed_at_unix_ms,
    })
}

fn push_optional_digest(
    encoder: &mut CanonicalEncoder,
    digest: Option<BindingDigest>,
) -> Result<(), CoreError> {
    match digest {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(&value.bytes())?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn push_optional_kind(encoder: &mut CanonicalEncoder, kind: Option<ObservedFileKind>) {
    encoder.push_u8(match kind {
        None => 0,
        Some(ObservedFileKind::Missing) => 1,
        Some(ObservedFileKind::RegularFile) => 2,
        Some(ObservedFileKind::Directory) => 3,
        Some(ObservedFileKind::SymlinkOrReparsePoint) => 4,
        Some(ObservedFileKind::Special) => 5,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    #[test]
    fn precondition_hash_binds_action_target_parent_target_and_content() {
        let target = RequestedTarget::new("src/lib.rs").unwrap();
        let expectation = FileMutationExpectation {
            expected_exists: true,
            expected_kind: Some(ObservedFileKind::RegularFile),
            expected_identity: Some(digest(2)),
            expected_content_digest: Some(digest(3)),
            expected_size: Some(4),
            expected_parent_identity: Some(digest(1)),
        };
        let base = file_preconditions_hash(FileWriteMode::Replace, &target, expectation).unwrap();
        assert_ne!(
            base,
            file_preconditions_hash(FileWriteMode::Write, &target, expectation).unwrap()
        );
        let mut changed = expectation;
        changed.expected_content_digest = Some(digest(9));
        assert_ne!(
            base,
            file_preconditions_hash(FileWriteMode::Replace, &target, changed).unwrap()
        );
    }

    #[cfg(not(unix))]
    #[test]
    fn unsupported_platform_is_explicit_denial() {
        assert_eq!(
            FileMutationError::UnsupportedPlatform.to_string(),
            "identity-preserving filesystem mutation is not qualified on this platform"
        );
    }
}
