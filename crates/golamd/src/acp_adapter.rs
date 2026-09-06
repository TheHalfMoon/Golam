#![forbid(unsafe_code)]

//! ACP interoperability binding over Golam's authenticated local-client boundary.
//!
//! This adapter carries no `KernelApi`, capability lease, approval, or Effect authority. It can
//! only bind an ACP request to an already authenticated local client, the exact authenticated
//! connection epoch, the exact enrolled principal, and one reviewed scope. Any change requires a
//! fresh binding and normal downstream Kernel authorization.

use std::error::Error;
use std::fmt;

use golam_core::digest::sha256;
use golam_core::tool_request::BindingDigest;
use golam_core::{CanonicalEncoder, ClientId, CoreError};
use golam_ipc::lifecycle::{AuthenticatedLocalClient, ConnectionId};
use golam_kernel::{Principal, PrincipalKind};

const ACP_CLIENT_BINDING_DOMAIN: &[u8] = b"golam:acp-client-binding:v1";
const ACP_SCOPE_DOMAIN: &[u8] = b"golam:acp-scope:v1";
const MAX_PRINCIPAL_SUBJECT_BYTES: usize = 256;
const MAX_SCOPE_BYTES: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcpClientBinding {
    client_id: ClientId,
    connection_id: ConnectionId,
    server_epoch: u64,
    principal_subject: String,
    scope: String,
    scope_ref: BindingDigest,
    binding_digest: BindingDigest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcpRequestBinding {
    pub client_binding_digest: BindingDigest,
    pub client_id: ClientId,
    pub connection_id: ConnectionId,
    pub scope_ref: BindingDigest,
    pub request_ref: BindingDigest,
}

#[derive(Debug)]
pub enum AcpAdapterError {
    Core(CoreError),
    PrincipalNotEnrolled,
    PrincipalClientMismatch,
    InvalidPrincipalSubject,
    InvalidScope,
    InvalidRequestRef,
    ScopeMismatch,
    ClientBindingMismatch,
    ClientIdentityMismatch,
    ConnectionMismatch,
}

impl fmt::Display for AcpAdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core(error) => write!(f, "ACP canonical binding failed: {error}"),
            Self::PrincipalNotEnrolled => {
                f.write_str("ACP requires an already authenticated enrolled-client principal")
            }
            Self::PrincipalClientMismatch => {
                f.write_str("ACP principal client identity differs from authenticated IPC client")
            }
            Self::InvalidPrincipalSubject => {
                f.write_str("ACP principal subject is empty, oversized, or contains control data")
            }
            Self::InvalidScope => {
                f.write_str("ACP scope is empty, oversized, or contains control data")
            }
            Self::InvalidRequestRef => f.write_str("ACP request reference must be non-zero"),
            Self::ScopeMismatch => {
                f.write_str("ACP request scope differs from the authenticated reviewed scope")
            }
            Self::ClientBindingMismatch => {
                f.write_str("ACP request was prepared for a different authenticated binding")
            }
            Self::ClientIdentityMismatch => {
                f.write_str("ACP request client identity differs from the current binding")
            }
            Self::ConnectionMismatch => {
                f.write_str("ACP request belongs to a different authenticated connection")
            }
        }
    }
}

impl Error for AcpAdapterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CoreError> for AcpAdapterError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl AcpClientBinding {
    pub fn from_authenticated_local_client(
        authenticated: AuthenticatedLocalClient,
        principal: Principal<'_>,
        scope: &str,
    ) -> Result<Self, AcpAdapterError> {
        if principal.kind != PrincipalKind::EnrolledClient || principal.client_id.is_none() {
            return Err(AcpAdapterError::PrincipalNotEnrolled);
        }
        if principal.client_id != Some(authenticated.client_id()) {
            return Err(AcpAdapterError::PrincipalClientMismatch);
        }
        validate_text(
            principal.subject,
            MAX_PRINCIPAL_SUBJECT_BYTES,
            AcpAdapterError::InvalidPrincipalSubject,
        )?;
        validate_text(scope, MAX_SCOPE_BYTES, AcpAdapterError::InvalidScope)?;

        let scope_ref = scope_ref(scope)?;
        let binding_digest = client_binding_digest(authenticated, principal.subject, scope_ref)?;
        Ok(Self {
            client_id: authenticated.client_id(),
            connection_id: authenticated.connection_id(),
            server_epoch: authenticated.server_epoch(),
            principal_subject: principal.subject.to_owned(),
            scope: scope.to_owned(),
            scope_ref,
            binding_digest,
        })
    }

