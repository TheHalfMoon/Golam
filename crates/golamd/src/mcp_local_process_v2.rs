#![forbid(unsafe_code)]

//! Governed local-MCP process bridge for Spec 005.
//!
//! This module grants no process, filesystem, network, secret, capability or approval authority.
//! A reviewed `LocalStdio` MCP binding may only stage and launch through the already-admitted
//! Linux x86_64 process boundary. The exact active MCP binding/version/mapping/lifecycle and the
//! queued `PreparedToolRequest` are revalidated before staging and again immediately before
//! dispatch. The reviewed local server identity is the exact source target identity observed by
//! `process.stage`; a different executable fails closed.

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use golam_core::digest::sha256;
use golam_core::skills_protocol::{McpDispatchBinding, McpTransport};
use golam_core::tool_request::{BindingDigest, PreparedToolRequest};
use golam_core::{EffectId, SessionId};
use golam_kernel::{AuthorizationPolicy, CapabilityLease, KernelApi, Principal};

use crate::mcp_protocol::{McpLifecycle, McpProtocolError};
use crate::native_containment_v2::PROFILE_TOKEN;
use crate::process_dispatch_v2::{
    ExecuteStagedProcessV2, ProcessExecutionLimitsV2, ProcessExecutionReceiptV2,
    ProcessExecutionV2Error, execute_staged_process_v2,
};
use crate::process_execution_v2::{
    PROCESS_EXECUTE_ACTION, ProcessStageError, StageProcessExecutable, StagedExecutableV2,
    stage_process_executable_v2,
};

const MCP_LOCAL_PROFILE_DOMAIN: &[u8] = b"golam:mcp-local-process-profile:v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedLocalMcpServerV2 {
    pub mcp_dispatch: McpDispatchBinding,
    pub binding_id: BindingDigest,
    pub binding_digest: BindingDigest,
    pub server_identity: BindingDigest,
    pub process_profile_ref: BindingDigest,
    pub golam_local_mapping_ref: BindingDigest,
    pub golam_local_mapping_digest: BindingDigest,
    pub request_binding_ref: BindingDigest,
    pub staged: StagedExecutableV2,
}

pub struct ExecuteStagedLocalMcpServerV2<'a> {
    pub request: &'a PreparedToolRequest,
    pub lease: &'a CapabilityLease,
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
pub enum McpLocalProcessV2Error {
    Mcp(McpProtocolError),
    Stage(ProcessStageError),
    Execute(ProcessExecutionV2Error),
    InvalidTransport,
    UnadmittedProcessProfile,
    InvalidRequestBinding(&'static str),
    StaleQueuedRequest,
    ServerExecutableIdentityMismatch,
    StagedBindingMismatch,
}

impl fmt::Display for McpLocalProcessV2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mcp(error) => write!(f, "local MCP dispatch revalidation failed: {error}"),
            Self::Stage(error) => write!(f, "local MCP server staging failed: {error}"),
            Self::Execute(error) => write!(f, "local MCP server dispatch failed: {error}"),
            Self::InvalidTransport => f.write_str("local MCP process dispatch requires LocalStdio"),
            Self::UnadmittedProcessProfile => f.write_str(
                "local MCP binding does not name the exact admitted production process profile",
            ),
            Self::InvalidRequestBinding(reason) => {
                write!(f, "local MCP ToolRequest binding is invalid: {reason}")
            }
            Self::StaleQueuedRequest => f.write_str(
                "local MCP queued request no longer matches the prepared ToolRequest",
            ),
            Self::ServerExecutableIdentityMismatch => f.write_str(
                "local MCP staged executable does not match the reviewed server identity",
            ),
            Self::StagedBindingMismatch => f.write_str(
                "local MCP staged server is bound to a different reviewed dispatch state",
            ),
        }
    }
}

impl Error for McpLocalProcessV2Error {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Mcp(error) => Some(error),
            Self::Stage(error) => Some(error),
            Self::Execute(error) => Some(error),
            _ => None,
        }
    }
}

impl From<McpProtocolError> for McpLocalProcessV2Error {
    fn from(value: McpProtocolError) -> Self {
        Self::Mcp(value)
    }
}

