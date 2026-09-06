# Tasks — Spec 005 Local Tools, Context & Memory

**Canonical planning base**: `main@390ea842837a7d85dca165d9291d5eb54c3f11db`  
**Planning branch**: `spec/005-local-tools-context-memory`

Execute strictly in dependency order. A checked task means the task has completed under its exact recorded implementation/disposition evidence; it does not waive later exact-head/review/merge/post-merge gates. Conditional tasks closed without implementation must carry an explicit `NOT_APPLICABLE`, `NOT_REQUIRED`, or `DEFER_NOT_NEEDED` disposition rather than imply an unperformed runtime claim.

---

## Phase A — Planning bootstrap and authority

- [x] **T005-001** Re-read live canonical `main`, constitution, Spec 001 T050–T059, Spec 003 production-executor evidence and Spec 004 closeout before defining scope.
- [x] **T005-002** Confirm Spec 004 is `CLOSED_CANONICAL` from live merge + post-merge CI evidence, and confirm noncanonical PRs #6–#8 are not implementation predecessors.
- [x] **T005-003** Create `spec/005-local-tools-context-memory` from exact canonical `main@390ea842837a7d85dca165d9291d5eb54c3f11db` with no hidden delta.
- [x] **T005-004** Freeze bounded `spec.md` for local tools, L0 context, governed memory, skills/MCP/ACP and strict-local behavior while excluding Spec 006+ scope.
- [x] **T005-005** Close planning clarifications including path identity, production sandbox gating, memory authority, protocol authority and noncanonical-proposal handling.
- [x] **T005-006** Research exact current source states for Golam-Research, grok-build, OpenClaw, Hermes, MCP, ACP, Agent Skills, ripgrep, Tree-sitter, Mem0 and Qdrant.
- [x] **T005-007** Record Source Foundry planning dispositions and explicitly reject donor `skipApproval` semantics; admit no code/dependency during planning.
- [x] **T005-008** Freeze component ownership and phased implementation strategy in `plan.md` without creating empty crates.
- [x] **T005-009** Freeze pure data identities and invariants in `data-model.md`.
- [x] **T005-010** Freeze normative tool/context, memory and skills/protocol contracts under `contracts/`.
- [x] **T005-011** Freeze hermetic implementation/qualification guidance in `quickstart.md`.
- [x] **T005-012** Complete implementation-readiness checklist and preserve the production-executor denial state.
- [x] **T005-013** Create this dependency-ordered task ledger and planning analysis.

## Phase B — Planning closeout

- [x] **T005-014** Run exact-head planning CI on Windows, macOS and Ubuntu with the repository's complete required workflow set.
- [x] **T005-015** Obtain a substantive independent semantic review on the unchanged exact planning head after T005-014. Status-only/billing/rate-limit/self-review output is insufficient.
- [x] **T005-016** Repair every material planning/review finding forward-only; any branch mutation invalidates prior exact-head CI/review and requires requalification.
- [x] **T005-017** Converge the exact planning head with no unresolved material review threads, no waiver and no stale evidence.
- [x] **T005-018** Transition the qualified planning PR to non-Draft/Ready. If the connector lifecycle transition is unavailable, only a previously canonicalized same-SHA/no-content-delta lifecycle relay may be used, with its own CI and independent consistency review.
- [x] **T005-019** Re-fetch live base/head immediately before a guarded expected-head planning merge and merge only the exact qualified head.
- [x] **T005-020** Require push-triggered canonical-main CI to succeed on the exact planning merge SHA.
- [x] **T005-021** Set `SPEC_005_PLANNING_COMPLETE=YES` and `SPEC_005_PLANNING_CLOSED_CANONICAL=YES` only after T005-020.
- [x] **T005-022** Re-read canonical main after planning closeout and create the implementation branch from that exact verified main. Do not begin implementation before this task.

Planning closeout evidence: PR #19 merged to canonical `main@4bd23b218304663349fb2f703cedd40c7a3038af`; push CI #798 / run `33548321187` completed successfully on Windows, macOS and Ubuntu; implementation branch creation was then bound to that exact main.

---

