#![forbid(unsafe_code)]

//! Governed parent-side construction for the first Spec 005 process executor.
//!
//! This module is deliberately Linux x86_64-only for mutation. It stages an exact bounded static
//! ELF image into a fresh private Golam inode under a distinct Effect. Payload launch is layered on
//! top of the resulting receipt; constructing data here never grants process authority by itself.

use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use golam_core::digest::sha256;
use golam_core::target_identity::ObservedFileKind;
use golam_core::tool_request::{BindingDigest, PreparedToolRequest, RequestedOperationId};
use golam_core::{CanonicalEncoder, CoreError, EffectId, SessionId};
use golam_kernel::{
    AuthorizationPolicy, CompleteToolEffect, KernelApi, PrepareToolEffect, Principal,
    ToolEffectError, ToolExecutionCompletion,
};

use crate::local_fs::{LocalFsResolutionError, LocalFsResolver};
use crate::static_elf_v2::{MAX_STATIC_EXECUTABLE_BYTES, StaticElfV2Error, validate_static_elf_v2};

pub const PROCESS_STAGE_ACTION: &str = "process.stage";
pub const PROCESS_EXECUTE_ACTION: &str = "process.execute";
const PROCESS_STAGE_HANDLER_ID: &str = "golam-native-stage-linux-x86_64";
const PROCESS_STAGE_HANDLER_VERSION: &str = "2";
const STAGE_PRECONDITION_DOMAIN: &[u8] = b"golam:process-stage-preconditions:v2";
const STAGE_RECEIPT_DOMAIN: &[u8] = b"golam:process-stage-receipt:v2";
const STAGED_PERMISSION_BITS: u32 = 0o500;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedExecutableV2 {
    pub stage_effect_id: EffectId,
    pub path: PathBuf,
    pub device: u64,
    pub inode: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub content_digest: [u8; 32],
    pub byte_len: u64,
    pub source_target_identity: BindingDigest,
    pub source_metadata_digest: BindingDigest,
    pub prepared_request_digest: [u8; 32],
    pub receipt_digest: [u8; 32],
}

#[derive(Debug)]
pub enum ProcessStageError {
    UnsupportedPlatform,
    InvalidRequestBinding(&'static str),
    Resolution(LocalFsResolutionError),
    StaticElf(StaticElfV2Error),
    Core(CoreError),
    Io(io::Error),
    Effect(ToolEffectError),
    SourceTooLarge,
    SourceIdentityChanged,
    SourceContentChanged,
    InvalidStagingParent,
    StagingParentChanged,
    StagingCollision(PathBuf),
    StagedIdentityChanged,
    StagedContentChanged,
    StagedPermissionsChanged,
    AmbiguousAfterCreate(PathBuf),
}

impl fmt::Display for ProcessStageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                f.write_str("process staging v2 is supported only on Linux x86_64")
            }
            Self::InvalidRequestBinding(reason) => {
                write!(f, "process staging request binding is invalid: {reason}")
            }
            Self::Resolution(error) => {
                write!(f, "process staging target resolution failed: {error}")
            }
            Self::StaticElf(error) => {
                write!(f, "process staging executable class rejected: {error}")
            }
            Self::Core(error) => write!(f, "process staging canonical encoding failed: {error}"),
            Self::Io(error) => write!(f, "process staging I/O failed: {error}"),
            Self::Effect(error) => write!(f, "process staging Effect Gate failed: {error}"),
            Self::SourceTooLarge => {
                f.write_str("process staging source exceeds the executable byte bound")
            }
            Self::SourceIdentityChanged => {
                f.write_str("process staging source identity changed before preparation")
            }
            Self::SourceContentChanged => {
                f.write_str("process staging source content changed before preparation")
            }
            Self::InvalidStagingParent => {
                f.write_str("process staging parent is not an exact private directory")
            }
            Self::StagingParentChanged => {
                f.write_str("process staging parent identity changed before creation")
            }
            Self::StagingCollision(path) => write!(
                f,
                "process staging target already exists: {}",
                path.display()
            ),
            Self::StagedIdentityChanged => {
                f.write_str("fresh process staging inode identity changed during readback")
            }
            Self::StagedContentChanged => {
                f.write_str("fresh process staging content changed during readback")
            }
            Self::StagedPermissionsChanged => {
                f.write_str("fresh process staging permissions are not exactly 0500")
            }
            Self::AmbiguousAfterCreate(path) => write!(
                f,
                "process staging outcome is ambiguous after inode creation: {}",
                path.display()
            ),
        }
    }
}

