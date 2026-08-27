#![forbid(unsafe_code)]

mod authorization;
mod client_auth;
mod client_enrollment;
mod effect_execution;
mod operations;
pub mod policy_candidate;
mod resource;
mod startup;
mod synthetic_effect;

use std::error::Error;
use std::fmt;
use std::path::Path;

use golam_core::ClientId;
use golam_core::authority::{AuthorityLayout, AuthorityPathError};
use golam_core::paths::RuntimeLayout;

pub use authorization::{
    AuthorizationContext, AuthorizationDecision, AuthorizationError, AuthorizationOutcome,
    AuthorizationPolicy, AuthorizationRequest, BootstrapPolicy, DecisionId, DenyByDefault,
    PolicyDecision, Principal, PrincipalKind,
};
pub use client_auth::ClientAuthorityError;
pub use client_enrollment::{ClientEnrollmentError, EnrolledClientCredential};
pub use effect_execution::PreparedEffectDispatch;
pub use golam_ipc::credentials::GeneratedClientCredential;
pub use golam_ipc::lifecycle::{Authenticate, ClientKeyId, ConnectionId, Ready, ServerLifecycle};
pub use golam_ledger::checkpoint::{LoadedProjection, ProjectionSource};
pub use golam_ledger::clients::{ClientKind, ClientRecord};
pub use golam_ledger::dispatch::{
    EffectDispatchStoreError as EffectDispatchError, PrepareEffectDispatch,
    encode_effect_dependencies,
};
pub use golam_ledger::goal::GoalDocument;
pub use golam_ledger::recovery::{
    RecoveryError, RecoveryIssue, RecoveryIssueKind, RecoveryMode, RecoveryReport, RecoveryScanner,
};
pub use golam_ledger::session_read::SessionSummary;
pub use operations::{
    KernelAppendGoal, KernelCreateCheckpoint, KernelCreateFork, KernelCreateSession,
    KernelOperationError,
};
pub use resource::{ProtectedResourceError, UnprivilegedPath};
pub use startup::{KernelStartup, KernelStartupError, start_kernel};
pub use synthetic_effect::{
    CompleteSyntheticEffect, PrepareSyntheticEffect, ResolveSyntheticReconciliation,
    SyntheticEffectError, SyntheticEffectOutcome, SyntheticExecutionCompletion,
    SyntheticReconciliationContext, SyntheticReconciliationResult,
};

use authorization::AuthorizationEngine;
use client_auth::ClientAuthority;
use effect_execution::EffectExecutionAuthority;

#[derive(Debug)]
pub enum KernelError {
    AuthorityPath(AuthorityPathError),
    Authorization(AuthorizationError),
    AuthorizationDenied(AuthorizationOutcome),
    ClientAuthority(ClientAuthorityError),
    EffectDispatch(EffectDispatchError),
    Recovery(RecoveryError),
    RecoveryRequired(RecoveryReport),
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthorityPath(error) => write!(f, "kernel authority path error: {error}"),
            Self::Authorization(error) => write!(f, "kernel authorization error: {error}"),
            Self::AuthorizationDenied(outcome) => write!(
                f,
                "kernel authorization denied: decision={} reason={}",
                hex_decision_id(outcome.decision_id),
                outcome.reason_code
            ),
            Self::ClientAuthority(error) => write!(f, "kernel client authority error: {error}"),
            Self::EffectDispatch(error) => write!(f, "kernel effect dispatch error: {error}"),
            Self::Recovery(error) => write!(f, "kernel recovery scan error: {error}"),
            Self::RecoveryRequired(report) => write!(
                f,
                "kernel privileged service blocked by recovery mode {:?}",
                report.mode
            ),
        }
    }
}

impl Error for KernelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AuthorityPath(error) => Some(error),
            Self::Authorization(error) => Some(error),
            Self::ClientAuthority(error) => Some(error),
            Self::EffectDispatch(error) => Some(error),
            Self::Recovery(error) => Some(error),
            Self::AuthorizationDenied(_) | Self::RecoveryRequired(_) => None,
        }
    }
}

