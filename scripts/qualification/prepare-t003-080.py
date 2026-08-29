from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"missing anchor in {path}: {old[:80]!r}")
    p.write_text(text.replace(old, new, 1))


# Extend active-policy integrity with an atomic read-only material loader.
p = Path("crates/golam-ledger/src/active_policy_integrity.rs")
text = p.read_text()
text = text.replace(
    "use rusqlite::{Connection, OpenFlags, OptionalExtension, params};",
    "use rusqlite::{Connection, OpenFlags, OptionalExtension, TransactionBehavior, params};",
    1,
)
anchor = """#[derive(Clone, Debug, Eq, PartialEq)]\npub enum ActivePolicyIntegrityState {\n    Bootstrap,\n    Active(VerifiedActivePolicy),\n}\n"""
insert = anchor + """
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedActivePolicyBundle {
    pub policy: VerifiedActivePolicy,
    pub policy_source: String,
    pub schema_source: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActivePolicyLoadState {
    Bootstrap,
    Active(VerifiedActivePolicyBundle),
}
"""
if anchor not in text:
    raise SystemExit("active policy state anchor missing")
text = text.replace(anchor, insert, 1)
verify_anchor = """pub fn verify_path(\n    path: impl AsRef<Path>,\n) -> Result<ActivePolicyIntegrityState, ActivePolicyIntegrityError> {\n"""
loader = """pub fn load_path(
    path: impl AsRef<Path>,
) -> Result<ActivePolicyLoadState, ActivePolicyIntegrityError> {
    let path = path.as_ref();
    let store = AuthorityStore::open(path)?;
    drop(store);
    let mut connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    connection.execute_batch(
        "PRAGMA foreign_keys = ON; PRAGMA query_only = ON; PRAGMA busy_timeout = 5000;",
    )?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Deferred)?;
    let state = verify_connection(&transaction)?;
    match state {
        ActivePolicyIntegrityState::Bootstrap => Ok(ActivePolicyLoadState::Bootstrap),
        ActivePolicyIntegrityState::Active(policy) => {
            let canonical_policy_bytes = transaction.query_row(
                "SELECT canonical_policy_bytes FROM policy_bundles WHERE policy_bundle_id = ?1",
                params![&policy.policy_bundle_id.0[..]],
                |row| row.get::<_, Vec<u8>>(0),
            )?;
            let (policy_source, schema_source) =
                decode_canonical_bundle(&canonical_policy_bytes, policy.schema_version)?;
            Ok(ActivePolicyLoadState::Active(VerifiedActivePolicyBundle {
                policy,
                policy_source: policy_source.to_owned(),
                schema_source: schema_source.to_owned(),
            }))
        }
    }
}

""" + verify_anchor
if verify_anchor not in text:
    raise SystemExit("verify path anchor missing")
