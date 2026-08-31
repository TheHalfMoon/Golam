use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::sync::{Mutex, MutexGuard};

use aes_gcm::Aes256Gcm;
use aes_gcm::aead::{Aead, KeyInit, Nonce, Payload};
use keyring_core::api::CredentialStoreApi;
use keyring_core::{Entry, Error as KeyringError};
use zeroize::{Zeroize, Zeroizing};

const MASTER_KEY_BYTES: usize = 32;
const NONCE_BYTES: usize = 12;
const GCM_TAG_BYTES: usize = 16;
const SECRET_ID_BYTES: usize = 16;
const MAX_CLASSIFICATION_BYTES: usize = 128;
const VAULT_FORMAT_VERSION: u16 = 1;
const AAD_DOMAIN: &[u8] = b"golam:secret-vault:aad:v1";
const METADATA_MAGIC: [u8; 4] = *b"GSV1";
const METADATA_BYTES: usize = METADATA_MAGIC.len() + NONCE_BYTES;
const KEYRING_SERVICE: &str = "golam-authority-vault";
const KEYRING_USER: &str = "master-key-v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VaultBinding {
    secret_id: [u8; SECRET_ID_BYTES],
    version: u64,
    classification: String,
    security_metadata_version: u64,
}

impl VaultBinding {
    pub(crate) fn new(
        secret_id: [u8; SECRET_ID_BYTES],
        version: u64,
        classification: impl Into<String>,
        security_metadata_version: u64,
    ) -> Result<Self, VaultError> {
        let classification = classification.into();
        if version == 0 {
            return Err(VaultError::InvalidBinding(
                "secret version must be non-zero",
            ));
        }
        if security_metadata_version == 0 {
            return Err(VaultError::InvalidBinding(
                "security metadata version must be non-zero",
            ));
        }
        if classification.is_empty()
            || classification.len() > MAX_CLASSIFICATION_BYTES
            || classification.trim() != classification
            || classification.chars().any(char::is_control)
        {
            return Err(VaultError::InvalidBinding(
                "secret classification is invalid",
            ));
        }
        Ok(Self {
            secret_id,
            version,
            classification,
            security_metadata_version,
        })
    }

    fn associated_data(&self) -> Vec<u8> {
        let classification = self.classification.as_bytes();
        let mut bytes = Vec::with_capacity(
            AAD_DOMAIN.len() + 2 + SECRET_ID_BYTES + 8 + 2 + classification.len() + 8,
        );
        bytes.extend_from_slice(AAD_DOMAIN);
        bytes.extend_from_slice(&VAULT_FORMAT_VERSION.to_be_bytes());
        bytes.extend_from_slice(&self.secret_id);
        bytes.extend_from_slice(&self.version.to_be_bytes());
        bytes.extend_from_slice(&(classification.len() as u16).to_be_bytes());
        bytes.extend_from_slice(classification);
        bytes.extend_from_slice(&self.security_metadata_version.to_be_bytes());
        bytes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EncryptedSecretValue {
    ciphertext: Vec<u8>,
    algorithm_metadata: Vec<u8>,
    associated_data_hash: [u8; 32],
}

impl EncryptedSecretValue {
    pub(crate) fn ciphertext(&self) -> &[u8] {
        &self.ciphertext
    }

    pub(crate) fn algorithm_metadata(&self) -> &[u8] {
        &self.algorithm_metadata
    }

    pub(crate) const fn associated_data_hash(&self) -> [u8; 32] {
        self.associated_data_hash
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum KeyProtectionError {
    Missing,
    LockedOrUnavailable,
    Corrupt,
    Ambiguous,
    Unsupported,
    BackendFailure,
    LockPoisoned,
    AlreadyProvisioned,
}

impl fmt::Display for KeyProtectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => f.write_str("vault master key is missing"),
            Self::LockedOrUnavailable => {
                f.write_str("vault key protection is locked or unavailable")
            }
            Self::Corrupt => f.write_str("vault key protection data is corrupt"),
            Self::Ambiguous => f.write_str("vault key protection lookup is ambiguous"),
            Self::Unsupported => f.write_str("vault key protection is unsupported"),
            Self::BackendFailure => f.write_str("vault key protection backend failed"),
            Self::LockPoisoned => f.write_str("vault key protection lock is poisoned"),
            Self::AlreadyProvisioned => f.write_str("vault master key is already provisioned"),
        }
    }
}

impl Error for KeyProtectionError {}

#[derive(Debug)]
pub(crate) enum VaultError {
    KeyProtection(KeyProtectionError),
    Random(getrandom::Error),
    InvalidBinding(&'static str),
    InvalidMasterKey,
    InvalidAlgorithmMetadata,
    AssociatedDataMismatch,
    NonceReuse,
    EncryptionFailed,
    AuthenticationFailed,
    LockPoisoned,
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyProtection(error) => write!(f, "secret vault key protection failed: {error}"),
            Self::Random(error) => write!(f, "secret vault random-source error: {error}"),
            Self::InvalidBinding(reason) => write!(f, "secret vault binding is invalid: {reason}"),
            Self::InvalidMasterKey => f.write_str("secret vault master key is invalid"),
            Self::InvalidAlgorithmMetadata => {
                f.write_str("secret vault algorithm metadata is invalid")
            }
            Self::AssociatedDataMismatch => {
                f.write_str("secret vault associated-data binding does not match")
            }
            Self::NonceReuse => f.write_str("secret vault nonce reuse was detected"),
            Self::EncryptionFailed => f.write_str("secret vault encryption failed"),
            Self::AuthenticationFailed => {
                f.write_str("secret vault authentication/decryption failed")
            }
            Self::LockPoisoned => f.write_str("secret vault nonce registry lock is poisoned"),
        }
    }
}

