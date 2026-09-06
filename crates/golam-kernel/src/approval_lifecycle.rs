#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::{EffectId, EffectTransitionId, EventId, SessionId};
use golam_ledger::approval_binding::{
    APPROVAL_ISSUE_ACTION, APPROVAL_MUTATION_RISK_CLASS, ApprovalBindingError, ApprovalStore,
    prepare_approval,
};
use golam_ledger::approval_consumption::ApprovalConsumptionStore;
pub use golam_ledger::approval_consumption::{
    ApprovalConsumption, ApprovalConsumptionError, ApprovalReservation,
};
use golam_ledger::approval_runtime::ApprovalUseStore;
pub use golam_ledger::approval_runtime::{
    ApprovalUseError, ApprovalUseEvidence, ApprovalUseRequest,
};
use golam_ledger::approvals::{ApprovalRecord, ApprovalScope};
use golam_ledger::dispatch::{EffectDispatchStoreError, encode_effect_dependencies};
use golam_ledger::effects::{CompareAndSwapEffect, EffectStore, EffectStoreError, ProposeEffect};
use golam_ledger::run_preauthorization::RunPreauthorizationStore;
pub use golam_ledger::run_preauthorization::{
    MAX_UNATTENDED_IRREVERSIBLE_RUN_USES, RunPreauthorizationError, RunPreauthorizationUse,
    UnattendedIrreversibleRequest,
};

use crate::{
    AuthorizationContext, AuthorizationError, AuthorizationOutcome, AuthorizationPolicy,
    AuthorizationRequest, KernelApi, Principal, PrincipalKind,
};

pub struct PrepareApprovalIssueEffect<'a> {
    pub principal: Principal<'a>,
    pub approval_scope: ApprovalScope,
    pub risk_class: &'a str,
    pub taint_digest: [u8; 32],
    pub issued_at: &'a str,
    pub expires_at: Option<&'a str>,
    pub max_uses: u64,
    pub effect_id: EffectId,
    pub session_id: SessionId,
    pub proposed_event_id: EventId,
    pub proposed_transition_id: EffectTransitionId,
    pub authorized_event_id: EventId,
    pub authorized_transition_id: EffectTransitionId,
    pub authorization_scope: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedApprovalIssueEffect {
    resource: String,
}

impl PreparedApprovalIssueEffect {
    pub fn resource(&self) -> &str {
        &self.resource
    }
}

pub struct IssueApproval<'a> {
    pub principal: Principal<'a>,
    pub approval_scope: ApprovalScope,
    pub risk_class: &'a str,
    pub taint_digest: [u8; 32],
    pub issued_at: &'a str,
    pub expires_at: Option<&'a str>,
    pub max_uses: u64,
    pub issue_effect_id: EffectId,
    pub authorization_scope: &'a str,
}

#[derive(Debug)]
pub enum ApprovalMutationError {
    Authorization(AuthorizationError),
    AuthorizationDenied(AuthorizationOutcome),
    Binding(ApprovalBindingError),
    Dispatch(EffectDispatchStoreError),
    Store(EffectStoreError),
}

impl fmt::Display for ApprovalMutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Authorization(error) => {
                write!(f, "approval issuance authorization failed: {error}")
            }
            Self::AuthorizationDenied(outcome) => write!(
                f,
                "approval mutation denied: decision={:?} reason={}",
                outcome.decision_id, outcome.reason_code
            ),
            Self::Binding(error) => write!(f, "approval issuance binding failed: {error}"),
            Self::Dispatch(error) => {
                write!(f, "approval issuance dispatch encoding failed: {error}")
            }
            Self::Store(error) => write!(f, "approval issuance effect store failed: {error}"),
        }
    }
}

impl Error for ApprovalMutationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Authorization(error) => Some(error),
            Self::Binding(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            Self::Store(error) => Some(error),
            Self::AuthorizationDenied(_) => None,
        }
    }
}

impl From<AuthorizationError> for ApprovalMutationError {
    fn from(value: AuthorizationError) -> Self {
        Self::Authorization(value)
    }
}

impl From<ApprovalBindingError> for ApprovalMutationError {
    fn from(value: ApprovalBindingError) -> Self {
        Self::Binding(value)
    }
}

impl From<EffectDispatchStoreError> for ApprovalMutationError {
    fn from(value: EffectDispatchStoreError) -> Self {
        Self::Dispatch(value)
    }
}

