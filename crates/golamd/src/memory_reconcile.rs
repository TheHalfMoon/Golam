#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use golam_core::memory_storage::{MemoryLayout, MemoryLayoutError};
use golam_core::paths::RuntimeLayout;
use golam_core::target_identity::ObservedFileKind;
use golam_core::tool_request::{RequestedOperationId, RequestedTarget, ResourceClassId};
use golam_core::{CanonicalEncoder, CoreError};
use golam_kernel::{
    AuthorizationPolicy, KernelApi, ManagedMemoryRestartError, MemoryRestartObservation,
    MemoryRestartResolution,
};

use crate::local_fs::{LocalFsResolutionError, LocalFsResolver};
use crate::local_read::{LocalFileReadBounds, read_regular_file};

const MARKDOWN_READBACK_DOMAIN: &[u8] = b"golam:managed-markdown-readback:v1";
const MAX_RESTART_MARKDOWN_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MemoryRestartSummary {
    pub scanned: usize,
    pub committed: usize,
    pub no_mutation: usize,
    pub blocked_unknown: usize,
}

#[derive(Debug)]
pub enum MemoryStartupReconciliationError {
    Layout(MemoryLayoutError),
    Resolution(LocalFsResolutionError),
    Restart(ManagedMemoryRestartError),
    Core(CoreError),
    PathOutsideVault,
    NonUnicodePath,
    ClockBeforeEpoch,
}

impl fmt::Display for MemoryStartupReconciliationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Layout(error) => write!(f, "memory restart layout failed: {error}"),
            Self::Resolution(error) => write!(f, "memory restart target resolution failed: {error}"),
            Self::Restart(error) => write!(f, "memory restart kernel reconciliation failed: {error}"),
            Self::Core(error) => write!(f, "memory restart readback encoding failed: {error}"),
            Self::PathOutsideVault => {
                f.write_str("memory restart PREPARED Markdown path is outside the canonical vault")
            }
            Self::NonUnicodePath => {
                f.write_str("memory restart Markdown path is not representable by the bounded target contract")
            }
            Self::ClockBeforeEpoch => {
                f.write_str("memory restart cannot obtain a monotonic Unix-time observation")
            }
        }
    }
}

impl Error for MemoryStartupReconciliationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Layout(error) => Some(error),
            Self::Resolution(error) => Some(error),
            Self::Restart(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::PathOutsideVault | Self::NonUnicodePath | Self::ClockBeforeEpoch => None,
        }
    }
}

impl From<MemoryLayoutError> for MemoryStartupReconciliationError {
    fn from(value: MemoryLayoutError) -> Self {
        Self::Layout(value)
    }
}

impl From<LocalFsResolutionError> for MemoryStartupReconciliationError {
    fn from(value: LocalFsResolutionError) -> Self {
        Self::Resolution(value)
    }
}

impl From<ManagedMemoryRestartError> for MemoryStartupReconciliationError {
    fn from(value: ManagedMemoryRestartError) -> Self {
        Self::Restart(value)
    }
}

impl From<CoreError> for MemoryStartupReconciliationError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

pub fn reconcile_managed_memory_on_startup<P: AuthorizationPolicy>(
    runtime: &RuntimeLayout,
    kernel: &mut KernelApi<P>,
) -> Result<MemoryRestartSummary, MemoryStartupReconciliationError> {
    let cases = kernel.pending_managed_memory_restart_cases()?;
    if cases.is_empty() {
        return Ok(MemoryRestartSummary::default());
    }

    let memory = MemoryLayout::initialize(runtime)?;
    let operation = RequestedOperationId::new("read")
        .expect("constant memory restart read operation is valid");
    let resolver = LocalFsResolver::new(
        memory.vault_dir(),
        ResourceClassId::new("memory.reconcile")
            .expect("constant memory restart resource class is valid"),
        vec![operation.clone()],
        [],
    )?;
    let mut summary = MemoryRestartSummary {
        scanned: cases.len(),
        ..MemoryRestartSummary::default()
    };

    for case in cases {
        let observed_at_unix_ms = unix_time_ms()?;
        let observation = observe_case(
            &memory,
            &resolver,
            &operation,
            &case.markdown_path,
            observed_at_unix_ms,
        )?;
        let finished_at = format!("unix-ms:{observed_at_unix_ms}");
        match kernel.reconcile_managed_memory_restart_case(
            &case,
            &observation,
            &finished_at,
            observed_at_unix_ms,
        )? {
            MemoryRestartResolution::ReconciledCommitted => summary.committed += 1,
            MemoryRestartResolution::ReconciledNoMutation => summary.no_mutation += 1,
            MemoryRestartResolution::BlockedUnknownOutcome => summary.blocked_unknown += 1,
        }
    }

    Ok(summary)
}

