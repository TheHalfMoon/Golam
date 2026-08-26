#![forbid(unsafe_code)]

use golam_core::ClientId;
use golam_kernel::{
    AuthorizationContext, AuthorizationDecision, AuthorizationPolicy, AuthorizationRequest,
    BootstrapPolicy, Principal,
};

fn decision(action: &str) -> AuthorizationDecision {
    BootstrapPolicy::default()
        .authorize(&AuthorizationRequest {
            principal: Principal::enrolled_client("local-cli", ClientId(7)),
            action,
            resource: "test:resource",
            context: AuthorizationContext::local("local-cli"),
        })
        .decision
}

#[test]
fn enrolled_cli_can_use_required_checkpoint_and_reconciliation_operations() {
    assert_eq!(decision("checkpoint.create"), AuthorizationDecision::Allow);
    assert_eq!(decision("checkpoint.verify"), AuthorizationDecision::Allow);
    assert_eq!(decision("effect.simulate"), AuthorizationDecision::Allow);
    assert_eq!(decision("effect.reconcile"), AuthorizationDecision::Allow);
}

#[test]
fn enrolled_cli_cannot_expand_client_or_network_authority() {
    assert_eq!(decision("client.enroll"), AuthorizationDecision::Deny);
    assert_eq!(decision("client.revoke"), AuthorizationDecision::Deny);
    assert_eq!(decision("network.egress"), AuthorizationDecision::Deny);
}
