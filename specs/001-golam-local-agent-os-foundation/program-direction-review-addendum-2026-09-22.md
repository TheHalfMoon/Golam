# Golam Program Direction Review Addendum — 2026-09-22

**Authority**: PROGRAM DIRECTION / PLANNING ONLY — NO PRODUCT IMPLEMENTATION AUTHORITY

**Extends**:
- `program-direction-review-2026-09-13.md`
- `program-superiority-2026.md`
- `program-superiority-tasks.md`
- `program-superiority-gap-closure-tasks.md`
- `program-direction-tasks-2026-09-13.md`

**Source register**: `competitive-source-register-supplement-2026-09-22.md`

This addendum incorporates the 2026-09-22 review of Google AX, Treg, Laya, SemIf, Decider, Bespoke Nimble, current TinyFish/AgentQL, Desktop Commander and the founder-owned GitHub portfolio. It does not widen active Spec 006 and does not authorize implementation.

## 1. Executive direction

Golam should evolve from a local/private Agent OS into a **verified personal execution and decision operating system**.

The product should feel like one place where the user can:

- see what needs attention before opening ten apps;
- ask for a result rather than choose a tool/provider manually;
- delegate bounded work to local or remote execution nodes;
- let cheap local decision models handle frequent small judgments;
- escalate to powerful reasoning models only when the task requires them;
- search and understand personal/project knowledge with provenance;
- inspect exactly which account, provider, model, runtime, tool and data path will be used;
- approve consequential actions once they are fully previewable;
- interrupt or take over immediately;
- verify that work actually happened before Golam says it is complete;
- keep core operation useful without a Golam-hosted cloud.

The architectural thesis remains:

> **The smallest trustworthy local authority core, surrounded by the richest governed capability fabric.**

The product thesis becomes sharper:

> **Golam turns attention and intent into verified work across apps, code, files, web, knowledge, devices and services while keeping authority, evidence and data movement under the user's control.**

## 2. The wrong way to combine the new sources

Do not create:

```text
AX subsystem
+ Treg subsystem
+ Laya subsystem
+ SemIf subsystem
+ Decider subsystem
+ TinyFish subsystem
+ Desktop Commander subsystem
+ Morize subsystem
+ Kernux subsystem
= Golam
```

That would create:

- duplicate Task identities;
- duplicate event stores;
- duplicate approval paths;
- duplicate tool registries;
- duplicate secret systems;
- duplicate egress policy;
- duplicate worker lifecycle;
- duplicate memory/RAG state;
- conflicting retry semantics;
- multiple notions of "done";
- large dependency and update surfaces;
- impossible security reasoning.

The correct synthesis is to extract contracts and provider behaviors, then make them consume Golam's canonical spine.

## 3. Target architecture: five governed fabrics around one core

```text
┌───────────────────────────────────────────────────────────────┐
│                    EXPERIENCE / ATTENTION                     │
│ Inbox · Tasks · Workspaces · Actions · Knowledge · Routines  │
│ Briefings · Search · Chat · Voice · Capability Catalog       │
└──────────────────────────────┬────────────────────────────────┘
                               │ sanitized projections / intent
                               ▼
┌───────────────────────────────────────────────────────────────┐
│                         HARNESS                               │
│ TaskContract · Workflow/Delivery Graph · Workers · Routing    │
│ Context compiler · Checkpoints · Decision orchestration       │
└───────────────┬───────────────────────┬───────────────────────┘
                │                       │
                ▼                       ▼
┌───────────────────────────┐  ┌───────────────────────────────┐
│      DECISION FABRIC      │  │      CAPABILITY EXCHANGE      │
│ typed bounded judgments   │  │ search/get/select exact offer │
│ SemIf/Decider/Nimble/...  │  │ provider/account/cost/egress  │
└───────────────┬───────────┘  └──────────────┬────────────────┘
                │ advisory                     │ candidate route
                └───────────────┬───────────────┘
                                ▼
┌───────────────────────────────────────────────────────────────┐
│                    AUTHORITY + EVIDENCE CORE                  │
│ Principal · Lease · Policy · Secret Broker · Egress          │
│ Effect Gate · UNKNOWN_OUTCOME · Reconciliation · Verification│
│ Source Foundry · Audit/Integrity                             │
└──────────────────────────────┬────────────────────────────────┘
                               │ authorized typed dispatch
                               ▼
┌───────────────────────────────────────────────────────────────┐
│                      EXECUTION FABRIC                         │
│ local host · process/PTy · browser · app · documents         │
│ sandbox/container/VM · SSH/remote · optional cluster         │
│ workspace materialization · readiness · suspend/resume        │
└──────────────────────────────┬────────────────────────────────┘
                               │ observations/artifacts
                               ▼
┌───────────────────────────────────────────────────────────────┐
│                  CONTEXT / KNOWLEDGE FABRIC                   │
│ sources · Morize-style governed memory · retrieval            │
│ temporal truth · evidence graph · ContextBundle               │
└───────────────────────────────────────────────────────────────┘
```

The boxes are not independent security roots. The Authority + Evidence Core is the only place where protected authority and consequential Effect truth live.

## 4. Fabric A — Execution Fabric

### 4.1 Why AX is valuable

AX correctly models an agent workload as something with:

- desired state;
- isolated runtime;
- reusable workspace materialization;
- explicit network boundary;
- model configuration;
- readiness conditions;
- suspend/resume;
- controller reconciliation.

This maps well to Golam's existing T127/T151/T153/T167/T173 work.

### 4.2 Golam execution primitive

Define an `ExecutionEnvelope` or `WorkloadManifest` as a declarative binding around an existing `TaskContract`.

Candidate fields:

```text
execution_envelope_id
task_contract_revision
workspace_bindings[]
execution_backend_ref
isolation_profile_ref
compute_requirements
resource_budget
egress_policy_ref
execution_profile_ref
capability_set_ref
secret_handle_refs[]
readiness_conditions[]
observable_endpoints[]
checkpoint_policy
suspend_resume_policy
timeout_policy
cleanup_policy
artifact_output_policy
```

It must be possible to render this into:

- a local process;
- a local isolated process;
- a container;
- a VM/microVM;
- WSL where supported;
- an SSH/runtime node;
- a user-owned compute node;
- an optional future cluster backend.

The local profile must remain first-class. Kubernetes is a backend option, not product architecture.

### 4.3 Reconciliation

Workers should have desired/observed state:

```text
DESIRED: RUNNING
OBSERVED: STARTING / READY / DEGRADED / SUSPENDED / LOST / FAILED
```

A reconcile loop may try to restore desired runtime state but it cannot replay consequential external Effects blindly. Runtime reconciliation and Effect reconciliation remain separate.

```text
RUNTIME_RECONCILIATION != EFFECT_RETRY
TASK_READY != TASK_VERIFIED_COMPLETE
```

### 4.4 Workspace semantics

A Workspace binding is materialization metadata, not authority.

A workspace may specify:

- exact Git repo/revision/tree;
- selected local folders;
- artifact snapshots;
- skills/extensions;
- MCP capability projections;
- environment setup recipe;
- required tools;
- cache identities.

The runtime still receives only capabilities granted to the Task/Worker principal.

### 4.5 Debug and interactive access

AX-style SSH/debug service and Desktop Commander-style process sessions are useful, but debug surfaces are privileged capabilities.

Requirements:

- explicit debug capability;
- exact target runtime;
- authenticated client;
- visible session;
- bounded output;
- terminal escape sanitization at projection;
- audit;
- revoke/kill;
- no implied widening of filesystem/network grants.