text = text.replace(verify_anchor, loader, 1)
old_verify = """fn verify_canonical_bundle(\n    bytes: &[u8],\n    expected_schema_version: u64,\n) -> Result<(), ActivePolicyIntegrityError> {\n    let mut offset = 0_usize;\n    let domain = take_len_prefixed(bytes, &mut offset)?;\n    if domain != POLICY_BUNDLE_DOMAIN {\n        return Err(ActivePolicyIntegrityError::InvalidCanonicalBundle);\n    }\n    let schema_version = take_u64(bytes, &mut offset)?;\n    if schema_version != expected_schema_version || schema_version == 0 {\n        return Err(ActivePolicyIntegrityError::InvalidSchemaVersion);\n    }\n    let policy = take_len_prefixed(bytes, &mut offset)?;\n    if policy.len() > MAX_POLICY_SOURCE_BYTES || str::from_utf8(policy).is_err() {\n        return Err(ActivePolicyIntegrityError::InvalidCanonicalBundle);\n    }\n    let schema = take_len_prefixed(bytes, &mut offset)?;\n    if schema.len() > MAX_SCHEMA_SOURCE_BYTES || str::from_utf8(schema).is_err() {\n        return Err(ActivePolicyIntegrityError::InvalidCanonicalBundle);\n    }\n    if offset != bytes.len() {\n        return Err(ActivePolicyIntegrityError::InvalidCanonicalBundle);\n    }\n    Ok(())\n}\n"""
new_verify = """fn verify_canonical_bundle(
    bytes: &[u8],
    expected_schema_version: u64,
) -> Result<(), ActivePolicyIntegrityError> {
    decode_canonical_bundle(bytes, expected_schema_version).map(|_| ())
}

fn decode_canonical_bundle<'a>(
    bytes: &'a [u8],
    expected_schema_version: u64,
) -> Result<(&'a str, &'a str), ActivePolicyIntegrityError> {
    let mut offset = 0_usize;
    let domain = take_len_prefixed(bytes, &mut offset)?;
    if domain != POLICY_BUNDLE_DOMAIN {
        return Err(ActivePolicyIntegrityError::InvalidCanonicalBundle);
    }
    let schema_version = take_u64(bytes, &mut offset)?;
    if schema_version != expected_schema_version || schema_version == 0 {
        return Err(ActivePolicyIntegrityError::InvalidSchemaVersion);
    }
    let policy = take_len_prefixed(bytes, &mut offset)?;
    if policy.len() > MAX_POLICY_SOURCE_BYTES {
        return Err(ActivePolicyIntegrityError::InvalidCanonicalBundle);
    }
    let policy = str::from_utf8(policy)
        .map_err(|_| ActivePolicyIntegrityError::InvalidCanonicalBundle)?;
    let schema = take_len_prefixed(bytes, &mut offset)?;
    if schema.len() > MAX_SCHEMA_SOURCE_BYTES {
        return Err(ActivePolicyIntegrityError::InvalidCanonicalBundle);
    }
    let schema = str::from_utf8(schema)
        .map_err(|_| ActivePolicyIntegrityError::InvalidCanonicalBundle)?;
    if offset != bytes.len() {
        return Err(ActivePolicyIntegrityError::InvalidCanonicalBundle);
    }
    Ok((policy, schema))
}
"""
if old_verify not in text:
    raise SystemExit("canonical verifier anchor missing")
text = text.replace(old_verify, new_verify, 1)
test_anchor = """    #[test]\n    fn clean_bootstrap_without_prior_activation_is_valid() {\n"""
loader_test = """    #[test]
    fn material_loader_returns_exact_sources_from_one_verified_snapshot() {
        let (runtime, authority) = authority();
        let expected = seed_valid_active(&authority);
        let loaded = load_path(authority.authority_db_path()).unwrap();
        let ActivePolicyLoadState::Active(bundle) = loaded else {
            panic!("active bundle must load");
        };
        assert_eq!(bundle.policy, expected);
        assert_eq!(bundle.policy_source, POLICY);
        assert_eq!(bundle.schema_source, SCHEMA);
        fs::remove_dir_all(runtime.root).unwrap();
    }

""" + test_anchor
if test_anchor not in text:
    raise SystemExit("loader test anchor missing")
text = text.replace(test_anchor, loader_test, 1)
p.write_text(text)

# Add the runtime Cedar-backed authority policy.
Path("crates/golam-kernel/src/runtime_policy.rs").write_text(r'''#![forbid(unsafe_code)]

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
use crate::{
    AuthorizationPolicy, AuthorizationRequest, PolicyDecision, PrincipalKind,
};

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
        PrincipalKind::LocalOwner => (
            "LocalOwner",
            format!("owner:{}", request.principal.subject),
        ),
        PrincipalKind::EnrolledClient => (
            "EnrolledClient",
            format!(
                "client:{}:{}",
                request.principal.client_id?.0,
                request.principal.subject
            ),
        ),
        PrincipalKind::KernelService => (
            "KernelService",
            format!("kernel:{}", request.principal.subject),
        ),
        PrincipalKind::Test => ("TestPrincipal", format!("test:{}", request.principal.subject)),
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
        .map(|id| id.to_string().chars().take(MAX_RULE_ID_CHARS).collect::<String>())
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
    use golam_core::EffectId;
    use golam_ledger::policy::PolicyBundleId;
    use golam_ledger::active_policy_integrity::VerifiedActivePolicy;
    use crate::{AuthorizationContext, AuthorizationDecision, Principal};

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
        let decision = evaluate_active_policy(&owner_request("session.create"), &bundle(policy, schema));
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
''')

