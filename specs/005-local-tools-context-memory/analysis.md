# Analysis — Spec 005 Planning Convergence

**Candidate base**: `main@390ea842837a7d85dca165d9291d5eb54c3f11db`

## Scope consistency

The planning package remains bounded to Spec 001 T050–T059:

- local filesystem/search/Git/browser tool surfaces;
- L0 context and conditional L1 structural context;
- governed Markdown + dedicated memory-operational SQLite state;
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

```text
TOOL_DESCRIPTOR != CAPABILITY
TOOL_CALL_CANDIDATE != EFFECT_AUTHORIZATION
PATH_STRING != TARGET_IDENTITY
CONTEXT_RANK != AUTHORITY
MEMORY_CANDIDATE != DURABLE_MEMORY
DERIVATIVE_INDEX != CANONICAL_MEMORY
SKILL != AUTHORITY
STALE_SKILL_DISPATCH_BINDING != ACTIVE_AUTHORITY
MCP_ADVERTISEMENT != GOLAM_CAPABILITY
STALE_MCP_DISPATCH_BINDING != ACTIVE_AUTHORITY
ACP_CONNECTION != AUTHENTICATED_AUTHORITY
```

The inspected Golam-Research `skipApproval: true` shell semantic remains explicitly rejected.

## Filesystem and process consistency

The spec, plan, data model and tool contract agree that:

- authorization binds explicit roots and resolved target identity;
- protected Golam resources remain excluded from generic filesystem authority;
- aliases/symlinks/reparse/junctions and special files are security-relevant;
- race-sensitive mutations preserve checked identity or fail closed;
- stale expectations deny rather than silently retarget;
- failures preserve user data;
- production native execution remains `native:unqualified` until an exact profile is independently admitted;
- Phase D L0 search is in-process only while that denial state exists;
- `ProcessLaunchPlan` separately binds descendant supervision and process-tree terminal reconciliation; cancellation is not terminal proof.

```text
NATIVE_UNQUALIFIED != EXTERNAL_SEARCH_BINARY_AUTHORITY
PROCESS_CANCEL_REQUEST != PROCESS_TREE_TERMINAL_PROOF
```

## Tool-request and context consistency

`ToolIoBounds`, `ToolDurationBounds`, reconciliation policy and verification policy are explicit. Protected requests bind requested operation, resource class, target-resolution/identity state, current preconditions, authority context and exact tool identity/version. Once durably prepared, material target/operation/precondition/authority changes require a new request/effect identity.

L0 is mandatory and sufficient for the initial bounded slice. L1 requires measured need and exact Source Foundry admission. L2 graph/dataflow/vector/runtime infrastructure is deferred. Every context representation preserves provenance, authority, taint, permission and freshness metadata. Ranking/similarity cannot upgrade authority. Live authoritative state wins conflicts with stale memory.

## Memory consistency

All normative planning artifacts now agree that:

