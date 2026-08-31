#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use rusqlite::{Connection, OptionalExtension, params};

use crate::storage::{AuthorityStore, StorageError};

const SECRET_ID_BYTES: usize = 16;
const HANDLE_ID_BYTES: usize = 16;
const HASH_BYTES: usize = 32;
const MAX_CLASSIFICATION_BYTES: usize = 128;
const MAX_PRINCIPAL_BYTES: usize = 512;
const MAX_STATUS_BYTES: usize = 64;
const MAX_PURPOSE_SCOPE_BYTES: usize = 4096;
const MAX_ALGORITHM_METADATA_BYTES: usize = 4096;
const MAX_TIME_BYTES: usize = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretRecord {
    secret_id: [u8; SECRET_ID_BYTES],
    classification: String,
    owner_principal: String,
    current_version: u64,
    status: String,
    created_global_seq: u64,
    revoked_at: Option<String>,
}

impl SecretRecord {
    pub const fn secret_id(&self) -> [u8; SECRET_ID_BYTES] {
        self.secret_id
    }

    pub fn classification(&self) -> &str {
        &self.classification
    }

    pub fn owner_principal(&self) -> &str {
        &self.owner_principal
    }

    pub const fn current_version(&self) -> u64 {
        self.current_version
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub const fn created_global_seq(&self) -> u64 {
        self.created_global_seq
    }

    pub fn revoked_at(&self) -> Option<&str> {
        self.revoked_at.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretVersion {
    secret_id: [u8; SECRET_ID_BYTES],
    version: u64,
    algorithm_metadata: Vec<u8>,
    associated_data_hash: [u8; HASH_BYTES],
    created_global_seq: u64,
    rotated_from: Option<u64>,
    retired_at: Option<String>,
}

impl SecretVersion {
    pub const fn secret_id(&self) -> [u8; SECRET_ID_BYTES] {
        self.secret_id
    }

    pub const fn version(&self) -> u64 {
        self.version
    }

    pub fn algorithm_metadata(&self) -> &[u8] {
        &self.algorithm_metadata
    }

    pub const fn associated_data_hash(&self) -> [u8; HASH_BYTES] {
        self.associated_data_hash
    }

    pub const fn created_global_seq(&self) -> u64 {
        self.created_global_seq
    }

    pub const fn rotated_from(&self) -> Option<u64> {
        self.rotated_from
    }

    pub fn retired_at(&self) -> Option<&str> {
        self.retired_at.as_deref()
    }
}

/// Opaque reference to protected secret authority.
///
/// Callers can inspect only routing/scope metadata. There is deliberately no
/// constructor and no plaintext/ciphertext accessor; handles are loaded from
/// authenticated protected state and become useful only through later kernel
/// validation/broker paths.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretHandle {
    handle_id: [u8; HANDLE_ID_BYTES],
    secret_id: [u8; SECRET_ID_BYTES],
    version_constraint: Option<u64>,
    purpose_scope: Vec<u8>,
    expires_at: Option<String>,
}

impl SecretHandle {
    pub const fn handle_id(&self) -> [u8; HANDLE_ID_BYTES] {
        self.handle_id
    }

    pub const fn secret_id(&self) -> [u8; SECRET_ID_BYTES] {
        self.secret_id
    }

    pub const fn version_constraint(&self) -> Option<u64> {
        self.version_constraint
    }

    pub fn purpose_scope(&self) -> &[u8] {
        &self.purpose_scope
    }

    pub fn expires_at(&self) -> Option<&str> {
        self.expires_at.as_deref()
    }
}

#[derive(Debug)]
pub enum SecretInterfaceError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    AuthoritySecurity(String),
    InvalidStoredRecord(&'static str),
}

impl fmt::Display for SecretInterfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "secret interface authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "secret interface sqlite error: {error}"),
            Self::AuthoritySecurity(error) => {
                write!(f, "secret interface authority-security error: {error}")
            }
            Self::InvalidStoredRecord(reason) => {
                write!(f, "stored secret interface record is invalid: {reason}")
            }
        }
    }
}

impl Error for SecretInterfaceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::AuthoritySecurity(_) | Self::InvalidStoredRecord(_) => None,
        }
    }
}

impl From<StorageError> for SecretInterfaceError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for SecretInterfaceError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub struct SecretCatalog {
    connection: Connection,
}

