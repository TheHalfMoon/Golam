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
use golam_ledger::effect_read::{EffectReadError, EffectReader};
use golam_ledger::effects::{CompareAndSwapEffect, EffectStore, EffectStoreError, ProposeEffect};
use golam_ledger::manual_review::{
    ManualReviewError, ManualReviewReason, ManualReviewStore, PlaceEffectInManualReview,
};

use crate::{
    AuthorizationContext, AuthorizationPolicy, AuthorizationRequest, KernelApi, KernelError,
    PreparedEffectDispatch, Principal,
};

const SYNTHETIC_ID_STRIDE: u128 = 32;
const STAGE_PROPOSED_EVENT: u128 = 1;
const STAGE_PROPOSED_TRANSITION: u128 = 2;
const STAGE_AUTHORIZED_EVENT: u128 = 3;
const STAGE_AUTHORIZED_TRANSITION: u128 = 4;
const STAGE_ATTEMPT: u128 = 5;
const STAGE_EXECUTING_TRANSITION: u128 = 6;
const STAGE_EXECUTING_EVENT: u128 = 7;
const STAGE_COMPLETION_TRANSITION: u128 = 8;
const STAGE_COMPLETION_EVENT: u128 = 9;
const STAGE_RECONCILING_TRANSITION: u128 = 10;
const STAGE_RECONCILING_EVENT: u128 = 11;
const STAGE_RESOLUTION_TRANSITION: u128 = 12;
const STAGE_RESOLUTION_EVENT: u128 = 13;
const STAGE_MANUAL_REVIEW_TRANSITION: u128 = 14;
const STAGE_MANUAL_REVIEW_EVENT: u128 = 15;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyntheticExecutionCompletion {
    Succeeded,
    Failed,
    UnknownOutcome,
}

impl SyntheticExecutionCompletion {
    const fn ledger_completion(self) -> ExecutionCompletion {
        match self {
            Self::Succeeded => ExecutionCompletion::Succeeded,
            Self::Failed => ExecutionCompletion::Failed,
            Self::UnknownOutcome => ExecutionCompletion::UnknownOutcome,
        }
    }

    const fn state(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::UnknownOutcome => "unknown_outcome",
        }
    }
}

pub struct PrepareSyntheticEffect<'a> {
    pub effect_id: EffectId,
    pub session_id: SessionId,
    pub execution_semantics: &'a str,
    pub handler_id: &'a str,
    pub handler_version: &'a str,
    pub idempotency_key: Option<&'a str>,
    pub payload_hash: [u8; 32],
    pub started_at: &'a str,
}

