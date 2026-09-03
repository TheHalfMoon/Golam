#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::memory::{MemoryCandidateId, MemoryStoreId};
use golam_core::memory_storage::{MEMORY_OPERATIONAL_SCHEMA_VERSION, MemoryLayout};
use golam_core::tool_request::{BindingDigest, PrincipalId};
use golam_core::{CanonicalEncoder, CoreError};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

const PROMOTION_OPERATIONAL_DOMAIN: &[u8] = b"golam:memory-promotion-operational:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PromotionOperationalEvidence<'a> {
    pub evidence_id: BindingDigest,
    pub candidate_id: MemoryCandidateId,
    pub promotion_authority_ref: BindingDigest,
    pub approving_principal: Option<&'a PrincipalId>,
    pub verifier_policy_ref: Option<BindingDigest>,
    pub authority_evidence_ref: BindingDigest,
    pub recorded_at_unix_ms: u64,
}

#[derive(Debug)]
pub enum MemoryPromotionOperationalError {
    Sqlite(rusqlite::Error),
    Core(CoreError),
    OperationalMetaMissing,
    StoreBindingMismatch,
    SchemaBindingMismatch,
    InvalidAuthorityMode,
    ImmutablePromotionMismatch,
    InvalidRecord(&'static str),
    IntegerOverflow,
}

impl fmt::Display for MemoryPromotionOperationalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "memory promotion operational sqlite error: {error}"),
            Self::Core(error) => write!(f, "memory promotion operational encoding error: {error}"),
            Self::OperationalMetaMissing => {
                f.write_str("memory promotion operational store is not initialized")
            }
            Self::StoreBindingMismatch => {
                f.write_str("memory promotion row does not bind the exact operational store")
            }
            Self::SchemaBindingMismatch => {
                f.write_str("memory promotion row does not bind the exact operational schema")
            }
            Self::InvalidAuthorityMode => f.write_str(
                "memory promotion evidence must bind exactly one attributable authority mode",
            ),
            Self::ImmutablePromotionMismatch => {
                f.write_str("memory promotion evidence identity collision")
            }
            Self::InvalidRecord(reason) => {
                write!(f, "invalid memory promotion operational record: {reason}")
            }
            Self::IntegerOverflow => {
                f.write_str("memory promotion operational integer conversion overflow")
            }
        }
    }
}

impl Error for MemoryPromotionOperationalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for MemoryPromotionOperationalError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for MemoryPromotionOperationalError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

pub struct MemoryPromotionOperationalStore {
    connection: Connection,
    store_id: MemoryStoreId,
    schema_ref: BindingDigest,
}

impl MemoryPromotionOperationalStore {
    pub fn open(layout: &MemoryLayout) -> Result<Self, MemoryPromotionOperationalError> {
        let connection = Connection::open(layout.operational_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; \
             PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        verify_operational_meta(&connection, layout.store_id())?;
        connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS memory_promotion_state (
                evidence_id BLOB PRIMARY KEY NOT NULL CHECK (length(evidence_id) = 32),
                store_ref BLOB NOT NULL CHECK (length(store_ref) = 32),
                schema_ref BLOB NOT NULL CHECK (length(schema_ref) = 32),
                candidate_id BLOB NOT NULL CHECK (length(candidate_id) = 32),
                promotion_authority_ref BLOB NOT NULL CHECK (length(promotion_authority_ref) = 32),
                approving_principal TEXT,
                verifier_policy_ref BLOB CHECK (verifier_policy_ref IS NULL OR length(verifier_policy_ref) = 32),
                authority_evidence_ref BLOB NOT NULL CHECK (length(authority_evidence_ref) = 32),
                recorded_at_unix_ms INTEGER NOT NULL,
                integrity_hash BLOB NOT NULL CHECK (length(integrity_hash) = 32),
                CHECK ((approving_principal IS NULL) != (verifier_policy_ref IS NULL))
            );
            "#,
        )?;
        Ok(Self {
            connection,
            store_id: layout.store_id(),
            schema_ref: layout.schema_ref(),
        })
    }

