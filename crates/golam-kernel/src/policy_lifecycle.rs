#![forbid(unsafe_code)]

#[path = "approval_lifecycle.rs"]
pub mod approval_lifecycle;
pub use approval_lifecycle::{ApprovalMutationError, IssueApproval};
#[path = "capability_lease_effect.rs"]
pub mod capability_lease_effect;

use std::error::Error;
use std::fmt;

use golam_core::EffectId;
use golam_ledger::policy::{
    POLICY_ACTIVATE_ACTION, POLICY_STAGE_ACTION, PolicyActivationRecord, PolicyBundleId,
    PolicyBundleRecord, PolicyLifecycleError, PolicyStore, policy_bundle_resource,
    prepare_policy_bundle,
};

use crate::policy_candidate::{CandidatePolicyError, validate_policy_candidate};
use crate::{
    AuthorizationContext, AuthorizationError, AuthorizationOutcome, AuthorizationPolicy,
    AuthorizationRequest, KernelApi, Principal,
};

pub struct StagePolicyBundle<'a> {
    pub principal: Principal<'a>,
    pub schema_version: u64,
    pub policy_source: &'a str,
    pub schema_source: &'a str,
    pub scope: &'a str,
}

pub struct ActivatePolicyBundle<'a> {
    pub principal: Principal<'a>,
    pub policy_bundle_id: PolicyBundleId,
    pub approval_id: [u8; 16],
    pub activation_effect_id: EffectId,
    pub scope: &'a str,
}

#[derive(Debug)]
pub enum PolicyMutationError {
    Candidate(CandidatePolicyError),
    Authorization(AuthorizationError),
    AuthorizationDenied(AuthorizationOutcome),
    Lifecycle(PolicyLifecycleError),
}

impl fmt::Display for PolicyMutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Candidate(error) => write!(f, "policy mutation candidate rejected: {error}"),
            Self::Authorization(error) => {
                write!(f, "policy mutation authorization failed: {error}")
            }
            Self::AuthorizationDenied(outcome) => write!(
                f,
                "policy mutation denied: decision={:?} reason={}",
                outcome.decision_id, outcome.reason_code
            ),
            Self::Lifecycle(error) => write!(f, "policy mutation lifecycle failed: {error}"),
        }
    }
}

impl Error for PolicyMutationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Candidate(error) => Some(error),
            Self::Authorization(error) => Some(error),
            Self::Lifecycle(error) => Some(error),
            Self::AuthorizationDenied(_) => None,
        }
    }
}

impl From<CandidatePolicyError> for PolicyMutationError {
    fn from(value: CandidatePolicyError) -> Self {
        Self::Candidate(value)
    }
}

impl From<AuthorizationError> for PolicyMutationError {
    fn from(value: AuthorizationError) -> Self {
        Self::Authorization(value)
    }
}

impl From<PolicyLifecycleError> for PolicyMutationError {
    fn from(value: PolicyLifecycleError) -> Self {
        Self::Lifecycle(value)
    }
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn stage_policy_bundle(
        &mut self,
        input: StagePolicyBundle<'_>,
    ) -> Result<PolicyBundleRecord, PolicyMutationError> {
        let validated = validate_policy_candidate(input.policy_source, input.schema_source)?;
        let (policy_source, schema_source) = validated.into_sources();
        let prepared = prepare_policy_bundle(input.schema_version, &policy_source, &schema_source)?;
        let resource = policy_bundle_resource(prepared.policy_bundle_id());
        let authority_decision_id = self.authorize_policy_mutation(
            input.principal,
            POLICY_STAGE_ACTION,
            &resource,
            input.scope,
        )?;
        let mut store = PolicyStore::open(&self.authority)?;
        Ok(store.stage_prepared(prepared, authority_decision_id)?)
    }

