# Research — Spec 005 Local Tools, Context & Memory

**Observed**: 2026-09-01  
**Canonical Golam base**: `390ea842837a7d85dca165d9291d5eb54c3f11db`

This document records exact research state for planning. It does not admit source code, dependencies, binaries, services or runtime authority.

## Canonical predecessor evidence

### Spec 003 production-executor boundary

Canonical `specs/003-identity-policy-secrets-sandbox/implementation/sandbox-native-executor-qualification.md` records:

- T003-074 PASS for a bounded **test-only** Linux x86_64 native executor;
- production baseline remains `native:unqualified`;
- `PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO`;
- no universal native sandbox or network-capable managed-child claim.

Spec 005 therefore cannot enable shell/process/executable-skill/MCP-child execution merely because a test harness exists.

### Spec 004 authority boundaries carried forward

- model backend is not authority;
- tool-call normalization produces candidates, not effects;
- canonical history remains source of truth;
- strict-local hard denial dominates fallback;
- exact-head evidence and independent semantic review remain required.

## Exact source observations

| Source | Exact observed state | Planning use | Admission posture |
| --- | --- | --- | --- |
| `TheHalfMoon/Golam-research` | `main@a9f633e09d49a85829b8236331b9e21f7e612634`, tree `b68f24972427952c4934e4364736fec62661044f` | high-value mechanism evidence for host/box/tools/MCP/local execution | candidate only; exact bounded Source Foundry required |
| `xai-org/grok-build` | `main@bb7f39d5858cbf5e00de639367f59debbdcb0138` | current security/tool/context patterns | reference only unless later admitted |
| `openclaw/openclaw` | `main@caf1a67dd30a2e04df93a8b240504fb485bcdca0` | path/input hardening and local agent patterns | reference only unless later admitted |
| `NousResearch/hermes-agent` | `main@18a76be124d7c16ed98b629a358b23fef76a7f46` | MCP/tool naming/normalization and agent-tool research | reference only unless later admitted |
| `modelcontextprotocol/modelcontextprotocol` | `main@3ff697dcbea0804f3f397b864cfbbaaa10cba71a` | MCP protocol authority reference | specification reference |
| `modelcontextprotocol/rust-sdk` | `main@51ccb42993d6eb5075399672ce7a0c21a0e55eea` | Rust implementation candidate | not admitted; exact dependency qualification required |
| `agentclientprotocol/agent-client-protocol` | `main@01b9d6e9c094d31cdea6d88768a9dd31b089ccef` | ACP interoperability reference | specification/reference |
| `agentskills/agentskills` | `main@69ef37e9424c0a7ea9dd2293b559e43ec8176379` | Agent Skills packaging reference | format/reference |
| `BurntSushi/ripgrep` | `master@3fce3b5bb0236da2df6d99672afb8a719642eca7` | bounded L0 search candidate | not admitted; exact selected crate/binary closure required |
| `tree-sitter/tree-sitter` | `master@c206ad1e6a4af428942acdd81dbadce9922a72c2` | optional L1 structural context candidate | defer until measured need + exact admission |
| `mem0ai/mem0` | `main@71fba8d46436f88569d600f81a55208c38ad30b5` | memory behavior research | reference only; not canonical memory |
| `qdrant/qdrant` | `master@74f3e85b9473c62560006c043e13737ce6b48412` | derivative vector-index research | defer until measured need; never canonical memory |

## Golam-Research bounded inspection

The reconstruction describes readable TypeScript boundaries for host, coordinator, local execution, protocols, plugins/MCP and an optional Docker sandbox. It explicitly states that it is not the original upstream monorepo, so mechanism evidence and source-rights/admission remain separate questions.

Exact inspected files at `a9f633e...`:

### `source/host/box/box-capabilities.ts`

Useful pattern: optional mechanism interfaces for environment/MCP behavior and explicit unsupported errors.

