use std::error::Error;
use std::fmt;

use golam_core::ClientId;
use golam_core::authority::{AuthorityLayout, AuthorityPathError};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use crate::security_audit::{
    self, ClientEnrollmentAuditInput, ClientRevocationAuditInput,
};
use crate::storage::{AuthorityStore, StorageError};

pub const CLIENT_KEY_ID_LEN: usize = 32;
pub const CLIENT_PUBLIC_KEY_LEN: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientKind {
    Cli,
    DesktopFuture,
    IdeFuture,
    Test,
}
impl ClientKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::DesktopFuture => "desktop_future",
            Self::IdeFuture => "ide_future",
            Self::Test => "test",
        }
    }
    fn from_str(value: &str) -> Option<Self> {
        match value {
            "cli" => Some(Self::Cli),
            "desktop_future" => Some(Self::DesktopFuture),
            "ide_future" => Some(Self::IdeFuture),
            "test" => Some(Self::Test),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssuranceClass {
    OsProtectedV1,
    FilesystemUserPrivateV1,
}
impl AssuranceClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OsProtectedV1 => "os_protected_v1",
            Self::FilesystemUserPrivateV1 => "filesystem_user_private_v1",
        }
    }
    fn from_str(value: &str) -> Option<Self> {
        match value {
            "os_protected_v1" => Some(Self::OsProtectedV1),
            "filesystem_user_private_v1" => Some(Self::FilesystemUserPrivateV1),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientRecord {
    pub client_id: ClientId,
    pub key_id: [u8; CLIENT_KEY_ID_LEN],
    pub public_key: [u8; CLIENT_PUBLIC_KEY_LEN],
    pub kind: ClientKind,
    pub owner_principal: String,
    pub enrolled_at: String,
    pub last_authenticated_at: Option<String>,
    pub revoked_at: Option<String>,
    pub assurance_class: AssuranceClass,
}

pub struct EnrollClient<'a> {
    pub client_id: ClientId,
    pub key_id: [u8; CLIENT_KEY_ID_LEN],
    pub public_key: [u8; CLIENT_PUBLIC_KEY_LEN],
    pub kind: ClientKind,
    pub owner_principal: &'a str,
    pub enrolled_at: &'a str,
    pub assurance_class: AssuranceClass,
}

#[derive(Debug)]
pub enum ClientRegistryError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    AuthorityPath(AuthorityPathError),
    SecurityAudit(String),
    InvalidClientId,
    InvalidKeyId,
    InvalidPublicKey,
    InvalidMetadata,
    AlreadyEnrolled,
    UnknownClient,
    RevokedClient,
    ClientKeyMismatch,
    InvalidStoredClientId,
    InvalidStoredKeyId,
    InvalidStoredPublicKey,
    InvalidStoredKind(String),
    InvalidStoredAssurance(String),
}
impl fmt::Display for ClientRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(e) => write!(f, "client registry authority-store error: {e}"),
            Self::Sqlite(e) => write!(f, "client registry sqlite error: {e}"),
            Self::AuthorityPath(e) => write!(f, "client registry path error: {e}"),
            Self::SecurityAudit(e) => write!(f, "client registry integrity-chain error: {e}"),
            Self::InvalidClientId => f.write_str("client id must be non-zero"),
            Self::InvalidKeyId => f.write_str("client key id must not be all zero"),
            Self::InvalidPublicKey => f.write_str("client public key must not be all zero"),
            Self::InvalidMetadata => f.write_str("client enrollment metadata must not be empty"),
            Self::AlreadyEnrolled => {
                f.write_str("client id or key id is already enrolled and cannot be overwritten")
            }
            Self::UnknownClient => f.write_str("client is not enrolled"),
            Self::RevokedClient => f.write_str("client enrollment is revoked"),
            Self::ClientKeyMismatch => f.write_str("client key id does not match enrolled client"),
            Self::InvalidStoredClientId => f.write_str("stored client id is malformed"),
            Self::InvalidStoredKeyId => f.write_str("stored client key id is malformed"),
            Self::InvalidStoredPublicKey => f.write_str("stored client public key is malformed"),
            Self::InvalidStoredKind(v) => write!(f, "stored client kind is unsupported: {v}"),
            Self::InvalidStoredAssurance(v) => {
                write!(f, "stored client assurance class is unsupported: {v}")
            }
        }
    }
}
impl Error for ClientRegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(e) => Some(e),
            Self::Sqlite(e) => Some(e),
            Self::AuthorityPath(e) => Some(e),
            _ => None,
        }
    }
}
impl From<StorageError> for ClientRegistryError {
    fn from(v: StorageError) -> Self {
        Self::Storage(v)
    }
}
impl From<rusqlite::Error> for ClientRegistryError {
    fn from(v: rusqlite::Error) -> Self {
        Self::Sqlite(v)
    }
}
impl From<AuthorityPathError> for ClientRegistryError {
    fn from(v: AuthorityPathError) -> Self {
        Self::AuthorityPath(v)
    }
}

