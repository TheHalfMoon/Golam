#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::desktop_control::{
    DesktopControlError, DesktopControlLeaseId, DesktopControlLeaseState, DesktopControlMode,
    HumanInterruptEvidence, HumanInterruptOperation, VisibleControlChannelId,
    VisibleControlChannelKind, VisibleControlChannelState,
};
use golam_core::tool_request::BindingDigest;
use golam_core::{CanonicalEncoder, CoreError, EffectId, SessionId};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

const EFFECT_EVIDENCE_DOMAIN: &[u8] = b"golam:desktop-effect-evidence:v1";
const LEASE_STATE_DOMAIN: &[u8] = b"golam:desktop-durable-lease-state:v1";
const VISIBLE_CHANNEL_DOMAIN: &[u8] = b"golam:desktop-durable-visible-channel:v1";
const INTERRUPT_DOMAIN: &[u8] = b"golam:desktop-durable-interrupt:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopEvidenceOperation {
    SemanticAction,
    Focus,
    RawInputFallback,
    Capture,
    ClipboardRead,
    ClipboardWrite,
}

impl DesktopEvidenceOperation {
    const fn code(self) -> i64 {
        match self {
            Self::SemanticAction => 1,
            Self::Focus => 2,
            Self::RawInputFallback => 3,
            Self::Capture => 4,
            Self::ClipboardRead => 5,
            Self::ClipboardWrite => 6,
        }
    }