impl SecretCatalog {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, SecretInterfaceError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA query_only = ON; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn record(
        &self,
        secret_id: [u8; SECRET_ID_BYTES],
    ) -> Result<Option<SecretRecord>, SecretInterfaceError> {
        self.verify_before_read()?;
        let row = self
            .connection
            .query_row(
                "SELECT secret_id, classification, owner_principal, current_version, status, created_global_seq, revoked_at FROM secret_records WHERE secret_id = ?1",
                params![&secret_id[..]],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, Option<String>>(6)?,
                    ))
                },
            )
            .optional()?;
        row.map(decode_secret_record).transpose()
    }

    pub fn version(
        &self,
        secret_id: [u8; SECRET_ID_BYTES],
        version: u64,
    ) -> Result<Option<SecretVersion>, SecretInterfaceError> {
        if version == 0 {
            return Err(SecretInterfaceError::InvalidStoredRecord(
                "secret version must be non-zero",
            ));
        }
        self.verify_before_read()?;
        let version_i64 = i64::try_from(version).map_err(|_| {
            SecretInterfaceError::InvalidStoredRecord("secret version exceeds sqlite integer range")
        })?;
        let row = self
            .connection
            .query_row(
                "SELECT secret_id, version, nonce_or_algorithm_metadata, associated_data_hash, created_global_seq, rotated_from, retired_at FROM secret_versions WHERE secret_id = ?1 AND version = ?2",
                params![&secret_id[..], version_i64],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                        row.get::<_, Vec<u8>>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, Option<i64>>(5)?,
                        row.get::<_, Option<String>>(6)?,
                    ))
                },
            )
            .optional()?;
        row.map(decode_secret_version).transpose()
    }

    pub fn handle(
        &self,
        handle_id: [u8; HANDLE_ID_BYTES],
    ) -> Result<Option<SecretHandle>, SecretInterfaceError> {
        self.verify_before_read()?;
        let row = self
            .connection
            .query_row(
                "SELECT handle_id, secret_id, version_constraint, purpose_scope, expires_at FROM secret_handles WHERE handle_id = ?1",
                params![&handle_id[..]],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, Vec<u8>>(1)?,
                        row.get::<_, Option<i64>>(2)?,
                        row.get::<_, Vec<u8>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                    ))
                },
            )
            .optional()?;
        let Some(row) = row else {
            return Ok(None);
        };
        let handle = decode_secret_handle(row)?;
        let exists = self
            .connection
            .query_row(
                "SELECT 1 FROM secret_records WHERE secret_id = ?1 LIMIT 1",
                params![&handle.secret_id[..]],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if !exists {
            return Err(SecretInterfaceError::InvalidStoredRecord(
                "secret handle references a missing secret record",
            ));
        }
        Ok(Some(handle))
    }

    fn verify_before_read(&self) -> Result<(), SecretInterfaceError> {
        crate::integrity::verify(&self.connection)
            .map_err(|error| SecretInterfaceError::AuthoritySecurity(error.to_string()))?;
        crate::authority_security_v2::verify(&self.connection)
            .map_err(|error| SecretInterfaceError::AuthoritySecurity(error.to_string()))
    }
}

type SecretRecordRow = (Vec<u8>, String, String, i64, String, i64, Option<String>);
type SecretVersionRow = (
    Vec<u8>,
    i64,
    Vec<u8>,
    Vec<u8>,
    i64,
    Option<i64>,
    Option<String>,
);
type SecretHandleRow = (Vec<u8>, Vec<u8>, Option<i64>, Vec<u8>, Option<String>);

fn decode_secret_record(row: SecretRecordRow) -> Result<SecretRecord, SecretInterfaceError> {
    let secret_id = fixed_id::<SECRET_ID_BYTES>(row.0, "secret_id must be exactly 16 bytes")?;
    validate_text(
        &row.1,
        MAX_CLASSIFICATION_BYTES,
        "classification is invalid",
    )?;
    validate_text(&row.2, MAX_PRINCIPAL_BYTES, "owner principal is invalid")?;
    let current_version = positive_u64(row.3, "current secret version must be positive")?;
    validate_text(&row.4, MAX_STATUS_BYTES, "secret status is invalid")?;
    let created_global_seq = nonnegative_u64(row.5, "created sequence is negative")?;
    validate_optional_text(&row.6, MAX_TIME_BYTES, "revocation timestamp is invalid")?;
    Ok(SecretRecord {
        secret_id,
        classification: row.1,
        owner_principal: row.2,
        current_version,
        status: row.4,
        created_global_seq,
        revoked_at: row.6,
    })
}

