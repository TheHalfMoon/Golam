# Tasks — Spec 005 Local Tools, Context & Memory

**Canonical planning base**: `main@390ea842837a7d85dca165d9291d5eb54c3f11db`  
**Planning branch**: `spec/005-local-tools-context-memory`

Execute strictly in dependency order. A checked planning task means the artifact is present in the atomic planning candidate; it does not waive the later exact-head/review/merge gates.

---

## Phase A — Planning bootstrap and authority

- [x] **T005-001** Re-read live canonical `main`, constitution, Spec 001 T050–T059, Spec 003 production-executor evidence and Spec 004 closeout before defining scope.
- [x] **T005-002** Confirm Spec 004 is `CLOSED_CANONICAL` from live merge and push-triggered main CI, and confirm noncanonical PRs #6–#8 are not implementation predecessors.
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

- [ ] **T005-014** Run exact-head planning CI on Windows, macOS and Ubuntu with the repository's complete required workflow set.
- [ ] **T005-015** Obtain a substantive independent semantic review on the unchanged exact planning head after T005-014. Status-only/billing/rate-limit/self-review output is insufficient.
- [ ] **T005-016** Repair every material planning/review finding forward-only; any branch mutation invalidates prior exact-head CI/review and requires requalification.
- [ ] **T005-017** Converge the exact planning head with no unresolved material review threads, no waiver and no stale evidence.
- [ ] **T005-018** Transition the qualified planning PR to non-Draft/Ready. If the connector lifecycle transition is unavailable, only a previously canonicalized same-SHA/no-content-delta lifecycle relay may be used, with its own CI and independent consistency review.
- [ ] **T005-019** Re-fetch live base/head immediately before a guarded expected-head planning merge and merge only the exact qualified head.
- [ ] **T005-020** Require push-triggered canonical-main CI to succeed on the exact planning merge SHA.
- [ ] **T005-021** Set `SPEC_005_PLANNING_COMPLETE=YES` and `SPEC_005_PLANNING_CLOSED_CANONICAL=YES` only after T005-020.
- [ ] **T005-022** Re-read canonical main after planning closeout and create the implementation branch from that exact verified main. Do not begin implementation before this task.

---

## Phase C — Core tool/context/memory contracts in Rust

- [ ] **T005-025** Implement pure versioned `ToolId`, `ToolVersion`, `ToolIoBounds`, `ToolDurationBounds`, `ToolReconciliationPolicy`, `ToolVerificationPolicy`, `ToolDescriptor`, operation/network/sandbox enums and deterministic validation in `golam-core` without privileged state. Bounds applicable to executable/read operations must be explicit and finite; “unbounded by omission” is invalid.
- [ ] **T005-026** Implement immutable `ToolRequest`, `ToolResult`, candidate binding and deterministic serialization/digest semantics. Once a protected request is durably prepared, target/operation/precondition/authority changes require a new request/effect identity.
- [ ] **T005-027** Implement pure authorized-root, resolved-target-identity and file-mutation-expectation types with platform-neutral invariants.
- [ ] **T005-028** Implement `ContextEvidence`, `EvidenceRequirement`, `ContextCapsule` and sufficiency/freshness/authority/taint validation types.
- [ ] **T005-029** Implement memory candidate/item/version/operation/reconciliation/derivative-generation types plus `MemoryMutationIntent`/terminal outcome types with deterministic validation and no authority-bearing free-form fields.
- [ ] **T005-030** Implement skill/protocol descriptor/binding types that cannot encode authority-bearing capability material.
- [ ] **T005-031** Extend durable ledger evidence for tool requests/results and context provenance without making projections canonical authority.
- [ ] **T005-032** Extend durable memory governance/version/reconciliation/promotion/effect evidence with required integrity behavior for security-critical records, including PREPARED intent and terminal/`UNKNOWN_OUTCOME` evidence identities.
- [ ] **T005-033** Add focused property/adversarial tests proving invalid descriptors/requests/candidates cannot mint authority, clear taint, encode unbounded operations or mutate a durably prepared protected request.
- [ ] **T005-034** Run focused exact-head qualification for Phase C before proceeding.

## Phase D — Bounded filesystem reads and L0 context

