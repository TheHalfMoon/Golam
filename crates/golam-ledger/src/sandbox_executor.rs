#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::{CanonicalEncoder, CoreError};

use crate::sandbox_enforcement::{SandboxEnforcementDescriptor, SandboxNetworkRequest};
use crate::sandbox_plan::SandboxLaunchPlan;
use crate::sandbox_profile::{SandboxProfileClass, SandboxSpawnRule};

const RESOLUTION_DOMAIN: &[u8] = b"golam:sandbox-executor-capability-resolution:v1";
const MANIFEST_DOMAIN: &[u8] = b"golam:sandbox-executor-capability-manifest:v1";
const MAX_EXECUTOR_ID_BYTES: usize = 128;
const MAX_CAPABILITIES: usize = 128;
const MAX_CAPABILITY_BYTES: usize = 512;

pub const CONTROL_ENVIRONMENT_CLEAR: &str = "golam.control.environment.clear";
pub const CONTROL_FS_READ_ROOTS: &str = "golam.control.fs.read_roots";
pub const CONTROL_FS_WRITE_ROOTS: &str = "golam.control.fs.write_roots";
pub const CONTROL_NETWORK_DENY: &str = "golam.control.network.deny";
pub const CONTROL_NETWORK_LOOPBACK: &str = "golam.control.network.loopback";
pub const CONTROL_NETWORK_PERMIT_BOUND: &str = "golam.control.network.permit_bound";
pub const CONTROL_SPAWN_DENY: &str = "golam.control.spawn.deny";
pub const CONTROL_SPAWN_DIRECT_CHILD: &str = "golam.control.spawn.direct_child";
pub const CONTROL_SPAWN_MANAGED_DESCENDANTS: &str = "golam.control.spawn.managed_descendants";
pub const CONTROL_RESOURCE_CPU: &str = "golam.control.resource.cpu";
pub const CONTROL_RESOURCE_MEMORY: &str = "golam.control.resource.memory";
pub const CONTROL_RESOURCE_TIME: &str = "golam.control.resource.time";
pub const CONTROL_RESOURCE_OUTPUT: &str = "golam.control.resource.output";
pub const CONTROL_DEVICE_ALLOWLIST: &str = "golam.control.device.allowlist";
pub const CONTROL_IPC_ALLOWLIST: &str = "golam.control.ipc.allowlist";
pub const CONTROL_HANDLE_ALLOWLIST: &str = "golam.control.handle.allowlist";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxPlatform {
    Linux,
    MacOs,
    Windows,
    Other,
}