    pub fn activate_policy_bundle(
        &mut self,
        input: ActivatePolicyBundle<'_>,
    ) -> Result<PolicyActivationRecord, PolicyMutationError> {
        let resource = policy_bundle_resource(input.policy_bundle_id);
        let authority_decision_id = self.authorize_policy_mutation(
            input.principal,
            POLICY_ACTIVATE_ACTION,
            &resource,
            input.scope,
        )?;
        let mut store = PolicyStore::open(&self.authority)?;
        Ok(store.activate(
            input.policy_bundle_id,
            authority_decision_id,
            input.approval_id,
            input.activation_effect_id,
        )?)
    }

    fn authorize_policy_mutation(
        &mut self,
        principal: Principal<'_>,
        action: &str,
        resource: &str,
        scope: &str,
    ) -> Result<[u8; 16], PolicyMutationError> {
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
                Ok(grant.decision_id().0)
            }
            None => Err(PolicyMutationError::AuthorizationDenied(outcome)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuthorizationDecision, DenyByDefault, PolicyDecision};
    use golam_core::paths::RuntimeLayout;
    use golam_ledger::policy::PolicyStore;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    const SCHEMA: &str = "entity User;\nentity Photo;\naction view appliesTo { principal: [User], resource: [Photo] };\n";
    const POLICY: &str =
        "permit(principal is User, action == Action::\"view\", resource is Photo);\n";

    static N: AtomicU64 = AtomicU64::new(0);

    fn runtime() -> RuntimeLayout {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-kernel-policy-lifecycle-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    struct PolicyStageAuthority;

    impl AuthorizationPolicy for PolicyStageAuthority {
        fn authorize(&self, request: &AuthorizationRequest<'_>) -> PolicyDecision {
            if request.action == POLICY_STAGE_ACTION {
                PolicyDecision::allow("test_policy_stage_allow")
            } else {
                PolicyDecision::deny("test_no_matching_allow")
            }
        }
    }

    #[test]
    fn invalid_candidate_fails_before_authorization_or_storage() {
        let runtime = runtime();
        let mut kernel = KernelApi::open(&runtime, PolicyStageAuthority).unwrap();
        assert!(matches!(
            kernel.stage_policy_bundle(StagePolicyBundle {
                principal: Principal::local_owner("owner"),
                schema_version: 1,
                policy_source: "permit(",
                schema_source: SCHEMA,
                scope: "local-owner",
            }),
            Err(PolicyMutationError::Candidate(_))
        ));
        assert_eq!(kernel.authorization_decision_count().unwrap(), 0);
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn kernel_stages_only_after_cedar_validation_and_current_allow() {
        let runtime = runtime();
        let mut kernel = KernelApi::open(&runtime, PolicyStageAuthority).unwrap();
        let staged = kernel
            .stage_policy_bundle(StagePolicyBundle {
                principal: Principal::local_owner("owner"),
                schema_version: 1,
                policy_source: POLICY,
                schema_source: SCHEMA,
                scope: "local-owner",
            })
            .unwrap();
        assert_eq!(staged.version, 1);
        assert_eq!(kernel.authorization_decision_count().unwrap(), 1);
        let store = PolicyStore::open(&kernel.authority).unwrap();
        assert!(store.active().unwrap().is_none());
        drop(store);
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn denied_stage_never_creates_a_bundle() {
        let runtime = runtime();
        let mut kernel = KernelApi::open(&runtime, DenyByDefault).unwrap();
        let result = kernel.stage_policy_bundle(StagePolicyBundle {
            principal: Principal::local_owner("owner"),
            schema_version: 1,
            policy_source: POLICY,
            schema_source: SCHEMA,
            scope: "local-owner",
        });
        let Err(PolicyMutationError::AuthorizationDenied(outcome)) = result else {
            panic!("stage must fail with a durable authorization denial");
        };
        assert_eq!(outcome.decision, AuthorizationDecision::Deny);
        assert_eq!(kernel.authorization_decision_count().unwrap(), 1);
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
