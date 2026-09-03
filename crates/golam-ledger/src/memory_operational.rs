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
use golam_core::{CanonicalEncoder, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

pub const REQUIRED_MEMORY_OPERATIONAL_TABLES: &[&str] = &[
    "memory_operational_meta",
    "memory_effect_state",
    "memory_items",
    "memory_versions",
    "memory_relations",
    "memory_reconciliation_state",
    "memory_derivative_generations",
];

#[derive(Debug)]
pub enum MemoryOperationalError {
    Sqlite(rusqlite::Error),
    InvalidRecord(&'static str),
    StoreBindingMismatch,
    FutureSchema { found: i64, supported: i64 },
    MissingPreparedEffect(EffectId),
    IntentDigestMismatch,
    ImmutableVersionMismatch,
    StaleCurrentVersion,
    NonUnicodePath,
    IntegerOverflow,
}

impl fmt::Display for MemoryOperationalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "memory operational sqlite error: {error}"),
            Self::InvalidRecord(reason) => write!(f, "invalid memory operational record: {reason}"),
            Self::StoreBindingMismatch => f.write_str(
                "memory mutation or database does not bind the exact operational store identity",
            ),
            Self::FutureSchema { found, supported } => write!(
                f,
                "memory operational schema {found} is newer than supported {supported}"
            ),
            Self::MissingPreparedEffect(effect_id) => write!(
                f,
                "memory operational effect {} has no PREPARED state",
                effect_id.0
            ),
            Self::IntentDigestMismatch => {
                f.write_str("memory operational row does not bind the PREPARED intent digest")
            }
            Self::ImmutableVersionMismatch => {
                f.write_str("memory version identity already exists with different protected state")
            }
            Self::StaleCurrentVersion => {
                f.write_str("memory current version does not match the prepared expected version")
            }
            Self::NonUnicodePath => {
                f.write_str("canonical managed Markdown path is not valid UTF-8")
            }
            Self::IntegerOverflow => f.write_str("memory operational integer conversion overflow"),
        }
    }
}

impl Error for MemoryOperationalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for MemoryOperationalError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub struct MemoryOperationalStore {
    connection: Connection,
    store_id: MemoryStoreId,
}

impl MemoryOperationalStore {
    pub fn open(layout: &MemoryLayout) -> Result<Self, MemoryOperationalError> {
        Self::initialize(Connection::open(layout.operational_db_path())?, layout.store_id())
    }

    pub fn open_in_memory(store_id: MemoryStoreId) -> Result<Self, MemoryOperationalError> {
        Self::initialize(Connection::open_in_memory()?, store_id)
    }