    fn from_code(value: i64) -> Result<Self, DesktopControlEvidenceError> {
        match value {
            1 => Ok(Self::SemanticAction),
            2 => Ok(Self::Focus),
            3 => Ok(Self::RawInputFallback),
            4 => Ok(Self::Capture),
            5 => Ok(Self::ClipboardRead),
            6 => Ok(Self::ClipboardWrite),
            _ => Err(DesktopControlEvidenceError::InvalidStoredRecord(
                "operation code",
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopEvidenceStatus {
    Prepared,
    Succeeded,
    Failed,
    UnknownOutcome,
    Interrupted,
    Reconciling,
    ReconciledSucceeded,
    ReconciledFailed,
    ManualReview,
}

impl DesktopEvidenceStatus {
    const fn code(self) -> i64 {
        match self {
            Self::Prepared => 1,
            Self::Succeeded => 2,
            Self::Failed => 3,
            Self::UnknownOutcome => 4,
            Self::Interrupted => 5,
            Self::Reconciling => 6,
            Self::ReconciledSucceeded => 7,
            Self::ReconciledFailed => 8,
            Self::ManualReview => 9,
        }
    }

    fn from_code(value: i64) -> Result<Self, DesktopControlEvidenceError> {
        match value {
            1 => Ok(Self::Prepared),
            2 => Ok(Self::Succeeded),
            3 => Ok(Self::Failed),
            4 => Ok(Self::UnknownOutcome),
            5 => Ok(Self::Interrupted),
            6 => Ok(Self::Reconciling),
            7 => Ok(Self::ReconciledSucceeded),
            8 => Ok(Self::ReconciledFailed),
            9 => Ok(Self::ManualReview),
            _ => Err(DesktopControlEvidenceError::InvalidStoredRecord(
                "status code",
            )),
        }
    }

    pub const fn unresolved(self) -> bool {
        matches!(Self::UnknownOutcome | Self::Reconciling, self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopEffectEvidence {
    pub effect_id: EffectId,
    pub session_id: SessionId,
    pub operation: DesktopEvidenceOperation,
    pub request_digest: BindingDigest,
    pub effect_digest: BindingDigest,
    pub intent_digest: BindingDigest,
    pub fallback_eligibility_digest: Option<BindingDigest>,
    pub control_lease_digest: Option<BindingDigest>,
    pub visible_channel_digest: Option<BindingDigest>,
    pub permission_session_digest: BindingDigest,
    pub target_or_source_digest: BindingDigest,
    pub status: DesktopEvidenceStatus,
    pub reconciliation_ref: Option<BindingDigest>,
    pub recorded_at_unix_ms: u64,
}

impl DesktopEffectEvidence {
    pub fn validate(&self) -> Result<(), DesktopControlEvidenceError> {
        if self.effect_id.0 == 0 || self.session_id.0 == 0 || self.recorded_at_unix_ms == 0 {
            return Err(DesktopControlEvidenceError::InvalidEvidence);
        }
        for digest in [
            self.request_digest,
            self.effect_digest,
            self.intent_digest,
            self.permission_session_digest,
            self.target_or_source_digest,
        ] {
            require_digest(digest)?;
        }
        for digest in [
            self.fallback_eligibility_digest,
            self.control_lease_digest,
            self.visible_channel_digest,
            self.reconciliation_ref,
        ]
        .into_iter()
        .flatten()
        {
            require_digest(digest)?;
        }
        match self.operation {
            DesktopEvidenceOperation::SemanticAction | DesktopEvidenceOperation::Focus => {
                if self.fallback_eligibility_digest.is_some()
                    || self.control_lease_digest.is_none()
                    || self.visible_channel_digest.is_none()
                {
                    return Err(DesktopControlEvidenceError::InvalidEvidence);
                }
            }
            DesktopEvidenceOperation::RawInputFallback => {
                if self.fallback_eligibility_digest.is_none()
                    || self.control_lease_digest.is_none()
                    || self.visible_channel_digest.is_none()
                {
                    return Err(DesktopControlEvidenceError::InvalidEvidence);
                }
            }
            DesktopEvidenceOperation::Capture
            | DesktopEvidenceOperation::ClipboardRead
            | DesktopEvidenceOperation::ClipboardWrite => {
                if self.control_lease_digest.is_some() || self.visible_channel_digest.is_some() {
                    return Err(DesktopControlEvidenceError::InvalidEvidence);
                }
            }
        }
        if matches!(
            self.status,
            DesktopEvidenceStatus::Reconciling
                | DesktopEvidenceStatus::ReconciledSucceeded
                | DesktopEvidenceStatus::ReconciledFailed
                | DesktopEvidenceStatus::ManualReview
        ) != self.reconciliation_ref.is_some()
        {
            return Err(DesktopControlEvidenceError::InvalidEvidence);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DesktopControlEvidenceError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(EFFECT_EVIDENCE_DOMAIN)?;
        encoder.push_u128(self.effect_id.0);
        encoder.push_u128(self.session_id.0);
        encoder.push_u8(u8::try_from(self.operation.code()).map_err(|_| {
            DesktopControlEvidenceError::InvalidStoredRecord("operation integer")
        })?);
        push_digest(&mut encoder, self.request_digest)?;
        push_digest(&mut encoder, self.effect_digest)?;
        push_digest(&mut encoder, self.intent_digest)?;
        push_optional_digest(&mut encoder, self.fallback_eligibility_digest)?;
        push_optional_digest(&mut encoder, self.control_lease_digest)?;
        push_optional_digest(&mut encoder, self.visible_channel_digest)?;
        push_digest(&mut encoder, self.permission_session_digest)?;
        push_digest(&mut encoder, self.target_or_source_digest)?;
        encoder.push_u8(
            u8::try_from(self.status.code())
                .map_err(|_| DesktopControlEvidenceError::InvalidStoredRecord("status integer"))?,
        );
        push_optional_digest(&mut encoder, self.reconciliation_ref)?;
        encoder.push_u64(self.recorded_at_unix_ms);
        Ok(encoder.finish())
    }

    pub fn integrity_hash(&self) -> Result<[u8; 32], DesktopControlEvidenceError> {
        Ok(crate::payload_hash(&self.canonical_bytes()?))
    }
}

pub struct DesktopControlEvidenceStore {
    connection: Connection,
}

impl DesktopControlEvidenceStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, DesktopControlEvidenceError> {
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; \
             PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        migrate(&connection)?;
        Ok(Self { connection })
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, DesktopControlEvidenceError> {
        let connection = Connection::open_in_memory()?;
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;
        migrate(&connection)?;
        Ok(Self { connection })
    }

    pub fn append_effect_evidence(
        &mut self,
        evidence: DesktopEffectEvidence,
    ) -> Result<[u8; 32], DesktopControlEvidenceError> {
        evidence.validate()?;
        let record_bytes = evidence.canonical_bytes()?;
        let integrity_hash = evidence.integrity_hash()?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let previous = load_latest_effect_status(&tx, evidence.effect_id)?;
        validate_effect_transition(previous.map(|value| value.0), evidence.status)?;
        if let Some((_, latest_time)) = previous
            && evidence.recorded_at_unix_ms < latest_time
        {
            return Err(DesktopControlEvidenceError::NonMonotonicTime);
        }
        let previous_hash = tx
            .query_row(
                "SELECT integrity_hash FROM desktop_effect_evidence \
                 WHERE effect_id = ?1 ORDER BY sequence DESC LIMIT 1",
                params![effect_bytes(evidence.effect_id)],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()?;
        let previous_hash = previous_hash
            .map(|value| hash32(value, "previous effect hash"))
            .transpose()?;
        let chain_hash = effect_chain_hash(integrity_hash, previous_hash)?;
        tx.execute(
            r#"INSERT INTO desktop_effect_evidence
               (effect_id, session_id, operation, request_digest, effect_digest, intent_digest,
                fallback_eligibility_digest, control_lease_digest, visible_channel_digest,
                permission_session_digest, target_or_source_digest, status, reconciliation_ref,
                recorded_at_unix_ms, record_bytes, payload_hash, previous_integrity_hash,
                integrity_hash)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                       ?15, ?16, ?17, ?18)"#,
            params![
                effect_bytes(evidence.effect_id),
                session_bytes(evidence.session_id),
                evidence.operation.code(),
                evidence.request_digest.bytes().to_vec(),
                evidence.effect_digest.bytes().to_vec(),
                evidence.intent_digest.bytes().to_vec(),
                optional_digest_bytes(evidence.fallback_eligibility_digest),
                optional_digest_bytes(evidence.control_lease_digest),
                optional_digest_bytes(evidence.visible_channel_digest),
                evidence.permission_session_digest.bytes().to_vec(),
                evidence.target_or_source_digest.bytes().to_vec(),
                evidence.status.code(),
                optional_digest_bytes(evidence.reconciliation_ref),
                i64_from_u64(evidence.recorded_at_unix_ms)?,
                record_bytes,
                integrity_hash.to_vec(),
                previous_hash.map(|value| value.to_vec()),
                chain_hash.to_vec(),
            ],
        )?;
        tx.commit()?;
        Ok(chain_hash)
    }

    pub fn latest_effect_evidence(
        &self,
        effect_id: EffectId,
    ) -> Result<Option<DesktopEffectEvidence>, DesktopControlEvidenceError> {
        self.connection
            .query_row(
                r#"SELECT session_id, operation, request_digest, effect_digest, intent_digest,
                          fallback_eligibility_digest, control_lease_digest, visible_channel_digest,
                          permission_session_digest, target_or_source_digest, status,
                          reconciliation_ref, recorded_at_unix_ms, record_bytes, payload_hash
                   FROM desktop_effect_evidence
                   WHERE effect_id = ?1 ORDER BY sequence DESC LIMIT 1"#,
                params![effect_bytes(effect_id)],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                        row.get::<_, Vec<u8>>(3)?,
                        row.get::<_, Vec<u8>>(4)?,
                        row.get::<_, Option<Vec<u8>>>(5)?,
                        row.get::<_, Option<Vec<u8>>>(6)?,
                        row.get::<_, Option<Vec<u8>>>(7)?,
                        row.get::<_, Vec<u8>>(8)?,
                        row.get::<_, Vec<u8>>(9)?,
                        row.get::<_, i64>(10)?,
                        row.get::<_, Option<Vec<u8>>>(11)?,
                        row.get::<_, i64>(12)?,
                        row.get::<_, Vec<u8>>(13)?,
                        row.get::<_, Vec<u8>>(14)?,
                    ))
                },
            )
            .optional()?
            .map(|raw| decode_effect_evidence(effect_id, raw))
            .transpose()
    }

    pub fn has_unresolved_unknown_outcome(
        &self,
        session_id: SessionId,
    ) -> Result<bool, DesktopControlEvidenceError> {
        let mut statement = self.connection.prepare(
            r#"SELECT e.status
               FROM desktop_effect_evidence e
               JOIN (
                 SELECT effect_id, MAX(sequence) AS sequence
                 FROM desktop_effect_evidence
                 WHERE session_id = ?1
                 GROUP BY effect_id
               ) latest ON latest.sequence = e.sequence
               WHERE e.status IN (?2, ?3)
               LIMIT 1"#,
        )?;
        let found = statement
            .query_row(
                params![
                    session_bytes(session_id),
                    DesktopEvidenceStatus::UnknownOutcome.code(),
                    DesktopEvidenceStatus::Reconciling.code(),
                ],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        Ok(found)
    }

    pub fn persist_lease_state(
        &mut self,
        lease: DesktopControlLeaseState,
    ) -> Result<[u8; 32], DesktopControlEvidenceError> {
        lease.validate()?;
        let canonical = lease.canonical_bytes()?;
        let integrity_hash = durable_lease_hash(&lease)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = tx
            .query_row(
                "SELECT generation, canonical_bytes, integrity_hash \
                 FROM desktop_control_lease_state WHERE lease_id = ?1",
                params![id_bytes(lease.lease_id.as_u128())],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, Vec<u8>>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                    ))
                },
            )
            .optional()?;
        if let Some((generation, bytes, hash)) = existing {
            let generation = u64_from_i64(generation, "lease generation")?;
            if generation == lease.generation
                && bytes == canonical
                && hash.as_slice() == integrity_hash
            {
                tx.commit()?;
                return Ok(integrity_hash);
            }
            if lease.generation <= generation {
                return Err(DesktopControlEvidenceError::StaleGeneration);
            }
        }
        tx.execute(
            r#"INSERT INTO desktop_control_lease_state
               (lease_id, schema_version, generation, controlling_principal_ref, mode,
                issued_at_unix_ms, updated_at_unix_ms, expires_at_unix_ms, capability_ref,
                policy_ref, interrupt_cause_ref, canonical_bytes, integrity_hash)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
               ON CONFLICT(lease_id) DO UPDATE SET
                 schema_version = excluded.schema_version,
                 generation = excluded.generation,
                 controlling_principal_ref = excluded.controlling_principal_ref,
                 mode = excluded.mode,
                 issued_at_unix_ms = excluded.issued_at_unix_ms,
                 updated_at_unix_ms = excluded.updated_at_unix_ms,
                 expires_at_unix_ms = excluded.expires_at_unix_ms,
                 capability_ref = excluded.capability_ref,
                 policy_ref = excluded.policy_ref,
                 interrupt_cause_ref = excluded.interrupt_cause_ref,
                 canonical_bytes = excluded.canonical_bytes,
                 integrity_hash = excluded.integrity_hash"#,
            params![
                id_bytes(lease.lease_id.as_u128()),
                i64::from(lease.schema_version),
                i64_from_u64(lease.generation)?,
                lease.controlling_principal_ref.bytes().to_vec(),
                control_mode_code(lease.mode),
                i64_from_u64(lease.issued_at_unix_ms)?,
                i64_from_u64(lease.updated_at_unix_ms)?,
                i64_from_u64(lease.expires_at_unix_ms)?,
                lease.capability_ref.bytes().to_vec(),
                lease.policy_ref.bytes().to_vec(),
                optional_digest_bytes(lease.interrupt_cause_ref),
                canonical,
                integrity_hash.to_vec(),
            ],
        )?;
        tx.commit()?;
        Ok(integrity_hash)
    }

    pub fn load_lease_state(
        &self,
        lease_id: DesktopControlLeaseId,
    ) -> Result<Option<DesktopControlLeaseState>, DesktopControlEvidenceError> {
        self.connection
            .query_row(
                r#"SELECT schema_version, generation, controlling_principal_ref, mode,
                          issued_at_unix_ms, updated_at_unix_ms, expires_at_unix_ms,
                          capability_ref, policy_ref, interrupt_cause_ref, canonical_bytes,
                          integrity_hash
                   FROM desktop_control_lease_state WHERE lease_id = ?1"#,
                params![id_bytes(lease_id.as_u128())],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, i64>(6)?,
                        row.get::<_, Vec<u8>>(7)?,
                        row.get::<_, Vec<u8>>(8)?,
                        row.get::<_, Option<Vec<u8>>>(9)?,
                        row.get::<_, Vec<u8>>(10)?,
                        row.get::<_, Vec<u8>>(11)?,
                    ))
                },
            )
            .optional()?
            .map(|raw| decode_lease_state(lease_id, raw))
            .transpose()
    }

    pub fn persist_visible_channel(
        &mut self,
        channel: VisibleControlChannelState,
    ) -> Result<[u8; 32], DesktopControlEvidenceError> {
        channel.validate()?;
        let canonical = channel.canonical_bytes()?;
        let integrity_hash = durable_visible_channel_hash(&channel)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = tx
            .query_row(
                "SELECT generation, kind, trusted_host_ref, canonical_bytes, integrity_hash \
                 FROM desktop_visible_channel_state WHERE channel_id = ?1",
                params![id_bytes(channel.channel_id.as_u128())],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                        row.get::<_, Vec<u8>>(3)?,
                        row.get::<_, Vec<u8>>(4)?,
                    ))
                },
            )
            .optional()?;
        if let Some((generation, kind, trusted_host, bytes, hash)) = existing {
            let generation = u64_from_i64(generation, "visible generation")?;
            if generation == channel.generation
                && bytes == canonical
                && hash.as_slice() == integrity_hash
            {
                tx.commit()?;
                return Ok(integrity_hash);
            }
            if channel.generation <= generation
                || kind != visible_channel_kind_code(channel.kind)
                || trusted_host.as_slice() != channel.trusted_host_ref.bytes()
            {
                return Err(DesktopControlEvidenceError::StaleGeneration);
            }
        }
        tx.execute(
            r#"INSERT INTO desktop_visible_channel_state
               (channel_id, schema_version, generation, kind, trusted_host_ref, visible, live,
                supports_pause, supports_stop, supports_takeover, observed_at_unix_ms,
                heartbeat_deadline_unix_ms, canonical_bytes, integrity_hash)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
               ON CONFLICT(channel_id) DO UPDATE SET
                 schema_version = excluded.schema_version,
                 generation = excluded.generation,
                 kind = excluded.kind,
                 trusted_host_ref = excluded.trusted_host_ref,
                 visible = excluded.visible,
                 live = excluded.live,
                 supports_pause = excluded.supports_pause,
                 supports_stop = excluded.supports_stop,
                 supports_takeover = excluded.supports_takeover,
                 observed_at_unix_ms = excluded.observed_at_unix_ms,
                 heartbeat_deadline_unix_ms = excluded.heartbeat_deadline_unix_ms,
                 canonical_bytes = excluded.canonical_bytes,
                 integrity_hash = excluded.integrity_hash"#,
            params![
                id_bytes(channel.channel_id.as_u128()),
                i64::from(channel.schema_version),
                i64_from_u64(channel.generation)?,
                visible_channel_kind_code(channel.kind),
                channel.trusted_host_ref.bytes().to_vec(),
                i64::from(channel.visible),
                i64::from(channel.live),
                i64::from(channel.supports_pause),
                i64::from(channel.supports_stop),
                i64::from(channel.supports_takeover),
                i64_from_u64(channel.observed_at_unix_ms)?,
                i64_from_u64(channel.heartbeat_deadline_unix_ms)?,
                canonical,
                integrity_hash.to_vec(),
            ],
        )?;
        tx.commit()?;
        Ok(integrity_hash)
    }

    pub fn load_visible_channels(
        &self,
    ) -> Result<Vec<VisibleControlChannelState>, DesktopControlEvidenceError> {
        let mut statement = self.connection.prepare(
            r#"SELECT channel_id, schema_version, generation, kind, trusted_host_ref, visible, live,
                      supports_pause, supports_stop, supports_takeover, observed_at_unix_ms,
                      heartbeat_deadline_unix_ms, canonical_bytes, integrity_hash
               FROM desktop_visible_channel_state ORDER BY channel_id ASC"#,
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, Vec<u8>>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, Vec<u8>>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, i64>(8)?,
                row.get::<_, i64>(9)?,
                row.get::<_, i64>(10)?,
                row.get::<_, i64>(11)?,
                row.get::<_, Vec<u8>>(12)?,
                row.get::<_, Vec<u8>>(13)?,
            ))
        })?;
        let mut channels = Vec::new();
        for row in rows {
            channels.push(decode_visible_channel(row?)?);
        }
        Ok(channels)
    }

    pub fn persist_interrupt(
        &mut self,
        evidence: &HumanInterruptEvidence,
    ) -> Result<[u8; 32], DesktopControlEvidenceError> {
        evidence.validate()?;
        let canonical = evidence.canonical_bytes()?;
        let integrity_hash = durable_interrupt_hash(evidence)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = tx
            .query_row(
                "SELECT canonical_bytes, integrity_hash FROM desktop_human_interrupt_evidence \
                 WHERE interrupt_id = ?1",
                params![id_bytes(evidence.interrupt_id)],
                |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
            )
            .optional()?;
        if let Some((bytes, hash)) = existing {
            if bytes == canonical && hash.as_slice() == integrity_hash {
                tx.commit()?;
                return Ok(integrity_hash);
            }
            return Err(DesktopControlEvidenceError::ImmutableEvidenceMismatch);
        }
        tx.execute(
            r#"INSERT INTO desktop_human_interrupt_evidence
               (interrupt_id, operation, lease_id, prior_generation, resulting_generation,
                accepted_at_unix_ms, authority_revoked_at_unix_ms, takeover_latency_ms,
                canonical_bytes, integrity_hash)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
            params![
                id_bytes(evidence.interrupt_id),
                interrupt_operation_code(evidence.operation),
                id_bytes(evidence.prior_lease_id.as_u128()),
                i64_from_u64(evidence.prior_generation)?,
                i64_from_u64(evidence.resulting_generation)?,
                i64_from_u64(evidence.accepted_at_unix_ms)?,
                i64_from_u64(evidence.authority_revoked_at_unix_ms)?,
                i64_from_u64(evidence.takeover_latency_ms()?)?,
                canonical,
                integrity_hash.to_vec(),
            ],
        )?;
        tx.commit()?;
        Ok(integrity_hash)
    }
}

