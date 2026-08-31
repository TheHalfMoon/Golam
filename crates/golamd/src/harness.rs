#![forbid(unsafe_code)]

use std::collections::{HashSet, VecDeque};

use golam_core::harness::{RequestAttemptId, RequestSeriesId};
use golam_core::harness_state::{
    ModelEvent, ModelEventAcceptance, ModelEventKind, ModelRequest, RequestAttempt,
    RequestAttemptState,
};
use golam_core::model_backend::{
    BackendEmission, HarnessEvidenceSink, HarnessEvidenceSinkError, ModelBackend,
    ModelBackendError, ModelBackendFailureClass, ModelBackendSession,
};

const ATTEMPT_RECORD_V1: &[u8] = b"golam-harness-attempt-v1";
const EVENT_RECORD_V1: &[u8] = b"golam-harness-event-v1";
const DEFAULT_MAX_POLLS: u64 = 4096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HarnessRunControl {
    pub cancel_after_polls: Option<u64>,
    pub poll_duration_ms: u64,
    pub max_polls: u64,
}

impl Default for HarnessRunControl {
    fn default() -> Self {
        Self {
            cancel_after_polls: None,
            poll_duration_ms: 1,
            max_polls: DEFAULT_MAX_POLLS,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HarnessAttemptOutcome {
    pub request_series_id: RequestSeriesId,
    pub request_attempt_id: RequestAttemptId,
    pub terminal_state: RequestAttemptState,
    pub accepted_event_refs: Vec<String>,
    pub failure_class: Option<String>,
}

impl HarnessAttemptOutcome {
    pub fn retryable_transient(&self) -> bool {
        self.terminal_state == RequestAttemptState::FailedTransient
    }

    pub fn needs_context_reprojection(&self) -> bool {
        self.terminal_state == RequestAttemptState::FailedContextOverflow
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HarnessRunError {
    InvalidRequest,
    InvalidRetrySeries,
    Evidence(String),
    Backend(ModelBackendError),
}

impl From<HarnessEvidenceSinkError> for HarnessRunError {
    fn from(value: HarnessEvidenceSinkError) -> Self {
        Self::Evidence(value.message)
    }
}

pub struct HarnessCoordinator<B> {
    backend: B,
}

impl<B: ModelBackend> HarnessCoordinator<B> {
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn run_attempt<E: HarnessEvidenceSink>(
        &mut self,
        sink: &mut E,
        request: &ModelRequest,
        prepared_at_unix_ms: u64,
        control: HarnessRunControl,
    ) -> Result<HarnessAttemptOutcome, HarnessRunError> {
        request
            .validate()
            .map_err(|_| HarnessRunError::InvalidRequest)?;
        if control.poll_duration_ms == 0 || control.max_polls == 0 {
            return Err(HarnessRunError::InvalidRequest);
        }

        let mut attempt = RequestAttempt {
            request_series_id: request.request_series_id,
            request_attempt_id: request.request_attempt_id,
            initiator_principal_ref: request.initiator_principal_ref.clone(),
            state: RequestAttemptState::Prepared,
            execution_profile_id: request.execution_profile_id,
            request_digest: request.request_digest,
            backend_instance_ref: None,
            accepted_event_refs: Vec::new(),
            accepted_output_digest: None,
            failure_class: None,
            prepared_at_unix_ms,
            terminal_at_unix_ms: None,
        };
        sink.persist_prepared_attempt(request.session_id, &attempt, ATTEMPT_RECORD_V1)?;

        let mut session = match self.backend.start(request) {
            Ok(session) => session,
            Err(error) => {
                finish_backend_error(sink, request, &mut attempt, prepared_at_unix_ms, &error)?;
                return Ok(outcome(&attempt));
            }
        };

        attempt.backend_instance_ref = Some(session.backend_instance_ref().to_owned());
        transition_and_persist(sink, request, &mut attempt, RequestAttemptState::Dispatched)?;

        let mut expected_sequence = 0_u64;
        let mut polls = 0_u64;
        loop {
            let elapsed_ms = polls.saturating_mul(control.poll_duration_ms);
            let now = prepared_at_unix_ms.saturating_add(elapsed_ms);

            if control.cancel_after_polls == Some(polls) {
                transition_and_persist(
                    sink,
                    request,
                    &mut attempt,
                    RequestAttemptState::CancelRequested,
                )?;
                session.request_cancel().map_err(HarnessRunError::Backend)?;
                finish_terminal(
                    sink,
                    request,
                    &mut attempt,
                    RequestAttemptState::Cancelled,
                    now,
                    None,
                )?;
                return Ok(outcome(&attempt));
            }

            if elapsed_ms >= request.max_runtime_ms {
                session.request_cancel().map_err(HarnessRunError::Backend)?;
                finish_terminal(
                    sink,
                    request,
                    &mut attempt,
                    RequestAttemptState::TimedOut,
                    now,
                    Some("request_timeout"),
                )?;
                return Ok(outcome(&attempt));
            }

            if polls >= control.max_polls {
                finish_terminal(
                    sink,
                    request,
                    &mut attempt,
                    RequestAttemptState::FailedDeterministic,
                    now,
                    Some("max_backend_polls_exceeded"),
                )?;
                return Ok(outcome(&attempt));
            }

            let emission = match session.next_emission() {
                Ok(Some(emission)) => emission,
                Ok(None) => {
                    finish_terminal(
                        sink,
                        request,
                        &mut attempt,
                        RequestAttemptState::FailedDeterministic,
                        now,
                        Some("backend_ended_without_stop"),
                    )?;
                    return Ok(outcome(&attempt));
                }
                Err(error) => {
                    finish_backend_error(sink, request, &mut attempt, now, &error)?;
                    return Ok(outcome(&attempt));
                }
            };
            polls = polls.saturating_add(1);

            if emission.validate().is_err() || emission.sequence != expected_sequence {
                finish_terminal(
                    sink,
                    request,
                    &mut attempt,
                    RequestAttemptState::FailedDeterministic,
                    now,
                    Some("invalid_backend_event_order_or_size"),
                )?;
                return Ok(outcome(&attempt));
            }
            expected_sequence = expected_sequence.saturating_add(1);

            let evidence_ref = canonical_event_ref(request.request_attempt_id, emission.sequence);
            let event = ModelEvent {
                request_attempt_id: request.request_attempt_id,
                sequence: emission.sequence,
                kind: emission.kind,
                payload: emission.payload,
                acceptance: ModelEventAcceptance::Accepted,
                canonical_evidence_ref: Some(evidence_ref.clone()),
            };
            event
                .validate()
                .map_err(|_| HarnessRunError::InvalidRequest)?;
            sink.append_model_event(&event, EVENT_RECORD_V1)?;
            attempt.accepted_event_refs.push(evidence_ref);

            if attempt.state == RequestAttemptState::Dispatched {
                attempt
                    .transition(RequestAttemptState::Streaming)
                    .map_err(|_| HarnessRunError::InvalidRequest)?;
            }

            if event.kind == ModelEventKind::Stop {
                finish_terminal(
                    sink,
                    request,
                    &mut attempt,
                    RequestAttemptState::Completed,
                    prepared_at_unix_ms
                        .saturating_add(polls.saturating_mul(control.poll_duration_ms)),
                    None,
                )?;
                return Ok(outcome(&attempt));
            }

            sink.persist_attempt_state(request.session_id, &attempt, ATTEMPT_RECORD_V1)?;
        }
    }

    pub fn run_retry_series<E: HarnessEvidenceSink>(
        &mut self,
        sink: &mut E,
        requests: &[ModelRequest],
        prepared_at_unix_ms: u64,
        control: HarnessRunControl,
    ) -> Result<Vec<HarnessAttemptOutcome>, HarnessRunError> {
        validate_retry_requests(requests)?;
        let mut outcomes = Vec::new();
        for request in requests {
            let result = self.run_attempt(sink, request, prepared_at_unix_ms, control)?;
            let retry = result.retryable_transient();
            let reprojection = result.needs_context_reprojection();
            outcomes.push(result);
            if reprojection || !retry {
                break;
            }
        }
        Ok(outcomes)
    }
}

fn validate_retry_requests(requests: &[ModelRequest]) -> Result<(), HarnessRunError> {
    let Some(first) = requests.first() else {
        return Err(HarnessRunError::InvalidRetrySeries);
    };
    let mut ids = HashSet::new();
    for request in requests {
        if request.request_series_id != first.request_series_id
            || !ids.insert(request.request_attempt_id)
            || request.validate().is_err()
        {
            return Err(HarnessRunError::InvalidRetrySeries);
        }
    }
    Ok(())
}

fn transition_and_persist<E: HarnessEvidenceSink>(
    sink: &mut E,
    request: &ModelRequest,
    attempt: &mut RequestAttempt,
    state: RequestAttemptState,
) -> Result<(), HarnessRunError> {
    attempt
        .transition(state)
        .map_err(|_| HarnessRunError::InvalidRequest)?;
    sink.persist_attempt_state(request.session_id, attempt, ATTEMPT_RECORD_V1)?;
    Ok(())
}

fn finish_backend_error<E: HarnessEvidenceSink>(
    sink: &mut E,
    request: &ModelRequest,
    attempt: &mut RequestAttempt,
    terminal_time: u64,
    error: &ModelBackendError,
) -> Result<(), HarnessRunError> {
    let (state, failure) = match error.class {
        ModelBackendFailureClass::Transient => {
            (RequestAttemptState::FailedTransient, "backend_transient")
        }
        ModelBackendFailureClass::Deterministic => (
            RequestAttemptState::FailedDeterministic,
            "backend_deterministic",
        ),
        ModelBackendFailureClass::ContextOverflow => (
            RequestAttemptState::FailedContextOverflow,
            "backend_context_overflow",
        ),
        ModelBackendFailureClass::Crashed => {
            (RequestAttemptState::FailedTransient, "backend_crashed")
        }
    };
    finish_terminal(sink, request, attempt, state, terminal_time, Some(failure))
}

fn finish_terminal<E: HarnessEvidenceSink>(
    sink: &mut E,
    request: &ModelRequest,
    attempt: &mut RequestAttempt,
    state: RequestAttemptState,
    terminal_time: u64,
    failure_class: Option<&str>,
) -> Result<(), HarnessRunError> {
    attempt
        .transition(state)
        .map_err(|_| HarnessRunError::InvalidRequest)?;
    attempt.terminal_at_unix_ms = Some(terminal_time);
    attempt.failure_class = failure_class.map(str::to_owned);
    sink.persist_attempt_state(request.session_id, attempt, ATTEMPT_RECORD_V1)?;
    Ok(())
}

fn canonical_event_ref(attempt_id: RequestAttemptId, sequence: u64) -> String {
    format!("model-event:{attempt_id}:{sequence}")
}

fn outcome(attempt: &RequestAttempt) -> HarnessAttemptOutcome {
    HarnessAttemptOutcome {
        request_series_id: attempt.request_series_id,
        request_attempt_id: attempt.request_attempt_id,
        terminal_state: attempt.state,
        accepted_event_refs: attempt.accepted_event_refs.clone(),
        failure_class: attempt.failure_class.clone(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScriptStep {
    Emit(BackendEmission),
    Fail(ModelBackendFailureClass),
    End,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ScriptedBackend {
    scripts: VecDeque<VecDeque<ScriptStep>>,
    starts: u64,
}

impl ScriptedBackend {
    pub fn new(scripts: Vec<Vec<ScriptStep>>) -> Self {
        Self {
            scripts: scripts.into_iter().map(VecDeque::from).collect(),
            starts: 0,
        }
    }

    pub const fn starts(&self) -> u64 {
        self.starts
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScriptedSession {
    backend_instance_ref: String,
    steps: VecDeque<ScriptStep>,
    cancelled: bool,
}

impl ModelBackend for ScriptedBackend {
    type Session = ScriptedSession;

    fn start(&mut self, _request: &ModelRequest) -> Result<Self::Session, ModelBackendError> {
        self.starts = self.starts.saturating_add(1);
        let steps = self.scripts.pop_front().ok_or_else(|| {
            ModelBackendError::new(
                ModelBackendFailureClass::Deterministic,
                "no scripted backend fixture remains",
            )
        })?;
        Ok(ScriptedSession {
            backend_instance_ref: format!("scripted:session:{}", self.starts),
            steps,
            cancelled: false,
        })
    }
}

impl ModelBackendSession for ScriptedSession {
    fn backend_instance_ref(&self) -> &str {
        &self.backend_instance_ref
    }

    fn next_emission(&mut self) -> Result<Option<BackendEmission>, ModelBackendError> {
        if self.cancelled {
            return Ok(None);
        }
        match self.steps.pop_front() {
            Some(ScriptStep::Emit(emission)) => Ok(Some(emission)),
            Some(ScriptStep::Fail(class)) => {
                Err(ModelBackendError::new(class, "scripted backend failure"))
            }
            Some(ScriptStep::End) | None => Ok(None),
        }
    }

    fn request_cancel(&mut self) -> Result<(), ModelBackendError> {
        self.cancelled = true;
        Ok(())
    }
}

pub fn text_delta(sequence: u64, value: impl AsRef<[u8]>) -> ScriptStep {
    ScriptStep::Emit(BackendEmission {
        sequence,
        kind: ModelEventKind::TextDelta,
        payload: value.as_ref().to_vec(),
    })
}

pub fn stop(sequence: u64) -> ScriptStep {
    ScriptStep::Emit(BackendEmission {
        sequence,
        kind: ModelEventKind::Stop,
        payload: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::SessionId;
    use golam_core::harness::ExecutionProfileId;

    #[derive(Default)]
    struct RecordingSink {
        prepared: Vec<RequestAttempt>,
        states: Vec<RequestAttempt>,
        events: Vec<ModelEvent>,
        protected_effect_dispatches: u64,
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

    fn request(series: u128, attempt: u128, runtime_ms: u64) -> ModelRequest {
        ModelRequest {
            request_series_id: RequestSeriesId::from_u128(series),
            request_attempt_id: RequestAttemptId::from_u128(attempt),
            initiator_principal_ref: "principal:owner".into(),
            session_id: SessionId(7),
            turn_ref: "turn:1".into(),
            execution_profile_id: ExecutionProfileId::from_u128(9),
            context_projection_ref: "projection:1".into(),
            message_refs: vec!["event:user:1".into()],
            tool_schema_digest: None,
            max_input_tokens: 32,
            max_output_tokens: 32,
            max_runtime_ms: runtime_ms,
            request_digest: [5; 32],
        }
    }

    #[test]
    fn scripted_backend_runs_prepare_dispatch_stream_terminal() {
        let backend = ScriptedBackend::new(vec![vec![text_delta(0, b"hi"), stop(1)]]);
        let mut coordinator = HarnessCoordinator::new(backend);
        let mut sink = RecordingSink::default();
        let result = coordinator
            .run_attempt(
                &mut sink,
                &request(1, 1, 100),
                10,
                HarnessRunControl::default(),
            )
            .unwrap();
        assert_eq!(result.terminal_state, RequestAttemptState::Completed);
        assert_eq!(sink.prepared[0].state, RequestAttemptState::Prepared);
        assert_eq!(sink.events.len(), 2);
    }

    #[test]
    fn duplicate_or_out_of_order_event_fails_deterministically() {
        let backend = ScriptedBackend::new(vec![vec![text_delta(0, b"a"), text_delta(0, b"b")]]);
        let mut coordinator = HarnessCoordinator::new(backend);
        let mut sink = RecordingSink::default();
        let result = coordinator
            .run_attempt(
                &mut sink,
                &request(1, 1, 100),
                10,
                HarnessRunControl::default(),
            )
            .unwrap();
        assert_eq!(
            result.terminal_state,
            RequestAttemptState::FailedDeterministic
        );
        assert_eq!(sink.events.len(), 1);
    }

    #[test]
    fn cancellation_is_distinct_and_preserves_accepted_prefix() {
        let backend =
            ScriptedBackend::new(vec![vec![text_delta(0, b"prefix"), text_delta(1, b"late")]]);
        let mut coordinator = HarnessCoordinator::new(backend);
        let mut sink = RecordingSink::default();
        let result = coordinator
            .run_attempt(
                &mut sink,
                &request(1, 1, 100),
                10,
                HarnessRunControl {
                    cancel_after_polls: Some(1),
                    ..HarnessRunControl::default()
                },
            )
            .unwrap();
        assert_eq!(result.terminal_state, RequestAttemptState::Cancelled);
        assert_eq!(sink.events.len(), 1);
        assert_eq!(sink.events[0].payload, b"prefix");
        assert!(
            sink.states
                .iter()
                .any(|state| state.state == RequestAttemptState::CancelRequested)
        );
    }

    #[test]
    fn timeout_is_not_user_cancellation() {
        let backend =
            ScriptedBackend::new(vec![vec![text_delta(0, b"prefix"), text_delta(1, b"late")]]);
        let mut coordinator = HarnessCoordinator::new(backend);
        let mut sink = RecordingSink::default();
        let result = coordinator
            .run_attempt(
                &mut sink,
                &request(1, 1, 1),
                10,
                HarnessRunControl::default(),
            )
            .unwrap();
        assert_eq!(result.terminal_state, RequestAttemptState::TimedOut);
        assert_eq!(result.failure_class.as_deref(), Some("request_timeout"));
        assert!(
            !sink
                .states
                .iter()
                .any(|state| state.state == RequestAttemptState::CancelRequested)
        );
    }

    #[test]
    fn transient_retry_uses_new_attempt_id_without_rewriting_prior_attempt() {
        let backend = ScriptedBackend::new(vec![
            vec![ScriptStep::Fail(ModelBackendFailureClass::Crashed)],
            vec![text_delta(0, b"ok"), stop(1)],
        ]);
        let mut coordinator = HarnessCoordinator::new(backend);
        let mut sink = RecordingSink::default();
        let outcomes = coordinator
            .run_retry_series(
                &mut sink,
                &[request(9, 1, 100), request(9, 2, 100)],
                10,
                HarnessRunControl::default(),
            )
            .unwrap();
        assert_eq!(outcomes.len(), 2);
        assert_eq!(
            outcomes[0].request_attempt_id,
            RequestAttemptId::from_u128(1)
        );
        assert_eq!(
            outcomes[1].request_attempt_id,
            RequestAttemptId::from_u128(2)
        );
        assert_eq!(sink.prepared.len(), 2);
        assert_eq!(sink.protected_effect_dispatches, 0);
    }

    #[test]
    fn deterministic_and_context_overflow_do_not_blind_retry() {
        for class in [
            ModelBackendFailureClass::Deterministic,
            ModelBackendFailureClass::ContextOverflow,
        ] {
            let backend = ScriptedBackend::new(vec![
                vec![ScriptStep::Fail(class)],
                vec![text_delta(0, b"must-not-run"), stop(1)],
            ]);
            let mut coordinator = HarnessCoordinator::new(backend);
            let mut sink = RecordingSink::default();
            let outcomes = coordinator
                .run_retry_series(
                    &mut sink,
                    &[request(9, 1, 100), request(9, 2, 100)],
                    10,
                    HarnessRunControl::default(),
                )
                .unwrap();
            assert_eq!(outcomes.len(), 1);
            assert_eq!(coordinator.backend().starts(), 1);
            assert_eq!(sink.protected_effect_dispatches, 0);
        }
    }

    #[test]
    fn retry_series_rejects_duplicate_attempt_identity() {
        let backend = ScriptedBackend::default();
        let mut coordinator = HarnessCoordinator::new(backend);
        let mut sink = RecordingSink::default();
        let result = coordinator.run_retry_series(
            &mut sink,
            &[request(9, 1, 100), request(9, 1, 100)],
            10,
            HarnessRunControl::default(),
        );
        assert_eq!(result, Err(HarnessRunError::InvalidRetrySeries));
    }
}
