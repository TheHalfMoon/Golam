use std::error::Error;
use std::fmt;

use ed25519_dalek::VerifyingKey;
use golam_core::ClientId;
use golam_core::authority::AuthorityLayout;
use golam_ipc::credentials::{GeneratedClientCredential, key_id_for_public_key};
use golam_ipc::lifecycle::{
    Authenticate, ClientKeyId, ConnectionId, EnrolledClientKey, LifecycleError, LifecyclePhase,
    Ready, ServerLifecycle, ShutdownReason,
};
use golam_ledger::clients::{
    AssuranceClass, ClientKind, ClientRecord, ClientRegistry, ClientRegistryError, EnrollClient,
};
#[cfg(test)]
use golam_ledger::protocol_audit::ProtocolAuditRecord;
use golam_ledger::protocol_audit::{
    AppendProtocolRejection, ProtocolAuditError, ProtocolAuditLog, ProtocolRejectionReason,
};

#[derive(Debug)]
pub enum ClientAuthorityError {
    Registry(ClientRegistryError),
    Audit(ProtocolAuditError),
    InvalidPublicKey,
    KeyFingerprintMismatch,
    Lifecycle(LifecycleError),
    AuthenticatedClientMismatch,
}

impl fmt::Display for ClientAuthorityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registry(error) => write!(f, "client authority registry error: {error}"),
            Self::Audit(error) => write!(f, "client authority protocol audit error: {error}"),
            Self::InvalidPublicKey => f.write_str("client public key is not a valid Ed25519 key"),
            Self::KeyFingerprintMismatch => {
                f.write_str("client key id does not match the public key fingerprint")
            }
            Self::Lifecycle(error) => write!(f, "client lifecycle authentication error: {error}"),
            Self::AuthenticatedClientMismatch => {
                f.write_str("authenticated lifecycle client does not match requested client")
            }
        }
    }
}

impl Error for ClientAuthorityError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Registry(error) => Some(error),
            Self::Audit(error) => Some(error),
            Self::Lifecycle(error) => Some(error),
            Self::InvalidPublicKey
            | Self::KeyFingerprintMismatch
            | Self::AuthenticatedClientMismatch => None,
        }
    }
}

impl From<ClientRegistryError> for ClientAuthorityError {
    fn from(value: ClientRegistryError) -> Self {
        Self::Registry(value)
    }
}

impl From<ProtocolAuditError> for ClientAuthorityError {
    fn from(value: ProtocolAuditError) -> Self {
        Self::Audit(value)
    }
}

impl From<LifecycleError> for ClientAuthorityError {
    fn from(value: LifecycleError) -> Self {
        Self::Lifecycle(value)
    }
}

pub(crate) struct ClientAuthority {
    registry: ClientRegistry,
    audit: ProtocolAuditLog,
}
impl ClientAuthority {
    pub(crate) fn open(layout: &AuthorityLayout) -> Result<Self, ClientAuthorityError> {
        Ok(Self {
            registry: ClientRegistry::open(layout)?,
            audit: ProtocolAuditLog::open(layout)?,
        })
    }

