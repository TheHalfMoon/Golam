from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file_path = Path(path)
    content = file_path.read_text()
    if old not in content:
        raise SystemExit(f"expected v2 pattern missing in {path}: {old[:160]!r}")
    file_path.write_text(content.replace(old, new, 1))


replace_once(
    "crates/golam-kernel/src/capability_lease_effect.rs",
    """impl PreparedCapabilityLeaseIssueEffect {
    pub fn resource(&self) -> &str {
        &self.resource
    }
}

#[derive(Debug)]
""",
    """impl PreparedCapabilityLeaseIssueEffect {
    pub fn resource(&self) -> &str {
        &self.resource
    }
}

pub struct PrepareCapabilityLeaseIssueApprovalEffect<'a> {
    pub issuer: Principal<'a>,
    pub lease_issue_effect_id: EffectId,
    pub resource: &'a str,
    pub issued_at: &'a str,
    pub approval_issue_effect_id: EffectId,
    pub session_id: SessionId,
    pub proposed_event_id: EventId,
    pub proposed_transition_id: EffectTransitionId,
    pub authorized_event_id: EventId,
    pub authorized_transition_id: EffectTransitionId,
    pub authorization_scope: &'a str,
}

#[derive(Debug)]
""",
)

replace_once(
    "crates/golam-kernel/src/capability_lease_effect.rs",
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
""",
    """    pub fn prepare_capability_lease_issue_once_approval_effect(
        &mut self,
        input: PrepareCapabilityLeaseIssueApprovalEffect<'_>,
    ) -> Result<(), CapabilityLeaseEffectError> {
        let approval_scope = ApprovalScope::once(
            input.lease_issue_effect_id,
            CAPABILITY_LEASE_ISSUE_ACTION,
            input.resource,
        )?;
        let prepared = self.prepare_approval_issue_effect(PrepareApprovalIssueEffect {
            principal: input.issuer,
            approval_scope,
            risk_class: CAPABILITY_LEASE_MUTATION_RISK_CLASS,
            taint_digest: [0; 32],
            issued_at: input.issued_at,
            expires_at: None,
            max_uses: 1,
            effect_id: input.approval_issue_effect_id,
            session_id: input.session_id,
            proposed_event_id: input.proposed_event_id,
            proposed_transition_id: input.proposed_transition_id,
            authorized_event_id: input.authorized_event_id,
            authorized_transition_id: input.authorized_transition_id,
            authorization_scope: input.authorization_scope,
        })?;
        debug_assert!(!prepared.resource().is_empty());
        Ok(())
    }
""",
)

replace_once(
    "crates/golamd/tests/process_v2_qualification.rs",
    "use golam_kernel::policy_lifecycle::capability_lease_effect::PrepareCapabilityLeaseIssueEffect;\n",
    """use golam_kernel::policy_lifecycle::capability_lease_effect::{
        PrepareCapabilityLeaseIssueApprovalEffect, PrepareCapabilityLeaseIssueEffect,
    };
""",
)

replace_once(
    "crates/golamd/tests/process_v2_qualification.rs",
    """        kernel
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
""",
    """        kernel
            .prepare_capability_lease_issue_once_approval_effect(
                PrepareCapabilityLeaseIssueApprovalEffect {
                    issuer: Principal::local_owner("issuer"),
                    lease_issue_effect_id: effect_id,
                    resource: &resource,
                    issued_at: "2026-09-05T19:15:29Z",
                    approval_issue_effect_id: EffectId(0x6010),
                    session_id: SessionId(0x5000),
                    proposed_event_id: EventId(0x6011),
                    proposed_transition_id: EffectTransitionId(0x6012),
                    authorized_event_id: EventId(0x6014),
                    authorized_transition_id: EffectTransitionId(0x6013),
                    authorization_scope: SCOPE,
                },
            )
""",
)
