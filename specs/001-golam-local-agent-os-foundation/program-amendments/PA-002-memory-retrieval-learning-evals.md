# PA-002 — Memory, Retrieval, Learning, Orchestration, and Evaluation Architecture

**Status**: PROPOSED_FOR_REVIEW  
**Date**: 2026-08-28  
**Founder input**: Study and absorb useful mechanisms from Mem0, LangGraph, LangChain, Qdrant, Firecrawl, Braintrust, LlamaIndex, Hermes Agent, autoresearch, and DeepSeek Harness while preserving Golam's Rust-first/local-first/security architecture.  
**Stacked planning base**: `plan/phone-channel-access@33c094d68fdd58dc9244fe3d259756685ad27444`  
**Canonical main observed before this work**: `main@82de7084384009ff3a00522f4e0aef09bf549529`  
**Implementation authorization**: NONE. This amendment strengthens future Spec 004/005/008/009/010 requirements. It MUST NOT enter the active Spec 003 implementation scope or authorize new runtime dependencies now.

---

## 1. Executive decision

Golam will not become a Rust shell around a Python agent framework.

The researched systems demonstrate strong mechanisms, but Golam keeps ownership of the semantics that make it trustworthy and durable:

- canonical session/event/effect history;
- goal and causality state;
- identity, capability, approval and egress authority;
- canonical user memory;
- model/harness semantics;
- context/evidence provenance;
- worker lifecycle;
- learning and skill revision governance;
- evaluation evidence and release claims.

External frameworks, vector engines, web services, evaluators and integration ecosystems may become **replaceable adapters, bounded donors, benchmark references or compatibility surfaces**. They never become Golam's trust root or canonical source of truth.

The resulting product direction is:

```text
                         GOLAM INTELLIGENCE PLANE

 canonical evidence                                               derived/rebuildable
 +-------------------+                                            +-------------------+
 | event/effect log  |------------------------------------------->| search indexes     |
 | Goal Ledger       |                                            | embeddings         |
 | Markdown memory   |                                            | entity graph       |
 | SQLite ops state  |                                            | summaries/caches   |
 +---------+---------+                                            +---------+---------+
           |                                                                |
           v                                                                v
 +-------------------+       +----------------------+       +-----------------------+
 | Memory Governor   |------>| Context Compiler     |------>| Harness / Worker Run  |
 | candidate/promote |       | route/retrieve/rank  |       | turn/step/tools       |
 | contradict/forget |       | evidence/sufficiency |       | interrupt/resume      |
 +---------+---------+       +----------+-----------+       +-----------+-----------+
           |                            |                               |
           |                            |                               v
           |                            |                   +-----------------------+
           |                            +------------------>| Effect Gate / Kernel  |
           |                                                +-----------------------+
           v
 +-------------------+       +----------------------+       +-----------------------+
 | Learning Review   |------>| Learning Proposal    |------>| Eval / Behavior Gate  |
 | memory/skill/etc. |       | immutable candidate  |       | regressions/trajectory|
 +-------------------+       +----------------------+       +-----------+-----------+
                                                                        |
                                                                        v
                                                             approve/verify/promote
```

---

## 2. Source set and high-level disposition

Exact source evidence and licenses are recorded in `PA-002-source-foundry-research.md` and the donor register. The binding architectural disposition is:

| Source | Golam use | Default code posture |
|---|---|---|
| `mem0ai/mem0` | memory extraction/retrieval/scoping/temporal/entity benchmark patterns | semantic/behavior port; no managed service dependency |
| `langchain-ai/langgraph` | durable graph, interrupt/resume, checkpointer conformance | semantic port/conformance reference; no canonical persistence delegation |
| `langchain-ai/langchain` | provider/tool/retriever integration vocabulary and ecosystem pressure test | ecosystem reference; no core framework dependency |
| `qdrant/qdrant` | local dense/sparse/multivector/hybrid derived index candidate | optional Rust dependency/sidecar candidate after Spec 005 qualification |
| `firecrawl/firecrawl` | web search/scrape/crawl/map/extract/interact behavior target | external adapter + behavior reference by default; no hidden service dependency |
| Braintrust (`agentbehavior`, `autoevals`, relevant eval repos) | trajectory behavior specs, scorer/eval organization, comparative harness experiments | open-format compatibility / semantic port; cloud optional |
| `run-llama/llama_index` | ingestion, connector, retrieval, rerank, citation and document-agent patterns | semantic port; optional external adapters only |
| `NousResearch/hermes-agent` | curated memory, session search, learning journey, skill improvement, messaging continuity, scheduler/subagent UX | high-value behavioral donor; Golam security rules dominate |
| `karpathy/autoresearch` | bounded autonomous experiment loop, fixed budget, baseline/keep/discard discipline | high-value research pattern; no uncontrolled self-modifying kernel |
| `deepseek-ai/deepseek-harness` | turn/step/session-log, capability seams, profiles, extension composition, durable model-visible events | high-value harness reference; `everything-is-a-plugin` MUST NOT include Golam authority |

