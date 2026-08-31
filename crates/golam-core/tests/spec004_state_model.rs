use golam_core::SessionId;
use golam_core::harness::{
    CompactionId, ExecutionProfileId, HardwareProfileId, RequestAttemptId, RequestSeriesId,
    ToolCallCandidateId,
};
use golam_core::harness_state::{
    BenchmarkMetrics, BenchmarkRecord, BenchmarkResult, CompactionArtifact, CompactionAttempt,
    CompactionState, ContextProjection, HardwarePrivacyClass, HardwareProfile,
    HardwareProfileSource, HarnessStateError, ModelEvent, ModelEventAcceptance, ModelEventKind,
    ModelRequest, RequestAttempt, RequestAttemptState, ToolCallCandidate, ToolCallParseStatus,
    ToolCallSourceMode,
};

#[test]
fn harness_identifiers_round_trip_at_boundary_values() {
    for value in [0, 1, u128::MAX] {
        let id = RequestAttemptId::from_u128(value);
        let text = id.to_string();
        assert_eq!(text.len(), 32);
        assert_eq!(text.parse::<RequestAttemptId>().unwrap(), id);
    }

    assert!(
        "fffffffffffffffffffffffffffffffff"
            .parse::<CompactionId>()
            .is_err()
    );
    assert!(
        "0000000000000000000000000000000g"
            .parse::<ToolCallCandidateId>()
            .is_err()
    );
}

#[test]
fn request_state_machine_rejects_terminal_rewrite() {
    let mut attempt = RequestAttempt {
        request_series_id: RequestSeriesId::from_u128(1),
        request_attempt_id: RequestAttemptId::from_u128(2),
        initiator_principal_ref: "principal:owner".into(),
        state: RequestAttemptState::Prepared,
        execution_profile_id: ExecutionProfileId::from_u128(3),
        request_digest: [4; 32],
        backend_instance_ref: Some("scripted:fixture".into()),
        accepted_event_refs: vec!["event:1".into()],
        accepted_output_digest: Some([5; 32]),
        failure_class: None,
        prepared_at_unix_ms: 10,
        terminal_at_unix_ms: None,
    };
    attempt.transition(RequestAttemptState::Dispatched).unwrap();
    attempt.transition(RequestAttemptState::Streaming).unwrap();
    attempt.transition(RequestAttemptState::Completed).unwrap();
    attempt.terminal_at_unix_ms = Some(20);
    attempt.validate().unwrap();

    assert_eq!(
        attempt.transition(RequestAttemptState::FailedTransient),
        Err(HarnessStateError::InvalidTransition)
    );
}

#[test]
fn request_and_event_bounds_fail_closed() {
    let request = ModelRequest {
        request_series_id: RequestSeriesId::from_u128(1),
        request_attempt_id: RequestAttemptId::from_u128(1),
        initiator_principal_ref: "principal:owner".into(),
        session_id: SessionId(1),
        turn_ref: "turn:1".into(),
        execution_profile_id: ExecutionProfileId::from_u128(1),
        context_projection_ref: "projection:1".into(),
        message_refs: vec!["event:1".into()],
        tool_schema_digest: None,
        max_input_tokens: 0,
        max_output_tokens: 1,
        max_runtime_ms: 1,
        request_digest: [1; 32],
    };
    assert_eq!(request.validate(), Err(HarnessStateError::InvalidBounds));

    let event = ModelEvent {
        request_attempt_id: RequestAttemptId::from_u128(1),
        sequence: 0,
        kind: ModelEventKind::TextDelta,
        payload: vec![0; (1024 * 1024) + 1],
        acceptance: ModelEventAcceptance::RejectedOversized,
        canonical_evidence_ref: None,
    };
    assert_eq!(event.validate(), Err(HarnessStateError::PayloadTooLarge));
}

#[test]
fn validated_tool_candidate_requires_complete_non_authority_payload() {
    let candidate = ToolCallCandidate {
        candidate_id: ToolCallCandidateId::from_u128(1),
        request_attempt_id: RequestAttemptId::from_u128(2),
        source_mode: ToolCallSourceMode::GrammarConstrained,
        source_event_refs: vec!["event:tool-fragment".into()],
        requested_tool_name: Some("read_fixture".into()),
        schema_digest: Some([2; 32]),
        arguments_digest: Some([3; 32]),
        parse_status: ToolCallParseStatus::ValidatedCandidate,
        candidate_digest: [4; 32],
    };
    candidate.validate().unwrap();

    let mut incomplete = candidate;
    incomplete.schema_digest = None;
    assert_eq!(
        incomplete.validate(),
        Err(HarnessStateError::InvalidBounds)
    );
}

