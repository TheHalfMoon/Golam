# Implementation Readiness — Spec 005

**Status**: PLANNING CANDIDATE

## Authority and scope

- [x] Canonical predecessor is exact `main@390ea842837a7d85dca165d9291d5eb54c3f11db`.
- [x] Spec 001 T050 authorizes bounded Spec 005 after Spec 004 canonical closeout.
- [x] Spec 004 is closed by live merge + post-merge CI evidence.
- [x] Noncanonical PRs #6–#8 are not treated as predecessors or authority.
- [x] Later Spec 006+ features remain out of scope.

## Constitutional fit

- [x] Local ownership and strict-local hard denial are preserved.
- [x] Rust owns trusted-path semantics; adapters remain untrusted/replaceable.
- [x] Tool/model/protocol output cannot self-authorize.
- [x] Consequential mutations remain behind the Effect Gate.
- [x] Secrets/taint rules survive tool/context/memory derivation; `SECRET_DERIVED` cannot be downgraded inside Spec 005.
- [x] Managed Markdown plus dedicated memory-operational SQLite canonical-memory model is preserved without aliasing authority state.
- [x] Source Foundry gates are explicit before dependency/code admission.
- [x] Exact-head verification and independent review remain mandatory.

## Production execution boundary

- [x] Planning explicitly records `PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO` from canonical Spec 003.
- [x] Shell/process/local executable MCP/skill launch remains disabled until exact production containment admission.
- [x] Phase D L0 search is explicitly in-process; an external search binary is denied while `native:unqualified` and can only be reconsidered after an exact production containment profile is admitted.
- [x] Donor `skipApproval` semantics are explicitly rejected.
- [x] Unsupported platforms/profiles fail closed.

## Tool safety

- [x] Tool I/O byte/count and duration bounds are explicit and finite; protected requests become immutable once durably prepared.
- [x] Path strings are not authority.
- [x] Symlink/reparse/junction/mount alias handling is explicit.
- [x] Protected Golam resources are excluded from generic filesystem authority.
- [x] Special files and read/list/walk bounds are explicit.
- [x] Mutation TOCTOU/precondition behavior is explicit.
- [x] Git writes are effects; destructive history rewriting is excluded.
- [x] Browser/network tools are egress-gated and do not pull Desktop control into scope.
- [x] Credential-bearing network hops require authenticated encrypted endpoint identity; redirects/origin/protocol changes strip and revalidate/re-broker sensitive material or fail closed.
- [x] Process launch binds descendant supervision **and** process-tree terminal reconciliation; cancellation alone is not proof that descendants terminated.

## Context

- [x] L0 evidence is required and sufficient for the initial slice.
- [x] Initial L0 text search is in-process and cannot create a hidden pre-containment child-process dependency.
- [x] L1 Tree-sitter/LSP requires measured need + exact admission.
- [x] L2 graph/dataflow/vector/runtime indexing is deferred.
- [x] Every context item carries provenance, authority, taint, permission and freshness metadata.
- [x] Ranking cannot raise authority or clear taint.
- [x] Live authoritative state outranks stale memory/context.

## Memory

