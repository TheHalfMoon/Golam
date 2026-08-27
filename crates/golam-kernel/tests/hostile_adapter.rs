use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::ClientId;
use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;
use golam_ipc::credentials::ClientCredentialStore;
use golam_kernel::{
    AuthorizationDecision, BootstrapPolicy, ClientKind, KernelApi, KernelError, Principal,
    ProtectedResourceError,
};

static N: AtomicU64 = AtomicU64::new(0);

fn runtime() -> RuntimeLayout {
    let n = N.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    RuntimeLayout::initialize(std::env::temp_dir().join(format!(
        "golam-hostile-adapter-{}-{t}-{n}",
        std::process::id()
    )))
    .unwrap()
}

#[test]
fn hostile_adapter_cannot_cross_kernel_authority_boundary() {
    let runtime = runtime();
    let authority = AuthorityLayout::initialize(&runtime).unwrap();
    let credential_store = ClientCredentialStore::new(&authority);
    let generated = credential_store.generate(ClientId(910)).unwrap();
    let hostile = Principal::enrolled_client("hostile", ClientId(911));
    let mut kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();

    assert!(matches!(
        kernel.enroll_generated_client(
            hostile,
            &generated,
            ClientKind::Test,
            "2026-08-25T08:02:00Z",
            "local-client",
        ),
        Err(KernelError::AuthorizationDenied(_))
    ));
    assert!(matches!(
        kernel.revoke_client(
            hostile,
            generated.client_id,
            "2026-08-25T08:03:00Z",
            "local-client",
        ),
        Err(KernelError::AuthorizationDenied(_))
    ));
    assert!(matches!(
        kernel.admit_unprivileged_path(authority.authority_db_path()),
        Err(ProtectedResourceError::AuthorityReserved(_))
    ));

    let egress = kernel
        .network_egress_authorize(hostile, "https://example.invalid", "local-client")
        .unwrap();
    assert_eq!(egress.decision, AuthorizationDecision::Deny);

    drop(kernel);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn hostile_adapter_has_no_direct_privileged_ledger_dependency() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("kernel crate lives under workspace/crates");

    for manifest in [
        "crates/golam-core/Cargo.toml",
        "crates/golam-effects/Cargo.toml",
        "crates/golam-ipc/Cargo.toml",
        "crates/golamd/Cargo.toml",
        "crates/golam/Cargo.toml",
    ] {
        let text = fs::read_to_string(workspace.join(manifest)).unwrap();
        assert!(
            !text.contains("golam-ledger"),
            "non-kernel product crate links privileged ledger directly: {manifest}"
        );
    }
}
