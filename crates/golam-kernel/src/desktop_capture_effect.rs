#![forbid(unsafe_code)]

use core::fmt;

use golam_core::desktop_backend::{
    CaptureBackendReceipt, CaptureDispatchContext, DesktopBackend, DesktopBackendError,
    DesktopBackendTerminalStatus,
};
use golam_core::desktop_intent::{
    AuthorityBindings, CaptureIntent, CaptureLimits, CaptureRetentionPolicy, RequestBinding,
};
use golam_core::digest::sha256;
use golam_core::tool_request::{BindingDigest, PreparedToolRequest};
use golam_core::{CanonicalEncoder, CoreError, EffectId, SessionId};
use golam_ledger::desktop_control_evidence::{
    DesktopControlEvidenceError, DesktopControlEvidenceStore, DesktopEffectEvidence,
    DesktopEvidenceOperation, DesktopEvidenceStatus,
};

use crate::desktop_effect::effect_binding;
use crate::{
    AuthorizationPolicy, CompleteToolEffect, KernelApi, PrepareToolEffect, PreparedToolEffect,
    Principal, ToolEffectError, ToolExecutionCompletion,
};

const CAPTURE_PAYLOAD_DOMAIN: &[u8] = b"golam:desktop-capture-payload:v1";
const CAPTURE_RECEIPT_DOMAIN: &[u8] = b"golam:desktop-capture-receipt:v1";
const CAPTURE_HANDLER_ID: &str = "golam-desktop-capture";
const CAPTURE_HANDLER_VERSION: &str = "1";

pub struct PrepareDesktopCapture<'a> {
    pub request: &'a PreparedToolRequest,
    pub effect_id: EffectId,
    pub session_id: SessionId,
    pub selected_source_identity_digest: BindingDigest,
    pub authority: AuthorityBindings,
    pub limits: CaptureLimits,
    pub include_cursor: bool,
    pub permission_session_evidence_ref: BindingDigest,
    pub expires_at_unix_ms: u64,
    pub started_at: &'a str,
    pub idempotency_key: Option<&'a str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedDesktopCapture {
    effect: PreparedToolEffect,
    intent: CaptureIntent,
}

impl PreparedDesktopCapture {
    pub fn effect(&self) -> &PreparedToolEffect {
        &self.effect
    }

    pub const fn intent(&self) -> &CaptureIntent {
        &self.intent
    }
}

pub struct DispatchDesktopCapture<'a, B: DesktopBackend> {
    pub prepared: &'a PreparedDesktopCapture,
    pub current_request: &'a PreparedToolRequest,
    pub current_authority: AuthorityBindings,
    pub current_source_identity_digest: BindingDigest,
    /// Caller input can only conservatively add a block. Durable protected
    /// evidence is authoritative and cannot be cleared by passing `false`.
    pub unresolved_conflicting_unknown_outcome: bool,
    pub now_unix_ms: u64,
    pub finished_at: &'a str,
    pub backend: &'a mut B,
}