    pub const fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub const fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }

    pub const fn server_epoch(&self) -> u64 {
        self.server_epoch
    }

    pub fn principal(&self) -> Principal<'_> {
        Principal::enrolled_client(&self.principal_subject, self.client_id)
    }

    pub fn scope(&self) -> &str {
        &self.scope
    }

    pub const fn scope_ref(&self) -> BindingDigest {
        self.scope_ref
    }

    pub const fn binding_digest(&self) -> BindingDigest {
        self.binding_digest
    }

    pub fn bind_request(
        &self,
        request_ref: BindingDigest,
        requested_scope_ref: BindingDigest,
    ) -> Result<AcpRequestBinding, AcpAdapterError> {
        if request_ref.bytes() == [0; 32] {
            return Err(AcpAdapterError::InvalidRequestRef);
        }
        if requested_scope_ref != self.scope_ref {
            return Err(AcpAdapterError::ScopeMismatch);
        }
        Ok(AcpRequestBinding {
            client_binding_digest: self.binding_digest,
            client_id: self.client_id,
            connection_id: self.connection_id,
            scope_ref: self.scope_ref,
            request_ref,
        })
    }

    pub fn revalidate_request(&self, request: &AcpRequestBinding) -> Result<(), AcpAdapterError> {
        if request.client_binding_digest != self.binding_digest {
            return Err(AcpAdapterError::ClientBindingMismatch);
        }
        if request.client_id != self.client_id {
            return Err(AcpAdapterError::ClientIdentityMismatch);
        }
        if request.connection_id != self.connection_id {
            return Err(AcpAdapterError::ConnectionMismatch);
        }
        if request.scope_ref != self.scope_ref {
            return Err(AcpAdapterError::ScopeMismatch);
        }
        if request.request_ref.bytes() == [0; 32] {
            return Err(AcpAdapterError::InvalidRequestRef);
        }
        Ok(())
    }
}

fn scope_ref(scope: &str) -> Result<BindingDigest, AcpAdapterError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(ACP_SCOPE_DOMAIN)?;
    encoder.push_bytes(scope.as_bytes())?;
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn client_binding_digest(
    authenticated: AuthenticatedLocalClient,
    principal_subject: &str,
    scope_ref: BindingDigest,
) -> Result<BindingDigest, AcpAdapterError> {
    let limits = authenticated.limits();
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(ACP_CLIENT_BINDING_DOMAIN)?;
    encoder.push_u128(authenticated.client_id().0);
    encoder.push_u128(authenticated.connection_id().0);
    encoder.push_u64(authenticated.server_epoch());
    encoder.push_bytes(principal_subject.as_bytes())?;
    encoder.push_bytes(&scope_ref.bytes())?;
    encoder.push_u64(u64::from(limits.max_frame_bytes));
    encoder.push_u64(u64::from(limits.max_pending_requests));
    encoder.push_u64(u64::from(limits.max_concurrent_clients));
    Ok(BindingDigest::new(sha256(&encoder.finish())))
}