impl From<ProcessStageError> for McpLocalProcessV2Error {
    fn from(value: ProcessStageError) -> Self {
        Self::Stage(value)
    }
}

impl From<ProcessExecutionV2Error> for McpLocalProcessV2Error {
    fn from(value: ProcessExecutionV2Error) -> Self {
        Self::Execute(value)
    }
}

pub fn admitted_local_mcp_profile_ref() -> BindingDigest {
    let mut identity = Vec::with_capacity(MCP_LOCAL_PROFILE_DOMAIN.len() + PROFILE_TOKEN.len());
    identity.extend_from_slice(MCP_LOCAL_PROFILE_DOMAIN);
    identity.extend_from_slice(PROFILE_TOKEN.as_bytes());
    BindingDigest::new(sha256(&identity))
}

pub fn stage_local_mcp_server_v2<P: AuthorizationPolicy>(
    kernel: &mut KernelApi<P>,
    principal: Principal<'_>,
    lifecycle: &McpLifecycle,
    dispatch: &McpDispatchBinding,
    input: StageProcessExecutable<'_>,
    scope: &str,
) -> Result<StagedLocalMcpServerV2, McpLocalProcessV2Error> {
    let request_binding_ref = validate_local_request_binding(lifecycle, dispatch, input.request)?;
    let binding = lifecycle.binding();
    let staged = stage_process_executable_v2(kernel, principal, input, scope)?;
    if staged.source_target_identity != binding.server_identity {
        return Err(McpLocalProcessV2Error::ServerExecutableIdentityMismatch);
    }
    Ok(StagedLocalMcpServerV2 {
        mcp_dispatch: dispatch.clone(),
        binding_id: binding.binding_id,
        binding_digest: binding.binding_digest,
        server_identity: binding.server_identity,
        process_profile_ref: binding.process_profile_ref_or_remote_endpoint,
        golam_local_mapping_ref: binding.golam_local_mapping_ref,
        golam_local_mapping_digest: binding.golam_local_mapping_digest,
        request_binding_ref,
        staged,
    })
}

pub fn execute_staged_local_mcp_server_v2<P: AuthorizationPolicy>(
    kernel: &mut KernelApi<P>,
    principal: Principal<'_>,
    lifecycle: &McpLifecycle,
    dispatch: &McpDispatchBinding,
    staged_server: &StagedLocalMcpServerV2,
    input: ExecuteStagedLocalMcpServerV2<'_>,
    scope: &str,
) -> Result<ProcessExecutionReceiptV2, McpLocalProcessV2Error> {
    let request_binding_ref = validate_local_request_binding(lifecycle, dispatch, input.request)?;
    let binding = lifecycle.binding();
    if &staged_server.mcp_dispatch != dispatch
        || staged_server.binding_id != binding.binding_id
        || staged_server.binding_digest != binding.binding_digest
        || staged_server.server_identity != binding.server_identity
        || staged_server.process_profile_ref != binding.process_profile_ref_or_remote_endpoint
        || staged_server.golam_local_mapping_ref != binding.golam_local_mapping_ref
        || staged_server.golam_local_mapping_digest != binding.golam_local_mapping_digest
        || staged_server.request_binding_ref != request_binding_ref
    {
        return Err(McpLocalProcessV2Error::StagedBindingMismatch);
    }
    if staged_server.staged.source_target_identity != binding.server_identity {
        return Err(McpLocalProcessV2Error::ServerExecutableIdentityMismatch);
    }

    Ok(execute_staged_process_v2(
        kernel,
        principal,
        ExecuteStagedProcessV2 {
            request: input.request,
            lease: input.lease,
            staged: &staged_server.staged,
            helper_path: input.helper_path,
            cwd: input.cwd,
            filesystem_read_paths: input.filesystem_read_paths,
            filesystem_write_paths: input.filesystem_write_paths,
            argv: input.argv,
            limits: input.limits,
            execute_effect_id: input.execute_effect_id,
            session_id: input.session_id,
            started_at: input.started_at,
            dispatch_at: input.dispatch_at,
            finished_at: input.finished_at,
            cancellation: input.cancellation,
        },
        scope,
    )?)
}

