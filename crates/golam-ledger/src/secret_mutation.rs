#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use zeroize::{Zeroize, Zeroizing};

use crate::authority_security_write::{
    append_approval_consumption_snapshot, append_secret_record_snapshot,
    append_secret_version_snapshot,
};
use crate::secret_vault::{KeyProtector, OsKeyProtector, SecretVault, VaultBinding, VaultError};
use crate::storage::{AuthorityStore, StorageError};

const SECRET_ID_BYTES: usize = 16;
const MAX_CLASSIFICATION_BYTES: usize = 128;
const MAX_PRINCIPAL_BYTES: usize = 512;
const MAX_SECRET_BYTES: usize = 65_536;
const SECURITY_METADATA_VERSION: u64 = 1;
const SECRET_CREATE_INTENT_DOMAIN: &[u8] = b"golam:secret-create-intent:v1";
const SECRET_ROTATE_INTENT_DOMAIN: &[u8] = b"golam:secret-rotate-intent:v1";
const SECRET_REVOKE_INTENT_DOMAIN: &[u8] = b"golam:secret-revoke-intent:v1";
const SECRET_ID_DOMAIN: &[u8] = b"golam:secret-id:v1";
const APPROVAL_CONSUMPTION_DOMAIN: &[u8] = b"golam:secret-approval-consumption:v1";
const SECRET_COMMITMENT_DOMAIN: &[u8] = b"golam:secret-value-commitment:v1";

pub(crate) const SECRET_CREATE_ACTION: &str = "secret.create";
pub(crate) const SECRET_ROTATE_ACTION: &str = "secret.rotate";
pub(crate) const SECRET_REVOKE_ACTION: &str = "secret.revoke";
pub(crate) const SECRET_MUTATION_RISK_CLASS: &str = "secret_vault_mutation";

pub(crate) struct PreparedSecretCreate {
    classification: String,
    owner_principal: String,
    plaintext: Zeroizing<Vec<u8>>,
    intent_digest: [u8; 32],
    resource: String,
}

impl PreparedSecretCreate {
    pub(crate) fn resource(&self) -> &str {
        &self.resource
    }

    pub(crate) const fn intent_digest(&self) -> [u8; 32] {
        self.intent_digest
    }
}

pub(crate) struct PreparedSecretRotate {
    secret_id: [u8; SECRET_ID_BYTES],
    expected_current_version: u64,
    plaintext: Zeroizing<Vec<u8>>,
    retired_at: String,
    intent_digest: [u8; 32],
    resource: String,
}

impl PreparedSecretRotate {
    pub(crate) fn resource(&self) -> &str {
        &self.resource
    }

