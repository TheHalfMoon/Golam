#![forbid(unsafe_code)]

#[path = "../src/desktop_observation.rs"]
pub mod desktop_observation;

use desktop_observation::{NativeObservationRequest, observe_native_desktop};
use golam_core::EffectId;
use golam_core::desktop_backend::{DesktopActionDispatchContext, DesktopBackendError};
use golam_core::desktop_control::{
    DESKTOP_CONTROL_SCHEMA_VERSION, DesktopCapabilitySet, DesktopControlLeaseId,
    DesktopControlLeaseState, DesktopControlMode, DesktopLimits, DesktopObservation,
    DesktopObservationId, DesktopPlatform, DesktopSessionKind, PixelRegion, PixelTargetHint,
    VisibleControlChannelId, VisibleControlChannelKind, VisibleControlChannelState,
};
use golam_core::desktop_intent::{
    AuthorityBindings, DesktopActionKind, DesktopIntentError, EffectBinding,
    InteractiveAuthorityBinding, PreparedDesktopAction, RequestBinding,
};
use golam_core::tool_request::{BindingDigest, ToolRequestId};

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

fn action(kind: DesktopActionKind) -> PreparedDesktopAction {
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
        fallback_eligibility_evidence_digest: None,
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

fn context<'a>(
    prepared: &'a PreparedDesktopAction,
    caps: &'a DesktopCapabilitySet,
    observed: &'a DesktopObservation,
    current_lease: &'a DesktopControlLeaseState,
    current_channel: &'a VisibleControlChannelState,
) -> DesktopActionDispatchContext<'a> {
    DesktopActionDispatchContext {
        action: prepared,
        now_unix_ms: 110,
        current_request_digest: digest(1),
        current_effect_digest: digest(2),
        current_gate_authorization_digest: digest(3),
        current_authority: authority(),
        current_capabilities: caps,
        current_observation: observed,
        current_target_identity_digest: digest(30),
        current_lease,
        current_visible_channel: current_channel,
        fallback_evidence: None,
        pixel_hint: None,
        unresolved_conflicting_unknown_outcome: false,
    }
}

fn pixel_hint() -> PixelTargetHint {
    PixelTargetHint {
        schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
        source_identity_digest: digest(30),
        capture_observation_digest: observation().binding_digest().unwrap(),
        region: PixelRegion {
            x: 10,
            y: 20,
            width: 40,
            height: 30,
        },
        coordinate_space_digest: digest(52),
        producer_provenance_ref: digest(53),
        confidence_millis: Some(900),
        created_at_unix_ms: 100,
        expires_at_unix_ms: 200,
    }
}

#[test]
fn native_observation_adapter_is_linked_and_request_validation_is_fail_closed() {
    let observe = observe_native_desktop
        as fn(
            NativeObservationRequest,
        ) -> Result<
            desktop_observation::NativeObservationReport,
            desktop_observation::NativeObservationError,
        >;
    let _ = observe;

    let valid = NativeObservationRequest {
        platform: DesktopPlatform::Linux,
        session_kind: DesktopSessionKind::LinuxX11,
        capability_session_evidence: BindingDigest::new([7; 32]),
        observed_at_unix_ms: 1_900_000_000_000,
        observation_generation: 1,
        limits: DesktopLimits::default(),
    };
    assert!(valid.validate().is_ok());

    let substituted = NativeObservationRequest {
        session_kind: DesktopSessionKind::WindowsInteractive,
        ..valid
    };
    assert!(substituted.validate().is_err());
}