### Reconciliation must be fenced, not merely retried

AX-style desired/observed reconciliation is useful only if a replaced runtime cannot keep acting as a valid actor. Every reconciled execution instance therefore needs a monotonic runtime generation or equivalent fencing identity bound into dispatch authorization.

A resumed/reprovisioned worker must receive a fresh execution incarnation. Stale incarnations may finish local computation, but they must fail closed at the protected action boundary if their generation is no longer current.

```text
RUNTIME_RESTART != SAME_EXECUTION_INCARNATION
STALE_WORKER != CURRENT_EFFECT_ACTOR
RECONCILER_CONVERGENCE != AUTHORITY_CONTINUITY
```

This is especially important when a crash occurs around an external side effect: reconciliation may recreate compute, but it must reconcile the existing Effect before allowing a conflicting retry.

## 5. Fabric B — Capability Exchange

### 5.1 Product goal

A user should be able to say:

> "Find the company, verify its domain, enrich the founders, compare three providers, and spend no more than $0.20."

The user should not need to know which vendor or MCP server implements each step.

The Capability Exchange answers:

```text
What can do this?
Which exact provider/account would be used?
Is it local or remote?
What data would leave?
What does it cost?
What side effects can it cause?
Is it currently admitted and healthy?
What stronger/safer alternative exists?
```

### 5.2 Canonical objects

Candidate objects:

```text
CapabilityDefinition
CapabilityOffer
ProviderRevision
AccountBinding
CredentialBindingRef
CapabilityAvailability
CapabilityQualification
ToolCallPlan
```

`CapabilityDefinition` is provider-neutral semantics.

`CapabilityOffer` says one exact provider/revision can currently satisfy that capability under stated constraints.

`ToolCallPlan` is a proposed exact call, not authority.

### 5.3 Search versus call

The public/tool-facing surface should stay compact:

```text
capability_search(query, constraints)
capability_get(capability_or_offer_id)
capability_call(exact_offer_id, operation, arguments)
```

MCP/agent clients should not receive 3,000 dynamically changing tool schemas.

This preserves:

- stable client context;
- predictable safety annotations;
- lower tool-selection confusion;
- one authorization/effect path.

### 5.4 Provider selection

Selection order is not simply cheapest-first.

Hard compatibility gates:

1. authority/capability compatibility;
2. privacy profile;
3. strict-local rule;
4. data-class/egress compatibility;
5. account/credential availability;
6. required operation/effect semantics;
7. provider qualification/freshness;
8. budget ceiling.

Optimization may then consider:

- quality;
- reliability;
- latency;
- cost;
- local resource pressure.

A fallback may never silently widen privacy, authority, provider scope or spend.

### 5.5 Unknown tools

For an arbitrary private tool or relay where Golam cannot prove semantics:

```text
operation_class = OPEN_WORLD
side_effect_class = UNKNOWN_OR_CONSEQUENTIAL
idempotency = UNKNOWN
retry = NO_BLIND_RETRY
verification = EXPLICIT
```

This is safer than inventing read/write semantics from a tool name.

### 5.6 Credential brokering

Use Golam's existing secret broker.

A provider may require multiple bindings:

- OAuth bearer;
- API key header;
- developer token;
- query credential;
- signed request material.

Bindings are applied only for an exact authorized destination/provider/account.

The model and caller receive opaque handles, never secret values when brokering is possible.

## 6. Fabric C — Decision Fabric

### 6.1 Why this matters

A modern agent wastes latency, tokens and money when every binary/enum judgment invokes a generative reasoning model.

Examples suitable for a System-1 path:

- priority: low/normal/high/critical;
- select one provider among qualified offers;
- classify request into a bounded queue;
- decide whether evidence is sufficient / insufficient / contradictory;
- select a workflow branch;
- estimate risk band;
- decide whether to escalate from exact search to broader retrieval;
- classify whether an incoming event needs immediate attention;
- choose among legal UI targets after deterministic candidate binding.

### 6.2 Provider-neutral contract

Golam should not choose SemIf versus Decider versus Nimble globally. It should qualify them by workload.

`DecisionProvider` is a replaceable capability.

Potential decision types:

```text
CHOICE
BOOLEAN / NOUL
ORDERED_SCORE
MULTI_LABEL only when independently qualified
```

Free-form text generation belongs to a reasoning/generative model path, not this contract.

### 6.3 Calibration is workload-specific

A probability only has operational meaning when evaluated for the relevant workload and configuration.

Qualification should record:

- accuracy/balanced accuracy as appropriate;
- NLL/log loss;
- Brier score;
- ECE/reliability;
- AURC/selective risk;
- abstention coverage;
- domain shift;
- perturbation robustness;
- option-order invariance;
- schema-size sensitivity;
- context-length behavior;
- backend/quantization drift;
- latency/resources.

No global rule such as "0.9 means safe."

### 6.4 Escalation ladder

```text
deterministic rule/exact fact
    -> local DecisionProvider
    -> larger local DecisionProvider
    -> generative reasoning model
    -> independent verifier / authoritative source
    -> human review where required
```

The ladder is driven by task need, uncertainty, privacy and budget.

A high-confidence decision model never replaces a deterministic authoritative check when one exists.

### 6.5 Security ceiling

Decision providers may:

- prioritize;
- recommend;
- raise risk;
- request more evidence;
- select among already-authorized bounded routes.

They may not:

- mint capabilities;
- lower consequence class;
- authorize egress;
- reveal secrets;
- satisfy owner presence;
- certify source truth;
- certify `VERIFIED_COMPLETE`;
- turn a denial into allow.



### 6.6 Cross-field consistency and bounded decision graphs

Typed decision providers often score fields independently. Golam must not assume that independently plausible answers are jointly valid.

A future Decision Fabric should support deterministic cross-field constraints and small dependency graphs:

```text
DecisionRequest
  -> independent typed scores
  -> abstention/applicability checks
  -> deterministic constraint validation
  -> contradiction/impossible-state detection
  -> escalation when constraints cannot be satisfied safely
```

Examples include route/provider choices that conflict with a strict-local profile, a "safe to retry" answer that conflicts with an at-most-once Effect class, or several classifications that cannot all be true simultaneously.

```text
FIELD_PROBABILITY != JOINT_CONSISTENCY
HIGH_CONFIDENCE_FIELD != VALID_DECISION_SET
MODEL_CONSISTENCY != POLICY_CONSISTENCY
```

## 7. Fabric D — Proactive Attention

### 7.1 Move beyond chat

Chat remains useful, but a personal Agent OS should not require the user to remember every task.

Golam should have an Attention surface that combines:

- incoming connector events;
- scheduled obligations;
- worker blockers;
- expiring approvals;
- unknown outcomes;
- high-confidence actionable opportunities;
- security/privacy warnings;
- pending decisions;
- daily/weekly briefing.

### 7.2 Attention pipeline

```text
Connector observation
 -> canonical ingest receipt
 -> normalization
 -> trust/taint classification
 -> exact/entity linkage
 -> optional bounded DecisionProvider classification
 -> ContextBundle/research
 -> AttentionItem projection
 -> optional ActionProposal
```

Every stage binds exact source revisions.

### 7.3 Action Proposal

An `ActionProposal` is user-facing precomputed intent:

```text
proposal_id
source_attention_item_ids[]
task_ref
target
account
provider
route
operation/effect class
proposed payload/diff
data leaving device
cost
irreversibility/retry class
evidence/context
verification plan
expiry/freshness
```

