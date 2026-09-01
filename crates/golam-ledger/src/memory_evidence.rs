#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::Path;

use golam_core::memory::{
    MemoryCandidateId, MemoryMutationOutcome, MemoryMutationStatus, MemoryReconciliationState,
    MemoryVersion, PreparedMemoryMutationIntent,
};
use golam_core::tool_request::{BindingDigest, PrincipalId};
use golam_core::{CanonicalEncoder, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

pub const MEMORY_EVIDENCE_SCHEMA_VERSION: i64 = 1;
const MEMORY_SECURITY_CHAIN_DOMAIN: &[u8] = b"golam:memory-security-chain:v1";

pub const REQUIRED_MEMORY_EVIDENCE_TABLES: &[&str] = &[
    "memory_evidence_schema_meta",
    "memory_prepared_intents",
    "memory_version_evidence",
    "memory_promotion_evidence",
    "memory_reconciliation_evidence",
    "memory_terminal_outcomes",
    "memory_security_chain",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PromotionEvidence<'a> {
    pub evidence_id: BindingDigest,
    pub candidate_id: MemoryCandidateId,
    pub promotion_authority_ref: BindingDigest,
    pub approving_principal: Option<&'a PrincipalId>,
    pub verifier_policy_ref: Option<BindingDigest>,
    pub record_bytes: &'a [u8],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReconciliationEvidence<'a> {
    pub evidence_id: BindingDigest,
    pub effect_id: EffectId,
    pub state: MemoryReconciliationState,
    pub authority_journal_readback_ref: Option<BindingDigest>,
    pub markdown_readback_ref: Option<BindingDigest>,
    pub memory_sqlite_readback_ref: Option<BindingDigest>,
    pub record_bytes: &'a [u8],
}

#[derive(Debug)]
pub enum MemoryEvidenceError {
    Sqlite(rusqlite::Error),
    InvalidRecord(&'static str),
    FutureSchema { found: i64, supported: i64 },
    ImmutableEvidenceMismatch(&'static str),
    MissingPreparedIntent(EffectId),
    IntentDigestMismatch,
    IntegerOverflow,
}

impl fmt::Display for MemoryEvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "memory evidence sqlite error: {error}"),
            Self::InvalidRecord(reason) => write!(f, "invalid memory evidence: {reason}"),
            Self::FutureSchema { found, supported } => write!(
                f,
                "memory evidence schema {found} is newer than supported {supported}"
            ),
            Self::ImmutableEvidenceMismatch(kind) => {
                write!(f, "immutable {kind} evidence identity collision")
            }
            Self::MissingPreparedIntent(effect_id) => write!(
                f,
                "memory effect {} has no durable PREPARED intent",
                effect_id.0
            ),
            Self::IntentDigestMismatch => {
                f.write_str("memory evidence does not bind the durable PREPARED intent digest")
            }
            Self::IntegerOverflow => f.write_str("integer conversion overflow"),
        }
    }
}

impl Error for MemoryEvidenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for MemoryEvidenceError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub struct MemoryEvidenceStore {
    connection: Connection,
}

