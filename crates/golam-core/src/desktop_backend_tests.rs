#![forbid(unsafe_code)]

use crate::EffectId;
use crate::desktop_backend::{
    ClipboardDispatchContext, DesktopActionDispatchContext, DesktopBackend, DesktopBackendError,
    FakeDesktopBackend,
};
use crate::desktop_control::{
    ControlRoute, DESKTOP_CONTROL_SCHEMA_VERSION, DesktopCapabilitySet, DesktopControlLeaseId,
    DesktopControlLeaseState, DesktopControlMode, DesktopLimits, DesktopObservation,
    DesktopObservationId, DesktopPlatform, DesktopSessionKind, FallbackEligibilityEvidence,
    RouteDisposition, RouteEvaluation, VisibleControlChannelId, VisibleControlChannelKind,
    VisibleControlChannelState,
};
use crate::desktop_intent::{
    AuthorityBindings, ClipboardIntent, ClipboardOperation, DesktopActionKind, EffectBinding,
    InteractiveAuthorityBinding, PreparedDesktopAction, RequestBinding,
};
use crate::tool_request::{BindingDigest, ToolRequestId};

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

fn action(kind: DesktopActionKind, fallback: Option<BindingDigest>) -> PreparedDesktopAction {
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
        fallback_eligibility_evidence_digest: fallback,
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
    action: &'a PreparedDesktopAction,
    caps: &'a DesktopCapabilitySet,
    observed: &'a DesktopObservation,
    current_lease: &'a DesktopControlLeaseState,
    current_channel: &'a VisibleControlChannelState,
    route: Option<&'a FallbackEligibilityEvidence>,
) -> DesktopActionDispatchContext<'a> {
    DesktopActionDispatchContext {
        action,
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
        fallback_evidence: route,
        pixel_hint: None,
        unresolved_conflicting_unknown_outcome: false,
    }
}

#[test]
fn semantic_focus_and_fake_dispatch_require_exact_safety_state() {
    let caps = capabilities();
    let observed = observation();
    let current_lease = lease();
    let current_channel = channel();
    for kind in [DesktopActionKind::SemanticAction, DesktopActionKind::Focus] {
        let prepared = action(kind, None);
        let validated = context(
            &prepared,
            &caps,
            &observed,
            &current_lease,
            &current_channel,
            None,
        )
        .authorize()
        .unwrap();
        let mut backend = FakeDesktopBackend::new(caps, observed.clone()).unwrap();
        assert!(backend.dispatch_action(validated).is_ok());
        assert_eq!(backend.dispatch_count(), 1);
    }
}

#[test]
fn request_gate_target_lease_and_visible_channel_drift_fail_closed() {
    let caps = capabilities();
    let observed = observation();
    let current_lease = lease();
    let current_channel = channel();
    let prepared = action(DesktopActionKind::SemanticAction, None);

    let mut request_drift = context(
        &prepared,
        &caps,
        &observed,
        &current_lease,
        &current_channel,
        None,
    );
    request_drift.current_request_digest = digest(99);
    assert_eq!(
        request_drift.authorize().unwrap_err(),
        DesktopBackendError::InvalidRequestBinding
    );

    let mut missing_gate = context(
        &prepared,
        &caps,
        &observed,
        &current_lease,
        &current_channel,
        None,
    );
    missing_gate.current_gate_authorization_digest = BindingDigest::new([0; 32]);
    assert_eq!(
        missing_gate.authorize().unwrap_err(),
        DesktopBackendError::MissingGateAuthorization
    );

    let mut stale_lease = current_lease;
    stale_lease.generation += 1;
    assert_eq!(
        context(
            &prepared,
            &caps,
            &observed,
            &stale_lease,
            &current_channel,
            None,
        )
        .authorize()
        .unwrap_err(),
        DesktopBackendError::StaleOrSupersededLease
    );

    let mut hidden = current_channel;
    hidden.visible = false;
    let hidden_context = context(&prepared, &caps, &observed, &current_lease, &hidden, None);
    assert_eq!(
        hidden_context.authorize().unwrap_err(),
        DesktopBackendError::AutonomousActuationSuspended
    );
}

#[test]
fn raw_fallback_requires_canonical_route_evidence_and_blocks_unknown_outcome() {
    let caps = capabilities();
    let observed = observation();
    let current_lease = lease();
    let current_channel = channel();
    let route = fallback();
    let prepared = action(
        DesktopActionKind::RawInputFallback,
        Some(route.binding_digest().unwrap()),
    );
    assert!(
        context(
            &prepared,
            &caps,
            &observed,
            &current_lease,
            &current_channel,
            Some(&route),
        )
        .authorize()
        .is_ok()
    );

    let mut blocked = context(
        &prepared,
        &caps,
        &observed,
        &current_lease,
        &current_channel,
        Some(&route),
    );
    blocked.unresolved_conflicting_unknown_outcome = true;
    assert_eq!(
        blocked.authorize().unwrap_err(),
        DesktopBackendError::UnknownOutcomeBlocksDispatch
    );

    let mut unsafe_route = route;
    unsafe_route.route_evaluations[1].disposition = RouteDisposition::UnknownOutcome;
    assert!(unsafe_route.validate().is_err());
}

#[test]
fn permission_loss_and_clipboard_denial_are_deterministic() {
    let caps = capabilities();
    let observed = observation();
    let mut backend = FakeDesktopBackend::new(caps, observed).unwrap();
    backend.set_permission_granted(false);
    assert_eq!(
        backend.observe(DesktopLimits::default()).unwrap_err(),
        DesktopBackendError::PermissionOrSessionDrift
    );

    let mut denied = caps;
    denied.clipboard_read_supported = false;
    let intent = ClipboardIntent {
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
    assert_eq!(
        ClipboardDispatchContext {
            intent: &intent,
            now_unix_ms: 110,
            current_request_digest: digest(21),
            current_effect_digest: digest(22),
            current_gate_authorization_digest: digest(23),
            current_authority: authority(),
            current_capabilities: &denied,
            unresolved_conflicting_unknown_outcome: false,
        }
        .authorize()
        .unwrap_err(),
        DesktopBackendError::ClipboardDenied
    );
}
