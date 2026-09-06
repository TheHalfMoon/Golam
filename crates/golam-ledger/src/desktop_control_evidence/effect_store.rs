#![forbid(unsafe_code)]

use golam_core::{CanonicalEncoder, EffectId, SessionId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use super::{DesktopControlEvidenceStore, DesktopEffectEvidence, DesktopEvidenceStatus};
use crate::desktop_control_evidence::DesktopControlEvidenceError;

const EFFECT_CHAIN_DOMAIN: &[u8] = b"golam:desktop-effect-chain:v1";

pub(crate) fn migrate(connection: &Connection) -> Result<(), DesktopControlEvidenceError> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS desktop_effect_evidence (
            sequence INTEGER PRIMARY KEY AUTOINCREMENT,
            effect_id BLOB NOT NULL CHECK (length(effect_id) = 16),
            session_id BLOB NOT NULL CHECK (length(session_id) = 16),
            operation INTEGER NOT NULL,
            status INTEGER NOT NULL,
            recorded_at_unix_ms INTEGER NOT NULL,
            reconciliation_ref BLOB CHECK (
                reconciliation_ref IS NULL OR length(reconciliation_ref) = 32
            ),
            record_bytes BLOB NOT NULL,
            payload_hash BLOB NOT NULL CHECK (length(payload_hash) = 32),
            previous_integrity_hash BLOB CHECK (
                previous_integrity_hash IS NULL OR length(previous_integrity_hash) = 32
            ),
            integrity_hash BLOB UNIQUE NOT NULL CHECK (length(integrity_hash) = 32)
        );
        CREATE INDEX IF NOT EXISTS desktop_effect_evidence_effect_sequence
            ON desktop_effect_evidence(effect_id, sequence);
        CREATE INDEX IF NOT EXISTS desktop_effect_evidence_session_sequence
            ON desktop_effect_evidence(session_id, sequence);
        "#,
    )?;
    Ok(())
}

