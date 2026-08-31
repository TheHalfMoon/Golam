# Data Model — Spec 004 Harness & Local Intelligence

**Status**: PLANNING CONTRACT — NO PRODUCT IMPLEMENTATION

## Identity types

All IDs are opaque Golam-owned identifiers. A model/provider string is never used as a security or canonical identity substitute.

### `ExecutionProfileId`

Stable identity for one immutable/versioned execution profile definition.

Material profile changes MUST create a new identity.

### `HardwareProfileId`

Stable identity for one bounded hardware capability observation/calibration subject.

### `RequestSeriesId`

Groups attributable retry attempts for one logical model request lifecycle.

### `RequestAttemptId`

Uniquely identifies one backend dispatch attempt.

### `CompactionId`

Uniquely identifies one compaction transaction/attempt family.

### `ToolCallCandidateId`

Uniquely identifies one normalized model-generated tool-call candidate.

## `ExecutionProfile`

Required fields:

```text
profile_id: ExecutionProfileId
schema_version
model_identity
model_revision
tokenizer_identity
chat_template_identity
backend: BackendIdentity
locality: LOCAL | EXPLICIT_CLOUD
quantization_or_precision
hardware_mapping
harness_profile
reasoning_mode
tool_call_conformance: NATIVE_TOOLS | GRAMMAR_CONSTRAINED | TEXT_PROTOCOL_FALLBACK
tool_schema_mode
sampling
context_policy
prompt_prefix_cache_policy
kv_cache_policy
warm_residency_policy
workload_class: INTERACTIVE | BATCH | BACKGROUND
multimodal_capabilities
resource_budget
time_budget
token_budget
latency_quality_budget
privacy_constraints
network_constraints
load_policy
failure_policy
fallback_policy
benchmark_refs[]
content_digest
```

Invariants:
- `profile_id` and `content_digest` bind the complete material definition;
- strict-local selection requires `locality=LOCAL` and compatible network constraints;
- fallback targets must remain inside the allowed privacy/network class;
- benchmark refs may point to evidence but do not alter the profile definition;
- a backend/model auto-detection result cannot silently overwrite explicit profile identity.

## `BackendIdentity`

```text
kind: SCRIPTED | MISTRAL_RS | LLAMA_CPP | OTHER_QUALIFIED
source_or_distribution_id
exact_revision_or_build_id
adapter_schema_version
launch_or_feature_digest
```

For sidecars, build/executable hash and qualified launch policy are part of evidence. For in-process Rust dependencies, exact crate/version/source/feature closure is part of evidence.

## `HardwareProfile`

```text
hardware_profile_id
observed_at
platform
architecture
cpu_capabilities
memory_capacity_or_bucket
accelerators[]
backend_capabilities[]
source: LOCAL_PROBE | FIXTURE
privacy_class
content_digest
```

`AcceleratorObservation`:

```text
backend_device_id
device_class
memory_capacity_if_known
feature_flags[]
measurement_status
```

Invariants:
- contains only execution-relevant capability evidence;
- does not imply permission to use a device/backend;
- is not an authority object;
- stale hardware observations are explicitly replaceable and benchmark records bind the exact observation used.

## `CalibrationRun`

```text
calibration_id
hardware_profile_id
backend_identity
profile_candidate_digest
workload_fixture_id
started_at
finished_at
resource_limits
observations[]
result: SUPPORTED | UNSUPPORTED | FAILED | CANCELLED
failure_class?
evidence_refs[]
```

Calibration never changes privacy/locality policy itself.

## `ModelRequest`

```text
request_series_id
request_attempt_id
session_id
turn_ref
execution_profile_id
context_projection_id
message_refs[]
tool_schema_digest?
input_budget
output_budget
time_budget
request_digest
```

The request contains no authority-bearing capability/approval material that a backend could reinterpret as permission.

## `RequestAttempt`

```text
request_series_id
request_attempt_id
attempt_index
execution_profile_id
request_digest
started_at
state
retry_parent_attempt_id?
retry_reason?
backend_observation
accepted_output_refs[]
usage?
terminal_reason?
finished_at?
```

States:

```text
PREPARED
DISPATCHED
STREAMING
CANCEL_REQUESTED
COMPLETED
CANCELLED
TIMED_OUT
FAILED_TRANSIENT
FAILED_DETERMINISTIC
FAILED_CONTEXT_OVERFLOW
```