## Phase C — Core tool/context/memory contracts in Rust

- [x] **T005-025** Implement pure versioned `ToolId`, `ToolVersion`, `ToolIoBounds`, `ToolDurationBounds`, `ToolReconciliationPolicy`, `ToolVerificationPolicy`, `ToolDescriptor`, operation/network/sandbox enums and deterministic validation in `golam-core` without privileged state. Bounds applicable to executable/read operations must be explicit and finite; “unbounded by omission” is invalid.
- [x] **T005-026** Implement immutable `ToolRequest`, `ToolResult`, candidate binding and deterministic serialization/digest semantics. Bind requested operation, authorized resource class, target identity/resolution plan and current preconditions. Once a protected request is durably prepared, target/operation/precondition/authority changes require a new request/effect identity.
- [x] **T005-027** Implement pure authorized-root, resolved-target-identity and file-mutation-expectation types with platform-neutral invariants.
- [x] **T005-028** Implement `ContextEvidence`, `EvidenceRequirement`, `ContextCapsule` and sufficiency/freshness/authority/taint validation types.
- [x] **T005-029** Implement memory candidate/item/version/operation/reconciliation/derivative-generation types plus deterministic `MemoryMutationIntent`/terminal outcome types with no authority-bearing free-form fields. `MemoryMutationIntent` must bind initiating principal, current Kernel authorization, applicable promotion authority, expected current versions, exact expected Markdown target identity, expected Markdown digest/version, exact dedicated memory-operational-SQLite store binding and unique Effect identity, with all protected fields included in the immutable intent digest and surviving PREPARED unchanged. `MemoryVersion` must preserve initiating/creating principal, governed committing-writer identity and exact mutation Effect reference as distinct attribution.
- [x] **T005-030** Implement skill/protocol descriptor/binding types that cannot encode authority-bearing capability material, including explicit `SkillDispatchBinding` and `McpDispatchBinding` identities for the exact reviewed package/binding digest, version, local capability/mapping identity and queued/cached decision refs. Replaced/revoked/unreviewed/mismatched bindings and stale queued/prepared/cached capability/approval/dispatch decisions fail closed and cannot silently retain prior authority.
- [x] **T005-031** Extend durable ledger evidence for tool requests/results and context provenance without making projections canonical authority.
- [x] **T005-032** Extend durable memory governance/version/reconciliation/promotion/effect evidence with required integrity behavior for security-critical records, including PREPARED intent, exact memory-store binding, creator/writer/effect attribution, authority-journal/Markdown/memory-SQLite read-back references and terminal/`UNKNOWN_OUTCOME` evidence identities.
- [x] **T005-033** Add focused property/adversarial tests proving invalid descriptors/requests/candidates cannot mint authority, clear taint, encode unbounded operations, mutate a durably prepared protected request, alter PREPARED Markdown target/digest/version/store bindings, collapse memory creator/writer/effect attribution, revive a revoked MCP binding, or dispatch from stale/replaced/revoked skill/MCP queued/prepared/cached/approval state.
- [x] **T005-034** Run focused exact-head qualification for Phase C before proceeding.

## Phase D — Bounded filesystem reads and L0 context

- [x] **T005-035** Implement platform-aware authorized-root and target-resolution service for read-only operations, including symlink/reparse/junction/protected-resource exclusion evidence.
- [x] **T005-036** Implement bounded regular-file stat/read with size/type/permission limits and failure-preserving behavior.
- [x] **T005-037** Implement bounded directory list/walk with count/depth/time limits and deterministic ordering where required.
- [x] **T005-038** Source-Foundry-qualify an exact **in-process** L0 text-search implementation before adding a dependency. Phase D may use Golam-owned bounded search or exact admitted Rust crate surfaces only. An external search executable is ineligible while production is `native:unqualified` and may be reconsidered only after T005-077.
- [x] **T005-039** Implement bounded in-process text search under authorized roots with exact match-file/line/content provenance, no shell interpolation and no child-process launch path.
- [x] **T005-040** Implement bounded Git read evidence: repository identity, HEAD/ref, status, diff, log/tree/blob observation without mutation authority.
- [x] **T005-041** Implement the L0 Context Compiler pipeline over user-selected files, file reads, in-process search, Git, canonical goal/evidence and permitted memory.
- [x] **T005-042** Implement permission/authority/freshness/taint filtering and prove retrieval score cannot raise authority or clear taint.
- [x] **T005-043** Implement explicit sufficiency/missing-requirement output and bounded replan without unbounded recursive retrieval.
- [x] **T005-044** Run path/read/search/Git/context adversarial qualification and exact-head Phase D CI, including proof that L0 search cannot spawn an external utility while `native:unqualified`.

