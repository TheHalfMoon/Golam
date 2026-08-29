#![forbid(unsafe_code)]

use crate::sandbox_enforcement::{
    SandboxNetworkRequest, SandboxRequestedRights, resolve_sandbox_enforcement,
};
use crate::sandbox_executor::{
    SandboxExecutorError, current_platform_executor_capabilities,
    resolve_platform_executor_capabilities,
};
use crate::sandbox_plan::{SandboxLaunchPlan, SandboxLocality};
use crate::sandbox_profile::{SandboxNetworkRule, SandboxProfileClass, SandboxSpawnRule};

pub(crate) const QUALIFICATION_PLATFORM_REQUIREMENT: &str =
    "platform:linux-x86_64-bwrap-seccomp-test-v1";

fn minimum_native_test_plan() -> SandboxLaunchPlan {
    SandboxLaunchPlan {
        profile_id: [0x74; 16],
        profile_version: 1,
        profile_class: SandboxProfileClass::NativeUntrustedSubprocess,
        filesystem_read_roots: vec![
            "/bin".into(),
            "/lib".into(),
            "/lib64".into(),
            "/usr".into(),
        ],
        filesystem_write_roots: vec!["/tmp".into()],
        network_rule: SandboxNetworkRule::DenyAll,
        environment_allowlist: vec![],
        spawn_rule: SandboxSpawnRule::ManagedDescendants,
        cpu_limit: None,
        memory_limit: None,
        time_limit: None,
        output_limit: None,
        device_allowlist: vec!["device:null".into()],
        ipc_allowlist: vec![],
        inherited_handle_rules: vec![],
        platform_requirements: vec![QUALIFICATION_PLATFORM_REQUIREMENT.into()],
        principal_or_process: "process:t003-074-native-test".into(),
        decision_id: [0x21; 16],
        lease_id: [0x22; 16],
        lease_generation: 1,
        policy_bundle_id: [0x23; 16],
        policy_bundle_hash: [0x24; 32],
        egress: None,
        locality: SandboxLocality::StrictLocal,
        observed_at: "2026-08-29T00:00:00Z".into(),
        plan_hash: [0x25; 32],
    }
}

fn requested_rights<'a>() -> SandboxRequestedRights<'a> {
    SandboxRequestedRights {
        filesystem_read_roots: &["/bin", "/lib", "/lib64", "/usr"],
        filesystem_write_roots: &["/tmp"],
        environment_names: &[],
        network: SandboxNetworkRequest::None,
        spawn_rule: SandboxSpawnRule::ManagedDescendants,
        cpu_limit: None,
        memory_limit: None,
        time_limit: None,
        output_limit: None,
        devices: &["device:null"],
        ipc_endpoints: &[],
        inherited_handle_rules: &[],
    }
}

#[test]
fn minimum_native_test_profile_is_exactly_bounded_to_the_qualified_harness() {
    let descriptor =
        resolve_sandbox_enforcement(&minimum_native_test_plan(), requested_rights())
            .expect("minimum native qualification descriptor");

    assert!(descriptor.clears_environment());
    assert!(!descriptor.claims_platform_containment());
    assert_eq!(
        descriptor.filesystem_read_roots,
        ["/bin", "/lib", "/lib64", "/usr"]
    );
    assert_eq!(descriptor.filesystem_write_roots, ["/tmp"]);
    assert!(descriptor.environment_names.is_empty());
    assert_eq!(descriptor.network, SandboxNetworkRequest::None);
    assert_eq!(descriptor.spawn_rule, SandboxSpawnRule::ManagedDescendants);
    assert_eq!(descriptor.devices, ["device:null"]);
    assert!(descriptor.ipc_endpoints.is_empty());
    assert!(descriptor.inherited_handle_rules.is_empty());
}

#[test]
fn qualification_profile_does_not_admit_a_production_native_executor() {
    let plan = minimum_native_test_plan();
    let descriptor = resolve_sandbox_enforcement(&plan, requested_rights())
        .expect("minimum native qualification descriptor");
    let capabilities =
        current_platform_executor_capabilities().expect("production capability baseline");

    assert_eq!(capabilities.executor_id(), "native:unqualified");
    assert!(matches!(
        resolve_platform_executor_capabilities(&plan, &descriptor, &capabilities),
        Err(SandboxExecutorError::UnsupportedRequiredControl(_))
    ));
}