User approval is an authenticated approval input. The kernel still checks current target, capability lease, data/egress policy and effect freshness immediately before dispatch.

### 7.4 Briefings

Daily/periodic summaries must expose:

- time window;
- included sources;
- omissions/offline connectors;
- unresolved contradictions;
- stale data;
- action items;
- which statements are direct observations versus synthesized interpretation.

The briefing is a projection, not canonical history.

### 7.5 Coherence

Cross-app entity coherence should use Morize-style evidence-linked identity/relation semantics.

A Jira ticket, GitHub PR, Slack thread and email may be linked into one narrative, but a model-generated link remains a candidate until rules/evidence satisfy the relation policy.

User link/unlink corrections may generate versioned relation/routine candidates.



### 7.6 Attention is a scarce user resource

A proactive system can fail by being technically correct too often. Golam should treat interruption cost as a governed product resource.

Attention ranking should account for urgency, expected user value, freshness, confidence/applicability, duplicate/related items, current user focus, quiet policy and whether an item can be safely deferred into a briefing. The system should expose why an item surfaced and support user correction.

Release evaluation should measure not only recall of important items, but unnecessary interruption rate, duplicate surfacing, stale-card rate, deferred-item recovery and correction learning without silent policy mutation.

```text
ATTENTION_SCORE != USER_PRIORITY_TRUTH
HIGH_MODEL_CONFIDENCE != INTERRUPT_NOW
PROACTIVE != ALWAYS_INTERRUPTIVE
CORRECTION_SIGNAL != ACTIVE_POLICY_MUTATION
```

## 8. Fabric E — Context and Knowledge

Do not create another Laya-style RAG database in Golam.

Use the existing direction:

- source artifacts remain canonical evidence;
- Morize-style governed memory owns durable remembered propositions;
- retrieval derivatives are rebuildable;
- ContextBundle discloses sources, omissions and budget;
- temporal/contradiction semantics remain first-class;
- external text remains untrusted instructions;
- optional vectors/reranking only after measured need.

For cross-application retrieval, use a layered approach:

```text
exact IDs/entities
-> metadata/time/scope
-> FTS/BM25
-> relation graph
-> optional local embeddings
-> optional reranker
-> synthesis
```

Reciprocal Rank Fusion may be a useful retrieval fusion technique where benchmarked, but it is not canonical truth.

## 9. Internal donor portfolio strategy

The founder-owned portfolio already contains several capabilities that external sources now reinforce.

### Use Kernux as the primary execution pattern

Do not integrate AX, Desktop Commander and TinyFish directly into three separate Golam runtimes.

Desired relationship:

```text
Golam canonical contracts
  -> execution/capability provider contract
     -> native Golam/Kernux-derived provider
     -> AX-style backend where scale warrants
     -> Desktop Commander-compatible adapter where useful
     -> TinyFish/AgentQL browser semantic adapter
```

### Use Morize semantics for memory/context

Do not create separate Golam memory, Laya memory and RAG memory.

### Use Ascout/Winds/Diffcipline for proof

Every capability/provider decision should inherit:

- exact subject/revision identity;
- negative evidence;
- independent verification;
- explicit unknown;
- prove-before-done.

### Use MESC/MSTR/commandMed for model qualification

Decision providers must be measured like infrastructure, not selected by README claims.

### Use commandF patterns for package identity

Source/tool/model/extension artifacts should be digest-bound and machine-reportable.

## 10. Product information architecture

The current six durable objects remain good:

1. Tasks
2. Sessions
3. Workers
4. Actions
5. Knowledge
6. Routines

Add two top-level *views*, not new authority roots:

7. **Attention**
8. **Capabilities**

Suggested desktop navigation:

```text
Attention
Work
  Tasks
  Sessions
  Workers
Actions
Knowledge
Routines
Capabilities
Activity / Evidence
Settings
```

### Attention

The place for:

- incoming items;
- blockers;
- proposals;
- briefings;
- unknown outcomes;
- pending user decisions.

### Work

Execution lifecycle and live worker steering.

### Actions

Consequential Effect timeline, preview, reconciliation and verification.

### Knowledge

Sources, memory, context, contradictions and provenance.

### Routines

Versioned approved automation/workflow definitions and rehearsals.

### Capabilities

Searchable local/connector/provider capability catalog with exact locality, account, price, permissions, data path, qualification and availability.

## 11. "Ask for the outcome" flow

A north-star interaction:

> "Research the three best enterprise prospects in Riyadh, verify the decision makers, prepare personalized outreach, but do not send anything and spend less than $1."

Golam should:

1. create a TaskContract with explicit no-send constraint and budget;
2. discover qualified search/enrichment capabilities;
3. present provider/data-egress/cost constraints if the policy requires a choice;
4. use bounded local decision models for cheap routing/classification;
5. run research workers in isolated ExecutionEnvelopes;
6. build source-linked evidence;
7. produce draft outreach ActionProposals;
8. stop before external send Effects;
9. verify research and artifact criteria;
10. show one coherent result with provenance, cost and unresolved uncertainty.

The user interacts with the goal, not the internal provider graph.

## 12. "Handle my morning" flow

1. approved connectors ingest new messages/tickets/calendar events;
2. exact IDs and relation graph link known projects/entities;
3. a local DecisionProvider handles priority/persona/bounded triage;
4. stronger models research only high-value ambiguous items;
5. Attention shows a compact prioritized briefing;
6. ActionProposals pre-draft responses;
7. sends/comments/changes remain Effect-gated;
8. user can approve, edit, defer, reject or turn a pattern into a RoutineCandidate.

This provides Laya-like proactive value without inheriting Laya as a second authority/runtime architecture.

## 13. Zero-cost-to-founder and local-first implications

The default Golam architecture should not depend on founder-funded variable infrastructure.

Default execution should prefer:

- user device;
- user local models;
- user's existing subscriptions/CLI agents;
- user's API accounts;
- user's remote machines;
- organization's own infrastructure.

Optional managed services can be commercial and separately metered later.

Capability Exchange economics must not silently cause Golam to fund third-party calls on behalf of users.

```text
CATALOG_DISCOVERY != FOUNDER_FUNDED_PROVIDER_CREDITS
LOCAL_PRODUCT_ENTITLEMENT != RUNTIME_AUTHORITY
SUBSCRIPTION_STATE != CAPABILITY_GRANT
```

## 14. Reliability architecture

### Every provider has availability, not assumed presence

A capability provider should expose:

```text
AVAILABLE
DEGRADED
UNAVAILABLE
NOT_CONFIGURED
NOT_AUTHORIZED
NOT_QUALIFIED
STALE
```

Unknown is not Available.

### Every fallback is attributable

A route transition must record:

- why preferred provider was unavailable/inapplicable;
- whether privacy/cost/authority changes;
- current target/account;
- user approval if required.

### Every long-running worker is resumable

Checkpoint binds:

- TaskContract revision;
- ExecutionEnvelope revision;
- workspace/source revisions;
- exact active capability/provider revisions;
- current Effect uncertainty;
- current context/evidence;
- pending verification obligations.

Resume revalidates live truth.

## 15. Security additions from this source pass

### Capability catalog poisoning

Threat: malicious or stale capability metadata causes routing to wrong provider/endpoint.

Controls:

- signed/admitted provider identity where applicable;
- exact revision;
- destination allowlist;
- Source Foundry/conformance reference;
- observed capability probes;
- catalog metadata cannot override kernel Effect mapping.

### Credential confusion

