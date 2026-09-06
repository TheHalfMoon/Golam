#![forbid(unsafe_code)]

use golam_core::tool_request::BindingDigest;
use golam_core::{EffectId, SessionId};
use rusqlite::{Connection, OptionalExtension, Transaction, params};

use super::{
    DesktopControlEvidenceError, DesktopEffectEvidence, DesktopEvidenceOperation,
    DesktopEvidenceStatus,
};

pub(crate) fn migrate(connection: &Connection) -> Result<(), DesktopControlEvidenceError> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS desktop_effect_bindings (
            effect_id BLOB PRIMARY KEY NOT NULL CHECK (length(effect_id) = 16),
            session_id BLOB NOT NULL CHECK (length(session_id) = 16),
            operation INTEGER NOT NULL,
            request_digest BLOB NOT NULL CHECK (length(request_digest) = 32),
            effect_digest BLOB NOT NULL CHECK (length(effect_digest) = 32),
            intent_digest BLOB NOT NULL CHECK (length(intent_digest) = 32),
            fallback_eligibility_digest BLOB CHECK (
                fallback_eligibility_digest IS NULL OR length(fallback_eligibility_digest) = 32
            ),
            control_lease_digest BLOB CHECK (
                control_lease_digest IS NULL OR length(control_lease_digest) = 32
            ),
            visible_channel_digest BLOB CHECK (
                visible_channel_digest IS NULL OR length(visible_channel_digest) = 32
            ),
            permission_session_digest BLOB NOT NULL CHECK (length(permission_session_digest) = 32),
            target_or_source_digest BLOB NOT NULL CHECK (length(target_or_source_digest) = 32)
        );
        "#,
    )?;
    Ok(())
}

pub(crate) fn ensure_effect_binding(
    tx: &Transaction<'_>,
    evidence: &DesktopEffectEvidence,
) -> Result<(), DesktopControlEvidenceError> {
    let stored = tx
        .query_row(
            r#"SELECT session_id, operation, request_digest, effect_digest, intent_digest,
                      fallback_eligibility_digest, control_lease_digest, visible_channel_digest,
                      permission_session_digest, target_or_source_digest
               FROM desktop_effect_bindings WHERE effect_id = ?1"#,
            params![id_bytes(evidence.effect_id.0)],
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
                ))
            },
        )
        .optional()?;

    if let Some(stored) = stored {
        if stored.0 != id_bytes(evidence.session_id.0)
            || stored.1 != evidence.operation.code()
            || stored.2 != digest_bytes(evidence.request_digest)
            || stored.3 != digest_bytes(evidence.effect_digest)
            || stored.4 != digest_bytes(evidence.intent_digest)
            || stored.5 != optional_digest_bytes(evidence.fallback_eligibility_digest)
            || stored.6 != optional_digest_bytes(evidence.control_lease_digest)
            || stored.7 != optional_digest_bytes(evidence.visible_channel_digest)
            || stored.8 != digest_bytes(evidence.permission_session_digest)
            || stored.9 != digest_bytes(evidence.target_or_source_digest)
        {
            return Err(DesktopControlEvidenceError::ImmutableEvidenceMismatch);
        }
        return Ok(());
    }

    if evidence.status != DesktopEvidenceStatus::Prepared {
        return Err(DesktopControlEvidenceError::InvalidEvidenceTransition);
    }

    tx.execute(
        r#"INSERT INTO desktop_effect_bindings
           (effect_id, session_id, operation, request_digest, effect_digest, intent_digest,
            fallback_eligibility_digest, control_lease_digest, visible_channel_digest,
            permission_session_digest, target_or_source_digest)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
        params![
            id_bytes(evidence.effect_id.0),
            id_bytes(evidence.session_id.0),
            evidence.operation.code(),
            digest_bytes(evidence.request_digest),
            digest_bytes(evidence.effect_digest),
            digest_bytes(evidence.intent_digest),
            optional_digest_bytes(evidence.fallback_eligibility_digest),
            optional_digest_bytes(evidence.control_lease_digest),
            optional_digest_bytes(evidence.visible_channel_digest),
            digest_bytes(evidence.permission_session_digest),
            digest_bytes(evidence.target_or_source_digest),
        ],
    )?;
    Ok(())
}

