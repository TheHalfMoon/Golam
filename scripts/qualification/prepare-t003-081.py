from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")


def replace_once(path: str, old: str, new: str) -> None:
    content = read(path)
    count = content.count(old)
    if count != 1:
        raise SystemExit(f"expected one anchor in {path}, found {count}: {old[:80]!r}")
    write(path, content.replace(old, new, 1))


# Ledger-only deterministic canary preparation. Plaintext never leaves this module.
write(
    "crates/golam-ledger/src/admin_qualification.rs",
    r'''#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::{EventId, SessionId};

use crate::secret_entry::{PrepareDesignatedSecretEntryRequest, prepare_designated_secret_entry};

const UNKNOWN_FORMAT_CANARY: &[u8] =
    b"golam-spec003-t003081-canary::opaque-unknown-format::f8d2e4c1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretCanaryQualificationError {
    PreparationFailed,
}

impl fmt::Display for SecretCanaryQualificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PreparationFailed => {
                f.write_str("deterministic secret-canary preparation failed closed")
            }
        }
    }
}

impl Error for SecretCanaryQualificationError {}

/// Exercises the explicit designated-secret preparation boundary with a fixed
/// unknown-format canary. The prepared value is never committed, returned,
/// logged or formatted; dropping it zeroizes the protected plaintext owner.
pub fn qualify_designated_secret_canary() -> Result<(), SecretCanaryQualificationError> {
    let prepared = prepare_designated_secret_entry(PrepareDesignatedSecretEntryRequest {
        session_id: SessionId(1),
        expected_session_seq: 0,
        event_id: EventId(1),
        actor_principal: "owner:qualification",
        owner_principal: "owner:qualification",
        recorded_at: "2026-08-29T00:00:00Z",
        classification: "qualification_canary",
        purpose_scope: "qualification-only",
        expires_at: None,
        value: UNKNOWN_FORMAT_CANARY.to_vec(),
    })
    .map_err(|_| SecretCanaryQualificationError::PreparationFailed)?;
    drop(prepared);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canary_qualification_uses_designated_secret_path_without_exposing_value() {
        qualify_designated_secret_canary().unwrap();
        let rendered = SecretCanaryQualificationError::PreparationFailed.to_string();
        assert!(!rendered.contains("f8d2e4c1"));
        assert!(!rendered.contains("opaque-unknown-format"));
    }
}
''',
)
replace_once(
    "crates/golam-ledger/src/lib.rs",
    "pub mod active_policy_integrity;\n",
    "pub mod active_policy_integrity;\npub mod admin_qualification;\n",
)

