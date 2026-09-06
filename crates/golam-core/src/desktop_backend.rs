#![forbid(unsafe_code)]

use core::fmt;

use crate::desktop_control::{
    DesktopCapabilitySet, DesktopControlError, DesktopControlLeaseState, DesktopControlMode,
    DesktopLimits, DesktopObservation, FallbackEligibilityEvidence, PixelTargetHint,
    VisibleControlChannelState,
};
use crate::desktop_intent::{
    AuthorityBindings, CaptureIntent, ClipboardIntent, ClipboardOperation, DesktopActionKind,
    DesktopIntentError, PreparedDesktopAction,
};
use crate::digest::sha256;
use crate::tool_request::BindingDigest;

const FAKE_CAPTURE_DOMAIN: &[u8] = b"golam:desktop-fake-capture:v1";
const FAKE_CLIPBOARD_DOMAIN: &[u8] = b"golam:desktop-fake-clipboard:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopActionDispatchContext<'a> {
    pub action: &'a PreparedDesktopAction,
    pub now_unix_ms: u64,
    pub current_request_digest: BindingDigest,
    pub current_effect_digest: BindingDigest,
    pub current_gate_authorization_digest: BindingDigest,
    pub current_authority: AuthorityBindings,
    pub current_capabilities: &'a DesktopCapabilitySet,
    pub current_observation: &'a DesktopObservation,
    pub current_target_identity_digest: BindingDigest,
    pub current_lease: &'a DesktopControlLeaseState,
    pub current_visible_channel: &'a VisibleControlChannelState,
    pub fallback_evidence: Option<&'a FallbackEligibilityEvidence>,
    pub pixel_hint: Option<&'a PixelTargetHint>,
    pub unresolved_conflicting_unknown_outcome: bool,
}

impl<'a> DesktopActionDispatchContext<'a> {
    pub fn authorize(self) -> Result<ValidatedDesktopAction<'a>, DesktopBackendError> {
        self.action.validate()?;
        if self.action.is_expired(self.now_unix_ms) {
            return Err(DesktopBackendError::ExpiredIntent);
        }
        validate_request_effect_gate(
            self.action.request.canonical_request_digest,
            self.action.effect.immutable_effect_digest,
            self.action.effect.gate_authorization_digest,
            self.current_request_digest,
            self.current_effect_digest,
            self.current_gate_authorization_digest,
        )?;
        if self.action.authority != self.current_authority {
            return Err(DesktopBackendError::AuthorityBindingMismatch);
        }
        self.current_capabilities.validate()?;
        self.current_observation.validate()?;
        if self.current_capabilities.permission_session_evidence
            != self.action.prepared_permission_session_evidence_ref
            || self.current_observation.capability_session_evidence
                != self.action.prepared_permission_session_evidence_ref
        {
            return Err(DesktopBackendError::PermissionOrSessionDrift);
        }
        if self.current_observation.binding_digest()? != self.action.prepared_observation_digest {
            return Err(DesktopBackendError::ObservationDrift);
        }
        if self.current_target_identity_digest != self.action.exact_target_identity_digest {
            return Err(DesktopBackendError::StaleOrSubstitutedTarget);
        }
        validate_interactive_authority(
            self.action,
            self.current_lease,
            self.current_visible_channel,
            self.now_unix_ms,
        )?;
        if self.unresolved_conflicting_unknown_outcome {
            return Err(DesktopBackendError::UnknownOutcomeBlocksDispatch);
        }

        match self.action.operation_kind {
            DesktopActionKind::RawInputFallback => {
                let evidence = self
                    .fallback_evidence
                    .ok_or(DesktopBackendError::MissingFallbackEligibility)?;
                self.action
                    .validate_fallback_evidence(evidence, self.now_unix_ms)?;
                validate_pixel_hint_binding(self.action, self.pixel_hint, self.now_unix_ms)?;
            }
            DesktopActionKind::SemanticAction | DesktopActionKind::Focus => {
                if self.fallback_evidence.is_some() || self.pixel_hint.is_some() {
                    return Err(DesktopBackendError::UnexpectedFallbackEvidence);
                }
            }
        }