impl MemoryEvidenceStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, MemoryEvidenceError> {
        Self::initialize(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self, MemoryEvidenceError> {
        Self::initialize(Connection::open_in_memory()?)
    }

    fn initialize(connection: Connection) -> Result<Self, MemoryEvidenceError> {
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        migrate(&connection)?;
        verify_required_tables(&connection)?;
        Ok(Self { connection })
    }

    pub fn schema_version(&self) -> Result<i64, MemoryEvidenceError> {
        Ok(self.connection.query_row(
            "SELECT schema_version FROM memory_evidence_schema_meta WHERE singleton = 1",
            [],
            |row| row.get(0),
        )?)
    }

    pub fn persist_prepared_intent(
        &mut self,
        prepared: &PreparedMemoryMutationIntent,
    ) -> Result<(), MemoryEvidenceError> {
        let intent = prepared.intent();
        let canonical_bytes = intent
            .canonical_bytes()
            .map_err(|_| MemoryEvidenceError::InvalidRecord("memory mutation intent"))?;
        let intent_digest = prepared.binding_digest();
        let integrity_hash = crate::payload_hash(&canonical_bytes);
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if existing_hash(
            &tx,
            "memory_prepared_intents",
            "effect_id",
            &id_blob(intent.effect_id.0),
        )?
        .is_some_and(|existing| existing == integrity_hash)
        {
            tx.commit()?;
            return Ok(());
        }
        if effect_exists(&tx, "memory_prepared_intents", intent.effect_id)? {
            return Err(MemoryEvidenceError::ImmutableEvidenceMismatch(
                "PREPARED memory intent",
            ));
        }
        tx.execute(
            r#"INSERT INTO memory_prepared_intents
               (effect_id, intent_digest, memory_store_ref, initiating_principal,
                markdown_target_identity_ref, markdown_content_digest, markdown_version_ref,
                canonical_bytes, integrity_hash)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
            params![
                id_blob(intent.effect_id.0),
                &intent_digest[..],
                &intent.memory_operational_store_ref.0.bytes()[..],
                intent.initiating_principal.as_str(),
                &intent.expected_markdown_target_identity_ref.bytes()[..],
                &intent.expected_markdown_content_digest.bytes()[..],
                &intent.expected_markdown_version.0.bytes()[..],
                canonical_bytes,
                &integrity_hash[..],
            ],
        )?;
        append_security_chain(&tx, 1, &id_blob(intent.effect_id.0), integrity_hash)?;
        tx.commit()?;
        Ok(())
    }

    pub fn persist_version(
        &mut self,
        version: &MemoryVersion,
        record_bytes: &[u8],
    ) -> Result<(), MemoryEvidenceError> {
        version
            .validate()
            .map_err(|_| MemoryEvidenceError::InvalidRecord("memory version"))?;
        if record_bytes.is_empty() {
            return Err(MemoryEvidenceError::InvalidRecord("memory version bytes"));
        }
        self.require_prepared_effect(version.mutation_effect_ref, None)?;
        let integrity_hash = crate::payload_hash(record_bytes);
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let version_key = version.version_id.0.bytes();
        if existing_hash(&tx, "memory_version_evidence", "version_id", &version_key)?
            .is_some_and(|existing| existing == integrity_hash)
        {
            tx.commit()?;
            return Ok(());
        }
        if row_exists(&tx, "memory_version_evidence", "version_id", &version_key)? {
            return Err(MemoryEvidenceError::ImmutableEvidenceMismatch(
                "memory version",
            ));
        }
        tx.execute(
            r#"INSERT INTO memory_version_evidence
               (version_id, item_id, created_by_principal, committed_by_writer_identity,
                mutation_effect_id, record_bytes, integrity_hash)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![
                &version_key[..],
                &version.item_id.0.bytes()[..],
                version.created_by_principal.as_str(),
                &version.committed_by_writer_identity.0.bytes()[..],
                id_blob(version.mutation_effect_ref.0),
                record_bytes,
                &integrity_hash[..],
            ],
        )?;
        append_security_chain(&tx, 2, &version_key, integrity_hash)?;
        tx.commit()?;
        Ok(())
    }

    pub fn persist_promotion(
        &mut self,
        evidence: PromotionEvidence<'_>,
    ) -> Result<(), MemoryEvidenceError> {
        if evidence.record_bytes.is_empty()
            || evidence.approving_principal.is_some() == evidence.verifier_policy_ref.is_some()
        {
            return Err(MemoryEvidenceError::InvalidRecord(
                "promotion must bind exactly one attributable authority mode",
            ));
        }
        let integrity_hash = crate::payload_hash(evidence.record_bytes);
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let key = evidence.evidence_id.bytes();
        if existing_hash(&tx, "memory_promotion_evidence", "evidence_id", &key)?
            .is_some_and(|existing| existing == integrity_hash)
        {
            tx.commit()?;
            return Ok(());
        }
        if row_exists(&tx, "memory_promotion_evidence", "evidence_id", &key)? {
            return Err(MemoryEvidenceError::ImmutableEvidenceMismatch(
                "memory promotion",
            ));
        }
        tx.execute(
            r#"INSERT INTO memory_promotion_evidence
               (evidence_id, candidate_id, promotion_authority_ref, approving_principal,
                verifier_policy_ref, record_bytes, integrity_hash)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![
                &key[..],
                &evidence.candidate_id.0.bytes()[..],
                &evidence.promotion_authority_ref.bytes()[..],
                evidence.approving_principal.map(PrincipalId::as_str),
                evidence
                    .verifier_policy_ref
                    .map(|value| value.bytes().to_vec()),
                evidence.record_bytes,
                &integrity_hash[..],
            ],
        )?;
        append_security_chain(&tx, 3, &key, integrity_hash)?;
        tx.commit()?;
        Ok(())
    }

    pub fn persist_reconciliation(
        &mut self,
        evidence: ReconciliationEvidence<'_>,
    ) -> Result<(), MemoryEvidenceError> {
        if evidence.record_bytes.is_empty() {
            return Err(MemoryEvidenceError::InvalidRecord(
                "memory reconciliation bytes",
            ));
        }
        self.require_prepared_effect(evidence.effect_id, None)?;
        let integrity_hash = crate::payload_hash(evidence.record_bytes);
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let key = evidence.evidence_id.bytes();
        if existing_hash(&tx, "memory_reconciliation_evidence", "evidence_id", &key)?
            .is_some_and(|existing| existing == integrity_hash)
        {
            tx.commit()?;
            return Ok(());
        }
        if row_exists(&tx, "memory_reconciliation_evidence", "evidence_id", &key)? {
            return Err(MemoryEvidenceError::ImmutableEvidenceMismatch(
                "memory reconciliation",
            ));
        }
        tx.execute(
            r#"INSERT INTO memory_reconciliation_evidence
               (evidence_id, effect_id, state, authority_journal_readback_ref,
                markdown_readback_ref, memory_sqlite_readback_ref, record_bytes, integrity_hash)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            params![
                &key[..],
                id_blob(evidence.effect_id.0),
                reconciliation_state_code(evidence.state),
                evidence
                    .authority_journal_readback_ref
                    .map(|value| value.bytes().to_vec()),
                evidence
                    .markdown_readback_ref
                    .map(|value| value.bytes().to_vec()),
                evidence
                    .memory_sqlite_readback_ref
                    .map(|value| value.bytes().to_vec()),
                evidence.record_bytes,
                &integrity_hash[..],
            ],
        )?;
        append_security_chain(&tx, 4, &key, integrity_hash)?;
        tx.commit()?;
        Ok(())
    }

    pub fn persist_terminal_outcome(
        &mut self,
        terminal_evidence_id: BindingDigest,
        outcome: &MemoryMutationOutcome,
        record_bytes: &[u8],
    ) -> Result<(), MemoryEvidenceError> {
        outcome
            .validate()
            .map_err(|_| MemoryEvidenceError::InvalidRecord("memory terminal outcome"))?;
        if record_bytes.is_empty() {
            return Err(MemoryEvidenceError::InvalidRecord(
                "memory terminal outcome bytes",
            ));
        }
        if outcome.status == MemoryMutationStatus::UnknownOutcome
            && outcome.reconciliation_ref.is_none()
        {
            return Err(MemoryEvidenceError::InvalidRecord(
                "UNKNOWN_OUTCOME requires reconciliation evidence identity",
            ));
        }
        self.require_prepared_effect(outcome.effect_id, Some(outcome.mutation_intent_digest))?;
        let integrity_hash = crate::payload_hash(record_bytes);
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let key = terminal_evidence_id.bytes();
        if existing_hash(
            &tx,
            "memory_terminal_outcomes",
            "terminal_evidence_id",
            &key,
        )?
        .is_some_and(|existing| existing == integrity_hash)
        {
            tx.commit()?;
            return Ok(());
        }
        if row_exists(
            &tx,
            "memory_terminal_outcomes",
            "terminal_evidence_id",
            &key,
        )? || effect_exists(&tx, "memory_terminal_outcomes", outcome.effect_id)?
        {
            return Err(MemoryEvidenceError::ImmutableEvidenceMismatch(
                "memory terminal outcome",
            ));
        }
        tx.execute(
            r#"INSERT INTO memory_terminal_outcomes
               (terminal_evidence_id, effect_id, intent_digest, status,
                authority_journal_readback_ref, markdown_readback_ref,
                memory_sqlite_readback_ref, reconciliation_ref, record_bytes, integrity_hash)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
            params![
                &key[..],
                id_blob(outcome.effect_id.0),
                &outcome.mutation_intent_digest.bytes()[..],
                mutation_status_code(outcome.status),
                outcome
                    .authority_journal_readback_ref
                    .map(|value| value.bytes().to_vec()),
                outcome
                    .markdown_readback_ref
                    .map(|value| value.bytes().to_vec()),
                outcome
                    .memory_sqlite_readback_ref
                    .map(|value| value.bytes().to_vec()),
                outcome
                    .reconciliation_ref
                    .map(|value| value.bytes().to_vec()),
                record_bytes,
                &integrity_hash[..],
            ],
        )?;
        append_security_chain(&tx, 5, &key, integrity_hash)?;
        tx.commit()?;
        Ok(())
    }

    pub fn security_chain_len(&self) -> Result<u64, MemoryEvidenceError> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM memory_security_chain",
            [],
            |row| row.get(0),
        )?;
        u64::try_from(count).map_err(|_| MemoryEvidenceError::IntegerOverflow)
    }

    fn require_prepared_effect(
        &self,
        effect_id: EffectId,
        expected_digest: Option<BindingDigest>,
    ) -> Result<(), MemoryEvidenceError> {
        let stored = self
            .connection
            .query_row(
                "SELECT intent_digest FROM memory_prepared_intents WHERE effect_id = ?1",
                params![id_blob(effect_id.0)],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()?;
        let Some(stored) = stored else {
            return Err(MemoryEvidenceError::MissingPreparedIntent(effect_id));
        };
        if let Some(expected) = expected_digest {
            if stored.as_slice() != expected.bytes() {
                return Err(MemoryEvidenceError::IntentDigestMismatch);
            }
        }
        Ok(())
    }
}