Threat: wrong account/credential injected into a semantically correct tool.

Controls:

- AccountBinding is explicit;
- credential handle binds provider/account/purpose;
- immediate revalidation;
- UI preview shows account;
- cross-account fallback requires policy/approval.

### Decision-model overtrust

Threat: fast classifier becomes hidden policy engine.

Controls:

- deterministic authority ceiling;
- calibration/applicability evidence;
- no deny->allow conversion;
- abstain/escalate support;
- no generic confidence threshold.

### Proactive automation drift

Threat: learned card/routing rules gradually create autonomous behavior beyond original intent.

Controls:

- corrections create candidates;
- immutable version activation;
- rehearsal;
- effect ceiling;
- periodic review;
- stale version invalidation.

### Workspace bootstrap supply-chain

Threat: a declarative workspace goal installs arbitrary code before task start.

Controls:

- materialization is itself governed;
- dependency/download/network capabilities explicit;
- exact artifacts/digests where practical;
- setup worker authority narrower than later task;
- resulting workspace snapshot recorded;
- no secret ambient inheritance.

## 16. What not to import

Do not adopt as Golam defaults:

- AX's Kubernetes/Redis requirement;
- Treg's independent identity/billing/effect truth;
- Treg platform-owned shared third-party credentials as default product economics;
- Laya's n8n/Python/Chroma stack as privileged architecture;
- SemIf/Decider/Nimble output as authorization;
- TinyFish stealth/anti-bot behavior;
- Desktop Commander client-trust security posture;
- thousands of dynamic MCP tool schemas;
- a mandatory vector database;
- hidden cloud research/browser fallback;
- a second event store for Attention;
- a second memory truth system;
- model-generated completion as verification.

## 17. Roadmap impact

### Spec 006

No change. Finish Desktop Computer Control under its current contract.

### Spec 007

No change to its core purpose. Cross-device identity/transport remains distinct from the new capability and decision fabrics.

### Spec 008

Workers & Automations should consume the future ExecutionEnvelope/reconciliation contract. AX lessons belong here or in a bounded prerequisite package after live authority is verified.

### Spec 009 umbrella

The previous decomposition remains correct, but the shared foundation must now also include:

- Decision Provider Contract;
- Capability Catalog/Offer Contract;
- ExecutionEnvelope contract;
- proactive Attention projections.

These are shared contracts or bounded packages; they are not permission to turn Spec 009 into a mega-spec.

### Spec 010

Benchmark/release qualification should add:

- decision calibration/selective-risk tests;
- capability discovery precision/recall and wrong-provider tests;
- credential/account confusion tests;
- cost/egress fallback tests;
- workload reconciliation/resume tests;
- Attention false-positive/false-negative and stale-source tests;
- compact MCP catalog conformance;
- end-to-end "ask for outcome" journeys.

## 18. Recommended dependency order

```text
Existing constitutional / canonical prerequisites
  T112
  T149/T150
  T156/T177/T179
  T165/T180/T197
  T167
  T199
  T201

Then shared new contracts
  T203 Decision Provider Contract
  T206 Capability Catalog / Offer Contract

Then qualification + execution bindings
  T204 Decision Calibration / Escalation
  T205 ExecutionEnvelope / Reconciliation
  T207 Credential-Brokered Tool Relay

Then product projections
  T208 Proactive Attention / Action Proposals
  T209 Cross-Source Coherence / Briefing
  T210 Compact MCP Capability Surface

Portfolio governance remains cross-cutting
  T211 Owner Portfolio Reuse Matrix
```

No new task is implementation authority by itself.

## 19. North-star acceptance properties

Golam's future architecture is coherent only if all of the following can be true simultaneously:

- the user can complete valuable work with networking disabled;
- a local decision model can be removed without breaking authority semantics;
- Treg/TinyFish/Desktop Commander-compatible providers can be removed without corrupting canonical state;
- a worker can move from local process to VM/SSH/cluster backend without changing Task/Effect semantics;
- capability discovery can show remote providers without making them authorized;
- an Attention card can disappear and be rebuilt from canonical source state;
- knowledge indexes can be deleted and rebuilt;
- an unavailable preferred provider cannot silently widen egress or spend;
- no model, UI, extension or provider can mint authority;
- no ambiguous external Effect is blind-retried;
- no completion claim becomes verified without VerificationReceipts;
- exact account/provider/model/runtime/source identities remain inspectable.

## 20. Updated product thesis

A concise product statement for future design work:

> **Golam is the local-first Agent OS that watches what matters, understands your context, chooses the right governed capability, delegates work to the right runtime, and proves the result before calling it done.**

A technical statement:

> **Golam is a user-owned authority and evidence kernel with replaceable execution, capability, decision, context and experience providers.**

## 21. Planning disposition

```text
DEEP_DIRECTION_REVIEW_2026_09_22=COMPLETE
EXTERNAL_SOURCE_SYNTHESIS=COMPLETE
OWNER_PORTFOLIO_SYNTHESIS=COMPLETE
ACTIVE_SPEC_006_PR_24_WIDENED=NO
CURRENT_POINTER_CHANGED=NO
CONSTITUTION_CHANGED=NO
NEW_PRODUCT_IMPLEMENTATION_STARTED=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
WAIVER_TAKEN=NO
```


## 11. Owner-portfolio deep dive follow-up

A second authenticated deep dive compared all 36 owner repositories against the current Golam planning corpus and searched specifically for source patterns not already represented by T110–T212. It found four material architecture gaps plus two important refinements.

### 11.1 Evidence needs capability, fidelity, capture and absence semantics

Golam already has strong Effect/Verification semantics, but "no evidence" is not precise enough for a multi-provider Agent OS. The evidence layer must distinguish:

- what a provider/runtime could theoretically expose;
- what the current adapter actually implements;
- whether observation/capture was active for the run;
- which predicates were directly observed;
- which claims are deterministic derivations;
- what was unsupported, unavailable, failed, partial or genuinely not observed.

The system must not render "we could not observe this" as "this did not happen", nor "the scanner emitted no finding" as "the subject is clean".

This becomes T213 and extends the existing Evidence/Verification owners rather than introducing a new evidence database or canonical finding authority.

### 11.2 Delegation needs a disclosure receipt, not only a ContextBundle

A bounded worker should know exactly what was disclosed to it, and any later mutation proposal should be tied back to that disclosure. The safe pattern is:

```text
canonical state
-> ContextDisclosureReceipt(exact objects/revisions/omissions/budget/audience)
-> worker reasoning
-> WorkerProposal(disclosure_receipt_id + expected revisions)
-> canonical owner/reviewer transition
-> ordinary Golam Effect/Verification path
```

Context disclosure exports no capability, approval, lease or secret. A worker that learned an object ID elsewhere must not thereby acquire proposal authority over that object. Proposal origin and owner acceptance remain separate evidence. An explicit reviewed-through checkpoint also prevents a UI open/read event from being misrepresented as human review.

This becomes T214.

### 11.3 Skill evolution needs an IR/replay/repair spine before broad self-improvement

The current skill-evolution direction is strengthened into four distinct stages:

```text
typed Skill/Workflow IR
-> authority-free candidate compilation
-> deterministic/low-model replay with fresh authorization
-> divergence detection + localized repair + downstream invalidation
```

The IR records required capabilities, side effects, artifacts/dataflow, assumptions, pre/postconditions and verifiers; it never captures live demonstration-time grants or secrets. Repair produces a new candidate/version and invalidates affected evidence rather than rewriting prior history.