# Kernel-owned authenticated admin/qualification boundary.
write(
    "crates/golam-kernel/src/admin_qualification.rs",
    r'''#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::EffectId;
use golam_ledger::admin_qualification::{
    SecretCanaryQualificationError, qualify_designated_secret_canary,
};
use golam_ledger::approvals::{ApprovalScope, ApprovalScopeError};
use golam_ledger::sandbox_profile::{
    SandboxNetworkRule, SandboxProfileClass, SandboxProfileDefinition, SandboxProfileError,
    SandboxSpawnRule, prepare_sandbox_profile,
};

use crate::policy_candidate::{CandidatePolicyError, validate_policy_candidate};
use crate::{
    AuthorizationContext, AuthorizationError, AuthorizationOutcome, AuthorizationPolicy,
    AuthorizationRequest, CapabilityLeaseScope, CapabilityLeaseScopeError, DecisionId, KernelApi,
    Principal, PrincipalKind,
};

pub const POLICY_VALIDATE_ACTION: &str = "policy.validate";
pub const AUTHORITY_QUALIFY_ACTION: &str = "authority.qualify";
pub const AUTHORITY_EXPLAIN_ACTION: &str = "authority.explain";

const POLICY_VALIDATION_RESOURCE: &str = "policy-candidate:validation";
const QUALIFICATION_DOMAIN: &[u8] = b"golam:authority-qualification:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdminQualificationKind {
    Lease,
    Approval,
    SecretCanary,
    SandboxProfile,
}

impl AdminQualificationKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lease => "lease",
            Self::Approval => "approval",
            Self::SecretCanary => "secret-canary",
            Self::SandboxProfile => "sandbox-profile",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminQualificationReceipt {
    pub kind: &'static str,
    pub authorization_decision_id: [u8; 16],
    pub resource: String,
    pub evidence_digest: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyQualificationReceipt {
    pub authorization_decision_id: [u8; 16],
    pub policy_bytes: usize,
    pub schema_bytes: usize,
    pub evidence_digest: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationDecisionExplanation {
    pub decision_id: [u8; 16],
    pub principal: String,
    pub action: String,
    pub resource: String,
    pub context_hash: [u8; 32],
    pub hard_guard_result: String,
    pub lease_id: Option<[u8; 16]>,
    pub lease_generation: Option<u64>,
    pub policy_bundle_id: Option<[u8; 16]>,
    pub policy_bundle_hash: Option<[u8; 32]>,
    pub matched_rule_ids: Vec<String>,
    pub approval_id: Option<[u8; 16]>,
    pub decision: String,
    pub reason_code: String,
    pub global_seq: u64,
    pub authority_evidence_version: u64,
}

#[derive(Debug)]
pub enum AdminSurfaceError {
    Authorization(AuthorizationError),
    AuthorizationDenied(AuthorizationOutcome),
    Policy(CandidatePolicyError),
    Lease(CapabilityLeaseScopeError),
    Approval(ApprovalScopeError),
    SecretCanary(SecretCanaryQualificationError),
    Sandbox(SandboxProfileError),
    DecisionNotFound,
}

impl fmt::Display for AdminSurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Authorization(error) => write!(f, "admin authorization failed: {error}"),
            Self::AuthorizationDenied(outcome) => write!(
                f,
                "admin request denied: decision={:?} reason={}",
                outcome.decision_id, outcome.reason_code
            ),
            Self::Policy(error) => write!(f, "policy qualification failed: {error}"),
            Self::Lease(error) => write!(f, "lease qualification failed: {error}"),
            Self::Approval(error) => write!(f, "approval qualification failed: {error}"),
            Self::SecretCanary(error) => write!(f, "secret-canary qualification failed: {error}"),
            Self::Sandbox(error) => write!(f, "sandbox-profile qualification failed: {error}"),
            Self::DecisionNotFound => f.write_str("authorization decision was not found"),
        }
    }
}

impl Error for AdminSurfaceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Authorization(error) => Some(error),
            Self::Policy(error) => Some(error),
            Self::Lease(error) => Some(error),
            Self::Approval(error) => Some(error),
            Self::SecretCanary(error) => Some(error),
            Self::Sandbox(error) => Some(error),
            Self::AuthorizationDenied(_) | Self::DecisionNotFound => None,
        }
    }
}

impl From<AuthorizationError> for AdminSurfaceError {
    fn from(value: AuthorizationError) -> Self {
        Self::Authorization(value)
    }
}

impl From<CandidatePolicyError> for AdminSurfaceError {
    fn from(value: CandidatePolicyError) -> Self {
        Self::Policy(value)
    }
}

impl From<CapabilityLeaseScopeError> for AdminSurfaceError {
    fn from(value: CapabilityLeaseScopeError) -> Self {
        Self::Lease(value)
    }
}

impl From<ApprovalScopeError> for AdminSurfaceError {
    fn from(value: ApprovalScopeError) -> Self {
        Self::Approval(value)
    }
}

impl From<SecretCanaryQualificationError> for AdminSurfaceError {
    fn from(value: SecretCanaryQualificationError) -> Self {
        Self::SecretCanary(value)
    }
}

impl From<SandboxProfileError> for AdminSurfaceError {
    fn from(value: SandboxProfileError) -> Self {
        Self::Sandbox(value)
    }
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn qualify_policy_candidate(
        &mut self,
        principal: Principal<'_>,
        policy_source: &str,
        schema_source: &str,
        scope: &str,
    ) -> Result<PolicyQualificationReceipt, AdminSurfaceError> {
        let decision = self.authorize_admin(
            principal,
            POLICY_VALIDATE_ACTION,
            POLICY_VALIDATION_RESOURCE,
            scope,
        )?;
        let validated = validate_policy_candidate(policy_source, schema_source)?;
        let policy_hash = golam_ledger::payload_hash(validated.policy_source().as_bytes());
        let schema_hash = golam_ledger::payload_hash(validated.schema_source().as_bytes());
        let mut evidence = [0_u8; 64];
        evidence[..32].copy_from_slice(&policy_hash);
        evidence[32..].copy_from_slice(&schema_hash);
        Ok(PolicyQualificationReceipt {
            authorization_decision_id: decision.0,
            policy_bytes: validated.policy_source().len(),
            schema_bytes: validated.schema_source().len(),
            evidence_digest: golam_ledger::payload_hash(&evidence),
        })
    }

    pub fn qualify_admin_surface(
        &mut self,
        principal: Principal<'_>,
        kind: AdminQualificationKind,
        scope: &str,
    ) -> Result<AdminQualificationReceipt, AdminSurfaceError> {
        let resource = format!("authority-qualification:{}", kind.as_str());
        let decision = self.authorize_admin(principal, AUTHORITY_QUALIFY_ACTION, &resource, scope)?;
        let evidence_digest = match kind {
            AdminQualificationKind::Lease => qualify_lease_scope()?,
            AdminQualificationKind::Approval => qualify_approval_scope()?,
            AdminQualificationKind::SecretCanary => {
                qualify_designated_secret_canary()?;
                qualification_digest(b"secret-canary-prepared")
            }
            AdminQualificationKind::SandboxProfile => qualify_sandbox_profile(principal)?,
        };
        Ok(AdminQualificationReceipt {
            kind: kind.as_str(),
            authorization_decision_id: decision.0,
            resource,
            evidence_digest,
        })
    }

    pub fn explain_authorization_decision(
        &mut self,
        principal: Principal<'_>,
        decision_id: [u8; 16],
        scope: &str,
    ) -> Result<AuthorizationDecisionExplanation, AdminSurfaceError> {
        let resource = format!("authorization-decision:{}", hex16(decision_id));
        self.authorize_admin(principal, AUTHORITY_EXPLAIN_ACTION, &resource, scope)?;
        let record = self
            .authorization
            .records()?
            .into_iter()
            .find(|record| record.decision_id == decision_id)
            .ok_or(AdminSurfaceError::DecisionNotFound)?;
        Ok(AuthorizationDecisionExplanation {
            decision_id: record.decision_id,
            principal: record.principal,
            action: record.action,
            resource: record.resource,
            context_hash: record.context_hash,
            hard_guard_result: record.hard_guard_result,
            lease_id: record.lease_id,
            lease_generation: record.lease_generation,
            policy_bundle_id: record.policy_bundle_id,
            policy_bundle_hash: record.policy_bundle_hash,
            matched_rule_ids: record.matched_rule_ids,
            approval_id: record.approval_id,
            decision: record.decision.as_str().to_owned(),
            reason_code: record.reason_code,
            global_seq: record.global_seq,
            authority_evidence_version: record.authority_evidence_version,
        })
    }

    fn authorize_admin(
        &mut self,
        principal: Principal<'_>,
        action: &str,
        resource: &str,
        scope: &str,
    ) -> Result<DecisionId, AdminSurfaceError> {
        let request = AuthorizationRequest {
            principal,
            action,
            resource,
            context: AuthorizationContext::local(scope),
        };
        let (outcome, grant) = self.authorization.authorize(&request)?;
        match grant {
            Some(grant) => {
                debug_assert_eq!(grant.decision_id(), outcome.decision_id);
                Ok(grant.decision_id())
            }
            None => Err(AdminSurfaceError::AuthorizationDenied(outcome)),
        }
    }
}

fn qualify_lease_scope() -> Result<[u8; 32], CapabilityLeaseScopeError> {
    let parent = CapabilityLeaseScope::normalize(
        &[POLICY_VALIDATE_ACTION, AUTHORITY_QUALIFY_ACTION],
        &["authority-qualification:lease", POLICY_VALIDATION_RESOURCE],
        &[],
    )?;
    let child = CapabilityLeaseScope::normalize(
        &[AUTHORITY_QUALIFY_ACTION],
        &["authority-qualification:lease"],
        &[],
    )?;
    Ok(parent.derive_child(&child)?.digest())
}

fn qualify_approval_scope() -> Result<[u8; 32], ApprovalScopeError> {
    ApprovalScope::once(
        EffectId(1),
        AUTHORITY_QUALIFY_ACTION,
        "authority-qualification:approval",
    )?
    .scope_digest()
}

fn qualify_sandbox_profile(
    principal: Principal<'_>,
) -> Result<[u8; 32], SandboxProfileError> {
    let registered_by = audit_principal(principal);
    let prepared = prepare_sandbox_profile(
        SandboxProfileDefinition {
            profile_id: [0x81; 16],
            version: 1,
            class: SandboxProfileClass::NativeUntrustedSubprocess,
            filesystem_read_roots: &[],
            filesystem_write_roots: &[],
            network_rule: SandboxNetworkRule::DenyAll,
            environment_allowlist: &[],
            spawn_rule: SandboxSpawnRule::Deny,
            cpu_limit: Some(1),
            memory_limit: Some(1),
            time_limit: Some(1),
            output_limit: Some(1),
            device_allowlist: &[],
            ipc_allowlist: &[],
            inherited_handle_rules: &[],
            platform_requirements: &[],
        },
        &registered_by,
        [0; 32],
    )?;
    Ok(prepared.intent_digest())
}

fn audit_principal(principal: Principal<'_>) -> String {
    let class = match principal.kind {
        PrincipalKind::LocalOwner => "owner",
        PrincipalKind::EnrolledClient => "client",
        PrincipalKind::KernelService => "kernel",
        PrincipalKind::Test => "test",
        PrincipalKind::Unauthenticated => "unauthenticated",
    };
    match principal.client_id {
        Some(client_id) => format!("{class}:{}:{}", client_id.0, principal.subject),
        None => format!("{class}:{}", principal.subject),
    }
}

fn qualification_digest(label: &[u8]) -> [u8; 32] {
    let mut bytes = Vec::with_capacity(QUALIFICATION_DOMAIN.len() + label.len());
    bytes.extend_from_slice(QUALIFICATION_DOMAIN);
    bytes.extend_from_slice(label);
    golam_ledger::payload_hash(&bytes)
}

fn hex16(bytes: [u8; 16]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(32);
    for byte in bytes {
        out.push(char::from(HEX[(byte >> 4) as usize]));
        out.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuthorizationDecision, PolicyDecision};
    use golam_core::paths::RuntimeLayout;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    const SCHEMA: &str = r#"
entity LocalOwner;
entity GolamResource;
action "qualification.read" appliesTo { principal: LocalOwner, resource: GolamResource };
"#;
    const POLICY: &str =
        "permit(principal is LocalOwner, action == Action::\"qualification.read\", resource is GolamResource);";

    static N: AtomicU64 = AtomicU64::new(0);

    fn runtime() -> RuntimeLayout {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-kernel-admin-qualification-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    struct AllowAdmin;

    impl AuthorizationPolicy for AllowAdmin {
        fn authorize(&self, request: &AuthorizationRequest<'_>) -> PolicyDecision {
            if matches!(
                request.action,
                POLICY_VALIDATE_ACTION | AUTHORITY_QUALIFY_ACTION | AUTHORITY_EXPLAIN_ACTION
            ) {
                PolicyDecision::allow("test_admin_allow")
            } else {
                PolicyDecision::deny("test_admin_no_match")
            }
        }
    }

    #[test]
    fn all_minimal_admin_qualification_paths_are_authorized_and_non_mutating() {
        let runtime = runtime();
        let mut kernel = KernelApi::open(&runtime, AllowAdmin).unwrap();
        let principal = Principal::local_owner("owner");
        let policy = kernel
            .qualify_policy_candidate(principal, POLICY, SCHEMA, "local-owner")
            .unwrap();
        assert_eq!(policy.policy_bytes, POLICY.len());
        for kind in [
            AdminQualificationKind::Lease,
            AdminQualificationKind::Approval,
            AdminQualificationKind::SecretCanary,
            AdminQualificationKind::SandboxProfile,
        ] {
            let receipt = kernel
                .qualify_admin_surface(principal, kind, "local-owner")
                .unwrap();
            assert_ne!(receipt.evidence_digest, [0; 32]);
        }
        let records = kernel.authorization.records().unwrap();
        assert_eq!(records.len(), 5);
        let target = records[0].decision_id;
        let explained = kernel
            .explain_authorization_decision(principal, target, "local-owner")
            .unwrap();
        assert_eq!(explained.decision_id, target);
        assert_eq!(explained.decision, "allow");
        assert_eq!(kernel.authorization.records().unwrap().len(), 6);
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn denial_happens_before_policy_parser_or_qualification_work() {
        let runtime = runtime();
        let mut kernel = KernelApi::open(&runtime, crate::DenyByDefault).unwrap();
        let denied = kernel.qualify_policy_candidate(
            Principal::local_owner("owner"),
            "permit(",
            SCHEMA,
            "local-owner",
        );
        let Err(AdminSurfaceError::AuthorizationDenied(outcome)) = denied else {
            panic!("admin qualification must fail at authorization");
        };
        assert_eq!(outcome.decision, AuthorizationDecision::Deny);
        assert_eq!(kernel.authorization.records().unwrap().len(), 1);
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn lease_qualification_preserves_non_widening_rule() {
        let parent = CapabilityLeaseScope::normalize(
            &[AUTHORITY_QUALIFY_ACTION],
            &["authority-qualification:lease"],
            &[],
        )
        .unwrap();
        let wider = CapabilityLeaseScope::normalize(
            &[AUTHORITY_QUALIFY_ACTION, POLICY_VALIDATE_ACTION],
            &["authority-qualification:lease"],
            &[],
        )
        .unwrap();
        assert!(matches!(
            parent.derive_child(&wider),
            Err(CapabilityLeaseScopeError::RequestedWidening)
        ));
    }
}
''',
)
replace_once(
    "crates/golam-kernel/src/lib.rs",
    "mod authorization;\n",
    "mod admin_qualification;\nmod authorization;\n",
)
replace_once(
    "crates/golam-kernel/src/lib.rs",
    "pub use authorization::{\n",
    "pub use admin_qualification::{\n    AUTHORITY_EXPLAIN_ACTION, AUTHORITY_QUALIFY_ACTION, AdminQualificationKind,\n    AdminQualificationReceipt, AdminSurfaceError, AuthorizationDecisionExplanation,\n    POLICY_VALIDATE_ACTION, PolicyQualificationReceipt,\n};\npub use authorization::{\n",
)