fn validate_text(
    value: &str,
    max_bytes: usize,
    error: AcpAdapterError,
) -> Result<(), AcpAdapterError> {
    if value.is_empty() || value.len() > max_bytes || value.chars().any(char::is_control) {
        Err(error)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::authority::AuthorityLayout;
    use golam_core::paths::RuntimeLayout;
    use golam_core::{PROTOCOL_VERSION, ResourceLimits};
    use golam_ipc::client_handshake::sign_authenticate;
    use golam_ipc::credentials::ClientCredentialStore;
    use golam_ipc::lifecycle::{ClientKeyId, EnrolledClientKey, Hello, ServerLifecycle};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn runtime() -> RuntimeLayout {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        RuntimeLayout::initialize(
            std::env::temp_dir().join(format!("golamd-acp-{}-{t}-{n}", std::process::id())),
        )
        .unwrap()
    }

    fn authenticated_fixture() -> (RuntimeLayout, AuthenticatedLocalClient) {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let store = ClientCredentialStore::new(&authority);
        let generated = store.generate(ClientId(7001)).unwrap();
        let signing_key = store.load(generated.client_id, generated.key_id).unwrap();
        let hello = Hello {
            protocol_version: PROTOCOL_VERSION,
            client_id: generated.client_id,
            client_nonce: [5; 32],
        };
        let mut lifecycle =
            ServerLifecycle::new(71, [7; 32], ResourceLimits::default(), ConnectionId(72)).unwrap();
        let challenge = lifecycle.receive_hello(hello).unwrap();
        let authenticate =
            sign_authenticate(hello, challenge, generated.key_id, &signing_key).unwrap();
        lifecycle
            .authenticate(
                authenticate,
                &EnrolledClientKey {
                    key_id: ClientKeyId(generated.key_id.0),
                    verifying_key: signing_key.verifying_key(),
                },
            )
            .unwrap();
        let authenticated = lifecycle.authenticated_local_client().unwrap();
        (runtime, authenticated)
    }

    #[test]
    fn authenticated_client_binds_exact_enrolled_principal_and_scope() {
        let (runtime, authenticated) = authenticated_fixture();
        let binding = AcpClientBinding::from_authenticated_local_client(
            authenticated,
            Principal::enrolled_client("local-acp", authenticated.client_id()),
            "project:read",
        )
        .unwrap();
        assert_eq!(binding.client_id(), authenticated.client_id());
        assert_eq!(binding.connection_id(), authenticated.connection_id());
        assert_eq!(binding.principal().kind, PrincipalKind::EnrolledClient);
        assert_eq!(
            binding.principal().client_id,
            Some(authenticated.client_id())
        );
        assert_eq!(binding.scope(), "project:read");
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn owner_unauthenticated_and_wrong_client_cannot_become_acp_principals() {
        let (runtime, authenticated) = authenticated_fixture();
        assert!(matches!(
            AcpClientBinding::from_authenticated_local_client(
                authenticated,
                Principal::local_owner("owner"),
                "project:read"
            ),
            Err(AcpAdapterError::PrincipalNotEnrolled)
        ));
        assert!(matches!(
            AcpClientBinding::from_authenticated_local_client(
                authenticated,
                Principal::unauthenticated("localhost"),
                "project:read"
            ),
            Err(AcpAdapterError::PrincipalNotEnrolled)
        ));
        assert!(matches!(
            AcpClientBinding::from_authenticated_local_client(
                authenticated,
                Principal::enrolled_client("local-acp", ClientId(9999)),
                "project:read"
            ),
            Err(AcpAdapterError::PrincipalClientMismatch)
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn request_scope_and_authenticated_connection_are_revalidated() {
        let (runtime, authenticated) = authenticated_fixture();
        let binding = AcpClientBinding::from_authenticated_local_client(
            authenticated,
            Principal::enrolled_client("local-acp", authenticated.client_id()),
            "project:read",
        )
        .unwrap();
        assert!(matches!(
            binding.bind_request(digest(9), digest(10)),
            Err(AcpAdapterError::ScopeMismatch)
        ));
        let mut request = binding
            .bind_request(digest(9), binding.scope_ref())
            .unwrap();
        binding.revalidate_request(&request).unwrap();
        request.connection_id = ConnectionId(request.connection_id.0 + 1);
        assert!(matches!(
            binding.revalidate_request(&request),
            Err(AcpAdapterError::ConnectionMismatch)
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