fn migrate(connection: &Connection) -> Result<(), DesktopControlEvidenceError> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS desktop_effect_evidence (
            sequence INTEGER PRIMARY KEY AUTOINCREMENT,
            effect_id BLOB NOT NULL CHECK (length(effect_id) = 16),
            session_id BLOB NOT NULL CHECK (length(session_id) = 16),
            operation INTEGER NOT NULL,
            request_digest BLOB NOT NULL CHECK (length(request_digest) = 32),
            effect_digest BLOB NOT NULL CHECK (length(effect_digest) = 32),
            intent_digest BLOB NOT NULL CHECK (length(intent_digest) = 32),
            fallback_eligibility_digest BLOB CHECK (fallback_eligibility_digest IS NULL OR length(fallback_eligibility_digest) = 32),
            control_lease_digest BLOB CHECK (control_lease_digest IS NULL OR length(control_lease_digest) = 32),
            visible_channel_digest BLOB CHECK (visible_channel_digest IS NULL OR length(visible_channel_digest) = 32),
            permission_session_digest BLOB NOT NULL CHECK (length(permission_session_digest) = 32),
            target_or_source_digest BLOB NOT NULL CHECK (length(target_or_source_digest) = 32),
            status INTEGER NOT NULL,
            reconciliation_ref BLOB CHECK (reconciliation_ref IS NULL OR length(reconciliation_ref) = 32),
            recorded_at_unix_ms INTEGER NOT NULL,
            record_bytes BLOB NOT NULL,
            payload_hash BLOB NOT NULL CHECK (length(payload_hash) = 32),
            previous_integrity_hash BLOB CHECK (previous_integrity_hash IS NULL OR length(previous_integrity_hash) = 32),
            integrity_hash BLOB UNIQUE NOT NULL CHECK (length(integrity_hash) = 32)
        );
        CREATE INDEX IF NOT EXISTS desktop_effect_evidence_effect_sequence
            ON desktop_effect_evidence(effect_id, sequence);
        CREATE INDEX IF NOT EXISTS desktop_effect_evidence_session_sequence
            ON desktop_effect_evidence(session_id, sequence);

        CREATE TABLE IF NOT EXISTS desktop_control_lease_state (
            lease_id BLOB PRIMARY KEY NOT NULL CHECK (length(lease_id) = 16),
            schema_version INTEGER NOT NULL,
            generation INTEGER NOT NULL,
            controlling_principal_ref BLOB NOT NULL CHECK (length(controlling_principal_ref) = 32),
            mode INTEGER NOT NULL,
            issued_at_unix_ms INTEGER NOT NULL,
            updated_at_unix_ms INTEGER NOT NULL,
            expires_at_unix_ms INTEGER NOT NULL,
            capability_ref BLOB NOT NULL CHECK (length(capability_ref) = 32),
            policy_ref BLOB NOT NULL CHECK (length(policy_ref) = 32),
            interrupt_cause_ref BLOB CHECK (interrupt_cause_ref IS NULL OR length(interrupt_cause_ref) = 32),
            canonical_bytes BLOB NOT NULL,
            integrity_hash BLOB UNIQUE NOT NULL CHECK (length(integrity_hash) = 32)
        );

        CREATE TABLE IF NOT EXISTS desktop_visible_channel_state (
            channel_id BLOB PRIMARY KEY NOT NULL CHECK (length(channel_id) = 16),
            schema_version INTEGER NOT NULL,
            generation INTEGER NOT NULL,
            kind INTEGER NOT NULL,
            trusted_host_ref BLOB NOT NULL CHECK (length(trusted_host_ref) = 32),
            visible INTEGER NOT NULL,
            live INTEGER NOT NULL,
            supports_pause INTEGER NOT NULL,
            supports_stop INTEGER NOT NULL,
            supports_takeover INTEGER NOT NULL,
            observed_at_unix_ms INTEGER NOT NULL,
            heartbeat_deadline_unix_ms INTEGER NOT NULL,
            canonical_bytes BLOB NOT NULL,
            integrity_hash BLOB UNIQUE NOT NULL CHECK (length(integrity_hash) = 32)
        );

        CREATE TABLE IF NOT EXISTS desktop_human_interrupt_evidence (
            interrupt_id BLOB PRIMARY KEY NOT NULL CHECK (length(interrupt_id) = 16),
            operation INTEGER NOT NULL,
            lease_id BLOB NOT NULL CHECK (length(lease_id) = 16),
            prior_generation INTEGER NOT NULL,
            resulting_generation INTEGER NOT NULL,
            accepted_at_unix_ms INTEGER NOT NULL,
            authority_revoked_at_unix_ms INTEGER NOT NULL,
            takeover_latency_ms INTEGER NOT NULL,
            canonical_bytes BLOB NOT NULL,
            integrity_hash BLOB UNIQUE NOT NULL CHECK (length(integrity_hash) = 32)
        );
        "#,
    )?;
    Ok(())
}