fn decode_secret_version(row: SecretVersionRow) -> Result<SecretVersion, SecretInterfaceError> {
    let secret_id = fixed_id::<SECRET_ID_BYTES>(row.0, "secret_id must be exactly 16 bytes")?;
    let version = positive_u64(row.1, "secret version must be positive")?;
    if row.2.is_empty() || row.2.len() > MAX_ALGORITHM_METADATA_BYTES {
        return Err(SecretInterfaceError::InvalidStoredRecord(
            "secret algorithm metadata is invalid or too large",
        ));
    }
    let associated_data_hash = fixed_id::<HASH_BYTES>(
        row.3,
        "secret associated-data hash must be exactly 32 bytes",
    )?;
    let created_global_seq = nonnegative_u64(row.4, "created sequence is negative")?;
    let rotated_from = row
        .5
        .map(|value| positive_u64(value, "rotated_from must be positive"))
        .transpose()?;
    if rotated_from.is_some_and(|previous| previous >= version) {
        return Err(SecretInterfaceError::InvalidStoredRecord(
            "rotated_from must reference an earlier version",
        ));
    }
    validate_optional_text(&row.6, MAX_TIME_BYTES, "retirement timestamp is invalid")?;
    Ok(SecretVersion {
        secret_id,
        version,
        algorithm_metadata: row.2,
        associated_data_hash,
        created_global_seq,
        rotated_from,
        retired_at: row.6,
    })
}

fn decode_secret_handle(row: SecretHandleRow) -> Result<SecretHandle, SecretInterfaceError> {
    let handle_id = fixed_id::<HANDLE_ID_BYTES>(row.0, "handle_id must be exactly 16 bytes")?;
    let secret_id = fixed_id::<SECRET_ID_BYTES>(row.1, "secret_id must be exactly 16 bytes")?;
    let version_constraint = row
        .2
        .map(|value| positive_u64(value, "handle version constraint must be positive"))
        .transpose()?;
    if row.3.is_empty() || row.3.len() > MAX_PURPOSE_SCOPE_BYTES {
        return Err(SecretInterfaceError::InvalidStoredRecord(
            "secret handle purpose scope is invalid or too large",
        ));
    }
    validate_optional_text(&row.4, MAX_TIME_BYTES, "handle expiry is invalid")?;
    Ok(SecretHandle {
        handle_id,
        secret_id,
        version_constraint,
        purpose_scope: row.3,
        expires_at: row.4,
    })
}

fn fixed_id<const N: usize>(
    value: Vec<u8>,
    reason: &'static str,
) -> Result<[u8; N], SecretInterfaceError> {
    value
        .try_into()
        .map_err(|_| SecretInterfaceError::InvalidStoredRecord(reason))
}

fn positive_u64(value: i64, reason: &'static str) -> Result<u64, SecretInterfaceError> {
    let value = nonnegative_u64(value, reason)?;
    if value == 0 {
        return Err(SecretInterfaceError::InvalidStoredRecord(reason));
    }
    Ok(value)
}

fn nonnegative_u64(value: i64, reason: &'static str) -> Result<u64, SecretInterfaceError> {
    u64::try_from(value).map_err(|_| SecretInterfaceError::InvalidStoredRecord(reason))
}

fn validate_text(
    value: &str,
    max_bytes: usize,
    reason: &'static str,
) -> Result<(), SecretInterfaceError> {
    if value.is_empty()
        || value.len() > max_bytes
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(SecretInterfaceError::InvalidStoredRecord(reason));
    }
    Ok(())
}

