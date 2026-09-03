#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use golam_core::digest::sha256;
use golam_core::memory::{
    DerivativeIndexGeneration, DerivativeIndexStatus, MemoryDerivativeGenerationId, MemoryItemId,
    MemoryScope, MemoryVersionId, MemoryVersionStatus,
};
use golam_core::memory_markdown::{
    ManagedMarkdownDocument, ManagedMarkdownError, parse_managed_markdown,
};
use golam_core::memory_storage::MemoryLayout;
use golam_core::tool_request::BindingDigest;
use golam_core::{CanonicalEncoder, CoreError};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use crate::memory_operational::{MemoryOperationalError, MemoryOperationalStore};

const TEXT_METADATA_INDEX_KIND_DOMAIN: &[u8] = b"golam:memory-derivative:text-metadata-kind:v1";
const TEXT_METADATA_IMPLEMENTATION_DOMAIN: &[u8] =
    b"golam:memory-derivative:text-metadata-implementation:v1";
const CANONICAL_CUT_DOMAIN: &[u8] = b"golam:memory-derivative:canonical-cut:v1";
const GENERATION_DOMAIN: &[u8] = b"golam:memory-derivative:generation:v1";
const INDEX_FILE_DOMAIN: &[u8] = b"golam:memory-derivative:text-metadata-index:v1";
const INDEX_FILE_NAME: &str = "text-metadata-v1.idx";
const INDEX_TEMP_FILE_NAME: &str = "text-metadata-v1.idx.tmp";
const MAX_CANONICAL_MARKDOWN_BYTES: usize = 1024 * 1024;
const MAX_INDEX_ITEMS: usize = 4096;
const MAX_TOTAL_CANONICAL_BYTES: usize = 64 * 1024 * 1024;
const MAX_TERMS_PER_DOCUMENT: usize = 16 * 1024;
const MAX_UNIQUE_TERMS: usize = 262_144;
const MAX_TOTAL_POSTINGS: usize = 1_000_000;
const MAX_LEXICAL_TERM_BYTES: usize = 128;
const MAX_INDEX_TERM_BYTES: usize = 512;
const MAX_INDEX_BYTES: usize = 128 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryDerivativeIndexReceipt {
    pub generation: DerivativeIndexGeneration,
    pub index_digest: BindingDigest,
    pub entry_count: u64,
    pub term_count: u64,
    pub index_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedMemoryDerivativeIndex {
    pub receipt: MemoryDerivativeIndexReceipt,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub enum MemoryDerivativeError {
    Io(io::Error),
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Operational(MemoryOperationalError),
    Markdown(ManagedMarkdownError),
    InvalidRecord(&'static str),
    UnsafeCanonicalPath(PathBuf),
    UnsafeDerivativePath(PathBuf),
    CanonicalDigestMismatch(PathBuf),
    CanonicalAmbiguous,
    BoundExceeded(&'static str),
    DerivativeCorrupt,
    IntegerOverflow,
}

impl fmt::Display for MemoryDerivativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "memory derivative I/O failed: {error}"),
            Self::Sqlite(error) => write!(f, "memory derivative SQLite failed: {error}"),
            Self::Core(error) => write!(f, "memory derivative canonical encoding failed: {error}"),
            Self::Operational(error) => {
                write!(f, "memory derivative operational state failed: {error}")
            }
            Self::Markdown(error) => {
                write!(f, "memory derivative canonical Markdown failed: {error}")
            }
            Self::InvalidRecord(reason) => {
                write!(f, "invalid memory derivative source record: {reason}")
            }
            Self::UnsafeCanonicalPath(path) => write!(
                f,
                "memory derivative canonical source escapes the managed vault: {}",
                path.display()
            ),
            Self::UnsafeDerivativePath(path) => write!(
                f,
                "memory derivative path is not a regular private projection path: {}",
                path.display()
            ),
            Self::CanonicalDigestMismatch(path) => write!(
                f,
                "memory derivative canonical Markdown digest changed: {}",
                path.display()
            ),
            Self::CanonicalAmbiguous => f.write_str(
                "memory derivative rebuild is denied while canonical memory has UNKNOWN_OUTCOME",
            ),
            Self::BoundExceeded(bound) => write!(f, "memory derivative bound exceeded: {bound}"),
            Self::DerivativeCorrupt => {
                f.write_str("memory derivative index does not match the current canonical cut")
            }
            Self::IntegerOverflow => f.write_str("memory derivative integer conversion overflow"),
        }
    }
}

