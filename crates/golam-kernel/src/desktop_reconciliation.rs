#![forbid(unsafe_code)]

use core::fmt;

use golam_core::digest::sha256;
use golam_core::tool_request::BindingDigest;
use golam_core::{
    CanonicalEncoder, CoreError, EffectId, ToolReconciliationContext, ToolReconciliationResolution,
    ToolReconciliationResult,
};
use golam_ledger::desktop_control_evidence::{
    DesktopControlEvidenceError, DesktopControlEvidenceStore, DesktopEvidenceStatus,
};
use golam_ledger::effect_read::{EffectReadError, EffectReader, EffectSnapshot};
use golam_ledger::effects::StoredEffectAttempt;

use crate::{
    AuthorizationContext, AuthorizationPolicy, AuthorizationRequest, KernelApi, Principal,
    ToolEffectError, ToolMutationEvidenceKernelError, ToolMutationVerifiedStatus,
};

const DESKTOP_RECONCILIATION_DOMAIN: &[u8] = b"golam:desktop-reconciliation:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopReconciliationRepair {
    AlreadyConsistent(DesktopEvidenceStatus),
    Repaired(DesktopEvidenceStatus),
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn begin_desktop_reconciliation(
        &mut self,
        principal: Principal<'_>,
        effect_id: EffectId,
        detected_at: &str,
        recorded_at_unix_ms: u64,
        scope: &str,
    ) -> Result<ToolReconciliationContext, DesktopReconciliationError> {
        if recorded_at_unix_ms == 0 {
            return Err(DesktopReconciliationError::InvalidMetadata);
        }
        let context = self.begin_tool_reconciliation(principal, effect_id, detected_at, scope)?;
        let reconciliation_ref = reconciliation_ref(&context)?;
        let mut store = DesktopControlEvidenceStore::open(&self.authority)?;
        let status = store.latest_effect_status(effect_id)?.ok_or(
            DesktopReconciliationError::MissingDesktopEvidence(effect_id),
        )?;
        match status {
            DesktopEvidenceStatus::Prepared => {
                let recovered = store.recovered_unknown_evidence(effect_id, recorded_at_unix_ms)?;
                store.append_effect_evidence(recovered)?;
                let reconciling = store.reconciliation_evidence(
                    effect_id,
                    DesktopEvidenceStatus::Reconciling,
                    reconciliation_ref,
                    recorded_at_unix_ms,
                )?;
                store.append_effect_evidence(reconciling)?;
            }
            DesktopEvidenceStatus::UnknownOutcome => {
                let reconciling = store.reconciliation_evidence(
                    effect_id,
                    DesktopEvidenceStatus::Reconciling,
                    reconciliation_ref,
                    recorded_at_unix_ms,
                )?;
                store.append_effect_evidence(reconciling)?;
            }
            DesktopEvidenceStatus::Reconciling => {
                if store.latest_effect_reconciliation_ref(effect_id)? != Some(reconciliation_ref) {
                    return Err(DesktopReconciliationError::ReconciliationBindingMismatch(
                        effect_id,
                    ));
                }
            }
            actual => {
                return Err(DesktopReconciliationError::DesktopNotReconcilable {
                    effect_id,
                    actual,
                });
            }
        }
        Ok(context)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "desktop reconciliation keeps authority, effect identity, resolution, evidence, observation times and scope explicit"
    )]
    pub fn resolve_desktop_reconciliation(
        &mut self,
        principal: Principal<'_>,
        effect_id: EffectId,
        resolution: ToolReconciliationResolution,
        reason_code: Option<&str>,
        evidence_ref: Option<&[u8]>,
        detected_at: &str,
        recorded_at_unix_ms: u64,
        scope: &str,
    ) -> Result<ToolReconciliationResult, DesktopReconciliationError> {
        let context = self.begin_desktop_reconciliation(
            principal,
            effect_id,
            detected_at,
            recorded_at_unix_ms,
            scope,
        )?;
        let reconciliation_ref = reconciliation_ref(&context)?;
        let result = self.resolve_tool_reconciliation(
            principal,
            effect_id,
            resolution,
            reason_code,
            evidence_ref,
            detected_at,
            scope,
        )?;
        let status = match &result {
            ToolReconciliationResult::Resolved { state, .. } if state == "succeeded" => {
                DesktopEvidenceStatus::ReconciledSucceeded
            }
            ToolReconciliationResult::Resolved { state, .. } if state == "failed" => {
                DesktopEvidenceStatus::ReconciledFailed
            }
            ToolReconciliationResult::ManualReview { .. } => DesktopEvidenceStatus::ManualReview,
            ToolReconciliationResult::Resolved { state, .. } => {
                return Err(DesktopReconciliationError::UnexpectedTerminalState(
                    state.clone(),
                ));
            }
        };
        let mut store = DesktopControlEvidenceStore::open(&self.authority)?;
        let terminal = store.reconciliation_evidence(
            effect_id,
            status,
            reconciliation_ref,
            recorded_at_unix_ms,
        )?;
        store.append_effect_evidence(terminal)?;
        Ok(result)
    }

    /// Repairs only evidence-write crash windows. It never re-dispatches an
    /// external desktop effect and never promotes a generic unknown/reconciling
    /// state to terminal truth.
    pub fn repair_desktop_reconciliation_evidence(
        &mut self,
        principal: Principal<'_>,
        effect_id: EffectId,
        recorded_at_unix_ms: u64,
        scope: &str,
    ) -> Result<DesktopReconciliationRepair, DesktopReconciliationError> {
        if recorded_at_unix_ms == 0 {
            return Err(DesktopReconciliationError::InvalidMetadata);
        }
        let reader = EffectReader::open(&self.authority)?;
        let snapshot = reader
            .snapshot(effect_id)?
            .ok_or(DesktopReconciliationError::MissingEffect(effect_id))?;
        let attempt = snapshot
            .latest_attempt
            .as_ref()
            .ok_or(DesktopReconciliationError::MissingAttempt(effect_id))?;
        validate_repair_snapshot(&snapshot, attempt)?;
        self.require_authority(&AuthorizationRequest {
            principal,
            action: &snapshot.action,
            resource: &snapshot.resource,
            context: AuthorizationContext::local(scope),
        })?;
        let mut store = DesktopControlEvidenceStore::open(&self.authority)?;
        let desktop_status = store.latest_effect_status(effect_id)?.ok_or(
            DesktopReconciliationError::MissingDesktopEvidence(effect_id),
        )?;

        match snapshot.current_state.as_str() {
            "succeeded" => self.repair_terminal_desktop_state(
                &mut store,
                &snapshot,
                attempt,
                desktop_status,
                DesktopEvidenceStatus::Succeeded,
                DesktopEvidenceStatus::ReconciledSucceeded,
                ToolMutationVerifiedStatus::Succeeded,
                recorded_at_unix_ms,
            ),
            "failed" => self.repair_terminal_desktop_state(
                &mut store,
                &snapshot,
                attempt,
                desktop_status,
                DesktopEvidenceStatus::Failed,
                DesktopEvidenceStatus::ReconciledFailed,
                ToolMutationVerifiedStatus::Failed,
                recorded_at_unix_ms,
            ),
            "manual_review" => repair_manual_review_state(
                &mut store,
                &snapshot,
                attempt,
                desktop_status,
                recorded_at_unix_ms,
            ),
            actual => Err(DesktopReconciliationError::GenericEffectNotTerminal {
                effect_id,
                actual: actual.to_owned(),
            }),
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "repair keeps direct and reconciled terminal classifications plus verified status explicit"
    )]
    fn repair_terminal_desktop_state(
        &self,
        store: &mut DesktopControlEvidenceStore,
        snapshot: &EffectSnapshot,
        attempt: &StoredEffectAttempt,
        desktop_status: DesktopEvidenceStatus,
        direct_status: DesktopEvidenceStatus,
        reconciled_status: DesktopEvidenceStatus,
        verified_status: ToolMutationVerifiedStatus,
        recorded_at_unix_ms: u64,
    ) -> Result<DesktopReconciliationRepair, DesktopReconciliationError> {
        if desktop_status == direct_status || desktop_status == reconciled_status {
            return Ok(DesktopReconciliationRepair::AlreadyConsistent(
                desktop_status,
            ));
        }

        if desktop_status == DesktopEvidenceStatus::Prepared
            && attempt.outcome == snapshot.current_state
        {
            let terminal = store.binding_terminal_evidence(
                snapshot.effect_id,
                direct_status,
                recorded_at_unix_ms,
            )?;
            store.append_effect_evidence(terminal)?;
            return Ok(DesktopReconciliationRepair::Repaired(direct_status));
        }

        if !matches!(
            desktop_status,
            DesktopEvidenceStatus::Prepared
                | DesktopEvidenceStatus::UnknownOutcome
                | DesktopEvidenceStatus::Reconciling
        ) {
            return Err(DesktopReconciliationError::DesktopTerminalMismatch {
                effect_id: snapshot.effect_id,
                actual: desktop_status,
            });
        }

        let preconditions_hash = stored_preconditions_hash(snapshot)?;
        self.require_verified_tool_mutation_receipt(
            snapshot.effect_id,
            &snapshot.action,
            &snapshot.resource,
            preconditions_hash,
            snapshot.payload_hash,
            verified_status,
        )?;
        let reconciliation_ref = reconciliation_ref_from_snapshot(snapshot, attempt)?;
        normalize_to_reconciling(
            store,
            snapshot.effect_id,
            desktop_status,
            reconciliation_ref,
            recorded_at_unix_ms,
        )?;
        let terminal = store.reconciliation_evidence(
            snapshot.effect_id,
            reconciled_status,
            reconciliation_ref,
            recorded_at_unix_ms,
        )?;
        store.append_effect_evidence(terminal)?;
        Ok(DesktopReconciliationRepair::Repaired(reconciled_status))
    }
}