impl From<AuthorityPathError> for KernelError {
    fn from(value: AuthorityPathError) -> Self {
        Self::AuthorityPath(value)
    }
}

impl From<AuthorizationError> for KernelError {
    fn from(value: AuthorizationError) -> Self {
        Self::Authorization(value)
    }
}

impl From<ClientAuthorityError> for KernelError {
    fn from(value: ClientAuthorityError) -> Self {
        Self::ClientAuthority(value)
    }
}

impl From<EffectDispatchError> for KernelError {
    fn from(value: EffectDispatchError) -> Self {
        Self::EffectDispatch(value)
    }
}

impl From<RecoveryError> for KernelError {
    fn from(value: RecoveryError) -> Self {
        Self::Recovery(value)
    }
}

/// Privileged mutations are available only through this API. The authority
/// implementation modules and authority-bearing grants are intentionally not
/// part of the public crate surface.
///
/// ```compile_fail
/// use golam_kernel::client_auth::ClientAuthority;
/// ```
///
/// ```compile_fail
/// use golam_kernel::authorization::AuthorityGrant;
/// ```
pub struct KernelApi<P> {
    runtime: RuntimeLayout,
    authority: AuthorityLayout,
    authorization: AuthorizationEngine<P>,
    clients: ClientAuthority,
    effects: EffectExecutionAuthority,
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn open(runtime: &RuntimeLayout, policy: P) -> Result<Self, KernelError> {
        let report = RecoveryScanner::scan(runtime)?;
        if !report.privileged_service_allowed() {
            return Err(KernelError::RecoveryRequired(report));
        }
        Self::open_after_recovery(runtime, policy)
    }

    pub(crate) fn open_after_recovery(
        runtime: &RuntimeLayout,
        policy: P,
    ) -> Result<Self, KernelError> {
        let authority = AuthorityLayout::initialize(runtime)?;
        let authorization = AuthorizationEngine::open(&authority, policy)?;
        let clients = ClientAuthority::open(&authority)?;
        let effects = EffectExecutionAuthority::open(&authority)?;
        Ok(Self {
            runtime: runtime.clone(),
            authority,
            authorization,
            clients,
            effects,
        })
    }

    pub fn authorize(
        &mut self,
        request: &AuthorizationRequest<'_>,
    ) -> Result<AuthorizationOutcome, KernelError> {
        let (outcome, grant) = self.authorization.authorize(request)?;
        if let Some(grant) = grant {
            debug_assert_eq!(grant.decision_id(), outcome.decision_id);
        }
        Ok(outcome)
    }

    pub fn enroll_generated_client(
        &mut self,
        principal: Principal<'_>,
        generated: &GeneratedClientCredential,
        kind: ClientKind,
        enrolled_at: &str,
        scope: &str,
    ) -> Result<ClientRecord, KernelError> {
        let resource = format!("client:{}", generated.client_id.0);
        self.require_authority(&AuthorizationRequest {
            principal,
            action: "client.enroll",
            resource: &resource,
            context: AuthorizationContext::local(scope),
        })?;
        Ok(self
            .clients
            .enroll_generated(generated, kind, principal.subject, enrolled_at)?)
    }

    pub fn revoke_client(
        &mut self,
        principal: Principal<'_>,
        client_id: ClientId,
        revoked_at: &str,
        scope: &str,
    ) -> Result<ClientRecord, KernelError> {
        let resource = format!("client:{}", client_id.0);
        self.require_authority(&AuthorizationRequest {
            principal,
            action: "client.revoke",
            resource: &resource,
            context: AuthorizationContext::local(scope),
        })?;
        Ok(self.clients.revoke(client_id, revoked_at)?)
    }

    pub fn authenticate_registered_client(
        &mut self,
        lifecycle: &mut ServerLifecycle,
        connection_id: ConnectionId,
        client_id: ClientId,
        authenticate: Authenticate,
        authenticated_at: &str,
    ) -> Result<Ready, KernelError> {
        Ok(self.clients.authenticate_registered(
            lifecycle,
            connection_id,
            client_id,
            authenticate,
            authenticated_at,
        )?)
    }

