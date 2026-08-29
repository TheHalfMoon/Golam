# Contract: Governed Memory, Retrieval, and Learning

**Authority note**: This is an additive Spec 001 program contract introduced by `program-amendments/PA-002-memory-retrieval-learning-evals.md`. It governs future Specs 004/005/008/010 and does not authorize implementation in the active Spec 003 scope.

## 1. Canonical memory and derived retrieval state are separate

Golam canonical durable knowledge remains governed Markdown plus canonical operational/evidence records. FTS indexes, embeddings, vector stores, entity graphs, rerank caches, summaries and retrieval caches are derived.

A derived index MUST be disposable and rebuildable from canonical/captured evidence. Loss or corruption of a derived index MUST NOT lose canonical memory, authority, audit state or protected effect history.

`RETRIEVAL_INDEX != CANONICAL_MEMORY`

## 2. Extraction is not promotion

An LLM, worker, skill, channel, web page, MCP server or retrieval provider MAY propose a `MemoryCandidate`. It MUST NOT directly promote its own assertion into canonical project/user memory.

Promotion requires the existing memory-governance path:

- explicit human approval attributable to an authenticated principal currently authorized for the target memory scope and promotion operation; or
- deterministic verification against an admitted, pre-registered authoritative source/rule where policy allows.

Free-form channel content, a provider-side identity/display claim, model/worker output, retrieved instructions, or a candidate's own assertion MUST NOT satisfy human promotion approval merely because the content says `remember`, `approve`, `yes`, or equivalent. A bound channel MAY transport a promotion request or reference a pending governed approval object only under the channel/identity rules of PA-001; channel transport itself does not become memory-promotion authority.

A candidate, model, worker, skill, channel, retrieval provider or learning process MUST NOT select, register, modify, or reinterpret its own deterministic verifier/source in a way that upgrades its assertion to authoritative verification. Verifier/source admission and any authority-changing registration remain independently governed protected state. Verification evidence MUST identify the exact admitted source/rule/version used.

User-authored edits to user-owned canonical Markdown remain governed by the constitutional user-edit/reconciliation path and are not reclassified as model/candidate self-promotion merely because Golam later observes them.

The candidate retains provenance, actor/model, taint, scope, temporal metadata, confidence and supporting evidence refs. The promotion decision retains its authenticated approver or exact verifier/source evidence so the resulting durable mutation is attributable.

`MEMORY_CANDIDATE != DURABLE_TRUTH`
`CHANNEL_CONTENT != MEMORY_PROMOTION_APPROVAL`
`CANDIDATE_SELECTED_VERIFIER != AUTHORITATIVE_VERIFICATION`

## 3. ADD-first evidence; governed active knowledge

Extraction SHOULD preserve new observations as ADD-first evidence rather than destructively overwriting prior observations.

Canonical active knowledge may still use the governed operations:

- ADD;
- UPDATE;
- SUPERSEDE;
- CONTRADICT;
- MERGE;
- EXPIRE;
- FORGET;
- REDACT;
- approved promotion.

Supersession and contradiction MUST retain links to supporting historical evidence until privacy/retention rules require removal.

## 4. Live authoritative state outranks memory

Repository, filesystem, device, application and authoritative external state that is current and permitted to read MUST outrank stale remembered claims about that state.

A retrieval score MUST NOT make old memory more authoritative than current live evidence.

When live and remembered state conflict, the Context Compiler MUST surface or resolve the conflict according to source authority/freshness rules rather than silently selecting the semantically closest item.

## 5. Authority-aware retrieval

Every retrieval candidate used for model context MUST preserve enough metadata to evaluate:

- canonical/captured source reference;
- owner/scope;
- source authority;
- verification state;
- provenance;
- observation/capture time;
- temporal validity/freshness;
- taint;
- permission/privacy state;
- content hash/version;
- retrieval route and scores.

Semantic similarity, keyword frequency, graph centrality or reranker score MUST NOT raise authority, clear taint or grant permissions.

