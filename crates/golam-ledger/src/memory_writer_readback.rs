#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::Path;

use golam_core::memory::{MemoryVersion, PreparedMemoryMutationIntent};
use golam_core::memory_storage::MemoryLayout;
use golam_core::tool_request::BindingDigest;
use golam_core::{CanonicalEncoder, CoreError};
use rusqlite::{Connection, OptionalExtension, params};

const MEMORY_SQLITE_READBACK_DOMAIN: &[u8] = b"golam:memory-sqlite-readback:v1";

#[derive(Debug)]
pub enum MemoryWriterReadbackError {
    Sqlite(rusqlite::Error),
    Core(CoreError),
    MissingEffectState,
    MissingVersion,
    BindingMismatch(&'static str),
    NonUnicodePath,
}

impl fmt::Display for MemoryWriterReadbackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "managed-memory SQLite readback failed: {error}"),
            Self::Core(error) => write!(f, "managed-memory SQLite readback encoding failed: {error}"),
            Self::MissingEffectState => f.write_str("managed-memory SQLite effect state is missing"),
            Self::MissingVersion => f.write_str("managed-memory SQLite version row is missing"),
            Self::BindingMismatch(field) => {
                write!(f, "managed-memory SQLite readback binding mismatch: {field}")
            }
            Self::NonUnicodePath => f.write_str("managed-memory Markdown path is not UTF-8"),
        }
    }
}

impl Error for MemoryWriterReadbackError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for MemoryWriterReadbackError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for MemoryWriterReadbackError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

pub fn verify_memory_sqlite_readback(
    layout: &MemoryLayout,
    prepared: &PreparedMemoryMutationIntent,
    version: &MemoryVersion,
    markdown_path: &Path,
) -> Result<BindingDigest, MemoryWriterReadbackError> {
    let connection = Connection::open(layout.operational_db_path())?;
    connection.execute_batch("PRAGMA query_only = ON; PRAGMA busy_timeout = 5000;")?;
    let intent = prepared.intent();
    let intent_digest = BindingDigest::new(prepared.binding_digest());
    let path = markdown_path
        .to_str()
        .ok_or(MemoryWriterReadbackError::NonUnicodePath)?;

    let effect = connection
        .query_row(
            "SELECT store_ref, intent_digest FROM memory_effect_state WHERE effect_id = ?1",
            params![intent.effect_id.0.to_be_bytes().to_vec()],
            |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?
        .ok_or(MemoryWriterReadbackError::MissingEffectState)?;
    require_equal(
        effect.0,
        layout.store_id().0.bytes(),
        "effect store identity",
    )?;
    require_equal(effect.1, intent_digest.bytes(), "effect intent digest")?;

    let row = connection
        .query_row(
            "SELECT store_ref, item_id, markdown_path, content_digest, promotion_evidence_ref, \
                    created_by_principal, writer_id, effect_id, intent_digest \
             FROM memory_versions WHERE version_id = ?1",
            params![version.version_id.0.bytes().to_vec()],
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
                ))
            },
        )
        .optional()?
        .ok_or(MemoryWriterReadbackError::MissingVersion)?;
    require_equal(row.0, layout.store_id().0.bytes(), "version store identity")?;
    require_equal(row.1, version.item_id.0.bytes(), "item identity")?;
    if row.2 != path {
        return Err(MemoryWriterReadbackError::BindingMismatch("Markdown path"));
    }
    require_equal(row.3, version.content_digest.bytes(), "content digest")?;
    require_equal(
        row.4,
        version.promotion_evidence_ref.bytes(),
        "promotion evidence",
    )?;
    if row.5 != version.created_by_principal.as_str() {
        return Err(MemoryWriterReadbackError::BindingMismatch(
            "creating principal",
        ));
    }
    require_equal(
        row.6,
        version.committed_by_writer_identity.0.bytes(),
        "writer identity",
    )?;
    require_equal(
        row.7,
        version.mutation_effect_ref.0.to_be_bytes(),
        "effect identity",
    )?;
    require_equal(row.8, intent_digest.bytes(), "version intent digest")?;

    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(MEMORY_SQLITE_READBACK_DOMAIN)?;
    encoder.push_bytes(&layout.store_id().0.bytes())?;
    encoder.push_bytes(&layout.schema_ref().bytes())?;
    encoder.push_bytes(&intent_digest.bytes())?;
    encoder.push_u128(intent.effect_id.0);
    encoder.push_bytes(&version.version_id.0.bytes())?;
    encoder.push_bytes(&version.content_digest.bytes())?;
    encoder.push_bytes(path.as_bytes())?;
    Ok(BindingDigest::new(crate::payload_hash(&encoder.finish())))
}

fn require_equal<const N: usize>(
    stored: Vec<u8>,
    expected: [u8; N],
    field: &'static str,
) -> Result<(), MemoryWriterReadbackError> {
    if stored.as_slice() != expected {
        return Err(MemoryWriterReadbackError::BindingMismatch(field));
    }
    Ok(())
}
