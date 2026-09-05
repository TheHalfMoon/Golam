from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file_path = Path(path)
    content = file_path.read_text()
    if old not in content:
        raise SystemExit(f"expected pattern missing in {path}: {old[:120]!r}")
    file_path.write_text(content.replace(old, new, 1))


replace_once(
    "crates/golam-kernel/src/approval_lifecycle.rs",
    """use golam_core::EffectId;
use golam_ledger::approval_binding::{
    APPROVAL_ISSUE_ACTION, ApprovalBindingError, ApprovalStore, prepare_approval,
};
""",
    """use golam_core::{EffectId, EffectTransitionId, EventId, SessionId};
use golam_ledger::approval_binding::{
    APPROVAL_ISSUE_ACTION, APPROVAL_MUTATION_RISK_CLASS, ApprovalBindingError, ApprovalStore,
    prepare_approval,
};
""",
)
replace_once(
    "crates/golam-kernel/src/approval_lifecycle.rs",
    "use golam_ledger::approvals::{ApprovalRecord, ApprovalScope};\n",
    """use golam_ledger::approvals::{ApprovalRecord, ApprovalScope};
use golam_ledger::dispatch::{EffectDispatchStoreError, encode_effect_dependencies};
use golam_ledger::effects::{CompareAndSwapEffect, EffectStore, EffectStoreError, ProposeEffect};
""",
)
replace_once(
    "crates/golam-kernel/src/approval_lifecycle.rs",
    "pub struct IssueApproval<'a> {\n",
    """pub struct PrepareApprovalIssueEffect<'a> {
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
""",
)
replace_once(
    "crates/golam-kernel/src/approval_lifecycle.rs",
    """pub enum ApprovalMutationError {
    Authorization(AuthorizationError),
    AuthorizationDenied(AuthorizationOutcome),
    Binding(ApprovalBindingError),
}
""",
    """pub enum ApprovalMutationError {
    Authorization(AuthorizationError),
    AuthorizationDenied(AuthorizationOutcome),
    Binding(ApprovalBindingError),
    Dispatch(EffectDispatchStoreError),
    Store(EffectStoreError),
}
""",
)
replace_once(
    "crates/golam-kernel/src/approval_lifecycle.rs",
    '            Self::Binding(error) => write!(f, "approval issuance binding failed: {error}"),\n',
    """            Self::Binding(error) => write!(f, "approval issuance binding failed: {error}"),
            Self::Dispatch(error) => {
                write!(f, "approval issuance dispatch encoding failed: {error}")
            }
            Self::Store(error) => write!(f, "approval issuance effect store failed: {error}"),
""",
)
replace_once(
    "crates/golam-kernel/src/approval_lifecycle.rs",
    """            Self::Binding(error) => Some(error),
            Self::AuthorizationDenied(_) => None,
""",
    """            Self::Binding(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            Self::Store(error) => Some(error),
            Self::AuthorizationDenied(_) => None,
""",
)
replace_once(
    "crates/golam-kernel/src/approval_lifecycle.rs",
    """impl From<ApprovalBindingError> for ApprovalMutationError {
    fn from(value: ApprovalBindingError) -> Self {
        Self::Binding(value)
    }
}

""",
    """impl From<ApprovalBindingError> for ApprovalMutationError {
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

""",
)
replace_once(
    "crates/golam-kernel/src/approval_lifecycle.rs",
    """impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn issue_approval(
""",
    """impl<P: AuthorizationPolicy> KernelApi<P> {
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
""",
)
replace_once(
    "crates/golam-kernel/src/policy_lifecycle.rs",
    "pub use approval_lifecycle::{ApprovalMutationError, IssueApproval};\n",
    """pub use approval_lifecycle::{
    ApprovalMutationError, IssueApproval, PrepareApprovalIssueEffect, PreparedApprovalIssueEffect,
};
""",
)
replace_once(
    "crates/golam-kernel/src/capability_lease_effect.rs",
    "use super::approval_lifecycle::{ApprovalMutationError, IssueApproval};\n",
    """use super::approval_lifecycle::{
    ApprovalMutationError, IssueApproval, PrepareApprovalIssueEffect,
};
""",
)
replace_once(
    "crates/golam-kernel/src/capability_lease_effect.rs",
    "    pub fn issue_capability_lease_once_approval(\n",
    """    pub fn prepare_capability_lease_issue_once_approval_effect(
        &mut self,
        issuer: Principal<'_>,
        lease_issue_effect_id: EffectId,
        resource: &str,
        issued_at: &str,
        approval_issue_effect_id: EffectId,
        session_id: SessionId,
        proposed_event_id: EventId,
        proposed_transition_id: EffectTransitionId,
        authorized_event_id: EventId,
        authorized_transition_id: EffectTransitionId,
        authorization_scope: &str,
    ) -> Result<(), CapabilityLeaseEffectError> {
        let approval_scope = ApprovalScope::once(
            lease_issue_effect_id,
            CAPABILITY_LEASE_ISSUE_ACTION,
            resource,
        )?;
        let prepared = self.prepare_approval_issue_effect(PrepareApprovalIssueEffect {
            principal: issuer,
            approval_scope,
            risk_class: CAPABILITY_LEASE_MUTATION_RISK_CLASS,
            taint_digest: [0; 32],
            issued_at,
            expires_at: None,
            max_uses: 1,
            effect_id: approval_issue_effect_id,
            session_id,
            proposed_event_id,
            proposed_transition_id,
            authorized_event_id,
            authorized_transition_id,
            authorization_scope,
        })?;
        debug_assert!(!prepared.resource().is_empty());
        Ok(())
    }

    pub fn issue_capability_lease_once_approval(
""",
)
replace_once(
    "crates/golamd/tests/process_v2_qualification.rs",
    """        let resource = prepared.resource().to_owned();
        let approval_id = kernel
            .issue_capability_lease_once_approval(
""",
    """        let resource = prepared.resource().to_owned();
        kernel
            .prepare_capability_lease_issue_once_approval_effect(
                Principal::local_owner("issuer"),
                effect_id,
                &resource,
                "2026-09-05T19:15:29Z",
                EffectId(0x6010),
                SessionId(0x5000),
                EventId(0x6011),
                EffectTransitionId(0x6012),
                EventId(0x6014),
                EffectTransitionId(0x6013),
                SCOPE,
            )
            .expect("prepare lease approval issue effect");
        let approval_id = kernel
            .issue_capability_lease_once_approval(
""",
)
