#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use golam_core::digest::sha256;
use golam_core::memory::{
    MemoryMutationOutcome, MemoryMutationStatus, MemoryReconciliationState, MemoryVersion,
    MemoryWriterId, PreparedMemoryMutationIntent,
};
use golam_core::memory_storage::{MemoryLayout, MemoryLayoutError};
use golam_core::tool_request::BindingDigest;
use golam_core::{CanonicalEncoder, EffectAttemptId, EffectId, EffectTransitionId, EventId};
use golam_ledger::effects::{
    CompareAndSwapEffect, EffectStore, EffectStoreError, FinishEffectAttempt, StartEffectAttempt,
};
use golam_ledger::memory_evidence::{
    MemoryEvidenceError, MemoryEvidenceStore, PromotionEvidence, ReconciliationEvidence,
};
use golam_ledger::memory_operational::{MemoryOperationalError, MemoryOperationalStore};
use golam_ledger::memory_promotion_gate::QualifiedMemoryPromotion;
use golam_ledger::memory_promotion_operational::{
    MemoryPromotionOperationalError, MemoryPromotionOperationalStore,
};
use golam_ledger::memory_writer_authority::{
    MANAGED_MEMORY_HANDLER_ID, MANAGED_MEMORY_HANDLER_VERSION, MemoryWriterAuthorityError,
    MemoryWriterAuthorityStore,
};
use golam_ledger::memory_writer_readback::{
    MemoryWriterReadbackError, invalidate_memory_derivatives, verify_memory_sqlite_readback,
};

use golam_core::authority::AuthorityLayout;

const RECONCILIATION_DOMAIN: &[u8] = b"golam:managed-memory-reconciliation:v1";
const TERMINAL_DOMAIN: &[u8] = b"golam:managed-memory-terminal:v1";
const UNKNOWN_DOMAIN: &[u8] = b"golam:managed-memory-unknown:v1";
const WRITER_ID_DOMAIN: &[u8] = b"golam:managed-memory-writer-id:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManagedMemoryExecutionStart<'a> {
    pub executing_transition_id: EffectTransitionId,
    pub executing_event_id: EventId,
    pub attempt_id: EffectAttemptId,
    pub dispatch_token: &'a [u8],
    pub started_at: &'a str,
    pub promotion_recorded_at_unix_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManagedMemoryExecutionFinish<'a> {
    pub terminal_transition_id: EffectTransitionId,
    pub terminal_event_id: EventId,
    pub finished_at: &'a str,
    pub terminal_at_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedManagedMemoryWrite {
    prepared: PreparedMemoryMutationIntent,
    version: MemoryVersion,
    markdown_path: PathBuf,
    authority_readback_ref: BindingDigest,
    attempt_id: EffectAttemptId,
}

impl PreparedManagedMemoryWrite {
    pub fn prepared_intent(&self) -> &PreparedMemoryMutationIntent {
        &self.prepared
    }

    pub fn version(&self) -> &MemoryVersion {
        &self.version
    }

    pub fn markdown_path(&self) -> &Path {
        &self.markdown_path
    }

    pub const fn authority_readback_ref(&self) -> BindingDigest {
        self.authority_readback_ref
    }

    pub const fn attempt_id(&self) -> EffectAttemptId {
        self.attempt_id
    }

    pub fn effect_id(&self) -> EffectId {
        self.prepared.intent().effect_id
    }

    pub fn expected_target_identity_ref(&self) -> BindingDigest {
        self.prepared.intent().expected_markdown_target_identity_ref
    }