Phase D evidence: `implementation/l0-search-source-foundry.md`, Git-read Source Foundry/admission records and `implementation/phase-d-qualification.md`.

## Phase E — Canonical managed memory

T005-048 depends explicitly on T005-031..032, T005-045..047 **and T005-051**. T005-051 is a required prerequisite and MUST execute before T005-048 despite its higher numeric identifier. T005-049..050 and T005-052..058 then depend on the durable mutation lifecycle established by T005-048 unless a task is strictly read-only.

- [x] **T005-045** Freeze exact managed-vault on-disk Markdown schema plus the exact dedicated memory-operational-SQLite path/store identity/schema reference that `MemoryMutationIntent` binds, without exposing protected control state as generic memory files and without aliasing the authority database.
- [x] **T005-046** Implement canonical Markdown parser/serializer with stable item/version identity and bounded front-matter/content handling. Markdown body/front matter is content only; reserved authority-bearing fields that purport to set scope, taint, provenance authority, approval, authorization, managed version identity, promotion state or Effect Gate state are rejected/quarantined into explicit reconciliation rather than imported as authority.
- [x] **T005-047** Implement SQLite operational state for versions, promotion evidence, reconciliation, conflicts, supersession, derivative generations, creator/writer/effect attribution and effect-owned mutation state. Every effect-owned row must bind the exact memory store identity/schema ref, Effect identity, mutation-intent digest and relevant version/digest state.
- [x] **T005-051** **Prerequisite for T005-048.** Implement attributable human-promotion authority validation and deterministic pre-registered verifier promotion; reject free-form/model self-approval, candidate-selected verifier authority and stale/revoked authorization. Qualify this validator before the managed writer can be enabled for mutation.
- [x] **T005-048** Implement the single governed memory writer as an Effect Gate handler only after T005-051 is qualified. Every Golam-generated `MemoryMutationIntent` must bind initiating principal, current Kernel authorization, applicable approval/pre-registered verifier evidence, expected current versions, exact expected Markdown target identity, expected Markdown digest/version, exact dedicated memory-operational-SQLite store binding and unique Effect identity; persist that exact authorized intent as durable PREPARED authority evidence before the first canonical Markdown/SQLite mutation; immediately before replacement revalidate the exact Markdown identity/digest/version and use conditional compare-and-replace/identity-preserving replacement, failing closed as `USER_EDIT_DETECTED`/`CONFLICT` on mismatch or unpreservable identity; write operational rows only to the bound memory store under the exact Effect/intent digest; persist creator/writer/effect attribution; invalidate affected derivatives; require authority-journal + Markdown + memory-SQLite read-back/reconciliation before terminal success; then record integrity-chained terminal outcome evidence. Ambiguous completion remains `UNKNOWN_OUTCOME` and blocks dependent managed-memory mutation until reconciliation.
- [x] **T005-049** Implement restart reconciliation for every authority-PREPARED/writer/Markdown/memory-SQLite/terminal durability cut, including stale Markdown digest/version, target-identity swap, wrong memory-store binding, file-without-row, row-without-readable/expected-file, unreadable store, cross-store disagreement and `UNKNOWN_OUTCOME`; absence of terminal evidence is never success and creator/writer/effect attribution must survive recovery.
- [x] **T005-050** Implement user hand-edit detection and fail-closed reconciliation without silent overwrite, including edit-after-check-before-replace races, target identity changes at commit time and malicious/reserved authority-bearing Markdown/front-matter keys that require explicit quarantine/reconciliation.
- [x] **T005-052** Enforce monotonic `SECRET_DERIVED` rejection at the canonical memory admission boundary; redaction/summarization/transformation/verification cannot downgrade the taint. Only independently sourced non-secret provenance may form a separate candidate.
- [x] **T005-053** Implement `ADD`, `UPDATE`, `SUPERSEDE`, `CONTRADICT`, `MERGE`, and `EXPIRE` with immutable version lineage through T005-048's prepared-effect lifecycle.
- [x] **T005-054** Implement `FORGET` and `REDACT` through the same T005-048 effect lifecycle with active canonical-content removal, bounded non-plaintext audit facts, Markdown/SQLite/derivative reconciliation and explicit external-artifact honesty; partial multi-store completion cannot become success.
- [x] **T005-055** Implement deterministic local derivative text/metadata index generation that is discardable and rebuildable from canonical memory.
- [x] **T005-056** Ensure missing/corrupt derivatives do not block canonical memory startup/access. Derivative-dependent operations trigger governed rebuild from canonical state and fail only that dependent operation closed if rebuild/qualification cannot complete.
- [x] **T005-057** Prove live authoritative repository/filesystem state outranks stale memory and surfaces conflict evidence.
- [x] **T005-058** Run memory-poisoning, forged/stale-promotion, writer-before-validator enablement, secret-derived/taint-downgrade, creator/writer/effect attribution, stale Markdown digest/version, target-identity swap, edit-after-check-before-replace, conditional-replace failure, malicious authority-bearing front matter, wrong memory-store binding, authority-journal/Markdown/memory-SQLite split-store cuts, file-without-row, row-without-file, disk-full/crash, PREPARED/terminal/`UNKNOWN_OUTCOME`, dependent-mutation blocking, stale-memory and FORGET/REDACT partial-completion/resurrection qualification.