impl Error for VaultError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::KeyProtection(error) => Some(error),
            Self::Random(error) => Some(error),
            _ => None,
        }
    }
}

impl From<KeyProtectionError> for VaultError {
    fn from(value: KeyProtectionError) -> Self {
        Self::KeyProtection(value)
    }
}

impl From<getrandom::Error> for VaultError {
    fn from(value: getrandom::Error) -> Self {
        Self::Random(value)
    }
}

pub(crate) trait KeyProtector: Send + Sync {
    fn load_master_key(&self) -> Result<Zeroizing<Vec<u8>>, KeyProtectionError>;
    fn store_master_key(&self, key: &[u8]) -> Result<(), KeyProtectionError>;
}

impl<T: KeyProtector + ?Sized> KeyProtector for &T {
    fn load_master_key(&self) -> Result<Zeroizing<Vec<u8>>, KeyProtectionError> {
        (**self).load_master_key()
    }

    fn store_master_key(&self, key: &[u8]) -> Result<(), KeyProtectionError> {
        (**self).store_master_key(key)
    }
}

pub(crate) struct OsKeyProtector {
    operation_lock: Mutex<()>,
}

impl OsKeyProtector {
    pub(crate) const fn new() -> Self {
        Self {
            operation_lock: Mutex::new(()),
        }
    }

    fn lock(&self) -> Result<MutexGuard<'_, ()>, KeyProtectionError> {
        self.operation_lock
            .lock()
            .map_err(|_| KeyProtectionError::LockPoisoned)
    }

    fn entry(&self) -> Result<Entry, KeyProtectionError> {
        platform_entry()
    }
}

impl KeyProtector for OsKeyProtector {
    fn load_master_key(&self) -> Result<Zeroizing<Vec<u8>>, KeyProtectionError> {
        let _guard = self.lock()?;
        let entry = self.entry()?;
        let mut secret = Zeroizing::new(entry.get_secret().map_err(map_keyring_error)?);
        if secret.len() != MASTER_KEY_BYTES {
            secret.zeroize();
            return Err(KeyProtectionError::Corrupt);
        }
        Ok(secret)
    }