    pub fn reject_unauthenticated_request(
        &mut self,
        lifecycle: &mut ServerLifecycle,
        connection_id: ConnectionId,
        client_id: ClientId,
        key_id: Option<ClientKeyId>,
        detected_at: &str,
    ) -> Result<(), KernelError> {
        Ok(self.clients.reject_unauthenticated_request(
            lifecycle,
            connection_id,
            client_id,
            key_id,
            detected_at,
        )?)
    }

    pub fn prepare_effect_dispatch(
        &mut self,
        input: PrepareEffectDispatch<'_>,
    ) -> Result<PreparedEffectDispatch, KernelError> {
        Ok(self.effects.prepare(input)?)
    }

    pub fn network_egress_authorize(
        &mut self,
        principal: Principal<'_>,
        resource: &str,
        scope: &str,
    ) -> Result<AuthorizationOutcome, KernelError> {
        self.authorize(&AuthorizationRequest {
            principal,
            action: "network.egress",
            resource,
            context: AuthorizationContext::local(scope),
        })
    }

    pub fn admit_unprivileged_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<UnprivilegedPath, ProtectedResourceError> {
        resource::admit_unprivileged_path(&self.runtime, &self.authority, path)
    }

    pub fn authorization_decision_count(&self) -> Result<usize, KernelError> {
        Ok(self.authorization.records()?.len())
    }

    fn require_authority(&mut self, request: &AuthorizationRequest<'_>) -> Result<(), KernelError> {
        let (outcome, grant) = self.authorization.authorize(request)?;
        match grant {
            Some(grant) => {
                debug_assert_eq!(grant.decision_id(), outcome.decision_id);
                Ok(())
            }
            None => Err(KernelError::AuthorizationDenied(outcome)),
        }
    }
}

