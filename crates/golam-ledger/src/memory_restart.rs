#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use golam_core::authority::AuthorityLayout;
use golam_core::digest::sha256;
use golam_core::memory::{
    MemoryMutationOutcome, MemoryMutationStatus, MemoryOperation, MemoryReconciliationState,
    MemoryStoreId, MemoryVersionId,
};
use golam_core::memory_storage::MemoryLayout;
use golam_core::tool_request::BindingDigest;
use golam_core::{
    CanonicalEncoder, CoreError, EffectAttemptId, EffectId, EffectTransitionId, EventId,
};
use rusqlite::{Connection, OptionalExtension, params};

use crate::effects::{
    CompareAndSwapEffect, EffectStore, EffectStoreError, FinishEffectAttempt, StartEffectAttempt,
};
use crate::memory_evidence::{
    MemoryEvidenceError, MemoryEvidenceStore, ReconciliationEvidence,
};
use crate::memory_operational::{MemoryOperationalError, MemoryOperationalStore};
use crate::memory_writer_readback::{MemoryWriterReadbackError, invalidate_memory_derivatives};

const INTENT_DOMAIN: &[u8] = b"golam:memory-mutation-intent:v1";
const PREPARED_TARGET_DOMAIN: &[u8] = b"golam:managed-memory-prepared-target:v1";
const PREPARED_READBACK_DOMAIN: &[u8] = b"golam:managed-memory-prepared-readback:v1";
const SQLITE_READBACK_DOMAIN: &[u8] = b"golam:memory-sqlite-readback:v1";
const VERSION_EVIDENCE_DOMAIN: &[u8] = b"golam:memory-version-evidence:v1";
const VERSION_RECORD_DOMAIN: &[u8] = b"golam:managed-memory-version-record:v1";
const RECONCILIATION_EVIDENCE_DOMAIN: &[u8] = b"golam:memory-reconciliation-evidence:v1";
const RESTART_RECONCILIATION_DOMAIN: &[u8] = b"golam:managed-memory-restart-reconciliation:v1";
const TERMINAL_DOMAIN: &[u8] = b"golam:managed-memory-terminal:v1";
const RESTART_ID_DOMAIN: &[u8] = b"golam:managed-memory-restart-id:v1";
const RESTART_HANDLER_ID: &str = "golam-managed-memory-restart-reconciler";
const RESTART_HANDLER_VERSION: &str = "1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryRestartCase {
    pub effect_id: EffectId,
    pub intent_digest: BindingDigest,
    pub memory_store_ref: MemoryStoreId,
    pub operation: MemoryOperation,
    pub item_id: BindingDigest,
    pub version_id: MemoryVersionId,
    pub markdown_path: PathBuf,
    pub expected_target_identity_ref: BindingDigest,
    pub expected_content_digest: BindingDigest,
    pub expected_markdown_version: MemoryVersionId,
    pub effect_state: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemoryRestartObservation {
    Regular {
        target_identity_ref: BindingDigest,
        content_digest: BindingDigest,
        markdown_readback_ref: BindingDigest,
    },
    Missing,
    Unobservable {
        reason_code: String,
    },
}

impl MemoryRestartObservation {
    fn markdown_readback_ref(&self) -> Option<BindingDigest> {
        match self {
            Self::Regular {
                markdown_readback_ref,
                ..
            } => Some(*markdown_readback_ref),
            Self::Missing | Self::Unobservable { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryRestartResolution {
    ReconciledCommitted,
    ReconciledNoMutation,
    BlockedUnknownOutcome,
}

#[derive(Debug)]
pub enum MemoryRestartError {
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Evidence(MemoryEvidenceError),
    Operational(MemoryOperationalError),
    Effect(EffectStoreError),
    Readback(MemoryWriterReadbackError),
    InvalidRecord(&'static str),
    StoreBindingMismatch,
    StaleCase,
    ConflictingTerminalEvidence,
    UnsupportedEffectState(String),
}

impl fmt::Display for MemoryRestartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "managed-memory restart sqlite failed: {error}"),
            Self::Core(error) => write!(f, "managed-memory restart canonical encoding failed: {error}"),
            Self::Evidence(error) => write!(f, "managed-memory restart evidence failed: {error}"),
            Self::Operational(error) => {
                write!(f, "managed-memory restart operational state failed: {error}")
            }
            Self::Effect(error) => write!(f, "managed-memory restart effect transition failed: {error}"),
            Self::Readback(error) => write!(f, "managed-memory restart readback failed: {error}"),
            Self::InvalidRecord(field) => {
                write!(f, "managed-memory restart encountered invalid durable state: {field}")
            }
            Self::StoreBindingMismatch => f.write_str(
                "managed-memory restart PREPARED store identity does not match the current operational store",
            ),
            Self::StaleCase => f.write_str("managed-memory restart case changed before reconciliation"),
            Self::ConflictingTerminalEvidence => f.write_str(
                "managed-memory restart found terminal evidence that conflicts with current cross-store proof",
            ),
            Self::UnsupportedEffectState(state) => write!(
                f,
                "managed-memory restart cannot deterministically reconcile effect state: {state}"
            ),
        }
    }
}

impl Error for MemoryRestartError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::Evidence(error) => Some(error),
            Self::Operational(error) => Some(error),
            Self::Effect(error) => Some(error),
            Self::Readback(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for MemoryRestartError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for MemoryRestartError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<MemoryEvidenceError> for MemoryRestartError {
    fn from(value: MemoryEvidenceError) -> Self {
        Self::Evidence(value)
    }
}

impl From<MemoryOperationalError> for MemoryRestartError {
    fn from(value: MemoryOperationalError) -> Self {
        Self::Operational(value)
    }
}

impl From<EffectStoreError> for MemoryRestartError {
    fn from(value: EffectStoreError) -> Self {
        Self::Effect(value)
    }
}

impl From<MemoryWriterReadbackError> for MemoryRestartError {
    fn from(value: MemoryWriterReadbackError) -> Self {
        Self::Readback(value)
    }
}

pub struct MemoryRestartStore {
    authority: AuthorityLayout,
    memory: MemoryLayout,
}

impl MemoryRestartStore {
    pub fn open(
        authority: &AuthorityLayout,
        memory: &MemoryLayout,
    ) -> Result<Self, MemoryRestartError> {
        // Opening the canonical stores verifies/initializes the exact schemas before
        // restart code performs read-only classification or bounded reconciliation writes.
        drop(MemoryEvidenceStore::open(authority.authority_db_path())?);
        drop(MemoryOperationalStore::open(memory)?);
        Ok(Self {
            authority: authority.clone(),
            memory: memory.clone(),
        })
    }

    pub fn pending_cases(&self) -> Result<Vec<MemoryRestartCase>, MemoryRestartError> {
        let connection = self.authority_connection()?;
        let mut statement = connection.prepare(
            r#"SELECT t.effect_id, t.intent_digest, t.memory_store_ref, t.item_id, t.scope,
                      t.version_id, t.markdown_path, t.target_identity_ref,
                      t.expected_content_digest, t.expected_markdown_version_ref,
                      t.record_bytes, t.integrity_hash,
                      i.canonical_bytes, i.integrity_hash,
                      (SELECT e.to_state FROM effect_transitions e
                       WHERE e.effect_id = t.effect_id
                       ORDER BY e.global_seq DESC LIMIT 1)
               FROM memory_prepared_targets t
               JOIN memory_prepared_intents i ON i.effect_id = t.effect_id
               ORDER BY t.effect_id"#,
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, Vec<u8>>(0)?,
                row.get::<_, Vec<u8>>(1)?,
                row.get::<_, Vec<u8>>(2)?,
                row.get::<_, Vec<u8>>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, Vec<u8>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Vec<u8>>(7)?,
                row.get::<_, Vec<u8>>(8)?,
                row.get::<_, Vec<u8>>(9)?,
                row.get::<_, Vec<u8>>(10)?,
                row.get::<_, Vec<u8>>(11)?,
                row.get::<_, Vec<u8>>(12)?,
                row.get::<_, Vec<u8>>(13)?,
                row.get::<_, Option<String>>(14)?,
            ))
        })?;