    pub(crate) const fn intent_digest(&self) -> [u8; 32] {
        self.intent_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PreparedSecretRevoke {
    secret_id: [u8; SECRET_ID_BYTES],
    expected_current_version: u64,
    revoked_at: String,
    intent_digest: [u8; 32],
    resource: String,
}

impl PreparedSecretRevoke {
    pub(crate) fn resource(&self) -> &str {
        &self.resource
    }

    pub(crate) const fn intent_digest(&self) -> [u8; 32] {
        self.intent_digest
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SecretMutationOutcome {
    secret_id: [u8; SECRET_ID_BYTES],
    version: u64,
}

impl SecretMutationOutcome {
    pub(crate) const fn secret_id(self) -> [u8; SECRET_ID_BYTES] {
        self.secret_id
    }

    pub(crate) const fn version(self) -> u64 {
        self.version
    }
}

#[derive(Debug)]
pub(crate) enum SecretMutationError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Vault(VaultError),
    Random(getrandom::Error),
    Integrity(String),
    AuthoritySecurity(String),
    InvalidClassification,
    InvalidPrincipal,
    InvalidSecretValue,
    InvalidTime,
    InvalidVersion,
    IntegerOverflow,
    MissingAuthorityDecision,
    AuthorityDecisionMismatch,
    StaleAuthorityDecision,
    EffectNotFound,
    EffectMismatch,
    ApprovalNotFound,
    ApprovalMismatch,
    ApprovalAlreadyUsed,
    DuplicateSecret,
    SecretNotFound,
    SecretInactive,
    SecretAlreadyRevoked,
    StaleSecretVersion,
    SecretVersionNotFound,
    SecretVersionAlreadyRetired,
    InvalidStoredRecord(&'static str),
}

impl fmt::Display for SecretMutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "secret mutation authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "secret mutation sqlite error: {error}"),
            Self::Core(error) => write!(f, "secret mutation canonical encoding error: {error}"),
            Self::Vault(error) => write!(f, "secret mutation vault error: {error}"),
            Self::Random(error) => write!(f, "secret mutation random-source error: {error}"),
            Self::Integrity(error) => write!(f, "secret mutation integrity error: {error}"),
            Self::AuthoritySecurity(error) => {
                write!(f, "secret mutation authority-security error: {error}")
            }
            Self::InvalidClassification => {
                f.write_str("secret mutation classification is not canonical")
            }
            Self::InvalidPrincipal => f.write_str("secret mutation principal is not canonical"),
            Self::InvalidSecretValue => {
                f.write_str("secret mutation value is empty or exceeds the bounded size")
            }
            Self::InvalidTime => f.write_str("secret mutation timestamp is invalid"),
            Self::InvalidVersion => f.write_str("secret mutation version must be non-zero"),
            Self::IntegerOverflow => f.write_str("secret mutation integer conversion overflow"),
            Self::MissingAuthorityDecision => {
                f.write_str("secret mutation has no durable authorization decision")
            }
            Self::AuthorityDecisionMismatch => f.write_str(
                "secret mutation authorization decision does not match exact action/resource",
            ),
            Self::StaleAuthorityDecision => {
                f.write_str("secret mutation authorization decision is stale")
            }
            Self::EffectNotFound => f.write_str("secret mutation effect does not exist"),
            Self::EffectMismatch => f.write_str(
                "secret mutation effect is not exact authorized at-most-once elevated work",
            ),
            Self::ApprovalNotFound => f.write_str("secret mutation approval does not exist"),
            Self::ApprovalMismatch => f.write_str(
                "secret mutation approval does not match exact effect/action/resource/decision",
            ),
            Self::ApprovalAlreadyUsed => {
                f.write_str("secret mutation one-shot approval was already consumed")
            }
            Self::DuplicateSecret => f.write_str("secret already exists"),
            Self::SecretNotFound => f.write_str("secret does not exist"),
            Self::SecretInactive => f.write_str("secret is not active"),
            Self::SecretAlreadyRevoked => f.write_str("secret is already revoked"),
            Self::StaleSecretVersion => {
                f.write_str("secret current-version evidence is stale or mismatched")
            }
            Self::SecretVersionNotFound => f.write_str("secret current version does not exist"),
            Self::SecretVersionAlreadyRetired => {
                f.write_str("secret current version is already retired")
            }
            Self::InvalidStoredRecord(reason) => {
                write!(f, "stored secret mutation record is invalid: {reason}")
            }
        }
    }
}

impl Error for SecretMutationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::Vault(error) => Some(error),
            Self::Random(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for SecretMutationError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for SecretMutationError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for SecretMutationError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<VaultError> for SecretMutationError {
    fn from(value: VaultError) -> Self {
        Self::Vault(value)
    }
}

impl From<getrandom::Error> for SecretMutationError {
    fn from(value: getrandom::Error) -> Self {
        Self::Random(value)
    }
}

pub(crate) fn prepare_secret_create(
    classification: &str,
    owner_principal: &str,
    plaintext: Vec<u8>,
) -> Result<PreparedSecretCreate, SecretMutationError> {
    validate_classification(classification)?;
    validate_principal(owner_principal)?;
    validate_plaintext(&plaintext)?;
    let commitment = secret_commitment(&plaintext)?;
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(SECRET_CREATE_INTENT_DOMAIN)?;
    encoder.push_bytes(classification.as_bytes())?;
    encoder.push_bytes(owner_principal.as_bytes())?;
    encoder.push_bytes(&commitment)?;
    let intent_digest = crate::payload_hash(&encoder.finish());
    Ok(PreparedSecretCreate {
        classification: classification.to_owned(),
        owner_principal: owner_principal.to_owned(),
        plaintext: Zeroizing::new(plaintext),
        intent_digest,
        resource: format!("secret-create:{}", hex_bytes(&intent_digest)),
    })
}

pub(crate) fn prepare_secret_rotate(
    secret_id: [u8; SECRET_ID_BYTES],
    expected_current_version: u64,
    plaintext: Vec<u8>,
    retired_at: &str,
) -> Result<PreparedSecretRotate, SecretMutationError> {
    if expected_current_version == 0 {
        return Err(SecretMutationError::InvalidVersion);
    }
    validate_plaintext(&plaintext)?;
    if !valid_utc_second(retired_at) {
        return Err(SecretMutationError::InvalidTime);
    }
    let commitment = secret_commitment(&plaintext)?;
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(SECRET_ROTATE_INTENT_DOMAIN)?;
    encoder.push_bytes(&secret_id)?;
    encoder.push_u64(expected_current_version);
    encoder.push_bytes(retired_at.as_bytes())?;
    encoder.push_bytes(&commitment)?;
    let intent_digest = crate::payload_hash(&encoder.finish());
    Ok(PreparedSecretRotate {
        secret_id,
        expected_current_version,
        plaintext: Zeroizing::new(plaintext),
        retired_at: retired_at.to_owned(),
        intent_digest,
        resource: secret_resource(secret_id),
    })
}

pub(crate) fn prepare_secret_revoke(
    secret_id: [u8; SECRET_ID_BYTES],
    expected_current_version: u64,
    revoked_at: &str,
) -> Result<PreparedSecretRevoke, SecretMutationError> {
    if expected_current_version == 0 {
        return Err(SecretMutationError::InvalidVersion);
    }
    if !valid_utc_second(revoked_at) {
        return Err(SecretMutationError::InvalidTime);
    }
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(SECRET_REVOKE_INTENT_DOMAIN)?;
    encoder.push_bytes(&secret_id)?;
    encoder.push_u64(expected_current_version);
    encoder.push_bytes(revoked_at.as_bytes())?;
    let intent_digest = crate::payload_hash(&encoder.finish());
    Ok(PreparedSecretRevoke {
        secret_id,
        expected_current_version,
        revoked_at: revoked_at.to_owned(),
        intent_digest,
        resource: secret_resource(secret_id),
    })
}

pub(crate) struct SecretMutationStore {
    connection: Connection,
}

impl SecretMutationStore {
    pub(crate) fn open(layout: &AuthorityLayout) -> Result<Self, SecretMutationError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub(crate) fn create(
        &mut self,
        prepared: PreparedSecretCreate,
        authority_decision_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
    ) -> Result<SecretMutationOutcome, SecretMutationError> {
        self.create_inner(
            prepared,
            authority_decision_id,
            approval_id,
            effect_id,
            OsKeyProtector::new(),
        )
    }

    pub(crate) fn rotate(
        &mut self,
        prepared: PreparedSecretRotate,
        authority_decision_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
    ) -> Result<SecretMutationOutcome, SecretMutationError> {
        self.rotate_inner(
            prepared,
            authority_decision_id,
            approval_id,
            effect_id,
            OsKeyProtector::new(),
        )
    }

    pub(crate) fn revoke(
        &mut self,
        prepared: PreparedSecretRevoke,
        authority_decision_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
    ) -> Result<SecretMutationOutcome, SecretMutationError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        let authority = verify_current_authority(
            &transaction,
            authority_decision_id,
            SECRET_REVOKE_ACTION,
            &prepared.resource,
        )?;
        verify_mutation_effect(
            &transaction,
            effect_id,
            SECRET_REVOKE_ACTION,
            &prepared.resource,
            prepared.intent_digest,
        )?;
        verify_once_approval(
            &transaction,
            approval_id,
            authority_decision_id,
            effect_id,
            SECRET_REVOKE_ACTION,
            &prepared.resource,
        )?;
        let current = load_active_secret(&transaction, prepared.secret_id)?;
        if current.current_version != prepared.expected_current_version {
            return Err(SecretMutationError::StaleSecretVersion);
        }
        let changed = transaction.execute(
            "UPDATE secret_records SET status = 'revoked', revoked_at = ?2 WHERE secret_id = ?1 AND status = 'active' AND revoked_at IS NULL AND current_version = ?3",
            params![
                &prepared.secret_id[..],
                &prepared.revoked_at,
                to_i64(prepared.expected_current_version)?,
            ],
        )?;
        if changed != 1 {
            return Err(SecretMutationError::StaleSecretVersion);
        }
        append_secret_record_snapshot(&transaction, &prepared.secret_id)
            .map_err(|error| SecretMutationError::AuthoritySecurity(error.to_string()))?;
        consume_once_approval(&transaction, approval_id, effect_id, authority.global_seq)?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| SecretMutationError::AuthoritySecurity(error.to_string()))?;
        #[cfg(test)]
        test_pause_before_secret_commit("revoke");
        transaction.commit()?;
        Ok(SecretMutationOutcome {
            secret_id: prepared.secret_id,
            version: prepared.expected_current_version,
        })
    }

    fn create_inner<P: KeyProtector>(
        &mut self,
        prepared: PreparedSecretCreate,
        authority_decision_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
        protector: P,
    ) -> Result<SecretMutationOutcome, SecretMutationError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        let authority = verify_current_authority(
            &transaction,
            authority_decision_id,
            SECRET_CREATE_ACTION,
            &prepared.resource,
        )?;
        verify_mutation_effect(
            &transaction,
            effect_id,
            SECRET_CREATE_ACTION,
            &prepared.resource,
            prepared.intent_digest,
        )?;
        verify_once_approval(
            &transaction,
            approval_id,
            authority_decision_id,
            effect_id,
            SECRET_CREATE_ACTION,
            &prepared.resource,
        )?;

