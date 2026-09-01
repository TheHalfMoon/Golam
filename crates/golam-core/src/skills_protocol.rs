#![forbid(unsafe_code)]

use core::fmt;

use crate::taint::TaintSet;
use crate::tool_descriptor::{CapabilityClassId, ToolNetworkPosture};
use crate::tool_request::BindingDigest;

const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_BINDING_REFS: usize = 128;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SkillVersion(String);

impl SkillVersion {
    pub fn new(value: impl Into<String>) -> Result<Self, ProtocolValidationError> {
        let value = value.into();
        validate_identifier(&value, "skill_version")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SkillAdmissionState {
    Discovered,
    ProvenanceRecorded,
    Reviewed,
    InstructionAdmitted,
    ExecutableAdmitted,
    LockedVersion,
    Deprecated,
    Revoked,
    Unknown,
}

impl SkillAdmissionState {
    const fn is_dispatchable(self) -> bool {
        matches!(
            self,
            Self::InstructionAdmitted | Self::ExecutableAdmitted | Self::LockedVersion
        )
    }

    const fn allows_executable(self) -> bool {
        matches!(self, Self::ExecutableAdmitted | Self::LockedVersion)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillDescriptor {
    pub name_ref: BindingDigest,
    pub description_ref: BindingDigest,
    pub package_ref: BindingDigest,
    pub version: SkillVersion,
    pub content_digest: BindingDigest,
    pub instruction_ref: BindingDigest,
    pub script_refs: Vec<BindingDigest>,
    pub requested_capability_classes: Vec<CapabilityClassId>,
    pub network_posture: ToolNetworkPosture,
    pub provenance_refs: Vec<BindingDigest>,
    pub admission_state: SkillAdmissionState,
}

impl SkillDescriptor {
    pub fn validate(&self) -> Result<(), ProtocolValidationError> {
        validate_ordered_unique(&self.script_refs, "skill script_refs")?;
        validate_ordered_unique(
            &self.requested_capability_classes,
            "skill requested_capability_classes",
        )?;
        validate_ordered_unique(&self.provenance_refs, "skill provenance_refs")?;
        if self.provenance_refs.is_empty() {
            return Err(ProtocolValidationError::MissingRequirement(
                "skill provenance_refs",
            ));
        }
        if !self.script_refs.is_empty()
            && !matches!(
                self.admission_state,
                SkillAdmissionState::ExecutableAdmitted | SkillAdmissionState::LockedVersion
            )
        {
            return Err(ProtocolValidationError::ExecutableSkillNotAdmitted);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SkillDispatchKind {
    InstructionActivation,
    ExecutableDispatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillDispatchBinding {
    pub skill_package_ref: BindingDigest,
    pub skill_version: SkillVersion,
    pub reviewed_content_digest: BindingDigest,
    pub reviewed_admission_state_ref: BindingDigest,
    pub reviewed_capability_mapping_ref: BindingDigest,
    pub queued_request_ref: BindingDigest,
    pub capability_decision_ref: BindingDigest,
    pub approval_decision_ref: BindingDigest,
    pub dispatch_kind: SkillDispatchKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrentSkillDispatchState {
    pub skill_package_ref: BindingDigest,
    pub skill_version: SkillVersion,
    pub content_digest: BindingDigest,
    pub admission_state: SkillAdmissionState,
    pub admission_state_ref: BindingDigest,
    pub capability_mapping_ref: BindingDigest,
}

impl SkillDispatchBinding {
    pub fn revalidate(
        &self,
        current: &CurrentSkillDispatchState,
    ) -> Result<(), DispatchValidationError> {
        if self.skill_package_ref != current.skill_package_ref {
            return Err(DispatchValidationError::SkillPackageMismatch);
        }
        if self.skill_version != current.skill_version {
            return Err(DispatchValidationError::SkillVersionMismatch);
        }
        if self.reviewed_content_digest != current.content_digest {
            return Err(DispatchValidationError::SkillContentDigestMismatch);
        }
        if self.reviewed_admission_state_ref != current.admission_state_ref {
            return Err(DispatchValidationError::SkillAdmissionStateMismatch);
        }
        if self.reviewed_capability_mapping_ref != current.capability_mapping_ref {
            return Err(DispatchValidationError::SkillCapabilityMappingMismatch);
        }
        if !current.admission_state.is_dispatchable() {
            return Err(DispatchValidationError::SkillLifecycleNotDispatchable);
        }
        if self.dispatch_kind == SkillDispatchKind::ExecutableDispatch
            && !current.admission_state.allows_executable()
        {
            return Err(DispatchValidationError::ExecutableSkillNotAdmitted);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct McpVersionLock(String);

impl McpVersionLock {
    pub fn new(value: impl Into<String>) -> Result<Self, ProtocolValidationError> {
        let value = value.into();
        validate_identifier(&value, "mcp_version_lock")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum McpTransport {
    LocalStdio,
    RemoteHttp,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum McpLifecycleState {
    Reviewed,
    Active,
    Deprecated,
    Revoked,
    Replaced,
    Unknown,
}

impl McpLifecycleState {
    const fn is_dispatchable(self) -> bool {
        matches!(self, Self::Active)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProtocolFeatureId(pub BindingDigest);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpServerBinding {
    pub binding_id: BindingDigest,
    pub binding_digest: BindingDigest,
    pub server_identity: BindingDigest,
    pub transport: McpTransport,
    pub process_profile_ref_or_remote_endpoint: BindingDigest,
    pub allowed_protocol_features: Vec<ProtocolFeatureId>,
    pub golam_local_mapping_ref: BindingDigest,
    pub golam_local_mapping_digest: BindingDigest,
    pub network_policy_ref: BindingDigest,
    pub secret_policy_ref: BindingDigest,
    pub taint_class: TaintSet,
    pub version_lock: McpVersionLock,
    pub lifecycle_state: McpLifecycleState,
}

impl McpServerBinding {
    pub fn validate(&self) -> Result<(), ProtocolValidationError> {
        validate_ordered_unique(
            &self.allowed_protocol_features,
            "mcp allowed_protocol_features",
        )?;
        if self.transport == McpTransport::RemoteHttp
            && self.lifecycle_state == McpLifecycleState::Active
            && self.network_policy_ref.bytes() == [0; 32]
        {
            return Err(ProtocolValidationError::MissingRequirement(
                "remote MCP network_policy_ref",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpDispatchBinding {
    pub binding_id: BindingDigest,
    pub binding_digest: BindingDigest,
    pub version_lock: McpVersionLock,
    pub golam_local_mapping_ref: BindingDigest,
    pub golam_local_mapping_digest: BindingDigest,
    pub lifecycle_state_ref: BindingDigest,
    pub queued_request_ref: BindingDigest,
    pub capability_decision_ref: BindingDigest,
    pub approval_decision_ref: BindingDigest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrentMcpDispatchState {
    pub binding_id: BindingDigest,
    pub binding_digest: BindingDigest,
    pub version_lock: McpVersionLock,
    pub golam_local_mapping_ref: BindingDigest,
    pub golam_local_mapping_digest: BindingDigest,
    pub lifecycle_state: McpLifecycleState,
    pub lifecycle_state_ref: BindingDigest,
}

impl McpDispatchBinding {
    pub fn revalidate(
        &self,
        current: &CurrentMcpDispatchState,
    ) -> Result<(), DispatchValidationError> {
        if self.binding_id != current.binding_id {
            return Err(DispatchValidationError::McpBindingIdMismatch);
        }
        if self.binding_digest != current.binding_digest {
            return Err(DispatchValidationError::McpBindingDigestMismatch);
        }
        if self.version_lock != current.version_lock {
            return Err(DispatchValidationError::McpVersionMismatch);
        }
        if self.golam_local_mapping_ref != current.golam_local_mapping_ref
            || self.golam_local_mapping_digest != current.golam_local_mapping_digest
        {
            return Err(DispatchValidationError::McpMappingMismatch);
        }
        if self.lifecycle_state_ref != current.lifecycle_state_ref {
            return Err(DispatchValidationError::McpLifecycleStateMismatch);
        }
        if !current.lifecycle_state.is_dispatchable() {
            return Err(DispatchValidationError::McpLifecycleNotDispatchable);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalToolDescriptor {
    pub server_tool_identity: BindingDigest,
    pub input_schema_digest: BindingDigest,
    pub output_schema_digest: BindingDigest,
    pub golam_local_mapping_ref: BindingDigest,
    pub golam_local_mapping_digest: BindingDigest,
    pub taint_class: TaintSet,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchValidationError {
    SkillPackageMismatch,
    SkillVersionMismatch,
    SkillContentDigestMismatch,
    SkillAdmissionStateMismatch,
    SkillCapabilityMappingMismatch,
    SkillLifecycleNotDispatchable,
    ExecutableSkillNotAdmitted,
    McpBindingIdMismatch,
    McpBindingDigestMismatch,
    McpVersionMismatch,
    McpMappingMismatch,
    McpLifecycleStateMismatch,
    McpLifecycleNotDispatchable,
}

impl fmt::Display for DispatchValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SkillPackageMismatch => f.write_str("skill package identity changed"),
            Self::SkillVersionMismatch => f.write_str("skill version changed"),
            Self::SkillContentDigestMismatch => f.write_str("skill content digest changed"),
            Self::SkillAdmissionStateMismatch => {
                f.write_str("skill admission-state evidence changed")
            }
            Self::SkillCapabilityMappingMismatch => {
                f.write_str("skill capability mapping changed")
            }
            Self::SkillLifecycleNotDispatchable => {
                f.write_str("skill lifecycle state is not dispatchable")
            }
            Self::ExecutableSkillNotAdmitted => {
                f.write_str("skill executable dispatch is not admitted")
            }
            Self::McpBindingIdMismatch => f.write_str("MCP binding identity changed"),
            Self::McpBindingDigestMismatch => f.write_str("MCP binding digest changed"),
            Self::McpVersionMismatch => f.write_str("MCP version lock changed"),
            Self::McpMappingMismatch => f.write_str("MCP local mapping changed"),
            Self::McpLifecycleStateMismatch => {
                f.write_str("MCP lifecycle-state evidence changed")
            }
            Self::McpLifecycleNotDispatchable => {
                f.write_str("MCP lifecycle state is not dispatchable")
            }
        }
    }
}

impl std::error::Error for DispatchValidationError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolValidationError {
    InvalidIdentifier(&'static str),
    TooManyReferences(&'static str),
    UnsortedOrDuplicate(&'static str),
    MissingRequirement(&'static str),
    ExecutableSkillNotAdmitted,
}

impl fmt::Display for ProtocolValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentifier(field) => {
                write!(f, "invalid bounded protocol identifier: {field}")
            }
            Self::TooManyReferences(field) => {
                write!(f, "protocol reference bound exceeded: {field}")
            }
            Self::UnsortedOrDuplicate(field) => {
                write!(f, "protocol references must be sorted and unique: {field}")
            }
            Self::MissingRequirement(field) => {
                write!(f, "required protocol binding is missing: {field}")
            }
            Self::ExecutableSkillNotAdmitted => {
                f.write_str("skill scripts require executable admission")
            }
        }
    }
}

impl std::error::Error for ProtocolValidationError {}

fn validate_identifier(
    value: &str,
    field: &'static str,
) -> Result<(), ProtocolValidationError> {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES {
        return Err(ProtocolValidationError::InvalidIdentifier(field));
    }
    if !value.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':' | b'+')
    }) {
        return Err(ProtocolValidationError::InvalidIdentifier(field));
    }
    Ok(())
}

fn validate_ordered_unique<T: Ord>(
    values: &[T],
    field: &'static str,
) -> Result<(), ProtocolValidationError> {
    if values.len() > MAX_BINDING_REFS {
        return Err(ProtocolValidationError::TooManyReferences(field));
    }
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(ProtocolValidationError::UnsortedOrDuplicate(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn skill_state() -> CurrentSkillDispatchState {
        CurrentSkillDispatchState {
            skill_package_ref: digest(1),
            skill_version: SkillVersion::new("1.0.0").unwrap(),
            content_digest: digest(2),
            admission_state: SkillAdmissionState::InstructionAdmitted,
            admission_state_ref: digest(3),
            capability_mapping_ref: digest(4),
        }
    }

    fn skill_binding() -> SkillDispatchBinding {
        SkillDispatchBinding {
            skill_package_ref: digest(1),
            skill_version: SkillVersion::new("1.0.0").unwrap(),
            reviewed_content_digest: digest(2),
            reviewed_admission_state_ref: digest(3),
            reviewed_capability_mapping_ref: digest(4),
            queued_request_ref: digest(5),
            capability_decision_ref: digest(6),
            approval_decision_ref: digest(7),
            dispatch_kind: SkillDispatchKind::InstructionActivation,
        }
    }

    #[test]
    fn skill_dispatch_rejects_stale_or_revoked_state() {
        let binding = skill_binding();
        assert_eq!(binding.revalidate(&skill_state()), Ok(()));

        let mut changed = skill_state();
        changed.content_digest = digest(99);
        assert_eq!(
            binding.revalidate(&changed),
            Err(DispatchValidationError::SkillContentDigestMismatch)
        );

        let mut revoked = skill_state();
        revoked.admission_state = SkillAdmissionState::Revoked;
        assert_eq!(
            binding.revalidate(&revoked),
            Err(DispatchValidationError::SkillLifecycleNotDispatchable)
        );
    }

    #[test]
    fn executable_skill_requires_executable_admission() {
        let mut binding = skill_binding();
        binding.dispatch_kind = SkillDispatchKind::ExecutableDispatch;
        assert_eq!(
            binding.revalidate(&skill_state()),
            Err(DispatchValidationError::ExecutableSkillNotAdmitted)
        );

        let mut admitted = skill_state();
        admitted.admission_state = SkillAdmissionState::ExecutableAdmitted;
        assert_eq!(binding.revalidate(&admitted), Ok(()));
    }

    #[test]
    fn mcp_dispatch_rejects_replacement_version_and_mapping_drift() {
        let binding = McpDispatchBinding {
            binding_id: digest(1),
            binding_digest: digest(2),
            version_lock: McpVersionLock::new("2025-06-18").unwrap(),
            golam_local_mapping_ref: digest(3),
            golam_local_mapping_digest: digest(4),
            lifecycle_state_ref: digest(5),
            queued_request_ref: digest(6),
            capability_decision_ref: digest(7),
            approval_decision_ref: digest(8),
        };
        let current = CurrentMcpDispatchState {
            binding_id: digest(1),
            binding_digest: digest(2),
            version_lock: McpVersionLock::new("2025-06-18").unwrap(),
            golam_local_mapping_ref: digest(3),
            golam_local_mapping_digest: digest(4),
            lifecycle_state: McpLifecycleState::Active,
            lifecycle_state_ref: digest(5),
        };
        assert_eq!(binding.revalidate(&current), Ok(()));

        let mut replaced = current.clone();
        replaced.lifecycle_state = McpLifecycleState::Replaced;
        assert_eq!(
            binding.revalidate(&replaced),
            Err(DispatchValidationError::McpLifecycleNotDispatchable)
        );

        let mut remapped = current;
        remapped.golam_local_mapping_digest = digest(99);
        assert_eq!(
            binding.revalidate(&remapped),
            Err(DispatchValidationError::McpMappingMismatch)
        );
    }

    #[test]
    fn descriptor_capability_classes_are_bounded_classification_only() {
        let descriptor = SkillDescriptor {
            name_ref: digest(1),
            description_ref: digest(2),
            package_ref: digest(3),
            version: SkillVersion::new("1.0.0").unwrap(),
            content_digest: digest(4),
            instruction_ref: digest(5),
            script_refs: vec![],
            requested_capability_classes: vec![
                CapabilityClassId::new("workspace.read").unwrap(),
            ],
            network_posture: ToolNetworkPosture::Denied,
            provenance_refs: vec![digest(6)],
            admission_state: SkillAdmissionState::InstructionAdmitted,
        };
        assert_eq!(descriptor.validate(), Ok(()));
    }
}