Terminal states are append-only evidence. A retry creates a new `RequestAttempt`.

## `ModelEvent`

Normalized event family:

```text
TEXT_DELTA
REASONING_DELTA
TOOL_CALL_FRAGMENT
TOOL_CALL_COMPLETE
USAGE
STOP
BACKEND_WARNING
BACKEND_ERROR
```

Each event binds:
- request attempt;
- backend sequence/order evidence where available;
- bounded payload;
- acceptance/rejection status;
- canonical evidence reference when accepted.

Unknown/unbounded event types reject rather than becoming arbitrary canonical payload.

## `ToolCallCandidate`

```text
candidate_id
request_attempt_id
tool_name
arguments_canonical
source_mode
source_event_refs[]
taint
parse_status
schema_digest
candidate_digest
```

`parse_status`:

```text
VALIDATED_CANDIDATE
REJECTED_MALFORMED
REJECTED_OVERSIZED
REJECTED_UNKNOWN_TOOL
REJECTED_SCHEMA
REJECTED_AMBIGUOUS
REJECTED_DUPLICATE
```

Invariants:
- never stores a capability lease or approval as model-minted authority;
- validation does not execute the tool;
- fixture execution, if used for harness tests, remains behind explicit test-only boundaries;
- candidate content retains model-generated/untrusted taint until an owning later-spec path processes it.

## `ContextProjection`

```text
context_projection_id
session_id
execution_profile_id
source_event_refs[]
source_artifact_refs[]
goal_refs[]
compaction_artifact_refs[]
taint
budget
render_policy_digest
rendered_digest
created_at
```

This is what request construction derives from. It is a projection, not canonical history.

## `CompactionAttempt`

```text
compaction_id
attempt_index
session_id
source_projection_id
source_event_refs[]
source_digest
execution_profile_id?
method: DETERMINISTIC | MODEL_BACKED
state
started_at
failure_class?
finished_at?
```

States:

```text
STARTED
DERIVING
VALIDATING
COMMITTED
CANCELLED
FAILED_CHANGED_SOURCE
FAILED_TRANSIENT
FAILED_DETERMINISTIC
FAILED_PERSISTENCE
```

A `STARTED` record without terminal completion after crash is incomplete evidence, not success.

## `CompactionArtifact`

```text
compaction_artifact_id
compaction_id
source_event_refs[]
source_digest
summary_or_projection_ref
projection_digest
taint
method_profile_digest
estimated_tokens_before?
estimated_tokens_after?
created_at
invalidated_by[]
```

Invariants:
- does not delete or mutate source canonical events;
- references exact source evidence;
- material source/goal/policy changes may invalidate reuse;
- `SECRET_DERIVED` and other taint remains subject to Spec 003 rules.

## `BenchmarkRecord`

```text
benchmark_id
benchmark_schema_version
code_revision
execution_profile_id
hardware_profile_id
workload_fixture_id
backend_metrics
harness_metrics
started_at
finished_at
result
raw_evidence_refs[]
```

`BackendMetrics` may include load time, TTFT, prompt/decode throughput, memory and cache observations.

`HarnessMetrics` may include candidate-validity rate, retries by class, cancellation correctness/latency, compaction outcomes, deterministic fixture success and protected-effect duplicate count.

Invariants:
- no benchmark result can be reused across a materially changed profile under the same identity;
- real-model stochastic evidence is distinguished from scripted deterministic harness evidence;
- benchmark scores are evidence, not authority or Task verification by themselves.

## Persistence classification

Canonical/durable:
- profile definitions and selections;
- request series/attempt lifecycle;
- accepted model-visible evidence;
- normalized candidate evidence needed for replay/explanation;
- compaction transaction/artifact metadata;
- calibration/benchmark records and evidence references.

Rebuildable/transient:
- model process handles;
- active network connections;
- live stream buffers before acceptance;
- tokenizer/runtime object caches;
- KV/prefix caches;
- warm model residency;
- derived routing recommendation caches.

## Security classification

No structure in this data model is authority-bearing merely because it names a model, tool, device or route. Existing Spec 003 protected authority state remains separate.

`MODEL_STATE != AUTHORITY_STATE`
`PROFILE_COMPATIBILITY != PERMISSION`
`BENCHMARK_SCORE != RELEASE_OR_EFFECT_AUTHORITY`