impl Error for MemoryDerivativeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::Operational(error) => Some(error),
            Self::Markdown(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for MemoryDerivativeError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<rusqlite::Error> for MemoryDerivativeError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for MemoryDerivativeError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<MemoryOperationalError> for MemoryDerivativeError {
    fn from(value: MemoryOperationalError) -> Self {
        Self::Operational(value)
    }
}

impl From<ManagedMarkdownError> for MemoryDerivativeError {
    fn from(value: ManagedMarkdownError) -> Self {
        Self::Markdown(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CanonicalIndexSource {
    item_id: MemoryItemId,
    version_id: MemoryVersionId,
    scope: MemoryScope,
    status: MemoryVersionStatus,
    markdown_path: PathBuf,
    content_digest: BindingDigest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IndexedDocument {
    item_id: MemoryItemId,
    version_id: MemoryVersionId,
    scope: MemoryScope,
    status: MemoryVersionStatus,
    content_digest: BindingDigest,
    relative_path: String,
}

struct IndexPlan {
    generation: DerivativeIndexGeneration,
    index_digest: BindingDigest,
    entry_count: u64,
    term_count: u64,
    bytes: Vec<u8>,
}

pub fn text_metadata_index_kind_ref() -> BindingDigest {
    BindingDigest::new(sha256(TEXT_METADATA_INDEX_KIND_DOMAIN))
}

pub fn text_metadata_implementation_identity() -> BindingDigest {
    BindingDigest::new(sha256(TEXT_METADATA_IMPLEMENTATION_DOMAIN))
}

pub fn text_metadata_index_path(layout: &MemoryLayout) -> PathBuf {
    layout
        .operational_dir()
        .join("derivatives")
        .join(INDEX_FILE_NAME)
}

pub fn rebuild_text_metadata_index(
    layout: &MemoryLayout,
    built_at_unix_ms: u64,
) -> Result<MemoryDerivativeIndexReceipt, MemoryDerivativeError> {
    let plan = build_index_plan(layout, built_at_unix_ms)?;
    persist_generation(layout, &plan.generation, DerivativeIndexStatus::Rebuilding)?;
    match install_index(layout, &plan.bytes) {
        Ok(()) => {
            persist_generation(layout, &plan.generation, DerivativeIndexStatus::Current)?;
            Ok(receipt(layout, &plan))
        }
        Err(error) => {
            let _ = persist_generation(layout, &plan.generation, DerivativeIndexStatus::Failed);
            Err(error)
        }
    }
}

pub fn load_or_rebuild_text_metadata_index(
    layout: &MemoryLayout,
    built_at_unix_ms: u64,
) -> Result<LoadedMemoryDerivativeIndex, MemoryDerivativeError> {
    let plan = build_index_plan(layout, built_at_unix_ms)?;
    if let Some(current) = current_generation(layout)? {
        if same_generation_identity(&current, &plan.generation) {
            let path = text_metadata_index_path(layout);
            match read_existing_index(&path) {
                Ok(bytes) if bytes == plan.bytes => {
                    return Ok(LoadedMemoryDerivativeIndex {
                        receipt: MemoryDerivativeIndexReceipt {
                            generation: current,
                            index_digest: plan.index_digest,
                            entry_count: plan.entry_count,
                            term_count: plan.term_count,
                            index_path: path,
                        },
                        bytes,
                    });
                }
                Ok(_)
                | Err(MemoryDerivativeError::Io(_))
                | Err(MemoryDerivativeError::DerivativeCorrupt) => {
                    let _ = persist_generation(layout, &current, DerivativeIndexStatus::Failed);
                }
                Err(error) => return Err(error),
            }
        } else {
            let _ = persist_generation(layout, &current, DerivativeIndexStatus::Stale);
        }
    }

    persist_generation(layout, &plan.generation, DerivativeIndexStatus::Rebuilding)?;
    match install_index(layout, &plan.bytes) {
        Ok(()) => {
            persist_generation(layout, &plan.generation, DerivativeIndexStatus::Current)?;
            Ok(LoadedMemoryDerivativeIndex {
                receipt: receipt(layout, &plan),
                bytes: plan.bytes,
            })
        }
        Err(error) => {
            let _ = persist_generation(layout, &plan.generation, DerivativeIndexStatus::Failed);
            Err(error)
        }
    }
}

pub fn invalidate_memory_derivatives(
    layout: &MemoryLayout,
) -> Result<usize, MemoryDerivativeError> {
    drop(MemoryOperationalStore::open(layout)?);
    let connection = Connection::open(layout.operational_db_path())?;
    connection.execute_batch(
        "PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
    )?;
    let changed = connection.execute(
        "UPDATE memory_derivative_generations SET status = 2 \
         WHERE store_ref = ?1 AND status != 2",
        params![layout.store_id().0.bytes().to_vec()],
    )?;
    remove_projection_file(&text_metadata_index_path(layout))?;
    remove_projection_file(
        &layout
            .operational_dir()
            .join("derivatives")
            .join(INDEX_TEMP_FILE_NAME),
    )?;
    Ok(changed)
}

fn build_index_plan(
    layout: &MemoryLayout,
    built_at_unix_ms: u64,
) -> Result<IndexPlan, MemoryDerivativeError> {
    let operational = MemoryOperationalStore::open(layout)?;
    if operational.has_blocking_unknown_outcome()? {
        return Err(MemoryDerivativeError::CanonicalAmbiguous);
    }
    drop(operational);

    let sources = load_canonical_sources(layout)?;
    if sources.len() > MAX_INDEX_ITEMS {
        return Err(MemoryDerivativeError::BoundExceeded("item count"));
    }
    let canonical_vault = fs::canonicalize(layout.vault_dir())?;
    let mut total_canonical_bytes = 0usize;
    let mut documents = Vec::with_capacity(sources.len());
    let mut postings: BTreeMap<String, BTreeSet<([u8; 32], [u8; 32])>> = BTreeMap::new();
    let mut total_postings = 0usize;
    let mut cut_encoder = CanonicalEncoder::new();
    cut_encoder.push_bytes(CANONICAL_CUT_DOMAIN)?;
    cut_encoder.push_bytes(&layout.store_id().0.bytes())?;
    cut_encoder.push_bytes(&layout.schema_ref().bytes())?;
    cut_encoder.push_u64(to_u64(sources.len())?);

    for source in sources {
        let (document, relative_path, byte_len) =
            read_canonical_document(layout, &canonical_vault, &source)?;
        total_canonical_bytes = total_canonical_bytes
            .checked_add(byte_len)
            .ok_or(MemoryDerivativeError::IntegerOverflow)?;
        if total_canonical_bytes > MAX_TOTAL_CANONICAL_BYTES {
            return Err(MemoryDerivativeError::BoundExceeded(
                "aggregate canonical Markdown bytes",
            ));
        }

        cut_encoder.push_bytes(&source.item_id.0.bytes())?;
        cut_encoder.push_bytes(&source.version_id.0.bytes())?;
        cut_encoder.push_u8(scope_code(source.scope));
        cut_encoder.push_u8(status_code(source.status));
        cut_encoder.push_bytes(&source.content_digest.bytes())?;
        cut_encoder.push_bytes(relative_path.as_bytes())?;

        let key = (source.item_id.0.bytes(), source.version_id.0.bytes());
        let terms = document_terms(&document, source.scope, source.status)?;
        for term in terms {
            add_posting(&mut postings, term, key, &mut total_postings)?;
        }
        documents.push(IndexedDocument {
            item_id: source.item_id,
            version_id: source.version_id,
            scope: source.scope,
            status: source.status,
            content_digest: source.content_digest,
            relative_path,
        });
    }

    let canonical_cut_digest = BindingDigest::new(sha256(&cut_encoder.finish()));
    let index_kind_ref = text_metadata_index_kind_ref();
    let implementation_identity = text_metadata_implementation_identity();
    let generation_id = generation_id(
        layout,
        index_kind_ref,
        canonical_cut_digest,
        implementation_identity,
    )?;
    let generation = DerivativeIndexGeneration {
        index_kind_ref,
        generation_id,
        canonical_cut_digest,
        implementation_identity,
        status: DerivativeIndexStatus::Current,
        built_at_unix_ms,
    };
    let bytes = serialize_index(layout, &generation, &documents, &postings)?;
    if bytes.len() > MAX_INDEX_BYTES {
        return Err(MemoryDerivativeError::BoundExceeded(
            "serialized index bytes",
        ));
    }
    let index_digest = BindingDigest::new(sha256(&bytes));
    Ok(IndexPlan {
        generation,
        index_digest,
        entry_count: to_u64(documents.len())?,
        term_count: to_u64(postings.len())?,
        bytes,
    })
}

fn load_canonical_sources(
    layout: &MemoryLayout,
) -> Result<Vec<CanonicalIndexSource>, MemoryDerivativeError> {
    let connection = Connection::open(layout.operational_db_path())?;
    connection.execute_batch("PRAGMA query_only = ON; PRAGMA busy_timeout = 5000;")?;
    let mut statement = connection.prepare(
        "SELECT i.item_id, i.scope, i.current_version_id, i.state, \
                v.scope, v.markdown_path, v.content_digest, v.status, \
                e.state, e.terminal_status \
         FROM memory_items i \
         JOIN memory_versions v ON v.version_id = i.current_version_id \
         JOIN memory_effect_state e ON e.effect_id = v.effect_id \
         WHERE i.store_ref = ?1 AND v.store_ref = ?1 AND e.store_ref = ?1 \
         ORDER BY i.item_id ASC",
    )?;
    let rows = statement.query_map(params![layout.store_id().0.bytes().to_vec()], |row| {
        Ok((
            row.get::<_, Vec<u8>>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, Vec<u8>>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, Vec<u8>>(6)?,
            row.get::<_, i64>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, Option<i64>>(9)?,
        ))
    })?;

    let mut output = Vec::new();
    for row in rows {
        let (
            item_id,
            item_scope,
            version_id,
            item_state,
            version_scope,
            markdown_path,
            content_digest,
            version_status,
            effect_state,
            terminal_status,
        ) = row?;
        if output.len() >= MAX_INDEX_ITEMS {
            return Err(MemoryDerivativeError::BoundExceeded("item count"));
        }
        if item_scope != version_scope || item_state != version_status {
            return Err(MemoryDerivativeError::InvalidRecord(
                "memory item/version scope or state mismatch",
            ));
        }
        if effect_state != "terminal" || terminal_status != Some(1) {
            return Err(MemoryDerivativeError::InvalidRecord(
                "current memory version lacks committed terminal effect evidence",
            ));
        }
        output.push(CanonicalIndexSource {
            item_id: MemoryItemId(BindingDigest::new(hash32(item_id, "item id")?)),
            version_id: MemoryVersionId(BindingDigest::new(hash32(version_id, "version id")?)),
            scope: decode_scope(item_scope)?,
            status: decode_status(version_status)?,
            markdown_path: PathBuf::from(markdown_path),
            content_digest: BindingDigest::new(hash32(content_digest, "content digest")?),
        });
    }
    Ok(output)
}

fn read_canonical_document(
    layout: &MemoryLayout,
    canonical_vault: &Path,
    source: &CanonicalIndexSource,
) -> Result<(ManagedMarkdownDocument, String, usize), MemoryDerivativeError> {
    if !source.markdown_path.starts_with(layout.vault_dir())
        || layout.is_operational_path(&source.markdown_path)
    {
        return Err(MemoryDerivativeError::UnsafeCanonicalPath(
            source.markdown_path.clone(),
        ));
    }
    let canonical_path = fs::canonicalize(&source.markdown_path)?;
    if !canonical_path.starts_with(canonical_vault) {
        return Err(MemoryDerivativeError::UnsafeCanonicalPath(canonical_path));
    }
    let metadata = fs::symlink_metadata(&canonical_path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(MemoryDerivativeError::UnsafeCanonicalPath(canonical_path));
    }
    let mut file = File::open(&canonical_path)?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(to_u64(MAX_CANONICAL_MARKDOWN_BYTES + 1)?)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_CANONICAL_MARKDOWN_BYTES {
        return Err(MemoryDerivativeError::BoundExceeded(
            "canonical Markdown item bytes",
        ));
    }
    if BindingDigest::new(sha256(&bytes)) != source.content_digest {
        return Err(MemoryDerivativeError::CanonicalDigestMismatch(
            source.markdown_path.clone(),
        ));
    }
    let document = parse_managed_markdown(&bytes)?;
    let relative = source
        .markdown_path
        .strip_prefix(layout.vault_dir())
        .map_err(|_| MemoryDerivativeError::UnsafeCanonicalPath(source.markdown_path.clone()))?;
    let relative = relative
        .to_str()
        .ok_or_else(|| MemoryDerivativeError::UnsafeCanonicalPath(source.markdown_path.clone()))?
        .replace('\\', "/");
    Ok((document, relative, bytes.len()))
}

fn document_terms(
    document: &ManagedMarkdownDocument,
    scope: MemoryScope,
    status: MemoryVersionStatus,
) -> Result<BTreeSet<String>, MemoryDerivativeError> {
    let mut terms = BTreeSet::new();
    insert_term(&mut terms, format!("scope:{}", scope_name(scope)))?;
    insert_term(&mut terms, format!("status:{}", status_name(status)))?;
    if is_retrievable(status) {
        for term in lexical_terms(document.body())? {
            insert_term(&mut terms, format!("text:{term}"))?;
        }
        for (key, value) in document.metadata() {
            insert_term(&mut terms, format!("meta-key:{key}"))?;
            for term in lexical_terms(value)? {
                insert_term(&mut terms, format!("meta:{key}:{term}"))?;
            }
        }
    }
    if terms.len() > MAX_TERMS_PER_DOCUMENT {
        return Err(MemoryDerivativeError::BoundExceeded(
            "terms per canonical document",
        ));
    }
    Ok(terms)
}

fn lexical_terms(text: &str) -> Result<BTreeSet<String>, MemoryDerivativeError> {
    let mut output = BTreeSet::new();
    let mut current = String::new();
    let mut oversized = false;
    for character in text.chars().chain(std::iter::once(' ')) {
        if character.is_alphanumeric() || matches!(character, '_' | '-') {
            if !oversized {
                for lower in character.to_lowercase() {
                    if current.len() + lower.len_utf8() > MAX_LEXICAL_TERM_BYTES {
                        current.clear();
                        oversized = true;
                        break;
                    }
                    current.push(lower);
                }
            }
        } else {
            if !oversized && !current.is_empty() {
                output.insert(std::mem::take(&mut current));
                if output.len() > MAX_TERMS_PER_DOCUMENT {
                    return Err(MemoryDerivativeError::BoundExceeded(
                        "lexical terms per document",
                    ));
                }
            } else {
                current.clear();
            }
            oversized = false;
        }
    }
    Ok(output)
}

fn insert_term(terms: &mut BTreeSet<String>, term: String) -> Result<(), MemoryDerivativeError> {
    if term.len() > MAX_INDEX_TERM_BYTES {
        return Err(MemoryDerivativeError::BoundExceeded("index term bytes"));
    }
    terms.insert(term);
    Ok(())
}

fn add_posting(
    postings: &mut BTreeMap<String, BTreeSet<([u8; 32], [u8; 32])>>,
    term: String,
    key: ([u8; 32], [u8; 32]),
    total_postings: &mut usize,
) -> Result<(), MemoryDerivativeError> {
    if !postings.contains_key(&term) && postings.len() >= MAX_UNIQUE_TERMS {
        return Err(MemoryDerivativeError::BoundExceeded("unique index terms"));
    }
    let values = postings.entry(term).or_default();
    if values.insert(key) {
        *total_postings = total_postings
            .checked_add(1)
            .ok_or(MemoryDerivativeError::IntegerOverflow)?;
        if *total_postings > MAX_TOTAL_POSTINGS {
            return Err(MemoryDerivativeError::BoundExceeded("total index postings"));
        }
    }
    Ok(())
}

fn generation_id(
    layout: &MemoryLayout,
    index_kind_ref: BindingDigest,
    canonical_cut_digest: BindingDigest,
    implementation_identity: BindingDigest,
) -> Result<MemoryDerivativeGenerationId, MemoryDerivativeError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(GENERATION_DOMAIN)?;
    encoder.push_bytes(&layout.store_id().0.bytes())?;
    encoder.push_bytes(&layout.schema_ref().bytes())?;
    encoder.push_bytes(&index_kind_ref.bytes())?;
    encoder.push_bytes(&canonical_cut_digest.bytes())?;
    encoder.push_bytes(&implementation_identity.bytes())?;
    Ok(MemoryDerivativeGenerationId(BindingDigest::new(sha256(
        &encoder.finish(),
    ))))
}

fn serialize_index(
    layout: &MemoryLayout,
    generation: &DerivativeIndexGeneration,
    documents: &[IndexedDocument],
    postings: &BTreeMap<String, BTreeSet<([u8; 32], [u8; 32])>>,
) -> Result<Vec<u8>, MemoryDerivativeError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(INDEX_FILE_DOMAIN)?;
    encoder.push_bytes(&layout.store_id().0.bytes())?;
    encoder.push_bytes(&layout.schema_ref().bytes())?;
    encoder.push_bytes(&generation.index_kind_ref.bytes())?;
    encoder.push_bytes(&generation.generation_id.0.bytes())?;
    encoder.push_bytes(&generation.canonical_cut_digest.bytes())?;
    encoder.push_bytes(&generation.implementation_identity.bytes())?;
    encoder.push_u64(to_u64(documents.len())?);
    for document in documents {
        encoder.push_bytes(&document.item_id.0.bytes())?;
        encoder.push_bytes(&document.version_id.0.bytes())?;
        encoder.push_u8(scope_code(document.scope));
        encoder.push_u8(status_code(document.status));
        encoder.push_bytes(&document.content_digest.bytes())?;
        encoder.push_bytes(document.relative_path.as_bytes())?;
    }
    encoder.push_u64(to_u64(postings.len())?);
    for (term, values) in postings {
        encoder.push_bytes(term.as_bytes())?;
        encoder.push_u64(to_u64(values.len())?);
        for (item_id, version_id) in values {
            encoder.push_bytes(item_id)?;
            encoder.push_bytes(version_id)?;
        }
    }
    Ok(encoder.finish())
}

fn current_generation(
    layout: &MemoryLayout,
) -> Result<Option<DerivativeIndexGeneration>, MemoryDerivativeError> {
    let connection = Connection::open(layout.operational_db_path())?;
    connection.execute_batch("PRAGMA query_only = ON; PRAGMA busy_timeout = 5000;")?;
    let kind = text_metadata_index_kind_ref();
    let current_count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM memory_derivative_generations \
         WHERE store_ref = ?1 AND index_kind_ref = ?2 AND status = 1",
        params![layout.store_id().0.bytes().to_vec(), kind.bytes().to_vec()],
        |row| row.get(0),
    )?;
    if current_count > 1 {
        return Err(MemoryDerivativeError::InvalidRecord(
            "multiple current derivative generations",
        ));
    }
    connection
        .query_row(
            "SELECT generation_id, canonical_cut_digest, implementation_identity, built_at_unix_ms \
             FROM memory_derivative_generations \
             WHERE store_ref = ?1 AND index_kind_ref = ?2 AND status = 1 LIMIT 1",
            params![layout.store_id().0.bytes().to_vec(), kind.bytes().to_vec()],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .optional()?
        .map(|(generation_id, canonical_cut, implementation, built_at)| {
            Ok(DerivativeIndexGeneration {
                index_kind_ref: kind,
                generation_id: MemoryDerivativeGenerationId(BindingDigest::new(hash32(
                    generation_id,
                    "derivative generation id",
                )?)),
                canonical_cut_digest: BindingDigest::new(hash32(
                    canonical_cut,
                    "derivative canonical cut",
                )?),
                implementation_identity: BindingDigest::new(hash32(
                    implementation,
                    "derivative implementation identity",
                )?),
                status: DerivativeIndexStatus::Current,
                built_at_unix_ms: u64::try_from(built_at).map_err(|_| {
                    MemoryDerivativeError::InvalidRecord("negative derivative timestamp")
                })?,
            })
        })
        .transpose()
}

fn persist_generation(
    layout: &MemoryLayout,
    generation: &DerivativeIndexGeneration,
    status: DerivativeIndexStatus,
) -> Result<(), MemoryDerivativeError> {
    let mut connection = Connection::open(layout.operational_db_path())?;
    connection.execute_batch(
        "PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
    )?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let generation_key = generation.generation_id.0.bytes().to_vec();
    let existing = transaction
        .query_row(
            "SELECT store_ref, index_kind_ref, canonical_cut_digest, implementation_identity \
             FROM memory_derivative_generations WHERE generation_id = ?1",
            params![generation_key.clone()],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                ))
            },
        )
        .optional()?;
    if let Some(existing) = existing {
        if existing.0 != layout.store_id().0.bytes().to_vec()
            || existing.1 != generation.index_kind_ref.bytes().to_vec()
            || existing.2 != generation.canonical_cut_digest.bytes().to_vec()
            || existing.3 != generation.implementation_identity.bytes().to_vec()
        {
            return Err(MemoryDerivativeError::InvalidRecord(
                "derivative generation identity collision",
            ));
        }
        transaction.execute(
            "UPDATE memory_derivative_generations SET status = ?1, built_at_unix_ms = ?2 \
             WHERE generation_id = ?3",
            params![
                derivative_status_code(status),
                to_i64(generation.built_at_unix_ms)?,
                generation_key,
            ],
        )?;
    } else {
        transaction.execute(
            "INSERT INTO memory_derivative_generations \
             (generation_id, store_ref, index_kind_ref, canonical_cut_digest, \
              implementation_identity, status, built_at_unix_ms) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                generation_key,
                layout.store_id().0.bytes().to_vec(),
                generation.index_kind_ref.bytes().to_vec(),
                generation.canonical_cut_digest.bytes().to_vec(),
                generation.implementation_identity.bytes().to_vec(),
                derivative_status_code(status),
                to_i64(generation.built_at_unix_ms)?,
            ],
        )?;
    }
    if status == DerivativeIndexStatus::Current {
        transaction.execute(
            "UPDATE memory_derivative_generations SET status = 2 \
             WHERE store_ref = ?1 AND index_kind_ref = ?2 AND generation_id != ?3 AND status != 2",
            params![
                layout.store_id().0.bytes().to_vec(),
                generation.index_kind_ref.bytes().to_vec(),
                generation.generation_id.0.bytes().to_vec(),
            ],
        )?;
    }
    transaction.commit()?;
    Ok(())
}

