# PA-002 Source Foundry Research — Memory, Retrieval, Learning, Harness, and Evals

**Status**: PLANNING_EVIDENCE_ONLY  
**Research date**: 2026-08-28  
**Companion architecture amendment**: `PA-002-memory-retrieval-learning-evals.md`  
**Code admission**: NONE. Every source remains below `ADMITTED`; a future owning Spec Kit package must re-pin and close exact Source Foundry qualification before reuse.

## 1. Purpose

This record freezes the source state and useful mechanisms inspected for PA-002. It deliberately separates:

- public/project claims from independently inspected source behavior;
- behavior/architecture reference from direct code reuse;
- license posture from technical/security admission;
- current research evidence from future implementation decisions.

A source being useful, permissively licensed, or founder-permission-attested does not make it part of Golam's trusted path.

---

## 2. Exact snapshot register

| ID | Source | Branch / inspected head | License evidence | Planning classification | Owning future spec(s) |
|---|---|---|---|---|---|
| SF2-001 | `mem0ai/mem0` | `main@fdfb763d6e5e5509bdb35d4ddc9ca8003f6af009` | root `LICENSE`: Apache-2.0 | HIGH_VALUE_REFERENCE / SELECTIVE_SEMANTIC_PORT candidate | 005, 010 |
| SF2-002 | `langchain-ai/langgraph` | `main@d5f4b2aa960940effc8430165ab3604038e817af` | root `LICENSE`: MIT | HIGH_VALUE_REFERENCE / CONFORMANCE_PATTERN candidate | 004, 008, 010 |
| SF2-003 | `langchain-ai/langchain` | `master@5893459c4f2bfac6c8d3262cae1e3f2246d9287f` | root `LICENSE`: MIT | ECOSYSTEM_REFERENCE / ADAPTER_PATTERN | 004, 005 |
| SF2-004 | `qdrant/qdrant` | `master@74f3e85b9473c62560006c043e13737ce6b48412` | Apache-2.0; `lib/edge/Cargo.toml` declares Apache-2.0 | DIRECT_DEPENDENCY / SIDECAR candidate for derived index only | 005, 010 |
| SF2-005 | `firecrawl/firecrawl` | `main@83df13affe6373ffe32b5daa99dd000ebab2ec73` | root `LICENSE`: AGPL-3.0 | BEHAVIOR_REFERENCE / EXTERNAL_ADAPTER candidate | 005, 009, 010 |
| SF2-006 | `run-llama/llama_index` | `main@39f481fc41d1da26cf511ef697762b3b0a93636d` | root `LICENSE`: MIT | HIGH_VALUE_REFERENCE / SELECTIVE_SEMANTIC_PORT candidate | 005, 009 |
| SF2-007 | `NousResearch/hermes-agent` | `main@4e7eb39947f132f961923f9e3f600bc8e63066dd` | root `LICENSE`: MIT | HIGH_VALUE_BEHAVIOR_REFERENCE / SELECTIVE_PORT candidate | 005, 007, 008, 009, 010 |
| SF2-008 | `karpathy/autoresearch` | `master@228791fb499afffb54b46200aca536f79142f117` | README declares MIT; root `LICENSE` was not found during this research pass | BEHAVIOR_REFERENCE; code reuse requires license-file/rights re-verification | 008, 010 |
| SF2-009 | `deepseek-ai/deepseek-harness` | `master@cd5ef8148158c3a752a658978873241fdf8e2bbc` | root `LICENSE`: MIT | HIGH_VALUE_HARNESS_REFERENCE / SELECTIVE_PORT candidate | 004, 008, 010 |
| SF2-010 | `braintrustdata/agentbehavior` | `main@1866cffb530c93412719b7d3e243612a11bedf97` | root `LICENSE`: Apache-2.0 | OPEN_FORMAT / EVAL_REFERENCE / compatibility candidate | 010 |
| SF2-011 | `braintrustdata/autoevals` | `main@b0e1055892bea1305a10f8d42fdc47ff1b41ffa4` | root `LICENSE`: MIT | SCORER_REFERENCE / SELECTIVE_SEMANTIC_PORT candidate | 010 |
| SF2-012 | `braintrustdata/bash-agent-evals` | `main@a13ca02330fdd4f000ca7ad5e8a3b6958afd27b8` | root `LICENSE` not found and `package.json` inspected without a license field | BEHAVIOR/BENCHMARK_REFERENCE_ONLY pending rights clarification | 005, 010 |

