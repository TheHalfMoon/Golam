# Spec 005 — Local Tools, Context & Memory

**Status**: PLANNING_ACTIVE  
**Canonical predecessor**: `main@390ea842837a7d85dca165d9291d5eb54c3f11db`  
**Owning program tasks**: Spec 001 `T050–T059`

## Purpose

Spec 005 turns Golam's trusted spine and model-independent harness into a useful local agent without expanding model authority. It introduces bounded local tool surfaces, evidence-oriented context compilation, user-owned governed memory, instruction-first skills, and untrusted MCP/ACP interoperability.

The core invariant is:

`TOOL_OR_PROTOCOL_OUTPUT != AUTHORITY_OR_EFFECT_COMMIT`

All consequential writes and external effects remain authorized by the existing Kernel/Effect Gate. Context ranking does not raise source authority. Memory promotion does not turn model output into truth.

## User outcomes

1. Golam can inspect a local project using bounded filesystem, search and Git evidence while preserving provenance, taint and live-state freshness.
2. Golam can propose and execute authorized local edits only through explicit capability/effect boundaries with durable evidence and post-action verification.
3. Golam can maintain durable user/project memory in human-readable Markdown with SQLite operational state, a single governed writer, and safe reconciliation of user hand-edits.
4. Golam can build context capsules from live evidence and memory without letting stale memory outrank authoritative current state.
5. Golam can consume Agent Skills-compatible instruction packages and MCP/ACP interoperability as untrusted inputs, never as authority.
6. Strict-local remains useful and externally verifiable with no hidden cloud fallback or unexpected egress.

## Scope

### Local tools

- bounded filesystem inspect/read/list/stat and authorized mutation operations;
- bounded text/search and Git read context;
- authorized Git mutation operations without force/history rewrite;
- bounded shell/process execution only after a production native-executor containment path is independently qualified;
- bounded browser/network tool behavior only through explicit network capability and egress policy;
- every consequential operation through the existing Effect Gate and durable evidence model.

### Context compiler

- L0: direct files, bounded text search, Git state/history and exact user-selected artifacts;
- L1: Tree-sitter/LSP structural evidence only when measured need justifies exact dependency admission;
- L2 graph/dataflow/runtime indexing is deferred unless reproducible evidence proves L0/L1 insufficient;
- every context item carries exact provenance, authority class, observed/version identity, taint, permission scope and content reference;
- sufficiency/replan is explicit; ranking is not authority.

### Memory

- managed Markdown is canonical human-readable long-lived knowledge;
- a dedicated memory SQLite database is canonical operational memory state, separate from the protected Effect Gate authority database;
- the existing Effect Gate PREPARED/terminal journal remains in the canonical authority database and no cross-store atomicity is assumed;
- search/vector/entity/graph caches are rebuildable derivatives;
- one Golam writer owns managed-vault mutation;
- user hand-edits remain allowed and are detected through content/version reconciliation, including commit-time digest/identity revalidation;
- Markdown/front matter is content only and cannot mint scope, taint, provenance, approval, authorization, managed version or Effect Gate authority;
- operations: `ADD`, `UPDATE`, `SUPERSEDE`, `CONTRADICT`, `MERGE`, `EXPIRE`, `FORGET`, `REDACT`;
- durable promotion requires attributable approval or deterministic verification against a pre-registered authoritative source/rule;
- every Golam-generated managed-memory mutation is a protected Effect Gate transaction with durable PREPARED intent before canonical mutation, terminal outcome/verification evidence, and fail-closed `UNKNOWN_OUTCOME` reconciliation across authority journal, Markdown and memory SQLite;
- `SECRET_DERIVED` content is rejected from canonical long-term memory and its taint cannot be downgraded within Spec 005;
- live repository/filesystem/device/authoritative external state outranks remembered content.

### Skills and protocols

- Agent Skills-compatible `SKILL.md` instruction packages with provenance, version locking and capability declaration;
- executable skill scripts remain disabled until the owning production sandbox profile is qualified;
- skill/MCP lifecycle and exact version are revalidated immediately before dispatch; `DEPRECATED`/`REVOKED`/replaced bindings invalidate queued calls and cached capability/approval decisions;
- MCP tools/resources/prompts are untrusted interoperability inputs; server-advertised capabilities do not mint Golam capabilities;
- MCP child processes require qualified production containment and explicit FS/network/environment/resource limits;
- ACP is a client/IDE interoperability boundary and must preserve authenticated local-client semantics.

## Hard boundaries

