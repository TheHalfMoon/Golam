#![forbid(unsafe_code)]

//! Governed executable-skill bridge for Spec 005.
//!
//! This module grants no authority. It binds an already-reviewed executable skill target to the
//! existing admitted Linux x86_64 `process.stage` and `process.execute` boundaries. The exact live
//! skill package, lifecycle state, queued ToolRequest binding and reviewed target are revalidated
//! before staging and again immediately before dispatch. Unqualified platforms remain denied by
//! the existing process boundary.

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use golam_core::skills_protocol::SkillDispatchBinding;
use golam_core::tool_request::{BindingDigest, PreparedToolRequest};
use golam_core::{EffectId, SessionId};
use golam_kernel::{AuthorizationPolicy, CapabilityLease, KernelApi, Principal};

use crate::process_dispatch_v2::{
    ExecuteStagedProcessV2, ProcessExecutionLimitsV2, ProcessExecutionReceiptV2,
    ProcessExecutionV2Error, execute_staged_process_v2,
};
use crate::process_execution_v2::{
    PROCESS_EXECUTE_ACTION, ProcessStageError, StageProcessExecutable, StagedExecutableV2,
    stage_process_executable_v2,
};
use crate::skill_packages::{ExecutableSkillTarget, SkillLifecycle, SkillPackageError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedSkillExecutableV2 {
    pub skill_binding: SkillDispatchBinding,
    pub skill_package_ref: BindingDigest,
    pub skill_content_digest: BindingDigest,
    pub reviewed_capability_mapping_ref: BindingDigest,
    pub request_binding_ref: BindingDigest,
    pub target: ExecutableSkillTarget,
    pub staged: StagedExecutableV2,
}

pub struct ExecuteStagedSkillV2<'a> {
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
pub enum SkillProcessV2Error {
    Skill(SkillPackageError),
    Stage(ProcessStageError),
    Execute(ProcessExecutionV2Error),
    InvalidRequestBinding(&'static str),
    StaleQueuedRequest,
    PackageRootMismatch,
    StagedTargetMismatch,
    StagedSkillBindingMismatch,
}

impl fmt::Display for SkillProcessV2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Skill(error) => write!(f, "skill dispatch revalidation failed: {error}"),
            Self::Stage(error) => write!(f, "skill executable staging failed: {error}"),
            Self::Execute(error) => write!(f, "skill executable dispatch failed: {error}"),
            Self::InvalidRequestBinding(reason) => {
                write!(
                    f,
                    "skill executable ToolRequest binding is invalid: {reason}"
                )
            }
            Self::StaleQueuedRequest => f.write_str(
                "skill executable queued request no longer matches the prepared ToolRequest",
            ),
            Self::PackageRootMismatch => f.write_str(
                "skill executable source resolver is not rooted at the reviewed package",
            ),
            Self::StagedTargetMismatch => {
                f.write_str("staged executable no longer matches the reviewed skill target")
            }
            Self::StagedSkillBindingMismatch => {
                f.write_str("staged executable is bound to a different reviewed skill dispatch")
            }
        }
    }
}

impl Error for SkillProcessV2Error {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Skill(error) => Some(error),
            Self::Stage(error) => Some(error),
            Self::Execute(error) => Some(error),
            _ => None,
        }
    }
}

impl From<SkillPackageError> for SkillProcessV2Error {
    fn from(value: SkillPackageError) -> Self {
        Self::Skill(value)
    }
}

impl From<ProcessStageError> for SkillProcessV2Error {
    fn from(value: ProcessStageError) -> Self {
        Self::Stage(value)
    }
}

impl From<ProcessExecutionV2Error> for SkillProcessV2Error {
    fn from(value: ProcessExecutionV2Error) -> Self {
        Self::Execute(value)
    }
}