impl super::DesktopControlEvidenceStore {
    pub fn reconciliation_evidence(
        &self,
        effect_id: EffectId,
        status: DesktopEvidenceStatus,
        reconciliation_ref: BindingDigest,
        recorded_at_unix_ms: u64,
    ) -> Result<DesktopEffectEvidence, DesktopControlEvidenceError> {
        let row = self
            .connection
            .query_row(
                r#"SELECT session_id, operation, request_digest, effect_digest, intent_digest,
                          fallback_eligibility_digest, control_lease_digest, visible_channel_digest,
                          permission_session_digest, target_or_source_digest
                   FROM desktop_effect_bindings WHERE effect_id = ?1"#,
                params![id_bytes(effect_id.0)],
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
                    ))
                },
            )
            .optional()?
            .ok_or(DesktopControlEvidenceError::InvalidStoredRecord(
                "missing desktop effect binding",
            ))?;

        let evidence = DesktopEffectEvidence {
            effect_id,
            session_id: SessionId(id128(row.0, "desktop binding session id")?),
            operation: operation_from_code(row.1)?,
            request_digest: digest(row.2, "desktop binding request digest")?,
            effect_digest: digest(row.3, "desktop binding effect digest")?,
            intent_digest: digest(row.4, "desktop binding intent digest")?,
            fallback_eligibility_digest: optional_digest(
                row.5,
                "desktop binding fallback eligibility digest",
            )?,
            control_lease_digest: optional_digest(row.6, "desktop binding control lease digest")?,
            visible_channel_digest: optional_digest(
                row.7,
                "desktop binding visible channel digest",
            )?,
            permission_session_digest: digest(
                row.8,
                "desktop binding permission session digest",
            )?,
            target_or_source_digest: digest(row.9, "desktop binding target/source digest")?,
            status,
            reconciliation_ref: Some(reconciliation_ref),
            recorded_at_unix_ms,
        };
        evidence.validate()?;
        Ok(evidence)
    }

    pub fn recovered_unknown_evidence(
        &self,
        effect_id: EffectId,
        recorded_at_unix_ms: u64,
    ) -> Result<DesktopEffectEvidence, DesktopControlEvidenceError> {
        let reconciliation_ref = BindingDigest::new([1; 32]);
        let mut evidence = self.reconciliation_evidence(
            effect_id,
            DesktopEvidenceStatus::Reconciling,
            reconciliation_ref,
            recorded_at_unix_ms,
        )?;
        evidence.status = DesktopEvidenceStatus::UnknownOutcome;
        evidence.reconciliation_ref = None;
        evidence.validate()?;
        Ok(evidence)
    }

    pub fn latest_effect_reconciliation_ref(
        &self,
        effect_id: EffectId,
    ) -> Result<Option<BindingDigest>, DesktopControlEvidenceError> {
        self.connection
            .query_row(
                "SELECT reconciliation_ref FROM desktop_effect_evidence \
                 WHERE effect_id = ?1 ORDER BY sequence DESC LIMIT 1",
                params![id_bytes(effect_id.0)],
                |row| row.get::<_, Option<Vec<u8>>>(0),
            )
            .optional()?
            .flatten()
            .map(|value| digest(value, "desktop reconciliation ref"))
            .transpose()
    }
}

fn operation_from_code(value: i64) -> Result<DesktopEvidenceOperation, DesktopControlEvidenceError> {
    match value {
        1 => Ok(DesktopEvidenceOperation::SemanticAction),
        2 => Ok(DesktopEvidenceOperation::Focus),
        3 => Ok(DesktopEvidenceOperation::RawInputFallback),
        4 => Ok(DesktopEvidenceOperation::Capture),
        5 => Ok(DesktopEvidenceOperation::ClipboardRead),
        6 => Ok(DesktopEvidenceOperation::ClipboardWrite),
        _ => Err(DesktopControlEvidenceError::InvalidStoredRecord(
            "desktop binding operation",
        )),
    }
}

fn id_bytes(value: u128) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

fn digest_bytes(value: BindingDigest) -> Vec<u8> {
    value.bytes().to_vec()
}

fn optional_digest_bytes(value: Option<BindingDigest>) -> Option<Vec<u8>> {
    value.map(digest_bytes)
}

fn id128(value: Vec<u8>, field: &'static str) -> Result<u128, DesktopControlEvidenceError> {
    let bytes: [u8; 16] = value
        .try_into()
        .map_err(|_| DesktopControlEvidenceError::InvalidStoredRecord(field))?;
    let value = u128::from_be_bytes(bytes);
    if value == 0 {
        return Err(DesktopControlEvidenceError::InvalidStoredRecord(field));
    }
    Ok(value)
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

fn optional_digest(
    value: Option<Vec<u8>>,
    field: &'static str,
) -> Result<Option<BindingDigest>, DesktopControlEvidenceError> {
    value.map(|value| digest(value, field)).transpose()
}