fn repair_manual_review_state(
    store: &mut DesktopControlEvidenceStore,
    snapshot: &EffectSnapshot,
    attempt: &StoredEffectAttempt,
    desktop_status: DesktopEvidenceStatus,
    recorded_at_unix_ms: u64,
) -> Result<DesktopReconciliationRepair, DesktopReconciliationError> {
    if desktop_status == DesktopEvidenceStatus::ManualReview {
        return Ok(DesktopReconciliationRepair::AlreadyConsistent(
            desktop_status,
        ));
    }
    if !matches!(
        desktop_status,
        DesktopEvidenceStatus::Prepared
            | DesktopEvidenceStatus::UnknownOutcome
            | DesktopEvidenceStatus::Reconciling
    ) {
        return Err(DesktopReconciliationError::DesktopTerminalMismatch {
            effect_id: snapshot.effect_id,
            actual: desktop_status,
        });
    }
    let reconciliation_ref = reconciliation_ref_from_snapshot(snapshot, attempt)?;
    normalize_to_reconciling(
        store,
        snapshot.effect_id,
        desktop_status,
        reconciliation_ref,
        recorded_at_unix_ms,
    )?;
    let terminal = store.reconciliation_evidence(
        snapshot.effect_id,
        DesktopEvidenceStatus::ManualReview,
        reconciliation_ref,
        recorded_at_unix_ms,
    )?;
    store.append_effect_evidence(terminal)?;
    Ok(DesktopReconciliationRepair::Repaired(
        DesktopEvidenceStatus::ManualReview,
    ))
}