# Narrow bootstrap additions: authenticated clients receive read/qualification only.
replace_once(
    "crates/golam-kernel/src/authorization.rs",
    '            | "recovery.status.read"\n    )\n}\n\nfn client_action_allowed',
    '            | "recovery.status.read"\n            | "policy.validate"\n            | "authority.qualify"\n            | "authority.explain"\n    )\n}\n\nfn client_action_allowed',
)
replace_once(
    "crates/golam-kernel/src/authorization.rs",
    '            | "recovery.status.read"\n    )\n}\n\nfn kernel_service_action_allowed',
    '            | "recovery.status.read"\n            | "policy.validate"\n            | "authority.qualify"\n            | "authority.explain"\n    )\n}\n\nfn kernel_service_action_allowed',
)

replace_once(
    "crates/golam-kernel/src/runtime_policy.rs",
    "use crate::policy_candidate::validate_policy_candidate;\n",
    "use crate::admin_qualification::{\n    AUTHORITY_EXPLAIN_ACTION, AUTHORITY_QUALIFY_ACTION, POLICY_VALIDATE_ACTION,\n};\nuse crate::policy_candidate::validate_policy_candidate;\n",
)
replace_once(
    "crates/golam-kernel/src/runtime_policy.rs",
    r'''fn bootstrap_administration(request: &AuthorizationRequest<'_>) -> PolicyDecision {
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
''',
    r'''fn bootstrap_administration(request: &AuthorizationRequest<'_>) -> PolicyDecision {
    let local_scope = request.context.scope.starts_with("local");
    let read_only_admin_action = matches!(
        request.action,
        POLICY_VALIDATE_ACTION | AUTHORITY_QUALIFY_ACTION | AUTHORITY_EXPLAIN_ACTION
    );
    let owner_admin_action = matches!(
        request.action,
        POLICY_STAGE_ACTION
            | POLICY_ACTIVATE_ACTION
            | APPROVAL_ISSUE_ACTION
            | POLICY_VALIDATE_ACTION
            | AUTHORITY_QUALIFY_ACTION
            | AUTHORITY_EXPLAIN_ACTION
            | "recovery.status.read"
    );
    let allowed = local_scope
        && ((request.principal.kind == PrincipalKind::LocalOwner && owner_admin_action)
            || (request.principal.kind == PrincipalKind::EnrolledClient
                && read_only_admin_action));
    if allowed {
        PolicyDecision::allow("bootstrap_admin_explicit_allow")
    } else {
        PolicyDecision::deny("bootstrap_admin_no_matching_allow")
    }
}
''',
)
replace_once(
    "crates/golam-kernel/src/runtime_policy.rs",
    r'''        let client = AuthorizationRequest {
            principal: Principal::enrolled_client("local-cli", golam_core::ClientId(1)),
            action: POLICY_STAGE_ACTION,
            resource: "policy-bundle:bootstrap",
            context: AuthorizationContext::local("local-ipc"),
        };
        assert_eq!(
            bootstrap_administration(&client).decision,
            AuthorizationDecision::Deny
        );
''',
    r'''        let client = AuthorizationRequest {
            principal: Principal::enrolled_client("local-cli", golam_core::ClientId(1)),
            action: POLICY_STAGE_ACTION,
            resource: "policy-bundle:bootstrap",
            context: AuthorizationContext::local("local-ipc"),
        };
        assert_eq!(
            bootstrap_administration(&client).decision,
            AuthorizationDecision::Deny
        );
        let client_qualification = AuthorizationRequest {
            action: AUTHORITY_QUALIFY_ACTION,
            resource: "authority-qualification:lease",
            ..client
        };
        assert_eq!(
            bootstrap_administration(&client_qualification).decision,
            AuthorizationDecision::Allow
        );
''',
)