struct CompleteDesktopCapture<'a> {
    prepared: &'a PreparedDesktopCapture,
    finished_at: &'a str,
    completion: ToolExecutionCompletion,
    evidence_status: DesktopEvidenceStatus,
    reason_code: &'a str,
    receipt: Option<&'a [u8]>,
    recorded_at_unix_ms: u64,
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn prepare_desktop_capture(
        &mut self,
        principal: Principal<'_>,
        input: PrepareDesktopCapture<'_>,
        scope: &str,
    ) -> Result<PreparedDesktopCapture, DesktopCaptureError> {
        validate_prepare(&input)?;
        let request_digest = BindingDigest::new(input.request.binding_digest());
        let payload_hash = capture_payload_hash(&input, request_digest)?;
        let resource = capture_resource(input.selected_source_identity_digest);
        let prepared = self.prepare_tool_effect(
            principal,
            PrepareToolEffect {
                effect_id: input.effect_id,
                session_id: input.session_id,
                action: "desktop.capture",
                resource: &resource,
                execution_semantics: "at_most_once",
                handler_id: CAPTURE_HANDLER_ID,
                handler_version: CAPTURE_HANDLER_VERSION,
                idempotency_key: input.idempotency_key,
                preconditions_hash: input.permission_session_evidence_ref.bytes(),
                payload_hash,
                started_at: input.started_at,
            },
            scope,
        )?;
        let intent = CaptureIntent {
            schema_version: golam_core::desktop_control::DESKTOP_CONTROL_SCHEMA_VERSION,
            request: RequestBinding {
                request_id: input.request.request().request_id,
                canonical_request_digest: request_digest,
            },
            effect: effect_binding(&prepared)?,
            selected_source_identity_digest: input.selected_source_identity_digest,
            authority: input.authority,
            limits: input.limits,
            include_cursor: input.include_cursor,
            audio_enabled: false,
            prepared_permission_session_evidence_ref: input.permission_session_evidence_ref,
            retention_policy: CaptureRetentionPolicy::EphemeralOnly,
            expires_at_unix_ms: input.expires_at_unix_ms,
        };
        intent.validate()?;
        Ok(PreparedDesktopCapture {
            effect: prepared,
            intent,
        })
    }

    pub fn dispatch_desktop_capture<B: DesktopBackend>(
        &mut self,
        principal: Principal<'_>,
        input: DispatchDesktopCapture<'_, B>,
        scope: &str,
    ) -> Result<CaptureBackendReceipt, DesktopCaptureError> {
        if input.finished_at.is_empty() || input.now_unix_ms == 0 {
            return Err(DesktopCaptureError::InvalidDispatchInput);
        }
        let mut evidence_store = DesktopControlEvidenceStore::open(&self.authority)?;
        let durable_unknown = evidence_store
            .has_unresolved_unknown_outcome_for_effect(input.prepared.effect().effect_id())?;
        let capabilities = input.backend.capabilities()?;
        let effect = effect_binding(input.prepared.effect())?;
        evidence_store.append_effect_evidence(capture_evidence(
            &evidence_store,
            input.prepared,
            DesktopEvidenceStatus::Prepared,
            input.now_unix_ms,
        )?)?;
        let context = CaptureDispatchContext {
            intent: input.prepared.intent(),
            now_unix_ms: input.now_unix_ms,
            current_request_digest: BindingDigest::new(input.current_request.binding_digest()),
            current_effect_digest: effect.immutable_effect_digest,
            current_gate_authorization_digest: effect.gate_authorization_digest,
            current_authority: input.current_authority,
            current_capabilities: &capabilities,
            current_source_identity_digest: input.current_source_identity_digest,
            unresolved_conflicting_unknown_outcome: durable_unknown
                || input.unresolved_conflicting_unknown_outcome,
        };
        let validated = match context.authorize() {
            Ok(validated) => validated,
            Err(error) => {
                self.complete_capture(
                    &mut evidence_store,
                    principal,
                    CompleteDesktopCapture {
                        prepared: input.prepared,
                        finished_at: input.finished_at,
                        completion: ToolExecutionCompletion::Failed,
                        evidence_status: DesktopEvidenceStatus::Failed,
                        reason_code: "desktop_capture_revalidation_failed_before_dispatch",
                        receipt: None,
                        recorded_at_unix_ms: input.now_unix_ms,
                    },
                    scope,
                )?;
                return Err(DesktopCaptureError::Revalidation(error));
            }
        };
        let receipt = match input.backend.capture(validated) {
            Ok(receipt) => receipt,
            Err(error) => {
                self.complete_capture(
                    &mut evidence_store,
                    principal,
                    CompleteDesktopCapture {
                        prepared: input.prepared,
                        finished_at: input.finished_at,
                        completion: ToolExecutionCompletion::UnknownOutcome,
                        evidence_status: DesktopEvidenceStatus::UnknownOutcome,
                        reason_code: "desktop_capture_adapter_error_after_dispatch_boundary",
                        receipt: None,
                        recorded_at_unix_ms: input.now_unix_ms,
                    },
                    scope,
                )?;
                return Err(DesktopCaptureError::AdapterUncertain(error));
            }
        };
        let completion = classify_receipt(input.prepared, &receipt);
        let receipt_bytes = capture_receipt_bytes(&receipt)?;
        let evidence_status = evidence_status_for_receipt(completion, receipt.status);
        let reason_code = match completion {
            ToolExecutionCompletion::Succeeded => "desktop_capture_succeeded",
            ToolExecutionCompletion::Failed => "desktop_capture_terminal_failure",
            ToolExecutionCompletion::UnknownOutcome => "desktop_capture_unknown_outcome",
        };
        self.complete_capture(
            &mut evidence_store,
            principal,
            CompleteDesktopCapture {
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
            return Err(DesktopCaptureError::UnknownOutcome(receipt));
        }
        Ok(receipt)
    }

    fn complete_capture(
        &mut self,
        evidence_store: &mut DesktopControlEvidenceStore,
        principal: Principal<'_>,
        input: CompleteDesktopCapture<'_>,
        scope: &str,
    ) -> Result<(), DesktopCaptureError> {
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
        evidence_store.append_effect_evidence(capture_evidence(
            evidence_store,
            input.prepared,
            input.evidence_status,
            input.recorded_at_unix_ms,
        )?)?;
        Ok(())
    }
}