pub fn stage_skill_executable_v2<P: AuthorizationPolicy>(
    kernel: &mut KernelApi<P>,
    principal: Principal<'_>,
    lifecycle: &SkillLifecycle,
    binding: &SkillDispatchBinding,
    input: StageProcessExecutable<'_>,
    scope: &str,
) -> Result<StagedSkillExecutableV2, SkillProcessV2Error> {
    let request_binding_ref = validate_request_binding(lifecycle, binding, input.request)?;
    if input.source_resolver.root_path() != lifecycle.reviewed().canonical_root() {
        return Err(SkillProcessV2Error::PackageRootMismatch);
    }
    let relative_target = requested_target(input.request)?;
    let target = lifecycle.revalidate_executable_target(binding, relative_target)?;
    let descriptor = lifecycle.reviewed().descriptor();
    let staged = stage_process_executable_v2(kernel, principal, input, scope)?;
    if staged.content_digest != target.content_digest.bytes() || staged.byte_len != target.byte_len
    {
        return Err(SkillProcessV2Error::StagedTargetMismatch);
    }
    Ok(StagedSkillExecutableV2 {
        skill_binding: binding.clone(),
        skill_package_ref: descriptor.package_ref,
        skill_content_digest: descriptor.content_digest,
        reviewed_capability_mapping_ref: lifecycle.reviewed().reviewed_capability_mapping_ref(),
        request_binding_ref,
        target,
        staged,
    })
}

pub fn execute_staged_skill_v2<P: AuthorizationPolicy>(
    kernel: &mut KernelApi<P>,
    principal: Principal<'_>,
    lifecycle: &SkillLifecycle,
    binding: &SkillDispatchBinding,
    staged_skill: &StagedSkillExecutableV2,
    input: ExecuteStagedSkillV2<'_>,
    scope: &str,
) -> Result<ProcessExecutionReceiptV2, SkillProcessV2Error> {
    let request_binding_ref = validate_request_binding(lifecycle, binding, input.request)?;
    if &staged_skill.skill_binding != binding {
        return Err(SkillProcessV2Error::StagedSkillBindingMismatch);
    }
    if staged_skill.request_binding_ref != request_binding_ref {
        return Err(SkillProcessV2Error::StaleQueuedRequest);
    }
    let relative_target = requested_target(input.request)?;
    let live_target = lifecycle.revalidate_executable_target(binding, relative_target)?;
    let descriptor = lifecycle.reviewed().descriptor();
    if staged_skill.skill_package_ref != descriptor.package_ref
        || staged_skill.skill_content_digest != descriptor.content_digest
        || staged_skill.reviewed_capability_mapping_ref
            != lifecycle.reviewed().reviewed_capability_mapping_ref()
        || staged_skill.target != live_target
        || staged_skill.staged.content_digest != live_target.content_digest.bytes()
        || staged_skill.staged.byte_len != live_target.byte_len
    {
        return Err(SkillProcessV2Error::StagedTargetMismatch);
    }

    Ok(execute_staged_process_v2(
        kernel,
        principal,
        ExecuteStagedProcessV2 {
            request: input.request,
            lease: input.lease,
            staged: &staged_skill.staged,
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

fn validate_request_binding(
    lifecycle: &SkillLifecycle,
    binding: &SkillDispatchBinding,
    request: &PreparedToolRequest,
) -> Result<BindingDigest, SkillProcessV2Error> {
    if request.request().requested_operation.as_str() != PROCESS_EXECUTE_ACTION {
        return Err(SkillProcessV2Error::InvalidRequestBinding(
            "operation is not process.execute",
        ));
    }
    let request_binding_ref = BindingDigest::new(request.binding_digest());
    if binding.queued_request_ref != request_binding_ref {
        return Err(SkillProcessV2Error::StaleQueuedRequest);
    }
    let relative_target = requested_target(request)?;
    lifecycle.revalidate_executable_target(binding, relative_target)?;
    Ok(request_binding_ref)
}

fn requested_target(request: &PreparedToolRequest) -> Result<&str, SkillProcessV2Error> {
    request
        .request()
        .requested_target
        .as_ref()
        .map(|target| target.as_str())
        .ok_or(SkillProcessV2Error::InvalidRequestBinding(
            "process request has no executable target",
        ))
}
