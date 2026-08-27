#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::EffectId;
use golam_ledger::approval_binding::{
    APPROVAL_ISSUE_ACTION, ApprovalBindingError, ApprovalStore, prepare_approval,
};
use golam_ledger::approvals::{ApprovalRecord, ApprovalScope};

use crate::{
    AuthorizationContext, AuthorizationError, AuthorizationOutcome, AuthorizationPolicy,
    AuthorizationRequest, KernelApi, Principal, PrincipalKind,
};

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
}

impl fmt::Display for ApprovalMutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Authorization(error) => {
                write!(f, "approval issuance authorization failed: {error}")
            }
            Self::AuthorizationDenied(outcome) => write!(
                f,
                "approval issuance denied: decision={:?} reason={}",
                outcome.decision_id, outcome.reason_code
            ),
            Self::Binding(error) => write!(f, "approval issuance binding failed: {error}"),
        }
    }
}

impl Error for ApprovalMutationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Authorization(error) => Some(error),
            Self::Binding(error) => Some(error),
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

impl<P: AuthorizationPolicy> KernelApi<P> {
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
