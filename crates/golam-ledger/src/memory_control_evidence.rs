#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::memory::{MemoryOperation, PreparedMemoryMutationIntent};
use golam_core::tool_request::BindingDigest;
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::memory_control_authority::QualifiedMemoryControlAuthority;
use crate::memory_evidence::MemoryEvidenceStore;

const CONTROL_EVIDENCE_DOMAIN: &[u8] = b"golam:memory-control-authority-evidence:v1";
const CONTROL_CHAIN_DOMAIN: &[u8] = b"golam:memory-control-authority-chain:v1";
const CONTROL_SCOPE_FACT: &[u8] = b"canonical-managed-memory-only;external-artifacts-not-mutated";

#[derive(Debug)]
pub enum MemoryControlEvidenceError {
    Sqlite(rusqlite::Error),
    Core(CoreError),
    BindingMismatch(&'static str),
    MissingPreparedEffect(EffectId),
    ImmutableEvidenceMismatch,
    InvalidStoredRecord(&'static str),
    ChainIntegrity,
    IntegerOverflow,
}

impl fmt::Display for MemoryControlEvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "memory control evidence sqlite error: {error}"),
            Self::Core(error) => write!(f, "memory control evidence encoding error: {error}"),
            Self::BindingMismatch(field) => {
                write!(f, "memory control evidence binding mismatch: {field}")
            }
            Self::MissingPreparedEffect(effect_id) => write!(
                f,
                "memory control effect {} has no durable PREPARED intent",
                effect_id.0
            ),
            Self::ImmutableEvidenceMismatch => {
                f.write_str("memory control authority evidence identity collision")
            }
            Self::InvalidStoredRecord(reason) => {
                write!(f, "invalid stored memory control evidence: {reason}")
            }
            Self::ChainIntegrity => {
                f.write_str("memory control authority evidence chain integrity failed")
            }
            Self::IntegerOverflow => f.write_str("memory control evidence integer overflow"),
        }
    }
}

impl Error for MemoryControlEvidenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for MemoryControlEvidenceError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for MemoryControlEvidenceError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

pub struct MemoryControlEvidenceStore {
    connection: Connection,
}

