#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use golam_core::EffectId;
pub use golam_ledger::tool_mutation_evidence::ToolMutationVerifiedStatus;
use golam_ledger::tool_mutation_evidence::{
    RecordToolMutationIntent, RecordToolMutationReceipt, ToolMutationEvidenceError,
    ToolMutationEvidenceStore,
};

use crate::{
    AuthorizationContext, AuthorizationPolicy, AuthorizationRequest, KernelApi, KernelError,
    PreparedToolEffect, Principal,
};

const TOOL_CONTEXT_EVIDENCE_DB: &str = "tool-context-evidence.sqlite";

#[derive(Debug)]
pub enum ToolMutationEvidenceKernelError {
    Kernel(KernelError),
    Store(ToolMutationEvidenceError),
    MissingEvidence(EffectId),
    BindingMismatch(EffectId),
    MissingVerifiedReceipt(EffectId),
    VerifiedStatusMismatch(EffectId),
}

impl fmt::Display for ToolMutationEvidenceKernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kernel(error) => write!(f, "tool mutation evidence kernel error: {error}"),
            Self::Store(error) => write!(f, "tool mutation evidence store error: {error}"),
            Self::MissingEvidence(effect_id) => write!(
                f,
                "tool mutation reconciliation evidence is missing for effect {}",
                effect_id.0
            ),
            Self::BindingMismatch(effect_id) => write!(
                f,
                "tool mutation reconciliation evidence binding mismatch for effect {}",
                effect_id.0
            ),
            Self::MissingVerifiedReceipt(effect_id) => write!(
                f,
                "tool mutation has no durable provider-verified receipt for effect {}",
                effect_id.0
            ),
            Self::VerifiedStatusMismatch(effect_id) => write!(
                f,
                "tool mutation verified receipt status mismatch for effect {}",
                effect_id.0
            ),
        }
    }
}

impl Error for ToolMutationEvidenceKernelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Kernel(error) => Some(error),
            Self::Store(error) => Some(error),
            Self::MissingEvidence(_)
            | Self::BindingMismatch(_)
            | Self::MissingVerifiedReceipt(_)
            | Self::VerifiedStatusMismatch(_) => None,
        }
    }
}

impl From<KernelError> for ToolMutationEvidenceKernelError {
    fn from(value: KernelError) -> Self {
        Self::Kernel(value)
    }
}