Phase E evidence: `implementation/phase-e-memory-qualification.md` plus the canonical managed-memory, restart, promotion, writer, control and derivative test/evidence surfaces.

## Phase F — Consequential filesystem/Git mutations

- [x] **T005-060** Implement identity-preserving file create/write/replace through the existing Effect Gate with expected-parent/target/content preconditions and read-back verification.
- [x] **T005-061** Implement governed rename/delete operations with explicit target-vs-parent authority and stale-identity denial.
- [x] **T005-062** Add symlink/reparse/junction/rename-swap TOCTOU adversarial harnesses for mutation boundaries on supported platforms.
- [x] **T005-063** Implement bounded Git add/commit/branch mutation as Effect Gate operations bound to expected repository HEAD/index/worktree state.
- [x] **T005-064** Keep force push, force ref movement, rebase/shared-history rewrite and equivalent destructive operations outside ordinary tool authority.
- [x] **T005-065** Implement deterministic post-operation verification and reconciliation for ambiguous filesystem/Git effects using existing effect semantics.
- [x] **T005-066** Prove generic file/Git tool capability cannot reach protected Golam resources or rewrite policy/lease/approval/secret/effect/audit state.
- [x] **T005-067** Run restart/UNKNOWN_OUTCOME/idempotency/stale-state adversarial qualification for mutations.
- [x] **T005-068** Run exact-head Phase F qualification.

Phase F evidence: `implementation/phase-f-mutation-qualification.md` and `implementation/phase-f-exact-head-closeout.md`.

## Phase G — Production native containment and process tools

