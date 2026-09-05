#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{CanonicalEncoder, ClientId, CoreError};
use golam_ledger::authorization::{
    AppendAuthorizationDecision, AuthorizationAuditError, AuthorizationAuditLog,
    AuthorizationDecisionEvidence, AuthorizationDecisionKind, StoredAuthorizationDecision,
};

const HARD_SAFETY_DENIAL: &str = "safety_denial_monotonic";
const STRICT_LOCAL_EGRESS_DENIAL: &str = "strict_local_egress_denied";
const POLICY_INPUT_DOMAIN: &[u8] = b"golam:authorization-policy-input:v1";
const MAX_PRINCIPAL_SUBJECT_BYTES: usize = 256;
const MAX_ACTION_BYTES: usize = 128;
const MAX_RESOURCE_BYTES: usize = 2048;
const MAX_CONTEXT_SCOPE_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrincipalKind {
    LocalOwner,
    EnrolledClient,
    KernelService,
    Test,
    Unauthenticated,
}

impl PrincipalKind {
    const fn code(self) -> u8 {
        match self {
            Self::LocalOwner => 1,
            Self::EnrolledClient => 2,
            Self::KernelService => 3,
            Self::Test => 4,
            Self::Unauthenticated => 5,
        }
    }
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

    pub(crate) fn audit_subject(self) -> String {
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
struct NormalizedAuthorizationRequest<'a> {
    request: AuthorizationRequest<'a>,
}

impl<'a> NormalizedAuthorizationRequest<'a> {
    fn new(request: &AuthorizationRequest<'a>) -> Result<Self, AuthorizationError> {
        validate_principal(request.principal)?;
        validate_action(request.action)?;
        validate_resource(request.resource)?;
        validate_scope(request.context.scope)?;
        Ok(Self { request: *request })
    }

    const fn request(&self) -> &AuthorizationRequest<'a> {
        &self.request
    }

    fn policy_input_bytes(&self) -> Result<Vec<u8>, CoreError> {
        let request = self.request;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(POLICY_INPUT_DOMAIN)?;
        encoder.push_u8(request.principal.kind.code());
        encoder.push_bytes(request.principal.subject.as_bytes())?;
        match request.principal.client_id {
            Some(client_id) => {
                encoder.push_u8(1);
                encoder.push_u128(client_id.0);
            }
            None => encoder.push_u8(0),
        }
        encoder.push_bytes(request.action.as_bytes())?;
        encoder.push_bytes(request.resource.as_bytes())?;
        encoder.push_bytes(request.context.scope.as_bytes())?;
        encoder.push_u8(u8::from(request.context.safety_denied));
        encoder.push_u8(u8::from(request.context.test_mode));
        Ok(encoder.finish())
    }
}

fn validate_principal(principal: Principal<'_>) -> Result<(), AuthorizationError> {
    validate_bounded_text(
        principal.subject,
        MAX_PRINCIPAL_SUBJECT_BYTES,
        AuthorizationError::EmptyPrincipal,
        AuthorizationError::PrincipalTooLarge,
        AuthorizationError::NonCanonicalPrincipal,
    )?;
    match (principal.kind, principal.client_id) {
        (PrincipalKind::EnrolledClient, Some(_)) => Ok(()),
        (PrincipalKind::EnrolledClient, None) => Err(AuthorizationError::InvalidPrincipalShape),
        (_, Some(_)) => Err(AuthorizationError::InvalidPrincipalShape),
        (_, None) => Ok(()),
    }
}

fn validate_action(action: &str) -> Result<(), AuthorizationError> {
    if action.is_empty() {
        return Err(AuthorizationError::EmptyAction);
    }
    if action.len() > MAX_ACTION_BYTES {
        return Err(AuthorizationError::ActionTooLarge);
    }
    let bytes = action.as_bytes();
    let first_is_canonical = bytes.first().is_some_and(u8::is_ascii_lowercase);
    let last_is_canonical = bytes.last().is_some_and(u8::is_ascii_alphanumeric);
    if !first_is_canonical
        || !last_is_canonical
        || bytes.windows(2).any(|pair| pair == b"..")
        || bytes.iter().any(|byte| {
            !(byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(*byte, b'.' | b'_' | b'-'))
        })
    {
        return Err(AuthorizationError::NonCanonicalAction);
    }
    Ok(())
}

