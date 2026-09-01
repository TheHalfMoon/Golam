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

Canonical Spec 003 explicitly leaves production native execution unadmitted. The Spec 005 plan therefore does not assume shell/process/local MCP execution exists and orders production containment qualification before those features or any external utility process.

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

The planning model makes bounded execution semantics explicit rather than relying on undefined aggregate fields:

- `ToolIoBounds` carries finite byte/count/nesting/field limits;
- `ToolDurationBounds` carries finite total/idle duration limits where applicable;
- reconciliation and verification policies have explicit schemas;
- requested operation, authorized resource class, target-resolution plan and current preconditions are first-class immutable protected-request bindings;
- once a protected request is durably prepared, material target/operation/precondition/authority changes require a new request/effect identity.

This closes the prior data-model under-binding that could have allowed an implementation to claim conformance while omitting duration or stale-state constraints.

## Process and L0-search sequencing consistency

Production native execution remains `native:unqualified`. That denial applies to **every** child process, including utility binaries that might otherwise look like implementation details.

The planning package now makes the boundary explicit:

- Phase D L0 search is in-process only;
- the eligible Phase D choices are a Golam-owned bounded implementation or an exact Source-Foundry-admitted Rust crate surface that executes in-process;
- no external ripgrep/search executable may launch while production remains `native:unqualified`;
- an external search binary may be reconsidered only after an exact production containment profile reaches `ADMITTED` at T005-077 and then requires its own exact Source Foundry record plus the normal governed process/tool boundary;
- `ProcessLaunchPlan` separately binds descendant supervision and process-tree terminal reconciliation; cancellation is not proof that descendants terminated.

```text
NATIVE_UNQUALIFIED != EXTERNAL_SEARCH_BINARY_AUTHORITY
PROCESS_CANCEL_REQUEST != PROCESS_TREE_TERMINAL_PROOF
```

This closes the earlier sequencing ambiguity where dependency admission for a search binary could have been mistaken for process-launch authority before containment qualification.

## Context consistency

L0 is mandatory and designed to make the first slice useful without heavy retrieval infrastructure. Initial L0 text search is explicitly in-process. L1 requires measured need and exact Source Foundry admission. L2 graph/dataflow/vector/runtime infrastructure is deferred.

Every context representation preserves provenance/authority/taint/permission/freshness metadata. Ranking/similarity cannot upgrade authority. Live authoritative state wins conflicts with stale memory.

## Memory consistency

All planning artifacts now agree that:

- managed Markdown is canonical durable knowledge;
- SQLite is canonical operational state;
- derivatives are rebuildable and optional;
- one Golam writer owns Golam-generated managed memory mutation;
- promotion-authority validation is implemented and qualified before that writer may be enabled for mutation;
- every managed mutation is a protected Effect Gate transaction with immutable `MemoryMutationIntent`;
- current Kernel authorization plus applicable approval/pre-registered verifier evidence is bound before mutation;
- authorized PREPARED evidence is durable before the first canonical Markdown/SQLite mutation;
- terminal outcome/read-back verification is integrity-chained;
- ambiguous completion remains `UNKNOWN_OUTCOME` and blocks dependent managed-memory mutation until reconciliation;
- every committed `MemoryVersion` separately preserves initiating/creating principal, governed committing-writer identity and exact mutation Effect attribution;
- restart/reconciliation preserves those identities rather than collapsing attribution into a generic system creator;
- FORGET/REDACT use the same lifecycle across Markdown, SQLite and derivative invalidation;
- user edits are detected/reconciled;
- the commit boundary immediately revalidates expected Markdown digest/version and target identity and uses conditional compare-and-replace / identity-preserving replacement, so an intervening edit cannot become silent last-writer-wins overwrite;
- promotion requires attributable approval or deterministic pre-registered verification;
- `SECRET_DERIVED` is excluded and monotonic within Spec 005; sanitization is not declassification;
- contradiction/supersession lineage is explicit;
- missing derivatives do not block canonical memory access; only derivative-dependent operations rebuild or fail closed;
- FORGET/REDACT removes active canonical content and invalidates derivatives without falsely claiming external erasure.

No external memory framework is made canonical or a startup dependency.

```text
MEMORY_WRITER_ENABLEMENT_REQUIRES_PROMOTION_AUTHORITY_VALIDATOR
MEMORY_CREATOR_IDENTITY != GOVERNED_WRITER_IDENTITY
MEMORY_VERSION_REQUIRES_MUTATION_EFFECT_ATTRIBUTION
```

## Network credential consistency

General egress authorization is explicitly separated from credential disclosure authority.

Credential-bearing hops require authenticated encrypted endpoint identity and exact credential scope. Redirect/origin/protocol/proxy changes strip sensitive material, revalidate endpoint identity and egress, and re-broker only under fresh explicit authorization. Credential-bearing transport downgrade or unprovable endpoint identity/scope fails closed.