    fn store_master_key(&self, key: &[u8]) -> Result<(), KeyProtectionError> {
        if key.len() != MASTER_KEY_BYTES {
            return Err(KeyProtectionError::Corrupt);
        }
        let _guard = self.lock()?;
        let entry = self.entry()?;
        entry.set_secret(key).map_err(map_keyring_error)?;
        let mut round_trip = Zeroizing::new(entry.get_secret().map_err(map_keyring_error)?);
        let matches = round_trip.as_slice() == key;
        round_trip.zeroize();
        if !matches {
            return Err(KeyProtectionError::Corrupt);
        }
        Ok(())
    }
}

pub(crate) fn provision_master_key(
    protector: &impl KeyProtector,
) -> Result<(), KeyProtectionError> {
    match protector.load_master_key() {
        Ok(mut existing) => {
            existing.zeroize();
            return Err(KeyProtectionError::AlreadyProvisioned);
        }
        Err(KeyProtectionError::Missing) => {}
        Err(error) => return Err(error),
    }

    let mut key = Zeroizing::new([0_u8; MASTER_KEY_BYTES]);
    getrandom::fill(&mut *key).map_err(|_| KeyProtectionError::BackendFailure)?;
    protector.store_master_key(key.as_slice())?;
    let mut verified = protector.load_master_key()?;
    if verified.as_slice() != key.as_slice() {
        verified.zeroize();
        return Err(KeyProtectionError::Corrupt);
    }
    verified.zeroize();
    Ok(())
}

pub(crate) struct SecretVault<P: KeyProtector> {
    protector: P,
    used_nonces: Mutex<HashSet<[u8; NONCE_BYTES]>>,
}

impl<P: KeyProtector> SecretVault<P> {
    pub(crate) fn from_persisted_algorithm_metadata<'a>(
        protector: P,
        metadata: impl IntoIterator<Item = &'a [u8]>,
    ) -> Result<Self, VaultError> {
        let mut used_nonces = HashSet::new();
        for value in metadata {
            let nonce = decode_algorithm_metadata(value)?;
            if !used_nonces.insert(nonce) {
                return Err(VaultError::NonceReuse);
            }
        }
        Ok(Self {
            protector,
            used_nonces: Mutex::new(used_nonces),
        })
    }

    pub(crate) fn seal(
        &self,
        binding: &VaultBinding,
        plaintext: &[u8],
    ) -> Result<EncryptedSecretValue, VaultError> {
        let mut nonce = [0_u8; NONCE_BYTES];
        getrandom::fill(&mut nonce)?;
        self.seal_with_nonce_inner(binding, plaintext, nonce)
    }

    fn seal_with_nonce_inner(
        &self,
        binding: &VaultBinding,
        plaintext: &[u8],
        nonce_bytes: [u8; NONCE_BYTES],
    ) -> Result<EncryptedSecretValue, VaultError> {
        self.reserve_nonce(nonce_bytes)?;

        let mut key = self.protector.load_master_key()?;
        if key.len() != MASTER_KEY_BYTES {
            key.zeroize();
            return Err(VaultError::InvalidMasterKey);
        }
        let cipher =
            Aes256Gcm::new_from_slice(key.as_slice()).map_err(|_| VaultError::InvalidMasterKey)?;
        key.zeroize();

        let associated_data = binding.associated_data();
        let associated_data_hash = *blake3::hash(&associated_data).as_bytes();
        let nonce: Nonce<Aes256Gcm> = nonce_bytes.into();
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: plaintext,
                    aad: &associated_data,
                },
            )
            .map_err(|_| VaultError::EncryptionFailed)?;

        let mut algorithm_metadata = Vec::with_capacity(METADATA_BYTES);
        algorithm_metadata.extend_from_slice(&METADATA_MAGIC);
        algorithm_metadata.extend_from_slice(&nonce_bytes);
        Ok(EncryptedSecretValue {
            ciphertext,
            algorithm_metadata,
            associated_data_hash,
        })
    }

    fn reserve_nonce(&self, nonce: [u8; NONCE_BYTES]) -> Result<(), VaultError> {
        let mut used_nonces = self
            .used_nonces
            .lock()
            .map_err(|_| VaultError::LockPoisoned)?;
        if !used_nonces.insert(nonce) {
            return Err(VaultError::NonceReuse);
        }
        Ok(())
    }

    pub(crate) fn with_persisted_plaintext<R>(
        &self,
        binding: &VaultBinding,
        ciphertext: &[u8],
        algorithm_metadata: &[u8],
        associated_data_hash: [u8; 32],
        callback: impl FnOnce(&[u8]) -> R,
    ) -> Result<R, VaultError> {
        let encrypted = EncryptedSecretValue {
            ciphertext: ciphertext.to_vec(),
            algorithm_metadata: algorithm_metadata.to_vec(),
            associated_data_hash,
        };
        let plaintext = self.open(binding, &encrypted)?;
        Ok(callback(plaintext.as_slice()))
    }

    fn open(
        &self,
        binding: &VaultBinding,
        encrypted: &EncryptedSecretValue,
    ) -> Result<Zeroizing<Vec<u8>>, VaultError> {
        if encrypted.ciphertext.len() < GCM_TAG_BYTES {
            return Err(VaultError::AuthenticationFailed);
        }
        let nonce_bytes = decode_algorithm_metadata(&encrypted.algorithm_metadata)?;
        let associated_data = binding.associated_data();
        if *blake3::hash(&associated_data).as_bytes() != encrypted.associated_data_hash {
            return Err(VaultError::AssociatedDataMismatch);
        }

        let mut key = self.protector.load_master_key()?;
        if key.len() != MASTER_KEY_BYTES {
            key.zeroize();
            return Err(VaultError::InvalidMasterKey);
        }
        let cipher =
            Aes256Gcm::new_from_slice(key.as_slice()).map_err(|_| VaultError::InvalidMasterKey)?;
        key.zeroize();
        let nonce: Nonce<Aes256Gcm> = nonce_bytes.into();
        let plaintext = cipher
            .decrypt(
                &nonce,
                Payload {
                    msg: &encrypted.ciphertext,
                    aad: &associated_data,
                },
            )
            .map_err(|_| VaultError::AuthenticationFailed)?;
        Ok(Zeroizing::new(plaintext))
    }

    #[cfg(test)]
    fn seal_with_nonce_for_test(
        &self,
        binding: &VaultBinding,
        plaintext: &[u8],
        nonce: [u8; NONCE_BYTES],
    ) -> Result<EncryptedSecretValue, VaultError> {
        self.seal_with_nonce_inner(binding, plaintext, nonce)
    }
}

