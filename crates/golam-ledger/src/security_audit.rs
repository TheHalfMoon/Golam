#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::{CanonicalEncoder, CoreError};
use rusqlite::{Connection, OptionalExtension, Transaction, params};

const CHAIN_NAME: &str = "authority-security";
const RECORD_DOMAIN: &[u8] = b"golam:authority-security-audit:v1";

pub(crate) const KIND_CLIENT_ENROLLED: &str = "client_enrolled";
pub(crate) const KIND_CLIENT_REVOKED: &str = "client_revoked";
pub(crate) const KIND_AUTHORIZATION_DECISION: &str = "authorization_decision";
pub(crate) const KIND_EFFECT_INTENT: &str = "effect_intent";
pub(crate) const KIND_EFFECT_TRANSITION: &str = "effect_transition";
pub(crate) const KIND_EFFECT_ATTEMPT_STARTED: &str = "effect_attempt_started";
pub(crate) const KIND_EFFECT_ATTEMPT_FINISHED: &str = "effect_attempt_finished";
pub(crate) const KIND_RECOVERY_INCIDENT: &str = "recovery_incident";

#[derive(Debug)]
pub(crate) enum SecurityAuditError {
    Sqlite(rusqlite::Error),
    Core(CoreError),
    InvalidRecord,
    SequenceOverflow,
    Coverage(&'static str),
    Integrity(&'static str),
}

impl fmt::Display for SecurityAuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "authority security audit sqlite error: {error}"),
            Self::Core(error) => write!(f, "authority security audit encoding error: {error}"),
            Self::InvalidRecord => f.write_str("authority security audit record is malformed"),
            Self::SequenceOverflow => f.write_str("authority security audit sequence overflow"),
            Self::Coverage(reason) => write!(f, "authority security audit coverage gap: {reason}"),
            Self::Integrity(reason) => {
                write!(f, "authority security audit integrity failure: {reason}")
            }
        }
    }
}

impl Error for SecurityAuditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for SecurityAuditError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for SecurityAuditError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

pub(crate) struct ClientEnrollmentAuditInput<'a> {
    pub client_id: &'a [u8],
    pub key_id: &'a str,
    pub public_key: &'a [u8],
    pub kind: &'a str,
    pub owner_principal: &'a str,
    pub enrolled_at: &'a str,
    pub assurance_class: &'a str,
}

pub(crate) struct ClientRevocationAuditInput<'a> {
    pub client_id: &'a [u8],
    pub revoked_at: &'a str,
}

pub(crate) struct AuthorizationAuditInput<'a> {
    pub decision_id: &'a [u8],
    pub principal: &'a str,
    pub action: &'a str,
    pub resource: &'a str,
    pub context_hash: &'a [u8],
    pub decision: &'a str,
    pub reason_code: &'a str,
    pub global_seq: u64,
}

pub(crate) struct EffectIntentAuditInput<'a> {
    pub effect_id: &'a [u8],
    pub session_id: &'a [u8],
    pub requested_by: &'a str,
    pub action: &'a str,
    pub resource: &'a str,
    pub risk_class: &'a str,
    pub execution_semantics: &'a str,
    pub idempotency_key: Option<&'a str>,
    pub preconditions: &'a [u8],
    pub dependencies: &'a [u8],
    pub payload_hash: &'a [u8],
    pub proposed_event_id: &'a [u8],
}

pub(crate) struct EffectTransitionAuditInput<'a> {
    pub transition_id: &'a [u8],
    pub effect_id: &'a [u8],
    pub global_seq: u64,
    pub from_state: Option<&'a str>,
    pub to_state: &'a str,
    pub attempt_id: Option<&'a [u8]>,
    pub reason_code: Option<&'a str>,
    pub evidence_ref: Option<&'a [u8]>,
    pub event_id: &'a [u8],
}

pub(crate) struct EffectAttemptStartedAuditInput<'a> {
    pub attempt_id: &'a [u8],
    pub effect_id: &'a [u8],
    pub started_global_seq: u64,
    pub handler_id: &'a str,
    pub handler_version: &'a str,
    pub dispatch_token: &'a [u8],
    pub started_at: &'a str,
}