#[test]
fn compaction_provenance_distinguishes_deterministic_and_model_backed() {
    let deterministic = CompactionArtifact {
        compaction_id: CompactionId::from_u128(1),
        source_projection_ref: "projection:1".into(),
        source_event_refs: vec!["event:1".into()],
        goal_refs: vec!["goal:1".into()],
        deterministic: true,
        producing_request_attempt_id: None,
        accepted_output_ref: None,
        artifact_digest: [1; 32],
    };
    deterministic.validate().unwrap();

    let model_backed = CompactionArtifact {
        deterministic: false,
        producing_request_attempt_id: Some(RequestAttemptId::from_u128(9)),
        accepted_output_ref: Some("event:model-output".into()),
        ..deterministic.clone()
    };
    model_backed.validate().unwrap();

    let invalid = CompactionArtifact {
        accepted_output_ref: None,
        ..model_backed
    };
    assert_eq!(invalid.validate(), Err(HarnessStateError::InvalidBounds));
}

#[test]
fn projection_hardware_and_benchmark_bounds_are_independent() {
    let projection = ContextProjection {
        projection_ref: "projection:1".into(),
        session_id: SessionId(1),
        execution_profile_id: ExecutionProfileId::from_u128(1),
        source_event_refs: vec!["event:1".into()],
        source_artifact_refs: Vec::new(),
        goal_refs: vec!["goal:1".into()],
        compaction_refs: Vec::new(),
        taint_refs: Vec::new(),
        max_tokens: 0,
        render_policy_digest: [1; 32],
        rendered_digest: [2; 32],
        created_at_unix_ms: 1,
    };
    assert_eq!(projection.validate(), Err(HarnessStateError::InvalidBounds));

    let hardware = HardwareProfile {
        hardware_profile_id: HardwareProfileId::from_u128(1),
        observed_at_unix_ms: 1,
        platform: "linux".into(),
        architecture: "x86_64".into(),
        cpu_capabilities: Vec::new(),
        memory_capacity_or_bucket: "fixture".into(),
        accelerators: Vec::new(),
        backend_capabilities: Vec::new(),
        source: HardwareProfileSource::Fixture,
        privacy_class: HardwarePrivacyClass::FixtureSynthetic,
        content_digest: [3; 32],
    };
    hardware.validate().unwrap();

    let benchmark = BenchmarkRecord {
        benchmark_id: 1,
        schema_version: 1,
        code_revision: "fixture".into(),
        execution_profile_id: ExecutionProfileId::from_u128(1),
        hardware_profile_id: hardware.hardware_profile_id,
        workload_fixture_id: "harness-only".into(),
        backend_metrics: vec![BenchmarkMetrics {
            metric_name: "backend_events".into(),
            value_milli: 1_000,
            unit: "count_milli".into(),
        }],
        harness_metrics: vec![BenchmarkMetrics {
            metric_name: "accepted_events".into(),
            value_milli: 1_000,
            unit: "count_milli".into(),
        }],
        started_at_unix_ms: 10,
        finished_at_unix_ms: 9,
        result: BenchmarkResult::Invalidated,
        raw_evidence_refs: vec!["evidence:fixture".into()],
    };
    assert_eq!(benchmark.validate(), Err(HarnessStateError::InvalidBounds));
}

#[test]
fn compaction_terminal_states_remain_absorbing() {
    let mut attempt = CompactionAttempt {
        compaction_id: CompactionId::from_u128(1),
        session_id: SessionId(1),
        source_projection_ref: "projection:1".into(),
        state: CompactionState::Started,
        deterministic: true,
        producing_request_attempt_id: None,
        started_at_unix_ms: 1,
        terminal_at_unix_ms: None,
        failure_class: None,
    };
    attempt.transition(CompactionState::Deriving).unwrap();
    attempt.transition(CompactionState::Validating).unwrap();
    attempt.transition(CompactionState::Committed).unwrap();
    attempt.terminal_at_unix_ms = Some(2);
    attempt.validate().unwrap();
    assert_eq!(
        attempt.transition(CompactionState::Deriving),
        Err(HarnessStateError::InvalidTransition)
    );
}