- [x] Managed Markdown is canonical durable knowledge.
- [x] A dedicated memory-operational SQLite store is canonical operational state and is separate from the protected authority database.
- [x] Derived indexes are rebuildable and optional; missing derivatives do not block canonical memory access.
- [x] One governed writer owns Golam-generated managed memory mutation.
- [x] Promotion-authority validation is an explicit prerequisite before the managed writer may be enabled for mutation.
- [x] Every managed mutation binds current Kernel authorization plus applicable approval/verifier evidence in an immutable `MemoryMutationIntent`.
- [x] `MemoryMutationIntent` also binds exact expected Markdown target identity, expected digest/version, exact dedicated memory-operational-SQLite store identity/schema and unique Effect identity; every protected binding participates in the immutable intent digest and survives PREPARED unchanged.
- [x] A durable authorized Effect Gate PREPARED intent exists in the authority database before the first canonical Markdown/memory-SQLite mutation; the memory-operational store is separate and is never treated as authority.
- [x] Every effect-owned memory-operational row binds the exact memory-store identity/schema, Effect identity, mutation-intent digest and relevant expected/committed version state.
- [x] Markdown replacement performs immediate commit-time target-identity + digest/version revalidation and conditional compare-and-replace/identity-preserving replacement; mismatch or unpreservable identity fails closed as `USER_EDIT_DETECTED`/`CONFLICT`.
- [x] Markdown body/front matter is content only; reserved authority-bearing fields are rejected/quarantined for explicit reconciliation rather than setting scope, taint, provenance authority, approval, authorization, managed version identity, promotion state or Effect Gate state.
- [x] Terminal success requires read-back/reconciliation across the authority journal, exact Markdown identity/digest/version and exact memory-operational-store/effect rows; file-without-row, row-without-file, wrong-store, stale-digest/identity and unreadable/partial cuts cannot become success.
- [x] Terminal outcome and required read-back/verification evidence are integrity-chained; ambiguous completion remains `UNKNOWN_OUTCOME` and blocks dependent memory mutations until reconciliation.
- [x] Every committed managed version preserves initiating/creating principal, governed writer identity and exact mutation Effect attribution through restart/reconciliation.
- [x] User hand-edits are detected/reconciled rather than overwritten.
- [x] Promotion requires attributable approval or deterministic pre-registered verification.
- [x] `SECRET_DERIVED` is excluded from canonical long-term memory and cannot be cleared by redaction/summarization/transformation/verification.
- [x] `FORGET`/`REDACT` uses the same durable effect lifecycle, invalidates/rebuilds derivatives, reconciles partial multi-store completion, and makes no false external-erasure claim.

## Skills and protocols

- [x] Instruction-only skills may precede executable skills.
- [x] Executable skills are production-sandbox gated.
- [x] `SkillDispatchBinding` explicitly binds the exact reviewed package/version/content digest, reviewed lifecycle/admission state, reviewed Golam-local capability mapping and queued/prepared/cached decision references.
- [x] Exact skill dispatch bindings are revalidated immediately before every instruction activation or executable dispatch; deprecated/revoked/replaced/unknown/version/digest/mapping-mismatched state invalidates queued, prepared-but-not-dispatched, cached capability/approval and dispatch-decision state.
- [x] MCP advertisements/results remain untrusted and cannot mint Golam capabilities.
- [x] `McpDispatchBinding` explicitly binds exact reviewed binding identity/digest, version lock, Golam-local mapping identity/digest, lifecycle state and queued/prepared/cached decision references.
- [x] Exact MCP dispatch bindings are revalidated immediately before every local or remote dispatch; stale/replaced/revoked/unreviewed/unknown/version/digest/mapping-mismatched state invalidates queued/prepared calls, mapped descriptors, capability/approval caches and dispatch decisions.
- [x] Remote MCP is egress/strict-local/endpoint-identity/credential-scope gated.
- [x] ACP preserves authenticated local-client semantics.
- [x] Official MCP Rust SDK is a candidate, not automatically admitted.

## Verification posture

- [x] Ordinary CI remains hermetic and credential/model/service independent.
- [x] Path/protected-resource, process containment/reconciliation, network credential, context, memory and protocol adversarial families are enumerated.
- [x] Memory adversarial readiness explicitly covers stale Markdown digest/version, target-identity swap, authority-bearing front matter, wrong memory-store binding and every authority-journal/Markdown/memory-SQLite partial-store cut.
- [x] Skill/MCP adversarial readiness covers dispatch after deprecation/revocation/replacement/version/digest/mapping mismatch and rejection of stale queued/prepared/cached/approved decisions.
- [x] Planning closeout requires exact-head Windows/macOS/Ubuntu CI.
- [x] Planning closeout requires substantive independent semantic review on the unchanged head.
- [x] Material findings must be repaired and requalified before Ready/merge.
- [x] Expected-head guarded merge and push-triggered post-merge main CI are required.

## Remaining planning gates

- [x] Planning package is committed as bounded planning artifacts on the planning branch.
- [x] Internal convergence analysis now propagates all presently known memory/store/CAS/front-matter/skill/MCP review contracts across the normative planning package.
- [ ] Fresh exact-head planning CI succeeds after the final convergence propagation commit.
- [ ] Independent semantic review succeeds on that same unchanged exact head after CI.
- [ ] Reconciliation closes all material findings without waiver and without stale evidence.
- [ ] Planning PR becomes non-Draft only after required gates.
- [ ] Guarded expected-head planning merge succeeds.
- [ ] Push-triggered canonical-main CI succeeds.
- [ ] Planning is marked `CLOSED_CANONICAL` and implementation branch is created from the verified main.