```text
EGRESS_ALLOWED != CREDENTIAL_DISCLOSURE_AUTHORIZED
```

## Protocol consistency

Agent Skills, MCP and ACP are compatibility/interoperability surfaces only. Executable skills/local MCP child processes share the production containment gate. Remote MCP shares network/egress/authenticated-endpoint/credential-scope/strict-local gates. ACP shares authenticated local-client semantics.

`McpServerBinding` binds the reviewed Golam-local maximum mapping and explicit lifecycle state. Version replacement, revocation or an unreviewed binding cannot silently inherit prior authority. T005-030, T005-089, T005-092 and T005-094 explicitly qualify narrowing/replacement/revocation semantics.

The official MCP Rust SDK remains a candidate pending exact minimal dependency qualification.

## Review finding reconciliation

Historical substantive reviews are defect-discovery evidence, not final T005-015 evidence. Final qualification must bind the final unchanged planning head after fresh exact-head CI.

The known finding set is reconciled as follows:

1. **Credential-bearing network redirects** — authenticated encrypted endpoint identity, credential-origin/operation scope, strip/revalidate/re-broker semantics and downgrade denial are required across redirects/origin/protocol/proxy transitions.
2. **ToolDescriptor/ToolRequest under-binding** — explicit finite I/O/duration schemas, reconciliation/verification schemas, requested operation/resource/precondition bindings and post-PREPARED immutability are present.
3. **Derivative availability contradiction** — missing/corrupt derivatives never block canonical Markdown/SQLite startup or ordinary reads; only derivative-dependent operations rebuild or fail that operation closed.
4. **Managed-memory mutation lifecycle** — every Golam-generated managed mutation is PREPARED durably through the Effect Gate before canonical mutation, executes only through the governed writer, records integrity-chained terminal/verification evidence, and keeps ambiguous completion `UNKNOWN_OUTCOME` until reconciliation.
5. **Process descendant supervision under-binding** — `ProcessLaunchPlan` now separately binds descendant supervision and terminal process-tree reconciliation; cancellation is not terminal proof.
6. **MCP local-mapping/lifecycle under-binding** — the data model and task ledger explicitly bind reviewed local mapping identity, lifecycle state, replacement and revocation qualification.
7. **Memory version creation attribution under-binding** — `MemoryVersion` now preserves initiating/creating principal, committing writer identity and mutation Effect reference through restart/reconciliation.
8. **Promotion validator ordered after writer enablement** — T005-051 is now an explicit prerequisite that MUST execute before T005-048 despite numeric ordering; the writer has no direct-write pre-validator phase.
9. **Phase D external search binary before containment admission** — Phase D is now in-process only. External search binaries are ineligible until after T005-077 and then require exact Source Foundry + admitted process-boundary qualification.
10. **Ambient environment ambiguity** — process launch continues to require a cleared ambient environment with only explicitly bound values/secret handles.
11. **Secret-taint downgrade ambiguity** — `SECRET_DERIVED` provenance cannot be downgraded by redaction/summarization/transformation/verification; only independently sourced non-secret evidence may begin a separate candidate provenance chain.
12. **Managed-memory plan omitted the commit-time TOCTOU guard** — the plan lifecycle now explicitly revalidates expected Markdown digest/version plus target identity immediately at the commit boundary and requires conditional compare-and-replace / identity-preserving replacement with fail-closed `USER_EDIT_DETECTED`/`CONFLICT` behavior.
13. **Verification narrative could be read as a self-updating qualification ledger** — final exact-head CI/review dispositions are recorded in immutable GitHub PR/check/review evidence rather than committed back into this file after qualification, because such a post-qualification content mutation would itself invalidate the qualified head.
14. **Spec 004 closeout checklist challenge** — rejected as not valid against live authority. PR #14 records T004-113 post-merge canonical-main CI PASS on CI #777 / run `33507343928`, `SPEC_004_IMPLEMENTATION_COMPLETE=YES`, and `SPEC_004_CLOSED_CANONICAL=YES` at exact `main@390ea842837a7d85dca165d9291d5eb54c3f11db`. That live merge/post-merge evidence explicitly records T004-113/T004-114 without a self-invalidating content mutation; live GitHub truth outranks stale unchecked historical ledger boxes.

## Verification consistency

Planning and implementation both preserve:

- hermetic ordinary CI;
- exact-head evidence;
- substantive independent semantic review after CI;
- forward-only repair and requalification after head mutation;
- expected-head guarded merge;
- push-triggered post-merge canonical-main verification;
- no waiver.

Qualification evidence binds the head that the pull request itself advertises. A raw branch ref that has moved but is not yet represented as the PR head is not sufficient exact-head PR evidence and MUST NOT be used to satisfy CI/review/Ready/merge gates.