fn migrate(connection: &Connection) -> Result<(), MemoryEvidenceError> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS memory_evidence_schema_meta (
            singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
            schema_version INTEGER NOT NULL
        );
        INSERT OR IGNORE INTO memory_evidence_schema_meta (singleton, schema_version) VALUES (1, 1);

        CREATE TABLE IF NOT EXISTS memory_prepared_intents (
            effect_id BLOB PRIMARY KEY NOT NULL CHECK (length(effect_id) = 16),
            intent_digest BLOB NOT NULL CHECK (length(intent_digest) = 32),
            memory_store_ref BLOB NOT NULL CHECK (length(memory_store_ref) = 32),
            initiating_principal TEXT NOT NULL,
            markdown_target_identity_ref BLOB NOT NULL CHECK (length(markdown_target_identity_ref) = 32),
            markdown_content_digest BLOB NOT NULL CHECK (length(markdown_content_digest) = 32),
            markdown_version_ref BLOB NOT NULL CHECK (length(markdown_version_ref) = 32),
            canonical_bytes BLOB NOT NULL,
            integrity_hash BLOB NOT NULL CHECK (length(integrity_hash) = 32)
        );

        CREATE TABLE IF NOT EXISTS memory_version_evidence (
            version_id BLOB PRIMARY KEY NOT NULL CHECK (length(version_id) = 32),
            item_id BLOB NOT NULL CHECK (length(item_id) = 32),
            created_by_principal TEXT NOT NULL,
            committed_by_writer_identity BLOB NOT NULL CHECK (length(committed_by_writer_identity) = 32),
            mutation_effect_id BLOB NOT NULL CHECK (length(mutation_effect_id) = 16),
            record_bytes BLOB NOT NULL,
            integrity_hash BLOB NOT NULL CHECK (length(integrity_hash) = 32),
            FOREIGN KEY(mutation_effect_id) REFERENCES memory_prepared_intents(effect_id)
        );

        CREATE TABLE IF NOT EXISTS memory_promotion_evidence (
            evidence_id BLOB PRIMARY KEY NOT NULL CHECK (length(evidence_id) = 32),
            candidate_id BLOB NOT NULL CHECK (length(candidate_id) = 32),
            promotion_authority_ref BLOB NOT NULL CHECK (length(promotion_authority_ref) = 32),
            approving_principal TEXT,
            verifier_policy_ref BLOB CHECK (verifier_policy_ref IS NULL OR length(verifier_policy_ref) = 32),
            record_bytes BLOB NOT NULL,
            integrity_hash BLOB NOT NULL CHECK (length(integrity_hash) = 32),
            CHECK ((approving_principal IS NULL) != (verifier_policy_ref IS NULL))
        );

        CREATE TABLE IF NOT EXISTS memory_reconciliation_evidence (
            evidence_id BLOB PRIMARY KEY NOT NULL CHECK (length(evidence_id) = 32),
            effect_id BLOB NOT NULL CHECK (length(effect_id) = 16),
            state INTEGER NOT NULL,
            authority_journal_readback_ref BLOB,
            markdown_readback_ref BLOB,
            memory_sqlite_readback_ref BLOB,
            record_bytes BLOB NOT NULL,
            integrity_hash BLOB NOT NULL CHECK (length(integrity_hash) = 32),
            FOREIGN KEY(effect_id) REFERENCES memory_prepared_intents(effect_id)
        );

        CREATE TABLE IF NOT EXISTS memory_terminal_outcomes (
            terminal_evidence_id BLOB PRIMARY KEY NOT NULL CHECK (length(terminal_evidence_id) = 32),
            effect_id BLOB UNIQUE NOT NULL CHECK (length(effect_id) = 16),
            intent_digest BLOB NOT NULL CHECK (length(intent_digest) = 32),
            status INTEGER NOT NULL,
            authority_journal_readback_ref BLOB,
            markdown_readback_ref BLOB,
            memory_sqlite_readback_ref BLOB,
            reconciliation_ref BLOB,
            record_bytes BLOB NOT NULL,
            integrity_hash BLOB NOT NULL CHECK (length(integrity_hash) = 32),
            FOREIGN KEY(effect_id) REFERENCES memory_prepared_intents(effect_id)
        );

        CREATE TABLE IF NOT EXISTS memory_security_chain (
            sequence INTEGER PRIMARY KEY AUTOINCREMENT,
            record_kind INTEGER NOT NULL,
            record_identity BLOB NOT NULL,
            payload_hash BLOB NOT NULL CHECK (length(payload_hash) = 32),
            previous_integrity_hash BLOB,
            integrity_hash BLOB UNIQUE NOT NULL CHECK (length(integrity_hash) = 32)
        );
        "#,
    )?;
    let version: i64 = connection.query_row(
        "SELECT schema_version FROM memory_evidence_schema_meta WHERE singleton = 1",
        [],
        |row| row.get(0),
    )?;
    if version > MEMORY_EVIDENCE_SCHEMA_VERSION {
        return Err(MemoryEvidenceError::FutureSchema {
            found: version,
            supported: MEMORY_EVIDENCE_SCHEMA_VERSION,
        });
    }
    Ok(())
}