- [x] **T005-070** Re-read canonical Spec 003 sandbox/executor evidence and freeze exact production containment requirements per target platform before selecting an implementation.
- [x] **T005-071** Research and Source-Foundry-qualify exact production native containment primitives/dependencies for the first supported platform; do not infer cross-platform equivalence.
- [x] **T005-072** Implement the first production containment profile with cleared ambient environment, explicit FS/network/device/resource rights, executable/cwd identity, handle rules, descendant supervision, process-tree ownership/discovery, cancellation and terminal process-tree reconciliation. Cancellation alone is not terminal proof.
- [x] **T005-073** Integrate secret brokerage/unbrokerable fallback and value-aware redaction into process launch evidence without argv/ambient-secret leakage.
- [x] **T005-074** Qualify strict-local no-network behavior through an external descendant-aware observation on the admitted production profile.
- [x] **T005-075** Qualify filesystem/namespace/OS containment claims only to the exact proven platform boundary; unsupported claims remain explicit denial states.
- [x] **T005-076** Add hostile payload corpus for process-tree escape, environment leakage, descriptor/handle inheritance, forbidden filesystem/device access, timeout/cancel and descendant persistence/reconciliation.
- [x] **T005-077** Mark the exact profile `ADMITTED` only after focused + repository CI evidence and independent semantic/security review are clean.
- [x] **T005-078** Implement governed argv-style process execution through ToolRequest + Kernel/Effect Gate + admitted executor. If an external search binary is selected, it may be Source-Foundry-qualified and launched only here or later, after T005-077, under the same admitted process boundary; otherwise keep that path unavailable.
- [x] **T005-079** If shell syntax is selected, implement explicit parse/ambiguity/redirection/substitution evidence and deny ambiguous command graphs; never implement donor `skipApproval` semantics. **Disposition: NOT_APPLICABLE — shell syntax was not selected; no ordinary shell path was implemented and donor `skipApproval` remains rejected.**
- [x] **T005-080** Requalify strict-local, secret isolation, UNKNOWN_OUTCOME/restart, descendant supervision and terminal process-tree reconciliation on the exact process-tool head.

Phase G live admission evidence recorded `T005_077=PASS`, `PRODUCTION_PROFILE_ADMITTED=YES`, exact profile `platform:linux-x86_64-landlock-v4-seccomp-v2`, with shell still disabled. Later process-v2 requalification evidence is recorded in `implementation/process-v2-requalification-evidence.md` and ordinary CI continues to execute the governed process-v2 E2E gate on Linux x86_64.

## Phase H — Skills, MCP and ACP

- [x] **T005-085** Implement Agent Skills-compatible instruction package discovery/provenance/version-lock validation with content treated as untrusted context. Freeze the exact reviewed package/version/content digest and reviewed capability-mapping identity used by `SkillDispatchBinding`.
- [x] **T005-086** Implement governed skill lifecycle; instruction-only admission is independent of executable admission. Every lifecycle transition to deprecated/revoked/replaced/unknown or any version/digest/mapping mismatch invalidates queued calls, prepared-but-not-dispatched calls, cached capability/approval material and cached dispatch decisions bound to the old skill identity.
- [x] **T005-087** Keep executable skill scripts disabled on unqualified profiles; when enabled, immediately before every instruction activation or executable dispatch revalidate the exact active reviewed skill package/version/content digest/capability mapping and current lifecycle state, reject stale queued/prepared/cached/approved decisions, then route execution through the admitted process/tool/effect boundary.
- [x] **T005-088** Qualify the exact MCP implementation strategy; if using official Rust SDK, select minimal exact crates/features/transitive/network/process closure before dependency admission.
- [x] **T005-089** Implement MCP descriptor/resource/prompt normalization into untrusted Golam protocol/tool types with bounded parsing and stable server/version identity. Persist exact reviewed `McpServerBinding` identity/digest, version lock, Golam-local mapping identity/digest and lifecycle state; version replacement/revocation creates a new reviewed binding state rather than silently inheriting prior authority.
- [x] **T005-090** Implement local MCP binding/launch only through an admitted production containment profile and current policy/capability/effect authority. Immediately before process dispatch, revalidate the exact active binding identity/digest, version lock and local mapping identity/digest; reject stale queued/prepared calls, mapped descriptors, capability/approval caches and dispatch decisions after deprecation/revocation/replacement/mismatch.
- [x] **T005-091** Implement remote MCP binding only under explicit network/egress/authenticated-endpoint/credential-scope/secret policy; strict-local denies external remote MCP. Immediately before each remote dispatch, revalidate the exact active binding/version/mapping and reject stale queued/prepared/cached/approved decisions rather than sending under superseded authority.
- [x] **T005-092** Prove MCP advertisements/nested calls cannot mint or widen Golam capabilities, set approvals, clear taint or directly mutate protected state; prove local mapping narrowing plus binding deprecation/revocation/replacement/version/digest/mapping mismatch invalidates queued calls, prepared-but-not-dispatched calls, cached mapped descriptors, cached capability/approval material and cached dispatch decisions.
- [x] **T005-093** Implement ACP adapter preserving authenticated local-client enrollment and scoped capability semantics without privileged KernelApi exposure.
- [x] **T005-094** Run malicious schema/payload/capability-spoof and dispatch-boundary adversarial protocol qualification, including skill and MCP activation/dispatch after deprecation, revocation, replacement, version mismatch, digest mismatch, mapping mismatch, stale queued/prepared/cached/approved state, and disconnect-during-effect.
- [x] **T005-095** Run exact-head Phase H qualification.

