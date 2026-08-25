use std::error::Error;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use ed25519_dalek::SigningKey;
use golam_core::authority::{AuthorityLayout, AuthorityPathError};
use golam_core::ClientId;

use crate::lifecycle::ClientKeyId;

const CREDENTIAL_MAGIC: [u8; 4] = *b"GKEY";
const CREDENTIAL_VERSION: u16 = 1;
const CREDENTIAL_BYTES: usize = 118;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedClientCredential {
    pub client_id: ClientId,
    pub key_id: ClientKeyId,
    pub public_key: [u8; 32],
    pub path: PathBuf,
}

#[derive(Debug)]
pub enum CredentialError {
    Io(io::Error),
    Random(getrandom::Error),
    AuthorityPath(AuthorityPathError),
    InvalidClientId,
    InvalidCredentialLength { actual: usize },
    InvalidCredentialMagic,
    UnsupportedCredentialVersion(u16),
    CredentialClientMismatch,
    CredentialKeyMismatch,
    CredentialPublicKeyMismatch,
}

impl fmt::Display for CredentialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "client credential I/O error: {error}"),
            Self::Random(error) => write!(f, "client credential random-source error: {error}"),
            Self::AuthorityPath(error) => write!(f, "client credential path error: {error}"),
            Self::InvalidClientId => f.write_str("client id must be non-zero"),
            Self::InvalidCredentialLength { actual } => write!(f, "client credential has {actual} bytes; expected {CREDENTIAL_BYTES}"),
            Self::InvalidCredentialMagic => f.write_str("client credential magic is invalid"),
            Self::UnsupportedCredentialVersion(version) => write!(f, "client credential version {version} is unsupported"),
            Self::CredentialClientMismatch => f.write_str("client credential belongs to a different client id"),
            Self::CredentialKeyMismatch => f.write_str("client credential key fingerprint does not match"),
            Self::CredentialPublicKeyMismatch => f.write_str("client credential public key does not match derived key"),
        }
    }
}

impl Error for CredentialError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Random(error) => Some(error),
            Self::AuthorityPath(error) => Some(error),
            _ => None,
        }
    }
}
impl From<io::Error> for CredentialError { fn from(value: io::Error) -> Self { Self::Io(value) } }
impl From<getrandom::Error> for CredentialError { fn from(value: getrandom::Error) -> Self { Self::Random(value) } }
impl From<AuthorityPathError> for CredentialError { fn from(value: AuthorityPathError) -> Self { Self::AuthorityPath(value) } }

