#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::Path;

use golam_core::{CanonicalEncoder, CoreError, EffectId};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

const INTENT_DOMAIN: &[u8] = b"golam:tool-mutation-intent-evidence:v1";
const RECEIPT_DOMAIN: &[u8] = b"golam:tool-mutation-receipt-evidence:v1";
const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_RESOURCE_BYTES: usize = 16 * 1024;
const MAX_EVIDENCE_BYTES: usize = 128 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolMutationVerifiedStatus {
    Succeeded,
    Failed,
}

impl ToolMutationVerifiedStatus {
    const fn code(self) -> i64 {
        match self {
            Self::Succeeded => 1,
            Self::Failed => 2,
        }
    }

    fn from_code(value: i64) -> Result<Self, ToolMutationEvidenceError> {
        match value {
            1 => Ok(Self::Succeeded),
            2 => Ok(Self::Failed),
            _ => Err(ToolMutationEvidenceError::InvalidStoredRecord),
        }
    }
}

pub struct RecordToolMutationIntent<'a> {
    pub effect_id: EffectId,
    pub action: &'a str,
    pub resource: &'a str,
    pub preconditions_hash: [u8; 32],
    pub payload_hash: [u8; 32],
    pub provider_id: &'a str,
    pub intent_bytes: &'a [u8],
}

pub struct RecordToolMutationReceipt<'a> {
    pub effect_id: EffectId,
    pub action: &'a str,
    pub resource: &'a str,
    pub preconditions_hash: [u8; 32],
    pub payload_hash: [u8; 32],
    pub provider_id: &'a str,
    pub verified_status: ToolMutationVerifiedStatus,
    pub receipt_bytes: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredToolMutationEvidence {
    pub effect_id: EffectId,
    pub action: String,
    pub resource: String,
    pub preconditions_hash: [u8; 32],
    pub payload_hash: [u8; 32],
    pub provider_id: String,
    pub intent_bytes: Vec<u8>,
    pub intent_integrity_hash: [u8; 32],
    pub verified_status: Option<ToolMutationVerifiedStatus>,
    pub receipt_bytes: Option<Vec<u8>>,
    pub receipt_integrity_hash: Option<[u8; 32]>,
}

#[derive(Debug)]
pub enum ToolMutationEvidenceError {
    Sqlite(rusqlite::Error),
    Core(CoreError),
    InvalidMetadata,
    EvidenceTooLarge,
    ImmutableIntentMismatch(EffectId),
    MissingIntent(EffectId),
    ReceiptBindingMismatch(EffectId),
    ReceiptAlreadyRecorded(EffectId),
    InvalidStoredRecord,
}

impl fmt::Display for ToolMutationEvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "tool mutation evidence sqlite error: {error}"),
            Self::Core(error) => write!(f, "tool mutation evidence encoding error: {error}"),
            Self::InvalidMetadata => f.write_str("tool mutation evidence metadata is invalid"),
            Self::EvidenceTooLarge => f.write_str("tool mutation evidence exceeds bounded size"),
            Self::ImmutableIntentMismatch(effect_id) => write!(
                f,
                "tool mutation intent is immutable for effect {}",
                effect_id.0
            ),
            Self::MissingIntent(effect_id) => {
                write!(
                    f,
                    "tool mutation intent is missing for effect {}",
                    effect_id.0
                )
            }
            Self::ReceiptBindingMismatch(effect_id) => write!(
                f,
                "tool mutation receipt does not bind the prepared intent for effect {}",
                effect_id.0
            ),
            Self::ReceiptAlreadyRecorded(effect_id) => write!(
                f,
                "tool mutation receipt is immutable for effect {}",
                effect_id.0
            ),
            Self::InvalidStoredRecord => {
                f.write_str("stored tool mutation evidence is malformed or integrity-invalid")
            }
        }
    }
}

impl Error for ToolMutationEvidenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for ToolMutationEvidenceError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for ToolMutationEvidenceError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

pub struct ToolMutationEvidenceStore {
    connection: Connection,
}

