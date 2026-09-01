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
- [x] Markdown/SQLite canonical memory model is preserved.
- [x] Source Foundry gates are explicit before dependency/code admission.
- [x] Exact-head verification and independent review remain mandatory.

## Production execution boundary

- [x] Planning explicitly records `PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO` from canonical Spec 003.
- [x] Shell/process/local executable MCP/skill launch remains disabled until exact production containment admission.
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

## Context

- [x] L0 evidence is required and sufficient for the initial slice.
- [x] L1 Tree-sitter/LSP requires measured need + exact admission.
- [x] L2 graph/dataflow/vector/runtime indexing is deferred.
- [x] Every context item carries provenance, authority, taint, permission and freshness metadata.
- [x] Ranking cannot raise authority or clear taint.
- [x] Live authoritative state outranks stale memory/context.

## Memory

- [x] Managed Markdown is canonical durable knowledge.
- [x] SQLite is canonical operational state.
- [x] Derived indexes are rebuildable and optional; missing derivatives do not block canonical memory access.
- [x] One governed writer owns Golam-generated managed memory mutation.
- [x] Every managed mutation binds current Kernel authorization plus applicable approval/verifier evidence in an immutable `MemoryMutationIntent`.
- [x] A durable authorized Effect Gate PREPARED intent exists before the first canonical Markdown/SQLite mutation.
- [x] Terminal outcome and required read-back/verification evidence are integrity-chained; ambiguous completion remains `UNKNOWN_OUTCOME` and blocks dependent memory mutations until reconciliation.
- [x] User hand-edits are detected/reconciled rather than overwritten.
- [x] Promotion requires attributable approval or deterministic pre-registered verification.
- [x] `SECRET_DERIVED` is excluded from canonical long-term memory and cannot be cleared by redaction/summarization/transformation/verification.
- [x] `FORGET`/`REDACT` uses the same durable effect lifecycle, invalidates/rebuilds derivatives, reconciles partial multi-store completion, and makes no false external-erasure claim.

## Skills and protocols

- [x] Instruction-only skills may precede executable skills.
- [x] Executable skills are production-sandbox gated.
- [x] MCP advertisements/results remain untrusted and cannot mint Golam capabilities.
- [x] Remote MCP is egress/strict-local/endpoint-identity/credential-scope gated.
- [x] ACP preserves authenticated local-client semantics.
- [x] Official MCP Rust SDK is a candidate, not automatically admitted.

## Verification posture

- [x] Ordinary CI remains hermetic and credential/model/service independent.
- [x] Path/protected-resource, process containment, network credential, context, memory and protocol adversarial families are enumerated.
- [x] Planning closeout requires exact-head Windows/macOS/Ubuntu CI.
- [x] Planning closeout requires substantive independent semantic review on the unchanged head.
- [x] Material findings must be repaired and requalified before Ready/merge.
- [x] Expected-head guarded merge and push-triggered post-merge main CI are required.

## Remaining planning gates

- [ ] Planning package is committed atomically to the planning branch.
- [ ] Planning analysis confirms no internal contradiction/material omission.
- [ ] Exact-head planning CI succeeds.
- [ ] Independent semantic review succeeds on that exact head.
- [ ] Reconciliation closes all material findings without waiver.
- [ ] Planning PR becomes non-Draft only after required gates.
- [ ] Guarded expected-head planning merge succeeds.
- [ ] Push-triggered canonical-main CI succeeds.
- [ ] Planning is marked `CLOSED_CANONICAL` and implementation branch is created from the verified main.