    pub fn expected_content_digest(&self) -> BindingDigest {
        self.prepared.intent().expected_markdown_content_digest
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManagedMarkdownCommitObservation {
    pub readback_ref: BindingDigest,
    pub target_identity_ref: BindingDigest,
    pub content_digest: BindingDigest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedManagedMemoryWrite {
    pub terminal_evidence_id: BindingDigest,
    pub reconciliation_ref: BindingDigest,
    pub authority_journal_readback_ref: BindingDigest,
    pub markdown_readback_ref: BindingDigest,
    pub memory_sqlite_readback_ref: BindingDigest,
}

#[derive(Debug)]
pub enum ManagedMemoryWriterError {
    Layout(MemoryLayoutError),
    Authority(MemoryWriterAuthorityError),
    Operational(MemoryOperationalError),
    PromotionOperational(MemoryPromotionOperationalError),
    Evidence(MemoryEvidenceError),
    Effect(EffectStoreError),
    Readback(MemoryWriterReadbackError),
    BindingMismatch(&'static str),
    BlockingUnknownOutcome,
    MarkdownPathOutsideVault,
    UnknownOutcomeAfterCommit(String),
    CanonicalEncoding,
}

impl fmt::Display for ManagedMemoryWriterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Layout(error) => write!(f, "managed-memory layout failed: {error}"),
            Self::Authority(error) => {
                write!(f, "managed-memory PREPARED authority failed: {error}")
            }
            Self::Operational(error) => {
                write!(f, "managed-memory operational state failed: {error}")
            }
            Self::PromotionOperational(error) => {
                write!(
                    f,
                    "managed-memory promotion operational state failed: {error}"
                )
            }
            Self::Evidence(error) => write!(f, "managed-memory authority evidence failed: {error}"),
            Self::Effect(error) => {
                write!(f, "managed-memory Effect Gate transition failed: {error}")
            }
            Self::Readback(error) => {
                write!(f, "managed-memory cross-store readback failed: {error}")
            }
            Self::BindingMismatch(field) => {
                write!(f, "managed-memory writer binding mismatch: {field}")
            }
            Self::BlockingUnknownOutcome => {
                f.write_str("managed-memory mutation is blocked by an unresolved UNKNOWN_OUTCOME")
            }
            Self::MarkdownPathOutsideVault => {
                f.write_str("managed-memory Markdown target is outside the canonical memory vault")
            }
            Self::UnknownOutcomeAfterCommit(reason) => write!(
                f,
                "managed-memory completion is ambiguous after the Markdown commit boundary: {reason}"
            ),
            Self::CanonicalEncoding => {
                f.write_str("managed-memory terminal evidence canonical encoding failed")
            }
        }
    }
}

impl Error for ManagedMemoryWriterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Layout(error) => Some(error),
            Self::Authority(error) => Some(error),
            Self::Operational(error) => Some(error),
            Self::PromotionOperational(error) => Some(error),
            Self::Evidence(error) => Some(error),
            Self::Effect(error) => Some(error),
            Self::Readback(error) => Some(error),
            _ => None,
        }
    }
}

impl From<MemoryLayoutError> for ManagedMemoryWriterError {
    fn from(value: MemoryLayoutError) -> Self {
        Self::Layout(value)
    }
}

impl From<MemoryWriterAuthorityError> for ManagedMemoryWriterError {
    fn from(value: MemoryWriterAuthorityError) -> Self {
        Self::Authority(value)
    }
}

impl From<MemoryOperationalError> for ManagedMemoryWriterError {
    fn from(value: MemoryOperationalError) -> Self {
        Self::Operational(value)
    }
}

impl From<MemoryPromotionOperationalError> for ManagedMemoryWriterError {
    fn from(value: MemoryPromotionOperationalError) -> Self {
        Self::PromotionOperational(value)
    }
}

impl From<MemoryEvidenceError> for ManagedMemoryWriterError {
    fn from(value: MemoryEvidenceError) -> Self {
        Self::Evidence(value)
    }
}

impl From<EffectStoreError> for ManagedMemoryWriterError {
    fn from(value: EffectStoreError) -> Self {
        Self::Effect(value)
    }
}

impl From<MemoryWriterReadbackError> for ManagedMemoryWriterError {
    fn from(value: MemoryWriterReadbackError) -> Self {
        Self::Readback(value)
    }
}

pub struct ManagedMemoryWriter {
    authority: AuthorityLayout,
    memory: MemoryLayout,
}

impl ManagedMemoryWriter {
    pub fn new(authority: AuthorityLayout, memory: MemoryLayout) -> Self {
        Self { authority, memory }
    }

    pub fn writer_id() -> MemoryWriterId {
        let mut encoder = CanonicalEncoder::new();
        encoder
            .push_bytes(WRITER_ID_DOMAIN)
            .expect("constant writer domain is bounded");
        encoder
            .push_bytes(MANAGED_MEMORY_HANDLER_ID.as_bytes())
            .expect("constant handler id is bounded");
        encoder
            .push_bytes(MANAGED_MEMORY_HANDLER_VERSION.as_bytes())
            .expect("constant handler version is bounded");
        MemoryWriterId(BindingDigest::new(sha256(&encoder.finish())))
    }