impl ToolMutationEvidenceStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ToolMutationEvidenceError> {
        Self::initialize(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self, ToolMutationEvidenceError> {
        Self::initialize(Connection::open_in_memory()?)
    }

    fn initialize(connection: Connection) -> Result<Self, ToolMutationEvidenceError> {
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; \
             PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000; \
             CREATE TABLE IF NOT EXISTS tool_mutation_evidence (\
               effect_id BLOB PRIMARY KEY NOT NULL CHECK (length(effect_id) = 16),\
               action TEXT NOT NULL,\
               resource TEXT NOT NULL,\
               preconditions_hash BLOB NOT NULL CHECK (length(preconditions_hash) = 32),\
               payload_hash BLOB NOT NULL CHECK (length(payload_hash) = 32),\
               provider_id TEXT NOT NULL,\
               intent_bytes BLOB NOT NULL,\
               intent_integrity_hash BLOB NOT NULL CHECK (length(intent_integrity_hash) = 32),\
               verified_status INTEGER,\
               receipt_bytes BLOB,\
               receipt_integrity_hash BLOB CHECK (receipt_integrity_hash IS NULL OR length(receipt_integrity_hash) = 32),\
               CHECK ((verified_status IS NULL) = (receipt_bytes IS NULL)),\
               CHECK ((receipt_bytes IS NULL) = (receipt_integrity_hash IS NULL))\
             );",
        )?;
        Ok(Self { connection })
    }

