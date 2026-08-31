use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;
use golam_core::taint::{TaintLabel, TaintSet};
use golam_core::{ClientId, EffectId};
use golam_ipc::credentials::ClientCredentialStore;
use golam_kernel::policy_lifecycle::{
    ActivatePolicyBundle, ApprovalMutationError, IssueApproval, PolicyMutationError,
};
use golam_kernel::{
    AuthorizationContext, AuthorizationDecision, AuthorizationPolicy, AuthorizationRequest,
    BootstrapPolicy, CapabilityLeaseScope, ClientKind, DecisionId, DenyByDefault, KernelApi,
    KernelError, PolicyDecision, Principal, ProtectedResourceError,
};
use golam_ledger::approvals::ApprovalScope;
use golam_ledger::capability_leases::CapabilityLeaseMutationError;
use golam_ledger::policy::PolicyBundleId;
use golam_ledger::sandbox_profile::{
    SandboxNetworkRule, SandboxProfileClass, SandboxProfileDefinition, SandboxProfileError,
    SandboxProfileStore, SandboxSpawnRule, prepare_sandbox_profile,
};
use golam_ledger::verifier_registry::{
    VerifierRegistryError, VerifierRuleKind, VerifierRuleStore, prepare_verifier_rule,
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
fn hostile_adapter_cannot_mint_capability_with_fabricated_evidence() {
    let runtime = runtime();
    let mut kernel = KernelApi::open(&runtime, DenyByDefault).unwrap();
    let scope = CapabilityLeaseScope::normalize(
        &["session.read"],
        &["session:hostile-target"],
        &["local-client"],
    )
    .unwrap();

    let result = kernel.issue_capability_lease(
        "client:911:hostile",
        None,
        scope,
        None,
        None,
        (DecisionId([0x11; 16]), [0x22; 16], EffectId(0x33)),
    );
    assert!(matches!(
        result,
        Err(CapabilityLeaseMutationError::MissingAuthorityDecision)
    ));

    drop(kernel);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn hostile_adapter_cannot_activate_policy_or_forge_approval() {
    let runtime = runtime();
    let hostile = Principal::enrolled_client("hostile", ClientId(911));
    let mut kernel = KernelApi::open(&runtime, DenyByDefault).unwrap();

    let activation = kernel.activate_policy_bundle(ActivatePolicyBundle {
        principal: hostile,
        policy_bundle_id: PolicyBundleId([0x44; 16]),
        approval_id: [0x45; 16],
        activation_effect_id: EffectId(0x46),
        scope: "local-client",
    });
    assert!(matches!(
        activation,
        Err(PolicyMutationError::AuthorizationDenied(_))
    ));

    let approval_scope = ApprovalScope::once(
        EffectId(0x50),
        "sandbox.profile.register",
        "sandbox-profile:hostile",
    )
    .unwrap();
    let forged = kernel.issue_approval(IssueApproval {
        principal: hostile,
        approval_scope,
        risk_class: "sandbox_profile_mutation",
        taint_digest: [0; 32],
        issued_at: "2026-08-30T10:00:00Z",
        expires_at: None,
        max_uses: 1,
        issue_effect_id: EffectId(0x51),
        authorization_scope: "local-client",
    });
    assert!(matches!(
        forged,
        Err(ApprovalMutationError::AuthorizationDenied(_))
    ));

    drop(kernel);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn permissive_adapter_policy_cannot_bypass_strict_local_hard_guard() {
    struct Permissive;

    impl AuthorizationPolicy for Permissive {
        fn authorize(&self, _request: &AuthorizationRequest<'_>) -> PolicyDecision {
            PolicyDecision::allow("hostile_permissive_policy")
        }
    }

    let runtime = runtime();
    let mut kernel = KernelApi::open(&runtime, Permissive).unwrap();
    let outcome = kernel
        .authorize(&AuthorizationRequest {
            principal: Principal::enrolled_client("hostile", ClientId(911)),
            action: "network.egress",
            resource: "https://example.invalid",
            context: AuthorizationContext::local("local-client"),
        })
        .unwrap();
    assert_eq!(outcome.decision, AuthorizationDecision::Deny);
    assert_eq!(outcome.reason_code, "strict_local_egress_denied");

    drop(kernel);
    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn fabricated_verifier_and_weakened_profile_evidence_fail_closed() {
    let runtime = runtime();
    let authority = AuthorityLayout::initialize(&runtime).unwrap();

    let prepared_verifier = prepare_verifier_rule(
        VerifierRuleKind::DeterministicVerifier,
        1,
        b"hostile-self-asserted-source",
        TaintSet::from_labels([TaintLabel::WebUntrusted]),
        "client:911:hostile",
        TaintSet::from_labels([TaintLabel::UserTrusted]),
    )
    .unwrap();
    let mut verifiers = VerifierRuleStore::open(&authority).unwrap();
    assert!(matches!(
        verifiers.register(prepared_verifier, [0x61; 16], [0x62; 16], EffectId(0x63),),
        Err(VerifierRegistryError::MissingAuthorityDecision)
    ));
    drop(verifiers);

    let prepared_profile = prepare_sandbox_profile(
        SandboxProfileDefinition {
            profile_id: [0x71; 16],
            version: 1,
            class: SandboxProfileClass::NativeUntrustedSubprocess,
            filesystem_read_roots: &[],
            filesystem_write_roots: &[],
            network_rule: SandboxNetworkRule::PermitRequired,
            environment_allowlist: &[],
            spawn_rule: SandboxSpawnRule::ManagedDescendants,
            cpu_limit: Some(1),
            memory_limit: Some(1),
            time_limit: Some(1),
            output_limit: Some(1),
            device_allowlist: &[],
            ipc_allowlist: &[],
            inherited_handle_rules: &[],
            platform_requirements: &[],
        },
        "client:911:hostile",
        [0; 32],
    )
    .unwrap();
    let mut profiles = SandboxProfileStore::open(&authority).unwrap();
    assert!(matches!(
        profiles.register(prepared_profile, [0x72; 16], [0x73; 16], EffectId(0x74),),
        Err(SandboxProfileError::MissingAuthorityDecision)
    ));
    drop(profiles);

    fs::remove_dir_all(runtime.root).unwrap();
}

#[test]
fn hostile_adapter_has_no_vault_plaintext_surface_or_direct_privileged_ledger_dependency() {
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

    let ledger_root = fs::read_to_string(workspace.join("crates/golam-ledger/src/lib.rs")).unwrap();
    for private_boundary in [
        "pub(crate) mod secret_broker;",
        "pub(crate) mod secret_entry;",
        "pub(crate) mod secret_fallback;",
        "mod secret_vault;",
        "pub(crate) mod secret_mutation;",
    ] {
        assert!(
            ledger_root.contains(private_boundary),
            "plaintext-bearing secret boundary became externally public: {private_boundary}"
        );
    }

    let public_secret_interface =
        fs::read_to_string(workspace.join("crates/golam-ledger/src/secrets.rs")).unwrap();
    assert!(!public_secret_interface.contains("pub fn plaintext"));
    assert!(!public_secret_interface.contains("pub fn ciphertext"));
    assert!(!public_secret_interface.contains("SELECT ciphertext"));
}