Historical qualification evidence is kept attributable rather than rewritten as current evidence. CI #782 / run `33516670304` succeeded on `feff88dfbbd0c54912118c5adc1cc8f6ceac028a` and became stale after later repair. CI #785 / run `33523752838` then succeeded on `b73b10c0c0416b55df3e2999229b799fc098a728` and enabled an independent Cubic 14-file review that found three reported issues: one Spec 004-closeout false positive contradicted by live canonical evidence, one material managed-memory plan omission, and one verification-narrative consistency issue. The present forward-only repair supersedes `b73b10c0...`, so CI #785 and that Cubic review are historical/stale for final qualification.

The successor exact planning head MUST obtain fresh Windows/macOS/Ubuntu exact-head CI and, only after that CI succeeds, a fresh substantive independent semantic review. Terminal PASS/FAIL dispositions for those gates MUST be recorded in GitHub PR/check/review evidence and MUST NOT be committed back into this file after qualification merely to mirror the result: that would mutate the qualified head and immediately make the evidence stale. This section therefore records the verification procedure and historical lineage, not a self-referential current PASS checkbox.

## Material-risk review

### Risk 1 — production process containment could become a cross-platform scope sink

Mitigation: containment is admitted per exact platform/profile. Unsupported platforms remain explicit denial states; no cross-platform equivalence is inferred. Utility binaries do not receive an exception.

### Risk 2 — generic file tools could undermine kernel protections

Mitigation: protected-resource exclusion is independent of lexical path authority and is enforced at the protected action boundary.

### Risk 3 — memory could become self-reinforcing model truth

Mitigation: candidate/promotion separation, explicit authority class, live-state precedence, attributable promotion, contradiction preservation, derivative non-authority and monotonic secret-derived taint.

### Risk 4 — memory durability or attribution could bypass Effect Gate truth

Mitigation: promotion-authority validation precedes writer enablement; every Golam-generated managed-memory mutation uses durable PREPARED Effect Gate evidence before canonical Markdown/SQLite mutation, one governed writer, creator/writer/effect attribution, integrity-chained terminal/read-back evidence and fail-closed `UNKNOWN_OUTCOME` reconciliation.

### Risk 5 — MCP/skills could smuggle shell/network authority

Mitigation: protocol/package metadata cannot mint capabilities; executable paths require production containment plus current Kernel/Effect Gate authority; remote transports require egress plus endpoint/credential authority; MCP local mappings/lifecycle state are reviewed and revocation fails closed.

### Risk 6 — optional retrieval infrastructure could become a hidden dependency or hidden child process

Mitigation: L0 is complete enough for the initial slice; Phase D search is in-process; no external utility launches while `native:unqualified`; missing/corrupt derivatives do not block canonical memory; derivative-dependent operations rebuild or fail locally; L1/vector admission is evidence-dependent.

### Risk 7 — redirects could leak brokered credentials despite valid egress

Mitigation: credential disclosure has a separate authenticated endpoint/scope gate and sensitive material is stripped across changed hops until fresh authorization re-brokers it.

### Risk 8 — cancellation could be mistaken for process-tree termination

Mitigation: descendant supervision and terminal process-tree reconciliation are separate mandatory bindings/evidence; unresolved descendants are a failure/reconciliation state, not success.

## Planning convergence result

The current convergence repair candidate reconciles every presently known material internal/review finding across the normative planning package. This is a self-analysis statement, not final qualification: the branch mutation intentionally invalidates prior exact-head CI/review evidence.

This self-analysis is **not** independent semantic review and does not satisfy T005-015.

```text
T005_PLANNING_INTERNAL_CONVERGENCE=MATERIAL_FINDINGS_REPAIRED_REQUALIFICATION_REQUIRED
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
PHASE_D_EXTERNAL_SEARCH_BINARY=DENIED_NATIVE_UNQUALIFIED
MEMORY_WRITER_PRE_VALIDATOR_DIRECT_WRITE=DENIED
PLANNING_CODE_REUSED=NO
PLANNING_DEPENDENCY_ADDED=NO
NONCANONICAL_PR_6_7_8_PROMOTED_TO_AUTHORITY=NO
PRE_REPAIR_CI_782=SUCCESS_STALE_AFTER_REPAIR
PRE_REPAIR_CI_785=SUCCESS_STALE_AFTER_REPAIR
PRE_REPAIR_CUBIC_REVIEW=THREE_REPORTED_ONE_FALSE_POSITIVE_TWO_REPAIRED_STALE_AFTER_REPAIR
FINAL_EXACT_HEAD_CI=REQUIRED_EXTERNAL_GITHUB_EVIDENCE
FINAL_INDEPENDENT_REVIEW=REQUIRED_EXTERNAL_GITHUB_EVIDENCE_AFTER_CI
PR_READY=NO
MERGE_AUTHORIZED=NO
WAIVER_TAKEN=NO
```
