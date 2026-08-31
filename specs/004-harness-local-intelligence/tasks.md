# Tasks — Spec 004 Harness & Local Intelligence

**Authority**: implementation task graph only after this planning package closes canonical.  
**Planning branch rule**: DO NOT execute product implementation tasks from this branch.

## Execution rules

- Execute in dependency order.
- Re-read exact canonical `main`, `AGENTS.md`, constitution and the complete Spec 004 package before implementation begins and after each completed canonical unit.
- Any production dependency/donor use requires a completed exact Source Foundry admission record before code import/addition.
- No force-push/rebase/history rewrite/waiver.
- Any branch mutation after closeout evidence invalidates earlier exact-head CI/review evidence.
- No model output, backend feature or benchmark result becomes authority.

## Phase A — Planning closeout

- [x] **T004-001** Complete specify/clarify/research/plan/data-model/contracts/quickstart/checklist/tasks/analyze artifacts with zero unresolved material inconsistency.
- [x] **T004-002** Update branch `AGENTS.md` to the Spec 004 planning authority/read order without claiming canonical closure before merge.
- [x] **T004-003** Open a Draft planning PR from `spec/004-harness-local-intelligence` to current canonical `main`.
- [ ] **T004-004** Obtain exact-head Windows/macOS/Ubuntu CI success for the final planning head.
- [ ] **T004-005** Obtain the required substantive independent external semantic review under current repository review policy after exact-head CI.
- [ ] **T004-006** Reconcile every material planning finding without waiver; after any mutation rerun exact-head CI and fresh review.
- [ ] **T004-007** Mark planning Ready only when all planning gates pass, merge with expected-head protection and no rebase, then verify canonical `main` moved to the returned merge SHA.
- [ ] **T004-008** Verify post-merge canonical-main CI success. Only then set `SPEC_004_PLANNING_CLOSED_CANONICAL=YES` and create the implementation branch from that exact main.

## Phase B — Implementation baseline and state model

Depends on T004-008.

- [ ] **T004-010** Re-read canonical main/governance/full Spec 004 package and record the exact implementation base SHA.
- [ ] **T004-011** Create `impl/004-harness-local-intelligence` from the exact canonical planning-closeout main SHA.
- [ ] **T004-012** Map existing seven-crate ownership and prove no new crate is needed for the first bounded slice; record any justified split before creating it.
- [ ] **T004-013** Implement pure Golam-owned core types for `ExecutionProfileId`, `HardwareProfileId`, `RequestSeriesId`, `RequestAttemptId`, `CompactionId` and `ToolCallCandidateId` with bounded validation.
- [ ] **T004-014** Implement immutable/versioned `ExecutionProfile` representation carrying every frozen Spec 001 field and stable execution-semantic content identity; benchmark backlinks are non-semantic evidence metadata.
- [ ] **T004-015** Add profile validation proving material execution field changes alter identity and invalid profiles fail closed.
- [ ] **T004-016** Implement bounded `HardwareProfile` and calibration record types without broad device fingerprinting fields.
- [ ] **T004-017** Implement `ModelRequest`, normalized `ModelEvent`, `RequestAttempt` states, authenticated initiating-principal provenance and terminal reason classes.
- [ ] **T004-018** Implement `ToolCallCandidate` and bounded parse/validation result classes.
- [ ] **T004-019** Implement `ContextProjection`, `CompactionAttempt`, `CompactionArtifact` and `BenchmarkRecord` data types.
- [ ] **T004-020** Add property/unit tests for identifier/profile digest stability, size bounds, invalid enum/state transitions and serialization round trips where durable encoding exists.

## Phase C — Canonical evidence integration

Depends on T004-013..020.

- [ ] **T004-025** Define/implement forward-only ledger/operational schema additions for profile selection, request attempts, accepted model-visible evidence, compaction lifecycle and benchmark/calibration references using existing storage patterns.
- [ ] **T004-026** Prove migrations are crash-safe, preserve existing Specs 002–003 state and reject unsupported future/inconsistent schema.
- [ ] **T004-027** Persist request-attempt PREPARED with initiator attribution before backend dispatch and explicit terminal evidence after accepted stream processing.
- [ ] **T004-028** Persist accepted streamed prefixes with attempt/source identity so interruption cannot erase user-visible/model-visible evidence.
- [ ] **T004-029** Implement profile-switch canonical evidence; an in-flight attempt cannot silently change profile.
- [ ] **T004-030** Implement replay/projection support proving model-visible request history derives from canonical evidence and not from an independent mutable transcript.
- [ ] **T004-031** Add restart/fault-injection tests for incomplete request attempts and accepted partial output.