pub struct ClientRegistry {
    connection: Connection,
}
impl ClientRegistry {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, ClientRegistryError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;")?;
        Ok(Self { connection })
    }

    pub fn enroll(&mut self, input: EnrollClient<'_>) -> Result<ClientRecord, ClientRegistryError> {
        validate_enrollment(&input)?;
        let client_blob = input.client_id.0.to_be_bytes();
        let key_text = encode_hex(&input.key_id);
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let duplicate = tx
            .query_row(
                "SELECT 1 FROM clients WHERE client_id = ?1 OR key_id = ?2 LIMIT 1",
                params![&client_blob[..], &key_text],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;
        if duplicate.is_some() {
            return Err(ClientRegistryError::AlreadyEnrolled);
        }
        tx.execute("INSERT INTO clients (client_id,key_id,public_key,kind,owner_principal,enrolled_at,last_authenticated_at,revoked_at,assurance_class) VALUES (?1,?2,?3,?4,?5,?6,NULL,NULL,?7)", params![&client_blob[..], &key_text, &input.public_key[..], input.kind.as_str(), input.owner_principal, input.enrolled_at, input.assurance_class.as_str()])?;
        security_audit::append_client_enrollment(
            &tx,
            ClientEnrollmentAuditInput {
                client_id: &client_blob,
                key_id: &key_text,
                public_key: &input.public_key,
                kind: input.kind.as_str(),
                owner_principal: input.owner_principal,
                enrolled_at: input.enrolled_at,
                assurance_class: input.assurance_class.as_str(),
            },
        )
        .map_err(|error| ClientRegistryError::SecurityAudit(error.to_string()))?;
        tx.commit()?;
        self.resolve_active(input.client_id, input.key_id)
    }

    pub fn resolve_active(
        &self,
        client_id: ClientId,
        key_id: [u8; 32],
    ) -> Result<ClientRecord, ClientRegistryError> {
        let record = self
            .record_for_client(client_id)?
            .ok_or(ClientRegistryError::UnknownClient)?;
        if record.key_id != key_id {
            return Err(ClientRegistryError::ClientKeyMismatch);
        }
        if record.revoked_at.is_some() {
            return Err(ClientRegistryError::RevokedClient);
        }
        Ok(record)
    }

    pub fn mark_authenticated(
        &mut self,
        client_id: ClientId,
        key_id: [u8; 32],
        authenticated_at: &str,
    ) -> Result<(), ClientRegistryError> {
        if authenticated_at.is_empty() {
            return Err(ClientRegistryError::InvalidMetadata);
        }
        self.resolve_active(client_id, key_id)?;
        let client_blob = client_id.0.to_be_bytes();
        let updated = self.connection.execute("UPDATE clients SET last_authenticated_at=?1 WHERE client_id=?2 AND key_id=?3 AND revoked_at IS NULL", params![authenticated_at, &client_blob[..], encode_hex(&key_id)])?;
        if updated != 1 {
            return Err(ClientRegistryError::UnknownClient);
        }
        Ok(())
    }

    pub fn revoke(
        &mut self,
        client_id: ClientId,
        revoked_at: &str,
    ) -> Result<ClientRecord, ClientRegistryError> {
        if revoked_at.is_empty() {
            return Err(ClientRegistryError::InvalidMetadata);
        }
        let current = self
            .record_for_client(client_id)?
            .ok_or(ClientRegistryError::UnknownClient)?;
        if current.revoked_at.is_some() {
            return Err(ClientRegistryError::RevokedClient);
        }
        let blob = client_id.0.to_be_bytes();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let updated = tx.execute(
            "UPDATE clients SET revoked_at=?1 WHERE client_id=?2 AND revoked_at IS NULL",
            params![revoked_at, &blob[..]],
        )?;
        if updated != 1 {
            return Err(ClientRegistryError::RevokedClient);
        }
        security_audit::append_client_revocation(
            &tx,
            ClientRevocationAuditInput {
                client_id: &blob,
                revoked_at,
            },
        )
        .map_err(|error| ClientRegistryError::SecurityAudit(error.to_string()))?;
        tx.commit()?;
        self.record_for_client(client_id)?
            .ok_or(ClientRegistryError::UnknownClient)
    }

    pub fn record_for_client(
        &self,
        client_id: ClientId,
    ) -> Result<Option<ClientRecord>, ClientRegistryError> {
        let blob = client_id.0.to_be_bytes();
        let raw = self.connection.query_row("SELECT client_id,key_id,public_key,kind,owner_principal,enrolled_at,last_authenticated_at,revoked_at,assurance_class FROM clients WHERE client_id=?1", params![&blob[..]], |r| Ok((r.get::<_,Vec<u8>>(0)?,r.get::<_,String>(1)?,r.get::<_,Vec<u8>>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?,r.get::<_,Option<String>>(6)?,r.get::<_,Option<String>>(7)?,r.get::<_,String>(8)?))).optional()?;
        raw.map(parse_record).transpose()
    }
}

