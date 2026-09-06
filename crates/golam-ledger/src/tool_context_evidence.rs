#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::Path;

use golam_core::context_evidence::{ContextEvidence, EvidenceAuthorityClass, EvidenceSourceKind};
use golam_core::tool_request::{ToolRequest, ToolRequestId, ToolResult};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

pub const TOOL_CONTEXT_EVIDENCE_SCHEMA_VERSION: i64 = 1;

pub const REQUIRED_TOOL_CONTEXT_TABLES: &[&str] = &[
    "tool_context_schema_meta",
    "tool_request_evidence",
    "tool_result_evidence",
    "context_provenance_evidence",
];

#[derive(Debug)]
pub enum ToolContextEvidenceError {
    Sqlite(rusqlite::Error),
    InvalidRecord(&'static str),
    FutureSchema { found: i64, supported: i64 },
    MissingToolRequest(ToolRequestId),
    ImmutableEvidenceMismatch(&'static str),
    IntegerOverflow,
}

impl fmt::Display for ToolContextEvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "tool/context evidence sqlite error: {error}"),
            Self::InvalidRecord(reason) => write!(f, "invalid tool/context evidence: {reason}"),
            Self::FutureSchema { found, supported } => write!(
                f,
                "tool/context evidence schema {found} is newer than supported {supported}"
            ),
            Self::MissingToolRequest(request_id) => write!(
                f,
                "tool result references missing request {}",
                request_id.as_u128()
            ),
            Self::ImmutableEvidenceMismatch(kind) => {
                write!(f, "immutable {kind} evidence identity collision")
            }
            Self::IntegerOverflow => f.write_str("integer conversion overflow"),
        }
    }
}

impl Error for ToolContextEvidenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for ToolContextEvidenceError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub struct ToolContextEvidenceStore {
    connection: Connection,
}

