use std::error::Error;
use std::fmt;

use ed25519_dalek::VerifyingKey;
use golam_core::ClientId;
use golam_core::authority::AuthorityLayout;
use golam_ledger::clients::{
    AssuranceClass, ClientKind, ClientRecord, ClientRegistry, ClientRegistryError, EnrollClient,
};
use golam_ledger::protocol_audit::{
    AppendProtocolRejection, ProtocolAuditError, ProtocolAuditLog, ProtocolAuditRecord,
    ProtocolRejectionReason,
};

use crate::credentials::{GeneratedClientCredential, key_id_for_public_key};
use crate::lifecycle::{
    Authenticate, ClientKeyId, ConnectionId, EnrolledClientKey, LifecycleError, LifecyclePhase,
    Ready, ServerLifecycle, ShutdownReason,
};

#[derive(Debug)]
pub enum EnrollmentError {
    Registry(ClientRegistryError),
    Audit(ProtocolAuditError),
    InvalidPublicKey,
    KeyFingerprintMismatch,
    Lifecycle(LifecycleError),
    AuthenticatedClientMismatch,
}

impl fmt::Display for EnrollmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registry(error) => write!(f, "client enrollment registry error: {error}"),
            Self::Audit(error) => write!(f, "client protocol audit error: {error}"),
            Self::InvalidPublicKey => f.write_str("client public key is not a valid Ed25519 key"),
            Self::KeyFingerprintMismatch => {
                f.write_str("client key id does not match the enrolled public key fingerprint")
            }
            Self::Lifecycle(error) => write!(f, "client lifecycle authentication error: {error}"),
            Self::AuthenticatedClientMismatch => {
                f.write_str("authenticated lifecycle client does not match requested enrollment")
            }
        }
    }
}
impl Error for EnrollmentError {
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
impl From<ClientRegistryError> for EnrollmentError {
    fn from(value: ClientRegistryError) -> Self {
        Self::Registry(value)
    }
}
impl From<ProtocolAuditError> for EnrollmentError {
    fn from(value: ProtocolAuditError) -> Self {
        Self::Audit(value)
    }
}
impl From<LifecycleError> for EnrollmentError {
    fn from(value: LifecycleError) -> Self {
        Self::Lifecycle(value)
    }
}

pub struct LocalClientEnrollment {
    registry: ClientRegistry,
    audit: ProtocolAuditLog,
}
impl LocalClientEnrollment {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, EnrollmentError> {
        Ok(Self {
            registry: ClientRegistry::open(layout)?,
            audit: ProtocolAuditLog::open(layout)?,
        })
    }