pub struct CompleteSyntheticEffect<'a> {
    pub effect_id: EffectId,
    pub attempt_id: EffectAttemptId,
    pub finished_at: &'a str,
    pub completion: SyntheticExecutionCompletion,
    pub reason_code: Option<&'a str>,
    pub evidence_ref: Option<&'a [u8]>,
    pub receipt: Option<&'a [u8]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyntheticEffectOutcome {
    pub effect_id: EffectId,
    pub attempt_id: EffectAttemptId,
    pub state: String,
    pub receipt: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyntheticReconciliationContext {
    pub effect_id: EffectId,
    pub session_id: SessionId,
    pub action: String,
    pub resource: String,
    pub execution_semantics: String,
    pub idempotency_key: Option<String>,
    pub payload_hash: [u8; 32],
    pub attempt_id: EffectAttemptId,
    pub started_global_seq: u64,
    pub handler_id: String,
    pub handler_version: String,
    pub dispatch_token: Vec<u8>,
    pub attempt_outcome: String,
    pub receipt: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyntheticReconciliationResult {
    Resolved {
        effect_id: EffectId,
        state: String,
    },
    ManualReview {
        effect_id: EffectId,
        incident_id: [u8; 16],
    },
}

#[derive(Debug)]
pub enum SyntheticEffectError {
    Kernel(KernelError),
    EffectDispatch(EffectDispatchStoreError),
    EffectStore(EffectStoreError),
    EffectCompletion(EffectCompletionError),
    EffectRead(EffectReadError),
    ManualReview(ManualReviewError),
    InvalidMetadata,
    UnsupportedSemantics(String),
    IdentifierOverflow(EffectId),
    EffectNotFound(EffectId),
    MissingAttempt(EffectId),
    AttemptMismatch {
        expected: EffectAttemptId,
        actual: EffectAttemptId,
    },
}

impl fmt::Display for SyntheticEffectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kernel(error) => write!(f, "synthetic effect kernel error: {error}"),
            Self::EffectDispatch(error) => write!(f, "synthetic effect dispatch error: {error}"),
            Self::EffectStore(error) => write!(f, "synthetic effect store error: {error}"),
            Self::EffectCompletion(error) => {
                write!(f, "synthetic effect completion error: {error}")
            }
            Self::EffectRead(error) => write!(f, "synthetic effect read error: {error}"),
            Self::ManualReview(error) => write!(f, "synthetic effect manual-review error: {error}"),
            Self::InvalidMetadata => f.write_str("synthetic effect metadata must be non-empty"),
            Self::UnsupportedSemantics(value) => {
                write!(f, "unsupported synthetic effect semantics: {value}")
            }
            Self::IdentifierOverflow(effect_id) => write!(
                f,
                "synthetic effect derived identifier overflow for effect {}",
                effect_id.0
            ),
            Self::EffectNotFound(effect_id) => {
                write!(f, "synthetic effect not found: {}", effect_id.0)
            }
            Self::MissingAttempt(effect_id) => {
                write!(
                    f,
                    "synthetic effect has no durable attempt: {}",
                    effect_id.0
                )
            }
            Self::AttemptMismatch { expected, actual } => write!(
                f,
                "synthetic effect attempt mismatch: expected={}, actual={}",
                expected.0, actual.0
            ),
        }
    }
}

impl Error for SyntheticEffectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Kernel(error) => Some(error),
            Self::EffectDispatch(error) => Some(error),
            Self::EffectStore(error) => Some(error),
            Self::EffectCompletion(error) => Some(error),
            Self::EffectRead(error) => Some(error),
            Self::ManualReview(error) => Some(error),
            Self::InvalidMetadata
            | Self::UnsupportedSemantics(_)
            | Self::IdentifierOverflow(_)
            | Self::EffectNotFound(_)
            | Self::MissingAttempt(_)
            | Self::AttemptMismatch { .. } => None,
        }
    }
}

impl From<KernelError> for SyntheticEffectError {
    fn from(value: KernelError) -> Self {
        Self::Kernel(value)
    }
}

impl From<EffectDispatchStoreError> for SyntheticEffectError {
    fn from(value: EffectDispatchStoreError) -> Self {
        Self::EffectDispatch(value)
    }
}

impl From<EffectStoreError> for SyntheticEffectError {
    fn from(value: EffectStoreError) -> Self {
        Self::EffectStore(value)
    }
}

impl From<EffectCompletionError> for SyntheticEffectError {
    fn from(value: EffectCompletionError) -> Self {
        Self::EffectCompletion(value)
    }
}

impl From<EffectReadError> for SyntheticEffectError {
    fn from(value: EffectReadError) -> Self {
        Self::EffectRead(value)
    }
}

