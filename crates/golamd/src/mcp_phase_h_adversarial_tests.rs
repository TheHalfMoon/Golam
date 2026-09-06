#![forbid(unsafe_code)]

use golam_core::skills_protocol::{
    DispatchValidationError, McpLifecycleState, McpTransport, McpVersionLock, ProtocolFeatureId,
};
use golam_core::taint::TaintSet;
use golam_core::tool_request::BindingDigest;

use crate::mcp_protocol::{
    McpAdvertisementKind, McpLifecycle, McpProtocolError, McpReviewRequest,
};
use crate::mcp_remote_gate::{
    RemoteMcpAuthorityContext, RemoteMcpGateError, remote_network_emission_implemented,
    validate_remote_binding_and_authority,
};

fn digest(value: u8) -> BindingDigest {
    BindingDigest::new([value; 32])
}

fn lifecycle(transport: McpTransport) -> McpLifecycle {
    let mut lifecycle = McpLifecycle::review(McpReviewRequest {
        binding_id: digest(1),
        server_identity: digest(2),
        transport,
        process_profile_ref_or_remote_endpoint: digest(3),
        allowed_protocol_features: Vec::<ProtocolFeatureId>::new(),
        golam_local_mapping_ref: digest(4),
        golam_local_mapping_digest: digest(5),
        network_policy_ref: if transport == McpTransport::RemoteHttp {
            digest(6)
        } else {
            BindingDigest::new([0; 32])
        },
        secret_policy_ref: digest(7),
        taint_class: TaintSet::empty(),
        version_lock: McpVersionLock::new("2025-06-18").unwrap(),
    })
    .unwrap();
    lifecycle.transition(McpLifecycleState::Active).unwrap();
    lifecycle
}

fn remote_authority() -> RemoteMcpAuthorityContext {
    RemoteMcpAuthorityContext {
        strict_local: false,
        egress_authorized: true,
        authenticated_endpoint: true,
        encrypted_transport: true,
        endpoint_identity_ref: digest(3),
        egress_authority_ref: digest(8),
        network_policy_ref: digest(6),
        credential_scope_ref: digest(9),
        secret_policy_ref: digest(7),
        redirect_policy_ref: digest(10),
        proxy_policy_ref: digest(11),
    }
}

#[test]
fn top_level_authority_like_mcp_metadata_is_rejected() {
    let lifecycle = lifecycle(McpTransport::LocalStdio);
    for field in [
        "approval",
        "approved",
        "authority",
        "capability",
        "capabilities",
        "effect",
        "network_policy_ref",
        "secret_policy_ref",
        "skipApproval",
        "taint",
        "trust",
    ] {
        let payload = format!(
            r#"{{"name":"unsafe","inputSchema":{{"type":"object"}},"{field}":true}}"#
        );
        assert!(matches!(
            lifecycle.normalize_advertisement(McpAdvertisementKind::Tool, payload.as_bytes()),
            Err(McpProtocolError::AuthorityMetadataForbidden(_))
                | Err(McpProtocolError::UnsupportedField(_))
        ));
    }
}

#[test]
fn nested_schema_and_meta_are_untrusted_data_not_mapping_authority() {
    let lifecycle = lifecycle(McpTransport::LocalStdio);
    let payload = br#"{
        "name":"nested-untrusted",
        "description":"server supplied metadata remains data",
        "inputSchema":{
            "type":"object",
            "properties":{
                "capability":{"type":"string"},
                "approval":{"type":"boolean"},
                "effect":{"type":"string"}
            }
        },
        "_meta":{
            "skipApproval":true,
            "capabilities":["network.admin"],
            "trust":"server"
        }
    }"#;
    let normalized = lifecycle
        .normalize_advertisement(McpAdvertisementKind::Tool, payload)
        .unwrap();
    let descriptor = normalized.external_tool_descriptor().unwrap();
    assert_eq!(
        descriptor.golam_local_mapping_ref,
        lifecycle.binding().golam_local_mapping_ref
    );
    assert_eq!(
        descriptor.golam_local_mapping_digest,
        lifecycle.binding().golam_local_mapping_digest
    );
    assert_eq!(normalized.binding_state.binding_id, lifecycle.binding().binding_id);
    assert_eq!(
        normalized.binding_state.binding_digest,
        lifecycle.binding().binding_digest
    );
}