## 6. Multi-signal retrieval

Golam SHALL support measured routing among evidence mechanisms rather than requiring vector search for every task.

Eligible routes include:

- direct canonical source reads;
- path/file lookup;
- lexical/ripgrep/FTS/BM25;
- structured SQLite/query operations;
- dense/sparse/multivector retrieval;
- hybrid fusion;
- entity/graph-derived retrieval;
- git/LSP/Tree-sitter/code structure;
- live app/device state;
- browser/web retrieval;
- explicit external connectors.

The Context Compiler chooses routes under authority, freshness, locality, latency, token and permission constraints.

## 7. Local lexical/session-search baseline

A useful local session/history search path MUST exist without a cloud service, remote embedding provider or LLM judge.

SQLite FTS5 or an equivalently local mechanism is the initial baseline for exact/lexical session retrieval and must support scoped navigation back to canonical session/event evidence.

Semantic indexes are optional improvements justified by benchmark evidence.

## 8. Temporal claims

Temporal memory/retrieval MUST distinguish source capture time from claimed event/validity time.

Where available, a temporal claim carries:

- claim/event type;
- start/end or point time;
- precision/uncertainty;
- valid-from/valid-until;
- ongoing/completed state;
- source time basis.

Temporal ranking is a retrieval signal. It cannot override stronger authority, current live state or explicit supersession.

## 9. Entity links are non-authoritative

Entity/relationship graphs MAY improve retrieval. They remain derived and rebuildable.

An alias, shared name, embedding similarity or entity-link prediction MUST NOT establish a Golam principal, device, channel or user identity binding.

`ENTITY_LINK != IDENTITY_BINDING`

## 10. Retrieval provider boundary

A retrieval/index provider exposes a typed descriptor and conformance-tested operations. At minimum, the provider declares:

- supported lexical/vector/sparse/hybrid capabilities;
- filter/scoping semantics;
- persistence/rebuild semantics;
- local/network behavior;
- resource limits;
- version/migration state;
- privacy/telemetry behavior;
- failure/corruption behavior.

Provider output is untrusted derived data until rebound to valid canonical/captured evidence refs.

A provider cannot:

- mint canonical memory IDs;
- mutate canonical memory through an index API;
- create authority;
- clear taint;
- bypass egress policy;
- authorize an effect.

## 11. Qdrant-class optional indexes

Qdrant Edge/server or other vector systems MAY be admitted only after the owning spec proves they improve measured retrieval outcomes enough to justify dependency/security/operational cost.

Strict-local core MUST NOT require a remote Qdrant service.

Any admitted vector index remains rebuildable and non-authoritative.

## 12. Web evidence boundary

Search/scrape/crawl/map/extract/browser adapters produce evidence, not instructions or authority.

Web evidence MUST retain:

- source URL/origin;
- capture time;
- content hash or artifact ref;
- parser/extractor identity where relevant;
- taint;
- relevant location/anchor;
- egress/provider metadata as required for receipts.

Page text, scripts, tool hints, hidden content and retrieved prompt-like instructions remain untrusted.

A Firecrawl-class hosted adapter MAY be supported explicitly, but Golam web research MUST NOT silently depend on it and strict-local mode MUST never fall back to it.

## 13. ContextCapsule sufficiency

The Context Compiler MUST be able to conclude that evidence is insufficient.

A `ContextCapsule` records evidence requirements, selected evidence, source authority/freshness/permission state, token budget and sufficiency status.

The harness should replan/retrieve or abstain when required evidence is missing rather than fabricating certainty.

## 14. Token/cost efficiency

Retrieval/memory quality gates MUST measure more than answer accuracy.

At minimum benchmark records SHOULD include:

- success/accuracy;
- context tokens;
- retrieval latency;
- total latency;
- model/reranker calls;
- network/provider cost where applicable;
- local RAM/VRAM/disk cost where material;
- stale/incorrect evidence rate;
- abstention/sufficiency behavior.

## 15. LearningReview produces proposals