    pub fn record(
        &mut self,
        evidence: PromotionOperationalEvidence<'_>,
    ) -> Result<(), MemoryPromotionOperationalError> {
        validate_authority_mode(evidence)?;
        let integrity_hash = integrity_hash(self.store_id, self.schema_ref, evidence)?;
        let key = evidence.evidence_id.bytes().to_vec();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = tx
            .query_row(
                "SELECT integrity_hash FROM memory_promotion_state WHERE evidence_id = ?1",
                params![&key],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()?;
        if let Some(existing) = existing {
            if existing == integrity_hash.to_vec() {
                tx.commit()?;
                return Ok(());
            }
            return Err(MemoryPromotionOperationalError::ImmutablePromotionMismatch);
        }
        tx.execute(
            r#"INSERT INTO memory_promotion_state
               (evidence_id, store_ref, schema_ref, candidate_id, promotion_authority_ref,
                approving_principal, verifier_policy_ref, authority_evidence_ref,
                recorded_at_unix_ms, integrity_hash)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
            params![
                key,
                self.store_id.0.bytes().to_vec(),
                self.schema_ref.bytes().to_vec(),
                evidence.candidate_id.0.bytes().to_vec(),
                evidence.promotion_authority_ref.bytes().to_vec(),
                evidence.approving_principal.map(PrincipalId::as_str),
                evidence.verifier_policy_ref.map(|value| value.bytes().to_vec()),
                evidence.authority_evidence_ref.bytes().to_vec(),
                to_i64(evidence.recorded_at_unix_ms)?,
                integrity_hash.to_vec(),
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn require_exact(
        &self,
        evidence_id: BindingDigest,
        candidate_id: MemoryCandidateId,
    ) -> Result<(), MemoryPromotionOperationalError> {
        let row = self
            .connection
            .query_row(
                "SELECT store_ref, schema_ref, candidate_id FROM memory_promotion_state \
                 WHERE evidence_id = ?1",
                params![evidence_id.bytes().to_vec()],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, Vec<u8>>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                    ))
                },
            )
            .optional()?
            .ok_or(MemoryPromotionOperationalError::InvalidRecord(
                "promotion evidence is missing",
            ))?;
        if row.0 != self.store_id.0.bytes().to_vec() {
            return Err(MemoryPromotionOperationalError::StoreBindingMismatch);
        }
        if row.1 != self.schema_ref.bytes().to_vec() {
            return Err(MemoryPromotionOperationalError::SchemaBindingMismatch);
        }
        if row.2 != candidate_id.0.bytes().to_vec() {
            return Err(MemoryPromotionOperationalError::InvalidRecord(
                "promotion evidence candidate mismatch",
            ));
        }
        Ok(())
    }
}

fn verify_operational_meta(
    connection: &Connection,
    store_id: MemoryStoreId,
) -> Result<(), MemoryPromotionOperationalError> {
    let row = connection
        .query_row(
            "SELECT schema_version, store_ref FROM memory_operational_meta WHERE singleton = 1",
            [],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )
        .optional()
        .map_err(|error| match error {
            rusqlite::Error::SqliteFailure(_, Some(message))
                if message.contains("no such table: memory_operational_meta") =>
            {
                MemoryPromotionOperationalError::OperationalMetaMissing
            }
            other => MemoryPromotionOperationalError::Sqlite(other),
        })?
        .ok_or(MemoryPromotionOperationalError::OperationalMetaMissing)?;
    if row.0 != i64::from(MEMORY_OPERATIONAL_SCHEMA_VERSION) {
        return Err(MemoryPromotionOperationalError::SchemaBindingMismatch);
    }
    if row.1 != store_id.0.bytes().to_vec() {
        return Err(MemoryPromotionOperationalError::StoreBindingMismatch);
    }
    Ok(())
}

fn validate_authority_mode(
    evidence: PromotionOperationalEvidence<'_>,
) -> Result<(), MemoryPromotionOperationalError> {
    if evidence.approving_principal.is_some() == evidence.verifier_policy_ref.is_some() {
        return Err(MemoryPromotionOperationalError::InvalidAuthorityMode);
    }
    Ok(())
}

fn integrity_hash(
    store_id: MemoryStoreId,
    schema_ref: BindingDigest,
    evidence: PromotionOperationalEvidence<'_>,
) -> Result<[u8; 32], MemoryPromotionOperationalError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(PROMOTION_OPERATIONAL_DOMAIN)?;
    encoder.push_bytes(&evidence.evidence_id.bytes())?;
    encoder.push_bytes(&store_id.0.bytes())?;
    encoder.push_bytes(&schema_ref.bytes())?;
    encoder.push_bytes(&evidence.candidate_id.0.bytes())?;
    encoder.push_bytes(&evidence.promotion_authority_ref.bytes())?;
    match (evidence.approving_principal, evidence.verifier_policy_ref) {
        (Some(principal), None) => {
            encoder.push_u8(1);
            encoder.push_bytes(principal.as_str().as_bytes())?;
        }
        (None, Some(verifier)) => {
            encoder.push_u8(2);
            encoder.push_bytes(&verifier.bytes())?;
        }
        _ => return Err(MemoryPromotionOperationalError::InvalidAuthorityMode),
    }
    encoder.push_bytes(&evidence.authority_evidence_ref.bytes())?;
    encoder.push_u64(evidence.recorded_at_unix_ms);
    Ok(crate::payload_hash(&encoder.finish()))
}