# Typed IPC additions.
replace_once(
    "crates/golam-ipc/src/command.rs",
    "pub const METHOD_CLIENT_ENROLL: MethodId = MethodId(111);\n",
    "pub const METHOD_CLIENT_ENROLL: MethodId = MethodId(111);\npub const METHOD_POLICY_VALIDATE: MethodId = MethodId(112);\npub const METHOD_AUTHORITY_QUALIFY: MethodId = MethodId(113);\npub const METHOD_AUTHORITY_EXPLAIN: MethodId = MethodId(114);\n",
)
replace_once(
    "crates/golam-ipc/src/command.rs",
    "#[derive(Clone, Debug, Eq, PartialEq)]\npub enum Command {\n",
    r'''#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorityQualificationKind {
    Lease,
    Approval,
    SecretCanary,
    SandboxProfile,
}

impl AuthorityQualificationKind {
    const fn code(self) -> u8 {
        match self {
            Self::Lease => 1,
            Self::Approval => 2,
            Self::SecretCanary => 3,
            Self::SandboxProfile => 4,
        }
    }

    const fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Lease),
            2 => Some(Self::Approval),
            3 => Some(Self::SecretCanary),
            4 => Some(Self::SandboxProfile),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
''',
)
replace_once(
    "crates/golam-ipc/src/command.rs",
    "    EffectReconcile {\n        effect_id: EffectId,\n    },\n    Doctor,\n",
    "    EffectReconcile {\n        effect_id: EffectId,\n    },\n    PolicyValidate {\n        policy_source: String,\n        schema_source: String,\n    },\n    AuthorityQualify {\n        kind: AuthorityQualificationKind,\n    },\n    AuthorityExplain {\n        decision_id: [u8; 16],\n    },\n    Doctor,\n",
)
replace_once(
    "crates/golam-ipc/src/command.rs",
    "    InvalidSyntheticSemantics(u8),\n    LengthOverflow,\n",
    "    InvalidSyntheticSemantics(u8),\n    InvalidAuthorityQualificationKind(u8),\n    LengthOverflow,\n",
)
replace_once(
    "crates/golam-ipc/src/command.rs",
    "            Self::InvalidSyntheticSemantics(code) => {\n                write!(f, \"unknown synthetic effect semantics code {code}\")\n            }\n            Self::LengthOverflow",
    "            Self::InvalidSyntheticSemantics(code) => {\n                write!(f, \"unknown synthetic effect semantics code {code}\")\n            }\n            Self::InvalidAuthorityQualificationKind(code) => {\n                write!(f, \"unknown authority qualification kind code {code}\")\n            }\n            Self::LengthOverflow",
)
replace_once(
    "crates/golam-ipc/src/command.rs",
    "        Command::EffectReconcile { effect_id } => {\n            let mut body = Writer::new();\n            body.u128(effect_id.0);\n            (METHOD_EFFECT_RECONCILE, body.finish())\n        }\n        Command::Doctor =>",
    "        Command::EffectReconcile { effect_id } => {\n            let mut body = Writer::new();\n            body.u128(effect_id.0);\n            (METHOD_EFFECT_RECONCILE, body.finish())\n        }\n        Command::PolicyValidate {\n            policy_source,\n            schema_source,\n        } => {\n            check_field(policy_source.len(), MAX_TEXT_BYTES)?;\n            check_field(schema_source.len(), MAX_TEXT_BYTES)?;\n            let mut body = Writer::new();\n            body.bytes(policy_source.as_bytes())?;\n            body.bytes(schema_source.as_bytes())?;\n            (METHOD_POLICY_VALIDATE, body.finish())\n        }\n        Command::AuthorityQualify { kind } => {\n            let mut body = Writer::new();\n            body.u8(kind.code());\n            (METHOD_AUTHORITY_QUALIFY, body.finish())\n        }\n        Command::AuthorityExplain { decision_id } => {\n            let mut body = Writer::new();\n            body.u128(u128::from_be_bytes(*decision_id));\n            (METHOD_AUTHORITY_EXPLAIN, body.finish())\n        }\n        Command::Doctor =>",
)
replace_once(
    "crates/golam-ipc/src/command.rs",
    "        METHOD_EFFECT_RECONCILE => Command::EffectReconcile {\n            effect_id: EffectId(reader.u128()?),\n        },\n        METHOD_DOCTOR => Command::Doctor,\n",
    "        METHOD_EFFECT_RECONCILE => Command::EffectReconcile {\n            effect_id: EffectId(reader.u128()?),\n        },\n        METHOD_POLICY_VALIDATE => Command::PolicyValidate {\n            policy_source: reader.text(MAX_TEXT_BYTES)?,\n            schema_source: reader.text(MAX_TEXT_BYTES)?,\n        },\n        METHOD_AUTHORITY_QUALIFY => {\n            let code = reader.u8()?;\n            let kind = AuthorityQualificationKind::from_code(code)\n                .ok_or(CommandCodecError::InvalidAuthorityQualificationKind(code))?;\n            Command::AuthorityQualify { kind }\n        }\n        METHOD_AUTHORITY_EXPLAIN => Command::AuthorityExplain {\n            decision_id: reader.u128()?.to_be_bytes(),\n        },\n        METHOD_DOCTOR => Command::Doctor,\n",
)
replace_once(
    "crates/golam-ipc/src/command.rs",
    "            Command::EffectReconcile {\n                effect_id: EffectId(11),\n            },\n            Command::Doctor,\n",
    "            Command::EffectReconcile {\n                effect_id: EffectId(11),\n            },\n            Command::PolicyValidate {\n                policy_source: \"permit(principal, action, resource);\".to_owned(),\n                schema_source: \"entity User;\".to_owned(),\n            },\n            Command::AuthorityQualify {\n                kind: AuthorityQualificationKind::SecretCanary,\n            },\n            Command::AuthorityExplain { decision_id: [7; 16] },\n            Command::Doctor,\n",
)
replace_once(
    "crates/golam-ipc/src/command.rs",
    "        assert_eq!(\n            decode_command(&RequestMessage {\n                method: METHOD_EFFECT_SIMULATE,\n                body,\n            }),\n            Err(CommandCodecError::InvalidSyntheticSemantics(255))\n        );\n",
    "        assert_eq!(\n            decode_command(&RequestMessage {\n                method: METHOD_EFFECT_SIMULATE,\n                body,\n            }),\n            Err(CommandCodecError::InvalidSyntheticSemantics(255))\n        );\n        assert_eq!(\n            decode_command(&RequestMessage {\n                method: METHOD_AUTHORITY_QUALIFY,\n                body: vec![255],\n            }),\n            Err(CommandCodecError::InvalidAuthorityQualificationKind(255))\n        );\n",
)

