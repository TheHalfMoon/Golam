# Analysis — Spec 005 Planning Convergence

**Candidate base**: `main@390ea842837a7d85dca165d9291d5eb54c3f11db`

## Scope consistency

The planning package is bounded to Spec 001 T050–T059:

- local filesystem/search/Git/browser tool surfaces;
- L0 context and conditional L1 structural context;
- governed Markdown + SQLite memory;
- single managed-memory writer and reconciliation;
- Agent Skills-compatible instruction lifecycle;
- MCP/ACP interoperability as untrusted boundaries;
- memory/path/protocol/strict-local adversarial qualification.

Desktop/computer control, GolamConnect/channels, workers/scheduler, autonomous learning, broad parity and final release qualification remain later-spec scope.

## Canonical predecessor consistency

The package relies on canonical Specs 001–004 only. PRs #6–#8 remain noncanonical proposals and are not implementation predecessors.

Canonical Spec 003 explicitly leaves production native execution unadmitted. The Spec 005 plan therefore does not assume shell/process/local MCP execution exists and orders production containment qualification before those features.

## Authority consistency

No planning artifact grants authority to a model, tool, path, memory framework, skill or protocol.

Binding separations:

```text
TOOL_DESCRIPTOR != CAPABILITY
TOOL_CALL_CANDIDATE != EFFECT_AUTHORIZATION
PATH_STRING != TARGET_IDENTITY
CONTEXT_RANK != AUTHORITY
MEMORY_CANDIDATE != DURABLE_MEMORY
DERIVATIVE_INDEX != CANONICAL_MEMORY
SKILL != AUTHORITY
MCP_ADVERTISEMENT != GOLAM_CAPABILITY
ACP_CONNECTION != AUTHENTICATED_AUTHORITY
```

The inspected Golam-Research `skipApproval: true` shell semantic is explicitly rejected rather than normalized into Golam behavior.

## Filesystem security consistency

The spec, plan, data model and tool contract agree that:

- authorization binds explicit roots and resolved target identity;
- protected Golam resources stay excluded from generic filesystem authority;
- aliases/symlinks/reparse/junctions and special files are security-relevant;
- race-sensitive mutations preserve checked identity or fail closed;
- stale expectations deny rather than silently retarget;
- failures preserve user data.

Recent reference-source symlink/file-hardening observations support, but do not themselves authorize, these requirements.

## Context consistency

L0 is mandatory and designed to make the first slice useful without heavy retrieval infrastructure. L1 requires measured need and exact Source Foundry admission. L2 graph/dataflow/vector/runtime infrastructure is deferred.

Every context representation preserves provenance/authority/taint/permission/freshness metadata. Ranking/similarity cannot upgrade authority. Live authoritative state wins conflicts with stale memory.

## Memory consistency

All planning artifacts agree that:

- managed Markdown is canonical durable knowledge;
- SQLite is canonical operational state;
- derivatives are rebuildable and optional;
- one Golam writer owns Golam-generated managed memory mutation;
- user edits are detected/reconciled;
- promotion requires attributable approval or deterministic pre-registered verification;
- `SECRET_DERIVED` is excluded;
- contradiction/supersession lineage is explicit;
- FORGET/REDACT removes active canonical content and invalidates derivatives without falsely claiming external erasure.

No external memory framework is made canonical or a startup dependency.

## Protocol consistency

Agent Skills, MCP and ACP are compatibility/interoperability surfaces only. Executable skills/local MCP child processes share the production containment gate. Remote MCP shares network/egress/strict-local gates. ACP shares authenticated local-client semantics.

The official MCP Rust SDK remains a candidate pending exact minimal dependency qualification.

## Verification consistency

Planning and implementation both preserve:

- hermetic ordinary CI;
- exact-head evidence;
- substantive independent semantic review after CI;
- forward-only repair and requalification after head mutation;
- expected-head guarded merge;
- push-triggered post-merge canonical-main verification;
- no waiver.

## Material-risk review

### Risk 1 — production process containment could become a cross-platform scope sink

Mitigation: containment is admitted per exact platform/profile. Unsupported platforms remain explicit denial states; no cross-platform equivalence is inferred.

### Risk 2 — generic file tools could undermine kernel protections

Mitigation: protected-resource exclusion is independent of lexical path authority and is enforced at the protected action boundary.

### Risk 3 — memory could become self-reinforcing model truth

Mitigation: candidate/promotion separation, explicit authority class, live-state precedence, attributable promotion, contradiction preservation and derivative non-authority.

### Risk 4 — MCP/skills could smuggle shell/network authority

Mitigation: protocol/package metadata cannot mint capabilities; executable paths require production containment plus current Kernel/Effect Gate authority; remote transports require egress authority.

### Risk 5 — optional retrieval infrastructure could become a hidden dependency

Mitigation: L0 is complete enough for the initial slice; missing/corrupt derivatives do not block canonical memory; L1/vector admission is evidence-dependent.

## Planning convergence result

No material internal contradiction was found across the planning candidate after the above reconciliation.

This self-analysis is **not** independent semantic review and does not satisfy T005-015.

```text
T005_PLANNING_INTERNAL_CONVERGENCE=MATERIAL_FINDINGS_NONE
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
PLANNING_CODE_REUSED=NO
PLANNING_DEPENDENCY_ADDED=NO
NONCANONICAL_PR_6_7_8_PROMOTED_TO_AUTHORITY=NO
PLANNING_EXACT_HEAD_CI=PENDING_T005_014
INDEPENDENT_REVIEW=PENDING_T005_015
PR_READY=NO
MERGE_AUTHORIZED=NO
WAIVER_TAKEN=NO
```