fn decode_algorithm_metadata(value: &[u8]) -> Result<[u8; NONCE_BYTES], VaultError> {
    if value.len() != METADATA_BYTES || value[..METADATA_MAGIC.len()] != METADATA_MAGIC {
        return Err(VaultError::InvalidAlgorithmMetadata);
    }
    value[METADATA_MAGIC.len()..]
        .try_into()
        .map_err(|_| VaultError::InvalidAlgorithmMetadata)
}

#[cfg(target_os = "macos")]
fn platform_entry() -> Result<Entry, KeyProtectionError> {
    let store = apple_native_keyring_store::keychain::Store::new().map_err(map_keyring_error)?;
    store
        .build(KEYRING_SERVICE, KEYRING_USER, None)
        .map_err(map_keyring_error)
}

#[cfg(target_os = "windows")]
fn platform_entry() -> Result<Entry, KeyProtectionError> {
    let store = windows_native_keyring_store::Store::new().map_err(map_keyring_error)?;
    store
        .build(KEYRING_SERVICE, KEYRING_USER, None)
        .map_err(map_keyring_error)
}

#[cfg(target_os = "linux")]
fn platform_entry() -> Result<Entry, KeyProtectionError> {
    let store = zbus_secret_service_keyring_store::Store::new().map_err(map_keyring_error)?;
    store
        .build(KEYRING_SERVICE, KEYRING_USER, None)
        .map_err(map_keyring_error)
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn platform_entry() -> Result<Entry, KeyProtectionError> {
    Err(KeyProtectionError::Unsupported)
}

fn map_keyring_error(error: KeyringError) -> KeyProtectionError {
    match error {
        KeyringError::NoEntry => KeyProtectionError::Missing,
        KeyringError::NoStorageAccess(_) | KeyringError::PlatformFailure(_) => {
            KeyProtectionError::LockedOrUnavailable
        }
        KeyringError::BadEncoding(mut bytes) => {
            bytes.zeroize();
            KeyProtectionError::Corrupt
        }
        KeyringError::BadDataFormat(mut bytes, _) => {
            bytes.zeroize();
            KeyProtectionError::Corrupt
        }
        KeyringError::BadStoreFormat(_) => KeyProtectionError::Corrupt,
        KeyringError::Ambiguous(_) => KeyProtectionError::Ambiguous,
        KeyringError::NotSupportedByStore(_) => KeyProtectionError::Unsupported,
        KeyringError::TooLong(_, _)
        | KeyringError::Invalid(_, _)
        | KeyringError::NoDefaultStore => KeyProtectionError::BackendFailure,
        _ => KeyProtectionError::BackendFailure,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CANARY: &[u8] = b"golam-t003-051-deterministic-canary-secret-material";

    struct FakeKeyProtector {
        state: Mutex<FakeKeyState>,
    }

    enum FakeKeyState {
        Missing,
        Available([u8; MASTER_KEY_BYTES]),
        Corrupt(Vec<u8>),
        Unavailable,
    }

    impl FakeKeyProtector {
        fn missing() -> Self {
            Self {
                state: Mutex::new(FakeKeyState::Missing),
            }
        }

        fn available(byte: u8) -> Self {
            Self {
                state: Mutex::new(FakeKeyState::Available([byte; MASTER_KEY_BYTES])),
            }
        }

        fn corrupt() -> Self {
            Self {
                state: Mutex::new(FakeKeyState::Corrupt(vec![7_u8; 7])),
            }
        }

        fn unavailable() -> Self {
            Self {
                state: Mutex::new(FakeKeyState::Unavailable),
            }
        }
    }

    impl KeyProtector for FakeKeyProtector {
        fn load_master_key(&self) -> Result<Zeroizing<Vec<u8>>, KeyProtectionError> {
            let state = self
                .state
                .lock()
                .map_err(|_| KeyProtectionError::LockPoisoned)?;
            match &*state {
                FakeKeyState::Missing => Err(KeyProtectionError::Missing),
                FakeKeyState::Available(key) => Ok(Zeroizing::new(key.to_vec())),
                FakeKeyState::Corrupt(bytes) => Ok(Zeroizing::new(bytes.clone())),
                FakeKeyState::Unavailable => Err(KeyProtectionError::LockedOrUnavailable),
            }
        }

        fn store_master_key(&self, key: &[u8]) -> Result<(), KeyProtectionError> {
            if key.len() != MASTER_KEY_BYTES {
                return Err(KeyProtectionError::Corrupt);
            }
            let mut state = self
                .state
                .lock()
                .map_err(|_| KeyProtectionError::LockPoisoned)?;
            let mut copy = [0_u8; MASTER_KEY_BYTES];
            copy.copy_from_slice(key);
            *state = FakeKeyState::Available(copy);
            Ok(())
        }
    }

    fn binding() -> VaultBinding {
        VaultBinding::new([1_u8; SECRET_ID_BYTES], 2, "api_credential", 3).unwrap()
    }

    #[test]
    fn encrypted_canary_round_trips_without_plaintext_storage() {
        let vault = SecretVault::from_persisted_algorithm_metadata(
            FakeKeyProtector::available(9),
            std::iter::empty(),
        )
        .unwrap();
        let encrypted = vault
            .seal_with_nonce_for_test(&binding(), CANARY, [4_u8; NONCE_BYTES])
            .unwrap();

        assert_ne!(encrypted.ciphertext(), CANARY);
        assert!(
            !encrypted
                .ciphertext()
                .windows(CANARY.len())
                .any(|window| window == CANARY)
        );
        assert!(
            !encrypted
                .algorithm_metadata()
                .windows(CANARY.len())
                .any(|window| window == CANARY)
        );
        let mut plaintext = vault.open(&binding(), &encrypted).unwrap();
        assert_eq!(plaintext.as_slice(), CANARY);
        plaintext.zeroize();
    }

    #[test]
    fn every_aad_binding_dimension_is_authenticated() {
        let vault = SecretVault::from_persisted_algorithm_metadata(
            FakeKeyProtector::available(10),
            std::iter::empty(),
        )
        .unwrap();
        let encrypted = vault
            .seal_with_nonce_for_test(&binding(), CANARY, [5_u8; NONCE_BYTES])
            .unwrap();

        for changed in [
            VaultBinding::new([2_u8; SECRET_ID_BYTES], 2, "api_credential", 3).unwrap(),
            VaultBinding::new([1_u8; SECRET_ID_BYTES], 3, "api_credential", 3).unwrap(),
            VaultBinding::new([1_u8; SECRET_ID_BYTES], 2, "oauth_token", 3).unwrap(),
            VaultBinding::new([1_u8; SECRET_ID_BYTES], 2, "api_credential", 4).unwrap(),
        ] {
            assert!(matches!(
                vault.open(&changed, &encrypted),
                Err(VaultError::AssociatedDataMismatch)
            ));
        }
    }

    #[test]
    fn nonce_reuse_and_persisted_duplicate_nonce_fail_closed() {
        let vault = SecretVault::from_persisted_algorithm_metadata(
            FakeKeyProtector::available(11),
            std::iter::empty(),
        )
        .unwrap();
        let nonce = [6_u8; NONCE_BYTES];
        let encrypted = vault
            .seal_with_nonce_for_test(&binding(), CANARY, nonce)
            .unwrap();
        assert!(matches!(
            vault.seal_with_nonce_for_test(&binding(), CANARY, nonce),
            Err(VaultError::NonceReuse)
        ));
        assert!(matches!(
            SecretVault::from_persisted_algorithm_metadata(
                FakeKeyProtector::available(11),
                [
                    encrypted.algorithm_metadata(),
                    encrypted.algorithm_metadata(),
                ],
            ),
            Err(VaultError::NonceReuse)
        ));
    }

    #[test]
    fn corrupt_or_unavailable_key_protection_fails_closed() {
        let corrupt = SecretVault::from_persisted_algorithm_metadata(
            FakeKeyProtector::corrupt(),
            std::iter::empty(),
        )
        .unwrap();
        assert!(matches!(
            corrupt.seal_with_nonce_for_test(&binding(), CANARY, [7_u8; NONCE_BYTES]),
            Err(VaultError::InvalidMasterKey)
        ));

        let unavailable = SecretVault::from_persisted_algorithm_metadata(
            FakeKeyProtector::unavailable(),
            std::iter::empty(),
        )
        .unwrap();
        assert!(matches!(
            unavailable.seal_with_nonce_for_test(&binding(), CANARY, [8_u8; NONCE_BYTES]),
            Err(VaultError::KeyProtection(
                KeyProtectionError::LockedOrUnavailable
            ))
        ));
    }

    #[test]
    fn tampering_and_malformed_metadata_fail_authentication() {
        let vault = SecretVault::from_persisted_algorithm_metadata(
            FakeKeyProtector::available(12),
            std::iter::empty(),
        )
        .unwrap();
        let mut encrypted = vault
            .seal_with_nonce_for_test(&binding(), CANARY, [9_u8; NONCE_BYTES])
            .unwrap();
        encrypted.ciphertext[0] ^= 0x80;
        assert!(matches!(
            vault.open(&binding(), &encrypted),
            Err(VaultError::AuthenticationFailed)
        ));

        encrypted.algorithm_metadata.truncate(METADATA_BYTES - 1);
        assert!(matches!(
            vault.open(&binding(), &encrypted),
            Err(VaultError::InvalidAlgorithmMetadata)
        ));
    }

    #[test]
    fn provisioning_refuses_overwrite_and_propagates_unavailability() {
        let missing = FakeKeyProtector::missing();
        provision_master_key(&missing).unwrap();
        assert!(matches!(
            provision_master_key(&missing),
            Err(KeyProtectionError::AlreadyProvisioned)
        ));

        assert!(matches!(
            provision_master_key(&FakeKeyProtector::unavailable()),
            Err(KeyProtectionError::LockedOrUnavailable)
        ));
    }
}