impl From<ManualReviewError> for SyntheticEffectError {
    fn from(value: ManualReviewError) -> Self {
        Self::ManualReview(value)
    }
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn prepare_synthetic_effect(
        &mut self,
        principal: Principal<'_>,
        input: PrepareSyntheticEffect<'_>,
        scope: &str,
    ) -> Result<PreparedEffectDispatch, SyntheticEffectError> {
        validate_semantics(input.execution_semantics)?;
        if input.handler_id.is_empty()
            || input.handler_version.is_empty()
            || input.started_at.is_empty()
        {
            return Err(SyntheticEffectError::InvalidMetadata);
        }
        let resource = synthetic_resource(input.effect_id);
        self.require_authority(&AuthorizationRequest {
            principal,
            action: "effect.simulate",
            resource: &resource,
            context: AuthorizationContext::local(scope),
        })?;

        let dependencies = encode_effect_dependencies(&[])?;
        let mut effects = EffectStore::open(&self.authority)?;
        effects.propose(ProposeEffect {
            effect_id: input.effect_id,
            session_id: input.session_id,
            requested_by: principal.subject,
            action: synthetic_action(input.execution_semantics),
            resource: &resource,
            risk_class: "synthetic",
            execution_semantics: input.execution_semantics,
            idempotency_key: input.idempotency_key,
            preconditions: b"synthetic-only",
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
            reason_code: Some("synthetic_effect_authorized"),
            evidence_ref: None,
            event_id: EventId(stage_id(input.effect_id, STAGE_AUTHORIZED_EVENT)?),
        })?;
        drop(effects);

        let dispatch_token = synthetic_dispatch_token(input.effect_id);
        Ok(self.prepare_effect_dispatch(PrepareEffectDispatch {
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
        })?)
    }

    pub fn complete_synthetic_effect(
        &mut self,
        principal: Principal<'_>,
        input: CompleteSyntheticEffect<'_>,
        scope: &str,
    ) -> Result<SyntheticEffectOutcome, SyntheticEffectError> {
        let expected_attempt = EffectAttemptId(stage_id(input.effect_id, STAGE_ATTEMPT)?);
        if input.attempt_id != expected_attempt {
            return Err(SyntheticEffectError::AttemptMismatch {
                expected: expected_attempt,
                actual: input.attempt_id,
            });
        }
        let resource = synthetic_resource(input.effect_id);
        self.require_authority(&AuthorizationRequest {
            principal,
            action: "effect.simulate",
            resource: &resource,
            context: AuthorizationContext::local(scope),
        })?;
        let mut completion = EffectCompletionStore::open(&self.authority)?;
        let completed = completion.complete(CompleteEffectExecution {
            effect_id: input.effect_id,
            attempt_id: input.attempt_id,
            transition_id: EffectTransitionId(stage_id(
                input.effect_id,
                STAGE_COMPLETION_TRANSITION,
            )?),
            event_id: EventId(stage_id(input.effect_id, STAGE_COMPLETION_EVENT)?),
            finished_at: input.finished_at,
            completion: input.completion.ledger_completion(),
            reason_code: input.reason_code,
            evidence_ref: input.evidence_ref,
            receipt: input.receipt,
        })?;
        Ok(SyntheticEffectOutcome {
            effect_id: input.effect_id,
            attempt_id: input.attempt_id,
            state: input.completion.state().to_owned(),
            receipt: completed.attempt.receipt,
        })
    }

    pub fn begin_synthetic_reconciliation(
        &mut self,
        principal: Principal<'_>,
        effect_id: EffectId,
        scope: &str,
    ) -> Result<SyntheticReconciliationContext, SyntheticEffectError> {
        let resource = synthetic_resource(effect_id);
        self.require_authority(&AuthorizationRequest {
            principal,
            action: "effect.reconcile",
            resource: &resource,
            context: AuthorizationContext::local(scope),
        })?;
        let reader = EffectReader::open(&self.authority)?;
        let snapshot = reader
            .snapshot(effect_id)?
            .ok_or(SyntheticEffectError::EffectNotFound(effect_id))?;
        let attempt = snapshot
            .latest_attempt
            .clone()
            .ok_or(SyntheticEffectError::MissingAttempt(effect_id))?;
        drop(reader);

        let mut effects = EffectStore::open(&self.authority)?;
        effects.compare_and_swap(CompareAndSwapEffect {
            transition_id: EffectTransitionId(stage_id(effect_id, STAGE_RECONCILING_TRANSITION)?),
            effect_id,
            expected_state: "unknown_outcome",
            next_state: "reconciling",
            attempt_id: Some(attempt.attempt_id),
            reason_code: Some("synthetic_reconciliation_started"),
            evidence_ref: None,
            event_id: EventId(stage_id(effect_id, STAGE_RECONCILING_EVENT)?),
        })?;

        Ok(SyntheticReconciliationContext {
            effect_id,
            session_id: snapshot.session_id,
            action: snapshot.action,
            resource: snapshot.resource,
            execution_semantics: snapshot.execution_semantics,
            idempotency_key: snapshot.idempotency_key,
            payload_hash: snapshot.payload_hash,
            attempt_id: attempt.attempt_id,
            started_global_seq: attempt.started_global_seq,
            handler_id: attempt.handler_id,
            handler_version: attempt.handler_version,
            dispatch_token: attempt.dispatch_token,
            attempt_outcome: attempt.outcome,
            receipt: attempt.receipt,
        })
    }

