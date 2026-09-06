#![forbid(unsafe_code)]

use core::fmt;

use crate::digest::sha256;
use crate::tool_request::BindingDigest;
use crate::{CanonicalEncoder, CoreError};

pub const DESKTOP_CONTROL_SCHEMA_VERSION: u16 = 1;
const CAPABILITY_DOMAIN: &[u8] = b"golam:desktop-capability:v1";
const WORK_SURFACE_DOMAIN: &[u8] = b"golam:desktop-work-surface:v1";
const SEMANTIC_ELEMENT_DOMAIN: &[u8] = b"golam:desktop-semantic-element:v1";
const OBSERVATION_DOMAIN: &[u8] = b"golam:desktop-observation:v1";
const LEASE_DOMAIN: &[u8] = b"golam:desktop-control-lease:v1";
const VISIBLE_CHANNEL_DOMAIN: &[u8] = b"golam:desktop-visible-channel:v1";
const FALLBACK_DOMAIN: &[u8] = b"golam:desktop-fallback-eligibility:v1";
const PIXEL_HINT_DOMAIN: &[u8] = b"golam:desktop-pixel-hint:v1";
const HUMAN_INTERRUPT_DOMAIN: &[u8] = b"golam:desktop-human-interrupt:v1";
const MAX_OBSERVED_SURFACES: usize = 64;
const MAX_ROUTE_EVALUATIONS: usize = 6;
const MAX_INTERRUPT_REFS: usize = 128;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DesktopPlatform {
    Windows,
    Macos,
    Linux,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DesktopSessionKind {
    WindowsInteractive,
    MacosLogin,
    LinuxX11,
    LinuxWayland,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorkSurfaceId(u128);

impl WorkSurfaceId {
    pub const fn from_u128(value: u128) -> Self {
        Self(value)
    }

    pub const fn as_u128(self) -> u128 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticElementId(u128);

impl SemanticElementId {
    pub const fn from_u128(value: u128) -> Self {
        Self(value)
    }

    pub const fn as_u128(self) -> u128 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DesktopObservationId(u128);

impl DesktopObservationId {
    pub const fn from_u128(value: u128) -> Self {
        Self(value)
    }

    pub const fn as_u128(self) -> u128 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DesktopControlLeaseId(u128);

impl DesktopControlLeaseId {
    pub const fn from_u128(value: u128) -> Self {
        Self(value)
    }

    pub const fn as_u128(self) -> u128 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VisibleControlChannelId(u128);

impl VisibleControlChannelId {
    pub const fn from_u128(value: u128) -> Self {
        Self(value)
    }

    pub const fn as_u128(self) -> u128 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopLimits {
    pub max_work_surfaces: u16,
    pub max_semantic_nodes: u32,
    pub max_string_bytes: u32,
    pub max_capture_width: u32,
    pub max_capture_height: u32,
    pub max_capture_bytes: u32,
    pub max_action_duration_ms: u64,
    pub max_pixel_hint_age_ms: u64,
}

impl Default for DesktopLimits {
    fn default() -> Self {
        Self {
            max_work_surfaces: 32,
            max_semantic_nodes: 2_048,
            max_string_bytes: 16 * 1024,
            max_capture_width: 7_680,
            max_capture_height: 4_320,
            max_capture_bytes: 64 * 1024 * 1024,
            max_action_duration_ms: 30_000,
            max_pixel_hint_age_ms: 5_000,
        }
    }
}

impl DesktopLimits {
    pub fn validate(&self) -> Result<(), DesktopControlError> {
        if self.max_work_surfaces == 0
            || self.max_work_surfaces as usize > MAX_OBSERVED_SURFACES
            || self.max_semantic_nodes == 0
            || self.max_semantic_nodes > 65_536
            || self.max_string_bytes == 0
            || self.max_string_bytes > 1024 * 1024
            || self.max_capture_width == 0
            || self.max_capture_width > 16_384
            || self.max_capture_height == 0
            || self.max_capture_height > 16_384
            || self.max_capture_bytes == 0
            || self.max_capture_bytes > 256 * 1024 * 1024
            || self.max_action_duration_ms == 0
            || self.max_action_duration_ms > 300_000
            || self.max_pixel_hint_age_ms == 0
            || self.max_pixel_hint_age_ms > 60_000
        {
            return Err(DesktopControlError::InvalidLimits);
        }
        Ok(())
    }

    fn encode(&self, encoder: &mut CanonicalEncoder) {
        encoder.push_u16(self.max_work_surfaces);
        encoder.push_u64(u64::from(self.max_semantic_nodes));
        encoder.push_u64(u64::from(self.max_string_bytes));
        encoder.push_u64(u64::from(self.max_capture_width));
        encoder.push_u64(u64::from(self.max_capture_height));
        encoder.push_u64(u64::from(self.max_capture_bytes));
        encoder.push_u64(self.max_action_duration_ms);
        encoder.push_u64(self.max_pixel_hint_age_ms);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopCapabilitySet {
    pub schema_version: u16,
    pub platform: DesktopPlatform,
    pub session_kind: DesktopSessionKind,
    pub observation_kinds: u64,
    pub semantic_action_kinds: u64,
    pub capture_source_kinds: u64,
    pub raw_fallback_supported: bool,
    pub pixel_hint_supported: bool,
    pub clipboard_read_supported: bool,
    pub clipboard_write_supported: bool,
    pub human_interrupt_supported: bool,
    pub visible_control_supported: bool,
    pub permission_session_evidence: BindingDigest,
}

impl DesktopCapabilitySet {
    pub fn validate(&self) -> Result<(), DesktopControlError> {
        if self.schema_version != DESKTOP_CONTROL_SCHEMA_VERSION {
            return Err(DesktopControlError::InvalidSchemaVersion);
        }
        if digest_is_zero(self.permission_session_evidence) {
            return Err(DesktopControlError::MissingBinding(
                "permission_session_evidence",
            ));
        }
        if self.raw_fallback_supported && !self.visible_control_supported {
            return Err(DesktopControlError::UnsafeCapabilityCombination);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DesktopControlError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(CAPABILITY_DOMAIN)?;
        encoder.push_u16(self.schema_version);
        encoder.push_u8(platform_code(self.platform));
        encoder.push_u8(session_kind_code(self.session_kind));
        encoder.push_u64(self.observation_kinds);
        encoder.push_u64(self.semantic_action_kinds);
        encoder.push_u64(self.capture_source_kinds);
        encoder.push_u8(u8::from(self.raw_fallback_supported));
        encoder.push_u8(u8::from(self.pixel_hint_supported));
        encoder.push_u8(u8::from(self.clipboard_read_supported));
        encoder.push_u8(u8::from(self.clipboard_write_supported));
        encoder.push_u8(u8::from(self.human_interrupt_supported));
        encoder.push_u8(u8::from(self.visible_control_supported));
        push_digest(&mut encoder, self.permission_session_evidence)?;
        Ok(encoder.finish())
    }

    pub fn binding_digest(&self) -> Result<BindingDigest, DesktopControlError> {
        Ok(BindingDigest::new(sha256(&self.canonical_bytes()?)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkSurfaceIdentity {
    pub schema_version: u16,
    pub platform: DesktopPlatform,
    pub session_kind: DesktopSessionKind,
    pub surface_id: WorkSurfaceId,
    pub application_identity: Option<BindingDigest>,
    pub incarnation_evidence: BindingDigest,
    pub bounds_geometry_digest: BindingDigest,
    pub observation_generation: u64,
}

impl WorkSurfaceIdentity {
    pub fn validate(&self) -> Result<(), DesktopControlError> {
        validate_schema(self.schema_version)?;
        if self.surface_id.as_u128() == 0 || self.observation_generation == 0 {
            return Err(DesktopControlError::InvalidIdentity);
        }
        require_digest(self.incarnation_evidence, "incarnation_evidence")?;
        require_digest(self.bounds_geometry_digest, "bounds_geometry_digest")?;
        if let Some(value) = self.application_identity {
            require_digest(value, "application_identity")?;
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DesktopControlError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(WORK_SURFACE_DOMAIN)?;
        encoder.push_u16(self.schema_version);
        encoder.push_u8(platform_code(self.platform));
        encoder.push_u8(session_kind_code(self.session_kind));
        encoder.push_u128(self.surface_id.as_u128());
        push_optional_digest(&mut encoder, self.application_identity)?;
        push_digest(&mut encoder, self.incarnation_evidence)?;
        push_digest(&mut encoder, self.bounds_geometry_digest)?;
        encoder.push_u64(self.observation_generation);
        Ok(encoder.finish())
    }

    pub fn binding_digest(&self) -> Result<BindingDigest, DesktopControlError> {
        Ok(BindingDigest::new(sha256(&self.canonical_bytes()?)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticElementIdentity {
    pub schema_version: u16,
    pub parent_work_surface_digest: BindingDigest,
    pub element_id: SemanticElementId,
    pub platform_reference_digest: BindingDigest,
    pub role_control_type_digest: BindingDigest,
    pub supported_action_set_digest: BindingDigest,
    pub state_geometry_digest: BindingDigest,
    pub observation_generation: u64,
}

impl SemanticElementIdentity {
    pub fn validate(&self) -> Result<(), DesktopControlError> {
        validate_schema(self.schema_version)?;
        if self.element_id.as_u128() == 0 || self.observation_generation == 0 {
            return Err(DesktopControlError::InvalidIdentity);
        }
        for (value, field) in [
            (
                self.parent_work_surface_digest,
                "parent_work_surface_digest",
            ),
            (self.platform_reference_digest, "platform_reference_digest"),
            (self.role_control_type_digest, "role_control_type_digest"),
            (
                self.supported_action_set_digest,
                "supported_action_set_digest",
            ),
            (self.state_geometry_digest, "state_geometry_digest"),
        ] {
            require_digest(value, field)?;
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DesktopControlError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(SEMANTIC_ELEMENT_DOMAIN)?;
        encoder.push_u16(self.schema_version);
        push_digest(&mut encoder, self.parent_work_surface_digest)?;
        encoder.push_u128(self.element_id.as_u128());
        push_digest(&mut encoder, self.platform_reference_digest)?;
        push_digest(&mut encoder, self.role_control_type_digest)?;
        push_digest(&mut encoder, self.supported_action_set_digest)?;
        push_digest(&mut encoder, self.state_geometry_digest)?;
        encoder.push_u64(self.observation_generation);
        Ok(encoder.finish())
    }

    pub fn binding_digest(&self) -> Result<BindingDigest, DesktopControlError> {
        Ok(BindingDigest::new(sha256(&self.canonical_bytes()?)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesktopObservation {
    pub schema_version: u16,
    pub observation_id: DesktopObservationId,
    pub observed_at_unix_ms: u64,
    pub capability_session_evidence: BindingDigest,
    pub work_surface_digests: Vec<BindingDigest>,
    pub semantic_summary_digest: BindingDigest,
    pub focused_surface_digest: Option<BindingDigest>,
    pub focused_element_digest: Option<BindingDigest>,
    pub limits: DesktopLimits,
}

impl DesktopObservation {
    pub fn validate(&self) -> Result<(), DesktopControlError> {
        validate_schema(self.schema_version)?;
        self.limits.validate()?;
        if self.observation_id.as_u128() == 0 || self.observed_at_unix_ms == 0 {
            return Err(DesktopControlError::InvalidObservation);
        }
        require_digest(
            self.capability_session_evidence,
            "capability_session_evidence",
        )?;
        require_digest(self.semantic_summary_digest, "semantic_summary_digest")?;
        validate_digest_list(
            &self.work_surface_digests,
            self.limits.max_work_surfaces as usize,
        )?;
        if let Some(value) = self.focused_surface_digest {
            require_digest(value, "focused_surface_digest")?;
        }
        if let Some(value) = self.focused_element_digest {
            require_digest(value, "focused_element_digest")?;
            if self.focused_surface_digest.is_none() {
                return Err(DesktopControlError::InvalidObservation);
            }
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DesktopControlError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(OBSERVATION_DOMAIN)?;
        encoder.push_u16(self.schema_version);
        encoder.push_u128(self.observation_id.as_u128());
        encoder.push_u64(self.observed_at_unix_ms);
        push_digest(&mut encoder, self.capability_session_evidence)?;
        encoder.push_u64(self.work_surface_digests.len() as u64);
        for digest in &self.work_surface_digests {
            push_digest(&mut encoder, *digest)?;
        }
        push_digest(&mut encoder, self.semantic_summary_digest)?;
        push_optional_digest(&mut encoder, self.focused_surface_digest)?;
        push_optional_digest(&mut encoder, self.focused_element_digest)?;
        self.limits.encode(&mut encoder);
        Ok(encoder.finish())
    }

    pub fn binding_digest(&self) -> Result<BindingDigest, DesktopControlError> {
        Ok(BindingDigest::new(sha256(&self.canonical_bytes()?)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopControlMode {
    AgentAllowed,
    Paused,
    HumanExclusive,
    Revoked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopControlLeaseState {
    pub schema_version: u16,
    pub lease_id: DesktopControlLeaseId,
    pub generation: u64,
    pub controlling_principal_ref: BindingDigest,
    pub mode: DesktopControlMode,
    pub issued_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub capability_ref: BindingDigest,
    pub policy_ref: BindingDigest,
    pub interrupt_cause_ref: Option<BindingDigest>,
}

impl DesktopControlLeaseState {
    pub fn validate(&self) -> Result<(), DesktopControlError> {
        validate_schema(self.schema_version)?;
        if self.lease_id.as_u128() == 0
            || self.generation == 0
            || self.issued_at_unix_ms == 0
            || self.updated_at_unix_ms < self.issued_at_unix_ms
            || self.expires_at_unix_ms <= self.updated_at_unix_ms
        {
            return Err(DesktopControlError::InvalidLease);
        }
        require_digest(self.controlling_principal_ref, "controlling_principal_ref")?;
        require_digest(self.capability_ref, "capability_ref")?;
        require_digest(self.policy_ref, "policy_ref")?;
        if let Some(value) = self.interrupt_cause_ref {
            require_digest(value, "interrupt_cause_ref")?;
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DesktopControlError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(LEASE_DOMAIN)?;
        encoder.push_u16(self.schema_version);
        encoder.push_u128(self.lease_id.as_u128());
        encoder.push_u64(self.generation);
        push_digest(&mut encoder, self.controlling_principal_ref)?;
        encoder.push_u8(control_mode_code(self.mode));
        encoder.push_u64(self.issued_at_unix_ms);
        encoder.push_u64(self.updated_at_unix_ms);
        encoder.push_u64(self.expires_at_unix_ms);
        push_digest(&mut encoder, self.capability_ref)?;
        push_digest(&mut encoder, self.policy_ref)?;
        push_optional_digest(&mut encoder, self.interrupt_cause_ref)?;
        Ok(encoder.finish())
    }

    pub fn binding_digest(&self) -> Result<BindingDigest, DesktopControlError> {
        Ok(BindingDigest::new(sha256(&self.canonical_bytes()?)))
    }

    pub fn allows_agent_input(&self, now_unix_ms: u64) -> bool {
        self.mode == DesktopControlMode::AgentAllowed && now_unix_ms < self.expires_at_unix_ms
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VisibleControlChannelKind {
    TauriNativeWindow,
    SystemTray,
    PlatformIndicator,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VisibleControlChannelState {
    pub schema_version: u16,
    pub channel_id: VisibleControlChannelId,
    pub generation: u64,
    pub kind: VisibleControlChannelKind,
    pub trusted_host_ref: BindingDigest,
    pub visible: bool,
    pub live: bool,
    pub supports_pause: bool,
    pub supports_stop: bool,
    pub supports_takeover: bool,
    pub observed_at_unix_ms: u64,
    pub heartbeat_deadline_unix_ms: u64,
}

impl VisibleControlChannelState {
    pub fn validate(&self) -> Result<(), DesktopControlError> {
        validate_schema(self.schema_version)?;
        if self.channel_id.as_u128() == 0
            || self.generation == 0
            || self.observed_at_unix_ms == 0
            || self.heartbeat_deadline_unix_ms <= self.observed_at_unix_ms
        {
            return Err(DesktopControlError::InvalidVisibleChannel);
        }
        require_digest(self.trusted_host_ref, "trusted_host_ref")?;
        if !(self.supports_pause && self.supports_stop && self.supports_takeover) {
            return Err(DesktopControlError::UnsafeCapabilityCombination);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DesktopControlError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(VISIBLE_CHANNEL_DOMAIN)?;
        encoder.push_u16(self.schema_version);
        encoder.push_u128(self.channel_id.as_u128());
        encoder.push_u64(self.generation);
        encoder.push_u8(visible_channel_kind_code(self.kind));
        push_digest(&mut encoder, self.trusted_host_ref)?;
        encoder.push_u8(u8::from(self.visible));
        encoder.push_u8(u8::from(self.live));
        encoder.push_u8(u8::from(self.supports_pause));
        encoder.push_u8(u8::from(self.supports_stop));
        encoder.push_u8(u8::from(self.supports_takeover));
        encoder.push_u64(self.observed_at_unix_ms);
        encoder.push_u64(self.heartbeat_deadline_unix_ms);
        Ok(encoder.finish())
    }

    pub fn binding_digest(&self) -> Result<BindingDigest, DesktopControlError> {
        Ok(BindingDigest::new(sha256(&self.canonical_bytes()?)))
    }

    pub fn qualifies_for_autonomous_actuation(&self, now_unix_ms: u64) -> bool {
        self.visible && self.live && now_unix_ms < self.heartbeat_deadline_unix_ms
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ControlRoute {
    DomainApplicationApi,
    NativeOsAutomationApi,
    AccessibilitySemanticTree,
    BrowserDomProtocol,
    DeterministicKeyboardMouse,
    VisionPixelFallback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouteDisposition {
    Selected,
    Inapplicable,
    Unavailable,
    NotSupported,
    AuthorityDenied,
    PermissionDenied,
    FailedBeforeEffect,
    UnknownOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RouteEvaluation {
    pub route: ControlRoute,
    pub disposition: RouteDisposition,
    pub evidence_ref: BindingDigest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FallbackEligibilityEvidence {
    pub schema_version: u16,
    pub target_task_scope_digest: BindingDigest,
    pub route_evaluations: Vec<RouteEvaluation>,
    pub highest_eligible_route: ControlRoute,
    pub created_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
}

impl FallbackEligibilityEvidence {
    pub fn validate(&self) -> Result<(), DesktopControlError> {
        validate_schema(self.schema_version)?;
        require_digest(self.target_task_scope_digest, "target_task_scope_digest")?;
        if self.route_evaluations.is_empty()
            || self.route_evaluations.len() > MAX_ROUTE_EVALUATIONS
            || self.created_at_unix_ms == 0
            || self.expires_at_unix_ms <= self.created_at_unix_ms
        {
            return Err(DesktopControlError::InvalidFallbackEvidence);
        }

        let mut previous_rank = None;
        let mut selected = None;
        for evaluation in &self.route_evaluations {
            require_digest(evaluation.evidence_ref, "route_evidence_ref")?;
            let rank = route_rank(evaluation.route);
            if previous_rank.is_some_and(|previous| rank <= previous) {
                return Err(DesktopControlError::InvalidRouteOrder);
            }
            previous_rank = Some(rank);
            if evaluation.disposition == RouteDisposition::Selected
                && selected.replace(evaluation.route).is_some()
            {
                return Err(DesktopControlError::MultipleSelectedRoutes);
            }
        }

        if selected != Some(self.highest_eligible_route) {
            return Err(DesktopControlError::SelectedRouteMismatch);
        }

        let selected_rank = route_rank(self.highest_eligible_route);
        for evaluation in &self.route_evaluations {
            if route_rank(evaluation.route) < selected_rank
                && matches!(
                    evaluation.disposition,
                    RouteDisposition::Selected | RouteDisposition::UnknownOutcome
                )
            {
                return Err(DesktopControlError::UnsafeFallbackEscalation);
            }
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DesktopControlError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(FALLBACK_DOMAIN)?;
        encoder.push_u16(self.schema_version);
        push_digest(&mut encoder, self.target_task_scope_digest)?;
        encoder.push_u64(self.route_evaluations.len() as u64);
        for evaluation in &self.route_evaluations {
            encoder.push_u8(route_rank(evaluation.route));
            encoder.push_u8(route_disposition_code(evaluation.disposition));
            push_digest(&mut encoder, evaluation.evidence_ref)?;
        }
        encoder.push_u8(route_rank(self.highest_eligible_route));
        encoder.push_u64(self.created_at_unix_ms);
        encoder.push_u64(self.expires_at_unix_ms);
        Ok(encoder.finish())
    }

    pub fn binding_digest(&self) -> Result<BindingDigest, DesktopControlError> {
        Ok(BindingDigest::new(sha256(&self.canonical_bytes()?)))
    }

    pub fn permits_route(&self, route: ControlRoute, now_unix_ms: u64) -> bool {
        self.validate().is_ok()
            && self.highest_eligible_route == route
            && now_unix_ms < self.expires_at_unix_ms
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PixelRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PixelTargetHint {
    pub schema_version: u16,
    pub source_identity_digest: BindingDigest,
    pub capture_observation_digest: BindingDigest,
    pub region: PixelRegion,
    pub coordinate_space_digest: BindingDigest,
    pub producer_provenance_ref: BindingDigest,
    pub confidence_millis: Option<u16>,
    pub created_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
}

impl PixelTargetHint {
    pub fn validate(&self) -> Result<(), DesktopControlError> {
        validate_schema(self.schema_version)?;
        for (value, field) in [
            (self.source_identity_digest, "source_identity_digest"),
            (
                self.capture_observation_digest,
                "capture_observation_digest",
            ),
            (self.coordinate_space_digest, "coordinate_space_digest"),
            (self.producer_provenance_ref, "producer_provenance_ref"),
        ] {
            require_digest(value, field)?;
        }
        if self.region.width == 0
            || self.region.height == 0
            || self.confidence_millis.is_some_and(|value| value > 1_000)
            || self.created_at_unix_ms == 0
            || self.expires_at_unix_ms <= self.created_at_unix_ms
        {
            return Err(DesktopControlError::InvalidPixelHint);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DesktopControlError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(PIXEL_HINT_DOMAIN)?;
        encoder.push_u16(self.schema_version);
        push_digest(&mut encoder, self.source_identity_digest)?;
        push_digest(&mut encoder, self.capture_observation_digest)?;
        encoder.push_u64(u64::from(self.region.x));
        encoder.push_u64(u64::from(self.region.y));
        encoder.push_u64(u64::from(self.region.width));
        encoder.push_u64(u64::from(self.region.height));
        push_digest(&mut encoder, self.coordinate_space_digest)?;
        push_digest(&mut encoder, self.producer_provenance_ref)?;
        match self.confidence_millis {
            Some(value) => {
                encoder.push_u8(1);
                encoder.push_u16(value);
            }
            None => encoder.push_u8(0),
        }
        encoder.push_u64(self.created_at_unix_ms);
        encoder.push_u64(self.expires_at_unix_ms);
        Ok(encoder.finish())
    }

    pub fn binding_digest(&self) -> Result<BindingDigest, DesktopControlError> {
        Ok(BindingDigest::new(sha256(&self.canonical_bytes()?)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HumanInterruptOperation {
    Pause,
    Stop,
    Takeover,
    ReleaseHumanExclusive,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HumanInterruptEvidence {
    pub schema_version: u16,
    pub interrupt_id: u128,
    pub attributed_local_source_ref: BindingDigest,
    pub operation: HumanInterruptOperation,
    pub prior_lease_id: DesktopControlLeaseId,
    pub prior_generation: u64,
    pub resulting_lease_id: DesktopControlLeaseId,
    pub resulting_generation: u64,
    pub accepted_at_unix_ms: u64,
    pub authority_revoked_at_unix_ms: u64,
    pub affected_operation_refs: Vec<BindingDigest>,
    pub cancellation_reconciliation_refs: Vec<BindingDigest>,
}

impl HumanInterruptEvidence {
    pub fn validate(&self) -> Result<(), DesktopControlError> {
        validate_schema(self.schema_version)?;
        if self.interrupt_id == 0
            || self.prior_lease_id.as_u128() == 0
            || self.resulting_lease_id.as_u128() == 0
            || self.prior_generation == 0
            || self.resulting_generation <= self.prior_generation
            || self.accepted_at_unix_ms == 0
            || self.authority_revoked_at_unix_ms < self.accepted_at_unix_ms
        {
            return Err(DesktopControlError::InvalidInterruptEvidence);
        }
        require_digest(
            self.attributed_local_source_ref,
            "attributed_local_source_ref",
        )?;
        validate_digest_list(&self.affected_operation_refs, MAX_INTERRUPT_REFS)?;
        validate_digest_list(&self.cancellation_reconciliation_refs, MAX_INTERRUPT_REFS)?;
        Ok(())
    }

    pub fn takeover_latency_ms(&self) -> Result<u64, DesktopControlError> {
        self.validate()?;
        Ok(self.authority_revoked_at_unix_ms - self.accepted_at_unix_ms)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DesktopControlError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(HUMAN_INTERRUPT_DOMAIN)?;
        encoder.push_u16(self.schema_version);
        encoder.push_u128(self.interrupt_id);
        push_digest(&mut encoder, self.attributed_local_source_ref)?;
        encoder.push_u8(interrupt_operation_code(self.operation));
        encoder.push_u128(self.prior_lease_id.as_u128());
        encoder.push_u64(self.prior_generation);
        encoder.push_u128(self.resulting_lease_id.as_u128());
        encoder.push_u64(self.resulting_generation);
        encoder.push_u64(self.accepted_at_unix_ms);
        encoder.push_u64(self.authority_revoked_at_unix_ms);
        push_digest_list(&mut encoder, &self.affected_operation_refs)?;
        push_digest_list(&mut encoder, &self.cancellation_reconciliation_refs)?;
        Ok(encoder.finish())
    }

    pub fn binding_digest(&self) -> Result<BindingDigest, DesktopControlError> {
        Ok(BindingDigest::new(sha256(&self.canonical_bytes()?)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopControlError {
    InvalidSchemaVersion,
    InvalidLimits,
    InvalidIdentity,
    InvalidObservation,
    InvalidLease,
    InvalidVisibleChannel,
    InvalidFallbackEvidence,
    InvalidRouteOrder,
    MultipleSelectedRoutes,
    SelectedRouteMismatch,
    UnsafeFallbackEscalation,
    InvalidPixelHint,
    InvalidInterruptEvidence,
    UnsafeCapabilityCombination,
    MissingBinding(&'static str),
    TooManyBindings,
    UnsortedOrDuplicateBindings,
    CanonicalEncoding(CoreError),
}

impl fmt::Display for DesktopControlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSchemaVersion => f.write_str("invalid desktop-control schema version"),
            Self::InvalidLimits => f.write_str("desktop-control limits are invalid or unbounded"),
            Self::InvalidIdentity => f.write_str("desktop identity is invalid or incomplete"),
            Self::InvalidObservation => {
                f.write_str("desktop observation is invalid or inconsistent")
            }
            Self::InvalidLease => f.write_str("desktop control lease is invalid"),
            Self::InvalidVisibleChannel => f.write_str("visible control channel is invalid"),
            Self::InvalidFallbackEvidence => {
                f.write_str("fallback eligibility evidence is invalid")
            }
            Self::InvalidRouteOrder => {
                f.write_str("control routes are not in constitutional order")
            }
            Self::MultipleSelectedRoutes => {
                f.write_str("fallback evidence selects multiple routes")
            }
            Self::SelectedRouteMismatch => {
                f.write_str("selected route does not match highest eligible route")
            }
            Self::UnsafeFallbackEscalation => {
                f.write_str("fallback escalation is blocked by stronger-route state")
            }
            Self::InvalidPixelHint => {
                f.write_str("pixel target hint is invalid, stale, or unbounded")
            }
            Self::InvalidInterruptEvidence => f.write_str("human interrupt evidence is invalid"),
            Self::UnsafeCapabilityCombination => {
                f.write_str("desktop capability combination is unsafe")
            }
            Self::MissingBinding(field) => write!(f, "missing canonical desktop binding: {field}"),
            Self::TooManyBindings => f.write_str("too many bounded desktop bindings"),
            Self::UnsortedOrDuplicateBindings => {
                f.write_str("desktop binding list must be sorted and unique")
            }
            Self::CanonicalEncoding(error) => write!(f, "canonical encoding error: {error}"),
        }
    }
}

impl std::error::Error for DesktopControlError {}

impl From<CoreError> for DesktopControlError {
    fn from(value: CoreError) -> Self {
        Self::CanonicalEncoding(value)
    }
}

fn validate_schema(schema_version: u16) -> Result<(), DesktopControlError> {
    if schema_version != DESKTOP_CONTROL_SCHEMA_VERSION {
        return Err(DesktopControlError::InvalidSchemaVersion);
    }
    Ok(())
}

fn require_digest(digest: BindingDigest, field: &'static str) -> Result<(), DesktopControlError> {
    if digest_is_zero(digest) {
        return Err(DesktopControlError::MissingBinding(field));
    }
    Ok(())
}

fn digest_is_zero(digest: BindingDigest) -> bool {
    digest.bytes().iter().all(|byte| *byte == 0)
}

fn validate_digest_list(values: &[BindingDigest], max: usize) -> Result<(), DesktopControlError> {
    if values.len() > max {
        return Err(DesktopControlError::TooManyBindings);
    }
    if values.iter().any(|value| digest_is_zero(*value)) {
        return Err(DesktopControlError::MissingBinding("digest_list"));
    }
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(DesktopControlError::UnsortedOrDuplicateBindings);
    }
    Ok(())
}

fn push_digest(
    encoder: &mut CanonicalEncoder,
    digest: BindingDigest,
) -> Result<(), DesktopControlError> {
    encoder.push_bytes(&digest.bytes())?;
    Ok(())
}

fn push_optional_digest(
    encoder: &mut CanonicalEncoder,
    digest: Option<BindingDigest>,
) -> Result<(), DesktopControlError> {
    match digest {
        Some(value) => {
            encoder.push_u8(1);
            push_digest(encoder, value)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn push_digest_list(
    encoder: &mut CanonicalEncoder,
    values: &[BindingDigest],
) -> Result<(), DesktopControlError> {
    encoder.push_u64(values.len() as u64);
    for value in values {
        push_digest(encoder, *value)?;
    }
    Ok(())
}

const fn platform_code(platform: DesktopPlatform) -> u8 {
    match platform {
        DesktopPlatform::Windows => 1,
        DesktopPlatform::Macos => 2,
        DesktopPlatform::Linux => 3,
    }
}

const fn session_kind_code(kind: DesktopSessionKind) -> u8 {
    match kind {
        DesktopSessionKind::WindowsInteractive => 1,
        DesktopSessionKind::MacosLogin => 2,
        DesktopSessionKind::LinuxX11 => 3,
        DesktopSessionKind::LinuxWayland => 4,
    }
}

const fn control_mode_code(mode: DesktopControlMode) -> u8 {
    match mode {
        DesktopControlMode::AgentAllowed => 1,
        DesktopControlMode::Paused => 2,
        DesktopControlMode::HumanExclusive => 3,
        DesktopControlMode::Revoked => 4,
    }
}

const fn visible_channel_kind_code(kind: VisibleControlChannelKind) -> u8 {
    match kind {
        VisibleControlChannelKind::TauriNativeWindow => 1,
        VisibleControlChannelKind::SystemTray => 2,
        VisibleControlChannelKind::PlatformIndicator => 3,
    }
}

const fn route_rank(route: ControlRoute) -> u8 {
    match route {
        ControlRoute::DomainApplicationApi => 1,
        ControlRoute::NativeOsAutomationApi => 2,
        ControlRoute::AccessibilitySemanticTree => 3,
        ControlRoute::BrowserDomProtocol => 4,
        ControlRoute::DeterministicKeyboardMouse => 5,
        ControlRoute::VisionPixelFallback => 6,
    }
}

const fn route_disposition_code(disposition: RouteDisposition) -> u8 {
    match disposition {
        RouteDisposition::Selected => 1,
        RouteDisposition::Inapplicable => 2,
        RouteDisposition::Unavailable => 3,
        RouteDisposition::NotSupported => 4,
        RouteDisposition::AuthorityDenied => 5,
        RouteDisposition::PermissionDenied => 6,
        RouteDisposition::FailedBeforeEffect => 7,
        RouteDisposition::UnknownOutcome => 8,
    }
}

const fn interrupt_operation_code(operation: HumanInterruptOperation) -> u8 {
    match operation {
        HumanInterruptOperation::Pause => 1,
        HumanInterruptOperation::Stop => 2,
        HumanInterruptOperation::Takeover => 3,
        HumanInterruptOperation::ReleaseHumanExclusive => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    #[test]
    fn defaults_are_bounded() {
        assert!(DesktopLimits::default().validate().is_ok());
    }

    #[test]
    fn work_surface_identity_changes_when_incarnation_changes() {
        let mut identity = WorkSurfaceIdentity {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            platform: DesktopPlatform::Linux,
            session_kind: DesktopSessionKind::LinuxWayland,
            surface_id: WorkSurfaceId::from_u128(7),
            application_identity: Some(digest(1)),
            incarnation_evidence: digest(2),
            bounds_geometry_digest: digest(3),
            observation_generation: 4,
        };
        let first = identity.binding_digest().unwrap();
        identity.incarnation_evidence = digest(9);
        assert_ne!(first, identity.binding_digest().unwrap());
    }

    #[test]
    fn constitutional_route_order_is_fail_closed() {
        let evidence = FallbackEligibilityEvidence {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            target_task_scope_digest: digest(1),
            route_evaluations: vec![
                RouteEvaluation {
                    route: ControlRoute::AccessibilitySemanticTree,
                    disposition: RouteDisposition::Unavailable,
                    evidence_ref: digest(2),
                },
                RouteEvaluation {
                    route: ControlRoute::DeterministicKeyboardMouse,
                    disposition: RouteDisposition::Selected,
                    evidence_ref: digest(3),
                },
            ],
            highest_eligible_route: ControlRoute::DeterministicKeyboardMouse,
            created_at_unix_ms: 10,
            expires_at_unix_ms: 20,
        };
        assert!(evidence.validate().is_ok());

        let mut unsafe_evidence = evidence.clone();
        unsafe_evidence.route_evaluations[0].disposition = RouteDisposition::UnknownOutcome;
        assert_eq!(
            unsafe_evidence.validate(),
            Err(DesktopControlError::UnsafeFallbackEscalation)
        );
    }

    #[test]
    fn visible_channel_must_expose_all_immediate_controls() {
        let state = VisibleControlChannelState {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            channel_id: VisibleControlChannelId::from_u128(1),
            generation: 1,
            kind: VisibleControlChannelKind::TauriNativeWindow,
            trusted_host_ref: digest(1),
            visible: true,
            live: true,
            supports_pause: true,
            supports_stop: true,
            supports_takeover: false,
            observed_at_unix_ms: 10,
            heartbeat_deadline_unix_ms: 20,
        };
        assert_eq!(
            state.validate(),
            Err(DesktopControlError::UnsafeCapabilityCombination)
        );
    }

    #[test]
    fn human_interrupt_requires_generation_advance() {
        let evidence = HumanInterruptEvidence {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            interrupt_id: 1,
            attributed_local_source_ref: digest(1),
            operation: HumanInterruptOperation::Takeover,
            prior_lease_id: DesktopControlLeaseId::from_u128(1),
            prior_generation: 4,
            resulting_lease_id: DesktopControlLeaseId::from_u128(1),
            resulting_generation: 4,
            accepted_at_unix_ms: 10,
            authority_revoked_at_unix_ms: 11,
            affected_operation_refs: vec![],
            cancellation_reconciliation_refs: vec![],
        };
        assert_eq!(
            evidence.validate(),
            Err(DesktopControlError::InvalidInterruptEvidence)
        );
    }
}
