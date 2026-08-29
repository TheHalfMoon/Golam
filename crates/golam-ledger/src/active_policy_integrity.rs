#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::Path;
use std::str;

use rusqlite::{Connection, OpenFlags, OptionalExtension, TransactionBehavior, params};

use crate::policy::PolicyBundleId;
use crate::storage::{AuthorityStore, StorageError};

const POLICY_BUNDLE_DOMAIN: &[u8] = b"golam:policy-bundle:v1";
const POLICY_BUNDLE_ID_DOMAIN: &[u8] = b"golam:policy-bundle-id:v1";
const MAX_POLICY_SOURCE_BYTES: usize = 131_072;
const MAX_SCHEMA_SOURCE_BYTES: usize = 131_072;
const MAX_CANONICAL_POLICY_BUNDLE_BYTES: usize =
    4 + POLICY_BUNDLE_DOMAIN.len() + 8 + 4 + MAX_POLICY_SOURCE_BYTES + 4 + MAX_SCHEMA_SOURCE_BYTES;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedActivePolicy {
    pub policy_bundle_id: PolicyBundleId,
    pub version: u64,
    pub schema_version: u64,
    pub bundle_hash: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActivePolicyIntegrityState {
    Bootstrap,
    Active(VerifiedActivePolicy),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedActivePolicyBundle {
    pub policy: VerifiedActivePolicy,
    pub policy_source: String,
    pub schema_source: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActivePolicyLoadState {
    Bootstrap,
    Active(VerifiedActivePolicyBundle),
}

#[derive(Debug)]
pub enum ActivePolicyIntegrityError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    MissingActivePolicyAfterActivation,
    BundleNotFound,
    BundleNotValidated,
    InvalidBundleVersion,
    DuplicateBundleVersion,
    InvalidSchemaVersion,
    BundleTooLarge,
    InvalidCanonicalBundle,
    BundleHashMismatch,
    BundleIdMismatch,
    ActivePointerHashMismatch,
    InvalidLifecycleState,
    InvalidStoredRecord,
}

impl fmt::Display for ActivePolicyIntegrityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "active-policy authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "active-policy sqlite error: {error}"),
            Self::MissingActivePolicyAfterActivation => {
                f.write_str("active-policy pointer is missing after a prior normal activation")
            }
            Self::BundleNotFound => {
                f.write_str("active-policy pointer does not resolve to an immutable bundle")
            }
            Self::BundleNotValidated => {
                f.write_str("active-policy bundle is not in validated lifecycle state")
            }
            Self::InvalidBundleVersion => {
                f.write_str("active-policy bundle version is not a positive canonical version")
            }
            Self::DuplicateBundleVersion => {
                f.write_str("active-policy bundle version is not globally unique")
            }
            Self::InvalidSchemaVersion => {
                f.write_str("active-policy bundle schema version is invalid")
            }
            Self::BundleTooLarge => {
                f.write_str("active-policy canonical bundle exceeds the qualified size bound")
            }
            Self::InvalidCanonicalBundle => {
                f.write_str("active-policy canonical bundle encoding is malformed")
            }
            Self::BundleHashMismatch => {
                f.write_str("active-policy immutable bundle hash does not match its bytes")
            }
            Self::BundleIdMismatch => {
                f.write_str("active-policy immutable bundle id does not match its hash")
            }
            Self::ActivePointerHashMismatch => {
                f.write_str("active-policy pointer hash does not match the resolved bundle")
            }
            Self::InvalidLifecycleState => {
                f.write_str("active-policy lifecycle sequencing is invalid")
            }
            Self::InvalidStoredRecord => f.write_str("active-policy protected record is malformed"),
        }
    }
}

