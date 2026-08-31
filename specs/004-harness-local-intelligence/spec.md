# Feature Specification: Harness & Local Intelligence

**Feature Branch**: `spec/004-harness-local-intelligence`  
**Base**: `main@6719e9997862cbe617b60e33870ef056fa3c0c70`  
**Created**: 2026-08-31  
**Status**: DRAFT_SPECIFY  
**Planning rule**: NO PRODUCT IMPLEMENTATION OR DEPENDENCY ADMISSION IN THIS PR

## Purpose

Spec 004 adds Golam's model-independent execution harness and the first bounded local-inference boundary on top of the closed-canonical Specs 002–003 trusted spine.

The harness is Golam-owned runtime logic. A model backend may generate tokens, reasoning content, structured tool-call candidates, or completion signals, but it does not own session truth, authority, effect execution, durable history, privacy mode, retry/cancellation semantics, compaction policy, or task completion truth.

Spec 004 implements the Spec 001 `ExecutionProfile` contract, qualifies exact local-inference backend candidates, and proves that harness quality can be measured separately from model quality.

## Product slice

At the end of Spec 004 implementation, the authenticated local daemon/CLI spine can prove that:

- a bounded model-independent harness can execute deterministic turn/request-series lifecycles over canonical session history;
- model-visible history is derived from canonical logged history rather than becoming a second source of truth;
- compaction is a reversible projection with explicit provenance and never destroys canonical history;
- cancellation, timeout and retry semantics do not bypass authority or duplicate protected effects;
- model output can propose typed tool calls but cannot execute or authorize effects directly;
- exact `ExecutionProfile` records bind model revision, tokenizer/chat template, backend, locality, hardware mapping, harness behavior, cache strategy, budgets, privacy/network policy and benchmark evidence;
- strict-local mode cannot silently select a cloud backend or a backend that requires external network access;
- a local backend can be selected through a replaceable adapter without making backend-specific prompt behavior core semantics;
- `mistral.rs` and `llama.cpp` are qualified at exact source/version states before any implementation-time admission;
- model capability and harness capability are benchmarked independently.

## User stories

### US1 — Replaceable model-independent harness

As a local owner, I can change model/backend without changing Golam's authority, durable-session, cancellation, retry, compaction, or tool-call semantics.

Acceptance:
- harness state uses Golam-owned types and stable interfaces;
- backend adapters receive bounded model requests and return bounded model events/results;
- backend-specific prompt/template behavior is selected through an `ExecutionProfile`, not hard-coded into core task/session semantics;
- no backend may mint capabilities, approve effects, mutate protected state, or declare a Task verified;
- unavailable or incompatible backends fail explicitly.

### US2 — Canonical history survives model context limits

As a local owner, I can use long sessions without losing canonical evidence when a model context window requires compaction.

Acceptance:
- canonical session/event history is never rewritten by compaction;
- model-visible context is a projection with source event/artifact references;
- compaction outputs are attributable model/generated or deterministic derived artifacts with taint/provenance;
- Goal Ledger/non-negotiable constraints remain outside ordinary conversational compaction;
- a compacted context can be rebuilt or invalidated when its inputs materially change;
- `FULL_CANONICAL_HISTORY_SURVIVES_COMPACTION` remains true.

### US3 — Safe cancellation and retries

As a local owner, I can stop or time-bound inference without leaving the harness in an ambiguous state or causing repeated protected effects.

Acceptance:
- cancellation has explicit requested/observed/terminal states;
- timeout/cancellation of model generation cannot retroactively erase already durable model-visible/input/output evidence;
- retry creates a new attributable request attempt rather than silently replacing prior evidence;
- a retry never replays a protected effect outside the existing Effect Gate/reconciliation semantics;
- partial model output has explicit disposition and is not silently promoted to verified completion.

### US4 — Typed tool-call candidates, not model authority

As a local owner, model-produced tool calls are treated as untrusted proposals that still pass Golam's typed authority/effect boundaries.

Acceptance:
- native tool calls, grammar-constrained calls and text-protocol fallback normalize into one Golam-owned candidate representation;
- malformed/oversized/unknown calls are rejected without execution;
- model/tool text cannot instantiate leases, approvals, secret values or protected mutations;
- no broad product tool implementation is added by this spec; deterministic fixture tools are sufficient to prove harness conformance;
- `MODEL_TOOL_CALL != AUTHORITY_OR_EFFECT_COMMIT`.

### US5 — Explicit locality and fallback behavior

As a strict-local user, local inference failure never silently sends content to a cloud model.

Acceptance:
- every profile has a locality class and network constraints;
- strict-local routing only selects local profiles whose required runtime behavior is compatible with strict-local egress rules;
- backend/model load failure is surfaced clearly;
- fallback may change model/backend only inside an explicitly allowed privacy/network class;
- cloud adapters, if represented at all in this spec, are contract fixtures only and are not required for core operation.

