#![forbid(unsafe_code)]

use core::fmt;

use golam_core::desktop_control::{
    ControlRoute, DESKTOP_CONTROL_SCHEMA_VERSION, DesktopControlLeaseId, FallbackEligibilityEvidence,
    RouteDisposition, RouteEvaluation, VisibleControlChannelId,
};
use golam_core::desktop_intent::{
    AuthorityBindings, DesktopActionKind, EffectBinding, InteractiveAuthorityBinding,
    PreparedDesktopAction, RequestBinding,
};
use golam_core::digest::sha256;
use golam_core::tool_request::{BindingDigest, PreparedToolRequest};
use golam_core::{CanonicalEncoder, CoreError, EffectId, SessionId};

use crate::{
    AuthorizationPolicy, KernelApi, PrepareToolEffect, PreparedToolEffect, Principal, ToolEffectError,
};

const ROUTE_SCOPE_DOMAIN: &[u8] = b"golam:desktop-route-scope:v1";
const ACTION_PAYLOAD_DOMAIN: &[u8] = b"golam:desktop-kernel-action-payload:v1";
const EFFECT_BINDING_DOMAIN: &[u8] = b"golam:desktop-effect-binding:v1";
const GATE_BINDING_DOMAIN: &[u8] = b"golam:desktop-gate-binding:v1";
const HANDLER_ID: &str = "golam-desktop-kernel";
const HANDLER_VERSION: &str = "1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustedRouteDisposition {
    Eligible,
    Inapplicable,
    Unavailable,
    NotSupported,
    AuthorityDenied,
    PermissionDenied,
    FailedBeforeEffect,
    UnknownOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrustedRouteCandidate {
    pub route: ControlRoute,
    pub disposition: TrustedRouteDisposition,
    pub evidence_ref: BindingDigest,
}

pub fn evaluate_desktop_routes(
    target_task_scope_digest: BindingDigest,
    candidates: &[TrustedRouteCandidate],
    created_at_unix_ms: u64,
    expires_at_unix_ms: u64,
) -> Result<FallbackEligibilityEvidence, DesktopKernelError> {
    if target_task_scope_digest.bytes() == [0; 32]
        || candidates.is_empty()
        || candidates.len() > 6
        || created_at_unix_ms == 0
        || expires_at_unix_ms <= created_at_unix_ms
    {
        return Err(DesktopKernelError::InvalidRouteEvidence);
    }

    let order = route_order();
    let mut evaluations = Vec::with_capacity(candidates.len());
    let mut selected = None;
    for (index, candidate) in candidates.iter().enumerate() {
        if candidate.route != order[index] || candidate.evidence_ref.bytes() == [0; 32] {
            return Err(DesktopKernelError::InvalidRouteEvidence);
        }
        let disposition = match candidate.disposition {
            TrustedRouteDisposition::Eligible => {
                selected = Some(candidate.route);
                RouteDisposition::Selected
            }
            TrustedRouteDisposition::Inapplicable => RouteDisposition::Inapplicable,
            TrustedRouteDisposition::Unavailable => RouteDisposition::Unavailable,
            TrustedRouteDisposition::NotSupported => RouteDisposition::NotSupported,
            TrustedRouteDisposition::AuthorityDenied => RouteDisposition::AuthorityDenied,
            TrustedRouteDisposition::PermissionDenied => RouteDisposition::PermissionDenied,
            TrustedRouteDisposition::FailedBeforeEffect => RouteDisposition::FailedBeforeEffect,
            TrustedRouteDisposition::UnknownOutcome => {
                return Err(DesktopKernelError::UnknownOutcomeBlocksRouteEscalation);
            }
        };
        evaluations.push(RouteEvaluation {
            route: candidate.route,
            disposition,
            evidence_ref: candidate.evidence_ref,
        });
        if selected.is_some() {
            break;
        }
    }

    let selected = selected.ok_or(DesktopKernelError::NoEligibleRoute)?;
    let evidence = FallbackEligibilityEvidence {
        schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
        target_task_scope_digest,
        route_evaluations: evaluations,
        highest_eligible_route: selected,
        created_at_unix_ms,
        expires_at_unix_ms,
    };
    evidence.validate()?;
    Ok(evidence)
}

