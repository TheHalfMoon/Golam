#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use golam_core::{EventId, SCHEMA_VERSION, SessionId};
use rusqlite::{Connection, OptionalExtension, params};

use crate::{EventKind, EventRecord, audit_integrity_hash, event_integrity_hash, payload_hash};

const SECURITY_AUDIT_CHAIN: &str = "security";

#[derive(Debug)]
pub enum IntegrityError {
    Sqlite(rusqlite::Error),
    Core(golam_core::CoreError),
    Violation(&'static str),
}

impl fmt::Display for IntegrityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "sqlite error during canonical verification: {error}"),
            Self::Core(error) => write!(f, "canonical encoding error: {error}"),
            Self::Violation(reason) => write!(f, "canonical integrity violation: {reason}"),
        }
    }
}

impl Error for IntegrityError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::Violation(_) => None,
        }
    }
}

impl From<rusqlite::Error> for IntegrityError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<golam_core::CoreError> for IntegrityError {
    fn from(value: golam_core::CoreError) -> Self {
        Self::Core(value)
    }
}

#[derive(Clone, Copy)]
struct SessionHead {
    session_seq: u64,
    event_hash: [u8; 32],
}

pub fn verify(connection: &Connection) -> Result<(), IntegrityError> {
    let mut statement = connection.prepare(
        "SELECT event_id, global_seq, session_id, session_seq, event_type, schema_version, \
         actor_principal, recorded_at, payload_bytes, payload_hash, previous_session_event_hash, \
         event_hash, security_critical, previous_audit_hash, audit_hash \
         FROM session_events ORDER BY global_seq ASC",
    )?;
    let mut rows = statement.query([])?;
    let mut expected_global_seq = 1_u64;
    let mut session_heads: HashMap<SessionId, SessionHead> = HashMap::new();
    let mut audit_head: Option<(u64, [u8; 32])> = None;

    while let Some(row) = rows.next()? {
        let event_id = EventId(id_from_vec(row.get(0)?)?);
        let global_seq = seq_from_i64(row.get(1)?)?;
        let session_id = SessionId(id_from_vec(row.get(2)?)?);
        let session_seq = seq_from_i64(row.get(3)?)?;
        let event_code = u8::try_from(row.get::<_, i64>(4)?)
            .map_err(|_| IntegrityError::Violation("invalid event type code"))?;
        let kind = EventKind::from_code(event_code)
            .ok_or(IntegrityError::Violation("unknown event type code"))?;
        let schema_version = u16::try_from(row.get::<_, i64>(5)?)
            .map_err(|_| IntegrityError::Violation("invalid event schema version"))?;
        if schema_version != SCHEMA_VERSION {
            return Err(IntegrityError::Violation(
                "unsupported event schema version",
            ));
        }
        if global_seq != expected_global_seq {
            return Err(IntegrityError::Violation(
                "global event sequence is not contiguous",
            ));
        }

        let actor_principal: String = row.get(6)?;
        let recorded_at: String = row.get(7)?;
        let payload_bytes: Vec<u8> = row.get(8)?;
        let stored_payload_hash = hash_from_vec(row.get(9)?)?;
        if payload_hash(&payload_bytes) != stored_payload_hash {
            return Err(IntegrityError::Violation("payload hash mismatch"));
        }
        let previous_session_event_hash = optional_hash(row.get(10)?)?;
        let stored_event_hash = hash_from_vec(row.get(11)?)?;
        let security_critical: bool = row.get(12)?;
        let previous_audit_hash = optional_hash(row.get(13)?)?;
        let stored_audit_hash = optional_hash(row.get(14)?)?;

        match session_heads.get(&session_id) {
            Some(head) => {
                if session_seq != head.session_seq + 1 {
                    return Err(IntegrityError::Violation(
                        "per-session event sequence is not contiguous",
                    ));
                }
                if previous_session_event_hash != Some(head.event_hash) {
                    return Err(IntegrityError::Violation(
                        "session hash-chain link mismatch",
                    ));
                }
            }
            None => {
                if session_seq != 1 || previous_session_event_hash.is_some() {
                    return Err(IntegrityError::Violation(
                        "invalid first session event anchor",
                    ));
                }
                if !matches!(kind, EventKind::SessionCreated | EventKind::SessionForked) {
                    return Err(IntegrityError::Violation(
                        "invalid first session event type",
                    ));
                }
            }
        }

        let record = EventRecord {
            event_id,
            session_id,
            global_seq,
            session_seq,
            schema_version,
            kind,
            actor_principal,
            recorded_at,
            payload_hash: stored_payload_hash,
            previous_session_event_hash,
            security_critical,
            previous_audit_hash,
        };
        let computed_event_hash = event_integrity_hash(&record)?;
        if computed_event_hash != stored_event_hash {
            return Err(IntegrityError::Violation("event hash mismatch"));
        }

        if security_critical {
            let expected_previous_audit = audit_head.map(|(_, hash)| hash);
            if previous_audit_hash != expected_previous_audit {
                return Err(IntegrityError::Violation("audit hash-chain link mismatch"));
            }
            let computed_audit_hash = audit_integrity_hash(&record, computed_event_hash)?;
            if stored_audit_hash != Some(computed_audit_hash) {
                return Err(IntegrityError::Violation("audit hash mismatch"));
            }
            audit_head = Some((global_seq, computed_audit_hash));
        } else if previous_audit_hash.is_some() || stored_audit_hash.is_some() {
            return Err(IntegrityError::Violation(
                "non-security event contains audit-chain fields",
            ));
        }

        session_heads.insert(
            session_id,
            SessionHead {
                session_seq,
                event_hash: computed_event_hash,
            },
        );
        expected_global_seq = expected_global_seq
            .checked_add(1)
            .ok_or(IntegrityError::Violation("global event sequence overflow"))?;
    }
    drop(rows);
    drop(statement);

    verify_session_heads(connection, &mut session_heads)?;
    verify_audit_head(connection, audit_head)?;
    Ok(())
}