---

## 3. Memory architecture — canonical truth is not a vector database

### 3.1 Memory layers

Golam's memory architecture SHALL distinguish at least:

1. **Working context** — ephemeral model-facing context for a request/turn.
2. **Run/session memory** — temporary durable task context, recoverable but not automatically user truth.
3. **Episodic memory** — attributable events/experiences derived from canonical session/effect records.
4. **Semantic memory** — durable facts/preferences/conventions that pass memory governance.
5. **Procedural memory** — skills/workflows that pass skill revision and evaluation gates.
6. **Relationship/entity memory** — rebuildable entity links connecting canonical records.
7. **Project memory** — project-scoped durable knowledge with live repository state taking precedence.
8. **User memory** — human-readable user-owned long-lived knowledge.
9. **Worker memory** — explicitly scoped worker knowledge that cannot silently become user/project truth.
10. **Shared team memory** — later explicitly shared scope; never inferred from unrelated workers or channels.

Markdown remains the canonical human-readable long-lived representation. SQLite remains canonical operational state. Vector, BM25, entity and graph indexes are derivatives.

### 3.2 MemoryCandidate pipeline

Golam SHALL separate **extraction** from **promotion**.

```text
canonical events / user edits / authoritative source
        |
        v
candidate extraction
        |
        v
normalize + provenance + taint + temporal metadata
        |
        v
deduplicate / similarity / entity linking
        |
        v
conflict + supersession analysis
        |
        +----> reject / expire / keep run-scoped
        |
        v
human approval OR deterministic authoritative verification
        |
        v
canonical memory mutation
        |
        v
rebuild/update derived indexes
```

An LLM may propose memory candidates. It cannot promote its own assertions to durable truth.

### 3.3 ADD-first evidence, governed canonical consolidation

Mem0's current ADD-only extraction is useful because it avoids erasing historical state prematurely. Golam adopts the **evidence-preservation property**, not an unconditional append-only user-memory UX.

Rules:

- extracted observations are ADD-first evidence records;
- old observations are not deleted merely because a newer model extraction conflicts;
- canonical memory may SUPERSEDE, CONTRADICT, MERGE, EXPIRE, FORGET or REDACT according to the existing memory governance contract;
- supersession preserves provenance links to the older evidence;
- current-state retrieval must prefer valid/live/superseding knowledge while still allowing historical queries;
- deletion/privacy requests operate on canonical active knowledge and derivatives according to existing FORGET/REDACT semantics.

### 3.4 Authority-aware memory

Memory retrieval SHALL rank **authority and validity before convenience**.

A memory item includes or derives:

- origin/source refs;
- actor/principal;
- observation time;
- event time or validity interval;
- confidence;
- verification state;
- authority class;
- taint labels;
- supersession/contradiction links;
- owner and scope;
- live-state reference when applicable.

Agent-generated facts MUST NOT receive equal authority merely because the agent said them confidently. An agent completion can be evidence that the agent made a statement or performed an audited action; it is not automatically evidence that an external fact is true.

### 3.5 Temporal memory

Future Spec 005 SHALL model time explicitly, including:

- `observed_at`;
- `event_time` or interval;
- precision/uncertainty;
- `valid_from` / `valid_until` when known;
- state type such as event, ongoing state, plan, preference, relationship, absence/negative evidence;
- supersession relationship;
- query temporal intent.

Temporal scoring is a retrieval signal, not a mechanism for overriding stronger source authority.

### 3.6 Entity linking

Entity extraction/linking may improve retrieval, but the entity graph is rebuildable and non-authoritative.

- entity aliases never establish identity/authority by themselves;
- channel/user/device identity remains governed by stable identity contracts;
- entity links carry source provenance;
- graph corruption must be recoverable by rebuilding from canonical evidence;
- graph support remains benchmark-justified rather than becoming a mandatory graph database dependency.

---

## 4. Multi-signal retrieval and the Context Compiler

### 4.1 Retrieval is a routing problem

Golam SHALL NOT assume embeddings are the correct retrieval mechanism for every task.

The Context Compiler chooses among evidence routes such as:

- exact path/file reads;
- ripgrep/lexical search;
- SQLite/structured query;
- FTS/BM25;
- dense vector search;
- sparse vector search;
- hybrid fusion;
- entity/graph traversal;
- git/history/LSP/Tree-sitter;
- live application state;
- browser/web evidence;
- external connectors when policy permits.

Routing is based on query/task class, source authority, freshness, privacy, latency/token budget and measured benchmark performance.

### 4.2 Retrieval stages

