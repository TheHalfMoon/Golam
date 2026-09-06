#![forbid(unsafe_code)]

use core::fmt;

use golam_core::desktop_backend::{
    ClipboardBackendReceipt, ClipboardDispatchContext, DesktopBackend, DesktopBackendError,
    DesktopBackendTerminalStatus,
};
use golam_core::desktop_intent::{
    AuthorityBindings, ClipboardIntent, ClipboardOperation, RequestBinding,
};
use golam_core::digest::sha256;
use golam_core::tool_request::{BindingDigest, PreparedToolRequest};
use golam_core::{CanonicalEncoder, CoreError, EffectId, SessionId};

use crate::desktop_effect::effect_binding;
use crate::{
    AuthorizationPolicy, CompleteToolEffect, KernelApi, PrepareToolEffect, PreparedToolEffect,
    Principal, ToolEffectError, ToolExecutionCompletion,
};

const CLIPBOARD_PAYLOAD_DOMAIN: &[u8] = b"golam:desktop-clipboard-payload:v1";
const CLIPBOARD_RECEIPT_DOMAIN: &[u8] = b"golam:desktop-clipboard-receipt:v1";
const CLIPBOARD_HANDLER_ID: &str = "golam-desktop-clipboard";
const CLIPBOARD_HANDLER_VERSION: &str = "1";

pub struct PrepareDesktopClipboard<'a> {
    pub request: &'a PreparedToolRequest,
    pub effect_id: EffectId,
    pub session_id: SessionId,
    pub operation: ClipboardOperation,
    pub authority: AuthorityBindings,
    pub max_bytes: u32,
    pub content_digest: Option<BindingDigest>,
    pub permission_session_evidence_ref: BindingDigest,
    pub expires_at_unix_ms: u64,
    pub started_at: &'a str,
    pub idempotency_key: Option<&'a str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedDesktopClipboard {
    effect: PreparedToolEffect,
    intent: ClipboardIntent,
}

impl PreparedDesktopClipboard {
    pub fn effect(&self) -> &PreparedToolEffect {
        &self.effect
    }

    pub const fn intent(&self) -> &ClipboardIntent {
        &self.intent
    }
}