fn verify_required_tables(connection: &Connection) -> Result<(), MemoryEvidenceError> {
    for table in REQUIRED_MEMORY_EVIDENCE_TABLES {
        let present = connection
            .query_row(
                "SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?1 LIMIT 1",
                params![table],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if !present {
            return Err(MemoryEvidenceError::InvalidRecord(
                "required memory evidence table missing",
            ));
        }
    }
    Ok(())
}

fn append_security_chain(
    tx: &Transaction<'_>,
    record_kind: i64,
    record_identity: &[u8],
    payload_hash: [u8; 32],
) -> Result<(), MemoryEvidenceError> {
    let previous = tx
        .query_row(
            "SELECT integrity_hash FROM memory_security_chain ORDER BY sequence DESC LIMIT 1",
            [],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .optional()?
        .map(vec_to_hash)
        .transpose()?;
    let mut encoder = CanonicalEncoder::new();
    encoder
        .push_bytes(MEMORY_SECURITY_CHAIN_DOMAIN)
        .map_err(|_| MemoryEvidenceError::InvalidRecord("security-chain domain"))?;
    encoder.push_u64(u64::try_from(record_kind).map_err(|_| MemoryEvidenceError::IntegerOverflow)?);
    encoder
        .push_bytes(record_identity)
        .map_err(|_| MemoryEvidenceError::InvalidRecord("security-chain identity"))?;
    encoder
        .push_bytes(&payload_hash)
        .map_err(|_| MemoryEvidenceError::InvalidRecord("security-chain payload"))?;
    match previous {
        Some(hash) => {
            encoder.push_u8(1);
            encoder
                .push_bytes(&hash)
                .map_err(|_| MemoryEvidenceError::InvalidRecord("security-chain previous"))?;
        }
        None => encoder.push_u8(0),
    }
    let integrity_hash = crate::payload_hash(&encoder.finish());
    tx.execute(
        r#"INSERT INTO memory_security_chain
           (record_kind, record_identity, payload_hash, previous_integrity_hash, integrity_hash)
           VALUES (?1, ?2, ?3, ?4, ?5)"#,
        params![
            record_kind,
            record_identity,
            &payload_hash[..],
            previous.map(|hash| hash.to_vec()),
            &integrity_hash[..],
        ],
    )?;
    Ok(())
}

fn existing_hash(
    tx: &Transaction<'_>,
    table: &str,
    key_column: &str,
    key: &[u8],
) -> Result<Option<[u8; 32]>, MemoryEvidenceError> {
    let sql = format!("SELECT integrity_hash FROM {table} WHERE {key_column} = ?1");
    tx.query_row(&sql, params![key], |row| row.get::<_, Vec<u8>>(0))
        .optional()?
        .map(vec_to_hash)
        .transpose()
}

fn row_exists(
    tx: &Transaction<'_>,
    table: &str,
    key_column: &str,
    key: &[u8],
) -> Result<bool, MemoryEvidenceError> {
    let sql = format!("SELECT 1 FROM {table} WHERE {key_column} = ?1 LIMIT 1");
    Ok(tx
        .query_row(&sql, params![key], |row| row.get::<_, i64>(0))
        .optional()?
        .is_some())
}

fn effect_exists(
    tx: &Transaction<'_>,
    table: &str,
    effect_id: EffectId,
) -> Result<bool, MemoryEvidenceError> {
    let sql = format!("SELECT 1 FROM {table} WHERE effect_id = ?1 LIMIT 1");
    Ok(tx
        .query_row(&sql, params![id_blob(effect_id.0)], |row| {
            row.get::<_, i64>(0)
        })
        .optional()?
        .is_some())
}

fn id_blob(value: u128) -> [u8; 16] {
    value.to_be_bytes()
}

fn vec_to_hash(value: Vec<u8>) -> Result<[u8; 32], MemoryEvidenceError> {
    value
        .try_into()
        .map_err(|_| MemoryEvidenceError::InvalidRecord("stored integrity hash"))
}

const fn reconciliation_state_code(state: MemoryReconciliationState) -> i64 {
    match state {
        MemoryReconciliationState::InSync => 1,
        MemoryReconciliationState::UserEditDetected => 2,
        MemoryReconciliationState::Conflict => 3,
        MemoryReconciliationState::Reconciled => 4,
        MemoryReconciliationState::Blocked => 5,
    }
}

const fn mutation_status_code(status: MemoryMutationStatus) -> i64 {
    match status {
        MemoryMutationStatus::Committed => 1,
        MemoryMutationStatus::Rejected => 2,
        MemoryMutationStatus::Failed => 3,
        MemoryMutationStatus::UnknownOutcome => 4,
    }
}

#[cfg(test)]
mod tests {
    use golam_core::memory::{
        ExpectedMemoryVersion, MemoryItemId, MemoryOperation, MemoryStoreId, MemoryVersionId,
        MemoryWriterId,
    };
    use golam_core::taint::{TaintLabel, TaintSet};

    use super::*;

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn prepared(effect_id: u128) -> PreparedMemoryMutationIntent {
        golam_core::memory::MemoryMutationIntent {
            operation: MemoryOperation::Update,
            item_ids: vec![MemoryItemId(digest(1))],
            expected_current_versions: vec![ExpectedMemoryVersion {
                item_id: MemoryItemId(digest(1)),
                expected_version: Some(MemoryVersionId(digest(2))),
            }],
            expected_markdown_target_identity_ref: digest(3),
            expected_markdown_content_digest: digest(4),
            expected_markdown_version: MemoryVersionId(digest(5)),
            memory_operational_store_ref: MemoryStoreId(digest(6)),
            candidate_ref: Some(MemoryCandidateId(digest(7))),
            kernel_authorization_ref: digest(8),
            promotion_authority_ref: digest(9),
            effect_id: EffectId(effect_id),
            reason_ref: digest(10),
            initiating_principal: PrincipalId::new("principal.local").unwrap(),
            created_at_unix_ms: 11,
        }
        .prepare()
        .unwrap()
    }

    #[test]
    fn prepared_intent_is_immutable_and_security_chained() {
        let mut store = MemoryEvidenceStore::open_in_memory().unwrap();
        let first = prepared(12);
        store.persist_prepared_intent(&first).unwrap();
        store.persist_prepared_intent(&first).unwrap();
        assert_eq!(store.security_chain_len().unwrap(), 1);

        let mut changed = first.intent().clone();
        changed.expected_markdown_content_digest = digest(99);
        let changed = changed.prepare().unwrap();
        assert!(matches!(
            store.persist_prepared_intent(&changed),
            Err(MemoryEvidenceError::ImmutableEvidenceMismatch(
                "PREPARED memory intent"
            ))
        ));
    }

    #[test]
    fn version_keeps_creator_writer_and_effect_attribution() {
        let mut store = MemoryEvidenceStore::open_in_memory().unwrap();
        store.persist_prepared_intent(&prepared(12)).unwrap();
        let version = MemoryVersion {
            item_id: MemoryItemId(digest(1)),
            version_id: MemoryVersionId(digest(20)),
            scope: golam_core::memory::MemoryScope::Project,
            canonical_markdown_ref: digest(21),
            content_digest: digest(22),
            provenance_refs: vec![digest(23)],
            taint_set: TaintSet::from_labels([TaintLabel::UserTrusted]),
            status: golam_core::memory::MemoryVersionStatus::Active,
            predecessor_versions: vec![MemoryVersionId(digest(2))],
            conflict_refs: vec![],
            promotion_evidence_ref: digest(24),
            created_by_principal: PrincipalId::new("principal.creator").unwrap(),
            committed_by_writer_identity: MemoryWriterId(digest(25)),
            mutation_effect_ref: EffectId(12),
            created_at_unix_ms: 13,
        };
        store.persist_version(&version, b"version-record").unwrap();
        assert_eq!(store.security_chain_len().unwrap(), 2);
    }

    #[test]
    fn unknown_outcome_requires_reconciliation_identity_and_prepared_digest() {
        let mut store = MemoryEvidenceStore::open_in_memory().unwrap();
        let prepared = prepared(12);
        let digest_bytes = BindingDigest::new(prepared.binding_digest());
        store.persist_prepared_intent(&prepared).unwrap();
        let mut outcome = MemoryMutationOutcome {
            effect_id: EffectId(12),
            mutation_intent_digest: digest_bytes,
            status: MemoryMutationStatus::UnknownOutcome,
            canonical_version_refs: vec![],
            authority_journal_readback_ref: None,
            markdown_readback_ref: None,
            memory_sqlite_readback_ref: None,
            reconciliation_ref: None,
            verification_refs: vec![],
            integrity_evidence_refs: vec![],
            terminal_at_unix_ms: 20,
        };
        assert!(matches!(
            store.persist_terminal_outcome(digest(30), &outcome, b"unknown"),
            Err(MemoryEvidenceError::InvalidRecord(_))
        ));
        outcome.reconciliation_ref = Some(digest(31));
        store
            .persist_terminal_outcome(digest(30), &outcome, b"unknown")
            .unwrap();
        assert_eq!(store.security_chain_len().unwrap(), 2);
    }
}
