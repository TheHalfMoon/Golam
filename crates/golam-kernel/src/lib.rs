#![forbid(unsafe_code)]

mod authorization;
mod resource;

use std::error::Error;
use std::fmt;
use std::path::Path;

use golam_core::authority::{AuthorityLayout, AuthorityPathError};
use golam_core::paths::RuntimeLayout;

pub use authorization::{
    AuthorizationContext, AuthorizationDecision, AuthorizationError, AuthorizationOutcome,
    AuthorizationPolicy, AuthorizationRequest, BootstrapPolicy, DecisionId, DenyByDefault,
    PolicyDecision, Principal, PrincipalKind,
};
pub use resource::{ProtectedResourceError, UnprivilegedPath};

use authorization::AuthorizationEngine;

#[derive(Debug)]
pub enum KernelError {
    AuthorityPath(AuthorityPathError),
    Authorization(AuthorizationError),
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthorityPath(error) => write!(f, "kernel authority path error: {error}"),
            Self::Authorization(error) => write!(f, "kernel authorization error: {error}"),
        }
    }
}

impl Error for KernelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AuthorityPath(error) => Some(error),
            Self::Authorization(error) => Some(error),
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

pub struct KernelApi<P> {
    runtime: RuntimeLayout,
    authority: AuthorityLayout,
    authorization: AuthorizationEngine<P>,
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn open(runtime: &RuntimeLayout, policy: P) -> Result<Self, KernelError> {
        let authority = AuthorityLayout::initialize(runtime)?;
        let authorization = AuthorizationEngine::open(&authority, policy)?;
        Ok(Self {
            runtime: runtime.clone(),
            authority,
            authorization,
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
}

#[cfg(test)]
mod tests {
    use super::*;
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
        RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-kernel-api-{}-{t}-{n}",
            std::process::id()
        )))
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
}
