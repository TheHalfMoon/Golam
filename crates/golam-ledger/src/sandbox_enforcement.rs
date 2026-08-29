#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::{CanonicalEncoder, CoreError};

use crate::sandbox_plan::{SandboxLaunchPlan, SandboxLocality};
use crate::sandbox_profile::{SandboxNetworkRule, SandboxSpawnRule};

const DESCRIPTOR_DOMAIN: &[u8] = b"golam:sandbox-enforcement-descriptor:v1";
const MAX_REQUEST_ITEMS: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxNetworkRequest {
    None,
    LoopbackOnly,
    PermitBoundExternal,
}

impl SandboxNetworkRequest {
    const fn code(self) -> u8 {
        match self {
            Self::None => 1,
            Self::LoopbackOnly => 2,
            Self::PermitBoundExternal => 3,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SandboxRequestedRights<'a> {
    pub filesystem_read_roots: &'a [&'a str],
    pub filesystem_write_roots: &'a [&'a str],
    pub environment_names: &'a [&'a str],
    pub network: SandboxNetworkRequest,
    pub spawn_rule: SandboxSpawnRule,
    pub cpu_limit: Option<u64>,
    pub memory_limit: Option<u64>,
    pub time_limit: Option<u64>,
    pub output_limit: Option<u64>,
    pub devices: &'a [&'a str],
    pub ipc_endpoints: &'a [&'a str],
    pub inherited_handle_rules: &'a [&'a str],
}