impl SandboxPlatform {
    pub const fn current() -> Self {
        if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "macos") {
            Self::MacOs
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Other
        }
    }

    const fn code(self) -> u8 {
        match self {
            Self::Linux => 1,
            Self::MacOs => 2,
            Self::Windows => 3,
            Self::Other => 255,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformExecutorCapabilities {
    executor_id: String,
    platform: SandboxPlatform,
    supported_controls: Vec<String>,
    manifest_hash: [u8; 32],
}

impl PlatformExecutorCapabilities {
    /// Only trusted Golam executor code inside this crate may construct a capability manifest.
    /// Requesters receive the resulting manifest but cannot self-assert platform support.
    fn from_trusted_manifest(
        executor_id: &str,
        supported_controls: &[&str],
    ) -> Result<Self, SandboxExecutorError> {
        Self::from_manifest_for_platform(
            executor_id,
            SandboxPlatform::current(),
            supported_controls,
        )
    }

    fn from_manifest_for_platform(
        executor_id: &str,
        platform: SandboxPlatform,
        supported_controls: &[&str],
    ) -> Result<Self, SandboxExecutorError> {
        validate_token(executor_id, MAX_EXECUTOR_ID_BYTES)
            .map_err(|_| SandboxExecutorError::InvalidExecutorId)?;
        if supported_controls.len() > MAX_CAPABILITIES {
            return Err(SandboxExecutorError::TooManyCapabilities);
        }

        let mut controls = Vec::with_capacity(supported_controls.len());
        for control in supported_controls {
            validate_token(control, MAX_CAPABILITY_BYTES)
                .map_err(|_| SandboxExecutorError::InvalidCapability)?;
            controls.push((*control).to_owned());
        }
        controls.sort_unstable();
        if controls.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(SandboxExecutorError::DuplicateCapability);
        }

        let manifest_hash = manifest_hash(executor_id, platform, &controls)?;
        Ok(Self {
            executor_id: executor_id.to_owned(),
            platform,
            supported_controls: controls,
            manifest_hash,
        })
    }

    pub fn executor_id(&self) -> &str {
        &self.executor_id
    }

    pub const fn platform(&self) -> SandboxPlatform {
        self.platform
    }

    pub const fn manifest_hash(&self) -> [u8; 32] {
        self.manifest_hash
    }
}

/// Return the only production capability manifest admitted by T003-073.
///
/// No native executor containment control is qualified yet, so the trusted baseline
/// deliberately advertises zero controls. Any profile that requires containment therefore
/// fails closed until a later ordered task implements and qualifies concrete enforcement.
pub fn current_platform_executor_capabilities()
-> Result<PlatformExecutorCapabilities, SandboxExecutorError> {
    PlatformExecutorCapabilities::from_trusted_manifest("native:unqualified", &[])
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxExecutorCapabilityResolution {
    pub executor_id: String,
    pub platform: SandboxPlatform,
    pub source_plan_hash: [u8; 32],
    pub source_descriptor_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub required_controls: Vec<String>,
    pub resolution_hash: [u8; 32],
}

impl SandboxExecutorCapabilityResolution {
    /// T003-073 proves pre-launch capability resolution only.
    pub const fn claims_platform_containment(&self) -> bool {
        false
    }

    /// Native process launch remains ordered after T003-073.
    pub const fn launches_process(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub enum SandboxExecutorError {
    Canonical(CoreError),
    InvalidExecutorId,
    InvalidCapability,
    TooManyCapabilities,
    DuplicateCapability,
    DescriptorPlanMismatch,
    PlatformMismatch,
    WasmExecutorNotAdmitted,
    UnsupportedRequiredControl(String),
}

impl fmt::Display for SandboxExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Canonical(error) => {
                write!(f, "sandbox executor capability encoding error: {error}")
            }
            Self::InvalidExecutorId => f.write_str("sandbox executor id is not canonical"),
            Self::InvalidCapability => {
                f.write_str("sandbox executor capability token is not canonical")
            }
            Self::TooManyCapabilities => {
                f.write_str("sandbox executor capability manifest exceeds its bound")
            }
            Self::DuplicateCapability => {
                f.write_str("sandbox executor capability manifest contains a duplicate")
            }
            Self::DescriptorPlanMismatch => {
                f.write_str("sandbox enforcement descriptor does not bind the launch plan")
            }
            Self::PlatformMismatch => {
                f.write_str("sandbox executor manifest does not match the current platform")
            }
            Self::WasmExecutorNotAdmitted => {
                f.write_str("WASM/WASI executor is not admitted for Spec 003")
            }
            Self::UnsupportedRequiredControl(control) => {
                write!(f, "sandbox executor lacks required control: {control}")
            }
        }
    }
}

impl Error for SandboxExecutorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Canonical(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CoreError> for SandboxExecutorError {
    fn from(value: CoreError) -> Self {
        Self::Canonical(value)
    }
}

pub fn resolve_platform_executor_capabilities(
    plan: &SandboxLaunchPlan,
    descriptor: &SandboxEnforcementDescriptor,
    capabilities: &PlatformExecutorCapabilities,
) -> Result<SandboxExecutorCapabilityResolution, SandboxExecutorError> {
    if descriptor.source_plan_hash != plan.plan_hash {
        return Err(SandboxExecutorError::DescriptorPlanMismatch);
    }
    if capabilities.platform != SandboxPlatform::current() {
        return Err(SandboxExecutorError::PlatformMismatch);
    }
    if plan.profile_class == SandboxProfileClass::WasmWasiExtension {
        return Err(SandboxExecutorError::WasmExecutorNotAdmitted);
    }

    let required_controls = required_controls(plan, descriptor);
    for control in &required_controls {
        if capabilities
            .supported_controls
            .binary_search(control)
            .is_err()
        {
            return Err(SandboxExecutorError::UnsupportedRequiredControl(
                control.clone(),
            ));
        }
    }

    let resolution_hash = resolution_hash(plan, descriptor, capabilities, &required_controls)?;
    Ok(SandboxExecutorCapabilityResolution {
        executor_id: capabilities.executor_id.clone(),
        platform: capabilities.platform,
        source_plan_hash: plan.plan_hash,
        source_descriptor_hash: descriptor.descriptor_hash,
        manifest_hash: capabilities.manifest_hash,
        required_controls,
        resolution_hash,
    })
}

fn required_controls(
    plan: &SandboxLaunchPlan,
    descriptor: &SandboxEnforcementDescriptor,
) -> Vec<String> {
    let mut controls = Vec::with_capacity(plan.platform_requirements.len() + 16);
    controls.push(CONTROL_ENVIRONMENT_CLEAR.to_owned());

    if !descriptor.filesystem_read_roots.is_empty() {
        controls.push(CONTROL_FS_READ_ROOTS.to_owned());
    }
    if !descriptor.filesystem_write_roots.is_empty() {
        controls.push(CONTROL_FS_WRITE_ROOTS.to_owned());
    }

    controls.push(
        match descriptor.network {
            SandboxNetworkRequest::None => CONTROL_NETWORK_DENY,
            SandboxNetworkRequest::LoopbackOnly => CONTROL_NETWORK_LOOPBACK,
            SandboxNetworkRequest::PermitBoundExternal => CONTROL_NETWORK_PERMIT_BOUND,
        }
        .to_owned(),
    );
    controls.push(
        match descriptor.spawn_rule {
            SandboxSpawnRule::Deny => CONTROL_SPAWN_DENY,
            SandboxSpawnRule::DirectChildOnly => CONTROL_SPAWN_DIRECT_CHILD,
            SandboxSpawnRule::ManagedDescendants => CONTROL_SPAWN_MANAGED_DESCENDANTS,
        }
        .to_owned(),
    );

    if descriptor.cpu_limit.is_some() {
        controls.push(CONTROL_RESOURCE_CPU.to_owned());
    }
    if descriptor.memory_limit.is_some() {
        controls.push(CONTROL_RESOURCE_MEMORY.to_owned());
    }
    if descriptor.time_limit.is_some() {
        controls.push(CONTROL_RESOURCE_TIME.to_owned());
    }
    if descriptor.output_limit.is_some() {
        controls.push(CONTROL_RESOURCE_OUTPUT.to_owned());
    }
    if !descriptor.devices.is_empty() {
        controls.push(CONTROL_DEVICE_ALLOWLIST.to_owned());
    }
    if !descriptor.ipc_endpoints.is_empty() {
        controls.push(CONTROL_IPC_ALLOWLIST.to_owned());
    }
    if !descriptor.inherited_handle_rules.is_empty() {
        controls.push(CONTROL_HANDLE_ALLOWLIST.to_owned());
    }

    controls.extend(plan.platform_requirements.iter().cloned());
    controls.sort_unstable();
    controls.dedup();
    controls
}

fn manifest_hash(
    executor_id: &str,
    platform: SandboxPlatform,
    controls: &[String],
) -> Result<[u8; 32], SandboxExecutorError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(MANIFEST_DOMAIN)?;
    encoder.push_bytes(executor_id.as_bytes())?;
    encoder.push_u8(platform.code());
    encode_controls(&mut encoder, controls)?;
    Ok(crate::payload_hash(&encoder.finish()))
}

fn resolution_hash(
    plan: &SandboxLaunchPlan,
    descriptor: &SandboxEnforcementDescriptor,
    capabilities: &PlatformExecutorCapabilities,
    controls: &[String],
) -> Result<[u8; 32], SandboxExecutorError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(RESOLUTION_DOMAIN)?;
    encoder.push_bytes(&plan.plan_hash)?;
    encoder.push_bytes(&descriptor.descriptor_hash)?;
    encoder.push_bytes(&capabilities.manifest_hash)?;
    encoder.push_bytes(capabilities.executor_id.as_bytes())?;
    encoder.push_u8(capabilities.platform.code());
    encode_controls(&mut encoder, controls)?;
    Ok(crate::payload_hash(&encoder.finish()))
}