        let mut cases = Vec::new();
        for row in rows {
            let row = row?;
            let effect_id = EffectId(u128::from_be_bytes(array16(row.0, "effect id")?));
            let intent_digest = digest32(row.1, "intent digest")?;
            let memory_store_ref = MemoryStoreId(digest32(row.2, "memory store ref")?);
            let item_id = digest32(row.3, "item id")?;
            let version_id = MemoryVersionId(digest32(row.5, "version id")?);
            let expected_target_identity_ref = digest32(row.7, "target identity")?;
            let expected_content_digest = digest32(row.8, "expected content digest")?;
            let expected_markdown_version =
                MemoryVersionId(digest32(row.9, "expected Markdown version")?);
            let effect_state = row
                .14
                .ok_or(MemoryRestartError::InvalidRecord("effect current state"))?;

            if row.13.as_slice() != crate::payload_hash(&row.12) {
                return Err(MemoryRestartError::InvalidRecord(
                    "prepared intent integrity hash",
                ));
            }
            if intent_digest.bytes() != sha256(&row.12) {
                return Err(MemoryRestartError::InvalidRecord(
                    "prepared intent binding digest",
                ));
            }
            let operation = decode_operation(&row.12)?;
            let target_bytes = prepared_target_record_bytes(
                effect_id,
                intent_digest,
                memory_store_ref,
                item_id,
                row.4,
                version_id,
                &row.6,
                expected_target_identity_ref,
                expected_content_digest,
                expected_markdown_version,
            )?;
            if row.10 != target_bytes || row.11.as_slice() != crate::payload_hash(&target_bytes) {
                return Err(MemoryRestartError::InvalidRecord(
                    "prepared target integrity binding",
                ));
            }

            if !matches!(effect_state.as_str(), "succeeded" | "failed" | "denied") {
                cases.push(MemoryRestartCase {
                    effect_id,
                    intent_digest,
                    memory_store_ref,
                    operation,
                    item_id,
                    version_id,
                    markdown_path: PathBuf::from(row.6),
                    expected_target_identity_ref,
                    expected_content_digest,
                    expected_markdown_version,
                    effect_state,
                });
            }
        }
        Ok(cases)
    }

    pub fn reconcile(
        &self,
        case: &MemoryRestartCase,
        observation: &MemoryRestartObservation,
        finished_at: &str,
        terminal_at_unix_ms: u64,
    ) -> Result<MemoryRestartResolution, MemoryRestartError> {
        if finished_at.is_empty() {
            return Err(MemoryRestartError::InvalidRecord("restart completion time"));
        }
        let live = self
            .pending_cases()?
            .into_iter()
            .find(|candidate| candidate.effect_id == case.effect_id)
            .ok_or(MemoryRestartError::StaleCase)?;
        if &live != case {
            return Err(MemoryRestartError::StaleCase);
        }
        if case.memory_store_ref != self.memory.store_id() {
            // A stale/wrong store binding cannot be projected into the current operational
            // store without changing the PREPARED authority. Fail the whole startup closed.
            return Err(MemoryRestartError::StoreBindingMismatch);
        }

        self.ensure_operational_prepared(case)?;
        let authority_readback_ref = self.authority_readback_ref(case.effect_id, case.intent_digest)?;

        if let MemoryRestartObservation::Regular {
            target_identity_ref,
            content_digest,
            markdown_readback_ref,
        } = observation
            && *target_identity_ref == case.expected_target_identity_ref
            && *content_digest == case.expected_content_digest
        {
            return self.resolve_no_mutation(
                case,
                authority_readback_ref,
                *markdown_readback_ref,
                finished_at,
                terminal_at_unix_ms,
            );
        }

        if let Some(committed) = self.prove_committed(case, observation, authority_readback_ref)? {
            return self.resolve_committed(
                case,
                committed,
                finished_at,
                terminal_at_unix_ms,
            );
        }

        self.resolve_unknown(
            case,
            authority_readback_ref,
            observation.markdown_readback_ref(),
            finished_at,
            terminal_at_unix_ms,
        )
    }

    fn resolve_no_mutation(
        &self,
        case: &MemoryRestartCase,
        authority_readback_ref: BindingDigest,
        markdown_readback_ref: BindingDigest,
        finished_at: &str,
        terminal_at_unix_ms: u64,
    ) -> Result<MemoryRestartResolution, MemoryRestartError> {
        self.require_terminal_compatible(case.effect_id, MemoryMutationStatus::Failed)?;
        let bytes = restart_reconciliation_bytes(
            case,
            MemoryReconciliationState::Reconciled,
            Some(authority_readback_ref),
            Some(markdown_readback_ref),
            None,
            b"no_mutation",
        )?;
        let reconciliation_ref = BindingDigest::new(sha256(&bytes));
        self.persist_reconciliation(
            case,
            MemoryReconciliationState::Reconciled,
            reconciliation_ref,
            Some(authority_readback_ref),
            Some(markdown_readback_ref),
            None,
            &bytes,
        )?;

        let outcome = MemoryMutationOutcome {
            effect_id: case.effect_id,
            mutation_intent_digest: case.intent_digest,
            status: MemoryMutationStatus::Failed,
            canonical_version_refs: Vec::new(),
            authority_journal_readback_ref: Some(authority_readback_ref),
            markdown_readback_ref: Some(markdown_readback_ref),
            memory_sqlite_readback_ref: None,
            reconciliation_ref: Some(reconciliation_ref),
            verification_refs: vec![markdown_readback_ref],
            integrity_evidence_refs: vec![authority_readback_ref, reconciliation_ref],
            terminal_at_unix_ms,
        };
        self.persist_terminal_if_absent(&outcome)?;
        self.mark_operational_terminal(case, MemoryMutationStatus::Failed)?;
        self.resolve_effect_state(
            case.effect_id,
            DesiredEffectOutcome::Failed,
            reconciliation_ref,
            finished_at,
        )?;
        Ok(MemoryRestartResolution::ReconciledNoMutation)
    }

    fn resolve_committed(
        &self,
        case: &MemoryRestartCase,
        committed: CommittedProof,
        finished_at: &str,
        terminal_at_unix_ms: u64,
    ) -> Result<MemoryRestartResolution, MemoryRestartError> {
        if case.effect_state == "authorized" {
            return self.resolve_unknown(
                case,
                committed.authority_readback_ref,
                Some(committed.markdown_readback_ref),
                finished_at,
                terminal_at_unix_ms,
            );
        }
        self.require_terminal_compatible(case.effect_id, MemoryMutationStatus::Committed)?;
        invalidate_memory_derivatives(&self.memory)?;
        let bytes = restart_reconciliation_bytes(
            case,
            MemoryReconciliationState::Reconciled,
            Some(committed.authority_readback_ref),
            Some(committed.markdown_readback_ref),
            Some(committed.sqlite_readback_ref),
            b"committed",
        )?;
        let reconciliation_ref = BindingDigest::new(sha256(&bytes));
        self.persist_reconciliation(
            case,
            MemoryReconciliationState::Reconciled,
            reconciliation_ref,
            Some(committed.authority_readback_ref),
            Some(committed.markdown_readback_ref),
            Some(committed.sqlite_readback_ref),
            &bytes,
        )?;

        let mut verification_refs = vec![
            committed.markdown_readback_ref,
            committed.sqlite_readback_ref,
        ];
        verification_refs.sort_unstable();
        verification_refs.dedup();
        let mut integrity_evidence_refs = vec![
            committed.authority_readback_ref,
            committed.promotion_evidence_ref,
            committed.prior_reconciliation_ref,
            reconciliation_ref,
        ];
        integrity_evidence_refs.sort_unstable();
        integrity_evidence_refs.dedup();
        let outcome = MemoryMutationOutcome {
            effect_id: case.effect_id,
            mutation_intent_digest: case.intent_digest,
            status: MemoryMutationStatus::Committed,
            canonical_version_refs: vec![case.version_id],
            authority_journal_readback_ref: Some(committed.authority_readback_ref),
            markdown_readback_ref: Some(committed.markdown_readback_ref),
            memory_sqlite_readback_ref: Some(committed.sqlite_readback_ref),
            reconciliation_ref: Some(reconciliation_ref),
            verification_refs,
            integrity_evidence_refs,
            terminal_at_unix_ms,
        };
        self.persist_terminal_if_absent(&outcome)?;
        self.mark_operational_terminal(case, MemoryMutationStatus::Committed)?;
        self.resolve_effect_state(
            case.effect_id,
            DesiredEffectOutcome::Succeeded,
            reconciliation_ref,
            finished_at,
        )?;
        Ok(MemoryRestartResolution::ReconciledCommitted)
    }

    fn resolve_unknown(
        &self,
        case: &MemoryRestartCase,
        authority_readback_ref: BindingDigest,
        markdown_readback_ref: Option<BindingDigest>,
        finished_at: &str,
        terminal_at_unix_ms: u64,
    ) -> Result<MemoryRestartResolution, MemoryRestartError> {
        self.require_terminal_compatible(case.effect_id, MemoryMutationStatus::UnknownOutcome)?;
        let bytes = restart_reconciliation_bytes(
            case,
            MemoryReconciliationState::Blocked,
            Some(authority_readback_ref),
            markdown_readback_ref,
            None,
            b"unknown_outcome",
        )?;
        let reconciliation_ref = BindingDigest::new(sha256(&bytes));
        self.persist_reconciliation(
            case,
            MemoryReconciliationState::Blocked,
            reconciliation_ref,
            Some(authority_readback_ref),
            markdown_readback_ref,
            None,
            &bytes,
        )?;
        let outcome = MemoryMutationOutcome {
            effect_id: case.effect_id,
            mutation_intent_digest: case.intent_digest,
            status: MemoryMutationStatus::UnknownOutcome,
            canonical_version_refs: Vec::new(),
            authority_journal_readback_ref: Some(authority_readback_ref),
            markdown_readback_ref,
            memory_sqlite_readback_ref: None,
            reconciliation_ref: Some(reconciliation_ref),
            verification_refs: Vec::new(),
            integrity_evidence_refs: vec![authority_readback_ref, reconciliation_ref],
            terminal_at_unix_ms,
        };
        self.persist_terminal_if_absent(&outcome)?;
        self.mark_operational_terminal(case, MemoryMutationStatus::UnknownOutcome)?;
        self.resolve_effect_state(
            case.effect_id,
            DesiredEffectOutcome::Unknown,
            reconciliation_ref,
            finished_at,
        )?;
        Ok(MemoryRestartResolution::BlockedUnknownOutcome)
    }

    fn prove_committed(
        &self,
        case: &MemoryRestartCase,
        observation: &MemoryRestartObservation,
        authority_readback_ref: BindingDigest,
    ) -> Result<Option<CommittedProof>, MemoryRestartError> {
        let MemoryRestartObservation::Regular {
            content_digest,
            markdown_readback_ref,
            ..
        } = observation
        else {
            return Ok(None);
        };
        let operational = self.operational_connection()?;
        let row = operational
            .query_row(
                r#"SELECT v.store_ref, v.item_id, v.markdown_path, v.content_digest,
                          v.promotion_evidence_ref, v.created_by_principal, v.writer_id,
                          v.effect_id, v.intent_digest,
                          i.current_version_id
                   FROM memory_versions v
                   JOIN memory_items i ON i.item_id = v.item_id
                   WHERE v.version_id = ?1"#,
                params![case.version_id.0.bytes().to_vec()],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, Vec<u8>>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Vec<u8>>(3)?,
                        row.get::<_, Vec<u8>>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, Vec<u8>>(6)?,
                        row.get::<_, Vec<u8>>(7)?,
                        row.get::<_, Vec<u8>>(8)?,
                        row.get::<_, Vec<u8>>(9)?,
                    ))
                },
            )
            .optional()?;
        let Some(row) = row else {
            return Ok(None);
        };
        if row.0.as_slice() != self.memory.store_id().0.bytes()
            || digest32(row.1.clone(), "operational item id")? != case.item_id
            || Path::new(&row.2) != case.markdown_path.as_path()
            || digest32(row.3.clone(), "operational content digest")? != *content_digest
            || row.7.as_slice() != case.effect_id.0.to_be_bytes()
            || digest32(row.8.clone(), "operational intent digest")? != case.intent_digest
            || digest32(row.9.clone(), "operational current version")? != case.version_id.0
        {
            return Ok(None);
        }
        let promotion_evidence_ref = digest32(row.4, "promotion evidence ref")?;
        let writer_id = digest32(row.6, "writer id")?;
        if !self.verify_version_authority_evidence(
            case,
            *content_digest,
            &row.5,
            writer_id,
        )? {
            return Ok(None);
        }
        if !self.promotion_evidence_exists(promotion_evidence_ref)? {
            return Ok(None);
        }
        let sqlite_readback_ref = sqlite_readback_ref(
            &self.memory,
            case,
            *content_digest,
            &row.2,
        )?;
        let prior_reconciliation_ref = match self.find_matching_reconciliation(
            case,
            authority_readback_ref,
            *markdown_readback_ref,
            sqlite_readback_ref,
        )? {
            Some(value) => value,
            None => return Ok(None),
        };
        Ok(Some(CommittedProof {
            authority_readback_ref,
            markdown_readback_ref: *markdown_readback_ref,
            sqlite_readback_ref,
            promotion_evidence_ref,
            prior_reconciliation_ref,
        }))
    }

    fn verify_version_authority_evidence(
        &self,
        case: &MemoryRestartCase,
        content_digest: BindingDigest,
        created_by_principal: &str,
        writer_id: BindingDigest,
    ) -> Result<bool, MemoryRestartError> {
        let connection = self.authority_connection()?;
        let row = connection
            .query_row(
                r#"SELECT item_id, created_by_principal, committed_by_writer_identity,
                          mutation_effect_id, record_bytes, integrity_hash
                   FROM memory_version_evidence WHERE version_id = ?1"#,
                params![case.version_id.0.bytes().to_vec()],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                        row.get::<_, Vec<u8>>(3)?,
                        row.get::<_, Vec<u8>>(4)?,
                        row.get::<_, Vec<u8>>(5)?,
                    ))
                },
            )
            .optional()?;
        let Some(row) = row else {
            return Ok(false);
        };
        if digest32(row.0, "authority version item")? != case.item_id
            || row.1 != created_by_principal
            || digest32(row.2, "authority writer id")? != writer_id
            || row.3.as_slice() != case.effect_id.0.to_be_bytes()
        {
            return Ok(false);
        }
        let expected_record = version_record_bytes(case, content_digest)?;
        if row.4 != expected_record {
            return Ok(false);
        }
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(VERSION_EVIDENCE_DOMAIN)?;
        encoder.push_bytes(&case.version_id.0.bytes())?;
        encoder.push_bytes(&case.item_id.bytes())?;
        encoder.push_bytes(created_by_principal.as_bytes())?;
        encoder.push_bytes(&writer_id.bytes())?;
        encoder.push_u128(case.effect_id.0);
        encoder.push_bytes(&expected_record)?;
        Ok(row.5.as_slice() == crate::payload_hash(&encoder.finish()))
    }

    fn promotion_evidence_exists(
        &self,
        promotion_evidence_ref: BindingDigest,
    ) -> Result<bool, MemoryRestartError> {
        let connection = self.authority_connection()?;
        Ok(connection
            .query_row(
                "SELECT 1 FROM memory_promotion_evidence WHERE evidence_id = ?1 LIMIT 1",
                params![promotion_evidence_ref.bytes().to_vec()],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some())
    }

    fn find_matching_reconciliation(
        &self,
        case: &MemoryRestartCase,
        authority_ref: BindingDigest,
        markdown_ref: BindingDigest,
        sqlite_ref: BindingDigest,
    ) -> Result<Option<BindingDigest>, MemoryRestartError> {
        let operational = self.operational_connection()?;
        let operational_match = operational
            .query_row(
                r#"SELECT evidence_ref FROM memory_reconciliation_state
                   WHERE effect_id = ?1 AND store_ref = ?2 AND intent_digest = ?3
                     AND state IN (1, 4)"#,
                params![
                    case.effect_id.0.to_be_bytes().to_vec(),
                    self.memory.store_id().0.bytes().to_vec(),
                    case.intent_digest.bytes().to_vec(),
                ],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()?;
        let Some(operational_ref) = operational_match else {
            return Ok(None);
        };
        let operational_ref = digest32(operational_ref, "operational reconciliation ref")?;

        let authority = self.authority_connection()?;
        let mut statement = authority.prepare(
            r#"SELECT evidence_id, state, authority_journal_readback_ref,
                      markdown_readback_ref, memory_sqlite_readback_ref,
                      record_bytes, integrity_hash
               FROM memory_reconciliation_evidence WHERE effect_id = ?1"#,
        )?;
        let rows = statement.query_map(
            params![case.effect_id.0.to_be_bytes().to_vec()],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<Vec<u8>>>(2)?,
                    row.get::<_, Option<Vec<u8>>>(3)?,
                    row.get::<_, Option<Vec<u8>>>(4)?,
                    row.get::<_, Vec<u8>>(5)?,
                    row.get::<_, Vec<u8>>(6)?,
                ))
            },
        )?;
        for row in rows {
            let row = row?;
            if !matches!(row.1, 1 | 4) {
                continue;
            }
            let evidence_id = digest32(row.0, "reconciliation evidence id")?;
            if evidence_id != operational_ref
                || optional_digest(row.2, "authority reconciliation ref")? != Some(authority_ref)
                || optional_digest(row.3, "Markdown reconciliation ref")? != Some(markdown_ref)
                || optional_digest(row.4, "SQLite reconciliation ref")? != Some(sqlite_ref)
                || evidence_id.bytes() != sha256(&row.5)
            {
                continue;
            }
            let mut encoder = CanonicalEncoder::new();
            encoder.push_bytes(RECONCILIATION_EVIDENCE_DOMAIN)?;
            encoder.push_bytes(&evidence_id.bytes())?;
            encoder.push_u128(case.effect_id.0);
            encoder.push_u64(u64::try_from(row.1).map_err(|_| {
                MemoryRestartError::InvalidRecord("reconciliation state")
            })?);
            push_optional_digest(&mut encoder, Some(authority_ref))?;
            push_optional_digest(&mut encoder, Some(markdown_ref))?;
            push_optional_digest(&mut encoder, Some(sqlite_ref))?;
            encoder.push_bytes(&row.5)?;
            if row.6.as_slice() == crate::payload_hash(&encoder.finish()) {
                return Ok(Some(evidence_id));
            }
        }
        Ok(None)
    }

    fn ensure_operational_prepared(
        &self,
        case: &MemoryRestartCase,
    ) -> Result<(), MemoryRestartError> {
        drop(MemoryOperationalStore::open(&self.memory)?);
        let connection = self.operational_connection()?;
        let existing = connection
            .query_row(
                "SELECT store_ref, intent_digest FROM memory_effect_state WHERE effect_id = ?1",
                params![case.effect_id.0.to_be_bytes().to_vec()],
                |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
            )
            .optional()?;
        if let Some(existing) = existing {
            if existing.0.as_slice() != self.memory.store_id().0.bytes()
                || existing.1.as_slice() != case.intent_digest.bytes()
            {
                return Err(MemoryRestartError::StoreBindingMismatch);
            }
            return Ok(());
        }
        connection.execute(
            r#"INSERT INTO memory_effect_state
               (effect_id, store_ref, intent_digest, operation,
                expected_markdown_target_identity_ref, expected_markdown_content_digest,
                expected_markdown_version_ref, state, terminal_status)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'prepared', NULL)"#,
            params![
                case.effect_id.0.to_be_bytes().to_vec(),
                self.memory.store_id().0.bytes().to_vec(),
                case.intent_digest.bytes().to_vec(),
                operation_code(case.operation),
                case.expected_target_identity_ref.bytes().to_vec(),
                case.expected_content_digest.bytes().to_vec(),
                case.expected_markdown_version.0.bytes().to_vec(),
            ],
        )?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn persist_reconciliation(
        &self,
        case: &MemoryRestartCase,
        state: MemoryReconciliationState,
        evidence_id: BindingDigest,
        authority_ref: Option<BindingDigest>,
        markdown_ref: Option<BindingDigest>,
        sqlite_ref: Option<BindingDigest>,
        record_bytes: &[u8],
    ) -> Result<(), MemoryRestartError> {
        let mut operational = MemoryOperationalStore::open(&self.memory)?;
        operational.record_reconciliation(case.effect_id, case.intent_digest, state, evidence_id)?;
        let mut evidence = MemoryEvidenceStore::open(self.authority.authority_db_path())?;
        evidence.persist_reconciliation(ReconciliationEvidence {
            evidence_id,
            effect_id: case.effect_id,
            state,
            authority_journal_readback_ref: authority_ref,
            markdown_readback_ref: markdown_ref,
            memory_sqlite_readback_ref: sqlite_ref,
            record_bytes,
        })?;
        Ok(())
    }

    fn mark_operational_terminal(
        &self,
        case: &MemoryRestartCase,
        status: MemoryMutationStatus,
    ) -> Result<(), MemoryRestartError> {
        let mut operational = MemoryOperationalStore::open(&self.memory)?;
        operational.mark_terminal(case.effect_id, case.intent_digest, status)?;
        Ok(())
    }

    fn persist_terminal_if_absent(
        &self,
        outcome: &MemoryMutationOutcome,
    ) -> Result<Option<BindingDigest>, MemoryRestartError> {
        let existing = self.existing_terminal(outcome.effect_id)?;
        if let Some((id, status)) = existing {
            if terminal_status_compatible(status, outcome.status) {
                return Ok(Some(id));
            }
            return Err(MemoryRestartError::ConflictingTerminalEvidence);
        }
        let bytes = terminal_bytes(outcome)?;
        let id = BindingDigest::new(sha256(&bytes));
        let mut evidence = MemoryEvidenceStore::open(self.authority.authority_db_path())?;
        evidence.persist_terminal_outcome(id, outcome, &bytes)?;
        Ok(Some(id))
    }

    fn existing_terminal(
        &self,
        effect_id: EffectId,
    ) -> Result<Option<(BindingDigest, MemoryMutationStatus)>, MemoryRestartError> {
        let connection = self.authority_connection()?;
        connection
            .query_row(
                "SELECT terminal_evidence_id, status FROM memory_terminal_outcomes WHERE effect_id = ?1",
                params![effect_id.0.to_be_bytes().to_vec()],
                |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()?
            .map(|row| {
                Ok((
                    digest32(row.0, "terminal evidence id")?,
                    mutation_status_from_code(row.1)?,
                ))
            })
            .transpose()
    }

    fn require_terminal_compatible(
        &self,
        effect_id: EffectId,
        desired: MemoryMutationStatus,
    ) -> Result<(), MemoryRestartError> {
        if let Some((_, existing)) = self.existing_terminal(effect_id)?
            && !terminal_status_compatible(existing, desired)
        {
            return Err(MemoryRestartError::ConflictingTerminalEvidence);
        }
        Ok(())
    }

    fn resolve_effect_state(
        &self,
        effect_id: EffectId,
        desired: DesiredEffectOutcome,
        evidence_ref: BindingDigest,
        finished_at: &str,
    ) -> Result<(), MemoryRestartError> {
        let mut effects = EffectStore::open(&self.authority)?;
        let mut state = effects
            .current_state(effect_id)?
            .ok_or(MemoryRestartError::InvalidRecord("effect current state"))?;

        if matches!(state.as_str(), "succeeded" | "failed") {
            return if desired.matches_terminal(&state) {
                Ok(())
            } else {
                Err(MemoryRestartError::ConflictingTerminalEvidence)
            };
        }
        if state == "manual_review" {
            return if desired == DesiredEffectOutcome::Unknown {
                Ok(())
            } else {
                Err(MemoryRestartError::UnsupportedEffectState(state))
            };
        }

        if state == "authorized" {
            let attempt_id = EffectAttemptId(restart_u128(effect_id, b"attempt"));
            effects.compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(restart_u128(effect_id, b"authorized-executing")),
                effect_id,
                expected_state: "authorized",
                next_state: "executing",
                attempt_id: Some(attempt_id),
                reason_code: Some("managed_memory_restart_reconciliation"),
                evidence_ref: Some(&evidence_ref.bytes()),
                event_id: EventId(restart_u128(effect_id, b"authorized-executing-event")),
            })?;
            effects.start_attempt(StartEffectAttempt {
                attempt_id,
                effect_id,
                handler_id: RESTART_HANDLER_ID,
                handler_version: RESTART_HANDLER_VERSION,
                dispatch_token: &restart_digest(effect_id, b"dispatch").bytes(),
                started_at: finished_at,
            })?;
            state = "executing".to_owned();
        }

        if state == "executing" {
            let attempt = self.latest_attempt(effect_id)?;
            let attempt_id = match attempt {
                Some(attempt) => {
                    if let Some(existing) = attempt.finished_outcome.as_deref() {
                        if existing != desired.attempt_outcome() && existing != "unknown" {
                            return Err(MemoryRestartError::ConflictingTerminalEvidence);
                        }
                    } else {
                        effects.finish_attempt(FinishEffectAttempt {
                            attempt_id: attempt.attempt_id,
                            finished_at,
                            outcome: desired.attempt_outcome(),
                            receipt: Some(&evidence_ref.bytes()),
                        })?;
                    }
                    attempt.attempt_id
                }
                None => {
                    let attempt_id = EffectAttemptId(restart_u128(effect_id, b"attempt-orphan"));
                    effects.start_attempt(StartEffectAttempt {
                        attempt_id,
                        effect_id,
                        handler_id: RESTART_HANDLER_ID,
                        handler_version: RESTART_HANDLER_VERSION,
                        dispatch_token: &restart_digest(effect_id, b"dispatch-orphan").bytes(),
                        started_at: finished_at,
                    })?;
                    effects.finish_attempt(FinishEffectAttempt {
                        attempt_id,
                        finished_at,
                        outcome: desired.attempt_outcome(),
                        receipt: Some(&evidence_ref.bytes()),
                    })?;
                    attempt_id
                }
            };

            if desired == DesiredEffectOutcome::Unknown {
                effects.compare_and_swap(CompareAndSwapEffect {
                    transition_id: EffectTransitionId(restart_u128(effect_id, b"executing-unknown")),
                    effect_id,
                    expected_state: "executing",
                    next_state: "unknown_outcome",
                    attempt_id: Some(attempt_id),
                    reason_code: Some("managed_memory_restart_unknown_outcome"),
                    evidence_ref: Some(&evidence_ref.bytes()),
                    event_id: EventId(restart_u128(effect_id, b"executing-unknown-event")),
                })?;
                return Ok(());
            }

            if self
                .latest_attempt(effect_id)?
                .and_then(|value| value.finished_outcome)
                .as_deref()
                == Some("unknown")
            {
                effects.compare_and_swap(CompareAndSwapEffect {
                    transition_id: EffectTransitionId(restart_u128(effect_id, b"executing-unknown")),
                    effect_id,
                    expected_state: "executing",
                    next_state: "unknown_outcome",
                    attempt_id: Some(attempt_id),
                    reason_code: Some("managed_memory_restart_prior_unknown"),
                    evidence_ref: Some(&evidence_ref.bytes()),
                    event_id: EventId(restart_u128(effect_id, b"executing-unknown-event")),
                })?;
                state = "unknown_outcome".to_owned();
            } else {
                effects.compare_and_swap(CompareAndSwapEffect {
                    transition_id: EffectTransitionId(restart_u128(effect_id, desired.direct_phase())),
                    effect_id,
                    expected_state: "executing",
                    next_state: desired.terminal_state(),
                    attempt_id: Some(attempt_id),
                    reason_code: Some(desired.reason_code()),
                    evidence_ref: Some(&evidence_ref.bytes()),
                    event_id: EventId(restart_u128(effect_id, desired.direct_event_phase())),
                })?;
                return Ok(());
            }
        }

        if state == "unknown_outcome" {
            effects.compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(restart_u128(effect_id, b"unknown-reconciling")),
                effect_id,
                expected_state: "unknown_outcome",
                next_state: "reconciling",
                attempt_id: self.latest_attempt(effect_id)?.map(|value| value.attempt_id),
                reason_code: Some("managed_memory_restart_reconciling"),
                evidence_ref: Some(&evidence_ref.bytes()),
                event_id: EventId(restart_u128(effect_id, b"unknown-reconciling-event")),
            })?;
            state = "reconciling".to_owned();
        }

        if state == "reconciling" {
            if desired == DesiredEffectOutcome::Unknown {
                return Ok(());
            }
            effects.compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(restart_u128(effect_id, desired.reconcile_phase())),
                effect_id,
                expected_state: "reconciling",
                next_state: desired.terminal_state(),
                attempt_id: self.latest_attempt(effect_id)?.map(|value| value.attempt_id),
                reason_code: Some(desired.reason_code()),
                evidence_ref: Some(&evidence_ref.bytes()),
                event_id: EventId(restart_u128(effect_id, desired.reconcile_event_phase())),
            })?;
            return Ok(());
        }

        if desired == DesiredEffectOutcome::Unknown && state == "unknown_outcome" {
            return Ok(());
        }
        Err(MemoryRestartError::UnsupportedEffectState(state))
    }

    fn latest_attempt(
        &self,
        effect_id: EffectId,
    ) -> Result<Option<RestartAttempt>, MemoryRestartError> {
        let connection = self.authority_connection()?;
        connection
            .query_row(
                r#"SELECT attempt_id, finished_at, outcome FROM effect_attempts
                   WHERE effect_id = ?1 ORDER BY started_global_seq DESC LIMIT 1"#,
                params![effect_id.0.to_be_bytes().to_vec()],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()?
            .map(|row| {
                Ok(RestartAttempt {
                    attempt_id: EffectAttemptId(u128::from_be_bytes(array16(
                        row.0,
                        "effect attempt id",
                    )?)),
                    finished_outcome: row.1.map(|_| row.2),
                })
            })
            .transpose()
    }

    fn authority_readback_ref(
        &self,
        effect_id: EffectId,
        intent_digest: BindingDigest,
    ) -> Result<BindingDigest, MemoryRestartError> {
        let connection = self.authority_connection()?;
        let row = connection
            .query_row(
                r#"SELECT i.intent_digest, i.integrity_hash, t.integrity_hash
                   FROM memory_prepared_intents i
                   JOIN memory_prepared_targets t ON t.effect_id = i.effect_id
                   WHERE i.effect_id = ?1"#,
                params![effect_id.0.to_be_bytes().to_vec()],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, Vec<u8>>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                    ))
                },
            )
            .optional()?
            .ok_or(MemoryRestartError::InvalidRecord("PREPARED authority readback"))?;
        if digest32(row.0, "PREPARED intent digest")? != intent_digest {
            return Err(MemoryRestartError::InvalidRecord(
                "PREPARED intent readback mismatch",
            ));
        }
        let intent_hash = digest32(row.1, "PREPARED integrity hash")?;
        let target_hash = digest32(row.2, "PREPARED target integrity hash")?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(PREPARED_READBACK_DOMAIN)?;
        encoder.push_u128(effect_id.0);
        encoder.push_bytes(&intent_digest.bytes())?;
        encoder.push_bytes(&intent_hash.bytes())?;
        encoder.push_bytes(&target_hash.bytes())?;
        Ok(BindingDigest::new(crate::payload_hash(&encoder.finish())))
    }

    fn authority_connection(&self) -> Result<Connection, MemoryRestartError> {
        let connection = Connection::open(self.authority.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(connection)
    }

    fn operational_connection(&self) -> Result<Connection, MemoryRestartError> {
        let connection = Connection::open(self.memory.operational_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(connection)
    }
}

#[derive(Clone, Copy)]
struct CommittedProof {
    authority_readback_ref: BindingDigest,
    markdown_readback_ref: BindingDigest,
    sqlite_readback_ref: BindingDigest,
    promotion_evidence_ref: BindingDigest,
    prior_reconciliation_ref: BindingDigest,
}

struct RestartAttempt {
    attempt_id: EffectAttemptId,
    finished_outcome: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DesiredEffectOutcome {
    Succeeded,
    Failed,
    Unknown,
}

impl DesiredEffectOutcome {
    const fn attempt_outcome(self) -> &'static str {
        match self {
            Self::Succeeded => "success",
            Self::Failed => "failure",
            Self::Unknown => "unknown",
        }
    }

    const fn terminal_state(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Unknown => "unknown_outcome",
        }
    }

    const fn matches_terminal(self, state: &str) -> bool {
        matches!((self, state), (Self::Succeeded, "succeeded") | (Self::Failed, "failed"))
    }

    const fn reason_code(self) -> &'static str {
        match self {
            Self::Succeeded => "managed_memory_restart_verified",
            Self::Failed => "managed_memory_restart_no_mutation",
            Self::Unknown => "managed_memory_restart_unknown_outcome",
        }
    }

    const fn direct_phase(self) -> &'static [u8] {
        match self {
            Self::Succeeded => b"executing-succeeded",
            Self::Failed => b"executing-failed",
            Self::Unknown => b"executing-unknown",
        }
    }

    const fn direct_event_phase(self) -> &'static [u8] {
        match self {
            Self::Succeeded => b"executing-succeeded-event",
            Self::Failed => b"executing-failed-event",
            Self::Unknown => b"executing-unknown-event",
        }
    }

    const fn reconcile_phase(self) -> &'static [u8] {
        match self {
            Self::Succeeded => b"reconciling-succeeded",
            Self::Failed => b"reconciling-failed",
            Self::Unknown => b"reconciling-unknown",
        }
    }

    const fn reconcile_event_phase(self) -> &'static [u8] {
        match self {
            Self::Succeeded => b"reconciling-succeeded-event",
            Self::Failed => b"reconciling-failed-event",
            Self::Unknown => b"reconciling-unknown-event",
        }
    }
}

