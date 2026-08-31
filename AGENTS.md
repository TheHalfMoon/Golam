# Golam Agent Instructions

## Current phase

Golam is in **Spec 004 planning: Harness & Local Intelligence** on branch `spec/004-harness-local-intelligence`.

Spec 002 — Kernel & Durable Session Spine — is `CLOSED_CANONICAL`.

Spec 003 — Identity, Policy, Secrets & Sandbox — is `CLOSED_CANONICAL` at `main@6719e9997862cbe617b60e33870ef056fa3c0c70` after the exact qualified implementation merge and post-merge CI #666 / run `33410723102` completed successfully on Windows, macOS and Ubuntu.

Spec 004 planning is authorized by frozen Spec 001 T040–T045 because Spec 003 has closed canonical. This planning branch MUST NOT contain Spec 004 product implementation or production dependency/donor admission.

## Authority order

1. exact live GitHub/repository truth;
2. `.specify/memory/constitution.md` v1.2.0 or later;
3. frozen Spec 001 program architecture, tasks and contracts;
4. canonical Spec 002 package and closeout evidence;
5. canonical Spec 003 package, implementation evidence and closeout truth;
6. the complete active Spec 004 planning package;
7. exact research/source records and any later admitted Source Foundry records.

Open planning proposals that have not merged into canonical `main` are not canonical predecessors merely because they exist or are stacked in GitHub.

## Spec 004 read order

1. `.specify/memory/constitution.md`
2. `specs/001-golam-local-agent-os-foundation/spec.md`
3. `specs/001-golam-local-agent-os-foundation/plan.md`
4. `specs/001-golam-local-agent-os-foundation/tasks.md`
5. `specs/001-golam-local-agent-os-foundation/contracts/execution-profile-contract.md`
6. canonical Spec 002 closeout package/evidence
7. canonical Spec 003 package and `implementation/` closeout evidence
8. `specs/004-harness-local-intelligence/spec.md`
9. `specs/004-harness-local-intelligence/clarification-closeout.md`
10. `specs/004-harness-local-intelligence/research.md`
11. `specs/004-harness-local-intelligence/donor-qualification.md`
12. `specs/004-harness-local-intelligence/plan.md`
13. `specs/004-harness-local-intelligence/data-model.md`
14. all `specs/004-harness-local-intelligence/contracts/`
15. `specs/004-harness-local-intelligence/quickstart.md`
16. `specs/004-harness-local-intelligence/checklists/implementation-readiness.md`
17. `specs/004-harness-local-intelligence/tasks.md`
18. `specs/004-harness-local-intelligence/analysis.md`

## Spec 004 hard boundaries

- Preserve the existing seven-package workspace initially. Do not create empty `golam-harness` or `golam-models` crates. Split only when implementation evidence proves an independent ownership/testing boundary.
- The model backend is replaceable and unprivileged. It never becomes KernelApi, Effect Gate, policy/lease/approval authority, secret authority, memory truth or Task verification truth.
- `MODEL_TOOL_CALL != AUTHORITY_OR_EFFECT_COMMIT`. Backend-native tool/agent/MCP/shell/code-execution features are not Golam authority paths.
- Canonical session/event evidence remains the source of truth. Model-visible history is a projection. `FULL_CANONICAL_HISTORY_SURVIVES_COMPACTION` remains binding.
- Compaction creates provenance-bound projections/artifacts and never rewrites/deletes canonical history. Goal/non-negotiable constraint evidence remains independently durable.
- Cancellation and timeout are explicit states. Accepted streamed prefixes remain attributable evidence. Retry creates a new request attempt and never rewrites prior evidence.
- Harness retry cannot blind-replay protected effects and cannot clear existing `UNKNOWN_OUTCOME` state.
- Preserve every frozen Spec 001 `ExecutionProfile` field. Material profile changes produce distinct immutable/versioned identity and invalidate stale benchmark binding.
- Strict-local hard denial remains above routing. Local backend failure never silently selects cloud or enables model download/telemetry/RPC/network behavior.
- `HardwareProfile` and calibration are bounded execution evidence, not device authority or a telemetry/fingerprinting surface.
- Ordinary CI must not depend on model downloads, cloud credentials or specialized accelerators. A deterministic scripted backend is the mandatory harness-semantics oracle.
- `mistral.rs` is only `PRIMARY_CANDIDATE_NOT_YET_ADMITTED` during planning. Before implementation admission, select and qualify the exact minimal crate/feature/transitive/native/network closure. Do not delegate Golam harness/tools/authority to mistral.rs built-in agentic features.
- `llama.cpp` is only `COMPATIBILITY_CANDIDATE_NOT_YET_ADMITTED` during planning. If later admitted, default to a supervised out-of-process sidecar with local model path, offline policy and authenticated/private local transport. No direct C/C++ FFI inside `golamd` and no generic unauthenticated localhost control surface.
- Golam-Research, grok-build, Goose and DeepSeek Harness remain `REFERENCE_ONLY` unless an exact bounded Source Foundry record later admits selected code.
- No product filesystem/shell/git/browser tools, canonical long-term memory product, Desktop/computer control, GolamConnect/channels, workers/scheduler, parity breadth or final release qualification in Spec 004.
- Never claim CI/tests/review PASS without exact-head evidence. Any planning or implementation branch mutation invalidates earlier exact-head closeout evidence.
- Do not mark the planning PR Ready, merge it, start Spec 004 implementation, or admit production dependencies until the complete planning task order and planning closeout gates are genuinely satisfied.
- After planning merges, implementation MUST start from the exact canonical `main` produced by the planning merge and its successful post-merge CI.
- Do not start Spec 005 until Spec 004 implementation is genuinely `CLOSED_CANONICAL` after exact-head implementation gates, guarded merge and successful post-merge canonical-main CI.

## Review discipline

Use the exact live repository review policy at the time of each closeout. Codex review remains excluded by founder direction. Do not assume a historical Qodo-only rule applies to Spec 004, and do not treat status-only, billing/rate-limit/unavailable responses, automated summaries or self-review as a substantive independent semantic review. A qualifying review must be bound to the exact head after exact-head CI and must have no unresolved material findings before Ready/merge authorization.
