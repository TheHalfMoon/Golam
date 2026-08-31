# Implementation Plan — Spec 004 Harness & Local Intelligence

**Branch**: `spec/004-harness-local-intelligence`  
**Base**: `main@6719e9997862cbe617b60e33870ef056fa3c0c70`  
**Status**: PLANNING — NO PRODUCT IMPLEMENTATION IN THIS PR

## Summary

Build a small Golam-owned model harness above the closed Specs 002–003 trusted spine. The harness derives bounded model requests from canonical session evidence, records request attempts and streamed outcomes, normalizes model tool-call output into untrusted typed candidates, supports explicit cancellation/timeout/bounded retry, and creates provenance-bound compaction projections without rewriting canonical history.

Implement the frozen Spec 001 `ExecutionProfile`, add a bounded `HardwareProfile`/calibration record, and qualify replaceable local inference backends. `mistral.rs` remains the primary Rust-native candidate; `llama.cpp` remains an out-of-process compatibility candidate. Neither is admitted by this planning PR.

## Constitution check

| Gate | Result |
|---|---|
| Local ownership / strict-local | PASS_SPEC — deterministic harness is fully local; profile routing cannot silently widen locality/network class |
| Rust trusted path | PASS_SPEC — Golam harness/supervision is Rust-first |
| Small privileged kernel | PASS_SPEC — model/harness remain unprivileged; existing KernelApi remains authority root |
| Explicit authority | PASS_SPEC — model/tool output is candidate data only |
| Durable effects | PASS_SPEC — retries/cancellation cannot bypass Effect Gate or UNKNOWN_OUTCOME semantics |
| Secrets/taint | PASS_SPEC — model-visible projections inherit Spec 003 redaction/taint rules |
| Model replaceability | PASS_SPEC — Golam-owned backend contract + immutable ExecutionProfile |
| Source governance | PASS — planning admits zero production dependencies/donors |
| Verification | PASS_SPEC — scripted backend plus exact profile/hardware/benchmark binding |

## Preserve the seven-package spine

Spec 004 begins inside the current workspace:

```text
crates/
  golam-core
  golam-ledger
  golam-effects
  golam-ipc
  golam-kernel
  golamd
  golam
```

Do not create `golam-harness` or `golam-models` just to mirror architecture names. Start with modules in existing ownership boundaries. Split only if implementation evidence demonstrates a durable independent testing/ownership boundary and the split does not widen authority.

Initial ownership target:
- `golam-core`: pure bounded harness/profile/backend protocol types, validation and deterministic state transitions that do not require privileged state;
- `golam-ledger`: canonical request/profile/compaction evidence persistence/projection hooks using existing ledger durability patterns;
- `golamd`: unprivileged harness coordinator, backend lifecycle/supervision and routing under kernel decisions;
- `golam`: bounded diagnostics/qualification UX only where needed to prove the slice;
- `golam-kernel` / `golam-effects`: no model semantics; only existing authority/effect APIs are consumed.

## Harness architecture

```text
canonical Session/Goal evidence
          |
          v
Context Projection Builder
          |  exact source refs + taint + budgets
          v
ExecutionProfile Router ---- HardwareProfile / Calibration
          |
          v
ModelRequest + RequestAttempt START (durable)
          |
          v
ModelBackend adapter
          |
          +--> StreamDelta / ReasoningDelta / Usage / Stop / Error
          +--> ToolCallCandidate fragments
          |
          v
Normalize + validate bounded ModelEvents
          |
          +--> append accepted model-visible evidence
          +--> ToolCallCandidate (UNTRUSTED, no execution)
          |
          v
Harness terminal transition
  COMPLETE | CANCELLED | TIMED_OUT | FAILED | RETRY_ELIGIBLE
```

A later owning tool layer may submit a validated candidate through normal typed authority/effect gates. Spec 004 fixture tools stop before real product execution.

## Request identity and retry model

Use a stable `RequestSeriesId` for one logical model request lifecycle and a unique `RequestAttemptId` per dispatch.

