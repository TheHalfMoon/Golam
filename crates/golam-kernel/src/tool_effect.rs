#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::{
    EffectAttemptId, EffectId, EffectTransitionId, EventId, SessionId, ToolReconciliationContext,
    ToolReconciliationResolution, ToolReconciliationResult,
};
use golam_ledger::dispatch::{
    EffectDispatchStoreError, PrepareEffectDispatch, encode_effect_dependencies,
};
use golam_ledger::effect_completion::{
    CompleteEffectExecution, EffectCompletionError, EffectCompletionStore, ExecutionCompletion,
};
use golam_ledger::effect_read::{EffectReadError, EffectReader, EffectSnapshot};
use golam_ledger::effects::{
    CompareAndSwapEffect, EffectStore, EffectStoreError, ProposeEffect, StoredEffectAttempt,
};
use golam_ledger::manual_review::{
    ManualReviewError, ManualReviewReason, ManualReviewStore, PlaceEffectInManualReview,
};

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
const STAGE_RECONCILING_TRANSITION: u128 = 10;
const STAGE_RECONCILING_EVENT: u128 = 11;
const STAGE_RESOLUTION_TRANSITION: u128 = 12;
const STAGE_RESOLUTION_EVENT: u128 = 13;
const STAGE_MANUAL_REVIEW_TRANSITION: u128 = 14;
const STAGE_MANUAL_REVIEW_EVENT: u128 = 15;
const STAGE_RECOVERY_UNKNOWN_TRANSITION: u128 = 16;
const STAGE_RECOVERY_UNKNOWN_EVENT: u128 = 17;

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
    Read(EffectReadError),
    ManualReview(ManualReviewError),
    InvalidMetadata,
    UnsupportedSemantics(String),
    IdentifierOverflow(EffectId),
    EffectNotFound(EffectId),
    MissingAttempt(EffectId),
    NotToolEffect(EffectId),
    InvalidStoredPreconditions(EffectId),
    NotReconcilable { effect_id: EffectId, actual: String },
    MissingTerminalEvidence(EffectId),
}

