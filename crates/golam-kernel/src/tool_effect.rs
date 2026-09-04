#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::{EffectAttemptId, EffectId, EffectTransitionId, EventId, SessionId};
use golam_ledger::dispatch::{
    EffectDispatchStoreError, PrepareEffectDispatch, encode_effect_dependencies,
};
use golam_ledger::effect_completion::{
    CompleteEffectExecution, EffectCompletionError, EffectCompletionStore, ExecutionCompletion,
};
use golam_ledger::effects::{CompareAndSwapEffect, EffectStore, EffectStoreError, ProposeEffect};

use crate::{
    AuthorizationContext, AuthorizationPolicy, AuthorizationRequest, KernelApi, KernelError,
    PreparedEffectDispatch, Principal,
};

const TOOL_EFFECT_ID_STRIDE: u128 = 32;
const STAGE_PROPOSED_EVENT: u128 = 1;
const STAGE_PROPOSED_TRANSITION: u128 = 2;
const STAGE_AUTHORIZED_EVENT: u128 = 3;
const STAGE_AUTHORIZED_TRANSITION: u128 = 4;
const STAGE_ATTEMPT: u128 = 5;
const STAGE_EXECUTING_TRANSITION: u128 = 6;
const STAGE_EXECUTING_EVENT: u128 = 7;
const STAGE_COMPLETION_TRANSITION: u128 = 8;
const STAGE_COMPLETION_EVENT: u128 = 9;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolExecutionCompletion {
    Succeeded,
    Failed,
    UnknownOutcome,
}

impl ToolExecutionCompletion {
    const fn ledger_completion(self) -> ExecutionCompletion {
        match self {
            Self::Succeeded => ExecutionCompletion::Succeeded,
            Self::Failed => ExecutionCompletion::Failed,
            Self::UnknownOutcome => ExecutionCompletion::UnknownOutcome,
        }
    }
}

pub struct PrepareToolEffect<'a> {
    pub effect_id: EffectId,
    pub session_id: SessionId,
    pub action: &'a str,
    pub resource: &'a str,
    pub execution_semantics: &'a str,
    pub handler_id: &'a str,
    pub handler_version: &'a str,
    pub idempotency_key: Option<&'a str>,
    pub preconditions_hash: [u8; 32],
    pub payload_hash: [u8; 32],
    pub started_at: &'a str,
}

/// Kernel-issued proof that one exact consequential tool mutation was
/// authorized, durably prepared and moved to EXECUTING.
///
/// The binding fields are private so tool providers cannot construct or alter
/// authority-bearing preparation locally.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedToolEffect {
    dispatch: PreparedEffectDispatch,
    action: String,
    resource: String,
    preconditions_hash: [u8; 32],
    payload_hash: [u8; 32],
}

impl PreparedToolEffect {
    pub const fn effect_id(&self) -> EffectId {
        self.dispatch.effect_id()
    }

    pub const fn attempt_id(&self) -> EffectAttemptId {
        self.dispatch.attempt_id()
    }

    pub fn action(&self) -> &str {
        &self.action
    }

    pub fn resource(&self) -> &str {
        &self.resource
    }

    pub const fn preconditions_hash(&self) -> [u8; 32] {
        self.preconditions_hash
    }

    pub const fn payload_hash(&self) -> [u8; 32] {
        self.payload_hash
    }
}

pub struct CompleteToolEffect<'a> {
    pub prepared: &'a PreparedToolEffect,
    pub finished_at: &'a str,
    pub completion: ToolExecutionCompletion,
    pub reason_code: Option<&'a str>,
    pub evidence_ref: Option<&'a [u8]>,
    pub receipt: Option<&'a [u8]>,
}

#[derive(Debug)]
pub enum ToolEffectError {
    Kernel(KernelError),
    Dispatch(EffectDispatchStoreError),
    Store(EffectStoreError),
    Completion(EffectCompletionError),
    InvalidMetadata,
    UnsupportedSemantics(String),
    IdentifierOverflow(EffectId),
}