fn load_latest_effect_status(
    tx: &rusqlite::Transaction<'_>,
    effect_id: EffectId,
) -> Result<Option<(DesktopEvidenceStatus, u64)>, DesktopControlEvidenceError> {
    tx.query_row(
        "SELECT status, recorded_at_unix_ms FROM desktop_effect_evidence \
         WHERE effect_id = ?1 ORDER BY sequence DESC LIMIT 1",
        params![effect_bytes(effect_id)],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
    )
    .optional()?
    .map(|(status, time)| {
        Ok((
            DesktopEvidenceStatus::from_code(status)?,
            u64_from_i64(time, "effect evidence time")?,
        ))
    })
    .transpose()
}

fn validate_effect_transition(
    previous: Option<DesktopEvidenceStatus>,
    next: DesktopEvidenceStatus,
) -> Result<(), DesktopControlEvidenceError> {
    let valid = matches!(
        (previous, next),
        (None, DesktopEvidenceStatus::Prepared)
            | (Some(DesktopEvidenceStatus::Prepared), DesktopEvidenceStatus::Succeeded)
            | (Some(DesktopEvidenceStatus::Prepared), DesktopEvidenceStatus::Failed)
            | (
                Some(DesktopEvidenceStatus::Prepared),
                DesktopEvidenceStatus::UnknownOutcome
            )
            | (Some(DesktopEvidenceStatus::Prepared), DesktopEvidenceStatus::Interrupted)
            | (
                Some(DesktopEvidenceStatus::UnknownOutcome),
                DesktopEvidenceStatus::Reconciling
            )
            | (
                Some(DesktopEvidenceStatus::Reconciling),
                DesktopEvidenceStatus::ReconciledSucceeded
            )
            | (
                Some(DesktopEvidenceStatus::Reconciling),
                DesktopEvidenceStatus::ReconciledFailed
            )
            | (
                Some(DesktopEvidenceStatus::Reconciling),
                DesktopEvidenceStatus::ManualReview
            )
    );
    if valid {
        Ok(())
    } else {
        Err(DesktopControlEvidenceError::InvalidEvidenceTransition)
    }
}

