#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use golam_core::authority::AuthorityLayout;
use golam_core::memory::{
    MemoryItemId, MemoryScope, MemoryStoreId, MemoryVersion, MemoryVersionId,
    PreparedMemoryMutationIntent,
};
use golam_core::tool_request::BindingDigest;
use golam_core::{CanonicalEncoder, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::memory_evidence::{MemoryEvidenceError, MemoryEvidenceStore};

pub const MEMORY_MUTATION_ACTION: &str = "memory.mutate";
pub const MEMORY_MUTATION_RISK_CLASS: &str = "memory_mutation";
pub const MANAGED_MEMORY_HANDLER_ID: &str = "golam-managed-memory-writer";
pub const MANAGED_MEMORY_HANDLER_VERSION: &str = "1";

const MEMORY_SECURITY_CHAIN_DOMAIN: &[u8] = b"golam:memory-security-chain:v1";
const PREPARED_TARGET_DOMAIN: &[u8] = b"golam:managed-memory-prepared-target:v1";
const PREPARED_READBACK_DOMAIN: &[u8] = b"golam:managed-memory-prepared-readback:v1";
const PREPARED_RECORD_KIND: i64 = 1;
const PREPARED_TARGET_RECORD_KIND: i64 = 6;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedManagedMemoryTarget {
    pub effect_id: EffectId,
    pub intent_digest: BindingDigest,
    pub memory_store_ref: MemoryStoreId,
    pub item_id: MemoryItemId,
    pub scope: MemoryScope,
    pub version_id: MemoryVersionId,
    pub markdown_path: PathBuf,
    pub target_identity_ref: BindingDigest,
    pub expected_content_digest: BindingDigest,
    pub expected_markdown_version: MemoryVersionId,
}

#[derive(Debug)]
pub enum MemoryWriterAuthorityError {
    Evidence(MemoryEvidenceError),
    Sqlite(rusqlite::Error),
    EffectMissing(EffectId),
    EffectNotAuthorized(String),
    EffectBindingMismatch(&'static str),
    PreparedIntentCollision,
    PreparedTargetCollision,
    NonUnicodePath,
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
            Self::PreparedTargetCollision => {
                f.write_str("managed-memory PREPARED target identity collision")
            }
            Self::NonUnicodePath => {
                f.write_str("managed-memory canonical Markdown path is not UTF-8")
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
        drop(MemoryEvidenceStore::open(layout.authority_db_path())?);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; \
             PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS memory_prepared_targets (
                effect_id BLOB PRIMARY KEY NOT NULL CHECK (length(effect_id) = 16),
                intent_digest BLOB NOT NULL CHECK (length(intent_digest) = 32),
                memory_store_ref BLOB NOT NULL CHECK (length(memory_store_ref) = 32),
                item_id BLOB NOT NULL CHECK (length(item_id) = 32),
                scope INTEGER NOT NULL,
                version_id BLOB NOT NULL CHECK (length(version_id) = 32),
                markdown_path TEXT NOT NULL,
                target_identity_ref BLOB NOT NULL CHECK (length(target_identity_ref) = 32),
                expected_content_digest BLOB NOT NULL CHECK (length(expected_content_digest) = 32),
                expected_markdown_version_ref BLOB NOT NULL CHECK (length(expected_markdown_version_ref) = 32),
                record_bytes BLOB NOT NULL,
                integrity_hash BLOB NOT NULL CHECK (length(integrity_hash) = 32),
                FOREIGN KEY(effect_id) REFERENCES memory_prepared_intents(effect_id)
            );
            "#,
        )?;
        Ok(Self { connection })
    }

    pub fn prepare(
        &mut self,
        prepared: &PreparedMemoryMutationIntent,
        version: &MemoryVersion,
        markdown_path: &Path,
    ) -> Result<PreparedManagedMemoryAuthority, MemoryWriterAuthorityError> {
        version.validate().map_err(|_| {
            MemoryWriterAuthorityError::InvalidStoredRecord("memory version contract")
        })?;
        let intent = prepared.intent();
        if version.mutation_effect_ref != intent.effect_id
            || !intent.item_ids.contains(&version.item_id)
        {
            return Err(MemoryWriterAuthorityError::EffectBindingMismatch(
                "prepared target version",
            ));
        }
        let markdown_path = markdown_path
            .to_str()
            .ok_or(MemoryWriterAuthorityError::NonUnicodePath)?;
        let intent_digest = BindingDigest::new(prepared.binding_digest());
        let expected_resource = memory_mutation_resource(prepared);
        let canonical_bytes = intent.canonical_bytes().map_err(|_| {
            MemoryWriterAuthorityError::InvalidStoredRecord("intent canonical bytes")
        })?;
        let integrity_hash = crate::payload_hash(&canonical_bytes);
        let effect_blob = intent.effect_id.0.to_be_bytes().to_vec();
        let target_bytes = prepared_target_record_bytes(
            intent.effect_id,
            intent_digest,
            intent.memory_operational_store_ref,
            version.item_id,
            version.scope,
            version.version_id,
            markdown_path,
            intent.expected_markdown_target_identity_ref,
            intent.expected_markdown_content_digest,
            intent.expected_markdown_version,
        )?;
        let target_hash = crate::payload_hash(&target_bytes);

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

        let existing_intent = tx
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
        let intent_exists = match existing_intent {
            Some(existing)
                if existing.0.as_slice() == prepared.binding_digest()
                    && existing.1 == canonical_bytes
                    && existing.2.as_slice() == integrity_hash =>
            {
                true
            }
            Some(_) => return Err(MemoryWriterAuthorityError::PreparedIntentCollision),
            None => false,
        };

        let existing_target = tx
            .query_row(
                "SELECT record_bytes, integrity_hash FROM memory_prepared_targets \
                 WHERE effect_id = ?1",
                params![&effect_blob],
                |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
            )
            .optional()?;
        let target_exists = match existing_target {
            Some(existing)
                if existing.0 == target_bytes && existing.1.as_slice() == target_hash =>
            {
                true
            }
            Some(_) => return Err(MemoryWriterAuthorityError::PreparedTargetCollision),
            None => false,
        };

        if !intent_exists {
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
            append_security_chain(&tx, PREPARED_RECORD_KIND, &effect_blob, integrity_hash)?;
        }
        if !target_exists {
            tx.execute(
                r#"INSERT INTO memory_prepared_targets
                   (effect_id, intent_digest, memory_store_ref, item_id, scope, version_id,
                    markdown_path, target_identity_ref, expected_content_digest,
                    expected_markdown_version_ref, record_bytes, integrity_hash)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"#,
                params![
                    &effect_blob,
                    intent_digest.bytes().to_vec(),
                    intent.memory_operational_store_ref.0.bytes().to_vec(),
                    version.item_id.0.bytes().to_vec(),
                    scope_code(version.scope),
                    version.version_id.0.bytes().to_vec(),
                    markdown_path,
                    intent
                        .expected_markdown_target_identity_ref
                        .bytes()
                        .to_vec(),
                    intent.expected_markdown_content_digest.bytes().to_vec(),
                    intent.expected_markdown_version.0.bytes().to_vec(),
                    target_bytes,
                    target_hash.to_vec(),
                ],
            )?;
            append_security_chain(&tx, PREPARED_TARGET_RECORD_KIND, &effect_blob, target_hash)?;
        }
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
                "SELECT i.intent_digest, i.integrity_hash, t.integrity_hash \
                 FROM memory_prepared_intents i \
                 JOIN memory_prepared_targets t ON t.effect_id = i.effect_id \
                 WHERE i.effect_id = ?1",
                params![prepared.effect_id.0.to_be_bytes().to_vec()],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, Vec<u8>>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                    ))
                },
            )
            .optional()?
            .ok_or(MemoryWriterAuthorityError::EffectMissing(
                prepared.effect_id,
            ))?;
        if row.0.as_slice() != prepared.intent_digest.bytes() {
            return Err(MemoryWriterAuthorityError::PreparedIntentCollision);
        }
        let intent_hash = hash32(row.1, "prepared integrity hash")?;
        let target_hash = hash32(row.2, "prepared target integrity hash")?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(PREPARED_READBACK_DOMAIN).map_err(|_| {
            MemoryWriterAuthorityError::InvalidStoredRecord("prepared readback domain")
        })?;
        encoder.push_u128(prepared.effect_id.0);
        encoder
            .push_bytes(&prepared.intent_digest.bytes())
            .map_err(|_| MemoryWriterAuthorityError::InvalidStoredRecord("intent digest"))?;
        encoder.push_bytes(&intent_hash).map_err(|_| {
            MemoryWriterAuthorityError::InvalidStoredRecord("prepared integrity hash")
        })?;
        encoder.push_bytes(&target_hash).map_err(|_| {
            MemoryWriterAuthorityError::InvalidStoredRecord("target integrity hash")
        })?;
        Ok(BindingDigest::new(crate::payload_hash(&encoder.finish())))
    }

    pub fn prepared_target(
        &self,
        effect_id: EffectId,
    ) -> Result<Option<PreparedManagedMemoryTarget>, MemoryWriterAuthorityError> {
        let row = self
            .connection
            .query_row(
                "SELECT intent_digest, memory_store_ref, item_id, scope, version_id,
                        markdown_path, target_identity_ref, expected_content_digest,
                        expected_markdown_version_ref, record_bytes, integrity_hash
                 FROM memory_prepared_targets WHERE effect_id = ?1",
                params![effect_id.0.to_be_bytes().to_vec()],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, Vec<u8>>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, Vec<u8>>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, Vec<u8>>(6)?,
                        row.get::<_, Vec<u8>>(7)?,
                        row.get::<_, Vec<u8>>(8)?,
                        row.get::<_, Vec<u8>>(9)?,
                        row.get::<_, Vec<u8>>(10)?,
                    ))
                },
            )
            .optional()?;
        let Some(row) = row else {
            return Ok(None);
        };
        let target = PreparedManagedMemoryTarget {
            effect_id,
            intent_digest: BindingDigest::new(hash32(row.0, "target intent digest")?),
            memory_store_ref: MemoryStoreId(BindingDigest::new(hash32(
                row.1,
                "target memory store",
            )?)),
            item_id: MemoryItemId(BindingDigest::new(hash32(row.2, "target item id")?)),
            scope: scope_from_code(row.3)?,
            version_id: MemoryVersionId(BindingDigest::new(hash32(row.4, "target version id")?)),
            markdown_path: PathBuf::from(row.5),
            target_identity_ref: BindingDigest::new(hash32(row.6, "target identity ref")?),
            expected_content_digest: BindingDigest::new(hash32(
                row.7,
                "target expected content digest",
            )?),
            expected_markdown_version: MemoryVersionId(BindingDigest::new(hash32(
                row.8,
                "target expected Markdown version",
            )?)),
        };
        let path = target
            .markdown_path
            .to_str()
            .ok_or(MemoryWriterAuthorityError::NonUnicodePath)?;
        let expected_bytes = prepared_target_record_bytes(
            target.effect_id,
            target.intent_digest,
            target.memory_store_ref,
            target.item_id,
            target.scope,
            target.version_id,
            path,
            target.target_identity_ref,
            target.expected_content_digest,
            target.expected_markdown_version,
        )?;
        let expected_hash = crate::payload_hash(&expected_bytes);
        if row.9 != expected_bytes || row.10.as_slice() != expected_hash {
            return Err(MemoryWriterAuthorityError::PreparedTargetCollision);
        }
        Ok(Some(target))
    }
}