    pub fn resolve_synthetic_reconciliation(
        &mut self,
        principal: Principal<'_>,
        effect_id: EffectId,
        resolution: SyntheticExecutionCompletion,
        reason_code: Option<&str>,
        evidence_ref: Option<&[u8]>,
        detected_at: &str,
        scope: &str,
    ) -> Result<SyntheticReconciliationResult, SyntheticEffectError> {
        let resource = synthetic_resource(effect_id);
        self.require_authority(&AuthorizationRequest {
            principal,
            action: "effect.reconcile",
            resource: &resource,
            context: AuthorizationContext::local(scope),
        })?;
        let reader = EffectReader::open(&self.authority)?;
        let snapshot = reader
            .snapshot(effect_id)?
            .ok_or(SyntheticEffectError::EffectNotFound(effect_id))?;
        let attempt_id = snapshot
            .latest_attempt
            .as_ref()
            .map(|attempt| attempt.attempt_id)
            .ok_or(SyntheticEffectError::MissingAttempt(effect_id))?;
        drop(reader);

        match resolution {
            SyntheticExecutionCompletion::Succeeded | SyntheticExecutionCompletion::Failed => {
                let target = resolution.state();
                let mut effects = EffectStore::open(&self.authority)?;
                effects.compare_and_swap(CompareAndSwapEffect {
                    transition_id: EffectTransitionId(stage_id(
                        effect_id,
                        STAGE_RESOLUTION_TRANSITION,
                    )?),
                    effect_id,
                    expected_state: "reconciling",
                    next_state: target,
                    attempt_id: Some(attempt_id),
                    reason_code,
                    evidence_ref,
                    event_id: EventId(stage_id(effect_id, STAGE_RESOLUTION_EVENT)?),
                })?;
                Ok(SyntheticReconciliationResult::Resolved {
                    effect_id,
                    state: target.to_owned(),
                })
            }
            SyntheticExecutionCompletion::UnknownOutcome => {
                if detected_at.is_empty() {
                    return Err(SyntheticEffectError::InvalidMetadata);
                }
                let mut manual = ManualReviewStore::open(&self.authority)?;
                let report = manual.place(PlaceEffectInManualReview {
                    effect_id,
                    transition_id: EffectTransitionId(stage_id(
                        effect_id,
                        STAGE_MANUAL_REVIEW_TRANSITION,
                    )?),
                    attempt_id: Some(attempt_id),
                    detected_at,
                    reason: ManualReviewReason::UnreconcilableAmbiguity,
                    evidence_ref,
                    event_id: EventId(stage_id(effect_id, STAGE_MANUAL_REVIEW_EVENT)?),
                })?;
                Ok(SyntheticReconciliationResult::ManualReview {
                    effect_id,
                    incident_id: report.incident_id,
                })
            }
        }
    }
}

fn validate_semantics(value: &str) -> Result<(), SyntheticEffectError> {
    if matches!(
        value,
        "read_only"
            | "idempotent_at_least_once"
            | "at_most_once"
            | "compensatable"
            | "irreversible"
    ) {
        Ok(())
    } else {
        Err(SyntheticEffectError::UnsupportedSemantics(value.to_owned()))
    }
}

fn synthetic_action(execution_semantics: &str) -> &'static str {
    if execution_semantics == "read_only" {
        "sim.read"
    } else {
        "sim.write"
    }
}