- Preserve the current seven-crate workspace initially. No empty `golam-tools`, `golam-memory`, or `golam-context` crate is created for architectural theater.
- `golam-kernel` owns authority, policy, capability and Effect-Gate decisions; it does not absorb tool/provider-specific semantics.
- `golam-effects` remains the only consequential-effect transaction boundary.
- `golam-core` may own pure tool/context/memory/protocol types and validation without privileged mutation authority.
- `golam-ledger` may persist durable tool/context/memory governance evidence and projections.
- `golamd` coordinates unprivileged tool/context/memory workflows under kernel decisions.
- The current Spec 003 native executor qualification is test-only Linux x86_64. `PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO`; no shell/process/MCP executable path may pretend otherwise.
- Generic filesystem authority never grants access to protected kernel resources.
- Path strings are not authority. Symlink/reparse/junction and path-race behavior must be handled fail-closed at the protected action boundary.
- No model/tool/plugin/MCP/skill may set `skipApproval`, self-authorize, mint capability material, weaken policy, clear taint, or directly commit protected state.
- Strict-local denial dominates routing and tool choice. Local failure never authorizes cloud, remote MCP, browser/network or telemetry fallback.
- Memory indexes, embeddings, Qdrant-class stores, Mem0-class systems and summaries are never canonical memory.
- No direct donor code/dependency admission occurs during planning.

## Functional requirements

- **FR-001** Tool descriptors MUST declare stable identity, operation class, explicit finite input/output byte/count bounds and duration bounds, required capability class, effect semantics, network posture, sandbox requirement, target-identity semantics and verifier/reconciliation expectations.
- **FR-002** Tool invocation MUST bind initiating principal, exact tool identity/version, requested operation/target, authorized resource class, capability/effect context, taint/provenance, idempotency material and current preconditions before consequential execution; a durably prepared protected request is immutable.
- **FR-003** Filesystem operations MUST resolve target identity within explicitly authorized roots and reject escapes through symlink/reparse/junction or equivalent aliases.
- **FR-004** Protected kernel resources MUST remain unreachable through generic filesystem or tool capability.
- **FR-005** Race-sensitive writes MUST either use platform primitives that preserve the checked target identity through commit or fail closed when identity cannot be maintained.
- **FR-006** Shell/process execution MUST remain unavailable until an exact production native-executor qualification is admitted for the requested platform/profile.
- **FR-007** Managed process launch MUST clear ambient environment, expose only explicitly authorized environment/handles/filesystem/network/resources, bind executable and cwd identity, supervise descendants, support cancellation, bind an explicit process-tree reconciliation policy, and preserve terminal evidence for the root and every descendant. Root exit, timeout or cancellation alone MUST NOT be treated as terminal success. Any surviving, unobservable or ownership-ambiguous descendant keeps the process effect failed/reconciling; when consequential completion is ambiguous it MUST remain `UNKNOWN_OUTCOME` until complete process-tree reconciliation succeeds.
- **FR-008** Git reads MAY feed context; Git writes MUST be consequential effects. Force push, destructive history rewrite and model-authorized bypass remain out of scope.
- **FR-009** Browser/network activity MUST require explicit egress authority and preserve URL/origin/redirect/download provenance. Credential-bearing hops MUST use authenticated encrypted transport, bind secrets to the authorized endpoint/origin, and strip/revalidate/re-broker sensitive material across redirects or endpoint/protocol changes. Strict-local MUST deny external network activity.
- **FR-010** Context evidence MUST retain source, observation identity/time, content digest/ref, authority class, taint and permission metadata.
- **FR-011** Context selection/ranking MUST NOT raise source authority or clear taint.
- **FR-012** Live authoritative state MUST supersede stale memory/context projections when they conflict.
- **FR-013** Every Golam-generated managed-memory mutation MUST use an immutable `MemoryMutationIntent` bound to the initiating principal, current Kernel authorization, applicable promotion approval/pre-registered verifier evidence, expected current versions, expected observed Markdown digest/identity and a unique effect identity. The authorized Effect Gate intent MUST be durably PREPARED in the existing protected authority database before the first canonical Markdown or memory-operational-SQLite mutation; the dedicated memory operational SQLite store remains separate and MUST bind the exact effect identity and intent digest. No cross-store atomicity may be assumed. Only the single governed memory writer/handler may execute the prepared mutation. Markdown replacement MUST revalidate the expected digest/version and target identity at the commit boundary and fail closed as `USER_EDIT_DETECTED`/`CONFLICT` when they changed or cannot be preserved. Integrity-chained terminal outcome plus required read-back/verification evidence across the authority journal, Markdown and memory SQLite MUST be recorded. Crash, disconnect or cross-store disagreement MUST remain `UNKNOWN_OUTCOME` and block dependent managed-memory mutations until reconciliation. This lifecycle applies to all operations, including multi-store `FORGET` and `REDACT`.
- **FR-014** User hand-edited Markdown MUST be detected and reconciled; Golam MUST NOT silently overwrite divergent user edits. Markdown body/front matter is content only; reserved authority-bearing fields MUST be rejected/quarantined for reconciliation and MUST NOT set scope, taint, provenance authority, approval, authorization, managed version identity or Effect Gate state.
- **FR-015** Memory promotion MUST require attributable approval or deterministic pre-registered authoritative verification; model/worker self-verification is invalid.
- **FR-016** `SECRET_DERIVED` content MUST NOT enter canonical long-term memory, and Spec 005 MUST NOT clear/downgrade `SECRET_DERIVED` provenance through redaction, summarization, transformation or verification.
- **FR-017** `FORGET`/`REDACT` MUST remove affected active canonical content and invalidate/rebuild derivatives while honestly retaining the fact that already-emitted external artifacts cannot be retroactively unseen; partial/ambiguous multi-store completion follows FR-013 reconciliation rather than being reported as success.
- **FR-018** Derived indexes MUST be rebuildable from canonical Markdown/operational evidence and may not become a hidden availability dependency. Canonical memory access MUST remain available when derivatives are absent; derivative-dependent operations rebuild or fail only that operation closed.
- **FR-019** Instruction-only skills MAY be admitted after provenance/capability review; executable scripts require qualified sandbox containment. Immediately before instruction activation or executable dispatch, the exact active reviewed skill version/content digest MUST be revalidated; deprecated/revoked/replaced versions invalidate queued/cached dispatch authority.
- **FR-020** MCP server output MUST be treated as untrusted/tainted input and server-advertised tools/resources MUST NOT mint Golam authority. Every MCP dispatch MUST revalidate the exact active reviewed binding/version and local mapping immediately before dispatch; deprecated/revoked/replaced bindings reject queued calls and cached capability/approval decisions.
- **FR-021** Remote MCP/network protocol use MUST pass normal egress, authenticated endpoint identity, credential-scope, secret and strict-local gates.
- **FR-022** ACP clients MUST authenticate through the existing local-client trust boundary; transport connection or localhost presence is insufficient.
- **FR-023** Ordinary CI MUST not require cloud credentials, external network, Docker, model downloads or specialized hardware.
- **FR-024** Every release-gating claim MUST be exact-head reproducible and independently reviewed according to live repository policy.