    pub fn record_intent(
        &mut self,
        input: RecordToolMutationIntent<'_>,
    ) -> Result<[u8; 32], ToolMutationEvidenceError> {
        validate_binding(
            input.action,
            input.resource,
            input.provider_id,
            input.intent_bytes,
        )?;
        let integrity_hash = intent_integrity_hash(&input)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed = tx.execute(
            r#"INSERT INTO tool_mutation_evidence
               (effect_id, action, resource, preconditions_hash, payload_hash, provider_id,
                intent_bytes, intent_integrity_hash, verified_status, receipt_bytes,
                receipt_integrity_hash)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, NULL, NULL)
               ON CONFLICT(effect_id) DO UPDATE SET effect_id = excluded.effect_id
               WHERE tool_mutation_evidence.action = excluded.action
                 AND tool_mutation_evidence.resource = excluded.resource
                 AND tool_mutation_evidence.preconditions_hash = excluded.preconditions_hash
                 AND tool_mutation_evidence.payload_hash = excluded.payload_hash
                 AND tool_mutation_evidence.provider_id = excluded.provider_id
                 AND tool_mutation_evidence.intent_bytes = excluded.intent_bytes
                 AND tool_mutation_evidence.intent_integrity_hash = excluded.intent_integrity_hash"#,
            params![
                input.effect_id.0.to_be_bytes().to_vec(),
                input.action,
                input.resource,
                input.preconditions_hash.to_vec(),
                input.payload_hash.to_vec(),
                input.provider_id,
                input.intent_bytes,
                integrity_hash.to_vec(),
            ],
        )?;
        if changed != 1 {
            return Err(ToolMutationEvidenceError::ImmutableIntentMismatch(
                input.effect_id,
            ));
        }
        tx.commit()?;
        Ok(integrity_hash)
    }

    pub fn record_verified_receipt(
        &mut self,
        input: RecordToolMutationReceipt<'_>,
    ) -> Result<[u8; 32], ToolMutationEvidenceError> {
        validate_binding(
            input.action,
            input.resource,
            input.provider_id,
            input.receipt_bytes,
        )?;
        let existing = self
            .load(input.effect_id)?
            .ok_or(ToolMutationEvidenceError::MissingIntent(input.effect_id))?;
        if existing.action != input.action
            || existing.resource != input.resource
            || existing.preconditions_hash != input.preconditions_hash
            || existing.payload_hash != input.payload_hash
            || existing.provider_id != input.provider_id
        {
            return Err(ToolMutationEvidenceError::ReceiptBindingMismatch(
                input.effect_id,
            ));
        }
        let integrity_hash = receipt_integrity_hash(&existing, &input)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current: (Option<i64>, Option<Vec<u8>>, Option<Vec<u8>>) = tx.query_row(
            "SELECT verified_status, receipt_bytes, receipt_integrity_hash \
             FROM tool_mutation_evidence WHERE effect_id = ?1",
            params![input.effect_id.0.to_be_bytes().to_vec()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        match current {
            (None, None, None) => {
                let changed = tx.execute(
                    "UPDATE tool_mutation_evidence SET verified_status = ?1, receipt_bytes = ?2, \
                     receipt_integrity_hash = ?3 WHERE effect_id = ?4 AND verified_status IS NULL",
                    params![
                        input.verified_status.code(),
                        input.receipt_bytes,
                        integrity_hash.to_vec(),
                        input.effect_id.0.to_be_bytes().to_vec(),
                    ],
                )?;
                if changed != 1 {
                    return Err(ToolMutationEvidenceError::ReceiptAlreadyRecorded(
                        input.effect_id,
                    ));
                }
            }
            (Some(status), Some(bytes), Some(hash))
                if status == input.verified_status.code()
                    && bytes == input.receipt_bytes
                    && hash.as_slice() == integrity_hash.as_slice() => {}
            _ => {
                return Err(ToolMutationEvidenceError::ReceiptAlreadyRecorded(
                    input.effect_id,
                ));
            }
        }
        tx.commit()?;
        Ok(integrity_hash)
    }

    pub fn load(
        &self,
        effect_id: EffectId,
    ) -> Result<Option<StoredToolMutationEvidence>, ToolMutationEvidenceError> {
        let raw = self
            .connection
            .query_row(
                "SELECT action, resource, preconditions_hash, payload_hash, provider_id, \
                 intent_bytes, intent_integrity_hash, verified_status, receipt_bytes, \
                 receipt_integrity_hash FROM tool_mutation_evidence WHERE effect_id = ?1",
                params![effect_id.0.to_be_bytes().to_vec()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                        row.get::<_, Vec<u8>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Vec<u8>>(5)?,
                        row.get::<_, Vec<u8>>(6)?,
                        row.get::<_, Option<i64>>(7)?,
                        row.get::<_, Option<Vec<u8>>>(8)?,
                        row.get::<_, Option<Vec<u8>>>(9)?,
                    ))
                },
            )
            .optional()?;
        let Some(raw) = raw else {
            return Ok(None);
        };
        let preconditions_hash = hash32(raw.2)?;
        let payload_hash = hash32(raw.3)?;
        let stored_intent_integrity_hash = hash32(raw.6)?;
        validate_binding(&raw.0, &raw.1, &raw.4, &raw.5)?;
        let status = raw
            .7
            .map(ToolMutationVerifiedStatus::from_code)
            .transpose()?;
        if status.is_some() != raw.8.is_some() || raw.8.is_some() != raw.9.is_some() {
            return Err(ToolMutationEvidenceError::InvalidStoredRecord);
        }
        let stored = StoredToolMutationEvidence {
            effect_id,
            action: raw.0,
            resource: raw.1,
            preconditions_hash,
            payload_hash,
            provider_id: raw.4,
            intent_bytes: raw.5,
            intent_integrity_hash: stored_intent_integrity_hash,
            verified_status: status,
            receipt_bytes: raw.8,
            receipt_integrity_hash: raw.9.map(hash32).transpose()?,
        };
        let expected_intent_hash = intent_integrity_hash(&RecordToolMutationIntent {
            effect_id,
            action: &stored.action,
            resource: &stored.resource,
            preconditions_hash: stored.preconditions_hash,
            payload_hash: stored.payload_hash,
            provider_id: &stored.provider_id,
            intent_bytes: &stored.intent_bytes,
        })?;
        if expected_intent_hash != stored.intent_integrity_hash {
            return Err(ToolMutationEvidenceError::InvalidStoredRecord);
        }
        if let (Some(verified_status), Some(receipt_bytes), Some(receipt_hash)) = (
            stored.verified_status,
            stored.receipt_bytes.as_deref(),
            stored.receipt_integrity_hash,
        ) {
            let expected_receipt_hash = receipt_integrity_hash(
                &stored,
                &RecordToolMutationReceipt {
                    effect_id,
                    action: &stored.action,
                    resource: &stored.resource,
                    preconditions_hash: stored.preconditions_hash,
                    payload_hash: stored.payload_hash,
                    provider_id: &stored.provider_id,
                    verified_status,
                    receipt_bytes,
                },
            )?;
            if expected_receipt_hash != receipt_hash {
                return Err(ToolMutationEvidenceError::InvalidStoredRecord);
            }
        }
        Ok(Some(stored))
    }
}