- managed Markdown is canonical durable knowledge;
- a dedicated memory-operational SQLite store is canonical operational memory state and is separate from the protected authority database;
- one governed writer owns Golam-generated managed memory mutation;
- promotion-authority validation is implemented and qualified before that writer may be enabled;
- every managed mutation is a protected Effect Gate transaction with immutable `MemoryMutationIntent`;
- the intent binds initiating principal, current Kernel authorization, applicable promotion authority, expected current versions, exact expected Markdown target identity, expected Markdown digest/version, exact dedicated memory-operational-store identity/schema and unique Effect identity;
- all protected bindings enter the immutable intent digest and survive PREPARED unchanged;
- durable authorized PREPARED evidence in the authority database precedes the first canonical Markdown or memory-SQLite mutation;
- the memory-operational store is never treated as an authority-database alias;
- Markdown body/front matter is content only; reserved authority-bearing fields enter explicit reconciliation/quarantine rather than changing scope, taint, provenance authority, approval, authorization, managed version identity, promotion state or Effect Gate state;
- immediately before Markdown replacement the writer revalidates exact target identity + digest/version and uses conditional compare-and-replace / identity-preserving replacement;
- changed content/identity or inability to preserve identity fails closed as `USER_EDIT_DETECTED`/`CONFLICT`; silent last-writer-wins is forbidden;
- operational rows bind the exact memory-store identity, Effect identity and mutation-intent digest;
- terminal success requires read-back/reconciliation across the authority journal, exact Markdown identity/digest/version and exact memory-operational-store rows;
- wrong-store, file-without-row, row-without-file, unreadable/partial cuts or disagreement cannot become success;
- ambiguous completion remains `UNKNOWN_OUTCOME` and blocks dependent managed-memory mutation until reconciliation;
- every committed `MemoryVersion` separately preserves initiating/creating principal, governed committing-writer identity and exact mutation Effect attribution;
- FORGET/REDACT use the same lifecycle and partial multi-store completion is never silently promoted to success;
- `SECRET_DERIVED` is excluded and monotonic within Spec 005;
- missing derivatives do not block canonical memory access; derivative-dependent operations rebuild or fail only that operation closed.

```text
MEMORY_OPERATIONAL_SQLITE != AUTHORITY_DATABASE
MEMORY_INTENT_TARGET_DIGEST_STORE_BINDING != OPTIONAL
MEMORY_WRITER_ENABLEMENT_REQUIRES_PROMOTION_AUTHORITY_VALIDATOR
MEMORY_CREATOR_IDENTITY != GOVERNED_WRITER_IDENTITY
MEMORY_VERSION_REQUIRES_MUTATION_EFFECT_ATTRIBUTION
```

## Network credential consistency

General egress authorization is separate from credential-disclosure authority. Credential-bearing hops require authenticated encrypted endpoint identity and exact credential scope. Redirect/origin/protocol/proxy changes strip sensitive material, revalidate endpoint identity and egress, and re-broker only under fresh explicit authorization. Credential-bearing downgrade or unprovable endpoint identity/scope fails closed.

```text
EGRESS_ALLOWED != CREDENTIAL_DISCLOSURE_AUTHORIZED
```

## Protocol consistency

Agent Skills, MCP and ACP remain compatibility/interoperability surfaces only.

A `SkillDispatchBinding` binds the exact reviewed skill package/version/content digest, reviewed lifecycle/admission state, reviewed Golam-local capability mapping and queued/prepared/cached decision references. Immediately before every instruction activation or executable dispatch, that exact binding is revalidated. Deprecation, revocation, replacement, unknown state or version/digest/mapping mismatch invalidates queued, prepared-but-not-dispatched, cached capability/approval and dispatch-decision state.

An `McpDispatchBinding` binds the exact reviewed MCP binding identity/digest, version lock, Golam-local mapping identity/digest, lifecycle state and queued/prepared/cached decision references. Every local or remote dispatch revalidates the exact active binding immediately before dispatch. Deprecated, revoked, replaced, unreviewed, unknown or version/digest/mapping-mismatched state rejects queued/prepared calls, mapped descriptors and cached capability/approval/dispatch decisions. A superseded binding cannot donate authority to a replacement.

Executable skills/local MCP child processes still share the production containment gate. Remote MCP still shares network/egress/authenticated-endpoint/credential-scope/strict-local gates. ACP retains authenticated local-client semantics. The official MCP Rust SDK remains a candidate pending exact minimal dependency qualification.

## Review finding reconciliation

Historical substantive reviews are defect-discovery evidence, not final T005-015 evidence. Final qualification must bind the final unchanged planning head after fresh exact-head CI.

The known finding set is reconciled as follows:

1. **Credential-bearing network redirects** — authenticated endpoint identity, credential-origin/operation scope, strip/revalidate/re-broker semantics and downgrade denial are explicit.
2. **ToolDescriptor/ToolRequest under-binding** — finite I/O/duration schemas, reconciliation/verification schemas, operation/resource/precondition bindings and post-PREPARED immutability are explicit.
3. **Derivative availability contradiction** — missing/corrupt derivatives do not block canonical Markdown/SQLite startup or ordinary reads.
4. **Managed-memory mutation lifecycle** — every Golam-generated managed mutation is durably PREPARED before canonical mutation, executes only through the governed writer, records integrity-chained terminal/read-back evidence and keeps ambiguous completion `UNKNOWN_OUTCOME` until reconciliation.
5. **Process descendant supervision under-binding** — descendant supervision and terminal process-tree reconciliation are separate bindings; cancellation is not terminal proof.
6. **MCP local-mapping/lifecycle under-binding** — exact reviewed mapping identity/digest, lifecycle, replacement and revocation qualification are explicit.
7. **Memory version attribution under-binding** — `MemoryVersion` preserves creating principal, committing writer and mutation Effect through restart/reconciliation.
8. **Promotion validator ordered after writer enablement** — T005-051 is an explicit prerequisite for T005-048; there is no pre-validator direct-write phase.
9. **Phase D external search binary before containment admission** — Phase D is in-process only; external search binaries remain ineligible until after T005-077 plus their own Source Foundry qualification.
10. **Ambient environment ambiguity** — process launch requires cleared ambient environment with only explicitly bound values/secret handles.
11. **Secret-taint downgrade ambiguity** — `SECRET_DERIVED` cannot be downgraded by redaction/summarization/transformation/verification.
12. **Managed-memory commit-time TOCTOU omission** — exact target/digest/version is revalidated immediately at commit and conditional identity-preserving replacement is mandatory.
13. **Verification narrative self-invalidating ledger risk** — terminal exact-head CI/review dispositions are recorded in immutable GitHub evidence rather than committed back after qualification.
14. **Spec 004 closeout checklist challenge** — rejected against higher-priority live PR #14 / CI #777 evidence recording `SPEC_004_CLOSED_CANONICAL=YES` on exact `main@390ea842837a7d85dca165d9291d5eb54c3f11db`.
15. **Memory intent missing exact Markdown/store structural bindings** — `MemoryMutationIntent` now binds exact Markdown target identity, digest/version and dedicated memory-operational-store identity/schema; all survive PREPARED unchanged.
16. **Memory cross-store reconciliation under-specified** — authority journal, Markdown and memory-operational SQLite are separate evidence surfaces; terminal success requires exact cross-store read-back, and wrong-store/split-store states fail closed.
17. **Markdown authority-bearing front matter under-specified** — front matter/body is content only; reserved authority-bearing fields are rejected/quarantined into reconciliation rather than imported as authority.
18. **Skill dispatch revocation binding under-specified** — exact package/version/content digest/local capability mapping plus queued/prepared/cached decision refs are now explicit and revalidated immediately before activation/dispatch.
19. **MCP dispatch revocation binding under-specified** — exact binding identity/digest/version lock/local mapping plus queued/prepared/cached decision refs are explicit and revalidated immediately before every local/remote dispatch.

## Qualification lineage

CI/review evidence is intentionally historical after any head mutation:

- CI #782 / run `33516670304` on `feff88df...`: SUCCESS, later stale.
- CI #785 / run `33523752838` on `b73b10c...`: SUCCESS, later stale.
- Cubic 14-file review on `b73b10c...`: substantive defect evidence, later stale after repairs.
- CI #788 / run `33528970640` on `0db8f42...`: SUCCESS, later stale.
- CI #790 / run `33534244464` on `72cc4c73766994ff4d9d97fa1d73714c8b9e0211`: SUCCESS on Windows/macOS/Ubuntu, but this convergence propagation commit intentionally supersedes that head and therefore makes #790 historical/stale for final T005-014.
- CodeRabbit historical full review on the pre-convergence lineage produced five material planning findings. Those findings drove the exact target/digest/store/CAS/front-matter/dispatch-binding repairs. Its old review is defect evidence, not final T005-015 evidence after this mutation.
- Cubic is monthly-quota blocked and Qodo is billing-blocked; status/billing output is not review evidence.