type RawEffectEvidence = (
    Vec<u8>,
    i64,
    Vec<u8>,
    Vec<u8>,
    Vec<u8>,
    Option<Vec<u8>>,
    Option<Vec<u8>>,
    Option<Vec<u8>>,
    Vec<u8>,
    Vec<u8>,
    i64,
    Option<Vec<u8>>,
    i64,
    Vec<u8>,
    Vec<u8>,
);

fn decode_effect_evidence(
    effect_id: EffectId,
    raw: RawEffectEvidence,
) -> Result<DesktopEffectEvidence, DesktopControlEvidenceError> {
    let evidence = DesktopEffectEvidence {
        effect_id,
        session_id: SessionId(u128_from_bytes(raw.0, "session id")?),
        operation: DesktopEvidenceOperation::from_code(raw.1)?,
        request_digest: digest32(raw.2, "request digest")?,
        effect_digest: digest32(raw.3, "effect digest")?,
        intent_digest: digest32(raw.4, "intent digest")?,
        fallback_eligibility_digest: optional_digest32(raw.5, "fallback digest")?,
        control_lease_digest: optional_digest32(raw.6, "lease digest")?,
        visible_channel_digest: optional_digest32(raw.7, "visible digest")?,
        permission_session_digest: digest32(raw.8, "permission digest")?,
        target_or_source_digest: digest32(raw.9, "target digest")?,
        status: DesktopEvidenceStatus::from_code(raw.10)?,
        reconciliation_ref: optional_digest32(raw.11, "reconciliation digest")?,
        recorded_at_unix_ms: u64_from_i64(raw.12, "recorded time")?,
    };
    evidence.validate()?;
    if raw.13 != evidence.canonical_bytes()? || raw.14.as_slice() != evidence.integrity_hash()? {
        return Err(DesktopControlEvidenceError::IntegrityMismatch);
    }
    Ok(evidence)
}