    pub fn prepare_existing_write(
        &self,
        promotion: &QualifiedMemoryPromotion,
        prepared: &PreparedMemoryMutationIntent,
        version: &MemoryVersion,
        markdown_path: &Path,
        execution: ManagedMemoryExecutionStart<'_>,
    ) -> Result<PreparedManagedMemoryWrite, ManagedMemoryWriterError> {
        validate_bindings(&self.memory, promotion, prepared, version, markdown_path)?;

        let mut evidence = MemoryEvidenceStore::open(self.authority.authority_db_path())?;
        evidence.persist_promotion(PromotionEvidence {
            evidence_id: promotion.evidence_id(),
            candidate_id: promotion.candidate_id(),
            promotion_authority_ref: promotion.promotion_authority_ref(),
            approving_principal: promotion.approving_principal(),
            verifier_policy_ref: promotion.verifier_policy_ref(),
            record_bytes: promotion.record_bytes(),
        })?;

        let mut authority = MemoryWriterAuthorityStore::open(&self.authority)?;
        let prepared_authority = authority.prepare(prepared)?;
        let authority_readback_ref = authority.readback_ref(prepared_authority)?;

        let mut operational = MemoryOperationalStore::open(&self.memory)?;
        if operational.has_blocking_unknown_outcome()? {
            return Err(ManagedMemoryWriterError::BlockingUnknownOutcome);
        }
        operational.record_prepared(prepared)?;

        let mut promotion_operational = MemoryPromotionOperationalStore::open(&self.memory)?;
        promotion_operational
            .record(promotion.operational_evidence(execution.promotion_recorded_at_unix_ms))?;
        promotion_operational.require_exact(promotion.evidence_id(), promotion.candidate_id())?;

        let intent = prepared.intent();
        let mut effects = EffectStore::open(&self.authority)?;
        effects.compare_and_swap(CompareAndSwapEffect {
            transition_id: execution.executing_transition_id,
            effect_id: intent.effect_id,
            expected_state: "authorized",
            next_state: "executing",
            attempt_id: Some(execution.attempt_id),
            reason_code: Some("managed_memory_prepared"),
            evidence_ref: Some(&authority_readback_ref.bytes()),
            event_id: execution.executing_event_id,
        })?;
        effects.start_attempt(StartEffectAttempt {
            attempt_id: execution.attempt_id,
            effect_id: intent.effect_id,
            handler_id: MANAGED_MEMORY_HANDLER_ID,
            handler_version: MANAGED_MEMORY_HANDLER_VERSION,
            dispatch_token: execution.dispatch_token,
            started_at: execution.started_at,
        })?;

        Ok(PreparedManagedMemoryWrite {
            prepared: prepared.clone(),
            version: version.clone(),
            markdown_path: markdown_path.to_path_buf(),
            authority_readback_ref,
            attempt_id: execution.attempt_id,
        })
    }

    pub fn finalize_existing_write(
        &self,
        write: &PreparedManagedMemoryWrite,
        markdown: ManagedMarkdownCommitObservation,
        finish: ManagedMemoryExecutionFinish<'_>,
    ) -> Result<CommittedManagedMemoryWrite, ManagedMemoryWriterError> {
        match self.finalize_existing_write_inner(write, markdown, finish) {
            Ok(result) => Ok(result),
            Err(error) => {
                let reason = error.to_string();
                self.record_unknown_outcome(write, Some(markdown.readback_ref), finish, &reason);
                Err(ManagedMemoryWriterError::UnknownOutcomeAfterCommit(reason))
            }
        }
    }