```text
series S
  attempt A1 -> transient failure
  attempt A2 -> context overflow -> compaction projection
  attempt A3 -> completed
```

Rules:
- attempts are append-only evidence;
- retry never mutates A1/A2 into success;
- retry reason and inherited profile/context identity are explicit;
- material profile/context change starts a new attributable attempt with the changed identity recorded;
- no model retry redispatches a protected external effect;
- bounded retry budgets are part of the harness profile.

## Streaming and cancellation

Accepted stream material is assembled through bounded event types. A delivered prefix is evidence even if generation is later cancelled.

Cancellation sequence:
1. record/request cancellation;
2. signal backend adapter;
3. stop accepting unauthorized late data beyond the adapter contract;
4. record backend acknowledgement/termination observation when available;
5. persist interrupted accepted prefix and terminal reason;
6. leave any protected effect state to existing Effect Gate reconciliation.

Timeout is represented independently from user cancellation even if both use the same backend cancellation primitive.

## Compaction model

Compaction is a projection transaction, never canonical deletion.

```text
CompactionAttempt START
  -> select exact source refs/range
  -> derive bounded summary/projection
  -> validate non-empty/budget/taint/source binding
  -> persist CompactionArtifact
  -> activate projection generation
CompactionAttempt END
```

Crash/incomplete work remains detectable from durable attempt state. A failed compaction does not pretend history changed. Model-backed compaction uses the same backend contract and cancellation/budget rules; deterministic pruning/projection may be used where sufficient.

Goal/non-negotiable constraints remain separately durable and injected by explicit context policy, not trusted to a summary.

## Tool-call normalization

Backend-specific output is normalized to:

```text
ToolCallCandidate {
  candidate_id
  request_attempt_id
  tool_name
  arguments
  source_mode
  source_event_refs
  taint
  parse_status
  schema_digest
}
```

Modes:
- `NATIVE_TOOLS`
- `GRAMMAR_CONSTRAINED`
- `TEXT_PROTOCOL_FALLBACK`

The normalized representation carries no capability/approval/effect authority. Duplicate IDs, unknown tools, invalid schema, malformed arguments, oversized payloads or ambiguous fallback framing reject before any fixture dispatch.

## ExecutionProfile

Preserve every frozen Spec 001 field. Implementation must assign a stable immutable/content-derived `profile_id` and a schema version.

Routing order:
1. requested/pinned profile constraints;
2. strict-local/privacy/network hard compatibility;
3. model/backend availability;
4. HardwareProfile compatibility;
5. workload/resource/latency constraints;
6. evidence-based recommendation among remaining candidates.

No ranking score can override a locality/privacy hard incompatibility.

## HardwareProfile and calibration

Bounded fields:
- platform/architecture;
- CPU execution-relevant capability summary;
- available/system memory bucket or measured bytes needed for fit decisions;
- backend-visible accelerators/devices and memory where measurable;
- backend/build feature support;
- calibration workload identity and result records.

Calibration is explicit local work. It uses deterministic/synthetic prompts and local artifacts, has time/resource limits and produces no hidden telemetry.

## Backend contract

Required operations conceptually:
- `probe()` — local capability/build identity without model execution where possible;
- `load(profile)` / explicit load result;
- `stream(request, cancel)` — bounded stream of normalized adapter events;
- `cancel(request_attempt)` where backend requires an explicit command;
- `unload()` / lifecycle observation;
- optional metrics snapshot.

Backend adapters do not receive KernelApi authority-bearing mutation capability. Any model-file access or process launch is separately authorized by the owning runtime path.

## mistral.rs implementation qualification plan

Before adding a dependency:
1. select exact source/release and minimal crate/feature set;
2. record complete Cargo feature/transitive/build/native closure;
3. verify licenses/notices/generated/vendor obligations;
4. verify local-file-only model loading with network unavailable;
5. exclude agent/MCP/shell/code-exec/web-search/skills/server features from Golam integration;
6. inspect unsafe/native/device boundaries and crash implications;
7. qualify CPU baseline plus platform-applicable accelerator builds separately;
8. prove no telemetry/update/model-download egress on strict-local path;
9. add only the minimum admitted surface.