- [ ] **T005-035** Implement platform-aware authorized-root and target-resolution service for read-only operations, including symlink/reparse/junction/protected-resource exclusion evidence.
- [ ] **T005-036** Implement bounded regular-file stat/read with size/type/permission limits and failure-preserving behavior.
- [ ] **T005-037** Implement bounded directory list/walk with count/depth/time limits and deterministic ordering where required.
- [ ] **T005-038** Qualify exact L0 text-search implementation choice through Source Foundry before adding a crate/binary/dependency.
- [ ] **T005-039** Implement bounded text search under authorized roots with exact match-file/line/content provenance and no shell interpolation.
- [ ] **T005-040** Implement bounded Git read evidence: repository identity, HEAD/ref, status, diff, log/tree/blob observation without mutation authority.
- [ ] **T005-041** Implement the L0 Context Compiler pipeline over user-selected files, file reads, search, Git, canonical goal/evidence and permitted memory.
- [ ] **T005-042** Implement permission/authority/freshness/taint filtering and prove retrieval score cannot raise authority or clear taint.
- [ ] **T005-043** Implement explicit sufficiency/missing-requirement output and bounded replan without unbounded recursive retrieval.
- [ ] **T005-044** Run path/read/search/Git/context adversarial qualification and exact-head Phase D CI.

## Phase E — Canonical managed memory

T005-048 depends explicitly on T005-031..032 and T005-045..047. T005-049..058 depend on the durable mutation lifecycle established by T005-048 unless a task is strictly read-only.

- [ ] **T005-045** Freeze exact managed-vault on-disk Markdown and SQLite operational schema from the planning model without exposing protected control state as generic memory files.
- [ ] **T005-046** Implement canonical Markdown parser/serializer with stable item/version identity and bounded front-matter/content handling.
- [ ] **T005-047** Implement SQLite operational state for versions, promotion evidence, reconciliation, conflicts, supersession, derivative generations and effect-owned mutation state.
- [ ] **T005-048** Implement the single governed memory writer as an Effect Gate handler. Every Golam-generated `MemoryMutationIntent` must bind initiating principal, current Kernel authorization, applicable approval/pre-registered verifier evidence, expected current versions and unique effect identity; persist authorized PREPARED intent before the first canonical Markdown/SQLite mutation; perform durability/atomic replacement and operational updates; invalidate affected derivatives; then record integrity-chained terminal outcome plus required read-back/verification evidence. Ambiguous completion remains `UNKNOWN_OUTCOME` and blocks dependent managed-memory mutation until reconciliation.
- [ ] **T005-049** Implement restart reconciliation for every PREPARED/writer/Markdown/SQLite/terminal durability cut, including file-without-record, record-without-readable-file and `UNKNOWN_OUTCOME` cases; absence of terminal evidence is never success.
- [ ] **T005-050** Implement user hand-edit detection and fail-closed reconciliation without silent overwrite.
- [ ] **T005-051** Implement attributable human-promotion authority validation and deterministic pre-registered verifier promotion; reject free-form/model self-approval and stale/revoked authorization.
- [ ] **T005-052** Enforce monotonic `SECRET_DERIVED` rejection at the canonical memory admission boundary; redaction/summarization/transformation/verification cannot downgrade the taint. Only independently sourced non-secret provenance may form a separate candidate.
- [ ] **T005-053** Implement `ADD`, `UPDATE`, `SUPERSEDE`, `CONTRADICT`, `MERGE`, and `EXPIRE` with immutable version lineage through T005-048's prepared-effect lifecycle.
- [ ] **T005-054** Implement `FORGET` and `REDACT` through the same T005-048 effect lifecycle with active canonical-content removal, bounded non-plaintext audit facts, Markdown/SQLite/derivative reconciliation and explicit external-artifact honesty; partial multi-store completion cannot become success.
- [ ] **T005-055** Implement deterministic local derivative text/metadata index generation that is discardable and rebuildable from canonical memory.
- [ ] **T005-056** Ensure missing/corrupt derivatives do not block canonical memory startup/access. Derivative-dependent operations trigger governed rebuild from canonical state and fail only that dependent operation closed if rebuild/qualification cannot complete.
- [ ] **T005-057** Prove live authoritative repository/filesystem state outranks stale memory and surfaces conflict evidence.
- [ ] **T005-058** Run memory-poisoning, forged/stale-promotion, secret-derived/taint-downgrade, user-edit-race, disk-full/crash, PREPARED/terminal/`UNKNOWN_OUTCOME`, dependent-mutation blocking, stale-memory and FORGET/REDACT partial-completion/resurrection qualification.

## Phase F — Consequential filesystem/Git mutations