impl SandboxRequestedRights<'_> {
    pub const fn deny_all() -> Self {
        Self {
            filesystem_read_roots: &[],
            filesystem_write_roots: &[],
            environment_names: &[],
            network: SandboxNetworkRequest::None,
            spawn_rule: SandboxSpawnRule::Deny,
            cpu_limit: None,
            memory_limit: None,
            time_limit: None,
            output_limit: None,
            devices: &[],
            ipc_endpoints: &[],
            inherited_handle_rules: &[],
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxEnforcementDescriptor {
    pub source_plan_hash: [u8; 32],
    pub filesystem_read_roots: Vec<String>,
    pub filesystem_write_roots: Vec<String>,
    pub environment_names: Vec<String>,
    pub network: SandboxNetworkRequest,
    pub spawn_rule: SandboxSpawnRule,
    pub cpu_limit: Option<u64>,
    pub memory_limit: Option<u64>,
    pub time_limit: Option<u64>,
    pub output_limit: Option<u64>,
    pub devices: Vec<String>,
    pub ipc_endpoints: Vec<String>,
    pub inherited_handle_rules: Vec<String>,
    pub descriptor_hash: [u8; 32],
}

impl SandboxEnforcementDescriptor {
    /// Golam-managed launches start from an empty environment. This descriptor carries only
    /// explicitly requested names that were already declared by the protected profile.
    pub const fn clears_environment(&self) -> bool {
        true
    }

    /// This task resolves bounded rights only. Platform containment is qualified separately.
    pub const fn claims_platform_containment(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub enum SandboxEnforcementError {
    Canonical(CoreError),
    TooManyRequestedItems,
    UndeclaredFilesystemRead,
    UndeclaredFilesystemWrite,
    UndeclaredEnvironmentName,
    NetworkWidening,
    SpawnWidening,
    InvalidResourceLimit,
    UndeclaredDevice,
    UndeclaredIpcEndpoint,
    UndeclaredInheritedHandleRule,
}

impl fmt::Display for SandboxEnforcementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Canonical(error) => write!(f, "sandbox enforcement encoding error: {error}"),
            Self::TooManyRequestedItems => {
                f.write_str("sandbox enforcement request exceeds bounded item count")
            }
            Self::UndeclaredFilesystemRead => {
                f.write_str("sandbox enforcement request widens filesystem read roots")
            }
            Self::UndeclaredFilesystemWrite => {
                f.write_str("sandbox enforcement request widens filesystem write roots")
            }
            Self::UndeclaredEnvironmentName => {
                f.write_str("sandbox enforcement request widens environment allowlist")
            }
            Self::NetworkWidening => {
                f.write_str("sandbox enforcement request widens network authority")
            }
            Self::SpawnWidening => {
                f.write_str("sandbox enforcement request widens process spawning authority")
            }
            Self::InvalidResourceLimit => f.write_str("sandbox enforcement resource limit is zero"),
            Self::UndeclaredDevice => {
                f.write_str("sandbox enforcement request widens device authority")
            }
            Self::UndeclaredIpcEndpoint => {
                f.write_str("sandbox enforcement request widens IPC authority")
            }
            Self::UndeclaredInheritedHandleRule => {
                f.write_str("sandbox enforcement request widens inherited handle authority")
            }
        }
    }
}

impl Error for SandboxEnforcementError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Canonical(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CoreError> for SandboxEnforcementError {
    fn from(value: CoreError) -> Self {
        Self::Canonical(value)
    }
}

pub fn resolve_sandbox_enforcement(
    plan: &SandboxLaunchPlan,
    requested: SandboxRequestedRights<'_>,
) -> Result<SandboxEnforcementDescriptor, SandboxEnforcementError> {
    let filesystem_read_roots = exact_subset(
        requested.filesystem_read_roots,
        &plan.filesystem_read_roots,
        SandboxEnforcementError::UndeclaredFilesystemRead,
    )?;
    let filesystem_write_roots = exact_subset(
        requested.filesystem_write_roots,
        &plan.filesystem_write_roots,
        SandboxEnforcementError::UndeclaredFilesystemWrite,
    )?;
    let environment_names = exact_subset(
        requested.environment_names,
        &plan.environment_allowlist,
        SandboxEnforcementError::UndeclaredEnvironmentName,
    )?;
    let devices = exact_subset(
        requested.devices,
        &plan.device_allowlist,
        SandboxEnforcementError::UndeclaredDevice,
    )?;
    let ipc_endpoints = exact_subset(
        requested.ipc_endpoints,
        &plan.ipc_allowlist,
        SandboxEnforcementError::UndeclaredIpcEndpoint,
    )?;
    let inherited_handle_rules = exact_subset(
        requested.inherited_handle_rules,
        &plan.inherited_handle_rules,
        SandboxEnforcementError::UndeclaredInheritedHandleRule,
    )?;

    validate_network(plan, requested.network)?;
    validate_spawn(plan.spawn_rule, requested.spawn_rule)?;

    let cpu_limit = narrower_limit(plan.cpu_limit, requested.cpu_limit)?;
    let memory_limit = narrower_limit(plan.memory_limit, requested.memory_limit)?;
    let time_limit = narrower_limit(plan.time_limit, requested.time_limit)?;
    let output_limit = narrower_limit(plan.output_limit, requested.output_limit)?;

    let descriptor_hash = descriptor_hash(
        plan,
        &filesystem_read_roots,
        &filesystem_write_roots,
        &environment_names,
        requested.network,
        requested.spawn_rule,
        cpu_limit,
        memory_limit,
        time_limit,
        output_limit,
        &devices,
        &ipc_endpoints,
        &inherited_handle_rules,
    )?;

    Ok(SandboxEnforcementDescriptor {
        source_plan_hash: plan.plan_hash,
        filesystem_read_roots,
        filesystem_write_roots,
        environment_names,
        network: requested.network,
        spawn_rule: requested.spawn_rule,
        cpu_limit,
        memory_limit,
        time_limit,
        output_limit,
        devices,
        ipc_endpoints,
        inherited_handle_rules,
        descriptor_hash,
    })
}

fn exact_subset(
    requested: &[&str],
    allowed: &[String],
    error: SandboxEnforcementError,
) -> Result<Vec<String>, SandboxEnforcementError> {
    if requested.len() > MAX_REQUEST_ITEMS {
        return Err(SandboxEnforcementError::TooManyRequestedItems);
    }
    let mut canonical = Vec::with_capacity(requested.len());
    for value in requested {
        if !allowed.iter().any(|allowed_value| allowed_value == value) {
            return Err(error);
        }
        canonical.push((*value).to_owned());
    }
    canonical.sort();
    canonical.dedup();
    Ok(canonical)
}

fn validate_network(
    plan: &SandboxLaunchPlan,
    requested: SandboxNetworkRequest,
) -> Result<(), SandboxEnforcementError> {
    let allowed = match requested {
        SandboxNetworkRequest::None => true,
        SandboxNetworkRequest::LoopbackOnly => {
            plan.network_rule == SandboxNetworkRule::LoopbackOnly
        }
        SandboxNetworkRequest::PermitBoundExternal => {
            plan.network_rule == SandboxNetworkRule::PermitRequired
                && plan.locality == SandboxLocality::NonStrict
                && plan.egress.is_some()
        }
    };
    if allowed {
        Ok(())
    } else {
        Err(SandboxEnforcementError::NetworkWidening)
    }
}

fn validate_spawn(
    allowed: SandboxSpawnRule,
    requested: SandboxSpawnRule,
) -> Result<(), SandboxEnforcementError> {
    let allowed = matches!(
        (allowed, requested),
        (SandboxSpawnRule::Deny, SandboxSpawnRule::Deny)
            | (SandboxSpawnRule::DirectChildOnly, SandboxSpawnRule::Deny)
            | (
                SandboxSpawnRule::DirectChildOnly,
                SandboxSpawnRule::DirectChildOnly
            )
            | (SandboxSpawnRule::ManagedDescendants, SandboxSpawnRule::Deny)
            | (
                SandboxSpawnRule::ManagedDescendants,
                SandboxSpawnRule::DirectChildOnly
            )
            | (
                SandboxSpawnRule::ManagedDescendants,
                SandboxSpawnRule::ManagedDescendants
            )
    );
    if allowed {
        Ok(())
    } else {
        Err(SandboxEnforcementError::SpawnWidening)
    }
}

fn narrower_limit(
    profile_limit: Option<u64>,
    requested_limit: Option<u64>,
) -> Result<Option<u64>, SandboxEnforcementError> {
    if requested_limit == Some(0) {
        return Err(SandboxEnforcementError::InvalidResourceLimit);
    }
    Ok(match (profile_limit, requested_limit) {
        (Some(profile), Some(requested)) => Some(profile.min(requested)),
        (Some(profile), None) => Some(profile),
        (None, Some(requested)) => Some(requested),
        (None, None) => None,
    })
}

#[allow(clippy::too_many_arguments)]
fn descriptor_hash(
    plan: &SandboxLaunchPlan,
    read_roots: &[String],
    write_roots: &[String],
    environment_names: &[String],
    network: SandboxNetworkRequest,
    spawn_rule: SandboxSpawnRule,
    cpu_limit: Option<u64>,
    memory_limit: Option<u64>,
    time_limit: Option<u64>,
    output_limit: Option<u64>,
    devices: &[String],
    ipc_endpoints: &[String],
    inherited_handle_rules: &[String],
) -> Result<[u8; 32], SandboxEnforcementError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(DESCRIPTOR_DOMAIN)?;
    encoder.push_bytes(&plan.plan_hash)?;
    encoder.push_u8(1); // cleared environment is mandatory
    encode_strings(&mut encoder, read_roots)?;
    encode_strings(&mut encoder, write_roots)?;
    encode_strings(&mut encoder, environment_names)?;
    encoder.push_u8(network.code());
    encoder.push_bytes(spawn_rule.as_str().as_bytes())?;
    encode_limit(&mut encoder, cpu_limit);
    encode_limit(&mut encoder, memory_limit);
    encode_limit(&mut encoder, time_limit);
    encode_limit(&mut encoder, output_limit);
    encode_strings(&mut encoder, devices)?;
    encode_strings(&mut encoder, ipc_endpoints)?;
    encode_strings(&mut encoder, inherited_handle_rules)?;
    Ok(crate::payload_hash(&encoder.finish()))
}

fn encode_strings(
    encoder: &mut CanonicalEncoder,
    values: &[String],
) -> Result<(), SandboxEnforcementError> {
    encoder.push_u64(values.len() as u64);
    for value in values {
        encoder.push_bytes(value.as_bytes())?;
    }
    Ok(())
}

fn encode_limit(encoder: &mut CanonicalEncoder, value: Option<u64>) {
    match value {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_u64(value);
        }
        None => encoder.push_u8(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox_plan::{SandboxEgressBinding, SandboxLocality};
    use crate::sandbox_profile::SandboxProfileClass;

    fn plan() -> SandboxLaunchPlan {
        SandboxLaunchPlan {
            profile_id: [1; 16],
            profile_version: 1,
            profile_class: SandboxProfileClass::NativeUntrustedSubprocess,
            filesystem_read_roots: vec!["/workspace/input".into(), "/workspace/shared".into()],
            filesystem_write_roots: vec!["/workspace/output".into()],
            network_rule: SandboxNetworkRule::PermitRequired,
            environment_allowlist: vec!["LANG".into(), "TZ".into()],
            spawn_rule: SandboxSpawnRule::DirectChildOnly,
            cpu_limit: Some(10),
            memory_limit: Some(512),
            time_limit: Some(30),
            output_limit: Some(4096),
            device_allowlist: vec!["device:null".into()],
            ipc_allowlist: vec!["ipc:fixture".into()],
            inherited_handle_rules: vec!["handle:fixture".into()],
            platform_requirements: vec!["env_clear".into()],
            principal_or_process: "process:fixture".into(),
            decision_id: [2; 16],
            lease_id: [3; 16],
            lease_generation: 1,
            policy_bundle_id: [4; 16],
            policy_bundle_hash: [5; 32],
            egress: Some(SandboxEgressBinding {
                permit_id: [6; 16],
                action: "network.egress.connect".into(),
                purpose: "sandbox-fixture".into(),
                destination_scope: "https://example.invalid".into(),
                protocol_port_scope: "https:443".into(),
                taint_digest: [7; 32],
                secret_handle_id: None,
                expires_at: Some("2026-08-30T00:00:00Z".into()),
                usage_limit: Some(3),
            }),
            locality: SandboxLocality::NonStrict,
            observed_at: "2026-08-29T00:00:00Z".into(),
            plan_hash: [8; 32],
        }
    }

    fn allowed_request<'a>() -> SandboxRequestedRights<'a> {
        SandboxRequestedRights {
            filesystem_read_roots: &["/workspace/shared", "/workspace/input"],
            filesystem_write_roots: &["/workspace/output"],
            environment_names: &["TZ"],
            network: SandboxNetworkRequest::PermitBoundExternal,
            spawn_rule: SandboxSpawnRule::DirectChildOnly,
            cpu_limit: Some(5),
            memory_limit: None,
            time_limit: Some(20),
            output_limit: Some(1024),
            devices: &["device:null"],
            ipc_endpoints: &["ipc:fixture"],
            inherited_handle_rules: &["handle:fixture"],
        }
    }

    #[test]
    fn descriptor_starts_from_cleared_environment_and_narrows_profile_rights() {
        let descriptor = resolve_sandbox_enforcement(&plan(), allowed_request()).unwrap();
        assert!(descriptor.clears_environment());
        assert!(!descriptor.claims_platform_containment());
        assert_eq!(descriptor.environment_names, ["TZ"]);
        assert_eq!(
            descriptor.filesystem_read_roots,
            ["/workspace/input", "/workspace/shared"]
        );
        assert_eq!(descriptor.cpu_limit, Some(5));
        assert_eq!(descriptor.memory_limit, Some(512));
        assert_eq!(descriptor.time_limit, Some(20));
        assert_eq!(descriptor.output_limit, Some(1024));
    }

    #[test]
    fn descriptor_is_deterministic_for_equivalent_requested_sets() {
        let first = resolve_sandbox_enforcement(&plan(), allowed_request()).unwrap();
        let mut alternate = allowed_request();
        alternate.filesystem_read_roots =
            &["/workspace/input", "/workspace/shared", "/workspace/input"];
        let second = resolve_sandbox_enforcement(&plan(), alternate).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn undeclared_environment_filesystem_device_ipc_and_handle_rights_deny() {
        let base = plan();
        let mut request = allowed_request();
        request.environment_names = &["HOME"];
        assert!(matches!(
            resolve_sandbox_enforcement(&base, request),
            Err(SandboxEnforcementError::UndeclaredEnvironmentName)
        ));
        let mut request = allowed_request();
        request.filesystem_write_roots = &["/workspace/input"];
        assert!(matches!(
            resolve_sandbox_enforcement(&base, request),
            Err(SandboxEnforcementError::UndeclaredFilesystemWrite)
        ));
        let mut request = allowed_request();
        request.devices = &["device:gpu0"];
        assert!(matches!(
            resolve_sandbox_enforcement(&base, request),
            Err(SandboxEnforcementError::UndeclaredDevice)
        ));
        let mut request = allowed_request();
        request.ipc_endpoints = &["ipc:ambient"];
        assert!(matches!(
            resolve_sandbox_enforcement(&base, request),
            Err(SandboxEnforcementError::UndeclaredIpcEndpoint)
        ));
        let mut request = allowed_request();
        request.inherited_handle_rules = &["handle:ambient"];
        assert!(matches!(
            resolve_sandbox_enforcement(&base, request),
            Err(SandboxEnforcementError::UndeclaredInheritedHandleRule)
        ));
    }

    #[test]
    fn network_and_spawn_widening_deny() {
        let mut deny_network = plan();
        deny_network.network_rule = SandboxNetworkRule::DenyAll;
        deny_network.egress = None;
        let mut request = allowed_request();
        request.network = SandboxNetworkRequest::PermitBoundExternal;
        assert!(matches!(
            resolve_sandbox_enforcement(&deny_network, request),
            Err(SandboxEnforcementError::NetworkWidening)
        ));

        let mut strict = plan();
        strict.locality = SandboxLocality::StrictLocal;
        assert!(matches!(
            resolve_sandbox_enforcement(&strict, allowed_request()),
            Err(SandboxEnforcementError::NetworkWidening)
        ));

        let mut no_spawn = plan();
        no_spawn.spawn_rule = SandboxSpawnRule::Deny;
        let mut request = allowed_request();
        request.spawn_rule = SandboxSpawnRule::DirectChildOnly;
        assert!(matches!(
            resolve_sandbox_enforcement(&no_spawn, request),
            Err(SandboxEnforcementError::SpawnWidening)
        ));
    }

    #[test]
    fn adversarial_rights_widening_is_denied_across_authority_axes() {
        let plan = plan();

        let mut request = allowed_request();
        request.filesystem_read_roots = &["/host/private"];
        assert!(matches!(
            resolve_sandbox_enforcement(&plan, request),
            Err(SandboxEnforcementError::UndeclaredFilesystemRead)
        ));

        let mut request = allowed_request();
        request.filesystem_write_roots = &["/host/private"];
        assert!(matches!(
            resolve_sandbox_enforcement(&plan, request),
            Err(SandboxEnforcementError::UndeclaredFilesystemWrite)
        ));

        let mut request = allowed_request();
        request.environment_names = &["GOLAM_AMBIENT_SECRET"];
        assert!(matches!(
            resolve_sandbox_enforcement(&plan, request),
            Err(SandboxEnforcementError::UndeclaredEnvironmentName)
        ));

        let mut request = allowed_request();
        request.devices = &["device:disk0"];
        assert!(matches!(
            resolve_sandbox_enforcement(&plan, request),
            Err(SandboxEnforcementError::UndeclaredDevice)
        ));

        let mut request = allowed_request();
        request.ipc_endpoints = &["ipc:host-daemon"];
        assert!(matches!(
            resolve_sandbox_enforcement(&plan, request),
            Err(SandboxEnforcementError::UndeclaredIpcEndpoint)
        ));

        let mut request = allowed_request();
        request.inherited_handle_rules = &["handle:ambient"];
        assert!(matches!(
            resolve_sandbox_enforcement(&plan, request),
            Err(SandboxEnforcementError::UndeclaredInheritedHandleRule)
        ));

        let mut strict = plan.clone();
        strict.locality = SandboxLocality::StrictLocal;
        let mut request = allowed_request();
        request.network = SandboxNetworkRequest::PermitBoundExternal;
        assert!(matches!(
            resolve_sandbox_enforcement(&strict, request),
            Err(SandboxEnforcementError::NetworkWidening)
        ));

        let mut deny_spawn = plan.clone();
        deny_spawn.spawn_rule = SandboxSpawnRule::Deny;
        let mut request = allowed_request();
        request.spawn_rule = SandboxSpawnRule::DirectChildOnly;
        assert!(matches!(
            resolve_sandbox_enforcement(&deny_spawn, request),
            Err(SandboxEnforcementError::SpawnWidening)
        ));
    }

    #[test]
    fn resource_requests_can_only_tighten_profile_bounds() {
        let descriptor = resolve_sandbox_enforcement(&plan(), allowed_request()).unwrap();
        assert_eq!(descriptor.cpu_limit, Some(5));
        assert_eq!(descriptor.memory_limit, Some(512));

        let mut request = allowed_request();
        request.cpu_limit = Some(50);
        let descriptor = resolve_sandbox_enforcement(&plan(), request).unwrap();
        assert_eq!(descriptor.cpu_limit, Some(10));

        let mut request = allowed_request();
        request.time_limit = Some(0);
        assert!(matches!(
            resolve_sandbox_enforcement(&plan(), request),
            Err(SandboxEnforcementError::InvalidResourceLimit)
        ));
    }

    #[test]
    fn deny_all_request_has_no_ambient_rights() {
        let descriptor =
            resolve_sandbox_enforcement(&plan(), SandboxRequestedRights::deny_all()).unwrap();
        assert!(descriptor.filesystem_read_roots.is_empty());
        assert!(descriptor.filesystem_write_roots.is_empty());
        assert!(descriptor.environment_names.is_empty());
        assert_eq!(descriptor.network, SandboxNetworkRequest::None);
        assert_eq!(descriptor.spawn_rule, SandboxSpawnRule::Deny);
        assert!(descriptor.devices.is_empty());
        assert!(descriptor.ipc_endpoints.is_empty());
        assert!(descriptor.inherited_handle_rules.is_empty());
    }
}