fn decode_operation(canonical_bytes: &[u8]) -> Result<MemoryOperation, MemoryRestartError> {
    if canonical_bytes.len() < 5 {
        return Err(MemoryRestartError::InvalidRecord("prepared intent canonical bytes"));
    }
    let length = u32::from_be_bytes(
        canonical_bytes[0..4]
            .try_into()
            .map_err(|_| MemoryRestartError::InvalidRecord("intent domain length"))?,
    ) as usize;
    let end = 4_usize
        .checked_add(length)
        .ok_or(MemoryRestartError::InvalidRecord("intent domain length"))?;
    if end >= canonical_bytes.len() || &canonical_bytes[4..end] != INTENT_DOMAIN {
        return Err(MemoryRestartError::InvalidRecord("prepared intent domain"));
    }
    match canonical_bytes[end] {
        1 => Ok(MemoryOperation::Add),
        2 => Ok(MemoryOperation::Update),
        3 => Ok(MemoryOperation::Supersede),
        4 => Ok(MemoryOperation::Contradict),
        5 => Ok(MemoryOperation::Merge),
        6 => Ok(MemoryOperation::Expire),
        7 => Ok(MemoryOperation::Forget),
        8 => Ok(MemoryOperation::Redact),
        _ => Err(MemoryRestartError::InvalidRecord("memory operation code")),
    }
}

