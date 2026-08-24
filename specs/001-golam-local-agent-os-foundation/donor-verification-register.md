# Donor Verification Register

**Purpose**: separate "mentioned/researched", "source verified", "permission attested", and "technically admitted". This register is planning evidence; code admission still happens per bounded implementation spec.

Status vocabulary:
- `VERIFIED_SNAPSHOT`: exact repository/head/tree/license or equivalent source state inspected during planning.
- `PARTIALLY_VERIFIED`: repository/behavior inspected but exact admission closure is incomplete.
- `UNVERIFIED_REFERENCE`: concept/source named but not independently reverified in the final review cycle.
- `BENCHMARK_ONLY`: public product behavior target, not necessarily a code donor.
- `FOUNDER_PERMISSION_ATTESTED`: founder states permission has been obtained for the source universe; exact per-source scope/evidence still must be recorded at admission.
- `AUTHORIZED_SOURCE_CANDIDATE`: source may be seriously evaluated for bounded code reuse/porting after exact Source Foundry qualification.
- `ADMITTED`: reserved for a later implementation spec after rights + technical/security qualification for the exact bounded component.

**Global permission state**: `FOUNDER_PERMISSION_ATTESTED` for all sources supplied by the founder and all sources introduced during Spec 001 research. See `source-permission-attestation.md`.