impl Error for ActivePolicyIntegrityError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for ActivePolicyIntegrityError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for ActivePolicyIntegrityError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub fn load_path(
    path: impl AsRef<Path>,
) -> Result<ActivePolicyLoadState, ActivePolicyIntegrityError> {
    let path = path.as_ref();
    let store = AuthorityStore::open(path)?;
    drop(store);
    let mut connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    connection.execute_batch(
        "PRAGMA foreign_keys = ON; PRAGMA query_only = ON; PRAGMA busy_timeout = 5000;",
    )?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Deferred)?;
    let state = verify_connection(&transaction)?;
    match state {
        ActivePolicyIntegrityState::Bootstrap => Ok(ActivePolicyLoadState::Bootstrap),
        ActivePolicyIntegrityState::Active(policy) => {
            let canonical_policy_bytes = transaction.query_row(
                "SELECT canonical_policy_bytes FROM policy_bundles WHERE policy_bundle_id = ?1",
                params![&policy.policy_bundle_id.0[..]],
                |row| row.get::<_, Vec<u8>>(0),
            )?;
            let (policy_source, schema_source) =
                decode_canonical_bundle(&canonical_policy_bytes, policy.schema_version)?;
            Ok(ActivePolicyLoadState::Active(VerifiedActivePolicyBundle {
                policy,
                policy_source: policy_source.to_owned(),
                schema_source: schema_source.to_owned(),
            }))
        }
    }
}

pub fn verify_path(
    path: impl AsRef<Path>,
) -> Result<ActivePolicyIntegrityState, ActivePolicyIntegrityError> {
    let path = path.as_ref();
    let store = AuthorityStore::open(path)?;
    drop(store);
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    connection.execute_batch(
        "PRAGMA foreign_keys = ON; PRAGMA query_only = ON; PRAGMA busy_timeout = 5000;",
    )?;
    verify_connection(&connection)
}

fn verify_connection(
    connection: &Connection,
) -> Result<ActivePolicyIntegrityState, ActivePolicyIntegrityError> {
    let active = connection
        .query_row(
            "SELECT policy_bundle_id, bundle_hash, activated_by, activation_effect_id, activated_global_seq FROM active_policy WHERE singleton_id = 1",
            [],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            },
        )
        .optional()?;

    let Some(active) = active else {
        let was_activated = connection
            .query_row(
                "SELECT 1 FROM authority_security_audit_v2 WHERE record_kind = 'active_policy' LIMIT 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        return if was_activated {
            Err(ActivePolicyIntegrityError::MissingActivePolicyAfterActivation)
        } else {
            Ok(ActivePolicyIntegrityState::Bootstrap)
        };
    };

    let policy_bundle_id = PolicyBundleId(id16(active.0)?);
    let active_bundle_hash = hash32(active.1)?;
    if active.2.is_empty() || active.3.len() != 16 {
        return Err(ActivePolicyIntegrityError::InvalidStoredRecord);
    }
    let activated_global_seq =
        positive_u64(active.4).ok_or(ActivePolicyIntegrityError::InvalidLifecycleState)?;

    let bundle = connection
        .query_row(
            "SELECT version, schema_version, length(canonical_policy_bytes), canonical_policy_bytes, bundle_hash, created_by, created_global_seq, validation_status FROM policy_bundles WHERE policy_bundle_id = ?1",
            params![&policy_bundle_id.0[..]],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, String>(7)?,
                ))
            },
        )
        .optional()?
        .ok_or(ActivePolicyIntegrityError::BundleNotFound)?;

    let version = positive_u64(bundle.0).ok_or(ActivePolicyIntegrityError::InvalidBundleVersion)?;
    let schema_version =
        positive_u64(bundle.1).ok_or(ActivePolicyIntegrityError::InvalidSchemaVersion)?;
    let stored_length =
        usize::try_from(bundle.2).map_err(|_| ActivePolicyIntegrityError::InvalidStoredRecord)?;
    if stored_length != bundle.3.len() {
        return Err(ActivePolicyIntegrityError::InvalidStoredRecord);
    }
    if stored_length > MAX_CANONICAL_POLICY_BUNDLE_BYTES {
        return Err(ActivePolicyIntegrityError::BundleTooLarge);
    }
    if bundle.5.is_empty() {
        return Err(ActivePolicyIntegrityError::InvalidStoredRecord);
    }
    let created_global_seq =
        positive_u64(bundle.6).ok_or(ActivePolicyIntegrityError::InvalidLifecycleState)?;
    if activated_global_seq <= created_global_seq {
        return Err(ActivePolicyIntegrityError::InvalidLifecycleState);
    }
    if bundle.7 != "validated" {
        return Err(ActivePolicyIntegrityError::BundleNotValidated);
    }

    let version_count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM policy_bundles WHERE version = ?1",
        params![bundle.0],
        |row| row.get(0),
    )?;
    if version_count != 1 {
        return Err(ActivePolicyIntegrityError::DuplicateBundleVersion);
    }

    verify_canonical_bundle(&bundle.3, schema_version)?;
    let bundle_hash = hash32(bundle.4)?;
    if *blake3::hash(&bundle.3).as_bytes() != bundle_hash {
        return Err(ActivePolicyIntegrityError::BundleHashMismatch);
    }
    if policy_bundle_id_from_hash(bundle_hash) != policy_bundle_id {
        return Err(ActivePolicyIntegrityError::BundleIdMismatch);
    }
    if active_bundle_hash != bundle_hash {
        return Err(ActivePolicyIntegrityError::ActivePointerHashMismatch);
    }

    Ok(ActivePolicyIntegrityState::Active(VerifiedActivePolicy {
        policy_bundle_id,
        version,
        schema_version,
        bundle_hash,
    }))
}