fn capture_evidence(
    store: &DesktopControlEvidenceStore,
    prepared: &PreparedDesktopCapture,
    status: DesktopEvidenceStatus,
    recorded_at_unix_ms: u64,
) -> Result<DesktopEffectEvidence, DesktopCaptureError> {
    let intent = prepared.intent();
    Ok(DesktopEffectEvidence {
        effect_id: prepared.effect().effect_id(),
        session_id: store.effect_session_id(prepared.effect().effect_id())?,
        operation: DesktopEvidenceOperation::Capture,
        request_digest: intent.request.canonical_request_digest,
        effect_digest: intent.effect.immutable_effect_digest,
        intent_digest: intent.intent_digest()?,
        fallback_eligibility_digest: None,
        control_lease_digest: None,
        visible_channel_digest: None,
        permission_session_digest: intent.prepared_permission_session_evidence_ref,
        target_or_source_digest: intent.selected_source_identity_digest,
        status,
        reconciliation_ref: None,
        recorded_at_unix_ms,
    })
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

fn validate_prepare(input: &PrepareDesktopCapture<'_>) -> Result<(), DesktopCaptureError> {
    input.limits.validate()?;
    if input.effect_id.0 == 0
        || input.session_id.0 == 0
        || input.started_at.is_empty()
        || input.expires_at_unix_ms == 0
        || input.selected_source_identity_digest.bytes() == [0; 32]
        || input.permission_session_evidence_ref.bytes() == [0; 32]
    {
        return Err(DesktopCaptureError::InvalidPrepareInput);
    }
    input.authority.validate()?;
    Ok(())
}

fn capture_payload_hash(
    input: &PrepareDesktopCapture<'_>,
    request_digest: BindingDigest,
) -> Result<[u8; 32], DesktopCaptureError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(CAPTURE_PAYLOAD_DOMAIN)?;
    encoder.push_bytes(&request_digest.bytes())?;
    encoder.push_bytes(&input.selected_source_identity_digest.bytes())?;
    encoder.push_bytes(&input.authority.capability_ref.bytes())?;
    encoder.push_bytes(&input.authority.policy_ref.bytes())?;
    encoder.push_bytes(&input.authority.approval_ref.bytes())?;
    encoder.push_u64(u64::from(input.limits.max_width));
    encoder.push_u64(u64::from(input.limits.max_height));
    encoder.push_u64(u64::from(input.limits.max_frame_bytes));
    encoder.push_u64(input.limits.max_duration_ms);
    encoder.push_u8(u8::from(input.include_cursor));
    encoder.push_u8(0);
    encoder.push_bytes(&input.permission_session_evidence_ref.bytes())?;
    encoder.push_u64(input.expires_at_unix_ms);
    Ok(sha256(&encoder.finish()))
}

