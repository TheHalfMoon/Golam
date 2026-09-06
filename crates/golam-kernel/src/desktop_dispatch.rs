#![forbid(unsafe_code)]

use core::fmt;

use golam_core::desktop_backend::{
    DesktopActionDispatchContext, DesktopActionReceipt, DesktopBackend, DesktopBackendError,
    DesktopBackendTerminalStatus,
};
use golam_core::desktop_control::{
    DesktopControlLeaseState, DesktopLimits, PixelTargetHint, VisibleControlChannelState,
};
use golam_core::desktop_intent::{AuthorityBindings, DesktopActionKind};
use golam_core::tool_request::{BindingDigest, PreparedToolRequest};
use golam_core::{CanonicalEncoder, CoreError};

use crate::desktop_effect::{PreparedDesktopKernelAction, effect_binding};
use crate::{
    AuthorizationPolicy, CompleteToolEffect, KernelApi, Principal, ToolEffectError,
    ToolExecutionCompletion,
};

const ACTION_RECEIPT_DOMAIN: &[u8] = b"golam:desktop-action-receipt:v1";

pub struct DispatchDesktopAction<'a, B: DesktopBackend> {
    pub prepared: &'a PreparedDesktopKernelAction,
    pub current_request: &'a PreparedToolRequest,
    pub current_authority: AuthorityBindings,
    pub current_lease: &'a DesktopControlLeaseState,
    pub current_visible_channel: &'a VisibleControlChannelState,
    pub current_target_identity_digest: BindingDigest,
    pub observation_limits: DesktopLimits,
    pub pixel_hint: Option<&'a PixelTargetHint>,
    pub unresolved_conflicting_unknown_outcome: bool,
    pub now_unix_ms: u64,
    pub finished_at: &'a str,
    pub backend: &'a mut B,
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn dispatch_desktop_action<B: DesktopBackend>(
        &mut self,
        principal: Principal<'_>,
        input: DispatchDesktopAction<'_, B>,
        scope: &str,
    ) -> Result<DesktopActionReceipt, DesktopDispatchError> {
        if input.finished_at.is_empty() || input.now_unix_ms == 0 {
            return Err(DesktopDispatchError::InvalidDispatchInput);
        }

        let current_capabilities = input.backend.capabilities()?;
        let current_observation = input.backend.observe(input.observation_limits)?;
        let current_effect = effect_binding(input.prepared.effect())?;
        let current_request_digest = BindingDigest::new(input.current_request.binding_digest());
        let fallback_evidence = match input.prepared.intent().operation_kind {
            DesktopActionKind::RawInputFallback => Some(input.prepared.route_evidence()),
            DesktopActionKind::SemanticAction | DesktopActionKind::Focus => None,
        };

        let context = DesktopActionDispatchContext {
            action: input.prepared.intent(),
            now_unix_ms: input.now_unix_ms,
            current_request_digest,
            current_effect_digest: current_effect.immutable_effect_digest,
            current_gate_authorization_digest: current_effect.gate_authorization_digest,
            current_authority: input.current_authority,
            current_capabilities: &current_capabilities,
            current_observation: &current_observation,
            current_target_identity_digest: input.current_target_identity_digest,
            current_lease: input.current_lease,
            current_visible_channel: input.current_visible_channel,
            fallback_evidence,
            pixel_hint: input.pixel_hint,
            unresolved_conflicting_unknown_outcome: input.unresolved_conflicting_unknown_outcome,
        };

        let validated = match context.authorize() {
            Ok(validated) => validated,
            Err(error) => {
                self.complete_desktop_dispatch(
                    principal,
                    input.prepared,
                    input.finished_at,
                    ToolExecutionCompletion::Failed,
                    "desktop_revalidation_failed_before_dispatch",
                    None,
                    scope,
                )?;
                return Err(DesktopDispatchError::Revalidation(error));
            }
        };

        let receipt = match input.backend.dispatch_action(validated) {
            Ok(receipt) => receipt,
            Err(error) => {
                self.complete_desktop_dispatch(
                    principal,
                    input.prepared,
                    input.finished_at,
                    ToolExecutionCompletion::UnknownOutcome,
                    "desktop_adapter_error_after_dispatch_boundary",
                    None,
                    scope,
                )?;
                return Err(DesktopDispatchError::AdapterUncertain(error));
            }
        };

        let receipt_bytes = action_receipt_bytes(&receipt)?;
        let completion = classify_action_receipt(input.prepared, &receipt);
        let reason = match completion {
            ToolExecutionCompletion::Succeeded => "desktop_action_succeeded",
            ToolExecutionCompletion::Failed => "desktop_action_terminal_failure",
            ToolExecutionCompletion::UnknownOutcome => "desktop_action_unknown_outcome",
        };
        self.complete_desktop_dispatch(
            principal,
            input.prepared,
            input.finished_at,
            completion,
            reason,
            Some(receipt_bytes.as_slice()),
            scope,
        )?;

        if completion == ToolExecutionCompletion::UnknownOutcome {
            return Err(DesktopDispatchError::UnknownOutcome(receipt));
        }
        Ok(receipt)
    }

    fn complete_desktop_dispatch(
        &mut self,
        principal: Principal<'_>,
        prepared: &PreparedDesktopKernelAction,
        finished_at: &str,
        completion: ToolExecutionCompletion,
        reason_code: &str,
        receipt: Option<&[u8]>,
        scope: &str,
    ) -> Result<(), DesktopDispatchError> {
        self.complete_tool_effect(
            principal,
            CompleteToolEffect {
                prepared: prepared.effect(),
                finished_at,
                completion,
                reason_code: Some(reason_code),
                evidence_ref: receipt,
                receipt,
            },
            scope,
        )?;
        Ok(())
    }
}

