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
        client_nonce: [u8; NONCE_LEN],
        server_nonce: [u8; NONCE_LEN],
        server_epoch: u64,
        limits: ResourceLimits,
    ) -> Authenticate {
        let hello = Hello {
            protocol_version: PROTOCOL_VERSION,
            client_id,
            client_nonce,
        };
        let challenge = Challenge {
            protocol_version: PROTOCOL_VERSION,
            server_epoch,
            server_nonce,
            limits,
        };
        let transcript = AuthTranscript::from_messages(hello, challenge).unwrap();
        Authenticate {
            key_id,
            client_nonce,
            signature: signing
                .sign(&transcript.canonical_bytes(key_id).unwrap())
                .to_bytes(),
        }
    }

    #[test]
    fn authority_owns_enrollment_authentication_revocation_and_protocol_audit() {
        let (runtime, authority) = authority();
        let store = ClientCredentialStore::new(&authority);
        let enrolled = store.generate(ClientId(701)).unwrap();
        let enrolled_signing = store.load(enrolled.client_id, enrolled.key_id).unwrap();
        let unknown = store.generate(ClientId(702)).unwrap();
        let unknown_signing = store.load(unknown.client_id, unknown.key_id).unwrap();
        let mut clients = ClientAuthority::open(&authority).unwrap();
        clients
            .enroll_generated(&enrolled, ClientKind::Test, "owner", "2026-08-25T02:00:00Z")
            .unwrap();
        let limits = ResourceLimits::default();

        let mut unknown_server =
            ServerLifecycle::new(50, [10; NONCE_LEN], limits, ConnectionId(300)).unwrap();
        let unknown_hello = Hello {
            protocol_version: PROTOCOL_VERSION,
            client_id: unknown.client_id,
            client_nonce: [11; NONCE_LEN],
        };
        unknown_server.receive_hello(unknown_hello).unwrap();
        assert!(matches!(
            clients.authenticate_registered(
                &mut unknown_server,
                ConnectionId(300),
                unknown.client_id,
                auth(
                    &unknown_signing,
                    unknown.client_id,
                    unknown.key_id,
                    unknown_hello.client_nonce,
                    [10; NONCE_LEN],
                    50,
                    limits,
                ),
                "2026-08-25T02:01:00Z",
            ),
            Err(ClientAuthorityError::Registry(
                ClientRegistryError::UnknownClient
            ))
        ));
        assert_eq!(unknown_server.phase(), LifecyclePhase::Closed);

        let hello = Hello {
            protocol_version: PROTOCOL_VERSION,
            client_id: enrolled.client_id,
            client_nonce: [13; NONCE_LEN],
        };
        let mut wrong_key =
            ServerLifecycle::new(51, [12; NONCE_LEN], limits, ConnectionId(301)).unwrap();
        wrong_key.receive_hello(hello).unwrap();
        assert!(matches!(
            clients.authenticate_registered(
                &mut wrong_key,
                ConnectionId(301),
                enrolled.client_id,
                auth(
                    &unknown_signing,
                    enrolled.client_id,
                    unknown.key_id,
                    hello.client_nonce,
                    [12; NONCE_LEN],
                    51,
                    limits,
                ),
                "2026-08-25T02:02:00Z",
            ),
            Err(ClientAuthorityError::Registry(
                ClientRegistryError::ClientKeyMismatch
            ))
        ));
        assert_eq!(wrong_key.phase(), LifecyclePhase::Closed);

        let captured_hello = Hello {
            protocol_version: PROTOCOL_VERSION,
            client_id: enrolled.client_id,
            client_nonce: [14; NONCE_LEN],
        };
        let captured = auth(
            &enrolled_signing,
            enrolled.client_id,
            enrolled.key_id,
            captured_hello.client_nonce,
            [15; NONCE_LEN],
            52,
            limits,
        );
        let mut valid =
            ServerLifecycle::new(52, [15; NONCE_LEN], limits, ConnectionId(302)).unwrap();
        valid.receive_hello(captured_hello).unwrap();
        clients
            .authenticate_registered(
                &mut valid,
                ConnectionId(302),
                enrolled.client_id,
                captured,
                "2026-08-25T02:03:00Z",
            )
            .unwrap();

        let mut replay =
            ServerLifecycle::new(53, [16; NONCE_LEN], limits, ConnectionId(303)).unwrap();
        replay.receive_hello(captured_hello).unwrap();
        assert!(matches!(
            clients.authenticate_registered(
                &mut replay,
                ConnectionId(303),
                enrolled.client_id,
                captured,
                "2026-08-25T02:04:00Z",
            ),
            Err(ClientAuthorityError::Lifecycle(
                LifecycleError::AuthenticationFailed
            ))
        ));
        assert_eq!(replay.phase(), LifecyclePhase::Closed);

        clients
            .revoke(enrolled.client_id, "2026-08-25T02:05:00Z")
            .unwrap();
        let revoked_hello = Hello {
            client_nonce: [18; NONCE_LEN],
            ..captured_hello
        };
        let mut revoked =
            ServerLifecycle::new(54, [17; NONCE_LEN], limits, ConnectionId(304)).unwrap();
        revoked.receive_hello(revoked_hello).unwrap();
        assert!(matches!(
            clients.authenticate_registered(
                &mut revoked,
                ConnectionId(304),
                enrolled.client_id,
                auth(
                    &enrolled_signing,
                    enrolled.client_id,
                    enrolled.key_id,
                    revoked_hello.client_nonce,
                    [17; NONCE_LEN],
                    54,
                    limits,
                ),
                "2026-08-25T02:06:00Z",
            ),
            Err(ClientAuthorityError::Registry(
                ClientRegistryError::RevokedClient
            ))
        ));
        assert_eq!(revoked.phase(), LifecyclePhase::Closed);

        let mut pre_ready =
            ServerLifecycle::new(55, [19; NONCE_LEN], limits, ConnectionId(305)).unwrap();
        pre_ready
            .receive_hello(Hello {
                client_nonce: [20; NONCE_LEN],
                ..captured_hello
            })
            .unwrap();
        clients
            .reject_unauthenticated_request(
                &mut pre_ready,
                ConnectionId(305),
                enrolled.client_id,
                None,
                "2026-08-25T02:07:00Z",
            )
            .unwrap();
        assert_eq!(pre_ready.phase(), LifecyclePhase::Closed);

        let reasons: Vec<_> = clients
            .protocol_audit_records()
            .unwrap()
            .iter()
            .map(|record| record.reason)
            .collect();
        assert_eq!(
            reasons,
            vec![
                ProtocolRejectionReason::UnknownClient,
                ProtocolRejectionReason::ClientKeyMismatch,
                ProtocolRejectionReason::AuthenticationFailed,
                ProtocolRejectionReason::RevokedClient,
                ProtocolRejectionReason::UnauthenticatedRequest,
            ]
        );
        drop(clients);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
