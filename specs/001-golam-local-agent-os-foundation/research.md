# Research: Golam Local Agent OS Foundation

**Research closeout date**: 2026-08-24  
**Decision**: DISCOVERY_COMPLETE_FOR_PLANNING  
**External architecture review**: PENDING_GLM_5_3

## 1. Research conclusion

Golam should not be built by porting the old Grok Bot reconstruction into Rust or by stacking multiple agent frameworks. The strongest architecture is a small Rust trusted core plus replaceable harness/model/tool/memory/control/connect adapters.

The donor set is now broad enough. Additional generic agent-framework discovery has diminishing value. Future research should be triggered by a concrete implementation gap, security question, or benchmark failure.

## 2. Final-gap findings from the last search

### 2.1 Spec Kit has matured

The current GitHub Spec Kit workflow is `constitution -> specify -> clarify -> plan -> checklist -> tasks -> analyze -> implement -> converge`. `converge` checks current implementation against spec/plan/tasks and appends traceable remaining work rather than rewriting intent. Golam should adopt this full loop.

Snapshot inspected:
- `github/spec-kit`
- main: `27f50f7e6b618ea14d74dd4037f9e7c60218b16c`
- release line: 1.0.1 / 1.0.2.dev0

### 2.2 Desktop control must be semantic-first

New/current sources reinforce semantic accessibility rather than vision-first control:

- `microsoft/winappCli`: Microsoft CLI uses Windows UI Automation patterns for inspecting/acting on Windows apps; input injection is reserved for operations UIA cannot perform. License: MIT.
- `lahfir/agent-desktop`: Rust-native agent desktop automation with compact snapshots, stable element refs, targeted traversal, JSON errors/recovery hints, and accessibility-first action semantics. Apache-2.0; macOS is the currently mature platform in the inspected release.
- `agent-sh/computer-use-linux`: Rust Linux desktop control over accessibility/Wayland-oriented mechanisms and MCP; useful as Linux behavior/reference candidate.
- `JoshuaALawrence/OpenControl`: strong Windows ideas (UIA + WGC + SendInput + compact observation + privacy blocklist) but AGPL-3.0; reference-only unless reciprocal obligations are explicitly approved.

**Decision**: build Golam's own `DesktopController` contract in Rust. Mine MIT/Apache sources after exact qualification; use reciprocal sources as behavioral references only by default.

### 2.3 Native remote-control substrate should reuse Rust networking/media work

`CasualOffice/RASystem` is unusually aligned with GolamConnect: Rust core, Tauri, Iroh/QUIC transport, PASETO grants, per-message host-side capability checks, consent, emergency stop, tamper-evident audit, H.264 screen pipeline, input injection, clipboard/files/audio/multi-monitor/reconnect. Snapshot inspected earlier in this research cycle:
- head `494a0883bf1ca6bd120069e8aec6052097051a3f`
- tree `7031400a69064775ea13dbce8a215edcedf49fc2`
- license reported by project: Apache-2.0

`n0-computer/iroh` provides public-key dialing, QUIC, direct hole-punching and relay fallback, and is dual MIT/Apache-2.0.

**Decision**: qualify RASystem for selective reuse and Iroh as a core transport dependency candidate. Do not implement a custom NAT traversal stack in P0.

### 2.4 Rust-native inference is now strong enough to be first-class

`mistral.rs` provides a Rust SDK, hardware-aware tuning, local GGUF/UQFF/other formats, multimodality, tool calling, prefix caching, Metal/CUDA support and agentic primitives. It is a primary local inference candidate, but Golam owns the harness and authority model.

`llama.cpp` remains a broad compatibility fallback, especially for GGUF ecosystem coverage and platform portability.

**Decision**: primary candidate `mistral.rs`; compatibility backend `llama.cpp`; adapters for Ollama/MLX/vLLM/SGLang as needed. Backend selection belongs to `ExecutionProfile`, not product semantics.

### 2.5 Cedar now has agent-specific tooling

Beyond the main Rust `cedar-policy` engine, `cedar-policy/cedar-for-agents` contains Rust tooling around MCP tool descriptions/schema generation.

**Decision**: Cedar remains the preferred policy engine candidate; Golam still owns the capability/effect schema and denial semantics.

### 2.6 Wasmtime is appropriate for untrusted portable extensions, with limits

Wasmtime/WASI is capability-oriented and intended for untrusted-code sandboxing. Component Model support is enabled but not fully final across all standards/proposals.

**Decision**: use Wasmtime/WASI for bounded plugin/skill code where its capability model fits; do not mistake it for a complete OS sandbox for arbitrary native tools.

### 2.7 GLM 5.3 is suitable as an external reviewer but not accessible here

Z.ai released GLM-5.3 on 2026-08-14 and positions it as improved on complex coding and long-horizon tasks. This makes it a useful independent architecture reviewer for this plan. No connected GLM/Z.ai invocation capability is available in this ChatGPT session, so review is a required external gate rather than a claimed completed action.

## 3. Donor/source map

Reuse labels:
- **DEPENDENCY_CANDIDATE**: may become a direct dependency after exact qualification.
- **SELECTIVE_DONOR**: mine or port bounded components/mechanisms after exact qualification.
- **PATTERN_REFERENCE**: study behavior/architecture; do not import source by default.
- **BENCHMARK_TARGET**: product/behavior target, not source donor.