### US6 — Hardware-aware, evidence-based selection

As a local owner, Golam can select or recommend a local execution profile using measured hardware/backend evidence instead of model-name heuristics.

Acceptance:
- a `HardwareProfile` records bounded CPU/RAM/GPU/VRAM/accelerator and backend-support evidence without collecting unrelated device data;
- calibration measures model load and representative inference/tool-call behavior using deterministic workloads;
- unsupported hardware/backend combinations remain explicit;
- recommendations are inspectable and reversible;
- calibration never silently changes privacy or enables egress.

### US7 — Model quality and harness quality are separable

As a developer/reviewer, I can tell whether a task failure is caused by the model, the backend, or Golam's harness behavior.

Acceptance:
- benchmark records bind exact `ExecutionProfile`, hardware record, workload fixture and harness revision;
- backend/model metrics include load time, TTFT, decode throughput and resource use where measurable;
- harness metrics include tool-call validity, repair/retry count, context/compaction behavior, cancellation behavior and deterministic task-fixture success;
- the same harness can be exercised against deterministic scripted/mock backends;
- the same model/backend can be compared under controlled harness-profile variations where feasible.

## Functional requirements

- **FR-001**: Preserve the Specs 002–003 KernelApi, authenticated IPC, Effect Gate, authority, taint, secret, egress and sandbox semantics without widening the privileged kernel.
- **FR-002**: Implement Golam-owned model-independent harness interfaces for `Turn`, `ModelRequest`, request attempt/series identity, streamed model events, completion, cancellation, timeout and bounded retry.
- **FR-003**: Every model-visible input and accepted model output MUST be attributable to canonical session/request evidence. Secret-ingestion redaction rules remain authoritative exceptions to plaintext logging.
- **FR-004**: Compaction MUST create derived context artifacts/projections and MUST NOT rewrite or delete canonical history.
- **FR-005**: Goal/non-negotiable constraint state MUST remain independently durable and MUST NOT rely solely on compacted conversational text.
- **FR-006**: Compaction inputs, output digest, method/profile, source references, taint/provenance and invalidation conditions MUST be recorded.
- **FR-007**: Normalize backend tool-call output into a bounded Golam-owned `ToolCallCandidate` representation before any later tool/effect processing.
- **FR-008**: Native tool calling, grammar-constrained calling and text-protocol fallback MUST share the same downstream candidate/validation semantics.
- **FR-009**: Malformed, oversized, unknown-schema or ambiguous tool-call candidates MUST fail closed and MUST NOT execute.
- **FR-010**: Model output MUST NOT mint authority, satisfy approvals, expose protected secret plaintext, mutate protected resources, bypass taint rules, or commit external effects.
- **FR-011**: Cancellation and timeout MUST be explicit state transitions. Retry MUST create a new attributable attempt and MUST NOT erase prior request/output evidence.
- **FR-012**: Harness retry policy MUST distinguish backend-transport/transient generation failure from semantic/tool-call repair and MUST apply bounded retry budgets.
- **FR-013**: Harness retry/cancellation MUST preserve Spec 002 UNKNOWN_OUTCOME/dependent-effect rules; no model retry may cause blind replay of protected effects.
- **FR-014**: Implement the frozen Spec 001 `ExecutionProfile` contract including exact model/revision, tokenizer/chat template, backend, locality, quantization/precision, hardware mapping, harness profile, reasoning mode, tool-call conformance, schema mode, sampling, context policy, prompt/prefix cache, KV cache, warm residency, workload class, multimodal flags, resource/time/token budgets, latency/quality budget, privacy/network constraints, load/failure/fallback policy and benchmark references.
- **FR-015**: `ExecutionProfile` identity MUST be immutable/versioned or content-addressed so benchmark/evidence records cannot ambiguously refer to a changed profile.
- **FR-016**: Profile switching MUST be attributable canonical state and MUST NOT silently change locality/privacy/network class.
- **FR-017**: Strict-local mode MUST reject any profile/backend whose required operation needs unauthorized external egress. There is no silent cloud fallback.
- **FR-018**: Define a bounded `HardwareProfile` and calibration record sufficient to support backend/profile compatibility and measured recommendation.
- **FR-019**: Calibration MUST be explicit, local by default, bounded in duration/resources, reproducible from recorded inputs, and must not become a hidden telemetry path.
- **FR-020**: Qualify exact `mistral.rs` source/version/API, license/notices, dependency closure, unsafe/FFI/JIT/device surface, model-loading behavior and network behavior before implementation admission as the primary local backend candidate.
- **FR-021**: Qualify exact `llama.cpp` source/version/build/API, license/notices, dependency closure, unsafe/native-code/device surface, process lifecycle and network behavior before implementation admission.
- **FR-022**: If `llama.cpp` is admitted, its default compatibility path MUST keep unsafe C/C++ FFI outside `golamd`, using a supervised out-of-process sidecar boundary unless a later reviewed qualification proves a safer equivalent.
- **FR-023**: Backend adapters MUST be replaceable and MUST NOT own canonical session history, authority, Effect Gate state, secret policy, taint policy or task-verification truth.
- **FR-024**: The planning/research phase MUST compare exact qualified mechanisms from `Golam-Research`, `grok-build`, Goose, DeepSeek Harness and other justified sources before final harness design; research evidence does not itself admit donor code.
- **FR-025**: Source Foundry admission is required before copying, porting, vendoring, forking, or adding a direct production dependency from any researched source.
- **FR-026**: Benchmarks MUST separate backend/model metrics from harness metrics and bind results to exact profile, hardware and code revisions.
- **FR-027**: Deterministic scripted/mock backend fixtures MUST exist so core harness state-machine, cancellation, retry, compaction and tool-call normalization behavior can be tested without downloading a real model.
- **FR-028**: Real-model qualification MUST be optional to ordinary unit tests and MUST not make CI depend on network model downloads, cloud credentials or specialized accelerators.
- **FR-029**: Spec 004 MUST NOT add broad filesystem/shell/git/browser product tools, long-term memory product behavior, Desktop/computer control, Connect/channels, workers/scheduler, or release-parity breadth owned by later specs.