impl DesktopControlEvidenceStore {
    pub fn effect_session_id(
        &self,
        effect_id: EffectId,
    ) -> Result<SessionId, DesktopControlEvidenceError> {
        let session = self
            .connection
            .query_row(
                "SELECT session_id FROM effect_intents WHERE effect_id = ?1",
                params![id_bytes(effect_id.0)],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()?
            .ok_or(DesktopControlEvidenceError::InvalidStoredRecord(
                "missing canonical desktop effect",
            ))?;
        let bytes: [u8; 16] = session.try_into().map_err(|_| {
            DesktopControlEvidenceError::InvalidStoredRecord("canonical effect session id")
        })?;
        let session_id = SessionId(u128::from_be_bytes(bytes));
        if session_id.0 == 0 {
            return Err(DesktopControlEvidenceError::InvalidStoredRecord(
                "zero canonical effect session id",
            ));
        }
        Ok(session_id)
    }

    pub fn has_unresolved_unknown_outcome_for_effect(
        &self,
        effect_id: EffectId,
    ) -> Result<bool, DesktopControlEvidenceError> {
        self.has_unresolved_unknown_outcome(self.effect_session_id(effect_id)?)
    }

    pub fn append_effect_evidence(
        &mut self,
        evidence: DesktopEffectEvidence,
    ) -> Result<[u8; 32], DesktopControlEvidenceError> {
        evidence.validate()?;
        if self.effect_session_id(evidence.effect_id)? != evidence.session_id {
            return Err(DesktopControlEvidenceError::InvalidEvidence);
        }
        let record_bytes = evidence.canonical_bytes()?;
        let payload_hash = evidence.payload_hash()?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let previous_chain_hash = verify_effect_chain(&tx, evidence.effect_id)?;
        let previous = latest_status(&tx, evidence.effect_id)?;
        validate_transition(previous.map(|value| value.0), evidence.status)?;
        if let Some((_, previous_time)) = previous
            && evidence.recorded_at_unix_ms < previous_time
        {
            return Err(DesktopControlEvidenceError::NonMonotonicTime);
        }
        let integrity_hash = chain_hash(payload_hash, previous_chain_hash)?;
        tx.execute(
            r#"INSERT INTO desktop_effect_evidence
               (effect_id, session_id, operation, status, recorded_at_unix_ms,
                reconciliation_ref, record_bytes, payload_hash, previous_integrity_hash,
                integrity_hash)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
            params![
                id_bytes(evidence.effect_id.0),
                id_bytes(evidence.session_id.0),
                evidence.operation.code(),
                evidence.status.code(),
                i64_from_u64(evidence.recorded_at_unix_ms)?,
                evidence
                    .reconciliation_ref
                    .map(|value| value.bytes().to_vec()),
                record_bytes,
                payload_hash.to_vec(),
                previous_chain_hash.map(|value| value.to_vec()),
                integrity_hash.to_vec(),
            ],
        )?;
        tx.commit()?;
        Ok(integrity_hash)
    }

    pub fn latest_effect_status(
        &self,
        effect_id: EffectId,
    ) -> Result<Option<DesktopEvidenceStatus>, DesktopControlEvidenceError> {
        self.connection
            .query_row(
                "SELECT status FROM desktop_effect_evidence \
                 WHERE effect_id = ?1 ORDER BY sequence DESC LIMIT 1",
                params![id_bytes(effect_id.0)],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .map(DesktopEvidenceStatus::from_code)
            .transpose()
    }

    pub fn has_unresolved_unknown_outcome(
        &self,
        session_id: SessionId,
    ) -> Result<bool, DesktopControlEvidenceError> {
        let found = self
            .connection
            .query_row(
                r#"SELECT 1
                   FROM desktop_effect_evidence current
                   WHERE current.session_id = ?1
                     AND current.sequence = (
                       SELECT MAX(latest.sequence)
                       FROM desktop_effect_evidence latest
                       WHERE latest.effect_id = current.effect_id
                     )
                     AND current.status IN (?2, ?3)
                   LIMIT 1"#,
                params![
                    id_bytes(session_id.0),
                    DesktopEvidenceStatus::UnknownOutcome.code(),
                    DesktopEvidenceStatus::Reconciling.code(),
                ],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        Ok(found)
    }

    pub fn verify_effect_evidence_chain(
        &self,
        effect_id: EffectId,
    ) -> Result<Option<[u8; 32]>, DesktopControlEvidenceError> {
        let tx = self.connection.unchecked_transaction()?;
        let result = verify_effect_chain(&tx, effect_id)?;
        tx.rollback()?;
        Ok(result)
    }
}

fn latest_status(
    tx: &Transaction<'_>,
    effect_id: EffectId,
) -> Result<Option<(DesktopEvidenceStatus, u64)>, DesktopControlEvidenceError> {
    tx.query_row(
        "SELECT status, recorded_at_unix_ms FROM desktop_effect_evidence \
         WHERE effect_id = ?1 ORDER BY sequence DESC LIMIT 1",
        params![id_bytes(effect_id.0)],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
    )
    .optional()?
    .map(|(status, recorded_at)| {
        Ok((
            DesktopEvidenceStatus::from_code(status)?,
            u64_from_i64(recorded_at, "desktop effect recorded time")?,
        ))
    })
    .transpose()
}

fn verify_effect_chain(
    tx: &Transaction<'_>,
    effect_id: EffectId,
) -> Result<Option<[u8; 32]>, DesktopControlEvidenceError> {
    let mut statement = tx.prepare(
        r#"SELECT record_bytes, payload_hash, previous_integrity_hash, integrity_hash
           FROM desktop_effect_evidence
           WHERE effect_id = ?1
           ORDER BY sequence ASC"#,
    )?;
    let rows = statement.query_map(params![id_bytes(effect_id.0)], |row| {
        Ok((
            row.get::<_, Vec<u8>>(0)?,
            row.get::<_, Vec<u8>>(1)?,
            row.get::<_, Option<Vec<u8>>>(2)?,
            row.get::<_, Vec<u8>>(3)?,
        ))
    })?;
    let mut previous = None;
    for row in rows {
        let (record_bytes, stored_payload, stored_previous, stored_integrity) = row?;
        let payload_hash = crate::payload_hash(&record_bytes);
        if stored_payload.as_slice() != payload_hash {
            return Err(DesktopControlEvidenceError::IntegrityMismatch);
        }
        let stored_previous = stored_previous
            .map(|value| hash32(value, "desktop previous chain hash"))
            .transpose()?;
        if stored_previous != previous {
            return Err(DesktopControlEvidenceError::IntegrityMismatch);
        }
        let expected = chain_hash(payload_hash, previous)?;
        if stored_integrity.as_slice() != expected {
            return Err(DesktopControlEvidenceError::IntegrityMismatch);
        }
        previous = Some(expected);
    }
    Ok(previous)
}

fn chain_hash(
    payload_hash: [u8; 32],
    previous: Option<[u8; 32]>,
) -> Result<[u8; 32], DesktopControlEvidenceError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(EFFECT_CHAIN_DOMAIN)?;
    encoder.push_bytes(&payload_hash)?;
    match previous {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(&value)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(crate::payload_hash(&encoder.finish()))
}

fn validate_transition(
    previous: Option<DesktopEvidenceStatus>,
    next: DesktopEvidenceStatus,
) -> Result<(), DesktopControlEvidenceError> {
    let valid = match previous {
        None => next == DesktopEvidenceStatus::Prepared,
        Some(DesktopEvidenceStatus::Prepared) => matches!(
            next,
            DesktopEvidenceStatus::Succeeded
                | DesktopEvidenceStatus::Failed
                | DesktopEvidenceStatus::UnknownOutcome
                | DesktopEvidenceStatus::Interrupted
        ),
        Some(DesktopEvidenceStatus::UnknownOutcome) => next == DesktopEvidenceStatus::Reconciling,
        Some(DesktopEvidenceStatus::Reconciling) => matches!(
            next,
            DesktopEvidenceStatus::ReconciledSucceeded
                | DesktopEvidenceStatus::ReconciledFailed
                | DesktopEvidenceStatus::ManualReview
        ),
        Some(
            DesktopEvidenceStatus::Succeeded
            | DesktopEvidenceStatus::Failed
            | DesktopEvidenceStatus::Interrupted
            | DesktopEvidenceStatus::ReconciledSucceeded
            | DesktopEvidenceStatus::ReconciledFailed
            | DesktopEvidenceStatus::ManualReview,
        ) => false,
    };
    if valid {
        Ok(())
    } else {
        Err(DesktopControlEvidenceError::InvalidEvidenceTransition)
    }
}

fn id_bytes(value: u128) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

fn i64_from_u64(value: u64) -> Result<i64, DesktopControlEvidenceError> {
    i64::try_from(value).map_err(|_| DesktopControlEvidenceError::IntegerOverflow)
}

fn u64_from_i64(value: i64, field: &'static str) -> Result<u64, DesktopControlEvidenceError> {
    u64::try_from(value).map_err(|_| DesktopControlEvidenceError::InvalidStoredRecord(field))
}

fn hash32(value: Vec<u8>, field: &'static str) -> Result<[u8; 32], DesktopControlEvidenceError> {
    value
        .try_into()
        .map_err(|_| DesktopControlEvidenceError::InvalidStoredRecord(field))
}

#[cfg(test)]
mod tests {
    use golam_core::tool_request::BindingDigest;

    use super::*;
    use crate::desktop_control_evidence::{DesktopEffectEvidence, DesktopEvidenceOperation};
    use crate::effects::{EffectStore, ProposeEffect};
    use golam_core::{EffectTransitionId, EventId};

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn evidence(
        status: DesktopEvidenceStatus,
        reconciliation_ref: Option<BindingDigest>,
        time: u64,
    ) -> DesktopEffectEvidence {
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
            recorded_at_unix_ms: time,
        }
    }

    fn register_effect(store: &mut DesktopControlEvidenceStore) {
        let mut effects = EffectStore::open_in_memory().unwrap();
        effects
            .propose(ProposeEffect {
                effect_id: EffectId(1),
                session_id: SessionId(2),
                requested_by: "owner",
                action: "desktop.raw_input",
                resource: "desktop-target:test",
                risk_class: "write",
                execution_semantics: "at_most_once",
                idempotency_key: None,
                preconditions: b"p",
                dependencies: b"[]",
                payload_hash: [1; 32],
                proposed_event_id: EventId(1),
                transition_id: EffectTransitionId(1),
            })
            .unwrap();
        drop(effects);
        let _ = store;
    }

    #[test]
    fn unknown_outcome_blocks_until_terminal_reconciliation() {
        let mut store = DesktopControlEvidenceStore::open_in_memory().unwrap();
        // In-memory evidence tests use the same schema but must seed the canonical effect row.
        store
            .connection
            .execute(
                "INSERT INTO effect_intents (effect_id, session_id, requested_by, action, resource, risk_class, execution_semantics, idempotency_key, preconditions, dependencies, payload_hash, proposed_global_seq) VALUES (?1, ?2, 'owner', 'desktop.raw_input', 'desktop-target:test', 'write', 'at_most_once', NULL, ?3, ?4, ?5, 1)",
                params![id_bytes(1), id_bytes(2), b"p".as_slice(), b"[]".as_slice(), [1_u8; 32].as_slice()],
            )
            .unwrap();
        store
            .append_effect_evidence(evidence(DesktopEvidenceStatus::Prepared, None, 10))
            .unwrap();
        store
            .append_effect_evidence(evidence(DesktopEvidenceStatus::UnknownOutcome, None, 11))
            .unwrap();
        assert!(store.has_unresolved_unknown_outcome_for_effect(EffectId(1)).unwrap());
        store
            .append_effect_evidence(evidence(
                DesktopEvidenceStatus::Reconciling,
                Some(digest(20)),
                12,
            ))
            .unwrap();
        assert!(store.has_unresolved_unknown_outcome(SessionId(2)).unwrap());
        store
            .append_effect_evidence(evidence(
                DesktopEvidenceStatus::ReconciledFailed,
                Some(digest(21)),
                13,
            ))
            .unwrap();
        assert!(!store.has_unresolved_unknown_outcome(SessionId(2)).unwrap());
        assert!(
            store
                .verify_effect_evidence_chain(EffectId(1))
                .unwrap()
                .is_some()
        );
    }
}
