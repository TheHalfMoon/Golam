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
2. filesystem read + L0 context + Git read;
3. canonical managed memory + reconciliation;
4. filesystem mutations through Effect Gate;
5. production native-executor Source Foundry/containment qualification;
6. only then shell/process and executable MCP/skills;
7. browser/network bounded surfaces under egress;
8. adversarial convergence and closeout.

This avoids making production sandbox availability a blocker for all of Spec 005 while still respecting T052.

## Tool architecture

Every tool has a versioned `ToolDescriptor` with:

- stable tool identity/version;
- operation category;
- input/output byte/count/time bounds;
- required Golam capability class;
- read-only vs consequential Effect semantics;
- network posture;
- sandbox requirement;
- target identity rules;
- verifier/reconciliation requirements.

The harness/model produces only `ToolCallCandidate`. The dispatcher validates it into a `ToolRequest`; the Kernel/Effect Gate independently evaluates authority.

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

A production profile must define platform identity, executable/cwd identity, cleared environment, explicit env/handles/FS/network/device/resource rights, descendant supervision, timeout/cancel, secret injection/redaction, external no-egress observation and fail-closed unsupported platforms.

Command strings alone are not trusted parsed authority. If shell syntax is supported, parsing/command graph ambiguity and redirections/substitutions must be explicit evidence; no donor `skipApproval` behavior is permitted.

### Git

Read surface: status, diff, log, tree/blob identity and bounded content can feed context.

Write surface: add/write/commit/branch operations are effects and bind repository identity, expected current head/index/worktree state and post-operation verification. Force push, rebase/history rewrite and equivalent destructive operations remain outside ordinary authority.

### Browser/network

Spec 005 browser scope is document/web tool behavior, not OS computer control. Network operations bind URL/origin, redirect policy, method, target class, download/output limits, taint and egress authority. Strict-local external network denial is absolute.

## Context Compiler

Pipeline:

`intent -> evidence requirements -> source routing -> retrieve -> permission/authority/freshness/taint filter -> rank -> sufficiency -> replan -> ContextCapsule`

### L0

- user-selected files;
- bounded filesystem reads;
- bounded text search;
- Git status/diff/log/tree/blob identity;
- canonical Golam goals/evidence and permitted managed memory.

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

### Single writer

Golam-generated managed memory mutation flows through one writer transaction:

`candidate -> validate taint/provenance -> promotion authority -> read current version -> conflict/reconcile -> write temp -> durability boundary -> atomic replace -> record canonical version -> invalidate derivatives`

User hand-edits bypass the Golam writer by design and are detected via content/version mismatch. The next managed operation must reconcile rather than overwrite.

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

### Derived search/index

Start with deterministic local text/metadata indexing that is rebuildable. Dense/vector indexing remains deferred until representative evaluation proves need. No derivative service is a startup dependency for canonical memory.

## Skills and protocol adapters

### Agent Skills

Instruction-only skill packages use compatible `SKILL.md` concepts while adding Golam provenance/version/capability lifecycle metadata. Skill content is untrusted context. Executable scripts are disabled until production sandbox admission.

### MCP

MCP adapter maps protocol tools/resources/prompts to untrusted candidates/results. A server's advertised capability is descriptive only. Every actual protected operation remains mapped through Golam capability/effect policy.

Local MCP child process execution requires the production executor gate. Remote MCP requires explicit network/egress/identity/secrets policy and is disabled by strict-local unless the target qualifies as explicitly permitted local transport.

### ACP

ACP is an authenticated client interoperability adapter; it does not replace KernelApi or local IPC authentication.

## Source Foundry plan

1. L0 search exact implementation decision: selected ripgrep crates/binary or Golam-owned bounded implementation.
2. production native executor per platform/profile before process-backed tools.
3. MCP Rust SDK exact minimal surface if used.
4. Tree-sitter/LSP only after measured L0 gap.
5. derivative vector/index system only after measured need.

No source admission may bundle unrelated agent authority, channels, scheduler, learning or computer-control features.

## Verification strategy

Ordinary CI must remain hermetic: no model download, cloud credential, external service, Docker requirement or network dependency.

Required test families include:

- path/root/alias/symlink/reparse/junction/special-file/TOCTOU;
- protected-resource escape;
- effect durability/reconciliation/restart;
- process env/secret/descendant/network containment where admitted;
- strict-local external observation;
- Git stale-head/index/worktree expectations;
- context provenance/freshness/taint/ranking;
- memory candidate/promotion/conflict/reconciliation/user-edit/restart/disk-full;
- FORGET/REDACT derivative rebuild;
- malicious MCP/skill/protocol payloads and capability spoofing;
- exact-head full CI + independent semantic review.

## Planning closeout

Planning closes only after the complete planning package is internally consistent, exact-head Windows/macOS/Ubuntu CI is successful, a substantive independent external semantic review is clean on the same head, every material finding is repaired without waiver, and an expected-head guarded merge plus post-merge canonical-main CI succeeds.

Implementation begins from that exact post-merge canonical main only.