pub struct PrepareDesktopAction<'a> {
    pub request: &'a PreparedToolRequest,
    pub effect_id: EffectId,
    pub session_id: SessionId,
    pub operation_kind: DesktopActionKind,
    pub route_candidates: &'a [TrustedRouteCandidate],
    pub route_evidence_created_at_unix_ms: u64,
    pub route_evidence_expires_at_unix_ms: u64,
    pub exact_target_identity_digest: BindingDigest,
    pub pixel_hint_digest: Option<BindingDigest>,
    pub action_payload_digest: BindingDigest,
    pub authority: AuthorityBindings,
    pub control_lease_id: DesktopControlLeaseId,
    pub control_lease_generation: u64,
    pub visible_channel_id: VisibleControlChannelId,
    pub visible_channel_generation: u64,
    pub visible_channel_state_digest: BindingDigest,
    pub permission_session_evidence_ref: BindingDigest,
    pub observation_digest: BindingDigest,
    pub expires_at_unix_ms: u64,
    pub started_at: &'a str,
    pub idempotency_key: Option<&'a str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedDesktopKernelAction {
    effect: PreparedToolEffect,
    intent: PreparedDesktopAction,
    route_evidence: FallbackEligibilityEvidence,
}

impl PreparedDesktopKernelAction {
    pub fn effect(&self) -> &PreparedToolEffect {
        &self.effect
    }

    pub fn intent(&self) -> &PreparedDesktopAction {
        &self.intent
    }

    pub fn route_evidence(&self) -> &FallbackEligibilityEvidence {
        &self.route_evidence
    }

    pub const fn selected_route(&self) -> ControlRoute {
        self.route_evidence.highest_eligible_route
    }
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn prepare_desktop_action(
        &mut self,
        principal: Principal<'_>,
        input: PrepareDesktopAction<'_>,
        scope: &str,
    ) -> Result<PreparedDesktopKernelAction, DesktopKernelError> {
        validate_prepare_input(&input)?;
        let request_digest = BindingDigest::new(input.request.binding_digest());
        let route_scope = desktop_route_scope_digest(
            request_digest,
            input.exact_target_identity_digest,
            input.action_payload_digest,
        )?;
        let route_evidence = evaluate_desktop_routes(
            route_scope,
            input.route_candidates,
            input.route_evidence_created_at_unix_ms,
            input.route_evidence_expires_at_unix_ms,
        )?;
        validate_operation_route(input.operation_kind, route_evidence.highest_eligible_route)?;
        let route_digest = route_evidence.binding_digest()?;
        let payload_hash = desktop_action_payload_hash(&input, request_digest, route_digest)?;
        let resource = desktop_target_resource(input.exact_target_identity_digest);
        let action = action_name(input.operation_kind);
        let prepared = self.prepare_tool_effect(
            principal,
            PrepareToolEffect {
                effect_id: input.effect_id,
                session_id: input.session_id,
                action,
                resource: &resource,
                execution_semantics: "at_most_once",
                handler_id: HANDLER_ID,
                handler_version: HANDLER_VERSION,
                idempotency_key: input.idempotency_key,
                preconditions_hash: input.observation_digest.bytes(),
                payload_hash,
                started_at: input.started_at,
            },
            scope,
        )?;
        let effect = effect_binding(&prepared)?;
        let intent = PreparedDesktopAction {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            request: RequestBinding {
                request_id: input.request.request().request_id,
                canonical_request_digest: request_digest,
            },
            effect,
            operation_kind: input.operation_kind,
            exact_target_identity_digest: input.exact_target_identity_digest,
            pixel_hint_digest: input.pixel_hint_digest,
            fallback_eligibility_evidence_digest: match input.operation_kind {
                DesktopActionKind::RawInputFallback => Some(route_digest),
                DesktopActionKind::SemanticAction | DesktopActionKind::Focus => None,
            },
            action_payload_digest: input.action_payload_digest,
            authority: input.authority,
            interactive_authority: InteractiveAuthorityBinding {
                lease_id: input.control_lease_id,
                lease_generation: input.control_lease_generation,
                visible_channel_id: input.visible_channel_id,
                visible_channel_generation: input.visible_channel_generation,
                visible_channel_state_digest: input.visible_channel_state_digest,
            },
            prepared_permission_session_evidence_ref: input.permission_session_evidence_ref,
            prepared_observation_digest: input.observation_digest,
            expires_at_unix_ms: input.expires_at_unix_ms,
        };
        intent.validate()?;
        Ok(PreparedDesktopKernelAction {
            effect: prepared,
            intent,
            route_evidence,
        })
    }
}