    fn finalize_existing_write_inner(
        &self,
        write: &PreparedManagedMemoryWrite,
        markdown: ManagedMarkdownCommitObservation,
        finish: ManagedMemoryExecutionFinish<'_>,
    ) -> Result<CommittedManagedMemoryWrite, ManagedMemoryWriterError> {
        if markdown.content_digest != write.version.content_digest {
            return Err(ManagedMemoryWriterError::BindingMismatch(
                "committed Markdown content digest",
            ));
        }

        let mut operational = MemoryOperationalStore::open(&self.memory)?;
        operational.record_version(&write.prepared, &write.version, &write.markdown_path)?;
        invalidate_memory_derivatives(&self.memory)?;
        let sqlite_readback_ref = verify_memory_sqlite_readback(
            &self.memory,
            &write.prepared,
            &write.version,
            &write.markdown_path,
        )?;

        let reconciliation_bytes = reconciliation_bytes(
            write.effect_id(),
            write.authority_readback_ref,
            markdown.readback_ref,
            sqlite_readback_ref,
            write.version.version_id.0,
        )?;
        let reconciliation_ref = BindingDigest::new(sha256(&reconciliation_bytes));
        operational.record_reconciliation(
            write.effect_id(),
            BindingDigest::new(write.prepared.binding_digest()),
            MemoryReconciliationState::InSync,
            reconciliation_ref,
        )?;

        let mut evidence = MemoryEvidenceStore::open(self.authority.authority_db_path())?;
        evidence.persist_version(&write.version, &version_record_bytes(&write.version)?)?;
        evidence.persist_reconciliation(ReconciliationEvidence {
            evidence_id: reconciliation_ref,
            effect_id: write.effect_id(),
            state: MemoryReconciliationState::InSync,
            authority_journal_readback_ref: Some(write.authority_readback_ref),
            markdown_readback_ref: Some(markdown.readback_ref),
            memory_sqlite_readback_ref: Some(sqlite_readback_ref),
            record_bytes: &reconciliation_bytes,
        })?;

        let mut verification_refs = vec![markdown.readback_ref, sqlite_readback_ref];
        verification_refs.sort_unstable();
        verification_refs.dedup();
        let mut integrity_refs = vec![
            write.authority_readback_ref,
            write.version.promotion_evidence_ref,
            reconciliation_ref,
        ];
        integrity_refs.sort_unstable();
        integrity_refs.dedup();
        let outcome = MemoryMutationOutcome {
            effect_id: write.effect_id(),
            mutation_intent_digest: BindingDigest::new(write.prepared.binding_digest()),
            status: MemoryMutationStatus::Committed,
            canonical_version_refs: vec![write.version.version_id],
            authority_journal_readback_ref: Some(write.authority_readback_ref),
            markdown_readback_ref: Some(markdown.readback_ref),
            memory_sqlite_readback_ref: Some(sqlite_readback_ref),
            reconciliation_ref: Some(reconciliation_ref),
            verification_refs,
            integrity_evidence_refs: integrity_refs,
            terminal_at_unix_ms: finish.terminal_at_unix_ms,
        };
        let terminal_bytes = terminal_bytes(&outcome)?;
        let terminal_evidence_id = BindingDigest::new(sha256(&terminal_bytes));
        evidence.persist_terminal_outcome(terminal_evidence_id, &outcome, &terminal_bytes)?;

        operational.mark_terminal(
            write.effect_id(),
            BindingDigest::new(write.prepared.binding_digest()),
            MemoryMutationStatus::Committed,
        )?;

        let mut effects = EffectStore::open(&self.authority)?;
        effects.finish_attempt(FinishEffectAttempt {
            attempt_id: write.attempt_id,
            finished_at: finish.finished_at,
            outcome: "success",
            receipt: Some(&terminal_bytes),
        })?;
        effects.compare_and_swap(CompareAndSwapEffect {
            transition_id: finish.terminal_transition_id,
            effect_id: write.effect_id(),
            expected_state: "executing",
            next_state: "succeeded",
            attempt_id: Some(write.attempt_id),
            reason_code: Some("managed_memory_verified"),
            evidence_ref: Some(&terminal_evidence_id.bytes()),
            event_id: finish.terminal_event_id,
        })?;

        Ok(CommittedManagedMemoryWrite {
            terminal_evidence_id,
            reconciliation_ref,
            authority_journal_readback_ref: write.authority_readback_ref,
            markdown_readback_ref: markdown.readback_ref,
            memory_sqlite_readback_ref: sqlite_readback_ref,
        })
    }