## Phase D — Scripted backend and harness state machine

Depends on Phase C. No new production inference dependency required.

- [ ] **T004-035** Implement the Golam-owned `ModelBackend` adapter contract with a deterministic scripted backend fixture.
- [ ] **T004-036** Implement harness prepare -> persist -> dispatch -> stream -> terminal state machine.
- [ ] **T004-037** Validate event type/order/size and reject unknown/out-of-contract backend events.
- [ ] **T004-038** Implement cooperative cancellation with explicit `CANCEL_REQUESTED` and terminal observation; preserve accepted prefix.
- [ ] **T004-039** Implement explicit timeout distinct from user cancellation.
- [ ] **T004-040** Implement bounded retry series with new attempt IDs and transient/deterministic/context-overflow classes.
- [ ] **T004-041** Prove retry never mutates prior attempts and never blind-replays a protected effect fixture.
- [ ] **T004-042** Add adversarial late-event/cancel race, duplicate event, out-of-order stream and backend-crash tests.

## Phase E — Tool-call candidate normalization

Depends on Phase D.

- [ ] **T004-045** Implement native structured tool-call normalization into `ToolCallCandidate`.
- [ ] **T004-046** Implement grammar-constrained normalization into the same candidate semantics.
- [ ] **T004-047** Implement bounded text-protocol fallback parser with unambiguous framing requirements.
- [ ] **T004-048** Reject malformed, oversized, unknown-tool, schema-invalid, ambiguous and duplicate candidates without execution.
- [ ] **T004-049** Prove model candidate content cannot construct/mint privileged capability/approval/protected mutation types.
- [ ] **T004-050** Add equivalent-fixture tests showing native/grammar/text modes converge on the same normalized candidate digest/semantics.

## Phase F — Context projection and compaction

Depends on Phase D and existing Spec 002 goal/session evidence.

- [ ] **T004-055** Implement bounded ContextProjection builder from exact canonical event/artifact/Goal references and current profile/context policy.
- [ ] **T004-056** Enforce Spec 003 taint/secret-redaction rules on model-visible projections.
- [ ] **T004-057** Implement deterministic compaction transaction with explicit source refs/digest and no canonical source mutation.
- [ ] **T004-058** Implement model-backed compaction through the same backend/request/cancellation contract only if needed to satisfy the bounded slice.
- [ ] **T004-059** Record compaction start/commit/failure lifecycle so crash/incomplete attempts are detectable.
- [ ] **T004-060** Implement compaction invalidation when material source/Goal/profile context changes.
- [ ] **T004-061** Prove Goal/non-negotiable constraints survive ordinary compaction and are injected independently from summaries.
- [ ] **T004-062** Add context-overflow -> compact/reproject -> new request attempt integration tests.
- [ ] **T004-063** Add fault-injection/property tests proving canonical history survives compaction and failed compaction never claims activation.

## Phase G — Profile routing and calibration

Depends on B/D.

- [ ] **T004-065** Implement routing hard-filter order: pin -> privacy/locality/network -> availability -> hardware -> budgets -> preference.
- [ ] **T004-066** Implement strict-local dominance tests proving explicit-cloud fixtures can never be selected after local failure.
- [ ] **T004-067** Implement explicit fallback policy restricted to allowed privacy/network classes.
- [ ] **T004-068** Implement local bounded HardwareProfile probes/fixture path and compatibility matching.
- [ ] **T004-069** Implement explicit bounded calibration runner using deterministic/synthetic workloads and no hidden telemetry.
- [ ] **T004-070** Implement inspectable/reversible profile recommendation evidence that never becomes authority.
- [ ] **T004-071** Add stale HardwareProfile/profile identity and unsupported-device regression tests.

## Phase H — mistral.rs exact qualification and bounded admission

Depends on scripted harness/profile correctness through Phase G. Do not add the dependency before T004-075 closes.

