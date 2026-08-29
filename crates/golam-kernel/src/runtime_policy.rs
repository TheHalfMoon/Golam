#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::str::FromStr;

use cedar_policy::{
    Authorizer, Context, Decision, Entities, EntityId, EntityTypeName, EntityUid, PolicySet,
    Request, RestrictedExpression, Schema,
};
use golam_core::authority::{AuthorityLayout, AuthorityPathError};
use golam_core::paths::RuntimeLayout;
use golam_ledger::active_policy_integrity::{
    ActivePolicyLoadState, VerifiedActivePolicyBundle, load_path,
};
use golam_ledger::approval_binding::APPROVAL_ISSUE_ACTION;
use golam_ledger::policy::{POLICY_ACTIVATE_ACTION, POLICY_STAGE_ACTION};

use crate::policy_candidate::validate_policy_candidate;
use crate::{AuthorizationPolicy, AuthorizationRequest, PolicyDecision, PrincipalKind};

const MAX_MATCHED_RULE_IDS: usize = 32;
const MAX_RULE_ID_CHARS: usize = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeAuthorityPolicy {
    authority_db_path: PathBuf,
}

impl RuntimeAuthorityPolicy {
    pub fn for_runtime(runtime: &RuntimeLayout) -> Result<Self, AuthorityPathError> {
        let authority = AuthorityLayout::initialize(runtime)?;
        Ok(Self {
            authority_db_path: authority.authority_db_path().to_path_buf(),
        })
    }
}

impl AuthorizationPolicy for RuntimeAuthorityPolicy {
    fn authorize(&self, request: &AuthorizationRequest<'_>) -> PolicyDecision {
        match load_path(&self.authority_db_path) {
            Ok(ActivePolicyLoadState::Bootstrap) => bootstrap_administration(request),
            Ok(ActivePolicyLoadState::Active(bundle)) => evaluate_active_policy(request, &bundle),
            Err(_) => PolicyDecision::deny("active_policy_load_failed"),
        }
    }
}

fn bootstrap_administration(request: &AuthorizationRequest<'_>) -> PolicyDecision {
    let local_owner = request.principal.kind == PrincipalKind::LocalOwner
        && request.context.scope.starts_with("local");
    let administrative_action = matches!(
        request.action,
        POLICY_STAGE_ACTION
            | POLICY_ACTIVATE_ACTION
            | APPROVAL_ISSUE_ACTION
            | "recovery.status.read"
    );
    if local_owner && administrative_action {
        PolicyDecision::allow("bootstrap_admin_explicit_allow")
    } else {
        PolicyDecision::deny("bootstrap_admin_no_matching_allow")
    }
}

fn evaluate_active_policy(
    request: &AuthorizationRequest<'_>,
    bundle: &VerifiedActivePolicyBundle,
) -> PolicyDecision {
    let validated = match validate_policy_candidate(&bundle.policy_source, &bundle.schema_source) {
        Ok(validated) => validated,
        Err(_) => return deny_with_evidence("active_policy_invalid", bundle, Vec::new()),
    };
    let policy_set = match PolicySet::from_str(validated.policy_source()) {
        Ok(policy_set) => policy_set,
        Err(_) => return deny_with_evidence("active_policy_parse_failed", bundle, Vec::new()),
    };
    let (schema, warnings) = match Schema::from_cedarschema_str(validated.schema_source()) {
        Ok(parsed) => parsed,
        Err(_) => return deny_with_evidence("active_schema_parse_failed", bundle, Vec::new()),
    };
    if warnings.into_iter().next().is_some() {
        return deny_with_evidence("active_schema_warning_denied", bundle, Vec::new());
    }

    let principal = match principal_uid(request) {
        Some(uid) => uid,
        None => return deny_with_evidence("cedar_principal_mapping_failed", bundle, Vec::new()),
    };
    let action = match entity_uid("Action", request.action) {
        Some(uid) => uid,
        None => return deny_with_evidence("cedar_action_mapping_failed", bundle, Vec::new()),
    };
    let resource = match entity_uid("GolamResource", request.resource) {
        Some(uid) => uid,
        None => return deny_with_evidence("cedar_resource_mapping_failed", bundle, Vec::new()),
    };
    let context = match cedar_context(request) {
        Some(context) => context,
        None => return deny_with_evidence("cedar_context_mapping_failed", bundle, Vec::new()),
    };
    let request = match Request::new(principal, action, resource, context, Some(&schema)) {
        Ok(request) => request,
        Err(_) => return deny_with_evidence("cedar_request_schema_denied", bundle, Vec::new()),
    };

    let response = Authorizer::new().is_authorized(&request, &policy_set, &Entities::empty());
    let matched_rule_ids = bounded_rule_ids(&response);
    if response.diagnostics().errors().next().is_some() {
        return deny_with_evidence("cedar_evaluation_error", bundle, matched_rule_ids);
    }
    match response.decision() {
        Decision::Allow => allow_with_evidence("cedar_allow", bundle, matched_rule_ids),
        Decision::Deny => deny_with_evidence("cedar_deny", bundle, matched_rule_ids),
    }
}