impl Error for ProcessStageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Resolution(error) => Some(error),
            Self::StaticElf(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::Effect(error) => Some(error),
            _ => None,
        }
    }
}

impl From<LocalFsResolutionError> for ProcessStageError {
    fn from(value: LocalFsResolutionError) -> Self {
        Self::Resolution(value)
    }
}

impl From<StaticElfV2Error> for ProcessStageError {
    fn from(value: StaticElfV2Error) -> Self {
        Self::StaticElf(value)
    }
}

impl From<CoreError> for ProcessStageError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<io::Error> for ProcessStageError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<ToolEffectError> for ProcessStageError {
    fn from(value: ToolEffectError) -> Self {
        Self::Effect(value)
    }
}

pub struct StageProcessExecutable<'a> {
    pub request: &'a PreparedToolRequest,
    pub source_resolver: &'a LocalFsResolver,
    pub staging_parent: &'a Path,
    pub stage_effect_id: EffectId,
    pub session_id: SessionId,
    pub started_at: &'a str,
    pub observed_at_unix_ms: u64,
}

pub fn stage_process_executable_v2<P: AuthorizationPolicy>(
    kernel: &mut KernelApi<P>,
    principal: Principal<'_>,
    input: StageProcessExecutable<'_>,
    scope: &str,
) -> Result<StagedExecutableV2, ProcessStageError> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        stage_linux_x86_64(kernel, principal, input, scope)
    }
    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    {
        let _ = (kernel, principal, input, scope);
        Err(ProcessStageError::UnsupportedPlatform)
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn stage_linux_x86_64<P: AuthorizationPolicy>(
    kernel: &mut KernelApi<P>,
    principal: Principal<'_>,
    input: StageProcessExecutable<'_>,
    scope: &str,
) -> Result<StagedExecutableV2, ProcessStageError> {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    use crate::fcntl::{OFlag, openat};
    use crate::local_fs::metadata_matches_resolved_identity;
    use crate::sys::stat::Mode;

    let request = input.request.request();
    if request.initiating_principal.as_str() != principal.subject {
        return Err(ProcessStageError::InvalidRequestBinding(
            "principal mismatch",
        ));
    }
    if request.requested_operation.as_str() != PROCESS_EXECUTE_ACTION {
        return Err(ProcessStageError::InvalidRequestBinding(
            "operation is not process.execute",
        ));
    }
    if request.authorized_resource_class
        != input
            .source_resolver
            .authorized_root()
            .policy_resource_class
    {
        return Err(ProcessStageError::InvalidRequestBinding(
            "resource class does not match the authorized root",
        ));
    }
    let requested =
        request
            .requested_target
            .as_ref()
            .ok_or(ProcessStageError::InvalidRequestBinding(
                "process request has no executable target",
            ))?;
    if request.target_resolution_plan_ref.is_some() {
        return Err(ProcessStageError::InvalidRequestBinding(
            "first process profile requires an exact resolved target",
        ));
    }
    let operation = RequestedOperationId::new(PROCESS_EXECUTE_ACTION).map_err(|_| {
        ProcessStageError::InvalidRequestBinding("process.execute is not canonical")
    })?;
    let resolved = input.source_resolver.resolve_read_target(
        requested,
        &operation,
        input.observed_at_unix_ms,
    )?;
    if resolved.file_kind != ObservedFileKind::RegularFile {
        return Err(ProcessStageError::InvalidRequestBinding(
            "executable target is not a regular file",
        ));
    }
    let source_identity =
        resolved
            .resolved_target_identity
            .ok_or(ProcessStageError::InvalidRequestBinding(
                "resolved executable identity is missing",
            ))?;
    if request.target_identity_ref != Some(source_identity) {
        return Err(ProcessStageError::InvalidRequestBinding(
            "prepared request target identity is stale",
        ));
    }

    let mut source = File::open(Path::new(resolved.normalized_path.as_str()))?;
    let source_metadata = source.metadata()?;
    if !metadata_matches_resolved_identity(&resolved, &source_metadata)? {
        return Err(ProcessStageError::SourceIdentityChanged);
    }
    if source_metadata.len() > MAX_STATIC_EXECUTABLE_BYTES as u64 {
        return Err(ProcessStageError::SourceTooLarge);
    }
    let mut source_bytes = Vec::with_capacity(usize::try_from(source_metadata.len()).unwrap_or(0));
    std::io::Read::by_ref(&mut source)
        .take((MAX_STATIC_EXECUTABLE_BYTES + 1) as u64)
        .read_to_end(&mut source_bytes)?;
    if source_bytes.len() > MAX_STATIC_EXECUTABLE_BYTES {
        return Err(ProcessStageError::SourceTooLarge);
    }
    validate_static_elf_v2(&source_bytes)?;
    let source_digest = sha256(&source_bytes);
    source.seek(SeekFrom::Start(0))?;
    let source_metadata_after_read = source.metadata()?;
    if !same_unix_object(&source_metadata, &source_metadata_after_read)
        || !metadata_matches_resolved_identity(&resolved, &source_metadata_after_read)?
    {
        return Err(ProcessStageError::SourceIdentityChanged);
    }

    let parent = canonical_staging_parent(input.staging_parent)?;
    let parent_metadata = fs::metadata(&parent)?;
    validate_staging_parent_metadata(&parent_metadata)?;
    let parent_binding = staging_parent_binding(&parent, &parent_metadata)?;
    let prepared_request_digest = input.request.binding_digest();
    let preconditions_hash = stage_preconditions_hash(
        prepared_request_digest,
        source_identity,
        resolved.observed_metadata_digest,
        source_digest,
        source_metadata.len(),
        parent_binding,
    )?;
    let resource = stage_resource(input.stage_effect_id);
    let prepared = kernel.prepare_tool_effect(
        principal,
        PrepareToolEffect {
            effect_id: input.stage_effect_id,
            session_id: input.session_id,
            action: PROCESS_STAGE_ACTION,
            resource: &resource,
            execution_semantics: "at_most_once",
            handler_id: PROCESS_STAGE_HANDLER_ID,
            handler_version: PROCESS_STAGE_HANDLER_VERSION,
            idempotency_key: Some(&resource),
            preconditions_hash,
            payload_hash: source_digest,
            started_at: input.started_at,
        },
        scope,
    )?;

    let current_parent_metadata = match fs::metadata(&parent) {
        Ok(metadata) => metadata,
        Err(error) => {
            complete_known_stage_failure(
                kernel,
                principal,
                &prepared,
                input.started_at,
                scope,
                "process_stage_parent_read_failed",
            )?;
            return Err(error.into());
        }
    };
    if staging_parent_binding(&parent, &current_parent_metadata)? != parent_binding {
        complete_known_stage_failure(
            kernel,
            principal,
            &prepared,
            input.started_at,
            scope,
            "process_stage_parent_changed",
        )?;
        return Err(ProcessStageError::StagingParentChanged);
    }

    let stage_name = format!("process-{:032x}.elf", input.stage_effect_id.0);
    let stage_path = parent.join(&stage_name);
    let parent_file = File::open(&parent)?;
    let created_fd = match openat(
        &parent_file,
        stage_name.as_str(),
        OFlag::O_RDWR | OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_CLOEXEC | OFlag::O_NOFOLLOW,
        Mode::from_bits_truncate(0o600),
    ) {
        Ok(fd) => fd,
        Err(error) if error == golam_core::unix_fs::Errno::EEXIST => {
            complete_known_stage_failure(
                kernel,
                principal,
                &prepared,
                input.started_at,
                scope,
                "process_stage_collision",
            )?;
            return Err(ProcessStageError::StagingCollision(stage_path));
        }
        Err(error) => {
            complete_known_stage_failure(
                kernel,
                principal,
                &prepared,
                input.started_at,
                scope,
                "process_stage_create_failed",
            )?;
            return Err(io::Error::from_raw_os_error(error as i32).into());
        }
    };

    let mut created = File::from(created_fd);
    let mutation_result = (|| -> Result<StagedExecutableV2, ProcessStageError> {
        created.write_all(&source_bytes)?;
        created.sync_all()?;
        created.set_permissions(fs::Permissions::from_mode(STAGED_PERMISSION_BITS))?;
        created.sync_all()?;
        created.seek(SeekFrom::Start(0))?;
        let mut readback = Vec::with_capacity(source_bytes.len());
        std::io::Read::by_ref(&mut created)
            .take((MAX_STATIC_EXECUTABLE_BYTES + 1) as u64)
            .read_to_end(&mut readback)?;
        if readback.len() != source_bytes.len() || sha256(&readback) != source_digest {
            return Err(ProcessStageError::StagedContentChanged);
        }
        validate_static_elf_v2(&readback)?;

        let metadata = created.metadata()?;
        if !metadata.is_file() || metadata.mode() & 0o7777 != STAGED_PERMISSION_BITS {
            return Err(ProcessStageError::StagedPermissionsChanged);
        }
        let observed = fs::File::open(&stage_path)?;
        let observed_metadata = observed.metadata()?;
        if !same_unix_object(&metadata, &observed_metadata) {
            return Err(ProcessStageError::StagedIdentityChanged);
        }
        let mut observed_reader = observed;
        let mut observed_bytes = Vec::with_capacity(readback.len());
        std::io::Read::by_ref(&mut observed_reader)
            .take((MAX_STATIC_EXECUTABLE_BYTES + 1) as u64)
            .read_to_end(&mut observed_bytes)?;
        if observed_bytes.len() != readback.len() || sha256(&observed_bytes) != source_digest {
            return Err(ProcessStageError::StagedContentChanged);
        }
        parent_file.sync_all()?;

        let mut staged = StagedExecutableV2 {
            stage_effect_id: input.stage_effect_id,
            path: fs::canonicalize(&stage_path)?,
            device: metadata.dev(),
            inode: metadata.ino(),
            mode: metadata.mode(),
            uid: metadata.uid(),
            gid: metadata.gid(),
            content_digest: source_digest,
            byte_len: metadata.len(),
            source_target_identity: source_identity,
            source_metadata_digest: resolved.observed_metadata_digest,
            prepared_request_digest,
            receipt_digest: [0; 32],
        };
        staged.receipt_digest = stage_receipt_digest(&staged)?;
        Ok(staged)
    })();

    match mutation_result {
        Ok(staged) => {
            kernel.complete_tool_effect(
                principal,
                CompleteToolEffect {
                    prepared: &prepared,
                    finished_at: input.started_at,
                    completion: ToolExecutionCompletion::Succeeded,
                    reason_code: Some("process_stage_verified"),
                    evidence_ref: Some(&staged.receipt_digest),
                    receipt: Some(&staged.receipt_digest),
                },
                scope,
            )?;
            Ok(staged)
        }
        Err(error) => {
            let evidence = sha256(stage_path.as_os_str().as_encoded_bytes());
            kernel.complete_tool_effect(
                principal,
                CompleteToolEffect {
                    prepared: &prepared,
                    finished_at: input.started_at,
                    completion: ToolExecutionCompletion::UnknownOutcome,
                    reason_code: Some("process_stage_ambiguous_after_create"),
                    evidence_ref: Some(&evidence),
                    receipt: None,
                },
                scope,
            )?;
            let _ = error;
            Err(ProcessStageError::AmbiguousAfterCreate(stage_path))
        }
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn complete_known_stage_failure<P: AuthorizationPolicy>(
    kernel: &mut KernelApi<P>,
    principal: Principal<'_>,
    prepared: &golam_kernel::PreparedToolEffect,
    finished_at: &str,
    scope: &str,
    reason_code: &str,
) -> Result<(), ToolEffectError> {
    kernel.complete_tool_effect(
        principal,
        CompleteToolEffect {
            prepared,
            finished_at,
            completion: ToolExecutionCompletion::Failed,
            reason_code: Some(reason_code),
            evidence_ref: None,
            receipt: None,
        },
        scope,
    )
}

pub fn stage_resource(effect_id: EffectId) -> String {
    format!("process-stage:{}", effect_id.0)
}

pub fn stage_preconditions_hash(
    prepared_request_digest: [u8; 32],
    source_target_identity: BindingDigest,
    source_metadata_digest: BindingDigest,
    source_content_digest: [u8; 32],
    source_size: u64,
    parent_binding: [u8; 32],
) -> Result<[u8; 32], CoreError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(STAGE_PRECONDITION_DOMAIN)?;
    encoder.push_bytes(&prepared_request_digest)?;
    encoder.push_bytes(&source_target_identity.bytes())?;
    encoder.push_bytes(&source_metadata_digest.bytes())?;
    encoder.push_bytes(&source_content_digest)?;
    encoder.push_u64(source_size);
    encoder.push_bytes(&parent_binding)?;
    encoder.push_u64(u64::from(STAGED_PERMISSION_BITS));
    Ok(sha256(&encoder.finish()))
}

pub fn stage_receipt_digest(staged: &StagedExecutableV2) -> Result<[u8; 32], CoreError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(STAGE_RECEIPT_DOMAIN)?;
    encoder.push_u128(staged.stage_effect_id.0);
    encoder.push_bytes(staged.path.as_os_str().as_encoded_bytes())?;
    encoder.push_u64(staged.device);
    encoder.push_u64(staged.inode);
    encoder.push_u64(u64::from(staged.mode));
    encoder.push_u64(u64::from(staged.uid));
    encoder.push_u64(u64::from(staged.gid));
    encoder.push_bytes(&staged.content_digest)?;
    encoder.push_u64(staged.byte_len);
    encoder.push_bytes(&staged.source_target_identity.bytes())?;
    encoder.push_bytes(&staged.source_metadata_digest.bytes())?;
    encoder.push_bytes(&staged.prepared_request_digest)?;
    Ok(sha256(&encoder.finish()))
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn canonical_staging_parent(path: &Path) -> Result<PathBuf, ProcessStageError> {
    let symlink_metadata = fs::symlink_metadata(path)?;
    if symlink_metadata.file_type().is_symlink() || !symlink_metadata.is_dir() {
        return Err(ProcessStageError::InvalidStagingParent);
    }
    let canonical = fs::canonicalize(path)?;
    let metadata = fs::metadata(&canonical)?;
    validate_staging_parent_metadata(&metadata)?;
    Ok(canonical)
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn validate_staging_parent_metadata(metadata: &fs::Metadata) -> Result<(), ProcessStageError> {
    use std::os::unix::fs::MetadataExt;

    if !metadata.is_dir() || metadata.mode() & 0o077 != 0 {
        return Err(ProcessStageError::InvalidStagingParent);
    }
    Ok(())
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn staging_parent_binding(path: &Path, metadata: &fs::Metadata) -> Result<[u8; 32], CoreError> {
    use std::os::unix::fs::MetadataExt;

    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(b"golam:process-stage-parent:v2")?;
    encoder.push_bytes(path.as_os_str().as_encoded_bytes())?;
    encoder.push_u64(metadata.dev());
    encoder.push_u64(metadata.ino());
    encoder.push_u64(u64::from(metadata.mode()));
    encoder.push_u64(u64::from(metadata.uid()));
    encoder.push_u64(u64::from(metadata.gid()));
    Ok(sha256(&encoder.finish()))
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn same_unix_object(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    left.dev() == right.dev()
        && left.ino() == right.ino()
        && left.mode() == right.mode()
        && left.uid() == right.uid()
        && left.gid() == right.gid()
        && left.len() == right.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_resource_is_canonical_and_effect_specific() {
        assert_eq!(stage_resource(EffectId(17)), "process-stage:17");
    }

    #[test]
    fn stage_preconditions_change_with_request_and_source() {
        let first = stage_preconditions_hash(
            [1; 32],
            BindingDigest::new([2; 32]),
            BindingDigest::new([3; 32]),
            [4; 32],
            5,
            [6; 32],
        )
        .unwrap();
        let second = stage_preconditions_hash(
            [9; 32],
            BindingDigest::new([2; 32]),
            BindingDigest::new([3; 32]),
            [4; 32],
            5,
            [6; 32],
        )
        .unwrap();
        assert_ne!(first, second);
    }
}