#[allow(clippy::too_many_arguments)]
fn prepared_target_record_bytes(
    effect_id: EffectId,
    intent_digest: BindingDigest,
    memory_store_ref: MemoryStoreId,
    item_id: MemoryItemId,
    scope: MemoryScope,
    version_id: MemoryVersionId,
    markdown_path: &str,
    target_identity_ref: BindingDigest,
    expected_content_digest: BindingDigest,
    expected_markdown_version: MemoryVersionId,
) -> Result<Vec<u8>, MemoryWriterAuthorityError> {
    let mut encoder = CanonicalEncoder::new();
    encoder
        .push_bytes(PREPARED_TARGET_DOMAIN)
        .map_err(|_| MemoryWriterAuthorityError::InvalidStoredRecord("prepared target domain"))?;
    encoder.push_u128(effect_id.0);
    for digest in [
        intent_digest,
        memory_store_ref.0,
        item_id.0,
        version_id.0,
        target_identity_ref,
        expected_content_digest,
        expected_markdown_version.0,
    ] {
        encoder.push_bytes(&digest.bytes()).map_err(|_| {
            MemoryWriterAuthorityError::InvalidStoredRecord("prepared target digest")
        })?;
    }
    encoder.push_u8(match scope {
        MemoryScope::User => 1,
        MemoryScope::Project => 2,
    });
    encoder
        .push_bytes(markdown_path.as_bytes())
        .map_err(|_| MemoryWriterAuthorityError::InvalidStoredRecord("prepared target path"))?;
    Ok(encoder.finish())
}

