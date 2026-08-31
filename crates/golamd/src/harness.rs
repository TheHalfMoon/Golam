#![forbid(unsafe_code)]

use std::collections::{HashSet, VecDeque};

use golam_core::harness::{RequestAttemptId, RequestSeriesId};
use golam_core::harness_state::{
    ModelEvent, ModelEventAcceptance, ModelEventKind, ModelRequest, RequestAttempt,
    RequestAttemptState,
};
use golam_core::model_backend::{
    BackendEmission, ModelBackend, ModelBackendError, ModelBackendFailureClass, ModelBackendSession,
};
use golam_ledger::harness_evidence::{HarnessEvidenceError, HarnessEvidenceStore};

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
    pub const fn retryable_transient(&self) -> bool {
        self.terminal_state == RequestAttemptState::FailedTransient
    }

    pub const fn needs_context_reprojection(&self) -> bool {
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

impl From<HarnessEvidenceError> for HarnessRunError {
    fn from(value: HarnessEvidenceError) -> Self {
        Self::Evidence(value.to_string())
    }
}

pub struct HarnessCoordinator<B> {
    backend: B,
}

impl<B> HarnessCoordinator<B>
where
    B: ModelBackend,
{
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    pub fn run_attempt(
        &mut self,
        store: &mut HarnessEvidenceStore,
        request: &ModelRequest,
        prepared_at_unix_ms: u64,
        control: HarnessRunControl,
    ) -> Result<HarnessAttemptOutcome, HarnessRunError> {
        request.validate().map_err(|_| HarnessRunError::InvalidRequest)?;
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
        store.persist_prepared_attempt(request.session_id, &attempt, ATTEMPT_RECORD_V1)?;

        let mut session = match self.backend.start(request) {
            Ok(session) => session,
            Err(error) => {
                finish_backend_error(
                    store,
                    request,
                    &mut attempt,
                    prepared_at_unix_ms,
                    &error,
                )?;
                return Ok(outcome(&attempt));
            }
        };

        attempt.backend_instance_ref = Some(session.backend_instance_ref().to_owned());
        attempt
            .transition(RequestAttemptState::Dispatched)
            .map_err(|_| HarnessRunError::InvalidRequest)?;
        store.persist_attempt_state(request.session_id, &attempt, ATTEMPT_RECORD_V1)?;

        let mut expected_sequence = 0_u64;
        let mut polls = 0_u64;
        loop {
            if polls >= control.max_polls {
                return finish_deterministic_failure(
                    store,
                    request,
                    &mut attempt,
                    prepared_at_unix_ms.saturating_add(
                        polls.saturating_mul(control.poll_duration_ms),
                    ),
                    "max_backend_polls_exceeded",
                );
            }

            if control.cancel_after_polls == Some(polls) {
                request_cancel(store, request, &mut attempt, &mut session)?;
                let terminal_time = prepared_at_unix_ms
                    .saturating_add(polls.saturating_mul(control.poll_duration_ms));
                finish_terminal(
                    store,
                    request,
                    &mut attempt,
                    RequestAttemptState::Cancelled,
                    terminal_time,
                    None,
                )?;
                return Ok(outcome(&attempt));
            }

            let elapsed_ms = polls.saturating_mul(control.poll_duration_ms);
            if elapsed_ms >= request.max_runtime_ms {
                session.request_cancel().map_err(HarnessRunError::Backend)?;
                finish_terminal(
                    store,
                    request,
                    &mut attempt,
                    RequestAttemptState::TimedOut,
                    prepared_at_unix_ms.saturating_add(elapsed_ms),
                    Some("request_timeout"),
                )?;
                return Ok(outcome(&attempt));
            }

            let emission = match session.next_emission() {
                Ok(Some(emission)) => emission,
                Ok(None) => {
                    return finish_deterministic_failure(
                        store,
                        request,
                        &mut attempt,
                        prepared_at_unix_ms.saturating_add(elapsed_ms),
                        "backend_ended_without_stop",
                    );
                }
                Err(error) => {
                    finish_backend_error(
                        store,
                        request,
                        &mut attempt,
                        prepared_at_unix_ms.saturating_add(elapsed_ms),
                        &error,
                    )?;
                    return Ok(outcome(&attempt));
                }
            };
            polls = polls.saturating_add(1);

            if emission.validate().is_err() || emission.sequence != expected_sequence {
                return finish_deterministic_failure(
                    store,
                    request,
                    &mut attempt,
                    prepared_at_unix_ms.saturating_add(elapsed_ms),
                    "invalid_backend_event_order_or_size",
                );
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
            store.append_model_event(&event, EVENT_RECORD_V1)?;
            attempt.accepted_event_refs.push(evidence_ref);

            if attempt.state == RequestAttemptState::Dispatched {
                attempt
                    .transition(RequestAttemptState::Streaming)
                    .map_err(|_| HarnessRunError::InvalidRequest)?;
            }

            if event.kind == ModelEventKind::Stop {
                finish_terminal(
                    store,
                    request,
                    &mut attempt,
                    RequestAttemptState::Completed,
                    prepared_at_unix_ms.saturating_add(
                        polls.saturating_mul(control.poll_duration_ms),
                    ),
                    None,
                )?;
                return Ok(outcome(&attempt));
            }

            store.persist_attempt_state(request.session_id, &attempt, ATTEMPT_RECORD_V1)?;
        }
    }

    pub fn run_retry_series(
        &mut self,
        store: &mut HarnessEvidenceStore,
        requests: &[ModelRequest],
        prepared_at_unix_ms: u64,
        control: HarnessRunControl,
    ) -> Result<Vec<HarnessAttemptOutcome>, HarnessRunError> {
        if requests.is_empty() {
            return Err(HarnessRunError::InvalidRetrySeries);
        }
        validate_retry_requests(requests)?;
        let mut outcomes = Vec::new();
        for request in requests {
            let outcome = self.run_attempt(store, request, prepared_at_unix_ms, control)?;
            let retry = outcome.retryable_transient();
            let requires_reprojection = outcome.needs_context_reprojection();
            outcomes.push(outcome);
            if requires_reprojection || !retry {
                break;
            }
        }
        Ok(outcomes)
    }
}

fn validate_retry_requests(requests: &[ModelRequest]) -> Result<(), HarnessRunError> {
    let series_id = requests[0].request_series_id;
    let mut attempt_ids = HashSet::new();
    for request in requests {
        if request.request_series_id != series_id
            || !attempt_ids.insert(request.request_attempt_id)
            || request.validate().is_err()
        {
            return Err(HarnessRunError::InvalidRetrySeries);
        }
    }
    Ok(())
}

fn request_cancel<S: ModelBackendSession>(
    store: &mut HarnessEvidenceStore,
    request: &ModelRequest,
    attempt: &mut RequestAttempt,
    session: &mut S,
) -> Result<(), HarnessRunError> {
    attempt
        .transition(RequestAttemptState::CancelRequested)
        .map_err(|_| HarnessRunError::InvalidRequest)?;
    store.persist_attempt_state(request.session_id, attempt, ATTEMPT_RECORD_V1)?;
    session.request_cancel().map_err(HarnessRunError::Backend)
}

fn finish_backend_error(
    store: &mut HarnessEvidenceStore,
    request: &ModelRequest,
    attempt: &mut RequestAttempt,
    terminal_time: u64,
    error: &ModelBackendError,
) -> Result<(), HarnessRunError> {
    let (state, failure_class) = match error.class {
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
    finish_terminal(
        store,
        request,
        attempt,
        state,
        terminal_time,
        Some(failure_class),
    )
}

fn finish_deterministic_failure(
    store: &mut HarnessEvidenceStore,
    request: &ModelRequest,
    attempt: &mut RequestAttempt,
    terminal_time: u64,
    failure_class: &'static str,
) -> Result<HarnessAttemptOutcome, HarnessRunError> {
    finish_terminal(
        store,
        request,
        attempt,
        RequestAttemptState::FailedDeterministic,
        terminal_time,
        Some(failure_class),
    )?;
    Ok(outcome(attempt))
}

fn finish_terminal(
    store: &mut HarnessEvidenceStore,
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
    store.persist_attempt_state(request.session_id, attempt, ATTEMPT_RECORD_V1)?;
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
    cancel_requests: u64,
}

impl ScriptedBackend {
    pub fn new(scripts: Vec<Vec<ScriptStep>>) -> Self {
        Self {
            scripts: scripts.into_iter().map(VecDeque::from).collect(),
            starts: 0,
            cancel_requests: 0,
        }
    }

    pub const fn starts(&self) -> u64 {
        self.starts
    }

    pub const fn cancel_requests(&self) -> u64 {
        self.cancel_requests
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
            request_digest: [11; 32],
        }
    }

    #[test]
    fn scripted_harness_persists_stream_and_completes() {
        let backend = ScriptedBackend::new(vec![vec![text_delta(0, b"hel"), text_delta(1, b"lo"), stop(2)]]);
        let mut harness = HarnessCoordinator::new(backend);
        let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
        let request = request(1, 1, 100);
        let outcome = harness
            .run_attempt(&mut store, &request, 1000, HarnessRunControl::default())
            .unwrap();
        assert_eq!(outcome.terminal_state, RequestAttemptState::Completed);
        assert_eq!(outcome.accepted_event_refs.len(), 3);
        let replay = store.accepted_events(request.request_attempt_id).unwrap();
        assert_eq!(replay[0].payload, b"hel");
        assert_eq!(replay[1].payload, b"lo");
    }

    #[test]
    fn cancellation_preserves_accepted_prefix_and_is_distinct_from_timeout() {
        let backend = ScriptedBackend::new(vec![vec![text_delta(0, b"prefix"), text_delta(1, b"late"), stop(2)]]);
        let mut harness = HarnessCoordinator::new(backend);
        let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
        let request = request(1, 2, 100);
        let outcome = harness
            .run_attempt(
                &mut store,
                &request,
                1000,
                HarnessRunControl {
                    cancel_after_polls: Some(1),
                    ..HarnessRunControl::default()
                },
            )
            .unwrap();
        assert_eq!(outcome.terminal_state, RequestAttemptState::Cancelled);
        assert_eq!(store.accepted_event_count(request.request_attempt_id).unwrap(), 1);

        let backend = ScriptedBackend::new(vec![vec![text_delta(0, b"prefix"), text_delta(1, b"late"), stop(2)]]);
        let mut harness = HarnessCoordinator::new(backend);
        let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
        let request = request(2, 3, 1);
        let timeout = harness
            .run_attempt(
                &mut store,
                &request,
                2000,
                HarnessRunControl {
                    poll_duration_ms: 1,
                    ..HarnessRunControl::default()
                },
            )
            .unwrap();
        assert_eq!(timeout.terminal_state, RequestAttemptState::TimedOut);
        assert_ne!(timeout.terminal_state, outcome.terminal_state);
    }

    #[test]
    fn out_of_order_and_duplicate_sequence_fail_closed() {
        for script in [
            vec![text_delta(1, b"out-of-order")],
            vec![text_delta(0, b"first"), text_delta(0, b"duplicate")],
        ] {
            let backend = ScriptedBackend::new(vec![script]);
            let mut harness = HarnessCoordinator::new(backend);
            let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
            let request = request(3, 4, 100);
            let outcome = harness
                .run_attempt(&mut store, &request, 3000, HarnessRunControl::default())
                .unwrap();
            assert_eq!(
                outcome.terminal_state,
                RequestAttemptState::FailedDeterministic
            );
        }
    }

    #[test]
    fn backend_crash_is_retryable_but_prior_attempt_is_not_rewritten() {
        let backend = ScriptedBackend::new(vec![
            vec![
                text_delta(0, b"partial"),
                ScriptStep::Fail(ModelBackendFailureClass::Crashed),
            ],
            vec![text_delta(0, b"retry"), stop(1)],
        ]);
        let mut harness = HarnessCoordinator::new(backend);
        let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
        let requests = [request(4, 5, 100), request(4, 6, 100)];
        let outcomes = harness
            .run_retry_series(
                &mut store,
                &requests,
                4000,
                HarnessRunControl::default(),
            )
            .unwrap();
        assert_eq!(outcomes.len(), 2);
        assert_eq!(
            outcomes[0].terminal_state,
            RequestAttemptState::FailedTransient
        );
        assert_eq!(outcomes[1].terminal_state, RequestAttemptState::Completed);
        assert_eq!(store.accepted_event_count(requests[0].request_attempt_id).unwrap(), 1);
        assert_eq!(store.accepted_event_count(requests[1].request_attempt_id).unwrap(), 2);
        assert_ne!(
            requests[0].request_attempt_id,
            requests[1].request_attempt_id
        );
    }

    #[test]
    fn deterministic_failure_and_context_overflow_do_not_blind_retry() {
        for class in [
            ModelBackendFailureClass::Deterministic,
            ModelBackendFailureClass::ContextOverflow,
        ] {
            let backend = ScriptedBackend::new(vec![
                vec![ScriptStep::Fail(class)],
                vec![text_delta(0, b"should-not-run"), stop(1)],
            ]);
            let mut harness = HarnessCoordinator::new(backend);
            let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
            let requests = [request(5, 7, 100), request(5, 8, 100)];
            let outcomes = harness
                .run_retry_series(
                    &mut store,
                    &requests,
                    5000,
                    HarnessRunControl::default(),
                )
                .unwrap();
            assert_eq!(outcomes.len(), 1);
            assert_eq!(harness.backend().starts(), 1);
            if class == ModelBackendFailureClass::ContextOverflow {
                assert!(outcomes[0].needs_context_reprojection());
            }
        }
    }

    #[test]
    fn oversized_backend_emission_is_rejected_without_accepting_it() {
        let backend = ScriptedBackend::new(vec![vec![ScriptStep::Emit(BackendEmission {
            sequence: 0,
            kind: ModelEventKind::TextDelta,
            payload: vec![0; golam_core::model_backend::MAX_BACKEND_EMISSION_BYTES + 1],
        })]]);
        let mut harness = HarnessCoordinator::new(backend);
        let mut store = HarnessEvidenceStore::open_in_memory().unwrap();
        let request = request(6, 9, 100);
        let outcome = harness
            .run_attempt(&mut store, &request, 6000, HarnessRunControl::default())
            .unwrap();
        assert_eq!(
            outcome.terminal_state,
            RequestAttemptState::FailedDeterministic
        );
        assert_eq!(store.accepted_event_count(request.request_attempt_id).unwrap(), 0);
    }
}