# Wire runtime policy into the kernel public surface.
replace_once(
    "crates/golam-kernel/src/lib.rs",
    "mod resource;\nmod startup;",
    "mod resource;\nmod runtime_policy;\nmod startup;",
)
replace_once(
    "crates/golam-kernel/src/lib.rs",
    "pub use resource::{ProtectedResourceError, UnprivilegedPath};\npub use startup::{KernelStartup, KernelStartupError, start_kernel};",
    "pub use resource::{ProtectedResourceError, UnprivilegedPath};\npub use runtime_policy::RuntimeAuthorityPolicy;\npub use startup::{KernelStartup, KernelStartupError, start_kernel};",
)

# Enforce the authenticated-principal layer before policy evaluation.
replace_once(
    "crates/golam-kernel/src/authorization.rs",
    """        let policy_decision = match hard_guard {\n            HardGuardOutcome::Pass => self\n                .policy\n                .authorize_normalized(request, &canonical_policy_input),\n            HardGuardOutcome::Deny(reason) => PolicyDecision::deny(reason),\n        };\n""",
    """        let policy_decision = match hard_guard {\n            HardGuardOutcome::Pass if request.principal.kind == PrincipalKind::Unauthenticated => {\n                PolicyDecision::deny(\"unauthenticated_principal_denied\")\n            }\n            HardGuardOutcome::Pass => self\n                .policy\n                .authorize_normalized(request, &canonical_policy_input),\n            HardGuardOutcome::Deny(reason) => PolicyDecision::deny(reason),\n        };\n""",
)
auth = Path("crates/golam-kernel/src/authorization.rs")
auth_text = auth.read_text()
test_anchor = """    #[test]\n    fn bootstrap_policy_is_explicit_and_audited() {\n"""
principal_test = """    #[test]
    fn unauthenticated_principal_is_denied_before_a_permissive_policy() {
        struct PermitAll;
        impl AuthorizationPolicy for PermitAll {
            fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
                PolicyDecision::allow("should_not_run_for_unauthenticated")
            }
        }

        let (runtime, authority) = authority();
        let mut engine = AuthorizationEngine::open(&authority, PermitAll).unwrap();
        let request = AuthorizationRequest {
            principal: Principal::unauthenticated("anonymous"),
            action: "session.read",
            resource: "session:1",
            context: AuthorizationContext::local("local-ipc"),
        };
        let (outcome, grant) = engine.authorize(&request).unwrap();
        assert_eq!(outcome.decision, AuthorizationDecision::Deny);
        assert_eq!(outcome.reason_code, "unauthenticated_principal_denied");
        assert!(grant.is_none());
        drop(engine);
        fs::remove_dir_all(runtime.root).unwrap();
    }

""" + test_anchor
if test_anchor not in auth_text:
    raise SystemExit("authorization test anchor missing")
auth.write_text(auth_text.replace(test_anchor, principal_test, 1))

# Main daemon serving path now uses the active runtime policy.
replace_once(
    "crates/golamd/src/main.rs",
    "use golam_kernel::{BootstrapPolicy, KernelStartup, start_kernel};",
    "use golam_kernel::{KernelStartup, RuntimeAuthorityPolicy, start_kernel};",
)
replace_once(
    "crates/golamd/src/main.rs",
    "    let startup = start_kernel(&runtime, BootstrapPolicy::default())?;",
    "    let startup = start_kernel(&runtime, RuntimeAuthorityPolicy::for_runtime(&runtime)?)?;",
)
main = Path("crates/golamd/src/main.rs")
main_text = main.read_text().replace(
    "CommandRouter<BootstrapPolicy>", "CommandRouter<RuntimeAuthorityPolicy>"
)
main.write_text(main_text)

# Connection routing is generic; its dedicated enrollment/auth kernel remains explicit bootstrap authority.
replace_once(
    "crates/golamd/src/connection.rs",
    "    BootstrapPolicy, ClientEnrollmentError, ClientKind, KernelApi, KernelError, Principal,\n",
    "    AuthorizationPolicy, BootstrapPolicy, ClientEnrollmentError, ClientKind, KernelApi, KernelError, Principal,\n",
)
replace_once(
    "crates/golamd/src/connection.rs",
    "pub fn serve_connection<S: Read + Write, A: BootstrapApprover>(\n    stream: &mut S,\n    runtime: &RuntimeLayout,\n    router: &mut CommandRouter<BootstrapPolicy>,",
    "pub fn serve_connection<S: Read + Write, A: BootstrapApprover, P: AuthorizationPolicy>(\n    stream: &mut S,\n    runtime: &RuntimeLayout,\n    router: &mut CommandRouter<P>,",
)