impl From<ToolMutationEvidenceError> for ToolMutationEvidenceKernelError {
    fn from(value: ToolMutationEvidenceError) -> Self {
        Self::Store(value)
    }
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn record_tool_mutation_intent(
        &mut self,
        principal: Principal<'_>,
        prepared: &PreparedToolEffect,
        provider_id: &str,
        intent_bytes: &[u8],
        scope: &str,
    ) -> Result<[u8; 32], ToolMutationEvidenceKernelError> {
        self.require_authority(&AuthorizationRequest {
            principal,
            action: prepared.action(),
            resource: prepared.resource(),
            context: AuthorizationContext::local(scope),
        })?;
        let mut store = ToolMutationEvidenceStore::open(self.tool_mutation_evidence_path())?;
        Ok(store.record_intent(RecordToolMutationIntent {
            effect_id: prepared.effect_id(),
            action: prepared.action(),
            resource: prepared.resource(),
            preconditions_hash: prepared.preconditions_hash(),
            payload_hash: prepared.payload_hash(),
            provider_id,
            intent_bytes,
        })?)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "verified mutation evidence keeps authority, provider identity, terminal classification and receipt explicit"
    )]
    pub fn record_tool_mutation_verified_receipt(
        &mut self,
        principal: Principal<'_>,
        prepared: &PreparedToolEffect,
        provider_id: &str,
        verified_status: ToolMutationVerifiedStatus,
        receipt_bytes: &[u8],
        scope: &str,
    ) -> Result<[u8; 32], ToolMutationEvidenceKernelError> {
        self.require_authority(&AuthorizationRequest {
            principal,
            action: prepared.action(),
            resource: prepared.resource(),
            context: AuthorizationContext::local(scope),
        })?;
        let mut store = ToolMutationEvidenceStore::open(self.tool_mutation_evidence_path())?;
        Ok(store.record_verified_receipt(RecordToolMutationReceipt {
            effect_id: prepared.effect_id(),
            action: prepared.action(),
            resource: prepared.resource(),
            preconditions_hash: prepared.preconditions_hash(),
            payload_hash: prepared.payload_hash(),
            provider_id,
            verified_status,
            receipt_bytes,
        })?)
    }

    pub(crate) fn require_verified_tool_mutation_receipt(
        &self,
        effect_id: EffectId,
        action: &str,
        resource: &str,
        preconditions_hash: [u8; 32],
        payload_hash: [u8; 32],
        required_status: ToolMutationVerifiedStatus,
    ) -> Result<[u8; 32], ToolMutationEvidenceKernelError> {
        let store = ToolMutationEvidenceStore::open(self.tool_mutation_evidence_path())?;
        let evidence = store
            .load(effect_id)?
            .ok_or(ToolMutationEvidenceKernelError::MissingEvidence(effect_id))?;
        if evidence.action != action
            || evidence.resource != resource
            || evidence.preconditions_hash != preconditions_hash
            || evidence.payload_hash != payload_hash
        {
            return Err(ToolMutationEvidenceKernelError::BindingMismatch(effect_id));
        }
        match evidence.verified_status {
            Some(status) if status == required_status => {}
            Some(_) => {
                return Err(ToolMutationEvidenceKernelError::VerifiedStatusMismatch(
                    effect_id,
                ));
            }
            None => {
                return Err(ToolMutationEvidenceKernelError::MissingVerifiedReceipt(
                    effect_id,
                ));
            }
        }
        evidence.receipt_integrity_hash.ok_or(
            ToolMutationEvidenceKernelError::MissingVerifiedReceipt(effect_id),
        )
    }

    fn tool_mutation_evidence_path(&self) -> PathBuf {
        self.runtime.data_dir.join(TOOL_CONTEXT_EVIDENCE_DB)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{KernelCreateSession, PolicyDecision, PrepareToolEffect};
    use golam_core::paths::RuntimeLayout;
    use golam_core::{EventId, SessionId};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    struct AllowTools;

    impl AuthorizationPolicy for AllowTools {
        fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
            PolicyDecision::allow("tool_mutation_evidence_qualification")
        }
    }

    fn runtime() -> RuntimeLayout {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-kernel-tool-mutation-evidence-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    fn prepared(kernel: &mut KernelApi<AllowTools>, effect_id: EffectId) -> PreparedToolEffect {
        kernel
            .prepare_tool_effect(
                Principal::test("evidence"),
                PrepareToolEffect {
                    effect_id,
                    session_id: SessionId(77),
                    action: "git.branch.create",
                    resource: "git-branch-create:candidate",
                    execution_semantics: "at_most_once",
                    handler_id: "golam-git-linux",
                    handler_version: "1",
                    idempotency_key: Some("evidence-qualification"),
                    preconditions_hash: [1; 32],
                    payload_hash: [2; 32],
                    started_at: "2026-09-05T09:30:01Z",
                },
                "evidence",
            )
            .unwrap()
    }

    fn kernel(runtime: &RuntimeLayout) -> KernelApi<AllowTools> {
        let mut kernel = KernelApi::open(runtime, AllowTools).unwrap();
        kernel
            .create_session(
                Principal::test("evidence"),
                KernelCreateSession {
                    session_id: SessionId(77),
                    event_id: EventId(1),
                    recorded_at: "2026-09-05T09:30:00Z",
                    payload: b"tool-mutation-evidence",
                },
                "evidence",
            )
            .unwrap();
        kernel
    }

    #[test]
    fn verified_receipt_is_protected_and_exact_effect_bound() {
        let runtime = runtime();
        let mut kernel = kernel(&runtime);
        let effect_id = EffectId(770);
        let prepared = prepared(&mut kernel, effect_id);
        kernel
            .record_tool_mutation_intent(
                Principal::test("evidence"),
                &prepared,
                "golam-git-linux-v1",
                b"branch:candidate",
                "evidence",
            )
            .unwrap();
        let receipt_hash = kernel
            .record_tool_mutation_verified_receipt(
                Principal::test("evidence"),
                &prepared,
                "golam-git-linux-v1",
                ToolMutationVerifiedStatus::Succeeded,
                b"verified-branch-ref",
                "evidence",
            )
            .unwrap();
        assert_eq!(
            kernel
                .require_verified_tool_mutation_receipt(
                    effect_id,
                    prepared.action(),
                    prepared.resource(),
                    prepared.preconditions_hash(),
                    prepared.payload_hash(),
                    ToolMutationVerifiedStatus::Succeeded,
                )
                .unwrap(),
            receipt_hash
        );
        assert!(matches!(
            kernel.require_verified_tool_mutation_receipt(
                effect_id,
                prepared.action(),
                prepared.resource(),
                prepared.preconditions_hash(),
                prepared.payload_hash(),
                ToolMutationVerifiedStatus::Failed,
            ),
            Err(ToolMutationEvidenceKernelError::VerifiedStatusMismatch(id)) if id == effect_id
        ));
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn missing_receipt_never_qualifies_terminal_success() {
        let runtime = runtime();
        let mut kernel = kernel(&runtime);
        let effect_id = EffectId(771);
        let prepared = prepared(&mut kernel, effect_id);
        kernel
            .record_tool_mutation_intent(
                Principal::test("evidence"),
                &prepared,
                "golam-git-linux-v1",
                b"branch:candidate",
                "evidence",
            )
            .unwrap();
        assert!(matches!(
            kernel.require_verified_tool_mutation_receipt(
                effect_id,
                prepared.action(),
                prepared.resource(),
                prepared.preconditions_hash(),
                prepared.payload_hash(),
                ToolMutationVerifiedStatus::Succeeded,
            ),
            Err(ToolMutationEvidenceKernelError::MissingVerifiedReceipt(id)) if id == effect_id
        ));
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
