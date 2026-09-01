use crate::SessionId;
use crate::harness::{
    CompactionId, ExecutionProfileId, HardwareProfileId, RequestAttemptId, RequestSeriesId,
    ToolCallCandidateId,
};

const MAX_REF_BYTES: usize = 1024;
const MAX_LIST_ITEMS: usize = 256;
const MAX_MODEL_EVENT_PAYLOAD_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HarnessStateError {
    EmptyReference,
    ReferenceTooLong,
    TooManyItems,
    PayloadTooLarge,
    InvalidTransition,
    InvalidBounds,
}

fn validate_ref(value: &str) -> Result<(), HarnessStateError> {
    if value.is_empty() {
        return Err(HarnessStateError::EmptyReference);
    }
    if value.len() > MAX_REF_BYTES {
        return Err(HarnessStateError::ReferenceTooLong);
    }
    Ok(())
}

fn validate_refs(values: &[String]) -> Result<(), HarnessStateError> {
    if values.len() > MAX_LIST_ITEMS {
        return Err(HarnessStateError::TooManyItems);
    }
    for value in values {
        validate_ref(value)?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HardwareProfileSource {
    LocalProbe,
    Fixture,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HardwarePrivacyClass {
    LocalOperational,
    FixtureSynthetic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeasurementStatus {
    Observed,
    Unavailable,
    Unsupported,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceleratorObservation {
    pub backend_device_id: String,
    pub device_class: String,
    pub memory_capacity_bytes: Option<u64>,
    pub feature_flags: Vec<String>,
    pub measurement_status: MeasurementStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HardwareProfile {
    pub hardware_profile_id: HardwareProfileId,
    pub observed_at_unix_ms: u64,
    pub platform: String,
    pub architecture: String,
    pub cpu_capabilities: Vec<String>,
    pub memory_capacity_or_bucket: String,
    pub accelerators: Vec<AcceleratorObservation>,
    pub backend_capabilities: Vec<String>,
    pub source: HardwareProfileSource,
    pub privacy_class: HardwarePrivacyClass,
    pub content_digest: [u8; 32],
}

impl HardwareProfile {
    pub fn validate(&self) -> Result<(), HarnessStateError> {
        validate_ref(&self.platform)?;
        validate_ref(&self.architecture)?;
        validate_ref(&self.memory_capacity_or_bucket)?;
        validate_refs(&self.cpu_capabilities)?;
        validate_refs(&self.backend_capabilities)?;
        if self.accelerators.len() > MAX_LIST_ITEMS {
            return Err(HarnessStateError::TooManyItems);
        }
        for accelerator in &self.accelerators {
            validate_ref(&accelerator.backend_device_id)?;
            validate_ref(&accelerator.device_class)?;
            validate_refs(&accelerator.feature_flags)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalibrationResult {
    Supported,
    Unsupported,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalibrationObservation {
    pub metric: String,
    pub value_milli: i64,
    pub unit: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalibrationRun {
    pub calibration_id: u128,
    pub hardware_profile_id: HardwareProfileId,
    pub backend_identity_ref: String,
    pub profile_candidate_digest: [u8; 32],
    pub workload_fixture_id: String,
    pub started_at_unix_ms: u64,
    pub finished_at_unix_ms: Option<u64>,
    pub max_memory_bytes: u64,
    pub max_runtime_ms: u64,
    pub observations: Vec<CalibrationObservation>,
    pub result: CalibrationResult,
    pub failure_class: Option<String>,
    pub evidence_refs: Vec<String>,
}

impl CalibrationRun {
    pub fn validate(&self) -> Result<(), HarnessStateError> {
        validate_ref(&self.backend_identity_ref)?;
        validate_ref(&self.workload_fixture_id)?;
        if self.max_memory_bytes == 0 || self.max_runtime_ms == 0 {
            return Err(HarnessStateError::InvalidBounds);
        }
        if let Some(finished) = self.finished_at_unix_ms
            && finished < self.started_at_unix_ms
        {
            return Err(HarnessStateError::InvalidBounds);
        }
        if self.observations.len() > MAX_LIST_ITEMS {
            return Err(HarnessStateError::TooManyItems);
        }
        for observation in &self.observations {
            validate_ref(&observation.metric)?;
            validate_ref(&observation.unit)?;
        }
        if let Some(failure_class) = &self.failure_class {
            validate_ref(failure_class)?;
        }
        validate_refs(&self.evidence_refs)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRequest {
    pub request_series_id: RequestSeriesId,
    pub request_attempt_id: RequestAttemptId,
    pub initiator_principal_ref: String,
    pub session_id: SessionId,
    pub turn_ref: String,
    pub execution_profile_id: ExecutionProfileId,
    pub context_projection_ref: String,
    pub message_refs: Vec<String>,
    pub tool_schema_digest: Option<[u8; 32]>,
    pub max_input_tokens: u32,
    pub max_output_tokens: u32,
    pub max_runtime_ms: u64,
    pub request_digest: [u8; 32],
}

impl ModelRequest {
    pub fn validate(&self) -> Result<(), HarnessStateError> {
        validate_ref(&self.initiator_principal_ref)?;
        validate_ref(&self.turn_ref)?;
        validate_ref(&self.context_projection_ref)?;
        validate_refs(&self.message_refs)?;
        if self.max_input_tokens == 0 || self.max_output_tokens == 0 || self.max_runtime_ms == 0 {
            return Err(HarnessStateError::InvalidBounds);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestAttemptState {
    Prepared,
    Dispatched,
    Streaming,
    CancelRequested,
    Completed,
    Cancelled,
    TimedOut,
    FailedTransient,
    FailedDeterministic,
    FailedContextOverflow,
}

impl RequestAttemptState {
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed
                | Self::Cancelled
                | Self::TimedOut
                | Self::FailedTransient
                | Self::FailedDeterministic
                | Self::FailedContextOverflow
        )
    }

    pub const fn can_transition_to(self, next: Self) -> bool {
        match self {
            Self::Prepared => matches!(
                next,
                Self::Dispatched | Self::CancelRequested | Self::Cancelled
            ),
            Self::Dispatched => matches!(
                next,
                Self::Streaming
                    | Self::CancelRequested
                    | Self::Completed
                    | Self::Cancelled
                    | Self::TimedOut
                    | Self::FailedTransient
                    | Self::FailedDeterministic
                    | Self::FailedContextOverflow
            ),
            Self::Streaming => matches!(
                next,
                Self::Streaming
                    | Self::CancelRequested
                    | Self::Completed
                    | Self::Cancelled
                    | Self::TimedOut
                    | Self::FailedTransient
                    | Self::FailedDeterministic
                    | Self::FailedContextOverflow
            ),
            Self::CancelRequested => matches!(
                next,
                Self::Streaming
                    | Self::Completed
                    | Self::Cancelled
                    | Self::TimedOut
                    | Self::FailedTransient
                    | Self::FailedDeterministic
            ),
            Self::Completed
            | Self::Cancelled
            | Self::TimedOut
            | Self::FailedTransient
            | Self::FailedDeterministic
            | Self::FailedContextOverflow => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestAttempt {
    pub request_series_id: RequestSeriesId,
    pub request_attempt_id: RequestAttemptId,
    pub initiator_principal_ref: String,
    pub state: RequestAttemptState,
    pub execution_profile_id: ExecutionProfileId,
    pub request_digest: [u8; 32],
    pub backend_instance_ref: Option<String>,
    pub accepted_event_refs: Vec<String>,
    pub accepted_output_digest: Option<[u8; 32]>,
    pub failure_class: Option<String>,
    pub prepared_at_unix_ms: u64,
    pub terminal_at_unix_ms: Option<u64>,
}

impl RequestAttempt {
    pub fn transition(&mut self, next: RequestAttemptState) -> Result<(), HarnessStateError> {
        if !self.state.can_transition_to(next) {
            return Err(HarnessStateError::InvalidTransition);
        }
        self.state = next;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), HarnessStateError> {
        validate_ref(&self.initiator_principal_ref)?;
        if let Some(backend_instance_ref) = &self.backend_instance_ref {
            validate_ref(backend_instance_ref)?;
        }
        validate_refs(&self.accepted_event_refs)?;
        if let Some(failure_class) = &self.failure_class {
            validate_ref(failure_class)?;
        }
        if self.state.is_terminal() != self.terminal_at_unix_ms.is_some() {
            return Err(HarnessStateError::InvalidBounds);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelEventKind {
    TextDelta,
    ReasoningDelta,
    ToolCallFragment,
    ToolCallComplete,
    Usage,
    Stop,
    BackendWarning,
    BackendError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelEventAcceptance {
    Accepted,
    RejectedMalformed,
    RejectedOversized,
    RejectedOutOfOrder,
    RejectedAfterTerminal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelEvent {
    pub request_attempt_id: RequestAttemptId,
    pub sequence: u64,
    pub kind: ModelEventKind,
    pub payload: Vec<u8>,
    pub acceptance: ModelEventAcceptance,
    pub canonical_evidence_ref: Option<String>,
}

impl ModelEvent {
    pub fn validate(&self) -> Result<(), HarnessStateError> {
        if self.payload.len() > MAX_MODEL_EVENT_PAYLOAD_BYTES {
            return Err(HarnessStateError::PayloadTooLarge);
        }
        if let Some(reference) = &self.canonical_evidence_ref {
            validate_ref(reference)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolCallParseStatus {
    ValidatedCandidate,
    RejectedMalformed,
    RejectedOversized,
    RejectedUnknownTool,
    RejectedSchema,
    RejectedAmbiguous,
    RejectedDuplicate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolCallSourceMode {
    NativeTools,
    GrammarConstrained,
    TextProtocolFallback,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolCallCandidate {
    pub candidate_id: ToolCallCandidateId,
    pub request_attempt_id: RequestAttemptId,
    pub source_mode: ToolCallSourceMode,
    pub source_event_refs: Vec<String>,
    pub requested_tool_name: Option<String>,
    pub schema_digest: Option<[u8; 32]>,
    pub arguments_digest: Option<[u8; 32]>,
    pub parse_status: ToolCallParseStatus,
    pub candidate_digest: [u8; 32],
}

impl ToolCallCandidate {
    pub fn validate(&self) -> Result<(), HarnessStateError> {
        validate_refs(&self.source_event_refs)?;
        if let Some(tool_name) = &self.requested_tool_name {
            validate_ref(tool_name)?;
        }
        if self.parse_status == ToolCallParseStatus::ValidatedCandidate
            && (self.requested_tool_name.is_none()
                || self.schema_digest.is_none()
                || self.arguments_digest.is_none())
        {
            return Err(HarnessStateError::InvalidBounds);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextProjection {
    pub projection_ref: String,
    pub session_id: SessionId,
    pub execution_profile_id: ExecutionProfileId,
    pub source_event_refs: Vec<String>,
    pub source_artifact_refs: Vec<String>,
    pub goal_refs: Vec<String>,
    pub compaction_refs: Vec<CompactionId>,
    pub taint_refs: Vec<String>,
    pub max_tokens: u32,
    pub render_policy_digest: [u8; 32],
    pub rendered_digest: [u8; 32],
    pub created_at_unix_ms: u64,
}

impl ContextProjection {
    pub fn validate(&self) -> Result<(), HarnessStateError> {
        validate_ref(&self.projection_ref)?;
        validate_refs(&self.source_event_refs)?;
        validate_refs(&self.source_artifact_refs)?;
        validate_refs(&self.goal_refs)?;
        validate_refs(&self.taint_refs)?;
        if self.compaction_refs.len() > MAX_LIST_ITEMS || self.max_tokens == 0 {
            return Err(HarnessStateError::InvalidBounds);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompactionState {
    Started,
    Deriving,
    Validating,
    Committed,
    Cancelled,
    FailedChangedSource,
    FailedTransient,
    FailedDeterministic,
    FailedPersistence,
}

impl CompactionState {
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Committed
                | Self::Cancelled
                | Self::FailedChangedSource
                | Self::FailedTransient
                | Self::FailedDeterministic
                | Self::FailedPersistence
        )
    }

    pub const fn can_transition_to(self, next: Self) -> bool {
        match self {
            Self::Started => matches!(
                next,
                Self::Deriving
                    | Self::Cancelled
                    | Self::FailedChangedSource
                    | Self::FailedPersistence
            ),
            Self::Deriving => matches!(
                next,
                Self::Validating
                    | Self::Cancelled
                    | Self::FailedChangedSource
                    | Self::FailedTransient
                    | Self::FailedDeterministic
                    | Self::FailedPersistence
            ),
            Self::Validating => matches!(
                next,
                Self::Committed
                    | Self::Cancelled
                    | Self::FailedChangedSource
                    | Self::FailedDeterministic
                    | Self::FailedPersistence
            ),
            Self::Committed
            | Self::Cancelled
            | Self::FailedChangedSource
            | Self::FailedTransient
            | Self::FailedDeterministic
            | Self::FailedPersistence => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactionAttempt {
    pub compaction_id: CompactionId,
    pub session_id: SessionId,
    pub source_projection_ref: String,
    pub state: CompactionState,
    pub deterministic: bool,
    pub producing_request_attempt_id: Option<RequestAttemptId>,
    pub started_at_unix_ms: u64,
    pub terminal_at_unix_ms: Option<u64>,
    pub failure_class: Option<String>,
}

impl CompactionAttempt {
    pub fn transition(&mut self, next: CompactionState) -> Result<(), HarnessStateError> {
        if !self.state.can_transition_to(next) {
            return Err(HarnessStateError::InvalidTransition);
        }
        self.state = next;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), HarnessStateError> {
        validate_ref(&self.source_projection_ref)?;
        if self.deterministic && self.producing_request_attempt_id.is_some() {
            return Err(HarnessStateError::InvalidBounds);
        }
        if !self.deterministic && self.producing_request_attempt_id.is_none() {
            return Err(HarnessStateError::InvalidBounds);
        }
        if self.state.is_terminal() != self.terminal_at_unix_ms.is_some() {
            return Err(HarnessStateError::InvalidBounds);
        }
        if let Some(failure_class) = &self.failure_class {
            validate_ref(failure_class)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactionArtifact {
    pub compaction_id: CompactionId,
    pub source_projection_ref: String,
    pub source_event_refs: Vec<String>,
    pub goal_refs: Vec<String>,
    pub deterministic: bool,
    pub producing_request_attempt_id: Option<RequestAttemptId>,
    pub accepted_output_ref: Option<String>,
    pub artifact_digest: [u8; 32],
}

impl CompactionArtifact {
    pub fn validate(&self) -> Result<(), HarnessStateError> {
        validate_ref(&self.source_projection_ref)?;
        validate_refs(&self.source_event_refs)?;
        validate_refs(&self.goal_refs)?;
        match (
            self.deterministic,
            self.producing_request_attempt_id,
            self.accepted_output_ref.as_ref(),
        ) {
            (true, None, None) => Ok(()),
            (false, Some(_), Some(output_ref)) => validate_ref(output_ref),
            _ => Err(HarnessStateError::InvalidBounds),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkMetrics {
    pub metric_name: String,
    pub value_milli: i64,
    pub unit: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BenchmarkResult {
    Passed,
    Failed,
    Cancelled,
    Invalidated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BenchmarkRecord {
    pub benchmark_id: u128,
    pub schema_version: u16,
    pub code_revision: String,
    pub execution_profile_id: ExecutionProfileId,
    pub hardware_profile_id: HardwareProfileId,
    pub workload_fixture_id: String,
    pub backend_metrics: Vec<BenchmarkMetrics>,
    pub harness_metrics: Vec<BenchmarkMetrics>,
    pub started_at_unix_ms: u64,
    pub finished_at_unix_ms: u64,
    pub result: BenchmarkResult,
    pub raw_evidence_refs: Vec<String>,
}

impl BenchmarkRecord {
    pub fn validate(&self) -> Result<(), HarnessStateError> {
        if self.schema_version == 0 || self.finished_at_unix_ms < self.started_at_unix_ms {
            return Err(HarnessStateError::InvalidBounds);
        }
        validate_ref(&self.code_revision)?;
        validate_ref(&self.workload_fixture_id)?;
        if self.backend_metrics.len() > MAX_LIST_ITEMS
            || self.harness_metrics.len() > MAX_LIST_ITEMS
        {
            return Err(HarnessStateError::TooManyItems);
        }
        for metric in self
            .backend_metrics
            .iter()
            .chain(self.harness_metrics.iter())
        {
            validate_ref(&metric.metric_name)?;
            validate_ref(&metric.unit)?;
        }
        validate_refs(&self.raw_evidence_refs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_terminal_states_are_absorbing() {
        for terminal in [
            RequestAttemptState::Completed,
            RequestAttemptState::Cancelled,
            RequestAttemptState::TimedOut,
            RequestAttemptState::FailedTransient,
            RequestAttemptState::FailedDeterministic,
            RequestAttemptState::FailedContextOverflow,
        ] {
            assert!(terminal.is_terminal());
            assert!(!terminal.can_transition_to(RequestAttemptState::Streaming));
        }
    }

    #[test]
    fn request_lifecycle_rejects_rewrite_after_completion() {
        let mut attempt = RequestAttempt {
            request_series_id: RequestSeriesId::from_u128(1),
            request_attempt_id: RequestAttemptId::from_u128(1),
            initiator_principal_ref: "principal:local:1".into(),
            state: RequestAttemptState::Prepared,
            execution_profile_id: ExecutionProfileId::from_u128(1),
            request_digest: [1; 32],
            backend_instance_ref: None,
            accepted_event_refs: Vec::new(),
            accepted_output_digest: None,
            failure_class: None,
            prepared_at_unix_ms: 1,
            terminal_at_unix_ms: None,
        };
        attempt.transition(RequestAttemptState::Dispatched).unwrap();
        attempt.transition(RequestAttemptState::Streaming).unwrap();
        attempt.transition(RequestAttemptState::Completed).unwrap();
        attempt.terminal_at_unix_ms = Some(2);
        attempt.validate().unwrap();
        assert_eq!(
            attempt.transition(RequestAttemptState::Streaming),
            Err(HarnessStateError::InvalidTransition)
        );
    }

    #[test]
    fn model_event_payload_is_bounded() {
        let event = ModelEvent {
            request_attempt_id: RequestAttemptId::from_u128(1),
            sequence: 0,
            kind: ModelEventKind::TextDelta,
            payload: vec![0; MAX_MODEL_EVENT_PAYLOAD_BYTES + 1],
            acceptance: ModelEventAcceptance::RejectedOversized,
            canonical_evidence_ref: None,
        };
        assert_eq!(event.validate(), Err(HarnessStateError::PayloadTooLarge));
    }

    #[test]
    fn validated_tool_candidate_requires_schema_and_arguments() {
        let candidate = ToolCallCandidate {
            candidate_id: ToolCallCandidateId::from_u128(1),
            request_attempt_id: RequestAttemptId::from_u128(1),
            source_mode: ToolCallSourceMode::NativeTools,
            source_event_refs: vec!["event:1".into()],
            requested_tool_name: Some("read".into()),
            schema_digest: None,
            arguments_digest: None,
            parse_status: ToolCallParseStatus::ValidatedCandidate,
            candidate_digest: [2; 32],
        };
        assert_eq!(candidate.validate(), Err(HarnessStateError::InvalidBounds));
    }

    #[test]
    fn deterministic_compaction_cannot_claim_model_attempt() {
        let artifact = CompactionArtifact {
            compaction_id: CompactionId::from_u128(1),
            source_projection_ref: "projection:1".into(),
            source_event_refs: vec!["event:1".into()],
            goal_refs: vec!["goal:1".into()],
            deterministic: true,
            producing_request_attempt_id: Some(RequestAttemptId::from_u128(1)),
            accepted_output_ref: None,
            artifact_digest: [3; 32],
        };
        assert_eq!(artifact.validate(), Err(HarnessStateError::InvalidBounds));
    }

    #[test]
    fn compaction_terminal_states_are_absorbing() {
        assert!(CompactionState::Committed.is_terminal());
        assert!(!CompactionState::Committed.can_transition_to(CompactionState::Deriving));
    }
}