type RawLeaseState = (
    i64,
    i64,
    Vec<u8>,
    i64,
    i64,
    i64,
    i64,
    Vec<u8>,
    Vec<u8>,
    Option<Vec<u8>>,
    Vec<u8>,
    Vec<u8>,
);

fn decode_lease_state(
    lease_id: DesktopControlLeaseId,
    raw: RawLeaseState,
) -> Result<DesktopControlLeaseState, DesktopControlEvidenceError> {
    let lease = DesktopControlLeaseState {
        schema_version: u16::try_from(raw.0).map_err(|_| {
            DesktopControlEvidenceError::InvalidStoredRecord("lease schema version")
        })?,
        lease_id,
        generation: u64_from_i64(raw.1, "lease generation")?,
        controlling_principal_ref: digest32(raw.2, "lease principal")?,
        mode: control_mode_from_code(raw.3)?,
        issued_at_unix_ms: u64_from_i64(raw.4, "lease issued time")?,
        updated_at_unix_ms: u64_from_i64(raw.5, "lease updated time")?,
        expires_at_unix_ms: u64_from_i64(raw.6, "lease expiry time")?,
        capability_ref: digest32(raw.7, "lease capability")?,
        policy_ref: digest32(raw.8, "lease policy")?,
        interrupt_cause_ref: optional_digest32(raw.9, "lease interrupt")?,
    };
    lease.validate()?;
    if raw.10 != lease.canonical_bytes()? || raw.11.as_slice() != durable_lease_hash(&lease)? {
        return Err(DesktopControlEvidenceError::IntegrityMismatch);
    }
    Ok(lease)
}

type RawVisibleChannel = (
    Vec<u8>,
    i64,
    i64,
    i64,
    Vec<u8>,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    Vec<u8>,
    Vec<u8>,
);

fn decode_visible_channel(
    raw: RawVisibleChannel,
) -> Result<VisibleControlChannelState, DesktopControlEvidenceError> {
    let channel = VisibleControlChannelState {
        schema_version: u16::try_from(raw.1).map_err(|_| {
            DesktopControlEvidenceError::InvalidStoredRecord("visible schema version")
        })?,
        channel_id: VisibleControlChannelId::from_u128(u128_from_bytes(raw.0, "channel id")?),
        generation: u64_from_i64(raw.2, "visible generation")?,
        kind: visible_channel_kind_from_code(raw.3)?,
        trusted_host_ref: digest32(raw.4, "trusted host")?,
        visible: bool_from_i64(raw.5, "visible flag")?,
        live: bool_from_i64(raw.6, "live flag")?,
        supports_pause: bool_from_i64(raw.7, "pause flag")?,
        supports_stop: bool_from_i64(raw.8, "stop flag")?,
        supports_takeover: bool_from_i64(raw.9, "takeover flag")?,
        observed_at_unix_ms: u64_from_i64(raw.10, "visible observed time")?,
        heartbeat_deadline_unix_ms: u64_from_i64(raw.11, "visible heartbeat time")?,
    };
    channel.validate()?;
    if raw.12 != channel.canonical_bytes()?
        || raw.13.as_slice() != durable_visible_channel_hash(&channel)?
    {
        return Err(DesktopControlEvidenceError::IntegrityMismatch);
    }
    Ok(channel)
}

fn durable_lease_hash(
    lease: &DesktopControlLeaseState,
) -> Result<[u8; 32], DesktopControlEvidenceError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(LEASE_STATE_DOMAIN)?;
    encoder.push_bytes(&lease.canonical_bytes()?)?;
    Ok(crate::payload_hash(&encoder.finish()))
}

fn durable_visible_channel_hash(
    channel: &VisibleControlChannelState,
) -> Result<[u8; 32], DesktopControlEvidenceError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(VISIBLE_CHANNEL_DOMAIN)?;
    encoder.push_bytes(&channel.canonical_bytes()?)?;
    Ok(crate::payload_hash(&encoder.finish()))
}