pub fn effect_binding(prepared: &PreparedToolEffect) -> Result<EffectBinding, DesktopKernelError> {
    let mut effect_encoder = CanonicalEncoder::new();
    effect_encoder.push_bytes(EFFECT_BINDING_DOMAIN)?;
    effect_encoder.push_u128(prepared.effect_id().0);
    effect_encoder.push_u128(prepared.attempt_id().0);
    effect_encoder.push_bytes(prepared.action().as_bytes())?;
    effect_encoder.push_bytes(prepared.resource().as_bytes())?;
    effect_encoder.push_bytes(&prepared.preconditions_hash())?;
    effect_encoder.push_bytes(&prepared.payload_hash())?;
    let immutable_effect_digest = BindingDigest::new(sha256(&effect_encoder.finish()));

    let mut gate_encoder = CanonicalEncoder::new();
    gate_encoder.push_bytes(GATE_BINDING_DOMAIN)?;
    gate_encoder.push_u128(prepared.effect_id().0);
    gate_encoder.push_u128(prepared.attempt_id().0);
    gate_encoder.push_bytes(&immutable_effect_digest.bytes())?;
    gate_encoder.push_bytes(b"executing")?;
    let gate_authorization_digest = BindingDigest::new(sha256(&gate_encoder.finish()));
    Ok(EffectBinding {
        effect_id: prepared.effect_id(),
        immutable_effect_digest,
        gate_authorization_digest,
    })
}

fn validate_prepare_input(input: &PrepareDesktopAction<'_>) -> Result<(), DesktopKernelError> {
    if input.effect_id.0 == 0
        || input.started_at.is_empty()
        || input.expires_at_unix_ms == 0
        || input.control_lease_id.as_u128() == 0
        || input.control_lease_generation == 0
        || input.visible_channel_id.as_u128() == 0
        || input.visible_channel_generation == 0
    {
        return Err(DesktopKernelError::InvalidPrepareInput);
    }
    for digest in [
        input.exact_target_identity_digest,
        input.action_payload_digest,
        input.authority.capability_ref,
        input.authority.policy_ref,
        input.authority.approval_ref,
        input.visible_channel_state_digest,
        input.permission_session_evidence_ref,
        input.observation_digest,
    ] {
        if digest.bytes() == [0; 32] {
            return Err(DesktopKernelError::InvalidPrepareInput);
        }
    }
    match input.operation_kind {
        DesktopActionKind::RawInputFallback => {}
        DesktopActionKind::SemanticAction | DesktopActionKind::Focus => {
            if input.pixel_hint_digest.is_some() {
                return Err(DesktopKernelError::UnexpectedPixelHint);
            }
        }
    }
    Ok(())
}

