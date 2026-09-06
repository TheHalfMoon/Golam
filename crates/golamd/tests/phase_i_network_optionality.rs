#![forbid(unsafe_code)]

use golam_core::harness::ToolCallCandidateId;
use golam_core::skills_protocol::{McpLifecycleState, McpTransport, McpVersionLock};
use golam_core::taint::TaintSet;
use golam_core::tool_descriptor::{ToolId, ToolVersion};
use golam_core::tool_request::{
    BindingDigest, PreparedToolRequest, PrincipalId, RequestedOperationId, ResourceClassId,
    ToolRequest, ToolRequestId,
};
use golamd::mcp_protocol::{McpLifecycle, McpReviewRequest};
use golamd::mcp_remote_gate::{
    RemoteMcpAuthorityContext, RemoteMcpGateError, prepare_remote_mcp_dispatch,
    remote_network_emission_implemented, revalidate_prepared_remote_mcp_dispatch,
};

fn digest(value: u8) -> BindingDigest {
    BindingDigest::new([value; 32])
}

fn remote_lifecycle() -> McpLifecycle {
    let mut lifecycle = McpLifecycle::review(McpReviewRequest {
        binding_id: digest(1),
        server_identity: digest(2),
        transport: McpTransport::RemoteHttp,
        process_profile_ref_or_remote_endpoint: digest(3),
        allowed_protocol_features: Vec::new(),
        golam_local_mapping_ref: digest(4),
        golam_local_mapping_digest: digest(5),
        network_policy_ref: digest(6),
        secret_policy_ref: digest(7),
        taint_class: TaintSet::empty(),
        version_lock: McpVersionLock::new("2025-06-18").unwrap(),
    })
    .unwrap();
    lifecycle.transition(McpLifecycleState::Active).unwrap();
    lifecycle
}

fn authority() -> RemoteMcpAuthorityContext {
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

fn request() -> PreparedToolRequest {
    ToolRequest {
        request_id: ToolRequestId::from_u128(100),
        initiating_principal: PrincipalId::new("phase-i-principal").unwrap(),
        tool_id: ToolId::new("mcp-remote").unwrap(),
        tool_version: ToolVersion::new("1").unwrap(),
        candidate_ref: ToolCallCandidateId::from_u128(101),
        requested_operation: RequestedOperationId::new("remote-call").unwrap(),
        requested_target: None,
        authorized_resource_class: ResourceClassId::new("remote-mcp").unwrap(),
        target_identity_ref: None,
        target_resolution_plan_ref: None,
        capability_context_ref: digest(20),
        taint_set: TaintSet::empty(),
        provenance_refs: vec![digest(21)],
        idempotency_material: digest(22),
        current_preconditions: vec![digest(23)],
        created_at_unix_ms: 1_000,
    }
    .prepare()
    .unwrap()
}

#[test]
fn prepared_remote_dispatch_rejects_credential_redirect_proxy_and_authority_drift() {
    assert!(!remote_network_emission_implemented());

    let lifecycle = remote_lifecycle();
    let request = request();
    let dispatch = lifecycle
        .bind_dispatch(
            BindingDigest::new(request.binding_digest()),
            digest(30),
            digest(31),
        )
        .unwrap();
    let baseline = authority();
    let prepared = prepare_remote_mcp_dispatch(&lifecycle, &dispatch, &request, baseline).unwrap();

    assert_eq!(
        revalidate_prepared_remote_mcp_dispatch(
            &lifecycle, &dispatch, &request, baseline, &prepared,
        ),
        Ok(())
    );

    for mutate in [
        |context: &mut RemoteMcpAuthorityContext| context.credential_scope_ref = digest(40),
        |context: &mut RemoteMcpAuthorityContext| context.redirect_policy_ref = digest(41),
        |context: &mut RemoteMcpAuthorityContext| context.proxy_policy_ref = digest(42),
        |context: &mut RemoteMcpAuthorityContext| context.egress_authority_ref = digest(43),
    ] {
        let mut changed = baseline;
        mutate(&mut changed);
        assert_eq!(
            revalidate_prepared_remote_mcp_dispatch(
                &lifecycle, &dispatch, &request, changed, &prepared,
            ),
            Err(RemoteMcpGateError::PreparedBindingMismatch)
        );
    }

    let mut endpoint = baseline;
    endpoint.endpoint_identity_ref = digest(44);
    assert_eq!(
        revalidate_prepared_remote_mcp_dispatch(
            &lifecycle, &dispatch, &request, endpoint, &prepared,
        ),
        Err(RemoteMcpGateError::EndpointIdentityMismatch)
    );

    let mut network = baseline;
    network.network_policy_ref = digest(45);
    assert_eq!(
        revalidate_prepared_remote_mcp_dispatch(
            &lifecycle, &dispatch, &request, network, &prepared,
        ),
        Err(RemoteMcpGateError::NetworkPolicyMismatch)
    );

    let mut secret = baseline;
    secret.secret_policy_ref = digest(46);
    assert_eq!(
        revalidate_prepared_remote_mcp_dispatch(
            &lifecycle, &dispatch, &request, secret, &prepared,
        ),
        Err(RemoteMcpGateError::SecretPolicyMismatch)
    );
}

#[test]
fn remote_dispatch_denies_strict_local_downgrade_unauthenticated_and_missing_egress() {
    let lifecycle = remote_lifecycle();
    let request = request();
    let dispatch = lifecycle
        .bind_dispatch(
            BindingDigest::new(request.binding_digest()),
            digest(30),
            digest(31),
        )
        .unwrap();

    let mut strict_local = authority();
    strict_local.strict_local = true;
    assert_eq!(
        prepare_remote_mcp_dispatch(&lifecycle, &dispatch, &request, strict_local),
        Err(RemoteMcpGateError::StrictLocalDenied)
    );

    let mut downgraded = authority();
    downgraded.encrypted_transport = false;
    assert_eq!(
        prepare_remote_mcp_dispatch(&lifecycle, &dispatch, &request, downgraded),
        Err(RemoteMcpGateError::UnencryptedTransport)
    );

    let mut unauthenticated = authority();
    unauthenticated.authenticated_endpoint = false;
    assert_eq!(
        prepare_remote_mcp_dispatch(&lifecycle, &dispatch, &request, unauthenticated),
        Err(RemoteMcpGateError::EndpointUnauthenticated)
    );

    let mut no_egress = authority();
    no_egress.egress_authorized = false;
    assert_eq!(
        prepare_remote_mcp_dispatch(&lifecycle, &dispatch, &request, no_egress),
        Err(RemoteMcpGateError::EgressDenied)
    );
}