        let secret_id = derived_secret_id(
            prepared.intent_digest,
            effect_id,
            authority_decision_id,
            approval_id,
        );
        let duplicate = transaction
            .query_row(
                "SELECT 1 FROM secret_records WHERE secret_id = ?1 LIMIT 1",
                params![&secret_id[..]],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if duplicate {
            return Err(SecretMutationError::DuplicateSecret);
        }

        let metadata = load_all_algorithm_metadata(&transaction)?;
        let vault = SecretVault::from_persisted_algorithm_metadata(
            protector,
            metadata.iter().map(Vec::as_slice),
        )?;
        let binding = VaultBinding::new(
            secret_id,
            1,
            prepared.classification.clone(),
            SECURITY_METADATA_VERSION,
        )?;
        let encrypted = vault.seal(&binding, prepared.plaintext.as_slice())?;

        transaction.execute(
            "INSERT INTO secret_records (secret_id, classification, owner_principal, current_version, status, created_global_seq, revoked_at) VALUES (?1, ?2, ?3, 1, 'active', ?4, NULL)",
            params![
                &secret_id[..],
                &prepared.classification,
                &prepared.owner_principal,
                to_i64(authority.global_seq)?,
            ],
        )?;
        append_secret_record_snapshot(&transaction, &secret_id)
            .map_err(|error| SecretMutationError::AuthoritySecurity(error.to_string()))?;
        transaction.execute(
            "INSERT INTO secret_versions (secret_id, version, ciphertext, nonce_or_algorithm_metadata, associated_data_hash, created_global_seq, rotated_from, retired_at) VALUES (?1, 1, ?2, ?3, ?4, ?5, NULL, NULL)",
            params![
                &secret_id[..],
                encrypted.ciphertext(),
                encrypted.algorithm_metadata(),
                &encrypted.associated_data_hash()[..],
                to_i64(authority.global_seq)?,
            ],
        )?;
        append_secret_version_snapshot(&transaction, &secret_id, 1)
            .map_err(|error| SecretMutationError::AuthoritySecurity(error.to_string()))?;
        consume_once_approval(&transaction, approval_id, effect_id, authority.global_seq)?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| SecretMutationError::AuthoritySecurity(error.to_string()))?;
        #[cfg(test)]
        test_pause_before_secret_commit("create");
        transaction.commit()?;
        Ok(SecretMutationOutcome {
            secret_id,
            version: 1,
        })
    }

    fn rotate_inner<P: KeyProtector>(
        &mut self,
        prepared: PreparedSecretRotate,
        authority_decision_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
        protector: P,
    ) -> Result<SecretMutationOutcome, SecretMutationError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        let authority = verify_current_authority(
            &transaction,
            authority_decision_id,
            SECRET_ROTATE_ACTION,
            &prepared.resource,
        )?;
        verify_mutation_effect(
            &transaction,
            effect_id,
            SECRET_ROTATE_ACTION,
            &prepared.resource,
            prepared.intent_digest,
        )?;
        verify_once_approval(
            &transaction,
            approval_id,
            authority_decision_id,
            effect_id,
            SECRET_ROTATE_ACTION,
            &prepared.resource,
        )?;
        let current = load_active_secret(&transaction, prepared.secret_id)?;
        if current.current_version != prepared.expected_current_version {
            return Err(SecretMutationError::StaleSecretVersion);
        }
        verify_current_version_active(
            &transaction,
            prepared.secret_id,
            prepared.expected_current_version,
        )?;
        let next_version = prepared
            .expected_current_version
            .checked_add(1)
            .ok_or(SecretMutationError::IntegerOverflow)?;

        let metadata = load_all_algorithm_metadata(&transaction)?;
        let vault = SecretVault::from_persisted_algorithm_metadata(
            protector,
            metadata.iter().map(Vec::as_slice),
        )?;
        let binding = VaultBinding::new(
            prepared.secret_id,
            next_version,
            current.classification.clone(),
            SECURITY_METADATA_VERSION,
        )?;
        let encrypted = vault.seal(&binding, prepared.plaintext.as_slice())?;

        let retired = transaction.execute(
            "UPDATE secret_versions SET retired_at = ?3 WHERE secret_id = ?1 AND version = ?2 AND retired_at IS NULL",
            params![
                &prepared.secret_id[..],
                to_i64(prepared.expected_current_version)?,
                &prepared.retired_at,
            ],
        )?;
        if retired != 1 {
            return Err(SecretMutationError::SecretVersionAlreadyRetired);
        }
        append_secret_version_snapshot(
            &transaction,
            &prepared.secret_id,
            to_i64(prepared.expected_current_version)?,
        )
        .map_err(|error| SecretMutationError::AuthoritySecurity(error.to_string()))?;

        transaction.execute(
            "INSERT INTO secret_versions (secret_id, version, ciphertext, nonce_or_algorithm_metadata, associated_data_hash, created_global_seq, rotated_from, retired_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)",
            params![
                &prepared.secret_id[..],
                to_i64(next_version)?,
                encrypted.ciphertext(),
                encrypted.algorithm_metadata(),
                &encrypted.associated_data_hash()[..],
                to_i64(authority.global_seq)?,
                to_i64(prepared.expected_current_version)?,
            ],
        )?;
        append_secret_version_snapshot(&transaction, &prepared.secret_id, to_i64(next_version)?)
            .map_err(|error| SecretMutationError::AuthoritySecurity(error.to_string()))?;

        let changed = transaction.execute(
            "UPDATE secret_records SET current_version = ?2 WHERE secret_id = ?1 AND current_version = ?3 AND status = 'active' AND revoked_at IS NULL",
            params![
                &prepared.secret_id[..],
                to_i64(next_version)?,
                to_i64(prepared.expected_current_version)?,
            ],
        )?;
        if changed != 1 {
            return Err(SecretMutationError::StaleSecretVersion);
        }
        append_secret_record_snapshot(&transaction, &prepared.secret_id)
            .map_err(|error| SecretMutationError::AuthoritySecurity(error.to_string()))?;
        consume_once_approval(&transaction, approval_id, effect_id, authority.global_seq)?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| SecretMutationError::AuthoritySecurity(error.to_string()))?;
        #[cfg(test)]
        test_pause_before_secret_commit("rotate");
        transaction.commit()?;
        Ok(SecretMutationOutcome {
            secret_id: prepared.secret_id,
            version: next_version,
        })
    }

    #[cfg(test)]
    pub(crate) fn create_with_protector<P: KeyProtector>(
        &mut self,
        prepared: PreparedSecretCreate,
        authority_decision_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
        protector: P,
    ) -> Result<SecretMutationOutcome, SecretMutationError> {
        self.create_inner(
            prepared,
            authority_decision_id,
            approval_id,
            effect_id,
            protector,
        )
    }

    #[cfg(test)]
    fn rotate_with_protector<P: KeyProtector>(
        &mut self,
        prepared: PreparedSecretRotate,
        authority_decision_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
        protector: P,
    ) -> Result<SecretMutationOutcome, SecretMutationError> {
        self.rotate_inner(
            prepared,
            authority_decision_id,
            approval_id,
            effect_id,
            protector,
        )
    }
}

