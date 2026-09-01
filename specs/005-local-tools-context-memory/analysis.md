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

## Tool-request and bounds consistency

The planning model now makes bounded execution semantics explicit rather than relying on undefined aggregate fields:

- `ToolIoBounds` carries finite byte/count/nesting/field limits;
- `ToolDurationBounds` carries finite total/idle duration limits where applicable;
- reconciliation and verification policies have explicit schemas;
- requested operation, authorized resource class, target-resolution plan and current preconditions are first-class immutable protected-request bindings;
- once a protected request is durably prepared, material target/operation/precondition/authority changes require a new request/effect identity.

This closes the prior data-model under-binding that could have allowed an implementation to claim conformance while omitting duration or stale-state constraints.

## Context consistency

L0 is mandatory and designed to make the first slice useful without heavy retrieval infrastructure. L1 requires measured need and exact Source Foundry admission. L2 graph/dataflow/vector/runtime infrastructure is deferred.

Every context representation preserves provenance/authority/taint/permission/freshness metadata. Ranking/similarity cannot upgrade authority. Live authoritative state wins conflicts with stale memory.

## Memory consistency

All planning artifacts now agree that:

- managed Markdown is canonical durable knowledge;
- SQLite is canonical operational state;
- derivatives are rebuildable and optional;
- one Golam writer owns Golam-generated managed memory mutation;
- every managed mutation is a protected Effect Gate transaction with immutable `MemoryMutationIntent`;
- current Kernel authorization plus applicable approval/pre-registered verifier evidence is bound before mutation;
- authorized PREPARED evidence is durable before the first canonical Markdown/SQLite mutation;
- terminal outcome/read-back verification is integrity-chained;
- ambiguous completion remains `UNKNOWN_OUTCOME` and blocks dependent managed-memory mutation until reconciliation;
- FORGET/REDACT use the same lifecycle across Markdown, SQLite and derivative invalidation;
- user edits are detected/reconciled;
- promotion requires attributable approval or deterministic pre-registered verification;
- `SECRET_DERIVED` is excluded and monotonic within Spec 005; sanitization is not declassification;
- contradiction/supersession lineage is explicit;
- missing derivatives do not block canonical memory access; only derivative-dependent operations rebuild or fail closed;
- FORGET/REDACT removes active canonical content and invalidates derivatives without falsely claiming external erasure.

No external memory framework is made canonical or a startup dependency.

## Network credential consistency

General egress authorization is explicitly separated from credential disclosure authority.

Credential-bearing hops require authenticated encrypted endpoint identity and exact credential scope. Redirect/origin/protocol/proxy changes strip sensitive material, revalidate endpoint identity and egress, and re-broker only under fresh explicit authorization. Credential-bearing transport downgrade or unprovable endpoint identity/scope fails closed.

```text
EGRESS_ALLOWED != CREDENTIAL_DISCLOSURE_AUTHORIZED
```

## Protocol consistency

Agent Skills, MCP and ACP are compatibility/interoperability surfaces only. Executable skills/local MCP child processes share the production containment gate. Remote MCP shares network/egress/authenticated-endpoint/credential-scope/strict-local gates. ACP shares authenticated local-client semantics.

The official MCP Rust SDK remains a candidate pending exact minimal dependency qualification.

## Review finding reconciliation

A substantive CodeRabbit review on historical planning head `779a5c8a49f1004c43182e123afe503037e34659` produced four actionable findings. That review is not final T005-015 evidence because later forward-only repairs changed the branch and the final review must occur after fresh exact-head CI. The findings remain valid defect evidence and were reconciled as follows:

1. **Credential-bearing network redirects** — repaired by requiring authenticated encrypted endpoint identity, credential-origin/operation scope, strip/revalidate/re-broker semantics and downgrade denial across redirects/origin/protocol/proxy transitions.
2. **ToolDescriptor/ToolRequest under-binding** — repaired with explicit finite I/O/duration schemas, reconciliation/verification schemas, requested operation/resource/precondition bindings and post-PREPARED immutability.
3. **Derivative availability contradiction** — repaired so missing/corrupt derivatives never block canonical Markdown/SQLite startup or ordinary reads; only derivative-dependent operations rebuild or fail that operation closed.
4. **Managed-memory mutation lifecycle** — repaired across spec, plan, contract, data model, readiness, clarification, quickstart and task ordering: every Golam-generated managed mutation is PREPARED durably through the Effect Gate before canonical mutation, executes only through the governed writer, records integrity-chained terminal/verification evidence, and keeps ambiguous completion `UNKNOWN_OUTCOME` until reconciliation.

Earlier convergence also repaired two additional planning ambiguities before final qualification:

- the process launch model now requires a cleared ambient environment exactly as the execution contract requires;
- `SECRET_DERIVED` provenance cannot be downgraded by redaction/summarization/transformation/verification; only independently sourced non-secret evidence may begin a separate candidate provenance chain.

## Verification consistency

Planning and implementation both preserve:

- hermetic ordinary CI;
- exact-head evidence;
- substantive independent semantic review after CI;
- forward-only repair and requalification after head mutation;
- expected-head guarded merge;
- push-triggered post-merge canonical-main verification;
- no waiver.

CI #781 / run `33513454450` succeeded on pre-reconciliation head `237562b7ad3368c548e07d535e5c9306a8afe8fe` across Windows/macOS/Ubuntu, but this repair commit mutates the branch, so #781 becomes historical/stale for the final T005-014 gate. Fresh exact-head CI and then fresh independent review are mandatory.

## Material-risk review

### Risk 1 — production process containment could become a cross-platform scope sink

Mitigation: containment is admitted per exact platform/profile. Unsupported platforms remain explicit denial states; no cross-platform equivalence is inferred.

### Risk 2 — generic file tools could undermine kernel protections

Mitigation: protected-resource exclusion is independent of lexical path authority and is enforced at the protected action boundary.

### Risk 3 — memory could become self-reinforcing model truth

Mitigation: candidate/promotion separation, explicit authority class, live-state precedence, attributable promotion, contradiction preservation, derivative non-authority and monotonic secret-derived taint.

### Risk 4 — memory durability could bypass Effect Gate truth

Mitigation: every Golam-generated managed-memory mutation uses durable PREPARED Effect Gate evidence before canonical Markdown/SQLite mutation, one governed writer, integrity-chained terminal/read-back evidence and fail-closed `UNKNOWN_OUTCOME` reconciliation.

### Risk 5 — MCP/skills could smuggle shell/network authority

Mitigation: protocol/package metadata cannot mint capabilities; executable paths require production containment plus current Kernel/Effect Gate authority; remote transports require egress plus endpoint/credential authority.

### Risk 6 — optional retrieval infrastructure could become a hidden dependency

Mitigation: L0 is complete enough for the initial slice; missing/corrupt derivatives do not block canonical memory; derivative-dependent operations rebuild or fail locally; L1/vector admission is evidence-dependent.

### Risk 7 — redirects could leak brokered credentials despite valid egress

Mitigation: credential disclosure has a separate authenticated endpoint/scope gate and sensitive material is stripped across changed hops until fresh authorization re-brokers it.

## Planning convergence result

The current repair candidate reconciles every presently known material internal/review finding across the normative planning package. This is a convergence statement, not final qualification: the branch mutation intentionally invalidates prior exact-head CI/review evidence.

This self-analysis is **not** independent semantic review and does not satisfy T005-015.

```text
T005_PLANNING_INTERNAL_CONVERGENCE=MATERIAL_FINDINGS_REPAIRED_REQUALIFICATION_REQUIRED
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
PLANNING_CODE_REUSED=NO
PLANNING_DEPENDENCY_ADDED=NO
NONCANONICAL_PR_6_7_8_PROMOTED_TO_AUTHORITY=NO
PRE_REPAIR_CI_781=SUCCESS_STALE_AFTER_REPAIR
INDEPENDENT_REVIEW=PENDING_FRESH_POST_CI
PR_READY=NO
MERGE_AUTHORIZED=NO
WAIVER_TAKEN=NO
```