pub struct DispatchDesktopClipboard<'a, B: DesktopBackend> {
    pub prepared: &'a PreparedDesktopClipboard,
    pub current_request: &'a PreparedToolRequest,
    pub current_authority: AuthorityBindings,
    pub unresolved_conflicting_unknown_outcome: bool,
    pub now_unix_ms: u64,
    pub finished_at: &'a str,
    pub backend: &'a mut B,
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn prepare_desktop_clipboard(
        &mut self,
        principal: Principal<'_>,
        input: PrepareDesktopClipboard<'_>,
        scope: &str,
    ) -> Result<PreparedDesktopClipboard, DesktopClipboardError> {
        validate_prepare(&input)?;
        let request_digest = BindingDigest::new(input.request.binding_digest());
        let payload_hash = clipboard_payload_hash(&input, request_digest)?;
        let action = match input.operation {
            ClipboardOperation::Read => "desktop.clipboard.read",
            ClipboardOperation::Write => "desktop.clipboard.write",
        };
        let prepared = self.prepare_tool_effect(
            principal,
            PrepareToolEffect {
                effect_id: input.effect_id,
                session_id: input.session_id,
                action,
                resource: "desktop-clipboard",
                execution_semantics: "at_most_once",
                handler_id: CLIPBOARD_HANDLER_ID,
                handler_version: CLIPBOARD_HANDLER_VERSION,
                idempotency_key: input.idempotency_key,
                preconditions_hash: input.permission_session_evidence_ref.bytes(),
                payload_hash,
                started_at: input.started_at,
            },
            scope,
        )?;
        let intent = ClipboardIntent {
            schema_version: golam_core::desktop_control::DESKTOP_CONTROL_SCHEMA_VERSION,
            request: RequestBinding {
                request_id: input.request.request().request_id,
                canonical_request_digest: request_digest,
            },
            effect: effect_binding(&prepared)?,
            operation: input.operation,
            authority: input.authority,
            max_bytes: input.max_bytes,
            content_digest: input.content_digest,
            prepared_permission_session_evidence_ref: input.permission_session_evidence_ref,
            expires_at_unix_ms: input.expires_at_unix_ms,
        };
        intent.validate()?;
        Ok(PreparedDesktopClipboard {
            effect: prepared,
            intent,
        })
    }

    pub fn dispatch_desktop_clipboard<B: DesktopBackend>(
        &mut self,
        principal: Principal<'_>,
        input: DispatchDesktopClipboard<'_, B>,
        scope: &str,
    ) -> Result<ClipboardBackendReceipt, DesktopClipboardError> {
        if input.finished_at.is_empty() || input.now_unix_ms == 0 {
            return Err(DesktopClipboardError::InvalidDispatchInput);
        }
        let capabilities = input.backend.capabilities()?;
        let effect = effect_binding(input.prepared.effect())?;
        let context = ClipboardDispatchContext {
            intent: input.prepared.intent(),
            now_unix_ms: input.now_unix_ms,
            current_request_digest: BindingDigest::new(input.current_request.binding_digest()),
            current_effect_digest: effect.immutable_effect_digest,
            current_gate_authorization_digest: effect.gate_authorization_digest,
            current_authority: input.current_authority,
            current_capabilities: &capabilities,
            unresolved_conflicting_unknown_outcome: input.unresolved_conflicting_unknown_outcome,
        };
        let validated = match context.authorize() {
            Ok(validated) => validated,
            Err(error) => {
                self.complete_clipboard(
                    principal,
                    input.prepared,
                    input.finished_at,
                    ToolExecutionCompletion::Failed,
                    "desktop_clipboard_revalidation_failed_before_dispatch",
                    None,
                    scope,
                )?;
                return Err(DesktopClipboardError::Revalidation(error));
            }
        };
        let receipt = match input.backend.clipboard(validated) {
            Ok(receipt) => receipt,
            Err(error) => {
                self.complete_clipboard(
                    principal,
                    input.prepared,
                    input.finished_at,
                    ToolExecutionCompletion::UnknownOutcome,
                    "desktop_clipboard_adapter_error_after_dispatch_boundary",
                    None,
                    scope,
                )?;
                return Err(DesktopClipboardError::AdapterUncertain(error));
            }
        };
        let completion = classify_receipt(input.prepared, &receipt);
        let receipt_bytes = clipboard_receipt_bytes(&receipt)?;
        let reason = match completion {
            ToolExecutionCompletion::Succeeded => "desktop_clipboard_succeeded",
            ToolExecutionCompletion::Failed => "desktop_clipboard_terminal_failure",
            ToolExecutionCompletion::UnknownOutcome => "desktop_clipboard_unknown_outcome",
        };
        self.complete_clipboard(
            principal,
            input.prepared,
            input.finished_at,
            completion,
            reason,
            Some(receipt_bytes.as_slice()),
            scope,
        )?;
        if completion == ToolExecutionCompletion::UnknownOutcome {
            return Err(DesktopClipboardError::UnknownOutcome(receipt));
        }
        Ok(receipt)
    }

    fn complete_clipboard(
        &mut self,
        principal: Principal<'_>,
        prepared: &PreparedDesktopClipboard,
        finished_at: &str,
        completion: ToolExecutionCompletion,
        reason_code: &str,
        receipt: Option<&[u8]>,
        scope: &str,
    ) -> Result<(), DesktopClipboardError> {
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

fn validate_prepare(input: &PrepareDesktopClipboard<'_>) -> Result<(), DesktopClipboardError> {
    if input.effect_id.0 == 0
        || input.started_at.is_empty()
        || input.expires_at_unix_ms == 0
        || input.max_bytes == 0
        || input.max_bytes > 16 * 1024 * 1024
        || input.permission_session_evidence_ref.bytes() == [0; 32]
    {
        return Err(DesktopClipboardError::InvalidPrepareInput);
    }
    input.authority.validate()?;
    match (input.operation, input.content_digest) {
        (ClipboardOperation::Read, None) => Ok(()),
        (ClipboardOperation::Write, Some(digest)) if digest.bytes() != [0; 32] => Ok(()),
        _ => Err(DesktopClipboardError::InvalidPrepareInput),
    }
}

fn clipboard_payload_hash(
    input: &PrepareDesktopClipboard<'_>,
    request_digest: BindingDigest,
) -> Result<[u8; 32], DesktopClipboardError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(CLIPBOARD_PAYLOAD_DOMAIN)?;
    encoder.push_bytes(&request_digest.bytes())?;
    encoder.push_u8(match input.operation {
        ClipboardOperation::Read => 1,
        ClipboardOperation::Write => 2,
    });
    encoder.push_bytes(&input.authority.capability_ref.bytes())?;
    encoder.push_bytes(&input.authority.policy_ref.bytes())?;
    encoder.push_bytes(&input.authority.approval_ref.bytes())?;
    encoder.push_u64(u64::from(input.max_bytes));
    match input.content_digest {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(&value.bytes())?;
        }
        None => encoder.push_u8(0),
    }
    encoder.push_bytes(&input.permission_session_evidence_ref.bytes())?;
    encoder.push_u64(input.expires_at_unix_ms);
    Ok(sha256(&encoder.finish()))
}

