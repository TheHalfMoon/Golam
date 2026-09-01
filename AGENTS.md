# Golam Agent Instructions

## Current phase

Golam is in **Spec 004 implementation closeout: Harness & Local Intelligence** on branch `impl/004-harness-local-intelligence`.

Spec 002 — Kernel & Durable Session Spine — is `CLOSED_CANONICAL`.

Spec 003 — Identity, Policy, Secrets & Sandbox — is `CLOSED_CANONICAL` at its qualified predecessor lineage.

Spec 004 planning is `CLOSED_CANONICAL` at `main@8b08ae9f787cb85f1257641d6d332810d7de9fa4` after guarded planning merge PR #12 and push-triggered post-merge CI #680 / run `33425624764` completed successfully on Windows, macOS and Ubuntu.

Spec 004 implementation Phases B–J have completed their selected bounded implementation posture. Phase K convergence, final exact-head qualification, independent semantic review, guarded merge and post-merge canonical-main verification remain required before canonical closure.

No later-spec scope and no production inference dependency is admitted merely by implementation progress.

## Authority order

1. exact live GitHub/repository truth;
2. `.specify/memory/constitution.md` v1.2.0 or later;
3. frozen Spec 001 program architecture, tasks, contracts and source-permission attestation;
4. canonical Spec 002 package and closeout evidence;
5. canonical Spec 003 package, implementation evidence and closeout truth;
6. canonical Spec 004 planning package and planning closeout evidence;
7. exact implementation evidence on `impl/004-harness-local-intelligence`;
8. exact Source Foundry records for any source that later reaches `ADMITTED`.

Open proposals or nonmerged PRs are not canonical predecessors merely because they exist.

## Spec 004 implementation read order

1. `.specify/memory/constitution.md`
2. `specs/001-golam-local-agent-os-foundation/spec.md`
3. `specs/001-golam-local-agent-os-foundation/plan.md`
4. `specs/001-golam-local-agent-os-foundation/tasks.md`
5. `specs/001-golam-local-agent-os-foundation/contracts/execution-profile-contract.md`
6. `specs/001-golam-local-agent-os-foundation/source-permission-attestation.md`
7. canonical Spec 002 closeout package/evidence
8. canonical Spec 003 package and `implementation/` closeout evidence
9. `specs/004-harness-local-intelligence/spec.md`
10. `specs/004-harness-local-intelligence/clarification-closeout.md`
11. `specs/004-harness-local-intelligence/research.md`
12. `specs/004-harness-local-intelligence/donor-qualification.md`
13. `specs/004-harness-local-intelligence/plan.md`
14. `specs/004-harness-local-intelligence/data-model.md`
15. all `specs/004-harness-local-intelligence/contracts/`
16. `specs/004-harness-local-intelligence/quickstart.md`
17. `specs/004-harness-local-intelligence/checklists/implementation-readiness.md`
18. `specs/004-harness-local-intelligence/tasks.md`
19. `specs/004-harness-local-intelligence/analysis.md`
20. `specs/004-harness-local-intelligence/implementation/BASELINE.md` and later exact implementation evidence

## Spec 004 hard boundaries

- Preserve the existing seven-package workspace initially. Do not create empty `golam-harness` or `golam-models` crates. Split only when implementation evidence proves an independent ownership/testing boundary.
- `golam-core` owns pure bounded harness/profile/backend protocol types, validation and deterministic state transitions without privileged state.
- `golam-ledger` owns canonical request/profile/compaction evidence persistence and projections using the existing append-oriented durability model.
- `golamd` owns unprivileged harness coordination, backend lifecycle/supervision and routing under kernel decisions.
- `golam-kernel` and `golam-effects` MUST NOT acquire model-specific semantics; the harness consumes their existing authority/effect boundaries.
- The model backend is replaceable and unprivileged. It never becomes KernelApi, Effect Gate, policy/lease/approval authority, secret authority, memory truth or Task verification truth.
- `MODEL_TOOL_CALL != AUTHORITY_OR_EFFECT_COMMIT`. Backend-native tool/agent/MCP/shell/code-execution features are not Golam authority paths.
- Canonical session/event evidence remains the source of truth. Model-visible history is a projection. `FULL_CANONICAL_HISTORY_SURVIVES_COMPACTION` remains binding.
- Compaction creates provenance-bound projections/artifacts and never rewrites or deletes canonical history. Goal/non-negotiable constraint evidence remains independently durable.
- Cancellation and timeout are explicit states. Accepted streamed prefixes remain attributable evidence. Retry creates a new request attempt and never rewrites prior evidence.
- Harness retry cannot blind-replay protected effects and cannot clear existing `UNKNOWN_OUTCOME` state.
- Preserve every frozen Spec 001 `ExecutionProfile` field. Material execution-semantic changes produce distinct immutable/versioned identity. Benchmark backlinks remain non-semantic evidence metadata.
- Strict-local hard denial remains above routing. Local backend failure never silently selects cloud or enables model download, telemetry, RPC or other network widening.
- `HardwareProfile` and calibration are bounded execution evidence, not device authority or a telemetry/fingerprinting surface.
- Ordinary CI must not depend on model downloads, cloud credentials or specialized accelerators. A deterministic scripted backend is the mandatory harness-semantics oracle.
- Exact `mistral.rs v0.9.0` qualification is `REJECTED` for Spec 004. No mistral.rs dependency/runtime artifact is admitted. Reopening requires a fresh exact Source Foundry qualification against changed evidence.
- Exact `llama.cpp v0.3.0` compatibility qualification is `DEFERRED` at the canonical Spec 003 native-executor containment gate. No llama.cpp binary/runtime artifact is admitted and no sidecar runtime/no-egress evidence is claimed.
- Golam-Research, grok-build, Goose, DeepSeek Harness and Munder Difflin remain `REFERENCE_ONLY` unless a bounded exact Source Foundry record later admits selected code.
- No product filesystem/shell/git/browser tools, canonical long-term memory product, Desktop/computer control, GolamConnect/channels, workers/scheduler, broad parity or final release qualification in Spec 004.
- Never claim CI/tests/review PASS without exact-head evidence. A branch mutation invalidates only CI/review evidence tied to that branch's prior exact head; unchanged canonical predecessor evidence remains valid unless superseded by live repository truth.
- Execute `tasks.md` in dependency order. Do not skip Source Foundry, exact-head CI, independent semantic review, expected-head merge or post-merge canonical-main CI gates.
- Do not start Spec 005 until Spec 004 implementation is genuinely `CLOSED_CANONICAL` after T004-113/T004-114.

## Review discipline

Use the exact live repository review policy at each closeout. Codex review remains excluded by founder direction. Qodo is not a required Spec 004 reviewer. Do not treat status-only, billing/rate-limit/unavailable responses, automated summaries, stale-head output, CI alone or self-review as a substantive independent semantic review. A qualifying final review must be bound to the exact unchanged implementation head after exact-head CI and must have no unresolved material findings before Ready/merge authorization.
