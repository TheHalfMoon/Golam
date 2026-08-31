# Cross-Artifact Analysis — Spec 004 Harness & Local Intelligence

**Date**: 2026-08-31  
**Analyzed branch**: `spec/004-harness-local-intelligence`  
**Canonical predecessor**: `main@6719e9997862cbe617b60e33870ef056fa3c0c70`

## Scope

Analyzed:
- constitution v1.2.0;
- frozen Spec 001 architecture/tasks/ExecutionProfile contract;
- canonical Specs 002–003 authority/durability/closeout constraints;
- Spec 004 spec, clarification, research, donor qualification, plan, data model, all contracts, quickstart and tasks;
- current branch `AGENTS.md`;
- live noncanonical planning PRs #6–#8 only to ensure they are not silently treated as canonical authority.

## Finding summary

| ID | Severity | Finding | Resolution |
|---|---|---|---|
| A004-001 | MAJOR | Branch `AGENTS.md` still described Spec 003 implementation as active after Spec 003 had closed canonical. | FIXED by advancing instructions to Spec 004 planning and binding exact predecessor `main@6719e999...`. |
| A004-011 | MAJOR | Treating Spec 001 `benchmark record references` as part of the execution-semantic profile digest would create a circular identity: benchmark binds profile while profile identity changes when benchmark backlink is added. | FIXED: benchmark refs remain required evidence backlinks/projection metadata but are excluded from execution-semantic `profile_id`/content digest. |
| A004-012 | MAJOR | Initial ModelRequest sketch relied on session/turn indirection for requester attribution and did not make initiating authenticated principal evidence explicit in the request/attempt model. | FIXED: request/attempt contracts now bind `initiator_principal_ref` as audit/provenance evidence while explicitly keeping it non-capability and non-model-visible by default. |
| A004-002 | NONE | Harness/backend authority separation | CONSISTENT |
| A004-003 | NONE | Canonical history/compaction semantics | CONSISTENT |
| A004-004 | NONE | Cancellation/retry vs Effect Gate semantics | CONSISTENT |
| A004-005 | NONE | ExecutionProfile frozen-field preservation | CONSISTENT AFTER A004-011 |
| A004-006 | NONE | Strict-local/backend fallback semantics | CONSISTENT |
| A004-007 | NONE | Source/donor admission discipline | CONSISTENT |
| A004-008 | NONE | Seven-crate initial workspace discipline | CONSISTENT |
| A004-009 | NONE | Benchmark model-vs-harness separation | CONSISTENT |
| A004-010 | NONE | Later-spec scope containment | CONSISTENT |

`UNRESOLVED_BLOCKER=0`  
`UNRESOLVED_MAJOR=0`

## A004-001 — Current-phase governance drift

Problem found:
- canonical `AGENTS.md` was historically correct for Spec 003 but became operationally stale after the Spec 003 implementation merge/post-merge closeout.

Repair:
- branch `AGENTS.md` now states Spec 004 planning as the active phase;
- records Spec 003 `CLOSED_CANONICAL` at `main@6719e999...` and CI #666;
- adds the complete Spec 004 read order;
- prohibits product implementation/dependency admission on the planning branch;
- preserves exact-head review/CI discipline and the existing no-bypass constraints;
- states nonmerged planning proposals are not canonical predecessors.

No constitutional amendment was required because this is phase/state synchronization, not a governance change.

## A004-011 — Profile/benchmark circular identity

Problem found during pre-review self-review:
- the frozen Spec 001 `ExecutionProfile` includes benchmark record references;
- Spec 004 also requires every benchmark to bind an already stable exact profile identity;
- if reverse benchmark references were included in the execution-semantic profile digest, creating a benchmark would mutate the profile identity that the benchmark references.

Repair:
- preserve benchmark references as required append-only/rebuildable evidence backlinks;
- exclude backlink membership from `profile_id`/execution-semantic content digest;
- keep every field that changes execution semantics inside identity;
- benchmark records continue to bind exact profile/hardware/backend/harness/workload identity.

This preserves the frozen field without creating a self-referential identity loop.

`PROFILE_IDENTITY != EVIDENCE_BACKLINK_SET`

## A004-012 — Explicit initiating-principal attribution

Problem found during pre-review self-review:
- session/turn references could indirectly recover who initiated a model request, but the ModelRequest/RequestAttempt contract did not make that provenance binding explicit.

Repair:
- add `initiator_principal_ref` to request/attempt state;
- bind it to authenticated canonical principal/turn evidence;
- explicitly state it is provenance/audit evidence, not a capability token and not automatically rendered into backend-visible prompt content;
- retries preserve/rebind the same canonical initiator attribution as appropriate.

`PRINCIPAL_ATTRIBUTION != CAPABILITY_TOKEN`

## Authority separation consistency

