#![forbid(unsafe_code)]

use core::fmt;

use crate::digest::sha256;
use crate::harness::ToolCallCandidateId;
use crate::taint::TaintSet;
use crate::tool_descriptor::{ToolId, ToolVersion};
use crate::{CanonicalEncoder, CoreError, EffectId};

const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_TARGET_BYTES: usize = 16 * 1024;
const MAX_BINDING_REFS: usize = 128;
const TOOL_REQUEST_DOMAIN: &[u8] = b"golam:tool-request:v1";
const TOOL_RESULT_DOMAIN: &[u8] = b"golam:tool-result:v1";

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ToolRequestId(u128);

impl ToolRequestId {
    pub const fn from_u128(value: u128) -> Self {
        Self(value)
    }

    pub const fn as_u128(self) -> u128 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PrincipalId(String);

impl PrincipalId {
    pub fn new(value: impl Into<String>) -> Result<Self, ToolRequestError> {
        let value = value.into();
        validate_identifier(&value, "initiating_principal")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RequestedOperationId(String);

impl RequestedOperationId {
    pub fn new(value: impl Into<String>) -> Result<Self, ToolRequestError> {
        let value = value.into();
        validate_identifier(&value, "requested_operation")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResourceClassId(String);

impl ResourceClassId {
    pub fn new(value: impl Into<String>) -> Result<Self, ToolRequestError> {
        let value = value.into();
        validate_identifier(&value, "authorized_resource_class")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Content supplied as an execution target. This type carries no authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestedTarget(String);

impl RequestedTarget {
    pub fn new(value: impl Into<String>) -> Result<Self, ToolRequestError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > MAX_TARGET_BYTES
            || value
                .chars()
                .any(|character| character == '\0' || character == '\r')
        {
            return Err(ToolRequestError::InvalidTarget);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BindingDigest([u8; 32]);

impl BindingDigest {
    pub const fn new(value: [u8; 32]) -> Self {
        Self(value)
    }

    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolRequest {
    pub request_id: ToolRequestId,
    pub initiating_principal: PrincipalId,
    pub tool_id: ToolId,
    pub tool_version: ToolVersion,
    pub candidate_ref: ToolCallCandidateId,
    pub requested_operation: RequestedOperationId,
    pub requested_target: Option<RequestedTarget>,
    pub authorized_resource_class: ResourceClassId,
    pub target_identity_ref: Option<BindingDigest>,
    pub target_resolution_plan_ref: Option<BindingDigest>,
    pub capability_context_ref: BindingDigest,
    pub taint_set: TaintSet,
    pub provenance_refs: Vec<BindingDigest>,
    pub idempotency_material: BindingDigest,
    pub current_preconditions: Vec<BindingDigest>,
    pub created_at_unix_ms: u64,
}

impl ToolRequest {
    pub fn validate(&self) -> Result<(), ToolRequestError> {
        validate_identifier(self.initiating_principal.as_str(), "initiating_principal")?;
        validate_identifier(self.tool_id.as_str(), "tool_id")?;
        validate_identifier(self.tool_version.as_str(), "tool_version")?;
        validate_identifier(self.requested_operation.as_str(), "requested_operation")?;
        validate_identifier(
            self.authorized_resource_class.as_str(),
            "authorized_resource_class",
        )?;
        validate_ordered_unique(&self.provenance_refs, "provenance_refs")?;
        validate_ordered_unique(&self.current_preconditions, "current_preconditions")?;

        match (
            self.requested_target.is_some(),
            self.target_identity_ref.is_some(),
            self.target_resolution_plan_ref.is_some(),
        ) {
            (false, false, false) => {}
            (true, true, false) | (true, false, true) => {}
            (false, _, _) => return Err(ToolRequestError::TargetBindingWithoutTarget),
            (true, false, false) => return Err(ToolRequestError::MissingTargetBinding),
            (true, true, true) => return Err(ToolRequestError::AmbiguousTargetBinding),
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ToolRequestError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(TOOL_REQUEST_DOMAIN)?;
        encoder.push_u128(self.request_id.as_u128());
        encoder.push_bytes(self.initiating_principal.as_str().as_bytes())?;
        encoder.push_bytes(self.tool_id.as_str().as_bytes())?;
        encoder.push_bytes(self.tool_version.as_str().as_bytes())?;
        encoder.push_u128(self.candidate_ref.as_u128());
        encoder.push_bytes(self.requested_operation.as_str().as_bytes())?;
        push_optional_text(&mut encoder, self.requested_target.as_ref())?;
        encoder.push_bytes(self.authorized_resource_class.as_str().as_bytes())?;
        push_optional_digest(&mut encoder, self.target_identity_ref)?;
        push_optional_digest(&mut encoder, self.target_resolution_plan_ref)?;
        push_digest(&mut encoder, self.capability_context_ref)?;
        encoder.push_bytes(&self.taint_set.canonical_bytes()?)?;
        push_digest_list(&mut encoder, &self.provenance_refs)?;
        push_digest(&mut encoder, self.idempotency_material)?;
        push_digest_list(&mut encoder, &self.current_preconditions)?;
        encoder.push_u64(self.created_at_unix_ms);
        Ok(encoder.finish())
    }

    pub fn binding_digest(&self) -> Result<[u8; 32], ToolRequestError> {
        Ok(sha256(&self.canonical_bytes()?))
    }

    /// Consumes the mutable construction value and freezes its exact binding.
    /// A materially changed request must be constructed with a new request/effect identity.
    pub fn prepare(self) -> Result<PreparedToolRequest, ToolRequestError> {
        let binding_digest = self.binding_digest()?;
        Ok(PreparedToolRequest {
            request: self,
            binding_digest,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedToolRequest {
    request: ToolRequest,
    binding_digest: [u8; 32],
}

impl PreparedToolRequest {
    pub fn request(&self) -> &ToolRequest {
        &self.request
    }

    pub const fn binding_digest(&self) -> [u8; 32] {
        self.binding_digest
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ToolResultStatus {
    Succeeded,
    Failed,
    Rejected,
    Cancelled,
    TimedOut,
    UnknownOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolResult {
    pub request_id: ToolRequestId,
    pub status: ToolResultStatus,
    pub observed_target_identity: Option<BindingDigest>,
    pub output_artifact_refs: Vec<BindingDigest>,
    pub stdout_or_text_ref: Option<BindingDigest>,
    pub stderr_or_error_ref: Option<BindingDigest>,
    pub external_effect_refs: Vec<EffectId>,
    pub verification_refs: Vec<BindingDigest>,
    pub taint_set: TaintSet,
    pub started_at_unix_ms: u64,
    pub terminal_at_unix_ms: u64,
}

impl ToolResult {
    pub fn validate(&self) -> Result<(), ToolRequestError> {
        validate_ordered_unique(&self.output_artifact_refs, "output_artifact_refs")?;
        validate_effect_ids(&self.external_effect_refs)?;
        validate_ordered_unique(&self.verification_refs, "verification_refs")?;
        if self.terminal_at_unix_ms < self.started_at_unix_ms {
            return Err(ToolRequestError::InvalidTimeRange);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ToolRequestError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(TOOL_RESULT_DOMAIN)?;
        encoder.push_u128(self.request_id.as_u128());
        encoder.push_u8(result_status_code(self.status));
        push_optional_digest(&mut encoder, self.observed_target_identity)?;
        push_digest_list(&mut encoder, &self.output_artifact_refs)?;
        push_optional_digest(&mut encoder, self.stdout_or_text_ref)?;
        push_optional_digest(&mut encoder, self.stderr_or_error_ref)?;
        encoder.push_u64(self.external_effect_refs.len() as u64);
        for effect in &self.external_effect_refs {
            encoder.push_u128(effect.0);
        }
        push_digest_list(&mut encoder, &self.verification_refs)?;
        encoder.push_bytes(&self.taint_set.canonical_bytes()?)?;
        encoder.push_u64(self.started_at_unix_ms);
        encoder.push_u64(self.terminal_at_unix_ms);
        Ok(encoder.finish())
    }

    pub fn evidence_digest(&self) -> Result<[u8; 32], ToolRequestError> {
        Ok(sha256(&self.canonical_bytes()?))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolRequestError {
    InvalidIdentifier(&'static str),
    InvalidTarget,
    TooManyBindings(&'static str),
    UnsortedOrDuplicate(&'static str),
    MissingTargetBinding,
    AmbiguousTargetBinding,
    TargetBindingWithoutTarget,
    InvalidTimeRange,
    CanonicalEncoding(CoreError),
}

impl fmt::Display for ToolRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentifier(field) => write!(f, "invalid bounded identifier: {field}"),
            Self::InvalidTarget => f.write_str("requested target is empty, oversized, or invalid"),
            Self::TooManyBindings(field) => write!(f, "too many bounded bindings: {field}"),
            Self::UnsortedOrDuplicate(field) => {
                write!(f, "bindings must be strictly sorted and unique: {field}")
            }
            Self::MissingTargetBinding => f.write_str(
                "requested target requires an exact identity or bounded resolution plan",
            ),
            Self::AmbiguousTargetBinding => {
                f.write_str("requested target cannot bind both identity and resolution plan")
            }
            Self::TargetBindingWithoutTarget => {
                f.write_str("target binding cannot exist without requested target content")
            }
            Self::InvalidTimeRange => f.write_str("tool result terminal time precedes start time"),
            Self::CanonicalEncoding(error) => write!(f, "canonical encoding error: {error}"),
        }
    }
}

impl std::error::Error for ToolRequestError {}

impl From<CoreError> for ToolRequestError {
    fn from(value: CoreError) -> Self {
        Self::CanonicalEncoding(value)
    }
}

fn validate_identifier(value: &str, field: &'static str) -> Result<(), ToolRequestError> {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES {
        return Err(ToolRequestError::InvalidIdentifier(field));
    }
    if !value.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':' | b'+')
    }) {
        return Err(ToolRequestError::InvalidIdentifier(field));
    }
    Ok(())
}

fn validate_ordered_unique<T: Ord>(
    values: &[T],
    field: &'static str,
) -> Result<(), ToolRequestError> {
    if values.len() > MAX_BINDING_REFS {
        return Err(ToolRequestError::TooManyBindings(field));
    }
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(ToolRequestError::UnsortedOrDuplicate(field));
    }
    Ok(())
}

fn validate_effect_ids(values: &[EffectId]) -> Result<(), ToolRequestError> {
    if values.len() > MAX_BINDING_REFS {
        return Err(ToolRequestError::TooManyBindings("external_effect_refs"));
    }
    if values.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
        return Err(ToolRequestError::UnsortedOrDuplicate(
            "external_effect_refs",
        ));
    }
    Ok(())
}

fn push_digest(
    encoder: &mut CanonicalEncoder,
    digest: BindingDigest,
) -> Result<(), ToolRequestError> {
    encoder.push_bytes(&digest.bytes())?;
    Ok(())
}

fn push_optional_digest(
    encoder: &mut CanonicalEncoder,
    digest: Option<BindingDigest>,
) -> Result<(), ToolRequestError> {
    match digest {
        Some(value) => {
            encoder.push_u8(1);
            push_digest(encoder, value)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn push_optional_text(
    encoder: &mut CanonicalEncoder,
    target: Option<&RequestedTarget>,
) -> Result<(), ToolRequestError> {
    match target {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(value.as_str().as_bytes())?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn push_digest_list(
    encoder: &mut CanonicalEncoder,
    values: &[BindingDigest],
) -> Result<(), ToolRequestError> {
    encoder.push_u64(values.len() as u64);
    for value in values {
        push_digest(encoder, *value)?;
    }
    Ok(())
}

const fn result_status_code(status: ToolResultStatus) -> u8 {
    match status {
        ToolResultStatus::Succeeded => 1,
        ToolResultStatus::Failed => 2,
        ToolResultStatus::Rejected => 3,
        ToolResultStatus::Cancelled => 4,
        ToolResultStatus::TimedOut => 5,
        ToolResultStatus::UnknownOutcome => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::taint::TaintLabel;

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn request() -> ToolRequest {
        ToolRequest {
            request_id: ToolRequestId::from_u128(7),
            initiating_principal: PrincipalId::new("principal.local").unwrap(),
            tool_id: ToolId::new("fs.read").unwrap(),
            tool_version: ToolVersion::new("1.0.0").unwrap(),
            candidate_ref: ToolCallCandidateId::from_u128(9),
            requested_operation: RequestedOperationId::new("read").unwrap(),
            requested_target: Some(RequestedTarget::new("src/lib.rs").unwrap()),
            authorized_resource_class: ResourceClassId::new("workspace.read").unwrap(),
            target_identity_ref: Some(digest(1)),
            target_resolution_plan_ref: None,
            capability_context_ref: digest(2),
            taint_set: TaintSet::from_labels([TaintLabel::UserTrusted]),
            provenance_refs: vec![digest(3), digest(4)],
            idempotency_material: digest(5),
            current_preconditions: vec![digest(6)],
            created_at_unix_ms: 10,
        }
    }

    #[test]
    fn request_binding_is_deterministic_and_prepared_request_is_frozen() {
        let request = request();
        let first = request.binding_digest().unwrap();
        let second = request.binding_digest().unwrap();
        assert_eq!(first, second);

        let prepared = request.prepare().unwrap();
        assert_eq!(prepared.binding_digest(), first);
        assert_eq!(prepared.request().request_id, ToolRequestId::from_u128(7));
    }

    #[test]
    fn target_binding_is_exactly_one_of_identity_or_resolution_plan() {
        let mut value = request();
        value.target_identity_ref = None;
        assert_eq!(
            value.validate(),
            Err(ToolRequestError::MissingTargetBinding)
        );

        value.target_resolution_plan_ref = Some(digest(8));
        assert_eq!(value.validate(), Ok(()));

        value.target_identity_ref = Some(digest(1));
        assert_eq!(
            value.validate(),
            Err(ToolRequestError::AmbiguousTargetBinding)
        );
    }

    #[test]
    fn authority_relevant_lists_must_be_canonical() {
        let mut value = request();
        value.provenance_refs = vec![digest(4), digest(3)];
        assert_eq!(
            value.validate(),
            Err(ToolRequestError::UnsortedOrDuplicate("provenance_refs"))
        );
    }

    #[test]
    fn result_digest_is_evidence_not_verified_success() {
        let result = ToolResult {
            request_id: ToolRequestId::from_u128(7),
            status: ToolResultStatus::Succeeded,
            observed_target_identity: Some(digest(1)),
            output_artifact_refs: vec![digest(2)],
            stdout_or_text_ref: None,
            stderr_or_error_ref: None,
            external_effect_refs: vec![],
            verification_refs: vec![],
            taint_set: TaintSet::from_labels([TaintLabel::LocalTrusted]),
            started_at_unix_ms: 10,
            terminal_at_unix_ms: 11,
        };
        assert!(result.evidence_digest().is_ok());
        assert!(result.verification_refs.is_empty());
    }

    #[test]
    fn result_rejects_reverse_time_and_duplicate_effects() {
        let mut result = ToolResult {
            request_id: ToolRequestId::from_u128(7),
            status: ToolResultStatus::UnknownOutcome,
            observed_target_identity: None,
            output_artifact_refs: vec![],
            stdout_or_text_ref: None,
            stderr_or_error_ref: Some(digest(8)),
            external_effect_refs: vec![],
            verification_refs: vec![digest(9)],
            taint_set: TaintSet::empty(),
            started_at_unix_ms: 12,
            terminal_at_unix_ms: 11,
        };
        assert_eq!(result.validate(), Err(ToolRequestError::InvalidTimeRange));

        result.terminal_at_unix_ms = 12;
        result.external_effect_refs = vec![EffectId(4), EffectId(4)];
        assert_eq!(
            result.validate(),
            Err(ToolRequestError::UnsortedOrDuplicate(
                "external_effect_refs"
            ))
        );
    }
}
