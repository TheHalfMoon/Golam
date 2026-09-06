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
use golam_core::digest::sha256;
use golam_core::tool_request::{BindingDigest, PreparedToolRequest};
use golam_core::{CanonicalEncoder, CoreError};
use golam_ledger::desktop_control_evidence::{
    DesktopControlEvidenceError, DesktopControlEvidenceStore, DesktopEffectEvidence,
    DesktopEvidenceOperation, DesktopEvidenceStatus,
};

use crate::desktop_effect::{PreparedDesktopKernelAction, effect_binding};
use crate::{
    AuthorizationPolicy, CompleteToolEffect, KernelApi, Principal, ToolEffectError,
    ToolExecutionCompletion,
};

const ACTION_RECEIPT_DOMAIN: &[u8] = b"golam:desktop-action-receipt:v1";
const LEASE_BINDING_DOMAIN: &[u8] = b"golam:desktop-prepared-lease-binding:v1";

pub struct DispatchDesktopAction<'a, B: DesktopBackend> {
    pub prepared: &'a PreparedDesktopKernelAction,
    pub current_request: &'a PreparedToolRequest,
    pub current_authority: AuthorityBindings,
    pub current_lease: &'a DesktopControlLeaseState,
    pub current_visible_channel: &'a VisibleControlChannelState,
    pub current_target_identity_digest: BindingDigest,
    pub observation_limits: DesktopLimits,
    pub pixel_hint: Option<&'a PixelTargetHint>,
    /// Caller input can only make dispatch more conservative. Protected durable
    /// evidence is always consulted and cannot be cleared by passing `false`.
    pub unresolved_conflicting_unknown_outcome: bool,
    pub now_unix_ms: u64,
    pub finished_at: &'a str,
    pub backend: &'a mut B,
}

struct CompleteDesktopAction<'a> {
    prepared: &'a PreparedDesktopKernelAction,
    finished_at: &'a str,
    completion: ToolExecutionCompletion,
    evidence_status: DesktopEvidenceStatus,
    reason_code: &'a str,
    receipt: Option<&'a [u8]>,
    recorded_at_unix_ms: u64,
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

        let mut evidence_store = DesktopControlEvidenceStore::open(&self.authority)?;
        let durable_unknown = evidence_store
            .has_unresolved_unknown_outcome_for_effect(input.prepared.effect().effect_id())?;
        let current_capabilities = input.backend.capabilities()?;
        let current_observation = input.backend.observe(input.observation_limits)?;
        let current_effect = effect_binding(input.prepared.effect())?;
        let current_request_digest = BindingDigest::new(input.current_request.binding_digest());
        let fallback_evidence = match input.prepared.intent().operation_kind {
            DesktopActionKind::RawInputFallback => Some(input.prepared.route_evidence()),
            DesktopActionKind::SemanticAction | DesktopActionKind::Focus => None,
        };

        evidence_store.append_effect_evidence(action_evidence(
            &evidence_store,
            input.prepared,
            DesktopEvidenceStatus::Prepared,
            input.now_unix_ms,
        )?)?;

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
            unresolved_conflicting_unknown_outcome: durable_unknown
                || input.unresolved_conflicting_unknown_outcome,
        };

        let validated = match context.authorize() {
            Ok(validated) => validated,
            Err(error) => {
                self.complete_desktop_dispatch(
                    &mut evidence_store,
                    principal,
                    CompleteDesktopAction {
                        prepared: input.prepared,
                        finished_at: input.finished_at,
                        completion: ToolExecutionCompletion::Failed,
                        evidence_status: DesktopEvidenceStatus::Failed,
                        reason_code: "desktop_revalidation_failed_before_dispatch",
                        receipt: None,
                        recorded_at_unix_ms: input.now_unix_ms,
                    },
                    scope,
                )?;
                return Err(DesktopDispatchError::Revalidation(error));
            }
        };

        let receipt = match input.backend.dispatch_action(validated) {
            Ok(receipt) => receipt,
            Err(error) => {
                self.complete_desktop_dispatch(
                    &mut evidence_store,
                    principal,
                    CompleteDesktopAction {
                        prepared: input.prepared,
                        finished_at: input.finished_at,
                        completion: ToolExecutionCompletion::UnknownOutcome,
                        evidence_status: DesktopEvidenceStatus::UnknownOutcome,
                        reason_code: "desktop_adapter_error_after_dispatch_boundary",
                        receipt: None,
                        recorded_at_unix_ms: input.now_unix_ms,
                    },
                    scope,
                )?;
                return Err(DesktopDispatchError::AdapterUncertain(error));
            }
        };

        let receipt_bytes = action_receipt_bytes(&receipt)?;
        let completion = classify_action_receipt(input.prepared, &receipt);
        let evidence_status = evidence_status_for_receipt(completion, receipt.status);
        let reason_code = match completion {
            ToolExecutionCompletion::Succeeded => "desktop_action_succeeded",
            ToolExecutionCompletion::Failed => "desktop_action_terminal_failure",
            ToolExecutionCompletion::UnknownOutcome => "desktop_action_unknown_outcome",
        };
        self.complete_desktop_dispatch(
            &mut evidence_store,
            principal,
            CompleteDesktopAction {
                prepared: input.prepared,
                finished_at: input.finished_at,
                completion,
                evidence_status,
                reason_code,
                receipt: Some(receipt_bytes.as_slice()),
                recorded_at_unix_ms: input.now_unix_ms,
            },
            scope,
        )?;

        if completion == ToolExecutionCompletion::UnknownOutcome {
            return Err(DesktopDispatchError::UnknownOutcome(receipt));
        }
        Ok(receipt)
    }

    fn complete_desktop_dispatch(
        &mut self,
        evidence_store: &mut DesktopControlEvidenceStore,
        principal: Principal<'_>,
        input: CompleteDesktopAction<'_>,
        scope: &str,
    ) -> Result<(), DesktopDispatchError> {
        self.complete_tool_effect(
            principal,
            CompleteToolEffect {
                prepared: input.prepared.effect(),
                finished_at: input.finished_at,
                completion: input.completion,
                reason_code: Some(input.reason_code),
                evidence_ref: input.receipt,
                receipt: input.receipt,
            },
            scope,
        )?;
        evidence_store.append_effect_evidence(action_evidence(
            evidence_store,
            input.prepared,
            input.evidence_status,
            input.recorded_at_unix_ms,
        )?)?;
        Ok(())
    }
}

