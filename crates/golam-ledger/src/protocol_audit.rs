#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::ClientId;
use golam_core::authority::AuthorityLayout;
use rusqlite::{Connection, params};

use crate::storage::{AuthorityStore, StorageError};

const PROTOCOL_AUDIT_DOMAIN: &[u8] = b"golam:protocol-audit:v1";
const PROTOCOL_INCIDENT_KIND: &str = "protocol";
const PROTOCOL_INCIDENT_SEVERITY: &str = "warning";
const PROTOCOL_RECOVERY_MODE: &str = "none";
const AFFECTED_REFS_VERSION: u8 = 1;
const AFFECTED_REFS_LEN: usize = 65;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolRejectionReason {
    UnauthenticatedRequest,
    UnknownClient,
    RevokedClient,
    ClientKeyMismatch,
    ClientNonceMismatch,
    KeyIdMismatch,
    AuthenticationFailed,
    InvalidPhase,
    ProtocolViolation,
}

impl ProtocolRejectionReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnauthenticatedRequest => "unauthenticated_request",
            Self::UnknownClient => "unknown_client",
            Self::RevokedClient => "revoked_client",
            Self::ClientKeyMismatch => "client_key_mismatch",
            Self::ClientNonceMismatch => "client_nonce_mismatch",
            Self::KeyIdMismatch => "key_id_mismatch",
            Self::AuthenticationFailed => "authentication_failed",
            Self::InvalidPhase => "invalid_phase",
            Self::ProtocolViolation => "protocol_violation",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "unauthenticated_request" => Some(Self::UnauthenticatedRequest),
            "unknown_client" => Some(Self::UnknownClient),
            "revoked_client" => Some(Self::RevokedClient),
            "client_key_mismatch" => Some(Self::ClientKeyMismatch),
            "client_nonce_mismatch" => Some(Self::ClientNonceMismatch),
            "key_id_mismatch" => Some(Self::KeyIdMismatch),
            "authentication_failed" => Some(Self::AuthenticationFailed),
            "invalid_phase" => Some(Self::InvalidPhase),
            "protocol_violation" => Some(Self::ProtocolViolation),
            _ => None,
        }
    }
}

pub struct AppendProtocolRejection<'a> {
    pub connection_id: u128,
    pub client_id: ClientId,
    pub key_id: [u8; 32],
    pub detected_at: &'a str,
    pub reason: ProtocolRejectionReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolAuditRecord {
    pub connection_id: u128,
    pub client_id: ClientId,
    pub key_id: [u8; 32],
    pub detected_at: String,
    pub reason: ProtocolRejectionReason,
}

#[derive(Debug)]
pub enum ProtocolAuditError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    InvalidMetadata,
    InvalidStoredRecord,
}

impl fmt::Display for ProtocolAuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "protocol audit authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "protocol audit sqlite error: {error}"),
            Self::InvalidMetadata => {
                f.write_str("protocol audit connection id and detected-at metadata are required")
            }
            Self::InvalidStoredRecord => f.write_str("stored protocol audit record is malformed"),
        }
    }
}

impl Error for ProtocolAuditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::InvalidMetadata | Self::InvalidStoredRecord => None,
        }
    }
}

impl From<StorageError> for ProtocolAuditError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for ProtocolAuditError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub struct ProtocolAuditLog {
    connection: Connection,
}

impl ProtocolAuditLog {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, ProtocolAuditError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn append_rejection(
        &mut self,
        input: AppendProtocolRejection<'_>,
    ) -> Result<ProtocolAuditRecord, ProtocolAuditError> {
        if input.connection_id == 0 || input.detected_at.is_empty() {
            return Err(ProtocolAuditError::InvalidMetadata);
        }
        let incident_id = protocol_incident_id(input.connection_id);
        let affected_refs = encode_affected_refs(input.connection_id, input.client_id, input.key_id);
        self.connection.execute(
            "INSERT INTO recovery_incidents \
             (incident_id, detected_at, kind, severity, affected_refs, recovery_mode, resolution) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &incident_id[..],
                input.detected_at,
                PROTOCOL_INCIDENT_KIND,
                PROTOCOL_INCIDENT_SEVERITY,
                affected_refs,
                PROTOCOL_RECOVERY_MODE,
                input.reason.as_str().as_bytes(),
            ],
        )?;
        Ok(ProtocolAuditRecord {
            connection_id: input.connection_id,
            client_id: input.client_id,
            key_id: input.key_id,
            detected_at: input.detected_at.to_owned(),
            reason: input.reason,
        })
    }

    pub fn records(&self) -> Result<Vec<ProtocolAuditRecord>, ProtocolAuditError> {
        let mut statement = self.connection.prepare(
            "SELECT detected_at, affected_refs, resolution \
             FROM recovery_incidents WHERE kind = ?1 ORDER BY rowid ASC",
        )?;
        let rows = statement.query_map(params![PROTOCOL_INCIDENT_KIND], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Vec<u8>>(1)?,
                row.get::<_, Option<Vec<u8>>>(2)?,
            ))
        })?;
        let mut records = Vec::new();
        for row in rows {
            let (detected_at, affected_refs, resolution) = row?;
            records.push(decode_record(detected_at, affected_refs, resolution)?);
        }
        Ok(records)
    }
}