    fn record_unknown_outcome(
        &self,
        write: &PreparedManagedMemoryWrite,
        markdown_readback_ref: Option<BindingDigest>,
        finish: ManagedMemoryExecutionFinish<'_>,
        reason: &str,
    ) {
        let reconciliation_bytes = unknown_bytes(write.effect_id(), reason);
        let reconciliation_ref = BindingDigest::new(sha256(&reconciliation_bytes));
        let intent_digest = BindingDigest::new(write.prepared.binding_digest());

        if let Ok(mut operational) = MemoryOperationalStore::open(&self.memory) {
            let _ = operational.record_reconciliation(
                write.effect_id(),
                intent_digest,
                MemoryReconciliationState::Blocked,
                reconciliation_ref,
            );
            let _ = operational.mark_terminal(
                write.effect_id(),
                intent_digest,
                MemoryMutationStatus::UnknownOutcome,
            );
        }

        if let Ok(mut evidence) = MemoryEvidenceStore::open(self.authority.authority_db_path()) {
            let _ = evidence.persist_reconciliation(ReconciliationEvidence {
                evidence_id: reconciliation_ref,
                effect_id: write.effect_id(),
                state: MemoryReconciliationState::Blocked,
                authority_journal_readback_ref: Some(write.authority_readback_ref),
                markdown_readback_ref,
                memory_sqlite_readback_ref: None,
                record_bytes: &reconciliation_bytes,
            });
            let outcome = MemoryMutationOutcome {
                effect_id: write.effect_id(),
                mutation_intent_digest: intent_digest,
                status: MemoryMutationStatus::UnknownOutcome,
                canonical_version_refs: Vec::new(),
                authority_journal_readback_ref: Some(write.authority_readback_ref),
                markdown_readback_ref,
                memory_sqlite_readback_ref: None,
                reconciliation_ref: Some(reconciliation_ref),
                verification_refs: Vec::new(),
                integrity_evidence_refs: vec![reconciliation_ref],
                terminal_at_unix_ms: finish.terminal_at_unix_ms,
            };
            if let Ok(bytes) = terminal_bytes(&outcome) {
                let id = BindingDigest::new(sha256(&bytes));
                let _ = evidence.persist_terminal_outcome(id, &outcome, &bytes);
            }
        }

        if let Ok(mut effects) = EffectStore::open(&self.authority) {
            let _ = effects.finish_attempt(FinishEffectAttempt {
                attempt_id: write.attempt_id,
                finished_at: finish.finished_at,
                outcome: "unknown",
                receipt: Some(&reconciliation_bytes),
            });
            let _ = effects.compare_and_swap(CompareAndSwapEffect {
                transition_id: finish.terminal_transition_id,
                effect_id: write.effect_id(),
                expected_state: "executing",
                next_state: "unknown_outcome",
                attempt_id: Some(write.attempt_id),
                reason_code: Some("managed_memory_unknown_outcome"),
                evidence_ref: Some(&reconciliation_ref.bytes()),
                event_id: finish.terminal_event_id,
            });
        }
    }
}

fn validate_bindings(
    memory: &MemoryLayout,
    promotion: &QualifiedMemoryPromotion,
    prepared: &PreparedMemoryMutationIntent,
    version: &MemoryVersion,
    markdown_path: &Path,
) -> Result<(), ManagedMemoryWriterError> {
    version
        .validate()
        .map_err(|_| ManagedMemoryWriterError::BindingMismatch("memory version contract"))?;
    let intent = prepared.intent();
    if intent.memory_operational_store_ref != memory.store_id() {
        return Err(ManagedMemoryWriterError::BindingMismatch(
            "memory operational store",
        ));
    }
    if intent.candidate_ref != Some(promotion.candidate_id()) {
        return Err(ManagedMemoryWriterError::BindingMismatch(
            "promotion candidate",
        ));
    }
    if intent.kernel_authorization_ref != promotion.kernel_authorization_ref() {
        return Err(ManagedMemoryWriterError::BindingMismatch(
            "Kernel authorization",
        ));
    }
    if intent.promotion_authority_ref != promotion.promotion_authority_ref() {
        return Err(ManagedMemoryWriterError::BindingMismatch(
            "promotion authority",
        ));
    }
    if version.promotion_evidence_ref != promotion.evidence_id() {
        return Err(ManagedMemoryWriterError::BindingMismatch(
            "promotion evidence",
        ));
    }
    if version.mutation_effect_ref != intent.effect_id {
        return Err(ManagedMemoryWriterError::BindingMismatch(
            "mutation effect identity",
        ));
    }
    if version.committed_by_writer_identity != ManagedMemoryWriter::writer_id() {
        return Err(ManagedMemoryWriterError::BindingMismatch(
            "managed writer identity",
        ));
    }
    if !intent.item_ids.contains(&version.item_id) {
        return Err(ManagedMemoryWriterError::BindingMismatch(
            "memory version item",
        ));
    }
    if !markdown_path.starts_with(memory.vault_dir()) || memory.is_operational_path(markdown_path) {
        return Err(ManagedMemoryWriterError::MarkdownPathOutsideVault);
    }
    Ok(())
}