fn observe_case(
    memory: &MemoryLayout,
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    markdown_path: &std::path::Path,
    observed_at_unix_ms: u64,
) -> Result<MemoryRestartObservation, MemoryStartupReconciliationError> {
    let relative = markdown_path
        .strip_prefix(memory.vault_dir())
        .map_err(|_| MemoryStartupReconciliationError::PathOutsideVault)?;
    let relative = relative
        .to_str()
        .ok_or(MemoryStartupReconciliationError::NonUnicodePath)?;
    let requested = RequestedTarget::new(relative.to_owned())
        .map_err(|_| MemoryStartupReconciliationError::NonUnicodePath)?;

    let identity = match resolver.resolve_read_target(&requested, operation, observed_at_unix_ms) {
        Ok(identity) => identity,
        Err(error) => {
            return Ok(MemoryRestartObservation::Unobservable {
                reason_code: format!("target_resolution:{error}"),
            });
        }
    };
    match identity.file_kind {
        ObservedFileKind::Missing => Ok(MemoryRestartObservation::Missing),
        ObservedFileKind::RegularFile => {
            let read = match read_regular_file(
                resolver,
                &requested,
                operation,
                LocalFileReadBounds {
                    max_bytes: MAX_RESTART_MARKDOWN_BYTES,
                    max_duration: Duration::from_secs(2),
                },
                observed_at_unix_ms,
                unix_time_ms()?,
            ) {
                Ok(read) => read,
                Err(error) => {
                    return Ok(MemoryRestartObservation::Unobservable {
                        reason_code: format!("bounded_read:{error}"),
                    });
                }
            };
            let Some(target_identity_ref) = read.identity.resolved_target_identity else {
                return Ok(MemoryRestartObservation::Unobservable {
                    reason_code: "regular_file_missing_identity".to_owned(),
                });
            };
            let markdown_readback_ref =
                markdown_readback_ref(target_identity_ref, read.content_digest)?;
            Ok(MemoryRestartObservation::Regular {
                target_identity_ref,
                content_digest: read.content_digest,
                markdown_readback_ref,
            })
        }
        kind => Ok(MemoryRestartObservation::Unobservable {
            reason_code: format!("unsupported_file_kind:{kind:?}"),
        }),
    }
}

fn markdown_readback_ref(
    target_identity_ref: golam_core::tool_request::BindingDigest,
    content_digest: golam_core::tool_request::BindingDigest,
) -> Result<golam_core::tool_request::BindingDigest, MemoryStartupReconciliationError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(MARKDOWN_READBACK_DOMAIN)?;
    encoder.push_bytes(&target_identity_ref.bytes())?;
    encoder.push_bytes(&content_digest.bytes())?;
    Ok(golam_core::tool_request::BindingDigest::new(
        golam_core::digest::sha256(&encoder.finish()),
    ))
}

fn unix_time_ms() -> Result<u64, MemoryStartupReconciliationError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| MemoryStartupReconciliationError::ClockBeforeEpoch)?;
    u64::try_from(duration.as_millis())
        .map_err(|_| MemoryStartupReconciliationError::ClockBeforeEpoch)
}