fn verify_session_heads(
    connection: &Connection,
    computed_heads: &mut HashMap<SessionId, SessionHead>,
) -> Result<(), IntegrityError> {
    let mut statement = connection.prepare(
        "SELECT session_id, latest_session_seq, latest_event_hash FROM sessions ORDER BY session_id",
    )?;
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        let session_id = SessionId(id_from_vec(row.get(0)?)?);
        let latest_session_seq = seq_from_i64(row.get(1)?)?;
        let latest_event_hash = hash_from_vec(row.get(2)?)?;
        let computed = computed_heads.remove(&session_id).ok_or(IntegrityError::Violation(
            "session exists without canonical event",
        ))?;
        if computed.session_seq != latest_session_seq || computed.event_hash != latest_event_hash {
            return Err(IntegrityError::Violation(
                "session head does not match event chain",
            ));
        }
    }
    if !computed_heads.is_empty() {
        return Err(IntegrityError::Violation(
            "event chain exists for unknown session",
        ));
    }
    Ok(())
}

fn verify_audit_head(
    connection: &Connection,
    computed: Option<(u64, [u8; 32])>,
) -> Result<(), IntegrityError> {
    let stored = connection
        .query_row(
            "SELECT last_global_seq, last_hash FROM audit_chain_heads WHERE chain_name = ?1",
            params![SECURITY_AUDIT_CHAIN],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?;
    let stored = stored
        .map(|(seq, hash)| Ok((seq_from_i64(seq)?, hash_from_vec(hash)?)))
        .transpose()?;
    if stored != computed {
        return Err(IntegrityError::Violation("audit chain head mismatch"));
    }
    Ok(())
}

fn id_from_vec(value: Vec<u8>) -> Result<u128, IntegrityError> {
    let bytes: [u8; 16] = value
        .try_into()
        .map_err(|_| IntegrityError::Violation("stored identifier is not 16 bytes"))?;
    Ok(u128::from_be_bytes(bytes))
}

fn hash_from_vec(value: Vec<u8>) -> Result<[u8; 32], IntegrityError> {
    value
        .try_into()
        .map_err(|_| IntegrityError::Violation("stored hash is not 32 bytes"))
}

fn optional_hash(value: Option<Vec<u8>>) -> Result<Option<[u8; 32]>, IntegrityError> {
    value.map(hash_from_vec).transpose()
}

fn seq_from_i64(value: i64) -> Result<u64, IntegrityError> {
    u64::try_from(value).map_err(|_| IntegrityError::Violation("stored sequence is negative"))
}
