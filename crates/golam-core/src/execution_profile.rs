use crate::{CanonicalEncoder, harness::ExecutionProfileId};

pub const EXECUTION_PROFILE_SCHEMA_VERSION: u16 = 1;
const MAX_PROFILE_STRING_BYTES: usize = 1024;
const MAX_PROFILE_LIST_ITEMS: usize = 64;
const MAX_BENCHMARK_REFS: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Locality {
    Local,
    ExplicitCloud,
}

impl Locality {
    const fn code(self) -> u8 {
        match self {
            Self::Local => 0,
            Self::ExplicitCloud => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolCallConformance {
    NativeTools,
    GrammarConstrained,
    TextProtocolFallback,
}

impl ToolCallConformance {
    const fn code(self) -> u8 {
        match self {
            Self::NativeTools => 0,
            Self::GrammarConstrained => 1,
            Self::TextProtocolFallback => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkloadClass {
    Interactive,
    Batch,
    Background,
}

impl WorkloadClass {
    const fn code(self) -> u8 {
        match self {
            Self::Interactive => 0,
            Self::Batch => 1,
            Self::Background => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendKind {
    Scripted,
    MistralRs,
    LlamaCpp,
    OtherQualified,
}

impl BackendKind {
    const fn code(self) -> u8 {
        match self {
            Self::Scripted => 0,
            Self::MistralRs => 1,
            Self::LlamaCpp => 2,
            Self::OtherQualified => 3,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendIdentity {
    pub kind: BackendKind,
    pub source_or_distribution_id: String,
    pub exact_revision_or_build_id: String,
    pub adapter_schema_version: u16,
    pub launch_or_feature_digest: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SamplingParameters {
    pub temperature_milli: u16,
    pub top_p_milli: u16,
    pub top_k: u32,
    pub seed: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WarmResidencyPolicy {
    pub load_behavior: String,
    pub keep_behavior: String,
    pub evict_behavior: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MultimodalCapabilities {
    pub text: bool,
    pub image_input: bool,
    pub audio_input: bool,
    pub audio_output: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceBudget {
    pub max_memory_bytes: u64,
    pub max_cpu_threads: u16,
    pub max_accelerator_memory_bytes: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeBudget {
    pub max_load_ms: u64,
    pub max_request_ms: u64,
    pub max_idle_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TokenBudget {
    pub max_input_tokens: u32,
    pub max_output_tokens: u32,
    pub max_total_tokens: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LatencyQualityBudget {
    pub max_ttft_ms: u64,
    pub max_total_latency_ms: u64,
    pub min_quality_milli: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrivacyConstraints {
    pub allow_user_content_external: bool,
    pub allow_telemetry: bool,
    pub allow_model_download: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkConstraints {
    pub allow_network: bool,
    pub allowed_endpoint_classes: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadPolicy {
    FailIfUnavailable,
    LoadOnDemand,
    RequireWarm,
}

impl LoadPolicy {
    const fn code(self) -> u8 {
        match self {
            Self::FailIfUnavailable => 0,
            Self::LoadOnDemand => 1,
            Self::RequireWarm => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureAction {
    Fail,
    RetrySeries,
    ReprojectContext,
}

impl FailureAction {
    const fn code(self) -> u8 {
        match self {
            Self::Fail => 0,
            Self::RetrySeries => 1,
            Self::ReprojectContext => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FailurePolicy {
    pub max_transient_retries: u8,
    pub retry_backoff_ms: u64,
    pub deterministic_failure: FailureAction,
    pub context_overflow: FailureAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FallbackPolicy {
    pub allow_backend_change: bool,
    pub allow_model_change: bool,
    pub allowed_profile_ids: Vec<ExecutionProfileId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionProfileDefinition {
    pub schema_version: u16,
    pub model_identity: String,
    pub model_revision: String,
    pub tokenizer_identity: String,
    pub chat_template_identity: String,
    pub backend: BackendIdentity,
    pub locality: Locality,
    pub quantization_or_precision: String,
    pub hardware_mapping: String,
    pub harness_profile: String,
    pub reasoning_mode: String,
    pub tool_call_conformance: ToolCallConformance,
    pub tool_schema_mode: String,
    pub sampling: SamplingParameters,
    pub context_policy: String,
    pub prompt_prefix_cache_policy: String,
    pub kv_cache_policy: String,
    pub warm_residency_policy: WarmResidencyPolicy,
    pub workload_class: WorkloadClass,
    pub multimodal_capabilities: MultimodalCapabilities,
    pub resource_budget: ResourceBudget,
    pub time_budget: TimeBudget,
    pub token_budget: TokenBudget,
    pub latency_quality_budget: LatencyQualityBudget,
    pub privacy_constraints: PrivacyConstraints,
    pub network_constraints: NetworkConstraints,
    pub load_policy: LoadPolicy,
    pub failure_policy: FailurePolicy,
    pub fallback_policy: FallbackPolicy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionProfile {
    profile_id: ExecutionProfileId,
    definition: ExecutionProfileDefinition,
    benchmark_refs: Vec<String>,
    content_digest: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProfileValidationError {
    UnsupportedSchemaVersion,
    EmptyField(&'static str),
    FieldTooLong(&'static str),
    ListTooLong(&'static str),
    NonCanonicalSet(&'static str),
    InvalidSampling,
    InvalidResourceBudget,
    InvalidTimeBudget,
    InvalidTokenBudget,
    InvalidLatencyQualityBudget,
    InvalidNetworkConstraints,
    InvalidFailurePolicy,
    InvalidBenchmarkRef,
    DuplicateBenchmarkRef,
    CanonicalEncoding,
    IdentityMismatch,
}

impl ExecutionProfile {
    pub fn new(
        definition: ExecutionProfileDefinition,
        benchmark_refs: Vec<String>,
    ) -> Result<Self, ProfileValidationError> {
        validate_definition(&definition)?;
        validate_benchmark_refs(&benchmark_refs)?;
        let content_digest = semantic_digest(&definition)?;
        let profile_id = profile_id_from_digest(content_digest);
        Ok(Self {
            profile_id,
            definition,
            benchmark_refs,
            content_digest,
        })
    }

    pub const fn profile_id(&self) -> ExecutionProfileId {
        self.profile_id
    }

    pub fn definition(&self) -> &ExecutionProfileDefinition {
        &self.definition
    }

    pub fn benchmark_refs(&self) -> &[String] {
        &self.benchmark_refs
    }

    pub const fn content_digest(&self) -> [u8; 32] {
        self.content_digest
    }

    pub fn with_benchmark_ref(
        &self,
        benchmark_ref: String,
    ) -> Result<Self, ProfileValidationError> {
        validate_text("benchmark_ref", &benchmark_ref)?;
        if self.benchmark_refs.len() >= MAX_BENCHMARK_REFS {
            return Err(ProfileValidationError::ListTooLong("benchmark_refs"));
        }
        if self
            .benchmark_refs
            .iter()
            .any(|existing| existing == &benchmark_ref)
        {
            return Err(ProfileValidationError::DuplicateBenchmarkRef);
        }
        let mut benchmark_refs = self.benchmark_refs.clone();
        benchmark_refs.push(benchmark_ref);
        Ok(Self {
            profile_id: self.profile_id,
            definition: self.definition.clone(),
            benchmark_refs,
            content_digest: self.content_digest,
        })
    }

    pub fn validate_identity(&self) -> Result<(), ProfileValidationError> {
        validate_definition(&self.definition)?;
        validate_benchmark_refs(&self.benchmark_refs)?;
        let expected_digest = semantic_digest(&self.definition)?;
        let expected_id = profile_id_from_digest(expected_digest);
        if expected_digest != self.content_digest || expected_id != self.profile_id {
            return Err(ProfileValidationError::IdentityMismatch);
        }
        Ok(())
    }
}

fn validate_definition(
    definition: &ExecutionProfileDefinition,
) -> Result<(), ProfileValidationError> {
    if definition.schema_version != EXECUTION_PROFILE_SCHEMA_VERSION {
        return Err(ProfileValidationError::UnsupportedSchemaVersion);
    }

    for (name, value) in [
        ("model_identity", definition.model_identity.as_str()),
        ("model_revision", definition.model_revision.as_str()),
        ("tokenizer_identity", definition.tokenizer_identity.as_str()),
        (
            "chat_template_identity",
            definition.chat_template_identity.as_str(),
        ),
        (
            "backend.source_or_distribution_id",
            definition.backend.source_or_distribution_id.as_str(),
        ),
        (
            "backend.exact_revision_or_build_id",
            definition.backend.exact_revision_or_build_id.as_str(),
        ),
        (
            "backend.launch_or_feature_digest",
            definition.backend.launch_or_feature_digest.as_str(),
        ),
        (
            "quantization_or_precision",
            definition.quantization_or_precision.as_str(),
        ),
        ("hardware_mapping", definition.hardware_mapping.as_str()),
        ("harness_profile", definition.harness_profile.as_str()),
        ("reasoning_mode", definition.reasoning_mode.as_str()),
        ("tool_schema_mode", definition.tool_schema_mode.as_str()),
        ("context_policy", definition.context_policy.as_str()),
        (
            "prompt_prefix_cache_policy",
            definition.prompt_prefix_cache_policy.as_str(),
        ),
        ("kv_cache_policy", definition.kv_cache_policy.as_str()),
        (
            "warm_residency_policy.load_behavior",
            definition.warm_residency_policy.load_behavior.as_str(),
        ),
        (
            "warm_residency_policy.keep_behavior",
            definition.warm_residency_policy.keep_behavior.as_str(),
        ),
        (
            "warm_residency_policy.evict_behavior",
            definition.warm_residency_policy.evict_behavior.as_str(),
        ),
    ] {
        validate_text(name, value)?;
    }

    if definition.backend.adapter_schema_version == 0 {
        return Err(ProfileValidationError::UnsupportedSchemaVersion);
    }
    if definition.sampling.temperature_milli > 5_000 || definition.sampling.top_p_milli > 1_000 {
        return Err(ProfileValidationError::InvalidSampling);
    }
    if definition.resource_budget.max_memory_bytes == 0
        || definition.resource_budget.max_cpu_threads == 0
    {
        return Err(ProfileValidationError::InvalidResourceBudget);
    }
    if definition.time_budget.max_load_ms == 0 || definition.time_budget.max_request_ms == 0 {
        return Err(ProfileValidationError::InvalidTimeBudget);
    }
    let combined_tokens = definition
        .token_budget
        .max_input_tokens
        .checked_add(definition.token_budget.max_output_tokens)
        .ok_or(ProfileValidationError::InvalidTokenBudget)?;
    if definition.token_budget.max_input_tokens == 0
        || definition.token_budget.max_output_tokens == 0
        || definition.token_budget.max_total_tokens < combined_tokens
    {
        return Err(ProfileValidationError::InvalidTokenBudget);
    }
    if definition.latency_quality_budget.max_ttft_ms == 0
        || definition.latency_quality_budget.max_total_latency_ms
            < definition.latency_quality_budget.max_ttft_ms
        || definition.latency_quality_budget.min_quality_milli > 1_000
    {
        return Err(ProfileValidationError::InvalidLatencyQualityBudget);
    }
    validate_canonical_strings(
        "network_constraints.allowed_endpoint_classes",
        &definition.network_constraints.allowed_endpoint_classes,
    )?;
    if definition.network_constraints.allow_network
        != !definition
            .network_constraints
            .allowed_endpoint_classes
            .is_empty()
    {
        return Err(ProfileValidationError::InvalidNetworkConstraints);
    }
    if definition.failure_policy.max_transient_retries > 16 {
        return Err(ProfileValidationError::InvalidFailurePolicy);
    }
    validate_canonical_ids(
        "fallback_policy.allowed_profile_ids",
        &definition.fallback_policy.allowed_profile_ids,
    )?;
    Ok(())
}

fn validate_benchmark_refs(benchmark_refs: &[String]) -> Result<(), ProfileValidationError> {
    if benchmark_refs.len() > MAX_BENCHMARK_REFS {
        return Err(ProfileValidationError::ListTooLong("benchmark_refs"));
    }
    for (index, benchmark_ref) in benchmark_refs.iter().enumerate() {
        validate_text("benchmark_ref", benchmark_ref)
            .map_err(|_| ProfileValidationError::InvalidBenchmarkRef)?;
        if benchmark_refs[..index].contains(benchmark_ref) {
            return Err(ProfileValidationError::DuplicateBenchmarkRef);
        }
    }
    Ok(())
}

fn validate_text(name: &'static str, value: &str) -> Result<(), ProfileValidationError> {
    if value.is_empty() {
        return Err(ProfileValidationError::EmptyField(name));
    }
    if value.len() > MAX_PROFILE_STRING_BYTES {
        return Err(ProfileValidationError::FieldTooLong(name));
    }
    Ok(())
}

fn validate_canonical_strings(
    name: &'static str,
    values: &[String],
) -> Result<(), ProfileValidationError> {
    if values.len() > MAX_PROFILE_LIST_ITEMS {
        return Err(ProfileValidationError::ListTooLong(name));
    }
    for value in values {
        validate_text(name, value)?;
    }
    if !values.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(ProfileValidationError::NonCanonicalSet(name));
    }
    Ok(())
}

fn validate_canonical_ids(
    name: &'static str,
    values: &[ExecutionProfileId],
) -> Result<(), ProfileValidationError> {
    if values.len() > MAX_PROFILE_LIST_ITEMS {
        return Err(ProfileValidationError::ListTooLong(name));
    }
    if !values.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(ProfileValidationError::NonCanonicalSet(name));
    }
    Ok(())
}

fn semantic_digest(
    definition: &ExecutionProfileDefinition,
) -> Result<[u8; 32], ProfileValidationError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_u16(definition.schema_version);
    push_text(&mut encoder, &definition.model_identity)?;
    push_text(&mut encoder, &definition.model_revision)?;
    push_text(&mut encoder, &definition.tokenizer_identity)?;
    push_text(&mut encoder, &definition.chat_template_identity)?;
    encoder.push_u8(definition.backend.kind.code());
    push_text(&mut encoder, &definition.backend.source_or_distribution_id)?;
    push_text(&mut encoder, &definition.backend.exact_revision_or_build_id)?;
    encoder.push_u16(definition.backend.adapter_schema_version);
    push_text(&mut encoder, &definition.backend.launch_or_feature_digest)?;
    encoder.push_u8(definition.locality.code());
    push_text(&mut encoder, &definition.quantization_or_precision)?;
    push_text(&mut encoder, &definition.hardware_mapping)?;
    push_text(&mut encoder, &definition.harness_profile)?;
    push_text(&mut encoder, &definition.reasoning_mode)?;
    encoder.push_u8(definition.tool_call_conformance.code());
    push_text(&mut encoder, &definition.tool_schema_mode)?;
    encoder.push_u16(definition.sampling.temperature_milli);
    encoder.push_u16(definition.sampling.top_p_milli);
    encoder.push_u64(u64::from(definition.sampling.top_k));
    push_optional_u64(&mut encoder, definition.sampling.seed);
    push_text(&mut encoder, &definition.context_policy)?;
    push_text(&mut encoder, &definition.prompt_prefix_cache_policy)?;
    push_text(&mut encoder, &definition.kv_cache_policy)?;
    push_text(
        &mut encoder,
        &definition.warm_residency_policy.load_behavior,
    )?;
    push_text(
        &mut encoder,
        &definition.warm_residency_policy.keep_behavior,
    )?;
    push_text(
        &mut encoder,
        &definition.warm_residency_policy.evict_behavior,
    )?;
    encoder.push_u8(definition.workload_class.code());
    push_bool(&mut encoder, definition.multimodal_capabilities.text);
    push_bool(&mut encoder, definition.multimodal_capabilities.image_input);
    push_bool(&mut encoder, definition.multimodal_capabilities.audio_input);
    push_bool(
        &mut encoder,
        definition.multimodal_capabilities.audio_output,
    );
    encoder.push_u64(definition.resource_budget.max_memory_bytes);
    encoder.push_u16(definition.resource_budget.max_cpu_threads);
    push_optional_u64(
        &mut encoder,
        definition.resource_budget.max_accelerator_memory_bytes,
    );
    encoder.push_u64(definition.time_budget.max_load_ms);
    encoder.push_u64(definition.time_budget.max_request_ms);
    encoder.push_u64(definition.time_budget.max_idle_ms);
    encoder.push_u64(u64::from(definition.token_budget.max_input_tokens));
    encoder.push_u64(u64::from(definition.token_budget.max_output_tokens));
    encoder.push_u64(u64::from(definition.token_budget.max_total_tokens));
    encoder.push_u64(definition.latency_quality_budget.max_ttft_ms);
    encoder.push_u64(definition.latency_quality_budget.max_total_latency_ms);
    encoder.push_u16(definition.latency_quality_budget.min_quality_milli);
    push_bool(
        &mut encoder,
        definition.privacy_constraints.allow_user_content_external,
    );
    push_bool(&mut encoder, definition.privacy_constraints.allow_telemetry);
    push_bool(
        &mut encoder,
        definition.privacy_constraints.allow_model_download,
    );
    push_bool(&mut encoder, definition.network_constraints.allow_network);
    push_text_list(
        &mut encoder,
        &definition.network_constraints.allowed_endpoint_classes,
    )?;
    encoder.push_u8(definition.load_policy.code());
    encoder.push_u8(definition.failure_policy.max_transient_retries);
    encoder.push_u64(definition.failure_policy.retry_backoff_ms);
    encoder.push_u8(definition.failure_policy.deterministic_failure.code());
    encoder.push_u8(definition.failure_policy.context_overflow.code());
    push_bool(
        &mut encoder,
        definition.fallback_policy.allow_backend_change,
    );
    push_bool(&mut encoder, definition.fallback_policy.allow_model_change);
    encoder.push_u64(definition.fallback_policy.allowed_profile_ids.len() as u64);
    for profile_id in &definition.fallback_policy.allowed_profile_ids {
        encoder.push_u128(profile_id.as_u128());
    }

    Ok(sha256(&encoder.finish()))
}

fn profile_id_from_digest(digest: [u8; 32]) -> ExecutionProfileId {
    let mut id_bytes = [0_u8; 16];
    id_bytes.copy_from_slice(&digest[..16]);
    ExecutionProfileId::from_u128(u128::from_be_bytes(id_bytes))
}

fn push_text(encoder: &mut CanonicalEncoder, value: &str) -> Result<(), ProfileValidationError> {
    encoder
        .push_bytes(value.as_bytes())
        .map_err(|_| ProfileValidationError::CanonicalEncoding)
}

fn push_text_list(
    encoder: &mut CanonicalEncoder,
    values: &[String],
) -> Result<(), ProfileValidationError> {
    encoder.push_u64(values.len() as u64);
    for value in values {
        push_text(encoder, value)?;
    }
    Ok(())
}

fn push_bool(encoder: &mut CanonicalEncoder, value: bool) {
    encoder.push_u8(u8::from(value));
}

fn push_optional_u64(encoder: &mut CanonicalEncoder, value: Option<u64>) {
    match value {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_u64(value);
        }
        None => encoder.push_u8(0),
    }
}

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256(input: &[u8]) -> [u8; 32] {
    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut padded = Vec::with_capacity(input.len().saturating_add(72));
    padded.extend_from_slice(input);
    padded.push(0x80);
    while (padded.len() & 63) != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    let mut state = [
        0x6a09e667_u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];

    for chunk in padded.chunks_exact(64) {
        let mut schedule = [0_u32; 64];
        for (index, word) in chunk.chunks_exact(4).enumerate() {
            schedule[index] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        let mut index = 16;
        while index < 64 {
            let s0 = schedule[index - 15].rotate_right(7)
                ^ schedule[index - 15].rotate_right(18)
                ^ (schedule[index - 15] >> 3);
            let s1 = schedule[index - 2].rotate_right(17)
                ^ schedule[index - 2].rotate_right(19)
                ^ (schedule[index - 2] >> 10);
            schedule[index] = schedule[index - 16]
                .wrapping_add(s0)
                .wrapping_add(schedule[index - 7])
                .wrapping_add(s1);
            index += 1;
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];

        for (&constant, &word) in SHA256_K.iter().zip(schedule.iter()) {
            let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choose = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(sum1)
                .wrapping_add(choose)
                .wrapping_add(constant)
                .wrapping_add(word);
            let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = sum0.wrapping_add(majority);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    let mut output = [0_u8; 32];
    for (index, word) in state.iter().enumerate() {
        let start = index * 4;
        output[start..start + 4].copy_from_slice(&word.to_be_bytes());
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition() -> ExecutionProfileDefinition {
        ExecutionProfileDefinition {
            schema_version: EXECUTION_PROFILE_SCHEMA_VERSION,
            model_identity: "fixture-model".into(),
            model_revision: "rev-1".into(),
            tokenizer_identity: "fixture-tokenizer".into(),
            chat_template_identity: "fixture-template".into(),
            backend: BackendIdentity {
                kind: BackendKind::Scripted,
                source_or_distribution_id: "golam-scripted".into(),
                exact_revision_or_build_id: "v1".into(),
                adapter_schema_version: 1,
                launch_or_feature_digest: "scripted-v1".into(),
            },
            locality: Locality::Local,
            quantization_or_precision: "deterministic".into(),
            hardware_mapping: "cpu".into(),
            harness_profile: "default".into(),
            reasoning_mode: "none".into(),
            tool_call_conformance: ToolCallConformance::NativeTools,
            tool_schema_mode: "canonical-json".into(),
            sampling: SamplingParameters {
                temperature_milli: 0,
                top_p_milli: 1_000,
                top_k: 1,
                seed: Some(7),
            },
            context_policy: "bounded".into(),
            prompt_prefix_cache_policy: "disabled".into(),
            kv_cache_policy: "request".into(),
            warm_residency_policy: WarmResidencyPolicy {
                load_behavior: "on-demand".into(),
                keep_behavior: "request".into(),
                evict_behavior: "idle".into(),
            },
            workload_class: WorkloadClass::Interactive,
            multimodal_capabilities: MultimodalCapabilities {
                text: true,
                image_input: false,
                audio_input: false,
                audio_output: false,
            },
            resource_budget: ResourceBudget {
                max_memory_bytes: 1_073_741_824,
                max_cpu_threads: 2,
                max_accelerator_memory_bytes: None,
            },
            time_budget: TimeBudget {
                max_load_ms: 1_000,
                max_request_ms: 30_000,
                max_idle_ms: 60_000,
            },
            token_budget: TokenBudget {
                max_input_tokens: 2_048,
                max_output_tokens: 512,
                max_total_tokens: 2_560,
            },
            latency_quality_budget: LatencyQualityBudget {
                max_ttft_ms: 2_000,
                max_total_latency_ms: 30_000,
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
            load_policy: LoadPolicy::LoadOnDemand,
            failure_policy: FailurePolicy {
                max_transient_retries: 2,
                retry_backoff_ms: 50,
                deterministic_failure: FailureAction::Fail,
                context_overflow: FailureAction::ReprojectContext,
            },
            fallback_policy: FallbackPolicy {
                allow_backend_change: false,
                allow_model_change: false,
                allowed_profile_ids: Vec::new(),
            },
        }
    }

    #[test]
    fn sha256_matches_known_vector() {
        assert_eq!(
            sha256(b"abc"),
            [
                0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
                0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
                0xf2, 0x00, 0x15, 0xad,
            ]
        );
    }

    #[test]
    fn semantic_identity_is_stable() {
        let first = ExecutionProfile::new(definition(), Vec::new()).unwrap();
        let second = ExecutionProfile::new(definition(), Vec::new()).unwrap();
        assert_eq!(first.profile_id(), second.profile_id());
        assert_eq!(first.content_digest(), second.content_digest());
        first.validate_identity().unwrap();
    }

    #[test]
    fn material_field_change_changes_identity() {
        let first = ExecutionProfile::new(definition(), Vec::new()).unwrap();
        let mut changed = definition();
        changed.model_revision = "rev-2".into();
        let second = ExecutionProfile::new(changed, Vec::new()).unwrap();
        assert_ne!(first.profile_id(), second.profile_id());
        assert_ne!(first.content_digest(), second.content_digest());
    }

    #[test]
    fn benchmark_backlinks_are_non_semantic() {
        let profile = ExecutionProfile::new(definition(), Vec::new()).unwrap();
        let with_benchmark = profile
            .with_benchmark_ref("benchmark:fixture:1".into())
            .unwrap();
        assert_eq!(profile.profile_id(), with_benchmark.profile_id());
        assert_eq!(profile.content_digest(), with_benchmark.content_digest());
        assert_eq!(with_benchmark.benchmark_refs(), &["benchmark:fixture:1"]);
    }

    #[test]
    fn invalid_profile_fails_closed() {
        let mut invalid = definition();
        invalid.model_identity.clear();
        assert_eq!(
            ExecutionProfile::new(invalid, Vec::new()),
            Err(ProfileValidationError::EmptyField("model_identity"))
        );

        let mut invalid_network = definition();
        invalid_network.network_constraints.allowed_endpoint_classes = vec!["model-api".into()];
        assert_eq!(
            ExecutionProfile::new(invalid_network, Vec::new()),
            Err(ProfileValidationError::InvalidNetworkConstraints)
        );
    }

    #[test]
    fn canonical_sets_reject_duplicates_and_reordering() {
        let mut invalid = definition();
        invalid.network_constraints.allow_network = true;
        invalid.network_constraints.allowed_endpoint_classes =
            vec!["provider-b".into(), "provider-a".into()];
        assert_eq!(
            ExecutionProfile::new(invalid, Vec::new()),
            Err(ProfileValidationError::NonCanonicalSet(
                "network_constraints.allowed_endpoint_classes"
            ))
        );
    }
}