fn verify_canonical_bundle(
    bytes: &[u8],
    expected_schema_version: u64,
) -> Result<(), ActivePolicyIntegrityError> {
    decode_canonical_bundle(bytes, expected_schema_version).map(|_| ())
}

fn decode_canonical_bundle(
    bytes: &[u8],
    expected_schema_version: u64,
) -> Result<(&str, &str), ActivePolicyIntegrityError> {
    let mut offset = 0_usize;
    let domain = take_len_prefixed(bytes, &mut offset)?;
    if domain != POLICY_BUNDLE_DOMAIN {
        return Err(ActivePolicyIntegrityError::InvalidCanonicalBundle);
    }
    let schema_version = take_u64(bytes, &mut offset)?;
    if schema_version != expected_schema_version || schema_version == 0 {
        return Err(ActivePolicyIntegrityError::InvalidSchemaVersion);
    }
    let policy = take_len_prefixed(bytes, &mut offset)?;
    if policy.len() > MAX_POLICY_SOURCE_BYTES {
        return Err(ActivePolicyIntegrityError::InvalidCanonicalBundle);
    }
    let policy =
        str::from_utf8(policy).map_err(|_| ActivePolicyIntegrityError::InvalidCanonicalBundle)?;
    let schema = take_len_prefixed(bytes, &mut offset)?;
    if schema.len() > MAX_SCHEMA_SOURCE_BYTES {
        return Err(ActivePolicyIntegrityError::InvalidCanonicalBundle);
    }
    let schema =
        str::from_utf8(schema).map_err(|_| ActivePolicyIntegrityError::InvalidCanonicalBundle)?;
    if offset != bytes.len() {
        return Err(ActivePolicyIntegrityError::InvalidCanonicalBundle);
    }
    Ok((policy, schema))
}

fn take_len_prefixed<'a>(
    bytes: &'a [u8],
    offset: &mut usize,
) -> Result<&'a [u8], ActivePolicyIntegrityError> {
    let len_end = offset
        .checked_add(4)
        .ok_or(ActivePolicyIntegrityError::InvalidCanonicalBundle)?;
    let len_bytes: [u8; 4] = bytes
        .get(*offset..len_end)
        .ok_or(ActivePolicyIntegrityError::InvalidCanonicalBundle)?
        .try_into()
        .map_err(|_| ActivePolicyIntegrityError::InvalidCanonicalBundle)?;
    *offset = len_end;
    let len = usize::try_from(u32::from_be_bytes(len_bytes))
        .map_err(|_| ActivePolicyIntegrityError::InvalidCanonicalBundle)?;
    let end = offset
        .checked_add(len)
        .ok_or(ActivePolicyIntegrityError::InvalidCanonicalBundle)?;
    let value = bytes
        .get(*offset..end)
        .ok_or(ActivePolicyIntegrityError::InvalidCanonicalBundle)?;
    *offset = end;
    Ok(value)
}

fn take_u64(bytes: &[u8], offset: &mut usize) -> Result<u64, ActivePolicyIntegrityError> {
    let end = offset
        .checked_add(8)
        .ok_or(ActivePolicyIntegrityError::InvalidCanonicalBundle)?;
    let value: [u8; 8] = bytes
        .get(*offset..end)
        .ok_or(ActivePolicyIntegrityError::InvalidCanonicalBundle)?
        .try_into()
        .map_err(|_| ActivePolicyIntegrityError::InvalidCanonicalBundle)?;
    *offset = end;
    Ok(u64::from_be_bytes(value))
}