# CLI parsing stays transport-only; no filesystem or authority access is added.
replace_once(
    "crates/golam/src/lib.rs",
    "use golam_ipc::command::{Command, SyntheticSemantics};\n",
    "use golam_ipc::command::{AuthorityQualificationKind, Command, SyntheticSemantics};\n",
)
replace_once(
    "crates/golam/src/lib.rs",
    "    InvalidSemantics(String),\n",
    "    InvalidSemantics(String),\n    InvalidHex { field: &'static str, value: String },\n    InvalidQualificationKind(String),\n",
)
replace_once(
    "crates/golam/src/lib.rs",
    "            Self::InvalidSemantics(value) => write!(\n                f,\n                \"invalid synthetic semantics {value}; expected read-only, idempotent-at-least-once, at-most-once, compensatable, or irreversible\"\n            ),\n",
    "            Self::InvalidSemantics(value) => write!(\n                f,\n                \"invalid synthetic semantics {value}; expected read-only, idempotent-at-least-once, at-most-once, compensatable, or irreversible\"\n            ),\n            Self::InvalidHex { field, value } => {\n                write!(f, \"invalid {field} hex value: {value}; expected exactly 32 lowercase or uppercase hex digits\")\n            }\n            Self::InvalidQualificationKind(value) => write!(\n                f,\n                \"invalid authority qualification kind {value}; expected lease, approval, secret-canary, or sandbox-profile\"\n            ),\n",
)
replace_once(
    "crates/golam/src/lib.rs",
    '  golam effect reconcile <effect-id>\\n\\\n  golam doctor"',
    '  golam effect reconcile <effect-id>\\n\\\n  golam policy validate <policy-source> <schema-source>\\n\\\n  golam authority qualify <lease|approval|secret-canary|sandbox-profile>\\n\\\n  golam authority explain <decision-id-hex>\\n\\\n  golam doctor"',
)
replace_once(
    "crates/golam/src/lib.rs",
    "        [\"effect\", \"reconcile\", effect_id] => Ok(Command::EffectReconcile {\n            effect_id: EffectId(parse_u128(\"effect-id\", effect_id)?),\n        }),\n        [\"doctor\"]",
    "        [\"effect\", \"reconcile\", effect_id] => Ok(Command::EffectReconcile {\n            effect_id: EffectId(parse_u128(\"effect-id\", effect_id)?),\n        }),\n        [\"policy\", \"validate\", policy_source, schema_source] => Ok(Command::PolicyValidate {\n            policy_source: (*policy_source).to_owned(),\n            schema_source: (*schema_source).to_owned(),\n        }),\n        [\"authority\", \"qualify\", kind] => Ok(Command::AuthorityQualify {\n            kind: parse_qualification_kind(kind)?,\n        }),\n        [\"authority\", \"explain\", decision_id] => Ok(Command::AuthorityExplain {\n            decision_id: parse_hex_16(\"decision-id\", decision_id)?,\n        }),\n        [\"doctor\"]",
)
replace_once(
    "crates/golam/src/lib.rs",
    "fn parse_semantics(value: &str) -> Result<SyntheticSemantics, CliError> {\n",
    r'''fn parse_hex_16(field: &'static str, value: &str) -> Result<[u8; 16], CliError> {
    if value.len() != 32 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(CliError::InvalidHex {
            field,
            value: value.to_owned(),
        });
    }
    u128::from_str_radix(value, 16)
        .map(u128::to_be_bytes)
        .map_err(|_| CliError::InvalidHex {
            field,
            value: value.to_owned(),
        })
}

fn parse_qualification_kind(value: &str) -> Result<AuthorityQualificationKind, CliError> {
    match value {
        "lease" => Ok(AuthorityQualificationKind::Lease),
        "approval" => Ok(AuthorityQualificationKind::Approval),
        "secret-canary" => Ok(AuthorityQualificationKind::SecretCanary),
        "sandbox-profile" => Ok(AuthorityQualificationKind::SandboxProfile),
        _ => Err(CliError::InvalidQualificationKind(value.to_owned())),
    }
}

fn parse_semantics(value: &str) -> Result<SyntheticSemantics, CliError> {
''',
)
replace_once(
    "crates/golam/src/lib.rs",
    "            (\n                vec![\"effect\", \"reconcile\", \"6\"],\n                Command::EffectReconcile {\n                    effect_id: EffectId(6),\n                },\n            ),\n            (vec![\"doctor\"], Command::Doctor),\n",
    "            (\n                vec![\"effect\", \"reconcile\", \"6\"],\n                Command::EffectReconcile {\n                    effect_id: EffectId(6),\n                },\n            ),\n            (\n                vec![\"policy\", \"validate\", \"permit(principal, action, resource);\", \"entity User;\"],\n                Command::PolicyValidate {\n                    policy_source: \"permit(principal, action, resource);\".to_owned(),\n                    schema_source: \"entity User;\".to_owned(),\n                },\n            ),\n            (\n                vec![\"authority\", \"qualify\", \"secret-canary\"],\n                Command::AuthorityQualify {\n                    kind: AuthorityQualificationKind::SecretCanary,\n                },\n            ),\n            (\n                vec![\"authority\", \"explain\", \"07070707070707070707070707070707\"],\n                Command::AuthorityExplain { decision_id: [7; 16] },\n            ),\n            (vec![\"doctor\"], Command::Doctor),\n",
)
replace_once(
    "crates/golam/src/lib.rs",
    "        assert!(matches!(parse_args([\"session\"]), Err(CliError::Usage(_))));\n",
    "        assert!(matches!(parse_args([\"session\"]), Err(CliError::Usage(_))));\n        assert!(matches!(\n            parse_args([\"authority\", \"explain\", \"xyz\"]),\n            Err(CliError::InvalidHex { .. })\n        ));\n        assert_eq!(\n            parse_args([\"authority\", \"qualify\", \"unknown\"]),\n            Err(CliError::InvalidQualificationKind(\"unknown\".to_owned()))\n        );\n",
)