fn to_i64(value: u64) -> Result<i64, MemoryPromotionOperationalError> {
    i64::try_from(value).map_err(|_| MemoryPromotionOperationalError::IntegerOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_operational::MemoryOperationalStore;
    use golam_core::paths::RuntimeLayout;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn runtime() -> RuntimeLayout {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-memory-promotion-operational-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    #[test]
    fn promotion_rows_bind_exact_store_schema_candidate_and_authority_mode() {
        let runtime = runtime();
        let layout = MemoryLayout::initialize(&runtime).unwrap();
        let _operational = MemoryOperationalStore::open(&layout).unwrap();
        let mut store = MemoryPromotionOperationalStore::open(&layout).unwrap();
        let principal = PrincipalId::new("owner:owner").unwrap();
        let evidence = PromotionOperationalEvidence {
            evidence_id: digest(1),
            candidate_id: MemoryCandidateId(digest(2)),
            promotion_authority_ref: digest(3),
            approving_principal: Some(&principal),
            verifier_policy_ref: None,
            authority_evidence_ref: digest(4),
            recorded_at_unix_ms: 5,
        };
        store.record(evidence).unwrap();
        store
            .require_exact(evidence.evidence_id, evidence.candidate_id)
            .unwrap();
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn model_or_dual_mode_cannot_be_stored_as_promotion_authority() {
        let runtime = runtime();
        let layout = MemoryLayout::initialize(&runtime).unwrap();
        let _operational = MemoryOperationalStore::open(&layout).unwrap();
        let mut store = MemoryPromotionOperationalStore::open(&layout).unwrap();
        let principal = PrincipalId::new("model:self").unwrap();
        let candidate = MemoryCandidateId(digest(10));
        let base = PromotionOperationalEvidence {
            evidence_id: digest(11),
            candidate_id: candidate,
            promotion_authority_ref: digest(12),
            approving_principal: None,
            verifier_policy_ref: None,
            authority_evidence_ref: digest(13),
            recorded_at_unix_ms: 14,
        };
        assert!(matches!(
            store.record(base),
            Err(MemoryPromotionOperationalError::InvalidAuthorityMode)
        ));
        assert!(matches!(
            store.record(PromotionOperationalEvidence {
                approving_principal: Some(&principal),
                verifier_policy_ref: Some(digest(15)),
                ..base
            }),
            Err(MemoryPromotionOperationalError::InvalidAuthorityMode)
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn evidence_identity_is_immutable() {
        let runtime = runtime();
        let layout = MemoryLayout::initialize(&runtime).unwrap();
        let _operational = MemoryOperationalStore::open(&layout).unwrap();
        let mut store = MemoryPromotionOperationalStore::open(&layout).unwrap();
        let principal = PrincipalId::new("owner:owner").unwrap();
        let first = PromotionOperationalEvidence {
            evidence_id: digest(20),
            candidate_id: MemoryCandidateId(digest(21)),
            promotion_authority_ref: digest(22),
            approving_principal: Some(&principal),
            verifier_policy_ref: None,
            authority_evidence_ref: digest(23),
            recorded_at_unix_ms: 24,
        };
        store.record(first).unwrap();
        assert!(matches!(
            store.record(PromotionOperationalEvidence {
                candidate_id: MemoryCandidateId(digest(99)),
                ..first
            }),
            Err(MemoryPromotionOperationalError::ImmutablePromotionMismatch)
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