- [ ] **T004-075** Refresh `mistral.rs` exact implementation candidate state and select the smallest required crate/feature surface.
- [ ] **T004-076** Record exact license/notices, Cargo feature/transitive/build/generated/native/device closure and Rust/MSRV compatibility.
- [ ] **T004-077** Inspect network/model-download/telemetry/update behavior for the selected closure and define a local-file-only strict-local configuration.
- [ ] **T004-078** Prove backend-owned agent/MCP/shell/code-exec/web-search/Skills/auto-download surfaces are excluded from the Golam adapter.
- [ ] **T004-079** Qualify panic/crash/resource behavior and all selected unsafe/native/accelerator boundaries relative to `golamd`.
- [ ] **T004-080** Decide `ADMIT_MINIMAL_IN_PROCESS`, `ADMIT_AS_SIDECAR`, or `REJECT`; record the exact Source Foundry admission with independent Golam tests.
- [ ] **T004-081** Only after ADMIT, add the exact dependency/adapter and lockfile changes atomically.
- [ ] **T004-082** Implement local-file inference adapter with bounded streaming/cancellation/profile identity and no backend-owned tool execution.
- [ ] **T004-083** Run externally observed strict-local no-egress qualification for the admitted mistral.rs path.
- [ ] **T004-084** If the candidate is rejected, preserve the backend contract and continue with scripted + compatibility path without weakening Spec 004 success criteria.

## Phase I — llama.cpp compatibility qualification

Can run after Phase G; production addition is optional unless needed for the bounded product slice.

- [ ] **T004-085** Refresh exact llama.cpp implementation candidate and select exact build/release/backend configuration.
- [ ] **T004-086** Record MIT/notices plus native/build/backend closure and executable/build identity.
- [ ] **T004-087** Define authenticated/private local sidecar transport; reject generic unauthenticated localhost control.
- [ ] **T004-088** Freeze strict-local launch policy: local model path, offline mode, no URL/HF/RPC, bounded environment/handles/output/resources.
- [ ] **T004-089** Qualify process launch/cancel/termination/crash/restart behavior under Spec 003 sandbox/egress supervision.
- [ ] **T004-090** Decide `ADMIT_SIDECAR` or `DEFER/REJECT` and create exact Source Foundry record before adding any runtime artifact/build integration.
- [ ] **T004-091** If admitted, implement sidecar adapter and authenticated transport with executable/launch identity bound to evidence.
- [ ] **T004-092** Run platform-applicable external no-egress and local-control authentication tests.

## Phase J — Benchmark separation

Depends on D–G and any admitted real backend.

- [ ] **T004-095** Implement versioned deterministic workload fixtures for harness-only benchmarks.
- [ ] **T004-096** Record backend/model metrics separately from harness metrics in `BenchmarkRecord`.
- [ ] **T004-097** Benchmark scripted backend to establish deterministic harness correctness independent of model quality.
- [ ] **T004-098** If a real backend is admitted, run a separately pinned local-model qualification and record load/TTFT/throughput/resource/cache plus harness candidate/retry metrics.
- [ ] **T004-099** Prove stale profile/backend/hardware/workload changes invalidate evidence binding rather than silently reusing scores.

## Phase K — Convergence and implementation closeout

Depends on all tasks required by the selected bounded implementation posture.

- [ ] **T004-105** Run full cross-artifact/code convergence against constitution, canonical Specs 001–003, Spec 004 package and exact admitted-source records.
- [ ] **T004-106** Run focused harness/profile/compaction/cancellation/tool-call/strict-local/property/fault-injection qualification.
- [ ] **T004-107** Run full workspace format + Clippy warnings denied + full tests + existing property/fuzz/IPC/authority/no-egress gates.
- [ ] **T004-108** Obtain exact-head Windows/macOS/Ubuntu CI success.
- [ ] **T004-109** Obtain fresh substantive independent external semantic review after exact-head CI under current repository policy.
- [ ] **T004-110** Repair every material finding without waiver; any mutation requires fresh exact-head CI and fresh review.
- [ ] **T004-111** Mark implementation Ready only after all required tasks/evidence are complete and review is clean.
- [ ] **T004-112** Merge with exact expected-head protection and no rebase/force/history rewrite.
- [ ] **T004-113** Verify post-merge canonical-main CI on the returned merge SHA.
- [ ] **T004-114** Set `SPEC_004_IMPLEMENTATION_COMPLETE=YES` and `SPEC_004_CLOSED_CANONICAL=YES` only after T004-113 succeeds.
- [ ] **T004-115** Re-read canonical main and only then create bounded Spec 005.

## Dependency summary

```text
Planning A
  -> B state/types
  -> C durability
  -> D scripted harness
  -> E tool candidates
  -> F compaction
  -> G routing/calibration
  -> H mistral qualification/admission
  -> I llama compatibility qualification (parallel after G where safe)
  -> J benchmarks
  -> K convergence/CI/review/merge/post-merge
  -> Spec 005
```

No later task may infer permission to skip an earlier authority/source/CI/review gate.