fn classify_receipt(
    prepared: &PreparedDesktopClipboard,
    receipt: &ClipboardBackendReceipt,
) -> ToolExecutionCompletion {
    if receipt.payload_bytes > prepared.intent().max_bytes {
        return ToolExecutionCompletion::UnknownOutcome;
    }
    match prepared.intent().operation {
        ClipboardOperation::Read if receipt.content_digest.is_none() => {
            return ToolExecutionCompletion::UnknownOutcome;
        }
        ClipboardOperation::Write if receipt.content_digest != prepared.intent().content_digest => {
            return ToolExecutionCompletion::UnknownOutcome;
        }
        ClipboardOperation::Read | ClipboardOperation::Write => {}
    }
    match receipt.status {
        DesktopBackendTerminalStatus::Committed => ToolExecutionCompletion::Succeeded,
        DesktopBackendTerminalStatus::FailedBeforeEffect
        | DesktopBackendTerminalStatus::Interrupted
        | DesktopBackendTerminalStatus::NotSupported => ToolExecutionCompletion::Failed,
        DesktopBackendTerminalStatus::UnknownOutcome => ToolExecutionCompletion::UnknownOutcome,
    }
}

fn clipboard_receipt_bytes(
    receipt: &ClipboardBackendReceipt,
) -> Result<Vec<u8>, DesktopClipboardError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(CLIPBOARD_RECEIPT_DOMAIN)?;
    encoder.push_u8(status_code(receipt.status));
    match receipt.content_digest {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(&value.bytes())?;
        }
        None => encoder.push_u8(0),
    }
    encoder.push_u64(u64::from(receipt.payload_bytes));
    Ok(encoder.finish())
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
pub enum DesktopClipboardError {
    InvalidPrepareInput,
    InvalidDispatchInput,
    Revalidation(DesktopBackendError),
    AdapterUncertain(DesktopBackendError),
    UnknownOutcome(ClipboardBackendReceipt),
    Backend(DesktopBackendError),
    Kernel(crate::DesktopKernelError),
    Intent(golam_core::desktop_intent::DesktopIntentError),
    Effect(ToolEffectError),
    Core(CoreError),
}

impl fmt::Display for DesktopClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrepareInput => {
                f.write_str("invalid desktop clipboard preparation input")
            }
            Self::InvalidDispatchInput => f.write_str("invalid desktop clipboard dispatch input"),
            Self::Revalidation(error) => {
                write!(f, "desktop clipboard revalidation failed: {error}")
            }
            Self::AdapterUncertain(error) => {
                write!(f, "desktop clipboard adapter outcome uncertain: {error}")
            }
            Self::UnknownOutcome(_) => f.write_str("desktop clipboard outcome is unknown"),
            Self::Backend(error) => write!(f, "desktop clipboard backend error: {error}"),
            Self::Kernel(error) => write!(f, "desktop clipboard kernel error: {error}"),
            Self::Intent(error) => write!(f, "desktop clipboard intent error: {error}"),
            Self::Effect(error) => write!(f, "desktop clipboard effect error: {error}"),
            Self::Core(error) => write!(f, "desktop clipboard encoding error: {error}"),
        }
    }
}

impl std::error::Error for DesktopClipboardError {}

impl From<DesktopBackendError> for DesktopClipboardError {
    fn from(value: DesktopBackendError) -> Self {
        Self::Backend(value)
    }
}

impl From<crate::DesktopKernelError> for DesktopClipboardError {
    fn from(value: crate::DesktopKernelError) -> Self {
        Self::Kernel(value)
    }
}

impl From<golam_core::desktop_intent::DesktopIntentError> for DesktopClipboardError {
    fn from(value: golam_core::desktop_intent::DesktopIntentError) -> Self {
        Self::Intent(value)
    }
}

impl From<ToolEffectError> for DesktopClipboardError {
    fn from(value: ToolEffectError) -> Self {
        Self::Effect(value)
    }
}

impl From<CoreError> for DesktopClipboardError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}
