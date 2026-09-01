#![forbid(unsafe_code)]

use crate::harness::{
    HarnessCoordinator, HarnessRunControl, ScriptStep, ScriptedBackend, stop, text_delta,
};
use golam_core::SessionId;
use golam_core::compaction::begin_deterministic_compaction;
use golam_core::context_projection::{
    ContextProjectionInput, build_context_projection, build_post_compaction_projection,
};
use golam_core::harness::{CompactionId, ExecutionProfileId, RequestAttemptId, RequestSeriesId};
use golam_core::harness_state::{ModelEvent, ModelRequest, RequestAttempt, RequestAttemptState};
use golam_core::model_backend::{
    HarnessEvidenceSink, HarnessEvidenceSinkError, ModelBackendFailureClass,
};
use golam_core::taint::{TaintLabel, TaintSet};

#[derive(Default)]
struct RecordingSink {
    prepared: Vec<RequestAttempt>,
    states: Vec<RequestAttempt>,
    events: Vec<ModelEvent>,
}

impl HarnessEvidenceSink for RecordingSink {
    fn persist_prepared_attempt(
        &mut self,
        _session_id: SessionId,
        attempt: &RequestAttempt,
        _record_bytes: &[u8],
    ) -> Result<(), HarnessEvidenceSinkError> {
        self.prepared.push(attempt.clone());
        Ok(())
    }

    fn persist_attempt_state(
        &mut self,
        _session_id: SessionId,
        attempt: &RequestAttempt,
        _record_bytes: &[u8],
    ) -> Result<(), HarnessEvidenceSinkError> {
        self.states.push(attempt.clone());
        Ok(())
    }

    fn append_model_event(
        &mut self,
        event: &ModelEvent,
        _record_bytes: &[u8],
    ) -> Result<(), HarnessEvidenceSinkError> {
        self.events.push(event.clone());
        Ok(())
    }
}

fn projection_input(projection_ref: &str, max_tokens: u32) -> ContextProjectionInput {
    ContextProjectionInput {
        projection_ref: projection_ref.into(),
        session_id: SessionId(7),
        execution_profile_id: ExecutionProfileId::from_u128(9),
        source_event_refs: vec!["event:user:1".into(), "event:assistant:1".into()],
        source_artifact_refs: vec!["artifact:context:1".into()],
        goal_refs: vec!["goal:7:version:3".into()],
        compaction_refs: Vec::new(),
        taint_refs: vec!["taint:model-generated".into()],
        source_taint: TaintSet::from_labels([TaintLabel::ModelGenerated]),
        max_tokens,
        render_policy_digest: [4; 32],
        rendered_digest: [5; 32],
        created_at_unix_ms: 10,
    }
}

fn request(attempt: u128, projection_ref: &str, request_digest: [u8; 32]) -> ModelRequest {
    ModelRequest {
        request_series_id: RequestSeriesId::from_u128(12),
        request_attempt_id: RequestAttemptId::from_u128(attempt),
        initiator_principal_ref: "principal:owner".into(),
        session_id: SessionId(7),
        turn_ref: "turn:4".into(),
        execution_profile_id: ExecutionProfileId::from_u128(9),
        context_projection_ref: projection_ref.into(),
        message_refs: vec!["event:user:1".into()],
        tool_schema_digest: None,
        max_input_tokens: 64,
        max_output_tokens: 32,
        max_runtime_ms: 100,
        request_digest,
    }
}

#[test]
fn context_overflow_compacts_reprojects_and_dispatches_fresh_attempt() {
    let backend = ScriptedBackend::new(vec![
        vec![ScriptStep::Fail(ModelBackendFailureClass::ContextOverflow)],
        vec![text_delta(0, b"reprojected-ok"), stop(1)],
    ]);
    let mut coordinator = HarnessCoordinator::new(backend);
    let mut sink = RecordingSink::default();

    let initial_projection =
        build_context_projection(projection_input("projection:before-compaction", 4096)).unwrap();
    let first = coordinator
        .run_attempt(
            &mut sink,
            &request(1, &initial_projection.projection_ref, [1; 32]),
            20,
            HarnessRunControl::default(),
        )
        .unwrap();
    assert_eq!(
        first.terminal_state,
        RequestAttemptState::FailedContextOverflow
    );
    assert!(first.needs_context_reprojection());

    let mut compaction = begin_deterministic_compaction(
        CompactionId::from_u128(30),
        &initial_projection,
        [6; 32],
        30,
    )
    .unwrap();
    compaction.begin_derivation().unwrap();
    let (compaction_attempt, artifact) = compaction.commit([7; 32], 31).unwrap();
    assert_eq!(
        compaction_attempt.state,
        golam_core::harness_state::CompactionState::Committed
    );

    let canonical_goal_refs = vec!["goal:7:version:3".into()];
    let reprojected = build_post_compaction_projection(
        projection_input("projection:after-compaction", 2048),
        &artifact,
        &canonical_goal_refs,
    )
    .unwrap();
    assert_eq!(reprojected.goal_refs, canonical_goal_refs);
    assert_eq!(reprojected.compaction_refs, [CompactionId::from_u128(30)]);

    let second = coordinator
        .run_attempt(
            &mut sink,
            &request(2, &reprojected.projection_ref, [2; 32]),
            40,
            HarnessRunControl::default(),
        )
        .unwrap();

    assert_eq!(second.terminal_state, RequestAttemptState::Completed);
    assert_eq!(first.request_series_id, second.request_series_id);
    assert_ne!(first.request_attempt_id, second.request_attempt_id);
    assert_eq!(sink.prepared.len(), 2);
    assert_eq!(
        sink.prepared[0].request_attempt_id,
        RequestAttemptId::from_u128(1)
    );
    assert_eq!(
        sink.prepared[1].request_attempt_id,
        RequestAttemptId::from_u128(2)
    );
    assert_eq!(coordinator.backend().starts(), 2);
}