fn classify_receipt(
    prepared: &PreparedDesktopCapture,
    receipt: &CaptureBackendReceipt,
) -> ToolExecutionCompletion {
    if receipt.source_identity_digest != prepared.intent().selected_source_identity_digest
        || receipt.payload_bytes > prepared.intent().limits.max_frame_bytes
        || receipt.payload_digest.bytes() == [0; 32]
    {
        return ToolExecutionCompletion::UnknownOutcome;
    }
    match receipt.status {
        DesktopBackendTerminalStatus::Committed => ToolExecutionCompletion::Succeeded,
        DesktopBackendTerminalStatus::FailedBeforeEffect
        | DesktopBackendTerminalStatus::Interrupted
        | DesktopBackendTerminalStatus::NotSupported => ToolExecutionCompletion::Failed,
        DesktopBackendTerminalStatus::UnknownOutcome => ToolExecutionCompletion::UnknownOutcome,
    }
}

fn capture_receipt_bytes(receipt: &CaptureBackendReceipt) -> Result<Vec<u8>, DesktopCaptureError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(CAPTURE_RECEIPT_DOMAIN)?;
    encoder.push_u8(status_code(receipt.status));
    encoder.push_bytes(&receipt.source_identity_digest.bytes())?;
    encoder.push_bytes(&receipt.payload_digest.bytes())?;
    encoder.push_u64(u64::from(receipt.payload_bytes));
    Ok(encoder.finish())
}

fn capture_resource(digest: BindingDigest) -> String {
    let bytes = digest.bytes();
    let mut output = String::from("desktop-capture-source:");
    for byte in &bytes[..8] {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
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
pub enum DesktopCaptureError {
    InvalidPrepareInput,
    InvalidDispatchInput,
    Revalidation(DesktopBackendError),
    AdapterUncertain(DesktopBackendError),
    UnknownOutcome(CaptureBackendReceipt),
    Backend(DesktopBackendError),
    Evidence(DesktopControlEvidenceError),
    Kernel(crate::DesktopKernelError),
    Intent(golam_core::desktop_intent::DesktopIntentError),
    Effect(ToolEffectError),
    Core(CoreError),
}

impl fmt::Display for DesktopCaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrepareInput => f.write_str("invalid desktop capture preparation input"),
            Self::InvalidDispatchInput => f.write_str("invalid desktop capture dispatch input"),
            Self::Revalidation(error) => write!(f, "desktop capture revalidation failed: {error}"),
            Self::AdapterUncertain(error) => {
                write!(f, "desktop capture adapter outcome uncertain: {error}")
            }
            Self::UnknownOutcome(_) => f.write_str("desktop capture outcome is unknown"),
            Self::Backend(error) => write!(f, "desktop capture backend error: {error}"),
            Self::Evidence(error) => write!(f, "desktop capture durable evidence error: {error}"),
            Self::Kernel(error) => write!(f, "desktop capture kernel error: {error}"),
            Self::Intent(error) => write!(f, "desktop capture intent error: {error}"),
            Self::Effect(error) => write!(f, "desktop capture effect error: {error}"),
            Self::Core(error) => write!(f, "desktop capture encoding error: {error}"),
        }
    }
}

impl std::error::Error for DesktopCaptureError {}

impl From<DesktopBackendError> for DesktopCaptureError {
    fn from(value: DesktopBackendError) -> Self {
        Self::Backend(value)
    }
}

impl From<DesktopControlEvidenceError> for DesktopCaptureError {
    fn from(value: DesktopControlEvidenceError) -> Self {
        Self::Evidence(value)
    }
}

impl From<crate::DesktopKernelError> for DesktopCaptureError {
    fn from(value: crate::DesktopKernelError) -> Self {
        Self::Kernel(value)
    }
}

impl From<golam_core::desktop_intent::DesktopIntentError> for DesktopCaptureError {
    fn from(value: golam_core::desktop_intent::DesktopIntentError) -> Self {
        Self::Intent(value)
    }
}

impl From<ToolEffectError> for DesktopCaptureError {
    fn from(value: ToolEffectError) -> Self {
        Self::Effect(value)
    }
}

impl From<CoreError> for DesktopCaptureError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}