```text
intent
  -> evidence requirements
  -> candidate source classes
  -> permission/locality/authority filter
  -> source-specific retrieval in parallel where justified
  -> lexical/vector/entity/temporal/structured scoring
  -> fusion
  -> optional rerank
  -> diversity/dedup
  -> provenance + citation binding
  -> sufficiency test
  -> replan if insufficient
  -> ContextCapsule
```

### 4.3 Retrieval evidence

Every retrieved item SHOULD retain:

- canonical source/artifact ref;
- source URI/path/provider;
- content hash/version;
- capture/fetch timestamp;
- parser/extractor identity/version;
- location/range/anchor;
- source authority class;
- permission/privacy state;
- taint;
- retrieval route;
- component scores;
- final fused/rerank score;
- freshness/validity state.

The model sees compact evidence, while the receipt/audit system retains enough references to reproduce what evidence was supplied.

### 4.4 Token efficiency is a first-class metric

Memory and context benchmarks MUST report at least:

- task success/accuracy;
- context tokens delivered;
- retrieval latency;
- total latency;
- index memory/disk cost;
- reranker/model cost if external;
- stale/incorrect evidence rate;
- abstention/sufficiency behavior.

A larger context window is not automatically a better memory system.

---

## 5. Retrieval-index provider boundary

### 5.1 Canonical/derived separation

A retrieval index MUST be destroyable and rebuildable without losing canonical Golam knowledge or authority state.

Index write APIs cannot mutate canonical memory. Index query output is untrusted derived data and is re-bound to canonical artifact/evidence identifiers before model use.

### 5.2 Baseline and candidates

Future Spec 005 SHALL benchmark at least:

- SQLite FTS5/structured local baseline;
- Golam-owned simple local vector baseline if available without unjustified complexity;
- Qdrant Edge as a serious Rust local/offline candidate;
- Qdrant server only as an optional explicit local/remote adapter if justified.

Qdrant Edge is promising because it is local, Rust, and exposes dense/sparse/hybrid capabilities, but its exact current crate state, maturity, dependency closure, unsafe/FFI posture, storage behavior and upgrade semantics MUST be qualified before admission.

### 5.3 Index compromise model

A compromised or corrupt retrieval index can cause bad ranking but MUST NOT:

- mint authority;
- modify canonical memory;
- clear taint;
- fabricate a canonical source identifier;
- bypass live-state checks;
- authorize an effect.

---

## 6. Web intelligence and Firecrawl-class capabilities

Golam's web subsystem SHALL target the useful behavior class demonstrated by modern web-context systems:

- search;
- fetch/scrape;
- structured extraction;
- clean Markdown/text extraction;
- screenshot/DOM evidence;
- URL map/discovery;
- bounded crawl;
- batch retrieval;
- interactive browser actions when needed;
- PDF/document extraction when safe;
- research synthesis with source citations.

### 6.1 Native first, adapter optional

Golam core should own a local-first web evidence path using ordinary HTTP/browser mechanisms and the existing semantic-first computer/browser control hierarchy.

Firecrawl may be exposed as an optional explicit network adapter when useful. It MUST NOT become a hidden prerequisite for web research or strict-local operation.

### 6.2 License and architecture posture

The inspected Firecrawl repository is AGPL-3.0. Default Golam posture is therefore behavior reference/external adapter. Any direct code reuse requires explicit Source Foundry license/redistribution closure and an intentional decision about reciprocal obligations.

### 6.3 Web evidence safety

Web content is hostile input.

- fetched text remains web-tainted;
- JavaScript/page instructions do not become agent authority;
- extracted structured data retains source provenance;
- credentials are brokered outside page/model context where possible;
- network access passes the egress gate;
- browser interaction remains effect-gated as appropriate;
- citations bind to captured source evidence, not only model-written URLs.

---

## 7. Harness architecture — absorb seams without surrendering authority

DeepSeek Harness demonstrates useful separation of session events, live agent events, capability seams, model adapters, tool registries, profiles and bundles. Golam adopts these ideas only outside the privileged authority boundary.

### 7.1 Two extension zones

```text
IMMUTABLE / PROTECTED AUTHORITY ZONE
  identity
  capabilities/leases
  policy hard denials
  approvals
  effect gate/journal/reconcile
  secrets
  egress
  pairing
  audit integrity
        |
        | typed requests / decisions
        v
REPLACEABLE RUNTIME EXTENSION ZONE
  model adapters
  harness strategies
  context providers
  retrieval/index providers
  tools
  MCP/skills
  browser/control adapters
  UI surfaces
  evaluators
  worker strategies
```

`everything-is-a-plugin` is explicitly rejected for the authority zone.

### 7.2 Turn/step/request-series model

Future Spec 004 SHALL make these concepts explicit:

- **Turn** — user/worker-triggered unit that may contain zero or more model/tool steps.
- **Step** — one model request plus resulting tool-call cycle.
- **RequestSeries** — sequence with stable model/profile/tool-schema envelope where useful for cache/retry semantics.
- **InboxItem** — queued user/worker/system input with causality and interruption priority.
- **Interrupt** — durable pause requiring an external event/decision before continuation.
- **Continuation** — resumable state reference; never an authority token.

`MODEL_VISIBLE => LOGGED` remains binding. Model-visible history is a projection of canonical events, not an independent mutable chat buffer.

### 7.3 Adapter conformance suites

Golam SHALL build conformance suites for replaceable providers, inspired by LangGraph's checkpointer conformance approach.

Candidate suites:

- model adapter conformance;
- tool adapter conformance;
- retrieval/index provider conformance;
- checkpoint provider conformance;
- channel adapter conformance;
- sandbox backend conformance;
- evaluation scorer conformance.

A provider can advertise optional capabilities, but all required base semantics must pass the shared suite before the provider is considered supported.

---

## 8. Durable worker graphs — LangGraph semantics on Golam evidence

Future Spec 008 may expose durable workflow/agent graphs, but the graph is a **projection/orchestration description** over Golam's canonical event/effect model.

### 8.1 WorkerGraph model

A graph may contain:

- deterministic computation nodes;
- model/harness nodes;
- tool/retrieval nodes;
- worker/subagent nodes;
- human interrupt/approval nodes;
- wait/timer/event nodes;
- branch/join nodes;
- verification/evaluation nodes.

Node completion is durably recorded. External effects remain separate EffectIntents and cannot be considered committed merely because a graph checkpoint says a node ran.

### 8.2 Durable execution is not effect correctness

This distinction is binding:

```text
WORKFLOW CHECKPOINT != EXTERNAL EFFECT COMMIT
```

A graph may safely replay computation. It MUST NOT replay AT_MOST_ONCE/IRREVERSIBLE external effects blindly. The Effect Gate/journal/reconciler remains the authority for effect recovery.

### 8.3 Interrupt/resume

Interrupts are first-class durable states carrying:

- reason;
- required input/decision class;
- current graph/node state reference;
- expiry/deadline if any;
- allowed resumers;
- causality;
- no embedded authority expansion.

Resume rechecks current policy/leases/approvals/live state.

---

## 9. Learning loop — Golam learns, but learning is governed

Hermes demonstrates the product value of active memory curation, session search, procedural skills and background learning. Golam will build a stronger governed version.

### 9.1 LearningReview

After eligible work, Golam MAY run a bounded learning review over attributable evidence to propose:

- memory candidate;
- user preference candidate;
- project convention candidate;
- skill creation candidate;
- skill revision candidate;
- tool routing hint;
- behavior/eval candidate;
- model/harness profile hint.

The review produces **LearningProposals**, not canonical writes.

### 9.2 Promotion rules

A proposal may become canonical only through the owning governance path:

- memory -> memory governance approval/authoritative verification;
- skill -> versioned skill package + capability review + tests/evals + install approval as required;
- behavior -> behavior-spec review/eval registration;
- model/profile hint -> benchmark/calibration evidence;
- routing hint -> measured context benchmark evidence.

The learning model cannot self-certify its proposal.

### 9.3 Learning Journey

Golam SHOULD expose a user-auditable timeline showing:

- what it learned;
- source session/evidence;
- who/what proposed it;
- promotion status;
- revisions/supersessions;
- skill diffs;
- evaluation evidence;
- rollback/forget/redact actions.

This is a product surface for trust and correction, not a decorative graph.

### 9.4 Session search

A lightweight local session-search path SHOULD exist before expensive semantic memory search.

Baseline:

- SQLite FTS5 over canonical/projection-safe session text;
- scoped search by project/worker/channel/time;
- exact message/event navigation;
- no LLM call required for basic search;
- semantic search optional after measured benefit.

---

## 10. Procedural learning and skill self-improvement

Skills MAY improve from experience, but never by silently rewriting a trusted installed skill in place.

```text
run evidence
  -> skill improvement proposal
  -> candidate patch + rationale + provenance
  -> static/security/capability review
  -> sandbox test
  -> task/behavior regression suite
  -> optional human approval
  -> versioned new skill revision
  -> staged rollout
  -> rollback available
```

A skill revision cannot expand capabilities merely by editing its manifest. Capability expansion is a separate protected decision.

Repeated success is evidence for quality, not permission.

---

## 11. Autonomous experimentation — an autoresearch-inspired bounded lab

Golam SHOULD eventually support an `ExperimentProgram` for safe autonomous optimization of non-authority surfaces.

### 11.1 ExperimentProgram

A program defines:

- objective;
- fixed baseline;
- allowed mutable files/configuration surfaces;
- immutable files/surfaces;
- allowed dependencies (normally frozen);
- dataset/eval corpus;
- primary and guardrail metrics;
- time/token/compute/cost budgets;
- sandbox/worktree;
- maximum concurrent experiments;
- acceptance rule;
- complexity penalty;
- stop conditions;
- rollback rule;
- whether human approval is required before adoption.

### 11.2 Experiment loop

```text
establish baseline
  -> propose bounded mutation
  -> commit/candidate snapshot
  -> run fixed-budget experiment
  -> evaluate quality + safety + cost + complexity
  -> keep / discard / crash
  -> record immutable ExperimentRun
  -> generate next proposal
```

### 11.3 Eligible targets

Early autonomous tuning MAY target:

- prompt/harness strategies;
- context routing thresholds;
- retrieval fusion weights;
- reranking policy;
- local model parameters/configuration;
- skill implementation under a sandbox;
- summarization/compaction strategy;
- test-generation strategy;
- model routing policy;
- hardware-specific ExecutionProfiles.

It MUST NOT autonomously modify or adopt changes to the privileged kernel, policy hard denials, secret broker, audit integrity, device trust root or equivalent authority-bearing code without an explicit separately reviewed development workflow.

### 11.4 Multi-objective acceptance

Golam SHALL avoid optimizing one benchmark into an unsafe/expensive system. Candidate adoption considers a Pareto-like set of:

- task quality;
- behavior compliance;
- safety/regression gates;
- latency;
- tokens;
- memory/VRAM/RAM/disk;
- network/cost;
- complexity/maintainability.

---

## 12. Evaluation architecture — behavior over vibes

### 12.1 Evaluation objects

Future GolamBench SHALL distinguish:

- `EvalDataset`;
- `EvalCase`;
- `ReferenceEvidence`;
- `ScorerDefinition`;
- `BehaviorSpec`;
- `TrajectoryTrace`;
- `TrajectoryEvaluation`;
- `ExperimentRun`;
- `RegressionComparison`;
- `ReleaseClaimEvidence`.

### 12.2 Scorer hierarchy

Prefer, in order when applicable:

1. deterministic assertions/invariants;
2. exact structured reference checks;
3. execution/environment verification;
4. statistical/heuristic scorers;
5. model-based judges as supplementary evidence.

LLM-as-judge output is not authority. Every model judge record retains model/revision/provider/locality, prompt/rubric hash, inputs, output/rationale, score, retry/variance information and cost/token metadata.

### 12.3 Behavior specs

Golam SHOULD support compatibility with the open `BEHAVIOR.md` convention where practical.

A behavior specification describes expected conduct across a trajectory, such as:

- gather current authoritative context before acting;
- do not claim success without verification;
- re-read live repository truth after state changes;
- use approvals correctly;
- avoid stale memory;
- preserve citation/source provenance;
- recover from failed tools without fabricating results;
- stop/replan when evidence is insufficient;
- honor user interruption;
- avoid premature completion.

Behavior specs are evaluation/instruction artifacts. They cannot grant capabilities or override policy.

### 12.4 Process evaluation

Long-horizon agent evaluation MUST inspect trajectory behavior, not only final answer/task success.

A run can fail even if the final artifact looks correct when it:

- used unauthorized access;
- fabricated verification;
- used stale evidence;
- leaked secrets;
- ignored an approval denial;
- duplicated an irreversible effect;
- violated task scope;
- silently changed canonical memory;
- falsely claimed completion.

---

## 13. Context-route benchmarking

Braintrust's comparative bash/filesystem/SQL/embedding experiment reinforces a key Golam principle: retrieval route should be measured per task class.

Future Spec 005/010 SHALL create a `ContextRouteBench` with common corpus/questions and equal model/harness budgets across routes such as:

- filesystem tools;
- ripgrep/FTS;
- structured SQLite/SQL;
- vector/hybrid retrieval;
- graph/entity retrieval when available;
- combinations chosen by the Context Compiler.

Task categories SHOULD include:

- exact lookup;
- aggregation;
- text reasoning;
- cross-entity;
- multi-hop;
- temporal;
- negation;
- semantic similarity;
- code/reference navigation;
- stale/current-state conflict;
- adversarial/injection-laden sources.

Report success, tokens, calls, latency and errors. No retrieval backend becomes default because of marketing claims.

---

## 14. Memory benchmarks

Future Spec 005/010 SHOULD include public memory benchmarks where licensing/data terms permit, including:

- LoCoMo;
- LongMemEval;
- BEAM at practical scale;
- Golam-specific contradiction/staleness/security suites.

Golam-specific suites MUST add dimensions conventional memory benchmarks underweight:

- live repository state outranks stale memory;
- authority/provenance ranking;
- contradiction surfacing;
- user edit reconciliation;
- FORGET/REDACT derived-state rebuild;
- secret-derived memory rejection;
- prompt-injection survival;
- cross-worker/channel scope isolation;
- project/user promotion approval;
- crash during memory promotion/index update;
- index corruption/rebuild;
- memory poisoning attempts.

