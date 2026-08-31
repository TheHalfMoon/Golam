# Research — Spec 004 Harness & Local Intelligence

**Date**: 2026-08-31  
**Base**: `main@6719e9997862cbe617b60e33870ef056fa3c0c70`  
**Posture**: RESEARCH EVIDENCE ONLY — NO CODE OR DEPENDENCY ADMISSION

## Research questions

1. Which harness lifecycle mechanisms should Golam own rather than delegate to a backend?
2. How should canonical history, compaction, cancellation, retries and tool-call candidates compose with the closed Specs 002–003 authority/effect spine?
3. Which local inference backend boundaries are technically plausible under Rust-first, strict-local and small-TCB constraints?
4. What evidence is required to separate model/backend quality from harness quality?

## Exact observed source states

| Source | Exact observed state | License observed | Research role |
|---|---|---|---|
| `TheHalfMoon/Golam-research` | `main@a9f633e09d49a85829b8236331b9e21f7e612634` | founder-permission/provenance rules remain governed by canonical Spec 001 attestation | reconstructed runtime behavior evidence |
| `xai-org/grok-build` | `main@bc7f02eddd3d84085849dc19ed216f11c23b0571` | Apache-2.0 | current Rust harness/session/compaction evidence |
| `aaif-goose/goose` | `main@7d97fe1eadedd8c7f8cc25e9a537b7ab8c8b1dd5` | Apache-2.0 | Rust agent state-machine and provider/tool lifecycle evidence |
| `deepseek-ai/deepseek-harness` | `master@0a53fb55bea101816fa226bb964ae2bed71c343b` | MIT | explicit harness capability seams/session log evidence |
| `EricLBuehler/mistral.rs` | `master@d348a88833c5da8403e2320997c28cdd02ae4f4b` | MIT | primary Rust-native local inference candidate |
| `ggml-org/llama.cpp` | `master@010be9683afabe14ce299197b38c329f94bae568` | MIT | compatibility local-inference sidecar candidate |

Exact upstream state is research evidence, not a production pin. Any later admission must record the exact implementation-selected state again.

## R1 — Golam-Research routing/transcript behavior

Inspected `source/node-agent-coordinator/inference-router.ts` at the exact Golam-Research state.

Observed mechanisms:
- explicit provider routing;
- local transcript persistence with bounded retained entries;
- per-agent serialized queues;
- streaming assistant projection;
- provider-specific MCP/direct-tool bridges;
- local/remote transcript projection.

Material incompatibility with Golam architecture:
- the router can dispatch routed tool execution directly from the provider path.

Decision:
- retain provider-routing, streaming and attributable-transcript behavior as reference evidence;
- reject direct backend/router-to-tool execution as a Golam architecture pattern;
- normalize model output to an untrusted `ToolCallCandidate` and preserve the existing KernelApi/Effect Gate path.

`PROVIDER_ROUTING != TOOL_AUTHORITY`

## R2 — grok-build compaction and turn evidence

At `xai-org/grok-build@bc7f02ed...`, relevant current files include session run-loop, turn, tool-call, goal-tracker, compaction and full-replace compaction helpers.

`full_replace_compaction.rs` shows:
- a compaction sampler seam separate from its observer/telemetry seam;
- explicit per-attempt evidence;
- transient versus deterministic failure classification;
- context-overflow classification;
- bounded retry/degradation accounting;
- cancellation via `tokio_util::sync::CancellationToken`;
- retention of exact attempted items and successful output telemetry.

Decision:
- adopt the mechanism principle of explicit attempt identity, error classes, cancellation and durable/reproducible compaction evidence;
- do not copy backend-specific prompts, telemetry schemas or shell semantics into Golam core contracts;
- Golam compaction remains a projection over its existing canonical event ledger.

## R3 — Goose agent state machine

At `aaif-goose/goose@7d97fe1e...`, `crates/goose/src/agents/agent.rs` demonstrates:
- an explicit Rust state-machine decomposition including compaction, retry, tool execution and steering operations;
- `CancellationToken` for cooperative cancellation;
- a dedicated `RetryManager`;
- persisted message identity before/while projecting conversation state;
- provider/model config separated from conversation/tool/system-prompt data.

