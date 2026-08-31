# Implementation Readiness Checklist — Spec 004

**Planning branch**: `spec/004-harness-local-intelligence`  
**Implementation authorization**: NO until planning merges and post-merge canonical CI succeeds.

## Specification quality

- [x] Scope is bounded to Harness & Local Intelligence.
- [x] User stories and measurable success criteria exist.
- [x] Later-spec tools/memory/Desktop/Connect/workers/parity scopes are explicitly excluded.
- [x] Model/backend output is explicitly non-authoritative.
- [x] Canonical history and compaction semantics preserve Specs 001–002 invariants.
- [x] Cancellation, timeout and retry semantics are explicit.
- [x] Strict-local fallback behavior is fail-closed.

## Clarification and research

- [x] Material planning ambiguities are closed in `clarification-closeout.md`.
- [x] Exact observed source states are recorded.
- [x] Golam-Research, grok-build, Goose and DeepSeek Harness mechanisms were compared.
- [x] mistral.rs and llama.cpp exact candidates were researched.
- [x] Current source licenses for mistral.rs, llama.cpp, Goose, grok-build and DeepSeek Harness were verified.
- [x] No production donor code or dependency is admitted by planning.
- [x] Exact implementation-time Source Foundry predecessor gates are stated.

## Plan and contracts

- [x] Constitution check is complete.
- [x] Existing seven-package workspace is preserved initially.
- [x] Harness/backend ownership boundary is explicit.
- [x] Request series/attempt, cancellation, retry and compaction architecture is explicit.
- [x] Frozen Spec 001 ExecutionProfile fields are fully preserved.
- [x] Benchmark backlinks are separated from execution-semantic profile identity, avoiding circular profile/benchmark identity.
- [x] Model request/attempt state binds authenticated initiating-principal provenance without treating it as capability authority.
- [x] HardwareProfile/calibration boundaries are explicit.
- [x] Model backend contract defines in-process and sidecar safety boundaries.
- [x] llama.cpp default path keeps native C/C++ outside `golamd`.
- [x] mistral.rs admission requires minimal exact feature/dependency closure and offline proof.
- [x] Benchmark/calibration evidence separates model/backend metrics from harness metrics.

## Data model

- [x] Stable IDs exist for profiles, hardware, request series/attempts, compaction and tool candidates.
- [x] Durable vs transient state is classified.
- [x] Attempt terminal states are explicit.
- [x] Compaction artifacts bind exact source evidence and do not delete canonical history.
- [x] Benchmark evidence binds exact code/profile/hardware/workload/backend identities.
- [x] Reverse benchmark backlink metadata is non-semantic/rebuildable and does not mutate execution identity.
- [x] Initiating-principal attribution is explicit and non-authority-bearing.
- [x] Model/profile/benchmark state is explicitly non-authority state.

## Verification design

- [x] Deterministic scripted backend is mandatory for ordinary CI.
- [x] Ordinary CI does not require model downloads, cloud credentials or accelerators.
- [x] Cancellation/late-event/retry/context-overflow adversarial tests are enumerated.
- [x] Tool-call normalization/rejection tests are enumerated.
- [x] Strict-local routing/no-hidden-cloud-fallback tests are enumerated.
- [x] Compaction crash/history-preservation tests are enumerated.
- [x] Existing authority/effect/no-egress regression gates remain mandatory.
- [x] Exact-head Windows/macOS/Ubuntu CI and post-merge main CI are mandatory.

## Cross-artifact consistency

- [x] Cross-artifact analysis completed.
- [x] Stale current-phase `AGENTS.md` finding repaired on the planning branch.
- [x] Profile/benchmark circular-identity finding repaired.
- [x] Explicit requester-attribution finding repaired.
- [x] Unresolved BLOCKER findings: 0.
- [x] Unresolved MAJOR findings: 0.
- [x] Noncanonical PRs #6–#8 are not treated as canonical predecessors.

## Planning closeout gates

- [x] T004-001 planning artifacts complete.
- [x] T004-002 branch `AGENTS.md` updated for Spec 004 planning.
- [x] T004-003 Draft planning PR #11 opened.
- [ ] T004-004 exact-head Windows/macOS/Ubuntu CI success on the final post-repair head.
- [ ] T004-005 substantive independent post-CI semantic review.
- [ ] T004-006 all material review findings reconciled with fresh evidence after mutations.
- [ ] T004-007 planning PR Ready + expected-head guarded merge.
- [ ] T004-008 post-merge canonical-main CI success.

## Decision

`IMPLEMENTATION_READY_BY_DESIGN=YES`

`IMPLEMENTATION_AUTHORIZED_NOW=NO`

Reason: design/readiness is complete, but the planning branch still requires final exact-head GitHub CI/review/merge/post-merge lifecycle. Product implementation MUST start only from the exact canonical main produced by that successful planning closeout.