struct AuthorityEvidence {
    global_seq: u64,
}

struct StoredSecret {
    classification: String,
    current_version: u64,
}

fn verify_transaction_integrity(transaction: &Transaction<'_>) -> Result<(), SecretMutationError> {
    crate::integrity::verify(transaction)
        .map_err(|error| SecretMutationError::Integrity(error.to_string()))?;
    crate::authority_security_v2::verify(transaction)
        .map_err(|error| SecretMutationError::AuthoritySecurity(error.to_string()))
}

fn verify_current_authority(
    transaction: &Transaction<'_>,
    decision_id: [u8; 16],
    expected_action: &str,
    expected_resource: &str,
) -> Result<AuthorityEvidence, SecretMutationError> {
    let row = transaction
        .query_row(
            "SELECT action, resource, decision, global_seq FROM authorization_decisions WHERE decision_id = ?1",
            params![&decision_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecretMutationError::MissingAuthorityDecision)?;
    if row.0 != expected_action || row.1 != expected_resource || row.2 != "allow" {
        return Err(SecretMutationError::AuthorityDecisionMismatch);
    }
    let global_seq = seq_from_i64(row.3)?;
    let latest: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(global_seq), 0) FROM (SELECT global_seq FROM session_events UNION ALL SELECT global_seq FROM effect_transitions UNION ALL SELECT global_seq FROM authorization_decisions)",
        [],
        |row| row.get(0),
    )?;
    if global_seq != seq_from_i64(latest)? {
        return Err(SecretMutationError::StaleAuthorityDecision);
    }
    Ok(AuthorityEvidence { global_seq })
}