#[allow(clippy::too_many_arguments)]
fn prepared_target_record_bytes(
    effect_id: EffectId,
    intent_digest: BindingDigest,
    memory_store_ref: MemoryStoreId,
    item_id: BindingDigest,
    scope_code: i64,
    version_id: MemoryVersionId,
    markdown_path: &str,
    target_identity_ref: BindingDigest,
    expected_content_digest: BindingDigest,
    expected_markdown_version: MemoryVersionId,
) -> Result<Vec<u8>, MemoryRestartError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(PREPARED_TARGET_DOMAIN)?;
    encoder.push_u128(effect_id.0);
    for digest in [
        intent_digest,
        memory_store_ref.0,
        item_id,
        version_id.0,
        target_identity_ref,
        expected_content_digest,
        expected_markdown_version.0,
    ] {
        encoder.push_bytes(&digest.bytes())?;
    }
    let scope = match scope_code {
        1 => 1,
        2 => 2,
        _ => return Err(MemoryRestartError::InvalidRecord("prepared target scope")),
    };
    encoder.push_u8(scope);
    encoder.push_bytes(markdown_path.as_bytes())?;
    Ok(encoder.finish())
}

fn sqlite_readback_ref(
    memory: &MemoryLayout,
    case: &MemoryRestartCase,
    content_digest: BindingDigest,
    markdown_path: &str,
) -> Result<BindingDigest, MemoryRestartError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(SQLITE_READBACK_DOMAIN)?;
    encoder.push_bytes(&memory.store_id().0.bytes())?;
    encoder.push_bytes(&memory.schema_ref().bytes())?;
    encoder.push_bytes(&case.intent_digest.bytes())?;
    encoder.push_u128(case.effect_id.0);
    encoder.push_bytes(&case.version_id.0.bytes())?;
    encoder.push_bytes(&content_digest.bytes())?;
    encoder.push_bytes(markdown_path.as_bytes())?;
    Ok(BindingDigest::new(crate::payload_hash(&encoder.finish())))
}