**Permission posture**: Golam's existing founder permission attestation applies to founder-supplied/researched sources, but the repository constitution still requires exact per-source scope/evidence plus license/dependency/security closure before any bounded code is admitted.

---

## 3. SF2-001 — Mem0

### Inspected behavior

Current project material describes a memory system with:

- single-pass ADD-only fact extraction;
- hash deduplication;
- vector semantic retrieval;
- BM25 keyword retrieval;
- entity extraction/linking and entity-aware retrieval boost;
- temporal metadata extraction and temporal-aware ranking;
- user/agent/app/run scoping;
- asynchronous memory processing;
- public memory evaluation on LoCoMo, LongMemEval and BEAM;
- explicit reporting of context-token budgets and retrieval latency.

The current architecture description separates vector/entity stores and describes an SQL history/rolling-message store in the managed memory design.

### High-value Golam lessons

1. **ADD-first evidence** reduces information loss from premature consolidation.
2. **Temporal metadata at write time** makes later historical/current-state queries better than timestamp-only retrieval.
3. **Multi-signal retrieval** is stronger than vector-only retrieval across heterogeneous query classes.
4. **Scope defaults matter**; cross-user/agent/run leakage must fail closed.
5. **Memory evaluation must include tokens and latency**, not recall alone.
6. **BEAM-scale evaluation** is more revealing than small-window memory demos.

### Required Golam divergence

Mem0's current project description treats agent-generated facts as first-class memories. Golam MUST NOT translate that into equal epistemic authority. A model/agent assertion may become a `MemoryCandidate`, but canonical project/user memory requires governed promotion and retains provenance/taint/verification state.

Golam also keeps Markdown + SQLite canonical; Mem0/vector/entity stores are not canonical truth.

### Candidate reuse posture

Prefer semantic reimplementation in Rust. Direct code reuse is only worth considering for narrowly bounded algorithms/tests after exact component qualification; no Python runtime or managed Mem0 service becomes mandatory.

---

## 4. SF2-002 — LangGraph

### Inspected behavior

The project presents itself as low-level orchestration for long-running stateful agents, with:

- durable execution;
- checkpoint persistence;
- human-in-the-loop interrupts/resume;
- short- and long-term memory integrations;
- graph/state abstractions;
- replay and retry machinery;
- a dedicated checkpoint conformance suite.

The checkpoint conformance package validates required and optional provider capabilities such as round trips, metadata preservation, namespace isolation, incremental writes, thread deletion/copy/prune and delta history.

### High-value Golam lessons

1. Provider semantics should be enforced by **shared conformance suites**.
2. Human interrupt/resume is a durable runtime state, not only a UI prompt.
3. Durable graph orchestration is useful for workers and long-running flows.
4. Required versus optional provider capabilities should be machine-testable.

### Required Golam divergence

- LangGraph checkpoint state MUST NOT replace Golam's canonical event/effect ledger.
- `checkpoint says node complete` MUST NOT imply an irreversible external effect safely committed.
- Resume MUST re-check current policy/lease/approval/live state.
- Python/LangGraph is not a trusted-path dependency.

### Candidate reuse posture

Behavior and conformance-pattern reference first. Any direct bounded port requires future Source Foundry qualification; an external compatibility adapter may be useful for developers but is not core runtime architecture.

---

## 5. SF2-003 — LangChain

### Inspected behavior

LangChain provides common interfaces and a broad integration ecosystem around:

- model providers;
- embedding providers;
- tools/toolkits;
- retrievers/vector stores;
- agent/application composition;
- LangGraph/LangSmith ecosystem integration.

### High-value Golam lessons

The main value to Golam is **ecosystem pressure testing**: a stable product needs replaceable seams because providers/tools/retrievers change continuously.

### Required Golam divergence