# Daemon router: authenticated principal flows through the new KernelApi methods.
replace_once(
    "crates/golamd/src/lib.rs",
    "use golam_ipc::command::{Command, SyntheticSemantics, decode_command};\n",
    "use golam_ipc::command::{AuthorityQualificationKind, Command, SyntheticSemantics, decode_command};\n",
)
replace_once(
    "crates/golamd/src/lib.rs",
    "    AuthorizationPolicy, ClientEnrollmentError, ClientKind, CompleteSyntheticEffect, KernelApi,\n    KernelError, KernelOperationError, PrepareSyntheticEffect, Principal,\n",
    "    AdminQualificationKind, AdminSurfaceError, AuthorizationPolicy, ClientEnrollmentError,\n    ClientKind, CompleteSyntheticEffect, KernelApi, KernelError, KernelOperationError,\n    PrepareSyntheticEffect, Principal,\n",
)
replace_once(
    "crates/golamd/src/lib.rs",
    "            Command::EffectReconcile { effect_id } => {\n                self.reconcile_effect(principal, effect_id, now, scope)\n            }\n            Command::Doctor =>",
    r'''            Command::EffectReconcile { effect_id } => {
                self.reconcile_effect(principal, effect_id, now, scope)
            }
            Command::PolicyValidate {
                policy_source,
                schema_source,
            } => match self.kernel.qualify_policy_candidate(
                principal,
                &policy_source,
                &schema_source,
                scope,
            ) {
                Ok(receipt) => reply(
                    ReplyStatus::Ok,
                    format!(
                        "kind=policy decision_id={} policy_bytes={} schema_bytes={} evidence_digest={}\n",
                        hex_prefix(&receipt.authorization_decision_id),
                        receipt.policy_bytes,
                        receipt.schema_bytes,
                        hex_prefix(&receipt.evidence_digest),
                    ),
                ),
                Err(error) => admin_error(error),
            },
            Command::AuthorityQualify { kind } => {
                let kind = match kind {
                    AuthorityQualificationKind::Lease => AdminQualificationKind::Lease,
                    AuthorityQualificationKind::Approval => AdminQualificationKind::Approval,
                    AuthorityQualificationKind::SecretCanary => AdminQualificationKind::SecretCanary,
                    AuthorityQualificationKind::SandboxProfile => AdminQualificationKind::SandboxProfile,
                };
                match self.kernel.qualify_admin_surface(principal, kind, scope) {
                    Ok(receipt) => reply(
                        ReplyStatus::Ok,
                        format!(
                            "kind={} decision_id={} resource={} evidence_digest={}\n",
                            receipt.kind,
                            hex_prefix(&receipt.authorization_decision_id),
                            receipt.resource,
                            hex_prefix(&receipt.evidence_digest),
                        ),
                    ),
                    Err(error) => admin_error(error),
                }
            }
            Command::AuthorityExplain { decision_id } => {
                match self
                    .kernel
                    .explain_authorization_decision(principal, decision_id, scope)
                {
                    Ok(explained) => {
                        let mut body = format!(
                            "decision_id={} decision={} principal={} action={} resource={} reason={} hard_guard={} global_seq={} evidence_version={}\ncontext_hash={} lease_id={} lease_generation={} policy_bundle_id={} policy_bundle_hash={} approval_id={}\n",
                            hex_prefix(&explained.decision_id),
                            explained.decision,
                            explained.principal,
                            explained.action,
                            explained.resource,
                            explained.reason_code,
                            explained.hard_guard_result,
                            explained.global_seq,
                            explained.authority_evidence_version,
                            hex_prefix(&explained.context_hash),
                            optional_hex(explained.lease_id.as_ref().map(|value| value.as_slice())),
                            explained
                                .lease_generation
                                .map(|value| value.to_string())
                                .unwrap_or_else(|| "-".to_owned()),
                            optional_hex(explained.policy_bundle_id.as_ref().map(|value| value.as_slice())),
                            optional_hex(explained.policy_bundle_hash.as_ref().map(|value| value.as_slice())),
                            optional_hex(explained.approval_id.as_ref().map(|value| value.as_slice())),
                        );
                        for rule_id in explained.matched_rule_ids {
                            body.push_str(&format!("matched_rule_id={rule_id}\n"));
                        }
                        reply(ReplyStatus::Ok, body)
                    }
                    Err(error) => admin_error(error),
                }
            }
            Command::Doctor =>''',
)
replace_once(
    "crates/golamd/src/lib.rs",
    "fn operation_error(error: KernelOperationError) -> ReplyMessage {\n",
    r'''fn admin_error(error: AdminSurfaceError) -> ReplyMessage {
    let status = match &error {
        AdminSurfaceError::AuthorizationDenied(_) => ReplyStatus::Denied,
        _ => ReplyStatus::Failed,
    };
    reply(status, format!("error={error}\n"))
}

fn operation_error(error: KernelOperationError) -> ReplyMessage {
''',
)
replace_once(
    "crates/golamd/src/lib.rs",
    "fn optional_u128(value: Option<u128>) -> String {\n",
    r'''fn optional_hex(value: Option<&[u8]>) -> String {
    value.map(hex_prefix).unwrap_or_else(|| "-".to_owned())
}

fn optional_u128(value: Option<u128>) -> String {
''',
)
replace_once(
    "crates/golamd/src/lib.rs",
    "    #[test]\n    fn invalid_command_is_rejected_before_kernel_dispatch() {\n",
    r'''    #[test]
    fn authenticated_admin_qualification_routes_through_kernel_api() {
        let runtime = runtime();
        let kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let mut router = CommandRouter::new(kernel);
        let principal = Principal::enrolled_client("local-cli", ClientId(9001));
        let qualified = router.route(
            principal,
            &request(Command::AuthorityQualify {
                kind: golam_ipc::command::AuthorityQualificationKind::Lease,
            }),
            "2026-08-29T01:00:00Z",
            "local-ipc",
        );
        assert_eq!(qualified.status, ReplyStatus::Ok);
        let qualified_text = String::from_utf8(qualified.body).unwrap();
        let decision_id = qualified_text
            .split_whitespace()
            .find_map(|field| field.strip_prefix("decision_id="))
            .unwrap();
        let mut parsed = [0_u8; 16];
        for (index, chunk) in decision_id.as_bytes().chunks_exact(2).enumerate() {
            parsed[index] = u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap();
        }
        let explained = router.route(
            principal,
            &request(Command::AuthorityExplain { decision_id: parsed }),
            "2026-08-29T01:00:01Z",
            "local-ipc",
        );
        assert_eq!(explained.status, ReplyStatus::Ok);
        assert!(String::from_utf8(explained.body).unwrap().contains("action=authority.qualify"));
        drop(router);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn invalid_command_is_rejected_before_kernel_dispatch() {
''',
)

