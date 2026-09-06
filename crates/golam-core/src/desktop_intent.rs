#![forbid(unsafe_code)]

use core::fmt;

use crate::desktop_control::{
    ControlRoute, DESKTOP_CONTROL_SCHEMA_VERSION, DesktopControlError, DesktopControlLeaseId,
    FallbackEligibilityEvidence, VisibleControlChannelId,
};
use crate::digest::sha256;
use crate::tool_request::{BindingDigest, ToolRequestId};
use crate::{CanonicalEncoder, CoreError, EffectId};

const ACTION_INTENT_DOMAIN: &[u8] = b"golam:desktop-action-intent:v1";
const CAPTURE_INTENT_DOMAIN: &[u8] = b"golam:desktop-capture-intent:v1";
const CLIPBOARD_INTENT_DOMAIN: &[u8] = b"golam:desktop-clipboard-intent:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestBinding {
    pub request_id: ToolRequestId,
    pub canonical_request_digest: BindingDigest,
}

impl RequestBinding {
    pub fn validate(&self) -> Result<(), DesktopIntentError> {
        if self.request_id.as_u128() == 0 {
            return Err(DesktopIntentError::MissingRequestBinding);
        }
        require_digest(self.canonical_request_digest, "canonical_request_digest")?;
        Ok(())
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) -> Result<(), DesktopIntentError> {
        self.validate()?;
        encoder.push_u128(self.request_id.as_u128());
        push_digest(encoder, self.canonical_request_digest)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectBinding {
    pub effect_id: EffectId,
    pub immutable_effect_digest: BindingDigest,
    pub gate_authorization_digest: BindingDigest,
}

impl EffectBinding {
    pub fn validate(&self) -> Result<(), DesktopIntentError> {
        if self.effect_id.0 == 0 {
            return Err(DesktopIntentError::MissingEffectBinding);
        }
        require_digest(self.immutable_effect_digest, "immutable_effect_digest")?;
        require_digest(self.gate_authorization_digest, "gate_authorization_digest")?;
        Ok(())
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) -> Result<(), DesktopIntentError> {
        self.validate()?;
        encoder.push_u128(self.effect_id.0);
        push_digest(encoder, self.immutable_effect_digest)?;
        push_digest(encoder, self.gate_authorization_digest)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorityBindings {
    pub capability_ref: BindingDigest,
    pub policy_ref: BindingDigest,
    pub approval_ref: BindingDigest,
}

impl AuthorityBindings {
    pub fn validate(&self) -> Result<(), DesktopIntentError> {
        require_digest(self.capability_ref, "capability_ref")?;
        require_digest(self.policy_ref, "policy_ref")?;
        require_digest(self.approval_ref, "approval_ref")?;
        Ok(())
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) -> Result<(), DesktopIntentError> {
        self.validate()?;
        push_digest(encoder, self.capability_ref)?;
        push_digest(encoder, self.policy_ref)?;
        push_digest(encoder, self.approval_ref)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InteractiveAuthorityBinding {
    pub lease_id: DesktopControlLeaseId,
    pub lease_generation: u64,
    pub visible_channel_id: VisibleControlChannelId,
    pub visible_channel_generation: u64,
    pub visible_channel_state_digest: BindingDigest,
}

impl InteractiveAuthorityBinding {
    pub fn validate(&self) -> Result<(), DesktopIntentError> {
        if self.lease_id.as_u128() == 0
            || self.lease_generation == 0
            || self.visible_channel_id.as_u128() == 0
            || self.visible_channel_generation == 0
        {
            return Err(DesktopIntentError::MissingInteractiveAuthority);
        }
        require_digest(
            self.visible_channel_state_digest,
            "visible_channel_state_digest",
        )?;
        Ok(())
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) -> Result<(), DesktopIntentError> {
        self.validate()?;
        encoder.push_u128(self.lease_id.as_u128());
        encoder.push_u64(self.lease_generation);
        encoder.push_u128(self.visible_channel_id.as_u128());
        encoder.push_u64(self.visible_channel_generation);
        push_digest(encoder, self.visible_channel_state_digest)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopActionKind {
    SemanticAction,
    RawInputFallback,
    Focus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedDesktopAction {
    pub schema_version: u16,
    pub request: RequestBinding,
    pub effect: EffectBinding,
    pub operation_kind: DesktopActionKind,
    pub exact_target_identity_digest: BindingDigest,
    pub pixel_hint_digest: Option<BindingDigest>,
    pub fallback_eligibility_evidence_digest: Option<BindingDigest>,
    pub action_payload_digest: BindingDigest,
    pub authority: AuthorityBindings,
    pub interactive_authority: InteractiveAuthorityBinding,
    pub prepared_permission_session_evidence_ref: BindingDigest,
    pub prepared_observation_digest: BindingDigest,
    pub expires_at_unix_ms: u64,
}

impl PreparedDesktopAction {
    pub fn validate(&self) -> Result<(), DesktopIntentError> {
        validate_schema(self.schema_version)?;
        self.request.validate()?;
        self.effect.validate()?;
        self.authority.validate()?;
        self.interactive_authority.validate()?;
        require_digest(
            self.exact_target_identity_digest,
            "exact_target_identity_digest",
        )?;
        require_digest(self.action_payload_digest, "action_payload_digest")?;
        require_digest(
            self.prepared_permission_session_evidence_ref,
            "prepared_permission_session_evidence_ref",
        )?;
        require_digest(
            self.prepared_observation_digest,
            "prepared_observation_digest",
        )?;
        if self.expires_at_unix_ms == 0 {
            return Err(DesktopIntentError::InvalidExpiry);
        }

        match self.operation_kind {
            DesktopActionKind::RawInputFallback => {
                let evidence = self
                    .fallback_eligibility_evidence_digest
                    .ok_or(DesktopIntentError::MissingFallbackEligibility)?;
                require_digest(evidence, "fallback_eligibility_evidence_digest")?;
                if let Some(hint) = self.pixel_hint_digest {
                    require_digest(hint, "pixel_hint_digest")?;
                }
            }
            DesktopActionKind::SemanticAction | DesktopActionKind::Focus => {
                if self.pixel_hint_digest.is_some()
                    || self.fallback_eligibility_evidence_digest.is_some()
                {
                    return Err(DesktopIntentError::UnexpectedFallbackBinding);
                }
            }
        }
        Ok(())
    }

    pub fn validate_fallback_evidence(
        &self,
        evidence: &FallbackEligibilityEvidence,
        now_unix_ms: u64,
    ) -> Result<(), DesktopIntentError> {
        self.validate()?;
        if self.operation_kind != DesktopActionKind::RawInputFallback {
            return Err(DesktopIntentError::UnexpectedFallbackBinding);
        }
        evidence.validate()?;
        if !evidence.permits_route(ControlRoute::DeterministicKeyboardMouse, now_unix_ms) {
            return Err(DesktopIntentError::FallbackNotEligible);
        }
        if Some(evidence.binding_digest()?) != self.fallback_eligibility_evidence_digest {
            return Err(DesktopIntentError::BindingMismatch(
                "fallback_eligibility_evidence",
            ));
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DesktopIntentError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(ACTION_INTENT_DOMAIN)?;
        encoder.push_u16(self.schema_version);
        self.request.encode(&mut encoder)?;
        self.effect.encode(&mut encoder)?;
        encoder.push_u8(action_kind_code(self.operation_kind));
        push_digest(&mut encoder, self.exact_target_identity_digest)?;
        push_optional_digest(&mut encoder, self.pixel_hint_digest)?;
        push_optional_digest(&mut encoder, self.fallback_eligibility_evidence_digest)?;
        push_digest(&mut encoder, self.action_payload_digest)?;
        self.authority.encode(&mut encoder)?;
        self.interactive_authority.encode(&mut encoder)?;
        push_digest(&mut encoder, self.prepared_permission_session_evidence_ref)?;
        push_digest(&mut encoder, self.prepared_observation_digest)?;
        encoder.push_u64(self.expires_at_unix_ms);
        Ok(encoder.finish())
    }

    pub fn intent_digest(&self) -> Result<BindingDigest, DesktopIntentError> {
        Ok(BindingDigest::new(sha256(&self.canonical_bytes()?)))
    }

    pub fn is_expired(&self, now_unix_ms: u64) -> bool {
        now_unix_ms >= self.expires_at_unix_ms
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureRetentionPolicy {
    EphemeralOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaptureLimits {
    pub max_width: u32,
    pub max_height: u32,
    pub max_frame_bytes: u32,
    pub max_duration_ms: u64,
}

impl CaptureLimits {
    pub fn validate(&self) -> Result<(), DesktopIntentError> {
        if self.max_width == 0
            || self.max_width > 16_384
            || self.max_height == 0
            || self.max_height > 16_384
            || self.max_frame_bytes == 0
            || self.max_frame_bytes > 256 * 1024 * 1024
            || self.max_duration_ms == 0
            || self.max_duration_ms > 60_000
        {
            return Err(DesktopIntentError::InvalidCaptureLimits);
        }
        Ok(())
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) {
        encoder.push_u64(u64::from(self.max_width));
        encoder.push_u64(u64::from(self.max_height));
        encoder.push_u64(u64::from(self.max_frame_bytes));
        encoder.push_u64(self.max_duration_ms);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaptureIntent {
    pub schema_version: u16,
    pub request: RequestBinding,
    pub effect: EffectBinding,
    pub selected_source_identity_digest: BindingDigest,
    pub authority: AuthorityBindings,
    pub limits: CaptureLimits,
    pub include_cursor: bool,
    pub audio_enabled: bool,
    pub prepared_permission_session_evidence_ref: BindingDigest,
    pub retention_policy: CaptureRetentionPolicy,
    pub expires_at_unix_ms: u64,
}

impl CaptureIntent {
    pub fn validate(&self) -> Result<(), DesktopIntentError> {
        validate_schema(self.schema_version)?;
        self.request.validate()?;
        self.effect.validate()?;
        self.authority.validate()?;
        self.limits.validate()?;
        require_digest(
            self.selected_source_identity_digest,
            "selected_source_identity_digest",
        )?;
        require_digest(
            self.prepared_permission_session_evidence_ref,
            "prepared_permission_session_evidence_ref",
        )?;
        if self.audio_enabled {
            return Err(DesktopIntentError::AudioDenied);
        }
        if self.expires_at_unix_ms == 0 {
            return Err(DesktopIntentError::InvalidExpiry);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DesktopIntentError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(CAPTURE_INTENT_DOMAIN)?;
        encoder.push_u16(self.schema_version);
        self.request.encode(&mut encoder)?;
        self.effect.encode(&mut encoder)?;
        push_digest(&mut encoder, self.selected_source_identity_digest)?;
        self.authority.encode(&mut encoder)?;
        self.limits.encode(&mut encoder);
        encoder.push_u8(u8::from(self.include_cursor));
        encoder.push_u8(u8::from(self.audio_enabled));
        push_digest(&mut encoder, self.prepared_permission_session_evidence_ref)?;
        encoder.push_u8(1);
        encoder.push_u64(self.expires_at_unix_ms);
        Ok(encoder.finish())
    }

    pub fn intent_digest(&self) -> Result<BindingDigest, DesktopIntentError> {
        Ok(BindingDigest::new(sha256(&self.canonical_bytes()?)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClipboardOperation {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClipboardIntent {
    pub schema_version: u16,
    pub request: RequestBinding,
    pub effect: EffectBinding,
    pub operation: ClipboardOperation,
    pub authority: AuthorityBindings,
    pub max_bytes: u32,
    pub content_digest: Option<BindingDigest>,
    pub prepared_permission_session_evidence_ref: BindingDigest,
    pub expires_at_unix_ms: u64,
}

impl ClipboardIntent {
    pub fn validate(&self) -> Result<(), DesktopIntentError> {
        validate_schema(self.schema_version)?;
        self.request.validate()?;
        self.effect.validate()?;
        self.authority.validate()?;
        require_digest(
            self.prepared_permission_session_evidence_ref,
            "prepared_permission_session_evidence_ref",
        )?;
        if self.max_bytes == 0 || self.max_bytes > 16 * 1024 * 1024 {
            return Err(DesktopIntentError::InvalidClipboardLimit);
        }
        if self.expires_at_unix_ms == 0 {
            return Err(DesktopIntentError::InvalidExpiry);
        }
        match (self.operation, self.content_digest) {
            (ClipboardOperation::Read, None) => {}
            (ClipboardOperation::Write, Some(value)) => {
                require_digest(value, "content_digest")?;
            }
            _ => return Err(DesktopIntentError::InvalidClipboardContentBinding),
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DesktopIntentError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(CLIPBOARD_INTENT_DOMAIN)?;
        encoder.push_u16(self.schema_version);
        self.request.encode(&mut encoder)?;
        self.effect.encode(&mut encoder)?;
        encoder.push_u8(clipboard_operation_code(self.operation));
        self.authority.encode(&mut encoder)?;
        encoder.push_u64(u64::from(self.max_bytes));
        push_optional_digest(&mut encoder, self.content_digest)?;
        push_digest(&mut encoder, self.prepared_permission_session_evidence_ref)?;
        encoder.push_u64(self.expires_at_unix_ms);
        Ok(encoder.finish())
    }

    pub fn intent_digest(&self) -> Result<BindingDigest, DesktopIntentError> {
        Ok(BindingDigest::new(sha256(&self.canonical_bytes()?)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopIntentError {
    InvalidSchemaVersion,
    MissingRequestBinding,
    MissingEffectBinding,
    MissingInteractiveAuthority,
    MissingBinding(&'static str),
    BindingMismatch(&'static str),
    MissingFallbackEligibility,
    UnexpectedFallbackBinding,
    FallbackNotEligible,
    InvalidExpiry,
    InvalidCaptureLimits,
    AudioDenied,
    InvalidClipboardLimit,
    InvalidClipboardContentBinding,
    DesktopControl(DesktopControlError),
    CanonicalEncoding(CoreError),
}

impl fmt::Display for DesktopIntentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSchemaVersion => f.write_str("invalid desktop intent schema version"),
            Self::MissingRequestBinding => f.write_str("missing immutable ToolRequest binding"),
            Self::MissingEffectBinding => f.write_str("missing immutable Effect binding"),
            Self::MissingInteractiveAuthority => {
                f.write_str("missing control-lease or visible-channel authority binding")
            }
            Self::MissingBinding(field) => write!(f, "missing desktop intent binding: {field}"),
            Self::BindingMismatch(field) => write!(f, "desktop intent binding mismatch: {field}"),
            Self::MissingFallbackEligibility => {
                f.write_str("raw fallback requires eligibility evidence")
            }
            Self::UnexpectedFallbackBinding => {
                f.write_str("non-fallback action cannot carry fallback bindings")
            }
            Self::FallbackNotEligible => {
                f.write_str("fallback evidence does not authorize deterministic input")
            }
            Self::InvalidExpiry => f.write_str("desktop intent expiry is invalid"),
            Self::InvalidCaptureLimits => f.write_str("capture limits are invalid or unbounded"),
            Self::AudioDenied => f.write_str("capture audio is denied in Spec 006"),
            Self::InvalidClipboardLimit => {
                f.write_str("clipboard byte limit is invalid or unbounded")
            }
            Self::InvalidClipboardContentBinding => {
                f.write_str("clipboard content binding does not match operation")
            }
            Self::DesktopControl(error) => write!(f, "desktop control validation error: {error}"),
            Self::CanonicalEncoding(error) => write!(f, "canonical encoding error: {error}"),
        }
    }
}

impl std::error::Error for DesktopIntentError {}

impl From<DesktopControlError> for DesktopIntentError {
    fn from(value: DesktopControlError) -> Self {
        Self::DesktopControl(value)
    }
}

impl From<CoreError> for DesktopIntentError {
    fn from(value: CoreError) -> Self {
        Self::CanonicalEncoding(value)
    }
}

fn validate_schema(schema_version: u16) -> Result<(), DesktopIntentError> {
    if schema_version != DESKTOP_CONTROL_SCHEMA_VERSION {
        return Err(DesktopIntentError::InvalidSchemaVersion);
    }
    Ok(())
}

fn require_digest(digest: BindingDigest, field: &'static str) -> Result<(), DesktopIntentError> {
    if digest.bytes().iter().all(|byte| *byte == 0) {
        return Err(DesktopIntentError::MissingBinding(field));
    }
    Ok(())
}

fn push_digest(
    encoder: &mut CanonicalEncoder,
    digest: BindingDigest,
) -> Result<(), DesktopIntentError> {
    encoder.push_bytes(&digest.bytes())?;
    Ok(())
}

fn push_optional_digest(
    encoder: &mut CanonicalEncoder,
    digest: Option<BindingDigest>,
) -> Result<(), DesktopIntentError> {
    match digest {
        Some(value) => {
            encoder.push_u8(1);
            push_digest(encoder, value)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

const fn action_kind_code(kind: DesktopActionKind) -> u8 {
    match kind {
        DesktopActionKind::SemanticAction => 1,
        DesktopActionKind::RawInputFallback => 2,
        DesktopActionKind::Focus => 3,
    }
}

const fn clipboard_operation_code(operation: ClipboardOperation) -> u8 {
    match operation {
        ClipboardOperation::Read => 1,
        ClipboardOperation::Write => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop_control::{
        ControlRoute, DESKTOP_CONTROL_SCHEMA_VERSION, RouteDisposition, RouteEvaluation,
    };

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn request() -> RequestBinding {
        RequestBinding {
            request_id: ToolRequestId::from_u128(1),
            canonical_request_digest: digest(1),
        }
    }

    fn effect() -> EffectBinding {
        EffectBinding {
            effect_id: EffectId(2),
            immutable_effect_digest: digest(2),
            gate_authorization_digest: digest(3),
        }
    }

    fn authority() -> AuthorityBindings {
        AuthorityBindings {
            capability_ref: digest(4),
            policy_ref: digest(5),
            approval_ref: digest(6),
        }
    }

    fn interactive() -> InteractiveAuthorityBinding {
        InteractiveAuthorityBinding {
            lease_id: DesktopControlLeaseId::from_u128(7),
            lease_generation: 2,
            visible_channel_id: VisibleControlChannelId::from_u128(8),
            visible_channel_generation: 3,
            visible_channel_state_digest: digest(9),
        }
    }

    #[test]
    fn missing_gate_authorization_fails_closed() {
        let mut effect = effect();
        effect.gate_authorization_digest = BindingDigest::new([0; 32]);
        assert!(matches!(
            effect.validate(),
            Err(DesktopIntentError::MissingBinding(
                "gate_authorization_digest"
            ))
        ));
    }

    #[test]
    fn raw_fallback_requires_exact_fresh_evidence() {
        let evidence = FallbackEligibilityEvidence {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            target_task_scope_digest: digest(10),
            route_evaluations: vec![
                RouteEvaluation {
                    route: ControlRoute::AccessibilitySemanticTree,
                    disposition: RouteDisposition::Unavailable,
                    evidence_ref: digest(11),
                },
                RouteEvaluation {
                    route: ControlRoute::DeterministicKeyboardMouse,
                    disposition: RouteDisposition::Selected,
                    evidence_ref: digest(12),
                },
            ],
            highest_eligible_route: ControlRoute::DeterministicKeyboardMouse,
            created_at_unix_ms: 10,
            expires_at_unix_ms: 20,
        };
        let action = PreparedDesktopAction {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            request: request(),
            effect: effect(),
            operation_kind: DesktopActionKind::RawInputFallback,
            exact_target_identity_digest: digest(13),
            pixel_hint_digest: None,
            fallback_eligibility_evidence_digest: Some(evidence.binding_digest().unwrap()),
            action_payload_digest: digest(14),
            authority: authority(),
            interactive_authority: interactive(),
            prepared_permission_session_evidence_ref: digest(15),
            prepared_observation_digest: digest(16),
            expires_at_unix_ms: 30,
        };
        assert!(action.validate_fallback_evidence(&evidence, 15).is_ok());
        assert_eq!(
            action.validate_fallback_evidence(&evidence, 20),
            Err(DesktopIntentError::FallbackNotEligible)
        );
    }

    #[test]
    fn semantic_action_rejects_pixel_hint_or_fallback_binding() {
        let action = PreparedDesktopAction {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            request: request(),
            effect: effect(),
            operation_kind: DesktopActionKind::SemanticAction,
            exact_target_identity_digest: digest(10),
            pixel_hint_digest: Some(digest(11)),
            fallback_eligibility_evidence_digest: None,
            action_payload_digest: digest(12),
            authority: authority(),
            interactive_authority: interactive(),
            prepared_permission_session_evidence_ref: digest(13),
            prepared_observation_digest: digest(14),
            expires_at_unix_ms: 20,
        };
        assert_eq!(
            action.validate(),
            Err(DesktopIntentError::UnexpectedFallbackBinding)
        );
    }

    #[test]
    fn capture_audio_is_denied() {
        let intent = CaptureIntent {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            request: request(),
            effect: effect(),
            selected_source_identity_digest: digest(10),
            authority: authority(),
            limits: CaptureLimits {
                max_width: 1920,
                max_height: 1080,
                max_frame_bytes: 8 * 1024 * 1024,
                max_duration_ms: 1_000,
            },
            include_cursor: true,
            audio_enabled: true,
            prepared_permission_session_evidence_ref: digest(11),
            retention_policy: CaptureRetentionPolicy::EphemeralOnly,
            expires_at_unix_ms: 20,
        };
        assert_eq!(intent.validate(), Err(DesktopIntentError::AudioDenied));
    }

    #[test]
    fn clipboard_read_cannot_bind_write_content() {
        let intent = ClipboardIntent {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            request: request(),
            effect: effect(),
            operation: ClipboardOperation::Read,
            authority: authority(),
            max_bytes: 4096,
            content_digest: Some(digest(10)),
            prepared_permission_session_evidence_ref: digest(11),
            expires_at_unix_ms: 20,
        };
        assert_eq!(
            intent.validate(),
            Err(DesktopIntentError::InvalidClipboardContentBinding)
        );
    }
}
