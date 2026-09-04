#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::io;
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;

use golam_core::digest::sha256;
use golam_core::target_identity::{FileMutationExpectation, ObservedFileKind};
#[cfg(unix)]
use golam_core::tool_request::RequestedOperationId;
use golam_core::tool_request::{BindingDigest, RequestedTarget};
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use golam_kernel::PreparedToolEffect;

use crate::local_fs::{LocalFsResolutionError, LocalFsResolver};

const RENAME_PRECONDITION_DOMAIN: &[u8] = b"golam:file-rename-preconditions:v1";
const DELETE_PRECONDITION_DOMAIN: &[u8] = b"golam:file-delete-preconditions:v1";
const RENAME_PAYLOAD_DOMAIN: &[u8] = b"golam:file-rename-payload:v1";
const DELETE_PAYLOAD_DOMAIN: &[u8] = b"golam:file-delete-payload:v1";
#[cfg(unix)]
const MAX_MUTATION_VERIFY_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathMutationReceipt {
    pub effect_id: EffectId,
    pub action: &'static str,
    pub source_identity_ref: BindingDigest,
    pub destination_identity_ref: Option<BindingDigest>,
    pub verified_at_unix_ms: u64,
}

#[derive(Debug)]
pub enum PathMutationError {
    Io(io::Error),
    Core(CoreError),
    Resolution(LocalFsResolutionError),
    UnsupportedPlatform,
    InvalidEffectBinding,
    InvalidExpectation,
    StaleSource,
    StaleParent,
    StaleContent,
    DestinationExists,
    InvalidTargetName,
    MutationTooLarge,
    GuardCollision(PathBuf),
    UnknownOutcome(PathBuf),
    #[cfg(unix)]
    Unix(nix::errno::Errno),
}

impl fmt::Display for PathMutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "filesystem path mutation I/O failed: {error}"),
            Self::Core(error) => write!(f, "filesystem path mutation encoding failed: {error}"),
            Self::Resolution(error) => {
                write!(f, "filesystem path mutation resolution failed: {error}")
            }
            Self::UnsupportedPlatform => {
                f.write_str("identity-preserving rename/delete is not qualified on this platform")
            }
            Self::InvalidEffectBinding => {
                f.write_str("prepared tool effect does not bind this exact path mutation")
            }
            Self::InvalidExpectation => {
                f.write_str("path mutation expectation is incomplete or inconsistent")
            }
            Self::StaleSource => f.write_str("path mutation source identity is stale"),
            Self::StaleParent => f.write_str("path mutation parent identity is stale"),
            Self::StaleContent => f.write_str("path mutation source content is stale"),
            Self::DestinationExists => f.write_str("rename destination already exists"),
            Self::InvalidTargetName => {
                f.write_str("path mutation target name is invalid for descriptor-relative use")
            }
            Self::MutationTooLarge => {
                f.write_str("path mutation verification exceeds the byte bound")
            }
            Self::GuardCollision(path) => {
                write!(f, "path mutation guard already exists: {}", path.display())
            }
            Self::UnknownOutcome(path) => write!(
                f,
                "path mutation completion is ambiguous and requires reconciliation at {}",
                path.display()
            ),
            #[cfg(unix)]
            Self::Unix(error) => {
                write!(f, "filesystem path mutation Unix primitive failed: {error}")
            }
        }
    }
}

impl Error for PathMutationError {
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

impl From<io::Error> for PathMutationError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<CoreError> for PathMutationError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<LocalFsResolutionError> for PathMutationError {
    fn from(value: LocalFsResolutionError) -> Self {
        Self::Resolution(value)
    }
}

#[cfg(unix)]
impl From<nix::errno::Errno> for PathMutationError {
    fn from(value: nix::errno::Errno) -> Self {
        Self::Unix(value)
    }
}

pub fn file_rename_resource(source: &RequestedTarget, destination: &RequestedTarget) -> String {
    format!("file-rename:{}->{}", source.as_str(), destination.as_str())
}

pub fn file_delete_resource(source: &RequestedTarget) -> String {
    format!("file-delete:{}", source.as_str())
}

pub fn file_rename_payload_hash(destination: &RequestedTarget) -> [u8; 32] {
    let mut bytes = Vec::with_capacity(RENAME_PAYLOAD_DOMAIN.len() + destination.as_str().len());
    bytes.extend_from_slice(RENAME_PAYLOAD_DOMAIN);
    bytes.extend_from_slice(destination.as_str().as_bytes());
    sha256(&bytes)
}

pub fn file_delete_payload_hash() -> [u8; 32] {
    sha256(DELETE_PAYLOAD_DOMAIN)
}

pub fn file_rename_preconditions_hash(
    source: &RequestedTarget,
    destination: &RequestedTarget,
    source_expectation: FileMutationExpectation,
    destination_parent_identity: BindingDigest,
) -> Result<[u8; 32], CoreError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(RENAME_PRECONDITION_DOMAIN)?;
    encoder.push_bytes(source.as_str().as_bytes())?;
    encoder.push_bytes(destination.as_str().as_bytes())?;
    push_expectation(&mut encoder, source_expectation)?;
    encoder.push_bytes(&destination_parent_identity.bytes())?;
    Ok(sha256(&encoder.finish()))
}

