#![forbid(unsafe_code)]

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
        let decision =
            self.authorize_admin(principal, AUTHORITY_QUALIFY_ACTION, &resource, scope)?;
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

fn qualify_sandbox_profile(principal: Principal<'_>) -> Result<[u8; 32], SandboxProfileError> {
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
    const POLICY: &str = "permit(principal is LocalOwner, action == Action::\"qualification.read\", resource is GolamResource);";

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