Golam adaptation: tool/provider capability discovery may describe mechanism availability, but capability advertisement must not be confused with Golam authorization.

### `source/host/box/box-shell-command.ts`

The donor helper constructs shell args and explicitly sets `skipApproval: true`.

**Disposition**: authority semantics rejected. Golam MUST NOT reuse or reproduce approval bypass. Useful information is limited to the existence of explicit command/working-directory/tool-call identity fields and parsed-command metadata as mechanism concepts.

`DONOR_SKIP_APPROVAL=REJECTED`

### `source/host/box/box-mcp.ts`

Useful pattern: MCP support is a separately detectable capability with explicit unsupported-state behavior rather than assumed availability.

Golam adaptation: unavailable or unqualified MCP must fail clearly; however, a server/control plane does not own Golam authority.

### `source/host/durable-file-policy.ts`

Useful pattern: operational files can be intentionally excluded from broader persisted/exported stores.

Golam adaptation: protected/operational evidence classes require explicit storage policy; generic memory/file export must not silently sweep in protected control state.

## Current upstream security signals

The observed `grok-build` head includes recent fixes for native Write/Edit policy bypass through repository symlinks and config interpolation RCE. This reinforces two planning requirements independent of donor implementation details:

1. authorization cannot rely on lexical path prefix alone;
2. untrusted content must not be interpolated into executable/config contexts without bounded encoding/validation.

The observed `openclaw` head hardens target-file handling around regular-file/size validation and failure-preserving behavior, reinforcing fail-closed special-file and bounded-read requirements.

These observations are reference evidence, not code admission.

## Context architecture research disposition

### L0 — required

- direct bounded file reads;
- bounded search;
- Git status/diff/history/object identity;
- exact user-selected artifacts;
- deterministic filters for permissions, freshness, taint and authority.

L0 must be sufficient for the initial bounded product slice.

### L1 — conditional

Tree-sitter/LSP may add symbol/structure evidence when reproducible tasks show material benefit over L0. No dependency is added during planning.

### L2 — deferred

Graph/dataflow/runtime/vector systems are not P0. Dense retrieval must prove measurable benefit and remain a rebuildable derivative.

`DENSE_VECTOR_INDEX=DEFER_UNTIL_MEASURED_NEED`

## Memory architecture research disposition

External memory frameworks demonstrate useful behaviors—candidate extraction, retrieval, user/project scoping, deduplication and lifecycle management—but Golam's constitution fixes different authority semantics:

- Markdown is canonical long-lived knowledge;
- SQLite is operational state;
- indexes are derivatives;
- promotion is governed;
- contradictions are surfaced;
- secret-derived content is excluded;
- live authoritative state outranks memory.

Therefore Mem0/Qdrant-class systems cannot become canonical truth or hidden core dependencies. A later exact Source Foundry decision may admit a derivative component only if measured need justifies it.

## Protocol disposition

### MCP

Use the official MCP specification as protocol reference. The Rust SDK is a candidate, not an automatic dependency. Implementation must qualify exact release/features/transitives/network/process behavior before admission.

### ACP

Use ACP as an IDE/client interoperability boundary. Existing authenticated local-client rules remain stronger than transport presence.

### Agent Skills

Adopt compatible instruction packaging semantics. `SKILL.md` content is untrusted instruction/configuration. Executable scripts require a separately qualified production sandbox profile.

## Planning conclusions

1. Build Golam-owned contracts first; adapters remain replaceable.
2. Start useful capability with filesystem/search/Git/context/memory that does not require native untrusted-process admission.
3. Make production native-executor qualification an explicit gate before shell/process/executable MCP/skill paths.
4. Treat path identity, symlink/reparse/junction behavior and protected-resource exclusion as first-class security state.
5. Keep canonical memory human-readable and derivative systems optional.
6. Preserve protocol interoperability without delegating authority to protocols or donors.
7. Do not import worker/scheduler/learning/Desktop/channel scope from research repositories or noncanonical PRs.
