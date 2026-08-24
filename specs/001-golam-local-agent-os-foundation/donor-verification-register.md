# Donor Verification Register

**Purpose**: separate "mentioned/researched" from "verified at an exact source state". This register is planning evidence only; it does not admit code.

Status vocabulary:
- `VERIFIED_SNAPSHOT`: exact repository/head/tree/license or equivalent source state inspected during planning.
- `PARTIALLY_VERIFIED`: repository/behavior inspected but exact admission closure is incomplete.
- `UNVERIFIED_REFERENCE`: concept/source named but not independently reverified in the final review cycle.
- `BENCHMARK_ONLY`: public product behavior target, never a code donor.
- `REJECT_CODE`: source code not admissible under current license/clean-room policy.

| Source | Status | Planning classification | Notes |
|---|---|---|---|
| xAI Grok Bot | BENCHMARK_ONLY | REFERENCE_ONLY | Public behavior/parity evidence only; no proprietary internals/assets. |
| xai-org/grok-build | VERIFIED_SNAPSHOT | SELECTIVE_PORT | Rust; Apache-2.0 snapshot inspected. Exact admission requalification still required. |
| deepseek-ai/deepseek-harness | VERIFIED_SNAPSHOT | REFERENCE_ONLY | MIT Node/TypeScript/Cordis system; use runtime/session/tool concepts, not as Golam core. |
| aaif-goose/goose | VERIFIED_SNAPSHOT | SELECTIVE_PORT | Rust general-purpose agent; Apache-2.0 snapshot inspected. |
| CopilotKit/OpenBot | VERIFIED_SNAPSHOT | REFERENCE_ONLY | MIT; gateway/audit/takeover/computer UX patterns. |
| CasualOffice/RASystem | VERIFIED_SNAPSHOT | SELECTIVE_PORT | Apache-2.0; Rust/Iroh remote-control substrate; Windows/Linux on-device qualification still required. |
| n0-computer/iroh | VERIFIED_SNAPSHOT | DIRECT_DEPENDENCY candidate | Rust QUIC/P2P/NAT/relay; MIT/Apache-2.0. |
| microsoft/winappCli | VERIFIED_SNAPSHOT | REFERENCE_ONLY | MIT; Windows UIA behavior reference. |
| EricLBuehler/mistral.rs | VERIFIED_SNAPSHOT | DIRECT_DEPENDENCY candidate | Rust inference candidate; exact release/API/hardware/no-egress qualification deferred to Spec 004. |
| cedar-policy/cedar | PARTIALLY_VERIFIED | DIRECT_DEPENDENCY candidate | Policy engine candidate; exact version/schema/perf qualification deferred to Spec 003. |
| bytecodealliance/wasmtime | PARTIALLY_VERIFIED | DIRECT_DEPENDENCY candidate | Bounded WASI extension runtime; not a native universal sandbox. |
| llama.cpp | PARTIALLY_VERIFIED | ADAPTER | Prefer sidecar; exact build/license/dependency qualification deferred to Spec 004. |
| lahfir/agent-desktop | PARTIALLY_VERIFIED | SELECTIVE_PORT candidate | Semantic snapshot/ref concepts; exact license/commit admission required before code use. |
| Graphify-Labs/graphify | UNVERIFIED_REFERENCE | OPTIONAL ADAPTER | Reverify/benchmark only if Spec 005 demonstrates L2 need. |
| vitali87/code-graph-rag | UNVERIFIED_REFERENCE | OPTIONAL ADAPTER / REFERENCE_ONLY | No mandatory graph DB. |
| TencentDB Agent Memory / Graphiti / Mem0 / Letta / OpenViking / IWE / AFFiNE | UNVERIFIED_REFERENCE | REFERENCE_ONLY | Memory-governance concepts only unless later qualified. |
| DeerFlow / Hermes / OpenFang / OpenFleet / IronClaw / ZeroClaw / PicoClaw / block/buzz | UNVERIFIED_REFERENCE | REFERENCE_ONLY | Concepts only until Source Foundry admission. |
| Restate / Temporal | UNVERIFIED_REFERENCE | REFERENCE_ONLY | Durable-execution semantics only; no server dependency. |
| RustDesk / OpenControl / reciprocal remote-control sources | REJECT_CODE | REFERENCE_ONLY behavior | Reciprocal code excluded by default absent explicit founder license decision. |
| Golam-Research reconstruction | REJECT_CODE | REFERENCE_ONLY evidence | Permanent clean-room boundary unless a bounded component is independently proven redistributable and founder-approved. |

## Admission rule

`VERIFIED_SNAPSHOT` is still not code admission. A later implementation spec must pin exact commit/tree/version and close license/notices, vendored/generated code, dependency closure, unsafe/FFI/process/network/telemetry/secrets behavior, platform evidence, selected files/crates, and independent Golam tests before source reuse.
