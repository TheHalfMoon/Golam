#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::ClientId;
use golam_core::authority::AuthorityLayout;
use golam_ledger::authorization::{
    AppendAuthorizationDecision, AuthorizationAuditError, AuthorizationAuditLog,
    AuthorizationDecisionKind, StoredAuthorizationDecision,
};

const HARD_SAFETY_DENIAL: &str = "safety_denial_monotonic";
const STRICT_LOCAL_EGRESS_DENIAL: &str = "strict_local_egress_denied";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrincipalKind {
    LocalOwner,
    EnrolledClient,
    KernelService,
    Test,
    Unauthenticated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Principal<'a> {
    pub kind: PrincipalKind,
    pub subject: &'a str,
    pub client_id: Option<ClientId>,
}

impl<'a> Principal<'a> {
    pub const fn local_owner(subject: &'a str) -> Self {
        Self {
            kind: PrincipalKind::LocalOwner,
            subject,
            client_id: None,
        }
    }

    pub const fn enrolled_client(subject: &'a str, client_id: ClientId) -> Self {
        Self {
            kind: PrincipalKind::EnrolledClient,
            subject,
            client_id: Some(client_id),
        }
    }

    pub const fn kernel_service(subject: &'a str) -> Self {
        Self {
            kind: PrincipalKind::KernelService,
            subject,
            client_id: None,
        }
    }

    pub const fn test(subject: &'a str) -> Self {
        Self {
            kind: PrincipalKind::Test,
            subject,
            client_id: None,
        }
    }

    pub const fn unauthenticated(subject: &'a str) -> Self {
        Self {
            kind: PrincipalKind::Unauthenticated,
            subject,
            client_id: None,
        }
    }

    fn audit_subject(self) -> String {
        let class = match self.kind {
            PrincipalKind::LocalOwner => "owner",
            PrincipalKind::EnrolledClient => "client",
            PrincipalKind::KernelService => "kernel",
            PrincipalKind::Test => "test",
            PrincipalKind::Unauthenticated => "unauthenticated",
        };
        match self.client_id {
            Some(client_id) => format!("{class}:{}:{}", client_id.0, self.subject),
            None => format!("{class}:{}", self.subject),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorizationContext<'a> {
    pub scope: &'a str,
    pub safety_denied: bool,
    pub test_mode: bool,
}

impl<'a> AuthorizationContext<'a> {
    pub const fn local(scope: &'a str) -> Self {
        Self {
            scope,
            safety_denied: false,
            test_mode: false,
        }
    }

    fn audit_text(self) -> String {
        format!(
            "scope={};safety_denied={};test_mode={}",
            self.scope, self.safety_denied, self.test_mode
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorizationRequest<'a> {
    pub principal: Principal<'a>,
    pub action: &'a str,
    pub resource: &'a str,
    pub context: AuthorizationContext<'a>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationDecision {
    Allow,
    Deny,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PolicyDecision {
    pub decision: AuthorizationDecision,
    pub reason_code: &'static str,
}

impl PolicyDecision {
    pub const fn allow(reason_code: &'static str) -> Self {
        Self {
            decision: AuthorizationDecision::Allow,
            reason_code,
        }
    }

    pub const fn deny(reason_code: &'static str) -> Self {
        Self {
            decision: AuthorizationDecision::Deny,
            reason_code,
        }
    }
}

pub trait AuthorizationPolicy {
    fn authorize(&self, request: &AuthorizationRequest<'_>) -> PolicyDecision;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecisionId(pub [u8; 16]);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationOutcome {
    pub decision_id: DecisionId,
    pub decision: AuthorizationDecision,
    pub reason_code: String,
    pub global_seq: u64,
}

#[derive(Debug)]
pub enum AuthorizationError {
    Audit(AuthorizationAuditError),
    EmptyPrincipal,
    EmptyAction,
    EmptyResource,
}

impl fmt::Display for AuthorizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Audit(error) => write!(f, "authorization audit failed: {error}"),
            Self::EmptyPrincipal => f.write_str("authorization principal is empty"),
            Self::EmptyAction => f.write_str("authorization action is empty"),
            Self::EmptyResource => f.write_str("authorization resource is empty"),
        }
    }
}

impl Error for AuthorizationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Audit(error) => Some(error),
            Self::EmptyPrincipal | Self::EmptyAction | Self::EmptyResource => None,
        }
    }
}

