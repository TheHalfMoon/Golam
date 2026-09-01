# Plan — Spec 005 Local Tools, Context & Memory

**Branch**: `spec/005-local-tools-context-memory`  
**Canonical planning base**: `main@390ea842837a7d85dca165d9291d5eb54c3f11db`

## Architecture strategy

Spec 005 extends the current seven-crate workspace without creating empty architectural crates. Split only after implementation evidence proves an independent ownership/testing boundary.

### Ownership

- `golam-core`: pure tool descriptors/requests/results, path target identities, context evidence/capsule types, memory candidate/operation/version types, skill/protocol descriptors and deterministic validation.
- `golam-kernel`: policy/capability/effect authorization only. It does not contain filesystem/search/Git/MCP/provider implementation semantics.
- `golam-effects`: consequential filesystem/Git/process/browser/memory protected mutations remain effect transactions with idempotency/reconciliation semantics.
- `golam-ledger`: durable tool intent/result evidence, context provenance references, memory operational/version/reconciliation/promotion evidence and projections.
- `golamd`: unprivileged coordination of tool execution, context compilation, memory services, skills and protocol adapters under kernel decisions.
- existing CLI app: inspectable user-facing commands only as needed for qualification; no separate authority surface.

## Sequencing principle

Deliver useful safe local behavior before broad execution:

1. pure contracts/state and evidence;
2. filesystem read + **in-process** L0 text search + L0 context + Git read;
3. canonical managed memory + reconciliation;
4. filesystem mutations through Effect Gate;
5. production native-executor Source Foundry/containment qualification;
6. only then process-backed tools, shell, external search binaries and executable MCP/skills;
7. browser/network bounded surfaces under egress;
8. adversarial convergence and closeout.

This avoids making production sandbox availability a blocker for all of Spec 005 while still respecting T052. It also prevents a “utility binary” exception from bypassing the same production process-containment gate that applies to every other child process.

## Tool architecture

Every tool has a versioned `ToolDescriptor` with:

- stable tool identity/version;
- operation category;
- explicit finite input/output byte/count bounds and duration bounds;
- required Golam capability class;
- read-only vs consequential Effect semantics;
- network posture;
- sandbox requirement;
- target identity rules;
- verifier/reconciliation requirements.

The harness/model produces only `ToolCallCandidate`. The dispatcher validates it into an immutable bounded `ToolRequest`; the Kernel/Effect Gate independently evaluates authority. Once a protected request is durably prepared, materially changed target/operation/preconditions/authority require a new request/effect identity.

### Filesystem target resolution

Protected actions use a two-stage model:

`requested path -> bounded lexical normalization -> platform-aware resolved target identity -> policy/effect check -> identity-preserving action`

Requirements:

- allowed roots are explicit;
- symlinks/reparse points/junctions and mount/alias behavior are detected;
- protected Golam state remains excluded regardless of path root;
- special files are denied unless a later explicit capability supports them;
- read size/count/depth limits are mandatory;
- race-sensitive writes use identity-preserving handles/primitives where supported or fail closed;
- rename/delete/create semantics distinguish target-vs-parent authority;
- failures preserve user data and do not destructively consume rejected input.

### Process/shell

The canonical production capability remains `native:unqualified`. The first process implementation task is therefore **qualification**, not launch feature enablement.

A production profile must define platform identity, executable/cwd identity, cleared environment, explicit env/handles/FS/network/device/resource rights, descendant supervision, process-tree ownership/discovery, timeout/cancel, terminal descendant observation/reconciliation, secret injection/redaction, external no-egress observation and fail-closed unsupported platforms.

A cancellation request is not proof that the root or descendants terminated. The launch plan and terminal evidence must independently bind process-tree reconciliation and unresolved-descendant behavior.

Command strings alone are not trusted parsed authority. If shell syntax is supported, parsing/command graph ambiguity and redirections/substitutions must be explicit evidence; no donor `skipApproval` behavior is permitted.

### Git

Read surface: status, diff, log, tree/blob identity and bounded content can feed context.

Write surface: add/write/commit/branch operations are effects and bind repository identity, expected current head/index/worktree state and post-operation verification. Force push, rebase/history rewrite and equivalent destructive operations remain outside ordinary authority.

### Browser/network

Spec 005 browser scope is document/web tool behavior, not OS computer control. Network operations bind URL/origin, redirect policy, method, target class, download/output limits, taint and egress authority. Strict-local external network denial is absolute.

Credential-bearing network hops add a separate disclosure gate: authenticated encrypted endpoint transport, exact endpoint/origin credential scope, and no automatic forwarding of credentials or secret-bearing bodies across redirects/origin/protocol changes. Every changed hop strips sensitive material, revalidates egress + endpoint identity + credential scope, and re-brokers only when explicitly authorized; downgrade/unprovable scope is denied.

## Context Compiler

Pipeline:

`intent -> evidence requirements -> source routing -> retrieve -> permission/authority/freshness/taint filter -> rank -> sufficiency -> replan -> ContextCapsule`

### L0

- user-selected files;
- bounded filesystem reads;
- bounded **in-process** text search while production native execution remains unadmitted;
- Git status/diff/log/tree/blob identity;
- canonical Golam goals/evidence and permitted managed memory.

The Phase D baseline MUST NOT spawn an external search binary. It may use a Golam-owned bounded implementation or exactly admitted Rust crates executing in-process. A pinned ripgrep executable remains a later process-backed option only after an exact production containment profile reaches `ADMITTED`; it must then be launched through the admitted process/tool boundary and receive its own exact Source Foundry qualification.

### L1

Tree-sitter/LSP structural evidence is conditional. Before adding a dependency, run representative L0 tasks and record the missing evidence class and expected measurable benefit.

### L2

Graph/dataflow/vector/runtime layers are deferred.

`RETRIEVAL_SCORE != SOURCE_AUTHORITY`

Each evidence item carries exact source identity, observed/version info, digest/ref, authority class, taint, permission scope and freshness semantics. A capsule is a projection and never replaces canonical source evidence.

## Memory architecture

### Canonical layout

Human-readable managed Markdown is canonical durable knowledge. SQLite records operational metadata, version identity, reconciliation state, promotion decisions, conflict/supersession relationships and derivative-index state.

A possible managed vault layout is conceptual, not frozen path naming:

```text
memory/
  user/
  projects/<project-id>/
  managed manifests/version metadata
```

Protected operational state must not be exposed as generic user memory files.

### Single writer + durable effect lifecycle

Every Golam-generated managed-memory mutation is consequential. It starts as an immutable `MemoryMutationIntent` bound to current Kernel authorization, applicable approval/pre-registered verifier evidence, expected current versions, the exact observed canonical Markdown target identity, expected Markdown digest/version, the exact dedicated memory-operational-SQLite store binding, and a unique effect identity. Promotion-authority validation is implemented and qualified **before** the writer may be enabled for mutation. The Effect Gate PREPARED record is durable before the first Markdown/SQLite canonical mutation, and all of those intent bindings survive PREPARED unchanged.

Markdown body/front matter is content only. Reserved fields may not set scope, taint, provenance authority, approval, authorization, managed version identity, promotion state or Effect Gate state; such input enters explicit `USER_EDIT_DETECTED`/`CONFLICT` reconciliation/quarantine rather than becoming authority.

Only the single governed memory writer executes the prepared intent:

`candidate -> validate taint/provenance -> current authorization + promotion authority -> expected-version/user-edit check -> durable PREPARED Effect -> write temp -> durability boundary -> immediately revalidate expected Markdown digest/version + target identity at commit time -> conditional compare-and-replace / identity-preserving Markdown replace (fail closed as USER_EDIT_DETECTED/CONFLICT on changed content/identity or unpreservable identity) -> SQLite operational/version update bound to the exact memory store + effect/intent digest -> invalidate derivatives -> cross-store read-back/reconciliation -> integrity-chained terminal outcome`

Every committed `MemoryVersion` preserves the initiating/creating principal, the governed writer identity and the exact mutation Effect reference. Restart reconciliation must retain these separate identities rather than collapsing attribution into a generic system actor.

User hand-edits bypass the Golam writer by design and are detected via content/version mismatch. The next managed operation must reconcile rather than overwrite. The governed commit boundary revalidates the expected observed digest/version and target identity immediately before replacement; an intervening edit or identity change MUST NOT be overwritten.

The authority Effect Gate journal, canonical Markdown, and dedicated memory operational SQLite store are separate evidence surfaces. No cross-store atomic transaction is assumed. Terminal success requires read-back agreement across all three on the exact effect identity, mutation-intent digest, expected/committed Markdown identity+digest+version, and memory-store binding. A file-without-row, row-without-file, wrong store binding, unreadable store, stale digest/identity, or other partial cut remains failed/reconciling or `UNKNOWN_OUTCOME` until deterministic reconciliation resolves it.

Crash/disconnect ambiguity remains `UNKNOWN_OUTCOME`; dependent managed-memory mutations are blocked until restart reconciliation determines exact canonical state. `FORGET`/`REDACT` use the same lifecycle across Markdown, SQLite and derivative invalidation; partial multi-store completion is never silently reported as success.

### Memory operations

- `ADD`: new proposition/evidence with scope/provenance;
- `UPDATE`: new version preserving predecessor relation;
- `SUPERSEDE`: old item remains historical but inactive;
- `CONTRADICT`: retain both with surfaced conflict state;
- `MERGE`: create a new attributed synthesis; do not erase source lineage;
- `EXPIRE`: mark inactive under time/policy rule;
- `FORGET`: remove active canonical knowledge as policy permits and rebuild derivatives;
- `REDACT`: remove prohibited sensitive content while retaining necessary non-plaintext audit facts.