## Security/adversarial requirements

Qualification MUST include path alias/symlink races, protected-resource escape, stale refs, oversized/special files, shell metacharacter ambiguity, ambient secret leakage, descendant process escape, root-exit/cancellation with surviving or unobservable descendants, process-tree `UNKNOWN_OUTCOME` reconciliation, strict-local egress, credential-forwarding/transport-downgrade redirects, malicious MCP schemas/results, skill prompt injection, cached/queued skill or MCP dispatch after replacement/revocation, memory poisoning, stale-memory conflict, user-edit races including edit-after-check-before-replace, forged/stale promotion approval, malicious authority-bearing Markdown front matter, authority-journal/Markdown/memory-SQLite cross-store disagreement, `SECRET_DERIVED` promotion/taint-downgrade attempts, memory Effect Gate crash/`UNKNOWN_OUTCOME` reconciliation, FORGET/REDACT partial completion/rebuild correctness, and crash/restart during memory/tool transactions.

## Out of scope

- Desktop/computer control and OS UI automation (Spec 006);
- native mobile, GolamConnect and messaging channels (Spec 007);
- workers, scheduler and autonomous background learning (Spec 008+);
- public parity breadth and final release qualification (Specs 009–010);
- universal graph/dataflow indexing;
- making Docker, Qdrant, Mem0, a cloud vector DB, a remote MCP provider, Python or Node a strict-local core dependency;
- importing noncanonical PR #6–#8 proposals as authority.

## Success criteria

- **SC-001** A real repository task can read/search/inspect and perform an authorized edit with exact provenance, durable effect evidence and deterministic verification.
- **SC-002** A strict-local task completes without externally observed unauthorized egress.
- **SC-003** Path escape and protected-resource attack corpus fails closed across supported platforms.
- **SC-004** Shell/process/MCP executable features remain unavailable on unqualified production sandbox profiles and become usable only after exact admitted qualification.
- **SC-005** Memory survives restart, reconciles a user hand-edit, surfaces contradictions, and never lets stale memory outrank fresher authoritative state.
- **SC-006** Managed-memory mutations prove authority-database PREPARED-before-mutation evidence, conditional Markdown commit-time precondition enforcement, memory-SQLite/effect binding, terminal cross-store verification and `UNKNOWN_OUTCOME` reconciliation; FORGET/REDACT removes active canonical knowledge and rebuilds every enabled derivative deterministically.
- **SC-007** Malicious skill/MCP/memory inputs cannot mint authority, clear taint or bypass Effect Gate, and stale/revoked protocol bindings cannot dispatch queued/cached calls.
- **SC-008** final exact-head Windows/macOS/Ubuntu CI and a substantive independent semantic review are clean before merge.