pub(crate) struct EffectAttemptFinishedAuditInput<'a> {
    pub attempt_id: &'a [u8],
    pub finished_at: &'a str,
    pub outcome: &'a str,
    pub receipt: Option<&'a [u8]>,
}

pub(crate) struct RecoveryIncidentAuditInput<'a> {
    pub incident_id: &'a [u8],
    pub detected_at: &'a str,
    pub kind: &'a str,
    pub severity: &'a str,
    pub affected_refs: &'a [u8],
    pub recovery_mode: &'a str,
    pub resolution: Option<&'a [u8]>,
}

pub(crate) fn append_client_enrollment(
    transaction: &Transaction<'_>,
    input: ClientEnrollmentAuditInput<'_>,
) -> Result<(), SecurityAuditError> {
    let payload = encode_client_enrollment(&input)?;
    append_record(transaction, KIND_CLIENT_ENROLLED, input.client_id, &payload)
}

pub(crate) fn append_client_revocation(
    transaction: &Transaction<'_>,
    input: ClientRevocationAuditInput<'_>,
) -> Result<(), SecurityAuditError> {
    let payload = encode_client_revocation(&input)?;
    append_record(transaction, KIND_CLIENT_REVOKED, input.client_id, &payload)
}

pub(crate) fn append_authorization_decision(
    transaction: &Transaction<'_>,
    input: AuthorizationAuditInput<'_>,
) -> Result<(), SecurityAuditError> {
    let payload = encode_authorization(&input)?;
    append_record(
        transaction,
        KIND_AUTHORIZATION_DECISION,
        input.decision_id,
        &payload,
    )
}

pub(crate) fn append_effect_intent(
    transaction: &Transaction<'_>,
    input: EffectIntentAuditInput<'_>,
) -> Result<(), SecurityAuditError> {
    let payload = encode_effect_intent(&input)?;
    append_record(transaction, KIND_EFFECT_INTENT, input.effect_id, &payload)
}

pub(crate) fn append_effect_transition(
    transaction: &Transaction<'_>,
    input: EffectTransitionAuditInput<'_>,
) -> Result<(), SecurityAuditError> {
    let payload = encode_effect_transition(&input)?;
    append_record(
        transaction,
        KIND_EFFECT_TRANSITION,
        input.transition_id,
        &payload,
    )
}

pub(crate) fn append_effect_attempt_started(
    transaction: &Transaction<'_>,
    input: EffectAttemptStartedAuditInput<'_>,
) -> Result<(), SecurityAuditError> {
    let payload = encode_effect_attempt_started(&input)?;
    append_record(
        transaction,
        KIND_EFFECT_ATTEMPT_STARTED,
        input.attempt_id,
        &payload,
    )
}

pub(crate) fn append_effect_attempt_finished(
    transaction: &Transaction<'_>,
    input: EffectAttemptFinishedAuditInput<'_>,
) -> Result<(), SecurityAuditError> {
    let payload = encode_effect_attempt_finished(&input)?;
    append_record(
        transaction,
        KIND_EFFECT_ATTEMPT_FINISHED,
        input.attempt_id,
        &payload,
    )
}

pub(crate) fn append_recovery_incident(
    transaction: &Transaction<'_>,
    input: RecoveryIncidentAuditInput<'_>,
) -> Result<(), SecurityAuditError> {
    let payload = encode_recovery_incident(&input)?;
    append_record(
        transaction,
        KIND_RECOVERY_INCIDENT,
        input.incident_id,
        &payload,
    )
}

fn ensure_table(transaction: &Transaction<'_>) -> Result<(), rusqlite::Error> {
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS authority_security_audit (\
           audit_seq INTEGER PRIMARY KEY,\
           record_kind TEXT NOT NULL,\
           record_id BLOB NOT NULL,\
           payload_hash BLOB NOT NULL,\
           previous_hash BLOB,\
           record_hash BLOB NOT NULL,\
           UNIQUE(record_kind, record_id)\
         );",
    )
}