impl fmt::Display for ToolEffectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kernel(error) => write!(f, "tool effect kernel error: {error}"),
            Self::Dispatch(error) => write!(f, "tool effect dispatch error: {error}"),
            Self::Store(error) => write!(f, "tool effect store error: {error}"),
            Self::Completion(error) => write!(f, "tool effect completion error: {error}"),
            Self::InvalidMetadata => f.write_str("tool effect metadata must be non-empty"),
            Self::UnsupportedSemantics(value) => {
                write!(
                    f,
                    "unsupported consequential tool effect semantics: {value}"
                )
            }
            Self::IdentifierOverflow(effect_id) => write!(
                f,
                "tool effect derived identifier overflow for effect {}",
                effect_id.0
            ),
        }
    }
}

impl Error for ToolEffectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Kernel(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            Self::Store(error) => Some(error),
            Self::Completion(error) => Some(error),
            Self::InvalidMetadata | Self::UnsupportedSemantics(_) | Self::IdentifierOverflow(_) => {
                None
            }
        }
    }
}

impl From<KernelError> for ToolEffectError {
    fn from(value: KernelError) -> Self {
        Self::Kernel(value)
    }
}

impl From<EffectDispatchStoreError> for ToolEffectError {
    fn from(value: EffectDispatchStoreError) -> Self {
        Self::Dispatch(value)
    }
}

impl From<EffectStoreError> for ToolEffectError {
    fn from(value: EffectStoreError) -> Self {
        Self::Store(value)
    }
}

impl From<EffectCompletionError> for ToolEffectError {
    fn from(value: EffectCompletionError) -> Self {
        Self::Completion(value)
    }
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn prepare_tool_effect(
        &mut self,
        principal: Principal<'_>,
        input: PrepareToolEffect<'_>,
        scope: &str,
    ) -> Result<PreparedToolEffect, ToolEffectError> {
        validate_semantics(input.execution_semantics)?;
        if input.action.is_empty()
            || input.resource.is_empty()
            || input.handler_id.is_empty()
            || input.handler_version.is_empty()
            || input.started_at.is_empty()
        {
            return Err(ToolEffectError::InvalidMetadata);
        }

        self.require_authority(&AuthorizationRequest {
            principal,
            action: input.action,
            resource: input.resource,
            context: AuthorizationContext::local(scope),
        })?;

        let dependencies = encode_effect_dependencies(&[])?;
        let mut effects = EffectStore::open(&self.authority)?;
        effects.propose(ProposeEffect {
            effect_id: input.effect_id,
            session_id: input.session_id,
            requested_by: principal.subject,
            action: input.action,
            resource: input.resource,
            risk_class: "tool_mutation",
            execution_semantics: input.execution_semantics,
            idempotency_key: input.idempotency_key,
            preconditions: &input.preconditions_hash,
            dependencies: &dependencies,
            payload_hash: input.payload_hash,
            proposed_event_id: EventId(stage_id(input.effect_id, STAGE_PROPOSED_EVENT)?),
            transition_id: EffectTransitionId(stage_id(
                input.effect_id,
                STAGE_PROPOSED_TRANSITION,
            )?),
        })?;
        effects.compare_and_swap(CompareAndSwapEffect {
            transition_id: EffectTransitionId(stage_id(
                input.effect_id,
                STAGE_AUTHORIZED_TRANSITION,
            )?),
            effect_id: input.effect_id,
            expected_state: "proposed",
            next_state: "authorized",
            attempt_id: None,
            reason_code: Some("tool_effect_authorized"),
            evidence_ref: None,
            event_id: EventId(stage_id(input.effect_id, STAGE_AUTHORIZED_EVENT)?),
        })?;
        drop(effects);

        let dispatch_token = dispatch_token(
            input.effect_id,
            input.preconditions_hash,
            input.payload_hash,
        );
        let dispatch = self.prepare_effect_dispatch(PrepareEffectDispatch {
            effect_id: input.effect_id,
            attempt_id: EffectAttemptId(stage_id(input.effect_id, STAGE_ATTEMPT)?),
            transition_id: EffectTransitionId(stage_id(
                input.effect_id,
                STAGE_EXECUTING_TRANSITION,
            )?),
            handler_id: input.handler_id,
            handler_version: input.handler_version,
            dispatch_token: &dispatch_token,
            started_at: input.started_at,
            event_id: EventId(stage_id(input.effect_id, STAGE_EXECUTING_EVENT)?),
        })?;

        Ok(PreparedToolEffect {
            dispatch,
            action: input.action.to_owned(),
            resource: input.resource.to_owned(),
            preconditions_hash: input.preconditions_hash,
            payload_hash: input.payload_hash,
        })
    }