| Source | Verification status | Permission posture | Planning classification | Notes |
|---|---|---|---|---|
| xAI Grok Bot public product | BENCHMARK_ONLY | FOUNDER_PERMISSION_ATTESTED where source/material permission applies | BENCHMARK_TARGET | Public behavior/parity target; proprietary implementation details still require exact source/provenance scope before reuse. |
| Golam-Research / Grok Bot 0.18 reconstruction | VERIFIED_SNAPSHOT | FOUNDER_PERMISSION_ATTESTED | HIGH_VALUE_IMPLEMENTATION_EVIDENCE / AUTHORIZED_SOURCE_CANDIDATE | Working source-oriented reconstruction grounded in pinned release artifacts. Mine runtime/protocol/tool/test behavior seriously. Exact component permission scope/evidence must be recorded before code admission; do not misrepresent reconstruction as original Anysphere monorepo. Renderer/assets/installers/trademarks remain separately scoped. |
| xai-org/grok-build | VERIFIED_SNAPSHOT | FOUNDER_PERMISSION_ATTESTED | SELECTIVE_PORT / AUTHORIZED_SOURCE_CANDIDATE | Rust; Apache-2.0 snapshot inspected. Exact admission requalification still required. |
| deepseek-ai/deepseek-harness | VERIFIED_SNAPSHOT | FOUNDER_PERMISSION_ATTESTED | REFERENCE / SELECTIVE_PORT candidate | MIT Node/TypeScript/Cordis system; use runtime/session/tool concepts or bounded ports, not as Golam core. |
| aaif-goose/goose | VERIFIED_SNAPSHOT | FOUNDER_PERMISSION_ATTESTED | SELECTIVE_PORT / AUTHORIZED_SOURCE_CANDIDATE | Rust general-purpose agent; Apache-2.0 snapshot inspected. |
| CopilotKit/OpenBot | VERIFIED_SNAPSHOT | FOUNDER_PERMISSION_ATTESTED | REFERENCE / SELECTIVE_PORT candidate | Gateway/audit/takeover/computer UX patterns. |
| CasualOffice/RASystem | VERIFIED_SNAPSHOT | FOUNDER_PERMISSION_ATTESTED | SELECTIVE_PORT / AUTHORIZED_SOURCE_CANDIDATE | Rust/Iroh remote-control substrate; Windows/Linux on-device qualification still required. |
| n0-computer/iroh | VERIFIED_SNAPSHOT | FOUNDER_PERMISSION_ATTESTED | DIRECT_DEPENDENCY candidate | Rust QUIC/P2P/NAT/relay. |
| microsoft/winappCli | VERIFIED_SNAPSHOT | FOUNDER_PERMISSION_ATTESTED | REFERENCE / SELECTIVE_PORT candidate | Windows UIA behavior/code candidate after bounded qualification. |
| EricLBuehler/mistral.rs | VERIFIED_SNAPSHOT | FOUNDER_PERMISSION_ATTESTED | DIRECT_DEPENDENCY candidate | Rust inference candidate; exact release/API/hardware/no-egress qualification deferred to Spec 004. |
| cedar-policy/cedar | PARTIALLY_VERIFIED | FOUNDER_PERMISSION_ATTESTED | DIRECT_DEPENDENCY candidate | Exact version/schema/perf qualification deferred to Spec 003. |
| bytecodealliance/wasmtime | PARTIALLY_VERIFIED | FOUNDER_PERMISSION_ATTESTED | DIRECT_DEPENDENCY candidate | Bounded WASI extension runtime; not a native universal sandbox. |
| llama.cpp | PARTIALLY_VERIFIED | FOUNDER_PERMISSION_ATTESTED | ADAPTER | Prefer sidecar in trusted architecture; exact build/dependency qualification deferred to Spec 004. |
| lahfir/agent-desktop | PARTIALLY_VERIFIED | FOUNDER_PERMISSION_ATTESTED | SELECTIVE_PORT candidate | Semantic snapshot/ref concepts; exact source state admission required. |
| Graphify-Labs/graphify | UNVERIFIED_REFERENCE | FOUNDER_PERMISSION_ATTESTED | OPTIONAL ADAPTER / SELECTIVE_PORT candidate | Reverify/benchmark only if Spec 005 demonstrates L2 need. |
| vitali87/code-graph-rag | UNVERIFIED_REFERENCE | FOUNDER_PERMISSION_ATTESTED | OPTIONAL ADAPTER / SELECTIVE_PORT candidate | Deep semantic/dataflow/runtime ideas; no mandatory graph DB. |
| TencentDB Agent Memory / Graphiti / Mem0 / Letta / OpenViking / IWE / AFFiNE | UNVERIFIED_REFERENCE | FOUNDER_PERMISSION_ATTESTED | REFERENCE / SELECTIVE_PORT candidates | Reverify and select only bounded mechanisms that improve Golam's governed memory. |
| DeerFlow / Hermes / OpenFang / OpenFleet / IronClaw / ZeroClaw / PicoClaw / block/buzz | UNVERIFIED_REFERENCE | FOUNDER_PERMISSION_ATTESTED | REFERENCE / SELECTIVE_PORT candidates | Permission removes default rights exclusion; full Source Foundry qualification still required before reuse. |
| Restate / Temporal | UNVERIFIED_REFERENCE | FOUNDER_PERMISSION_ATTESTED | REFERENCE / SELECTIVE_PORT candidates | Durable-execution semantics; Golam still avoids mandatory external server dependency. |
| RustDesk / OpenControl / reciprocal remote-control sources | PARTIALLY_VERIFIED / REFERENCE | FOUNDER_PERMISSION_ATTESTED | AUTHORIZED_SOURCE_CANDIDATE | Prior reciprocal-license exclusion is no longer automatic because founder permission is asserted. Exact permission must explicitly cover intended reuse/redistribution and any continuing license obligations before admission. |

## Admission rule

Permission and source verification are separate gates.

For each bounded component, later implementation specs MUST progress through:

```text
REFERENCE
  -> VERIFIED_SOURCE_STATE
  -> PERMISSION_RECORDED
  -> TECHNICALLY_QUALIFIED
  -> ADMITTED
```

`FOUNDER_PERMISSION_ATTESTED` is enough to remove the prior default rejection and authorize serious qualification. It is not enough to mark a component `ADMITTED` without recording exact permission scope/evidence and the technical/security closure.

A later implementation spec must pin exact commit/tree/version and close: permission scope; license/notices; trademarks/assets where relevant; vendored/generated code; dependency closure; unsafe/FFI/process/network/telemetry/secrets behavior; platform evidence; selected files/crates; modifications; and independent Golam tests/benchmarks before source reuse.