fn normalize_to_reconciling(
    store: &mut DesktopControlEvidenceStore,
    effect_id: EffectId,
    desktop_status: DesktopEvidenceStatus,
    reconciliation_ref: BindingDigest,
    recorded_at_unix_ms: u64,
) -> Result<(), DesktopReconciliationError> {
    match desktop_status {
        DesktopEvidenceStatus::Prepared => {
            let unknown = store.recovered_unknown_evidence(effect_id, recorded_at_unix_ms)?;
            store.append_effect_evidence(unknown)?;
            let reconciling = store.reconciliation_evidence(
                effect_id,
                DesktopEvidenceStatus::Reconciling,
                reconciliation_ref,
                recorded_at_unix_ms,
            )?;
            store.append_effect_evidence(reconciling)?;
        }
        DesktopEvidenceStatus::UnknownOutcome => {
            let reconciling = store.reconciliation_evidence(
                effect_id,
                DesktopEvidenceStatus::Reconciling,
                reconciliation_ref,
                recorded_at_unix_ms,
            )?;
            store.append_effect_evidence(reconciling)?;
        }
        DesktopEvidenceStatus::Reconciling => {
            if store.latest_effect_reconciliation_ref(effect_id)? != Some(reconciliation_ref) {
                return Err(DesktopReconciliationError::ReconciliationBindingMismatch(
                    effect_id,
                ));
            }
        }
        actual => {
            return Err(DesktopReconciliationError::DesktopNotReconcilable { effect_id, actual });
        }
    }
    Ok(())
}