## Non-functional requirements

- **NFR-001 Local-first**: core harness operation and deterministic qualification require no cloud model/service.
- **NFR-002 Rust trusted path**: harness/runtime and local backend supervision remain Rust-first; unsafe backend/native code stays outside the privileged kernel and, where required, outside `golamd`.
- **NFR-003 Small TCB**: model backends and harness adapters are unprivileged runtime services and cannot acquire kernel authority by being in-process modules.
- **NFR-004 Determinism**: scripted backend fixtures and normalized harness state transitions are deterministic for identical recorded inputs.
- **NFR-005 Boundedness**: prompts, model events, tool-call candidates, retries, compaction artifacts, calibration workloads, caches and diagnostic records have explicit limits.
- **NFR-006 Crash/restart evidence**: durable request/session evidence survives restart; transient backend process/runtime state is reconstructable and never treated as canonical truth.
- **NFR-007 Privacy**: profile/calibration/benchmark records contain no secret plaintext and do not silently enable telemetry or external egress.
- **NFR-008 Portability**: Windows, macOS and Linux remain qualification targets; hardware/backend support is represented as an honest capability matrix rather than false parity.
- **NFR-009 Reproducibility**: benchmark and calibration claims record exact source/profile/hardware/workload revisions and command/config evidence.
- **NFR-010 No false completion claim**: model completion/stop reason or well-formed tool call is not equivalent to verified Task success.

## Success criteria

- **SC-001**: deterministic harness tests prove canonical history survives compaction and compacted model context references attributable source evidence.
- **SC-002**: cancellation/timeout/retry tests prove prior evidence is preserved and retries create new attempts without duplicating protected effects.
- **SC-003**: tool-call conformance tests prove native/grammar/text fallback normalize to the same bounded candidate semantics and malformed calls never execute.
- **SC-004**: strict-local routing tests prove a local-backend failure cannot select an explicit-cloud profile or otherwise widen allowed egress.
- **SC-005**: `ExecutionProfile` serialization/identity tests prove a material profile change changes its immutable/versioned identity and invalidates stale benchmark binding.
- **SC-006**: calibration tests prove unsupported hardware/backend combinations are explicit and recommendations do not silently change privacy/network class.
- **SC-007**: scripted-backend benchmarks report harness metrics independently of real model quality.
- **SC-008**: optional real-backend qualification records separately report load time, TTFT, throughput/resource evidence and harness tool-call/task-fixture behavior.
- **SC-009**: exact-head Windows/macOS/Ubuntu CI passes without requiring model downloads, cloud credentials or specialized accelerators.
- **SC-010**: planning and later implementation convergence contain no unresolved material constitution/spec/contract/task inconsistencies before their respective closeout claims.

## Out of scope

- broad filesystem/shell/process/git/browser product tools and MCP/ACP product integrations (Spec 005);
- canonical long-term memory implementation, retrieval/index product behavior and governed memory writer (Spec 005);
- Desktop/computer control (Spec 006);
- GolamConnect, native mobile and channel bridges (Spec 007 or later reviewed scope);
- workers, scheduler, automations and swarm behavior (Spec 008);
- Grok parity closure (Spec 009);
- final release benchmark qualification (Spec 010);
- requiring any cloud model/provider for core operation;
- downloading model weights in ordinary CI;
- making `mistral.rs`, `llama.cpp`, Goose, DeepSeek Harness, grok-build or Golam-Research an authority root;
- treating model completion, reasoning text or benchmark score as authority or verified Task completion.
