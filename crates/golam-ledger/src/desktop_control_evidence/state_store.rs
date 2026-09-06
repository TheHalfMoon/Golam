#![forbid(unsafe_code)]

use golam_core::desktop_control::{
    DesktopControlLeaseId, DesktopControlLeaseState, DesktopControlMode, HumanInterruptEvidence,
    HumanInterruptOperation, VisibleControlChannelId, VisibleControlChannelKind,
    VisibleControlChannelState,
};
use golam_core::tool_request::BindingDigest;
use golam_core::CanonicalEncoder;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use super::{DesktopControlEvidenceError, DesktopControlEvidenceStore};

const LEASE_STATE_DOMAIN: &[u8] = b"golam:desktop-durable-lease-state:v1";
const VISIBLE_CHANNEL_DOMAIN: &[u8] = b"golam:desktop-durable-visible-channel:v1";
const INTERRUPT_DOMAIN: &[u8] = b"golam:desktop-durable-interrupt:v1";

pub(crate) fn migrate(connection: &Connection) -> Result<(), DesktopControlEvidenceError> {
    connection.execute_batch(
        r#"
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
            interrupt_cause_ref BLOB CHECK (
                interrupt_cause_ref IS NULL OR length(interrupt_cause_ref) = 32
            ),
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

impl DesktopControlEvidenceStore {
    pub fn persist_lease_state(
        &mut self,
        lease: DesktopControlLeaseState,
    ) -> Result<[u8; 32], DesktopControlEvidenceError> {
        lease.validate()?;
        let canonical = lease.canonical_bytes()?;
        let integrity_hash = lease_hash(&lease)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = tx
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
        if let Some((generation, bytes, hash)) = current {
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
                lease.interrupt_cause_ref.map(|value| value.bytes().to_vec()),
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
        let row = self
            .connection
            .query_row(
                r#"SELECT schema_version, generation, controlling_principal_ref, mode,
                          issued_at_unix_ms, updated_at_unix_ms, expires_at_unix_ms,
                          capability_ref, policy_ref, interrupt_cause_ref, canonical_bytes,
                          integrity_hash
                   FROM desktop_control_lease_state WHERE lease_id = ?1"#,
                params![id_bytes(lease_id.as_u128())],
                |row| {
                    Ok(LeaseRow {
                        schema_version: row.get(0)?,
                        generation: row.get(1)?,
                        controlling_principal_ref: row.get(2)?,
                        mode: row.get(3)?,
                        issued_at_unix_ms: row.get(4)?,
                        updated_at_unix_ms: row.get(5)?,
                        expires_at_unix_ms: row.get(6)?,
                        capability_ref: row.get(7)?,
                        policy_ref: row.get(8)?,
                        interrupt_cause_ref: row.get(9)?,
                        canonical_bytes: row.get(10)?,
                        integrity_hash: row.get(11)?,
                    })
                },
            )
            .optional()?;
        row.map(|value| decode_lease(lease_id, value)).transpose()
    }

    pub fn persist_visible_channel(
        &mut self,
        channel: VisibleControlChannelState,
    ) -> Result<[u8; 32], DesktopControlEvidenceError> {
        channel.validate()?;
        let canonical = channel.canonical_bytes()?;
        let integrity_hash = visible_channel_hash(&channel)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = tx
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
        if let Some((generation, kind, trusted_host, bytes, hash)) = current {
            let generation = u64_from_i64(generation, "visible channel generation")?;
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
                bool_code(channel.visible),
                bool_code(channel.live),
                bool_code(channel.supports_pause),
                bool_code(channel.supports_stop),
                bool_code(channel.supports_takeover),
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
            r#"SELECT channel_id, schema_version, generation, kind, trusted_host_ref, visible,
                      live, supports_pause, supports_stop, supports_takeover,
                      observed_at_unix_ms, heartbeat_deadline_unix_ms, canonical_bytes,
                      integrity_hash
               FROM desktop_visible_channel_state ORDER BY channel_id ASC"#,
        )?;
        let rows = statement.query_map([], |row| {
            Ok(ChannelRow {
                channel_id: row.get(0)?,
                schema_version: row.get(1)?,
                generation: row.get(2)?,
                kind: row.get(3)?,
                trusted_host_ref: row.get(4)?,
                visible: row.get(5)?,
                live: row.get(6)?,
                supports_pause: row.get(7)?,
                supports_stop: row.get(8)?,
                supports_takeover: row.get(9)?,
                observed_at_unix_ms: row.get(10)?,
                heartbeat_deadline_unix_ms: row.get(11)?,
                canonical_bytes: row.get(12)?,
                integrity_hash: row.get(13)?,
            })
        })?;
        let mut channels = Vec::new();
        for row in rows {
            channels.push(decode_channel(row?)?);
        }
        Ok(channels)
    }

    pub fn persist_interrupt(
        &mut self,
        evidence: &HumanInterruptEvidence,
    ) -> Result<[u8; 32], DesktopControlEvidenceError> {
        evidence.validate()?;
        let canonical = evidence.canonical_bytes()?;
        let integrity_hash = interrupt_hash(evidence)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = tx
            .query_row(
                "SELECT canonical_bytes, integrity_hash \
                 FROM desktop_human_interrupt_evidence WHERE interrupt_id = ?1",
                params![id_bytes(evidence.interrupt_id)],
                |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
            )
            .optional()?;
        if let Some((bytes, hash)) = current {
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

struct LeaseRow {
    schema_version: i64,
    generation: i64,
    controlling_principal_ref: Vec<u8>,
    mode: i64,
    issued_at_unix_ms: i64,
    updated_at_unix_ms: i64,
    expires_at_unix_ms: i64,
    capability_ref: Vec<u8>,
    policy_ref: Vec<u8>,
    interrupt_cause_ref: Option<Vec<u8>>,
    canonical_bytes: Vec<u8>,
    integrity_hash: Vec<u8>,
}

struct ChannelRow {
    channel_id: Vec<u8>,
    schema_version: i64,
    generation: i64,
    kind: i64,
    trusted_host_ref: Vec<u8>,
    visible: i64,
    live: i64,
    supports_pause: i64,
    supports_stop: i64,
    supports_takeover: i64,
    observed_at_unix_ms: i64,
    heartbeat_deadline_unix_ms: i64,
    canonical_bytes: Vec<u8>,
    integrity_hash: Vec<u8>,
}

fn decode_lease(
    lease_id: DesktopControlLeaseId,
    row: LeaseRow,
) -> Result<DesktopControlLeaseState, DesktopControlEvidenceError> {
    let lease = DesktopControlLeaseState {
        schema_version: u16::try_from(row.schema_version).map_err(|_| {
            DesktopControlEvidenceError::InvalidStoredRecord("lease schema version")
        })?,
        lease_id,
        generation: u64_from_i64(row.generation, "lease generation")?,
        controlling_principal_ref: digest(row.controlling_principal_ref, "lease principal")?,
        mode: control_mode_from_code(row.mode)?,
        issued_at_unix_ms: u64_from_i64(row.issued_at_unix_ms, "lease issued time")?,
        updated_at_unix_ms: u64_from_i64(row.updated_at_unix_ms, "lease updated time")?,
        expires_at_unix_ms: u64_from_i64(row.expires_at_unix_ms, "lease expiry time")?,
        capability_ref: digest(row.capability_ref, "lease capability")?,
        policy_ref: digest(row.policy_ref, "lease policy")?,
        interrupt_cause_ref: row
            .interrupt_cause_ref
            .map(|value| digest(value, "lease interrupt cause"))
            .transpose()?,
    };
    lease.validate()?;
    if row.canonical_bytes != lease.canonical_bytes()?
        || row.integrity_hash.as_slice() != lease_hash(&lease)?
    {
        return Err(DesktopControlEvidenceError::IntegrityMismatch);
    }
    Ok(lease)
}

fn decode_channel(
    row: ChannelRow,
) -> Result<VisibleControlChannelState, DesktopControlEvidenceError> {
    let channel = VisibleControlChannelState {
        schema_version: u16::try_from(row.schema_version).map_err(|_| {
            DesktopControlEvidenceError::InvalidStoredRecord("visible schema version")
        })?,
        channel_id: VisibleControlChannelId::from_u128(u128_from_bytes(
            row.channel_id,
            "visible channel id",
        )?),
        generation: u64_from_i64(row.generation, "visible channel generation")?,
        kind: visible_channel_kind_from_code(row.kind)?,
        trusted_host_ref: digest(row.trusted_host_ref, "visible trusted host")?,
        visible: bool_from_i64(row.visible, "visible flag")?,
        live: bool_from_i64(row.live, "live flag")?,
        supports_pause: bool_from_i64(row.supports_pause, "pause flag")?,
        supports_stop: bool_from_i64(row.supports_stop, "stop flag")?,
        supports_takeover: bool_from_i64(row.supports_takeover, "takeover flag")?,
        observed_at_unix_ms: u64_from_i64(row.observed_at_unix_ms, "visible observed time")?,
        heartbeat_deadline_unix_ms: u64_from_i64(
            row.heartbeat_deadline_unix_ms,
            "visible heartbeat deadline",
        )?,
    };
    channel.validate()?;
    if row.canonical_bytes != channel.canonical_bytes()?
        || row.integrity_hash.as_slice() != visible_channel_hash(&channel)?
    {
        return Err(DesktopControlEvidenceError::IntegrityMismatch);
    }
    Ok(channel)
}

fn lease_hash(
    lease: &DesktopControlLeaseState,
) -> Result<[u8; 32], DesktopControlEvidenceError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(LEASE_STATE_DOMAIN)?;
    encoder.push_bytes(&lease.canonical_bytes()?)?;
    Ok(crate::payload_hash(&encoder.finish()))
}

fn visible_channel_hash(
    channel: &VisibleControlChannelState,
) -> Result<[u8; 32], DesktopControlEvidenceError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(VISIBLE_CHANNEL_DOMAIN)?;
    encoder.push_bytes(&channel.canonical_bytes()?)?;
    Ok(crate::payload_hash(&encoder.finish()))
}

fn interrupt_hash(
    evidence: &HumanInterruptEvidence,
) -> Result<[u8; 32], DesktopControlEvidenceError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(INTERRUPT_DOMAIN)?;
    encoder.push_bytes(&evidence.canonical_bytes()?)?;
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
            "desktop control mode",
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

fn bool_code(value: bool) -> i64 {
    if value { 1 } else { 0 }
}

fn bool_from_i64(value: i64, field: &'static str) -> Result<bool, DesktopControlEvidenceError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(DesktopControlEvidenceError::InvalidStoredRecord(field)),
    }
}

fn digest(
    value: Vec<u8>,
    field: &'static str,
) -> Result<BindingDigest, DesktopControlEvidenceError> {
    let bytes: [u8; 32] = value
        .try_into()
        .map_err(|_| DesktopControlEvidenceError::InvalidStoredRecord(field))?;
    if bytes == [0; 32] {
        return Err(DesktopControlEvidenceError::InvalidStoredRecord(field));
    }
    Ok(BindingDigest::new(bytes))
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

#[cfg(test)]
mod tests {
    use golam_core::desktop_control::DESKTOP_CONTROL_SCHEMA_VERSION;

    use super::*;

    fn binding(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn lease() -> DesktopControlLeaseState {
        DesktopControlLeaseState {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            lease_id: DesktopControlLeaseId::from_u128(10),
            generation: 1,
            controlling_principal_ref: binding(1),
            mode: DesktopControlMode::AgentAllowed,
            issued_at_unix_ms: 1,
            updated_at_unix_ms: 1,
            expires_at_unix_ms: 100,
            capability_ref: binding(2),
            policy_ref: binding(3),
            interrupt_cause_ref: None,
        }
    }

    fn channel() -> VisibleControlChannelState {
        VisibleControlChannelState {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            channel_id: VisibleControlChannelId::from_u128(20),
            generation: 1,
            kind: VisibleControlChannelKind::TauriNativeWindow,
            trusted_host_ref: binding(4),
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
    fn stale_authority_generations_cannot_restore_old_state() {
        let mut store = DesktopControlEvidenceStore::open_in_memory().unwrap();
        store.persist_lease_state(lease()).unwrap();
        let mut next_lease = lease();
        next_lease.generation = 2;
        next_lease.updated_at_unix_ms = 2;
        next_lease.mode = DesktopControlMode::HumanExclusive;
        next_lease.interrupt_cause_ref = Some(binding(9));
        store.persist_lease_state(next_lease).unwrap();
        assert!(matches!(
            store.persist_lease_state(lease()),
            Err(DesktopControlEvidenceError::StaleGeneration)
        ));
        assert_eq!(
            store
                .load_lease_state(DesktopControlLeaseId::from_u128(10))
                .unwrap(),
            Some(next_lease)
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