fn validate_operation_route(
    kind: DesktopActionKind,
    route: ControlRoute,
) -> Result<(), DesktopKernelError> {
    match (kind, route) {
        (
            DesktopActionKind::SemanticAction | DesktopActionKind::Focus,
            ControlRoute::NativeOsAutomationApi | ControlRoute::AccessibilitySemanticTree,
        ) => Ok(()),
        (DesktopActionKind::RawInputFallback, ControlRoute::DeterministicKeyboardMouse) => Ok(()),
        (_, ControlRoute::DomainApplicationApi | ControlRoute::BrowserDomProtocol) => {
            Err(DesktopKernelError::HigherPriorityNonDesktopRouteSelected)
        }
        _ => Err(DesktopKernelError::UnsupportedDesktopRoute),
    }
}

fn desktop_route_scope_digest(
    request_digest: BindingDigest,
    target_digest: BindingDigest,
    payload_digest: BindingDigest,
) -> Result<BindingDigest, DesktopKernelError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(ROUTE_SCOPE_DOMAIN)?;
    encoder.push_bytes(&request_digest.bytes())?;
    encoder.push_bytes(&target_digest.bytes())?;
    encoder.push_bytes(&payload_digest.bytes())?;
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn desktop_action_payload_hash(
    input: &PrepareDesktopAction<'_>,
    request_digest: BindingDigest,
    route_digest: BindingDigest,
) -> Result<[u8; 32], DesktopKernelError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(ACTION_PAYLOAD_DOMAIN)?;
    encoder.push_bytes(&request_digest.bytes())?;
    encoder.push_u8(action_kind_code(input.operation_kind));
    encoder.push_bytes(&route_digest.bytes())?;
    encoder.push_bytes(&input.exact_target_identity_digest.bytes())?;
    push_optional_digest(&mut encoder, input.pixel_hint_digest)?;
    encoder.push_bytes(&input.action_payload_digest.bytes())?;
    encoder.push_bytes(&input.authority.capability_ref.bytes())?;
    encoder.push_bytes(&input.authority.policy_ref.bytes())?;
    encoder.push_bytes(&input.authority.approval_ref.bytes())?;
    encoder.push_u128(input.control_lease_id.as_u128());
    encoder.push_u64(input.control_lease_generation);
    encoder.push_u128(input.visible_channel_id.as_u128());
    encoder.push_u64(input.visible_channel_generation);
    encoder.push_bytes(&input.visible_channel_state_digest.bytes())?;
    encoder.push_bytes(&input.permission_session_evidence_ref.bytes())?;
    encoder.push_bytes(&input.observation_digest.bytes())?;
    encoder.push_u64(input.expires_at_unix_ms);
    Ok(sha256(&encoder.finish()))
}

