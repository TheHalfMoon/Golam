#![forbid(unsafe_code)]

use core::fmt;

const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_REQUIREMENTS: usize = 32;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ToolId(String);

impl ToolId {
    pub fn new(value: impl Into<String>) -> Result<Self, ToolValidationError> {
        let value = value.into();
        validate_identifier(&value, "tool_id")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ToolVersion(String);

impl ToolVersion {
    pub fn new(value: impl Into<String>) -> Result<Self, ToolValidationError> {
        let value = value.into();
        validate_identifier(&value, "tool_version")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Classification metadata only; this is not a lease, approval, or authority.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CapabilityClassId(String);

impl CapabilityClassId {
    pub fn new(value: impl Into<String>) -> Result<Self, ToolValidationError> {
        let value = value.into();
        validate_identifier(&value, "required_capability_class")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BoundDimension {
    NotApplicable,
    Finite(u64),
}

impl BoundDimension {
    fn validate(self, field: &'static str) -> Result<(), ToolValidationError> {
        match self {
            Self::NotApplicable => Ok(()),
            Self::Finite(0) => Err(ToolValidationError::ZeroFiniteBound(field)),
            Self::Finite(_) => Ok(()),
        }
    }

    pub const fn finite_value(self) -> Option<u64> {
        match self {
            Self::NotApplicable => None,
            Self::Finite(value) => Some(value),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolIoBounds {
    pub max_bytes: BoundDimension,
    pub max_items: BoundDimension,
    pub max_nesting_depth: BoundDimension,
    pub max_field_bytes: BoundDimension,
}

impl ToolIoBounds {
    pub fn validate(self) -> Result<(), ToolValidationError> {
        self.max_bytes.validate("max_bytes")?;
        self.max_items.validate("max_items")?;
        self.max_nesting_depth.validate("max_nesting_depth")?;
        self.max_field_bytes.validate("max_field_bytes")?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolDurationBounds {
    pub max_total_duration_ms: BoundDimension,
    pub max_idle_duration_ms: BoundDimension,
}

impl ToolDurationBounds {
    pub fn validate(self) -> Result<(), ToolValidationError> {
        self.max_total_duration_ms
            .validate("max_total_duration_ms")?;
        self.max_idle_duration_ms.validate("max_idle_duration_ms")?;
        if self.max_total_duration_ms.finite_value().is_none() {
            return Err(ToolValidationError::MissingFiniteBound(
                "max_total_duration_ms",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ToolOperationClass {
    Read,
    List,
    Search,
    ContextCompile,
    Mutation,
    GitRead,
    GitMutation,
    MemoryMutation,
    ProcessExecution,
    NetworkRequest,
    ProtocolDispatch,
}

impl ToolOperationClass {
    const fn is_read_only(self) -> bool {
        matches!(
            self,
            Self::Read | Self::List | Self::Search | Self::ContextCompile | Self::GitRead
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ToolEffectSemantics {
    ReadOnly,
    IdempotentAtLeastOnce,
    AtMostOnce,
    Compensatable,
    Irreversible,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ToolNetworkPosture {
    Denied,
    LoopbackOnly,
    ExplicitEgress,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ToolSandboxRequirement {
    NotRequired,
    BoundaryOnly,
    ProductionContainmentRequired,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ToolTargetIdentityKind {
    None,
    FilesystemObject,
    GitRepository,
    ProcessExecutable,
    NetworkEndpoint,
    ProtocolBinding,
    MemoryStore,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ToolTargetIdentityRule {
    ExactResolvedIdentity,
    BoundedResolutionPlan,
    RevalidateBeforeAction,
    ParentIdentityForCreation,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ToolReconcileTrigger {
    Timeout,
    Cancellation,
    PartialEffect,
    TransportDisconnect,
    Restart,
    VerificationFailure,
    ProcessTreeUnresolved,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ToolUnknownOutcomeBehavior {
    FailClosed,
    ReconcileBeforeRetry,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ToolDependentEffectBehavior {
    BlockUntilReconciled,
    DenyDependentEffects,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ToolEvidenceRequirement {
    AuthorityJournal,
    TargetIdentityObservation,
    TargetReadback,
    FilesystemMetadata,
    GitState,
    ProcessTreeTerminalState,
    NetworkEndpointIdentity,
    ProtocolBindingState,
    MemoryStoreReadback,
    ExplicitFailureRecord,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ToolReadbackSourceClass {
    Filesystem,
    Git,
    ProcessSupervisor,
    NetworkTransport,
    ProtocolAdapter,
    MemoryStore,
    AuthorityJournal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolReconciliationPolicy {
    pub reconcile_on: Vec<ToolReconcileTrigger>,
    pub unknown_outcome_behavior: ToolUnknownOutcomeBehavior,
    pub dependent_effect_behavior: ToolDependentEffectBehavior,
    pub observation_requirements: Vec<ToolEvidenceRequirement>,
    pub terminal_evidence_requirements: Vec<ToolEvidenceRequirement>,
}

impl ToolReconciliationPolicy {
    pub fn validate(&self) -> Result<(), ToolValidationError> {
        validate_ordered_unique(&self.reconcile_on, "reconcile_on")?;
        validate_ordered_unique(
            &self.observation_requirements,
            "reconciliation_observation_requirements",
        )?;
        validate_ordered_unique(
            &self.terminal_evidence_requirements,
            "reconciliation_terminal_evidence_requirements",
        )?;
        if self.unknown_outcome_behavior == ToolUnknownOutcomeBehavior::ReconcileBeforeRetry
            && (self.observation_requirements.is_empty()
                || self.terminal_evidence_requirements.is_empty())
        {
            return Err(ToolValidationError::MissingRequirement(
                "reconciliation evidence for UNKNOWN_OUTCOME",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolVerificationPolicy {
    pub success_evidence_requirements: Vec<ToolEvidenceRequirement>,
    pub independent_readback_required: bool,
    pub readback_source_class: Option<ToolReadbackSourceClass>,
    pub failure_evidence_requirements: Vec<ToolEvidenceRequirement>,
}

impl ToolVerificationPolicy {
    pub fn validate(&self) -> Result<(), ToolValidationError> {
        validate_ordered_unique(
            &self.success_evidence_requirements,
            "success_evidence_requirements",
        )?;
        validate_ordered_unique(
            &self.failure_evidence_requirements,
            "failure_evidence_requirements",
        )?;
        if self.success_evidence_requirements.is_empty() {
            return Err(ToolValidationError::MissingRequirement(
                "success_evidence_requirements",
            ));
        }
        if self.failure_evidence_requirements.is_empty() {
            return Err(ToolValidationError::MissingRequirement(
                "failure_evidence_requirements",
            ));
        }
        match (
            self.independent_readback_required,
            self.readback_source_class,
        ) {
            (true, None) => Err(ToolValidationError::MissingRequirement(
                "readback_source_class",
            )),
            (false, Some(_)) => Err(ToolValidationError::IncompatibleDescriptor(
                "readback_source_class requires independent_readback_required",
            )),
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolDescriptor {
    pub tool_id: ToolId,
    pub version: ToolVersion,
    pub operation_class: ToolOperationClass,
    pub input_bounds: ToolIoBounds,
    pub output_bounds: ToolIoBounds,
    pub duration_bounds: ToolDurationBounds,
    pub required_capability_class: CapabilityClassId,
    pub effect_semantics: ToolEffectSemantics,
    pub network_posture: ToolNetworkPosture,
    pub sandbox_requirement: ToolSandboxRequirement,
    pub target_identity_kind: ToolTargetIdentityKind,
    pub target_identity_rules: Vec<ToolTargetIdentityRule>,
    pub reconciliation_policy: ToolReconciliationPolicy,
    pub verification_policy: ToolVerificationPolicy,
}

impl ToolDescriptor {
    pub fn validate(&self) -> Result<(), ToolValidationError> {
        validate_identifier(self.tool_id.as_str(), "tool_id")?;
        validate_identifier(self.version.as_str(), "tool_version")?;
        validate_identifier(
            self.required_capability_class.as_str(),
            "required_capability_class",
        )?;
        self.input_bounds.validate()?;
        self.output_bounds.validate()?;
        self.duration_bounds.validate()?;
        self.reconciliation_policy.validate()?;
        self.verification_policy.validate()?;
        validate_ordered_unique(&self.target_identity_rules, "target_identity_rules")?;

        match self.target_identity_kind {
            ToolTargetIdentityKind::None if !self.target_identity_rules.is_empty() => {
                return Err(ToolValidationError::IncompatibleDescriptor(
                    "target identity rules require a target identity kind",
                ));
            }
            ToolTargetIdentityKind::None => {}
            _ if self.target_identity_rules.is_empty() => {
                return Err(ToolValidationError::MissingRequirement(
                    "target_identity_rules",
                ));
            }
            _ => {}
        }

        if self.operation_class.is_read_only()
            && self.effect_semantics != ToolEffectSemantics::ReadOnly
        {
            return Err(ToolValidationError::IncompatibleDescriptor(
                "read-only operation must use ReadOnly effect semantics",
            ));
        }
        if !self.operation_class.is_read_only()
            && self.effect_semantics == ToolEffectSemantics::ReadOnly
        {
            return Err(ToolValidationError::IncompatibleDescriptor(
                "non-read-only operation cannot use ReadOnly effect semantics",
            ));
        }
        if self.operation_class == ToolOperationClass::ProcessExecution
            && self.sandbox_requirement != ToolSandboxRequirement::ProductionContainmentRequired
        {
            return Err(ToolValidationError::IncompatibleDescriptor(
                "process execution requires production containment",
            ));
        }
        if self.operation_class == ToolOperationClass::NetworkRequest
            && self.network_posture == ToolNetworkPosture::Denied
        {
            return Err(ToolValidationError::IncompatibleDescriptor(
                "network request cannot advertise denied network posture",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolValidationError {
    InvalidIdentifier(&'static str),
    ZeroFiniteBound(&'static str),
    MissingFiniteBound(&'static str),
    TooManyRequirements(&'static str),
    UnsortedOrDuplicate(&'static str),
    MissingRequirement(&'static str),
    IncompatibleDescriptor(&'static str),
}

impl fmt::Display for ToolValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentifier(field) => write!(f, "invalid bounded identifier: {field}"),
            Self::ZeroFiniteBound(field) => write!(f, "finite bound must be positive: {field}"),
            Self::MissingFiniteBound(field) => {
                write!(f, "required finite bound is missing: {field}")
            }
            Self::TooManyRequirements(field) => {
                write!(f, "too many bounded requirements: {field}")
            }
            Self::UnsortedOrDuplicate(field) => {
                write!(
                    f,
                    "requirements must be strictly sorted and unique: {field}"
                )
            }
            Self::MissingRequirement(field) => {
                write!(f, "required descriptor evidence is missing: {field}")
            }
            Self::IncompatibleDescriptor(reason) => {
                write!(f, "incompatible tool descriptor: {reason}")
            }
        }
    }
}

impl std::error::Error for ToolValidationError {}

fn validate_identifier(value: &str, field: &'static str) -> Result<(), ToolValidationError> {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES {
        return Err(ToolValidationError::InvalidIdentifier(field));
    }
    if !value.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':' | b'+')
    }) {
        return Err(ToolValidationError::InvalidIdentifier(field));
    }
    Ok(())
}

fn validate_ordered_unique<T: Ord>(
    values: &[T],
    field: &'static str,
) -> Result<(), ToolValidationError> {
    if values.len() > MAX_REQUIREMENTS {
        return Err(ToolValidationError::TooManyRequirements(field));
    }
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(ToolValidationError::UnsortedOrDuplicate(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finite_io() -> ToolIoBounds {
        ToolIoBounds {
            max_bytes: BoundDimension::Finite(1024),
            max_items: BoundDimension::Finite(16),
            max_nesting_depth: BoundDimension::NotApplicable,
            max_field_bytes: BoundDimension::Finite(256),
        }
    }

    fn verification() -> ToolVerificationPolicy {
        ToolVerificationPolicy {
            success_evidence_requirements: vec![ToolEvidenceRequirement::TargetReadback],
            independent_readback_required: true,
            readback_source_class: Some(ToolReadbackSourceClass::Filesystem),
            failure_evidence_requirements: vec![ToolEvidenceRequirement::ExplicitFailureRecord],
        }
    }

    fn reconciliation() -> ToolReconciliationPolicy {
        ToolReconciliationPolicy {
            reconcile_on: vec![ToolReconcileTrigger::Restart],
            unknown_outcome_behavior: ToolUnknownOutcomeBehavior::ReconcileBeforeRetry,
            dependent_effect_behavior: ToolDependentEffectBehavior::BlockUntilReconciled,
            observation_requirements: vec![ToolEvidenceRequirement::TargetIdentityObservation],
            terminal_evidence_requirements: vec![ToolEvidenceRequirement::TargetReadback],
        }
    }

    fn read_descriptor() -> ToolDescriptor {
        ToolDescriptor {
            tool_id: ToolId::new("fs.read").unwrap(),
            version: ToolVersion::new("1.0.0").unwrap(),
            operation_class: ToolOperationClass::Read,
            input_bounds: finite_io(),
            output_bounds: finite_io(),
            duration_bounds: ToolDurationBounds {
                max_total_duration_ms: BoundDimension::Finite(5_000),
                max_idle_duration_ms: BoundDimension::NotApplicable,
            },
            required_capability_class: CapabilityClassId::new("filesystem.read").unwrap(),
            effect_semantics: ToolEffectSemantics::ReadOnly,
            network_posture: ToolNetworkPosture::Denied,
            sandbox_requirement: ToolSandboxRequirement::BoundaryOnly,
            target_identity_kind: ToolTargetIdentityKind::FilesystemObject,
            target_identity_rules: vec![
                ToolTargetIdentityRule::ExactResolvedIdentity,
                ToolTargetIdentityRule::RevalidateBeforeAction,
            ],
            reconciliation_policy: reconciliation(),
            verification_policy: verification(),
        }
    }

    #[test]
    fn identifiers_are_bounded() {
        assert!(ToolId::new("").is_err());
        assert!(ToolId::new("../../escape").is_err());
        assert!(CapabilityClassId::new("filesystem.read").is_ok());
    }

    #[test]
    fn finite_bounds_fail_closed() {
        let invalid = ToolIoBounds {
            max_bytes: BoundDimension::Finite(0),
            ..finite_io()
        };
        assert_eq!(
            invalid.validate(),
            Err(ToolValidationError::ZeroFiniteBound("max_bytes"))
        );

        let duration = ToolDurationBounds {
            max_total_duration_ms: BoundDimension::NotApplicable,
            max_idle_duration_ms: BoundDimension::NotApplicable,
        };
        assert_eq!(
            duration.validate(),
            Err(ToolValidationError::MissingFiniteBound(
                "max_total_duration_ms"
            ))
        );
    }

    #[test]
    fn policy_vectors_are_strictly_ordered() {
        let mut policy = reconciliation();
        policy.reconcile_on = vec![ToolReconcileTrigger::Restart, ToolReconcileTrigger::Restart];
        assert_eq!(
            policy.validate(),
            Err(ToolValidationError::UnsortedOrDuplicate("reconcile_on"))
        );
    }

    #[test]
    fn descriptor_rejects_authority_widening_shapes() {
        let mut descriptor = read_descriptor();
        assert_eq!(descriptor.validate(), Ok(()));

        descriptor.effect_semantics = ToolEffectSemantics::AtMostOnce;
        assert!(matches!(
            descriptor.validate(),
            Err(ToolValidationError::IncompatibleDescriptor(_))
        ));

        descriptor = read_descriptor();
        descriptor.target_identity_rules.clear();
        assert_eq!(
            descriptor.validate(),
            Err(ToolValidationError::MissingRequirement(
                "target_identity_rules"
            ))
        );

        descriptor = read_descriptor();
        descriptor.operation_class = ToolOperationClass::ProtocolDispatch;
        assert_eq!(
            descriptor.validate(),
            Err(ToolValidationError::IncompatibleDescriptor(
                "non-read-only operation cannot use ReadOnly effect semantics"
            ))
        );
    }

    #[test]
    fn process_execution_requires_production_containment() {
        let mut descriptor = read_descriptor();
        descriptor.operation_class = ToolOperationClass::ProcessExecution;
        descriptor.effect_semantics = ToolEffectSemantics::AtMostOnce;
        descriptor.target_identity_kind = ToolTargetIdentityKind::ProcessExecutable;
        descriptor.sandbox_requirement = ToolSandboxRequirement::BoundaryOnly;
        assert_eq!(
            descriptor.validate(),
            Err(ToolValidationError::IncompatibleDescriptor(
                "process execution requires production containment"
            ))
        );
    }
}