fn same_generation_identity(
    left: &DerivativeIndexGeneration,
    right: &DerivativeIndexGeneration,
) -> bool {
    left.index_kind_ref == right.index_kind_ref
        && left.generation_id == right.generation_id
        && left.canonical_cut_digest == right.canonical_cut_digest
        && left.implementation_identity == right.implementation_identity
}

fn receipt(layout: &MemoryLayout, plan: &IndexPlan) -> MemoryDerivativeIndexReceipt {
    MemoryDerivativeIndexReceipt {
        generation: plan.generation.clone(),
        index_digest: plan.index_digest,
        entry_count: plan.entry_count,
        term_count: plan.term_count,
        index_path: text_metadata_index_path(layout),
    }
}

fn install_index(layout: &MemoryLayout, bytes: &[u8]) -> Result<(), MemoryDerivativeError> {
    let directory = ensure_derivative_directory(layout)?;
    let target = directory.join(INDEX_FILE_NAME);
    let temporary = directory.join(INDEX_TEMP_FILE_NAME);
    remove_projection_file(&temporary)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    apply_private_file_permissions(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    remove_projection_file(&target)?;
    fs::rename(&temporary, &target)?;
    sync_parent_directory(&directory)?;
    Ok(())
}

fn read_existing_index(path: &Path) -> Result<Vec<u8>, MemoryDerivativeError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(MemoryDerivativeError::DerivativeCorrupt);
        }
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(MemoryDerivativeError::UnsafeDerivativePath(
            path.to_path_buf(),
        ));
    }
    let Ok(length) = usize::try_from(metadata.len()) else {
        return Err(MemoryDerivativeError::DerivativeCorrupt);
    };
    if length > MAX_INDEX_BYTES {
        return Err(MemoryDerivativeError::DerivativeCorrupt);
    }
    let bytes = fs::read(path)?;
    if bytes.len() > MAX_INDEX_BYTES {
        return Err(MemoryDerivativeError::DerivativeCorrupt);
    }
    Ok(bytes)
}