fn append_security_chain(
    tx: &Transaction<'_>,
    record_kind: i64,
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
        u64::try_from(record_kind).map_err(|_| MemoryWriterAuthorityError::IntegerOverflow)?,
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
            record_kind,
            record_identity,
            payload_hash.to_vec(),
            previous.map(|hash| hash.to_vec()),
            integrity_hash.to_vec(),
        ],
    )?;
    Ok(())
}

fn scope_code(scope: MemoryScope) -> i64 {
    match scope {
        MemoryScope::User => 1,
        MemoryScope::Project => 2,
    }
}

fn scope_from_code(value: i64) -> Result<MemoryScope, MemoryWriterAuthorityError> {
    match value {
        1 => Ok(MemoryScope::User),
        2 => Ok(MemoryScope::Project),
        _ => Err(MemoryWriterAuthorityError::InvalidStoredRecord(
            "prepared target scope",
        )),
    }
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

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    #[test]
    fn resource_binds_entire_prepared_intent_digest() {
        assert_eq!(MEMORY_MUTATION_ACTION, "memory.mutate");
        assert_eq!(MANAGED_MEMORY_HANDLER_VERSION, "1");
    }

    #[test]
    fn prepared_target_record_binds_canonical_path() {
        let common = (
            EffectId(1),
            digest(2),
            MemoryStoreId(digest(3)),
            MemoryItemId(digest(4)),
            MemoryScope::Project,
            MemoryVersionId(digest(5)),
            digest(6),
            digest(7),
            MemoryVersionId(digest(8)),
        );
        let first = prepared_target_record_bytes(
            common.0,
            common.1,
            common.2,
            common.3,
            common.4,
            common.5,
            "memory/a.md",
            common.6,
            common.7,
            common.8,
        )
        .unwrap();
        let second = prepared_target_record_bytes(
            common.0,
            common.1,
            common.2,
            common.3,
            common.4,
            common.5,
            "memory/b.md",
            common.6,
            common.7,
            common.8,
        )
        .unwrap();
        assert_ne!(first, second);
    }
}
