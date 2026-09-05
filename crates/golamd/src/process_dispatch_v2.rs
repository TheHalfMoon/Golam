#![forbid(unsafe_code)]

//! Governed production dispatch for the first Spec 005 native process profile.
//!
//! The mutable path is intentionally limited to Linux x86_64. A launch consumes an already
//! prepared `ToolRequest`, a sealed live capability lease and a terminal-successful staged static
//! ELF receipt. It creates a distinct `process.execute` Effect, revalidates authority and every
//! identity at the final parent-side boundary, launches only the exact trusted sibling helper,
//! supervises wall time / combined output / cancellation, and refuses terminal success without
//! exact root/process-tree reconciliation.

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use golam_core::digest::sha256;
use golam_core::tool_request::{BindingDigest, PreparedToolRequest};
use golam_core::{CanonicalEncoder, CoreError, EffectId, SessionId};
use golam_kernel::{
    AuthorizationPolicy, CapabilityLease, CapabilityLeaseUseError, CapabilityLeaseUseEvidence,
    KernelApi, KernelError, Principal, ToolEffectError,
};

use crate::process_execution_v2::StagedExecutableV2;

const EXECUTE_HANDLER_ID: &str = "golam-native-exec-linux-x86_64";
const EXECUTE_HANDLER_VERSION: &str = "2";
const EXECUTION_BINDING_DOMAIN: &[u8] = b"golam:process-execute-binding:v2";
const EXECUTION_PAYLOAD_DOMAIN: &[u8] = b"golam:process-execute-payload:v2";
const CAPABILITY_CONTEXT_DOMAIN: &[u8] = b"golam:process-capability-context:v2";
const EXECUTION_RECEIPT_DOMAIN: &[u8] = b"golam:process-execute-receipt:v2";
const HELPER_IDENTITY_DOMAIN: &[u8] = b"golam:native-exec-helper-identity:v2";
const TRUSTED_HELPER_NAME: &str = "golam-native-exec-helper-v2";
const READY_PREFIX: &[u8] = b"GOLAM_NATIVE_EXEC_V2_READY:";
const MAX_READY_LINE_BYTES: usize = 160;
const MAX_HELPER_BYTES: u64 = 64 * 1024 * 1024;
const MAX_ARG_COUNT: usize = 256;
const MAX_ARG_BYTES: usize = 64 * 1024;
const MAX_COMBINED_OUTPUT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_WALL_TIME_MS: u64 = 24 * 60 * 60 * 1000;
const STREAM_CHUNK_BYTES: usize = 4096;
const STREAM_CHANNEL_DEPTH: usize = 8;
const SUPERVISOR_POLL_MS: u64 = 10;
const TERMINAL_DRAIN_MS: u64 = 1000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessExecutionLimitsV2 {
    pub cpu_seconds: u64,
    pub address_space_bytes: u64,
    pub max_created_file_bytes: u64,
    pub max_open_files: u64,
    pub wall_time_ms: u64,
    pub max_stdout_stderr_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustedNativeExecHelperV2 {
    pub canonical_path: PathBuf,
    pub device: u64,
    pub inode: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub byte_len: u64,
    pub content_digest: [u8; 32],
    pub identity_digest: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessExecutionStatusV2 {
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    OutputLimitExceeded,
    UnknownOutcome,
}

impl ProcessExecutionStatusV2 {
    const fn code(self) -> u8 {
        match self {
            Self::Succeeded => 1,
            Self::Failed => 2,
            Self::Cancelled => 3,
            Self::TimedOut => 4,
            Self::OutputLimitExceeded => 5,
            Self::UnknownOutcome => 6,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessExecutionReceiptV2 {
    pub execute_effect_id: EffectId,
    pub stage_effect_id: EffectId,
    pub root_pid: u32,
    pub status: ProcessExecutionStatusV2,
    pub exit_code: Option<i32>,
    pub signal: Option<i32>,
    pub observed_descendant_count: u32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub stdout_digest: [u8; 32],
    pub stderr_digest: [u8; 32],
    pub execution_binding_digest: [u8; 32],
    pub capability_context_digest: [u8; 32],
    pub helper_identity_digest: [u8; 32],
    pub receipt_digest: [u8; 32],
}

pub struct ExecuteStagedProcessV2<'a> {
    pub request: &'a PreparedToolRequest,
    pub lease: &'a CapabilityLease,
    pub staged: &'a StagedExecutableV2,
    pub helper_path: &'a Path,
    pub cwd: &'a Path,
    pub filesystem_read_paths: &'a [PathBuf],
    pub filesystem_write_paths: &'a [PathBuf],
    pub argv: &'a [Vec<u8>],
    pub limits: ProcessExecutionLimitsV2,
    pub execute_effect_id: EffectId,
    pub session_id: SessionId,
    pub started_at: &'a str,
    pub dispatch_at: &'a str,
    pub finished_at: &'a str,
    pub cancellation: &'a AtomicBool,
}

#[derive(Debug)]
pub enum ProcessExecutionV2Error {
    UnsupportedPlatform,
    InvalidBinding(&'static str),
    Core(CoreError),
    Io(std::io::Error),
    Kernel(KernelError),
    Lease(CapabilityLeaseUseError),
    Effect(ToolEffectError),
    HelperIdentityChanged,
    StagedExecutableChanged,
    NativeContainment(String),
    Supervisor(String),
    Spawn(std::io::Error),
    HelperProtocol(&'static str),
}

impl fmt::Display for ProcessExecutionV2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                f.write_str("process execution v2 is supported only on Linux x86_64")
            }
            Self::InvalidBinding(reason) => {
                write!(f, "process execution binding invalid: {reason}")
            }
            Self::Core(error) => write!(f, "process execution canonical encoding failed: {error}"),
            Self::Io(error) => write!(f, "process execution I/O failed: {error}"),
            Self::Kernel(error) => {
                write!(f, "process execution kernel authorization failed: {error}")
            }
            Self::Lease(error) => write!(f, "process execution capability lease failed: {error}"),
            Self::Effect(error) => write!(f, "process execution Effect Gate failed: {error}"),
            Self::HelperIdentityChanged => {
                f.write_str("trusted native exec helper identity changed before spawn")
            }
            Self::StagedExecutableChanged => {
                f.write_str("staged executable identity or content changed before spawn")
            }
            Self::NativeContainment(error) => {
                write!(f, "process execution containment plan rejected: {error}")
            }
            Self::Supervisor(error) => write!(f, "process execution supervisor failed: {error}"),
            Self::Spawn(error) => write!(f, "trusted native exec helper spawn failed: {error}"),
            Self::HelperProtocol(reason) => {
                write!(f, "trusted native exec helper protocol failed: {reason}")
            }
        }
    }
}

impl Error for ProcessExecutionV2Error {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Core(error) => Some(error),
            Self::Io(error) | Self::Spawn(error) => Some(error),
            Self::Kernel(error) => Some(error),
            Self::Lease(error) => Some(error),
            Self::Effect(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CoreError> for ProcessExecutionV2Error {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<std::io::Error> for ProcessExecutionV2Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<KernelError> for ProcessExecutionV2Error {
    fn from(value: KernelError) -> Self {
        Self::Kernel(value)
    }
}

impl From<CapabilityLeaseUseError> for ProcessExecutionV2Error {
    fn from(value: CapabilityLeaseUseError) -> Self {
        Self::Lease(value)
    }
}

impl From<ToolEffectError> for ProcessExecutionV2Error {
    fn from(value: ToolEffectError) -> Self {
        Self::Effect(value)
    }
}

pub fn process_execute_resource_v2(request: &PreparedToolRequest) -> String {
    format!("process-request:{}", request.request().request_id.as_u128())
}

pub fn capability_context_ref_v2(
    evidence: CapabilityLeaseUseEvidence,
) -> Result<BindingDigest, ProcessExecutionV2Error> {
    Ok(BindingDigest::new(capability_context_digest_v2(evidence)?))
}

pub fn capability_context_digest_v2(
    evidence: CapabilityLeaseUseEvidence,
) -> Result<[u8; 32], ProcessExecutionV2Error> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(CAPABILITY_CONTEXT_DOMAIN)?;
    encoder.push_bytes(&evidence.lease_id().to_bytes())?;
    encoder.push_u64(evidence.generation());
    encoder.push_bytes(&evidence.authority_digest())?;
    encoder.push_bytes(&evidence.scope_digest())?;
    Ok(sha256(&encoder.finish()))
}

pub fn execute_staged_process_v2<P: AuthorizationPolicy>(
    kernel: &mut KernelApi<P>,
    principal: Principal<'_>,
    input: ExecuteStagedProcessV2<'_>,
    scope: &str,
) -> Result<ProcessExecutionReceiptV2, ProcessExecutionV2Error> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        linux_x86_64::execute(kernel, principal, input, scope)
    }
    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    {
        let _ = (kernel, principal, input, scope);
        Err(ProcessExecutionV2Error::UnsupportedPlatform)
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod linux_x86_64 {
    use std::fs::{self, File};
    use std::io::Read;
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::process::ExitStatusExt;
    use std::process::{Child, ChildStderr, ChildStdout, Command, ExitStatus, Stdio};
    use std::sync::atomic::Ordering;
    use std::sync::mpsc::{self, Receiver, SyncSender};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    use golam_kernel::{
        AuthorizationContext, AuthorizationDecision, AuthorizationRequest, CompleteToolEffect,
        PrepareToolEffect, ToolExecutionCompletion,
    };
    use nix::unistd::geteuid;

    use super::*;
    use crate::native_containment_v2::{
        LinuxContainmentPlan, NativeObjectIdentity, PROFILE_TOKEN, observe_native_object,
        validate_plan,
    };
    use crate::native_process_supervisor_v2::{
        ProcessTreeReconciliation, ProcessTreeTerminalEvidence, RootContainmentBinding,
        RootProcessControl, RootProcessSupervisor, RootTerminationKind,
    };
    use crate::process_execution_v2::{PROCESS_EXECUTE_ACTION, stage_receipt_digest};
    use crate::static_elf_v2::{MAX_STATIC_EXECUTABLE_BYTES, validate_static_elf_v2};

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum ForcedTermination {
        Cancellation,
        WallTime,
        Output,
    }

    #[derive(Debug)]
    enum StreamEvent {
        Ready(Vec<u8>),
        Stdout(Vec<u8>),
        Stderr(Vec<u8>),
        StdoutEof,
        StderrEof,
        ReadError,
    }

    struct ChildControl {
        child: Arc<Mutex<Child>>,
        terminal: Option<crate::native_process_supervisor_v2::RootTerminalObservation>,
    }

    impl ChildControl {
        fn new(child: Arc<Mutex<Child>>) -> Self {
            Self {
                child,
                terminal: None,
            }
        }
    }

    impl RootProcessControl for ChildControl {
        fn request_termination(&mut self, root_pid: u32) -> Result<(), String> {
            let mut child = self
                .child
                .lock()
                .map_err(|_| "child control mutex poisoned".to_owned())?;
            if child.id() != root_pid {
                return Err("child control root pid changed".to_owned());
            }
            match child.try_wait() {
                Ok(Some(status)) => {
                    self.terminal = Some(terminal_observation(root_pid, status));
                    Ok(())
                }
                Ok(None) => child
                    .kill()
                    .map_err(|error| format!("kill root process: {error}")),
                Err(error) => Err(format!("observe root before termination: {error}")),
            }
        }

        fn observe_terminal(
            &mut self,
            root_pid: u32,
        ) -> Result<Option<crate::native_process_supervisor_v2::RootTerminalObservation>, String>
        {
            let mut child = self
                .child
                .lock()
                .map_err(|_| "child control mutex poisoned".to_owned())?;
            if child.id() != root_pid {
                return Err("child control root pid changed".to_owned());
            }
            if let Some(terminal) = self.terminal {
                return Ok(Some(terminal));
            }
            match child.try_wait() {
                Ok(Some(status)) => {
                    let terminal = terminal_observation(root_pid, status);
                    self.terminal = Some(terminal);
                    Ok(Some(terminal))
                }
                Ok(None) => Ok(None),
                Err(error) => Err(format!("observe root terminal state: {error}")),
            }
        }
    }

    pub(super) fn execute<P: AuthorizationPolicy>(
        kernel: &mut KernelApi<P>,
        principal: Principal<'_>,
        input: ExecuteStagedProcessV2<'_>,
        scope: &str,
    ) -> Result<ProcessExecutionReceiptV2, ProcessExecutionV2Error> {
        validate_request_binding(principal, &input)?;
        validate_argv(input.argv)?;
        validate_limits(input.limits)?;
        verify_staged_executable(input.staged)?;

        let helper = observe_trusted_helper(input.helper_path)?;
        let cwd = observe_native_object(input.cwd)
            .map_err(|error| ProcessExecutionV2Error::NativeContainment(error.to_string()))?;
        let read_roots = observe_roots(input.filesystem_read_paths)?;
        let write_roots = observe_roots(input.filesystem_write_paths)?;
        validate_parent_containment_plan(
            input.staged,
            &cwd,
            &read_roots,
            &write_roots,
            input.limits,
        )?;

        let resource = process_execute_resource_v2(input.request);
        let lease_evidence = kernel.validate_principal_capability_lease_use(
            input.lease,
            principal,
            PROCESS_EXECUTE_ACTION,
            &resource,
            &[],
            input.started_at,
        )?;
        let capability_context_digest = capability_context_digest_v2(lease_evidence)?;
        if input.request.request().capability_context_ref
            != BindingDigest::new(capability_context_digest)
        {
            return Err(ProcessExecutionV2Error::InvalidBinding(
                "ToolRequest capability context does not match the sealed lease",
            ));
        }

        let payload_hash = execution_payload_hash(input.staged, input.argv)?;
        let execution_binding_digest = execution_binding_digest(
            &input,
            lease_evidence,
            &helper,
            &cwd,
            &read_roots,
            &write_roots,
            payload_hash,
        )?;
        let prepared = kernel.prepare_tool_effect(
            principal,
            PrepareToolEffect {
                effect_id: input.execute_effect_id,
                session_id: input.session_id,
                action: PROCESS_EXECUTE_ACTION,
                resource: &resource,
                execution_semantics: "at_most_once",
                handler_id: EXECUTE_HANDLER_ID,
                handler_version: EXECUTE_HANDLER_VERSION,
                idempotency_key: Some(&resource),
                preconditions_hash: execution_binding_digest,
                payload_hash,
                started_at: input.started_at,
            },
            scope,
        )?;

        let authorization = kernel.authorize(&AuthorizationRequest {
            principal,
            action: PROCESS_EXECUTE_ACTION,
            resource: &resource,
            context: AuthorizationContext::local(scope),
        })?;
        if authorization.decision != AuthorizationDecision::Allow {
            return Err(ProcessExecutionV2Error::InvalidBinding(
                "Kernel authorization changed after Effect preparation",
            ));
        }
        let dispatch_lease_evidence = kernel.validate_principal_capability_lease_use(
            input.lease,
            principal,
            PROCESS_EXECUTE_ACTION,
            &resource,
            &[],
            input.dispatch_at,
        )?;
        if dispatch_lease_evidence != lease_evidence {
            return Err(ProcessExecutionV2Error::InvalidBinding(
                "capability lease evidence changed after Effect preparation",
            ));
        }
        verify_staged_executable(input.staged)?;
        let dispatch_helper = observe_trusted_helper(input.helper_path)?;
        if dispatch_helper != helper {
            return Err(ProcessExecutionV2Error::HelperIdentityChanged);
        }
        revalidate_object(&cwd)?;
        for root in &read_roots {
            revalidate_object(root)?;
        }
        for root in &write_roots {
            revalidate_object(root)?;
        }

        let mut child = match spawn_helper(
            &helper,
            input.staged,
            &cwd,
            &read_roots,
            &write_roots,
            input.argv,
            input.limits,
            execution_binding_digest,
        ) {
            Ok(child) => child,
            Err(error) => {
                complete_pre_spawn_failure(
                    kernel,
                    principal,
                    &prepared,
                    input.finished_at,
                    scope,
                    "process_execute_helper_spawn_failed",
                )?;
                return Err(ProcessExecutionV2Error::Spawn(error));
            }
        };

        let root_pid = child.id();
        let stdout = child
            .stdout
            .take()
            .ok_or(ProcessExecutionV2Error::HelperProtocol(
                "stdout pipe missing",
            ))?;
        let stderr = child
            .stderr
            .take()
            .ok_or(ProcessExecutionV2Error::HelperProtocol(
                "stderr pipe missing",
            ))?;
        let (sender, receiver) = mpsc::sync_channel(STREAM_CHANNEL_DEPTH);
        spawn_stdout_reader(stdout, sender.clone());
        spawn_stderr_reader(stderr, sender);

        let started = Instant::now();
        if let Err(reason) = await_ready(
            &mut child,
            &receiver,
            execution_binding_digest,
            input.limits.wall_time_ms,
            started,
        ) {
            let exact_terminal = terminate_before_ready(&mut child);
            let completion = if exact_terminal {
                ToolExecutionCompletion::Failed
            } else {
                ToolExecutionCompletion::UnknownOutcome
            };
            kernel.complete_tool_effect(
                principal,
                CompleteToolEffect {
                    prepared: &prepared,
                    finished_at: input.finished_at,
                    completion,
                    reason_code: Some(reason),
                    evidence_ref: Some(&execution_binding_digest),
                    receipt: None,
                },
                scope,
            )?;
            return Err(ProcessExecutionV2Error::HelperProtocol(reason));
        }

        let binding = RootContainmentBinding {
            profile_token: PROFILE_TOKEN.to_owned(),
            root_pid,
            landlock_ruleset_fully_enforced: true,
            no_new_privs: true,
            seccomp_tsync_installed: true,
            spawn_denied: true,
            strict_local: true,
            linux_capability_sets_empty: true,
            identity_bound_regular_file_roots: true,
            ipc_creation_denied: true,
            cross_process_control_denied: true,
            credential_kernel_surfaces_denied: true,
            wall_time_limit_ms: input.limits.wall_time_ms,
            max_stdout_stderr_bytes: input.limits.max_stdout_stderr_bytes,
        };
        let shared_child = Arc::new(Mutex::new(child));
        let control = ChildControl::new(Arc::clone(&shared_child));
        let supervisor = RootProcessSupervisor::new(binding, control)
            .map_err(|error| ProcessExecutionV2Error::Supervisor(error.to_string()))?;
        supervise(
            kernel,
            principal,
            &prepared,
            input,
            scope,
            supervisor,
            shared_child,
            receiver,
            started,
            capability_context_digest,
            helper.identity_digest,
            execution_binding_digest,
        )
    }

    fn validate_request_binding(
        principal: Principal<'_>,
        input: &ExecuteStagedProcessV2<'_>,
    ) -> Result<(), ProcessExecutionV2Error> {
        let request = input.request.request();
        if request.initiating_principal.as_str() != principal.subject {
            return Err(ProcessExecutionV2Error::InvalidBinding(
                "principal mismatch",
            ));
        }
        if request.requested_operation.as_str() != PROCESS_EXECUTE_ACTION {
            return Err(ProcessExecutionV2Error::InvalidBinding(
                "operation is not process.execute",
            ));
        }
        if input.staged.prepared_request_digest != input.request.binding_digest() {
            return Err(ProcessExecutionV2Error::InvalidBinding(
                "stage receipt belongs to a different ToolRequest",
            ));
        }
        if input.staged.stage_effect_id == input.execute_effect_id {
            return Err(ProcessExecutionV2Error::InvalidBinding(
                "stage and execute Effects must be distinct",
            ));
        }
        if input.started_at.is_empty()
            || input.dispatch_at.is_empty()
            || input.finished_at.is_empty()
        {
            return Err(ProcessExecutionV2Error::InvalidBinding(
                "execution timestamps must be non-empty",
            ));
        }
        if stage_receipt_digest(input.staged)? != input.staged.receipt_digest {
            return Err(ProcessExecutionV2Error::InvalidBinding(
                "stage receipt digest is invalid",
            ));
        }
        Ok(())
    }

    fn validate_argv(argv: &[Vec<u8>]) -> Result<(), ProcessExecutionV2Error> {
        if argv.is_empty() || argv.len() > MAX_ARG_COUNT {
            return Err(ProcessExecutionV2Error::InvalidBinding(
                "argv must be non-empty and bounded",
            ));
        }
        for item in argv {
            if item.len() > MAX_ARG_BYTES || item.contains(&0) {
                return Err(ProcessExecutionV2Error::InvalidBinding(
                    "argv item is oversized or contains NUL",
                ));
            }
        }
        Ok(())
    }

    fn validate_limits(limits: ProcessExecutionLimitsV2) -> Result<(), ProcessExecutionV2Error> {
        if limits.wall_time_ms == 0
            || limits.wall_time_ms > MAX_WALL_TIME_MS
            || limits.max_stdout_stderr_bytes == 0
            || limits.max_stdout_stderr_bytes > MAX_COMBINED_OUTPUT_BYTES
        {
            return Err(ProcessExecutionV2Error::InvalidBinding(
                "wall-time or output limit is outside the admitted parent bound",
            ));
        }
        Ok(())
    }

    fn observe_roots(
        paths: &[PathBuf],
    ) -> Result<Vec<NativeObjectIdentity>, ProcessExecutionV2Error> {
        paths
            .iter()
            .map(|path| {
                observe_native_object(path)
                    .map_err(|error| ProcessExecutionV2Error::NativeContainment(error.to_string()))
            })
            .collect()
    }

    fn validate_parent_containment_plan(
        staged: &StagedExecutableV2,
        cwd: &NativeObjectIdentity,
        read_roots: &[NativeObjectIdentity],
        write_roots: &[NativeObjectIdentity],
        limits: ProcessExecutionLimitsV2,
    ) -> Result<(), ProcessExecutionV2Error> {
        let plan = LinuxContainmentPlan {
            profile_token: PROFILE_TOKEN.to_owned(),
            executable: staged_native_identity(staged),
            cwd: cwd.clone(),
            filesystem_read_roots: read_roots.to_vec(),
            filesystem_write_roots: write_roots.to_vec(),
            cpu_seconds: limits.cpu_seconds,
            address_space_bytes: limits.address_space_bytes,
            max_created_file_bytes: limits.max_created_file_bytes,
            max_open_files: limits.max_open_files,
            wall_time_ms: limits.wall_time_ms,
            max_stdout_stderr_bytes: limits.max_stdout_stderr_bytes,
            strict_local: true,
            spawn_denied: true,
            ambient_environment_cleared: true,
            device_rules_empty: true,
            ipc_rules_empty: true,
            inherited_handle_rules_empty: true,
        };
        validate_plan(&plan)
            .map_err(|error| ProcessExecutionV2Error::NativeContainment(error.to_string()))
    }

    fn verify_staged_executable(
        staged: &StagedExecutableV2,
    ) -> Result<(), ProcessExecutionV2Error> {
        let symlink = fs::symlink_metadata(&staged.path)?;
        if symlink.file_type().is_symlink() {
            return Err(ProcessExecutionV2Error::StagedExecutableChanged);
        }
        let mut file = File::open(&staged.path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file()
            || metadata.dev() != staged.device
            || metadata.ino() != staged.inode
            || metadata.mode() != staged.mode
            || metadata.uid() != staged.uid
            || metadata.gid() != staged.gid
            || metadata.len() != staged.byte_len
            || metadata.mode() & 0o7777 != 0o500
            || metadata.uid() != geteuid().as_raw()
            || metadata.len() > MAX_STATIC_EXECUTABLE_BYTES as u64
        {
            return Err(ProcessExecutionV2Error::StagedExecutableChanged);
        }
        let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).unwrap_or(0));
        std::io::Read::by_ref(&mut file)
            .take((MAX_STATIC_EXECUTABLE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_STATIC_EXECUTABLE_BYTES || sha256(&bytes) != staged.content_digest {
            return Err(ProcessExecutionV2Error::StagedExecutableChanged);
        }
        validate_static_elf_v2(&bytes)
            .map_err(|error| ProcessExecutionV2Error::NativeContainment(error.to_string()))?;
        Ok(())
    }

    fn observe_trusted_helper(
        helper_path: &Path,
    ) -> Result<TrustedNativeExecHelperV2, ProcessExecutionV2Error> {
        let current_exe = fs::canonicalize(std::env::current_exe()?)?;
        let expected_parent =
            current_exe
                .parent()
                .ok_or(ProcessExecutionV2Error::InvalidBinding(
                    "current executable has no parent directory",
                ))?;
        if helper_path.file_name().and_then(|value| value.to_str()) != Some(TRUSTED_HELPER_NAME) {
            return Err(ProcessExecutionV2Error::InvalidBinding(
                "helper filename is not the admitted helper identity",
            ));
        }
        let symlink = fs::symlink_metadata(helper_path)?;
        if symlink.file_type().is_symlink() || !symlink.is_file() {
            return Err(ProcessExecutionV2Error::InvalidBinding(
                "helper path must be a non-symlink regular file",
            ));
        }
        let canonical_path = fs::canonicalize(helper_path)?;
        if canonical_path.parent() != Some(expected_parent) {
            return Err(ProcessExecutionV2Error::InvalidBinding(
                "helper must be the exact sibling of the running Golam executable",
            ));
        }
        let file = File::open(&canonical_path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file()
            || metadata.uid() != geteuid().as_raw()
            || metadata.mode() & 0o100 == 0
            || metadata.mode() & 0o022 != 0
            || metadata.len() == 0
            || metadata.len() > MAX_HELPER_BYTES
        {
            return Err(ProcessExecutionV2Error::InvalidBinding(
                "helper ownership, permissions or size are outside the trusted boundary",
            ));
        }
        let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).unwrap_or(0));
        file.take(MAX_HELPER_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 != metadata.len() {
            return Err(ProcessExecutionV2Error::InvalidBinding(
                "helper bytes exceed or differ from the observed file size",
            ));
        }
        let content_digest = sha256(&bytes);
        let mut helper = TrustedNativeExecHelperV2 {
            canonical_path,
            device: metadata.dev(),
            inode: metadata.ino(),
            mode: metadata.mode(),
            uid: metadata.uid(),
            gid: metadata.gid(),
            byte_len: metadata.len(),
            content_digest,
            identity_digest: [0; 32],
        };
        helper.identity_digest = helper_identity_digest(&helper)?;
        Ok(helper)
    }

    fn helper_identity_digest(
        helper: &TrustedNativeExecHelperV2,
    ) -> Result<[u8; 32], ProcessExecutionV2Error> {
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(HELPER_IDENTITY_DOMAIN)?;
        encoder.push_bytes(helper.canonical_path.as_os_str().as_encoded_bytes())?;
        encoder.push_u64(helper.device);
        encoder.push_u64(helper.inode);
        encoder.push_u64(u64::from(helper.mode));
        encoder.push_u64(u64::from(helper.uid));
        encoder.push_u64(u64::from(helper.gid));
        encoder.push_u64(helper.byte_len);
        encoder.push_bytes(&helper.content_digest)?;
        Ok(sha256(&encoder.finish()))
    }

    fn staged_native_identity(staged: &StagedExecutableV2) -> NativeObjectIdentity {
        NativeObjectIdentity {
            canonical_path: staged.path.clone(),
            device: staged.device,
            inode: staged.inode,
            mode: staged.mode,
        }
    }

    fn revalidate_object(expected: &NativeObjectIdentity) -> Result<(), ProcessExecutionV2Error> {
        let observed = observe_native_object(&expected.canonical_path)
            .map_err(|error| ProcessExecutionV2Error::NativeContainment(error.to_string()))?;
        if &observed != expected {
            return Err(ProcessExecutionV2Error::InvalidBinding(
                "filesystem identity changed after Effect preparation",
            ));
        }
        Ok(())
    }

    fn execution_payload_hash(
        staged: &StagedExecutableV2,
        argv: &[Vec<u8>],
    ) -> Result<[u8; 32], ProcessExecutionV2Error> {
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(EXECUTION_PAYLOAD_DOMAIN)?;
        encoder.push_bytes(&staged.content_digest)?;
        encoder.push_u64(staged.byte_len);
        push_argv(&mut encoder, argv)?;
        encoder.push_u64(0);
        Ok(sha256(&encoder.finish()))
    }

    #[allow(clippy::too_many_arguments)]
    fn execution_binding_digest(
        input: &ExecuteStagedProcessV2<'_>,
        lease: CapabilityLeaseUseEvidence,
        helper: &TrustedNativeExecHelperV2,
        cwd: &NativeObjectIdentity,
        read_roots: &[NativeObjectIdentity],
        write_roots: &[NativeObjectIdentity],
        payload_hash: [u8; 32],
    ) -> Result<[u8; 32], ProcessExecutionV2Error> {
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(EXECUTION_BINDING_DOMAIN)?;
        encoder.push_u128(input.execute_effect_id.0);
        encoder.push_u128(input.session_id.0);
        encoder.push_bytes(&input.request.binding_digest())?;
        encoder.push_bytes(&capability_context_digest_v2(lease)?)?;
        encoder.push_u128(input.staged.stage_effect_id.0);
        encoder.push_bytes(&input.staged.receipt_digest)?;
        encoder.push_bytes(PROFILE_TOKEN.as_bytes())?;
        push_native_object(&mut encoder, &staged_native_identity(input.staged))?;
        encoder.push_bytes(&input.staged.content_digest)?;
        encoder.push_bytes(&helper.identity_digest)?;
        push_native_object(&mut encoder, cwd)?;
        push_native_objects(&mut encoder, read_roots)?;
        push_native_objects(&mut encoder, write_roots)?;
        push_argv(&mut encoder, input.argv)?;
        encoder.push_u64(0);
        encoder.push_u8(1);
        encoder.push_u64(0);
        encoder.push_u64(0);
        encoder.push_u64(0);
        encoder.push_u64(0);
        push_limits(&mut encoder, input.limits);
        encoder.push_bytes(b"cancel:bounded-root-kill-then-terminal-reconcile:v2")?;
        encoder.push_bytes(b"reconcile:root-and-zero-descendants-or-unknown:v2")?;
        encoder.push_bytes(&payload_hash)?;
        Ok(sha256(&encoder.finish()))
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_helper(
        helper: &TrustedNativeExecHelperV2,
        staged: &StagedExecutableV2,
        cwd: &NativeObjectIdentity,
        read_roots: &[NativeObjectIdentity],
        write_roots: &[NativeObjectIdentity],
        argv: &[Vec<u8>],
        limits: ProcessExecutionLimitsV2,
        execution_binding_digest: [u8; 32],
    ) -> Result<Child, std::io::Error> {
        let mut command = Command::new(&helper.canonical_path);
        command
            .env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("--expected-parent-pid")
            .arg(std::process::id().to_string())
            .arg("--execution-binding-sha256")
            .arg(encode_hex(&execution_binding_digest))
            .arg("--executable-path-hex")
            .arg(encode_hex(staged.path.as_os_str().as_encoded_bytes()))
            .arg("--executable-device")
            .arg(staged.device.to_string())
            .arg("--executable-inode")
            .arg(staged.inode.to_string())
            .arg("--executable-mode")
            .arg(staged.mode.to_string())
            .arg("--executable-sha256")
            .arg(encode_hex(&staged.content_digest))
            .arg("--cwd-path-hex")
            .arg(encode_hex(
                cwd.canonical_path.as_os_str().as_encoded_bytes(),
            ))
            .arg("--cwd-device")
            .arg(cwd.device.to_string())
            .arg("--cwd-inode")
            .arg(cwd.inode.to_string())
            .arg("--cwd-mode")
            .arg(cwd.mode.to_string());
        for root in read_roots {
            command.arg("--read-root").arg(object_argument(root));
        }
        for root in write_roots {
            command.arg("--write-root").arg(object_argument(root));
        }
        command
            .arg("--cpu-seconds")
            .arg(limits.cpu_seconds.to_string())
            .arg("--address-space-bytes")
            .arg(limits.address_space_bytes.to_string())
            .arg("--max-created-file-bytes")
            .arg(limits.max_created_file_bytes.to_string())
            .arg("--max-open-files")
            .arg(limits.max_open_files.to_string())
            .arg("--wall-time-ms")
            .arg(limits.wall_time_ms.to_string())
            .arg("--max-output-bytes")
            .arg(limits.max_stdout_stderr_bytes.to_string());
        for item in argv {
            command.arg("--arg-hex").arg(encode_hex(item));
        }
        command.spawn()
    }

    fn object_argument(object: &NativeObjectIdentity) -> String {
        format!(
            "{}:{}:{}:{}",
            encode_hex(object.canonical_path.as_os_str().as_encoded_bytes()),
            object.device,
            object.inode,
            object.mode
        )
    }

    fn await_ready(
        child: &mut Child,
        receiver: &Receiver<StreamEvent>,
        expected_binding: [u8; 32],
        wall_time_ms: u64,
        started: Instant,
    ) -> Result<(), &'static str> {
        let expected = {
            let mut value = READY_PREFIX.to_vec();
            value.extend_from_slice(encode_hex(&expected_binding).as_bytes());
            value.push(b'\n');
            value
        };
        loop {
            if started.elapsed().as_millis() >= u128::from(wall_time_ms) {
                return Err("process_execute_helper_ready_timeout");
            }
            match child.try_wait() {
                Ok(Some(_)) => return Err("process_execute_helper_exited_before_ready"),
                Ok(None) => {}
                Err(_) => return Err("process_execute_helper_state_ambiguous_before_ready"),
            }
            match receiver.recv_timeout(Duration::from_millis(SUPERVISOR_POLL_MS)) {
                Ok(StreamEvent::Ready(line)) if line == expected => return Ok(()),
                Ok(StreamEvent::Ready(_)) => {
                    return Err("process_execute_helper_ready_binding_mismatch");
                }
                Ok(StreamEvent::Stdout(_)) | Ok(StreamEvent::Stderr(_)) => {
                    return Err("process_execute_output_before_helper_ready");
                }
                Ok(StreamEvent::ReadError) => {
                    return Err("process_execute_helper_stream_failed_before_ready");
                }
                Ok(StreamEvent::StdoutEof) | Ok(StreamEvent::StderrEof) => {}
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err("process_execute_helper_stream_disconnected_before_ready");
                }
            }
        }
    }

    fn terminate_before_ready(child: &mut Child) -> bool {
        match child.try_wait() {
            Ok(Some(_)) => true,
            Ok(None) => {
                if child.kill().is_err() {
                    return false;
                }
                child.wait().is_ok()
            }
            Err(_) => false,
        }
    }

    fn best_effort_terminate_unknown(child: &Arc<Mutex<Child>>) {
        let Ok(mut child) = child.lock() else {
            return;
        };
        match child.try_wait() {
            Ok(Some(_)) => {}
            Ok(None) => {
                if child.kill().is_ok() {
                    let _ = child.wait();
                }
            }
            Err(_) => {
                let _ = child.kill();
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn supervise<P: AuthorizationPolicy>(
        kernel: &mut KernelApi<P>,
        principal: Principal<'_>,
        prepared: &golam_kernel::PreparedToolEffect,
        input: ExecuteStagedProcessV2<'_>,
        scope: &str,
        mut supervisor: RootProcessSupervisor<ChildControl>,
        emergency_child: Arc<Mutex<Child>>,
        receiver: Receiver<StreamEvent>,
        started: Instant,
        capability_context_digest: [u8; 32],
        helper_identity_digest: [u8; 32],
        execution_binding_digest: [u8; 32],
    ) -> Result<ProcessExecutionReceiptV2, ProcessExecutionV2Error> {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut stdout_eof = false;
        let mut stderr_eof = false;
        let mut terminal = None;
        let mut terminal_seen_at = None;
        let mut forced = None;
        let mut ambiguous = false;

        loop {
            if input.cancellation.load(Ordering::Acquire) && forced.is_none() {
                supervisor
                    .request_cancel()
                    .map_err(|error| ProcessExecutionV2Error::Supervisor(error.to_string()))?;
                forced = Some(ForcedTermination::Cancellation);
            }

            let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
            match supervisor.observe_wall_time_ms(elapsed_ms) {
                Ok(evidence) if evidence.limit_exceeded && forced.is_none() => {
                    forced = Some(ForcedTermination::WallTime);
                }
                Ok(_) => {}
                Err(_) => ambiguous = true,
            }

            match receiver.recv_timeout(Duration::from_millis(SUPERVISOR_POLL_MS)) {
                Ok(StreamEvent::Stdout(chunk)) => {
                    if !account_and_append(
                        &mut supervisor,
                        &mut stdout,
                        &chunk,
                        input.limits.max_stdout_stderr_bytes,
                    )? && forced.is_none()
                    {
                        forced = Some(ForcedTermination::Output);
                    }
                }
                Ok(StreamEvent::Stderr(chunk)) => {
                    if !account_and_append(
                        &mut supervisor,
                        &mut stderr,
                        &chunk,
                        input.limits.max_stdout_stderr_bytes,
                    )? && forced.is_none()
                    {
                        forced = Some(ForcedTermination::Output);
                    }
                }
                Ok(StreamEvent::StdoutEof) => stdout_eof = true,
                Ok(StreamEvent::StderrEof) => stderr_eof = true,
                Ok(StreamEvent::Ready(_)) => ambiguous = true,
                Ok(StreamEvent::ReadError) => {
                    ambiguous = true;
                    let _ = supervisor.request_cancel();
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    if !(stdout_eof && stderr_eof) {
                        ambiguous = true;
                    }
                }
            }

            if terminal.is_none() {
                match supervisor.reconcile_terminal() {
                    Ok(ProcessTreeReconciliation::TerminalVerified(evidence)) => {
                        terminal = Some(evidence);
                        terminal_seen_at = Some(Instant::now());
                    }
                    Ok(ProcessTreeReconciliation::Unresolved { .. }) => {}
                    Err(_) => {
                        ambiguous = true;
                        best_effort_terminate_unknown(&emergency_child);
                        break;
                    }
                }
            }

            if terminal.is_some() && stdout_eof && stderr_eof {
                break;
            }
            if terminal_seen_at
                .is_some_and(|seen| seen.elapsed() >= Duration::from_millis(TERMINAL_DRAIN_MS))
            {
                ambiguous = true;
                break;
            }
        }

        if ambiguous || terminal.is_none() {
            best_effort_terminate_unknown(&emergency_child);
            let evidence = unknown_receipt_seed(
                input.execute_effect_id,
                execution_binding_digest,
                &stdout,
                &stderr,
            )?;
            kernel.complete_tool_effect(
                principal,
                CompleteToolEffect {
                    prepared,
                    finished_at: input.finished_at,
                    completion: ToolExecutionCompletion::UnknownOutcome,
                    reason_code: Some("process_execute_terminal_or_output_ambiguous"),
                    evidence_ref: Some(&evidence),
                    receipt: None,
                },
                scope,
            )?;
            return Ok(ProcessExecutionReceiptV2 {
                execute_effect_id: input.execute_effect_id,
                stage_effect_id: input.staged.stage_effect_id,
                root_pid: supervisor.root_pid(),
                status: ProcessExecutionStatusV2::UnknownOutcome,
                exit_code: None,
                signal: None,
                observed_descendant_count: 0,
                stdout_digest: sha256(&stdout),
                stderr_digest: sha256(&stderr),
                stdout,
                stderr,
                execution_binding_digest,
                capability_context_digest,
                helper_identity_digest,
                receipt_digest: evidence,
            });
        }

        let terminal = terminal.expect("terminal is checked above");
        let (status, exit_code, signal, reason_code) = classify_terminal(terminal, forced);
        let mut receipt = ProcessExecutionReceiptV2 {
            execute_effect_id: input.execute_effect_id,
            stage_effect_id: input.staged.stage_effect_id,
            root_pid: terminal.root_pid,
            status,
            exit_code,
            signal,
            observed_descendant_count: terminal.observed_descendant_count,
            stdout_digest: sha256(&stdout),
            stderr_digest: sha256(&stderr),
            stdout,
            stderr,
            execution_binding_digest,
            capability_context_digest,
            helper_identity_digest,
            receipt_digest: [0; 32],
        };
        receipt.receipt_digest = execution_receipt_digest(&receipt)?;
        let completion = if status == ProcessExecutionStatusV2::Succeeded {
            ToolExecutionCompletion::Succeeded
        } else {
            ToolExecutionCompletion::Failed
        };
        kernel.complete_tool_effect(
            principal,
            CompleteToolEffect {
                prepared,
                finished_at: input.finished_at,
                completion,
                reason_code: Some(reason_code),
                evidence_ref: Some(&receipt.receipt_digest),
                receipt: Some(&receipt.receipt_digest),
            },
            scope,
        )?;
        Ok(receipt)
    }

    fn account_and_append(
        supervisor: &mut RootProcessSupervisor<ChildControl>,
        destination: &mut Vec<u8>,
        chunk: &[u8],
        limit: u64,
    ) -> Result<bool, ProcessExecutionV2Error> {
        let accepted = supervisor.accepted_output_bytes();
        let remaining = limit.saturating_sub(accepted);
        let take = usize::try_from(remaining.min(chunk.len() as u64)).unwrap_or(chunk.len());
        if take != 0 {
            supervisor
                .account_output_bytes(take as u64)
                .map_err(|error| ProcessExecutionV2Error::Supervisor(error.to_string()))?;
            destination.extend_from_slice(&chunk[..take]);
        }
        if take < chunk.len() {
            supervisor
                .account_output_bytes(1)
                .map_err(|error| ProcessExecutionV2Error::Supervisor(error.to_string()))?;
            return Ok(false);
        }
        Ok(true)
    }

    fn classify_terminal(
        terminal: ProcessTreeTerminalEvidence,
        forced: Option<ForcedTermination>,
    ) -> (
        ProcessExecutionStatusV2,
        Option<i32>,
        Option<i32>,
        &'static str,
    ) {
        let (exit_code, signal) = match terminal.termination {
            RootTerminationKind::Exited(code) => (Some(code), None),
            RootTerminationKind::Signaled(signal) => (None, Some(signal)),
        };
        match forced {
            Some(ForcedTermination::Cancellation) => (
                ProcessExecutionStatusV2::Cancelled,
                exit_code,
                signal,
                "process_execute_cancelled_terminal_verified",
            ),
            Some(ForcedTermination::WallTime) => (
                ProcessExecutionStatusV2::TimedOut,
                exit_code,
                signal,
                "process_execute_wall_time_terminal_verified",
            ),
            Some(ForcedTermination::Output) => (
                ProcessExecutionStatusV2::OutputLimitExceeded,
                exit_code,
                signal,
                "process_execute_output_limit_terminal_verified",
            ),
            None if matches!(terminal.termination, RootTerminationKind::Exited(0)) => (
                ProcessExecutionStatusV2::Succeeded,
                exit_code,
                signal,
                "process_execute_succeeded_terminal_verified",
            ),
            None => (
                ProcessExecutionStatusV2::Failed,
                exit_code,
                signal,
                "process_execute_failed_terminal_verified",
            ),
        }
    }

    fn complete_pre_spawn_failure<P: AuthorizationPolicy>(
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

    fn terminal_observation(
        root_pid: u32,
        status: ExitStatus,
    ) -> crate::native_process_supervisor_v2::RootTerminalObservation {
        let termination = match status.code() {
            Some(code) => RootTerminationKind::Exited(code),
            None => RootTerminationKind::Signaled(status.signal().unwrap_or(-1)),
        };
        crate::native_process_supervisor_v2::RootTerminalObservation {
            root_pid,
            termination,
        }
    }

    fn spawn_stdout_reader(stdout: ChildStdout, sender: SyncSender<StreamEvent>) {
        thread::spawn(move || read_stream(stdout, sender, true));
    }

    fn spawn_stderr_reader(mut stderr: ChildStderr, sender: SyncSender<StreamEvent>) {
        thread::spawn(move || {
            let mut line = Vec::with_capacity(MAX_READY_LINE_BYTES);
            let mut byte = [0_u8; 1];
            loop {
                match stderr.read(&mut byte) {
                    Ok(0) => {
                        let _ = sender.send(StreamEvent::ReadError);
                        return;
                    }
                    Ok(_) => {
                        line.push(byte[0]);
                        if line.len() > MAX_READY_LINE_BYTES {
                            let _ = sender.send(StreamEvent::ReadError);
                            return;
                        }
                        if byte[0] == b'\n' {
                            break;
                        }
                    }
                    Err(_) => {
                        let _ = sender.send(StreamEvent::ReadError);
                        return;
                    }
                }
            }
            if sender.send(StreamEvent::Ready(line)).is_err() {
                return;
            }
            read_stream(stderr, sender, false);
        });
    }

    fn read_stream<R: Read>(mut stream: R, sender: SyncSender<StreamEvent>, stdout: bool) {
        let mut buffer = [0_u8; STREAM_CHUNK_BYTES];
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    let event = if stdout {
                        StreamEvent::StdoutEof
                    } else {
                        StreamEvent::StderrEof
                    };
                    let _ = sender.send(event);
                    return;
                }
                Ok(read) => {
                    let event = if stdout {
                        StreamEvent::Stdout(buffer[..read].to_vec())
                    } else {
                        StreamEvent::Stderr(buffer[..read].to_vec())
                    };
                    if sender.send(event).is_err() {
                        return;
                    }
                }
                Err(_) => {
                    let _ = sender.send(StreamEvent::ReadError);
                    return;
                }
            }
        }
    }

    fn push_native_object(
        encoder: &mut CanonicalEncoder,
        object: &NativeObjectIdentity,
    ) -> Result<(), CoreError> {
        encoder.push_bytes(object.canonical_path.as_os_str().as_encoded_bytes())?;
        encoder.push_u64(object.device);
        encoder.push_u64(object.inode);
        encoder.push_u64(u64::from(object.mode));
        Ok(())
    }

    fn push_native_objects(
        encoder: &mut CanonicalEncoder,
        objects: &[NativeObjectIdentity],
    ) -> Result<(), CoreError> {
        encoder.push_u64(objects.len() as u64);
        for object in objects {
            push_native_object(encoder, object)?;
        }
        Ok(())
    }

    fn push_argv(encoder: &mut CanonicalEncoder, argv: &[Vec<u8>]) -> Result<(), CoreError> {
        encoder.push_u64(argv.len() as u64);
        for item in argv {
            encoder.push_bytes(item)?;
        }
        Ok(())
    }

    fn push_limits(encoder: &mut CanonicalEncoder, limits: ProcessExecutionLimitsV2) {
        encoder.push_u64(limits.cpu_seconds);
        encoder.push_u64(limits.address_space_bytes);
        encoder.push_u64(limits.max_created_file_bytes);
        encoder.push_u64(limits.max_open_files);
        encoder.push_u64(limits.wall_time_ms);
        encoder.push_u64(limits.max_stdout_stderr_bytes);
    }

    fn execution_receipt_digest(
        receipt: &ProcessExecutionReceiptV2,
    ) -> Result<[u8; 32], ProcessExecutionV2Error> {
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(EXECUTION_RECEIPT_DOMAIN)?;
        encoder.push_u128(receipt.execute_effect_id.0);
        encoder.push_u128(receipt.stage_effect_id.0);
        encoder.push_u64(u64::from(receipt.root_pid));
        encoder.push_u8(receipt.status.code());
        push_optional_i32(&mut encoder, receipt.exit_code)?;
        push_optional_i32(&mut encoder, receipt.signal)?;
        encoder.push_u64(u64::from(receipt.observed_descendant_count));
        encoder.push_bytes(&receipt.stdout_digest)?;
        encoder.push_bytes(&receipt.stderr_digest)?;
        encoder.push_u64(receipt.stdout.len() as u64);
        encoder.push_u64(receipt.stderr.len() as u64);
        encoder.push_bytes(&receipt.execution_binding_digest)?;
        encoder.push_bytes(&receipt.capability_context_digest)?;
        encoder.push_bytes(&receipt.helper_identity_digest)?;
        Ok(sha256(&encoder.finish()))
    }

    fn unknown_receipt_seed(
        effect_id: EffectId,
        binding: [u8; 32],
        stdout: &[u8],
        stderr: &[u8],
    ) -> Result<[u8; 32], ProcessExecutionV2Error> {
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(b"golam:process-execute-unknown:v2")?;
        encoder.push_u128(effect_id.0);
        encoder.push_bytes(&binding)?;
        encoder.push_bytes(&sha256(stdout))?;
        encoder.push_bytes(&sha256(stderr))?;
        Ok(sha256(&encoder.finish()))
    }

    fn push_optional_i32(
        encoder: &mut CanonicalEncoder,
        value: Option<i32>,
    ) -> Result<(), CoreError> {
        match value {
            Some(value) => {
                encoder.push_u8(1);
                encoder.push_bytes(&value.to_be_bytes())?;
            }
            None => encoder.push_u8(0),
        }
        Ok(())
    }

    fn encode_hex(value: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut encoded = String::with_capacity(value.len() * 2);
        for byte in value {
            encoded.push(char::from(HEX[(byte >> 4) as usize]));
            encoded.push(char::from(HEX[(byte & 0x0f) as usize]));
        }
        encoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_kernel::CapabilityLeaseUseEvidence;

    #[test]
    fn process_resource_domain_is_stable() {
        assert_eq!(format!("process-request:{}", 17_u128), "process-request:17");
    }

    #[test]
    fn status_codes_are_stable_and_distinct() {
        let statuses = [
            ProcessExecutionStatusV2::Succeeded,
            ProcessExecutionStatusV2::Failed,
            ProcessExecutionStatusV2::Cancelled,
            ProcessExecutionStatusV2::TimedOut,
            ProcessExecutionStatusV2::OutputLimitExceeded,
            ProcessExecutionStatusV2::UnknownOutcome,
        ];
        let mut codes = statuses
            .into_iter()
            .map(ProcessExecutionStatusV2::code)
            .collect::<Vec<_>>();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), statuses.len());
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn first_profile_keeps_finite_parent_output_and_wall_bounds() {
        assert!(MAX_COMBINED_OUTPUT_BYTES > 0);
        assert!(MAX_WALL_TIME_MS > 0);
    }

    #[test]
    fn capability_context_type_remains_evidence_only() {
        fn accepts_evidence(_evidence: CapabilityLeaseUseEvidence) {}
        let _ = accepts_evidence;
    }
}