`MEMORY_CANDIDATE != DURABLE_MEMORY`

`SECRET_DERIVED` provenance is monotonic within Spec 005. Redaction/summarization/transformation does not create declassification authority; only independently sourced non-secret evidence begins a separate eligible provenance chain.

### Derived search/index

Start with deterministic local text/metadata indexing that is rebuildable. Dense/vector indexing remains deferred until representative evaluation proves need. No derivative service is a startup dependency for canonical memory. Canonical memory access proceeds when a derivative is missing; only a derivative-dependent operation must rebuild it or fail that operation closed.

## Skills and protocol adapters

### Agent Skills

Instruction-only skill packages use compatible `SKILL.md` concepts while adding Golam provenance/version/capability lifecycle metadata. Skill content is untrusted context. Executable scripts are disabled until production sandbox admission.

Queued, prepared-but-not-dispatched, cached capability/approval and dispatch-decision state is scoped to the exact reviewed skill package/version/content digest and reviewed capability mapping. Immediately before every instruction activation or executable dispatch, Golam revalidates that exact binding and current lifecycle state. `DEPRECATED`, `REVOKED`, replaced, unknown, version/digest/mapping-mismatched state rejects the dispatch and invalidates stale queued/cached authority; a replacement requires fresh review and authority evaluation.

### MCP

MCP adapter maps protocol tools/resources/prompts to untrusted candidates/results. A server's advertised capability is descriptive only. Every actual protected operation remains mapped through Golam capability/effect policy.

Every MCP dispatch is bound to the exact reviewed `McpServerBinding` identity/digest, version lock and Golam-local mapping identity/digest. Immediately before local launch or remote request dispatch, Golam revalidates that exact active binding and mapping. `DEPRECATED`, `REVOKED`, replaced, unreviewed, unknown or mismatched state rejects queued/prepared calls, cached mapped descriptors, cached capability/approval decisions and stale dispatch decisions; superseded bindings cannot donate authority to replacements.

Local MCP child process execution requires the production executor gate. Remote MCP requires explicit network/egress/identity/secrets policy and is disabled by strict-local unless the target qualifies as explicitly permitted local transport.

### ACP

ACP is an authenticated client interoperability adapter; it does not replace KernelApi or local IPC authentication.

## Source Foundry plan

1. Phase D L0 search: qualify only a Golam-owned bounded implementation or exact Rust crate surface that executes in-process; no external binary may be admitted/launched while production remains `native:unqualified`.
2. qualify the production native executor per exact platform/profile before any process-backed tool.
3. only after an exact profile is `ADMITTED`, optionally qualify a pinned external search binary (including executable identity, process/sandbox/resource/network closure) and launch it through the admitted process boundary; otherwise record the binary path `NOT_APPLICABLE`.
4. qualify the exact MCP implementation dependency only if MCP implementation requires it.
5. qualify Tree-sitter/LSP only after measured L0 gap.
6. qualify any derivative memory index only after measured need; canonical memory remains independent.

No source admission may bundle unrelated agent authority, channels, scheduler, learning or computer-control features.

## Verification strategy

Ordinary CI must remain hermetic: no model download, cloud credential, external service, Docker requirement or network dependency.

Required test families include:

- path/root/alias/symlink/reparse/junction/special-file/TOCTOU;
- protected-resource escape;
- effect durability/reconciliation/restart;
- process env/secret/descendant/network containment where admitted;
- process-tree terminal reconciliation and unresolved-descendant behavior;
- strict-local external observation;
- credential-bearing redirect/origin/protocol transition and downgrade denial;
- Git stale-head/index/worktree expectations;
- context provenance/freshness/taint/ranking;
- memory candidate/promotion/conflict/reconciliation/user-edit/restart/disk-full;
- memory creator/writer/effect attribution through restart reconciliation;
- managed-memory PREPARED-before-mutation, exact target/digest/store bindings, commit-time conditional compare-and-replace, content-only front matter, terminal outcome, `UNKNOWN_OUTCOME` and dependent-mutation blocking;
- authority-journal/Markdown/memory-SQLite partial cuts including stale digest, target swap, wrong store binding, file-without-row and row-without-file;
- FORGET/REDACT partial multi-store completion + derivative rebuild;
- malicious MCP/skill/protocol payloads and capability spoofing;
- stale queued/prepared/cached/approved skill dispatch after version/digest/mapping replacement or revocation;
- stale queued/prepared/cached/approved MCP dispatch after binding/version/mapping replacement or revocation;
- Phase D proof that no external search process can launch while `native:unqualified`;
- exact-head full CI + independent semantic review.

## Planning closeout

Planning closes only after the complete planning package is internally consistent, exact-head Windows/macOS/Ubuntu CI is successful, a substantive independent external semantic review is clean on the same head, every material finding is repaired without waiver, and an expected-head guarded merge plus post-merge canonical-main CI succeeds.

Implementation begins from that exact post-merge canonical main only.