pub fn file_delete_preconditions_hash(
    source: &RequestedTarget,
    source_expectation: FileMutationExpectation,
) -> Result<[u8; 32], CoreError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(DELETE_PRECONDITION_DOMAIN)?;
    encoder.push_bytes(source.as_str().as_bytes())?;
    push_expectation(&mut encoder, source_expectation)?;
    Ok(sha256(&encoder.finish()))
}

pub fn execute_file_rename(
    resolver: &LocalFsResolver,
    prepared: &PreparedToolEffect,
    source: &RequestedTarget,
    destination: &RequestedTarget,
    source_expectation: FileMutationExpectation,
    destination_parent_identity: BindingDigest,
    observed_at_unix_ms: u64,
) -> Result<PathMutationReceipt, PathMutationError> {
    #[cfg(unix)]
    {
        execute_file_rename_unix(
            resolver,
            prepared,
            source,
            destination,
            source_expectation,
            destination_parent_identity,
            observed_at_unix_ms,
        )
    }
    #[cfg(not(unix))]
    {
        let _ = (
            resolver,
            prepared,
            source,
            destination,
            source_expectation,
            destination_parent_identity,
            observed_at_unix_ms,
        );
        Err(PathMutationError::UnsupportedPlatform)
    }
}

pub fn execute_file_delete(
    resolver: &LocalFsResolver,
    prepared: &PreparedToolEffect,
    source: &RequestedTarget,
    source_expectation: FileMutationExpectation,
    observed_at_unix_ms: u64,
) -> Result<PathMutationReceipt, PathMutationError> {
    #[cfg(unix)]
    {
        execute_file_delete_unix(
            resolver,
            prepared,
            source,
            source_expectation,
            observed_at_unix_ms,
        )
    }
    #[cfg(not(unix))]
    {
        let _ = (
            resolver,
            prepared,
            source,
            source_expectation,
            observed_at_unix_ms,
        );
        Err(PathMutationError::UnsupportedPlatform)
    }
}