fn verify_mutation_effect(
    transaction: &Transaction<'_>,
    effect_id: EffectId,
    expected_action: &str,
    expected_resource: &str,
    expected_payload_hash: [u8; 32],
) -> Result<(), SecretMutationError> {
    let row = transaction
        .query_row(
            "SELECT i.action, i.resource, i.risk_class, i.execution_semantics, i.payload_hash, t.to_state FROM effect_intents i JOIN effect_transitions t ON t.effect_id = i.effect_id WHERE i.effect_id = ?1 AND t.global_seq = (SELECT MAX(t2.global_seq) FROM effect_transitions t2 WHERE t2.effect_id = i.effect_id)",
            params![&effect_id.0.to_be_bytes()[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecretMutationError::EffectNotFound)?;
    if row.0 != expected_action
        || row.1 != expected_resource
        || row.2 != SECRET_MUTATION_RISK_CLASS
        || row.3 != "at_most_once"
        || row.4.as_slice() != expected_payload_hash
        || row.5 != "authorized"
    {
        return Err(SecretMutationError::EffectMismatch);
    }
    Ok(())
}

fn verify_once_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    decision_id: [u8; 16],
    effect_id: EffectId,
    expected_action: &str,
    expected_resource: &str,
) -> Result<(), SecretMutationError> {
    let row = transaction
        .query_row(
            "SELECT class, action_scope, resource_scope, effect_id, session_id, risk_class, expires_at, max_uses, revoked_at, parent_decision_id FROM approvals WHERE approval_id = ?1",
            params![&approval_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, Option<Vec<u8>>>(3)?,
                    row.get::<_, Option<Vec<u8>>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Vec<u8>>(9)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecretMutationError::ApprovalNotFound)?;
    if row.0 != "ONCE"
        || row.1.as_slice() != expected_action.as_bytes()
        || row.2.as_slice() != expected_resource.as_bytes()
        || row.3.as_deref() != Some(effect_id.0.to_be_bytes().as_slice())
        || row.4.is_some()
        || row.5 != SECRET_MUTATION_RISK_CLASS
        || row.6.is_some()
        || row.7 != Some(1)
        || row.8.is_some()
        || row.9.as_slice() != decision_id
    {
        return Err(SecretMutationError::ApprovalMismatch);
    }
    let already_used = transaction
        .query_row(
            "SELECT 1 FROM approval_consumptions WHERE approval_id = ?1 LIMIT 1",
            params![&approval_id[..]],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    if already_used {
        return Err(SecretMutationError::ApprovalAlreadyUsed);
    }
    Ok(())
}

fn consume_once_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    effect_id: EffectId,
    global_seq: u64,
) -> Result<(), SecretMutationError> {
    let consumption_id = approval_consumption_id(approval_id, effect_id);
    transaction.execute(
        "INSERT INTO approval_consumptions (consumption_id, approval_id, effect_or_operation_id, reserved_global_seq, consumed_global_seq, state) VALUES (?1, ?2, ?3, ?4, ?5, 'consumed')",
        params![
            &consumption_id[..],
            &approval_id[..],
            &effect_id.0.to_be_bytes()[..],
            to_i64(global_seq)?,
            to_i64(global_seq)?,
        ],
    )?;
    append_approval_consumption_snapshot(transaction, &consumption_id)
        .map_err(|error| SecretMutationError::AuthoritySecurity(error.to_string()))
}

fn load_active_secret(
    transaction: &Transaction<'_>,
    secret_id: [u8; SECRET_ID_BYTES],
) -> Result<StoredSecret, SecretMutationError> {
    let row = transaction
        .query_row(
            "SELECT classification, current_version, status, revoked_at FROM secret_records WHERE secret_id = ?1",
            params![&secret_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecretMutationError::SecretNotFound)?;
    validate_classification(&row.0)
        .map_err(|_| SecretMutationError::InvalidStoredRecord("classification is invalid"))?;
    let current_version = positive_u64(row.1, "current version is invalid")?;
    if row.2 == "revoked" || row.3.is_some() {
        return Err(SecretMutationError::SecretAlreadyRevoked);
    }
    if row.2 != "active" {
        return Err(SecretMutationError::SecretInactive);
    }
    Ok(StoredSecret {
        classification: row.0,
        current_version,
    })
}

fn verify_current_version_active(
    transaction: &Transaction<'_>,
    secret_id: [u8; SECRET_ID_BYTES],
    version: u64,
) -> Result<(), SecretMutationError> {
    let retired_at = transaction
        .query_row(
            "SELECT retired_at FROM secret_versions WHERE secret_id = ?1 AND version = ?2",
            params![&secret_id[..], to_i64(version)?],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()?
        .ok_or(SecretMutationError::SecretVersionNotFound)?;
    if retired_at.is_some() {
        return Err(SecretMutationError::SecretVersionAlreadyRetired);
    }
    Ok(())
}

fn load_all_algorithm_metadata(
    transaction: &Transaction<'_>,
) -> Result<Vec<Vec<u8>>, SecretMutationError> {
    let mut statement = transaction.prepare(
        "SELECT nonce_or_algorithm_metadata FROM secret_versions ORDER BY secret_id, version",
    )?;
    let rows = statement.query_map([], |row| row.get::<_, Vec<u8>>(0))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn secret_commitment(plaintext: &[u8]) -> Result<[u8; 32], SecretMutationError> {
    let mut key = [0_u8; 32];
    getrandom::fill(&mut key)?;
    let mut hasher = blake3::Hasher::new_keyed(&key);
    hasher.update(SECRET_COMMITMENT_DOMAIN);
    hasher.update(plaintext);
    let commitment = *hasher.finalize().as_bytes();
    key.zeroize();
    Ok(commitment)
}

fn derived_secret_id(
    intent_digest: [u8; 32],
    effect_id: EffectId,
    decision_id: [u8; 16],
    approval_id: [u8; 16],
) -> [u8; SECRET_ID_BYTES] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(SECRET_ID_DOMAIN);
    hasher.update(&intent_digest);
    hasher.update(&effect_id.0.to_be_bytes());
    hasher.update(&decision_id);
    hasher.update(&approval_id);
    let digest = hasher.finalize();
    let mut id = [0_u8; SECRET_ID_BYTES];
    id.copy_from_slice(&digest.as_bytes()[..SECRET_ID_BYTES]);
    id
}

fn approval_consumption_id(approval_id: [u8; 16], effect_id: EffectId) -> [u8; 16] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(APPROVAL_CONSUMPTION_DOMAIN);
    hasher.update(&approval_id);
    hasher.update(&effect_id.0.to_be_bytes());
    let digest = hasher.finalize();
    let mut id = [0_u8; 16];
    id.copy_from_slice(&digest.as_bytes()[..16]);
    id
}

fn secret_resource(secret_id: [u8; SECRET_ID_BYTES]) -> String {
    format!("secret:{}", hex_bytes(&secret_id))
}

fn validate_classification(value: &str) -> Result<(), SecretMutationError> {
    if value.is_empty()
        || value.len() > MAX_CLASSIFICATION_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(SecretMutationError::InvalidClassification);
    }
    Ok(())
}

fn validate_principal(value: &str) -> Result<(), SecretMutationError> {
    let known_prefix = ["owner:", "client:", "kernel:", "test:"]
        .iter()
        .any(|prefix| value.starts_with(prefix) && value.len() > prefix.len());
    if !known_prefix
        || value.len() > MAX_PRINCIPAL_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(SecretMutationError::InvalidPrincipal);
    }
    Ok(())
}

fn validate_plaintext(value: &[u8]) -> Result<(), SecretMutationError> {
    if value.is_empty() || value.len() > MAX_SECRET_BYTES {
        return Err(SecretMutationError::InvalidSecretValue);
    }
    Ok(())
}

fn valid_utc_second(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    for index in [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18] {
        if !bytes[index].is_ascii_digit() {
            return false;
        }
    }
    let year = decimal(bytes, 0, 4);
    let month = decimal(bytes, 5, 7);
    let day = decimal(bytes, 8, 10);
    let hour = decimal(bytes, 11, 13);
    let minute = decimal(bytes, 14, 16);
    let second = decimal(bytes, 17, 19);
    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    let max_day = match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    (1..=max_day).contains(&day)
}

fn decimal(bytes: &[u8], start: usize, end: usize) -> u32 {
    bytes[start..end]
        .iter()
        .fold(0_u32, |value, byte| value * 10 + u32::from(*byte - b'0'))
}

fn is_leap_year(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(char::from(HEX[(byte >> 4) as usize]));
        value.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    value
}

fn positive_u64(value: i64, reason: &'static str) -> Result<u64, SecretMutationError> {
    let value = seq_from_i64(value)?;
    if value == 0 {
        return Err(SecretMutationError::InvalidStoredRecord(reason));
    }
    Ok(value)
}

fn seq_from_i64(value: i64) -> Result<u64, SecretMutationError> {
    u64::try_from(value)
        .map_err(|_| SecretMutationError::InvalidStoredRecord("negative sequence/version"))
}

fn to_i64(value: u64) -> Result<i64, SecretMutationError> {
    i64::try_from(value).map_err(|_| SecretMutationError::IntegerOverflow)
}

#[cfg(test)]
fn test_pause_before_secret_commit(operation: &str) {
    const OP_ENV: &str = "GOLAM_T003_057_CRASH_OPERATION";
    const ROOT_ENV: &str = "GOLAM_T003_057_CRASH_ROOT";
    if std::env::var(OP_ENV).ok().as_deref() != Some(operation) {
        return;
    }
    let root = std::path::PathBuf::from(
        std::env::var_os(ROOT_ENV).expect("T003-057 crash root must be provided"),
    );
    std::fs::write(
        root.join(format!("secret-{operation}-before-commit.marker")),
        b"mutation-pending-not-committed",
    )
    .expect("T003-057 crash marker must be writable");
    loop {
        std::thread::park_timeout(std::time::Duration::from_secs(60));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority_security_write::{
        append_approval_snapshot, append_authorization_decision_v2_snapshot,
    };
    use crate::secret_vault::KeyProtectionError;
    use crate::security_audit::{
        AuthorizationAuditInput, EffectIntentAuditInput, EffectTransitionAuditInput,
        append_authorization_decision, append_effect_intent, append_effect_transition,
    };
    use golam_core::paths::RuntimeLayout;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    const CANARY_ONE: &[u8] = b"golam-t003-052-deterministic-canary-secret-one";
    const CANARY_TWO: &[u8] = b"golam-t003-052-deterministic-canary-secret-two";
    static N: AtomicU64 = AtomicU64::new(0);

    struct FakeKeyProtector {
        key: Option<[u8; 32]>,
        unavailable: bool,
    }

    impl FakeKeyProtector {
        fn available(byte: u8) -> Self {
            Self {
                key: Some([byte; 32]),
                unavailable: false,
            }
        }

        fn unavailable() -> Self {
            Self {
                key: None,
                unavailable: true,
            }
        }
    }

    impl KeyProtector for FakeKeyProtector {
        fn load_master_key(&self) -> Result<Zeroizing<Vec<u8>>, KeyProtectionError> {
            if self.unavailable {
                return Err(KeyProtectionError::LockedOrUnavailable);
            }
            self.key
                .map(|key| Zeroizing::new(key.to_vec()))
                .ok_or(KeyProtectionError::Missing)
        }

        fn store_master_key(&self, _key: &[u8]) -> Result<(), KeyProtectionError> {
            Err(KeyProtectionError::Unsupported)
        }
    }

    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-secret-mutation-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    struct WorkIds {
        effect: EffectId,
        decision: [u8; 16],
        approval: [u8; 16],
    }

    fn install_authorized_work(
        connection: &mut Connection,
        base_global_seq: u64,
        discriminator: u8,
        action: &str,
        resource: &str,
        payload_hash: [u8; 32],
    ) -> WorkIds {
        let effect = EffectId(u128::from(discriminator) + 1000);
        let effect_bytes = effect.0.to_be_bytes();
        let transition_id = [discriminator; 16];
        let decision = [discriminator.wrapping_add(40); 16];
        let approval = [discriminator.wrapping_add(80); 16];
        let session_id = [discriminator.wrapping_add(1); 16];
        let proposed_event_id = [discriminator.wrapping_add(2); 16];
        let transition_event_id = [discriminator.wrapping_add(3); 16];
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        transaction
            .execute(
                "INSERT INTO effect_intents (effect_id, session_id, requested_by, action, resource, risk_class, execution_semantics, idempotency_key, preconditions, dependencies, payload_hash, proposed_event_id) VALUES (?1, ?2, 'owner:owner', ?3, ?4, ?5, 'at_most_once', NULL, X'', X'', ?6, ?7)",
                params![
                    &effect_bytes[..],
                    &session_id[..],
                    action,
                    resource,
                    SECRET_MUTATION_RISK_CLASS,
                    &payload_hash[..],
                    &proposed_event_id[..],
                ],
            )
            .unwrap();
        append_effect_intent(
            &transaction,
            EffectIntentAuditInput {
                effect_id: &effect_bytes,
                session_id: &session_id,
                requested_by: "owner:owner",
                action,
                resource,
                risk_class: SECRET_MUTATION_RISK_CLASS,
                execution_semantics: "at_most_once",
                idempotency_key: None,
                preconditions: b"",
                dependencies: b"",
                payload_hash: &payload_hash,
                proposed_event_id: &proposed_event_id,
            },
        )
        .unwrap();
        transaction
            .execute(
                "INSERT INTO effect_transitions (transition_id, effect_id, global_seq, from_state, to_state, attempt_id, reason_code, evidence_ref, event_id) VALUES (?1, ?2, ?3, NULL, 'authorized', NULL, NULL, NULL, ?4)",
                params![
                    &transition_id[..],
                    &effect_bytes[..],
                    to_i64(base_global_seq).unwrap(),
                    &transition_event_id[..],
                ],
            )
            .unwrap();
        append_effect_transition(
            &transaction,
            EffectTransitionAuditInput {
                transition_id: &transition_id,
                effect_id: &effect_bytes,
                global_seq: base_global_seq,
                from_state: None,
                to_state: "authorized",
                attempt_id: None,
                reason_code: None,
                evidence_ref: None,
                event_id: &transition_event_id,
            },
        )
        .unwrap();
        transaction
            .execute(
                "INSERT INTO authorization_decisions (decision_id, principal, action, resource, context_hash, decision, reason_code, global_seq, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, matched_rule_ids, approval_id, authority_evidence_version) VALUES (?1, 'owner:owner', ?2, ?3, ?4, 'allow', 'test_allow', ?5, 'allow', NULL, NULL, NULL, NULL, X'', ?6, 2)",
                params![
                    &decision[..],
                    action,
                    resource,
                    &[0_u8; 32][..],
                    to_i64(base_global_seq + 1).unwrap(),
                    &approval[..],
                ],
            )
            .unwrap();
        append_authorization_decision(
            &transaction,
            AuthorizationAuditInput {
                decision_id: &decision,
                principal: "owner:owner",
                action,
                resource,
                context_hash: &[0_u8; 32],
                decision: "allow",
                reason_code: "test_allow",
                global_seq: base_global_seq + 1,
            },
        )
        .unwrap();
        append_authorization_decision_v2_snapshot(&transaction, &decision).unwrap();
        transaction
            .execute(
                "INSERT INTO approvals (approval_id, class, approver_principal, scope_digest, action_scope, resource_scope, effect_id, session_id, risk_class, taint_digest, parent_decision_id, issued_at, expires_at, max_uses, revoked_at) VALUES (?1, 'ONCE', 'owner:owner', ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8, '2026-08-28T00:00:00Z', NULL, 1, NULL)",
                params![
                    &approval[..],
                    &[1_u8; 32][..],
                    action.as_bytes(),
                    resource.as_bytes(),
                    &effect_bytes[..],
                    SECRET_MUTATION_RISK_CLASS,
                    &[0_u8; 32][..],
                    &decision[..],
                ],
            )
            .unwrap();
        append_approval_snapshot(&transaction, &approval).unwrap();
        crate::integrity::verify(&transaction).unwrap();
        crate::authority_security_v2::verify(&transaction).unwrap();
        transaction.commit().unwrap();
        WorkIds {
            effect,
            decision,
            approval,
        }
    }

    #[test]
    fn create_rotate_and_revoke_are_atomic_protected_transitions() {
        let (runtime, authority) = authority();
        let mut store = SecretMutationStore::open(&authority).unwrap();

        let create =
            prepare_secret_create("api_credential", "owner:owner", CANARY_ONE.to_vec()).unwrap();
        let create_work = install_authorized_work(
            &mut store.connection,
            1,
            10,
            SECRET_CREATE_ACTION,
            create.resource(),
            create.intent_digest(),
        );
        let created = store
            .create_with_protector(
                create,
                create_work.decision,
                create_work.approval,
                create_work.effect,
                FakeKeyProtector::available(9),
            )
            .unwrap();
        assert_eq!(created.version(), 1);
        let raw: Vec<u8> = store
            .connection
            .query_row(
                "SELECT ciphertext FROM secret_versions WHERE secret_id = ?1 AND version = 1",
                params![&created.secret_id()[..]],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            !raw.windows(CANARY_ONE.len())
                .any(|window| window == CANARY_ONE)
        );

        let rotate = prepare_secret_rotate(
            created.secret_id(),
            1,
            CANARY_TWO.to_vec(),
            "2026-08-28T01:00:00Z",
        )
        .unwrap();
        let rotate_work = install_authorized_work(
            &mut store.connection,
            3,
            11,
            SECRET_ROTATE_ACTION,
            rotate.resource(),
            rotate.intent_digest(),
        );
        let rotated = store
            .rotate_with_protector(
                rotate,
                rotate_work.decision,
                rotate_work.approval,
                rotate_work.effect,
                FakeKeyProtector::available(9),
            )
            .unwrap();
        assert_eq!(rotated.secret_id(), created.secret_id());
        assert_eq!(rotated.version(), 2);
        let retired_at: Option<String> = store
            .connection
            .query_row(
                "SELECT retired_at FROM secret_versions WHERE secret_id = ?1 AND version = 1",
                params![&created.secret_id()[..]],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(retired_at.as_deref(), Some("2026-08-28T01:00:00Z"));
        let current: i64 = store
            .connection
            .query_row(
                "SELECT current_version FROM secret_records WHERE secret_id = ?1",
                params![&created.secret_id()[..]],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(current, 2);

        let revoke = prepare_secret_revoke(created.secret_id(), 2, "2026-08-28T02:00:00Z").unwrap();
        let revoke_work = install_authorized_work(
            &mut store.connection,
            5,
            12,
            SECRET_REVOKE_ACTION,
            revoke.resource(),
            revoke.intent_digest(),
        );
        let revoked = store
            .revoke(
                revoke,
                revoke_work.decision,
                revoke_work.approval,
                revoke_work.effect,
            )
            .unwrap();
        assert_eq!(revoked.version(), 2);
        let status: (String, Option<String>) = store
            .connection
            .query_row(
                "SELECT status, revoked_at FROM secret_records WHERE secret_id = ?1",
                params![&created.secret_id()[..]],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(status.0, "revoked");
        assert_eq!(status.1.as_deref(), Some("2026-08-28T02:00:00Z"));
        crate::integrity::verify(&store.connection).unwrap();
        crate::authority_security_v2::verify(&store.connection).unwrap();

        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn vault_failure_rolls_back_secret_rows_and_approval_consumption() {
        let (runtime, authority) = authority();
        let mut store = SecretMutationStore::open(&authority).unwrap();
        let create =
            prepare_secret_create("api_credential", "owner:owner", CANARY_ONE.to_vec()).unwrap();
        let work = install_authorized_work(
            &mut store.connection,
            1,
            20,
            SECRET_CREATE_ACTION,
            create.resource(),
            create.intent_digest(),
        );
        assert!(matches!(
            store.create_with_protector(
                create,
                work.decision,
                work.approval,
                work.effect,
                FakeKeyProtector::unavailable(),
            ),
            Err(SecretMutationError::Vault(VaultError::KeyProtection(
                KeyProtectionError::LockedOrUnavailable
            )))
        ));
        let secret_count: i64 = store
            .connection
            .query_row("SELECT COUNT(*) FROM secret_records", [], |row| row.get(0))
            .unwrap();
        let consumption_count: i64 = store
            .connection
            .query_row("SELECT COUNT(*) FROM approval_consumptions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(secret_count, 0);
        assert_eq!(consumption_count, 0);
        crate::integrity::verify(&store.connection).unwrap();
        crate::authority_security_v2::verify(&store.connection).unwrap();
        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn stale_rotation_fails_before_consuming_approval() {
        let (runtime, authority) = authority();
        let mut store = SecretMutationStore::open(&authority).unwrap();
        let create =
            prepare_secret_create("api_credential", "owner:owner", CANARY_ONE.to_vec()).unwrap();
        let create_work = install_authorized_work(
            &mut store.connection,
            1,
            30,
            SECRET_CREATE_ACTION,
            create.resource(),
            create.intent_digest(),
        );
        let created = store
            .create_with_protector(
                create,
                create_work.decision,
                create_work.approval,
                create_work.effect,
                FakeKeyProtector::available(7),
            )
            .unwrap();

        let rotate = prepare_secret_rotate(
            created.secret_id(),
            2,
            CANARY_TWO.to_vec(),
            "2026-08-28T03:00:00Z",
        )
        .unwrap();
        let work = install_authorized_work(
            &mut store.connection,
            3,
            31,
            SECRET_ROTATE_ACTION,
            rotate.resource(),
            rotate.intent_digest(),
        );
        assert!(matches!(
            store.rotate_with_protector(
                rotate,
                work.decision,
                work.approval,
                work.effect,
                FakeKeyProtector::available(7),
            ),
            Err(SecretMutationError::StaleSecretVersion)
        ));
        let used: i64 = store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM approval_consumptions WHERE approval_id = ?1",
                params![&work.approval[..]],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(used, 0);
        let current: i64 = store
            .connection
            .query_row(
                "SELECT current_version FROM secret_records WHERE secret_id = ?1",
                params![&created.secret_id()[..]],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(current, 1);
        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    fn seed_t003_057_secret() -> (RuntimeLayout, AuthorityLayout, [u8; 16]) {
        let (runtime, authority) = authority();
        let mut store = SecretMutationStore::open(&authority).unwrap();
        let create =
            prepare_secret_create("api_credential", "owner:owner", CANARY_ONE.to_vec()).unwrap();
        let work = install_authorized_work(
            &mut store.connection,
            1,
            70,
            SECRET_CREATE_ACTION,
            create.resource(),
            create.intent_digest(),
        );
        let created = store
            .create_with_protector(
                create,
                work.decision,
                work.approval,
                work.effect,
                FakeKeyProtector::available(5),
            )
            .unwrap();
        let secret_id = created.secret_id();
        drop(store);
        (runtime, authority, secret_id)
    }

    #[test]
    fn disk_full_rotation_rolls_back_retirement_version_and_approval() {
        let (runtime, authority, secret_id) = seed_t003_057_secret();
        let mut store = SecretMutationStore::open(&authority).unwrap();
        let rotate = prepare_secret_rotate(
            secret_id,
            1,
            vec![b'R'; MAX_SECRET_BYTES],
            "2026-08-28T04:00:00Z",
        )
        .unwrap();
        let work = install_authorized_work(
            &mut store.connection,
            3,
            71,
            SECRET_ROTATE_ACTION,
            rotate.resource(),
            rotate.intent_digest(),
        );

        store
            .connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE); VACUUM;")
            .unwrap();
        let page_count: i64 = store
            .connection
            .query_row("PRAGMA page_count", [], |row| row.get(0))
            .unwrap();
        store
            .connection
            .execute_batch(&format!("PRAGMA max_page_count = {page_count};"))
            .unwrap();

        let error = store
            .rotate_with_protector(
                rotate,
                work.decision,
                work.approval,
                work.effect,
                FakeKeyProtector::available(5),
            )
            .expect_err("bounded database must report SQLITE_FULL before rotation commit");
        assert!(matches!(
            error,
            SecretMutationError::Sqlite(rusqlite::Error::SqliteFailure(ref code, _))
                if code.extended_code == rusqlite::ffi::SQLITE_FULL
        ));
        store
            .connection
            .execute_batch("PRAGMA max_page_count = 1073741823;")
            .unwrap();

        let state: (i64, Option<String>, i64) = store
            .connection
            .query_row(
                "SELECT r.current_version, v.retired_at, (SELECT COUNT(*) FROM secret_versions v2 WHERE v2.secret_id = r.secret_id) FROM secret_records r JOIN secret_versions v ON v.secret_id = r.secret_id AND v.version = 1 WHERE r.secret_id = ?1",
                params![&secret_id[..]],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(state, (1, None, 1));
        let used: i64 = store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM approval_consumptions WHERE approval_id = ?1",
                params![&work.approval[..]],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(used, 0);
        crate::integrity::verify(&store.connection).unwrap();
        crate::authority_security_v2::verify(&store.connection).unwrap();
        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn crash_before_secret_commit_child() {
        const OP_ENV: &str = "GOLAM_T003_057_CRASH_OPERATION";
        const ROOT_ENV: &str = "GOLAM_T003_057_CRASH_ROOT";
        let Some(operation) = std::env::var_os(OP_ENV) else {
            return;
        };
        let operation = operation.to_string_lossy();
        let root = std::path::PathBuf::from(
            std::env::var_os(ROOT_ENV).expect("T003-057 child root must be present"),
        );
        let runtime = RuntimeLayout::initialize(&root).unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let mut store = SecretMutationStore::open(&authority).unwrap();
        let secret_id: Vec<u8> = store
            .connection
            .query_row("SELECT secret_id FROM secret_records LIMIT 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        let secret_id: [u8; 16] = secret_id.try_into().unwrap();

        match operation.as_ref() {
            "rotate" => {
                let rotate = prepare_secret_rotate(
                    secret_id,
                    1,
                    CANARY_TWO.to_vec(),
                    "2026-08-28T05:00:00Z",
                )
                .unwrap();
                let work = install_authorized_work(
                    &mut store.connection,
                    3,
                    72,
                    SECRET_ROTATE_ACTION,
                    rotate.resource(),
                    rotate.intent_digest(),
                );
                let _ = store.rotate_with_protector(
                    rotate,
                    work.decision,
                    work.approval,
                    work.effect,
                    FakeKeyProtector::available(5),
                );
            }
            "revoke" => {
                let revoke = prepare_secret_revoke(secret_id, 1, "2026-08-28T06:00:00Z").unwrap();
                let work = install_authorized_work(
                    &mut store.connection,
                    3,
                    73,
                    SECRET_REVOKE_ACTION,
                    revoke.resource(),
                    revoke.intent_digest(),
                );
                let _ = store.revoke(revoke, work.decision, work.approval, work.effect);
            }
            other => panic!("unexpected T003-057 crash operation: {other}"),
        }
        panic!("T003-057 child mutation returned instead of pausing before commit");
    }

    fn kill_precommit_child(runtime: &RuntimeLayout, operation: &str) {
        let marker = runtime
            .root
            .join(format!("secret-{operation}-before-commit.marker"));
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("secret_mutation::tests::crash_before_secret_commit_child")
            .arg("--nocapture")
            .env("GOLAM_T003_057_CRASH_OPERATION", operation)
            .env("GOLAM_T003_057_CRASH_ROOT", &runtime.root)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while !marker.exists() {
            if let Some(status) = child.try_wait().unwrap() {
                panic!("T003-057 {operation} child exited before pre-commit marker: {status}");
            }
            assert!(
                std::time::Instant::now() < deadline,
                "T003-057 {operation} child did not reach pre-commit marker"
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        child.kill().unwrap();
        let _ = child.wait().unwrap();
        fs::remove_file(marker).unwrap();
    }

    #[test]
    fn process_kill_before_rotation_commit_preserves_old_current_authority() {
        let (runtime, authority, secret_id) = seed_t003_057_secret();
        kill_precommit_child(&runtime, "rotate");

        let mut store = SecretMutationStore::open(&authority).unwrap();
        let state: (i64, Option<String>, i64) = store
            .connection
            .query_row(
                "SELECT r.current_version, v.retired_at, (SELECT COUNT(*) FROM secret_versions v2 WHERE v2.secret_id = r.secret_id) FROM secret_records r JOIN secret_versions v ON v.secret_id = r.secret_id AND v.version = 1 WHERE r.secret_id = ?1",
                params![&secret_id[..]],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(state, (1, None, 1));
        let crashed_approval = [72_u8.wrapping_add(80); 16];
        let used: i64 = store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM approval_consumptions WHERE approval_id = ?1",
                params![&crashed_approval[..]],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(used, 0);
        crate::integrity::verify(&store.connection).unwrap();
        crate::authority_security_v2::verify(&store.connection).unwrap();

        let rotate =
            prepare_secret_rotate(secret_id, 1, CANARY_TWO.to_vec(), "2026-08-28T05:01:00Z")
                .unwrap();
        let work = install_authorized_work(
            &mut store.connection,
            5,
            74,
            SECRET_ROTATE_ACTION,
            rotate.resource(),
            rotate.intent_digest(),
        );
        let committed = store
            .rotate_with_protector(
                rotate,
                work.decision,
                work.approval,
                work.effect,
                FakeKeyProtector::available(5),
            )
            .unwrap();
        assert_eq!(committed.version(), 2);
        let current: i64 = store
            .connection
            .query_row(
                "SELECT current_version FROM secret_records WHERE secret_id = ?1",
                params![&secret_id[..]],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(current, 2);
        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn process_kill_before_revocation_commit_preserves_active_then_commit_revokes() {
        let (runtime, authority, secret_id) = seed_t003_057_secret();
        kill_precommit_child(&runtime, "revoke");

        let mut store = SecretMutationStore::open(&authority).unwrap();
        let state: (String, Option<String>, i64) = store
            .connection
            .query_row(
                "SELECT status, revoked_at, (SELECT COUNT(*) FROM secret_versions v WHERE v.secret_id = r.secret_id) FROM secret_records r WHERE secret_id = ?1",
                params![&secret_id[..]],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(state, ("active".to_owned(), None, 1));
        let crashed_approval = [73_u8.wrapping_add(80); 16];
        let used: i64 = store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM approval_consumptions WHERE approval_id = ?1",
                params![&crashed_approval[..]],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(used, 0);
        crate::integrity::verify(&store.connection).unwrap();
        crate::authority_security_v2::verify(&store.connection).unwrap();

        let revoke = prepare_secret_revoke(secret_id, 1, "2026-08-28T06:01:00Z").unwrap();
        let work = install_authorized_work(
            &mut store.connection,
            5,
            75,
            SECRET_REVOKE_ACTION,
            revoke.resource(),
            revoke.intent_digest(),
        );
        store
            .revoke(revoke, work.decision, work.approval, work.effect)
            .unwrap();
        let committed: (String, Option<String>, i64) = store
            .connection
            .query_row(
                "SELECT status, revoked_at, (SELECT COUNT(*) FROM secret_versions v WHERE v.secret_id = r.secret_id) FROM secret_records r WHERE secret_id = ?1",
                params![&secret_id[..]],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(
            committed,
            (
                "revoked".to_owned(),
                Some("2026-08-28T06:01:00Z".to_owned()),
                1
            )
        );
        crate::integrity::verify(&store.connection).unwrap();
        crate::authority_security_v2::verify(&store.connection).unwrap();
        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