Decision:
- explicit state-machine and cancellation/retry separation is consistent with Golam's target;
- Goose permission/tool architecture is not Golam authority and is not imported as an authority root;
- Golam's closed Spec 003 authorization/effect semantics remain upstream of any future product tool execution.

## R4 — DeepSeek Harness session/compaction lifecycle

At `deepseek-ai/deepseek-harness@0a53fb55...`:

`docs/subsystems/compaction.md` records:
- compaction as an optional capability seam rather than the agent-loop spine;
- durable log-only `compaction/start`, `compaction/summary` and `compaction/end` records;
- a summary replacement as a surface projection rather than canonical-log deletion;
- start-before-work/end-after-commit bracketing so a crash produces detectable incomplete evidence;
- summary provenance including selected/shadowed events, token count, provider/model and usage;
- explicit busy/cancelled/changed/summary/commit/persistence failure classes;
- cancellation signal forwarding;
- tool-call/result pairing checks around compaction boundaries.

`packages/core/agent-loop/README.md` records:
- turn/step lifecycle driven from persisted session history;
- each accepted fact appended before the next derived request;
- cooperative cancellation;
- delivered streamed prefix preserved when cancellation occurs;
- explicit request-error retry action;
- undispatched post-cancellation tool calls represented by synthetic aborted results;
- request route/header evidence sufficient for reconstruction.

Decision:
- adopt the durable lifecycle principles: explicit turn/request identity, logged route/profile, cancellation evidence, retry as new attempt and projection-only compaction;
- keep Golam-specific authority/effect/session schemas rather than importing the framework.

## R5 — mistral.rs qualification findings

Exact observed source: `EricLBuehler/mistral.rs@d348a88833c5da8403e2320997c28cdd02ae4f4b`.

Observed:
- workspace version `0.9.2`, MIT, Rust 1.94;
- Rust SDK and core crates exist;
- supported local model/GGUF paths and streaming/tool/grammar capabilities exist;
- hardware-aware tuning and multiple accelerator features exist;
- workspace includes broad optional agentic/MCP/code-exec/sandbox functionality;
- core/SDK dependency surface includes `hf-hub`, `reqwest` and other network-capable paths;
- Candle dependencies are git-pinned at exact upstream revision in this observed state;
- accelerator/native surfaces include CUDA/Metal/BLAS-related and other platform-specific code.

Risks for Golam:
- auto model retrieval/network-capable dependencies are incompatible with an unqualified strict-local assumption;
- importing broad agentic features would duplicate or bypass Golam-owned harness/authority/tool semantics;
- large transitive/native/device surface requires exact feature-closure qualification;
- an in-process backend can expand crash/unsafe/device dependency exposure even if authority types remain sealed.

Decision:
- `PRIMARY_CANDIDATE_NOT_YET_ADMITTED`;
- implementation qualification should prefer the smallest Rust SDK/core feature subset capable of local-file inference and streaming;
- disable/avoid backend-owned agent loop, shell, code execution, MCP, web search, skills and auto-download behavior;
- require deterministic offline/network tests and exact transitive/feature records before admission;
- if the minimal in-process closure is still too broad or conflicts with Golam's trust-path constraints, use a supervised sidecar or reject the candidate without changing the Golam backend contract.

## R6 — llama.cpp qualification findings

Exact observed source: `ggml-org/llama.cpp@010be9683afabe14ce299197b38c329f94bae568`.

Observed:
- MIT C/C++ implementation;
- broad CPU/GPU backend support;
- `llama-server` exposes OpenAI-compatible HTTP APIs plus tool/function/schema capabilities;
- command surface can download model artifacts from URLs/Hugging Face;
- `--offline` exists to force cache/local operation and prevent network access;
- optional RPC/device/network surfaces exist;
- the process is native C/C++ and therefore outside Golam's Rust/`unsafe_code = forbid` trusted implementation posture.

