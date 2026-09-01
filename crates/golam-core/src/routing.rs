#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::CanonicalEncoder;
use crate::execution_profile::{BackendIdentity, ExecutionProfile, Locality};
use crate::harness::{ExecutionProfileId, HardwareProfileId};
use crate::harness_state::{
    AcceleratorObservation, CalibrationObservation, CalibrationResult, CalibrationRun,
    HardwarePrivacyClass, HardwareProfile, HardwareProfileSource, MeasurementStatus,
};

const MAX_LOCAL_PROBE_CAPABILITIES: usize = 32;
const MAX_CALIBRATION_ITERATIONS: u32 = 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoutingStage {
    RequestedOrPinned,
    PrivacyLocalityNetwork,
    Availability,
    Hardware,
    Budgets,
    Preference,
}

pub const ROUTING_STAGE_ORDER: [RoutingStage; 6] = [
    RoutingStage::RequestedOrPinned,
    RoutingStage::PrivacyLocalityNetwork,
    RoutingStage::Availability,
    RoutingStage::Hardware,
    RoutingStage::Budgets,
    RoutingStage::Preference,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoutePolicy {
    pub pinned_profile_id: Option<ExecutionProfileId>,
    pub strict_local: bool,
    pub allow_explicit_cloud: bool,
    pub allow_network: bool,
    pub allow_external_user_content: bool,
    pub allow_telemetry: bool,
    pub allow_model_download: bool,
    pub max_hardware_age_ms: u64,
    pub max_memory_bytes: u64,
    pub max_cpu_threads: u16,
    pub max_total_tokens: u32,
    pub max_request_ms: u64,
}

impl RoutePolicy {
    fn validate(self) -> Result<(), RoutingError> {
        if self.max_hardware_age_ms == 0
            || self.max_memory_bytes == 0
            || self.max_cpu_threads == 0
            || self.max_total_tokens == 0
            || self.max_request_ms == 0
        {
            return Err(RoutingError::InvalidPolicy);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RouteCandidate<'a> {
    pub profile: &'a ExecutionProfile,
    pub available: bool,
    pub preference_milli: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HardwareFixtureInput {
    pub hardware_profile_id: HardwareProfileId,
    pub observed_at_unix_ms: u64,
    pub platform: String,
    pub architecture: String,
    pub cpu_capabilities: Vec<String>,
    pub memory_capacity_or_bucket: String,
    pub accelerators: Vec<AcceleratorObservation>,
    pub backend_capabilities: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutingDecision {
    pub selected_profile_id: ExecutionProfileId,
    pub hardware_profile_id: HardwareProfileId,
    pub evaluated_stages: Vec<RoutingStage>,
    pub eligible_profile_ids: Vec<ExecutionProfileId>,
    pub preference_milli: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RoutingError {
    InvalidPolicy,
    InvalidHardware,
    StaleHardware,
    NoEligibleProfile { failed_at: RoutingStage },
    InvalidFallbackPolicy,
    CalibrationInvalid,
}

pub fn route_profile(
    candidates: &[RouteCandidate<'_>],
    hardware: &HardwareProfile,
    now_unix_ms: u64,
    policy: RoutePolicy,
) -> Result<RoutingDecision, RoutingError> {
    policy.validate()?;
    hardware
        .validate()
        .map_err(|_| RoutingError::InvalidHardware)?;
    validate_hardware_freshness(hardware, now_unix_ms, policy.max_hardware_age_ms)?;

    let mut eligible = candidates.iter().collect::<Vec<_>>();
    if let Some(pinned) = policy.pinned_profile_id {
        eligible.retain(|candidate| candidate.profile.profile_id() == pinned);
    }
    ensure_nonempty(&eligible, RoutingStage::RequestedOrPinned)?;

    eligible.retain(|candidate| privacy_locality_network_compatible(candidate.profile, policy));
    ensure_nonempty(&eligible, RoutingStage::PrivacyLocalityNetwork)?;

    eligible.retain(|candidate| candidate.available);
    ensure_nonempty(&eligible, RoutingStage::Availability)?;

    eligible.retain(|candidate| hardware_supports_profile(hardware, candidate.profile));
    ensure_nonempty(&eligible, RoutingStage::Hardware)?;

    eligible.retain(|candidate| budgets_compatible(candidate.profile, policy));
    ensure_nonempty(&eligible, RoutingStage::Budgets)?;

    eligible.sort_by(|left, right| {
        right
            .preference_milli
            .cmp(&left.preference_milli)
            .then_with(|| left.profile.profile_id().cmp(&right.profile.profile_id()))
    });
    let selected = eligible.first().ok_or(RoutingError::NoEligibleProfile {
        failed_at: RoutingStage::Preference,
    })?;

    Ok(RoutingDecision {
        selected_profile_id: selected.profile.profile_id(),
        hardware_profile_id: hardware.hardware_profile_id,
        evaluated_stages: ROUTING_STAGE_ORDER.to_vec(),
        eligible_profile_ids: eligible
            .iter()
            .map(|candidate| candidate.profile.profile_id())
            .collect(),
        preference_milli: selected.preference_milli,
    })
}

pub fn route_explicit_fallback(
    failed_profile: &ExecutionProfile,
    candidates: &[RouteCandidate<'_>],
    hardware: &HardwareProfile,
    now_unix_ms: u64,
    mut policy: RoutePolicy,
) -> Result<RoutingDecision, RoutingError> {
    let fallback = &failed_profile.definition().fallback_policy;
    let allowed = fallback
        .allowed_profile_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if allowed.is_empty() {
        return Err(RoutingError::InvalidFallbackPolicy);
    }

    let filtered = candidates
        .iter()
        .copied()
        .filter(|candidate| allowed.contains(&candidate.profile.profile_id()))
        .filter(|candidate| same_privacy_network_class(failed_profile, candidate.profile))
        .filter(|candidate| {
            fallback.allow_backend_change
                || same_backend(
                    &failed_profile.definition().backend,
                    &candidate.profile.definition().backend,
                )
        })
        .filter(|candidate| {
            fallback.allow_model_change
                || (failed_profile.definition().model_identity
                    == candidate.profile.definition().model_identity
                    && failed_profile.definition().model_revision
                        == candidate.profile.definition().model_revision)
        })
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        return Err(RoutingError::InvalidFallbackPolicy);
    }

    policy.pinned_profile_id = None;
    route_profile(&filtered, hardware, now_unix_ms, policy)
}

pub fn hardware_supports_profile(hardware: &HardwareProfile, profile: &ExecutionProfile) -> bool {
    if hardware.validate().is_err() || profile.validate_identity().is_err() {
        return false;
    }
    let mapping = profile.definition().hardware_mapping.as_str();
    if mapping == "cpu" {
        return true;
    }
    if let Some(capability) = mapping.strip_prefix("backend:") {
        return hardware
            .backend_capabilities
            .iter()
            .any(|observed| observed == capability);
    }
    if let Some(device_id) = mapping.strip_prefix("accelerator:") {
        return hardware.accelerators.iter().any(|accelerator| {
            accelerator.backend_device_id == device_id
                && accelerator.measurement_status == MeasurementStatus::Observed
                && profile
                    .definition()
                    .resource_budget
                    .max_accelerator_memory_bytes
                    .is_none_or(|required| {
                        accelerator
                            .memory_capacity_bytes
                            .is_some_and(|available| available >= required)
                    })
        });
    }
    if let Some(expected_id) = mapping.strip_prefix("hardware:") {
        return expected_id == hardware.hardware_profile_id.to_string();
    }
    false
}

pub fn probe_local_hardware(
    hardware_profile_id: HardwareProfileId,
    observed_at_unix_ms: u64,
) -> Result<HardwareProfile, RoutingError> {
    let logical_cpus = std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(1)
        .min(MAX_LOCAL_PROBE_CAPABILITIES);
    build_hardware_profile(
        HardwareFixtureInput {
            hardware_profile_id,
            observed_at_unix_ms,
            platform: std::env::consts::OS.to_owned(),
            architecture: std::env::consts::ARCH.to_owned(),
            cpu_capabilities: vec![format!("logical-cpus:{logical_cpus}")],
            memory_capacity_or_bucket: "not-probed".into(),
            accelerators: Vec::new(),
            backend_capabilities: Vec::new(),
        },
        HardwareProfileSource::LocalProbe,
        HardwarePrivacyClass::LocalOperational,
    )
}

pub fn hardware_fixture(input: HardwareFixtureInput) -> Result<HardwareProfile, RoutingError> {
    build_hardware_profile(
        input,
        HardwareProfileSource::Fixture,
        HardwarePrivacyClass::FixtureSynthetic,
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CalibrationWorkload {
    pub calibration_id: u128,
    pub iterations: u32,
    pub synthetic_work_units: u32,
    pub max_memory_bytes: u64,
    pub max_runtime_ms: u64,
}

pub fn run_bounded_calibration(
    hardware: &HardwareProfile,
    profile: &ExecutionProfile,
    workload_fixture_id: &str,
    workload: CalibrationWorkload,
    started_at_unix_ms: u64,
) -> Result<CalibrationRun, RoutingError> {
    if workload_fixture_id.is_empty()
        || workload.iterations == 0
        || workload.iterations > MAX_CALIBRATION_ITERATIONS
        || workload.synthetic_work_units == 0
        || workload.max_memory_bytes == 0
        || workload.max_runtime_ms == 0
    {
        return Err(RoutingError::CalibrationInvalid);
    }
    hardware
        .validate()
        .map_err(|_| RoutingError::InvalidHardware)?;
    profile
        .validate_identity()
        .map_err(|_| RoutingError::CalibrationInvalid)?;

    let deterministic_units = u64::from(workload.iterations)
        .checked_mul(u64::from(workload.synthetic_work_units))
        .ok_or(RoutingError::CalibrationInvalid)?;
    if deterministic_units > workload.max_runtime_ms {
        return Err(RoutingError::CalibrationInvalid);
    }
    let finished_at_unix_ms = started_at_unix_ms
        .checked_add(deterministic_units)
        .ok_or(RoutingError::CalibrationInvalid)?;
    let supported = hardware_supports_profile(hardware, profile)
        && profile.definition().resource_budget.max_memory_bytes <= workload.max_memory_bytes
        && profile.definition().time_budget.max_request_ms <= workload.max_runtime_ms;
    let run = CalibrationRun {
        calibration_id: workload.calibration_id,
        hardware_profile_id: hardware.hardware_profile_id,
        backend_identity_ref: format!(
            "{}@{}",
            profile.definition().backend.source_or_distribution_id,
            profile.definition().backend.exact_revision_or_build_id
        ),
        profile_candidate_digest: profile.content_digest(),
        workload_fixture_id: workload_fixture_id.to_owned(),
        started_at_unix_ms,
        finished_at_unix_ms: Some(finished_at_unix_ms),
        max_memory_bytes: workload.max_memory_bytes,
        max_runtime_ms: workload.max_runtime_ms,
        observations: vec![
            CalibrationObservation {
                metric: "synthetic_work_units".into(),
                value_milli: i64::try_from(deterministic_units.saturating_mul(1000))
                    .unwrap_or(i64::MAX),
                unit: "milli-units".into(),
            },
            CalibrationObservation {
                metric: "compatibility".into(),
                value_milli: if supported { 1000 } else { 0 },
                unit: "milli-ratio".into(),
            },
        ],
        result: if supported {
            CalibrationResult::Supported
        } else {
            CalibrationResult::Unsupported
        },
        failure_class: if supported {
            None
        } else {
            Some("unsupported_fixture".into())
        },
        evidence_refs: vec![
            format!("hardware-profile:{}", hardware.hardware_profile_id),
            format!("execution-profile:{}", profile.profile_id()),
            format!("workload:{workload_fixture_id}"),
        ],
    };
    run.validate()
        .map_err(|_| RoutingError::CalibrationInvalid)?;
    Ok(run)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecommendationEvidence {
    pub recommendation_id: u128,
    pub hardware_profile_id: HardwareProfileId,
    pub previous_profile_id: Option<ExecutionProfileId>,
    pub recommended_profile_id: ExecutionProfileId,
    pub candidate_profile_ids: Vec<ExecutionProfileId>,
    pub reason_refs: Vec<String>,
}

impl RecommendationEvidence {
    pub const fn reverse_target(&self) -> Option<ExecutionProfileId> {
        self.previous_profile_id
    }
}

pub fn recommendation_from_decision(
    recommendation_id: u128,
    previous_profile_id: Option<ExecutionProfileId>,
    decision: &RoutingDecision,
) -> RecommendationEvidence {
    RecommendationEvidence {
        recommendation_id,
        hardware_profile_id: decision.hardware_profile_id,
        previous_profile_id,
        recommended_profile_id: decision.selected_profile_id,
        candidate_profile_ids: decision.eligible_profile_ids.clone(),
        reason_refs: vec![
            "routing:hard-filters-passed".into(),
            format!("preference-milli:{}", decision.preference_milli),
        ],
    }
}

fn build_hardware_profile(
    input: HardwareFixtureInput,
    source: HardwareProfileSource,
    privacy_class: HardwarePrivacyClass,
) -> Result<HardwareProfile, RoutingError> {
    let content_digest = hardware_digest(
        &input.platform,
        &input.architecture,
        &input.cpu_capabilities,
        &input.memory_capacity_or_bucket,
        &input.accelerators,
        &input.backend_capabilities,
    )?;
    let profile = HardwareProfile {
        hardware_profile_id: input.hardware_profile_id,
        observed_at_unix_ms: input.observed_at_unix_ms,
        platform: input.platform,
        architecture: input.architecture,
        cpu_capabilities: input.cpu_capabilities,
        memory_capacity_or_bucket: input.memory_capacity_or_bucket,
        accelerators: input.accelerators,
        backend_capabilities: input.backend_capabilities,
        source,
        privacy_class,
        content_digest,
    };
    profile
        .validate()
        .map_err(|_| RoutingError::InvalidHardware)?;
    Ok(profile)
}

fn ensure_nonempty<T>(values: &[T], stage: RoutingStage) -> Result<(), RoutingError> {
    if values.is_empty() {
        return Err(RoutingError::NoEligibleProfile { failed_at: stage });
    }
    Ok(())
}

fn privacy_locality_network_compatible(profile: &ExecutionProfile, policy: RoutePolicy) -> bool {
    let definition = profile.definition();
    if profile.validate_identity().is_err() {
        return false;
    }
    if policy.strict_local {
        return definition.locality == Locality::Local
            && !definition.network_constraints.allow_network
            && definition
                .network_constraints
                .allowed_endpoint_classes
                .is_empty()
            && !definition.privacy_constraints.allow_user_content_external
            && !definition.privacy_constraints.allow_telemetry
            && !definition.privacy_constraints.allow_model_download;
    }
    if definition.locality == Locality::ExplicitCloud && !policy.allow_explicit_cloud {
        return false;
    }
    if definition.network_constraints.allow_network && !policy.allow_network {
        return false;
    }
    if definition.privacy_constraints.allow_user_content_external
        && !policy.allow_external_user_content
    {
        return false;
    }
    if definition.privacy_constraints.allow_telemetry && !policy.allow_telemetry {
        return false;
    }
    if definition.privacy_constraints.allow_model_download && !policy.allow_model_download {
        return false;
    }
    true
}

fn budgets_compatible(profile: &ExecutionProfile, policy: RoutePolicy) -> bool {
    let definition = profile.definition();
    definition.resource_budget.max_memory_bytes <= policy.max_memory_bytes
        && definition.resource_budget.max_cpu_threads <= policy.max_cpu_threads
        && definition.token_budget.max_total_tokens <= policy.max_total_tokens
        && definition.time_budget.max_request_ms <= policy.max_request_ms
}

fn same_privacy_network_class(left: &ExecutionProfile, right: &ExecutionProfile) -> bool {
    let left = left.definition();
    let right = right.definition();
    left.locality == right.locality
        && left.privacy_constraints == right.privacy_constraints
        && left.network_constraints == right.network_constraints
}

fn same_backend(left: &BackendIdentity, right: &BackendIdentity) -> bool {
    left == right
}

fn validate_hardware_freshness(
    hardware: &HardwareProfile,
    now_unix_ms: u64,
    max_age_ms: u64,
) -> Result<(), RoutingError> {
    let age = now_unix_ms
        .checked_sub(hardware.observed_at_unix_ms)
        .ok_or(RoutingError::StaleHardware)?;
    if age > max_age_ms {
        return Err(RoutingError::StaleHardware);
    }
    Ok(())
}

fn hardware_digest(
    platform: &str,
    architecture: &str,
    cpu_capabilities: &[String],
    memory_capacity_or_bucket: &str,
    accelerators: &[AcceleratorObservation],
    backend_capabilities: &[String],
) -> Result<[u8; 32], RoutingError> {
    let mut encoder = CanonicalEncoder::new();
    encoder
        .push_bytes(b"golam:hardware-profile:v1")
        .map_err(|_| RoutingError::InvalidHardware)?;
    push_text(&mut encoder, platform)?;
    push_text(&mut encoder, architecture)?;
    push_text_list(&mut encoder, cpu_capabilities)?;
    push_text(&mut encoder, memory_capacity_or_bucket)?;
    encoder.push_u64(accelerators.len() as u64);
    for accelerator in accelerators {
        push_text(&mut encoder, &accelerator.backend_device_id)?;
        push_text(&mut encoder, &accelerator.device_class)?;
        match accelerator.memory_capacity_bytes {
            Some(value) => {
                encoder.push_u8(1);
                encoder.push_u64(value);
            }
            None => encoder.push_u8(0),
        }
        push_text_list(&mut encoder, &accelerator.feature_flags)?;
        encoder.push_u8(match accelerator.measurement_status {
            MeasurementStatus::Observed => 0,
            MeasurementStatus::Unavailable => 1,
            MeasurementStatus::Unsupported => 2,
            MeasurementStatus::Failed => 3,
        });
    }
    push_text_list(&mut encoder, backend_capabilities)?;
    Ok(stable_digest32(&encoder.finish()))
}

fn push_text(encoder: &mut CanonicalEncoder, value: &str) -> Result<(), RoutingError> {
    encoder
        .push_bytes(value.as_bytes())
        .map_err(|_| RoutingError::InvalidHardware)
}

fn push_text_list(encoder: &mut CanonicalEncoder, values: &[String]) -> Result<(), RoutingError> {
    encoder.push_u64(values.len() as u64);
    for value in values {
        push_text(encoder, value)?;
    }
    Ok(())
}

fn stable_digest32(bytes: &[u8]) -> [u8; 32] {
    const SEEDS: [u64; 4] = [
        0xcbf29ce484222325,
        0x9e3779b97f4a7c15,
        0x6a09e667f3bcc909,
        0xbb67ae8584caa73b,
    ];
    let mut output = [0_u8; 32];
    for (index, seed) in SEEDS.into_iter().enumerate() {
        let mut value = seed;
        for byte in bytes {
            value ^= u64::from(*byte);
            value = value.wrapping_mul(0x100000001b3);
            value ^= value.rotate_right(29);
        }
        output[index * 8..index * 8 + 8].copy_from_slice(&value.to_be_bytes());
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_profile::{
        BackendKind, EXECUTION_PROFILE_SCHEMA_VERSION, ExecutionProfileDefinition, FailureAction,
        FailurePolicy, FallbackPolicy, LatencyQualityBudget, LoadPolicy, MultimodalCapabilities,
        NetworkConstraints, PrivacyConstraints, ResourceBudget, SamplingParameters, TimeBudget,
        TokenBudget, ToolCallConformance, WarmResidencyPolicy, WorkloadClass,
    };

    fn definition(locality: Locality, mapping: &str) -> ExecutionProfileDefinition {
        ExecutionProfileDefinition {
            schema_version: EXECUTION_PROFILE_SCHEMA_VERSION,
            model_identity: "model-a".into(),
            model_revision: "rev-1".into(),
            tokenizer_identity: "tokenizer-1".into(),
            chat_template_identity: "template-1".into(),
            backend: BackendIdentity {
                kind: BackendKind::Scripted,
                source_or_distribution_id: "scripted-fixture".into(),
                exact_revision_or_build_id: "v1".into(),
                adapter_schema_version: 1,
                launch_or_feature_digest: "fixture-only".into(),
            },
            locality,
            quantization_or_precision: "fixture".into(),
            hardware_mapping: mapping.into(),
            harness_profile: "spec004".into(),
            reasoning_mode: "bounded".into(),
            tool_call_conformance: ToolCallConformance::NativeTools,
            tool_schema_mode: "fixture".into(),
            sampling: SamplingParameters {
                temperature_milli: 0,
                top_p_milli: 1000,
                top_k: 1,
                seed: Some(7),
            },
            context_policy: "bounded".into(),
            prompt_prefix_cache_policy: "disabled".into(),
            kv_cache_policy: "disabled".into(),
            warm_residency_policy: WarmResidencyPolicy {
                load_behavior: "fixture".into(),
                keep_behavior: "none".into(),
                evict_behavior: "immediate".into(),
            },
            workload_class: WorkloadClass::Interactive,
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
                max_load_ms: 10,
                max_request_ms: 100,
                max_idle_ms: 10,
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
                allow_user_content_external: locality == Locality::ExplicitCloud,
                allow_telemetry: false,
                allow_model_download: false,
            },
            network_constraints: NetworkConstraints {
                allow_network: locality == Locality::ExplicitCloud,
                allowed_endpoint_classes: if locality == Locality::ExplicitCloud {
                    vec!["explicit-cloud-api".into()]
                } else {
                    Vec::new()
                },
            },
            load_policy: LoadPolicy::FailIfUnavailable,
            failure_policy: FailurePolicy {
                max_transient_retries: 1,
                retry_backoff_ms: 1,
                deterministic_failure: FailureAction::Fail,
                context_overflow: FailureAction::ReprojectContext,
            },
            fallback_policy: FallbackPolicy {
                allow_backend_change: true,
                allow_model_change: true,
                allowed_profile_ids: Vec::new(),
            },
        }
    }

    fn profile(locality: Locality, mapping: &str) -> ExecutionProfile {
        ExecutionProfile::new(definition(locality, mapping), Vec::new()).unwrap()
    }

    fn hardware(observed_at: u64) -> HardwareProfile {
        hardware_fixture(HardwareFixtureInput {
            hardware_profile_id: HardwareProfileId::from_u128(7),
            observed_at_unix_ms: observed_at,
            platform: "fixture-os".into(),
            architecture: "fixture-arch".into(),
            cpu_capabilities: vec!["logical-cpus:8".into()],
            memory_capacity_or_bucket: "8-gib".into(),
            accelerators: Vec::new(),
            backend_capabilities: vec!["scripted".into()],
        })
        .unwrap()
    }

    fn strict_policy() -> RoutePolicy {
        RoutePolicy {
            pinned_profile_id: None,
            strict_local: true,
            allow_explicit_cloud: false,
            allow_network: false,
            allow_external_user_content: false,
            allow_telemetry: false,
            allow_model_download: false,
            max_hardware_age_ms: 1_000,
            max_memory_bytes: 4096,
            max_cpu_threads: 8,
            max_total_tokens: 1024,
            max_request_ms: 1000,
        }
    }

    #[test]
    fn hard_filter_order_precedes_preference() {
        let local = profile(Locality::Local, "backend:scripted");
        let cloud = profile(Locality::ExplicitCloud, "backend:scripted");
        let decision = route_profile(
            &[
                RouteCandidate {
                    profile: &cloud,
                    available: true,
                    preference_milli: 1_000_000,
                },
                RouteCandidate {
                    profile: &local,
                    available: true,
                    preference_milli: 1,
                },
            ],
            &hardware(100),
            150,
            strict_policy(),
        )
        .unwrap();
        assert_eq!(decision.selected_profile_id, local.profile_id());
        assert_eq!(decision.evaluated_stages, ROUTING_STAGE_ORDER);
    }

    #[test]
    fn pin_never_overrides_strict_local_denial() {
        let cloud = profile(Locality::ExplicitCloud, "backend:scripted");
        let mut policy = strict_policy();
        policy.pinned_profile_id = Some(cloud.profile_id());
        assert_eq!(
            route_profile(
                &[RouteCandidate {
                    profile: &cloud,
                    available: true,
                    preference_milli: 9_999,
                }],
                &hardware(100),
                150,
                policy,
            ),
            Err(RoutingError::NoEligibleProfile {
                failed_at: RoutingStage::PrivacyLocalityNetwork
            })
        );
    }

    #[test]
    fn local_failure_cannot_select_explicit_cloud_fixture() {
        let local = profile(Locality::Local, "backend:missing");
        let cloud = profile(Locality::ExplicitCloud, "backend:scripted");
        assert_eq!(
            route_profile(
                &[
                    RouteCandidate {
                        profile: &local,
                        available: false,
                        preference_milli: 0,
                    },
                    RouteCandidate {
                        profile: &cloud,
                        available: true,
                        preference_milli: 10_000,
                    },
                ],
                &hardware(100),
                150,
                strict_policy(),
            ),
            Err(RoutingError::NoEligibleProfile {
                failed_at: RoutingStage::Availability
            })
        );
    }

    #[test]
    fn explicit_fallback_is_named_and_same_privacy_network_class() {
        let target = profile(Locality::Local, "backend:scripted");
        let cloud = profile(Locality::ExplicitCloud, "backend:scripted");
        let mut failed_definition = definition(Locality::Local, "backend:scripted");
        failed_definition.fallback_policy.allowed_profile_ids =
            vec![target.profile_id(), cloud.profile_id()];
        failed_definition.fallback_policy.allowed_profile_ids.sort();
        let failed = ExecutionProfile::new(failed_definition, Vec::new()).unwrap();
        let decision = route_explicit_fallback(
            &failed,
            &[
                RouteCandidate {
                    profile: &cloud,
                    available: true,
                    preference_milli: 10_000,
                },
                RouteCandidate {
                    profile: &target,
                    available: true,
                    preference_milli: 1,
                },
            ],
            &hardware(100),
            150,
            strict_policy(),
        )
        .unwrap();
        assert_eq!(decision.selected_profile_id, target.profile_id());
    }

    #[test]
    fn hardware_fixture_matches_backend_and_rejects_unsupported_device() {
        let supported = profile(Locality::Local, "backend:scripted");
        let unsupported = profile(Locality::Local, "accelerator:gpu-9");
        let observed = hardware(100);
        assert!(hardware_supports_profile(&observed, &supported));
        assert!(!hardware_supports_profile(&observed, &unsupported));
    }

    #[test]
    fn local_probe_is_bounded_and_contains_no_remote_observation() {
        let observed = probe_local_hardware(HardwareProfileId::from_u128(99), 100).unwrap();
        assert_eq!(observed.source, HardwareProfileSource::LocalProbe);
        assert_eq!(
            observed.privacy_class,
            HardwarePrivacyClass::LocalOperational
        );
        assert!(observed.cpu_capabilities.len() <= MAX_LOCAL_PROBE_CAPABILITIES);
        assert!(observed.backend_capabilities.is_empty());
        assert!(observed.accelerators.is_empty());
    }

    #[test]
    fn deterministic_calibration_uses_only_fixture_inputs() {
        let selected = profile(Locality::Local, "backend:scripted");
        let observed = hardware(100);
        let workload = CalibrationWorkload {
            calibration_id: 55,
            iterations: 4,
            synthetic_work_units: 10,
            max_memory_bytes: 4096,
            max_runtime_ms: 1000,
        };
        let first =
            run_bounded_calibration(&observed, &selected, "fixture:v1", workload, 200).unwrap();
        let second =
            run_bounded_calibration(&observed, &selected, "fixture:v1", workload, 200).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.result, CalibrationResult::Supported);
        assert!(
            first
                .evidence_refs
                .iter()
                .all(|reference| !reference.contains("http"))
        );
    }

    #[test]
    fn calibration_rejects_runtime_bound_and_timestamp_overflow() {
        let selected = profile(Locality::Local, "backend:scripted");
        let observed = hardware(100);
        let too_slow = CalibrationWorkload {
            calibration_id: 56,
            iterations: 2,
            synthetic_work_units: 1,
            max_memory_bytes: 4096,
            max_runtime_ms: 1,
        };
        assert_eq!(
            run_bounded_calibration(&observed, &selected, "fixture:v1", too_slow, 200),
            Err(RoutingError::CalibrationInvalid)
        );

        let overflow = CalibrationWorkload {
            calibration_id: 57,
            iterations: 1,
            synthetic_work_units: 1,
            max_memory_bytes: 4096,
            max_runtime_ms: 1000,
        };
        assert_eq!(
            run_bounded_calibration(
                &observed,
                &selected,
                "fixture:v1",
                overflow,
                u64::MAX
            ),
            Err(RoutingError::CalibrationInvalid)
        );
    }

    #[test]
    fn recommendation_is_inspectable_reversible_and_not_privileged() {
        let selected = profile(Locality::Local, "backend:scripted");
        let decision = route_profile(
            &[RouteCandidate {
                profile: &selected,
                available: true,
                preference_milli: 42,
            }],
            &hardware(100),
            150,
            strict_policy(),
        )
        .unwrap();
        let previous = ExecutionProfileId::from_u128(123);
        let evidence = recommendation_from_decision(77, Some(previous), &decision);
        assert_eq!(evidence.reverse_target(), Some(previous));
        assert_eq!(evidence.recommended_profile_id, selected.profile_id());
        let source = include_str!("routing.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        for forbidden in ["CapabilityLease", "Approval", "EffectId", "EffectAttemptId"] {
            assert!(!source.contains(forbidden));
        }
    }

    #[test]
    fn stale_hardware_and_profile_identity_fail_closed() {
        let selected = profile(Locality::Local, "backend:scripted");
        assert_eq!(
            route_profile(
                &[RouteCandidate {
                    profile: &selected,
                    available: true,
                    preference_milli: 1,
                }],
                &hardware(1),
                2_000,
                strict_policy(),
            ),
            Err(RoutingError::StaleHardware)
        );

        let mut changed_definition = definition(Locality::Local, "backend:scripted");
        changed_definition.model_revision = "rev-2".into();
        let changed = ExecutionProfile::new(changed_definition, Vec::new()).unwrap();
        let mut policy = strict_policy();
        policy.pinned_profile_id = Some(selected.profile_id());
        assert_eq!(
            route_profile(
                &[RouteCandidate {
                    profile: &changed,
                    available: true,
                    preference_milli: 1,
                }],
                &hardware(100),
                150,
                policy,
            ),
            Err(RoutingError::NoEligibleProfile {
                failed_at: RoutingStage::RequestedOrPinned
            })
        );
    }

    #[test]
    fn accelerator_memory_compatibility_is_explicit() {
        let mut definition = definition(Locality::Local, "accelerator:gpu-0");
        definition.resource_budget.max_accelerator_memory_bytes = Some(2048);
        let selected = ExecutionProfile::new(definition, Vec::new()).unwrap();
        let observed = hardware_fixture(HardwareFixtureInput {
            hardware_profile_id: HardwareProfileId::from_u128(88),
            observed_at_unix_ms: 100,
            platform: "fixture-os".into(),
            architecture: "fixture-arch".into(),
            cpu_capabilities: vec!["logical-cpus:8".into()],
            memory_capacity_or_bucket: "8-gib".into(),
            accelerators: vec![AcceleratorObservation {
                backend_device_id: "gpu-0".into(),
                device_class: "fixture-gpu".into(),
                memory_capacity_bytes: Some(1024),
                feature_flags: Vec::new(),
                measurement_status: MeasurementStatus::Observed,
            }],
            backend_capabilities: Vec::new(),
        })
        .unwrap();
        assert!(!hardware_supports_profile(&observed, &selected));
    }
}