impl MemoryControlEvidenceStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, MemoryControlEvidenceError> {
        drop(
            MemoryEvidenceStore::open(layout.authority_db_path()).map_err(|_| {
                MemoryControlEvidenceError::InvalidStoredRecord("memory evidence schema")
            })?,
        );
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; \
             PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS memory_control_authority_evidence (
                evidence_id BLOB PRIMARY KEY NOT NULL CHECK (length(evidence_id) = 32),
                effect_id BLOB UNIQUE NOT NULL CHECK (length(effect_id) = 16),
                intent_digest BLOB NOT NULL CHECK (length(intent_digest) = 32),
                operation INTEGER NOT NULL,
                item_id BLOB NOT NULL CHECK (length(item_id) = 32),
                expected_version BLOB NOT NULL CHECK (length(expected_version) = 32),
                kernel_authorization_ref BLOB NOT NULL CHECK (length(kernel_authorization_ref) = 32),
                mutation_authority_ref BLOB NOT NULL CHECK (length(mutation_authority_ref) = 32),
                authority_evidence_ref BLOB NOT NULL CHECK (length(authority_evidence_ref) = 32),
                approving_principal TEXT,
                verifier_policy_ref BLOB CHECK (verifier_policy_ref IS NULL OR length(verifier_policy_ref) = 32),
                scope_fact BLOB NOT NULL,
                record_bytes BLOB NOT NULL,
                integrity_hash BLOB UNIQUE NOT NULL CHECK (length(integrity_hash) = 32),
                FOREIGN KEY(effect_id) REFERENCES memory_prepared_intents(effect_id),
                CHECK ((approving_principal IS NULL) != (verifier_policy_ref IS NULL))
            );
            CREATE TABLE IF NOT EXISTS memory_control_authority_chain (
                sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                evidence_id BLOB UNIQUE NOT NULL CHECK (length(evidence_id) = 32),
                payload_hash BLOB NOT NULL CHECK (length(payload_hash) = 32),
                previous_integrity_hash BLOB,
                integrity_hash BLOB UNIQUE NOT NULL CHECK (length(integrity_hash) = 32),
                FOREIGN KEY(evidence_id) REFERENCES memory_control_authority_evidence(evidence_id)
            );
            "#,
        )?;
        Ok(Self { connection })
    }

    pub fn persist(
        &mut self,
        prepared: &PreparedMemoryMutationIntent,
        authority: &QualifiedMemoryControlAuthority,
    ) -> Result<BindingDigest, MemoryControlEvidenceError> {
        validate_binding(prepared, authority)?;
        let intent = prepared.intent();
        let intent_digest = BindingDigest::new(prepared.binding_digest());
        let prepared_digest = self
            .connection
            .query_row(
                "SELECT intent_digest FROM memory_prepared_intents WHERE effect_id = ?1",
                params![intent.effect_id.0.to_be_bytes().to_vec()],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()?
            .ok_or(MemoryControlEvidenceError::MissingPreparedEffect(
                intent.effect_id,
            ))?;
        if prepared_digest.as_slice() != intent_digest.bytes() {
            return Err(MemoryControlEvidenceError::BindingMismatch(
                "durable PREPARED intent digest",
            ));
        }

        let record_bytes = authority.record_bytes();
        if record_bytes.is_empty()
            || authority.approving_principal().is_some()
                == authority.verifier_policy_ref().is_some()
        {
            return Err(MemoryControlEvidenceError::InvalidStoredRecord(
                "authority mode or record bytes",
            ));
        }
        let evidence_id = authority.evidence_id();
        let integrity_hash = control_integrity_hash(prepared, authority)?;
        let target = authority.target();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let chain_head = verify_chain(&tx)?;

        let existing = tx
            .query_row(
                "SELECT integrity_hash FROM memory_control_authority_evidence WHERE evidence_id = ?1",
                params![evidence_id.bytes().to_vec()],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()?;
        if let Some(existing) = existing {
            if existing.as_slice() == integrity_hash {
                tx.commit()?;
                return Ok(evidence_id);
            }
            return Err(MemoryControlEvidenceError::ImmutableEvidenceMismatch);
        }
        let effect_collision = tx
            .query_row(
                "SELECT 1 FROM memory_control_authority_evidence WHERE effect_id = ?1 LIMIT 1",
                params![intent.effect_id.0.to_be_bytes().to_vec()],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if effect_collision {
            return Err(MemoryControlEvidenceError::ImmutableEvidenceMismatch);
        }

        tx.execute(
            r#"INSERT INTO memory_control_authority_evidence
               (evidence_id, effect_id, intent_digest, operation, item_id, expected_version,
                kernel_authorization_ref, mutation_authority_ref, authority_evidence_ref,
                approving_principal, verifier_policy_ref, scope_fact, record_bytes, integrity_hash)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)"#,
            params![
                evidence_id.bytes().to_vec(),
                intent.effect_id.0.to_be_bytes().to_vec(),
                intent_digest.bytes().to_vec(),
                operation_code(target.operation),
                target.item_id.0.bytes().to_vec(),
                target.expected_version.0.bytes().to_vec(),
                authority.kernel_authorization_ref().bytes().to_vec(),
                authority.mutation_authority_ref().bytes().to_vec(),
                authority.authority_evidence_ref().bytes().to_vec(),
                authority.approving_principal().map(|value| value.as_str()),
                authority
                    .verifier_policy_ref()
                    .map(|value| value.bytes().to_vec()),
                CONTROL_SCOPE_FACT,
                record_bytes,
                integrity_hash.to_vec(),
            ],
        )?;
        let chain_hash = chain_integrity_hash(evidence_id, integrity_hash, chain_head)?;
        tx.execute(
            r#"INSERT INTO memory_control_authority_chain
               (evidence_id, payload_hash, previous_integrity_hash, integrity_hash)
               VALUES (?1, ?2, ?3, ?4)"#,
            params![
                evidence_id.bytes().to_vec(),
                integrity_hash.to_vec(),
                chain_head.map(|value| value.to_vec()),
                chain_hash.to_vec(),
            ],
        )?;
        tx.commit()?;
        Ok(evidence_id)
    }
}

fn validate_binding(
    prepared: &PreparedMemoryMutationIntent,
    authority: &QualifiedMemoryControlAuthority,
) -> Result<(), MemoryControlEvidenceError> {
    let intent = prepared.intent();
    let target = authority.target();
    if authority.effect_id() != intent.effect_id {
        return Err(MemoryControlEvidenceError::BindingMismatch(
            "effect identity",
        ));
    }
    if intent.candidate_ref.is_some() {
        return Err(MemoryControlEvidenceError::BindingMismatch(
            "candidate-less operation",
        ));
    }
    if intent.operation != target.operation {
        return Err(MemoryControlEvidenceError::BindingMismatch("operation"));
    }
    if intent.item_ids.len() != 1 || intent.item_ids[0] != target.item_id {
        return Err(MemoryControlEvidenceError::BindingMismatch("item identity"));
    }
    if intent.expected_current_versions.len() != 1
        || intent.expected_current_versions[0].item_id != target.item_id
        || intent.expected_current_versions[0].expected_version != Some(target.expected_version)
    {
        return Err(MemoryControlEvidenceError::BindingMismatch(
            "expected current version",
        ));
    }
    if intent.kernel_authorization_ref != authority.kernel_authorization_ref() {
        return Err(MemoryControlEvidenceError::BindingMismatch(
            "Kernel authorization",
        ));
    }
    if intent.promotion_authority_ref != authority.mutation_authority_ref() {
        return Err(MemoryControlEvidenceError::BindingMismatch(
            "mutation authority",
        ));
    }
    Ok(())
}

fn control_integrity_hash(
    prepared: &PreparedMemoryMutationIntent,
    authority: &QualifiedMemoryControlAuthority,
) -> Result<[u8; 32], MemoryControlEvidenceError> {
    let target = authority.target();
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(CONTROL_EVIDENCE_DOMAIN)?;
    encoder.push_bytes(&authority.evidence_id().bytes())?;
    encoder.push_u128(authority.effect_id().0);
    encoder.push_bytes(&prepared.binding_digest())?;
    encoder.push_u8(
        u8::try_from(operation_code(target.operation))
            .map_err(|_| MemoryControlEvidenceError::IntegerOverflow)?,
    );
    encoder.push_bytes(&target.item_id.0.bytes())?;
    encoder.push_bytes(&target.expected_version.0.bytes())?;
    encoder.push_bytes(&authority.kernel_authorization_ref().bytes())?;
    encoder.push_bytes(&authority.mutation_authority_ref().bytes())?;
    encoder.push_bytes(&authority.authority_evidence_ref().bytes())?;
    match (
        authority.approving_principal(),
        authority.verifier_policy_ref(),
    ) {
        (Some(principal), None) => {
            encoder.push_u8(1);
            encoder.push_bytes(principal.as_str().as_bytes())?;
        }
        (None, Some(verifier)) => {
            encoder.push_u8(2);
            encoder.push_bytes(&verifier.bytes())?;
        }
        _ => {
            return Err(MemoryControlEvidenceError::InvalidStoredRecord(
                "authority mode",
            ));
        }
    }
    encoder.push_bytes(CONTROL_SCOPE_FACT)?;
    encoder.push_bytes(authority.record_bytes())?;
    Ok(crate::payload_hash(&encoder.finish()))
}

fn verify_chain(tx: &Transaction<'_>) -> Result<Option<[u8; 32]>, MemoryControlEvidenceError> {
    let mut statement = tx.prepare(
        "SELECT evidence_id, payload_hash, previous_integrity_hash, integrity_hash \
         FROM memory_control_authority_chain ORDER BY sequence ASC",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, Vec<u8>>(0)?,
            row.get::<_, Vec<u8>>(1)?,
            row.get::<_, Option<Vec<u8>>>(2)?,
            row.get::<_, Vec<u8>>(3)?,
        ))
    })?;
    let mut previous = None;
    for row in rows {
        let (evidence_id, payload_hash, stored_previous, stored_hash) = row?;
        let evidence_id = hash32(evidence_id, "chain evidence identity")?;
        let payload_hash = hash32(payload_hash, "chain payload hash")?;
        let stored_previous = stored_previous
            .map(|value| hash32(value, "chain previous hash"))
            .transpose()?;
        let stored_hash = hash32(stored_hash, "chain integrity hash")?;
        if stored_previous != previous {
            return Err(MemoryControlEvidenceError::ChainIntegrity);
        }
        let expected =
            chain_integrity_hash(BindingDigest::new(evidence_id), payload_hash, previous)?;
        if expected != stored_hash {
            return Err(MemoryControlEvidenceError::ChainIntegrity);
        }
        previous = Some(stored_hash);
    }
    Ok(previous)
}