Scores must be reported with token budget and latency, not accuracy alone.

---

## 15. Provider / framework compatibility strategy

Golam SHOULD make it easy to consume the ecosystems around LangChain/LlamaIndex without importing their runtime into the authority path.

Preferred interoperability order:

1. native Golam Rust adapter for critical/high-value capability;
2. open protocol such as MCP/ACP/HTTP/OpenAPI where appropriate;
3. sandboxed sidecar/adapter;
4. Python/Node integration bridge only when optional and isolated;
5. direct framework dependency only with exceptional technical justification.

The existence of hundreds of provider integrations is evidence that adapters must be replaceable; it is not evidence that all integrations belong in the trusted core.

---

## 16. New logical entities

These are future logical model requirements, not current schema migrations.

### MemoryCandidate

- `candidate_id`
- `source_event_refs[]`
- `proposed_content`
- `memory_kind`
- `owner`
- `scope`
- `proposer_principal`
- `proposer_model?`
- `provenance[]`
- `taint_labels[]`
- `authority_class`
- `confidence`
- `observed_at`
- `temporal_claim?`
- `entity_refs[]`
- `similar_memory_refs[]`
- `contradiction_refs[]`
- `promotion_requirement`
- `status`: proposed | verified | approved | rejected | promoted | expired

### TemporalClaim

- `claim_type`: event | state | plan | preference | relationship | absence | other
- `event_start?`
- `event_end?`
- `precision`
- `ongoing_state?`
- `valid_from?`
- `valid_until?`
- `source_time_basis`

### EntityLink

- `entity_id`
- `canonical_or_local_label`
- `aliases[]`
- `linked_evidence_refs[]`
- `provenance[]`
- `confidence`
- `derived_index_version`

Invariant: `EntityLink` is not an authority identity binding.

### RetrievalIndexDescriptor

- `index_id`
- `provider`
- `version`
- `canonical_source_classes[]`
- `capabilities[]`
- `embedding_profile?`
- `sparse_profile?`
- `storage_ref`
- `built_from_checkpoint`
- `build_digest`
- `created_at`
- `last_rebuilt_at?`
- `corruption_state`

### RetrievalEvidence

- `evidence_id`
- `canonical_ref`
- `source_location`
- `content_hash`
- `captured_at`
- `retrieval_route`
- `component_scores`
- `final_score`
- `authority_class`
- `freshness_state`
- `permission_state`
- `taint_labels[]`

### LearningProposal

- `proposal_id`
- `kind`: memory | user_profile | project_knowledge | skill_create | skill_patch | behavior | routing | execution_profile
- `source_evidence_refs[]`
- `proposer`
- `candidate_artifact_ref`
- `requested_capability_delta?`
- `evaluation_requirements[]`
- `approval_requirement?`
- `status`
- `created_at`

### BehaviorSpecRecord

- `behavior_id`
- `name`
- `format_version`
- `source_ref`
- `content_hash`
- `applies_to[]`
- `evaluation_rubric_refs[]`
- `created_by`
- `approved_at?`
- `supersedes?`

### ExperimentProgram

- `program_id`
- `objective`
- `baseline_ref`
- `mutable_scope[]`
- `immutable_scope[]`
- `dependency_policy`
- `dataset_refs[]`
- `metrics[]`
- `guardrails[]`
- `resource_budget`
- `sandbox_profile`
- `acceptance_policy`
- `stop_policy`
- `adoption_approval_policy`
- `version`

### ExperimentRun

- `experiment_id`
- `program_id`
- `parent_candidate?`
- `code_or_config_digest`
- `patch_ref`
- `execution_profile`
- `environment_digest`
- `started_at`
- `completed_at?`
- `metrics`
- `resource_usage`
- `behavior_results[]`
- `safety_results[]`
- `status`: keep | discard | crash | timeout | rejected
- `adopted_at?`

---

## 17. Binding functional requirements

### Memory

- **FR-MEM-001**: Memory extraction and memory promotion MUST be separate operations.
- **FR-MEM-002**: Model/worker-generated memory candidates cannot self-promote to canonical project/user memory.
- **FR-MEM-003**: Observation evidence SHOULD be ADD-first and preserve historical contradictions/supersession.
- **FR-MEM-004**: Canonical memory SHALL retain provenance, authority, temporal validity, taint, scope and contradiction/supersession state.
- **FR-MEM-005**: Retrieval indexes/graphs/embeddings MUST be rebuildable and non-authoritative.
- **FR-MEM-006**: Memory retrieval SHALL support measured multi-signal routing/fusion rather than vector-only retrieval.
- **FR-MEM-007**: Local lexical/session search MUST remain available without a remote embedding/model dependency.
- **FR-MEM-008**: Memory quality claims MUST report accuracy/success together with token and latency budgets.