#[cfg(unix)]
#[allow(
    clippy::too_many_arguments,
    reason = "rename boundary keeps both parent authorities and exact source state explicit"
)]
fn execute_file_rename_unix(
    resolver: &LocalFsResolver,
    prepared: &PreparedToolEffect,
    source: &RequestedTarget,
    destination: &RequestedTarget,
    source_expectation: FileMutationExpectation,
    destination_parent_identity: BindingDigest,
    observed_at_unix_ms: u64,
) -> Result<PathMutationReceipt, PathMutationError> {
    use std::fs::File;

    use nix::fcntl::{AtFlags, OFlag, open, openat};
    use nix::sys::stat::Mode;
    use nix::unistd::{UnlinkatFlags, linkat, unlinkat};

    use crate::local_fs::metadata_matches_resolved_identity;

    validate_source_expectation(source_expectation)?;
    let preconditions = file_rename_preconditions_hash(
        source,
        destination,
        source_expectation,
        destination_parent_identity,
    )?;
    if prepared.action() != "file.rename"
        || prepared.resource() != file_rename_resource(source, destination)
        || prepared.preconditions_hash() != preconditions
        || prepared.payload_hash() != file_rename_payload_hash(destination)
    {
        return Err(PathMutationError::InvalidEffectBinding);
    }

    let operation = RequestedOperationId::new("file.rename")
        .map_err(|_| PathMutationError::InvalidEffectBinding)?;
    let source_identity = resolver.resolve_read_target(source, &operation, observed_at_unix_ms)?;
    validate_source_identity(&source_identity, source_expectation)?;
    let destination_identity =
        resolver.resolve_read_target(destination, &operation, observed_at_unix_ms)?;
    if destination_identity.file_kind != ObservedFileKind::Missing {
        return Err(PathMutationError::DestinationExists);
    }
    if destination_identity.resolved_parent_identity != Some(destination_parent_identity) {
        return Err(PathMutationError::StaleParent);
    }

    let source_parent_request = parent_request(source)?;
    let source_parent =
        resolver.resolve_read_target(&source_parent_request, &operation, observed_at_unix_ms)?;
    if source_parent.resolved_target_identity != source_expectation.expected_parent_identity {
        return Err(PathMutationError::StaleParent);
    }
    let destination_parent_request = parent_request(destination)?;
    let destination_parent = resolver.resolve_read_target(
        &destination_parent_request,
        &operation,
        observed_at_unix_ms,
    )?;
    if destination_parent.resolved_target_identity != Some(destination_parent_identity) {
        return Err(PathMutationError::StaleParent);
    }

    let source_parent_fd = open(
        Path::new(source_parent.normalized_path.as_str()),
        OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )?;
    let source_parent_file = File::from(source_parent_fd);
    if !metadata_matches_resolved_identity(&source_parent, &source_parent_file.metadata()?)? {
        return Err(PathMutationError::StaleParent);
    }
    let destination_parent_fd = open(
        Path::new(destination_parent.normalized_path.as_str()),
        OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )?;
    let destination_parent_file = File::from(destination_parent_fd);
    if !metadata_matches_resolved_identity(
        &destination_parent,
        &destination_parent_file.metadata()?,
    )? {
        return Err(PathMutationError::StaleParent);
    }

    let source_name = target_name(source)?;
    let destination_name = target_name(destination)?;
    let source_fd = openat(
        &source_parent_file,
        source_name,
        OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )?;
    let mut source_file = File::from(source_fd);
    if !metadata_matches_resolved_identity(&source_identity, &source_file.metadata()?)? {
        return Err(PathMutationError::StaleSource);
    }
    verify_content(&mut source_file, source_expectation)?;

    match linkat(
        &source_parent_file,
        source_name,
        &destination_parent_file,
        destination_name,
        AtFlags::empty(),
    ) {
        Ok(()) => {}
        Err(nix::errno::Errno::EEXIST) => return Err(PathMutationError::DestinationExists),
        Err(error) => return Err(PathMutationError::Unix(error)),
    }

    let destination_fd = openat(
        &destination_parent_file,
        destination_name,
        OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )?;
    let destination_file = File::from(destination_fd);
    if !same_unix_object(&source_file.metadata()?, &destination_file.metadata()?) {
        let _ = unlinkat(
            &destination_parent_file,
            destination_name,
            UnlinkatFlags::NoRemoveDir,
        );
        return Err(PathMutationError::StaleSource);
    }

    if unlinkat(&source_parent_file, source_name, UnlinkatFlags::NoRemoveDir).is_err() {
        return Err(PathMutationError::UnknownOutcome(PathBuf::from(
            destination_identity.normalized_path.as_str(),
        )));
    }
    source_parent_file.sync_all()?;
    destination_parent_file.sync_all()?;

    let source_after = resolver.resolve_read_target(source, &operation, observed_at_unix_ms)?;
    let destination_after =
        resolver.resolve_read_target(destination, &operation, observed_at_unix_ms)?;
    if source_after.file_kind != ObservedFileKind::Missing
        || destination_after.file_kind != ObservedFileKind::RegularFile
        || destination_after.resolved_target_identity != source_identity.resolved_target_identity
    {
        return Err(PathMutationError::UnknownOutcome(PathBuf::from(
            destination_after.normalized_path.as_str(),
        )));
    }

    Ok(PathMutationReceipt {
        effect_id: prepared.effect_id(),
        action: "file.rename",
        source_identity_ref: source_identity
            .resolved_target_identity
            .ok_or(PathMutationError::StaleSource)?,
        destination_identity_ref: destination_after.resolved_target_identity,
        verified_at_unix_ms: observed_at_unix_ms,
    })
}