pub struct ClientCredentialStore<'a> { authority: &'a AuthorityLayout }
impl<'a> ClientCredentialStore<'a> {
    pub const fn new(authority: &'a AuthorityLayout) -> Self { Self { authority } }

    pub fn generate(&self, client_id: ClientId) -> Result<GeneratedClientCredential, CredentialError> {
        if client_id.0 == 0 { return Err(CredentialError::InvalidClientId); }
        let mut secret = [0_u8; 32];
        getrandom::fill(&mut secret)?;
        let signing_key = SigningKey::from_bytes(&secret);
        let public_key = signing_key.verifying_key().to_bytes();
        let key_id = key_id_for_public_key(public_key);
        let path = self.authority.credential_path(client_id.0, &key_id.0)?;
        let mut bytes = encode_credential(client_id, key_id, secret, public_key);
        write_private_file(&path, &bytes)?;
        self.authority.protect_credential_file(&path)?;
        secret.fill(0);
        bytes.fill(0);
        Ok(GeneratedClientCredential { client_id, key_id, public_key, path })
    }

    pub fn load(&self, client_id: ClientId, key_id: ClientKeyId) -> Result<SigningKey, CredentialError> {
        let path = self.authority.credential_path(client_id.0, &key_id.0)?;
        self.authority.verify_credential_file(&path)?;
        let mut file = OpenOptions::new().read(true).open(&path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let result = decode_credential(&bytes, client_id, key_id);
        bytes.fill(0);
        result
    }

    pub fn remove(&self, client_id: ClientId, key_id: ClientKeyId) -> Result<(), CredentialError> {
        let path = self.authority.credential_path(client_id.0, &key_id.0)?;
        self.authority.verify_credential_file(&path)?;
        fs::remove_file(path)?;
        Ok(())
    }
}

pub fn key_id_for_public_key(public_key: [u8; 32]) -> ClientKeyId {
    ClientKeyId(*blake3::hash(&public_key).as_bytes())
}

fn encode_credential(client_id: ClientId, key_id: ClientKeyId, secret: [u8; 32], public_key: [u8; 32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(CREDENTIAL_BYTES);
    bytes.extend_from_slice(&CREDENTIAL_MAGIC);
    bytes.extend_from_slice(&CREDENTIAL_VERSION.to_be_bytes());
    bytes.extend_from_slice(&client_id.0.to_be_bytes());
    bytes.extend_from_slice(&key_id.0);
    bytes.extend_from_slice(&secret);
    bytes.extend_from_slice(&public_key);
    bytes
}

fn decode_credential(bytes: &[u8], expected_client: ClientId, expected_key_id: ClientKeyId) -> Result<SigningKey, CredentialError> {
    if bytes.len() != CREDENTIAL_BYTES { return Err(CredentialError::InvalidCredentialLength { actual: bytes.len() }); }
    if bytes[..4] != CREDENTIAL_MAGIC { return Err(CredentialError::InvalidCredentialMagic); }
    let version = u16::from_be_bytes([bytes[4], bytes[5]]);
    if version != CREDENTIAL_VERSION { return Err(CredentialError::UnsupportedCredentialVersion(version)); }
    let stored_client = ClientId(u128::from_be_bytes(bytes[6..22].try_into().expect("fixed credential client-id range")));
    if stored_client != expected_client { return Err(CredentialError::CredentialClientMismatch); }
    let stored_key_id = ClientKeyId(bytes[22..54].try_into().expect("fixed credential key-id range"));
    if stored_key_id != expected_key_id { return Err(CredentialError::CredentialKeyMismatch); }
    let mut secret: [u8; 32] = bytes[54..86].try_into().expect("fixed credential secret range");
    let stored_public: [u8; 32] = bytes[86..118].try_into().expect("fixed credential public-key range");
    let signing_key = SigningKey::from_bytes(&secret);
    secret.fill(0);
    let derived_public = signing_key.verifying_key().to_bytes();
    if derived_public != stored_public { return Err(CredentialError::CredentialPublicKeyMismatch); }
    if key_id_for_public_key(derived_public) != expected_key_id { return Err(CredentialError::CredentialKeyMismatch); }
    Ok(signing_key)
}

fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), CredentialError> {
    #[cfg(unix)]
    let mut file = {
        use std::os::unix::fs::OpenOptionsExt;
        OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)?
    };
    #[cfg(not(unix))]
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, Verifier};
    use golam_core::paths::RuntimeLayout;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static N: AtomicU64 = AtomicU64::new(0);
    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n=N.fetch_add(1,Ordering::Relaxed); let t=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let runtime=RuntimeLayout::initialize(std::env::temp_dir().join(format!("golam-credential-{}-{t}-{n}",std::process::id()))).unwrap();
        let authority=AuthorityLayout::initialize(&runtime).unwrap(); (runtime,authority)
    }
    #[test]
    fn generated_key_round_trips() {
        let (runtime,authority)=authority(); let store=ClientCredentialStore::new(&authority);
        let generated=store.generate(ClientId(11)).unwrap(); let signing=store.load(generated.client_id,generated.key_id).unwrap();
        assert_eq!(signing.verifying_key().to_bytes(),generated.public_key);
        let signature=signing.sign(b"golam-credential-proof"); signing.verifying_key().verify(b"golam-credential-proof",&signature).unwrap();
        store.remove(generated.client_id,generated.key_id).unwrap(); assert!(!generated.path.exists()); fs::remove_dir_all(runtime.root).unwrap();
    }
    #[test]
    fn corruption_fails_closed() {
        let (runtime,authority)=authority(); let store=ClientCredentialStore::new(&authority); let generated=store.generate(ClientId(12)).unwrap();
        let mut bytes=fs::read(&generated.path).unwrap(); bytes[117]^=0xff; fs::write(&generated.path,&bytes).unwrap(); authority.protect_credential_file(&generated.path).unwrap();
        assert!(matches!(store.load(generated.client_id,generated.key_id),Err(CredentialError::CredentialPublicKeyMismatch)));
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