If the in-process closure cannot satisfy trust/runtime constraints, switch to a sidecar without changing the Golam backend contract.

## llama.cpp implementation qualification plan

Before admission:
1. select exact release/commit and build configuration;
2. record native/build dependency and backend closure;
3. use local model paths and offline mode;
4. prohibit URL/HF/RPC options on strict-local profiles;
5. supervise the child under Spec 003 sandbox/egress rules;
6. use authenticated/private OS-local transport; generic unauthenticated localhost HTTP is forbidden;
7. bind executable/build hash and launch configuration to backend/profile evidence;
8. qualify cancellation/termination, stdout/stderr bounds, crash/restart and no-egress behavior on each claimed platform.

No direct libllama FFI inside `golamd` in Spec 004.

## Data durability

Extend existing forward-only operational/ledger schemas only when needed. Do not create a second canonical database.

Durable families:
- execution profile definition/selection events;
- request series/attempt lifecycle evidence;
- accepted model-visible output evidence/usage;
- compaction attempt/artifact/activation evidence;
- calibration/benchmark record references.

Transient token/KV/backend process state is cache/runtime state and is not canonical.

## Test strategy

### Deterministic harness tests
- request-series/attempt transition tables;
- streamed prefix assembly and duplicate/out-of-order event rejection;
- user cancellation, timeout, backend failure and late-event races;
- bounded transient retry vs deterministic no-retry;
- context-overflow -> compaction -> new attempt;
- canonical history unchanged by compaction;
- Goal Ledger injection survives compaction;
- tool-call native/grammar/text normalization equivalence;
- malformed/oversized/unknown candidate denial;
- strict-local profile routing dominance;
- immutable profile identity/change invalidation;
- HardwareProfile compatibility and reversible recommendation.

### Effect/authority regression
- model output cannot construct protected authority types;
- retry/cancel does not duplicate protected fixture effects;
- secret/taint/redaction invariants remain intact;
- model sidecar/network launch still passes current Spec 003 sandbox/egress gates.

### Backend qualification
- no real model required for ordinary CI;
- optional small local fixture/model qualification is separately pinned and reproducible;
- external strict-local observation is required for every admitted sidecar/backend process path;
- Windows/macOS/Ubuntu exact-head CI remains mandatory.

## Benchmark strategy

Record backend/model and harness dimensions separately.

Backend/model:
- load time/success;
- TTFT;
- prompt/decode throughput;
- peak/steady memory where measurable;
- warm residency/cache behavior;
- device/backend compatibility.

Harness:
- tool-call candidate validity;
- repair/retry counts by class;
- cancellation terminal correctness;
- compaction attempts/savings/failure classes;
- context rebuild correctness;
- deterministic task-fixture success;
- protected-effect non-duplication.

## Planning exit gate

The planning PR may close only when:
- specify/clarify/research/plan/data-model/contracts/quickstart/checklist/tasks/analyze are complete;
- production donor/dependency admission remains zero;
- cross-artifact analysis has no unresolved material findings;
- exact-head Windows/macOS/Ubuntu CI succeeds;
- required independent external semantic review is substantive and clean under the current repository review policy;
- planning PR is merged and canonical post-merge CI is green.

Only then may an implementation branch be created from the exact new canonical main.

## Implementation exit gate

Spec 004 implementation may close only when:
- all implementation tasks are complete in dependency order;
- deterministic harness/profile/compaction/cancellation/tool-call tests pass;
- any admitted backend has a complete exact Source Foundry record and strict-local qualification;
- model-vs-harness benchmark evidence is reproducible and revision-bound;
- exact-head Windows/macOS/Ubuntu CI is green;
- independent post-CI review has no unresolved material findings;
- implementation merges with expected-head protection and post-merge canonical main CI is green before Spec 005 starts.