fn policy_bundle_id_from_hash(bundle_hash: [u8; 32]) -> PolicyBundleId {
    let mut hasher = blake3::Hasher::new();
    hasher.update(POLICY_BUNDLE_ID_DOMAIN);
    hasher.update(&bundle_hash);
    let hash = hasher.finalize();
    let mut id = [0_u8; 16];
    id.copy_from_slice(&hash.as_bytes()[..16]);
    PolicyBundleId(id)
}

fn id16(value: Vec<u8>) -> Result<[u8; 16], ActivePolicyIntegrityError> {
    value
        .try_into()
        .map_err(|_| ActivePolicyIntegrityError::InvalidStoredRecord)
}

fn hash32(value: Vec<u8>) -> Result<[u8; 32], ActivePolicyIntegrityError> {
    value
        .try_into()
        .map_err(|_| ActivePolicyIntegrityError::InvalidStoredRecord)
}

fn positive_u64(value: i64) -> Option<u64> {
    let value = u64::try_from(value).ok()?;
    (value > 0).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority_security_write::{
        append_active_policy_snapshot, append_policy_bundle_snapshot,
    };
    use golam_core::CanonicalEncoder;
    use golam_core::authority::AuthorityLayout;
    use golam_core::paths::RuntimeLayout;
    use rusqlite::{TransactionBehavior, params};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    const POLICY: &str =
        "permit(principal is User, action == Action::\"view\", resource is Photo);\n";
    const SCHEMA: &str = "entity User;\nentity Photo;\naction view appliesTo { principal: [User], resource: [Photo] };\n";

    static N: AtomicU64 = AtomicU64::new(0);

    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-active-policy-integrity-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        drop(AuthorityStore::open(authority.authority_db_path()).unwrap());
        (runtime, authority)
    }

    fn canonical_bundle(schema_version: u64) -> (PolicyBundleId, Vec<u8>, [u8; 32]) {
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(POLICY_BUNDLE_DOMAIN).unwrap();
        encoder.push_u64(schema_version);
        encoder.push_bytes(POLICY.as_bytes()).unwrap();
        encoder.push_bytes(SCHEMA.as_bytes()).unwrap();
        let bytes = encoder.finish();
        let hash = *blake3::hash(&bytes).as_bytes();
        (policy_bundle_id_from_hash(hash), bytes, hash)
    }

    fn seed_valid_active(authority: &AuthorityLayout) -> VerifiedActivePolicy {
        let (policy_bundle_id, bytes, bundle_hash) = canonical_bundle(1);
        let mut connection = Connection::open(authority.authority_db_path()).unwrap();
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        transaction
            .execute(
                "INSERT INTO policy_bundles (policy_bundle_id, version, schema_version, canonical_policy_bytes, bundle_hash, created_by, created_global_seq, validation_status) VALUES (?1, 1, 1, ?2, ?3, 'owner:owner', 1, 'validated')",
                params![&policy_bundle_id.0[..], &bytes, &bundle_hash[..]],
            )
            .unwrap();
        append_policy_bundle_snapshot(&transaction, &policy_bundle_id.0).unwrap();
        transaction
            .execute(
                "INSERT INTO active_policy (singleton_id, policy_bundle_id, bundle_hash, activated_by, activation_effect_id, activated_global_seq) VALUES (1, ?1, ?2, 'owner:owner', ?3, 2)",
                params![
                    &policy_bundle_id.0[..],
                    &bundle_hash[..],
                    &7_u128.to_be_bytes()[..],
                ],
            )
            .unwrap();
        append_active_policy_snapshot(&transaction).unwrap();
        crate::authority_security_v2::verify(&transaction).unwrap();
        transaction.commit().unwrap();
        VerifiedActivePolicy {
            policy_bundle_id,
            version: 1,
            schema_version: 1,
            bundle_hash,
        }
    }

    #[test]
    fn empty_store_is_bootstrap_eligible_only_before_first_activation() {
        let (runtime, authority) = authority();
        assert_eq!(
            verify_path(authority.authority_db_path()).unwrap(),
            ActivePolicyIntegrityState::Bootstrap
        );
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn material_loader_returns_exact_sources_from_one_verified_snapshot() {
        let (runtime, authority) = authority();
        let expected = seed_valid_active(&authority);
        let loaded = load_path(authority.authority_db_path()).unwrap();
        let ActivePolicyLoadState::Active(bundle) = loaded else {
            panic!("active bundle must load");
        };
        assert_eq!(bundle.policy, expected);
        assert_eq!(bundle.policy_source, POLICY);
        assert_eq!(bundle.schema_source, SCHEMA);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn verified_active_policy_survives_restart() {
        let (runtime, authority) = authority();
        let expected = seed_valid_active(&authority);
        assert_eq!(
            verify_path(authority.authority_db_path()).unwrap(),
            ActivePolicyIntegrityState::Active(expected.clone())
        );
        assert_eq!(
            verify_path(authority.authority_db_path()).unwrap(),
            ActivePolicyIntegrityState::Active(expected)
        );
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn tampered_bundle_bytes_fail_closed() {
        let (runtime, authority) = authority();
        seed_valid_active(&authority);
        let connection = Connection::open(authority.authority_db_path()).unwrap();
        connection
            .execute(
                "UPDATE policy_bundles SET canonical_policy_bytes = ?1 WHERE version = 1",
                params![b"tampered".as_slice()],
            )
            .unwrap();
        assert!(matches!(
            verify_connection(&connection),
            Err(ActivePolicyIntegrityError::InvalidCanonicalBundle)
                | Err(ActivePolicyIntegrityError::BundleHashMismatch)
        ));
        drop(connection);
        assert!(matches!(
            verify_path(authority.authority_db_path()),
            Err(ActivePolicyIntegrityError::Storage(_))
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn pointer_hash_mismatch_fails_closed() {
        let (runtime, authority) = authority();
        seed_valid_active(&authority);
        let connection = Connection::open(authority.authority_db_path()).unwrap();
        connection
            .execute(
                "UPDATE active_policy SET bundle_hash = ?1 WHERE singleton_id = 1",
                params![&[9_u8; 32][..]],
            )
            .unwrap();
        assert!(matches!(
            verify_connection(&connection),
            Err(ActivePolicyIntegrityError::ActivePointerHashMismatch)
        ));
        drop(connection);
        assert!(matches!(
            verify_path(authority.authority_db_path()),
            Err(ActivePolicyIntegrityError::Storage(_))
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn missing_active_bundle_fails_closed() {
        let (runtime, authority) = authority();
        seed_valid_active(&authority);
        let connection = Connection::open(authority.authority_db_path()).unwrap();
        connection
            .execute("DELETE FROM policy_bundles", [])
            .unwrap();
        assert!(matches!(
            verify_connection(&connection),
            Err(ActivePolicyIntegrityError::BundleNotFound)
        ));
        drop(connection);
        assert!(matches!(
            verify_path(authority.authority_db_path()),
            Err(ActivePolicyIntegrityError::Storage(_))
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn invalid_lifecycle_state_fails_closed() {
        let (runtime, authority) = authority();
        seed_valid_active(&authority);
        let connection = Connection::open(authority.authority_db_path()).unwrap();
        connection
            .execute(
                "UPDATE policy_bundles SET validation_status = 'staged' WHERE version = 1",
                [],
            )
            .unwrap();
        assert!(matches!(
            verify_connection(&connection),
            Err(ActivePolicyIntegrityError::BundleNotValidated)
        ));
        drop(connection);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn missing_active_pointer_after_activation_never_reenters_bootstrap() {
        let (runtime, authority) = authority();
        seed_valid_active(&authority);
        let connection = Connection::open(authority.authority_db_path()).unwrap();
        connection.execute("DELETE FROM active_policy", []).unwrap();
        assert!(matches!(
            verify_connection(&connection),
            Err(ActivePolicyIntegrityError::MissingActivePolicyAfterActivation)
        ));
        drop(connection);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn authority_security_coverage_is_mandatory() {
        let (runtime, authority) = authority();
        let (policy_bundle_id, bytes, bundle_hash) = canonical_bundle(1);
        let connection = Connection::open(authority.authority_db_path()).unwrap();
        connection
            .execute(
                "INSERT INTO policy_bundles (policy_bundle_id, version, schema_version, canonical_policy_bytes, bundle_hash, created_by, created_global_seq, validation_status) VALUES (?1, 1, 1, ?2, ?3, 'owner:owner', 1, 'validated')",
                params![&policy_bundle_id.0[..], &bytes, &bundle_hash[..]],
            )
            .unwrap();
        drop(connection);
        assert!(matches!(
            verify_path(authority.authority_db_path()),
            Err(ActivePolicyIntegrityError::Storage(_))
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
