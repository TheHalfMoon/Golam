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
RUNTIME_RESTART != SAME_EXECUTION_INCARCATION
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