fn append_record(
    transaction: &Transaction<'_>,
    kind: &str,
    record_id: &[u8],
    payload: &[u8],
) -> Result<(), SecurityAuditError> {
    if kind.is_empty() || record_id.is_empty() {
        return Err(SecurityAuditError::InvalidRecord);
    }
    ensure_table(transaction)?;
    let previous = transaction
        .query_row(
            "SELECT last_global_seq, last_hash FROM audit_chain_heads WHERE chain_name = ?1",
            params![CHAIN_NAME],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?;
    let (audit_seq, previous_hash) = match previous {
        Some((seq, hash)) => {
            let seq = u64::try_from(seq).map_err(|_| SecurityAuditError::InvalidRecord)?;
            let previous_hash: [u8; 32] = hash
                .try_into()
                .map_err(|_| SecurityAuditError::InvalidRecord)?;
            (
                seq.checked_add(1)
                    .ok_or(SecurityAuditError::SequenceOverflow)?,
                Some(previous_hash),
            )
        }
        None => (1, None),
    };
    let payload_hash = *blake3::hash(payload).as_bytes();
    let record_hash = audit_record_hash(audit_seq, kind, record_id, payload_hash, previous_hash)?;
    transaction.execute(
        "INSERT INTO authority_security_audit \
         (audit_seq, record_kind, record_id, payload_hash, previous_hash, record_hash) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            i64::try_from(audit_seq).map_err(|_| SecurityAuditError::SequenceOverflow)?,
            kind,
            record_id,
            &payload_hash[..],
            previous_hash.map(|hash| hash.to_vec()),
            &record_hash[..],
        ],
    )?;
    transaction.execute(
        "INSERT INTO audit_chain_heads (chain_name, last_global_seq, last_hash) VALUES (?1, ?2, ?3) \
         ON CONFLICT(chain_name) DO UPDATE SET last_global_seq = excluded.last_global_seq, \
         last_hash = excluded.last_hash",
        params![
            CHAIN_NAME,
            i64::try_from(audit_seq).map_err(|_| SecurityAuditError::SequenceOverflow)?,
            &record_hash[..],
        ],
    )?;
    Ok(())
}

