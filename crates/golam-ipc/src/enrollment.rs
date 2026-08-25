use std::error::Error;
use std::fmt;

use ed25519_dalek::VerifyingKey;
use golam_core::authority::AuthorityLayout;
use golam_core::ClientId;
use golam_ledger::clients::{AssuranceClass, ClientKind, ClientRecord, ClientRegistry, ClientRegistryError, EnrollClient};

use crate::credentials::{key_id_for_public_key, GeneratedClientCredential};
use crate::lifecycle::{Authenticate, ClientKeyId, EnrolledClientKey, LifecycleError, Ready, ServerLifecycle};

#[derive(Debug)]
pub enum EnrollmentError {
    Registry(ClientRegistryError),
    InvalidPublicKey,
    KeyFingerprintMismatch,
    Lifecycle(LifecycleError),
    AuthenticatedClientMismatch,
}

impl fmt::Display for EnrollmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registry(error) => write!(f, "client enrollment registry error: {error}"),
            Self::InvalidPublicKey => f.write_str("client public key is not a valid Ed25519 key"),
            Self::KeyFingerprintMismatch => f.write_str("client key id does not match the enrolled public key fingerprint"),
            Self::Lifecycle(error) => write!(f, "client lifecycle authentication error: {error}"),
            Self::AuthenticatedClientMismatch => f.write_str("authenticated lifecycle client does not match requested enrollment"),
        }
    }
}
impl Error for EnrollmentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self { Self::Registry(e) => Some(e), Self::Lifecycle(e) => Some(e), _ => None }
    }
}
impl From<ClientRegistryError> for EnrollmentError { fn from(value: ClientRegistryError) -> Self { Self::Registry(value) } }
impl From<LifecycleError> for EnrollmentError { fn from(value: LifecycleError) -> Self { Self::Lifecycle(value) } }

pub struct LocalClientEnrollment { registry: ClientRegistry }
impl LocalClientEnrollment {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, EnrollmentError> {
        Ok(Self { registry: ClientRegistry::open(layout)? })
    }