fn validate_binding(
    action: &str,
    resource: &str,
    provider_id: &str,
    evidence_bytes: &[u8],
) -> Result<(), ToolMutationEvidenceError> {
    if action.is_empty()
        || action.len() > MAX_IDENTIFIER_BYTES
        || provider_id.is_empty()
        || provider_id.len() > MAX_IDENTIFIER_BYTES
        || resource.is_empty()
        || resource.len() > MAX_RESOURCE_BYTES
    {
        return Err(ToolMutationEvidenceError::InvalidMetadata);
    }
    if evidence_bytes.is_empty() {
        return Err(ToolMutationEvidenceError::InvalidMetadata);
    }
    if evidence_bytes.len() > MAX_EVIDENCE_BYTES {
        return Err(ToolMutationEvidenceError::EvidenceTooLarge);
    }
    Ok(())
}

fn intent_integrity_hash(
    input: &RecordToolMutationIntent<'_>,
) -> Result<[u8; 32], ToolMutationEvidenceError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(INTENT_DOMAIN)?;
    encoder.push_u128(input.effect_id.0);
    encoder.push_bytes(input.action.as_bytes())?;
    encoder.push_bytes(input.resource.as_bytes())?;
    encoder.push_bytes(&input.preconditions_hash)?;
    encoder.push_bytes(&input.payload_hash)?;
    encoder.push_bytes(input.provider_id.as_bytes())?;
    encoder.push_bytes(input.intent_bytes)?;
    Ok(crate::payload_hash(&encoder.finish()))
}

fn receipt_integrity_hash(
    intent: &StoredToolMutationEvidence,
    input: &RecordToolMutationReceipt<'_>,
) -> Result<[u8; 32], ToolMutationEvidenceError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(RECEIPT_DOMAIN)?;
    encoder.push_u128(input.effect_id.0);
    encoder.push_bytes(&intent.intent_integrity_hash)?;
    encoder.push_bytes(input.action.as_bytes())?;
    encoder.push_bytes(input.resource.as_bytes())?;
    encoder.push_bytes(&input.preconditions_hash)?;
    encoder.push_bytes(&input.payload_hash)?;
    encoder.push_bytes(input.provider_id.as_bytes())?;
    encoder.push_u8(input.verified_status.code() as u8);
    encoder.push_bytes(input.receipt_bytes)?;
    Ok(crate::payload_hash(&encoder.finish()))
}