fn ensure_derivative_directory(layout: &MemoryLayout) -> Result<PathBuf, MemoryDerivativeError> {
    let directory = layout.operational_dir().join("derivatives");
    match fs::symlink_metadata(&directory) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(MemoryDerivativeError::UnsafeDerivativePath(directory));
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => fs::create_dir(&directory)?,
        Err(error) => return Err(error.into()),
    }
    apply_private_directory_permissions(&directory)?;
    Ok(directory)
}

fn remove_projection_file(path: &Path) -> Result<(), MemoryDerivativeError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.is_dir() && !metadata.file_type().is_symlink() {
                return Err(MemoryDerivativeError::UnsafeDerivativePath(
                    path.to_path_buf(),
                ));
            }
            fs::remove_file(path)?;
            Ok(())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(unix)]
fn apply_private_directory_permissions(path: &Path) -> Result<(), MemoryDerivativeError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

#[cfg(not(unix))]
fn apply_private_directory_permissions(_path: &Path) -> Result<(), MemoryDerivativeError> {
    Ok(())
}

#[cfg(unix)]
fn apply_private_file_permissions(path: &Path) -> Result<(), MemoryDerivativeError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn apply_private_file_permissions(_path: &Path) -> Result<(), MemoryDerivativeError> {
    Ok(())
}