impl From<AuthorizationAuditError> for AuthorizationError {
    fn from(value: AuthorizationAuditError) -> Self {
        Self::Audit(value)
    }
}

#[derive(Debug)]
pub(crate) struct AuthorityGrant {
    decision_id: DecisionId,
    _sealed: (),
}

impl AuthorityGrant {
    pub(crate) const fn decision_id(&self) -> DecisionId {
        self.decision_id
    }
}

pub(crate) struct AuthorizationEngine<P> {
    policy: P,
    audit: AuthorizationAuditLog,
}

impl<P: AuthorizationPolicy> AuthorizationEngine<P> {
    pub(crate) fn open(authority: &AuthorityLayout, policy: P) -> Result<Self, AuthorizationError> {
        Ok(Self {
            policy,
            audit: AuthorizationAuditLog::open(authority)?,
        })
    }

    pub(crate) fn authorize(
        &mut self,
        request: &AuthorizationRequest<'_>,
    ) -> Result<(AuthorizationOutcome, Option<AuthorityGrant>), AuthorizationError> {
        if request.principal.subject.is_empty() {
            return Err(AuthorizationError::EmptyPrincipal);
        }
        if request.action.is_empty() {
            return Err(AuthorizationError::EmptyAction);
        }
        if request.resource.is_empty() {
            return Err(AuthorizationError::EmptyResource);
        }

        let policy_decision = if request.context.safety_denied {
            PolicyDecision::deny(HARD_SAFETY_DENIAL)
        } else if is_network_egress(request.action) {
            PolicyDecision::deny(STRICT_LOCAL_EGRESS_DENIAL)
        } else {
            self.policy.authorize(request)
        };
        let principal = request.principal.audit_subject();
        let context = request.context.audit_text();
        let decision = match policy_decision.decision {
            AuthorizationDecision::Allow => AuthorizationDecisionKind::Allow,
            AuthorizationDecision::Deny => AuthorizationDecisionKind::Deny,
        };
        let stored = self.audit.append(AppendAuthorizationDecision {
            principal: &principal,
            action: request.action,
            resource: request.resource,
            context: &context,
            decision,
            reason_code: policy_decision.reason_code,
        })?;
        let outcome = outcome_from_stored(&stored);
        let grant = if outcome.decision == AuthorizationDecision::Allow {
            Some(AuthorityGrant {
                decision_id: outcome.decision_id,
                _sealed: (),
            })
        } else {
            None
        };
        Ok((outcome, grant))
    }

    pub(crate) fn records(&self) -> Result<Vec<StoredAuthorizationDecision>, AuthorizationError> {
        Ok(self.audit.records()?)
    }
}

fn outcome_from_stored(stored: &StoredAuthorizationDecision) -> AuthorizationOutcome {
    AuthorizationOutcome {
        decision_id: DecisionId(stored.decision_id),
        decision: match stored.decision {
            AuthorizationDecisionKind::Allow => AuthorizationDecision::Allow,
            AuthorizationDecisionKind::Deny => AuthorizationDecision::Deny,
        },
        reason_code: stored.reason_code.clone(),
        global_seq: stored.global_seq,
    }
}

fn is_network_egress(action: &str) -> bool {
    action == "network.egress" || action.starts_with("network.egress.")
}

pub struct DenyByDefault;