fn hash32(value: Vec<u8>) -> Result<[u8; 32], ToolMutationEvidenceError> {
    value
        .try_into()
        .map_err(|_| ToolMutationEvidenceError::InvalidStoredRecord)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn intent<'a>(effect_id: EffectId, bytes: &'a [u8]) -> RecordToolMutationIntent<'a> {
        RecordToolMutationIntent {
            effect_id,
            action: "git.branch.create",
            resource: "git-branch-create:candidate",
            preconditions_hash: [1; 32],
            payload_hash: [2; 32],
            provider_id: "golam-git-linux-v1",
            intent_bytes: bytes,
        }
    }

    #[test]
    fn exact_intent_and_verified_receipt_survive_reopen() {
        let root = std::env::temp_dir().join(format!(
            "golam-tool-mutation-evidence-{}-{}.sqlite",
            std::process::id(),
            1_u128
        ));
        let _ = std::fs::remove_file(&root);
        let effect_id = EffectId(10);
        let mut store = ToolMutationEvidenceStore::open(&root).unwrap();
        let intent_hash = store
            .record_intent(intent(effect_id, b"branch:candidate"))
            .unwrap();
        assert_ne!(intent_hash, [0; 32]);
        let receipt_hash = store
            .record_verified_receipt(RecordToolMutationReceipt {
                effect_id,
                action: "git.branch.create",
                resource: "git-branch-create:candidate",
                preconditions_hash: [1; 32],
                payload_hash: [2; 32],
                provider_id: "golam-git-linux-v1",
                verified_status: ToolMutationVerifiedStatus::Succeeded,
                receipt_bytes: b"verified-branch-ref",
            })
            .unwrap();
        drop(store);

        let store = ToolMutationEvidenceStore::open(&root).unwrap();
        let loaded = store.load(effect_id).unwrap().unwrap();
        assert_eq!(loaded.intent_bytes, b"branch:candidate");
        assert_eq!(
            loaded.verified_status,
            Some(ToolMutationVerifiedStatus::Succeeded)
        );
        assert_eq!(loaded.receipt_integrity_hash, Some(receipt_hash));
        drop(store);
        let _ = std::fs::remove_file(root);
    }

    #[test]
    fn intent_identity_is_immutable_and_receipt_cannot_rebind_effect() {
        let mut store = ToolMutationEvidenceStore::open_in_memory().unwrap();
        let effect_id = EffectId(11);
        store.record_intent(intent(effect_id, b"branch:a")).unwrap();
        assert!(matches!(
            store.record_intent(intent(effect_id, b"branch:b")),
            Err(ToolMutationEvidenceError::ImmutableIntentMismatch(id)) if id == effect_id
        ));
        assert!(matches!(
            store.record_verified_receipt(RecordToolMutationReceipt {
                effect_id,
                action: "git.branch.create",
                resource: "git-branch-create:other",
                preconditions_hash: [1; 32],
                payload_hash: [2; 32],
                provider_id: "golam-git-linux-v1",
                verified_status: ToolMutationVerifiedStatus::Succeeded,
                receipt_bytes: b"forged",
            }),
            Err(ToolMutationEvidenceError::ReceiptBindingMismatch(id)) if id == effect_id
        ));
    }

    #[test]
    fn receipt_is_one_shot_and_idempotent_only_for_exact_evidence() {
        let mut store = ToolMutationEvidenceStore::open_in_memory().unwrap();
        let effect_id = EffectId(12);
        store.record_intent(intent(effect_id, b"branch:a")).unwrap();
        let exact = RecordToolMutationReceipt {
            effect_id,
            action: "git.branch.create",
            resource: "git-branch-create:candidate",
            preconditions_hash: [1; 32],
            payload_hash: [2; 32],
            provider_id: "golam-git-linux-v1",
            verified_status: ToolMutationVerifiedStatus::Succeeded,
            receipt_bytes: b"verified",
        };
        let first = store.record_verified_receipt(exact).unwrap();
        let second = store
            .record_verified_receipt(RecordToolMutationReceipt {
                effect_id,
                action: "git.branch.create",
                resource: "git-branch-create:candidate",
                preconditions_hash: [1; 32],
                payload_hash: [2; 32],
                provider_id: "golam-git-linux-v1",
                verified_status: ToolMutationVerifiedStatus::Succeeded,
                receipt_bytes: b"verified",
            })
            .unwrap();
        assert_eq!(first, second);
        assert!(matches!(
            store.record_verified_receipt(RecordToolMutationReceipt {
                effect_id,
                action: "git.branch.create",
                resource: "git-branch-create:candidate",
                preconditions_hash: [1; 32],
                payload_hash: [2; 32],
                provider_id: "golam-git-linux-v1",
                verified_status: ToolMutationVerifiedStatus::Failed,
                receipt_bytes: b"different",
            }),
            Err(ToolMutationEvidenceError::ReceiptAlreadyRecorded(id)) if id == effect_id
        ));
    }
}
