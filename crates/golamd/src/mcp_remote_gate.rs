#![forbid(unsafe_code)]

//! Fail-closed remote-MCP authority and freshness gate for Spec 005.
//!
//! Phase H deliberately implements no HTTP client and emits no network traffic. This module only
//! prepares and revalidates a non-authoritative remote-dispatch binding. A later transport, if
//! selected under T005-100, must pass this gate immediately before every send. Strict-local denial
//! dominates all other inputs. Endpoint authentication, encrypted transport, egress authority,
//! credential scope, network policy, secret policy, redirect policy and proxy policy are explicit
//! and immutable inputs to the prepared binding.

use std::error::Error;
use std::fmt;

use golam_core::skills_protocol::{McpDispatchBinding, McpTransport};
use golam_core::tool_request::{BindingDigest, PreparedToolRequest};

use crate::mcp_protocol::{McpLifecycle, McpProtocolError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RemoteMcpAuthorityContext {
    pub strict_local: bool,
    pub egress_authorized: bool,
    pub authenticated_endpoint: bool,
    pub encrypted_transport: bool,
    pub endpoint_identity_ref: BindingDigest,
    pub egress_authority_ref: BindingDigest,
    pub network_policy_ref: BindingDigest,
    pub credential_scope_ref: BindingDigest,
    pub secret_policy_ref: BindingDigest,
    pub redirect_policy_ref: BindingDigest,
    pub proxy_policy_ref: BindingDigest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedRemoteMcpDispatch {
    pub mcp_dispatch: McpDispatchBinding,
    pub request_binding_ref: BindingDigest,
    pub binding_id: BindingDigest,
    pub binding_digest: BindingDigest,
    pub endpoint_identity_ref: BindingDigest,
    pub egress_authority_ref: BindingDigest,
    pub network_policy_ref: BindingDigest,
    pub credential_scope_ref: BindingDigest,
    pub secret_policy_ref: BindingDigest,
    pub redirect_policy_ref: BindingDigest,
    pub proxy_policy_ref: BindingDigest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteMcpGateError {
    Protocol,
    InvalidTransport,
    StrictLocalDenied,
    EgressDenied,
    EndpointUnauthenticated,
    UnencryptedTransport,
    InvalidAuthorityReference(&'static str),
    EndpointIdentityMismatch,
    NetworkPolicyMismatch,
    SecretPolicyMismatch,
    StaleQueuedRequest,
    PreparedBindingMismatch,
}

impl fmt::Display for RemoteMcpGateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol => f.write_str("remote MCP binding revalidation failed"),
            Self::InvalidTransport => f.write_str("remote MCP gate requires RemoteHttp transport"),
            Self::StrictLocalDenied => {
                f.write_str("strict-local mode denies external remote MCP dispatch")
            }
            Self::EgressDenied => {
                f.write_str("remote MCP dispatch lacks explicit egress authority")
            }
            Self::EndpointUnauthenticated => {
                f.write_str("remote MCP endpoint identity is not authenticated")
            }
            Self::UnencryptedTransport => {
                f.write_str("remote MCP credential-bearing transport must be encrypted")
            }
            Self::InvalidAuthorityReference(field) => {
                write!(f, "remote MCP authority reference is missing: {field}")
            }
            Self::EndpointIdentityMismatch => {
                f.write_str("remote MCP endpoint identity differs from the reviewed binding")
            }
            Self::NetworkPolicyMismatch => {
                f.write_str("remote MCP network policy differs from the reviewed binding")
            }
            Self::SecretPolicyMismatch => {
                f.write_str("remote MCP secret policy differs from the reviewed binding")
            }
            Self::StaleQueuedRequest => {
                f.write_str("remote MCP queued request no longer matches the prepared ToolRequest")
            }
            Self::PreparedBindingMismatch => f.write_str(
                "remote MCP prepared dispatch no longer matches current binding or authority state",
            ),
        }
    }
}

impl Error for RemoteMcpGateError {}

impl From<McpProtocolError> for RemoteMcpGateError {
    fn from(_: McpProtocolError) -> Self {
        Self::Protocol
    }
}

pub const fn remote_network_emission_implemented() -> bool {
    false
}

pub fn prepare_remote_mcp_dispatch(
    lifecycle: &McpLifecycle,
    dispatch: &McpDispatchBinding,
    request: &PreparedToolRequest,
    authority: RemoteMcpAuthorityContext,
) -> Result<PreparedRemoteMcpDispatch, RemoteMcpGateError> {
    validate_remote_binding_and_authority(lifecycle, dispatch, authority)?;
    let request_binding_ref = BindingDigest::new(request.binding_digest());
    validate_queued_request(dispatch, request_binding_ref)?;
    let binding = lifecycle.binding();
    Ok(PreparedRemoteMcpDispatch {
        mcp_dispatch: dispatch.clone(),
        request_binding_ref,
        binding_id: binding.binding_id,
        binding_digest: binding.binding_digest,
        endpoint_identity_ref: authority.endpoint_identity_ref,
        egress_authority_ref: authority.egress_authority_ref,
        network_policy_ref: authority.network_policy_ref,
        credential_scope_ref: authority.credential_scope_ref,
        secret_policy_ref: authority.secret_policy_ref,
        redirect_policy_ref: authority.redirect_policy_ref,
        proxy_policy_ref: authority.proxy_policy_ref,
    })
}

pub fn revalidate_prepared_remote_mcp_dispatch(
    lifecycle: &McpLifecycle,
    dispatch: &McpDispatchBinding,
    request: &PreparedToolRequest,
    authority: RemoteMcpAuthorityContext,
    prepared: &PreparedRemoteMcpDispatch,
) -> Result<(), RemoteMcpGateError> {
    validate_remote_binding_and_authority(lifecycle, dispatch, authority)?;
    let request_binding_ref = BindingDigest::new(request.binding_digest());
    validate_queued_request(dispatch, request_binding_ref)?;
    let binding = lifecycle.binding();
    if &prepared.mcp_dispatch != dispatch
        || prepared.request_binding_ref != request_binding_ref
        || prepared.binding_id != binding.binding_id
        || prepared.binding_digest != binding.binding_digest
        || prepared.endpoint_identity_ref != authority.endpoint_identity_ref
        || prepared.egress_authority_ref != authority.egress_authority_ref
        || prepared.network_policy_ref != authority.network_policy_ref
        || prepared.credential_scope_ref != authority.credential_scope_ref
        || prepared.secret_policy_ref != authority.secret_policy_ref
        || prepared.redirect_policy_ref != authority.redirect_policy_ref
        || prepared.proxy_policy_ref != authority.proxy_policy_ref
    {
        return Err(RemoteMcpGateError::PreparedBindingMismatch);
    }
    Ok(())
}

pub fn validate_remote_binding_and_authority(
    lifecycle: &McpLifecycle,
    dispatch: &McpDispatchBinding,
    authority: RemoteMcpAuthorityContext,
) -> Result<(), RemoteMcpGateError> {
    lifecycle.revalidate_dispatch(dispatch)?;
    let binding = lifecycle.binding();
    if binding.transport != McpTransport::RemoteHttp {
        return Err(RemoteMcpGateError::InvalidTransport);
    }
    if authority.strict_local {
        return Err(RemoteMcpGateError::StrictLocalDenied);
    }
    if !authority.egress_authorized {
        return Err(RemoteMcpGateError::EgressDenied);
    }
    if !authority.authenticated_endpoint {
        return Err(RemoteMcpGateError::EndpointUnauthenticated);
    }
    if !authority.encrypted_transport {
        return Err(RemoteMcpGateError::UnencryptedTransport);
    }
    require_nonzero(authority.endpoint_identity_ref, "endpoint_identity_ref")?;
    require_nonzero(authority.egress_authority_ref, "egress_authority_ref")?;
    require_nonzero(authority.network_policy_ref, "network_policy_ref")?;
    require_nonzero(authority.credential_scope_ref, "credential_scope_ref")?;
    require_nonzero(authority.secret_policy_ref, "secret_policy_ref")?;
    require_nonzero(authority.redirect_policy_ref, "redirect_policy_ref")?;
    require_nonzero(authority.proxy_policy_ref, "proxy_policy_ref")?;
    if authority.endpoint_identity_ref != binding.process_profile_ref_or_remote_endpoint {
        return Err(RemoteMcpGateError::EndpointIdentityMismatch);
    }
    if authority.network_policy_ref != binding.network_policy_ref {
        return Err(RemoteMcpGateError::NetworkPolicyMismatch);
    }
    if authority.secret_policy_ref != binding.secret_policy_ref {
        return Err(RemoteMcpGateError::SecretPolicyMismatch);
    }
    Ok(())
}

fn validate_queued_request(
    dispatch: &McpDispatchBinding,
    request_binding_ref: BindingDigest,
) -> Result<(), RemoteMcpGateError> {
    if dispatch.queued_request_ref != request_binding_ref {
        return Err(RemoteMcpGateError::StaleQueuedRequest);
    }
    Ok(())
}

fn require_nonzero(value: BindingDigest, field: &'static str) -> Result<(), RemoteMcpGateError> {
    if value.bytes() == [0; 32] {
        return Err(RemoteMcpGateError::InvalidAuthorityReference(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp_protocol::{McpLifecycle, McpReviewRequest};
    use golam_core::skills_protocol::{McpLifecycleState, McpVersionLock, ProtocolFeatureId};
    use golam_core::taint::TaintSet;

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn remote_lifecycle() -> McpLifecycle {
        let mut lifecycle = McpLifecycle::review(McpReviewRequest {
            binding_id: digest(1),
            server_identity: digest(2),
            transport: McpTransport::RemoteHttp,
            process_profile_ref_or_remote_endpoint: digest(3),
            allowed_protocol_features: Vec::<ProtocolFeatureId>::new(),
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

    #[test]
    fn phase_h_emits_no_remote_network_traffic() {
        assert!(!remote_network_emission_implemented());
    }

    #[test]
    fn strict_local_denial_dominates_other_remote_authority() {
        let lifecycle = remote_lifecycle();
        let dispatch = lifecycle
            .bind_dispatch(digest(12), digest(13), digest(14))
            .unwrap();
        let mut context = authority();
        context.strict_local = true;
        assert_eq!(
            validate_remote_binding_and_authority(&lifecycle, &dispatch, context),
            Err(RemoteMcpGateError::StrictLocalDenied)
        );
    }

    #[test]
    fn remote_gate_requires_exact_endpoint_network_secret_and_credential_scope() {
        let lifecycle = remote_lifecycle();
        let dispatch = lifecycle
            .bind_dispatch(digest(12), digest(13), digest(14))
            .unwrap();
        assert_eq!(
            validate_remote_binding_and_authority(&lifecycle, &dispatch, authority()),
            Ok(())
        );

        let mut endpoint = authority();
        endpoint.endpoint_identity_ref = digest(99);
        assert_eq!(
            validate_remote_binding_and_authority(&lifecycle, &dispatch, endpoint),
            Err(RemoteMcpGateError::EndpointIdentityMismatch)
        );

        let mut network = authority();
        network.network_policy_ref = digest(99);
        assert_eq!(
            validate_remote_binding_and_authority(&lifecycle, &dispatch, network),
            Err(RemoteMcpGateError::NetworkPolicyMismatch)
        );

        let mut secret = authority();
        secret.secret_policy_ref = digest(99);
        assert_eq!(
            validate_remote_binding_and_authority(&lifecycle, &dispatch, secret),
            Err(RemoteMcpGateError::SecretPolicyMismatch)
        );

        let mut credential = authority();
        credential.credential_scope_ref = BindingDigest::new([0; 32]);
        assert_eq!(
            validate_remote_binding_and_authority(&lifecycle, &dispatch, credential),
            Err(RemoteMcpGateError::InvalidAuthorityReference(
                "credential_scope_ref"
            ))
        );
    }

    #[test]
    fn remote_gate_rejects_stale_lifecycle_and_local_transport() {
        let mut lifecycle = remote_lifecycle();
        let stale = lifecycle
            .bind_dispatch(digest(12), digest(13), digest(14))
            .unwrap();
        lifecycle.transition(McpLifecycleState::Revoked).unwrap();
        assert_eq!(
            validate_remote_binding_and_authority(&lifecycle, &stale, authority()),
            Err(RemoteMcpGateError::Protocol)
        );

        let mut local = McpLifecycle::review(McpReviewRequest {
            binding_id: digest(21),
            server_identity: digest(22),
            transport: McpTransport::LocalStdio,
            process_profile_ref_or_remote_endpoint: digest(23),
            allowed_protocol_features: Vec::new(),
            golam_local_mapping_ref: digest(24),
            golam_local_mapping_digest: digest(25),
            network_policy_ref: BindingDigest::new([0; 32]),
            secret_policy_ref: digest(26),
            taint_class: TaintSet::empty(),
            version_lock: McpVersionLock::new("2025-06-18").unwrap(),
        })
        .unwrap();
        local.transition(McpLifecycleState::Active).unwrap();
        let dispatch = local
            .bind_dispatch(digest(12), digest(13), digest(14))
            .unwrap();
        assert_eq!(
            validate_remote_binding_and_authority(&local, &dispatch, authority()),
            Err(RemoteMcpGateError::InvalidTransport)
        );
    }
}