Decision:
- `COMPATIBILITY_CANDIDATE_NOT_YET_ADMITTED`;
- do not link libllama directly into `golamd` by default;
- prefer a supervised sidecar with local model paths and explicit offline mode;
- do not expose a generic unauthenticated localhost server; use a private authenticated local IPC boundary or a narrowly authenticated loopback shim that satisfies Golam local-control protections;
- disable/deny model URL, Hugging Face download and RPC options for strict-local profiles;
- bind executable/build identity and launch arguments into the `ExecutionProfile`/backend evidence.

## R7 — Harness-owned state model

Research convergence supports the following Golam-owned sequence:

```text
canonical session evidence
  -> build ModelRequest from exact ExecutionProfile + context projection
  -> append request-attempt evidence
  -> backend adapter stream
  -> normalize ModelEvent records
  -> optional ToolCallCandidate records
  -> validate candidate (no execution authority)
  -> downstream typed authority/effect path when an owning tool exists
  -> append terminal request-attempt result
  -> continue / retry / compact / stop by explicit harness state
```

Core invariants:
- `MODEL_VISIBLE => LOGGED`
- `COMPACTION != CANONICAL_HISTORY_REWRITE`
- `RETRY != ATTEMPT_REWRITE`
- `CANCELLED_STREAM_PREFIX != DISCARDED_EVIDENCE`
- `MODEL_TOOL_CALL != EFFECT_COMMIT`
- `BACKEND_FAILURE != PERMISSION_TO_CHANGE_PRIVACY_CLASS`

## R8 — ExecutionProfile and HardwareProfile

The frozen Spec 001 `ExecutionProfile` remains authoritative and must not be narrowed to provider/model strings.

Planning addition:
- profile identity is immutable/versioned or content-addressed;
- benchmark evidence binds exact profile identity and hardware calibration record;
- backend executable/crate/build identity belongs in backend-specific profile evidence;
- hidden runtime auto-detection may inform a recommendation but cannot override explicit locality/privacy/network or exact-model requirements.

`HardwareProfile` remains bounded to execution-relevant local evidence. Calibration is explicit and user-inspectable and does not silently transmit telemetry.

## R9 — Benchmark separation

Backend/model metrics:
- load success/time;
- TTFT;
- prompt/decode throughput where measurable;
- memory/resource use;
- cache/warm-residency behavior;
- backend/device compatibility.

Harness metrics:
- normalized tool-call validity;
- repair/retry count and reason;
- cancellation latency/state correctness;
- compaction trigger/result/invalidation behavior;
- context budget behavior;
- deterministic fixture task success;
- stale-profile/route rejection;
- protected-effect non-duplication across retry/cancel paths.

A scripted deterministic backend is mandatory so harness correctness can be qualified independently from model stochasticity or model downloads.

## Source-admission result

Planning admits **no production donor code and no production dependency**.

| Candidate | Planning disposition |
|---|---|
| Golam-Research | `REFERENCE_ONLY` |
| grok-build | `REFERENCE_ONLY` |
| Goose | `REFERENCE_ONLY` |
| DeepSeek Harness | `REFERENCE_ONLY` |
| mistral.rs | `PRIMARY_CANDIDATE_NOT_YET_ADMITTED` |
| llama.cpp | `COMPATIBILITY_CANDIDATE_NOT_YET_ADMITTED` |

Any implementation-time admission requires a separate exact bounded Source Foundry record with license/notices, selected files/crates/features, transitive closure, unsafe/FFI/native/process/network behavior, platform posture and independent Golam verification.

## Research conclusion

No researched system should become Golam's harness or authority root. The strongest convergent pattern is a small Golam-owned turn/request state machine over canonical durable evidence, replaceable model adapters, projection-only compaction, explicit cancellation/retry outcomes, normalized untrusted tool-call candidates and profile-bound reproducible benchmarking.

`RESEARCH_CONVERGENCE=READY_FOR_PLAN`