fn validate_enrollment(input: &EnrollClient<'_>) -> Result<(), ClientRegistryError> {
    if input.client_id.0 == 0 {
        return Err(ClientRegistryError::InvalidClientId);
    }
    if input.key_id.iter().all(|b| *b == 0) {
        return Err(ClientRegistryError::InvalidKeyId);
    }
    if input.public_key.iter().all(|b| *b == 0) {
        return Err(ClientRegistryError::InvalidPublicKey);
    }
    if input.owner_principal.is_empty() || input.enrolled_at.is_empty() {
        return Err(ClientRegistryError::InvalidMetadata);
    }
    Ok(())
}

type RawClient = (
    Vec<u8>,
    String,
    Vec<u8>,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    String,
);
fn parse_record(raw: RawClient) -> Result<ClientRecord, ClientRegistryError> {
    let client_bytes: [u8; 16] = raw
        .0
        .try_into()
        .map_err(|_| ClientRegistryError::InvalidStoredClientId)?;
    let key_id = decode_hex_32(&raw.1).ok_or(ClientRegistryError::InvalidStoredKeyId)?;
    let public_key: [u8; 32] = raw
        .2
        .try_into()
        .map_err(|_| ClientRegistryError::InvalidStoredPublicKey)?;
    let kind = ClientKind::from_str(&raw.3)
        .ok_or_else(|| ClientRegistryError::InvalidStoredKind(raw.3.clone()))?;
    let assurance_class = AssuranceClass::from_str(&raw.8)
        .ok_or_else(|| ClientRegistryError::InvalidStoredAssurance(raw.8.clone()))?;
    Ok(ClientRecord {
        client_id: ClientId(u128::from_be_bytes(client_bytes)),
        key_id,
        public_key,
        kind,
        owner_principal: raw.4,
        enrolled_at: raw.5,
        last_authenticated_at: raw.6,
        revoked_at: raw.7,
        assurance_class,
    })
}
fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(char::from(HEX[(b >> 4) as usize]));
        out.push(char::from(HEX[(b & 0xf) as usize]));
    }
    out
}
fn decode_hex_32(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    let bytes = value.as_bytes();
    for i in 0..32 {
        out[i] = (nibble(bytes[i * 2])? << 4) | nibble(bytes[i * 2 + 1])?;
    }
    Some(out)
}
fn nibble(v: u8) -> Option<u8> {
    match v {
        b'0'..=b'9' => Some(v - b'0'),
        b'a'..=b'f' => Some(v - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::paths::RuntimeLayout;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static N: AtomicU64 = AtomicU64::new(0);
    fn layout() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let r = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-client-registry-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let a = AuthorityLayout::initialize(&r).unwrap();
        (r, a)
    }
    fn input(id: u128) -> EnrollClient<'static> {
        EnrollClient {
            client_id: ClientId(id),
            key_id: [7; 32],
            public_key: [9; 32],
            kind: ClientKind::Test,
            owner_principal: "owner",
            enrolled_at: "2026-08-25T00:00:00Z",
            assurance_class: AssuranceClass::FilesystemUserPrivateV1,
        }
    }
    #[test]
    fn enroll_authenticate_revoke_fail_closed() {
        let (r, a) = layout();
        let mut reg = ClientRegistry::open(&a).unwrap();
        reg.enroll(input(1)).unwrap();
        reg.mark_authenticated(ClientId(1), [7; 32], "2026-08-25T00:01:00Z")
            .unwrap();
        assert_eq!(
            reg.resolve_active(ClientId(1), [7; 32])
                .unwrap()
                .last_authenticated_at
                .as_deref(),
            Some("2026-08-25T00:01:00Z")
        );
        reg.revoke(ClientId(1), "2026-08-25T00:02:00Z").unwrap();
        assert!(matches!(
            reg.resolve_active(ClientId(1), [7; 32]),
            Err(ClientRegistryError::RevokedClient)
        ));
        drop(reg);
        AuthorityStore::open(a.authority_db_path())
            .unwrap()
            .verify_integrity()
            .unwrap();
        fs::remove_dir_all(r.root).unwrap();
    }
    #[test]
    fn duplicates_unknown_and_wrong_keys_rejected() {
        let (r, a) = layout();
        let mut reg = ClientRegistry::open(&a).unwrap();
        reg.enroll(input(1)).unwrap();
        assert!(matches!(
            reg.enroll(input(1)),
            Err(ClientRegistryError::AlreadyEnrolled)
        ));
        assert!(matches!(
            reg.resolve_active(ClientId(2), [7; 32]),
            Err(ClientRegistryError::UnknownClient)
        ));
        assert!(matches!(
            reg.resolve_active(ClientId(1), [8; 32]),
            Err(ClientRegistryError::ClientKeyMismatch)
        ));
        drop(reg);
        fs::remove_dir_all(r.root).unwrap();
    }
}
