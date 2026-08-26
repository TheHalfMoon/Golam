#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::ClientId;
use golam_ipc::credentials::{ClientCredentialStore, CredentialError, GeneratedClientCredential};
use golam_ledger::clients::{ClientKind, ClientRecord};

use crate::{
    AuthorizationContext, AuthorizationPolicy, AuthorizationRequest, ClientAuthorityError,
    KernelApi, KernelError, Principal,
};

#[derive(Debug)]
pub enum ClientEnrollmentError {
    Kernel(Box<KernelError>),
    Credential(CredentialError),
    Registry(Box<ClientAuthorityError>),
    RegistryCleanup {
        registry: Box<ClientAuthorityError>,
        cleanup: CredentialError,
    },
}

impl fmt::Display for ClientEnrollmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kernel(error) => write!(f, "client enrollment kernel error: {error}"),
            Self::Credential(error) => write!(f, "client enrollment credential error: {error}"),
            Self::Registry(error) => write!(f, "client enrollment registry error: {error}"),
            Self::RegistryCleanup { registry, cleanup } => write!(
                f,
                "client enrollment registry error: {registry}; credential cleanup also failed: {cleanup}"
            ),
        }
    }
}

impl Error for ClientEnrollmentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Kernel(error) => Some(error.as_ref()),
            Self::Credential(error) => Some(error),
            Self::Registry(error) => Some(error.as_ref()),
            Self::RegistryCleanup { registry, .. } => Some(registry.as_ref()),
        }
    }
}

impl From<KernelError> for ClientEnrollmentError {
    fn from(value: KernelError) -> Self {
        Self::Kernel(Box::new(value))
    }
}

impl From<CredentialError> for ClientEnrollmentError {
    fn from(value: CredentialError) -> Self {
        Self::Credential(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnrolledClientCredential {
    pub credential: GeneratedClientCredential,
    pub record: ClientRecord,
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn generate_and_enroll_client(
        &mut self,
        principal: Principal<'_>,
        client_id: ClientId,
        kind: ClientKind,
        enrolled_at: &str,
        scope: &str,
    ) -> Result<EnrolledClientCredential, ClientEnrollmentError> {
        let resource = format!("client:{}", client_id.0);
        self.require_authority(&AuthorizationRequest {
            principal,
            action: "client.enroll",
            resource: &resource,
            context: AuthorizationContext::local(scope),
        })?;

        let store = ClientCredentialStore::new(&self.authority);
        let generated = store.generate(client_id)?;
        match self
            .clients
            .enroll_generated(&generated, kind, principal.subject, enrolled_at)
        {
            Ok(record) => Ok(EnrolledClientCredential {
                credential: generated,
                record,
            }),
            Err(registry) => match store.remove(generated.client_id, generated.key_id) {
                Ok(()) => Err(ClientEnrollmentError::Registry(Box::new(registry))),
                Err(cleanup) => Err(ClientEnrollmentError::RegistryCleanup {
                    registry: Box::new(registry),
                    cleanup,
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BootstrapPolicy, Principal};
    use golam_core::paths::RuntimeLayout;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    fn runtime() -> RuntimeLayout {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-kernel-client-enrollment-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    #[test]
    fn generated_enrollment_is_authorized_and_registered_atomically_enough_for_bootstrap() {
        let runtime = runtime();
        let mut kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let enrolled = kernel
            .generate_and_enroll_client(
                Principal::local_owner("owner"),
                ClientId(700),
                ClientKind::Cli,
                "2026-08-26T01:20:00Z",
                "local-owner",
            )
            .unwrap();
        assert_eq!(enrolled.record.client_id, ClientId(700));
        assert_eq!(enrolled.record.owner_principal, "owner");
        assert!(enrolled.credential.path.is_file());
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn denied_enrollment_creates_no_credential_file() {
        let runtime = runtime();
        let mut kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let result = kernel.generate_and_enroll_client(
            Principal::enrolled_client("owner", ClientId(9)),
            ClientId(701),
            ClientKind::Cli,
            "2026-08-26T01:21:00Z",
            "local-client",
        );
        assert!(matches!(
            result,
            Err(ClientEnrollmentError::Kernel(error))
                if matches!(error.as_ref(), KernelError::AuthorizationDenied(_))
        ));
        let authority = golam_core::authority::AuthorityLayout::initialize(&runtime).unwrap();
        assert_eq!(fs::read_dir(authority.credential_dir()).unwrap().count(), 0);
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