    pub fn enroll_generated(&mut self, generated: &GeneratedClientCredential, kind: ClientKind, owner_principal: &str, enrolled_at: &str) -> Result<ClientRecord, EnrollmentError> {
        self.enroll_public(generated.client_id, generated.key_id, generated.public_key, kind, owner_principal, enrolled_at, AssuranceClass::FilesystemUserPrivateV1)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn enroll_public(&mut self, client_id: ClientId, key_id: ClientKeyId, public_key: [u8;32], kind: ClientKind, owner_principal: &str, enrolled_at: &str, assurance_class: AssuranceClass) -> Result<ClientRecord, EnrollmentError> {
        VerifyingKey::from_bytes(&public_key).map_err(|_| EnrollmentError::InvalidPublicKey)?;
        if key_id_for_public_key(public_key) != key_id { return Err(EnrollmentError::KeyFingerprintMismatch); }
        Ok(self.registry.enroll(EnrollClient { client_id, key_id: key_id.0, public_key, kind, owner_principal, enrolled_at, assurance_class })?)
    }

    pub fn revoke(&mut self, client_id: ClientId, revoked_at: &str) -> Result<ClientRecord, EnrollmentError> {
        Ok(self.registry.revoke(client_id, revoked_at)?)
    }

    pub fn resolve_active(&self, client_id: ClientId, key_id: ClientKeyId) -> Result<ClientRecord, EnrollmentError> {
        Ok(self.registry.resolve_active(client_id, key_id.0)?)
    }

    pub fn authenticate_registered(&mut self, lifecycle: &mut ServerLifecycle, client_id: ClientId, authenticate: Authenticate, authenticated_at: &str) -> Result<Ready, EnrollmentError> {
        let record = self.registry.resolve_active(client_id, authenticate.key_id.0)?;
        let verifying_key = VerifyingKey::from_bytes(&record.public_key).map_err(|_| EnrollmentError::InvalidPublicKey)?;
        let enrolled_key = EnrolledClientKey { key_id: ClientKeyId(record.key_id), verifying_key };
        let ready = lifecycle.authenticate(authenticate, &enrolled_key)?;
        if lifecycle.authenticated_client() != Some(client_id) { return Err(EnrollmentError::AuthenticatedClientMismatch); }
        self.registry.mark_authenticated(client_id, authenticate.key_id.0, authenticated_at)?;
        Ok(ready)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::ClientCredentialStore;
    use crate::lifecycle::{AuthTranscript, Challenge, ConnectionId, Hello, NONCE_LEN};
    use ed25519_dalek::Signer;
    use golam_core::paths::RuntimeLayout;
    use golam_core::{PROTOCOL_VERSION, ResourceLimits};
    use std::fs;
    use std::sync::atomic::{AtomicU64,Ordering};
    use std::time::{SystemTime,UNIX_EPOCH};
    static N:AtomicU64=AtomicU64::new(0);
    fn authority()->(RuntimeLayout,AuthorityLayout){let n=N.fetch_add(1,Ordering::Relaxed);let t=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();let runtime=RuntimeLayout::initialize(std::env::temp_dir().join(format!("golam-enrollment-{}-{t}-{n}",std::process::id()))).unwrap();let authority=AuthorityLayout::initialize(&runtime).unwrap();(runtime,authority)}
    fn auth(signing:&ed25519_dalek::SigningKey,client_id:ClientId,key_id:ClientKeyId,client_nonce:[u8;NONCE_LEN],server_nonce:[u8;NONCE_LEN],server_epoch:u64,limits:ResourceLimits)->Authenticate{let hello=Hello{protocol_version:PROTOCOL_VERSION,client_id,client_nonce};let challenge=Challenge{protocol_version:PROTOCOL_VERSION,server_epoch,server_nonce,limits};let transcript=AuthTranscript::from_messages(hello,challenge).unwrap();let signature=signing.sign(&transcript.canonical_bytes(key_id).unwrap()).to_bytes();Authenticate{key_id,client_nonce,signature}}
    #[test]
    fn enrolled_client_authenticates_then_revocation_blocks_new_session(){let(runtime,authority)=authority();let store=ClientCredentialStore::new(&authority);let generated=store.generate(ClientId(41)).unwrap();let signing=store.load(generated.client_id,generated.key_id).unwrap();let mut enrollment=LocalClientEnrollment::open(&authority).unwrap();enrollment.enroll_generated(&generated,ClientKind::Test,"owner","2026-08-25T00:00:00Z").unwrap();let limits=ResourceLimits::default();let mut first=ServerLifecycle::new(9,[3;NONCE_LEN],limits,ConnectionId(77)).unwrap();first.receive_hello(Hello{protocol_version:PROTOCOL_VERSION,client_id:generated.client_id,client_nonce:[2;NONCE_LEN]}).unwrap();enrollment.authenticate_registered(&mut first,generated.client_id,auth(&signing,generated.client_id,generated.key_id,[2;NONCE_LEN],[3;NONCE_LEN],9,limits),"2026-08-25T00:01:00Z").unwrap();enrollment.revoke(generated.client_id,"2026-08-25T00:02:00Z").unwrap();let mut second=ServerLifecycle::new(10,[4;NONCE_LEN],limits,ConnectionId(78)).unwrap();second.receive_hello(Hello{protocol_version:PROTOCOL_VERSION,client_id:generated.client_id,client_nonce:[5;NONCE_LEN]}).unwrap();assert!(matches!(enrollment.authenticate_registered(&mut second,generated.client_id,auth(&signing,generated.client_id,generated.key_id,[5;NONCE_LEN],[4;NONCE_LEN],10,limits),"2026-08-25T00:03:00Z"),Err(EnrollmentError::Registry(ClientRegistryError::RevokedClient))));fs::remove_dir_all(runtime.root).unwrap();}
}