| Source | Role | Default decision |
|---|---|---|
| xAI Grok Bot | Product/feature benchmark | BENCHMARK_TARGET |
| xai-org/grok-build | Rust harness, TUI, browser/tools, MCP/plugins/workflows/subagents | SELECTIVE_DONOR (Apache-2.0 snapshot must be re-qualified before import) |
| deepseek-ai/deepseek-harness | Session/tool/plugin/harness architecture | PATTERN_REFERENCE / selective concepts |
| aaif-goose/goose | Rust local general agent, Desktop+CLI+API, provider/MCP ecosystem | SELECTIVE_DONOR / reference |
| RightNow-AI/openfang | Rust Agent OS, channels, WASM, audit, skills | PATTERN_REFERENCE / selective donor after verification |
| bytedance/deer-flow | Goals, subagents, compaction, schedules, sandbox acquisition | PATTERN_REFERENCE; Python runtime not core |
| NousResearch/hermes-agent | Memory learning loop, self-improving skills, channel gateway, cron | PATTERN_REFERENCE; Python runtime not core |
| CopilotKit/OpenBot | watch/takeover UX, per-agent computer, action gateway, audit | SELECTIVE_DONOR / pattern (MIT) |
| block/buzz | Signed identity/events, relay, ACP/MCP separation, agent presence | SELECTIVE_DONOR / pattern (Apache-2.0) |
| CasualOffice/RASystem | Rust remote desktop/control/security over Iroh | SELECTIVE_DONOR (Apache-2.0 candidate) |
| n0-computer/iroh | P2P QUIC/NAT traversal/relay | DEPENDENCY_CANDIDATE (MIT/Apache-2.0) |
| Graphify-Labs/graphify | deterministic structural code graph | ADAPTER/SELECTIVE_DONOR after benchmark |
| vitali87/code-graph-rag | AST/deep semantic/dataflow/runtime graph ideas | OPTIONAL_ADAPTER / PATTERN_REFERENCE |
| TencentDB Agent Memory | governed memory assets/loadouts | PATTERN_REFERENCE |
| OpenSandbox/OpenShell | stronger sandbox/credential patterns | OPTIONAL_BACKEND / PATTERN_REFERENCE |
| cedar-policy/cedar | authorization | DEPENDENCY_CANDIDATE (Apache-2.0) |
| bytecodealliance/wasmtime | WASM/WASI extension sandbox | DEPENDENCY_CANDIDATE |
| EricLBuehler/mistral.rs | local Rust-native inference | DEPENDENCY_CANDIDATE |
| llama.cpp | broad local inference compatibility | OPTIONAL_BACKEND/FFI/sidecar candidate |
| github/spec-kit | project planning/governance | PROCESS_DEPENDENCY |
| Agent Skills specification | skill package interoperability | PROTOCOL/FORMAT TARGET |
| MCP | tools/resources/tasks protocol | PROTOCOL TARGET |
| ACP | IDE/client agent protocol | PROTOCOL TARGET |
| A2A | external agent federation | LATER PROTOCOL TARGET |
| microsoft/winappCli | Windows semantic control behaviors | SELECTIVE_DONOR / reference (MIT) |
| lahfir/agent-desktop | compact semantic observation/refs | SELECTIVE_DONOR candidate (Apache-2.0) |
| RustDesk/OpenControl/NevoFlux | remote/control behavior | REFERENCE_ONLY by default due reciprocal licensing where applicable |

## 4. Source Foundry admission record

No donor is admitted merely because it appears above. Before code admission record:

- repository URL;
- exact commit and tree;
- license and NOTICE obligations;
- generated/vendored code boundaries;
- direct and relevant transitive dependencies;
- reciprocal-license closure;
- network and telemetry behavior;
- credential handling;
- unsafe Rust/FFI/subprocess boundaries;
- platform support and test posture;
- exact files/crates proposed for reuse;
- reuse strategy;
- independent Golam tests/benchmarks required before acceptance.

## 5. Key architectural conclusions

1. Session, Harness, and Sandbox are separate abstractions.
2. Canonical event history is append-oriented; context/summary is a projection.
3. Goal Ledger is protected from ordinary context compaction.
4. Effect transactions carry idempotency semantics and receipts.
5. Model selection becomes `ExecutionProfile` selection.
6. Context compilation is evidence planning, not vector retrieval alone.
7. Memory is governed user-owned evidence.
8. Computer control is semantic-first; vision is fallback.
9. GolamConnect native transport is independent from Telegram/WhatsApp bridges.
10. Security kernel is privileged and non-pluggable; providers/tools/skills are pluggable.
11. Agent Skills/MCP/ACP compatibility matters more than inventing a new DSL.
12. A2A is for later external federation, not internal worker scheduling.
13. Prompt-prefix/cache stability is an explicit performance concern.
14. Long-horizon evaluation must include crash/resume, verification, goal retention, and premature stopping.

## 6. Research stop rule

Generic source discovery is closed. New external research is authorized only when one of these is true:
- an implementation task has an unresolved technical choice;
- security review identifies a threat without mitigation;
- a donor qualification fails and needs a replacement;
- a benchmark shows a specific capability gap;
- a protocol/platform changed materially.