#[cfg(unix)]
fn execute_file_delete_unix(
    resolver: &LocalFsResolver,
    prepared: &PreparedToolEffect,
    source: &RequestedTarget,
    source_expectation: FileMutationExpectation,
    observed_at_unix_ms: u64,
) -> Result<PathMutationReceipt, PathMutationError> {
    use std::fs::File;

    use nix::fcntl::{AtFlags, OFlag, open, openat};
    use nix::sys::stat::Mode;
    use nix::unistd::{UnlinkatFlags, linkat, unlinkat};

    use crate::local_fs::metadata_matches_resolved_identity;

    validate_source_expectation(source_expectation)?;
    let preconditions = file_delete_preconditions_hash(source, source_expectation)?;
    if prepared.action() != "file.delete"
        || prepared.resource() != file_delete_resource(source)
        || prepared.preconditions_hash() != preconditions
        || prepared.payload_hash() != file_delete_payload_hash()
    {
        return Err(PathMutationError::InvalidEffectBinding);
    }

    let operation = RequestedOperationId::new("file.delete")
        .map_err(|_| PathMutationError::InvalidEffectBinding)?;
    let source_identity = resolver.resolve_read_target(source, &operation, observed_at_unix_ms)?;
    validate_source_identity(&source_identity, source_expectation)?;
    let parent_request = parent_request(source)?;
    let parent = resolver.resolve_read_target(&parent_request, &operation, observed_at_unix_ms)?;
    if parent.resolved_target_identity != source_expectation.expected_parent_identity {
        return Err(PathMutationError::StaleParent);
    }
    let parent_fd = open(
        Path::new(parent.normalized_path.as_str()),
        OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )?;
    let parent_file = File::from(parent_fd);
    if !metadata_matches_resolved_identity(&parent, &parent_file.metadata()?)? {
        return Err(PathMutationError::StaleParent);
    }

    let source_name = target_name(source)?;
    let source_fd = openat(
        &parent_file,
        source_name,
        OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )?;
    let mut source_file = File::from(source_fd);
    if !metadata_matches_resolved_identity(&source_identity, &source_file.metadata()?)? {
        return Err(PathMutationError::StaleSource);
    }
    verify_content(&mut source_file, source_expectation)?;

    let guard_name = format!(".golam-delete-{:032x}.previous", prepared.effect_id().0);
    require_missing_at(&parent_file, &guard_name)?;
    linkat(
        &parent_file,
        source_name,
        &parent_file,
        guard_name.as_str(),
        AtFlags::empty(),
    )?;
    let guard_fd = openat(
        &parent_file,
        guard_name.as_str(),
        OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    )?;
    let guard_file = File::from(guard_fd);
    if !same_unix_object(&source_file.metadata()?, &guard_file.metadata()?) {
        let _ = unlinkat(
            &parent_file,
            guard_name.as_str(),
            UnlinkatFlags::NoRemoveDir,
        );
        return Err(PathMutationError::StaleSource);
    }

    if unlinkat(&parent_file, source_name, UnlinkatFlags::NoRemoveDir).is_err() {
        let _ = unlinkat(
            &parent_file,
            guard_name.as_str(),
            UnlinkatFlags::NoRemoveDir,
        );
        return Err(PathMutationError::StaleSource);
    }
    let source_after = resolver.resolve_read_target(source, &operation, observed_at_unix_ms)?;
    if source_after.file_kind != ObservedFileKind::Missing {
        return Err(PathMutationError::UnknownOutcome(PathBuf::from(
            source_after.normalized_path.as_str(),
        )));
    }
    if unlinkat(
        &parent_file,
        guard_name.as_str(),
        UnlinkatFlags::NoRemoveDir,
    )
    .is_err()
    {
        return Err(PathMutationError::UnknownOutcome(
            Path::new(parent.normalized_path.as_str()).join(&guard_name),
        ));
    }
    parent_file.sync_all()?;

    Ok(PathMutationReceipt {
        effect_id: prepared.effect_id(),
        action: "file.delete",
        source_identity_ref: source_identity
            .resolved_target_identity
            .ok_or(PathMutationError::StaleSource)?,
        destination_identity_ref: None,
        verified_at_unix_ms: observed_at_unix_ms,
    })
}

#[cfg(unix)]
fn validate_source_expectation(
    expectation: FileMutationExpectation,
) -> Result<(), PathMutationError> {
    expectation
        .validate()
        .map_err(|_| PathMutationError::InvalidExpectation)?;
    if !expectation.expected_exists
        || expectation.expected_kind != Some(ObservedFileKind::RegularFile)
        || expectation.expected_identity.is_none()
        || expectation.expected_parent_identity.is_none()
        || expectation.expected_content_digest.is_none()
    {
        return Err(PathMutationError::InvalidExpectation);
    }
    Ok(())
}

#[cfg(unix)]
fn validate_source_identity(
    resolved: &golam_core::target_identity::ResolvedTargetIdentity,
    expectation: FileMutationExpectation,
) -> Result<(), PathMutationError> {
    if resolved.file_kind != ObservedFileKind::RegularFile
        || resolved.resolved_target_identity != expectation.expected_identity
    {
        return Err(PathMutationError::StaleSource);
    }
    Ok(())
}