fn desktop_target_resource(digest: BindingDigest) -> String {
    let bytes = digest.bytes();
    let mut output = String::with_capacity(31);
    output.push_str("desktop-target:");
    for byte in &bytes[..8] {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn action_name(kind: DesktopActionKind) -> &'static str {
    match kind {
        DesktopActionKind::SemanticAction => "desktop.semantic_action",
        DesktopActionKind::RawInputFallback => "desktop.raw_input",
        DesktopActionKind::Focus => "desktop.focus",
    }
}

fn action_kind_code(kind: DesktopActionKind) -> u8 {
    match kind {
        DesktopActionKind::SemanticAction => 1,
        DesktopActionKind::RawInputFallback => 2,
        DesktopActionKind::Focus => 3,
    }
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

const fn route_order() -> [ControlRoute; 6] {
    [
        ControlRoute::DomainApplicationApi,
        ControlRoute::NativeOsAutomationApi,
        ControlRoute::AccessibilitySemanticTree,
        ControlRoute::BrowserDomProtocol,
        ControlRoute::DeterministicKeyboardMouse,
        ControlRoute::VisionPixelFallback,
    ]
}

#[derive(Debug)]
pub enum DesktopKernelError {
    InvalidRouteEvidence,
    UnknownOutcomeBlocksRouteEscalation,
    NoEligibleRoute,
    InvalidPrepareInput,
    UnexpectedPixelHint,
    HigherPriorityNonDesktopRouteSelected,
    UnsupportedDesktopRoute,
    Core(CoreError),
    Control(golam_core::desktop_control::DesktopControlError),
    Intent(golam_core::desktop_intent::DesktopIntentError),
    Effect(ToolEffectError),
}

impl fmt::Display for DesktopKernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRouteEvidence => f.write_str("invalid trusted desktop route evidence"),
            Self::UnknownOutcomeBlocksRouteEscalation => {
                f.write_str("UNKNOWN_OUTCOME blocks weaker desktop route escalation")
            }
            Self::NoEligibleRoute => f.write_str("no eligible desktop control route"),
            Self::InvalidPrepareInput => f.write_str("invalid desktop effect preparation input"),
            Self::UnexpectedPixelHint => f.write_str("pixel hint is permitted only for raw input"),
            Self::HigherPriorityNonDesktopRouteSelected => {
                f.write_str("higher-priority domain/browser route selected")
            }
            Self::UnsupportedDesktopRoute => f.write_str("selected route is unsupported here"),
            Self::Core(error) => write!(f, "desktop kernel encoding error: {error}"),
            Self::Control(error) => write!(f, "desktop kernel control error: {error}"),
            Self::Intent(error) => write!(f, "desktop kernel intent error: {error}"),
            Self::Effect(error) => write!(f, "desktop kernel effect error: {error}"),
        }
    }
}

impl std::error::Error for DesktopKernelError {}

impl From<CoreError> for DesktopKernelError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<golam_core::desktop_control::DesktopControlError> for DesktopKernelError {
    fn from(value: golam_core::desktop_control::DesktopControlError) -> Self {
        Self::Control(value)
    }
}

impl From<golam_core::desktop_intent::DesktopIntentError> for DesktopKernelError {
    fn from(value: golam_core::desktop_intent::DesktopIntentError) -> Self {
        Self::Intent(value)
    }
}

impl From<ToolEffectError> for DesktopKernelError {
    fn from(value: ToolEffectError) -> Self {
        Self::Effect(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    #[test]
    fn route_evaluator_selects_first_eligible_route_in_constitutional_order() {
        let evidence = evaluate_desktop_routes(
            digest(1),
            &[
                TrustedRouteCandidate {
                    route: ControlRoute::DomainApplicationApi,
                    disposition: TrustedRouteDisposition::Unavailable,
                    evidence_ref: digest(2),
                },
                TrustedRouteCandidate {
                    route: ControlRoute::NativeOsAutomationApi,
                    disposition: TrustedRouteDisposition::Eligible,
                    evidence_ref: digest(3),
                },
            ],
            10,
            20,
        )
        .unwrap();
        assert_eq!(
            evidence.highest_eligible_route,
            ControlRoute::NativeOsAutomationApi
        );
        assert_eq!(evidence.route_evaluations.len(), 2);
    }

    #[test]
    fn route_evaluator_rejects_skipped_stronger_route_and_unknown_outcome() {
        assert!(matches!(
            evaluate_desktop_routes(
                digest(1),
                &[TrustedRouteCandidate {
                    route: ControlRoute::NativeOsAutomationApi,
                    disposition: TrustedRouteDisposition::Eligible,
                    evidence_ref: digest(2),
                }],
                10,
                20,
            ),
            Err(DesktopKernelError::InvalidRouteEvidence)
        ));
        assert!(matches!(
            evaluate_desktop_routes(
                digest(1),
                &[TrustedRouteCandidate {
                    route: ControlRoute::DomainApplicationApi,
                    disposition: TrustedRouteDisposition::UnknownOutcome,
                    evidence_ref: digest(2),
                }],
                10,
                20,
            ),
            Err(DesktopKernelError::UnknownOutcomeBlocksRouteEscalation)
        ));
    }
}
