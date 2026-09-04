#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::Path;

use golam_core::memory::{
    DerivativeIndexGeneration, DerivativeIndexStatus, MemoryItemId, MemoryMutationStatus,
    MemoryOperation, MemoryReconciliationState, MemoryStoreId, MemoryVersion, MemoryVersionId,
    MemoryVersionStatus, PreparedMemoryMutationIntent,
};
use golam_core::memory_storage::{MEMORY_OPERATIONAL_SCHEMA_VERSION, MemoryLayout};
use golam_core::tool_request::BindingDigest;
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

pub const REQUIRED_MEMORY_OPERATIONAL_TABLES: &[&str] = &[
    "memory_operational_meta",
    "memory_effect_state",
    "memory_items",
    "memory_versions",
    "memory_relations",
    "memory_reconciliation_state",
    "memory_promotion_state",
    "memory_derivative_generations",
];

const MEMORY_PROMOTION_STATE_SCHEMA: &str = r#"
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
"#;

#[derive(Debug)]
pub enum MemoryOperationalError {
    Sqlite(rusqlite::Error),
    Core(CoreError),
    InvalidRecord(&'static str),
    StoreBindingMismatch,
    FutureSchema { found: i64, supported: i64 },
    UnsupportedSchema { found: i64, supported: i64 },
    MissingPreparedEffect(EffectId),
    IntentDigestMismatch,
    ImmutableVersionMismatch,
    TerminalStatusConflict,
    StaleCurrentVersion,
    NonUnicodePath,
    IntegerOverflow,
}

impl fmt::Display for MemoryOperationalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "memory operational sqlite error: {error}"),
            Self::Core(error) => write!(f, "memory operational encoding error: {error}"),
            Self::InvalidRecord(reason) => write!(f, "invalid memory operational record: {reason}"),
            Self::StoreBindingMismatch => {
                f.write_str("memory operational store binding does not match the prepared intent")
            }
            Self::FutureSchema { found, supported } => write!(
                f,
                "memory operational schema {found} is newer than supported {supported}"
            ),
            Self::UnsupportedSchema { found, supported } => write!(
                f,
                "memory operational schema {found} cannot be migrated to supported {supported}"
            ),
            Self::MissingPreparedEffect(effect_id) => write!(
                f,
                "memory operational effect {} has no PREPARED state",
                effect_id.0
            ),
            Self::IntentDigestMismatch => {
                f.write_str("memory operational state does not bind the PREPARED intent digest")
            }
            Self::ImmutableVersionMismatch => {
                f.write_str("memory version identity already exists with different protected state")
            }
            Self::TerminalStatusConflict => {
                f.write_str("memory effect terminal status is immutable once recorded")
            }
            Self::StaleCurrentVersion => {
                f.write_str("memory current version does not match the prepared expected version")
            }
            Self::NonUnicodePath => f.write_str("managed Markdown path is not valid UTF-8"),
            Self::IntegerOverflow => f.write_str("memory operational integer conversion overflow"),
        }
    }
}

impl Error for MemoryOperationalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for MemoryOperationalError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for MemoryOperationalError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

pub struct MemoryOperationalStore {
    connection: Connection,
    store_id: MemoryStoreId,
}

impl MemoryOperationalStore {
    pub fn open(layout: &MemoryLayout) -> Result<Self, MemoryOperationalError> {
        let connection = Connection::open(layout.operational_db_path())?;
        Self::initialize(connection, layout.store_id())
    }

    pub fn open_in_memory(store_id: MemoryStoreId) -> Result<Self, MemoryOperationalError> {
        Self::initialize(Connection::open_in_memory()?, store_id)
    }

    fn initialize(
        connection: Connection,
        store_id: MemoryStoreId,
    ) -> Result<Self, MemoryOperationalError> {
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; \
             PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        migrate(&connection, store_id)?;
        verify_required_tables(&connection)?;
        Ok(Self {
            connection,
            store_id,
        })
    }

    pub const fn store_id(&self) -> MemoryStoreId {
        self.store_id
    }