#[test]
fn substituted_gate_surface_session_and_visible_channel_fail_closed() {
    let caps = capabilities();
    let observed = observation();
    let current_lease = lease();
    let current_channel = channel();
    let prepared = action(DesktopActionKind::Focus);

    let mut substituted_gate = context(
        &prepared,
        &caps,
        &observed,
        &current_lease,
        &current_channel,
    );
    substituted_gate.current_gate_authorization_digest = digest(98);
    assert_eq!(
        substituted_gate.authorize().unwrap_err(),
        DesktopBackendError::GateAuthorizationMismatch
    );

    let mut stale_surface = context(
        &prepared,
        &caps,
        &observed,
        &current_lease,
        &current_channel,
    );
    stale_surface.current_target_identity_digest = digest(98);
    assert_eq!(
        stale_surface.authorize().unwrap_err(),
        DesktopBackendError::StaleOrSubstitutedTarget
    );

    let mut drifted_caps = caps;
    drifted_caps.permission_session_evidence = digest(98);
    assert_eq!(
        context(
            &prepared,
            &drifted_caps,
            &observed,
            &current_lease,
            &current_channel,
        )
        .authorize()
        .unwrap_err(),
        DesktopBackendError::PermissionOrSessionDrift
    );

    let mut drifted_observation = observed.clone();
    drifted_observation.capability_session_evidence = digest(98);
    assert_eq!(
        context(
            &prepared,
            &caps,
            &drifted_observation,
            &current_lease,
            &current_channel,
        )
        .authorize()
        .unwrap_err(),
        DesktopBackendError::PermissionOrSessionDrift
    );

    let mut superseded_generation = current_channel;
    superseded_generation.generation += 1;
    assert_eq!(
        context(
            &prepared,
            &caps,
            &observed,
            &current_lease,
            &superseded_generation,
        )
        .authorize()
        .unwrap_err(),
        DesktopBackendError::AutonomousActuationSuspended
    );

    let mut substituted_state = current_channel;
    substituted_state.trusted_host_ref = digest(98);
    assert_eq!(
        context(
            &prepared,
            &caps,
            &observed,
            &current_lease,
            &substituted_state,
        )
        .authorize()
        .unwrap_err(),
        DesktopBackendError::AutonomousActuationSuspended
    );
}

#[test]
fn observation_and_pixel_evidence_never_mint_authority_or_fallback() {
    let caps = capabilities();
    let observed = observation();
    let current_lease = lease();
    let current_channel = channel();
    let semantic = action(DesktopActionKind::SemanticAction);

    let mut observation_as_authority = context(
        &semantic,
        &caps,
        &observed,
        &current_lease,
        &current_channel,
    );
    observation_as_authority.current_authority = AuthorityBindings {
        capability_ref: observed.semantic_summary_digest,
        policy_ref: observed.semantic_summary_digest,
        approval_ref: observed.semantic_summary_digest,
    };
    assert_eq!(
        observation_as_authority.authorize().unwrap_err(),
        DesktopBackendError::AuthorityBindingMismatch
    );

    let hint = pixel_hint();
    let mut semantic_with_pixel_hint = context(
        &semantic,
        &caps,
        &observed,
        &current_lease,
        &current_channel,
    );
    semantic_with_pixel_hint.pixel_hint = Some(&hint);
    assert_eq!(
        semantic_with_pixel_hint.authorize().unwrap_err(),
        DesktopBackendError::UnexpectedFallbackEvidence
    );

    let mut pixel_only_fallback = action(DesktopActionKind::RawInputFallback);
    pixel_only_fallback.pixel_hint_digest = Some(hint.binding_digest().unwrap());
    assert_eq!(
        pixel_only_fallback.validate().unwrap_err(),
        DesktopIntentError::MissingFallbackEligibility
    );
}

#[test]
fn native_observation_source_has_no_authority_or_actuation_surface() {
    let source = include_str!("../src/desktop_observation.rs");
    for forbidden in [
        "AuthorityBindings",
        "FallbackEligibilityEvidence",
        "PreparedDesktopAction",
        "ClipboardIntent",
        "enigo::",
        "arboard::",
        "x11_clipboard",
        "SendInput",
    ] {
        assert!(
            !source.contains(forbidden),
            "observation adapter must not contain authority or actuation surface: {forbidden}"
        );
    }
}