    pub fn complete_tool_effect(
        &mut self,
        principal: Principal<'_>,
        input: CompleteToolEffect<'_>,
        scope: &str,
    ) -> Result<(), ToolEffectError> {
        if input.finished_at.is_empty() {
            return Err(ToolEffectError::InvalidMetadata);
        }
        self.require_authority(&AuthorizationRequest {
            principal,
            action: input.prepared.action(),
            resource: input.prepared.resource(),
            context: AuthorizationContext::local(scope),
        })?;

        let mut completion = EffectCompletionStore::open(&self.authority)?;
        completion.complete(CompleteEffectExecution {
            effect_id: input.prepared.effect_id(),
            attempt_id: input.prepared.attempt_id(),
            transition_id: EffectTransitionId(stage_id(
                input.prepared.effect_id(),
                STAGE_COMPLETION_TRANSITION,
            )?),
            event_id: EventId(stage_id(
                input.prepared.effect_id(),
                STAGE_COMPLETION_EVENT,
            )?),
            finished_at: input.finished_at,
            completion: input.completion.ledger_completion(),
            reason_code: input.reason_code,
            evidence_ref: input.evidence_ref,
            receipt: input.receipt,
        })?;
        Ok(())
    }
}

fn validate_semantics(value: &str) -> Result<(), ToolEffectError> {
    if matches!(
        value,
        "idempotent_at_least_once" | "at_most_once" | "compensatable" | "irreversible"
    ) {
        Ok(())
    } else {
        Err(ToolEffectError::UnsupportedSemantics(value.to_owned()))
    }
}

fn stage_id(effect_id: EffectId, stage: u128) -> Result<u128, ToolEffectError> {
    effect_id
        .0
        .checked_mul(TOOL_EFFECT_ID_STRIDE)
        .and_then(|base| base.checked_add(stage))
        .ok_or(ToolEffectError::IdentifierOverflow(effect_id))
}

fn dispatch_token(
    effect_id: EffectId,
    preconditions_hash: [u8; 32],
    payload_hash: [u8; 32],
) -> Vec<u8> {
    let mut token = Vec::with_capacity(16 + 32 + 32);
    token.extend_from_slice(&effect_id.0.to_be_bytes());
    token.extend_from_slice(&preconditions_hash);
    token.extend_from_slice(&payload_hash);
    token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_only_semantics_cannot_enter_consequential_tool_effect() {
        assert!(matches!(
            validate_semantics("read_only"),
            Err(ToolEffectError::UnsupportedSemantics(_))
        ));
        assert!(validate_semantics("at_most_once").is_ok());
    }

    #[test]
    fn dispatch_token_binds_effect_preconditions_and_payload() {
        let base = dispatch_token(EffectId(7), [1; 32], [2; 32]);
        assert_ne!(base, dispatch_token(EffectId(8), [1; 32], [2; 32]));
        assert_ne!(base, dispatch_token(EffectId(7), [3; 32], [2; 32]));
        assert_ne!(base, dispatch_token(EffectId(7), [1; 32], [4; 32]));
    }
}