    pub fn record_prepared(
        &mut self,
        prepared: &PreparedMemoryMutationIntent,
    ) -> Result<(), MemoryOperationalError> {
        let intent = prepared.intent();
        if intent.memory_operational_store_ref != self.store_id {
            return Err(MemoryOperationalError::StoreBindingMismatch);
        }
        let digest = BindingDigest::new(prepared.binding_digest());
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(stored) = prepared_digest(&tx, intent.effect_id)? {
            if stored != digest {
                return Err(MemoryOperationalError::IntentDigestMismatch);
            }
            tx.commit()?;
            return Ok(());
        }
        tx.execute(
            r#"INSERT INTO memory_effect_state
               (effect_id, store_ref, intent_digest, operation,
                expected_markdown_target_identity_ref, expected_markdown_content_digest,
                expected_markdown_version_ref, state, terminal_status)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'prepared', NULL)"#,
            params![
                effect_blob(intent.effect_id),
                self.store_id.0.bytes().to_vec(),
                digest.bytes().to_vec(),
                operation_code(intent.operation),
                intent
                    .expected_markdown_target_identity_ref
                    .bytes()
                    .to_vec(),
                intent.expected_markdown_content_digest.bytes().to_vec(),
                intent.expected_markdown_version.0.bytes().to_vec(),
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn record_version(
        &mut self,
        prepared: &PreparedMemoryMutationIntent,
        version: &MemoryVersion,
        markdown_path: &Path,
    ) -> Result<(), MemoryOperationalError> {
        version
            .validate()
            .map_err(|_| MemoryOperationalError::InvalidRecord("memory version contract"))?;
        let intent = prepared.intent();
        if intent.memory_operational_store_ref != self.store_id
            || version.mutation_effect_ref != intent.effect_id
        {
            return Err(MemoryOperationalError::StoreBindingMismatch);
        }
        let intent_digest = BindingDigest::new(prepared.binding_digest());
        let path = markdown_path
            .to_str()
            .ok_or(MemoryOperationalError::NonUnicodePath)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        require_prepared(&tx, self.store_id, intent.effect_id, intent_digest)?;
        require_expected_version(&tx, prepared, version.item_id)?;

        let version_key = version.version_id.0.bytes().to_vec();
        if let Some(existing) = version_identity(&tx, &version_key)? {
            let expected = (
                version.content_digest,
                version.mutation_effect_ref,
                intent_digest,
            );
            if existing == expected {
                tx.commit()?;
                return Ok(());
            }
            return Err(MemoryOperationalError::ImmutableVersionMismatch);
        }

        tx.execute(
            r#"INSERT INTO memory_versions
               (version_id, store_ref, item_id, scope, markdown_path, content_digest,
                provenance_refs, status, predecessor_refs, conflict_refs,
                promotion_evidence_ref, created_by_principal, writer_id, effect_id,
                intent_digest, created_at_unix_ms)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                       ?13, ?14, ?15, ?16)"#,
            params![
                version_key,
                self.store_id.0.bytes().to_vec(),
                version.item_id.0.bytes().to_vec(),
                scope_code(version.scope),
                path,
                version.content_digest.bytes().to_vec(),
                encode_refs(version.provenance_refs.iter().copied())?,
                version_status_code(version.status),
                encode_refs(version.predecessor_versions.iter().map(|value| value.0))?,
                encode_refs(version.conflict_refs.iter().copied())?,
                version.promotion_evidence_ref.bytes().to_vec(),
                version.created_by_principal.as_str(),
                version.committed_by_writer_identity.0.bytes().to_vec(),
                effect_blob(version.mutation_effect_ref),
                intent_digest.bytes().to_vec(),
                to_i64(version.created_at_unix_ms)?,
            ],
        )?;
        tx.execute(
            r#"INSERT INTO memory_items
               (item_id, store_ref, scope, current_version_id, state,
                last_effect_id, last_intent_digest)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
               ON CONFLICT(item_id) DO UPDATE SET
                 current_version_id = excluded.current_version_id,
                 state = excluded.state,
                 last_effect_id = excluded.last_effect_id,
                 last_intent_digest = excluded.last_intent_digest"#,
            params![
                version.item_id.0.bytes().to_vec(),
                self.store_id.0.bytes().to_vec(),
                scope_code(version.scope),
                version.version_id.0.bytes().to_vec(),
                version_status_code(version.status),
                effect_blob(version.mutation_effect_ref),
                intent_digest.bytes().to_vec(),
            ],
        )?;
        for predecessor in &version.predecessor_versions {
            insert_relation(
                &tx,
                self.store_id,
                predecessor.0,
                version.version_id.0,
                "predecessor",
                version.mutation_effect_ref,
                intent_digest,
            )?;
        }
        for conflict in &version.conflict_refs {
            insert_relation(
                &tx,
                self.store_id,
                *conflict,
                version.version_id.0,
                "conflict",
                version.mutation_effect_ref,
                intent_digest,
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn record_reconciliation(
        &mut self,
        effect_id: EffectId,
        intent_digest: BindingDigest,
        state: MemoryReconciliationState,
        evidence_ref: BindingDigest,
    ) -> Result<(), MemoryOperationalError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        require_prepared(&tx, self.store_id, effect_id, intent_digest)?;
        tx.execute(
            r#"INSERT INTO memory_reconciliation_state
               (effect_id, store_ref, intent_digest, state, evidence_ref)
               VALUES (?1, ?2, ?3, ?4, ?5)
               ON CONFLICT(effect_id) DO UPDATE SET
                 state = excluded.state,
                 evidence_ref = excluded.evidence_ref"#,
            params![
                effect_blob(effect_id),
                self.store_id.0.bytes().to_vec(),
                intent_digest.bytes().to_vec(),
                reconciliation_state_code(state),
                evidence_ref.bytes().to_vec(),
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn mark_terminal(
        &mut self,
        effect_id: EffectId,
        intent_digest: BindingDigest,
        status: MemoryMutationStatus,
    ) -> Result<(), MemoryOperationalError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        require_prepared(&tx, self.store_id, effect_id, intent_digest)?;
        let requested = mutation_status_code(status);
        let existing: Option<i64> = tx
            .query_row(
                "SELECT terminal_status FROM memory_effect_state WHERE effect_id = ?1",
                params![effect_blob(effect_id)],
                |row| row.get::<_, Option<i64>>(0),
            )
            .optional()?
            .flatten();
        match existing {
            Some(stored) if stored == requested => {}
            Some(_) => return Err(MemoryOperationalError::TerminalStatusConflict),
            None => {
                let changed = tx.execute(
                    "UPDATE memory_effect_state SET state = 'terminal', terminal_status = ?1 \
                     WHERE effect_id = ?2 AND terminal_status IS NULL",
                    params![requested, effect_blob(effect_id)],
                )?;
                if changed != 1 {
                    return Err(MemoryOperationalError::TerminalStatusConflict);
                }
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn record_derivative_generation(
        &mut self,
        generation: &DerivativeIndexGeneration,
    ) -> Result<(), MemoryOperationalError> {
        self.connection.execute(
            r#"INSERT INTO memory_derivative_generations
               (generation_id, store_ref, index_kind_ref, canonical_cut_digest,
                implementation_identity, status, built_at_unix_ms)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
               ON CONFLICT(generation_id) DO NOTHING"#,
            params![
                generation.generation_id.0.bytes().to_vec(),
                self.store_id.0.bytes().to_vec(),
                generation.index_kind_ref.bytes().to_vec(),
                generation.canonical_cut_digest.bytes().to_vec(),
                generation.implementation_identity.bytes().to_vec(),
                derivative_status_code(generation.status),
                to_i64(generation.built_at_unix_ms)?,
            ],
        )?;
        Ok(())
    }

    pub fn current_version(
        &self,
        item_id: MemoryItemId,
    ) -> Result<Option<MemoryVersionId>, MemoryOperationalError> {
        self.connection
            .query_row(
                "SELECT current_version_id FROM memory_items WHERE item_id = ?1 AND store_ref = ?2",
                params![item_id.0.bytes().to_vec(), self.store_id.0.bytes().to_vec()],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()?
            .map(|value| {
                let bytes = hash32(value, "current version id")?;
                Ok(MemoryVersionId(BindingDigest::new(bytes)))
            })
            .transpose()
    }

    pub fn has_blocking_unknown_outcome(&self) -> Result<bool, MemoryOperationalError> {
        Ok(self
            .connection
            .query_row(
                "SELECT 1 FROM memory_effect_state e \
                 LEFT JOIN memory_reconciliation_state r ON r.effect_id = e.effect_id \
                 WHERE e.store_ref = ?1 AND (e.terminal_status = ?2 OR r.state = ?3) LIMIT 1",
                params![
                    self.store_id.0.bytes().to_vec(),
                    mutation_status_code(MemoryMutationStatus::UnknownOutcome),
                    reconciliation_state_code(MemoryReconciliationState::Blocked)
                ],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some())
    }
}

fn migrate(connection: &Connection, store_id: MemoryStoreId) -> Result<(), MemoryOperationalError> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS memory_operational_meta (
            singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
            schema_version INTEGER NOT NULL,
            store_ref BLOB NOT NULL CHECK (length(store_ref) = 32)
        );
        CREATE TABLE IF NOT EXISTS memory_effect_state (
            effect_id BLOB PRIMARY KEY NOT NULL CHECK (length(effect_id) = 16),
            store_ref BLOB NOT NULL CHECK (length(store_ref) = 32),
            intent_digest BLOB NOT NULL CHECK (length(intent_digest) = 32),
            operation INTEGER NOT NULL,
            expected_markdown_target_identity_ref BLOB NOT NULL CHECK (length(expected_markdown_target_identity_ref) = 32),
            expected_markdown_content_digest BLOB NOT NULL CHECK (length(expected_markdown_content_digest) = 32),
            expected_markdown_version_ref BLOB NOT NULL CHECK (length(expected_markdown_version_ref) = 32),
            state TEXT NOT NULL,
            terminal_status INTEGER
        );
        CREATE TABLE IF NOT EXISTS memory_items (
            item_id BLOB PRIMARY KEY NOT NULL CHECK (length(item_id) = 32),
            store_ref BLOB NOT NULL CHECK (length(store_ref) = 32),
            scope INTEGER NOT NULL,
            current_version_id BLOB NOT NULL CHECK (length(current_version_id) = 32),
            state INTEGER NOT NULL,
            last_effect_id BLOB NOT NULL CHECK (length(last_effect_id) = 16),
            last_intent_digest BLOB NOT NULL CHECK (length(last_intent_digest) = 32)
        );
        CREATE TABLE IF NOT EXISTS memory_versions (
            version_id BLOB PRIMARY KEY NOT NULL CHECK (length(version_id) = 32),
            store_ref BLOB NOT NULL CHECK (length(store_ref) = 32),
            item_id BLOB NOT NULL CHECK (length(item_id) = 32),
            scope INTEGER NOT NULL,
            markdown_path TEXT NOT NULL,
            content_digest BLOB NOT NULL CHECK (length(content_digest) = 32),
            provenance_refs BLOB NOT NULL,
            status INTEGER NOT NULL,
            predecessor_refs BLOB NOT NULL,
            conflict_refs BLOB NOT NULL,
            promotion_evidence_ref BLOB NOT NULL CHECK (length(promotion_evidence_ref) = 32),
            created_by_principal TEXT NOT NULL,
            writer_id BLOB NOT NULL CHECK (length(writer_id) = 32),
            effect_id BLOB NOT NULL CHECK (length(effect_id) = 16),
            intent_digest BLOB NOT NULL CHECK (length(intent_digest) = 32),
            created_at_unix_ms INTEGER NOT NULL,
            FOREIGN KEY(effect_id) REFERENCES memory_effect_state(effect_id)
        );
        CREATE TABLE IF NOT EXISTS memory_relations (
            source_version_id BLOB NOT NULL CHECK (length(source_version_id) = 32),
            target_version_id BLOB NOT NULL CHECK (length(target_version_id) = 32),
            relation TEXT NOT NULL,
            store_ref BLOB NOT NULL CHECK (length(store_ref) = 32),
            effect_id BLOB NOT NULL CHECK (length(effect_id) = 16),
            intent_digest BLOB NOT NULL CHECK (length(intent_digest) = 32),
            PRIMARY KEY(source_version_id, target_version_id, relation)
        );
        CREATE TABLE IF NOT EXISTS memory_reconciliation_state (
            effect_id BLOB PRIMARY KEY NOT NULL CHECK (length(effect_id) = 16),
            store_ref BLOB NOT NULL CHECK (length(store_ref) = 32),
            intent_digest BLOB NOT NULL CHECK (length(intent_digest) = 32),
            state INTEGER NOT NULL,
            evidence_ref BLOB NOT NULL CHECK (length(evidence_ref) = 32),
            FOREIGN KEY(effect_id) REFERENCES memory_effect_state(effect_id)
        );
        CREATE TABLE IF NOT EXISTS memory_derivative_generations (
            generation_id BLOB PRIMARY KEY NOT NULL CHECK (length(generation_id) = 32),
            store_ref BLOB NOT NULL CHECK (length(store_ref) = 32),
            index_kind_ref BLOB NOT NULL CHECK (length(index_kind_ref) = 32),
            canonical_cut_digest BLOB NOT NULL CHECK (length(canonical_cut_digest) = 32),
            implementation_identity BLOB NOT NULL CHECK (length(implementation_identity) = 32),
            status INTEGER NOT NULL,
            built_at_unix_ms INTEGER NOT NULL
        );
        "#,
    )?;

    let supported = i64::from(MEMORY_OPERATIONAL_SCHEMA_VERSION);
    let meta = connection
        .query_row(
            "SELECT schema_version, store_ref FROM memory_operational_meta WHERE singleton = 1",
            [],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?;
    match meta {
        None => {
            let tx = connection.unchecked_transaction()?;
            tx.execute_batch(MEMORY_PROMOTION_STATE_SCHEMA)?;
            tx.execute(
                "INSERT INTO memory_operational_meta \
                 (singleton, schema_version, store_ref) VALUES (1, ?1, ?2)",
                params![supported, store_id.0.bytes().to_vec()],
            )?;
            tx.commit()?;
        }
        Some((found, stored)) => {
            if stored != store_id.0.bytes().to_vec() {
                return Err(MemoryOperationalError::StoreBindingMismatch);
            }
            if found > supported {
                return Err(MemoryOperationalError::FutureSchema { found, supported });
            }
            if found == supported {
                return Ok(());
            }
            if found == 1 && supported == 2 {
                let tx = connection.unchecked_transaction()?;
                tx.execute_batch(MEMORY_PROMOTION_STATE_SCHEMA)?;
                tx.execute(
                    "UPDATE memory_operational_meta SET schema_version = ?1 WHERE singleton = 1",
                    params![supported],
                )?;
                tx.commit()?;
            } else {
                return Err(MemoryOperationalError::UnsupportedSchema { found, supported });
            }
        }
    }
    Ok(())
}

fn verify_required_tables(connection: &Connection) -> Result<(), MemoryOperationalError> {
    for table in REQUIRED_MEMORY_OPERATIONAL_TABLES {
        let exists = connection
            .query_row(
                "SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?1 LIMIT 1",
                params![table],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if !exists {
            return Err(MemoryOperationalError::InvalidRecord(
                "required memory operational table missing",
            ));
        }
    }
    Ok(())
}

fn require_prepared(
    tx: &Transaction<'_>,
    store_id: MemoryStoreId,
    effect_id: EffectId,
    intent_digest: BindingDigest,
) -> Result<(), MemoryOperationalError> {
    let row = tx
        .query_row(
            "SELECT store_ref, intent_digest FROM memory_effect_state WHERE effect_id = ?1",
            params![effect_blob(effect_id)],
            |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?
        .ok_or(MemoryOperationalError::MissingPreparedEffect(effect_id))?;
    if row.0 != store_id.0.bytes().to_vec() {
        return Err(MemoryOperationalError::StoreBindingMismatch);
    }
    if row.1 != intent_digest.bytes().to_vec() {
        return Err(MemoryOperationalError::IntentDigestMismatch);
    }
    Ok(())
}

fn require_expected_version(
    tx: &Transaction<'_>,
    prepared: &PreparedMemoryMutationIntent,
    item_id: MemoryItemId,
) -> Result<(), MemoryOperationalError> {
    let expected = prepared
        .intent()
        .expected_current_versions
        .iter()
        .find(|binding| binding.item_id == item_id)
        .ok_or(MemoryOperationalError::InvalidRecord(
            "prepared intent does not bind the version item",
        ))?
        .expected_version;
    let current = tx
        .query_row(
            "SELECT current_version_id FROM memory_items WHERE item_id = ?1",
            params![item_id.0.bytes().to_vec()],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .optional()?;
    match (expected, current) {
        (None, None) => Ok(()),
        (Some(expected), Some(current)) if current == expected.0.bytes().to_vec() => Ok(()),
        _ => Err(MemoryOperationalError::StaleCurrentVersion),
    }
}

fn prepared_digest(
    tx: &Transaction<'_>,
    effect_id: EffectId,
) -> Result<Option<BindingDigest>, MemoryOperationalError> {
    tx.query_row(
        "SELECT intent_digest FROM memory_effect_state WHERE effect_id = ?1",
        params![effect_blob(effect_id)],
        |row| row.get::<_, Vec<u8>>(0),
    )
    .optional()?
    .map(|value| Ok(BindingDigest::new(hash32(value, "prepared intent digest")?)))
    .transpose()
}

fn version_identity(
    tx: &Transaction<'_>,
    version_id: &[u8],
) -> Result<Option<(BindingDigest, EffectId, BindingDigest)>, MemoryOperationalError> {
    tx.query_row(
        "SELECT content_digest, effect_id, intent_digest FROM memory_versions WHERE version_id = ?1",
        params![version_id],
        |row| {
            Ok((
                row.get::<_, Vec<u8>>(0)?,
                row.get::<_, Vec<u8>>(1)?,
                row.get::<_, Vec<u8>>(2)?,
            ))
        },
    )
    .optional()?
    .map(|(content, effect, intent)| {
        let content = BindingDigest::new(hash32(content, "version content digest")?);
        let effect = EffectId(u128::from_be_bytes(id16(effect, "version effect id")?));
        let intent = BindingDigest::new(hash32(intent, "version intent digest")?);
        Ok((content, effect, intent))
    })
    .transpose()
}

fn insert_relation(
    tx: &Transaction<'_>,
    store_id: MemoryStoreId,
    source: BindingDigest,
    target: BindingDigest,
    relation: &str,
    effect_id: EffectId,
    intent_digest: BindingDigest,
) -> Result<(), MemoryOperationalError> {
    tx.execute(
        r#"INSERT OR IGNORE INTO memory_relations
           (source_version_id, target_version_id, relation, store_ref, effect_id, intent_digest)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
        params![
            source.bytes().to_vec(),
            target.bytes().to_vec(),
            relation,
            store_id.0.bytes().to_vec(),
            effect_blob(effect_id),
            intent_digest.bytes().to_vec(),
        ],
    )?;
    Ok(())
}

fn encode_refs(
    values: impl IntoIterator<Item = BindingDigest>,
) -> Result<Vec<u8>, MemoryOperationalError> {
    let values = values.into_iter().collect::<Vec<_>>();
    let mut encoder = CanonicalEncoder::new();
    encoder.push_u64(
        u64::try_from(values.len()).map_err(|_| MemoryOperationalError::IntegerOverflow)?,
    );
    for value in values {
        encoder.push_bytes(&value.bytes())?;
    }
    Ok(encoder.finish())
}

fn hash32(value: Vec<u8>, reason: &'static str) -> Result<[u8; 32], MemoryOperationalError> {
    value
        .try_into()
        .map_err(|_| MemoryOperationalError::InvalidRecord(reason))
}

fn id16(value: Vec<u8>, reason: &'static str) -> Result<[u8; 16], MemoryOperationalError> {
    value
        .try_into()
        .map_err(|_| MemoryOperationalError::InvalidRecord(reason))
}

fn effect_blob(effect_id: EffectId) -> Vec<u8> {
    effect_id.0.to_be_bytes().to_vec()
}

fn to_i64(value: u64) -> Result<i64, MemoryOperationalError> {
    i64::try_from(value).map_err(|_| MemoryOperationalError::IntegerOverflow)
}

const fn operation_code(operation: MemoryOperation) -> i64 {
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

const fn scope_code(scope: golam_core::memory::MemoryScope) -> i64 {
    match scope {
        golam_core::memory::MemoryScope::User => 1,
        golam_core::memory::MemoryScope::Project => 2,
    }
}

const fn version_status_code(status: MemoryVersionStatus) -> i64 {
    match status {
        MemoryVersionStatus::Active => 1,
        MemoryVersionStatus::Superseded => 2,
        MemoryVersionStatus::Contradicted => 3,
        MemoryVersionStatus::Expired => 4,
        MemoryVersionStatus::Forgotten => 5,
        MemoryVersionStatus::Redacted => 6,
    }
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

const fn derivative_status_code(status: DerivativeIndexStatus) -> i64 {
    match status {
        DerivativeIndexStatus::Current => 1,
        DerivativeIndexStatus::Stale => 2,
        DerivativeIndexStatus::Rebuilding => 3,
        DerivativeIndexStatus::Failed => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::memory::{MemoryCandidateId, MemoryScope, MemoryWriterId};
    use golam_core::memory_storage::MemoryLayout;
    use golam_core::paths::RuntimeLayout;
    use golam_core::taint::{TaintLabel, TaintSet};
    use golam_core::tool_request::PrincipalId;
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
            "golam-memory-operational-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    fn prepared(
        store_id: MemoryStoreId,
        effect: u128,
        expected: Option<MemoryVersionId>,
    ) -> PreparedMemoryMutationIntent {
        golam_core::memory::MemoryMutationIntent {
            operation: if expected.is_some() {
                MemoryOperation::Update
            } else {
                MemoryOperation::Add
            },
            item_ids: vec![MemoryItemId(digest(1))],
            expected_current_versions: vec![golam_core::memory::ExpectedMemoryVersion {
                item_id: MemoryItemId(digest(1)),
                expected_version: expected,
            }],
            expected_markdown_target_identity_ref: digest(2),
            expected_markdown_content_digest: digest(3),
            expected_markdown_version: expected.unwrap_or(MemoryVersionId(digest(4))),
            memory_operational_store_ref: store_id,
            candidate_ref: Some(MemoryCandidateId(digest(5))),
            kernel_authorization_ref: digest(6),
            promotion_authority_ref: digest(7),
            effect_id: EffectId(effect),
            reason_ref: digest(8),
            initiating_principal: PrincipalId::new("principal.local").unwrap(),
            created_at_unix_ms: 9,
        }
        .prepare()
        .unwrap()
    }

    fn version(effect: u128, id: u8, predecessor: Option<MemoryVersionId>) -> MemoryVersion {
        MemoryVersion {
            item_id: MemoryItemId(digest(1)),
            version_id: MemoryVersionId(digest(id)),
            scope: MemoryScope::Project,
            canonical_markdown_ref: digest(10),
            content_digest: digest(id.wrapping_add(1)),
            provenance_refs: vec![digest(11)],
            taint_set: TaintSet::from_labels([TaintLabel::UserTrusted]),
            status: MemoryVersionStatus::Active,
            predecessor_versions: predecessor.into_iter().collect(),
            conflict_refs: vec![],
            promotion_evidence_ref: digest(12),
            created_by_principal: PrincipalId::new("principal.creator").unwrap(),
            committed_by_writer_identity: MemoryWriterId(digest(13)),
            mutation_effect_ref: EffectId(effect),
            created_at_unix_ms: 14,
        }
    }

    #[test]
    fn store_binds_exact_layout_identity_and_never_authority_db() {
        let runtime = runtime();
        let layout = MemoryLayout::initialize(&runtime).unwrap();
        let store = MemoryOperationalStore::open(&layout).unwrap();
        assert_eq!(store.store_id(), layout.store_id());
        assert_ne!(
            layout.operational_db_path(),
            runtime.data_dir.join("authority/golam.db")
        );
        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn prepared_and_version_rows_bind_store_effect_intent_and_expected_version() {
        let store_id = MemoryStoreId(digest(20));
        let mut store = MemoryOperationalStore::open_in_memory(store_id).unwrap();
        let first_prepared = prepared(store_id, 21, None);
        store.record_prepared(&first_prepared).unwrap();
        let first = version(21, 30, None);
        store
            .record_version(&first_prepared, &first, Path::new("memory/item.md"))
            .unwrap();
        assert_eq!(
            store.current_version(MemoryItemId(digest(1))).unwrap(),
            Some(MemoryVersionId(digest(30)))
        );

        let stale = prepared(store_id, 22, Some(MemoryVersionId(digest(99))));
        store.record_prepared(&stale).unwrap();
        let second = version(22, 31, Some(MemoryVersionId(digest(99))));
        assert!(matches!(
            store.record_version(&stale, &second, Path::new("memory/item.md")),
            Err(MemoryOperationalError::StaleCurrentVersion)
        ));
    }

    #[test]
    fn unknown_outcome_and_blocked_reconciliation_block_dependent_memory_work() {
        let store_id = MemoryStoreId(digest(40));
        let mut store = MemoryOperationalStore::open_in_memory(store_id).unwrap();
        let first = prepared(store_id, 41, None);
        let first_digest = BindingDigest::new(first.binding_digest());
        store.record_prepared(&first).unwrap();
        assert!(!store.has_blocking_unknown_outcome().unwrap());
        store
            .mark_terminal(
                EffectId(41),
                first_digest,
                MemoryMutationStatus::UnknownOutcome,
            )
            .unwrap();
        assert!(store.has_blocking_unknown_outcome().unwrap());

        let store_id = MemoryStoreId(digest(42));
        let mut store = MemoryOperationalStore::open_in_memory(store_id).unwrap();
        let second = prepared(store_id, 43, None);
        let second_digest = BindingDigest::new(second.binding_digest());
        store.record_prepared(&second).unwrap();
        store
            .record_reconciliation(
                EffectId(43),
                second_digest,
                MemoryReconciliationState::Blocked,
                digest(44),
            )
            .unwrap();
        assert!(store.has_blocking_unknown_outcome().unwrap());
    }

    #[test]
    fn terminal_status_is_one_shot_and_idempotent_for_same_status() {
        let store_id = MemoryStoreId(digest(45));
        let mut store = MemoryOperationalStore::open_in_memory(store_id).unwrap();
        let prepared = prepared(store_id, 46, None);
        let intent_digest = BindingDigest::new(prepared.binding_digest());
        store.record_prepared(&prepared).unwrap();
        store
            .mark_terminal(EffectId(46), intent_digest, MemoryMutationStatus::Committed)
            .unwrap();
        store
            .mark_terminal(EffectId(46), intent_digest, MemoryMutationStatus::Committed)
            .unwrap();
        assert!(matches!(
            store.mark_terminal(
                EffectId(46),
                intent_digest,
                MemoryMutationStatus::UnknownOutcome
            ),
            Err(MemoryOperationalError::TerminalStatusConflict)
        ));
    }

    #[test]
    fn legacy_v1_schema_migrates_atomically_to_v2() {
        let store_id = MemoryStoreId(digest(47));
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE memory_operational_meta (\
                    singleton INTEGER PRIMARY KEY CHECK (singleton = 1), \
                    schema_version INTEGER NOT NULL, \
                    store_ref BLOB NOT NULL CHECK (length(store_ref) = 32)\
                 );",
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO memory_operational_meta (singleton, schema_version, store_ref) \
                 VALUES (1, 1, ?1)",
                params![store_id.0.bytes().to_vec()],
            )
            .unwrap();
        let store = MemoryOperationalStore::initialize(connection, store_id).unwrap();
        let version: i64 = store
            .connection
            .query_row(
                "SELECT schema_version FROM memory_operational_meta WHERE singleton = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, 2);
        let promotion_table: i64 = store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table' AND name = 'memory_promotion_state'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(promotion_table, 1);
    }

    #[test]
    fn wrong_store_binding_fails_closed() {
        let expected = MemoryStoreId(digest(50));
        let other = MemoryStoreId(digest(51));
        let mut store = MemoryOperationalStore::open_in_memory(expected).unwrap();
        assert!(matches!(
            store.record_prepared(&prepared(other, 52, None)),
            Err(MemoryOperationalError::StoreBindingMismatch)
        ));
    }
}