fn reconciliation_ref(
    context: &ToolReconciliationContext,
) -> Result<BindingDigest, DesktopReconciliationError> {
    reconciliation_ref_fields(
        context.effect_id,
        context.session_id.0,
        &context.action,
        &context.resource,
        &context.execution_semantics,
        context.idempotency_key.as_deref(),
        context.preconditions_hash,
        context.payload_hash,
        context.attempt_id.0,
        context.started_global_seq,
        &context.handler_id,
        &context.handler_version,
        &context.dispatch_token,
    )
}

fn reconciliation_ref_from_snapshot(
    snapshot: &EffectSnapshot,
    attempt: &StoredEffectAttempt,
) -> Result<BindingDigest, DesktopReconciliationError> {
    reconciliation_ref_fields(
        snapshot.effect_id,
        snapshot.session_id.0,
        &snapshot.action,
        &snapshot.resource,
        &snapshot.execution_semantics,
        snapshot.idempotency_key.as_deref(),
        stored_preconditions_hash(snapshot)?,
        snapshot.payload_hash,
        attempt.attempt_id.0,
        attempt.started_global_seq,
        &attempt.handler_id,
        &attempt.handler_version,
        &attempt.dispatch_token,
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "the reconciliation identity intentionally binds every immutable effect and attempt field"
)]
fn reconciliation_ref_fields(
    effect_id: EffectId,
    session_id: u128,
    action: &str,
    resource: &str,
    execution_semantics: &str,
    idempotency_key: Option<&str>,
    preconditions_hash: [u8; 32],
    payload_hash: [u8; 32],
    attempt_id: u128,
    started_global_seq: u64,
    handler_id: &str,
    handler_version: &str,
    dispatch_token: &[u8],
) -> Result<BindingDigest, DesktopReconciliationError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(DESKTOP_RECONCILIATION_DOMAIN)?;
    encoder.push_u128(effect_id.0);
    encoder.push_u128(session_id);
    encoder.push_bytes(action.as_bytes())?;
    encoder.push_bytes(resource.as_bytes())?;
    encoder.push_bytes(execution_semantics.as_bytes())?;
    match idempotency_key {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(value.as_bytes())?;
        }
        None => encoder.push_u8(0),
    }
    encoder.push_bytes(&preconditions_hash)?;
    encoder.push_bytes(&payload_hash)?;
    encoder.push_u128(attempt_id);
    encoder.push_u64(started_global_seq);
    encoder.push_bytes(handler_id.as_bytes())?;
    encoder.push_bytes(handler_version.as_bytes())?;
    encoder.push_bytes(dispatch_token)?;
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn validate_repair_snapshot(
    snapshot: &EffectSnapshot,
    attempt: &StoredEffectAttempt,
) -> Result<(), DesktopReconciliationError> {
    if snapshot.risk_class != "tool_mutation"
        || snapshot.action.is_empty()
        || snapshot.resource.is_empty()
        || attempt.effect_id != snapshot.effect_id
        || attempt.handler_id.is_empty()
        || attempt.handler_version.is_empty()
        || attempt.dispatch_token.is_empty()
    {
        return Err(DesktopReconciliationError::InvalidEffectBinding(
            snapshot.effect_id,
        ));
    }
    let _ = stored_preconditions_hash(snapshot)?;
    Ok(())
}

fn stored_preconditions_hash(
    snapshot: &EffectSnapshot,
) -> Result<[u8; 32], DesktopReconciliationError> {
    snapshot
        .preconditions
        .as_slice()
        .try_into()
        .map_err(|_| DesktopReconciliationError::InvalidEffectBinding(snapshot.effect_id))
}