Phase H evidence: `implementation/mcp-json-source-foundry.md`, `implementation/phase-h-qualification-candidate.md`, phase-specific adversarial runs and subsequent exact-head ordinary CI.

## Phase I — Bounded browser/network tools and optional context decisions

- [x] **T005-100** Implement bounded HTTP/document fetch only if required by selected Spec 005 outcomes, binding method/origin/redirect/output limits/taint and explicit egress authority. Any credential-bearing hop requires authenticated encrypted endpoint identity, credential scope bound to the authorized origin/operation, and strip/revalidate/re-broker semantics on redirects/origin/protocol/proxy changes; downgrade or unprovable scope is denied. **Disposition: NOT_REQUIRED — selected Spec 005 outcomes do not require HTTP/document transport; no hidden HTTP client or remote fallback was admitted.**
- [x] **T005-101** Prove strict-local denial dominates browser/network selection and prevents remote fallback/telemetry/download widening.
- [x] **T005-102** Keep OS window/input/accessibility/screenshot-as-control behavior out of Spec 005 and fail closed rather than smuggling Spec 006 semantics into a browser tool.
- [x] **T005-103** Run representative L0 context evaluation and record whether a material structural-evidence gap exists.
- [x] **T005-104** If and only if T005-103 proves need, Source-Foundry-qualify exact Tree-sitter/LSP components and implement bounded L1 evidence. Otherwise record `L1=DEFER_NOT_NEEDED`. **Disposition: `L1=DEFER_NOT_NEEDED`.**
- [x] **T005-105** Evaluate whether dense/vector derivative search has reproducible value beyond local deterministic indexing. If not, record `DENSE_VECTOR_INDEX=DEFER_NOT_NEEDED`; if yes, create a separate exact Source Foundry admission gate before any dependency. **Disposition: `DENSE_VECTOR_INDEX=DEFER_NOT_NEEDED`.**
- [x] **T005-106** Run browser/network/context optionality qualification including credential forwarding, endpoint identity, origin/protocol redirects, downgrade denial and no hidden service dependency.

Phase I evidence: `implementation/phase-i-optionality-closeout.md` and `crates/golamd/tests/phase_i_network_optionality.rs`. The selected posture is `HTTP_TRANSPORT=NOT_REQUIRED`, `L1=DEFER_NOT_NEEDED`, `DENSE_VECTOR_INDEX=DEFER_NOT_NEEDED`, and OS window/input control remains outside Spec 005.

## Phase J — Core Alpha evidence and Spec 005 convergence