- [ ] **T005-060** Implement identity-preserving file create/write/replace through the existing Effect Gate with expected-parent/target/content preconditions and read-back verification.
- [ ] **T005-061** Implement governed rename/delete operations with explicit target-vs-parent authority and stale-identity denial.
- [ ] **T005-062** Add symlink/reparse/junction/rename-swap TOCTOU adversarial harnesses for mutation boundaries on supported platforms.
- [ ] **T005-063** Implement bounded Git add/commit/branch mutation as Effect Gate operations bound to expected repository HEAD/index/worktree state.
- [ ] **T005-064** Keep force push, force ref movement, rebase/shared-history rewrite and equivalent destructive operations outside ordinary tool authority.
- [ ] **T005-065** Implement deterministic post-operation verification and reconciliation for ambiguous filesystem/Git effects using existing effect semantics.
- [ ] **T005-066** Prove generic file/Git tool capability cannot reach protected Golam resources or rewrite policy/lease/approval/secret/effect/audit state.
- [ ] **T005-067** Run restart/UNKNOWN_OUTCOME/idempotency/stale-state adversarial qualification for mutations.
- [ ] **T005-068** Run exact-head Phase F qualification.

## Phase G — Production native containment and process tools

- [ ] **T005-070** Re-read canonical Spec 003 sandbox/executor evidence and freeze exact production containment requirements per target platform before selecting an implementation.
- [ ] **T005-071** Research and Source-Foundry-qualify exact production native containment primitives/dependencies for the first supported platform; do not infer cross-platform equivalence.
- [ ] **T005-072** Implement the first production containment profile with cleared ambient environment, explicit FS/network/device/resource rights, executable/cwd identity, handle rules, descendant supervision and cancellation.
- [ ] **T005-073** Integrate secret brokerage/unbrokerable fallback and value-aware redaction into process launch evidence without argv/ambient-secret leakage.
- [ ] **T005-074** Qualify strict-local no-network behavior through an external descendant-aware observation on the admitted production profile.
- [ ] **T005-075** Qualify filesystem/namespace/OS containment claims only to the exact proven platform boundary; unsupported claims remain explicit denial states.
- [ ] **T005-076** Add hostile payload corpus for process-tree escape, environment leakage, descriptor/handle inheritance, forbidden filesystem/device access, timeout/cancel and descendant persistence.
- [ ] **T005-077** Mark the exact profile `ADMITTED` only after focused + repository CI evidence and independent semantic/security review are clean.
- [ ] **T005-078** Implement governed argv-style process execution through ToolRequest + Kernel/Effect Gate + admitted executor.
- [ ] **T005-079** If shell syntax is selected, implement explicit parse/ambiguity/redirection/substitution evidence and deny ambiguous command graphs; never implement donor `skipApproval` semantics.
- [ ] **T005-080** Requalify strict-local, secret isolation, UNKNOWN_OUTCOME/restart and descendant supervision on the exact process-tool head.

## Phase H — Skills, MCP and ACP

- [ ] **T005-085** Implement Agent Skills-compatible instruction package discovery/provenance/version-lock validation with content treated as untrusted context.
- [ ] **T005-086** Implement governed skill lifecycle; instruction-only admission is independent of executable admission.
- [ ] **T005-087** Keep executable skill scripts disabled on unqualified profiles; when enabled, route through the admitted process/tool/effect boundary.
- [ ] **T005-088** Qualify the exact MCP implementation strategy; if using official Rust SDK, select minimal exact crates/features/transitive/network/process closure before dependency admission.
- [ ] **T005-089** Implement MCP descriptor/resource/prompt normalization into untrusted Golam protocol/tool types with bounded parsing and stable server/version identity.
- [ ] **T005-090** Implement local MCP binding/launch only through an admitted production containment profile and current policy/capability/effect authority.
- [ ] **T005-091** Implement remote MCP binding only under explicit network/egress/authenticated-endpoint/credential-scope/secret policy; strict-local denies external remote MCP.
- [ ] **T005-092** Prove MCP advertisements/nested calls cannot mint or widen Golam capabilities, set approvals, clear taint or directly mutate protected state.
- [ ] **T005-093** Implement ACP adapter preserving authenticated local-client enrollment and scoped capability semantics without privileged KernelApi exposure.
- [ ] **T005-094** Run malicious schema/payload/capability-spoof/version-replacement/disconnect-during-effect adversarial protocol qualification.
- [ ] **T005-095** Run exact-head Phase H qualification.

## Phase I — Bounded browser/network tools and optional context decisions