fn validate_local_request_binding(
    lifecycle: &McpLifecycle,
    dispatch: &McpDispatchBinding,
    request: &PreparedToolRequest,
) -> Result<BindingDigest, McpLocalProcessV2Error> {
    validate_local_binding(lifecycle, dispatch)?;
    if request.request().requested_operation.as_str() != PROCESS_EXECUTE_ACTION {
        return Err(McpLocalProcessV2Error::InvalidRequestBinding(
            "operation is not process.execute",
        ));
    }
    let request_binding_ref = BindingDigest::new(request.binding_digest());
    if dispatch.queued_request_ref != request_binding_ref {
        return Err(McpLocalProcessV2Error::StaleQueuedRequest);
    }
    Ok(request_binding_ref)
}

fn validate_local_binding(
    lifecycle: &McpLifecycle,
    dispatch: &McpDispatchBinding,
) -> Result<(), McpLocalProcessV2Error> {
    lifecycle.revalidate_dispatch(dispatch)?;
    let binding = lifecycle.binding();
    if binding.transport != McpTransport::LocalStdio {
        return Err(McpLocalProcessV2Error::InvalidTransport);
    }
    if binding.process_profile_ref_or_remote_endpoint != admitted_local_mcp_profile_ref() {
        return Err(McpLocalProcessV2Error::UnadmittedProcessProfile);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::skills_protocol::{McpLifecycleState, McpVersionLock, ProtocolFeatureId};
    use golam_core::taint::TaintSet;
    use crate::mcp_protocol::McpReviewRequest;

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn lifecycle(transport: McpTransport, profile: BindingDigest) -> McpLifecycle {
        let mut lifecycle = McpLifecycle::review(McpReviewRequest {
            binding_id: digest(1),
            server_identity: digest(2),
            transport,
            process_profile_ref_or_remote_endpoint: profile,
            allowed_protocol_features: Vec::<ProtocolFeatureId>::new(),
            golam_local_mapping_ref: digest(3),
            golam_local_mapping_digest: digest(4),
            network_policy_ref: if transport == McpTransport::RemoteHttp {
                digest(5)
            } else {
                BindingDigest::new([0; 32])
            },
            secret_policy_ref: digest(6),
            taint_class: TaintSet::empty(),
            version_lock: McpVersionLock::new("2025-06-18").unwrap(),
        })
        .unwrap();
        lifecycle.transition(McpLifecycleState::Active).unwrap();
        lifecycle
    }

    #[test]
    fn admitted_profile_ref_is_stable_and_nonzero() {
        let profile = admitted_local_mcp_profile_ref();
        assert_ne!(profile.bytes(), [0; 32]);
        assert_eq!(profile, admitted_local_mcp_profile_ref());
    }

    #[test]
    fn local_binding_requires_exact_admitted_profile() {
        let lifecycle = lifecycle(McpTransport::LocalStdio, admitted_local_mcp_profile_ref());
        let dispatch = lifecycle
            .bind_dispatch(digest(7), digest(8), digest(9))
            .unwrap();
        assert!(validate_local_binding(&lifecycle, &dispatch).is_ok());

        let wrong = lifecycle(McpTransport::LocalStdio, digest(99));
        let wrong_dispatch = wrong
            .bind_dispatch(digest(7), digest(8), digest(9))
            .unwrap();
        assert!(matches!(
            validate_local_binding(&wrong, &wrong_dispatch),
            Err(McpLocalProcessV2Error::UnadmittedProcessProfile)
        ));
    }

    #[test]
    fn remote_and_revoked_bindings_fail_before_process_dispatch() {
        let remote = lifecycle(McpTransport::RemoteHttp, digest(10));
        let remote_dispatch = remote
            .bind_dispatch(digest(7), digest(8), digest(9))
            .unwrap();
        assert!(matches!(
            validate_local_binding(&remote, &remote_dispatch),
            Err(McpLocalProcessV2Error::InvalidTransport)
        ));

        let mut revoked = lifecycle(McpTransport::LocalStdio, admitted_local_mcp_profile_ref());
        let stale = revoked
            .bind_dispatch(digest(7), digest(8), digest(9))
            .unwrap();
        revoked.transition(McpLifecycleState::Revoked).unwrap();
        assert!(matches!(
            validate_local_binding(&revoked, &stale),
            Err(McpLocalProcessV2Error::Mcp(_))
        ));
    }
}
