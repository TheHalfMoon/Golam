# Donor Qualification — Spec 005

**Status**: PLANNING QUALIFICATION ONLY — NO CODE ADMITTED

This record classifies exact observed sources for Spec 005. Founder permission eligibility does not replace exact per-component Source Foundry admission. Planning may identify reusable mechanisms; implementation may copy/port/vendor/depend only after a source-specific record reaches `ADMITTED`.

## Qualification rules

A source component is admitted only when exact source state, permission/license scope, notices, selected files/crates, dependency closure, generated/vendored code, network/telemetry/secrets behavior, unsafe/FFI/process boundaries, platform posture, reuse strategy and independent Golam verification are complete.

## `TheHalfMoon/Golam-research`

**Observed**: `a9f633e09d49a85829b8236331b9e21f7e612634`  
**Tree**: `b68f24972427952c4934e4364736fec62661044f`  
**Role**: HIGH_VALUE_IMPLEMENTATION_EVIDENCE / AUTHORIZED-SOURCE CANDIDATE  
**Current admission**: `NOT_ADMITTED`

The reconstruction itself states it is not the original upstream monorepo. The founder permission attestation makes bounded source use eligible for later admission but does not automatically admit original installers, renderer assets, trademarks, credentials, external services or broad dependencies.

### Candidate mechanism areas

- capability/availability interfaces;
- explicit unsupported-state behavior;
- environment and local-execution separation;
- MCP transport/control separation;
- file transfer and store-policy separation;
- explicit tool-call/working-directory identity;
- reconstructed protocol and lifecycle tests.

### Rejected authority semantics

Exact `source/host/box/box-shell-command.ts` sets `skipApproval: true` when constructing shell args. Golam rejects this semantic categorically.

```text
GOLAM_RESEARCH_SKIP_APPROVAL=REJECTED
DONOR_TOOL_AUTHORITY=NOT_ADOPTED
DONOR_MCP_AUTHORITY=NOT_ADOPTED
DONOR_SANDBOX_AUTHORITY=NOT_ADOPTED
```

Any future bounded port must route consequential execution through Golam's Kernel/Effect Gate and cannot preserve donor approval-bypass semantics.

## `xai-org/grok-build`

**Observed**: `bb7f39d5858cbf5e00de639367f59debbdcb0138`  
**Role**: REFERENCE / POTENTIAL BOUNDED SOURCE CANDIDATE  
**Current admission**: `NOT_ADMITTED`

The observed head contains recent security repairs involving symlink-based native Write/Edit policy bypass and configuration interpolation RCE. Those are strong negative lessons for Spec 005 path and command/config handling.

No code is admitted by planning. Any later selected crate/file requires exact Source Foundry qualification.

## `openclaw/openclaw`

**Observed**: `caf1a67dd30a2e04df93a8b240504fb485bcdca0`  
**Role**: REFERENCE  
**Current admission**: `NOT_ADMITTED`

The exact head hardens target-file reads using bounded regular-file validation and preserves failed input instead of destructively consuming it. Useful design lesson: special files, size limits, symlink semantics and failure preservation belong in the tool contract.

No OpenClaw authority model, plugin breadth, channel behavior or runtime dependency is adopted.

## `NousResearch/hermes-agent`

**Observed**: `18a76be124d7c16ed98b629a358b23fef76a7f46`  
**Role**: REFERENCE  
**Current admission**: `NOT_ADMITTED`

Useful research includes MCP/tool naming normalization and agent-tool UX. Python runtime/framework dependency is not a strict-local trusted-path candidate under current architecture.

## Official MCP sources

### `modelcontextprotocol/modelcontextprotocol`

**Observed**: `3ff697dcbea0804f3f397b864cfbbaaa10cba71a`  
**Disposition**: `PROTOCOL_REFERENCE`

### `modelcontextprotocol/rust-sdk`

**Observed**: `51ccb42993d6eb5075399672ce7a0c21a0e55eea`  
**Observed release commit**: v3.2.0  
**Disposition**: `DIRECT_DEPENDENCY_CANDIDATE_NOT_ADMITTED`

Before implementation admission, select the minimal exact crate/features and record transitive/network/runtime closure. Official status does not bypass Source Foundry.

## ACP

`agentclientprotocol/agent-client-protocol@01b9d6e9c094d31cdea6d88768a9dd31b089ccef`

**Disposition**: `PROTOCOL_REFERENCE`

No ACP transport may bypass authenticated local-client identity or capability scope.

## Agent Skills

`agentskills/agentskills@69ef37e9424c0a7ea9dd2293b559e43ec8176379`

**Disposition**: `FORMAT_REFERENCE`

Instruction packaging may be implemented independently. Executable skill behavior is not admitted by the format and remains sandbox-gated.

## ripgrep

`BurntSushi/ripgrep@3fce3b5bb0236da2df6d99672afb8a719642eca7`

**Disposition**: `L0_SEARCH_CANDIDATE_NOT_ADMITTED`

Implementation must decide whether to use selected Rust crates, a pinned external binary, or a Golam-owned bounded search path. The choice must compare dependency/process/sandbox/performance/ignore semantics and strict-local behavior before admission.

## Tree-sitter

`tree-sitter/tree-sitter@c206ad1e6a4af428942acdd81dbadce9922a72c2`

**Disposition**: `DEFER_L1_UNTIL_MEASURED_NEED`

No implementation dependency until L0 evaluation demonstrates a material structural-context gap.

## Mem0 and Qdrant

- `mem0ai/mem0@71fba8d46436f88569d600f81a55208c38ad30b5`
- `qdrant/qdrant@74f3e85b9473c62560006c043e13737ce6b48412`

**Disposition**: `REFERENCE_ONLY_DERIVATIVE_MEMORY_INDEX`

Neither may become canonical Golam memory. Initial Spec 005 requires no dense-vector service. A future measured-need decision must preserve local ownership, rebuildability, strict-local behavior and Markdown/SQLite canonical truth.

## Explicit non-admissions

```text
PLANNING_CODE_REUSED=NO
PLANNING_DEPENDENCY_ADDED=NO
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
MCP_RUST_SDK_ADMITTED=NO
RIPGREP_ADMITTED=NO
TREE_SITTER_ADMITTED=NO
MEM0_ADMITTED=NO
QDRANT_ADMITTED=NO
DONOR_APPROVAL_BYPASS_ADOPTED=NO
NONCANONICAL_PR_6_7_8_PROMOTED_TO_AUTHORITY=NO
```

## Implementation Source Foundry order

1. qualify the exact L0 search implementation before adding it;
2. qualify production native executor containment before shell/process/executable MCP/skill launch;
3. qualify the exact MCP implementation dependency only if MCP implementation requires it;
4. qualify Tree-sitter/LSP only after measured L0 insufficiency;
5. qualify any derivative memory index only after measured need; canonical memory remains independent.
