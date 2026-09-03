#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::memory::PreparedMemoryMutationIntent;
use golam_core::tool_request::BindingDigest;
use golam_core::{CanonicalEncoder, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::memory_evidence::{MemoryEvidenceError, MemoryEvidenceStore};

pub const MEMORY_MUTATION_ACTION: &str = "memory.mutate";
pub const MEMORY_MUTATION_RISK_CLASS: &str = "memory_mutation";
pub const MANAGED_MEMORY_HANDLER_ID: &str = "golam-managed-memory-writer";
pub const MANAGED_MEMORY_HANDLER_VERSION: &str = "1";

const MEMORY_SECURITY_CHAIN_DOMAIN: &[u8] = b"golam:memory-security-chain:v1";
const PREPARED_RECORD_KIND: i64 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparedManagedMemoryAuthority {
    effect_id: EffectId,
    intent_digest: BindingDigest,
}

impl PreparedManagedMemoryAuthority {
    pub const fn effect_id(self) -> EffectId {
        self.effect_id
    }

    pub const fn intent_digest(self) -> BindingDigest {
        self.intent_digest
    }
}

#[derive(Debug)]
pub enum MemoryWriterAuthorityError {
    Evidence(MemoryEvidenceError),
    Sqlite(rusqlite::Error),
    EffectMissing(EffectId),
    EffectNotAuthorized(String),
    EffectBindingMismatch(&'static str),
    PreparedIntentCollision,
    InvalidStoredRecord(&'static str),
    IntegerOverflow,
}

impl fmt::Display for MemoryWriterAuthorityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Evidence(error) => write!(f, "managed-memory authority evidence error: {error}"),
            Self::Sqlite(error) => write!(f, "managed-memory authority sqlite error: {error}"),
            Self::EffectMissing(effect_id) => {
                write!(f, "managed-memory effect is missing: {}", effect_id.0)
            }
            Self::EffectNotAuthorized(state) => write!(
                f,
                "managed-memory effect is not in the authorized state: {state}"
            ),
            Self::EffectBindingMismatch(field) => {
                write!(f, "managed-memory effect does not bind the exact {field}")
            }
            Self::PreparedIntentCollision => {
                f.write_str("managed-memory PREPARED effect identity collision")
            }
            Self::InvalidStoredRecord(field) => {
                write!(f, "managed-memory authority record is invalid: {field}")
            }
            Self::IntegerOverflow => f.write_str("managed-memory authority sequence overflow"),
        }
    }
}

impl Error for MemoryWriterAuthorityError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Evidence(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            _ => None,
        }
    }
}

impl From<MemoryEvidenceError> for MemoryWriterAuthorityError {
    fn from(value: MemoryEvidenceError) -> Self {
        Self::Evidence(value)
    }
}

impl From<rusqlite::Error> for MemoryWriterAuthorityError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub fn memory_mutation_resource(prepared: &PreparedMemoryMutationIntent) -> String {
    format!("memory:managed:{}", encode_hex(&prepared.binding_digest()))
}

pub struct MemoryWriterAuthorityStore {
    connection: Connection,
}

impl MemoryWriterAuthorityStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, MemoryWriterAuthorityError> {
        // Initializes the Spec 005 memory evidence tables in the protected authority DB.
        drop(MemoryEvidenceStore::open(layout.authority_db_path())?);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; \
             PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn prepare(
        &mut self,
        prepared: &PreparedMemoryMutationIntent,
    ) -> Result<PreparedManagedMemoryAuthority, MemoryWriterAuthorityError> {
        let intent = prepared.intent();
        let intent_digest = BindingDigest::new(prepared.binding_digest());
        let expected_resource = memory_mutation_resource(prepared);
        let canonical_bytes = intent.canonical_bytes().map_err(|_| {
            MemoryWriterAuthorityError::InvalidStoredRecord("intent canonical bytes")
        })?;
        let integrity_hash = crate::payload_hash(&canonical_bytes);
        let effect_blob = intent.effect_id.0.to_be_bytes().to_vec();

        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let effect = tx
            .query_row(
                "SELECT i.requested_by, i.action, i.resource, i.risk_class, i.payload_hash, \
                        t.to_state \
                 FROM effect_intents i \
                 JOIN effect_transitions t ON t.effect_id = i.effect_id \
                 WHERE i.effect_id = ?1 ORDER BY t.global_seq DESC LIMIT 1",
                params![&effect_blob],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Vec<u8>>(4)?,
                        row.get::<_, String>(5)?,
                    ))
                },
            )
            .optional()?
            .ok_or(MemoryWriterAuthorityError::EffectMissing(intent.effect_id))?;

        if effect.5 != "authorized" {
            return Err(MemoryWriterAuthorityError::EffectNotAuthorized(effect.5));
        }
        if effect.0 != intent.initiating_principal.as_str() {
            return Err(MemoryWriterAuthorityError::EffectBindingMismatch(
                "initiating principal",
            ));
        }
        if effect.1 != MEMORY_MUTATION_ACTION {
            return Err(MemoryWriterAuthorityError::EffectBindingMismatch("action"));
        }
        if effect.2 != expected_resource {
            return Err(MemoryWriterAuthorityError::EffectBindingMismatch(
                "resource",
            ));
        }
        if effect.3 != MEMORY_MUTATION_RISK_CLASS {
            return Err(MemoryWriterAuthorityError::EffectBindingMismatch(
                "risk class",
            ));
        }
        if effect.4.as_slice() != prepared.binding_digest() {
            return Err(MemoryWriterAuthorityError::EffectBindingMismatch(
                "mutation-intent digest",
            ));
        }

        let existing = tx
            .query_row(
                "SELECT intent_digest, canonical_bytes, integrity_hash \
                 FROM memory_prepared_intents WHERE effect_id = ?1",
                params![&effect_blob],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, Vec<u8>>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                    ))
                },
            )
            .optional()?;
        if let Some(existing) = existing {
            if existing.0.as_slice() == prepared.binding_digest()
                && existing.1 == canonical_bytes
                && existing.2.as_slice() == integrity_hash
            {
                tx.commit()?;
                return Ok(PreparedManagedMemoryAuthority {
                    effect_id: intent.effect_id,
                    intent_digest,
                });
            }
            return Err(MemoryWriterAuthorityError::PreparedIntentCollision);
        }

        tx.execute(
            r#"INSERT INTO memory_prepared_intents
               (effect_id, intent_digest, memory_store_ref, initiating_principal,
                markdown_target_identity_ref, markdown_content_digest, markdown_version_ref,
                canonical_bytes, integrity_hash)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
            params![
                &effect_blob,
                prepared.binding_digest().to_vec(),
                intent.memory_operational_store_ref.0.bytes().to_vec(),
                intent.initiating_principal.as_str(),
                intent
                    .expected_markdown_target_identity_ref
                    .bytes()
                    .to_vec(),
                intent.expected_markdown_content_digest.bytes().to_vec(),
                intent.expected_markdown_version.0.bytes().to_vec(),
                canonical_bytes,
                integrity_hash.to_vec(),
            ],
        )?;
        append_security_chain(&tx, &effect_blob, integrity_hash)?;
        tx.commit()?;
        Ok(PreparedManagedMemoryAuthority {
            effect_id: intent.effect_id,
            intent_digest,
        })
    }

    pub fn readback_ref(
        &self,
        prepared: PreparedManagedMemoryAuthority,
    ) -> Result<BindingDigest, MemoryWriterAuthorityError> {
        let row = self
            .connection
            .query_row(
                "SELECT intent_digest, integrity_hash FROM memory_prepared_intents \
                 WHERE effect_id = ?1",
                params![prepared.effect_id.0.to_be_bytes().to_vec()],
                |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
            )
            .optional()?
            .ok_or(MemoryWriterAuthorityError::EffectMissing(
                prepared.effect_id,
            ))?;
        if row.0.as_slice() != prepared.intent_digest.bytes() {
            return Err(MemoryWriterAuthorityError::PreparedIntentCollision);
        }
        Ok(BindingDigest::new(hash32(
            row.1,
            "prepared integrity hash",
        )?))
    }
}

fn append_security_chain(
    tx: &Transaction<'_>,
    record_identity: &[u8],
    payload_hash: [u8; 32],
) -> Result<(), MemoryWriterAuthorityError> {
    let previous = tx
        .query_row(
            "SELECT integrity_hash FROM memory_security_chain ORDER BY sequence DESC LIMIT 1",
            [],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .optional()?
        .map(|value| hash32(value, "previous security-chain hash"))
        .transpose()?;
    let mut encoder = CanonicalEncoder::new();
    encoder
        .push_bytes(MEMORY_SECURITY_CHAIN_DOMAIN)
        .map_err(|_| MemoryWriterAuthorityError::InvalidStoredRecord("security-chain domain"))?;
    encoder.push_u64(
        u64::try_from(PREPARED_RECORD_KIND)
            .map_err(|_| MemoryWriterAuthorityError::IntegerOverflow)?,
    );
    encoder
        .push_bytes(record_identity)
        .map_err(|_| MemoryWriterAuthorityError::InvalidStoredRecord("security-chain identity"))?;
    encoder
        .push_bytes(&payload_hash)
        .map_err(|_| MemoryWriterAuthorityError::InvalidStoredRecord("security-chain payload"))?;
    match previous {
        Some(hash) => {
            encoder.push_u8(1);
            encoder.push_bytes(&hash).map_err(|_| {
                MemoryWriterAuthorityError::InvalidStoredRecord("security-chain previous")
            })?;
        }
        None => encoder.push_u8(0),
    }
    let integrity_hash = crate::payload_hash(&encoder.finish());
    tx.execute(
        r#"INSERT INTO memory_security_chain
           (record_kind, record_identity, payload_hash, previous_integrity_hash, integrity_hash)
           VALUES (?1, ?2, ?3, ?4, ?5)"#,
        params![
            PREPARED_RECORD_KIND,
            record_identity,
            payload_hash.to_vec(),
            previous.map(|hash| hash.to_vec()),
            integrity_hash.to_vec(),
        ],
    )?;
    Ok(())
}

fn hash32(value: Vec<u8>, field: &'static str) -> Result<[u8; 32], MemoryWriterAuthorityError> {
    value
        .try_into()
        .map_err(|_| MemoryWriterAuthorityError::InvalidStoredRecord(field))
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_binds_entire_prepared_intent_digest() {
        assert_eq!(MEMORY_MUTATION_ACTION, "memory.mutate");
        assert_eq!(MANAGED_MEMORY_HANDLER_VERSION, "1");
    }
}