    fn initialize(
        connection: Connection,
        store_id: MemoryStoreId,
    ) -> Result<Self, MemoryOperationalError> {
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
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
        if let Some(existing) = effect_intent_digest(&tx, intent.effect_id)? {
            if existing != digest.bytes() {
                return Err(MemoryOperationalError::IntentDigestMismatch);
            }
            tx.commit()?;
            return Ok(());
        }
        tx.execute(
            r#"INSERT INTO memory_effect_state
               (effect_id, store_ref, intent_digest, operation, expected_markdown_target_identity_ref,
                expected_markdown_content_digest, expected_markdown_version_ref, state, terminal_status)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'prepared', NULL)"#,
            params![
                &intent.effect_id.0.to_be_bytes()[..],
                &self.store_id.0.bytes()[..],
                &digest.bytes()[..],
                operation_code(intent.operation),
                &intent.expected_markdown_target_identity_ref.bytes()[..],
                &intent.expected_markdown_content_digest.bytes()[..],
                &intent.expected_markdown_version.0.bytes()[..],
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
        if prepared.intent().memory_operational_store_ref != self.store_id
            || version.mutation_effect_ref != prepared.intent().effect_id
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
        require_prepared(&tx, self.store_id, version.mutation_effect_ref, intent_digest)?;
        require_expected_current_version(&tx, prepared, version.item_id)?;

        let version_id = version.version_id.0.bytes();
        let existing = tx
            .query_row(
                "SELECT content_digest, effect_id, intent_digest FROM memory_versions WHERE version_id = ?1",
                params![&version_id[..]],
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
            if existing.0.as_slice() == version.content_digest.bytes()
                && existing.1.as_slice() == version.mutation_effect_ref.0.to_be_bytes()
                && existing.2.as_slice() == intent_digest.bytes()
            {
                tx.commit()?;
                return Ok(());
            }
            return Err(MemoryOperationalError::ImmutableVersionMismatch);
        }

        let provenance = encode_digests(
            version
                .provenance_refs
                .iter()
                .copied(),
        )?;
        let predecessors = encode_digests(
            version
                .predecessor_versions
                .iter()
                .map(|value| value.0),
        )?;
        let conflicts = encode_digests(version.conflict_refs.iter().copied())?;
        tx.execute(
            r#"INSERT INTO memory_versions
               (version_id, store_ref, item_id, scope, markdown_path, content_digest,
                provenance_refs, status, predecessor_refs, conflict_refs, promotion_evidence_ref,
                created_by_principal, writer_id, effect_id, intent_digest, created_at_unix_ms)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)"#,
            params![
                &version_id[..],
                &self.store_id.0.bytes()[..],
                &version.item_id.0.bytes()[..],
                scope_code(version.scope),
                path,
                &version.content_digest.bytes()[..],
                provenance,
                version_status_code(version.status),
                predecessors,
                conflicts,
                &version.promotion_evidence_ref.bytes()[..],
                version.created_by_principal.as_str(),
                &version.committed_by_writer_identity.0.bytes()[..],
                &version.mutation_effect_ref.0.to_be_bytes()[..],
                &intent_digest.bytes()[..],
                to_i64(version.created_at_unix_ms)?,
            ],
        )?;
        tx.execute(
            r#"INSERT INTO memory_items
               (item_id, store_ref, scope, current_version_id, state, last_effect_id, last_intent_digest)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
               ON CONFLICT(item_id) DO UPDATE SET
                 current_version_id = excluded.current_version_id,
                 state = excluded.state,
                 last_effect_id = excluded.last_effect_id,
                 last_intent_digest = excluded.last_intent_digest"#,
            params![
                &version.item_id.0.bytes()[..],
                &self.store_id.0.bytes()[..],
                scope_code(version.scope),
                &version_id[..],
                version_status_code(version.status),
                &version.mutation_effect_ref.0.to_be_bytes()[..],
                &intent_digest.bytes()[..],
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
                &effect_id.0.to_be_bytes()[..],
                &self.store_id.0.bytes()[..],
                &intent_digest.bytes()[..],
                reconciliation_state_code(state),
                &evidence_ref.bytes()[..],
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
        tx.execute(
            "UPDATE memory_effect_state SET state = 'terminal', terminal_status = ?1 WHERE effect_id = ?2",
            params![mutation_status_code(status), &effect_id.0.to_be_bytes()[..]],
        )?;
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
                &generation.generation_id.0.bytes()[..],
                &self.store_id.0.bytes()[..],
                &generation.index_kind_ref.bytes()[..],
                &generation.canonical_cut_digest.bytes()[..],
                &generation.implementation_identity.bytes()[..],
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
                params![&item_id.0.bytes()[..], &self.store_id.0.bytes()[..]],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()?
            .map(|value| {
                let bytes: [u8; 32] = value
                    .try_into()
                    .map_err(|_| MemoryOperationalError::InvalidRecord("current version id"))?;
                Ok(MemoryVersionId(BindingDigest::new(bytes)))
            })
            .transpose()
    }

    pub fn has_blocking_unknown_outcome(&self) -> Result<bool, MemoryOperationalError> {
        Ok(self
            .connection
            .query_row(
                "SELECT 1 FROM memory_effect_state WHERE store_ref = ?1 AND terminal_status = ?2 LIMIT 1",
                params![
                    &self.store_id.0.bytes()[..],
                    mutation_status_code(MemoryMutationStatus::UnknownOutcome)
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
            connection.execute(
                "INSERT INTO memory_operational_meta (singleton, schema_version, store_ref) VALUES (1, ?1, ?2)",
                params![supported, &store_id.0.bytes()[..]],
            )?;
        }
        Some((found, stored)) => {
            if found > supported {
                return Err(MemoryOperationalError::FutureSchema { found, supported });
            }
            if found != supported || stored.as_slice() != store_id.0.bytes() {
                return Err(MemoryOperationalError::StoreBindingMismatch);
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
            params![&effect_id.0.to_be_bytes()[..]],
            |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?
        .ok_or(MemoryOperationalError::MissingPreparedEffect(effect_id))?;
    if row.0.as_slice() != store_id.0.bytes() {
        return Err(MemoryOperationalError::StoreBindingMismatch);
    }
    if row.1.as_slice() != intent_digest.bytes() {
        return Err(MemoryOperationalError::IntentDigestMismatch);
    }
    Ok(())
}

fn require_expected_current_version(
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
            params![&item_id.0.bytes()[..]],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .optional()?;
    match (expected, current) {
        (None, None) => Ok(()),
        (Some(expected), Some(current)) if current.as_slice() == expected.0.bytes() => Ok(()),
        _ => Err(MemoryOperationalError::StaleCurrentVersion),
    }
}

fn effect_intent_digest(
    tx: &Transaction<'_>,
    effect_id: EffectId,
) -> Result<Option<[u8; 32]>, MemoryOperationalError> {
    tx.query_row(
        "SELECT intent_digest FROM memory_effect_state WHERE effect_id = ?1",
        params![&effect_id.0.to_be_bytes()[..]],
        |row| row.get::<_, Vec<u8>>(0),
    )
    .optional()?
    .map(|value| {
        value
            .try_into()
            .map_err(|_| MemoryOperationalError::InvalidRecord("stored intent digest"))
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
            &source.bytes()[..],
            &target.bytes()[..],
            relation,
            &store_id.0.bytes()[..],
            &effect_id.0.to_be_bytes()[..],
            &intent_digest.bytes()[..],
        ],
    )?;
    Ok(())
}

fn encode_digests(
    values: impl IntoIterator<Item = BindingDigest>,
) -> Result<Vec<u8>, MemoryOperationalError> {
    let values = values.into_iter().collect::<Vec<_>>();
    let mut encoder = CanonicalEncoder::new();
    encoder.push_u64(
        u64::try_from(values.len()).map_err(|_| MemoryOperationalError::IntegerOverflow)?,
    );
    for value in values {
        encoder
            .push_bytes(&value.bytes())
            .map_err(|_| MemoryOperationalError::InvalidRecord("digest vector"))?;
    }
    Ok(encoder.finish())
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

fn to_i64(value: u64) -> Result<i64, MemoryOperationalError> {
    i64::try_from(value).map_err(|_| MemoryOperationalError::IntegerOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::memory::{
        ExpectedMemoryVersion, MemoryCandidateId, MemoryScope, MemoryWriterId,
    };
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

    fn prepared(store_id: MemoryStoreId, effect: u128, expected: Option<MemoryVersionId>) -> PreparedMemoryMutationIntent {
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
    fn unknown_outcome_blocks_dependent_memory_work() {
        let store_id = MemoryStoreId(digest(40));
        let mut store = MemoryOperationalStore::open_in_memory(store_id).unwrap();
        let prepared = prepared(store_id, 41, None);
        let intent_digest = BindingDigest::new(prepared.binding_digest());
        store.record_prepared(&prepared).unwrap();
        assert!(!store.has_blocking_unknown_outcome().unwrap());
        store
            .mark_terminal(
                EffectId(41),
                intent_digest,
                MemoryMutationStatus::UnknownOutcome,
            )
            .unwrap();
        assert!(store.has_blocking_unknown_outcome().unwrap());
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