pub(crate) fn verify(connection: &Connection) -> Result<(), SecurityAuditError> {
    let table_exists = connection
        .query_row(
            "SELECT 1 FROM sqlite_schema WHERE type = 'table' \
             AND name = 'authority_security_audit' LIMIT 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some();

    let protected_record_count: i64 = connection.query_row(
        "SELECT \
           (SELECT COUNT(*) FROM clients) + \
           (SELECT COUNT(*) FROM clients WHERE revoked_at IS NOT NULL) + \
           (SELECT COUNT(*) FROM authorization_decisions) + \
           (SELECT COUNT(*) FROM effect_intents) + \
           (SELECT COUNT(*) FROM effect_transitions) + \
           (SELECT COUNT(*) FROM effect_attempts) + \
           (SELECT COUNT(*) FROM effect_attempts WHERE finished_at IS NOT NULL) + \
           (SELECT COUNT(*) FROM recovery_incidents)",
        [],
        |row| row.get(0),
    )?;
    if !table_exists {
        if protected_record_count == 0 {
            return verify_head(connection, None);
        }
        return Err(SecurityAuditError::Coverage(
            "protected records exist without authority-security audit table",
        ));
    }

    verify_coverage(connection)?;
    let mut statement = connection.prepare(
        "SELECT audit_seq, record_kind, record_id, payload_hash, previous_hash, record_hash \
         FROM authority_security_audit ORDER BY audit_seq ASC",
    )?;
    let mut rows = statement.query([])?;
    let mut expected_seq = 1_u64;
    let mut previous_hash = None;
    let mut last = None;
    while let Some(row) = rows.next()? {
        let audit_seq =
            u64::try_from(row.get::<_, i64>(0)?).map_err(|_| SecurityAuditError::InvalidRecord)?;
        if audit_seq != expected_seq {
            return Err(SecurityAuditError::Integrity(
                "authority-security audit sequence is not contiguous",
            ));
        }
        let kind: String = row.get(1)?;
        let record_id: Vec<u8> = row.get(2)?;
        let stored_payload_hash = hash_from_vec(row.get(3)?)?;
        let stored_previous_hash = optional_hash(row.get(4)?)?;
        let stored_record_hash = hash_from_vec(row.get(5)?)?;
        if stored_previous_hash != previous_hash {
            return Err(SecurityAuditError::Integrity(
                "authority-security previous hash mismatch",
            ));
        }
        let source_payload = source_payload(connection, &kind, &record_id)?;
        let expected_payload_hash = *blake3::hash(&source_payload).as_bytes();
        if stored_payload_hash != expected_payload_hash {
            return Err(SecurityAuditError::Integrity(
                "authority-security source payload hash mismatch",
            ));
        }
        let expected_hash = audit_record_hash(
            audit_seq,
            &kind,
            &record_id,
            stored_payload_hash,
            stored_previous_hash,
        )?;
        if stored_record_hash != expected_hash {
            return Err(SecurityAuditError::Integrity(
                "authority-security record hash mismatch",
            ));
        }
        last = Some((audit_seq, stored_record_hash));
        previous_hash = Some(stored_record_hash);
        expected_seq = expected_seq
            .checked_add(1)
            .ok_or(SecurityAuditError::SequenceOverflow)?;
    }
    drop(rows);
    drop(statement);
    verify_head(connection, last)
}

fn verify_coverage(connection: &Connection) -> Result<(), SecurityAuditError> {
    require_zero_missing(
        connection,
        "SELECT COUNT(*) FROM clients c WHERE NOT EXISTS (\
         SELECT 1 FROM authority_security_audit a \
         WHERE a.record_kind = ?1 AND a.record_id = c.client_id)",
        KIND_CLIENT_ENROLLED,
        "client enrollment is missing integrity-chain coverage",
    )?;
    require_zero_missing(
        connection,
        "SELECT COUNT(*) FROM clients c WHERE c.revoked_at IS NOT NULL AND NOT EXISTS (\
         SELECT 1 FROM authority_security_audit a \
         WHERE a.record_kind = ?1 AND a.record_id = c.client_id)",
        KIND_CLIENT_REVOKED,
        "client revocation is missing integrity-chain coverage",
    )?;
    require_zero_missing(
        connection,
        "SELECT COUNT(*) FROM authorization_decisions d WHERE NOT EXISTS (\
         SELECT 1 FROM authority_security_audit a \
         WHERE a.record_kind = ?1 AND a.record_id = d.decision_id)",
        KIND_AUTHORIZATION_DECISION,
        "authorization decision is missing integrity-chain coverage",
    )?;
    require_zero_missing(
        connection,
        "SELECT COUNT(*) FROM effect_intents i WHERE NOT EXISTS (\
         SELECT 1 FROM authority_security_audit a \
         WHERE a.record_kind = ?1 AND a.record_id = i.effect_id)",
        KIND_EFFECT_INTENT,
        "effect intent is missing integrity-chain coverage",
    )?;
    require_zero_missing(
        connection,
        "SELECT COUNT(*) FROM effect_transitions t WHERE NOT EXISTS (\
         SELECT 1 FROM authority_security_audit a \
         WHERE a.record_kind = ?1 AND a.record_id = t.transition_id)",
        KIND_EFFECT_TRANSITION,
        "effect transition is missing integrity-chain coverage",
    )?;
    require_zero_missing(
        connection,
        "SELECT COUNT(*) FROM effect_attempts e WHERE NOT EXISTS (\
         SELECT 1 FROM authority_security_audit a \
         WHERE a.record_kind = ?1 AND a.record_id = e.attempt_id)",
        KIND_EFFECT_ATTEMPT_STARTED,
        "effect attempt start is missing integrity-chain coverage",
    )?;
    require_zero_missing(
        connection,
        "SELECT COUNT(*) FROM effect_attempts e WHERE e.finished_at IS NOT NULL AND NOT EXISTS (\
         SELECT 1 FROM authority_security_audit a \
         WHERE a.record_kind = ?1 AND a.record_id = e.attempt_id)",
        KIND_EFFECT_ATTEMPT_FINISHED,
        "effect attempt finish is missing integrity-chain coverage",
    )?;
    require_zero_missing(
        connection,
        "SELECT COUNT(*) FROM recovery_incidents r WHERE NOT EXISTS (\
         SELECT 1 FROM authority_security_audit a \
         WHERE a.record_kind = ?1 AND a.record_id = r.incident_id)",
        KIND_RECOVERY_INCIDENT,
        "recovery incident is missing integrity-chain coverage",
    )?;
    Ok(())
}

fn require_zero_missing(
    connection: &Connection,
    query: &str,
    kind: &str,
    reason: &'static str,
) -> Result<(), SecurityAuditError> {
    let missing: i64 = connection.query_row(query, params![kind], |row| row.get(0))?;
    if missing == 0 {
        Ok(())
    } else {
        Err(SecurityAuditError::Coverage(reason))
    }
}

fn verify_head(
    connection: &Connection,
    computed: Option<(u64, [u8; 32])>,
) -> Result<(), SecurityAuditError> {
    let stored = connection
        .query_row(
            "SELECT last_global_seq, last_hash FROM audit_chain_heads WHERE chain_name = ?1",
            params![CHAIN_NAME],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?;
    let stored = match stored {
        Some((seq, hash)) => Some((
            u64::try_from(seq).map_err(|_| SecurityAuditError::InvalidRecord)?,
            hash_from_vec(hash)?,
        )),
        None => None,
    };
    if stored != computed {
        return Err(SecurityAuditError::Integrity(
            "authority-security audit chain head mismatch",
        ));
    }
    Ok(())
}

fn source_payload(
    connection: &Connection,
    kind: &str,
    record_id: &[u8],
) -> Result<Vec<u8>, SecurityAuditError> {
    match kind {
        KIND_CLIENT_ENROLLED => source_client_enrollment(connection, record_id),
        KIND_CLIENT_REVOKED => source_client_revocation(connection, record_id),
        KIND_AUTHORIZATION_DECISION => source_authorization(connection, record_id),
        KIND_EFFECT_INTENT => source_effect_intent(connection, record_id),
        KIND_EFFECT_TRANSITION => source_effect_transition(connection, record_id),
        KIND_EFFECT_ATTEMPT_STARTED => source_effect_attempt_started(connection, record_id),
        KIND_EFFECT_ATTEMPT_FINISHED => source_effect_attempt_finished(connection, record_id),
        KIND_RECOVERY_INCIDENT => source_recovery_incident(connection, record_id),
        _ => Err(SecurityAuditError::Integrity(
            "unknown authority-security audit record kind",
        )),
    }
}

fn source_client_enrollment(
    connection: &Connection,
    record_id: &[u8],
) -> Result<Vec<u8>, SecurityAuditError> {
    let row = connection
        .query_row(
            "SELECT key_id, public_key, kind, owner_principal, enrolled_at, assurance_class \
             FROM clients WHERE client_id = ?1",
            params![record_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecurityAuditError::Coverage(
            "client-enrollment audit record has no source row",
        ))?;
    encode_client_enrollment(&ClientEnrollmentAuditInput {
        client_id: record_id,
        key_id: &row.0,
        public_key: &row.1,
        kind: &row.2,
        owner_principal: &row.3,
        enrolled_at: &row.4,
        assurance_class: &row.5,
    })
}

fn source_client_revocation(
    connection: &Connection,
    record_id: &[u8],
) -> Result<Vec<u8>, SecurityAuditError> {
    let revoked_at = connection
        .query_row(
            "SELECT revoked_at FROM clients WHERE client_id = ?1",
            params![record_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()?
        .flatten()
        .ok_or(SecurityAuditError::Coverage(
            "client-revocation audit record has no revoked source row",
        ))?;
    encode_client_revocation(&ClientRevocationAuditInput {
        client_id: record_id,
        revoked_at: &revoked_at,
    })
}

fn source_authorization(
    connection: &Connection,
    record_id: &[u8],
) -> Result<Vec<u8>, SecurityAuditError> {
    let row = connection
        .query_row(
            "SELECT principal, action, resource, context_hash, decision, reason_code, global_seq \
             FROM authorization_decisions WHERE decision_id = ?1",
            params![record_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecurityAuditError::Coverage(
            "authorization audit record has no source row",
        ))?;
    encode_authorization(&AuthorizationAuditInput {
        decision_id: record_id,
        principal: &row.0,
        action: &row.1,
        resource: &row.2,
        context_hash: &row.3,
        decision: &row.4,
        reason_code: &row.5,
        global_seq: u64::try_from(row.6).map_err(|_| SecurityAuditError::InvalidRecord)?,
    })
}

fn source_effect_intent(
    connection: &Connection,
    record_id: &[u8],
) -> Result<Vec<u8>, SecurityAuditError> {
    let row = connection
        .query_row(
            "SELECT session_id, requested_by, action, resource, risk_class, execution_semantics, \
             idempotency_key, preconditions, dependencies, payload_hash, proposed_event_id \
             FROM effect_intents WHERE effect_id = ?1",
            params![record_id],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Vec<u8>>(7)?,
                    row.get::<_, Vec<u8>>(8)?,
                    row.get::<_, Vec<u8>>(9)?,
                    row.get::<_, Vec<u8>>(10)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecurityAuditError::Coverage(
            "effect-intent audit record has no source row",
        ))?;
    encode_effect_intent(&EffectIntentAuditInput {
        effect_id: record_id,
        session_id: &row.0,
        requested_by: &row.1,
        action: &row.2,
        resource: &row.3,
        risk_class: &row.4,
        execution_semantics: &row.5,
        idempotency_key: row.6.as_deref(),
        preconditions: &row.7,
        dependencies: &row.8,
        payload_hash: &row.9,
        proposed_event_id: &row.10,
    })
}

fn source_effect_transition(
    connection: &Connection,
    record_id: &[u8],
) -> Result<Vec<u8>, SecurityAuditError> {
    let row = connection
        .query_row(
            "SELECT effect_id, global_seq, from_state, to_state, attempt_id, reason_code, \
             evidence_ref, event_id FROM effect_transitions WHERE transition_id = ?1",
            params![record_id],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<Vec<u8>>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<Vec<u8>>>(6)?,
                    row.get::<_, Vec<u8>>(7)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecurityAuditError::Coverage(
            "effect-transition audit record has no source row",
        ))?;
    encode_effect_transition(&EffectTransitionAuditInput {
        transition_id: record_id,
        effect_id: &row.0,
        global_seq: u64::try_from(row.1).map_err(|_| SecurityAuditError::InvalidRecord)?,
        from_state: row.2.as_deref(),
        to_state: &row.3,
        attempt_id: row.4.as_deref(),
        reason_code: row.5.as_deref(),
        evidence_ref: row.6.as_deref(),
        event_id: &row.7,
    })
}

fn source_effect_attempt_started(
    connection: &Connection,
    record_id: &[u8],
) -> Result<Vec<u8>, SecurityAuditError> {
    let row = connection
        .query_row(
            "SELECT effect_id, started_global_seq, handler_id, handler_version, dispatch_token, \
             started_at FROM effect_attempts WHERE attempt_id = ?1",
            params![record_id],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecurityAuditError::Coverage(
            "effect-attempt-start audit record has no source row",
        ))?;
    encode_effect_attempt_started(&EffectAttemptStartedAuditInput {
        attempt_id: record_id,
        effect_id: &row.0,
        started_global_seq: u64::try_from(row.1).map_err(|_| SecurityAuditError::InvalidRecord)?,
        handler_id: &row.2,
        handler_version: &row.3,
        dispatch_token: &row.4,
        started_at: &row.5,
    })
}

fn source_effect_attempt_finished(
    connection: &Connection,
    record_id: &[u8],
) -> Result<Vec<u8>, SecurityAuditError> {
    let row = connection
        .query_row(
            "SELECT finished_at, outcome, receipt FROM effect_attempts WHERE attempt_id = ?1",
            params![record_id],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<Vec<u8>>>(2)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecurityAuditError::Coverage(
            "effect-attempt-finish audit record has no source row",
        ))?;
    let finished_at = row.0.as_deref().ok_or(SecurityAuditError::Coverage(
        "effect-attempt-finish audit exists for unfinished attempt",
    ))?;
    encode_effect_attempt_finished(&EffectAttemptFinishedAuditInput {
        attempt_id: record_id,
        finished_at,
        outcome: &row.1,
        receipt: row.2.as_deref(),
    })
}

fn source_recovery_incident(
    connection: &Connection,
    record_id: &[u8],
) -> Result<Vec<u8>, SecurityAuditError> {
    let row = connection
        .query_row(
            "SELECT detected_at, kind, severity, affected_refs, recovery_mode, resolution \
             FROM recovery_incidents WHERE incident_id = ?1",
            params![record_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<Vec<u8>>>(5)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecurityAuditError::Coverage(
            "recovery-incident audit record has no source row",
        ))?;
    encode_recovery_incident(&RecoveryIncidentAuditInput {
        incident_id: record_id,
        detected_at: &row.0,
        kind: &row.1,
        severity: &row.2,
        affected_refs: &row.3,
        recovery_mode: &row.4,
        resolution: row.5.as_deref(),
    })
}

fn encode_client_enrollment(
    input: &ClientEnrollmentAuditInput<'_>,
) -> Result<Vec<u8>, SecurityAuditError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(KIND_CLIENT_ENROLLED.as_bytes())?;
    encoder.push_bytes(input.client_id)?;
    encoder.push_bytes(input.key_id.as_bytes())?;
    encoder.push_bytes(input.public_key)?;
    encoder.push_bytes(input.kind.as_bytes())?;
    encoder.push_bytes(input.owner_principal.as_bytes())?;
    encoder.push_bytes(input.enrolled_at.as_bytes())?;
    encoder.push_bytes(input.assurance_class.as_bytes())?;
    Ok(encoder.finish())
}

fn encode_client_revocation(
    input: &ClientRevocationAuditInput<'_>,
) -> Result<Vec<u8>, SecurityAuditError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(KIND_CLIENT_REVOKED.as_bytes())?;
    encoder.push_bytes(input.client_id)?;
    encoder.push_bytes(input.revoked_at.as_bytes())?;
    Ok(encoder.finish())
}

fn encode_authorization(
    input: &AuthorizationAuditInput<'_>,
) -> Result<Vec<u8>, SecurityAuditError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(KIND_AUTHORIZATION_DECISION.as_bytes())?;
    encoder.push_bytes(input.decision_id)?;
    encoder.push_bytes(input.principal.as_bytes())?;
    encoder.push_bytes(input.action.as_bytes())?;
    encoder.push_bytes(input.resource.as_bytes())?;
    encoder.push_bytes(input.context_hash)?;
    encoder.push_bytes(input.decision.as_bytes())?;
    encoder.push_bytes(input.reason_code.as_bytes())?;
    encoder.push_u64(input.global_seq);
    Ok(encoder.finish())
}

fn encode_effect_intent(input: &EffectIntentAuditInput<'_>) -> Result<Vec<u8>, SecurityAuditError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(KIND_EFFECT_INTENT.as_bytes())?;
    encoder.push_bytes(input.effect_id)?;
    encoder.push_bytes(input.session_id)?;
    encoder.push_bytes(input.requested_by.as_bytes())?;
    encoder.push_bytes(input.action.as_bytes())?;
    encoder.push_bytes(input.resource.as_bytes())?;
    encoder.push_bytes(input.risk_class.as_bytes())?;
    encoder.push_bytes(input.execution_semantics.as_bytes())?;
    encode_optional_str(&mut encoder, input.idempotency_key)?;
    encoder.push_bytes(input.preconditions)?;
    encoder.push_bytes(input.dependencies)?;
    encoder.push_bytes(input.payload_hash)?;
    encoder.push_bytes(input.proposed_event_id)?;
    Ok(encoder.finish())
}

fn encode_effect_transition(
    input: &EffectTransitionAuditInput<'_>,
) -> Result<Vec<u8>, SecurityAuditError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(KIND_EFFECT_TRANSITION.as_bytes())?;
    encoder.push_bytes(input.transition_id)?;
    encoder.push_bytes(input.effect_id)?;
    encoder.push_u64(input.global_seq);
    encode_optional_str(&mut encoder, input.from_state)?;
    encoder.push_bytes(input.to_state.as_bytes())?;
    encode_optional_bytes(&mut encoder, input.attempt_id)?;
    encode_optional_str(&mut encoder, input.reason_code)?;
    encode_optional_bytes(&mut encoder, input.evidence_ref)?;
    encoder.push_bytes(input.event_id)?;
    Ok(encoder.finish())
}

fn encode_effect_attempt_started(
    input: &EffectAttemptStartedAuditInput<'_>,
) -> Result<Vec<u8>, SecurityAuditError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(KIND_EFFECT_ATTEMPT_STARTED.as_bytes())?;
    encoder.push_bytes(input.attempt_id)?;
    encoder.push_bytes(input.effect_id)?;
    encoder.push_u64(input.started_global_seq);
    encoder.push_bytes(input.handler_id.as_bytes())?;
    encoder.push_bytes(input.handler_version.as_bytes())?;
    encoder.push_bytes(input.dispatch_token)?;
    encoder.push_bytes(input.started_at.as_bytes())?;
    Ok(encoder.finish())
}

fn encode_effect_attempt_finished(
    input: &EffectAttemptFinishedAuditInput<'_>,
) -> Result<Vec<u8>, SecurityAuditError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(KIND_EFFECT_ATTEMPT_FINISHED.as_bytes())?;
    encoder.push_bytes(input.attempt_id)?;
    encoder.push_bytes(input.finished_at.as_bytes())?;
    encoder.push_bytes(input.outcome.as_bytes())?;
    encode_optional_bytes(&mut encoder, input.receipt)?;
    Ok(encoder.finish())
}

fn encode_recovery_incident(
    input: &RecoveryIncidentAuditInput<'_>,
) -> Result<Vec<u8>, SecurityAuditError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(KIND_RECOVERY_INCIDENT.as_bytes())?;
    encoder.push_bytes(input.incident_id)?;
    encoder.push_bytes(input.detected_at.as_bytes())?;
    encoder.push_bytes(input.kind.as_bytes())?;
    encoder.push_bytes(input.severity.as_bytes())?;
    encoder.push_bytes(input.affected_refs)?;
    encoder.push_bytes(input.recovery_mode.as_bytes())?;
    encode_optional_bytes(&mut encoder, input.resolution)?;
    Ok(encoder.finish())
}

