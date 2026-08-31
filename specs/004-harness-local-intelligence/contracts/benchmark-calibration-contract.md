# Contract: Hardware Calibration and Harness Benchmarking

Calibration and benchmarking produce evidence. They do not grant authority, change privacy class, or declare Task/release success by themselves.

## Calibration input

A calibration run binds:
- exact Golam code revision;
- candidate backend/build identity;
- candidate `ExecutionProfile` or profile digest;
- exact `HardwareProfileId`;
- deterministic workload fixture/version;
- time/resource limits;
- locality/network policy.

## Calibration behavior

Calibration MUST:
- be explicit, bounded and locally inspectable;
- use synthetic/deterministic content by default;
- avoid real secrets and user private task content;
- respect current strict-local/egress policy;
- report unsupported combinations honestly;
- leave no hidden telemetry requirement;
- preserve cancellation/timeout evidence.

A tuning utility from an inference backend may provide evidence but cannot silently mutate the active Golam profile.

## Hardware evidence

Collect only execution-relevant observations:
- platform/architecture;
- CPU/runtime capabilities needed for compatibility;
- available memory needed for fit decisions;
- backend-visible accelerator/device classes and available device memory where measurable;
- selected backend feature support;
- explicit probe/calibration failures.

Avoid unrelated stable identifiers or broad device fingerprinting fields.

## Benchmark classes

### Harness-only deterministic benchmark

Uses scripted/mock backend behavior and measures:
- request lifecycle correctness;
- stream/event ordering validation;
- cancellation/timeout state correctness;
- retry classification/counts;
- compaction transaction/projection correctness;
- tool-call normalization/repair behavior;
- strict-local routing decisions;
- protected-effect duplicate count;
- deterministic fixture task outcome.

This class is mandatory in ordinary CI.

### Real-backend/model benchmark

Uses an exact locally available model/backend and may measure:
- model/backend load time;
- TTFT;
- prompt processing throughput;
- decode throughput;
- memory/resource observations;
- cache/warm-residency behavior;
- tool-call candidate validity under the exact model/template;
- harness repair/retry behavior;
- representative bounded task-fixture success.

This class is qualification evidence and MUST NOT be required for normal CI unless a future repository-controlled fixture is proven practical and platform-safe.

## Separation rule

A benchmark record MUST keep backend/model metrics separate from harness metrics.

A slow model with correct harness behavior and a fast model with broken harness behavior are different outcomes. Aggregating them into one opaque score is insufficient for release evidence.

## Reproducibility

Every record binds:
- code revision;
- benchmark schema version;
- execution profile identity;
- hardware profile identity;
- backend/model exact identity;
- workload fixture/version;
- commands/configuration needed to reproduce;
- start/end timestamps;
- raw evidence references;
- result/failure class.

## Invalid evidence

Evidence is stale/invalid for a claim when a material input changes, including:
- profile content under a different identity;
- model revision/template;
- backend build/feature closure;
- harness semantics;
- workload fixture/verifier;
- hardware profile relevant to the metric.

## Privacy and networking

Benchmark/calibration cannot enable network merely to improve results. Strict-local benchmark evidence must be compatible with external no-egress verification for any admitted backend process.

No secret plaintext belongs in benchmark artifacts.

## Result states

```text
PASS
FAIL
UNSUPPORTED
CANCELLED
INCONCLUSIVE
```

`UNSUPPORTED` and `INCONCLUSIVE` are not converted to PASS by fallback.

## Invariants

`CALIBRATION_RECOMMENDATION != PROFILE_AUTHORITY`
`BENCHMARK_SCORE != TASK_VERIFICATION`
`MODEL_METRICS != HARNESS_METRICS`
`STALE_EVIDENCE != CURRENT_PROOF`