fn version_record_bytes(
    case: &MemoryRestartCase,
    content_digest: BindingDigest,
) -> Result<Vec<u8>, MemoryRestartError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(VERSION_RECORD_DOMAIN)?;
    encoder.push_bytes(&case.version_id.0.bytes())?;
    encoder.push_bytes(&case.item_id.bytes())?;
    encoder.push_bytes(&content_digest.bytes())?;
    encoder.push_u128(case.effect_id.0);
    Ok(encoder.finish())
}

#[allow(clippy::too_many_arguments)]
fn restart_reconciliation_bytes(
    case: &MemoryRestartCase,
    state: MemoryReconciliationState,
    authority_ref: Option<BindingDigest>,
    markdown_ref: Option<BindingDigest>,
    sqlite_ref: Option<BindingDigest>,
    disposition: &[u8],
) -> Result<Vec<u8>, MemoryRestartError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(RESTART_RECONCILIATION_DOMAIN)?;
    encoder.push_u128(case.effect_id.0);
    encoder.push_bytes(&case.intent_digest.bytes())?;
    encoder.push_u8(reconciliation_state_code(state));
    push_optional_digest(&mut encoder, authority_ref)?;
    push_optional_digest(&mut encoder, markdown_ref)?;
    push_optional_digest(&mut encoder, sqlite_ref)?;
    encoder.push_bytes(disposition)?;
    Ok(encoder.finish())
}