    pub fn enroll_generated(
        &mut self,
        generated: &GeneratedClientCredential,
        kind: ClientKind,
        owner_principal: &str,
        enrolled_at: &str,
    ) -> Result<ClientRecord, EnrollmentError> {
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
    pub fn enroll_public(
        &mut self,
        client_id: ClientId,
        key_id: ClientKeyId,
        public_key: [u8; 32],
        kind: ClientKind,
        owner_principal: &str,
        enrolled_at: &str,
        assurance_class: AssuranceClass,
    ) -> Result<ClientRecord, EnrollmentError> {
        VerifyingKey::from_bytes(&public_key).map_err(|_| EnrollmentError::InvalidPublicKey)?;
        if key_id_for_public_key(public_key) != key_id {
            return Err(EnrollmentError::KeyFingerprintMismatch);
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

    pub fn revoke(
        &mut self,
        client_id: ClientId,
        revoked_at: &str,
    ) -> Result<ClientRecord, EnrollmentError> {
        Ok(self.registry.revoke(client_id, revoked_at)?)
    }

    pub fn resolve_active(
        &self,
        client_id: ClientId,
        key_id: ClientKeyId,
    ) -> Result<ClientRecord, EnrollmentError> {
        Ok(self.registry.resolve_active(client_id, key_id.0)?)
    }

    pub fn protocol_audit_records(&self) -> Result<Vec<ProtocolAuditRecord>, EnrollmentError> {
        Ok(self.audit.records()?)
    }

    pub fn reject_unauthenticated_request(
        &mut self,
        lifecycle: &mut ServerLifecycle,
        connection_id: ConnectionId,
        client_id: ClientId,
        key_id: Option<ClientKeyId>,
        detected_at: &str,
    ) -> Result<(), EnrollmentError> {
        self.audit_rejection(
            lifecycle,
            connection_id,
            client_id,
            key_id.unwrap_or(ClientKeyId([0; 32])),
            detected_at,
            ProtocolRejectionReason::UnauthenticatedRequest,
        )
    }

    pub fn authenticate_registered(
        &mut self,
        lifecycle: &mut ServerLifecycle,
        connection_id: ConnectionId,
        client_id: ClientId,
        authenticate: Authenticate,
        authenticated_at: &str,
    ) -> Result<Ready, EnrollmentError> {
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
                return Err(EnrollmentError::Registry(error));
            }
        };
        let verifying_key = match VerifyingKey::from_bytes(&record.public_key) {
            Ok(key) => key,
            Err(_) => {
                close_for_authentication_failure(lifecycle);
                return Err(EnrollmentError::InvalidPublicKey);
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
                return Err(EnrollmentError::Lifecycle(error));
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
            return Err(EnrollmentError::AuthenticatedClientMismatch);
        }
        if let Err(error) = self
            .registry
            .mark_authenticated(client_id, authenticate.key_id.0, authenticated_at)
        {
            close_for_authentication_failure(lifecycle);
            return Err(EnrollmentError::Registry(error));
        }
        Ok(ready)
    }

    fn audit_rejection(
        &mut self,
        lifecycle: &mut ServerLifecycle,
        connection_id: ConnectionId,
        client_id: ClientId,
        key_id: ClientKeyId,
        detected_at: &str,
        reason: ProtocolRejectionReason,
    ) -> Result<(), EnrollmentError> {
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
    use crate::credentials::ClientCredentialStore;
    use crate::lifecycle::{AuthTranscript, Challenge, Hello, NONCE_LEN};
    use ed25519_dalek::Signer;
    use golam_core::paths::RuntimeLayout;
    use golam_core::{PROTOCOL_VERSION, ResourceLimits};
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
        let runtime = RuntimeLayout::initialize(
            std::env::temp_dir().join(format!("golam-enrollment-{}-{t}-{n}", std::process::id())),
        )
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
        let signature = signing
            .sign(&transcript.canonical_bytes(key_id).unwrap())
            .to_bytes();
        Authenticate {
            key_id,
            client_nonce,
            signature,
        }
    }

    #[test]
    fn enrolled_client_authenticates_then_revocation_blocks_and_audits_new_session() {
        let (runtime, authority) = authority();
        let store = ClientCredentialStore::new(&authority);
        let generated = store.generate(ClientId(41)).unwrap();
        let signing = store.load(generated.client_id, generated.key_id).unwrap();
        let mut enrollment = LocalClientEnrollment::open(&authority).unwrap();
        enrollment
            .enroll_generated(
                &generated,
                ClientKind::Test,
                "owner",
                "2026-08-25T00:00:00Z",
            )
            .unwrap();
        let limits = ResourceLimits::default();
        let mut first = ServerLifecycle::new(9, [3; NONCE_LEN], limits, ConnectionId(77)).unwrap();
        first
            .receive_hello(Hello {
                protocol_version: PROTOCOL_VERSION,
                client_id: generated.client_id,
                client_nonce: [2; NONCE_LEN],
            })
            .unwrap();
        enrollment
            .authenticate_registered(
                &mut first,
                ConnectionId(77),
                generated.client_id,
                auth(
                    &signing,
                    generated.client_id,
                    generated.key_id,
                    [2; NONCE_LEN],
                    [3; NONCE_LEN],
                    9,
                    limits,
                ),
                "2026-08-25T00:01:00Z",
            )
            .unwrap();
        enrollment
            .revoke(generated.client_id, "2026-08-25T00:02:00Z")
            .unwrap();
        let mut second =
            ServerLifecycle::new(10, [4; NONCE_LEN], limits, ConnectionId(78)).unwrap();
        second
            .receive_hello(Hello {
                protocol_version: PROTOCOL_VERSION,
                client_id: generated.client_id,
                client_nonce: [5; NONCE_LEN],
            })
            .unwrap();
        assert!(matches!(
            enrollment.authenticate_registered(
                &mut second,
                ConnectionId(78),
                generated.client_id,
                auth(
                    &signing,
                    generated.client_id,
                    generated.key_id,
                    [5; NONCE_LEN],
                    [4; NONCE_LEN],
                    10,
                    limits
                ),
                "2026-08-25T00:03:00Z"
            ),
            Err(EnrollmentError::Registry(
                ClientRegistryError::RevokedClient
            ))
        ));
        assert_eq!(second.phase(), LifecyclePhase::Closed);
        let audit = enrollment.protocol_audit_records().unwrap();
        assert_eq!(audit.len(), 1);
        assert_eq!(audit[0].connection_id, 78);
        assert_eq!(audit[0].reason, ProtocolRejectionReason::RevokedClient);
        drop(enrollment);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn replay_and_unauthenticated_request_are_closed_and_audited() {
        let (runtime, authority) = authority();
        let store = ClientCredentialStore::new(&authority);
        let generated = store.generate(ClientId(51)).unwrap();
        let signing = store.load(generated.client_id, generated.key_id).unwrap();
        let mut enrollment = LocalClientEnrollment::open(&authority).unwrap();
        enrollment
            .enroll_generated(
                &generated,
                ClientKind::Test,
                "owner",
                "2026-08-25T01:00:00Z",
            )
            .unwrap();
        let limits = ResourceLimits::default();
        let hello = Hello {
            protocol_version: PROTOCOL_VERSION,
            client_id: generated.client_id,
            client_nonce: [6; NONCE_LEN],
        };
        let captured = auth(
            &signing,
            generated.client_id,
            generated.key_id,
            hello.client_nonce,
            [7; NONCE_LEN],
            20,
            limits,
        );
        let mut first =
            ServerLifecycle::new(20, [7; NONCE_LEN], limits, ConnectionId(90)).unwrap();
        first.receive_hello(hello).unwrap();
        enrollment
            .authenticate_registered(
                &mut first,
                ConnectionId(90),
                generated.client_id,
                captured,
                "2026-08-25T01:01:00Z",
            )
            .unwrap();

        let mut replay =
            ServerLifecycle::new(21, [8; NONCE_LEN], limits, ConnectionId(91)).unwrap();
        replay.receive_hello(hello).unwrap();
        assert!(matches!(
            enrollment.authenticate_registered(
                &mut replay,
                ConnectionId(91),
                generated.client_id,
                captured,
                "2026-08-25T01:02:00Z"
            ),
            Err(EnrollmentError::Lifecycle(
                LifecycleError::AuthenticationFailed
            ))
        ));
        assert_eq!(replay.phase(), LifecyclePhase::Closed);

        let mut pre_ready =
            ServerLifecycle::new(22, [9; NONCE_LEN], limits, ConnectionId(92)).unwrap();
        pre_ready.receive_hello(hello).unwrap();
        enrollment
            .reject_unauthenticated_request(
                &mut pre_ready,
                ConnectionId(92),
                generated.client_id,
                None,
                "2026-08-25T01:03:00Z",
            )
            .unwrap();
        assert_eq!(pre_ready.phase(), LifecyclePhase::Closed);

        let audit = enrollment.protocol_audit_records().unwrap();
        assert_eq!(audit.len(), 2);
        assert_eq!(audit[0].reason, ProtocolRejectionReason::AuthenticationFailed);
        assert_eq!(audit[1].reason, ProtocolRejectionReason::UnauthenticatedRequest);
        drop(enrollment);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