fn durable_interrupt_hash(
    evidence: &HumanInterruptEvidence,
) -> Result<[u8; 32], DesktopControlEvidenceError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(INTERRUPT_DOMAIN)?;
    encoder.push_bytes(&evidence.canonical_bytes()?)?;
    Ok(crate::payload_hash(&encoder.finish()))
}

fn effect_chain_hash(
    payload_hash: [u8; 32],
    previous_hash: Option<[u8; 32]>,
) -> Result<[u8; 32], DesktopControlEvidenceError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(EFFECT_EVIDENCE_DOMAIN)?;
    encoder.push_bytes(&payload_hash)?;
    match previous_hash {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(&value)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(crate::payload_hash(&encoder.finish()))
}

fn control_mode_code(mode: DesktopControlMode) -> i64 {
    match mode {
        DesktopControlMode::AgentAllowed => 1,
        DesktopControlMode::Paused => 2,
        DesktopControlMode::HumanExclusive => 3,
        DesktopControlMode::Revoked => 4,
    }
}

fn control_mode_from_code(value: i64) -> Result<DesktopControlMode, DesktopControlEvidenceError> {
    match value {
        1 => Ok(DesktopControlMode::AgentAllowed),
        2 => Ok(DesktopControlMode::Paused),
        3 => Ok(DesktopControlMode::HumanExclusive),
        4 => Ok(DesktopControlMode::Revoked),
        _ => Err(DesktopControlEvidenceError::InvalidStoredRecord(
            "control mode",
        )),
    }
}

fn visible_channel_kind_code(kind: VisibleControlChannelKind) -> i64 {
    match kind {
        VisibleControlChannelKind::TauriNativeWindow => 1,
        VisibleControlChannelKind::SystemTray => 2,
        VisibleControlChannelKind::PlatformIndicator => 3,
    }
}

fn visible_channel_kind_from_code(
    value: i64,
) -> Result<VisibleControlChannelKind, DesktopControlEvidenceError> {
    match value {
        1 => Ok(VisibleControlChannelKind::TauriNativeWindow),
        2 => Ok(VisibleControlChannelKind::SystemTray),
        3 => Ok(VisibleControlChannelKind::PlatformIndicator),
        _ => Err(DesktopControlEvidenceError::InvalidStoredRecord(
            "visible channel kind",
        )),
    }
}

fn interrupt_operation_code(operation: HumanInterruptOperation) -> i64 {
    match operation {
        HumanInterruptOperation::Pause => 1,
        HumanInterruptOperation::Stop => 2,
        HumanInterruptOperation::Takeover => 3,
        HumanInterruptOperation::ReleaseHumanExclusive => 4,
    }
}

fn require_digest(digest: BindingDigest) -> Result<(), DesktopControlEvidenceError> {
    if digest.bytes() == [0; 32] {
        Err(DesktopControlEvidenceError::InvalidEvidence)
    } else {
        Ok(())
    }
}

fn push_digest(
    encoder: &mut CanonicalEncoder,
    digest: BindingDigest,
) -> Result<(), DesktopControlEvidenceError> {
    require_digest(digest)?;
    encoder.push_bytes(&digest.bytes())?;
    Ok(())
}

fn push_optional_digest(
    encoder: &mut CanonicalEncoder,
    digest: Option<BindingDigest>,
) -> Result<(), DesktopControlEvidenceError> {
    match digest {
        Some(value) => {
            encoder.push_u8(1);
            push_digest(encoder, value)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn optional_digest_bytes(value: Option<BindingDigest>) -> Option<Vec<u8>> {
    value.map(|digest| digest.bytes().to_vec())
}

fn digest32(
    value: Vec<u8>,
    field: &'static str,
) -> Result<BindingDigest, DesktopControlEvidenceError> {
    Ok(BindingDigest::new(hash32(value, field)?))
}

fn optional_digest32(
    value: Option<Vec<u8>>,
    field: &'static str,
) -> Result<Option<BindingDigest>, DesktopControlEvidenceError> {
    value.map(|bytes| digest32(bytes, field)).transpose()
}

fn hash32(
    value: Vec<u8>,
    field: &'static str,
) -> Result<[u8; 32], DesktopControlEvidenceError> {
    value
        .try_into()
        .map_err(|_| DesktopControlEvidenceError::InvalidStoredRecord(field))
}

fn effect_bytes(effect_id: EffectId) -> Vec<u8> {
    effect_id.0.to_be_bytes().to_vec()
}

fn session_bytes(session_id: SessionId) -> Vec<u8> {
    session_id.0.to_be_bytes().to_vec()
}

fn id_bytes(value: u128) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

fn u128_from_bytes(
    value: Vec<u8>,
    field: &'static str,
) -> Result<u128, DesktopControlEvidenceError> {
    let bytes: [u8; 16] = value
        .try_into()
        .map_err(|_| DesktopControlEvidenceError::InvalidStoredRecord(field))?;
    Ok(u128::from_be_bytes(bytes))
}

fn i64_from_u64(value: u64) -> Result<i64, DesktopControlEvidenceError> {
    i64::try_from(value).map_err(|_| DesktopControlEvidenceError::IntegerOverflow)
}

fn u64_from_i64(
    value: i64,
    field: &'static str,
) -> Result<u64, DesktopControlEvidenceError> {
    u64::try_from(value).map_err(|_| DesktopControlEvidenceError::InvalidStoredRecord(field))
}

fn bool_from_i64(
    value: i64,
    field: &'static str,
) -> Result<bool, DesktopControlEvidenceError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(DesktopControlEvidenceError::InvalidStoredRecord(field)),
    }
}

#[derive(Debug)]
pub enum DesktopControlEvidenceError {
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Control(DesktopControlError),
    InvalidEvidence,
    InvalidEvidenceTransition,
    ImmutableEvidenceMismatch,
    StaleGeneration,
    NonMonotonicTime,
    IntegrityMismatch,
    IntegerOverflow,
    InvalidStoredRecord(&'static str),
}

impl fmt::Display for DesktopControlEvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "desktop control evidence sqlite error: {error}"),
            Self::Core(error) => write!(f, "desktop control evidence encoding error: {error}"),
            Self::Control(error) => write!(f, "desktop control evidence contract error: {error}"),
            Self::InvalidEvidence => f.write_str("desktop control evidence is invalid"),
            Self::InvalidEvidenceTransition => {
                f.write_str("desktop effect evidence transition is invalid")
            }
            Self::ImmutableEvidenceMismatch => {
                f.write_str("desktop evidence identity collision or immutable mismatch")
            }
            Self::StaleGeneration => {
                f.write_str("desktop authority state generation is stale or substituted")
            }
            Self::NonMonotonicTime => f.write_str("desktop evidence time moved backwards"),
            Self::IntegrityMismatch => f.write_str("desktop evidence integrity validation failed"),
            Self::IntegerOverflow => f.write_str("desktop evidence integer overflow"),
            Self::InvalidStoredRecord(field) => {
                write!(f, "desktop evidence stored record invalid: {field}")
            }
        }
    }
}