The successor exact planning head MUST obtain fresh Windows/macOS/Ubuntu exact-head CI and, only after that succeeds, a fresh substantive independent semantic review on the unchanged head. Any further content mutation invalidates those gates again.

## Material-risk review

### Risk 1 — production process containment could become a cross-platform scope sink
Mitigation: admission is per exact platform/profile; unsupported platforms remain explicit denial states.

### Risk 2 — generic file tools could undermine kernel protections
Mitigation: protected-resource exclusion is independent of lexical path authority and enforced at the protected action boundary.

### Risk 3 — memory could become self-reinforcing model truth
Mitigation: candidate/promotion separation, live-state precedence, attributable promotion, contradiction preservation, derivative non-authority and monotonic secret-derived taint.

### Risk 4 — memory durability or attribution could bypass Effect Gate truth
Mitigation: exact intent target/digest/version/store/effect bindings, PREPARED-before-mutation, authority-store separation, commit-time CAS, content-only Markdown authority boundary, creator/writer/effect attribution and three-surface reconciliation before success.

### Risk 5 — MCP/skills could smuggle stale authority
Mitigation: exact dispatch bindings plus immediate pre-dispatch lifecycle/version/digest/mapping revalidation invalidate queued/prepared/cached/approved state after replacement/revocation.

### Risk 6 — optional retrieval infrastructure could become a hidden dependency
Mitigation: L0 is sufficient; search is in-process while `native:unqualified`; derivative systems remain optional and rebuildable.

### Risk 7 — redirects could leak brokered credentials despite valid egress
Mitigation: credential disclosure has a separate authenticated endpoint/scope gate and sensitive material is stripped until fresh authorization re-brokers it.

### Risk 8 — cancellation could be mistaken for process-tree termination
Mitigation: descendant supervision and terminal process-tree reconciliation are separate mandatory evidence requirements.

## Planning convergence result

The convergence propagation commit reconciles every presently known material internal/review finding across the normative planning package. This is self-analysis only and does not satisfy T005-015.

Because this commit mutates the source head after CI #790, T005-014 and T005-015 both require fresh exact-head evidence.

```text
T005_PLANNING_INTERNAL_CONVERGENCE=MATERIAL_FINDINGS_REPAIRED_REQUALIFICATION_REQUIRED
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
PHASE_D_EXTERNAL_SEARCH_BINARY=DENIED_NATIVE_UNQUALIFIED
MEMORY_WRITER_PRE_VALIDATOR_DIRECT_WRITE=DENIED
PLANNING_CODE_REUSED=NO
PLANNING_DEPENDENCY_ADDED=NO
NONCANONICAL_PR_6_7_8_PROMOTED_TO_AUTHORITY=NO
PRE_REPAIR_CI_782=SUCCESS_STALE
PRE_REPAIR_CI_785=SUCCESS_STALE
PRE_REPAIR_CI_788=SUCCESS_STALE
PRE_REPAIR_CI_790=SUCCESS_STALE_AFTER_CONVERGENCE_PROPAGATION
PRE_REPAIR_CUBIC_REVIEW=SUBSTANTIVE_STALE_AFTER_REPAIR
PRE_REPAIR_CODERABBIT_REVIEW=FIVE_MATERIAL_FINDINGS_REPAIRED_STALE_AFTER_CONVERGENCE
FINAL_EXACT_HEAD_CI=REQUIRED_EXTERNAL_GITHUB_EVIDENCE
FINAL_INDEPENDENT_REVIEW=REQUIRED_EXTERNAL_GITHUB_EVIDENCE_AFTER_CI
PR_READY=NO
MERGE_AUTHORIZED=NO
WAIVER_TAKEN=NO
```