fn classify_action_receipt(
    prepared: &PreparedDesktopKernelAction,
    receipt: &DesktopActionReceipt,
) -> ToolExecutionCompletion {
    if receipt.observed_target_digest != prepared.intent().exact_target_identity_digest {
        return ToolExecutionCompletion::UnknownOutcome;
    }
    match receipt.status {
        DesktopBackendTerminalStatus::Committed if receipt.post_observation_digest.is_some() => {
            ToolExecutionCompletion::Succeeded
        }
        DesktopBackendTerminalStatus::Committed => ToolExecutionCompletion::UnknownOutcome,
        DesktopBackendTerminalStatus::FailedBeforeEffect
        | DesktopBackendTerminalStatus::Interrupted
        | DesktopBackendTerminalStatus::NotSupported => ToolExecutionCompletion::Failed,
        DesktopBackendTerminalStatus::UnknownOutcome => ToolExecutionCompletion::UnknownOutcome,
    }
}

fn action_receipt_bytes(receipt: &DesktopActionReceipt) -> Result<Vec<u8>, DesktopDispatchError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(ACTION_RECEIPT_DOMAIN)?;
    encoder.push_u8(status_code(receipt.status));
    encoder.push_bytes(&receipt.observed_target_digest.bytes())?;
    push_optional_digest(&mut encoder, receipt.post_observation_digest)?;
    push_optional_digest(&mut encoder, receipt.sanitized_error_class)?;
    Ok(encoder.finish())
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

const fn status_code(status: DesktopBackendTerminalStatus) -> u8 {
    match status {
        DesktopBackendTerminalStatus::Committed => 1,
        DesktopBackendTerminalStatus::FailedBeforeEffect => 2,
        DesktopBackendTerminalStatus::UnknownOutcome => 3,
        DesktopBackendTerminalStatus::Interrupted => 4,
        DesktopBackendTerminalStatus::NotSupported => 5,
    }
}

#[derive(Debug)]
pub enum DesktopDispatchError {
    InvalidDispatchInput,
    Revalidation(DesktopBackendError),
    AdapterUncertain(DesktopBackendError),
    UnknownOutcome(DesktopActionReceipt),
    Backend(DesktopBackendError),
    Kernel(crate::DesktopKernelError),
    Effect(ToolEffectError),
    Core(CoreError),
}

impl fmt::Display for DesktopDispatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDispatchInput => f.write_str("invalid desktop dispatch input"),
            Self::Revalidation(error) => write!(f, "desktop revalidation failed: {error}"),
            Self::AdapterUncertain(error) => {
                write!(f, "desktop adapter failed after dispatch boundary: {error}")
            }
            Self::UnknownOutcome(_) => f.write_str("desktop action outcome is unknown"),
            Self::Backend(error) => write!(f, "desktop backend error: {error}"),
            Self::Kernel(error) => write!(f, "desktop kernel error: {error}"),
            Self::Effect(error) => write!(f, "desktop effect error: {error}"),
            Self::Core(error) => write!(f, "desktop receipt encoding error: {error}"),
        }
    }
}

impl std::error::Error for DesktopDispatchError {}

impl From<DesktopBackendError> for DesktopDispatchError {
    fn from(value: DesktopBackendError) -> Self {
        Self::Backend(value)
    }
}

impl From<crate::DesktopKernelError> for DesktopDispatchError {
    fn from(value: crate::DesktopKernelError) -> Self {
        Self::Kernel(value)
    }
}

impl From<ToolEffectError> for DesktopDispatchError {
    fn from(value: ToolEffectError) -> Self {
        Self::Effect(value)
    }
}

impl From<CoreError> for DesktopDispatchError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}