        Ok(ValidatedDesktopAction {
            action: self.action,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaptureDispatchContext<'a> {
    pub intent: &'a CaptureIntent,
    pub now_unix_ms: u64,
    pub current_request_digest: BindingDigest,
    pub current_effect_digest: BindingDigest,
    pub current_gate_authorization_digest: BindingDigest,
    pub current_authority: AuthorityBindings,
    pub current_capabilities: &'a DesktopCapabilitySet,
    pub current_source_identity_digest: BindingDigest,
    pub unresolved_conflicting_unknown_outcome: bool,
}

impl<'a> CaptureDispatchContext<'a> {
    pub fn authorize(self) -> Result<ValidatedCaptureIntent<'a>, DesktopBackendError> {
        self.intent.validate()?;
        if self.now_unix_ms >= self.intent.expires_at_unix_ms {
            return Err(DesktopBackendError::ExpiredIntent);
        }
        validate_request_effect_gate(
            self.intent.request.canonical_request_digest,
            self.intent.effect.immutable_effect_digest,
            self.intent.effect.gate_authorization_digest,
            self.current_request_digest,
            self.current_effect_digest,
            self.current_gate_authorization_digest,
        )?;
        if self.intent.authority != self.current_authority {
            return Err(DesktopBackendError::AuthorityBindingMismatch);
        }
        self.current_capabilities.validate()?;
        if self.current_capabilities.permission_session_evidence
            != self.intent.prepared_permission_session_evidence_ref
        {
            return Err(DesktopBackendError::PermissionOrSessionDrift);
        }
        if self.current_source_identity_digest != self.intent.selected_source_identity_digest {
            return Err(DesktopBackendError::StaleOrSubstitutedTarget);
        }
        if self.unresolved_conflicting_unknown_outcome {
            return Err(DesktopBackendError::UnknownOutcomeBlocksDispatch);
        }
        Ok(ValidatedCaptureIntent {
            intent: self.intent,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClipboardDispatchContext<'a> {
    pub intent: &'a ClipboardIntent,
    pub now_unix_ms: u64,
    pub current_request_digest: BindingDigest,
    pub current_effect_digest: BindingDigest,
    pub current_gate_authorization_digest: BindingDigest,
    pub current_authority: AuthorityBindings,
    pub current_capabilities: &'a DesktopCapabilitySet,
    pub unresolved_conflicting_unknown_outcome: bool,
}

impl<'a> ClipboardDispatchContext<'a> {
    pub fn authorize(self) -> Result<ValidatedClipboardIntent<'a>, DesktopBackendError> {
        self.intent.validate()?;
        if self.now_unix_ms >= self.intent.expires_at_unix_ms {
            return Err(DesktopBackendError::ExpiredIntent);
        }
        validate_request_effect_gate(
            self.intent.request.canonical_request_digest,
            self.intent.effect.immutable_effect_digest,
            self.intent.effect.gate_authorization_digest,
            self.current_request_digest,
            self.current_effect_digest,
            self.current_gate_authorization_digest,
        )?;
        if self.intent.authority != self.current_authority {
            return Err(DesktopBackendError::AuthorityBindingMismatch);
        }
        self.current_capabilities.validate()?;
        if self.current_capabilities.permission_session_evidence
            != self.intent.prepared_permission_session_evidence_ref
        {
            return Err(DesktopBackendError::PermissionOrSessionDrift);
        }
        match self.intent.operation {
            ClipboardOperation::Read if !self.current_capabilities.clipboard_read_supported => {
                return Err(DesktopBackendError::ClipboardDenied);
            }
            ClipboardOperation::Write if !self.current_capabilities.clipboard_write_supported => {
                return Err(DesktopBackendError::ClipboardDenied);
            }
            ClipboardOperation::Read | ClipboardOperation::Write => {}
        }
        if self.unresolved_conflicting_unknown_outcome {
            return Err(DesktopBackendError::UnknownOutcomeBlocksDispatch);
        }
        Ok(ValidatedClipboardIntent {
            intent: self.intent,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedDesktopAction<'a> {
    action: &'a PreparedDesktopAction,
}

impl<'a> ValidatedDesktopAction<'a> {
    pub fn action(&self) -> &'a PreparedDesktopAction {
        self.action
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedCaptureIntent<'a> {
    intent: &'a CaptureIntent,
}

impl<'a> ValidatedCaptureIntent<'a> {
    pub fn intent(&self) -> &'a CaptureIntent {
        self.intent
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedClipboardIntent<'a> {
    intent: &'a ClipboardIntent,
}

impl<'a> ValidatedClipboardIntent<'a> {
    pub fn intent(&self) -> &'a ClipboardIntent {
        self.intent
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopBackendTerminalStatus {
    Committed,
    FailedBeforeEffect,
    UnknownOutcome,
    Interrupted,
    NotSupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopActionReceipt {
    pub status: DesktopBackendTerminalStatus,
    pub observed_target_digest: BindingDigest,
    pub post_observation_digest: Option<BindingDigest>,
    pub sanitized_error_class: Option<BindingDigest>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaptureBackendReceipt {
    pub status: DesktopBackendTerminalStatus,
    pub source_identity_digest: BindingDigest,
    pub payload_digest: BindingDigest,
    pub payload_bytes: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClipboardBackendReceipt {
    pub status: DesktopBackendTerminalStatus,
    pub content_digest: Option<BindingDigest>,
    pub payload_bytes: u32,
}

/// Platform adapters are intentionally untrusted executors. They receive only
/// dispatch values that have already passed exact core binding validation.
/// Route selection, fallback eligibility, authority, control-lease state and
/// visible-control-channel safety remain above this interface.
pub trait DesktopBackend {
    fn capabilities(&mut self) -> Result<DesktopCapabilitySet, DesktopBackendError>;

    fn observe(
        &mut self,
        limits: DesktopLimits,
    ) -> Result<DesktopObservation, DesktopBackendError>;

    fn dispatch_action(
        &mut self,
        action: ValidatedDesktopAction<'_>,
    ) -> Result<DesktopActionReceipt, DesktopBackendError>;

    fn capture(
        &mut self,
        intent: ValidatedCaptureIntent<'_>,
    ) -> Result<CaptureBackendReceipt, DesktopBackendError>;

    fn clipboard(
        &mut self,
        intent: ValidatedClipboardIntent<'_>,
    ) -> Result<ClipboardBackendReceipt, DesktopBackendError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FakeDesktopBackend {
    capabilities: DesktopCapabilitySet,
    observation: DesktopObservation,
    permission_granted: bool,
    next_terminal_status: Option<DesktopBackendTerminalStatus>,
    dispatch_count: u64,
}

impl FakeDesktopBackend {
    pub fn new(
        capabilities: DesktopCapabilitySet,
        observation: DesktopObservation,
    ) -> Result<Self, DesktopBackendError> {
        capabilities.validate()?;
        observation.validate()?;
        if observation.capability_session_evidence != capabilities.permission_session_evidence {
            return Err(DesktopBackendError::PermissionOrSessionDrift);
        }
        Ok(Self {
            capabilities,
            observation,
            permission_granted: true,
            next_terminal_status: None,
            dispatch_count: 0,
        })
    }

    pub fn set_permission_granted(&mut self, granted: bool) {
        self.permission_granted = granted;
    }

    pub fn set_observation(
        &mut self,
        observation: DesktopObservation,
    ) -> Result<(), DesktopBackendError> {
        observation.validate()?;
        if observation.capability_session_evidence != self.capabilities.permission_session_evidence
        {
            return Err(DesktopBackendError::PermissionOrSessionDrift);
        }
        self.observation = observation;
        Ok(())
    }

    pub fn script_next_terminal_status(&mut self, status: DesktopBackendTerminalStatus) {
        self.next_terminal_status = Some(status);
    }

    pub const fn dispatch_count(&self) -> u64 {
        self.dispatch_count
    }

    fn terminal_status(&mut self) -> DesktopBackendTerminalStatus {
        self.next_terminal_status
            .take()
            .unwrap_or(DesktopBackendTerminalStatus::Committed)
    }

    fn require_permission(&self) -> Result<(), DesktopBackendError> {
        if !self.permission_granted {
            return Err(DesktopBackendError::PermissionOrSessionDrift);
        }
        Ok(())
    }
}

impl DesktopBackend for FakeDesktopBackend {
    fn capabilities(&mut self) -> Result<DesktopCapabilitySet, DesktopBackendError> {
        self.capabilities.validate()?;
        Ok(self.capabilities)
    }

    fn observe(
        &mut self,
        limits: DesktopLimits,
    ) -> Result<DesktopObservation, DesktopBackendError> {
        self.require_permission()?;
        limits.validate()?;
        self.observation.validate()?;
        if self.observation.work_surface_digests.len() > usize::from(limits.max_work_surfaces) {
            return Err(DesktopBackendError::ObservationLimitExceeded);
        }
        Ok(self.observation.clone())
    }

    fn dispatch_action(
        &mut self,
        action: ValidatedDesktopAction<'_>,
    ) -> Result<DesktopActionReceipt, DesktopBackendError> {
        self.require_permission()?;
        action.action().validate()?;
        self.dispatch_count = self.dispatch_count.saturating_add(1);
        let status = self.terminal_status();
        let post_observation_digest = if status == DesktopBackendTerminalStatus::Committed {
            Some(self.observation.binding_digest()?)
        } else {
            None
        };
        Ok(DesktopActionReceipt {
            status,
            observed_target_digest: action.action().exact_target_identity_digest,
            post_observation_digest,
            sanitized_error_class: None,
        })
    }

    fn capture(
        &mut self,
        intent: ValidatedCaptureIntent<'_>,
    ) -> Result<CaptureBackendReceipt, DesktopBackendError> {
        self.require_permission()?;
        intent.intent().validate()?;
        self.dispatch_count = self.dispatch_count.saturating_add(1);
        let status = self.terminal_status();
        let canonical = intent.intent().canonical_bytes()?;
        let payload_digest = BindingDigest::new(sha256(
            &[FAKE_CAPTURE_DOMAIN, canonical.as_slice()].concat(),
        ));
        Ok(CaptureBackendReceipt {
            status,
            source_identity_digest: intent.intent().selected_source_identity_digest,
            payload_digest,
            payload_bytes: intent.intent().limits.max_frame_bytes.min(4_096),
        })
    }

    fn clipboard(
        &mut self,
        intent: ValidatedClipboardIntent<'_>,
    ) -> Result<ClipboardBackendReceipt, DesktopBackendError> {
        self.require_permission()?;
        intent.intent().validate()?;
        self.dispatch_count = self.dispatch_count.saturating_add(1);
        let status = self.terminal_status();
        let content_digest = match intent.intent().operation {
            ClipboardOperation::Read => {
                let canonical = intent.intent().canonical_bytes()?;
                Some(BindingDigest::new(sha256(
                    &[FAKE_CLIPBOARD_DOMAIN, canonical.as_slice()].concat(),
                )))
            }
            ClipboardOperation::Write => intent.intent().content_digest,
        };
        Ok(ClipboardBackendReceipt {
            status,
            content_digest,
            payload_bytes: intent.intent().max_bytes.min(256),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopBackendError {
    InvalidRequestBinding,
    InvalidEffectBinding,
    MissingGateAuthorization,
    GateAuthorizationMismatch,
    AuthorityBindingMismatch,
    PermissionOrSessionDrift,
    ObservationDrift,
    StaleOrSubstitutedTarget,
    StaleOrSupersededLease,
    AutonomousActuationSuspended,
    MissingFallbackEligibility,
    UnexpectedFallbackEvidence,
    StalePixelHint,
    PixelHintBindingMismatch,
    UnknownOutcomeBlocksDispatch,
    ExpiredIntent,
    ObservationLimitExceeded,
    ClipboardDenied,
    DesktopControl(DesktopControlError),
    DesktopIntent(DesktopIntentError),
}

impl fmt::Display for DesktopBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequestBinding => {
                f.write_str("desktop request binding is stale or substituted")
            }
            Self::InvalidEffectBinding => {
                f.write_str("desktop effect binding is stale or substituted")
            }
            Self::MissingGateAuthorization => {
                f.write_str("desktop Kernel/Effect Gate authorization is missing")
            }
            Self::GateAuthorizationMismatch => {
                f.write_str("desktop Kernel/Effect Gate authorization is stale or substituted")
            }
            Self::AuthorityBindingMismatch => {
                f.write_str("desktop capability/policy/approval bindings are stale or substituted")
            }
            Self::PermissionOrSessionDrift => {
                f.write_str("desktop permission or session evidence changed after prepare")
            }
            Self::ObservationDrift => f.write_str("desktop observation changed after prepare"),
            Self::StaleOrSubstitutedTarget => {
                f.write_str("desktop target identity is stale or substituted")
            }
            Self::StaleOrSupersededLease => {
                f.write_str("desktop control lease is stale, superseded, paused, or revoked")
            }
            Self::AutonomousActuationSuspended => {
                f.write_str("no qualified visible control channel permits autonomous actuation")
            }
            Self::MissingFallbackEligibility => {
                f.write_str("raw fallback requires canonical fallback eligibility")
            }
            Self::UnexpectedFallbackEvidence => {
                f.write_str("semantic or focus dispatch cannot carry fallback evidence")
            }
            Self::StalePixelHint => f.write_str("pixel hint is stale or invalid"),
            Self::PixelHintBindingMismatch => {
                f.write_str("pixel hint binding is stale or substituted")
            }
            Self::UnknownOutcomeBlocksDispatch => {
                f.write_str("unreconciled UNKNOWN_OUTCOME blocks conflicting desktop dispatch")
            }
            Self::ExpiredIntent => f.write_str("desktop prepared intent expired before dispatch"),
            Self::ObservationLimitExceeded => {
                f.write_str("desktop observation exceeds requested bounds")
            }
            Self::ClipboardDenied => {
                f.write_str("clipboard operation is not supported by current capabilities")
            }
            Self::DesktopControl(error) => write!(f, "desktop control error: {error}"),
            Self::DesktopIntent(error) => write!(f, "desktop intent error: {error}"),
        }
    }
}

impl std::error::Error for DesktopBackendError {}

impl From<DesktopControlError> for DesktopBackendError {
    fn from(value: DesktopControlError) -> Self {
        Self::DesktopControl(value)
    }
}

impl From<DesktopIntentError> for DesktopBackendError {
    fn from(value: DesktopIntentError) -> Self {
        Self::DesktopIntent(value)
    }
}

fn validate_request_effect_gate(
    prepared_request_digest: BindingDigest,
    prepared_effect_digest: BindingDigest,
    prepared_gate_digest: BindingDigest,
    current_request_digest: BindingDigest,
    current_effect_digest: BindingDigest,
    current_gate_digest: BindingDigest,
) -> Result<(), DesktopBackendError> {
    if prepared_request_digest != current_request_digest {
        return Err(DesktopBackendError::InvalidRequestBinding);
    }
    if prepared_effect_digest != current_effect_digest {
        return Err(DesktopBackendError::InvalidEffectBinding);
    }
    if current_gate_digest.bytes() == [0; 32] {
        return Err(DesktopBackendError::MissingGateAuthorization);
    }
    if prepared_gate_digest != current_gate_digest {
        return Err(DesktopBackendError::GateAuthorizationMismatch);
    }
    Ok(())
}

fn validate_interactive_authority(
    action: &PreparedDesktopAction,
    current_lease: &DesktopControlLeaseState,
    current_visible_channel: &VisibleControlChannelState,
    now_unix_ms: u64,
) -> Result<(), DesktopBackendError> {
    current_lease.validate()?;
    if current_lease.lease_id != action.interactive_authority.lease_id
        || current_lease.generation != action.interactive_authority.lease_generation
        || current_lease.mode != DesktopControlMode::AgentAllowed
        || !current_lease.allows_agent_input(now_unix_ms)
    {
        return Err(DesktopBackendError::StaleOrSupersededLease);
    }

    current_visible_channel.validate()?;
    if current_visible_channel.channel_id != action.interactive_authority.visible_channel_id
        || current_visible_channel.generation
            != action.interactive_authority.visible_channel_generation
        || current_visible_channel.binding_digest()?
            != action.interactive_authority.visible_channel_state_digest
        || !current_visible_channel.qualifies_for_autonomous_actuation(now_unix_ms)
    {
        return Err(DesktopBackendError::AutonomousActuationSuspended);
    }
    Ok(())
}

fn validate_pixel_hint_binding(
    action: &PreparedDesktopAction,
    pixel_hint: Option<&PixelTargetHint>,
    now_unix_ms: u64,
) -> Result<(), DesktopBackendError> {
    match (action.pixel_hint_digest, pixel_hint) {
        (None, None) => Ok(()),
        (Some(expected), Some(actual)) => {
            actual.validate()?;
            if now_unix_ms >= actual.expires_at_unix_ms {
                return Err(DesktopBackendError::StalePixelHint);
            }
            if actual.binding_digest()? != expected {
                return Err(DesktopBackendError::PixelHintBindingMismatch);
            }
            Ok(())
        }
        (Some(_), None) => Err(DesktopBackendError::StalePixelHint),
        (None, Some(_)) => Err(DesktopBackendError::UnexpectedFallbackEvidence),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EffectId;
    use crate::desktop_control::{
        ControlRoute, DESKTOP_CONTROL_SCHEMA_VERSION, DesktopControlLeaseId, DesktopObservationId,
        DesktopPlatform, DesktopSessionKind, RouteDisposition, RouteEvaluation,
        VisibleControlChannelId, VisibleControlChannelKind,
    };
    use crate::desktop_intent::{
        CaptureLimits, CaptureRetentionPolicy, EffectBinding, InteractiveAuthorityBinding,
        RequestBinding,
    };
    use crate::tool_request::ToolRequestId;

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn capabilities() -> DesktopCapabilitySet {
        DesktopCapabilitySet {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            platform: DesktopPlatform::Linux,
            session_kind: DesktopSessionKind::LinuxWayland,
            observation_kinds: 1,
            semantic_action_kinds: 1,
            capture_source_kinds: 1,
            raw_fallback_supported: true,
            pixel_hint_supported: true,
            clipboard_read_supported: true,
            clipboard_write_supported: true,
            human_interrupt_supported: true,
            visible_control_supported: true,
            permission_session_evidence: digest(20),
        }
    }

    fn observation() -> DesktopObservation {
        DesktopObservation {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            observation_id: DesktopObservationId::from_u128(1),
            observed_at_unix_ms: 100,
            capability_session_evidence: digest(20),
            work_surface_digests: vec![digest(30)],
            semantic_summary_digest: digest(31),
            focused_surface_digest: Some(digest(30)),
            focused_element_digest: None,
            limits: DesktopLimits::default(),
        }
    }

    fn authority() -> AuthorityBindings {
        AuthorityBindings {
            capability_ref: digest(4),
            policy_ref: digest(5),
            approval_ref: digest(6),
        }
    }

    fn lease() -> DesktopControlLeaseState {
        DesktopControlLeaseState {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            lease_id: DesktopControlLeaseId::from_u128(7),
            generation: 3,
            controlling_principal_ref: digest(7),
            mode: DesktopControlMode::AgentAllowed,
            issued_at_unix_ms: 90,
            updated_at_unix_ms: 95,
            expires_at_unix_ms: 1_000,
            capability_ref: digest(4),
            policy_ref: digest(5),
            interrupt_cause_ref: None,
        }
    }

    fn channel() -> VisibleControlChannelState {
        VisibleControlChannelState {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            channel_id: VisibleControlChannelId::from_u128(9),
            generation: 2,
            kind: VisibleControlChannelKind::SystemTray,
            trusted_host_ref: digest(8),
            visible: true,
            live: true,
            supports_pause: true,
            supports_stop: true,
            supports_takeover: true,
            observed_at_unix_ms: 95,
            heartbeat_deadline_unix_ms: 1_000,
        }
    }

    fn fallback() -> FallbackEligibilityEvidence {
        FallbackEligibilityEvidence {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            target_task_scope_digest: digest(40),
            route_evaluations: vec![
                RouteEvaluation {
                    route: ControlRoute::DomainApplicationApi,
                    disposition: RouteDisposition::Inapplicable,
                    evidence_ref: digest(41),
                },
                RouteEvaluation {
                    route: ControlRoute::NativeOsAutomationApi,
                    disposition: RouteDisposition::Unavailable,
                    evidence_ref: digest(42),
                },
                RouteEvaluation {
                    route: ControlRoute::AccessibilitySemanticTree,
                    disposition: RouteDisposition::PermissionDenied,
                    evidence_ref: digest(43),
                },
                RouteEvaluation {
                    route: ControlRoute::BrowserDomProtocol,
                    disposition: RouteDisposition::NotSupported,
                    evidence_ref: digest(44),
                },
                RouteEvaluation {
                    route: ControlRoute::DeterministicKeyboardMouse,
                    disposition: RouteDisposition::Selected,
                    evidence_ref: digest(45),
                },
            ],
            highest_eligible_route: ControlRoute::DeterministicKeyboardMouse,
            created_at_unix_ms: 90,
            expires_at_unix_ms: 500,
        }
    }

    fn action(
        kind: DesktopActionKind,
        fallback_ref: Option<BindingDigest>,
    ) -> PreparedDesktopAction {
        PreparedDesktopAction {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            request: RequestBinding {
                request_id: ToolRequestId::from_u128(1),
                canonical_request_digest: digest(1),
            },
            effect: EffectBinding {
                effect_id: EffectId(2),
                immutable_effect_digest: digest(2),
                gate_authorization_digest: digest(3),
            },
            operation_kind: kind,
            exact_target_identity_digest: digest(30),
            pixel_hint_digest: None,
            fallback_eligibility_evidence_digest: fallback_ref,
            action_payload_digest: digest(9),
            authority: authority(),
            interactive_authority: InteractiveAuthorityBinding {
                lease_id: DesktopControlLeaseId::from_u128(7),
                lease_generation: 3,
                visible_channel_id: VisibleControlChannelId::from_u128(9),
                visible_channel_generation: 2,
                visible_channel_state_digest: channel().binding_digest().unwrap(),
            },
            prepared_permission_session_evidence_ref: digest(20),
            prepared_observation_digest: observation().binding_digest().unwrap(),
            expires_at_unix_ms: 500,
        }
    }

    fn action_context<'a>(
        action: &'a PreparedDesktopAction,
        caps: &'a DesktopCapabilitySet,
        observation: &'a DesktopObservation,
        lease: &'a DesktopControlLeaseState,
        channel: &'a VisibleControlChannelState,
        fallback: Option<&'a FallbackEligibilityEvidence>,
    ) -> DesktopActionDispatchContext<'a> {
        DesktopActionDispatchContext {
            action,
            now_unix_ms: 110,
            current_request_digest: digest(1),
            current_effect_digest: digest(2),
            current_gate_authorization_digest: digest(3),
            current_authority: authority(),
            current_capabilities: caps,
            current_observation: observation,
            current_target_identity_digest: digest(30),
            current_lease: lease,
            current_visible_channel: channel,
            fallback_evidence: fallback,
            pixel_hint: None,
            unresolved_conflicting_unknown_outcome: false,
        }
    }

    #[test]
    fn semantic_action_dispatch_requires_exact_gate_and_interactive_state() {
        let caps = capabilities();
        let observed = observation();
        let current_lease = lease();
        let current_channel = channel();
        let prepared = action(DesktopActionKind::SemanticAction, None);
        let validated = action_context(
            &prepared,
            &caps,
            &observed,
            &current_lease,
            &current_channel,
            None,
        )
        .authorize()
        .unwrap();
        let mut backend = FakeDesktopBackend::new(caps, observed).unwrap();
        let receipt = backend.dispatch_action(validated).unwrap();
        assert_eq!(receipt.status, DesktopBackendTerminalStatus::Committed);
        assert_eq!(backend.dispatch_count(), 1);
    }

    #[test]
    fn missing_or_substituted_gate_fails_before_backend_dispatch() {
        let caps = capabilities();
        let observed = observation();
        let current_lease = lease();
        let current_channel = channel();
        let prepared = action(DesktopActionKind::Focus, None);
        let mut context = action_context(
            &prepared,
            &caps,
            &observed,
            &current_lease,
            &current_channel,
            None,
        );
        context.current_gate_authorization_digest = BindingDigest::new([0; 32]);
        assert_eq!(
            context.authorize().unwrap_err(),
            DesktopBackendError::MissingGateAuthorization
        );
    }

    #[test]
    fn permission_observation_target_and_authority_drift_fail_closed() {
        let mut caps = capabilities();
        let observed = observation();
        let current_lease = lease();
        let current_channel = channel();
        let prepared = action(DesktopActionKind::SemanticAction, None);
        caps.permission_session_evidence = digest(99);
        assert_eq!(
            action_context(
                &prepared,
                &caps,
                &observed,
                &current_lease,
                &current_channel,
                None,
            )
            .authorize()
            .unwrap_err(),
            DesktopBackendError::PermissionOrSessionDrift
        );
    }

    #[test]
    fn stale_lease_and_visible_channel_loss_suspend_actuation() {
        let caps = capabilities();
        let observed = observation();
        let mut current_lease = lease();
        let mut current_channel = channel();
        let prepared = action(DesktopActionKind::SemanticAction, None);
        current_lease.generation = 4;
        assert_eq!(
            action_context(
                &prepared,
                &caps,
                &observed,
                &current_lease,
                &current_channel,
                None,
            )
            .authorize()
            .unwrap_err(),
            DesktopBackendError::StaleOrSupersededLease
        );
        current_lease = lease();
        current_channel.live = false;
        assert_eq!(
            action_context(
                &prepared,
                &caps,
                &observed,
                &current_lease,
                &current_channel,
                None,
            )
            .authorize()
            .unwrap_err(),
            DesktopBackendError::AutonomousActuationSuspended
        );
    }

    #[test]
    fn raw_fallback_requires_fresh_canonical_eligibility_and_no_unknown_outcome() {
        let caps = capabilities();
        let observed = observation();
        let current_lease = lease();
        let current_channel = channel();
        let evidence = fallback();
        let prepared = action(
            DesktopActionKind::RawInputFallback,
            Some(evidence.binding_digest().unwrap()),
        );
        assert!(
            action_context(
                &prepared,
                &caps,
                &observed,
                &current_lease,
                &current_channel,
                Some(&evidence),
            )
            .authorize()
            .is_ok()
        );
        let mut context = action_context(
            &prepared,
            &caps,
            &observed,
            &current_lease,
            &current_channel,
            Some(&evidence),
        );
        context.unresolved_conflicting_unknown_outcome = true;
        assert_eq!(
            context.authorize().unwrap_err(),
            DesktopBackendError::UnknownOutcomeBlocksDispatch
        );
    }

    #[test]
    fn stronger_route_unknown_outcome_cannot_mint_weaker_eligibility() {
        let mut evidence = fallback();
        evidence.route_evaluations[1].disposition = RouteDisposition::UnknownOutcome;
        assert!(evidence.validate().is_err());
    }

    #[test]
    fn observation_bounds_are_enforced_by_fake_backend() {
        let caps = capabilities();
        let observed = observation();
        let mut backend = FakeDesktopBackend::new(caps, observed).unwrap();
        let mut limits = DesktopLimits::default();
        limits.max_work_surfaces = 1;
        assert!(backend.observe(limits).is_ok());
        limits.max_work_surfaces = 0;
        assert!(backend.observe(limits).is_err());
    }

    #[test]
    fn capture_and_clipboard_require_exact_bindings_and_current_support() {
        let caps = capabilities();
        let capture = CaptureIntent {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            request: RequestBinding {
                request_id: ToolRequestId::from_u128(10),
                canonical_request_digest: digest(10),
            },
            effect: EffectBinding {
                effect_id: EffectId(11),
                immutable_effect_digest: digest(11),
                gate_authorization_digest: digest(12),
            },
            selected_source_identity_digest: digest(30),
            authority: authority(),
            limits: CaptureLimits {
                max_width: 1_920,
                max_height: 1_080,
                max_frame_bytes: 8_192,
                max_duration_ms: 1_000,
            },
            include_cursor: false,
            audio_enabled: false,
            prepared_permission_session_evidence_ref: digest(20),
            retention_policy: CaptureRetentionPolicy::EphemeralOnly,
            expires_at_unix_ms: 500,
        };
        let validated_capture = CaptureDispatchContext {
            intent: &capture,
            now_unix_ms: 110,
            current_request_digest: digest(10),
            current_effect_digest: digest(11),
            current_gate_authorization_digest: digest(12),
            current_authority: authority(),
            current_capabilities: &caps,
            current_source_identity_digest: digest(30),
            unresolved_conflicting_unknown_outcome: false,
        }
        .authorize()
        .unwrap();
        let mut backend = FakeDesktopBackend::new(caps, observation()).unwrap();
        assert_eq!(
            backend.capture(validated_capture).unwrap().status,
            DesktopBackendTerminalStatus::Committed
        );

        let clipboard = ClipboardIntent {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            request: RequestBinding {
                request_id: ToolRequestId::from_u128(20),
                canonical_request_digest: digest(21),
            },
            effect: EffectBinding {
                effect_id: EffectId(21),
                immutable_effect_digest: digest(22),
                gate_authorization_digest: digest(23),
            },
            operation: ClipboardOperation::Read,
            authority: authority(),
            max_bytes: 128,
            content_digest: None,
            prepared_permission_session_evidence_ref: digest(20),
            expires_at_unix_ms: 500,
        };
        let validated_clipboard = ClipboardDispatchContext {
            intent: &clipboard,
            now_unix_ms: 110,
            current_request_digest: digest(21),
            current_effect_digest: digest(22),
            current_gate_authorization_digest: digest(23),
            current_authority: authority(),
            current_capabilities: &caps,
            unresolved_conflicting_unknown_outcome: false,
        }
        .authorize()
        .unwrap();
        assert_eq!(
            backend.clipboard(validated_clipboard).unwrap().status,
            DesktopBackendTerminalStatus::Committed
        );
    }
}