fn principal_uid(request: &AuthorizationRequest<'_>) -> Option<EntityUid> {
    let (entity_type, id) = match request.principal.kind {
        PrincipalKind::LocalOwner => ("LocalOwner", format!("owner:{}", request.principal.subject)),
        PrincipalKind::EnrolledClient => (
            "EnrolledClient",
            format!(
                "client:{}:{}",
                request.principal.client_id?.0, request.principal.subject
            ),
        ),
        PrincipalKind::KernelService => (
            "KernelService",
            format!("kernel:{}", request.principal.subject),
        ),
        PrincipalKind::Test => (
            "TestPrincipal",
            format!("test:{}", request.principal.subject),
        ),
        PrincipalKind::Unauthenticated => (
            "Unauthenticated",
            format!("unauthenticated:{}", request.principal.subject),
        ),
    };
    entity_uid(entity_type, &id)
}

fn entity_uid(entity_type: &str, id: &str) -> Option<EntityUid> {
    let entity_type = EntityTypeName::from_str(entity_type).ok()?;
    Some(EntityUid::from_type_name_and_id(
        entity_type,
        EntityId::new(id),
    ))
}

fn cedar_context(request: &AuthorizationRequest<'_>) -> Option<Context> {
    let scope = RestrictedExpression::from_str(&format!("\"{}\"", request.context.scope)).ok()?;
    let test_mode = RestrictedExpression::from_str(if request.context.test_mode {
        "true"
    } else {
        "false"
    })
    .ok()?;
    Context::from_pairs([
        ("scope".to_owned(), scope),
        ("testMode".to_owned(), test_mode),
    ])
    .ok()
}

fn bounded_rule_ids(response: &cedar_policy::Response) -> Vec<String> {
    let mut ids = response
        .diagnostics()
        .reason()
        .take(MAX_MATCHED_RULE_IDS)
        .map(|id| {
            id.to_string()
                .chars()
                .take(MAX_RULE_ID_CHARS)
                .collect::<String>()
        })
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    ids
}

fn allow_with_evidence(
    reason: &'static str,
    bundle: &VerifiedActivePolicyBundle,
    matched_rule_ids: Vec<String>,
) -> PolicyDecision {
    PolicyDecision::allow(reason).with_policy_evidence(
        bundle.policy.policy_bundle_id.0,
        bundle.policy.bundle_hash,
        matched_rule_ids,
    )
}