fn terminal_bytes(outcome: &MemoryMutationOutcome) -> Result<Vec<u8>, MemoryRestartError> {
    outcome
        .validate()
        .map_err(|_| MemoryRestartError::InvalidRecord("memory terminal outcome"))?;
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(TERMINAL_DOMAIN)?;
    encoder.push_u128(outcome.effect_id.0);
    encoder.push_bytes(&outcome.mutation_intent_digest.bytes())?;
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
                encoder.push_bytes(&value.bytes())?;
            }
            None => encoder.push_u8(0),
        }
    }
    Ok(encoder.finish())
}

fn push_optional_digest(
    encoder: &mut CanonicalEncoder,
    value: Option<BindingDigest>,
) -> Result<(), MemoryRestartError> {
    match value {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(&value.bytes())?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn restart_digest(effect_id: EffectId, phase: &[u8]) -> BindingDigest {
    let mut encoder = CanonicalEncoder::new();
    encoder
        .push_bytes(RESTART_ID_DOMAIN)
        .expect("constant restart id domain is bounded");
    encoder.push_u128(effect_id.0);
    encoder
        .push_bytes(phase)
        .expect("constant restart id phase is bounded");
    BindingDigest::new(crate::payload_hash(&encoder.finish()))
}

fn restart_u128(effect_id: EffectId, phase: &[u8]) -> u128 {
    let bytes = restart_digest(effect_id, phase).bytes();
    u128::from_be_bytes(bytes[..16].try_into().expect("digest prefix is 16 bytes"))
}

fn operation_code(operation: MemoryOperation) -> i64 {
    match operation {
        MemoryOperation::Add => 1,
        MemoryOperation::Update => 2,
        MemoryOperation::Supersede => 3,
        MemoryOperation::Contradict => 4,
        MemoryOperation::Merge => 5,
        MemoryOperation::Expire => 6,
        MemoryOperation::Forget => 7,
        MemoryOperation::Redact => 8,
    }
}

fn reconciliation_state_code(state: MemoryReconciliationState) -> u8 {
    match state {
        MemoryReconciliationState::InSync => 1,
        MemoryReconciliationState::UserEditDetected => 2,
        MemoryReconciliationState::Conflict => 3,
        MemoryReconciliationState::Reconciled => 4,
        MemoryReconciliationState::Blocked => 5,
    }
}

fn mutation_status_from_code(value: i64) -> Result<MemoryMutationStatus, MemoryRestartError> {
    match value {
        1 => Ok(MemoryMutationStatus::Committed),
        2 => Ok(MemoryMutationStatus::Rejected),
        3 => Ok(MemoryMutationStatus::Failed),
        4 => Ok(MemoryMutationStatus::UnknownOutcome),
        _ => Err(MemoryRestartError::InvalidRecord("memory terminal status")),
    }
}

fn terminal_status_compatible(
    existing: MemoryMutationStatus,
    desired: MemoryMutationStatus,
) -> bool {
    existing == desired || existing == MemoryMutationStatus::UnknownOutcome
}

fn digest32(value: Vec<u8>, field: &'static str) -> Result<BindingDigest, MemoryRestartError> {
    Ok(BindingDigest::new(
        value
            .try_into()
            .map_err(|_| MemoryRestartError::InvalidRecord(field))?,
    ))
}

fn optional_digest(
    value: Option<Vec<u8>>,
    field: &'static str,
) -> Result<Option<BindingDigest>, MemoryRestartError> {
    value.map(|value| digest32(value, field)).transpose()
}

fn array16(value: Vec<u8>, field: &'static str) -> Result<[u8; 16], MemoryRestartError> {
    value
        .try_into()
        .map_err(|_| MemoryRestartError::InvalidRecord(field))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restart_ids_are_stable_and_phase_separated() {
        let effect = EffectId(42);
        assert_eq!(restart_u128(effect, b"attempt"), restart_u128(effect, b"attempt"));
        assert_ne!(restart_u128(effect, b"attempt"), restart_u128(effect, b"transition"));
    }

    #[test]
    fn terminal_unknown_is_historical_and_compatible_with_later_resolution() {
        assert!(terminal_status_compatible(
            MemoryMutationStatus::UnknownOutcome,
            MemoryMutationStatus::Committed
        ));
        assert!(terminal_status_compatible(
            MemoryMutationStatus::UnknownOutcome,
            MemoryMutationStatus::Failed
        ));
        assert!(!terminal_status_compatible(
            MemoryMutationStatus::Committed,
            MemoryMutationStatus::Failed
        ));
    }
}