#[derive(Debug)]
pub enum DesktopReconciliationError {
    InvalidMetadata,
    Tool(ToolEffectError),
    Evidence(DesktopControlEvidenceError),
    Read(EffectReadError),
    MutationEvidence(ToolMutationEvidenceKernelError),
    Core(CoreError),
    Kernel(crate::KernelError),
    MissingEffect(EffectId),
    MissingAttempt(EffectId),
    MissingDesktopEvidence(EffectId),
    InvalidEffectBinding(EffectId),
    ReconciliationBindingMismatch(EffectId),
    DesktopNotReconcilable {
        effect_id: EffectId,
        actual: DesktopEvidenceStatus,
    },
    DesktopTerminalMismatch {
        effect_id: EffectId,
        actual: DesktopEvidenceStatus,
    },
    GenericEffectNotTerminal {
        effect_id: EffectId,
        actual: String,
    },
    UnexpectedTerminalState(String),
}

impl fmt::Display for DesktopReconciliationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMetadata => f.write_str("invalid desktop reconciliation metadata"),
            Self::Tool(error) => write!(f, "desktop reconciliation tool effect error: {error}"),
            Self::Evidence(error) => write!(f, "desktop reconciliation evidence error: {error}"),
            Self::Read(error) => write!(f, "desktop reconciliation effect read error: {error}"),
            Self::MutationEvidence(error) => {
                write!(f, "desktop reconciliation mutation evidence error: {error}")
            }
            Self::Core(error) => write!(f, "desktop reconciliation encoding error: {error}"),
            Self::Kernel(error) => write!(f, "desktop reconciliation kernel error: {error}"),
            Self::MissingEffect(effect_id) => {
                write!(
                    f,
                    "desktop reconciliation effect {} is missing",
                    effect_id.0
                )
            }
            Self::MissingAttempt(effect_id) => write!(
                f,
                "desktop reconciliation effect {} has no attempt",
                effect_id.0
            ),
            Self::MissingDesktopEvidence(effect_id) => write!(
                f,
                "desktop reconciliation effect {} has no desktop evidence",
                effect_id.0
            ),
            Self::InvalidEffectBinding(effect_id) => write!(
                f,
                "desktop reconciliation effect {} has invalid protected bindings",
                effect_id.0
            ),
            Self::ReconciliationBindingMismatch(effect_id) => write!(
                f,
                "desktop reconciliation reference mismatch for effect {}",
                effect_id.0
            ),
            Self::DesktopNotReconcilable { effect_id, actual } => write!(
                f,
                "desktop effect {} is not reconcilable from state {actual:?}",
                effect_id.0
            ),
            Self::DesktopTerminalMismatch { effect_id, actual } => write!(
                f,
                "desktop effect {} terminal evidence conflicts with generic ledger: {actual:?}",
                effect_id.0
            ),
            Self::GenericEffectNotTerminal { effect_id, actual } => write!(
                f,
                "generic effect {} is not terminal for desktop evidence repair: {actual}",
                effect_id.0
            ),
            Self::UnexpectedTerminalState(state) => {
                write!(
                    f,
                    "unexpected generic reconciliation terminal state: {state}"
                )
            }
        }
    }
}

impl std::error::Error for DesktopReconciliationError {}

impl From<ToolEffectError> for DesktopReconciliationError {
    fn from(value: ToolEffectError) -> Self {
        Self::Tool(value)
    }
}

impl From<DesktopControlEvidenceError> for DesktopReconciliationError {
    fn from(value: DesktopControlEvidenceError) -> Self {
        Self::Evidence(value)
    }
}

impl From<EffectReadError> for DesktopReconciliationError {
    fn from(value: EffectReadError) -> Self {
        Self::Read(value)
    }
}

impl From<ToolMutationEvidenceKernelError> for DesktopReconciliationError {
    fn from(value: ToolMutationEvidenceKernelError) -> Self {
        Self::MutationEvidence(value)
    }
}

impl From<CoreError> for DesktopReconciliationError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<crate::KernelError> for DesktopReconciliationError {
    fn from(value: crate::KernelError) -> Self {
        Self::Kernel(value)
    }
}