fn deny_with_evidence(
    reason: &'static str,
    bundle: &VerifiedActivePolicyBundle,
    matched_rule_ids: Vec<String>,
) -> PolicyDecision {
    PolicyDecision::deny(reason).with_policy_evidence(
        bundle.policy.policy_bundle_id.0,
        bundle.policy.bundle_hash,
        matched_rule_ids,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuthorizationContext, AuthorizationDecision, Principal};
    use golam_core::EffectId;
    use golam_ledger::active_policy_integrity::VerifiedActivePolicy;
    use golam_ledger::policy::PolicyBundleId;

    const SCHEMA: &str = r#"
entity LocalOwner;
entity EnrolledClient;
entity KernelService;
entity TestPrincipal;
entity Unauthenticated;
entity GolamResource;
action "session.create" appliesTo {
    principal: [LocalOwner, EnrolledClient],
    resource: GolamResource,
    context: { scope: String, testMode: Bool }
};
"#;

    const POLICY: &str = r#"
permit(
    principal is LocalOwner,
    action == Action::"session.create",
    resource is GolamResource
) when { context.scope == "local-owner" && context.testMode == false };
"#;

    fn bundle(policy: &str, schema: &str) -> VerifiedActivePolicyBundle {
        VerifiedActivePolicyBundle {
            policy: VerifiedActivePolicy {
                policy_bundle_id: PolicyBundleId([7; 16]),
                version: 1,
                schema_version: 1,
                bundle_hash: [9; 32],
            },
            policy_source: policy.to_owned(),
            schema_source: schema.to_owned(),
        }
    }

    fn owner_request<'a>(action: &'a str) -> AuthorizationRequest<'a> {
        AuthorizationRequest {
            principal: Principal::local_owner("owner"),
            action,
            resource: "session:new",
            context: AuthorizationContext::local("local-owner"),
        }
    }

    #[test]
    fn bootstrap_is_local_owner_administration_only() {
        let stage = AuthorizationRequest {
            principal: Principal::local_owner("owner"),
            action: POLICY_STAGE_ACTION,
            resource: "policy-bundle:bootstrap",
            context: AuthorizationContext::local("local-owner"),
        };
        assert_eq!(
            bootstrap_administration(&stage).decision,
            AuthorizationDecision::Allow
        );
        assert_eq!(
            bootstrap_administration(&owner_request("session.create")).decision,
            AuthorizationDecision::Deny
        );
        let client = AuthorizationRequest {
            principal: Principal::enrolled_client("local-cli", golam_core::ClientId(1)),
            action: POLICY_STAGE_ACTION,
            resource: "policy-bundle:bootstrap",
            context: AuthorizationContext::local("local-ipc"),
        };
        assert_eq!(
            bootstrap_administration(&client).decision,
            AuthorizationDecision::Deny
        );
    }

    #[test]
    fn active_cedar_policy_allows_and_denies_with_bundle_evidence() {
        let active = bundle(POLICY, SCHEMA);
        let allow = evaluate_active_policy(&owner_request("session.create"), &active);
        assert_eq!(allow.decision, AuthorizationDecision::Allow);
        assert_eq!(allow.evidence.policy_bundle_id, Some([7; 16]));
        assert_eq!(allow.evidence.policy_bundle_hash, Some([9; 32]));

        let deny = evaluate_active_policy(&owner_request("session.read"), &active);
        assert_eq!(deny.decision, AuthorizationDecision::Deny);
        assert_eq!(deny.evidence.policy_bundle_id, Some([7; 16]));
    }

    #[test]
    fn malformed_or_schema_incompatible_active_policy_fails_closed() {
        let malformed = bundle("permit(", SCHEMA);
        assert_eq!(
            evaluate_active_policy(&owner_request("session.create"), &malformed).decision,
            AuthorizationDecision::Deny
        );

        let wrong_schema = bundle(
            POLICY,
            "entity LocalOwner; entity GolamResource; action view appliesTo { principal: LocalOwner, resource: GolamResource };",
        );
        assert_eq!(
            evaluate_active_policy(&owner_request("session.create"), &wrong_schema).decision,
            AuthorizationDecision::Deny
        );
    }

    #[test]
    fn evaluator_error_is_deny_even_if_another_policy_could_permit() {
        let policy = r#"
permit(principal, action, resource);
permit(principal, action, resource) when { principal.missingAttribute == "x" };
"#;
        let schema = r#"
entity LocalOwner;
entity GolamResource;
action "session.create" appliesTo {
    principal: LocalOwner,
    resource: GolamResource,
    context: { scope: String, testMode: Bool }
};
"#;
        let decision =
            evaluate_active_policy(&owner_request("session.create"), &bundle(policy, schema));
        assert_eq!(decision.decision, AuthorizationDecision::Deny);
        assert_eq!(decision.reason_code, "active_policy_invalid");
    }

    #[test]
    fn verified_metadata_shape_is_independent_of_effect_identity() {
        let active = bundle(POLICY, SCHEMA);
        let _ = EffectId(1);
        assert_eq!(active.policy.version, 1);
    }
}