fn hex_decision_id(id: DecisionId) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(32);
    for byte in id.0 {
        value.push(char::from(HEX[(byte >> 4) as usize]));
        value.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::authority::AuthorityLayout;
    use golam_core::{EffectAttemptId, EffectId, EffectTransitionId, EventId, SessionId};
    use golam_ipc::credentials::ClientCredentialStore;
    use golam_ledger::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    fn runtime() -> RuntimeLayout {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        RuntimeLayout::initialize(
            std::env::temp_dir().join(format!("golam-kernel-api-{}-{t}-{n}", std::process::id())),
        )
        .unwrap()
    }

    #[test]
    fn kernel_api_returns_decisions_not_authority_tokens() {
        let runtime = runtime();
        let mut kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let outcome = kernel
            .authorize(&AuthorizationRequest {
                principal: Principal::local_owner("owner"),
                action: "session.create",
                resource: "session:new",
                context: AuthorizationContext::local("local-owner"),
            })
            .unwrap();
        assert_eq!(outcome.decision, AuthorizationDecision::Allow);
        assert_eq!(kernel.authorization_decision_count().unwrap(), 1);
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn strict_local_egress_is_denied_even_with_permissive_policy() {
        struct Permissive;
        impl AuthorizationPolicy for Permissive {
            fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
                PolicyDecision::allow("permissive")
            }
        }

        let runtime = runtime();
        let mut kernel = KernelApi::open(&runtime, Permissive).unwrap();
        let outcome = kernel
            .network_egress_authorize(
                Principal::local_owner("owner"),
                "https://example.invalid",
                "local-owner",
            )
            .unwrap();
        assert_eq!(outcome.decision, AuthorizationDecision::Deny);
        assert_eq!(outcome.reason_code, "strict_local_egress_denied");
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn generic_path_admission_cannot_target_authority_state() {
        let runtime = runtime();
        let kernel = KernelApi::open(&runtime, DenyByDefault).unwrap();
        let authority_db = runtime.data_dir.join("authority").join("golam.db");
        assert!(matches!(
            kernel.admit_unprivileged_path(&authority_db),
            Err(ProtectedResourceError::AuthorityReserved(_))
        ));
        assert!(
            kernel
                .admit_unprivileged_path(runtime.artifact_dir.join("safe.txt"))
                .is_ok()
        );
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn client_enrollment_and_revocation_require_kernel_authorization() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let store = ClientCredentialStore::new(&authority);
        let generated = store.generate(ClientId(800)).unwrap();
        let mut kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();

        assert!(matches!(
            kernel.enroll_generated_client(
                Principal::enrolled_client("owner", ClientId(9)),
                &generated,
                ClientKind::Test,
                "2026-08-25T05:20:00Z",
                "local-client",
            ),
            Err(KernelError::AuthorizationDenied(_))
        ));
        let enrolled = kernel
            .enroll_generated_client(
                Principal::local_owner("owner"),
                &generated,
                ClientKind::Test,
                "2026-08-25T05:21:00Z",
                "local-owner",
            )
            .unwrap();
        assert_eq!(enrolled.owner_principal, "owner");
        assert!(matches!(
            kernel.revoke_client(
                Principal::enrolled_client("owner", ClientId(9)),
                generated.client_id,
                "2026-08-25T05:22:00Z",
                "local-client",
            ),
            Err(KernelError::AuthorizationDenied(_))
        ));
        kernel
            .revoke_client(
                Principal::local_owner("owner"),
                generated.client_id,
                "2026-08-25T05:23:00Z",
                "local-owner",
            )
            .unwrap();
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn prepared_effect_dispatch_is_kernel_minted_after_durable_attempt() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let dependencies = encode_effect_dependencies(&[]).unwrap();
        let effect_id = EffectId(900);
        let attempt_id = EffectAttemptId(901);
        let mut effects = EffectStore::open(&authority).unwrap();
        effects
            .propose(ProposeEffect {
                effect_id,
                session_id: SessionId(1),
                requested_by: "owner",
                action: "sim.write",
                resource: "sim:item",
                risk_class: "synthetic",
                execution_semantics: "at_most_once",
                idempotency_key: None,
                preconditions: b"[]",
                dependencies: &dependencies,
                payload_hash: [7; 32],
                proposed_event_id: EventId(902),
                transition_id: EffectTransitionId(903),
            })
            .unwrap();
        let authorized = effects
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(904),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: Some("test_authorized"),
                evidence_ref: None,
                event_id: EventId(905),
            })
            .unwrap();
        drop(effects);

        let mut kernel = KernelApi::open(&runtime, DenyByDefault).unwrap();
        let prepared = kernel
            .prepare_effect_dispatch(PrepareEffectDispatch {
                effect_id,
                attempt_id,
                transition_id: EffectTransitionId(906),
                handler_id: "sim-at-most-once-write",
                handler_version: "1",
                dispatch_token: b"dispatch-901",
                started_at: "2026-08-25T10:10:00Z",
                event_id: EventId(907),
            })
            .unwrap();
        assert_eq!(prepared.effect_id(), effect_id);
        assert_eq!(prepared.attempt_id(), attempt_id);
        assert_eq!(prepared.started_global_seq(), authorized.global_seq);
        assert!(prepared.executing_global_seq() > prepared.started_global_seq());
        drop(kernel);

        let effects = EffectStore::open(&authority).unwrap();
        assert_eq!(effects.attempt_count(effect_id).unwrap(), 1);
        assert_eq!(
            effects.current_state(effect_id).unwrap().as_deref(),
            Some("executing")
        );
        drop(effects);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn unprivileged_workspace_crates_do_not_link_the_ledger_directly() {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("kernel crate lives under workspace/crates");
        for manifest in [
            "crates/golam-ipc/Cargo.toml",
            "crates/golamd/Cargo.toml",
            "crates/golam/Cargo.toml",
        ] {
            let text = fs::read_to_string(workspace.join(manifest)).unwrap();
            assert!(
                !text.contains("golam-ledger"),
                "unprivileged manifest links privileged ledger directly: {manifest}"
            );
        }
    }
}