### Context / retrieval

- **FR-CTX-001**: Context Compiler SHALL select evidence routes based on task/source/privacy/budget and benchmark evidence.
- **FR-CTX-002**: Every model-visible retrieved item MUST be attributable to canonical or captured source evidence.
- **FR-CTX-003**: Retrieval scores cannot raise source authority or clear taint.
- **FR-CTX-004**: Index corruption/loss MUST be recoverable by rebuilding from canonical sources.
- **FR-CTX-005**: Web research SHALL support native local/browser evidence paths and MAY support optional Firecrawl-class adapters without hidden cloud dependency.
- **FR-CTX-006**: Web/provider content remains untrusted and passes egress/taint/evidence rules.

### Harness / orchestration

- **FR-HARNESS-001**: Turn, Step, RequestSeries, InboxItem, Interrupt and Continuation semantics MUST be explicit in the harness contract.
- **FR-HARNESS-002**: `MODEL_VISIBLE => LOGGED` remains enforced from canonical event history.
- **FR-HARNESS-003**: Runtime extension seams cannot replace or bypass privileged authority services.
- **FR-HARNESS-004**: Replaceable provider interfaces SHALL have shared conformance suites before support claims.
- **FR-WORKER-001**: Durable graph/workflow checkpoints MUST NOT be treated as proof an external effect safely committed.
- **FR-WORKER-002**: Graph resume rechecks current authority/live state and cannot revive expired approvals/leases.

### Learning / experimentation

- **FR-LEARN-001**: Background learning produces immutable proposals, not silent canonical memory/skill mutation.
- **FR-LEARN-002**: Skill improvement requires a versioned candidate, security/capability review and regression evidence before adoption.
- **FR-LEARN-003**: Learning provenance and revision history SHALL be inspectable through a Learning Journey or equivalent surface.
- **FR-LEARN-004**: Autonomous experiment programs MUST declare allowed mutable scope, fixed evaluation, budgets, guardrails and rollback.
- **FR-LEARN-005**: Autonomous optimization MUST NOT directly adopt privileged-kernel/authority changes.
- **FR-LEARN-006**: Candidate adoption evaluates quality, safety, cost/resource and complexity, not a single reward metric alone.

### Evaluation

- **FR-EVAL-001**: GolamBench SHALL support trajectory-level behavior specifications in addition to outcome metrics.
- **FR-EVAL-002**: Behavior/eval artifacts cannot grant runtime authority.
- **FR-EVAL-003**: LLM judges are evidence only and must record exact judge configuration/provenance.
- **FR-EVAL-004**: Deterministic and execution-grounded scorers are preferred where possible.
- **FR-EVAL-005**: Context-route benchmarks SHALL compare filesystem/structured/lexical/vector/graph approaches on equal task/model budgets.
- **FR-EVAL-006**: Memory benchmarks SHALL include scale, contradiction, temporal, stale-state, privacy/taint and token-efficiency dimensions.
- **FR-EVAL-007**: Every release-gating benchmark must be reproducible from exact head + dataset/evaluator/profile versions.

---

## 18. Spec ownership and roadmap changes

### Spec 004 — Harness & Local Intelligence

Add/strengthen:

- explicit turn/step/request-series/inbox/interrupt semantics;
- capability seams outside the kernel;
- adapter conformance suites;
- profile composition without allowing plugins to redefine hard authority;
- benchmark-visible model vs harness separation.

### Spec 005 — Local Tools, Context & Memory

Add/strengthen:

- MemoryCandidate pipeline;
- temporal/entity metadata;
- lexical + structured + vector + hybrid routing;
- SQLite FTS5 session search baseline;
- Qdrant Edge exact qualification/benchmark as optional derived index;
- LlamaIndex/LangChain-inspired source/retriever/postprocessor boundaries implemented independently in Rust;
- Firecrawl-class web behavior target + optional adapter;
- ContextRouteBench and memory benchmark entry gates;
- LearningProposal generation for memory/skill candidates, with adoption still governed.

### Spec 008 — Workers & Automations

Add/strengthen:

- event-sourced durable WorkerGraph;
- explicit interrupt/resume/wait/join semantics;
- graph checkpoint vs Effect Gate separation;
- bounded background LearningReview;
- versioned skill-improvement candidates;
- experiment worker primitives after single-worker durability is proven.

### Spec 009 — Public Feature Parity

Use this architecture to close:

- better cross-session memory;
- proactive learning;
- deep research/web evidence;
- durable workers;
- searchable prior sessions/artifacts;
- teach/skill improvement with stronger governance than benchmark competitors.

### Spec 010 — GolamBench & Release Qualification

Add:

- `BEHAVIOR.md` compatibility evaluation path;
- trajectory behavior suites;
- ContextRouteBench;
- LoCoMo/LongMemEval/BEAM where admissible;
- Golam memory-security/staleness benchmark;
- autonomous experiment/adoption safety gates;
- scorer/judge provenance and variance reporting.

---

## 19. Explicit rejected shortcuts

- Do not make Mem0 Cloud or any managed memory service canonical Golam memory.
- Do not allow agent-generated facts to gain user-memory authority automatically.
- Do not make LangGraph checkpoints the canonical event/effect ledger.
- Do not import LangChain/LlamaIndex as the core agent runtime merely for integrations.
- Do not make Qdrant or another vector DB canonical knowledge storage.
- Do not require Qdrant server for strict-local core operation.
- Do not copy Firecrawl server code into Golam core without explicit reciprocal-license/architecture review.
- Do not make Firecrawl or another hosted scraper a hidden prerequisite for web research.
- Do not require Braintrust cloud for GolamBench or tracing.
- Do not treat LLM-as-judge as release authority.
- Do not copy Hermes' free-form messaging approval behavior into Golam; PA-001's stronger channel approval contract remains binding.
- Do not let background memory/skill review silently modify canonical state.
- Do not run autoresearch-style indefinite mutation over privileged authority code.
- Do not adopt DeepSeek Harness' `no privileged core` model; Golam's small privileged kernel remains constitutional.
- Do not optimize solely for benchmark score while ignoring tokens, latency, privacy, resource use, complexity or safety.

---

## 20. Threat-model additions

Future owning specs MUST include adversarial tests for:

- agent-generated false memory promotion;
- poisoned memory candidate;
- stale memory outranking live state;
- entity alias collision/identity confusion;
- temporal misclassification selecting obsolete facts;
- cross-user/project/worker scope leakage;
- malicious/corrupt vector index returning fabricated refs;
- embedding inversion/data disclosure risk where relevant;
- retrieval score manipulation by prompt-injected documents;
- web crawl prompt injection and malicious file parsing;
- evaluation judge prompt injection;
- evaluator self-serving score manipulation by the candidate agent;
- behavior spec injection attempting to grant authority;
- skill self-improvement capability expansion;
- autonomous experiment changing immutable/evaluator files;
- reward hacking / benchmark overfitting;
- experiment resource runaway;
- checkpoint replay causing duplicate effects;
- plugin/provider replacement attempting to mount an authority service;
- derived graph/index deletion/corruption and deterministic rebuild.

---

## 21. Success criteria

- **SC-PA2-001**: Future Spec 005 can rebuild all lexical/vector/entity indexes from canonical Golam state with no knowledge/authority loss.
- **SC-PA2-002**: A false model-generated memory candidate cannot become canonical durable user/project memory without the configured approval/verifier path.
- **SC-PA2-003**: Memory retrieval benchmarks report quality, token budget and latency and include stale/contradictory/current-state cases.
- **SC-PA2-004**: ContextRouteBench demonstrates measured routing across at least filesystem/lexical/structured/vector strategies rather than hard-coding embeddings.
- **SC-PA2-005**: A corrupt optional vector index can be discarded/rebuilt and cannot authorize effects or mutate canonical memory.
- **SC-PA2-006**: Web research works without Firecrawl when native allowed network/browser capabilities suffice; strict-local behavior remains honest.
- **SC-PA2-007**: Durable worker replay never duplicates a protected external effect because effect reconciliation remains independent of graph checkpointing.
- **SC-PA2-008**: Background learning creates inspectable proposals and cannot silently expand skill or worker capabilities.
- **SC-PA2-009**: An autonomous experiment cannot modify its evaluator/immutable scope and cannot auto-adopt a privileged-kernel change.
- **SC-PA2-010**: GolamBench can fail an agent trajectory for behavior violations even when final output superficially succeeds.

---

## 22. Review and merge gate

Before this amendment becomes canonical:

1. verify exact source snapshots and licenses in `PA-002-source-foundry-research.md`;
2. verify this amendment is monotonic with Constitution v1.2.0 and existing memory/effect/taint/kernel contracts;
3. verify it introduces no current Spec 003 runtime/dependency work;
4. verify Qdrant/Firecrawl/Braintrust/LangGraph/LangChain/LlamaIndex remain replaceable/non-authority by design;
5. verify background learning cannot bypass memory/skill governance;
6. verify graph checkpointing cannot bypass effect durability semantics;
7. verify autonomous experimentation excludes uncontrolled privileged-kernel mutation;
8. verify future task graph explicitly assigns the work to bounded Specs 004/005/008/010;
9. record exact PR head and exact-head CI/review evidence before merge.

After merge, future agents planning Specs 004/005/008/010 MUST read this amendment before freezing their implementation design.