#![forbid(unsafe_code)]

use golam_core::CanonicalEncoder;
use golam_core::SessionId;
use golam_core::digest::sha256;
use golam_core::execution_profile::{BackendKind, ExecutionProfile};
use golam_core::harness::{RequestAttemptId, RequestSeriesId};
use golam_core::harness_state::{
    BenchmarkMetrics, BenchmarkRecord, BenchmarkResult, HardwareProfile, ModelEvent, ModelRequest,
    RequestAttempt, RequestAttemptState,
};
use golam_core::model_backend::{
    HarnessEvidenceSink, HarnessEvidenceSinkError, MAX_BACKEND_EMISSION_BYTES,
};

use crate::harness::{
    HarnessCoordinator, HarnessRunControl, HarnessRunError, ScriptedBackend, stop, text_delta,
};

pub const HARNESS_BENCHMARK_SCHEMA_VERSION: u16 = 1;
const MAX_FIXTURE_FRAGMENTS: usize = 64;
const MAX_FIXTURE_ID_BYTES: usize = 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HarnessBenchmarkFixture {
    pub fixture_id: String,
    pub fixture_version: u16,
    pub text_fragments: Vec<Vec<u8>>,
    pub poll_duration_ms: u64,
    pub max_polls: u64,
}

impl HarnessBenchmarkFixture {
    pub fn spec004_scripted_v1() -> Self {
        Self {
            fixture_id: "spec004:harness-scripted:v1".into(),
            fixture_version: 1,
            text_fragments: vec![b"alpha".to_vec(), b"beta".to_vec()],
            poll_duration_ms: 1,
            max_polls: 8,
        }
    }

    pub fn validate(&self) -> Result<(), BenchmarkRunError> {
        if self.fixture_id.is_empty()
            || self.fixture_id.len() > MAX_FIXTURE_ID_BYTES
            || self.fixture_version == 0
            || self.text_fragments.is_empty()
            || self.text_fragments.len() > MAX_FIXTURE_FRAGMENTS
            || self.poll_duration_ms == 0
            || self.max_polls == 0
            || self.max_polls < self.text_fragments.len() as u64 + 1
        {
            return Err(BenchmarkRunError::InvalidFixture);
        }
        if self
            .text_fragments
            .iter()
            .any(|fragment| fragment.is_empty() || fragment.len() > MAX_BACKEND_EMISSION_BYTES)
        {
            return Err(BenchmarkRunError::InvalidFixture);
        }
        Ok(())
    }

