# Donor and Dependency Qualification — Spec 004

**Date**: 2026-08-31  
**Phase**: PLANNING  
**Rule**: this document records research qualification only. It does not admit production code or dependencies.

## Qualification rule

A source may inform architecture without becoming a production donor. Production admission requires exact implementation-selected source/version, permission/license evidence, notices, selected files/crates/features, dependency and generated/vendored closure, unsafe/FFI/native/process/network behavior, platform posture, reuse strategy and independent Golam verification.

## Q-001 — Golam-Research

- Source: `TheHalfMoon/Golam-research`
- Exact observed state: `a9f633e09d49a85829b8236331b9e21f7e612634`
- Permission posture: governed by canonical Spec 001 founder attestation and reconstruction-specific provenance caveats.
- Inspected bounded evidence: `source/node-agent-coordinator/inference-router.ts` and surrounding coordinator tree.
- Useful mechanisms: provider routing, serialized per-agent request queue, streaming projection, transcript persistence.
- Rejected mechanism: provider/router-owned direct routed-tool execution.
- Unsafe/FFI/native: not selected for reuse in this planning phase.
- Network/process surface: provider and MCP routing are network/process capable and incompatible with an implicit strict-local assumption.
- Disposition: `REFERENCE_ONLY`.

## Q-002 — grok-build

- Source: `xai-org/grok-build`
- Exact observed state: `bc7f02eddd3d84085849dc19ed216f11c23b0571`
- License observed: Apache-2.0.
- Inspected bounded evidence: session run-loop/turn/tool-call/goal files and `full_replace_compaction.rs`.
- Useful mechanisms: explicit compaction sampler seam, cancellation token, attempt evidence, transient/deterministic/context-overflow classification, retry degradation telemetry.
- Reuse risk: product-specific prompts, session/event schemas and telemetry are not Golam contracts.
- Disposition: `REFERENCE_ONLY`.

## Q-003 — Goose

- Source: `aaif-goose/goose`
- Exact observed state: `7d97fe1eadedd8c7f8cc25e9a537b7ab8c8b1dd5`
- License observed: Apache-2.0.
- Inspected bounded evidence: `crates/goose/src/agents/agent.rs` and indexed session/provider/tool lifecycle files.
- Useful mechanisms: explicit Rust state-machine decomposition, persisted message identity, cancellation token, retry manager, separated compaction/tool operations.
- Reuse risk: Goose permission/tool/provider semantics are not Golam authority and cannot replace KernelApi/Effect Gate.
- Disposition: `REFERENCE_ONLY`.

## Q-004 — DeepSeek Harness

- Source: `deepseek-ai/deepseek-harness`
- Exact observed state: `0a53fb55bea101816fa226bb964ae2bed71c343b`
- License observed: MIT.
- Inspected bounded evidence: `docs/subsystems/compaction.md`, `packages/core/agent-loop/README.md`, indexed session/streaming/tool references.
- Useful mechanisms: durable turn/step/request evidence, explicit request-error retry, cancellation-preserved stream prefix, synthetic aborted tool-call results, compaction start/summary/end lifecycle and source-range evidence.
- Reuse risk: TypeScript/plugin framework and framework persistence/event taxonomy are not a Rust trusted-path fit and must not become the Golam runtime root.
- Disposition: `REFERENCE_ONLY`.

## Q-005 — mistral.rs

- Source: `EricLBuehler/mistral.rs`
- Exact observed state: `d348a88833c5da8403e2320997c28cdd02ae4f4b`
- Workspace version observed: `0.9.2`
- License observed: MIT.
- Rust version observed: 1.94.
- Candidate role: primary Rust-native local inference backend.
- Relevant packages observed: `mistralrs`, `mistralrs-core` plus quantization/device/runtime packages.
- Relevant capability evidence: local GGUF/model loading, streaming, grammar/schema/tool-call generation, hardware-aware tuning.
- Network-capable surface observed: `hf-hub`, `reqwest`, URL/model-repository retrieval paths and server/network packages.
- Broad non-required surface observed: MCP, agent loop, shell, code execution, skills, web search, server/web UI.
- Native/device surface: accelerator-specific CUDA/Metal/BLAS/JIT/kernel paths and transitive Candle/device dependencies require exact feature closure review.
- Current blockers to admission:
  1. exact minimal crate/feature subset not yet selected;
  2. complete transitive/native/build dependency closure not yet frozen;
  3. strict-local offline behavior must be independently tested with model-download paths unavailable;
  4. in-process crash/native dependency impact must be assessed;
  5. notices/generated/vendored obligations must be recorded for selected closure.
- Required implementation posture: local-file operation; disable or exclude backend-owned tool execution/MCP/shell/code execution/web search/skills/auto-download behavior.
- Disposition: `PRIMARY_CANDIDATE_NOT_YET_ADMITTED`.

## Q-006 — llama.cpp

- Source: `ggml-org/llama.cpp`
- Exact observed state: `010be9683afabe14ce299197b38c329f94bae568`
- License observed: MIT.
- Candidate role: compatibility local inference backend.
- Relevant capability evidence: local inference, broad device support, schema-constrained output and function/tool-call formatting through `llama-server`.
- Native boundary: C/C++ implementation with multiple accelerator backends; incompatible with direct unsafe/native FFI inside `golamd` under the existing workspace posture.
- Network-capable surface observed: model URL/Hugging Face retrieval, optional RPC, HTTP server.
- Offline control observed: `--offline` forces cache/local operation and prevents network access.
- Current blockers to admission:
  1. exact executable/build/release identity not yet selected;
  2. sidecar transport/authentication contract not yet implemented/qualified;
  3. selected build backend and native dependency closure not yet frozen;
  4. strict-local launch arguments and external no-egress evidence not yet proven;
  5. server/control exposure must not be unauthenticated localhost.
- Required implementation posture: supervised out-of-process sidecar; local model path; offline mode; no RPC/download options; authenticated/private local transport; bounded stdout/stderr/process lifecycle.
- Disposition: `COMPATIBILITY_CANDIDATE_NOT_YET_ADMITTED`.

## Planning admission register

| ID | Candidate | Code admitted? | Dependency admitted? | State |
|---|---|---:|---:|---|
| Q-001 | Golam-Research | No | No | REFERENCE_ONLY |
| Q-002 | grok-build | No | No | REFERENCE_ONLY |
| Q-003 | Goose | No | No | REFERENCE_ONLY |
| Q-004 | DeepSeek Harness | No | No | REFERENCE_ONLY |
| Q-005 | mistral.rs | No | No | PRIMARY_CANDIDATE_NOT_YET_ADMITTED |
| Q-006 | llama.cpp | No | No | COMPATIBILITY_CANDIDATE_NOT_YET_ADMITTED |

`SPEC004_PLANNING_PRODUCTION_ADMISSION_COUNT=0`