    pub(crate) fn enroll_generated(
        &mut self,
        generated: &GeneratedClientCredential,
        kind: ClientKind,
        owner_principal: &str,
        enrolled_at: &str,
    ) -> Result<ClientRecord, ClientAuthorityError> {
        self.enroll_public(
            generated.client_id,
            generated.key_id,
            generated.public_key,
            kind,
            owner_principal,
            enrolled_at,
            AssuranceClass::FilesystemUserPrivateV1,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn enroll_public(
        &mut self,
        client_id: ClientId,
        key_id: ClientKeyId,
        public_key: [u8; 32],
        kind: ClientKind,
        owner_principal: &str,
        enrolled_at: &str,
        assurance_class: AssuranceClass,
    ) -> Result<ClientRecord, ClientAuthorityError> {
        VerifyingKey::from_bytes(&public_key)
            .map_err(|_| ClientAuthorityError::InvalidPublicKey)?;
        if key_id_for_public_key(public_key) != key_id {
            return Err(ClientAuthorityError::KeyFingerprintMismatch);
        }
        Ok(self.registry.enroll(EnrollClient {
            client_id,
            key_id: key_id.0,
            public_key,
            kind,
            owner_principal,
            enrolled_at,
            assurance_class,
        })?)
    }

    pub(crate) fn revoke(
        &mut self,
        client_id: ClientId,
        revoked_at: &str,
    ) -> Result<ClientRecord, ClientAuthorityError> {
        Ok(self.registry.revoke(client_id, revoked_at)?)
    }

    pub(crate) fn resolve_active_record(
        &self,
        client_id: ClientId,
        key_id: ClientKeyId,
    ) -> Result<ClientRecord, ClientAuthorityError> {
        Ok(self.registry.resolve_active(client_id, key_id.0)?)
    }

    pub(crate) fn authenticate_registered(
        &mut self,
        lifecycle: &mut ServerLifecycle,
        connection_id: ConnectionId,
        client_id: ClientId,
        authenticate: Authenticate,
        authenticated_at: &str,
    ) -> Result<Ready, ClientAuthorityError> {
        let record = match self
            .registry
            .resolve_active(client_id, authenticate.key_id.0)
        {
            Ok(record) => record,
            Err(error) => {
                close_for_authentication_failure(lifecycle);
                if let Some(reason) = registry_rejection_reason(&error) {
                    self.audit.append_rejection(AppendProtocolRejection {
                        connection_id: connection_id.0,
                        client_id,
                        key_id: authenticate.key_id.0,
                        detected_at: authenticated_at,
                        reason,
                    })?;
                }
                return Err(ClientAuthorityError::Registry(error));
            }
        };
        let verifying_key = match VerifyingKey::from_bytes(&record.public_key) {
            Ok(key) => key,
            Err(_) => {
                close_for_authentication_failure(lifecycle);
                return Err(ClientAuthorityError::InvalidPublicKey);
            }
        };
        let enrolled_key = EnrolledClientKey {
            key_id: ClientKeyId(record.key_id),
            verifying_key,
        };
        let ready = match lifecycle.authenticate(authenticate, &enrolled_key) {
            Ok(ready) => ready,
            Err(error) => {
                self.audit.append_rejection(AppendProtocolRejection {
                    connection_id: connection_id.0,
                    client_id,
                    key_id: authenticate.key_id.0,
                    detected_at: authenticated_at,
                    reason: lifecycle_rejection_reason(&error),
                })?;
                return Err(ClientAuthorityError::Lifecycle(error));
            }
        };
        if lifecycle.authenticated_client() != Some(client_id) {
            self.audit_rejection(
                lifecycle,
                connection_id,
                client_id,
                authenticate.key_id,
                authenticated_at,
                ProtocolRejectionReason::ProtocolViolation,
            )?;
            return Err(ClientAuthorityError::AuthenticatedClientMismatch);
        }
        if let Err(error) =
            self.registry
                .mark_authenticated(client_id, authenticate.key_id.0, authenticated_at)
        {
            close_for_authentication_failure(lifecycle);
            return Err(ClientAuthorityError::Registry(error));
        }
        Ok(ready)
    }

    pub(crate) fn reject_unauthenticated_request(
        &mut self,
        lifecycle: &mut ServerLifecycle,
        connection_id: ConnectionId,
        client_id: ClientId,
        key_id: Option<ClientKeyId>,
        detected_at: &str,
    ) -> Result<(), ClientAuthorityError> {
        self.audit_rejection(
            lifecycle,
            connection_id,
            client_id,
            key_id.unwrap_or(ClientKeyId([0; 32])),
            detected_at,
            ProtocolRejectionReason::UnauthenticatedRequest,
        )
    }

    #[cfg(test)]
    pub(crate) fn protocol_audit_records(
        &self,
    ) -> Result<Vec<ProtocolAuditRecord>, ClientAuthorityError> {
        Ok(self.audit.records()?)
    }

    fn audit_rejection(
        &mut self,
        lifecycle: &mut ServerLifecycle,
        connection_id: ConnectionId,
        client_id: ClientId,
        key_id: ClientKeyId,
        detected_at: &str,
        reason: ProtocolRejectionReason,
    ) -> Result<(), ClientAuthorityError> {
        close_for_authentication_failure(lifecycle);
        self.audit.append_rejection(AppendProtocolRejection {
            connection_id: connection_id.0,
            client_id,
            key_id: key_id.0,
            detected_at,
            reason,
        })?;
        Ok(())
    }
}

fn close_for_authentication_failure(lifecycle: &mut ServerLifecycle) {
    if lifecycle.phase() != LifecyclePhase::Closed {
        let _ = lifecycle.receive_shutdown(ShutdownReason::AuthenticationFailed);
    }
}

fn registry_rejection_reason(error: &ClientRegistryError) -> Option<ProtocolRejectionReason> {
    match error {
        ClientRegistryError::UnknownClient => Some(ProtocolRejectionReason::UnknownClient),
        ClientRegistryError::RevokedClient => Some(ProtocolRejectionReason::RevokedClient),
        ClientRegistryError::ClientKeyMismatch => Some(ProtocolRejectionReason::ClientKeyMismatch),
        _ => None,
    }
}

fn lifecycle_rejection_reason(error: &LifecycleError) -> ProtocolRejectionReason {
    match error {
        LifecycleError::ClientNonceMismatch => ProtocolRejectionReason::ClientNonceMismatch,
        LifecycleError::KeyIdMismatch => ProtocolRejectionReason::KeyIdMismatch,
        LifecycleError::AuthenticationFailed => ProtocolRejectionReason::AuthenticationFailed,
        LifecycleError::InvalidPhase { .. } => ProtocolRejectionReason::InvalidPhase,
        _ => ProtocolRejectionReason::ProtocolViolation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;
    use golam_core::paths::RuntimeLayout;
    use golam_core::{PROTOCOL_VERSION, ResourceLimits};
    use golam_ipc::credentials::ClientCredentialStore;
    use golam_ipc::lifecycle::{AuthTranscript, Challenge, Hello, NONCE_LEN};
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
            "golam-client-authority-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    fn auth(
        signing: &ed25519_dalek::SigningKey,
        client_id: ClientId,
        key_id: ClientKeyId,
        server_epoch: u64,
        server_nonce: [u8; NONCE_LEN],
        connection_id: ConnectionId,
    ) -> (ServerLifecycle, Authenticate) {
        let hello = golam_ipc::lifecycle::Hello {
            protocol_version: PROTOCOL_VERSION,
            client_id,
            client_nonce: [3; NONCE_LEN],
        };
        let mut lifecycle = ServerLifecycle::new(
            server_epoch,
            server_nonce,
            ResourceLimits::default(),
            connection_id,
        )
        .unwrap();
        let challenge = lifecycle.receive_hello(hello).unwrap();
        let transcript = AuthTranscript::from_messages(hello, challenge).unwrap();
        let authenticate = Authenticate {
            key_id,
            client_nonce: hello.client_nonce,
            signature: signing
                .sign(&transcript.canonical_bytes(key_id).unwrap())
                .to_bytes(),
        };
        (lifecycle, authenticate)
    }

    #[test]
    fn enrolled_key_authenticates_and_marks_last_seen() {
        let (runtime, layout) = authority();
        let store = ClientCredentialStore::new(&layout);
        let generated = store.generate(ClientId(10)).unwrap();
        let signing = store.load(generated.client_id, generated.key_id).unwrap();
        let mut clients = ClientAuthority::open(&layout).unwrap();
        clients
            .enroll_generated(&generated, ClientKind::Cli, "owner", "2026-08-25T00:00:00Z")
            .unwrap();
        let (mut lifecycle, authenticate) = auth(
            &signing,
            generated.client_id,
            generated.key_id,
            7,
            [9; NONCE_LEN],
            ConnectionId(11),
        );
        clients
            .authenticate_registered(
                &mut lifecycle,
                ConnectionId(11),
                generated.client_id,
                authenticate,
                "2026-08-25T00:01:00Z",
            )
            .unwrap();
        let record = ClientRegistry::open(&layout)
            .unwrap()
            .record_for_client(generated.client_id)
            .unwrap()
            .unwrap();
        assert_eq!(
            record.last_authenticated_at.as_deref(),
            Some("2026-08-25T00:01:00Z")
        );
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn unknown_client_fails_before_ready_and_is_audited() {
        let (runtime, layout) = authority();
        let store = ClientCredentialStore::new(&layout);
        let generated = store.generate(ClientId(12)).unwrap();
        let signing = store.load(generated.client_id, generated.key_id).unwrap();
        let mut clients = ClientAuthority::open(&layout).unwrap();
        let (mut lifecycle, authenticate) = auth(
            &signing,
            generated.client_id,
            generated.key_id,
            8,
            [4; NONCE_LEN],
            ConnectionId(13),
        );
        let result = clients.authenticate_registered(
            &mut lifecycle,
            ConnectionId(13),
            generated.client_id,
            authenticate,
            "2026-08-25T00:02:00Z",
        );
        assert!(matches!(
            result,
            Err(ClientAuthorityError::Registry(
                ClientRegistryError::UnknownClient
            ))
        ));
        let audit = clients.protocol_audit_records().unwrap();
        assert_eq!(audit.len(), 1);
        assert_eq!(audit[0].reason, ProtocolRejectionReason::UnknownClient);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn wrong_signature_closes_before_ready() {
        let (runtime, layout) = authority();
        let store = ClientCredentialStore::new(&layout);
        let generated = store.generate(ClientId(14)).unwrap();
        let wrong = store.generate(ClientId(15)).unwrap();
        let wrong_signing = store.load(wrong.client_id, wrong.key_id).unwrap();
        let mut clients = ClientAuthority::open(&layout).unwrap();
        clients
            .enroll_generated(&generated, ClientKind::Cli, "owner", "2026-08-25T00:03:00Z")
            .unwrap();
        let (mut lifecycle, authenticate) = auth(
            &wrong_signing,
            generated.client_id,
            generated.key_id,
            9,
            [5; NONCE_LEN],
            ConnectionId(16),
        );
        let result = clients.authenticate_registered(
            &mut lifecycle,
            ConnectionId(16),
            generated.client_id,
            authenticate,
            "2026-08-25T00:04:00Z",
        );
        assert!(matches!(
            result,
            Err(ClientAuthorityError::Lifecycle(
                LifecycleError::AuthenticationFailed
            ))
        ));
        assert_eq!(lifecycle.phase(), LifecyclePhase::Closed);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn reuse_across_connection_ids_is_rejected() {
        let (runtime, layout) = authority();
        let store = ClientCredentialStore::new(&layout);
        let generated = store.generate(ClientId(17)).unwrap();
        let signing = store.load(generated.client_id, generated.key_id).unwrap();
        let mut clients = ClientAuthority::open(&layout).unwrap();
        clients
            .enroll_generated(&generated, ClientKind::Cli, "owner", "2026-08-25T00:05:00Z")
            .unwrap();
        let (mut first, authenticate) = auth(
            &signing,
            generated.client_id,
            generated.key_id,
            10,
            [6; NONCE_LEN],
            ConnectionId(18),
        );
        clients
            .authenticate_registered(
                &mut first,
                ConnectionId(18),
                generated.client_id,
                authenticate,
                "2026-08-25T00:06:00Z",
            )
            .unwrap();
        assert_eq!(first.phase(), LifecyclePhase::Ready);
        let (mut second, _) = auth(
            &signing,
            generated.client_id,
            generated.key_id,
            11,
            [7; NONCE_LEN],
            ConnectionId(19),
        );
        let result = clients.authenticate_registered(
            &mut second,
            ConnectionId(19),
            generated.client_id,
            authenticate,
            "2026-08-25T00:07:00Z",
        );
        assert!(matches!(
            result,
            Err(ClientAuthorityError::Lifecycle(
                LifecycleError::AuthenticationFailed
            ))
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