impl fmt::Display for ToolEffectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kernel(error) => write!(f, "tool effect kernel error: {error}"),
            Self::Dispatch(error) => write!(f, "tool effect dispatch error: {error}"),
            Self::Store(error) => write!(f, "tool effect store error: {error}"),
            Self::Completion(error) => write!(f, "tool effect completion error: {error}"),
            Self::Read(error) => write!(f, "tool effect read error: {error}"),
            Self::ManualReview(error) => write!(f, "tool effect manual-review error: {error}"),
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
            Self::EffectNotFound(effect_id) => {
                write!(f, "tool effect not found: {}", effect_id.0)
            }
            Self::MissingAttempt(effect_id) => {
                write!(f, "tool effect has no durable attempt: {}", effect_id.0)
            }
            Self::NotToolEffect(effect_id) => {
                write!(
                    f,
                    "effect is not an exact durable tool effect: {}",
                    effect_id.0
                )
            }
            Self::InvalidStoredPreconditions(effect_id) => write!(
                f,
                "tool effect stored precondition binding is malformed: {}",
                effect_id.0
            ),
            Self::NotReconcilable { effect_id, actual } => write!(
                f,
                "tool effect is not reconcilable: effect={} state={actual}",
                effect_id.0
            ),
            Self::MissingTerminalEvidence(effect_id) => write!(
                f,
                "tool effect reconciliation cannot resolve success without terminal evidence: {}",
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
            Self::Read(error) => Some(error),
            Self::ManualReview(error) => Some(error),
            Self::InvalidMetadata
            | Self::UnsupportedSemantics(_)
            | Self::IdentifierOverflow(_)
            | Self::EffectNotFound(_)
            | Self::MissingAttempt(_)
            | Self::NotToolEffect(_)
            | Self::InvalidStoredPreconditions(_)
            | Self::NotReconcilable { .. }
            | Self::MissingTerminalEvidence(_) => None,
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

impl From<EffectReadError> for ToolEffectError {
    fn from(value: EffectReadError) -> Self {
        Self::Read(value)
    }
}

impl From<ManualReviewError> for ToolEffectError {
    fn from(value: ManualReviewError) -> Self {
        Self::ManualReview(value)
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

    pub fn begin_tool_reconciliation(
        &mut self,
        principal: Principal<'_>,
        effect_id: EffectId,
        detected_at: &str,
        scope: &str,
    ) -> Result<ToolReconciliationContext, ToolEffectError> {
        if detected_at.is_empty() {
            return Err(ToolEffectError::InvalidMetadata);
        }
        let reader = EffectReader::open(&self.authority)?;
        let snapshot = reader
            .snapshot(effect_id)?
            .ok_or(ToolEffectError::EffectNotFound(effect_id))?;
        let attempt = snapshot
            .latest_attempt
            .clone()
            .ok_or(ToolEffectError::MissingAttempt(effect_id))?;
        validate_tool_snapshot(&snapshot, &attempt)?;
        drop(reader);

        self.require_authority(&AuthorizationRequest {
            principal,
            action: &snapshot.action,
            resource: &snapshot.resource,
            context: AuthorizationContext::local(scope),
        })?;

        match snapshot.current_state.as_str() {
            "executing" => {
                let mut completion = EffectCompletionStore::open(&self.authority)?;
                completion.complete(CompleteEffectExecution {
                    effect_id,
                    attempt_id: attempt.attempt_id,
                    transition_id: EffectTransitionId(stage_id(
                        effect_id,
                        STAGE_RECOVERY_UNKNOWN_TRANSITION,
                    )?),
                    event_id: EventId(stage_id(effect_id, STAGE_RECOVERY_UNKNOWN_EVENT)?),
                    finished_at: detected_at,
                    completion: ExecutionCompletion::UnknownOutcome,
                    reason_code: Some("interrupted_tool_dispatch_recovered_as_unknown_outcome"),
                    evidence_ref: None,
                    receipt: None,
                })?;
            }
            "unknown_outcome" => {}
            "reconciling" => return reconciliation_context(snapshot, attempt),
            actual => {
                return Err(ToolEffectError::NotReconcilable {
                    effect_id,
                    actual: actual.to_owned(),
                });
            }
        }

        let mut effects = EffectStore::open(&self.authority)?;
        effects.compare_and_swap(CompareAndSwapEffect {
            transition_id: EffectTransitionId(stage_id(effect_id, STAGE_RECONCILING_TRANSITION)?),
            effect_id,
            expected_state: "unknown_outcome",
            next_state: "reconciling",
            attempt_id: Some(attempt.attempt_id),
            reason_code: Some("tool_reconciliation_started"),
            evidence_ref: None,
            event_id: EventId(stage_id(effect_id, STAGE_RECONCILING_EVENT)?),
        })?;
        reconciliation_context(snapshot, attempt)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "reconciliation keeps authority, effect identity, terminal classification, evidence and observation time explicit"
    )]
    pub fn resolve_tool_reconciliation(
        &mut self,
        principal: Principal<'_>,
        effect_id: EffectId,
        resolution: ToolReconciliationResolution,
        reason_code: Option<&str>,
        evidence_ref: Option<&[u8]>,
        detected_at: &str,
        scope: &str,
    ) -> Result<ToolReconciliationResult, ToolEffectError> {
        let reader = EffectReader::open(&self.authority)?;
        let snapshot = reader
            .snapshot(effect_id)?
            .ok_or(ToolEffectError::EffectNotFound(effect_id))?;
        let attempt = snapshot
            .latest_attempt
            .as_ref()
            .ok_or(ToolEffectError::MissingAttempt(effect_id))?;
        validate_tool_snapshot(&snapshot, attempt)?;
        if snapshot.current_state != "reconciling" {
            return Err(ToolEffectError::NotReconcilable {
                effect_id,
                actual: snapshot.current_state,
            });
        }
        let attempt_id = attempt.attempt_id;
        drop(reader);

        self.require_authority(&AuthorizationRequest {
            principal,
            action: &snapshot.action,
            resource: &snapshot.resource,
            context: AuthorizationContext::local(scope),
        })?;

        match resolution {
            ToolReconciliationResolution::Succeeded => {
                if evidence_ref.is_none_or(<[u8]>::is_empty) {
                    return Err(ToolEffectError::MissingTerminalEvidence(effect_id));
                }
                resolve_tool_terminal(
                    &self.authority,
                    effect_id,
                    attempt_id,
                    "succeeded",
                    reason_code,
                    evidence_ref,
                )?;
                Ok(ToolReconciliationResult::Resolved {
                    effect_id,
                    state: "succeeded".to_owned(),
                })
            }
            ToolReconciliationResolution::Failed => {
                resolve_tool_terminal(
                    &self.authority,
                    effect_id,
                    attempt_id,
                    "failed",
                    reason_code,
                    evidence_ref,
                )?;
                Ok(ToolReconciliationResult::Resolved {
                    effect_id,
                    state: "failed".to_owned(),
                })
            }
            ToolReconciliationResolution::UnknownOutcome => {
                if detected_at.is_empty() {
                    return Err(ToolEffectError::InvalidMetadata);
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
                Ok(ToolReconciliationResult::ManualReview {
                    effect_id,
                    incident_id: report.incident_id,
                })
            }
        }
    }
}

fn resolve_tool_terminal(
    authority: &golam_core::authority::AuthorityLayout,
    effect_id: EffectId,
    attempt_id: EffectAttemptId,
    state: &str,
    reason_code: Option<&str>,
    evidence_ref: Option<&[u8]>,
) -> Result<(), ToolEffectError> {
    let mut effects = EffectStore::open(authority)?;
    effects.compare_and_swap(CompareAndSwapEffect {
        transition_id: EffectTransitionId(stage_id(effect_id, STAGE_RESOLUTION_TRANSITION)?),
        effect_id,
        expected_state: "reconciling",
        next_state: state,
        attempt_id: Some(attempt_id),
        reason_code,
        evidence_ref,
        event_id: EventId(stage_id(effect_id, STAGE_RESOLUTION_EVENT)?),
    })?;
    Ok(())
}

fn reconciliation_context(
    snapshot: EffectSnapshot,
    attempt: StoredEffectAttempt,
) -> Result<ToolReconciliationContext, ToolEffectError> {
    let preconditions_hash = stored_preconditions_hash(&snapshot)?;
    Ok(ToolReconciliationContext {
        effect_id: snapshot.effect_id,
        session_id: snapshot.session_id,
        action: snapshot.action,
        resource: snapshot.resource,
        execution_semantics: snapshot.execution_semantics,
        idempotency_key: snapshot.idempotency_key,
        preconditions_hash,
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

fn validate_tool_snapshot(
    snapshot: &EffectSnapshot,
    attempt: &StoredEffectAttempt,
) -> Result<(), ToolEffectError> {
    if snapshot.risk_class != "tool_mutation"
        || snapshot.action.is_empty()
        || snapshot.resource.is_empty()
        || validate_semantics(&snapshot.execution_semantics).is_err()
        || attempt.effect_id != snapshot.effect_id
        || attempt.attempt_id != EffectAttemptId(stage_id(snapshot.effect_id, STAGE_ATTEMPT)?)
        || attempt.handler_id.is_empty()
        || attempt.handler_version.is_empty()
    {
        return Err(ToolEffectError::NotToolEffect(snapshot.effect_id));
    }
    let preconditions_hash = stored_preconditions_hash(snapshot)?;
    let expected_dispatch_token = dispatch_token(
        snapshot.effect_id,
        preconditions_hash,
        snapshot.payload_hash,
    );
    if attempt.dispatch_token != expected_dispatch_token {
        return Err(ToolEffectError::NotToolEffect(snapshot.effect_id));
    }
    Ok(())
}

fn stored_preconditions_hash(snapshot: &EffectSnapshot) -> Result<[u8; 32], ToolEffectError> {
    snapshot
        .preconditions
        .as_slice()
        .try_into()
        .map_err(|_| ToolEffectError::InvalidStoredPreconditions(snapshot.effect_id))
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
    use crate::{KernelCreateSession, PolicyDecision};
    use golam_core::paths::RuntimeLayout;
    use golam_ledger::effects::EffectStore;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    struct AllowTools;

    impl AuthorizationPolicy for AllowTools {
        fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
            PolicyDecision::allow("phase_f_tool_reconciliation_qualification")
        }
    }

    fn runtime() -> RuntimeLayout {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-tool-effect-reconcile-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    fn prepare(kernel: &mut KernelApi<AllowTools>, effect_id: EffectId) -> PreparedToolEffect {
        kernel
            .prepare_tool_effect(
                Principal::test("phase-f-reconcile"),
                PrepareToolEffect {
                    effect_id,
                    session_id: SessionId(65),
                    action: "git.add",
                    resource: "git-add:note.txt",
                    execution_semantics: "at_most_once",
                    handler_id: "golam-git-linux",
                    handler_version: "1",
                    idempotency_key: Some("phase-f-reconciliation"),
                    preconditions_hash: [11; 32],
                    payload_hash: [12; 32],
                    started_at: "2026-09-05T00:00:01Z",
                },
                "phase-f-reconcile",
            )
            .unwrap()
    }

    fn kernel_with_session(runtime: &RuntimeLayout) -> KernelApi<AllowTools> {
        let mut kernel = KernelApi::open(runtime, AllowTools).unwrap();
        kernel
            .create_session(
                Principal::test("phase-f-reconcile"),
                KernelCreateSession {
                    session_id: SessionId(65),
                    event_id: EventId(1),
                    recorded_at: "2026-09-05T00:00:00Z",
                    payload: b"phase-f-tool-reconciliation",
                },
                "phase-f-reconcile",
            )
            .unwrap();
        kernel
    }

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

    #[test]
    fn interrupted_tool_effect_reconciles_without_redispatch_and_preserves_binding() {
        let runtime = runtime();
        let mut kernel = kernel_with_session(&runtime);
        let effect_id = EffectId(650);
        let prepared = prepare(&mut kernel, effect_id);
        let context = kernel
            .begin_tool_reconciliation(
                Principal::test("phase-f-reconcile"),
                effect_id,
                "2026-09-05T00:00:02Z",
                "phase-f-reconcile",
            )
            .unwrap();
        assert_eq!(context.attempt_id, prepared.attempt_id());
        assert_eq!(context.preconditions_hash, [11; 32]);
        assert_eq!(context.payload_hash, [12; 32]);
        assert_eq!(
            context.dispatch_token,
            dispatch_token(effect_id, [11; 32], [12; 32])
        );

        let effects = EffectStore::open(&kernel.authority).unwrap();
        assert_eq!(effects.attempt_count(effect_id).unwrap(), 1);
        assert_eq!(
            effects.current_state(effect_id).unwrap().as_deref(),
            Some("reconciling")
        );
        drop(effects);
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn reconciliation_success_requires_terminal_evidence_and_ambiguity_goes_manual_review() {
        let runtime = runtime();
        let mut kernel = kernel_with_session(&runtime);
        let effect_id = EffectId(651);
        prepare(&mut kernel, effect_id);
        kernel
            .begin_tool_reconciliation(
                Principal::test("phase-f-reconcile"),
                effect_id,
                "2026-09-05T00:00:02Z",
                "phase-f-reconcile",
            )
            .unwrap();

        assert!(matches!(
            kernel.resolve_tool_reconciliation(
                Principal::test("phase-f-reconcile"),
                effect_id,
                ToolReconciliationResolution::Succeeded,
                Some("readback_verified"),
                None,
                "2026-09-05T00:00:03Z",
                "phase-f-reconcile",
            ),
            Err(ToolEffectError::MissingTerminalEvidence(id)) if id == effect_id
        ));
        let resolved = kernel
            .resolve_tool_reconciliation(
                Principal::test("phase-f-reconcile"),
                effect_id,
                ToolReconciliationResolution::Succeeded,
                Some("readback_verified"),
                Some(b"exact-target-readback-evidence"),
                "2026-09-05T00:00:03Z",
                "phase-f-reconcile",
            )
            .unwrap();
        assert_eq!(
            resolved,
            ToolReconciliationResult::Resolved {
                effect_id,
                state: "succeeded".to_owned()
            }
        );

        let second = EffectId(652);
        prepare(&mut kernel, second);
        kernel
            .begin_tool_reconciliation(
                Principal::test("phase-f-reconcile"),
                second,
                "2026-09-05T00:00:04Z",
                "phase-f-reconcile",
            )
            .unwrap();
        assert!(matches!(
            kernel
                .resolve_tool_reconciliation(
                    Principal::test("phase-f-reconcile"),
                    second,
                    ToolReconciliationResolution::UnknownOutcome,
                    Some("still_ambiguous"),
                    Some(b"contradictory-readback-evidence"),
                    "2026-09-05T00:00:05Z",
                    "phase-f-reconcile",
                )
                .unwrap(),
            ToolReconciliationResult::ManualReview { effect_id, .. } if effect_id == second
        ));
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