fn protocol_incident_id(connection_id: u128) -> [u8; 16] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(PROTOCOL_AUDIT_DOMAIN);
    hasher.update(&connection_id.to_be_bytes());
    let hash = hasher.finalize();
    let mut incident_id = [0_u8; 16];
    incident_id.copy_from_slice(&hash.as_bytes()[..16]);
    incident_id
}

fn encode_affected_refs(
    connection_id: u128,
    client_id: ClientId,
    key_id: [u8; 32],
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(AFFECTED_REFS_LEN);
    bytes.push(AFFECTED_REFS_VERSION);
    bytes.extend_from_slice(&connection_id.to_be_bytes());
    bytes.extend_from_slice(&client_id.0.to_be_bytes());
    bytes.extend_from_slice(&key_id);
    bytes
}

fn decode_record(
    detected_at: String,
    affected_refs: Vec<u8>,
    resolution: Option<Vec<u8>>,
) -> Result<ProtocolAuditRecord, ProtocolAuditError> {
    if detected_at.is_empty()
        || affected_refs.len() != AFFECTED_REFS_LEN
        || affected_refs[0] != AFFECTED_REFS_VERSION
    {
        return Err(ProtocolAuditError::InvalidStoredRecord);
    }
    let connection_id = u128::from_be_bytes(
        affected_refs[1..17]
            .try_into()
            .map_err(|_| ProtocolAuditError::InvalidStoredRecord)?,
    );
    let client_id = ClientId(u128::from_be_bytes(
        affected_refs[17..33]
            .try_into()
            .map_err(|_| ProtocolAuditError::InvalidStoredRecord)?,
    ));
    let key_id = affected_refs[33..65]
        .try_into()
        .map_err(|_| ProtocolAuditError::InvalidStoredRecord)?;
    let resolution = resolution.ok_or(ProtocolAuditError::InvalidStoredRecord)?;
    let reason_text =
        std::str::from_utf8(&resolution).map_err(|_| ProtocolAuditError::InvalidStoredRecord)?;
    let reason = ProtocolRejectionReason::from_str(reason_text)
        .ok_or(ProtocolAuditError::InvalidStoredRecord)?;
    Ok(ProtocolAuditRecord {
        connection_id,
        client_id,
        key_id,
        detected_at,
        reason,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::paths::RuntimeLayout;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-protocol-audit-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    #[test]
    fn protocol_rejections_are_bounded_durable_records() {
        let (runtime, authority) = authority();
        let mut log = ProtocolAuditLog::open(&authority).unwrap();
        let record = log
            .append_rejection(AppendProtocolRejection {
                connection_id: 77,
                client_id: ClientId(41),
                key_id: [9; 32],
                detected_at: "2026-08-25T05:00:00Z",
                reason: ProtocolRejectionReason::AuthenticationFailed,
            })
            .unwrap();
        assert_eq!(record.connection_id, 77);
        assert_eq!(log.records().unwrap(), vec![record]);
        drop(log);
        AuthorityStore::open(authority.authority_db_path())
            .unwrap()
            .verify_integrity()
            .unwrap();
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn invalid_protocol_audit_metadata_is_rejected() {
        let (runtime, authority) = authority();
        let mut log = ProtocolAuditLog::open(&authority).unwrap();
        assert!(matches!(
            log.append_rejection(AppendProtocolRejection {
                connection_id: 0,
                client_id: ClientId(0),
                key_id: [0; 32],
                detected_at: "",
                reason: ProtocolRejectionReason::UnauthenticatedRequest,
            }),
            Err(ProtocolAuditError::InvalidMetadata)
        ));
        drop(log);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