# Quickstart now documents the exact bounded T003-081 surface and explicitly
# preserves protected mutation APIs rather than inventing CLI evidence.
write(
    "specs/003-identity-policy-secrets-sandbox/quickstart.md",
    r'''# Quickstart — Spec 003 Authority Qualification Surface

The Spec 003 implementation exposes a deliberately small authenticated local CLI/admin/test surface. It is for qualification and explanation; it does **not** create a generic protected-authority mutation console.

## Existing baseline

Spec 002 provides authenticated local `golamd`/`golam`, protected authority storage, durable effects/recovery, and strict-local no-egress proof. Spec 003 keeps all protected mutations behind typed `KernelApi` methods with exact authorization/effect/approval evidence.

## Exact T003-081 CLI surface

```text
golam policy validate <policy-source> <schema-source>
golam authority qualify lease
golam authority qualify approval
golam authority qualify secret-canary
golam authority qualify sandbox-profile
golam authority explain <decision-id-hex>
```

The policy/schema arguments are bounded inline Cedar source strings intended for deterministic admin/test qualification. Shell quoting is required when the source contains spaces or punctuation.

All commands travel over the existing authenticated local IPC path. `policy.validate`, `authority.qualify`, and `authority.explain` are non-mutating. In bootstrap state they are the only new read/qualification actions admitted for an authenticated enrolled local client; policy staging/activation, lease issuance/revocation, approval issuance/revocation, secret mutation, and sandbox-profile registration are **not** granted by this CLI surface.

## Protected mutation boundary

The planning examples below remain implemented as typed kernel lifecycle APIs, not as free-form CLI shortcuts:

- policy stage/activate require current policy authority and exact protected lifecycle evidence;
- capability lease issue/derive/revoke remains kernel-minted, non-self-expanding, and exact-evidence-bound;
- approval issue/revoke/use remains protected and class/scope/effect bound;
- secret create/rotate/revoke remains protected and never accepts production plaintext through this qualification CLI;
- sandbox-profile registration remains protected and requires exact decision/effect/ONCE-approval evidence.

T003-081 intentionally does not fabricate the decision/approval/effect tuples those mutation paths require.

## Qualification behavior

### Policy

`policy validate` runs the same bounded strict Cedar candidate parser/schema validator used before policy staging. It does not stage or activate the candidate.

### Lease

`authority qualify lease` exercises canonical lease-scope normalization and a strict child narrowing. The returned receipt is evidence only and is not a `CapabilityLease`.

### Approval

`authority qualify approval` constructs and canonically digests a bounded ONCE approval scope. It does not issue an approval.

### Secret canary

`authority qualify secret-canary` sends no caller-supplied secret. A fixed unknown-format deterministic canary exists only inside the ledger qualification module, enters the same explicit designated-secret preparation path, is never committed or returned, and is dropped through the zeroizing protected owner. T003-093 remains the full durable leakage suite.

### Decision explain

`authority explain` returns bounded stored authorization evidence: principal, action, resource, context hash, hard-guard result, lease/policy/approval identifiers, matched policy rule IDs, decision/reason, sequence and evidence version. It does not return secret plaintext or raw authorization context.

### Sandbox profile

`authority qualify sandbox-profile` validates and canonicalizes a fixed deny-all, no-spawn, empty-inheritance native profile and returns non-authority intent evidence. It neither registers the profile nor launches a process and is not containment proof.

## Expected strict-local behavior

Even with a policy rule, lease, approval and egress permit that otherwise match, external network access in strict-local mode remains denied before the policy permit can become effective.

## Expected secret behavior

Normal diagnostic/listing/explain APIs display opaque identifiers and metadata only. The deterministic canary must not appear in durable logs/events/errors or model-visible history.

## Expected sandbox behavior

A profile that requires a containment feature unavailable on the current platform is rejected before process launch. A profile definition or qualification receipt alone is not reported as sandbox proof.

## Evidence commands

Implementation retains the pinned Rust exact-head gates:

```bash
cargo +1.98.0 fmt --all -- --check
cargo +1.98.0 clippy --locked --workspace --all-targets -- -D warnings
cargo +1.98.0 test --locked --workspace --all-targets
```

Focused T003-081 qualification additionally covers the command codec, CLI parser, kernel admin boundary, daemon router, unauthenticated/denied ordering and bootstrap mutation denial.
''',
)

print("T003-081 source preparation complete")