fn synthetic_resource(effect_id: EffectId) -> String {
    format!("sim:effect:{}", effect_id.0)
}

fn synthetic_dispatch_token(effect_id: EffectId) -> [u8; 17] {
    let mut token = [0_u8; 17];
    token[..16].copy_from_slice(&effect_id.0.to_be_bytes());
    token[16] = u8::try_from(STAGE_ATTEMPT).expect("synthetic stage fits u8");
    token
}

fn stage_id(effect_id: EffectId, stage: u128) -> Result<u128, SyntheticEffectError> {
    effect_id
        .0
        .checked_mul(SYNTHETIC_ID_STRIDE)
        .and_then(|base| base.checked_add(stage))
        .ok_or(SyntheticEffectError::IdentifierOverflow(effect_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BootstrapPolicy;
    use golam_core::paths::RuntimeLayout;
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
            "golam-kernel-synthetic-effect-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    #[test]
    fn synthetic_effect_is_durable_before_completion() {
        let runtime = runtime();
        let mut kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let principal = Principal::local_owner("owner");
        let effect_id = EffectId(100);
        let prepared = kernel
            .prepare_synthetic_effect(
                principal,
                PrepareSyntheticEffect {
                    effect_id,
                    session_id: SessionId(7),
                    execution_semantics: "at_most_once",
                    handler_id: "sim-at-most-once-write",
                    handler_version: "1",
                    idempotency_key: None,
                    payload_hash: [7; 32],
                    started_at: "2026-08-25T13:45:00Z",
                },
                "local-owner",
            )
            .unwrap();
        let outcome = kernel
            .complete_synthetic_effect(
                principal,
                CompleteSyntheticEffect {
                    effect_id,
                    attempt_id: prepared.attempt_id(),
                    finished_at: "2026-08-25T13:45:01Z",
                    completion: SyntheticExecutionCompletion::Succeeded,
                    reason_code: Some("simulated_success"),
                    evidence_ref: None,
                    receipt: Some(b"receipt"),
                },
                "local-owner",
            )
            .unwrap();
        assert_eq!(outcome.state, "succeeded");
        assert_eq!(outcome.receipt.as_deref(), Some(b"receipt".as_slice()));
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn unknown_synthetic_effect_enters_manual_review_when_reconciliation_stays_ambiguous() {
        let runtime = runtime();
        let mut kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let principal = Principal::local_owner("owner");
        let effect_id = EffectId(200);
        let prepared = kernel
            .prepare_synthetic_effect(
                principal,
                PrepareSyntheticEffect {
                    effect_id,
                    session_id: SessionId(8),
                    execution_semantics: "irreversible",
                    handler_id: "sim-irreversible-write",
                    handler_version: "1",
                    idempotency_key: None,
                    payload_hash: [8; 32],
                    started_at: "2026-08-25T13:46:00Z",
                },
                "local-owner",
            )
            .unwrap();
        kernel
            .complete_synthetic_effect(
                principal,
                CompleteSyntheticEffect {
                    effect_id,
                    attempt_id: prepared.attempt_id(),
                    finished_at: "2026-08-25T13:46:01Z",
                    completion: SyntheticExecutionCompletion::UnknownOutcome,
                    reason_code: Some("accepted_without_ack"),
                    evidence_ref: Some(b"ambiguous"),
                    receipt: None,
                },
                "local-owner",
            )
            .unwrap();
        let context = kernel
            .begin_synthetic_reconciliation(principal, effect_id, "local-owner")
            .unwrap();
        assert_eq!(context.effect_id, effect_id);
        assert_eq!(context.attempt_id, prepared.attempt_id());
        let result = kernel
            .resolve_synthetic_reconciliation(
                principal,
                effect_id,
                SyntheticExecutionCompletion::UnknownOutcome,
                Some("still_ambiguous"),
                Some(b"status-missing"),
                "2026-08-25T13:46:02Z",
                "local-owner",
            )
            .unwrap();
        assert!(matches!(
            result,
            SyntheticReconciliationResult::ManualReview { effect_id: id, .. } if id == effect_id
        ));
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