- no LangChain agent loop in the trusted/core path;
- no provider abstraction may own Golam authority semantics;
- no Python dependency required for strict-local core;
- interoperability should prefer native Rust adapters, MCP/ACP/HTTP/OpenAPI, then sandboxed optional bridges.

### Candidate reuse posture

Reference and compatibility only unless an exceptionally bounded component proves worthwhile.

---

## 6. SF2-004 — Qdrant

### Inspected behavior

The Rust project supports:

- dense vectors;
- sparse vectors;
- multivectors/late-interaction-style representations;
- rich payload filters;
- hybrid query fusion such as RRF/DBSF;
- relevance tuning/MMR;
- quantization and on-disk storage;
- WAL/durability;
- distributed server operation;
- **Qdrant Edge**, a lightweight local/in-process mode for edge/offline use.

The inspected `lib/edge/Cargo.toml` reports package `edge` version `0.1.0` and depends on Qdrant's internal BM25, segment, shard, sparse and WAL components.

### High-value Golam lessons

Qdrant Edge is the strongest candidate in this wave for a Rust-native local derived retrieval index because it can support semantic + sparse/hybrid retrieval without requiring a separate remote service.

### Admission concerns

Before Spec 005 can admit it, requalify:

- whether Edge is published/stable for intended embedding use or remains internal/early;
- dependency closure and compile footprint;
- unsafe/FFI/network behavior;
- persistence and crash semantics;
- encryption/privacy expectations;
- migration/upgrade compatibility;
- memory/disk/latency versus SQLite FTS and simpler local alternatives;
- whether in-process or sidecar isolation is preferable.

### Binding boundary

Qdrant is a **derived index**. Deleting/corrupting it must not lose canonical memory, and it cannot mint canonical source IDs, modify memory authority, clear taint or authorize effects.

---

## 7. SF2-005 — Firecrawl

### Inspected behavior

Current project material targets an agent-ready web-context layer with:

- web search;
- scrape to Markdown/HTML/JSON/screenshot;
- interaction/action over pages;
- crawl;
- site URL map;
- batch scraping;
- autonomous data-gathering agent;
- media/document parsing;
- MCP/skill access.

### High-value Golam lessons

Golam should make **clean, attributable web evidence** a first-class capability rather than returning giant raw DOM/page dumps. Search/crawl/map/extract/interact should be separate typed operations with explicit limits and evidence.

### License/architecture constraint

The inspected repository root license is AGPL-3.0. Default Golam posture is therefore:

- behavior reference;
- optional external adapter when user explicitly enables network access;
- no direct server-code import into Golam core without a deliberate reciprocal-license and architecture decision.

### Required Golam divergence

- native/local browser+HTTP path remains available;
- Firecrawl is never a strict-local prerequisite;
- web output stays tainted/untrusted;
- citations bind to captured evidence;
- egress and secrets remain kernel-governed.

---

## 8. SF2-006 — LlamaIndex

### Inspected behavior

The OSS framework describes:

- connectors for APIs, files, PDFs, documents, SQL and other sources;
- ingestion and structured indexing;
- retrievers/query engines;
- reranking/postprocessing;
- citation query patterns;
- large integration ecosystem;
- separate optional managed LlamaParse/document-agent services.

### High-value Golam lessons

Useful decomposition for the Context Compiler:

```text
source adapter
 -> parse/segment
 -> canonical/captured evidence objects
 -> index
 -> retrieve
 -> postprocess/filter
 -> rerank/fuse
 -> cite
 -> synthesize
```

### Required Golam divergence

- Golam evidence provenance/authority/taint rules wrap every stage;
- LlamaIndex Python is not core runtime;
- managed parsing services remain explicit optional network adapters;
- indexing cannot become canonical memory.

---

## 9. SF2-007 — Hermes Agent

### Inspected behavior

Current Hermes project/docs include:

- CLI/TUI plus Telegram/Discord/Slack/WhatsApp/Signal gateway continuity;
- voice memo transcription;
- model/provider switching;
- persistent curated `MEMORY.md` and `USER.md`;
- bounded prompt-resident memory;
- SQLite FTS5 session search;
- memory write-approval option;
- background self-improvement review;
- skill creation/improvement;
- learning-journey timeline with memory/skill correction;
- cron scheduling;
- subagents;
- multiple execution backends;
- command approval, hardline blocklist and sandbox-related security controls.

### High-value Golam lessons

1. **Small always-on memory + deep on-demand session search** is a practical layered UX.
2. A visible **Learning Journey** gives users correction/control over what the agent learned.
3. Learning reviews can convert repeated lessons into memory/skill candidates.
4. One gateway/session continuity across channels is important product UX.
5. Skills are procedural memory and should have lifecycle/version semantics.

### Security divergence

Hermes documentation currently allows some messaging approval flows through free-form affirmative replies and has modes that can disable broad approval checks. Golam MUST NOT copy those semantics for consequential protected effects.

PA-001 remains stronger:

- channel content is not native device trust;
- free-form `yes`/emoji/voice does not authorize high-risk effects;
- no global always-allow for irreversible effects;
- hard kernel denial and bounded RUN_PREAUTHORIZATION remain authoritative.

### Candidate reuse posture

High-value behavior/source donor for future bounded qualification, especially session search, learning UX, skill lifecycle and gateway patterns. Python implementation is not a trusted-core dependency.

---

## 10. SF2-008 — autoresearch

### Inspected behavior

`program.md` defines an intentionally small autonomous experiment loop:

- create a fresh run branch;
- fixed immutable evaluator/data-prep file;
- exactly one mutable main file;
- frozen dependencies;
- establish baseline first;
- fixed five-minute experiment budget;
- record metric/resource outcome;
- keep improvements, discard regressions, record crashes;
- continue autonomously until manually stopped.

The README says the project is MIT, but this research pass did not find a root `LICENSE` file. Therefore direct code reuse remains rights-verification-gated despite the README statement.

### High-value Golam lessons

The core idea is not model training; it is **bounded autonomous optimization under an immutable evaluator**.

Golam can generalize this to prompt/harness/context/retrieval/skill/model-routing experiments with explicit mutable scope, worktree/sandbox, fixed budgets, guardrails and keep/discard evidence.

### Required Golam divergence

- no uncontrolled infinite mutation of protected/privileged kernel code;
- no candidate may modify its evaluator/guardrails unless the ExperimentProgram explicitly creates a separately reviewed meta-experiment;
- adoption is separate from experiment success;
- multi-objective quality/safety/cost/complexity replaces single-score reward hacking.

---

## 11. SF2-009 — DeepSeek Harness

### Inspected behavior

The project is a developer-preview TypeScript/Node agent harness using Cordis. Architecture material describes:

- an `everything-is-a-plugin` composition model;
- services, typed events and reversible registrations;
- profiles/bundles/patch layers;
- append-only session events;
- separate durable session events and live agent/capability events;
- turn/step lifecycle;
- scoped capability seams;
- model/tool/session/agent replaceability;
- `MODEL_VISIBLE => LOGGED`;
- subagent/team extension points.

The safety notice states that the project is experimental, not security audited/production-ready, can execute model-generated commands/plugins, and that sandbox/approval controls do not guarantee isolation.

### High-value Golam lessons

- explicit Turn/Step lifecycle;
- one durable event stream as model-history source;
- separate durable facts from live extension events;
- capability seam = service definition + provider + consumer;
- profile composition for replaceable runtime capabilities;
- tool execution pipeline interception;
- configuration dumpability/inspectability.

### Binding Golam divergence

DeepSeek Harness explicitly says there is no privileged core to patch. Golam rejects this for authority.

Golam's privileged kernel is deliberately **not a plugin** and cannot be replaced through a profile/bundle/patch mechanism. Replaceability begins only outside the authority boundary.

---

## 12. SF2-010 — Braintrust Agent Behavior

### Inspected behavior

The project proposes an open `BEHAVIOR.md` convention for specifying desired agent conduct across a whole trajectory. Example structure can cover intent, evidence, decision, execution, recovery and failure modes. Behavior evaluation is meant to supervise process, not only final output.

### High-value Golam lessons