- [x] **T005-110** Execute a real local repository task using governed read/in-process-search/context plus an authorized file edit and deterministic verification; record exact evidence without claiming broader product parity.
- [x] **T005-111** Execute a strict-local end-to-end task with externally observed zero unauthorized egress.
- [x] **T005-112** Execute memory restart/user-edit/conflict/live-state-precedence/PREPARED/`UNKNOWN_OUTCOME`/creator-writer-effect-attribution/FORGET/REDACT scenarios against canonical storage and derivative rebuilds.
- [x] **T005-113** Execute malicious skill/MCP/memory/path/network-credential corpus proving no authority minting, taint clearing, secret redirect leakage, revoked-binding revival or protected-state bypass.
- [x] **T005-114** Run convergence across requirements/contracts/tasks/implementation/evidence and repair every material inconsistency forward-only.
- [x] **T005-115** Run focused qualification for every Spec 005 implementation boundary on the exact head.
- [x] **T005-116** Run full repository qualification locally/officially as permitted without fabricating platform evidence.
- [ ] **T005-117** Require exact-head Windows/macOS/Ubuntu CI success on the final implementation head.
- [ ] **T005-118** Obtain substantive independent semantic/security review on the unchanged exact implementation head after T005-117.
- [ ] **T005-119** Reconcile all material findings; any head mutation requires fresh exact-head CI and review.
- [ ] **T005-120** Transition the exact qualified implementation PR to non-Draft/Ready; use same-SHA lifecycle relay only if canonical precedent and live connector failure justify it, with relay CI + independent consistency review.
- [ ] **T005-121** Re-fetch live base/head and perform guarded expected-head merge only on the exact qualified head.
- [ ] **T005-122** Require push-triggered canonical-main CI success on the exact implementation merge SHA.
- [ ] **T005-123** Set `SPEC_005_IMPLEMENTATION_COMPLETE=YES` and `SPEC_005_CLOSED_CANONICAL=YES` only after T005-122.
- [ ] **T005-124** Re-read canonical main and program ordering. Only then enter the Core Alpha gate / bounded Spec 006 successor authorized by canonical governance.

T005-110..116 evidence: `implementation/phase-j-convergence-closeout.md`. Phase J focused run `34042118632` completed `SUCCESS`; pre-closeout full repository CI #1370 / run `34042120906` completed `SUCCESS` on `3ba511ab471f08eb39e5ac7602c593d1f81538b4` on Windows, macOS and Ubuntu. The closeout/ledger mutations intentionally make that CI stale for T005-117; final exact-head CI must run on the unchanged final documentation head.

---

## Non-waivable invariants

```text
TOOL_OR_PROTOCOL_OUTPUT != AUTHORITY_OR_EFFECT_COMMIT
PATH_STRING != TARGET_IDENTITY
PROTECTED_STATE != GENERIC_FILESYSTEM_RESOURCE
NATIVE_UNQUALIFIED != RUNNABLE_PROFILE
NATIVE_UNQUALIFIED != EXTERNAL_SEARCH_BINARY_AUTHORITY
PROCESS_CANCEL_REQUEST != PROCESS_TREE_TERMINAL_PROOF
MODEL_ASSERTION != MEMORY_PROMOTION_AUTHORITY
MEMORY_WRITER_ENABLEMENT_REQUIRES_PROMOTION_AUTHORITY_VALIDATOR
MEMORY_INTENT_TARGET_DIGEST_STORE_BINDING != OPTIONAL
MEMORY_CREATOR_IDENTITY != GOVERNED_WRITER_IDENTITY
MEMORY_VERSION_REQUIRES_MUTATION_EFFECT_ATTRIBUTION
SECRET_DERIVED != CANONICAL_LONG_TERM_MEMORY
SANITIZATION != DECLASSIFICATION_AUTHORITY
RETRIEVAL_SCORE != SOURCE_AUTHORITY
DERIVATIVE_INDEX != CANONICAL_MEMORY
DERIVATIVE_UNAVAILABLE != CANONICAL_MEMORY_UNAVAILABLE
EGRESS_ALLOWED != CREDENTIAL_DISCLOSURE_AUTHORIZED
PREPARED_MEMORY_EFFECT != MUTABLE_REQUEST
UNKNOWN_OUTCOME != SUCCESS
SKILL != AUTHORITY
STALE_SKILL_DISPATCH_BINDING != ACTIVE_AUTHORITY
MCP_CAPABILITY_ADVERTISEMENT != GOLAM_CAPABILITY
MCP_REVOKED_BINDING != ACTIVE_AUTHORITY
STALE_MCP_DISPATCH_BINDING != ACTIVE_AUTHORITY
ACP_CONNECTION != AUTHENTICATED_AUTHORITY
STRICT_LOCAL_FAILURE != CLOUD_OR_REMOTE_FALLBACK_PERMISSION
WAIVER_TAKEN=NO
```