The following all agree:
- constitution: model is replaceable and harness is product; authority stays outside model reasoning;
- Spec 001: `ExecutionProfile` + replaceable backend; `mistral.rs` primary candidate and `llama.cpp` compatibility sidecar;
- Spec 004 spec/clarification/plan/contracts: backend emits inference data/candidates only;
- tasks: no real tool execution path is introduced before later owning scopes.

No artifact permits model/backend text to mint leases, satisfy approval, mutate protected state or directly commit an effect.

`MODEL_BACKEND != AUTHORITY_ROOT`

## Canonical history and compaction consistency

All artifacts preserve:
- Spec 001 `FULL_CANONICAL_HISTORY_SURVIVES_COMPACTION`;
- Spec 002 append-oriented canonical event/session truth;
- Spec 004 projection-only compaction with exact source refs/digest and explicit failed/incomplete attempt evidence;
- independently durable Goal/non-negotiable constraint state.

No artifact authorizes deleting/replacing canonical source history as compaction behavior.

## Retry/cancellation/effect consistency

Spec, plan, data model, harness contract, quickstart and tasks consistently require:
- explicit cancel/timeout terminal classes;
- preservation of accepted streamed prefix;
- retry as a new attempt;
- transient/deterministic/context-overflow distinction;
- no model retry rewriting prior evidence;
- existing Spec 002 UNKNOWN_OUTCOME/no-blind-retry behavior to dominate protected effects.

No contradiction found with Spec 002/003 effect/authority semantics.

## ExecutionProfile consistency

The Spec 004 data model/contract retains every material field frozen by Spec 001:
- model/revision;
- tokenizer/template;
- backend;
- locality;
- precision/quantization;
- hardware mapping;
- harness/reasoning/tool/schema modes;
- sampling/context/cache/warm residency;
- workload/multimodal;
- budgets;
- privacy/network;
- load/failure/fallback;
- benchmark references.

Spec 004 adds immutable/versioned/content-derived execution identity without removing frozen semantics. Benchmark refs remain evidence linkage but are non-semantic backlinks, resolving A004-011 without losing the field.

## Strict-local consistency

All artifacts agree:
- strict-local hard denial precedes routing preference;
- only compatible LOCAL profiles are eligible;
- local backend failure cannot select explicit cloud as an implicit fallback;
- model auto-download/update/telemetry/RPC paths require explicit qualification/authority and are disabled for strict-local candidate paths;
- sidecars remain governed Golam-managed descendants under Spec 003 sandbox/egress rules.

## Source governance consistency

Research/donor register explicitly admits zero production donors/dependencies during planning.

- Golam-Research: REFERENCE_ONLY
- grok-build: REFERENCE_ONLY
- Goose: REFERENCE_ONLY
- DeepSeek Harness: REFERENCE_ONLY
- mistral.rs: PRIMARY_CANDIDATE_NOT_YET_ADMITTED
- llama.cpp: COMPATIBILITY_CANDIDATE_NOT_YET_ADMITTED

Tasks place exact Source Foundry qualification before dependency/artifact addition.

## Workspace consistency

Canonical workspace contains seven packages. Spec 004 plan/tasks preserve them initially and prohibit empty architectural crates. Planned ownership is compatible with current crate roles without proving a new crate boundary prematurely.

## Scope containment

Spec 004 does not claim implementation ownership for:
- broad filesystem/shell/git/browser tools;
- MCP/ACP product integration;
- long-term memory/context retrieval product;
- Desktop/computer control;
- Connect/mobile/channels;
- workers/scheduler/automations;
- parity or release qualification.

These remain Specs 005–010.

Noncanonical PRs #6–#8 do not alter this result because they have not merged into canonical main.

## Review-policy analysis

Spec 003's founder direction removed Qodo from the Spec 003 closeout path and allowed available substantive independent repository-integrated reviewers while excluding Codex. No canonical artifact examined establishes a new Spec-004-specific Qodo-only rule.

Therefore Spec 004 planning does not hard-code a reviewer vendor. Closeout must use the exact live repository policy at the time, after exact-head CI, and only a substantive semantic result bound to that head counts. Status-only/rate-limit/billing/unavailable/summary-only/self-review responses do not count.

## Final convergence

After A004-001, A004-011 and A004-012 repairs:

- constitution conflicts: 0;
- frozen Spec 001 contract removals: 0;
- canonical Spec 002/003 authority/durability conflicts: 0;
- unresolved planning blocker findings: 0;
- unresolved planning major findings: 0;
- production dependency/donor admissions: 0;
- later-spec scope leaks: 0.

`SPEC_004_PLANNING_CROSS_ARTIFACT_CONVERGENCE=MATERIAL_FINDINGS_NONE_AFTER_REPAIRS`

Remaining gates are operational rather than design findings: exact-head CI, substantive independent external review, guarded merge and post-merge canonical-main CI.