fn chain_integrity_hash(
    evidence_id: BindingDigest,
    payload_hash: [u8; 32],
    previous: Option<[u8; 32]>,
) -> Result<[u8; 32], MemoryControlEvidenceError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(CONTROL_CHAIN_DOMAIN)?;
    encoder.push_bytes(&evidence_id.bytes())?;
    encoder.push_bytes(&payload_hash)?;
    match previous {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(&value)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(crate::payload_hash(&encoder.finish()))
}

fn operation_code(operation: MemoryOperation) -> i64 {
    match operation {
        MemoryOperation::Expire => 1,
        MemoryOperation::Forget => 2,
        MemoryOperation::Redact => 3,
        _ => 0,
    }
}

fn hash32(value: Vec<u8>, reason: &'static str) -> Result<[u8; 32], MemoryControlEvidenceError> {
    value
        .try_into()
        .map_err(|_| MemoryControlEvidenceError::InvalidStoredRecord(reason))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_fact_never_claims_external_artifact_deletion() {
        assert_eq!(
            CONTROL_SCOPE_FACT,
            b"canonical-managed-memory-only;external-artifacts-not-mutated"
        );
        assert!(
            !CONTROL_SCOPE_FACT
                .windows(7)
                .any(|value| value == b"deleted")
        );
    }
}