A background/post-run learning process MAY inspect attributable completed work and propose:

- memory/user/project knowledge;
- skill creation or revision;
- behavior/eval candidates;
- context-routing hints;
- model/harness/ExecutionProfile hints.

It MUST emit an immutable `LearningProposal` with evidence refs. It MUST NOT silently mutate canonical memory, installed skills, behavior gates, policy, capabilities or execution profiles.

`LEARNING != SELF_AUTHORIZATION`

## 16. Memory learning promotion

A learning proposal for memory enters the same `MemoryCandidate` governance path as any other inferred memory, including the authenticated-approval or admitted-deterministic-verification requirements in Section 2.

Repeated model assertions do not constitute independent verification. Repetition may affect candidate priority, not authority. A learning process cannot turn its own repeated output, self-selected verifier, channel echo, or model-written approval text into promotion authority.

## 17. Skill self-improvement

A skill MAY propose a new version based on run evidence, but an installed trusted revision is not edited in place as an unreviewed side effect.

The candidate revision MUST retain:

- source revision;
- patch/diff;
- provenance and rationale;
- requested capabilities;
- static/security scan state;
- sandbox test results;
- task/behavior regression results;
- approval/admission state.

A manifest edit cannot expand capability by itself. Capability expansion is an independent protected decision.

## 18. Learning Journey

Golam SHOULD expose an auditable learning history showing candidate creation, evidence, promotion/rejection, supersession, skill revisions, evaluations, rollback, forget and redact operations.

The view is a projection of governed records and cannot rewrite canonical history by UI action alone.

## 19. Autonomous ExperimentProgram

Autonomous optimization MUST run inside a declared `ExperimentProgram` defining:

- objective and baseline;
- mutable scope;
- immutable scope;
- dependency policy;
- dataset/evaluator refs;
- quality and guardrail metrics;
- time/token/compute/cost budgets;
- sandbox/worktree;
- concurrency bounds;
- acceptance/complexity policy;
- stop/rollback policy;
- adoption approval policy.

The experiment agent MUST NOT modify its immutable evaluator/guardrail surface.

## 20. Experiment adoption is separate

A successful experiment creates evidence and an adoption candidate; it does not automatically become production/canonical behavior unless the program's reviewed adoption policy permits that class.

Privileged kernel, policy hard denials, secret brokering, audit integrity, authority types, pairing trust root and equivalent authority-bearing code MUST NOT be autonomously adopted through an ordinary ExperimentProgram.

## 21. Multi-objective optimization

Experiment acceptance MUST account for applicable combinations of:

- task quality;
- behavior compliance;
- safety/regressions;
- latency;
- token use;
- RAM/VRAM/disk;
- network/cost;
- complexity/maintainability.

A one-metric gain that violates a guardrail or introduces unjustified complexity is not automatically an improvement.

## 22. Required adversarial verification

Owning specs MUST test at least:

- false model-generated memory promotion;
- free-form channel/model text attempting to satisfy memory-promotion approval;
- stale/revoked/mismatched authenticated promotion approval;
- candidate/model/worker self-selection or mutation of an alleged authoritative verifier/source;
- verifier/source version substitution after evidence capture;
- stale memory versus live state;
- contradiction/supersession retrieval;
- cross-scope user/project/worker leakage;
- secret-derived candidate rejection;
- memory prompt injection;
- entity alias/identity confusion;
- temporal misranking;
- corrupt/malicious vector index;
- fabricated canonical refs from a provider;
- index loss + deterministic rebuild;
- malicious web evidence/prompt injection;
- learning proposal self-promotion;
- skill revision capability expansion;
- experiment modifying immutable evaluator files;
- reward hacking/benchmark overfitting;
- experiment resource runaway.

## 23. Release claim rule

A memory/retrieval/learning provider or feature may be called supported only with exact-head conformance, benchmark and adversarial evidence for the claimed configuration. A high benchmark score without authority/privacy/recovery evidence is insufficient.