impl Error for DesktopControlEvidenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::Control(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for DesktopControlEvidenceError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for DesktopControlEvidenceError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<DesktopControlError> for DesktopControlEvidenceError {
    fn from(value: DesktopControlError) -> Self {
        Self::Control(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::desktop_control::{DESKTOP_CONTROL_SCHEMA_VERSION, VisibleControlChannelKind};

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn effect(status: DesktopEvidenceStatus, reconciliation_ref: Option<BindingDigest>) -> DesktopEffectEvidence {
        DesktopEffectEvidence {
            effect_id: EffectId(1),
            session_id: SessionId(2),
            operation: DesktopEvidenceOperation::RawInputFallback,
            request_digest: digest(1),
            effect_digest: digest(2),
            intent_digest: digest(3),
            fallback_eligibility_digest: Some(digest(4)),
            control_lease_digest: Some(digest(5)),
            visible_channel_digest: Some(digest(6)),
            permission_session_digest: digest(7),
            target_or_source_digest: digest(8),
            status,
            reconciliation_ref,
            recorded_at_unix_ms: match status {
                DesktopEvidenceStatus::Prepared => 10,
                DesktopEvidenceStatus::UnknownOutcome => 11,
                DesktopEvidenceStatus::Reconciling => 12,
                DesktopEvidenceStatus::ReconciledFailed => 13,
                _ => 14,
            },
        }
    }

    fn lease() -> DesktopControlLeaseState {
        DesktopControlLeaseState {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            lease_id: DesktopControlLeaseId::from_u128(10),
            generation: 1,
            controlling_principal_ref: digest(10),
            mode: DesktopControlMode::AgentAllowed,
            issued_at_unix_ms: 1,
            updated_at_unix_ms: 1,
            expires_at_unix_ms: 100,
            capability_ref: digest(11),
            policy_ref: digest(12),
            interrupt_cause_ref: None,
        }
    }

    fn channel() -> VisibleControlChannelState {
        VisibleControlChannelState {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            channel_id: VisibleControlChannelId::from_u128(20),
            generation: 1,
            kind: VisibleControlChannelKind::TauriNativeWindow,
            trusted_host_ref: digest(13),
            visible: true,
            live: true,
            supports_pause: true,
            supports_stop: true,
            supports_takeover: true,
            observed_at_unix_ms: 1,
            heartbeat_deadline_unix_ms: 50,
        }
    }

    #[test]
    fn unknown_outcome_survives_reopen_semantics_until_reconciled() {
        let mut store = DesktopControlEvidenceStore::open_in_memory().unwrap();
        store
            .append_effect_evidence(effect(DesktopEvidenceStatus::Prepared, None))
            .unwrap();
        store
            .append_effect_evidence(effect(DesktopEvidenceStatus::UnknownOutcome, None))
            .unwrap();
        assert!(store.has_unresolved_unknown_outcome(SessionId(2)).unwrap());
        store
            .append_effect_evidence(effect(
                DesktopEvidenceStatus::Reconciling,
                Some(digest(20)),
            ))
            .unwrap();
        assert!(store.has_unresolved_unknown_outcome(SessionId(2)).unwrap());
        store
            .append_effect_evidence(effect(
                DesktopEvidenceStatus::ReconciledFailed,
                Some(digest(21)),
            ))
            .unwrap();
        assert!(!store.has_unresolved_unknown_outcome(SessionId(2)).unwrap());
    }

    #[test]
    fn stale_lease_and_visible_generations_cannot_overwrite_current_state() {
        let mut store = DesktopControlEvidenceStore::open_in_memory().unwrap();
        store.persist_lease_state(lease()).unwrap();
        let mut next_lease = lease();
        next_lease.generation = 2;
        next_lease.updated_at_unix_ms = 2;
        next_lease.mode = DesktopControlMode::HumanExclusive;
        next_lease.interrupt_cause_ref = Some(digest(30));
        store.persist_lease_state(next_lease).unwrap();
        assert_eq!(
            store.persist_lease_state(lease()).unwrap_err().to_string(),
            "desktop authority state generation is stale or substituted"
        );
        assert_eq!(
            store
                .load_lease_state(DesktopControlLeaseId::from_u128(10))
                .unwrap()
                .unwrap(),
            next_lease
        );

        store.persist_visible_channel(channel()).unwrap();
        let mut next_channel = channel();
        next_channel.generation = 2;
        next_channel.visible = false;
        next_channel.observed_at_unix_ms = 2;
        next_channel.heartbeat_deadline_unix_ms = 60;
        store.persist_visible_channel(next_channel).unwrap();
        assert!(matches!(
            store.persist_visible_channel(channel()),
            Err(DesktopControlEvidenceError::StaleGeneration)
        ));
        assert_eq!(store.load_visible_channels().unwrap(), vec![next_channel]);
    }
}
