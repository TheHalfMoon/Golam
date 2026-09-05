#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::{EffectId, EffectTransitionId, EventId, SessionId};
use golam_ledger::approvals::ApprovalScope;
use golam_ledger::capability_leases::{
    CAPABILITY_LEASE_ISSUE_ACTION, CAPABILITY_LEASE_MUTATION_RISK_CLASS,
    CapabilityLeaseMutationError,
};
use golam_ledger::dispatch::{EffectDispatchStoreError, encode_effect_dependencies};
use golam_ledger::effects::{CompareAndSwapEffect, EffectStore, EffectStoreError, ProposeEffect};

use super::approval_lifecycle::{ApprovalMutationError, IssueApproval};
use crate::{
    AuthorizationContext, AuthorizationOutcome, AuthorizationPolicy, AuthorizationRequest,
    CapabilityLease, CapabilityLeaseScope, KernelApi, KernelError, Principal,
};

pub struct PrepareCapabilityLeaseIssueEffect<'a> {
    pub issuer: Principal<'a>,
    pub beneficiary_principal_id: &'a str,
    pub parent: Option<&'a CapabilityLease>,
    pub scope: &'a CapabilityLeaseScope,
    pub not_before: Option<&'a str>,
    pub expires_at: Option<&'a str>,
    pub effect_id: EffectId,
    pub session_id: SessionId,
    pub proposed_event_id: EventId,
    pub proposed_transition_id: EffectTransitionId,
    pub authorized_event_id: EventId,
    pub authorized_transition_id: EffectTransitionId,
    pub authorization_scope: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedCapabilityLeaseIssueEffect {
    resource: String,
}

impl PreparedCapabilityLeaseIssueEffect {
    pub fn resource(&self) -> &str {
        &self.resource
    }
}

#[derive(Debug)]
pub enum CapabilityLeaseEffectError {
    Kernel(KernelError),
    Lease(CapabilityLeaseMutationError),
    Dispatch(EffectDispatchStoreError),
    Store(EffectStoreError),
    Approval(ApprovalMutationError),
    ApprovalScope(golam_ledger::approvals::ApprovalScopeError),
}

impl fmt::Display for CapabilityLeaseEffectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kernel(error) => write!(f, "capability lease effect kernel error: {error}"),
            Self::Lease(error) => write!(f, "capability lease effect binding error: {error}"),
            Self::Dispatch(error) => write!(
                f,
                "capability lease effect dispatch encoding error: {error}"
            ),
            Self::Store(error) => write!(f, "capability lease effect store error: {error}"),
            Self::Approval(error) => write!(f, "capability lease approval error: {error}"),
            Self::ApprovalScope(error) => {
                write!(f, "capability lease approval scope error: {error}")
            }
        }
    }
}

impl Error for CapabilityLeaseEffectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Kernel(error) => Some(error),
            Self::Lease(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            Self::Store(error) => Some(error),
            Self::Approval(error) => Some(error),
            Self::ApprovalScope(error) => Some(error),
        }
    }
}

impl From<KernelError> for CapabilityLeaseEffectError {
    fn from(value: KernelError) -> Self {
        Self::Kernel(value)
    }
}

impl From<CapabilityLeaseMutationError> for CapabilityLeaseEffectError {
    fn from(value: CapabilityLeaseMutationError) -> Self {
        Self::Lease(value)
    }
}

impl From<EffectDispatchStoreError> for CapabilityLeaseEffectError {
    fn from(value: EffectDispatchStoreError) -> Self {
        Self::Dispatch(value)
    }
}

impl From<EffectStoreError> for CapabilityLeaseEffectError {
    fn from(value: EffectStoreError) -> Self {
        Self::Store(value)
    }
}

impl From<ApprovalMutationError> for CapabilityLeaseEffectError {
    fn from(value: ApprovalMutationError) -> Self {
        Self::Approval(value)
    }
}

impl From<golam_ledger::approvals::ApprovalScopeError> for CapabilityLeaseEffectError {
    fn from(value: golam_ledger::approvals::ApprovalScopeError) -> Self {
        Self::ApprovalScope(value)
    }
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn prepare_capability_lease_issue_effect(
        &mut self,
        input: PrepareCapabilityLeaseIssueEffect<'_>,
    ) -> Result<PreparedCapabilityLeaseIssueEffect, CapabilityLeaseEffectError> {
        let (resource, payload_hash) = self.capability_lease_issue_effect_binding(
            input.beneficiary_principal_id,
            input.parent,
            input.scope,
            input.not_before,
            input.expires_at,
        )?;

        self.require_authority(&AuthorizationRequest {
            principal: input.issuer,
            action: CAPABILITY_LEASE_ISSUE_ACTION,
            resource: &resource,
            context: AuthorizationContext::local(input.authorization_scope),
        })?;

        let dependencies = encode_effect_dependencies(&[])?;
        let mut effects = EffectStore::open(&self.authority)?;
        effects.propose(ProposeEffect {
            effect_id: input.effect_id,
            session_id: input.session_id,
            requested_by: input.issuer.subject,
            action: CAPABILITY_LEASE_ISSUE_ACTION,
            resource: &resource,
            risk_class: CAPABILITY_LEASE_MUTATION_RISK_CLASS,
            execution_semantics: "at_most_once",
            idempotency_key: None,
            preconditions: b"[]",
            dependencies: &dependencies,
            payload_hash,
            proposed_event_id: input.proposed_event_id,
            transition_id: input.proposed_transition_id,
        })?;
        effects.compare_and_swap(CompareAndSwapEffect {
            transition_id: input.authorized_transition_id,
            effect_id: input.effect_id,
            expected_state: "proposed",
            next_state: "authorized",
            attempt_id: None,
            reason_code: Some("capability_lease_issue_authorized"),
            evidence_ref: None,
            event_id: input.authorized_event_id,
        })?;

        Ok(PreparedCapabilityLeaseIssueEffect { resource })
    }

    pub fn issue_capability_lease_once_approval(
        &mut self,
        issuer: Principal<'_>,
        effect_id: EffectId,
        resource: &str,
        issued_at: &str,
        issue_effect_id: EffectId,
        authorization_scope: &str,
    ) -> Result<[u8; 16], CapabilityLeaseEffectError> {
        let approval_scope =
            ApprovalScope::once(effect_id, CAPABILITY_LEASE_ISSUE_ACTION, resource)?;
        let approval = self.issue_approval(IssueApproval {
            principal: issuer,
            approval_scope,
            risk_class: CAPABILITY_LEASE_MUTATION_RISK_CLASS,
            taint_digest: [0; 32],
            issued_at,
            expires_at: None,
            max_uses: 1,
            issue_effect_id,
            authorization_scope,
        })?;
        Ok(approval.approval_id())
    }

    pub fn authorize_capability_lease_issue(
        &mut self,
        issuer: Principal<'_>,
        resource: &str,
        authorization_scope: &str,
    ) -> Result<AuthorizationOutcome, KernelError> {
        self.authorize(&AuthorizationRequest {
            principal: issuer,
            action: CAPABILITY_LEASE_ISSUE_ACTION,
            resource,
            context: AuthorizationContext::local(authorization_scope),
        })
    }
}