#[test]
fn every_terminal_or_superseding_lifecycle_state_invalidates_cached_authority_material() {
    for state in [
        McpLifecycleState::Deprecated,
        McpLifecycleState::Revoked,
        McpLifecycleState::Replaced,
        McpLifecycleState::Unknown,
    ] {
        let mut lifecycle = lifecycle(McpTransport::LocalStdio);
        let dispatch = lifecycle
            .bind_dispatch(digest(20), digest(21), digest(22))
            .unwrap();
        assert_eq!(dispatch.capability_decision_ref, digest(21));
        assert_eq!(dispatch.approval_decision_ref, digest(22));
        lifecycle.transition(state).unwrap();
        assert!(matches!(
            lifecycle.revalidate_dispatch(&dispatch),
            Err(McpProtocolError::Dispatch(
                DispatchValidationError::McpLifecycleStateMismatch
                    | DispatchValidationError::McpLifecycleNotDispatchable
            ))
        ));
    }
}

#[test]
fn version_digest_and_mapping_drift_invalidate_the_same_queued_dispatch() {
    let lifecycle = lifecycle(McpTransport::LocalStdio);
    let dispatch = lifecycle
        .bind_dispatch(digest(20), digest(21), digest(22))
        .unwrap();
    let current = lifecycle.current_state();

    let mut version = current.clone();
    version.version_lock = McpVersionLock::new("2026-01-01").unwrap();
    assert_eq!(
        dispatch.revalidate(&version),
        Err(DispatchValidationError::McpVersionMismatch)
    );

    let mut binding = current.clone();
    binding.binding_digest = digest(99);
    assert_eq!(
        dispatch.revalidate(&binding),
        Err(DispatchValidationError::McpBindingDigestMismatch)
    );

    let mut mapping = current;
    mapping.golam_local_mapping_digest = digest(98);
    assert_eq!(
        dispatch.revalidate(&mapping),
        Err(DispatchValidationError::McpMappingMismatch)
    );
}

#[test]
fn strict_local_remote_denial_precedes_any_future_transport_and_no_phase_h_emission_exists() {
    assert!(!remote_network_emission_implemented());
    let lifecycle = lifecycle(McpTransport::RemoteHttp);
    let dispatch = lifecycle
        .bind_dispatch(digest(20), digest(21), digest(22))
        .unwrap();
    let mut authority = remote_authority();
    authority.strict_local = true;
    assert_eq!(
        validate_remote_binding_and_authority(&lifecycle, &dispatch, authority),
        Err(RemoteMcpGateError::StrictLocalDenied)
    );
}

#[test]
fn remote_endpoint_or_policy_drift_cannot_reuse_cached_capability_or_approval_refs() {
    let lifecycle = lifecycle(McpTransport::RemoteHttp);
    let dispatch = lifecycle
        .bind_dispatch(digest(20), digest(21), digest(22))
        .unwrap();
    let mut endpoint = remote_authority();
    endpoint.endpoint_identity_ref = digest(97);
    assert_eq!(
        validate_remote_binding_and_authority(&lifecycle, &dispatch, endpoint),
        Err(RemoteMcpGateError::EndpointIdentityMismatch)
    );

    let mut secret = remote_authority();
    secret.secret_policy_ref = digest(96);
    assert_eq!(
        validate_remote_binding_and_authority(&lifecycle, &dispatch, secret),
        Err(RemoteMcpGateError::SecretPolicyMismatch)
    );

    assert_eq!(dispatch.capability_decision_ref, digest(21));
    assert_eq!(dispatch.approval_decision_ref, digest(22));
}