#[cfg(unix)]
fn verify_content(
    file: &mut std::fs::File,
    expectation: FileMutationExpectation,
) -> Result<(), PathMutationError> {
    use std::io::{Read, Seek, SeekFrom};

    let metadata = file.metadata()?;
    if metadata.len() > MAX_MUTATION_VERIFY_BYTES as u64 {
        return Err(PathMutationError::MutationTooLarge);
    }
    if let Some(size) = expectation.expected_size
        && size != metadata.len()
    {
        return Err(PathMutationError::StaleContent);
    }
    file.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut bytes)?;
    if expectation.expected_content_digest != Some(BindingDigest::new(sha256(&bytes))) {
        return Err(PathMutationError::StaleContent);
    }
    Ok(())
}

#[cfg(unix)]
fn parent_request(requested: &RequestedTarget) -> Result<RequestedTarget, PathMutationError> {
    let parent = Path::new(requested.as_str())
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let value = if parent.as_os_str().is_empty() {
        "."
    } else {
        parent
            .to_str()
            .ok_or(PathMutationError::InvalidTargetName)?
    };
    RequestedTarget::new(value).map_err(|_| PathMutationError::InvalidTargetName)
}

#[cfg(unix)]
fn target_name(requested: &RequestedTarget) -> Result<&str, PathMutationError> {
    Path::new(requested.as_str())
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty() && *value != "." && *value != "..")
        .ok_or(PathMutationError::InvalidTargetName)
}

#[cfg(unix)]
fn require_missing_at(parent: &std::fs::File, name: &str) -> Result<(), PathMutationError> {
    use nix::fcntl::{OFlag, openat};
    use nix::sys::stat::Mode;

    match openat(
        parent,
        name,
        OFlag::O_RDONLY | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::empty(),
    ) {
        Ok(_) => Err(PathMutationError::GuardCollision(PathBuf::from(name))),
        Err(nix::errno::Errno::ENOENT) => Ok(()),
        Err(error) => Err(PathMutationError::Unix(error)),
    }
}

#[cfg(unix)]
fn same_unix_object(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    left.dev() == right.dev() && left.ino() == right.ino()
}

fn push_expectation(
    encoder: &mut CanonicalEncoder,
    expectation: FileMutationExpectation,
) -> Result<(), CoreError> {
    encoder.push_u8(u8::from(expectation.expected_exists));
    encoder.push_u8(match expectation.expected_kind {
        None => 0,
        Some(ObservedFileKind::Missing) => 1,
        Some(ObservedFileKind::RegularFile) => 2,
        Some(ObservedFileKind::Directory) => 3,
        Some(ObservedFileKind::SymlinkOrReparsePoint) => 4,
        Some(ObservedFileKind::Special) => 5,
    });
    push_optional_digest(encoder, expectation.expected_identity)?;
    push_optional_digest(encoder, expectation.expected_content_digest)?;
    match expectation.expected_size {
        Some(size) => {
            encoder.push_u8(1);
            encoder.push_u64(size);
        }
        None => encoder.push_u8(0),
    }
    push_optional_digest(encoder, expectation.expected_parent_identity)?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    #[test]
    fn rename_preconditions_bind_both_targets_and_parents() {
        let source = RequestedTarget::new("a.txt").unwrap();
        let destination = RequestedTarget::new("b.txt").unwrap();
        let expectation = FileMutationExpectation {
            expected_exists: true,
            expected_kind: Some(ObservedFileKind::RegularFile),
            expected_identity: Some(digest(1)),
            expected_content_digest: Some(digest(2)),
            expected_size: Some(3),
            expected_parent_identity: Some(digest(3)),
        };
        let base =
            file_rename_preconditions_hash(&source, &destination, expectation, digest(4)).unwrap();
        assert_ne!(
            base,
            file_rename_preconditions_hash(&source, &destination, expectation, digest(5)).unwrap()
        );
        assert_ne!(
            base,
            file_rename_preconditions_hash(
                &source,
                &RequestedTarget::new("c.txt").unwrap(),
                expectation,
                digest(4),
            )
            .unwrap()
        );
    }

    #[test]
    fn destructive_resource_shapes_are_operation_specific() {
        let source = RequestedTarget::new("a.txt").unwrap();
        let destination = RequestedTarget::new("b.txt").unwrap();
        assert_eq!(file_delete_resource(&source), "file-delete:a.txt");
        assert_eq!(
            file_rename_resource(&source, &destination),
            "file-rename:a.txt->b.txt"
        );
    }
}