fn reconciliation_bytes(
    effect_id: EffectId,
    authority: BindingDigest,
    markdown: BindingDigest,
    sqlite: BindingDigest,
    version: BindingDigest,
) -> Result<Vec<u8>, ManagedMemoryWriterError> {
    let mut encoder = CanonicalEncoder::new();
    encoder
        .push_bytes(RECONCILIATION_DOMAIN)
        .map_err(|_| ManagedMemoryWriterError::CanonicalEncoding)?;
    encoder.push_u128(effect_id.0);
    for digest in [authority, markdown, sqlite, version] {
        encoder
            .push_bytes(&digest.bytes())
            .map_err(|_| ManagedMemoryWriterError::CanonicalEncoding)?;
    }
    Ok(encoder.finish())
}

fn version_record_bytes(version: &MemoryVersion) -> Result<Vec<u8>, ManagedMemoryWriterError> {
    let mut encoder = CanonicalEncoder::new();
    encoder
        .push_bytes(b"golam:managed-memory-version-record:v1")
        .map_err(|_| ManagedMemoryWriterError::CanonicalEncoding)?;
    encoder
        .push_bytes(&version.version_id.0.bytes())
        .map_err(|_| ManagedMemoryWriterError::CanonicalEncoding)?;
    encoder
        .push_bytes(&version.item_id.0.bytes())
        .map_err(|_| ManagedMemoryWriterError::CanonicalEncoding)?;
    encoder
        .push_bytes(&version.content_digest.bytes())
        .map_err(|_| ManagedMemoryWriterError::CanonicalEncoding)?;
    encoder.push_u128(version.mutation_effect_ref.0);
    Ok(encoder.finish())
}

fn terminal_bytes(outcome: &MemoryMutationOutcome) -> Result<Vec<u8>, ManagedMemoryWriterError> {
    outcome
        .validate()
        .map_err(|_| ManagedMemoryWriterError::CanonicalEncoding)?;
    let mut encoder = CanonicalEncoder::new();
    encoder
        .push_bytes(TERMINAL_DOMAIN)
        .map_err(|_| ManagedMemoryWriterError::CanonicalEncoding)?;
    encoder.push_u128(outcome.effect_id.0);
    encoder
        .push_bytes(&outcome.mutation_intent_digest.bytes())
        .map_err(|_| ManagedMemoryWriterError::CanonicalEncoding)?;
    encoder.push_u8(match outcome.status {
        MemoryMutationStatus::Committed => 1,
        MemoryMutationStatus::Rejected => 2,
        MemoryMutationStatus::Failed => 3,
        MemoryMutationStatus::UnknownOutcome => 4,
    });
    encoder.push_u64(outcome.terminal_at_unix_ms);
    for value in [
        outcome.authority_journal_readback_ref,
        outcome.markdown_readback_ref,
        outcome.memory_sqlite_readback_ref,
        outcome.reconciliation_ref,
    ] {
        match value {
            Some(value) => {
                encoder.push_u8(1);
                encoder
                    .push_bytes(&value.bytes())
                    .map_err(|_| ManagedMemoryWriterError::CanonicalEncoding)?;
            }
            None => encoder.push_u8(0),
        }
    }
    Ok(encoder.finish())
}

fn unknown_bytes(effect_id: EffectId, reason: &str) -> Vec<u8> {
    let mut encoder = CanonicalEncoder::new();
    let _ = encoder.push_bytes(UNKNOWN_DOMAIN);
    encoder.push_u128(effect_id.0);
    let _ = encoder.push_bytes(reason.as_bytes());
    encoder.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_writer_identity_is_stable_and_distinct() {
        assert_eq!(
            ManagedMemoryWriter::writer_id(),
            ManagedMemoryWriter::writer_id()
        );
        assert_ne!(
            ManagedMemoryWriter::writer_id().0,
            BindingDigest::new([0; 32])
        );
    }
}