fn validate_resource(resource: &str) -> Result<(), AuthorizationError> {
    validate_bounded_text(
        resource,
        MAX_RESOURCE_BYTES,
        AuthorizationError::EmptyResource,
        AuthorizationError::ResourceTooLarge,
        AuthorizationError::NonCanonicalResource,
    )
}

fn validate_scope(scope: &str) -> Result<(), AuthorizationError> {
    if scope.is_empty() {
        return Err(AuthorizationError::EmptyContextScope);
    }
    if scope.len() > MAX_CONTEXT_SCOPE_BYTES {
        return Err(AuthorizationError::ContextScopeTooLarge);
    }
    let bytes = scope.as_bytes();
    let first_is_canonical = bytes.first().is_some_and(u8::is_ascii_lowercase);
    let last_is_canonical = bytes.last().is_some_and(u8::is_ascii_alphanumeric);
    if !first_is_canonical
        || !last_is_canonical
        || bytes.iter().any(|byte| {
            !(byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(*byte, b'.' | b'_' | b'-' | b':'))
        })
    {
        return Err(AuthorizationError::NonCanonicalContextScope);
    }
    Ok(())
}

fn validate_bounded_text(
    value: &str,
    max_bytes: usize,
    empty_error: AuthorizationError,
    too_large_error: AuthorizationError,
    noncanonical_error: AuthorizationError,
) -> Result<(), AuthorizationError> {
    if value.is_empty() {
        return Err(empty_error);
    }
    if value.len() > max_bytes {
        return Err(too_large_error);
    }
    if value.trim() != value || value.chars().any(char::is_control) {
        return Err(noncanonical_error);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationDecision {
    Allow,
    Deny,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PolicyEvaluationEvidence {
    pub policy_bundle_id: Option<[u8; 16]>,
    pub policy_bundle_hash: Option<[u8; 32]>,
    pub matched_rule_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyDecision {
    pub decision: AuthorizationDecision,
    pub reason_code: &'static str,
    pub evidence: PolicyEvaluationEvidence,
}

impl PolicyDecision {
    pub fn allow(reason_code: &'static str) -> Self {
        Self {
            decision: AuthorizationDecision::Allow,
            reason_code,
            evidence: PolicyEvaluationEvidence::default(),
        }
    }

    pub fn deny(reason_code: &'static str) -> Self {
        Self {
            decision: AuthorizationDecision::Deny,
            reason_code,
            evidence: PolicyEvaluationEvidence::default(),
        }
    }

    pub fn with_policy_evidence(
        mut self,
        policy_bundle_id: [u8; 16],
        policy_bundle_hash: [u8; 32],
        matched_rule_ids: Vec<String>,
    ) -> Self {
        self.evidence = PolicyEvaluationEvidence {
            policy_bundle_id: Some(policy_bundle_id),
            policy_bundle_hash: Some(policy_bundle_hash),
            matched_rule_ids,
        };
        self
    }
}

pub trait AuthorizationPolicy {
    fn authorize(&self, request: &AuthorizationRequest<'_>) -> PolicyDecision;

    fn authorize_normalized(
        &self,
        request: &AuthorizationRequest<'_>,
        _canonical_policy_input: &[u8],
    ) -> PolicyDecision {
        self.authorize(request)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HardGuardOutcome {
    Pass,
    Deny(&'static str),
}

impl HardGuardOutcome {
    const fn evidence_result(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Deny(reason) => reason,
        }
    }
}

fn evaluate_hard_guards(request: &AuthorizationRequest<'_>) -> HardGuardOutcome {
    if request.context.safety_denied {
        HardGuardOutcome::Deny(HARD_SAFETY_DENIAL)
    } else if is_network_egress(request.action) {
        HardGuardOutcome::Deny(STRICT_LOCAL_EGRESS_DENIAL)
    } else {
        HardGuardOutcome::Pass
    }
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
    CanonicalInput(CoreError),
    EmptyPrincipal,
    EmptyAction,
    EmptyResource,
    EmptyContextScope,
    PrincipalTooLarge,
    ActionTooLarge,
    ResourceTooLarge,
    ContextScopeTooLarge,
    NonCanonicalPrincipal,
    NonCanonicalAction,
    NonCanonicalResource,
    NonCanonicalContextScope,
    InvalidPrincipalShape,
}

impl fmt::Display for AuthorizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Audit(error) => write!(f, "authorization audit failed: {error}"),
            Self::CanonicalInput(error) => {
                write!(f, "authorization canonical input encoding failed: {error}")
            }
            Self::EmptyPrincipal => f.write_str("authorization principal is empty"),
            Self::EmptyAction => f.write_str("authorization action is empty"),
            Self::EmptyResource => f.write_str("authorization resource is empty"),
            Self::EmptyContextScope => f.write_str("authorization context scope is empty"),
            Self::PrincipalTooLarge => f.write_str("authorization principal exceeds bounded size"),
            Self::ActionTooLarge => f.write_str("authorization action exceeds bounded size"),
            Self::ResourceTooLarge => f.write_str("authorization resource exceeds bounded size"),
            Self::ContextScopeTooLarge => {
                f.write_str("authorization context scope exceeds bounded size")
            }
            Self::NonCanonicalPrincipal => f.write_str("authorization principal is not canonical"),
            Self::NonCanonicalAction => f.write_str("authorization action is not canonical"),
            Self::NonCanonicalResource => f.write_str("authorization resource is not canonical"),
            Self::NonCanonicalContextScope => {
                f.write_str("authorization context scope is not canonical")
            }
            Self::InvalidPrincipalShape => {
                f.write_str("authorization principal kind/client binding is invalid")
            }
        }
    }
}

impl Error for AuthorizationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Audit(error) => Some(error),
            Self::CanonicalInput(error) => Some(error),
            _ => None,
        }
    }
}

impl From<AuthorizationAuditError> for AuthorizationError {
    fn from(value: AuthorizationAuditError) -> Self {
        Self::Audit(value)
    }
}

impl From<CoreError> for AuthorizationError {
    fn from(value: CoreError) -> Self {
        Self::CanonicalInput(value)
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
        let normalized = NormalizedAuthorizationRequest::new(request)?;
        let request = normalized.request();
        let canonical_policy_input = normalized.policy_input_bytes()?;
        let hard_guard = evaluate_hard_guards(request);
        let policy_decision = match hard_guard {
            HardGuardOutcome::Pass if request.principal.kind == PrincipalKind::Unauthenticated => {
                PolicyDecision::deny("unauthenticated_principal_denied")
            }
            HardGuardOutcome::Pass => self
                .policy
                .authorize_normalized(request, &canonical_policy_input),
            HardGuardOutcome::Deny(reason) => PolicyDecision::deny(reason),
        };
        let principal = request.principal.audit_subject();
        let context = request.context.audit_text();
        let decision = match policy_decision.decision {
            AuthorizationDecision::Allow => AuthorizationDecisionKind::Allow,
            AuthorizationDecision::Deny => AuthorizationDecisionKind::Deny,
        };
        let matched_rule_ids = policy_decision
            .evidence
            .matched_rule_ids
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let stored = self.audit.append(AppendAuthorizationDecision {
            principal: &principal,
            action: request.action,
            resource: request.resource,
            context: &context,
            evidence: AuthorizationDecisionEvidence {
                hard_guard_result: hard_guard.evidence_result(),
                lease_id: None,
                lease_generation: None,
                policy_bundle_id: policy_decision.evidence.policy_bundle_id.as_ref(),
                policy_bundle_hash: policy_decision.evidence.policy_bundle_hash.as_ref(),
                matched_rule_ids: &matched_rule_ids,
                approval_id: None,
            },
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

    pub(crate) fn record(
        &self,
        decision_id: [u8; 16],
    ) -> Result<Option<StoredAuthorizationDecision>, AuthorizationError> {
        Ok(self.audit.record(decision_id)?)
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
            | "policy.validate"
            | "authority.qualify"
            | "authority.explain"
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
            | "policy.validate"
            | "authority.qualify"
            | "authority.explain"
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

    fn owner_request<'a>(
        action: &'a str,
        resource: &'a str,
        scope: &'a str,
    ) -> AuthorizationRequest<'a> {
        AuthorizationRequest {
            principal: Principal::local_owner("owner"),
            action,
            resource,
            context: AuthorizationContext::local(scope),
        }
    }

    #[test]
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

    #[test]
    fn bootstrap_policy_is_explicit_and_audited() {
        let (runtime, authority) = authority();
        let mut engine = AuthorizationEngine::open(&authority, BootstrapPolicy::default()).unwrap();
        let allow = owner_request("session.create", "session:new", "local-owner");
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
        let records = engine.records().unwrap();
        assert_eq!(records.len(), 2);
        assert!(
            records
                .iter()
                .all(|record| record.hard_guard_result == "pass")
        );
        assert!(
            records
                .iter()
                .all(|record| record.authority_evidence_version == 2)
        );
        drop(engine);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn canonical_policy_input_is_stable_and_field_sensitive() {
        let request = owner_request("session.create", "session:new", "local-owner");
        let normalized = NormalizedAuthorizationRequest::new(&request).unwrap();
        let first = normalized.policy_input_bytes().unwrap();
        assert_eq!(first, normalized.policy_input_bytes().unwrap());
        assert!(first.starts_with(&[0, 0, 0, 35]));
        assert!(
            first
                .windows(POLICY_INPUT_DOMAIN.len())
                .any(|window| window == POLICY_INPUT_DOMAIN)
        );

        let changed = owner_request("session.read", "session:new", "local-owner");
        let changed = NormalizedAuthorizationRequest::new(&changed).unwrap();
        assert_ne!(first, changed.policy_input_bytes().unwrap());
    }

    #[test]
    fn bounded_canonical_input_vectors_fail_closed() {
        let invalid_actions = [
            "Session.create",
            ".session",
            "session.",
            "session..read",
            "session read",
        ];
        for action in invalid_actions {
            assert!(matches!(
                NormalizedAuthorizationRequest::new(&owner_request(
                    action,
                    "session:new",
                    "local-owner"
                )),
                Err(AuthorizationError::NonCanonicalAction)
            ));
        }

        assert!(matches!(
            NormalizedAuthorizationRequest::new(&owner_request(
                &"a".repeat(MAX_ACTION_BYTES + 1),
                "session:new",
                "local-owner"
            )),
            Err(AuthorizationError::ActionTooLarge)
        ));
        assert!(matches!(
            NormalizedAuthorizationRequest::new(&owner_request(
                "session.read",
                &"r".repeat(MAX_RESOURCE_BYTES + 1),
                "local-owner"
            )),
            Err(AuthorizationError::ResourceTooLarge)
        ));
        assert!(matches!(
            NormalizedAuthorizationRequest::new(&owner_request(
                "session.read",
                " session:new",
                "local-owner"
            )),
            Err(AuthorizationError::NonCanonicalResource)
        ));
        assert!(matches!(
            NormalizedAuthorizationRequest::new(&owner_request(
                "session.read",
                "session:new",
                "Local-owner"
            )),
            Err(AuthorizationError::NonCanonicalContextScope)
        ));

        let malformed = AuthorizationRequest {
            principal: Principal {
                kind: PrincipalKind::LocalOwner,
                subject: "owner",
                client_id: Some(ClientId(9)),
            },
            action: "session.read",
            resource: "session:new",
            context: AuthorizationContext::local("local-owner"),
        };
        assert!(matches!(
            NormalizedAuthorizationRequest::new(&malformed),
            Err(AuthorizationError::InvalidPrincipalShape)
        ));
    }

    #[test]
    fn hard_guards_dominate_without_calling_policy() {
        struct MustNotRun;
        impl AuthorizationPolicy for MustNotRun {
            fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
                panic!("policy evaluator must not run after a hard denial");
            }
        }

        let (runtime, authority) = authority();
        let mut engine = AuthorizationEngine::open(&authority, MustNotRun).unwrap();
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

        let egress = owner_request("network.egress", "https://example.invalid", "local-owner");
        let (outcome, grant) = engine.authorize(&egress).unwrap();
        assert_eq!(outcome.decision, AuthorizationDecision::Deny);
        assert_eq!(outcome.reason_code, STRICT_LOCAL_EGRESS_DENIAL);
        assert!(grant.is_none());
        let records = engine.records().unwrap();
        assert_eq!(records[0].hard_guard_result, HARD_SAFETY_DENIAL);
        assert_eq!(records[1].hard_guard_result, STRICT_LOCAL_EGRESS_DENIAL);
        assert!(
            records
                .iter()
                .all(|record| record.policy_bundle_id.is_none())
        );
        drop(engine);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn strict_local_egress_dominates_downstream_permit_policy_evaluation() {
        static DOWNSTREAM_CALLS: AtomicU64 = AtomicU64::new(0);

        struct DownstreamPermitPolicy;
        impl AuthorizationPolicy for DownstreamPermitPolicy {
            fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
                DOWNSTREAM_CALLS.fetch_add(1, Ordering::SeqCst);
                PolicyDecision::allow("downstream_egress_permit_would_allow").with_policy_evidence(
                    [9_u8; 16],
                    [10_u8; 32],
                    vec!["rule:egress-permit".to_owned()],
                )
            }
        }

        DOWNSTREAM_CALLS.store(0, Ordering::SeqCst);
        let (runtime, authority) = authority();
        let mut engine = AuthorizationEngine::open(&authority, DownstreamPermitPolicy).unwrap();
        let request = owner_request(
            "network.egress.connect",
            "https://example.invalid:443",
            "local-owner",
        );

        let (outcome, grant) = engine.authorize(&request).unwrap();

        assert_eq!(outcome.decision, AuthorizationDecision::Deny);
        assert_eq!(outcome.reason_code, STRICT_LOCAL_EGRESS_DENIAL);
        assert!(grant.is_none());
        assert_eq!(DOWNSTREAM_CALLS.load(Ordering::SeqCst), 0);

        let records = engine.records().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].hard_guard_result, STRICT_LOCAL_EGRESS_DENIAL);
        assert!(records[0].policy_bundle_id.is_none());
        assert!(records[0].policy_bundle_hash.is_none());
        assert!(records[0].matched_rule_ids.is_empty());

        drop(engine);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn normalized_non_hard_request_reaches_policy() {
        struct AllowRead;
        impl AuthorizationPolicy for AllowRead {
            fn authorize(&self, request: &AuthorizationRequest<'_>) -> PolicyDecision {
                assert_eq!(request.action, "session.read");
                assert_eq!(request.context.scope, "local-owner");
                PolicyDecision::allow("normalized_test_allow")
            }

            fn authorize_normalized(
                &self,
                request: &AuthorizationRequest<'_>,
                canonical_policy_input: &[u8],
            ) -> PolicyDecision {
                assert!(canonical_policy_input.starts_with(&[0, 0, 0, 35]));
                self.authorize(request)
            }
        }

        let (runtime, authority) = authority();
        let mut engine = AuthorizationEngine::open(&authority, AllowRead).unwrap();
        let request = owner_request("session.read", "session:1", "local-owner");
        let (outcome, grant) = engine.authorize(&request).unwrap();
        assert_eq!(outcome.decision, AuthorizationDecision::Allow);
        assert!(grant.is_some());
        let records = engine.records().unwrap();
        assert_eq!(records[0].hard_guard_result, "pass");
        drop(engine);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn policy_bundle_hash_and_stable_rule_refs_are_bound_to_decision_evidence() {
        struct EvidencePolicy;
        impl AuthorizationPolicy for EvidencePolicy {
            fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
                PolicyDecision::allow("cedar_test_allow").with_policy_evidence(
                    [3_u8; 16],
                    [4_u8; 32],
                    vec!["rule:z".to_owned(), "rule:a".to_owned()],
                )
            }
        }

        let (runtime, authority) = authority();
        let mut engine = AuthorizationEngine::open(&authority, EvidencePolicy).unwrap();
        let request = owner_request("session.read", "session:1", "local-owner");
        let (outcome, grant) = engine.authorize(&request).unwrap();
        assert_eq!(outcome.decision, AuthorizationDecision::Allow);
        assert!(grant.is_some());
        let records = engine.records().unwrap();
        assert_eq!(records[0].policy_bundle_id, Some([3_u8; 16]));
        assert_eq!(records[0].policy_bundle_hash, Some([4_u8; 32]));
        assert_eq!(records[0].matched_rule_ids, vec!["rule:a", "rule:z"]);
        drop(engine);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