fn action_evidence(
    store: &DesktopControlEvidenceStore,
    prepared: &PreparedDesktopKernelAction,
    status: DesktopEvidenceStatus,
    recorded_at_unix_ms: u64,
) -> Result<DesktopEffectEvidence, DesktopDispatchError> {
    let intent = prepared.intent();
    Ok(DesktopEffectEvidence {
        effect_id: prepared.effect().effect_id(),
        session_id: store.effect_session_id(prepared.effect().effect_id())?,
        operation: match intent.operation_kind {
            DesktopActionKind::SemanticAction => DesktopEvidenceOperation::SemanticAction,
            DesktopActionKind::Focus => DesktopEvidenceOperation::Focus,
            DesktopActionKind::RawInputFallback => DesktopEvidenceOperation::RawInputFallback,
        },
        request_digest: intent.request.canonical_request_digest,
        effect_digest: intent.effect.immutable_effect_digest,
        intent_digest: intent
            .intent_digest()
            .map_err(crate::DesktopKernelError::from)?,
        fallback_eligibility_digest: intent.fallback_eligibility_evidence_digest,
        control_lease_digest: Some(prepared_lease_binding_digest(intent)?),
        visible_channel_digest: Some(intent.interactive_authority.visible_channel_state_digest),
        permission_session_digest: intent.prepared_permission_session_evidence_ref,
        target_or_source_digest: intent.exact_target_identity_digest,
        status,
        reconciliation_ref: None,
        recorded_at_unix_ms,
    })
}

fn prepared_lease_binding_digest(
    intent: &golam_core::desktop_intent::PreparedDesktopAction,
) -> Result<BindingDigest, DesktopDispatchError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(LEASE_BINDING_DOMAIN)?;
    encoder.push_u128(intent.interactive_authority.lease_id.as_u128());
    encoder.push_u64(intent.interactive_authority.lease_generation);
    encoder.push_bytes(&intent.authority.capability_ref.bytes())?;
    encoder.push_bytes(&intent.authority.policy_ref.bytes())?;
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn evidence_status_for_receipt(
    completion: ToolExecutionCompletion,
    status: DesktopBackendTerminalStatus,
) -> DesktopEvidenceStatus {
    match (completion, status) {
        (_, DesktopBackendTerminalStatus::Interrupted) => DesktopEvidenceStatus::Interrupted,
        (ToolExecutionCompletion::Succeeded, _) => DesktopEvidenceStatus::Succeeded,
        (ToolExecutionCompletion::Failed, _) => DesktopEvidenceStatus::Failed,
        (ToolExecutionCompletion::UnknownOutcome, _) => DesktopEvidenceStatus::UnknownOutcome,
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
    Evidence(DesktopControlEvidenceError),
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
            Self::Evidence(error) => write!(f, "desktop durable evidence error: {error}"),
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

impl From<DesktopControlEvidenceError> for DesktopDispatchError {
    fn from(value: DesktopControlEvidenceError) -> Self {
        Self::Evidence(value)
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