A schema-valid or compilable generated skill is not automatically semantically faithful to the originating user intent. Semantic-faithfulness evidence and safe abstention are part of qualification.

This becomes T215 and refines T122/T123/T162/T163/T194 rather than creating a second workflow system.

### 11.4 Verification needs orthogonal dimensions and EvidenceBundles

One green status is too coarse for consequential effects and durable artifacts. A future VerificationReceipt/EvidenceBundle projection should allow independent dimensions such as:

- exact input/source and output artifact identity;
- target/account and backend/route identity;
- authorization/approval binding;
- Effect terminal/reconciliation state;
- constraint/postcondition satisfaction;
- freshness/time basis;
- coverage/fidelity;
- verifier independence;
- provider attestations;
- unresolved/unsupported dimensions.

The owning VerificationObligation defines which dimensions are mandatory. Mandatory UNKNOWN or missing dimensions block `VERIFIED_COMPLETE`.

This becomes T216 and extends T149/T150; it is not a new truth ledger.

### 11.5 Execution approval must bind the actual workload

The portfolio review strengthened T205: a user approving a command label or requested sandbox profile is insufficient if the executable/workload can change before dispatch. Consequential process execution should bind approval and Effect dispatch to exact workload/artifact identity where observable, workspace/source revision, execution incarnation, backend identity, requested confinement and trusted observed-confinement evidence.

```text
APPROVED_COMMAND_NAME != APPROVED_EXECUTABLE_BYTES
SANDBOX_LABEL != CONFINEMENT_PROOF
BACKEND_CAPABILITY != OBSERVED_CONFINEMENT
```

### 11.6 Voice stop is a control path, not a transcription result

T196 is strengthened so an emergency interrupt can mute audible output and request cancellation without waiting for final ASR transcription. The later transcript is evidence/content, not the authority to stop.

```text
FINAL_TRANSCRIPT != STOP_PREREQUISITE
```

### 11.7 Portfolio reuse discipline

The deep dive did not justify copying whole applications into Golam. Reuse disposition remains component-level:

```text
ADOPT_CONTRACT
PORT_OR_ADAPT_BOUNDED_COMPONENT
REFERENCE_METHOD
BENCHMARK_ONLY
NO_CURRENT_MEASURED_GAP
```

Public sources with measured gaps are recorded in the source supplement. Private source identities remain undisclosed in this public repository; useful patterns may inform planning, but any actual private code reuse still requires an authorized exact-component Source Foundry record with publishable provenance decided separately.

```text
PORTFOLIO_DEEP_DIVE_COMPLETED=YES
NEW_PARALLEL_AUTHORITY_SYSTEM=NO
ACTIVE_SPEC_006_PR_24_WIDENED=NO
NEW_PRODUCT_IMPLEMENTATION_STARTED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
```


## 22. Voice/audio and personal-agent deep dive

A focused follow-up reviewed the current OpenJev Hub artifact, the owner-pinned Golam-research/Grok Bot reconstruction, Wispral, Himsat and Meta's newly launched Muse product/safety architecture. The result is a stronger voice direction than the original T140/T196 "optional real-time voice" umbrella.

### 22.1 Product goal: speak naturally, work continuously

Golam should support a conversation where the user can:

- speak or type into the same durable work/conversation spine;
- hear low-latency spoken responses;
- interrupt Golam while it is speaking;
- add a second task before the first response finishes;
- steer/cancel/queue/branch long-running work;
- leave while background work continues and return to a transparent activity/task state;
- keep strict-local voice useful without network access;
- use deterministic approval controls for consequential actions.

This is not a turn-by-turn voice chatbot. It is conversational control over the same Task/Effect/Evidence system.

### 22.2 OpenJev belongs behind speech, not in place of speech

The reviewed `AlexWortega/openjev` artifact is a Qwen3.5-derived text-classification/NLI decision model, not an STT/TTS/VAD engine. Its useful Golam role is after textual/contextual state exists:

```text
audio/acoustic signals
  -> streaming STT revisions
  -> bounded transcript/context state
  -> optional T203/OpenJev-style decisions
       command vs aside
       question vs dictation
       semantic turn completeness hint
       urgency/routing/clarification hint
  -> deterministic policy + conversation/task handling
```

It may improve latency/cost for bounded semantic decisions, but it never authenticates the speaker, replaces acoustic endpointing, authorizes an Effect or proves that a partial transcript is final user intent.

### 22.3 Grok Bot reconstruction is a baseline to surpass

The owner-pinned Golam-research reconstruction contains a recovered voice controller with:

- explicit microphone permission acquisition;
- MediaRecorder capture with bounded duration;
- cancellation/abort handling;
- agent/account session scoping;
- whole-clip transcription after recording stops;
- transcript-card and deterministic permission/auto-review UX.

Those are valuable bounded components/patterns. The whole-clip `record -> stop -> transcribe -> send` path is not sufficient for Golam's target. Golam needs incremental STT, typed turn events, low-latency playback, barge-in and independent cancellation across audio and agent lifecycles.

### 22.4 Wispral supplies the correct voice control-plane shape

Wispral's strongest architecture lessons are retained directly:

- voice is typed events/state transitions, not one `audio -> text -> shell` function;
- microphone, STT, TTS, agent requests and rendering are independently cancellable;
- provenance preserves source audio/non-persistence marker, raw transcript, normalized transcript, entity binding, semantic class, final instruction and authorization separately;
- `COMMAND` and `ASIDE` are semantic concepts behind deterministic policy;
- speech engines are replaceable;
- push-to-talk is the founding reliability baseline;
- full-duplex/hands-free/AEC are later independently qualified capabilities;
- exact timestamps make latency claims measurable.

### 22.5 Himsat supplies the runtime and benchmark substrate

Himsat contributes the stronger runtime decomposition:

```text
capture + capture health
  -> conditioning / AEC / NS / AGC
  -> VAD / segmentation
  -> Voice Runtime Router
  -> streaming STT / quality pass
  -> optional diarization
```

Golam should reuse the contract lessons, not blindly import all engines. The route must bind exact engine/model artifacts, hardware, language, locality, privacy, health and resource envelope. Arabic/English/code-switch, device swaps, Bluetooth, noise, false VAD, latency, six-hour stability and strict-local network denial belong in qualification evidence.

### 22.6 Muse contributes the interaction model and independent safety architecture

Meta Muse is proprietary and is used as public behavior/security reference only.

High-value product lessons:

- one long-running conversation rather than rigid turn locking;
- new messages while prior work continues;
- side chats when scoped context is useful;
- background tasks/goals;
- proactive messages only when worth interrupting the user;
- visible activity/history;
- rich Artifacts instead of forcing every answer into prose;
- deterministic approval cards for critical actions.

High-value security lessons map cleanly onto existing Golam concepts:

```text
Muse runtime cell             -> Golam bounded ExecutionNode/worker
Sentinel permission authority -> Golam Authority Kernel + Effect Gate
credential surrogation        -> Golam Secret/Account broker
network egress checks         -> T179 Egress Broker
tainted external data         -> T179 trust zones / T213 evidence coverage
durable state outside runtime -> Golam canonical user-owned state
```

Golam should not add a second Sentinel authority. The useful lesson is architectural separation: the model proposes; a separate deterministic authority path decides.

### 22.7 Full-duplex is a set of contracts, not a checkbox

The target flow is:

```text
Microphone / explicit voice session
  -> Capture Health
  -> Conditioning / AEC
  -> VAD / acoustic endpoint evidence
  -> Streaming STT (partial revisions)
  -> Semantic turn/intent hints
  -> Conversation / Task spine
  -> Agent stream
  -> Streaming TTS
  -> Audio playback
          ^
          |
     immediate barge-in
```

Each arrow is observable and independently cancellable.

Important source-channel rules:

```text
MICROPHONE_AUDIO != OWNER_IDENTITY
SYSTEM_AUDIO != OWNER_COMMAND
REMOTE_PARTY_SPEECH != OWNER_COMMAND
TTS_OUTPUT != USER_INTENT
PARTIAL_TRANSCRIPT != FINAL_USER_INTENT
```

This prevents meeting audio, a video, another person or Golam's own speaker output from becoming commands merely because the transcript contains imperative text.

### 22.8 Voice output has its own privacy boundary

Spoken output can leak information in a room even when network privacy is perfect. Therefore TTS routing must consider data class and output device. Sensitive content may require screen-only display, headphones, an explicit disclosure or a user-selected permissive profile.

A generated or cloned voice is presentation, never identity.

### 22.9 Resulting task decomposition

T196 remains the umbrella contract; new bounded tasks are:

```text
T217 AudioSession / Duplex Control
T218 Speech Runtime Router / Capture Health
T219 Streaming STT / Turn Detection / Semantic Interpretation
T220 Streaming TTS / Barge-In / Voice Presence
T221 Conversational Work / Multi-Task Voice UX
T222 Golam VoiceBench / Multilingual Safety / Accessibility
```

No task admits a speech model, cloud service, native library or proprietary Muse component. Exact implementation still requires live successor authority, Source Foundry and Model Artifact Foundry qualification.


### 22.10 Muse family and native-duplex benchmark separation

The Muse name now spans materially different artifacts and must not be treated as one provider:

- **Muse Agent / Muse Spark 1.3** — long-horizon collaboration, multitasking, background work and harness/security behavior reference;
- **Muse Voice Transcribe** — streaming speech perception reference/provider with real-time ASR, endpointing, diarization and multilingual code-switch behavior; currently treated as non-strict/remote unless an independently qualified local artifact is available;
- **Muse Glimmer 30B** — Apache-2.0 open-weight local multimodal/agentic model candidate suitable for T175/T152 qualification; it is not a speech model and does not replace STT/TTS/VAD;
- **native full-duplex speech-to-speech systems such as NemotronLabs VoiceChat** — benchmark/candidate class for unified listening, incremental transcription, speech generation and structured tool output.

This separation prevents product-name matching from becoming architecture. Golam selects capabilities by qualified contract, locality, evidence and artifact identity.

Current public VoiceChat evidence is particularly useful because it shows that native full-duplex tool calling can coexist with natural interruption handling while tool argument accuracy still remains an independent problem. Golam therefore evaluates native-duplex conversational quality and tool semantics separately; a speech model's structured tool call is still an untrusted proposal routed through T199 and the Effect Gate.

```text
MUSE_PRODUCT_NAME != CAPABILITY_IDENTITY
MUSE_GLIMMER != SPEECH_ENGINE
REMOTE_STREAMING_STT != STRICT_LOCAL_ROUTE
NATIVE_DUPLEX_TOOL_CALL != EFFECT_AUTHORIZATION
VOICE_CONVERSATION_QUALITY != TOOL_ARGUMENT_CORRECTNESS
```


### 22.11 Voice planning completeness closure

A final lifecycle audit found several concerns that were present only implicitly in T217–T222. They are now assigned explicit owners rather than left as implementation-time interpretation:

- T223 — activation, optional wake word, source attribution, diarization/floor semantics;
- T224 — critical numbers/entities, correction lineage and bound high-risk voice confirmation;
- T225 — audio transport, clock drift, jitter, device hot-plug, Bluetooth and remote-stream generation;
- T226 — consent, raw/transcript retention, biometrics, replay/deepfake/media injection and abuse safety;
- T227 — Arabic/English/code-switch scope, personal lexicon/pronunciation, persona and accessibility;
- T228 — graceful degradation, model/runtime update/rollback, SLO evidence and zero-tolerance release gates.

The companion `voice-audio-completeness-matrix-2026-09-22.md` maps the whole lifecycle from activation through playback and release qualification to a canonical owner and minimum evidence.

This closes a class of gaps that should not be deferred to implementation:

```text
WAKE_DETECTED != OWNER_AUTHENTICATED
SPEAKER_MATCH != OPERATION_AUTHORIZED
GENERIC_YES != BOUND_OPERATION_APPROVAL
RECONNECTED_STREAM != SAME_AUDIO_SESSION
BUFFERED_AUDIO != CURRENT_USER_INTENT
RAW_AUDIO_RETENTION != TRANSCRIPT_RETENTION
PERSONAL_LEXICON != SOURCE_TRANSCRIPT_TRUTH
DEGRADED_ROUTE != POLICY_RELAXATION
```

The plan now explicitly covers:

- activation and wake lifecycle;
- shared-room / meeting source and floor attribution;
- high-risk numeric/entity read-back and confirmation binding;
- local and remote audio transport integrity;
- clock/jitter/device recovery;
- raw-audio and transcript retention as separate privacy decisions;
- voice replay/deepfake/media injection;
- multilingual/personal lexicon behavior;
- accessible non-voice controls;
- deterministic degradation under model/device/network/resource failure;
- model/runtime update and rollback;
- route/hardware/language-specific SLO evidence;
- zero-tolerance release gates for authority and privacy failures.

No new canonical authority system is introduced. T223–T228 refine the existing T169/T174/T175/T179/T196/T199/T201/T205/T213/T216 contracts and keep implementation authority unchanged.

```text
VOICE_COMPLETENESS_MATRIX_PRESENT=YES
VOICE_TASK_GRAPH_EXTENDS_THROUGH_T228=YES
NEW_PARALLEL_AUTHORITY_SYSTEM=NO
ACTIVE_SPEC_006_PR_24_WIDENED=NO
NEW_PRODUCT_IMPLEMENTATION_STARTED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
```


### 22.12 OpenJev first-class bounded integration decision

The founder explicitly authorized use of the supplied `AlexWortega/openjev` model/source and asked that it become part of Golam when technically justified.

The current Hub state supports making it a **first-class bounded DecisionProvider target**, not an authority-bearing core dependency.

Current reviewed role:

```text
OpenJev
  -> bounded semantic classification / reranking
  -> T203 DecisionProvider
  -> T204/T229 calibration + applicability
  -> deterministic Golam caller/policy
```

High-value workloads include:

- voice utterance class and semantic turn-completeness;
- clarification-needed;
- attention triage;
- admitted capability/provider candidate ranking;
- retrieval reranking;
- bounded "what should be checked next?" verification triage.

The exact integration contract is recorded in `openjev-integration-plan-2026-09-22.md`, and T229 owns the future adapter/qualification lifecycle.

OpenJev is intentionally optional. Golam must remain correct when it is disabled, removed, unavailable, corrupt, out-of-domain or rejected for a particular workload.

This gives Golam a useful System-1 layer: cheap/fast bounded decisions can avoid unnecessary large-model calls, while exact rules, stronger models, authoritative sources and human review remain available through deterministic escalation.

The architecture ceiling remains:

```text
OPENJEV != AUTHORITY
OPENJEV != POLICY_ENGINE
OPENJEV != EFFECT_GATE
OPENJEV != OWNER_PRESENCE
OPENJEV != VERIFIED_FACT
OPENJEV_HIGH_SCORE != ALLOW
OPENJEV_UNAVAILABLE != GOLAM_UNAVAILABLE
```