This maps directly to Golam's long-horizon verification principle. GolamBench should be able to say:

> Final output was correct, but the trajectory failed because it used stale evidence / skipped verification / violated approval rules / fabricated a PASS claim.

### Required boundary

A behavior file is instruction/evaluation data, not policy or capability authority. Malicious behavior text cannot authorize a protected action.

### Candidate posture

Support format compatibility where practical, while keeping Golam's canonical BehaviorSpec record and integrity/provenance rules.

---

## 13. SF2-011 — Braintrust Autoevals

### Inspected behavior

Autoevals packages multiple scoring approaches:

- model-graded scorers;
- factuality/safety/summary/RAG metrics;
- heuristic metrics such as exact/Levenshtein/JSON diff;
- statistical metrics;
- custom scorers;
- compact score objects;
- optional Braintrust logging/gateway integration.

### High-value Golam lessons

A scorer should be a typed replaceable object with explicit inputs, expected values, score, metadata and failure behavior.

### Required Golam divergence

- deterministic/execution-grounded evidence outranks LLM judges where possible;
- model judge configuration is recorded exactly;
- cloud Braintrust/gateway is optional, not required;
- judge score alone cannot close a security/release gate.

---

## 14. SF2-012 — Braintrust Bash vs SQL vs Embeddings eval

### Inspected behavior

This repository compares multiple agent evidence-access strategies on the same dataset/questions/model:

- sandboxed Bash;
- filesystem tools;
- SQLite/SQL;
- embedding search;
- later code-mode variants in current scripts.

Its question set includes categories such as aggregation, text reasoning, cross-entity, multi-hop, temporal, negation and semantic analysis, with explicit reference-answer validation.

### High-value Golam lesson

Do not choose the Context Compiler retrieval route by fashion. Benchmark routes on equal corpora, models and budgets.

### Rights caveat

No root `LICENSE` was found during this pass, and the inspected `package.json` did not declare one. Treat code as behavior/reference-only unless a later rights record resolves the intended license or separate permission scope.

---

## 15. Cross-source synthesis

### 15.1 Memory stack

Best combined architecture:

```text
Hermes-style compact high-value prompt memory
        +
SQLite FTS/session search baseline
        +
Mem0-style ADD-first candidate extraction + temporal/entity metadata
        +
Qdrant-style optional hybrid derived index
        +
Golam authority/provenance/taint/promotion governance
```

This is stronger than any one source because it preserves user ownership and historical evidence while keeping retrieval fast and rebuildable.

### 15.2 Context stack

```text
LlamaIndex-style source/retriever/postprocess decomposition
        +
LangChain-style replaceable provider pressure
        +
Braintrust route-comparison benchmarks
        +
Firecrawl-class web evidence capabilities
        +
Golam authority/freshness/permission/taint/sufficiency compiler
```

### 15.3 Harness/worker stack

```text
DeepSeek Harness turn/step/event/capability seams
        +
LangGraph durable graph/interrupt/checkpoint conformance
        +
Golam event/effect/goal ledger and privileged kernel
```

The kernel/effect ledger is the part the donor frameworks do not get to replace.

### 15.4 Learning/eval stack

```text
Hermes learning review + learning journey
        +
autoresearch bounded experiment loop
        +
Braintrust BehaviorSpec + scorer patterns
        +
Golam governed promotion / capability / exact-head verification
```

---

## 16. Future Source Foundry entry gates

Before any exact component from this wave is copied, ported, vendored or added as a dependency, the owning spec MUST record:

1. exact commit/tree/version at implementation time;
2. exact selected files/crates/packages;
3. permission scope/evidence and license/notices;
4. dependency and generated/vendored-code closure;
5. network/telemetry/update behavior;
6. credential/secret behavior;
7. unsafe/FFI/native-process boundaries;
8. canonical-state and migration implications;
9. platform support evidence;
10. sandbox/egress needs;
11. independent Golam conformance tests;
12. benchmark evidence against a simpler native alternative;
13. why direct reuse is better than semantic reimplementation or an adapter.

Until all applicable gates close, the status remains reference/candidate rather than `ADMITTED`.