fn validate_optional_text(
    value: &Option<String>,
    max_bytes: usize,
    reason: &'static str,
) -> Result<(), SecretInterfaceError> {
    if let Some(value) = value {
        validate_text(value, max_bytes, reason)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority_security_write::{
        append_secret_handle_snapshot, append_secret_record_snapshot,
        append_secret_version_snapshot,
    };
    use rusqlite::TransactionBehavior;
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
            "golam-secret-interface-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    use golam_core::paths::RuntimeLayout;

    fn install_protected_fixture(authority: &AuthorityLayout) {
        let store = AuthorityStore::open(authority.authority_db_path()).unwrap();
        drop(store);
        let mut connection = Connection::open(authority.authority_db_path()).unwrap();
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        let secret_id = [1_u8; SECRET_ID_BYTES];
        let handle_id = [2_u8; HANDLE_ID_BYTES];
        transaction
            .execute(
                "INSERT INTO secret_records (secret_id, classification, owner_principal, current_version, status, created_global_seq, revoked_at) VALUES (?1, 'api_credential', 'owner:owner', 2, 'active', 10, NULL)",
                params![&secret_id[..]],
            )
            .unwrap();
        append_secret_record_snapshot(&transaction, &secret_id).unwrap();
        transaction
            .execute(
                "INSERT INTO secret_versions (secret_id, version, ciphertext, nonce_or_algorithm_metadata, associated_data_hash, created_global_seq, rotated_from, retired_at) VALUES (?1, 2, ?2, ?3, ?4, 11, 1, NULL)",
                params![
                    &secret_id[..],
                    b"encrypted-canary-bytes".as_slice(),
                    b"aes-256-gcm:v1;nonce-ref=test".as_slice(),
                    &[3_u8; HASH_BYTES][..],
                ],
            )
            .unwrap();
        append_secret_version_snapshot(&transaction, &secret_id, 2).unwrap();
        transaction
            .execute(
                "INSERT INTO secret_handles (handle_id, secret_id, version_constraint, purpose_scope, expires_at) VALUES (?1, ?2, 2, ?3, '2026-08-29T00:00:00Z')",
                params![&handle_id[..], &secret_id[..], b"purpose=test-api".as_slice()],
            )
            .unwrap();
        append_secret_handle_snapshot(&transaction, &handle_id).unwrap();
        crate::authority_security_v2::verify(&transaction).unwrap();
        transaction.commit().unwrap();
    }

    #[test]
    fn protected_catalog_exposes_metadata_and_opaque_handle_only() {
        let (runtime, authority) = authority();
        install_protected_fixture(&authority);
        let catalog = SecretCatalog::open(&authority).unwrap();
        let record = catalog.record([1; SECRET_ID_BYTES]).unwrap().unwrap();
        assert_eq!(record.secret_id(), [1; SECRET_ID_BYTES]);
        assert_eq!(record.classification(), "api_credential");
        assert_eq!(record.owner_principal(), "owner:owner");
        assert_eq!(record.current_version(), 2);
        assert_eq!(record.status(), "active");
        assert_eq!(record.created_global_seq(), 10);
        assert_eq!(record.revoked_at(), None);

        let version = catalog.version([1; SECRET_ID_BYTES], 2).unwrap().unwrap();
        assert_eq!(version.secret_id(), [1; SECRET_ID_BYTES]);
        assert_eq!(version.version(), 2);
        assert_eq!(
            version.algorithm_metadata(),
            b"aes-256-gcm:v1;nonce-ref=test"
        );
        assert_eq!(version.associated_data_hash(), [3; HASH_BYTES]);
        assert_eq!(version.created_global_seq(), 11);
        assert_eq!(version.rotated_from(), Some(1));
        assert_eq!(version.retired_at(), None);
        assert!(!format!("{version:?}").contains("encrypted-canary-bytes"));

        let handle = catalog.handle([2; HANDLE_ID_BYTES]).unwrap().unwrap();
        assert_eq!(handle.handle_id(), [2; HANDLE_ID_BYTES]);
        assert_eq!(handle.secret_id(), [1; SECRET_ID_BYTES]);
        assert_eq!(handle.version_constraint(), Some(2));
        assert_eq!(handle.purpose_scope(), b"purpose=test-api");
        assert_eq!(handle.expires_at(), Some("2026-08-29T00:00:00Z"));

        drop(catalog);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn missing_authenticated_secret_snapshot_fails_closed() {
        let (runtime, authority) = authority();
        let store = AuthorityStore::open(authority.authority_db_path()).unwrap();
        drop(store);
        let connection = Connection::open(authority.authority_db_path()).unwrap();
        connection
            .execute(
                "INSERT INTO secret_records (secret_id, classification, owner_principal, current_version, status, created_global_seq, revoked_at) VALUES (?1, 'api_credential', 'owner:owner', 1, 'active', 1, NULL)",
                params![&[9_u8; SECRET_ID_BYTES][..]],
            )
            .unwrap();
        drop(connection);

        assert!(matches!(
            SecretCatalog::open(&authority),
            Err(SecretInterfaceError::Storage(
                StorageError::IntegrityCheckFailed(_)
            ))
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn malformed_handle_reference_fails_closed_after_authenticated_read() {
        let (runtime, authority) = authority();
        let store = AuthorityStore::open(authority.authority_db_path()).unwrap();
        drop(store);
        let mut connection = Connection::open(authority.authority_db_path()).unwrap();
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        let handle_id = [7_u8; HANDLE_ID_BYTES];
        transaction
            .execute(
                "INSERT INTO secret_handles (handle_id, secret_id, version_constraint, purpose_scope, expires_at) VALUES (?1, ?2, 1, ?3, NULL)",
                params![
                    &handle_id[..],
                    &[8_u8; SECRET_ID_BYTES - 1][..],
                    b"purpose=test".as_slice(),
                ],
            )
            .unwrap();
        append_secret_handle_snapshot(&transaction, &handle_id).unwrap();
        crate::authority_security_v2::verify(&transaction).unwrap();
        transaction.commit().unwrap();

        let catalog = SecretCatalog::open(&authority).unwrap();
        assert!(matches!(
            catalog.handle(handle_id),
            Err(SecretInterfaceError::InvalidStoredRecord(
                "secret_id must be exactly 16 bytes"
            ))
        ));
        drop(catalog);
        drop(connection);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