#[cfg(unix)]
fn sync_parent_directory(path: &Path) -> Result<(), MemoryDerivativeError> {
    File::open(path)?.sync_all()?;
    Ok(())
}

#[cfg(not(unix))]
fn sync_parent_directory(_path: &Path) -> Result<(), MemoryDerivativeError> {
    Ok(())
}

fn hash32(value: Vec<u8>, reason: &'static str) -> Result<[u8; 32], MemoryDerivativeError> {
    value
        .try_into()
        .map_err(|_| MemoryDerivativeError::InvalidRecord(reason))
}

fn to_u64(value: usize) -> Result<u64, MemoryDerivativeError> {
    u64::try_from(value).map_err(|_| MemoryDerivativeError::IntegerOverflow)
}

fn to_i64(value: u64) -> Result<i64, MemoryDerivativeError> {
    i64::try_from(value).map_err(|_| MemoryDerivativeError::IntegerOverflow)
}

fn decode_scope(value: i64) -> Result<MemoryScope, MemoryDerivativeError> {
    match value {
        1 => Ok(MemoryScope::User),
        2 => Ok(MemoryScope::Project),
        _ => Err(MemoryDerivativeError::InvalidRecord("memory scope")),
    }
}

fn decode_status(value: i64) -> Result<MemoryVersionStatus, MemoryDerivativeError> {
    match value {
        1 => Ok(MemoryVersionStatus::Active),
        2 => Ok(MemoryVersionStatus::Superseded),
        3 => Ok(MemoryVersionStatus::Contradicted),
        4 => Ok(MemoryVersionStatus::Expired),
        5 => Ok(MemoryVersionStatus::Forgotten),
        6 => Ok(MemoryVersionStatus::Redacted),
        _ => Err(MemoryDerivativeError::InvalidRecord(
            "memory version status",
        )),
    }
}