impl AuthorizationPolicy for DenyByDefault {
    fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
        PolicyDecision::deny("deny_by_default")
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BootstrapPolicy {
    pub test_fault_injection: bool,
}

impl AuthorizationPolicy for BootstrapPolicy {
    fn authorize(&self, request: &AuthorizationRequest<'_>) -> PolicyDecision {
        if !request.context.scope.starts_with("local") {
            return PolicyDecision::deny("non_local_scope_denied");
        }

        let allowed = match request.principal.kind {
            PrincipalKind::LocalOwner => owner_action_allowed(request.action),
            PrincipalKind::EnrolledClient => client_action_allowed(request.action),
            PrincipalKind::KernelService => kernel_service_action_allowed(request.action),
            PrincipalKind::Test => {
                request.context.test_mode
                    && self.test_fault_injection
                    && request.action == "fault.inject"
            }
            PrincipalKind::Unauthenticated => false,
        };
        if allowed {
            PolicyDecision::allow("bootstrap_explicit_allow")
        } else {
            PolicyDecision::deny("bootstrap_no_matching_allow")
        }
    }
}

fn owner_action_allowed(action: &str) -> bool {
    matches!(
        action,
        "session.read"
            | "session.create"
            | "session.fork"
            | "session.event.append"
            | "goal.append"
            | "checkpoint.create"
            | "checkpoint.verify"
            | "replay.run"
            | "client.enroll"
            | "client.revoke"
            | "effect.simulate"
            | "effect.reconcile"
            | "recovery.status.read"
    )
}

fn client_action_allowed(action: &str) -> bool {
    matches!(
        action,
        "session.read"
            | "session.create"
            | "session.fork"
            | "session.event.append"
            | "goal.append"
            | "checkpoint.create"
            | "checkpoint.verify"
            | "replay.run"
            | "effect.simulate"
            | "effect.reconcile"
            | "recovery.status.read"
    )
}

fn kernel_service_action_allowed(action: &str) -> bool {
    matches!(
        action,
        "checkpoint.create"
            | "checkpoint.verify"
            | "replay.run"
            | "effect.simulate"
            | "effect.reconcile"
            | "recovery.status.read"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::authority::AuthorityLayout;
    use golam_core::paths::RuntimeLayout;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(
            std::env::temp_dir().join(format!("golam-kernel-auth-{}-{t}-{n}", std::process::id())),
        )
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    #[test]
    fn bootstrap_policy_is_explicit_and_audited() {
        let (runtime, authority) = authority();
        let mut engine = AuthorizationEngine::open(&authority, BootstrapPolicy::default()).unwrap();
        let allow = AuthorizationRequest {
            principal: Principal::local_owner("owner"),
            action: "session.create",
            resource: "session:new",
            context: AuthorizationContext::local("local-owner"),
        };
        let (allowed, grant) = engine.authorize(&allow).unwrap();
        assert_eq!(allowed.decision, AuthorizationDecision::Allow);
        assert_eq!(grant.unwrap().decision_id(), allowed.decision_id);

        let deny = AuthorizationRequest {
            principal: Principal::enrolled_client("owner", ClientId(7)),
            action: "client.revoke",
            resource: "client:9",
            context: AuthorizationContext::local("local-client"),
        };
        let (denied, grant) = engine.authorize(&deny).unwrap();
        assert_eq!(denied.decision, AuthorizationDecision::Deny);
        assert!(grant.is_none());
        assert_eq!(engine.records().unwrap().len(), 2);
        drop(engine);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn safety_denial_and_network_denial_are_monotonic() {
        struct AllowEverything;
        impl AuthorizationPolicy for AllowEverything {
            fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
                PolicyDecision::allow("test_allow_everything")
            }
        }

        let (runtime, authority) = authority();
        let mut engine = AuthorizationEngine::open(&authority, AllowEverything).unwrap();
        let safety = AuthorizationRequest {
            principal: Principal::local_owner("owner"),
            action: "session.create",
            resource: "session:new",
            context: AuthorizationContext {
                scope: "local-owner",
                safety_denied: true,
                test_mode: false,
            },
        };
        let (outcome, grant) = engine.authorize(&safety).unwrap();
        assert_eq!(outcome.decision, AuthorizationDecision::Deny);
        assert_eq!(outcome.reason_code, HARD_SAFETY_DENIAL);
        assert!(grant.is_none());

        let egress = AuthorizationRequest {
            principal: Principal::local_owner("owner"),
            action: "network.egress",
            resource: "https://example.invalid",
            context: AuthorizationContext::local("local-owner"),
        };
        let (outcome, grant) = engine.authorize(&egress).unwrap();
        assert_eq!(outcome.decision, AuthorizationDecision::Deny);
        assert_eq!(outcome.reason_code, STRICT_LOCAL_EGRESS_DENIAL);
        assert!(grant.is_none());
        drop(engine);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