    pub fn content_digest(&self) -> Result<[u8; 32], BenchmarkRunError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder
            .push_bytes(b"golam:harness-benchmark-fixture:v1")
            .map_err(|_| BenchmarkRunError::InvalidFixture)?;
        encoder
            .push_bytes(self.fixture_id.as_bytes())
            .map_err(|_| BenchmarkRunError::InvalidFixture)?;
        encoder.push_u16(self.fixture_version);
        encoder.push_u64(self.text_fragments.len() as u64);
        for fragment in &self.text_fragments {
            encoder
                .push_bytes(fragment)
                .map_err(|_| BenchmarkRunError::InvalidFixture)?;
        }
        encoder.push_u64(self.poll_duration_ms);
        encoder.push_u64(self.max_polls);
        Ok(sha256(&encoder.finish()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HarnessBenchmarkBinding {
    pub code_revision: String,
    pub execution_profile_id: golam_core::harness::ExecutionProfileId,
    pub profile_content_digest: [u8; 32],
    pub hardware_profile_id: golam_core::harness::HardwareProfileId,
    pub hardware_observation_digest: [u8; 32],
    pub backend_identity_digest: [u8; 32],
    pub workload_fixture_id: String,
    pub workload_fixture_digest: [u8; 32],
}

impl HarnessBenchmarkBinding {
    pub fn from_inputs(
        code_revision: &str,
        profile: &ExecutionProfile,
        hardware: &HardwareProfile,
        fixture: &HarnessBenchmarkFixture,
    ) -> Result<Self, BenchmarkRunError> {
        if code_revision.is_empty() || code_revision.len() > 1024 {
            return Err(BenchmarkRunError::InvalidBinding);
        }
        profile
            .validate_identity()
            .map_err(|_| BenchmarkRunError::InvalidProfile)?;
        hardware
            .validate()
            .map_err(|_| BenchmarkRunError::InvalidHardware)?;
        Ok(Self {
            code_revision: code_revision.to_owned(),
            execution_profile_id: profile.profile_id(),
            profile_content_digest: profile.content_digest(),
            hardware_profile_id: hardware.hardware_profile_id,
            hardware_observation_digest: hardware_observation_digest(hardware),
            backend_identity_digest: backend_identity_digest(profile)?,
            workload_fixture_id: fixture.fixture_id.clone(),
            workload_fixture_digest: fixture.content_digest()?,
        })
    }

    pub fn evidence_refs(&self) -> Vec<String> {
        vec![
            format!(
                "benchmark-binding:profile-content:{}",
                hex32(self.profile_content_digest)
            ),
            format!(
                "benchmark-binding:hardware-observation:{}",
                hex32(self.hardware_observation_digest)
            ),
            format!(
                "benchmark-binding:backend-identity:{}",
                hex32(self.backend_identity_digest)
            ),
            format!(
                "benchmark-binding:workload:{}",
                hex32(self.workload_fixture_digest)
            ),
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HarnessBenchmarkRun {
    pub binding: HarnessBenchmarkBinding,
    pub record: BenchmarkRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BenchmarkRunError {
    InvalidFixture,
    InvalidProfile,
    InvalidHardware,
    InvalidBinding,
    Harness(HarnessRunError),
    InvalidRecord,
}

#[derive(Default)]
struct BenchmarkSink {
    states: Vec<RequestAttempt>,
    events: Vec<ModelEvent>,
}

impl HarnessEvidenceSink for BenchmarkSink {
    fn persist_prepared_attempt(
        &mut self,
        _session_id: SessionId,
        attempt: &RequestAttempt,
        _record_bytes: &[u8],
    ) -> Result<(), HarnessEvidenceSinkError> {
        self.states.push(attempt.clone());
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

pub fn run_scripted_harness_benchmark(
    benchmark_id: u128,
    code_revision: &str,
    profile: &ExecutionProfile,
    hardware: &HardwareProfile,
    fixture: &HarnessBenchmarkFixture,
    started_at_unix_ms: u64,
) -> Result<HarnessBenchmarkRun, BenchmarkRunError> {
    fixture.validate()?;
    if profile.definition().backend.kind != BackendKind::Scripted {
        return Err(BenchmarkRunError::InvalidProfile);
    }
    let binding = HarnessBenchmarkBinding::from_inputs(code_revision, profile, hardware, fixture)?;
    let maximum_fixture_runtime = fixture.max_polls.saturating_mul(fixture.poll_duration_ms);
    if maximum_fixture_runtime > profile.definition().time_budget.max_request_ms {
        return Err(BenchmarkRunError::InvalidFixture);
    }

    let mut steps = fixture
        .text_fragments
        .iter()
        .enumerate()
        .map(|(sequence, fragment)| text_delta(sequence as u64, fragment))
        .collect::<Vec<_>>();
    steps.push(stop(fixture.text_fragments.len() as u64));

    let backend = ScriptedBackend::new(vec![steps]);
    let mut coordinator = HarnessCoordinator::new(backend);
    let mut sink = BenchmarkSink::default();
    let request = ModelRequest {
        request_series_id: RequestSeriesId::from_u128(benchmark_id),
        request_attempt_id: RequestAttemptId::from_u128(benchmark_id),
        initiator_principal_ref: "principal:benchmark-fixture".into(),
        session_id: SessionId(benchmark_id),
        turn_ref: format!("benchmark-turn:{benchmark_id}"),
        execution_profile_id: profile.profile_id(),
        context_projection_ref: format!("benchmark-projection:{benchmark_id}"),
        message_refs: vec![format!("benchmark-workload:{}", fixture.fixture_id)],
        tool_schema_digest: None,
        max_input_tokens: profile.definition().token_budget.max_input_tokens,
        max_output_tokens: profile.definition().token_budget.max_output_tokens,
        max_runtime_ms: profile.definition().time_budget.max_request_ms,
        request_digest: binding.workload_fixture_digest,
    };
    let outcome = coordinator
        .run_attempt(
            &mut sink,
            &request,
            started_at_unix_ms,
            HarnessRunControl {
                cancel_after_polls: None,
                poll_duration_ms: fixture.poll_duration_ms,
                max_polls: fixture.max_polls,
            },
        )
        .map_err(BenchmarkRunError::Harness)?;

    let finished_at_unix_ms = sink
        .states
        .iter()
        .rev()
        .find_map(|state| state.terminal_at_unix_ms)
        .unwrap_or(started_at_unix_ms);
    let expected_events = fixture.text_fragments.len() + 1;
    let passed = outcome.terminal_state == RequestAttemptState::Completed
        && sink.events.len() == expected_events
        && coordinator.backend().starts() == 1;

    let mut raw_evidence_refs = binding.evidence_refs();
    raw_evidence_refs.extend(outcome.accepted_event_refs.iter().cloned());
    let record = BenchmarkRecord {
        benchmark_id,
        schema_version: HARNESS_BENCHMARK_SCHEMA_VERSION,
        code_revision: code_revision.to_owned(),
        execution_profile_id: profile.profile_id(),
        hardware_profile_id: hardware.hardware_profile_id,
        workload_fixture_id: fixture.fixture_id.clone(),
        backend_metrics: vec![
            metric("scripted_backend_starts", coordinator.backend().starts()),
            metric("scripted_backend_emissions", sink.events.len() as u64),
        ],
        harness_metrics: vec![
            metric("accepted_events", sink.events.len() as u64),
            metric("completed_attempts", u64::from(passed)),
            metric("transient_retries", 0),
            metric("protected_effect_duplicate_count", 0),
        ],
        started_at_unix_ms,
        finished_at_unix_ms,
        result: if passed {
            BenchmarkResult::Passed
        } else {
            BenchmarkResult::Failed
        },
        raw_evidence_refs,
    };
    record
        .validate()
        .map_err(|_| BenchmarkRunError::InvalidRecord)?;

    Ok(HarnessBenchmarkRun { binding, record })
}

pub fn benchmark_record_matches_binding(
    record: &BenchmarkRecord,
    binding: &HarnessBenchmarkBinding,
) -> bool {
    if record.schema_version != HARNESS_BENCHMARK_SCHEMA_VERSION
        || record.code_revision != binding.code_revision
        || record.execution_profile_id != binding.execution_profile_id
        || record.hardware_profile_id != binding.hardware_profile_id
        || record.workload_fixture_id != binding.workload_fixture_id
    {
        return false;
    }
    binding
        .evidence_refs()
        .iter()
        .all(|required| record.raw_evidence_refs.contains(required))
}

fn metric(name: &str, value: u64) -> BenchmarkMetrics {
    BenchmarkMetrics {
        metric_name: name.into(),
        value_milli: i64::try_from(value.saturating_mul(1000)).unwrap_or(i64::MAX),
        unit: "count_milli".into(),
    }
}

fn hardware_observation_digest(hardware: &HardwareProfile) -> [u8; 32] {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_u128(hardware.hardware_profile_id.as_u128());
    encoder.push_u64(hardware.observed_at_unix_ms);
    let _ = encoder.push_bytes(&hardware.content_digest);
    sha256(&encoder.finish())
}

fn backend_identity_digest(profile: &ExecutionProfile) -> Result<[u8; 32], BenchmarkRunError> {
    let backend = &profile.definition().backend;
    let mut encoder = CanonicalEncoder::new();
    encoder.push_u8(match backend.kind {
        BackendKind::Scripted => 0,
        BackendKind::MistralRs => 1,
        BackendKind::LlamaCpp => 2,
        BackendKind::OtherQualified => 3,
    });
    encoder
        .push_bytes(backend.source_or_distribution_id.as_bytes())
        .map_err(|_| BenchmarkRunError::InvalidBinding)?;
    encoder
        .push_bytes(backend.exact_revision_or_build_id.as_bytes())
        .map_err(|_| BenchmarkRunError::InvalidBinding)?;
    encoder.push_u16(backend.adapter_schema_version);
    encoder
        .push_bytes(backend.launch_or_feature_digest.as_bytes())
        .map_err(|_| BenchmarkRunError::InvalidBinding)?;
    Ok(sha256(&encoder.finish()))
}

fn hex32(bytes: [u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(64);
    for byte in bytes {
        output.push(char::from(HEX[(byte >> 4) as usize]));
        output.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::execution_profile::{
        BackendIdentity, EXECUTION_PROFILE_SCHEMA_VERSION, ExecutionProfileDefinition,
        FailureAction, FailurePolicy, FallbackPolicy, LatencyQualityBudget, LoadPolicy, Locality,
        MultimodalCapabilities, NetworkConstraints, PrivacyConstraints, ResourceBudget,
        SamplingParameters, TimeBudget, TokenBudget, ToolCallConformance, WarmResidencyPolicy,
        WorkloadClass,
    };
    use golam_core::harness::HardwareProfileId;
    use golam_core::harness_state::{HardwarePrivacyClass, HardwareProfileSource};

    fn profile() -> ExecutionProfile {
        ExecutionProfile::new(
            ExecutionProfileDefinition {
                schema_version: EXECUTION_PROFILE_SCHEMA_VERSION,
                model_identity: "scripted-model".into(),
                model_revision: "fixture-v1".into(),
                tokenizer_identity: "fixture-tokenizer".into(),
                chat_template_identity: "fixture-template".into(),
                backend: BackendIdentity {
                    kind: BackendKind::Scripted,
                    source_or_distribution_id: "golam-scripted".into(),
                    exact_revision_or_build_id: "v1".into(),
                    adapter_schema_version: 1,
                    launch_or_feature_digest: "deterministic".into(),
                },
                locality: Locality::Local,
                quantization_or_precision: "fixture".into(),
                hardware_mapping: "cpu".into(),
                harness_profile: "spec004".into(),
                reasoning_mode: "bounded".into(),
                tool_call_conformance: ToolCallConformance::NativeTools,
                tool_schema_mode: "fixture".into(),
                sampling: SamplingParameters {
                    temperature_milli: 0,
                    top_p_milli: 1000,
                    top_k: 1,
                    seed: Some(1),
                },
                context_policy: "bounded".into(),
                prompt_prefix_cache_policy: "disabled".into(),
                kv_cache_policy: "disabled".into(),
                warm_residency_policy: WarmResidencyPolicy {
                    load_behavior: "fixture".into(),
                    keep_behavior: "none".into(),
                    evict_behavior: "immediate".into(),
                },
                workload_class: WorkloadClass::Batch,
                multimodal_capabilities: MultimodalCapabilities {
                    text: true,
                    image_input: false,
                    audio_input: false,
                    audio_output: false,
                },
                resource_budget: ResourceBudget {
                    max_memory_bytes: 1024,
                    max_cpu_threads: 2,
                    max_accelerator_memory_bytes: None,
                },
                time_budget: TimeBudget {
                    max_load_ms: 100,
                    max_request_ms: 100,
                    max_idle_ms: 100,
                },
                token_budget: TokenBudget {
                    max_input_tokens: 64,
                    max_output_tokens: 32,
                    max_total_tokens: 96,
                },
                latency_quality_budget: LatencyQualityBudget {
                    max_ttft_ms: 50,
                    max_total_latency_ms: 100,
                    min_quality_milli: 0,
                },
                privacy_constraints: PrivacyConstraints {
                    allow_user_content_external: false,
                    allow_telemetry: false,
                    allow_model_download: false,
                },
                network_constraints: NetworkConstraints {
                    allow_network: false,
                    allowed_endpoint_classes: Vec::new(),
                },
                load_policy: LoadPolicy::FailIfUnavailable,
                failure_policy: FailurePolicy {
                    max_transient_retries: 0,
                    retry_backoff_ms: 0,
                    deterministic_failure: FailureAction::Fail,
                    context_overflow: FailureAction::ReprojectContext,
                },
                fallback_policy: FallbackPolicy {
                    allow_backend_change: false,
                    allow_model_change: false,
                    allowed_profile_ids: Vec::new(),
                },
            },
            Vec::new(),
        )
        .unwrap()
    }

    fn hardware(observed_at_unix_ms: u64) -> HardwareProfile {
        HardwareProfile {
            hardware_profile_id: HardwareProfileId::from_u128(7),
            observed_at_unix_ms,
            platform: "fixture-os".into(),
            architecture: "fixture-arch".into(),
            cpu_capabilities: vec!["logical-cpus:2".into()],
            memory_capacity_or_bucket: "fixture".into(),
            accelerators: Vec::new(),
            backend_capabilities: vec!["scripted".into()],
            source: HardwareProfileSource::Fixture,
            privacy_class: HardwarePrivacyClass::FixtureSynthetic,
            content_digest: [7; 32],
        }
    }

    #[test]
    fn scripted_benchmark_separates_backend_and_harness_metrics() {
        let run = run_scripted_harness_benchmark(
            1,
            "revision-a",
            &profile(),
            &hardware(10),
            &HarnessBenchmarkFixture::spec004_scripted_v1(),
            100,
        )
        .unwrap();

        assert_eq!(run.record.result, BenchmarkResult::Passed);
        assert_eq!(run.record.backend_metrics.len(), 2);
        assert_eq!(run.record.harness_metrics.len(), 4);
        assert!(benchmark_record_matches_binding(&run.record, &run.binding));
    }

    #[test]
    fn scripted_benchmark_is_deterministic_for_same_inputs() {
        let fixture = HarnessBenchmarkFixture::spec004_scripted_v1();
        let selected_profile = profile();
        let observed_hardware = hardware(10);
        let first = run_scripted_harness_benchmark(
            2,
            "revision-a",
            &selected_profile,
            &observed_hardware,
            &fixture,
            100,
        )
        .unwrap();
        let second = run_scripted_harness_benchmark(
            2,
            "revision-a",
            &selected_profile,
            &observed_hardware,
            &fixture,
            100,
        )
        .unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn material_binding_changes_invalidate_benchmark_reuse() {
        let fixture = HarnessBenchmarkFixture::spec004_scripted_v1();
        let selected_profile = profile();
        let observed_hardware = hardware(10);
        let run = run_scripted_harness_benchmark(
            3,
            "revision-a",
            &selected_profile,
            &observed_hardware,
            &fixture,
            100,
        )
        .unwrap();

        let changed_revision = HarnessBenchmarkBinding::from_inputs(
            "revision-b",
            &selected_profile,
            &observed_hardware,
            &fixture,
        )
        .unwrap();
        assert!(!benchmark_record_matches_binding(
            &run.record,
            &changed_revision
        ));

        let changed_hardware = hardware(11);
        let changed_hardware_binding = HarnessBenchmarkBinding::from_inputs(
            "revision-a",
            &selected_profile,
            &changed_hardware,
            &fixture,
        )
        .unwrap();
        assert!(!benchmark_record_matches_binding(
            &run.record,
            &changed_hardware_binding
        ));

        let mut changed_fixture = fixture.clone();
        changed_fixture.text_fragments.push(b"gamma".to_vec());
        let changed_workload = HarnessBenchmarkBinding::from_inputs(
            "revision-a",
            &selected_profile,
            &observed_hardware,
            &changed_fixture,
        )
        .unwrap();
        assert!(!benchmark_record_matches_binding(
            &run.record,
            &changed_workload
        ));

        let mut changed_definition = selected_profile.definition().clone();
        changed_definition.backend.exact_revision_or_build_id = "v2".into();
        let changed_profile = ExecutionProfile::new(changed_definition, Vec::new()).unwrap();
        let changed_backend = HarnessBenchmarkBinding::from_inputs(
            "revision-a",
            &changed_profile,
            &observed_hardware,
            &fixture,
        )
        .unwrap();
        assert!(!benchmark_record_matches_binding(
            &run.record,
            &changed_backend
        ));
    }

    #[test]
    fn binding_digests_use_canonical_sha256() {
        let fixture = HarnessBenchmarkFixture::spec004_scripted_v1();
        let digest = fixture.content_digest().unwrap();
        let mut encoder = CanonicalEncoder::new();
        encoder
            .push_bytes(b"golam:harness-benchmark-fixture:v1")
            .unwrap();
        encoder.push_bytes(fixture.fixture_id.as_bytes()).unwrap();
        encoder.push_u16(fixture.fixture_version);
        encoder.push_u64(fixture.text_fragments.len() as u64);
        for fragment in &fixture.text_fragments {
            encoder.push_bytes(fragment).unwrap();
        }
        encoder.push_u64(fixture.poll_duration_ms);
        encoder.push_u64(fixture.max_polls);
        assert_eq!(digest, sha256(&encoder.finish()));
    }

    #[test]
    fn invalid_or_unbounded_fixture_fails_closed() {
        let mut fixture = HarnessBenchmarkFixture::spec004_scripted_v1();
        fixture.max_polls = 1;
        assert_eq!(fixture.validate(), Err(BenchmarkRunError::InvalidFixture));

        fixture = HarnessBenchmarkFixture::spec004_scripted_v1();
        fixture.text_fragments = vec![vec![0; MAX_BACKEND_EMISSION_BYTES + 1]];
        assert_eq!(fixture.validate(), Err(BenchmarkRunError::InvalidFixture));
    }
}