fn audit_record_hash(
    audit_seq: u64,
    kind: &str,
    record_id: &[u8],
    payload_hash: [u8; 32],
    previous_hash: Option<[u8; 32]>,
) -> Result<[u8; 32], SecurityAuditError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(RECORD_DOMAIN)?;
    encoder.push_u64(audit_seq);
    encoder.push_bytes(kind.as_bytes())?;
    encoder.push_bytes(record_id)?;
    encoder.push_bytes(&payload_hash)?;
    match previous_hash {
        Some(hash) => {
            encoder.push_u8(1);
            encoder.push_bytes(&hash)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(*blake3::hash(&encoder.finish()).as_bytes())
}

fn encode_optional_str(
    encoder: &mut CanonicalEncoder,
    value: Option<&str>,
) -> Result<(), CoreError> {
    match value {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(value.as_bytes())?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn encode_optional_bytes(
    encoder: &mut CanonicalEncoder,
    value: Option<&[u8]>,
) -> Result<(), CoreError> {
    match value {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(value)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn hash_from_vec(value: Vec<u8>) -> Result<[u8; 32], SecurityAuditError> {
    value
        .try_into()
        .map_err(|_| SecurityAuditError::InvalidRecord)
}

fn optional_hash(value: Option<Vec<u8>>) -> Result<Option<[u8; 32]>, SecurityAuditError> {
    value.map(hash_from_vec).transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_authority_state_needs_no_secondary_chain() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE clients (client_id BLOB PRIMARY KEY, revoked_at TEXT);\
                 CREATE TABLE authorization_decisions (decision_id BLOB PRIMARY KEY);\
                 CREATE TABLE effect_intents (effect_id BLOB PRIMARY KEY);\
                 CREATE TABLE effect_transitions (transition_id BLOB PRIMARY KEY);\
                 CREATE TABLE effect_attempts (attempt_id BLOB PRIMARY KEY, finished_at TEXT);\
                 CREATE TABLE recovery_incidents (incident_id BLOB PRIMARY KEY);\
                 CREATE TABLE audit_chain_heads (chain_name TEXT PRIMARY KEY, last_global_seq INTEGER, last_hash BLOB);",
            )
            .unwrap();
        verify(&connection).unwrap();
    }
}