This is a planning decision only. Exact artifact admission still requires immutable revision/file digests, base-model/transitive rights closure, T175 runtime qualification, workload-specific T204/T229 benchmark evidence and removal/rollback proof.

```text
OPENJEV_FIRST_CLASS_PROVIDER_TARGET=YES
OPENJEV_MODEL_ADMITTED=NO
OPENJEV_RUNTIME_DEPENDENCY_ADMITTED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
```


## 23. AutoClaw / Z.AI deep-dive conclusions

A focused review of official AutoClaw product material plus available OpenClaw/Z.AI source concluded that Golam should adopt selected operator and delivery disciplines, not copy AutoClaw as a monolith.

### 23.1 What AutoClaw validates

AutoClaw validates a user-facing product shape that matches Golam's direction:

```text
one user goal
-> agent plans and executes
-> tools/files/browser/skills
-> visible progress
-> durable deliverable/result
-> return result into the same conversation/channel
```

Official product material also exposes four high-value behaviors:

1. Cluster Mode: plan -> research -> parallelize -> audit -> revise -> deliver;
2. multi-agent work/life isolation with separate memory/workspace and same-agent continuity across IM channels;
3. Hermes self-evolution for preferences/corrections plus whole-workflow skill evolution;
4. IM as a remote task/progress/result surface.

These behaviors strengthen Golam's Harness/Experience contracts but do not change protected Authority/Evidence ownership.

### 23.2 Cluster Mode maps onto existing Golam primitives

Golam already has T163 DeliveryGraph, T167 Task/Session/Run/Worker identities, T184 dissent, T185 UI projection and T216 verification.

The missing product-level abstraction is an adaptive `DeliveryFormation` that chooses solo versus bounded team execution and declares role/evidence/review obligations before work begins.

T230 fills that gap.

Key boundary:

```text
FORMATION != AUTHORITY
PARALLELISM != QUALITY
REVIEWER_OPINION != VERIFIED_COMPLETE
```

### 23.3 Same agent across channels is not the same as shared ambient state

AutoClaw's same-agent cross-channel continuity is useful, but only with explicit bindings.

Golam should bind:

```text
AgentIdentity
-> MemoryNamespace
-> WorkspaceBindings
-> AccountBindings
-> CapabilityProfile
-> PrivacyProfile
-> ChannelBindings
```

T231 makes this explicit. Different agents remain isolated even on one host. Shared channel/group membership never implies shared memory, credentials or authority.

### 23.4 Hermes should become a governed low-friction learning surface

Golam already has the stronger safety spine: T122/T123/T162/T181/T215 prohibit in-place autonomous protected mutation and require versioned candidates/evaluation/activation.

The AutoClaw/Synapse lesson is product ergonomics: every meaningful correction, preference, tool failure or recurring workflow should have a cheap path into a visible candidate.

T232 adds:

```text
LearningObservation
-> PreferenceRuleCandidate
   / ToolKnowledgeCandidate
   / AgentWorkflowCandidate
   / SkillCandidate
-> preview/conflict/eval
-> governed activation
```

This preserves user intent without making a model-written AGENTS/SOUL/TOOLS file itself authoritative.

### 23.5 Z.AI GLM-skills expose the missing operator lifecycle around skills

Golam's Source Foundry and extension security are stronger than ordinary skill loaders. The useful Z.AI/OpenClaw addition is the complete skill-pack lifecycle:

- manifest and prerequisites;
- inspect before enable;
- install/import;
- qualify;
- enable/disable;
- update;
- revoke;
- rollback;
- remove;
- progressive disclosure of full instructions/assets only when selected.

T233 adds this lifecycle without creating a second plugin authority.

### 23.6 OpenClaw completeness rubrics are valuable as an external no-gap audit

The exact-pinned OpenClaw repository contains unusually broad completeness rubrics across multi-agent, sessions/memory, channels, automation, plugins, security, browser/sandbox, voice, clients, providers and observability.

T234 uses these as a periodic external parity checklist with explicit Golam dispositions:

```text
SUPPORTED
PLANNED
INTENTIONALLY_DIFFERENT
OUT_OF_SCOPE
BLOCKED
UNKNOWN
```

This is a no-gap audit, not a feature-count contest or architecture authority.

### 23.7 Open-AutoGLM is useful code, but its action loop must be re-authorized

The public source cleanly separates screenshot/model/action/device adapters and contains useful Android/HarmonyOS/iOS primitives.

Golam may selectively port/adapt these device primitives, but the source's direct model-response -> action-handler loop is not retained as authority.

Required translation:

```text
model visual output
-> parsed Operation candidate
-> target / consequence / taint classification
-> canonical Effect Gate
-> protected device adapter
-> post-action evidence / verification
```

Local callback confirmation/takeover becomes Golam's existing approval/OwnerPresence/takeover generation semantics.

### 23.8 Rights/source posture

The founder states Z.AI granted permission to copy/use available Z.AI AutoClaw-related source.

Public reviewed Z.AI repositories are Apache-2.0. Independent OpenClaw is MIT with third-party notices.

No public source repository for the AutoClaw desktop application itself was identified in this review. Product behavior remains reference/reimplementation input unless exact AutoClaw source is separately supplied and admitted.

### 23.9 Resulting program tasks

```text
T230 Adaptive Delivery Formation / Cluster Discipline
T231 Cross-Channel Agent Identity / Memory / Workspace Binding
T232 Governed Preference / Tool-Knowledge / Workflow Evolution
T233 Skill Pack / Prerequisite / Progressive-Disclosure Lifecycle
T234 External Agent-OS Operator Completeness Parity Harness
```

These tasks refine existing T122/T162/T163/T167/T169/T180/T184/T185/T215 contracts. They do not widen active Spec 006 and grant no implementation authority.

```text
AUTOCLAW_ZAI_DEEP_DIVE_COMPLETE=YES
MONOLITHIC_AUTOCLAW_COPY_PLANNED=NO
ZAI_AVAILABLE_SOURCE_PERMISSION_ATTESTED=YES
NEW_PARALLEL_AUTHORITY_SYSTEM=NO
ACTIVE_SPEC_006_PR_24_WIDENED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
```


## 24. OpenMuse and Laya decision-model follow-up

A 2026-09-23 follow-up reviewed the founder-supplied `CopilotKit/openmuse`, `convaiinnovations/laya` and reaffirmed `AlexWortega/openjev` under explicit reuse permission.

### 24.1 Laya belongs beside OpenJev, not underneath it

The Hugging Face `convaiinnovations/laya` artifact is materially smaller than OpenJev and is tagged for System-1/calibrated decision workloads such as classification, routing, scoring, guardrails and moderation.

This supports a two-tier bounded DecisionProvider strategy:

```text
small/high-frequency bounded decision
  -> Laya if qualified

deeper NLI/cross-encoder/reranking
  -> OpenJev if qualified

ambiguous/high-consequence/out-of-domain
  -> exact rule / stronger provider / verifier / human
```

No global model winner is declared. T235 requires matched held-out workload/hardware tournaments and workload-specific admission.

A guardrail/moderation model is not policy authority.

```text
LAYA_GUARDRAIL_SCORE != POLICY_DECISION
LAYA_MODERATION_SCORE != EFFECT_PERMISSION
LAYA_HIGH_SCORE != ALLOW
```

### 24.2 The two Laya sources must never share identity

