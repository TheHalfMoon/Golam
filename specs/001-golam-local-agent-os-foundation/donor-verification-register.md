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
| deepseek-ai/deepseek-harness | VERIFIED_SNAPSHOT — `master@cd5ef8148158c3a752a658978873241fdf8e2bbc` | FOUNDER_PERMISSION_ATTESTED | HIGH_VALUE_HARNESS_REFERENCE / SELECTIVE_PORT candidate | MIT. PA-002 inspected turn/step/session-event/capability-seam/profile architecture and safety notice. Adopt replaceable runtime patterns only outside Golam's privileged kernel; reject its no-privileged-core/everything-plugin assumption for authority. |
| aaif-goose/goose | VERIFIED_SNAPSHOT | FOUNDER_PERMISSION_ATTESTED | SELECTIVE_PORT / AUTHORIZED_SOURCE_CANDIDATE | Rust general-purpose agent; Apache-2.0 snapshot inspected. |
| CopilotKit/OpenBot | VERIFIED_SNAPSHOT | FOUNDER_PERMISSION_ATTESTED | REFERENCE / SELECTIVE_PORT candidate | Gateway/audit/takeover/computer UX patterns. |
| CasualOffice/RASystem | VERIFIED_SNAPSHOT | FOUNDER_PERMISSION_ATTESTED | SELECTIVE_PORT / AUTHORIZED_SOURCE_CANDIDATE | Rust/Iroh remote-control substrate; Windows/Linux on-device qualification still required. |
| n0-computer/iroh | VERIFIED_SNAPSHOT | FOUNDER_PERMISSION_ATTESTED | DIRECT_DEPENDENCY candidate | Rust QUIC/P2P/NAT/relay. |
| microsoft/winappCli | VERIFIED_SNAPSHOT | FOUNDER_PERMISSION_ATTESTED | REFERENCE / SELECTIVE_PORT candidate | Windows UIA behavior/code candidate after bounded qualification. |
| EricLBuehler/mistral.rs | VERIFIED_SNAPSHOT | FOUNDER_PERMISSION_ATTESTED | DIRECT_DEPENDENCY candidate | Rust inference candidate; exact release/API/hardware/no-egress qualification deferred to Spec 004. |
| cedar-policy/cedar | PARTIALLY_VERIFIED | FOUNDER_PERMISSION_ATTESTED | DIRECT_DEPENDENCY candidate | Exact version/schema/perf qualification deferred to Spec 003. |
| bytecodealliance/wasmtime | PARTIALLY_VERIFIED | FOUNDER_PERMISSION_ATTESTED | DIRECT_DEPENDENCY candidate | Bounded WASI extension runtime; not a native universal sandbox. |
| opensandbox-group/OpenSandbox | VERIFIED_SNAPSHOT — `main@48b0215f1bd097b31d0f022a44640e00c11ac49d` | FOUNDER_PERMISSION_ATTESTED | HIGH_VALUE_SANDBOX_PLATFORM / PROTOCOL / DIRECT_PROVIDER candidate | Apache-2.0. PA-002A inspected lifecycle/provider/data-plane separation, Docker/Kubernetes runtimes, `execd`, PTY/SSE, snapshots/pause-resume, egress controls, secure-runtime options and Credential Vault. Candidate only behind Golam-owned sandbox/authority contracts; provider state, snapshots and runtime policy never become authority. |
| llama.cpp | PARTIALLY_VERIFIED | FOUNDER_PERMISSION_ATTESTED | ADAPTER | Prefer sidecar in trusted architecture; exact build/dependency qualification deferred to Spec 004. |
| lahfir/agent-desktop | PARTIALLY_VERIFIED | FOUNDER_PERMISSION_ATTESTED | SELECTIVE_PORT candidate | Semantic snapshot/ref concepts; exact source state admission required. |
| mem0ai/mem0 | VERIFIED_SNAPSHOT — `main@fdfb763d6e5e5509bdb35d4ddc9ca8003f6af009` | FOUNDER_PERMISSION_ATTESTED | HIGH_VALUE_MEMORY_REFERENCE / SELECTIVE_SEMANTIC_PORT candidate | Apache-2.0. PA-002 inspected ADD-first extraction, temporal/entity metadata, scoped memory and semantic+BM25+entity retrieval. Golam keeps promotion governance and does not grant agent-generated facts equal authority. |
| langchain-ai/langgraph | VERIFIED_SNAPSHOT — `main@d5f4b2aa960940effc8430165ab3604038e817af` | FOUNDER_PERMISSION_ATTESTED | HIGH_VALUE_ORCHESTRATION_REFERENCE / CONFORMANCE_PATTERN candidate | MIT. Durable execution, interrupts/resume and checkpoint conformance are useful; graph/checkpoint state never replaces Golam event/effect authority. |
| langchain-ai/langchain | VERIFIED_SNAPSHOT — `master@5893459c4f2bfac6c8d3262cae1e3f2246d9287f` | FOUNDER_PERMISSION_ATTESTED | ECOSYSTEM_REFERENCE / ADAPTER_PATTERN | MIT. Broad provider/tool/retriever interfaces pressure-test replaceable seams; no Python core dependency. |
| qdrant/qdrant | VERIFIED_SNAPSHOT — `master@74f3e85b9473c62560006c043e13737ce6b48412` | FOUNDER_PERMISSION_ATTESTED | OPTIONAL DIRECT_DEPENDENCY / SIDECAR candidate | Apache-2.0. Qdrant Edge is a serious Spec 005 local/offline derived-index candidate; inspected Edge package is currently 0.1.0 and requires maturity/dependency/storage/security benchmarking before admission. Never canonical memory. |
| firecrawl/firecrawl | VERIFIED_SNAPSHOT — `main@83df13affe6373ffe32b5daa99dd000ebab2ec73` | FOUNDER_PERMISSION_ATTESTED | BEHAVIOR_REFERENCE / EXTERNAL_ADAPTER candidate | Root license inspected as AGPL-3.0. Target search/scrape/crawl/map/extract/interact behavior; default no core code import without explicit reciprocal-license decision. Never a hidden strict-local dependency. |
| run-llama/llama_index | VERIFIED_SNAPSHOT — `main@39f481fc41d1da26cf511ef697762b3b0a93636d` | FOUNDER_PERMISSION_ATTESTED | HIGH_VALUE_CONTEXT_REFERENCE / SELECTIVE_SEMANTIC_PORT candidate | MIT. Source/ingest/retrieve/postprocess/rerank/citation decomposition is useful; implement independently in Rust and keep managed services optional. |
| NousResearch/hermes-agent | VERIFIED_SNAPSHOT — `main@4e7eb39947f132f961923f9e3f600bc8e63066dd` | FOUNDER_PERMISSION_ATTESTED | HIGH_VALUE_BEHAVIOR_REFERENCE / SELECTIVE_PORT candidate | MIT. PA-002 inspected curated memory, FTS5 session search, learning journey, background learning, skills, scheduling/subagents and messaging UX. Golam explicitly rejects free-form messaging approval and broad yolo/always-allow semantics for protected effects. |
| openclaw/openclaw | VERIFIED_SNAPSHOT — `main@23a681efa6fc0e264e562c4249d8906c0785b5e4` | FOUNDER_PERMISSION_ATTESTED | HIGH_VALUE_PRODUCT / MEMORY / CHANNEL / SECURITY REFERENCE; SELECTIVE_PORT candidate | MIT. PA-002A inspected Gateway/device pairing, typed node capabilities, Markdown-first USER/MEMORY/daily/DREAMS model, hybrid search, dreaming, taint-gated promotion, Memory Wiki, sandbox provider UX and operator security audit. Golam rejects OpenClaw's single-trusted-operator Gateway assumptions as an authority model and preserves stronger per-effect/device/channel boundaries. |
| karpathy/autoresearch | PARTIALLY_VERIFIED — `master@228791fb499afffb54b46200aca536f79142f117` | FOUNDER_PERMISSION_ATTESTED | HIGH_VALUE_EXPERIMENT_PATTERN / REFERENCE | README declares MIT but root LICENSE was not found during PA-002 verification. Use bounded baseline/fixed-budget/immutable-evaluator/keep-discard pattern; direct code reuse requires rights re-verification. |
| braintrustdata/agentbehavior | VERIFIED_SNAPSHOT — `main@1866cffb530c93412719b7d3e243612a11bedf97` | FOUNDER_PERMISSION_ATTESTED | OPEN_BEHAVIOR_FORMAT / EVAL_REFERENCE candidate | Apache-2.0. `BEHAVIOR.md` trajectory-level specification is a strong GolamBench compatibility target; behavior text is never authority. |
| braintrustdata/autoevals | VERIFIED_SNAPSHOT — `main@b0e1055892bea1305a10f8d42fdc47ff1b41ffa4` | FOUNDER_PERMISSION_ATTESTED | SCORER_REFERENCE / SELECTIVE_SEMANTIC_PORT candidate | MIT. Deterministic/heuristic/statistical/model scorers and compact score objects are useful; LLM judge remains supplementary evidence. |
| braintrustdata/bash-agent-evals | PARTIALLY_VERIFIED — `main@a13ca02330fdd4f000ca7ad5e8a3b6958afd27b8` | FOUNDER_PERMISSION_ATTESTED | BENCHMARK_BEHAVIOR_REFERENCE_ONLY pending rights closure | Compares Bash/filesystem/SQL/embedding routes on common corpus/questions. Root LICENSE not found and inspected package.json had no license field; no direct code reuse without rights clarification. |
| Graphify-Labs/graphify | UNVERIFIED_REFERENCE | FOUNDER_PERMISSION_ATTESTED | OPTIONAL ADAPTER / SELECTIVE_PORT candidate | Reverify/benchmark only if Spec 005 demonstrates L2 need. |
| vitali87/code-graph-rag | UNVERIFIED_REFERENCE | FOUNDER_PERMISSION_ATTESTED | OPTIONAL ADAPTER / SELECTIVE_PORT candidate | Deep semantic/dataflow/runtime ideas; no mandatory graph DB. |
| TencentDB Agent Memory / Graphiti / Letta / OpenViking / IWE / AFFiNE | UNVERIFIED_REFERENCE | FOUNDER_PERMISSION_ATTESTED | REFERENCE / SELECTIVE_PORT candidates | Reverify and select only bounded mechanisms that improve Golam's governed memory. Mem0 moved to its own verified PA-002 row. |
| DeerFlow / OpenFang / OpenFleet / IronClaw / ZeroClaw / PicoClaw / block/buzz | UNVERIFIED_REFERENCE | FOUNDER_PERMISSION_ATTESTED | REFERENCE / SELECTIVE_PORT candidates | Permission removes default rights exclusion; full Source Foundry qualification still required before reuse. Hermes and OpenClaw moved to their own verified PA-002/PA-002A rows. |
| Restate / Temporal | UNVERIFIED_REFERENCE | FOUNDER_PERMISSION_ATTESTED | REFERENCE / SELECTIVE_PORT candidates | Durable-execution semantics; Golam still avoids mandatory external server dependency. |
| RustDesk / OpenControl / reciprocal remote-control sources | PARTIALLY_VERIFIED / REFERENCE | FOUNDER_PERMISSION_ATTESTED | AUTHORIZED_SOURCE_CANDIDATE | Prior reciprocal-license exclusion is no longer automatic because founder permission is asserted. Exact permission must explicitly cover intended reuse/redistribution and any continuing license obligations before admission. |

## PA-002 research record

Detailed exact-head findings, source classifications, prohibited transplants and owning-spec assignments for the newly verified memory/retrieval/harness/learning/eval/sandbox wave are recorded in:

- `program-amendments/PA-002-source-foundry-research.md`
- `program-amendments/PA-002-memory-retrieval-learning-evals.md`
- `program-amendments/PA-002A-openclaw-opensandbox-source-foundry.md`
- `contracts/memory-retrieval-learning-contract.md`
- `contracts/behavior-evaluation-contract.md`

No PA-002 or PA-002A source is marked `ADMITTED` by this planning wave.

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