impl From<EffectStoreError> for ApprovalMutationError {
    fn from(value: EffectStoreError) -> Self {
        Self::Store(value)
    }
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn prepare_approval_issue_effect(
        &mut self,
        input: PrepareApprovalIssueEffect<'_>,
    ) -> Result<PreparedApprovalIssueEffect, ApprovalMutationError> {
        let approver_principal = audit_principal(input.principal);
        let prepared = prepare_approval(
            &approver_principal,
            input.approval_scope,
            input.risk_class,
            input.taint_digest,
            input.issued_at,
            input.expires_at,
            input.max_uses,
        )?;
        let resource = prepared.resource().to_owned();
        let request = AuthorizationRequest {
            principal: input.principal,
            action: APPROVAL_ISSUE_ACTION,
            resource: &resource,
            context: AuthorizationContext::local(input.authorization_scope),
        };
        let (outcome, grant) = self.authorization.authorize(&request)?;
        if grant.is_none() {
            return Err(ApprovalMutationError::AuthorizationDenied(outcome));
        }

        let dependencies = encode_effect_dependencies(&[])?;
        let mut effects = EffectStore::open(&self.authority)?;
        effects.propose(ProposeEffect {
            effect_id: input.effect_id,
            session_id: input.session_id,
            requested_by: &approver_principal,
            action: APPROVAL_ISSUE_ACTION,
            resource: &resource,
            risk_class: APPROVAL_MUTATION_RISK_CLASS,
            execution_semantics: "at_most_once",
            idempotency_key: None,
            preconditions: b"[]",
            dependencies: &dependencies,
            payload_hash: prepared.intent_digest(),
            proposed_event_id: input.proposed_event_id,
            transition_id: input.proposed_transition_id,
        })?;
        effects.compare_and_swap(CompareAndSwapEffect {
            transition_id: input.authorized_transition_id,
            effect_id: input.effect_id,
            expected_state: "proposed",
            next_state: "authorized",
            attempt_id: None,
            reason_code: Some("approval_issue_authorized"),
            evidence_ref: None,
            event_id: input.authorized_event_id,
        })?;

        Ok(PreparedApprovalIssueEffect { resource })
    }

    pub fn issue_approval(
        &mut self,
        input: IssueApproval<'_>,
    ) -> Result<ApprovalRecord, ApprovalMutationError> {
        let approver_principal = audit_principal(input.principal);
        let prepared = prepare_approval(
            &approver_principal,
            input.approval_scope,
            input.risk_class,
            input.taint_digest,
            input.issued_at,
            input.expires_at,
            input.max_uses,
        )?;
        let request = AuthorizationRequest {
            principal: input.principal,
            action: APPROVAL_ISSUE_ACTION,
            resource: prepared.resource(),
            context: AuthorizationContext::local(input.authorization_scope),
        };
        let (outcome, grant) = self.authorization.authorize(&request)?;
        let parent_decision_id = match grant {
            Some(grant) => {
                debug_assert_eq!(grant.decision_id(), outcome.decision_id);
                grant.decision_id().0
            }
            None => return Err(ApprovalMutationError::AuthorizationDenied(outcome)),
        };
        let mut store = ApprovalStore::open(&self.authority)?;
        Ok(store.issue(prepared, parent_decision_id, input.issue_effect_id)?)
    }

    /// Revalidates an approval against the exact protected use and current
    /// protected authority snapshot immediately before execution. The returned
    /// value is evidence only: consumptive execution still requires the atomic
    /// reservation/consumption path introduced by T003-033.
    pub fn validate_approval_use(
        &self,
        request: ApprovalUseRequest<'_>,
    ) -> Result<ApprovalUseEvidence, ApprovalUseError> {
        let mut store = ApprovalUseStore::open(&self.authority)?;
        store.validate(request)
    }

    /// Revalidates and durably reserves one exact ONCE approval immediately
    /// before protected execution. A crash after this call leaves the durable
    /// reservation in place, so retry cannot silently execute a second time.
    pub fn reserve_once_approval_use(
        &mut self,
        request: ApprovalUseRequest<'_>,
    ) -> Result<ApprovalReservation, ApprovalConsumptionError> {
        let mut store = ApprovalConsumptionStore::open(&self.authority)?;
        store.reserve_once(request)
    }

    /// Marks the exact durable ONCE reservation consumed only after the bound
    /// effect has progressed into execution. Repeating this call for the same
    /// reservation is idempotent and cannot create another use.
    pub fn consume_once_approval(
        &mut self,
        reservation: ApprovalReservation,
    ) -> Result<ApprovalConsumption, ApprovalConsumptionError> {
        let mut store = ApprovalConsumptionStore::open(&self.authority)?;
        store.consume_once(reservation)
    }

    /// Claims one bounded per-run authorization for unattended irreversible
    /// work. The protected effect supplies action/resource/risk/session; callers
    /// cannot widen those fields. Other approval classes, sessionless run scopes,
    /// replay, exhausted limits and limits above the Spec 003 ceiling deny.
    pub fn claim_unattended_irreversible_run_preauthorization(
        &mut self,
        request: UnattendedIrreversibleRequest<'_>,
    ) -> Result<RunPreauthorizationUse, RunPreauthorizationError> {
        let mut store = RunPreauthorizationStore::open(&self.authority)?;
        store.claim_unattended_irreversible(request)
    }
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