fn encode_controls(
    encoder: &mut CanonicalEncoder,
    controls: &[String],
) -> Result<(), SandboxExecutorError> {
    encoder.push_u64(controls.len() as u64);
    for control in controls {
        encoder.push_bytes(control.as_bytes())?;
    }
    Ok(())
}

fn validate_token(value: &str, max_bytes: usize) -> Result<(), ()> {
    if value.is_empty()
        || value.len() > max_bytes
        || value.trim() != value
        || value.chars().any(char::is_control)
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'_' | b'-' | b'.' | b':' | b'/' | b'@' | b'*')
        })
    {
        return Err(());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox_enforcement::{SandboxRequestedRights, resolve_sandbox_enforcement};
    use crate::sandbox_plan::{SandboxEgressBinding, SandboxLocality};
    use crate::sandbox_profile::{SandboxNetworkRule, SandboxProfileClass};

    fn plan() -> SandboxLaunchPlan {
        SandboxLaunchPlan {
            profile_id: [1; 16],
            profile_version: 1,
            profile_class: SandboxProfileClass::NativeUntrustedSubprocess,
            filesystem_read_roots: vec!["/workspace/input".into()],
            filesystem_write_roots: vec!["/workspace/output".into()],
            network_rule: SandboxNetworkRule::PermitRequired,
            environment_allowlist: vec!["LANG".into()],
            spawn_rule: SandboxSpawnRule::DirectChildOnly,
            cpu_limit: Some(10),
            memory_limit: Some(512),
            time_limit: Some(30),
            output_limit: Some(4096),
            device_allowlist: vec!["device:null".into()],
            ipc_allowlist: vec!["ipc:fixture".into()],
            inherited_handle_rules: vec!["handle:fixture".into()],
            platform_requirements: vec!["platform:test-control".into()],
            principal_or_process: "principal:test".into(),
            decision_id: [2; 16],
            lease_id: [3; 16],
            lease_generation: 1,
            policy_bundle_id: [4; 16],
            policy_bundle_hash: [5; 32],
            egress: Some(SandboxEgressBinding {
                permit_id: [6; 16],
                action: "network.connect".into(),
                purpose: "fixture".into(),
                destination_scope: "example.test".into(),
                protocol_port_scope: "tcp:443".into(),
                taint_digest: [7; 32],
                secret_handle_id: None,
                expires_at: None,
                usage_limit: Some(1),
            }),
            locality: SandboxLocality::NonStrict,
            observed_at: "2026-08-29T00:00:00Z".into(),
            plan_hash: [8; 32],
        }
    }

    fn descriptor(plan: &SandboxLaunchPlan) -> SandboxEnforcementDescriptor {
        resolve_sandbox_enforcement(
            plan,
            SandboxRequestedRights {
                filesystem_read_roots: &["/workspace/input"],
                filesystem_write_roots: &["/workspace/output"],
                environment_names: &["LANG"],
                network: SandboxNetworkRequest::PermitBoundExternal,
                spawn_rule: SandboxSpawnRule::DirectChildOnly,
                cpu_limit: Some(5),
                memory_limit: Some(256),
                time_limit: Some(20),
                output_limit: Some(2048),
                devices: &["device:null"],
                ipc_endpoints: &["ipc:fixture"],
                inherited_handle_rules: &["handle:fixture"],
            },
        )
        .expect("descriptor")
    }

    fn all_controls() -> Vec<&'static str> {
        vec![
            CONTROL_ENVIRONMENT_CLEAR,
            CONTROL_FS_READ_ROOTS,
            CONTROL_FS_WRITE_ROOTS,
            CONTROL_NETWORK_PERMIT_BOUND,
            CONTROL_SPAWN_DIRECT_CHILD,
            CONTROL_RESOURCE_CPU,
            CONTROL_RESOURCE_MEMORY,
            CONTROL_RESOURCE_TIME,
            CONTROL_RESOURCE_OUTPUT,
            CONTROL_DEVICE_ALLOWLIST,
            CONTROL_IPC_ALLOWLIST,
            CONTROL_HANDLE_ALLOWLIST,
            "platform:test-control",
        ]
    }

    #[test]
    fn exact_manifest_resolves_without_claiming_containment_or_launch() {
        let plan = plan();
        let descriptor = descriptor(&plan);
        let capabilities = PlatformExecutorCapabilities::from_trusted_manifest(
            "native:test-executor",
            &all_controls(),
        )
        .expect("manifest");
        let resolution = resolve_platform_executor_capabilities(&plan, &descriptor, &capabilities)
            .expect("resolution");
        assert_eq!(resolution.executor_id, "native:test-executor");
        assert_eq!(resolution.platform, SandboxPlatform::current());
        assert!(!resolution.claims_platform_containment());
        assert!(!resolution.launches_process());
        assert_eq!(resolution.source_plan_hash, plan.plan_hash);
        assert_eq!(
            resolution.source_descriptor_hash,
            descriptor.descriptor_hash
        );
    }

    #[test]
    fn missing_derived_control_fails_closed() {
        let plan = plan();
        let descriptor = descriptor(&plan);
        let controls: Vec<_> = all_controls()
            .into_iter()
            .filter(|control| *control != CONTROL_ENVIRONMENT_CLEAR)
            .collect();
        let capabilities =
            PlatformExecutorCapabilities::from_trusted_manifest("native:test-executor", &controls)
                .expect("manifest");
        assert!(matches!(
            resolve_platform_executor_capabilities(&plan, &descriptor, &capabilities),
            Err(SandboxExecutorError::UnsupportedRequiredControl(control))
                if control == CONTROL_ENVIRONMENT_CLEAR
        ));
    }

    #[test]
    fn missing_profile_requirement_fails_closed() {
        let plan = plan();
        let descriptor = descriptor(&plan);
        let controls: Vec<_> = all_controls()
            .into_iter()
            .filter(|control| *control != "platform:test-control")
            .collect();
        let capabilities =
            PlatformExecutorCapabilities::from_trusted_manifest("native:test-executor", &controls)
                .expect("manifest");
        assert!(matches!(
            resolve_platform_executor_capabilities(&plan, &descriptor, &capabilities),
            Err(SandboxExecutorError::UnsupportedRequiredControl(control))
                if control == "platform:test-control"
        ));
    }

    #[test]
    fn descriptor_from_another_plan_is_denied() {
        let plan = plan();
        let mut other = plan.clone();
        other.plan_hash = [9; 32];
        let descriptor = descriptor(&other);
        let capabilities = PlatformExecutorCapabilities::from_trusted_manifest(
            "native:test-executor",
            &all_controls(),
        )
        .expect("manifest");
        assert!(matches!(
            resolve_platform_executor_capabilities(&plan, &descriptor, &capabilities),
            Err(SandboxExecutorError::DescriptorPlanMismatch)
        ));
    }

    #[test]
    fn wasm_executor_remains_not_admitted() {
        let mut plan = plan();
        plan.profile_class = SandboxProfileClass::WasmWasiExtension;
        let descriptor = descriptor(&plan);
        let capabilities = PlatformExecutorCapabilities::from_trusted_manifest(
            "native:test-executor",
            &all_controls(),
        )
        .expect("manifest");
        assert!(matches!(
            resolve_platform_executor_capabilities(&plan, &descriptor, &capabilities),
            Err(SandboxExecutorError::WasmExecutorNotAdmitted)
        ));
    }

    #[test]
    fn non_current_platform_manifest_is_denied() {
        let plan = plan();
        let descriptor = descriptor(&plan);
        let other = match SandboxPlatform::current() {
            SandboxPlatform::Linux => SandboxPlatform::Windows,
            SandboxPlatform::Windows => SandboxPlatform::MacOs,
            SandboxPlatform::MacOs | SandboxPlatform::Other => SandboxPlatform::Linux,
        };
        let capabilities = PlatformExecutorCapabilities::from_manifest_for_platform(
            "native:test-executor",
            other,
            &all_controls(),
        )
        .expect("manifest");
        assert!(matches!(
            resolve_platform_executor_capabilities(&plan, &descriptor, &capabilities),
            Err(SandboxExecutorError::PlatformMismatch)
        ));
    }

    #[test]
    fn production_baseline_advertises_no_unqualified_containment() {
        let plan = plan();
        let descriptor = descriptor(&plan);
        let capabilities = current_platform_executor_capabilities().expect("baseline manifest");
        assert_eq!(capabilities.executor_id(), "native:unqualified");
        assert_eq!(capabilities.platform(), SandboxPlatform::current());
        assert!(matches!(
            resolve_platform_executor_capabilities(&plan, &descriptor, &capabilities),
            Err(SandboxExecutorError::UnsupportedRequiredControl(_))
        ));
    }

    #[test]
    fn capability_manifest_is_canonical_and_duplicate_free() {
        assert!(matches!(
            PlatformExecutorCapabilities::from_trusted_manifest(
                "native:test-executor",
                &[CONTROL_ENVIRONMENT_CLEAR, CONTROL_ENVIRONMENT_CLEAR],
            ),
            Err(SandboxExecutorError::DuplicateCapability)
        ));
        assert!(matches!(
            PlatformExecutorCapabilities::from_trusted_manifest(
                "native test executor",
                &[CONTROL_ENVIRONMENT_CLEAR],
            ),
            Err(SandboxExecutorError::InvalidExecutorId)
        ));
    }

    #[test]
    fn deny_network_and_spawn_are_still_required_controls() {
        let mut plan = plan();
        plan.network_rule = SandboxNetworkRule::DenyAll;
        plan.egress = None;
        plan.spawn_rule = SandboxSpawnRule::Deny;
        let descriptor = resolve_sandbox_enforcement(&plan, SandboxRequestedRights::deny_all())
            .expect("deny-all descriptor");
        let controls = required_controls(&plan, &descriptor);
        assert!(
            controls
                .iter()
                .any(|control| control == CONTROL_NETWORK_DENY)
        );
        assert!(controls.iter().any(|control| control == CONTROL_SPAWN_DENY));
        assert!(
            controls
                .iter()
                .any(|control| control == CONTROL_ENVIRONMENT_CLEAR)
        );
    }
}
