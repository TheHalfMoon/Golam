#![forbid(unsafe_code)]

use golam_core::ClientId;
use golam_ipc::lifecycle::ClientKeyId;
use golam_ledger::clients::{ClientRegistry, ClientRegistryError};

use crate::{AuthorizationPolicy, ClientAuthorityError, KernelApi, KernelError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientRegistrationStatus {
    Active,
    Unknown,
    KeyMismatch,
    Revoked,
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn client_registration_status(
        &self,
        client_id: ClientId,
        key_id: ClientKeyId,
    ) -> Result<ClientRegistrationStatus, KernelError> {
        let registry = ClientRegistry::open(&self.authority)
            .map_err(|error| KernelError::ClientAuthority(ClientAuthorityError::Registry(error)))?;
        match registry.resolve_active(client_id, key_id.0) {
            Ok(_) => Ok(ClientRegistrationStatus::Active),
            Err(ClientRegistryError::UnknownClient) => Ok(ClientRegistrationStatus::Unknown),
            Err(ClientRegistryError::ClientKeyMismatch) => {
                Ok(ClientRegistrationStatus::KeyMismatch)
            }
            Err(ClientRegistryError::RevokedClient) => Ok(ClientRegistrationStatus::Revoked),
            Err(error) => Err(KernelError::ClientAuthority(ClientAuthorityError::Registry(
                error,
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BootstrapPolicy, ClientKind, Principal};
    use golam_core::authority::AuthorityLayout;
    use golam_core::paths::RuntimeLayout;
    use golam_ipc::credentials::ClientCredentialStore;
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
            "golam-kernel-client-status-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    #[test]
    fn status_distinguishes_unknown_active_mismatch_and_revoked() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let store = ClientCredentialStore::new(&authority);
        let enrolled = store.generate(ClientId(811)).unwrap();
        let other = store.generate(ClientId(812)).unwrap();
        let mut kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();

        assert_eq!(
            kernel
                .client_registration_status(enrolled.client_id, enrolled.key_id)
                .unwrap(),
            ClientRegistrationStatus::Unknown
        );
        kernel
            .enroll_precreated_client(
                Principal::local_owner("owner"),
                enrolled.client_id,
                enrolled.key_id,
                ClientKind::Cli,
                "2026-08-26T07:10:00Z",
                "local-owner",
            )
            .unwrap();
        assert_eq!(
            kernel
                .client_registration_status(enrolled.client_id, enrolled.key_id)
                .unwrap(),
            ClientRegistrationStatus::Active
        );
        assert_eq!(
            kernel
                .client_registration_status(enrolled.client_id, other.key_id)
                .unwrap(),
            ClientRegistrationStatus::KeyMismatch
        );
        kernel
            .revoke_client(
                Principal::local_owner("owner"),
                enrolled.client_id,
                "2026-08-26T07:11:00Z",
                "local-owner",
            )
            .unwrap();
        assert_eq!(
            kernel
                .client_registration_status(enrolled.client_id, enrolled.key_id)
                .unwrap(),
            ClientRegistrationStatus::Revoked
        );
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