- [ ] **T005-100** Implement bounded HTTP/document fetch only if required by selected Spec 005 outcomes, binding method/origin/redirect/output limits/taint and explicit egress authority. Any credential-bearing hop requires authenticated encrypted endpoint identity, credential scope bound to the authorized origin/operation, and strip/revalidate/re-broker semantics on redirects/origin/protocol/proxy changes; downgrade or unprovable scope is denied.
- [ ] **T005-101** Prove strict-local denial dominates browser/network selection and prevents remote fallback/telemetry/download widening.
- [ ] **T005-102** Keep OS window/input/accessibility/screenshot-as-control behavior out of Spec 005 and fail closed rather than smuggling Spec 006 semantics into a browser tool.
- [ ] **T005-103** Run representative L0 context evaluation and record whether a material structural-evidence gap exists.
- [ ] **T005-104** If and only if T005-103 proves need, Source-Foundry-qualify exact Tree-sitter/LSP components and implement bounded L1 evidence. Otherwise record `L1=DEFER_NOT_NEEDED`.
- [ ] **T005-105** Evaluate whether dense/vector derivative search has reproducible value beyond local deterministic indexing. If not, record `DENSE_VECTOR_INDEX=DEFER_NOT_NEEDED`; if yes, create a separate exact Source Foundry admission gate before any dependency.
- [ ] **T005-106** Run browser/network/context optionality qualification including credential forwarding, endpoint identity, origin/protocol redirects, downgrade denial and no hidden service dependency.

## Phase J — Core Alpha evidence and Spec 005 convergence

- [ ] **T005-110** Execute a real local repository task using governed read/search/context plus an authorized file edit and deterministic verification; record exact evidence without claiming broader product parity.
- [ ] **T005-111** Execute a strict-local end-to-end task with externally observed zero unauthorized egress.
- [ ] **T005-112** Execute memory restart/user-edit/conflict/live-state-precedence/PREPARED/`UNKNOWN_OUTCOME`/FORGET/REDACT scenarios against canonical storage and derivative rebuilds.
- [ ] **T005-113** Execute malicious skill/MCP/memory/path/network-credential corpus proving no authority minting, taint clearing, secret redirect leakage or protected-state bypass.
- [ ] **T005-114** Run convergence across requirements/contracts/tasks/implementation/evidence and repair every material inconsistency forward-only.
- [ ] **T005-115** Run focused qualification for every Spec 005 implementation boundary on the exact head.
- [ ] **T005-116** Run full repository qualification locally/officially as permitted without fabricating platform evidence.
- [ ] **T005-117** Require exact-head Windows/macOS/Ubuntu CI success on the final implementation head.
- [ ] **T005-118** Obtain substantive independent semantic/security review on the unchanged exact implementation head after T005-117.
- [ ] **T005-119** Reconcile all material findings; any head mutation requires fresh exact-head CI and review.
- [ ] **T005-120** Transition the exact qualified implementation PR to non-Draft/Ready; use same-SHA lifecycle relay only if canonical precedent and live connector failure justify it, with relay CI + independent consistency review.
- [ ] **T005-121** Re-fetch live base/head and perform guarded expected-head merge only on the exact qualified head.
- [ ] **T005-122** Require push-triggered canonical-main CI success on the exact implementation merge SHA.
- [ ] **T005-123** Set `SPEC_005_IMPLEMENTATION_COMPLETE=YES` and `SPEC_005_CLOSED_CANONICAL=YES` only after T005-122.
- [ ] **T005-124** Re-read canonical main and program ordering. Only then enter the Core Alpha gate / bounded Spec 006 successor authorized by canonical governance.

---

## Non-waivable invariants

```text
TOOL_OR_PROTOCOL_OUTPUT != AUTHORITY_OR_EFFECT_COMMIT
PATH_STRING != TARGET_IDENTITY
PROTECTED_STATE != GENERIC_FILESYSTEM_RESOURCE
NATIVE_UNQUALIFIED != RUNNABLE_PROFILE
MODEL_ASSERTION != MEMORY_PROMOTION_AUTHORITY
SECRET_DERIVED != CANONICAL_LONG_TERM_MEMORY
SANITIZATION != DECLASSIFICATION_AUTHORITY
RETRIEVAL_SCORE != SOURCE_AUTHORITY
DERIVATIVE_INDEX != CANONICAL_MEMORY
DERIVATIVE_UNAVAILABLE != CANONICAL_MEMORY_UNAVAILABLE
EGRESS_ALLOWED != CREDENTIAL_DISCLOSURE_AUTHORIZED
PREPARED_MEMORY_EFFECT != MUTABLE_REQUEST
UNKNOWN_OUTCOME != SUCCESS
SKILL != AUTHORITY
MCP_CAPABILITY_ADVERTISEMENT != GOLAM_CAPABILITY
ACP_CONNECTION != AUTHENTICATED_AUTHORITY
STRICT_LOCAL_FAILURE != CLOUD_OR_REMOTE_FALLBACK_PERMISSION
WAIVER_TAKEN=NO
```