const fn scope_code(scope: MemoryScope) -> u8 {
    match scope {
        MemoryScope::User => 1,
        MemoryScope::Project => 2,
    }
}

const fn status_code(status: MemoryVersionStatus) -> u8 {
    match status {
        MemoryVersionStatus::Active => 1,
        MemoryVersionStatus::Superseded => 2,
        MemoryVersionStatus::Contradicted => 3,
        MemoryVersionStatus::Expired => 4,
        MemoryVersionStatus::Forgotten => 5,
        MemoryVersionStatus::Redacted => 6,
    }
}

const fn derivative_status_code(status: DerivativeIndexStatus) -> i64 {
    match status {
        DerivativeIndexStatus::Current => 1,
        DerivativeIndexStatus::Stale => 2,
        DerivativeIndexStatus::Rebuilding => 3,
        DerivativeIndexStatus::Failed => 4,
    }
}

const fn is_retrievable(status: MemoryVersionStatus) -> bool {
    matches!(
        status,
        MemoryVersionStatus::Active | MemoryVersionStatus::Contradicted
    )
}

const fn scope_name(scope: MemoryScope) -> &'static str {
    match scope {
        MemoryScope::User => "user",
        MemoryScope::Project => "project",
    }
}

const fn status_name(status: MemoryVersionStatus) -> &'static str {
    match status {
        MemoryVersionStatus::Active => "active",
        MemoryVersionStatus::Superseded => "superseded",
        MemoryVersionStatus::Contradicted => "contradicted",
        MemoryVersionStatus::Expired => "expired",
        MemoryVersionStatus::Forgotten => "forgotten",
        MemoryVersionStatus::Redacted => "redacted",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use golam_core::EffectId;
    use golam_core::memory::{
        ExpectedMemoryVersion, MemoryCandidateId, MemoryMutationIntent, MemoryOperation,
        MemoryStoreId, MemoryVersion, MemoryWriterId,
    };
    use golam_core::memory_storage::{MemoryLayout, MemoryVaultScope};
    use golam_core::paths::RuntimeLayout;
    use golam_core::taint::{TaintLabel, TaintSet};
    use golam_core::tool_request::PrincipalId;

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
            "golam-memory-derivative-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    fn fixture() -> (RuntimeLayout, MemoryLayout, PathBuf, BindingDigest) {
        let runtime = runtime();
        let layout = MemoryLayout::initialize(&runtime).unwrap();
        let item_id = MemoryItemId(digest(1));
        let version_id = MemoryVersionId(digest(2));
        let path = layout.item_path(MemoryVaultScope::User, item_id).unwrap();
        let mut metadata = BTreeMap::new();
        metadata.insert("topic".to_owned(), "Alpha Memory".to_owned());
        let document =
            ManagedMarkdownDocument::new(metadata, "Alpha beta durable memory\n").unwrap();
        let bytes = document.serialize().unwrap();
        let content_digest = BindingDigest::new(sha256(&bytes));
        fs::write(&path, bytes).unwrap();

        let prepared = MemoryMutationIntent {
            operation: MemoryOperation::Add,
            item_ids: vec![item_id],
            expected_current_versions: vec![ExpectedMemoryVersion {
                item_id,
                expected_version: None,
            }],
            expected_markdown_target_identity_ref: digest(3),
            expected_markdown_content_digest: digest(4),
            expected_markdown_version: version_id,
            memory_operational_store_ref: layout.store_id(),
            candidate_ref: Some(MemoryCandidateId(digest(5))),
            kernel_authorization_ref: digest(6),
            promotion_authority_ref: digest(7),
            effect_id: EffectId(8),
            reason_ref: digest(9),
            initiating_principal: PrincipalId::new("principal.local").unwrap(),
            created_at_unix_ms: 10,
        }
        .prepare()
        .unwrap();
        let version = MemoryVersion {
            item_id,
            version_id,
            scope: MemoryScope::User,
            canonical_markdown_ref: digest(11),
            content_digest,
            provenance_refs: vec![digest(12)],
            taint_set: TaintSet::from_labels([TaintLabel::UserTrusted]),
            status: MemoryVersionStatus::Active,
            predecessor_versions: Vec::new(),
            conflict_refs: Vec::new(),
            promotion_evidence_ref: digest(13),
            created_by_principal: PrincipalId::new("principal.creator").unwrap(),
            committed_by_writer_identity: MemoryWriterId(digest(14)),
            mutation_effect_ref: EffectId(8),
            created_at_unix_ms: 15,
        };
        let mut store = MemoryOperationalStore::open(&layout).unwrap();
        store.record_prepared(&prepared).unwrap();
        store.record_version(&prepared, &version, &path).unwrap();
        store
            .mark_terminal(
                EffectId(8),
                BindingDigest::new(prepared.binding_digest()),
                golam_core::memory::MemoryMutationStatus::Committed,
            )
            .unwrap();
        drop(store);
        (runtime, layout, path, content_digest)
    }

    #[test]
    fn rebuild_is_deterministic_and_records_one_current_generation() {
        let (runtime, layout, _, _) = fixture();
        let first = rebuild_text_metadata_index(&layout, 100).unwrap();
        let first_bytes = fs::read(&first.index_path).unwrap();
        let second = rebuild_text_metadata_index(&layout, 200).unwrap();
        let second_bytes = fs::read(&second.index_path).unwrap();
        assert_eq!(
            first.generation.generation_id,
            second.generation.generation_id
        );
        assert_eq!(
            first.generation.canonical_cut_digest,
            second.generation.canonical_cut_digest
        );
        assert_eq!(first.index_digest, second.index_digest);
        assert_eq!(first_bytes, second_bytes);
        assert_eq!(
            current_generation(&layout).unwrap().unwrap().generation_id,
            second.generation.generation_id
        );
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn missing_or_corrupt_derivative_rebuilds_without_becoming_canonical() {
        let (runtime, layout, _, _) = fixture();
        let first = rebuild_text_metadata_index(&layout, 100).unwrap();
        fs::write(&first.index_path, b"corrupt derivative").unwrap();
        let loaded = load_or_rebuild_text_metadata_index(&layout, 200).unwrap();
        assert_eq!(
            loaded.receipt.generation.generation_id,
            first.generation.generation_id
        );
        assert_eq!(
            BindingDigest::new(sha256(&loaded.bytes)),
            first.index_digest
        );
        fs::remove_file(&first.index_path).unwrap();
        let reopened = MemoryOperationalStore::open(&layout).unwrap();
        drop(reopened);
        let rebuilt = load_or_rebuild_text_metadata_index(&layout, 300).unwrap();
        assert_eq!(
            rebuilt.receipt.generation.generation_id,
            first.generation.generation_id
        );
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn invalidation_removes_active_projection_and_marks_generation_non_current() {
        let (runtime, layout, _, _) = fixture();
        let built = rebuild_text_metadata_index(&layout, 100).unwrap();
        assert!(built.index_path.exists());
        assert!(invalidate_memory_derivatives(&layout).unwrap() >= 1);
        assert!(!built.index_path.exists());
        assert!(current_generation(&layout).unwrap().is_none());
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn canonical_digest_disagreement_fails_only_derivative_access() {
        let (runtime, layout, markdown_path, _) = fixture();
        fs::write(&markdown_path, b"tampered canonical content\n").unwrap();
        assert!(matches!(
            load_or_rebuild_text_metadata_index(&layout, 100),
            Err(MemoryDerivativeError::CanonicalDigestMismatch(_))
        ));
        let canonical_store = MemoryOperationalStore::open(&layout).unwrap();
        assert_eq!(canonical_store.store_id(), layout.store_id());
        drop(canonical_store);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn expired_and_forgotten_statuses_are_not_text_retrievable() {
        let document = ManagedMarkdownDocument::new(BTreeMap::new(), "secret alpha\n").unwrap();
        let expired =
            document_terms(&document, MemoryScope::User, MemoryVersionStatus::Expired).unwrap();
        let forgotten =
            document_terms(&document, MemoryScope::User, MemoryVersionStatus::Forgotten).unwrap();
        assert!(!expired.iter().any(|term| term.contains("secret")));
        assert!(!forgotten.iter().any(|term| term.contains("secret")));
        assert!(expired.contains("status:expired"));
        assert!(forgotten.contains("status:forgotten"));
    }

    #[test]
    fn generation_identity_binds_store_cut_and_implementation() {
        let first_runtime = runtime();
        let second_runtime = runtime();
        let first_layout = MemoryLayout::initialize(&first_runtime).unwrap();
        let second_layout = MemoryLayout::initialize(&second_runtime).unwrap();
        let kind = text_metadata_index_kind_ref();
        let implementation = text_metadata_implementation_identity();
        let first = generation_id(&first_layout, kind, digest(30), implementation).unwrap();
        let second = generation_id(&second_layout, kind, digest(30), implementation).unwrap();
        assert_ne!(first, second);
        fs::remove_dir_all(first_runtime.root).unwrap();
        fs::remove_dir_all(second_runtime.root).unwrap();
    }

    #[test]
    fn source_store_binding_is_not_free_form() {
        let store = MemoryStoreId(digest(40));
        assert_ne!(store, MemoryStoreId(text_metadata_index_kind_ref()));
    }
}