impl ToolContextEvidenceStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ToolContextEvidenceError> {
        Self::initialize(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self, ToolContextEvidenceError> {
        Self::initialize(Connection::open_in_memory()?)
    }

    fn initialize(connection: Connection) -> Result<Self, ToolContextEvidenceError> {
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        migrate(&connection)?;
        verify_required_tables(&connection)?;
        Ok(Self { connection })
    }

    pub fn schema_version(&self) -> Result<i64, ToolContextEvidenceError> {
        Ok(self.connection.query_row(
            "SELECT schema_version FROM tool_context_schema_meta WHERE singleton = 1",
            [],
            |row| row.get(0),
        )?)
    }

    pub fn persist_tool_request(
        &mut self,
        request: &ToolRequest,
    ) -> Result<(), ToolContextEvidenceError> {
        let canonical_bytes = request
            .canonical_bytes()
            .map_err(|_| ToolContextEvidenceError::InvalidRecord("tool request"))?;
        let binding_digest = request
            .binding_digest()
            .map_err(|_| ToolContextEvidenceError::InvalidRecord("tool request digest"))?;
        let integrity_hash = crate::payload_hash(&canonical_bytes);
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed = tx.execute(
            r#"INSERT INTO tool_request_evidence
               (request_id, binding_digest, canonical_bytes, integrity_hash)
               VALUES (?1, ?2, ?3, ?4)
               ON CONFLICT(request_id) DO UPDATE SET request_id = excluded.request_id
               WHERE tool_request_evidence.binding_digest = excluded.binding_digest
                 AND tool_request_evidence.canonical_bytes = excluded.canonical_bytes
                 AND tool_request_evidence.integrity_hash = excluded.integrity_hash"#,
            params![
                id_blob(request.request_id.as_u128()),
                &binding_digest[..],
                canonical_bytes,
                &integrity_hash[..],
            ],
        )?;
        if changed != 1 {
            return Err(ToolContextEvidenceError::ImmutableEvidenceMismatch(
                "tool request",
            ));
        }
        tx.commit()?;
        Ok(())
    }

    pub fn persist_tool_result(
        &mut self,
        result: &ToolResult,
    ) -> Result<(), ToolContextEvidenceError> {
        let canonical_bytes = result
            .canonical_bytes()
            .map_err(|_| ToolContextEvidenceError::InvalidRecord("tool result"))?;
        let evidence_digest = result
            .evidence_digest()
            .map_err(|_| ToolContextEvidenceError::InvalidRecord("tool result digest"))?;
        let integrity_hash = crate::payload_hash(&canonical_bytes);
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let request_exists = tx
            .query_row(
                "SELECT 1 FROM tool_request_evidence WHERE request_id = ?1 LIMIT 1",
                params![id_blob(result.request_id.as_u128())],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if !request_exists {
            return Err(ToolContextEvidenceError::MissingToolRequest(
                result.request_id,
            ));
        }
        let changed = tx.execute(
            r#"INSERT INTO tool_result_evidence
               (request_id, evidence_digest, canonical_bytes, integrity_hash)
               VALUES (?1, ?2, ?3, ?4)
               ON CONFLICT(request_id) DO UPDATE SET request_id = excluded.request_id
               WHERE tool_result_evidence.evidence_digest = excluded.evidence_digest
                 AND tool_result_evidence.canonical_bytes = excluded.canonical_bytes
                 AND tool_result_evidence.integrity_hash = excluded.integrity_hash"#,
            params![
                id_blob(result.request_id.as_u128()),
                &evidence_digest[..],
                canonical_bytes,
                &integrity_hash[..],
            ],
        )?;
        if changed != 1 {
            return Err(ToolContextEvidenceError::ImmutableEvidenceMismatch(
                "tool result",
            ));
        }
        tx.commit()?;
        Ok(())
    }

    pub fn persist_context_evidence(
        &mut self,
        evidence: &ContextEvidence,
        observed_now_unix_ms: u64,
        record_bytes: &[u8],
    ) -> Result<(), ToolContextEvidenceError> {
        evidence
            .validate(observed_now_unix_ms)
            .map_err(|_| ToolContextEvidenceError::InvalidRecord("context provenance"))?;
        if record_bytes.is_empty() {
            return Err(ToolContextEvidenceError::InvalidRecord(
                "context provenance bytes",
            ));
        }
        let taint_bytes = evidence
            .taint_set
            .canonical_bytes()
            .map_err(|_| ToolContextEvidenceError::InvalidRecord("context taint"))?;
        let integrity_hash = crate::payload_hash(record_bytes);
        let changed = self.connection.execute(
            r#"INSERT INTO context_provenance_evidence
               (evidence_id, source_id, source_kind, source_version_or_observation,
                content_ref, content_digest, authority_class, taint_bytes,
                permission_scope, observed_at_unix_ms, record_bytes, integrity_hash)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
               ON CONFLICT(evidence_id) DO UPDATE SET evidence_id = excluded.evidence_id
               WHERE context_provenance_evidence.source_id = excluded.source_id
                 AND context_provenance_evidence.source_kind = excluded.source_kind
                 AND context_provenance_evidence.source_version_or_observation = excluded.source_version_or_observation
                 AND context_provenance_evidence.content_ref = excluded.content_ref
                 AND context_provenance_evidence.content_digest = excluded.content_digest
                 AND context_provenance_evidence.authority_class = excluded.authority_class
                 AND context_provenance_evidence.taint_bytes = excluded.taint_bytes
                 AND context_provenance_evidence.permission_scope = excluded.permission_scope
                 AND context_provenance_evidence.observed_at_unix_ms = excluded.observed_at_unix_ms
                 AND context_provenance_evidence.record_bytes = excluded.record_bytes
                 AND context_provenance_evidence.integrity_hash = excluded.integrity_hash"#,
            params![
                &evidence.evidence_id.bytes()[..],
                &evidence.source_id.0.bytes()[..],
                source_kind_code(evidence.source_kind),
                &evidence.source_version_or_observation.bytes()[..],
                &evidence.content_ref.bytes()[..],
                &evidence.content_digest.bytes()[..],
                authority_class_code(evidence.authority_class),
                taint_bytes,
                &evidence.permission_scope.0.bytes()[..],
                u64_to_i64(evidence.observed_at_unix_ms)?,
                record_bytes,
                &integrity_hash[..],
            ],
        )?;
        if changed != 1 {
            return Err(ToolContextEvidenceError::ImmutableEvidenceMismatch(
                "context provenance",
            ));
        }
        Ok(())
    }

    pub fn tool_request_integrity_hash(
        &self,
        request_id: ToolRequestId,
    ) -> Result<Option<[u8; 32]>, ToolContextEvidenceError> {
        let value = self
            .connection
            .query_row(
                "SELECT integrity_hash FROM tool_request_evidence WHERE request_id = ?1",
                params![id_blob(request_id.as_u128())],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()?;
        value.map(vec_to_hash).transpose()
    }
}

fn migrate(connection: &Connection) -> Result<(), ToolContextEvidenceError> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS tool_context_schema_meta (
            singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
            schema_version INTEGER NOT NULL
        );
        INSERT OR IGNORE INTO tool_context_schema_meta (singleton, schema_version) VALUES (1, 1);

        CREATE TABLE IF NOT EXISTS tool_request_evidence (
            request_id BLOB PRIMARY KEY NOT NULL CHECK (length(request_id) = 16),
            binding_digest BLOB NOT NULL CHECK (length(binding_digest) = 32),
            canonical_bytes BLOB NOT NULL,
            integrity_hash BLOB NOT NULL CHECK (length(integrity_hash) = 32)
        );

        CREATE TABLE IF NOT EXISTS tool_result_evidence (
            request_id BLOB PRIMARY KEY NOT NULL CHECK (length(request_id) = 16),
            evidence_digest BLOB NOT NULL CHECK (length(evidence_digest) = 32),
            canonical_bytes BLOB NOT NULL,
            integrity_hash BLOB NOT NULL CHECK (length(integrity_hash) = 32),
            FOREIGN KEY(request_id) REFERENCES tool_request_evidence(request_id)
        );

        CREATE TABLE IF NOT EXISTS context_provenance_evidence (
            evidence_id BLOB PRIMARY KEY NOT NULL CHECK (length(evidence_id) = 32),
            source_id BLOB NOT NULL CHECK (length(source_id) = 32),
            source_kind INTEGER NOT NULL,
            source_version_or_observation BLOB NOT NULL CHECK (length(source_version_or_observation) = 32),
            content_ref BLOB NOT NULL CHECK (length(content_ref) = 32),
            content_digest BLOB NOT NULL CHECK (length(content_digest) = 32),
            authority_class INTEGER NOT NULL,
            taint_bytes BLOB NOT NULL,
            permission_scope BLOB NOT NULL CHECK (length(permission_scope) = 32),
            observed_at_unix_ms INTEGER NOT NULL,
            record_bytes BLOB NOT NULL,
            integrity_hash BLOB NOT NULL CHECK (length(integrity_hash) = 32)
        );
        "#,
    )?;
    let version: i64 = connection.query_row(
        "SELECT schema_version FROM tool_context_schema_meta WHERE singleton = 1",
        [],
        |row| row.get(0),
    )?;
    if version > TOOL_CONTEXT_EVIDENCE_SCHEMA_VERSION {
        return Err(ToolContextEvidenceError::FutureSchema {
            found: version,
            supported: TOOL_CONTEXT_EVIDENCE_SCHEMA_VERSION,
        });
    }
    Ok(())
}

fn verify_required_tables(connection: &Connection) -> Result<(), ToolContextEvidenceError> {
    for table in REQUIRED_TOOL_CONTEXT_TABLES {
        let present = connection
            .query_row(
                "SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?1 LIMIT 1",
                params![table],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if !present {
            return Err(ToolContextEvidenceError::InvalidRecord(
                "required tool/context table missing",
            ));
        }
    }
    Ok(())
}

fn id_blob(value: u128) -> [u8; 16] {
    value.to_be_bytes()
}

fn u64_to_i64(value: u64) -> Result<i64, ToolContextEvidenceError> {
    i64::try_from(value).map_err(|_| ToolContextEvidenceError::IntegerOverflow)
}

fn vec_to_hash(value: Vec<u8>) -> Result<[u8; 32], ToolContextEvidenceError> {
    value
        .try_into()
        .map_err(|_| ToolContextEvidenceError::InvalidRecord("stored integrity hash"))
}

const fn source_kind_code(kind: EvidenceSourceKind) -> i64 {
    match kind {
        EvidenceSourceKind::UserSelectedArtifact => 1,
        EvidenceSourceKind::File => 2,
        EvidenceSourceKind::GitObject => 3,
        EvidenceSourceKind::CanonicalLedger => 4,
        EvidenceSourceKind::ManagedMemory => 5,
        EvidenceSourceKind::ProtocolResource => 6,
        EvidenceSourceKind::ExternalDocument => 7,
    }
}

const fn authority_class_code(class: EvidenceAuthorityClass) -> i64 {
    match class {
        EvidenceAuthorityClass::UntrustedContent => 1,
        EvidenceAuthorityClass::UserAttributed => 2,
        EvidenceAuthorityClass::LocalObserved => 3,
        EvidenceAuthorityClass::CanonicalGolam => 4,
        EvidenceAuthorityClass::ExternalAuthoritative => 5,
    }
}

#[cfg(test)]
mod tests {
    use golam_core::context_evidence::{EvidenceSourceId, FreshnessPolicy, PermissionScopeId};
    use golam_core::harness::ToolCallCandidateId;
    use golam_core::taint::{TaintLabel, TaintSet};
    use golam_core::tool_descriptor::{ToolId, ToolVersion};
    use golam_core::tool_request::{
        BindingDigest, PrincipalId, RequestedOperationId, RequestedTarget, ResourceClassId,
        ToolResultStatus,
    };

    use super::*;

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn request() -> ToolRequest {
        ToolRequest {
            request_id: ToolRequestId::from_u128(7),
            initiating_principal: PrincipalId::new("principal.local").unwrap(),
            tool_id: ToolId::new("fs.read").unwrap(),
            tool_version: ToolVersion::new("1.0.0").unwrap(),
            candidate_ref: ToolCallCandidateId::from_u128(9),
            requested_operation: RequestedOperationId::new("read").unwrap(),
            requested_target: Some(RequestedTarget::new("src/lib.rs").unwrap()),
            authorized_resource_class: ResourceClassId::new("workspace.read").unwrap(),
            target_identity_ref: Some(digest(1)),
            target_resolution_plan_ref: None,
            capability_context_ref: digest(2),
            taint_set: TaintSet::from_labels([TaintLabel::LocalTrusted]),
            provenance_refs: vec![digest(3)],
            idempotency_material: digest(4),
            current_preconditions: vec![digest(5)],
            created_at_unix_ms: 10,
        }
    }

    #[test]
    fn request_evidence_is_immutable_and_idempotent() {
        let mut store = ToolContextEvidenceStore::open_in_memory().unwrap();
        let first = request();
        store.persist_tool_request(&first).unwrap();
        store.persist_tool_request(&first).unwrap();

        let mut changed = first;
        changed.created_at_unix_ms += 1;
        assert!(matches!(
            store.persist_tool_request(&changed),
            Err(ToolContextEvidenceError::ImmutableEvidenceMismatch(
                "tool request"
            ))
        ));
    }

    #[test]
    fn result_requires_durable_request_and_is_immutable() {
        let mut store = ToolContextEvidenceStore::open_in_memory().unwrap();
        let result = ToolResult {
            request_id: ToolRequestId::from_u128(7),
            status: ToolResultStatus::Succeeded,
            observed_target_identity: Some(digest(1)),
            output_artifact_refs: vec![digest(6)],
            stdout_or_text_ref: None,
            stderr_or_error_ref: None,
            external_effect_refs: vec![],
            verification_refs: vec![digest(7)],
            taint_set: TaintSet::from_labels([TaintLabel::LocalTrusted]),
            started_at_unix_ms: 11,
            terminal_at_unix_ms: 12,
        };
        assert!(matches!(
            store.persist_tool_result(&result),
            Err(ToolContextEvidenceError::MissingToolRequest(_))
        ));
        store.persist_tool_request(&request()).unwrap();
        store.persist_tool_result(&result).unwrap();
        store.persist_tool_result(&result).unwrap();
    }

    #[test]
    fn context_provenance_preserves_taint_and_integrity() {
        let mut store = ToolContextEvidenceStore::open_in_memory().unwrap();
        let evidence = ContextEvidence {
            evidence_id: digest(20),
            source_id: EvidenceSourceId(digest(21)),
            source_kind: EvidenceSourceKind::File,
            source_version_or_observation: digest(22),
            content_ref: digest(23),
            content_digest: digest(24),
            authority_class: EvidenceAuthorityClass::LocalObserved,
            taint_set: TaintSet::from_labels([TaintLabel::LocalUnverified]),
            permission_scope: PermissionScopeId(digest(25)),
            freshness_policy: FreshnessPolicy::MaxAgeMs(100),
            observed_at_unix_ms: 950,
            supersedes_or_conflicts_with: vec![],
        };
        store
            .persist_context_evidence(&evidence, 1_000, b"context-record")
            .unwrap();
        store
            .persist_context_evidence(&evidence, 1_000, b"context-record")
            .unwrap();
        assert_eq!(store.schema_version().unwrap(), 1);
    }
}