Golam already reviewed `aayushch/laya` as a product/experience reference. The newly supplied `convaiinnovations/laya` is a separate Hugging Face model.

The name collision is explicitly closed:

```text
aayushch/laya
  = product / experience / harness reference

convaiinnovations/laya
  = model artifact / DecisionProvider target
```

Distinct Source Foundry IDs, registry keys and artifact identities are mandatory.

### 24.3 OpenMuse is a bounded donor, not a replacement runtime

The reviewed OpenMuse pin exposes high-value implementation material in five areas:

1. visible follow-up queue + honest run-failure behavior;
2. rich thread/task/artifact hydration through durable IDs;
3. durable task worker leases, pause/resume/cancel/retry and uncertain-write handling;
4. persistent Chromium + manual takeover;
5. isolated Linux computer and strong smoke/security fixtures.

It also provides useful Goals, monitors, Ideas and background-update UX.

The correct Golam strategy is exact-component selection, not a wholesale fork.

```text
OpenMuse Experience code
  -> Golam Experience Plane

OpenMuse worker/recovery patterns
  -> existing T167/T205 contracts

OpenMuse browser/computer
  -> existing Capability/Execution contracts

OpenMuse review UI/tests
  -> canonical Golam approval/effect contracts
```

The following donor boundaries are mandatory:

```text
OPENMUSE_TASK_STORE != GOLAM_TASK_AUTHORITY
OPENMUSE_REVIEW != GOLAM_APPROVAL_AUTHORITY
COPILOTKIT_THREAD != GOLAM_CANONICAL_TASK
PLAYWRIGHT_CONTAINER != KERNEL_SECURITY_BOUNDARY
DOCKER_CONTAINER != HOSTILE_TENANT_VM
```

### 24.4 OpenMuse test evidence is itself a valuable donor

The strongest reusable material may be its verification fixtures rather than runtime code. High-value examples include:

- lease races and expired-lease recovery;
- uncertain external writes with no blind replay;
- cancellation while an already-dispatched provider request may still complete;
- account/hash/version-bound review invalidation;
- persistent browser-profile recovery;
- failed-profile cleanup;
- browser destination/DNS/egress checks;
- no host-shell fallback;
- stale executor fencing;
- interrupted command recovery without replay;
- filesystem traversal/special-file rejection.

T236 requires explicit port/reimplementation decisions for these fixtures.

### 24.5 CopilotKit Intelligence is not required for Golam

OpenMuse uses CopilotKit Intelligence for Rich Threads, but the repository itself documents that service as separately configured.

Golam may copy/adapt the thread UI or projection behavior without adopting that external service as canonical storage or a mandatory runtime dependency.

Durable conversation/task truth remains Golam-owned.

### 24.6 Resulting tasks

```text
T235 Laya Low-Resource Decision Provider Qualification / OpenJev Tournament
T236 OpenMuse Exact-Component Port Matrix / Parity Qualification
```

The detailed source plans are:

- `laya-integration-plan-2026-09-23.md`;
- `openmuse-integration-plan-2026-09-23.md`.

These tasks extend the current planning overlay without widening active Spec 006 or granting implementation authority.

```text
OPENMUSE_BOUNDED_DONOR=YES
OPENMUSE_WHOLESALE_FORK=NO
LAYA_FIRST_CLASS_LOW_RESOURCE_PROVIDER_TARGET=YES
LAYA_MODEL_ADMITTED=NO
OPENMUSE_CODE_ADMITTED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
```


## 25. Major lifecycle and operability review

The 2026-09-23 program-level review intentionally stopped looking for additional agent/model features and instead audited Golam from installation through decommission.

The detailed review is `program-major-review-2026-09-23.md`.

Nine residual lifecycle gaps were promoted into explicit tasks:

```text
T237 Owner Bootstrap / Recovery / Device Replacement / Decommission
T238 Secret / Credential Lifecycle
T239 Canonical State Integrity / Backup / Restore / Safe Repair
T240 Secure Update / Offline Bundle / Compatibility / Rollback
T241 Connector Auth / Remote Event / Webhook / Sync Lifecycle
T242 OS Permission / Platform Capability Drift
T243 Deployment / Tenancy / Enterprise Boundary
T244 Operational Health / Diagnostics / Safe Repair
T245 Canonical Configuration Revision / Policy Precedence / Drift
```

These are not new authority systems. They make previously distributed goals operationally complete.

### 25.1 Why these gaps matter

Golam already has strong contracts for what an agent may do. A trustworthy Agent OS also needs exact behavior when:

- the user installs it for the first time;
- the primary device is lost;
- a secret expires or is compromised;
- the canonical database is corrupt;
- an update is validly signed but broken;
- the machine is air-gapped;
- an OAuth token expires during work;
- a webhook is replayed or delivered twice;
- the OS revokes Accessibility, Screen Recording or microphone access;
- a personal installation is mistakenly treated as multi-user secure;
- the operator needs to diagnose and repair the system without sending private data away.

A system that only specifies the happy-path action loop is not operationally complete.

### 25.2 New source set

The review adds a deliberately small source/reference set:

- The Update Framework — update trust/freshness/rollback protection;
- Sigstore/Cosign — release signature/attestation verification;
- Tauri updater — desktop update transport/install mechanics only;
- restic — encrypted/verifiable backup/restore principles;
- keyring-rs — OS-native credential-store adapter candidate;
- Nango — connector OAuth/token/webhook/sync lifecycle behavior reference only by default due the reviewed Elastic License 2.0 posture.

No source is admitted as a runtime dependency by this review.

### 25.3 Product baseline is now explicit

The architecture baseline remains:

```text
PERSONAL_SINGLE_OWNER
LOCAL/PRIVATE FIRST
ONE PROTECTED AUTHORITY HOST
MULTIPLE BOUNDED AGENTS / WORKERS / DEVICES / EXECUTION NODES
```

A future enterprise/team mode is allowed only through a separate tenancy/security lifecycle; adding accounts to a shared local process is not enterprise isolation.

### 25.4 New lifecycle invariants

```text
RECOVERY_MATERIAL != EFFECT_AUTHORIZATION
BACKUP_PRESENT != RESTORE_PROVEN
DATABASE_REPAIR != EFFECT_OUTCOME_REWRITE
UPDATE_SIGNATURE != UPDATE_POLICY
OFFLINE_BUNDLE != ARTIFACT_ADMISSION
ACCOUNT_REAUTH != PENDING_EFFECT_REAUTHORIZATION
WEBHOOK_SIGNATURE != EVENT_SEMANTIC_TRUTH
OS_PERMISSION_GRANTED_ONCE != CURRENT_ROUTE_APPLICABILITY
SAME_HOST != SAME_TENANT
TRACE != REPAIR_AUTHORITY
STALE_CONFIG_WRITE != VALID_CONFIGURATION_UPDATE
RUNTIME_DERIVED_CONFIG != CANONICAL_CONFIGURATION
UNINSTALL != REMOTE_DATA_ERASURE
```

### 25.5 Review conclusion

The review found no reason to replace the existing Authority / Effect / Evidence / Task spine. The main improvement is lifecycle closure around it.

```text
MAJOR_PROGRAM_REVIEW_2026_09_23_COMPLETE=YES
MEASURED_LIFECYCLE_GAPS_FOUND=9
TASK_GRAPH_EXTENDS_THROUGH_T245=YES
NEW_PARALLEL_AUTHORITY_SYSTEM=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_PRODUCT_IMPLEMENTATION_STARTED=NO
ACTIVE_SPEC_006_PR_24_WIDENED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
```
