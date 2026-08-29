use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::authority::AuthorityLayout;
use golam_core::taint::{TaintLabel, TaintSet};
use golam_core::{ClientId, EffectId};
use golam_ipc::credentials::ClientCredentialStore;
use golam_kernel::policy_lifecycle::{
    ActivatePolicyBundle, ApprovalMutationError, IssueApproval, PolicyMutationError,
};
use golam_kernel::{
    AuthorizationDecision, AuthorizationPolicy, AuthorizationRequest, BootstrapPolicy,
    CapabilityLeaseScope, ClientKind, DecisionId, KernelApi, KernelError, PolicyDecision, Principal,
    ProtectedResourceError,
};
use golam_ledger::approvals::ApprovalScope;
use golam_ledger::policy::PolicyBundleId;
use golam_ledger::sandbox_profile::{
    SandboxNetworkRule, SandboxProfileClass, SandboxProfileDefinition, SandboxProfileError,
    SandboxProfileStore, SandboxSpawnRule, prepare_sandbox_profile,
};
use golam_ledger::verifier_registry::{
    VerifierRegistryError, VerifierRuleKind, VerifierRuleStore, prepare_verifier_rule,
};

static N: AtomicU64 = AtomicU64::new(0);

fn runtime() -> golam_core::paths::RuntimeLayout {
    let n = N.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    golam_core::paths::RuntimeLayout::initialize(std::env::temp_dir().join(format!(
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
    drop(credential_store);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn hostile_adapter_cannot_mint_activate_or_forge_authority() {
    let runtime = runtime();
    let hostile = Principal::enrolled_client("hostile", ClientId(911));
    let mut kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();

    let lease_scope = CapabilityLeaseScope::normalize(
        &["session.create"],
        &["session:new"],
        &["scope:local-client"],
    )
    .unwrap();
    assert!(
        kernel
            .issue_capability_lease(
                "client:911:hostile",
                None,
                lease_scope,
                None,
                None,
                (DecisionId([0x11; 16]), [0x22; 16], EffectId(911)),
            )
            .is_err(),
        "fabricated decision/approval/effect evidence must not mint a capability lease"
    );

    assert!(matches!(
        kernel.activate_policy_bundle(ActivatePolicyBundle {
            principal: hostile,
            policy_bundle_id: PolicyBundleId([0x33; 16]),
            approval_id: [0x44; 16],
            activation_effect_id: EffectId(912),
            scope: "local-client",
        }),
        Err(PolicyMutationError::AuthorizationDenied(_))
    ));

    let approval_scope = ApprovalScope::once(
        EffectId(913),
        "policy.activate",
        "policy-bundle:33333333333333333333333333333333",
    )
    .unwrap();
    assert!(matches!(
        kernel.issue_approval(IssueApproval {
            principal: hostile,
            approval_scope,
            risk_class: "policy_mutation",
            taint_digest: [0x55; 32],
            issued_at: "2026-08-29T00:00:00Z",
            expires_at: None,
            max_uses: 1,
            issue_effect_id: EffectId(914),
            authorization_scope: "local-client",
        }),
        Err(ApprovalMutationError::AuthorizationDenied(_))
    ));

    drop(kernel);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn hostile_adapter_cannot_self_register_verifier_or_weaken_sandbox_profile() {
    let runtime = runtime();
    let authority = AuthorityLayout::initialize(&runtime).unwrap();

    // Even a fabricated trusted-origin taint assertion cannot substitute for
    // the exact current decision/effect/ONCE-approval registration evidence.
    let prepared_verifier = prepare_verifier_rule(
        VerifierRuleKind::DeterministicVerifier,
        1,
        b"qualification:hostile-adapter",
        TaintSet::from_labels([TaintLabel::LocalUnverified]),
        "client:911:hostile",
        TaintSet::from_labels([TaintLabel::UserTrusted]),
    )
    .unwrap();
    let mut verifier_store = VerifierRuleStore::open(&authority).unwrap();
    assert!(matches!(
        verifier_store.register(
            prepared_verifier,
            [0x61; 16],
            [0x62; 16],
            EffectId(915),
        ),
        Err(VerifierRegistryError::MissingAuthorityDecision)
    ));

    let profile_id = [0x71; 16];
    let prepared_profile = prepare_sandbox_profile(
        SandboxProfileDefinition {
            profile_id,
            version: 1,
            class: SandboxProfileClass::NativeUntrustedSubprocess,
            filesystem_read_roots: &["/"],
            filesystem_write_roots: &["/tmp"],
            network_rule: SandboxNetworkRule::PermitRequired,
            environment_allowlist: &["PATH"],
            spawn_rule: SandboxSpawnRule::ManagedDescendants,
            cpu_limit: None,
            memory_limit: None,
            time_limit: None,
            output_limit: None,
            device_allowlist: &[],
            ipc_allowlist: &[],
            inherited_handle_rules: &[],
            platform_requirements: &[],
        },
        "client:911:hostile",
        [0x72; 32],
    )
    .unwrap();
    let mut profile_store = SandboxProfileStore::open(&authority).unwrap();
    assert!(matches!(
        profile_store.register(
            prepared_profile,
            [0x73; 16],
            [0x74; 16],
            EffectId(916),
        ),
        Err(SandboxProfileError::MissingAuthorityDecision)
    ));
    assert!(matches!(
        profile_store.profile(profile_id, 1),
        Err(SandboxProfileError::ProfileNotFound)
    ));

    drop(profile_store);
    drop(verifier_store);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn hostile_adapter_cannot_bypass_strict_local_with_permissive_policy() {
    struct Permissive;

    impl AuthorizationPolicy for Permissive {
        fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
            PolicyDecision::allow("hostile_test_permit")
        }
    }

    let runtime = runtime();
    let hostile = Principal::enrolled_client("hostile", ClientId(911));
    let mut kernel = KernelApi::open(&runtime, Permissive).unwrap();
    let outcome = kernel
        .network_egress_authorize(hostile, "https://example.invalid", "local-client")
        .unwrap();
    assert_eq!(outcome.decision, AuthorizationDecision::Deny);
    assert_eq!(outcome.reason_code, "strict_local_egress_denied");
    drop(kernel);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn hostile_adapter_has_no_direct_privileged_ledger_or_plaintext_vault_surface() {
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

    let ledger_surface = fs::read_to_string(workspace.join("crates/golam-ledger/src/lib.rs")).unwrap();
    assert!(ledger_surface.contains("mod secret_vault;"));
    assert!(!ledger_surface.contains("pub mod secret_vault;"));

    let secret_interface =
        fs::read_to_string(workspace.join("crates/golam-ledger/src/secrets.rs")).unwrap();
    for forbidden_public_accessor in [
        "pub fn plaintext",
        "pub fn ciphertext",
        "pub fn secret_value",
        "pub fn value(",
    ] {
        assert!(
            !secret_interface.contains(forbidden_public_accessor),
            "public secret metadata interface exposes plaintext-bearing accessor: {forbidden_public_accessor}"
        );
    }
}
