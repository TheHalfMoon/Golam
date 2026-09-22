# Golam Canonical Direction Task Extension — 2026-09-22

**Authority**: PROGRAM ORCHESTRATION ONLY — NO PRODUCT IMPLEMENTATION AUTHORITY

**Extends**:
- `program-superiority-tasks.md` T110–T165
- `program-superiority-gap-closure-tasks.md` T166–T184
- `program-direction-tasks-2026-09-13.md` T185–T202

**Direction review**: `program-direction-review-addendum-2026-09-22.md`

**Source review**: `competitive-source-register-supplement-2026-09-22.md`

These tasks capture gaps exposed by the 2026-09-22 source/portfolio review. They do not widen active Spec 006 PR #24, change `specs/CURRENT.md`, amend the Constitution, admit any dependency, or authorize a successor implementation unit.

## Phase R — Semantic decision and capability exchange foundations

- [ ] **T203 — Semantic Decision Provider Contract.** Define one provider-neutral `DecisionProvider` / `DecisionRequest` / `DecisionReceipt` family for bounded typed decisions. Support at minimum Choice, Boolean/Noul and ordered Score semantics where independently qualified. Bind exact state/context digest, schema/option identities, explicit no-match/abstain semantics, model artifact, backend/device/runtime, prompt/schema digest, probability output, calibration profile, input-limit/truncation evidence, latency/resource/cost and provider revision. Define deterministic cross-field constraints / bounded dependency graphs so independently scored fields cannot silently form an impossible or policy-inconsistent decision set; contradiction must cause abstention, deterministic repair only where specified, or escalation. SemIf, Decider and Nimble are candidate/reference implementations; none becomes the canonical policy engine. The contract must explicitly prohibit a provider from minting capability, lowering a trusted consequence class, authorizing egress, satisfying owner presence/approval, declaring source truth or emitting `VERIFIED_COMPLETE`.

- [ ] **T204 — Decision Calibration, Applicability and Escalation Qualification.** Extend T143/T145/T152/T158/T175/T203 with workload-specific qualification for decision providers. Measure appropriate accuracy/balanced accuracy, NLL/log loss, Brier, ECE/reliability, AURC/selective risk, abstention coverage, option-order/schema perturbation, cross-field contradiction/impossible-state rate, context-length and truncation sensitivity, quantization/backend drift, domain shift, tainted/adversarial input behavior, latency and resources. Define a deterministic escalation policy from exact rules/source truth -> bounded local decision provider -> stronger provider/generative reasoning -> independent verification/human review. Generic confidence thresholds are forbidden unless calibrated for the exact workload/configuration.

- [ ] **T206 — Capability Catalog and Provider Offer Contract.** Extend T119/T152/T154/T165/T180/T197/T199/T201 with provider-neutral `CapabilityDefinition`, exact `CapabilityOffer`, `ProviderRevision`, `AccountBinding`, `CapabilityAvailability`, `CapabilityQualification` and proposed `ToolCallPlan`. Discovery is by task/capability rather than provider name alone. Every offer must expose exact provider/revision, locality, account/credential requirements, operation/effect classes, egress destinations/data classes, retention expectation where knowable, cost model/estimate, latency/reliability evidence, availability freshness and Source Foundry/conformance references. Catalog state is a projection of admitted/observed provider state and must not become a second capability, identity, authorization, billing or Effect authority.

### T203 acceptance requirements

A future owning spec must prove:

1. provider-neutral fixtures work against at least two independently qualified providers or one real provider plus a deterministic fake;
2. provider output cannot lower deterministic risk/authority;
3. model/provider replacement changes no protected authority semantics;
4. exact artifact/backend/schema/calibration identities are attributable;
5. abstention and unavailable/not-qualified are distinct from a negative answer;
6. prompt/context truncation cannot be hidden.

### T206 acceptance requirements

A future owning spec must prove:

1. one stable capability ID may expose multiple exact provider offers without collapsing provider/account identity;
2. an offer cannot become callable merely by appearing in search;
3. stale/unqualified/unavailable offers fail closed;
4. strict-local search may display remote offers but cannot select/dispatch them;
5. provider fallback cannot silently change egress, account, spend or effect class;
6. arbitrary private/unmodeled tools remain `OPEN_WORLD` until qualified.

## Phase S — Reconciled execution and credential-brokered tool use

- [ ] **T205 — Agent Workload Manifest / Reconciled Execution Contract.** Refine T127/T129/T130/T151/T153/T159/T167/T168/T173 using Google AX as a reconciliation/lifecycle reference without importing Kubernetes/Redis as local baseline requirements. Define a versioned `ExecutionEnvelope` bound to a canonical `TaskContract` with workspace bindings, execution backend, isolation profile, compute/resource budget, egress policy reference, execution profile/model reference, capability set, secret-handle references, readiness conditions, observable endpoints, checkpoint/suspend/resume/timeout/cleanup policy and output-artifact policy. Separate desired runtime state from observed runtime state. Bind each runtime instance to an execution incarnation/generation (or equivalent fencing token) so a replaced/stale worker fails closed at the protected action boundary after reconciliation or reprovisioning. For process/container/tool execution, bind authorization across approval wait to exact executable/workload identity where observable: executable or artifact digest, workspace/source revision, backend identity, requested confinement profile and the trusted observed-confinement evidence available for the actual attempt. A backend label, requested sandbox profile, or pre-approval command name is not proof that the post-approval workload bytes or confinement remained identical. Runtime reconciliation may restart/reprovision a worker where allowed but must never become a blind retry path for ambiguous external Effects.

- [ ] **T207 — Credential-Brokered Tool Relay Contract.** Extend T119/T165/T172/T179/T190/T199/T201/T206 with a Treg-inspired but Golam-authorized relay path. Support exact provider/account selection, multiple credential bindings, secret-handle injection, destination binding, control-header/cookie stripping, SSRF/private-address policy, raw path/query fidelity where required, bounded request/response sizes, streaming, cancellation, idempotency material, bounded pre-dispatch cost quote/reservation where a provider is paid, final cost receipt/reconciliation, and failure capture with redaction. Cost overrun beyond the authorized budget must stop/fail closed or require a new budget decision; spend authorization is separate from Effect authorization. Authorization, Effect state, egress, identity, secret state and final verification remain canonical Golam concerns. A relay transport success is not an Effect verification result.

### T205 invariants

```text
WORKLOAD_MANIFEST != CAPABILITY_GRANT
WORKSPACE_BINDING != FILESYSTEM_AUTHORITY
GATEWAY_DECLARATION != EGRESS_AUTHORIZATION
RUNTIME_READY != TASK_VERIFIED_COMPLETE
RUNTIME_RECONCILIATION != EFFECT_RETRY
CHECKPOINT_STATE != CURRENT_AUTHORITY
APPROVED_COMMAND_NAME != APPROVED_EXECUTABLE_BYTES
SANDBOX_LABEL != CONFINEMENT_PROOF
BACKEND_CAPABILITY != OBSERVED_CONFINEMENT
APPROVAL_WAIT != WORKLOAD_IDENTITY_CONTINUITY
```

### T207 invariants

```text
CREDENTIAL_BINDING != CALL_AUTHORIZATION
RELAY_TARGET_RESOLUTION != EFFECT_PERMISSION
HTTP_SUCCESS != EFFECT_VERIFIED
TOOL_METADATA != TRUSTED_OPERATION_CLASS
OPEN_WORLD_TOOL != SAFE_READ
```

### T205/T207 shared acceptance

A future owning spec must prove:

- resume revalidates TaskContract, current capability generations, workspace/source revisions, provider availability and pending Effect uncertainty;
- a lost worker cannot cause duplicate at-most-once/irreversible Effects;
- setup/bootstrap authority is narrower than or equal to the authorized envelope and cannot install arbitrary software through hidden egress;
- credential values never need to enter model context when brokered use is possible;
- exact provider account is visible before consequential dispatch;
- cancellation has honest semantics when an upstream may already have received the request;
- `UNKNOWN_OUTCOME` blocks dependent/conflicting operations until reconciliation.

## Phase T — Proactive attention, coherence and compact agent-facing discovery

- [ ] **T208 — Proactive Attention and Action Proposal Fabric.** Extend T133/T135/T167/T179/T183/T185/T186/T201 with Laya-inspired proactive product semantics built strictly as projections over canonical Golam state. Define `AttentionItem` and `ActionProposal` objects sourced from connector observations, schedules, worker blockers, approval expiry, `UNKNOWN_OUTCOME`, verification gaps and user-governed opportunity rules. An ActionProposal must disclose exact target/account/provider/route, proposed payload or diff, operation/effect class, egress/data path, cost, irreversibility/retry semantics, source/context evidence, verification plan and freshness/expiry. UI approval is authenticated approval input only; the Effect Gate immediately revalidates live authority before dispatch.

- [ ] **T209 — Cross-Source Coherence and Briefing Projection.** Refine T120/T125/T126/T136/T161/T164/T189/T192 using Laya's Coherence/Omni ideas and Morize semantics. Define evidence-linked cross-source entity/relation candidates, temporal views and daily/periodic briefing projections across connectors, Tasks, Actions and Knowledge. Retrieval should prefer exact identity/metadata/time -> FTS/BM25 -> explicit relation graph -> optional local vectors/reranking -> synthesis. Where fusion such as Reciprocal Rank Fusion is used it remains ranking evidence, not truth. Briefings must disclose time window, included/offline/stale sources, omissions, unresolved contradictions and which statements are direct observations versus synthesis. Manual link/unlink/classification corrections create immutable rule/routine candidates; they cannot mutate active trusted policy in place.

- [ ] **T210 — Compact MCP Capability Surface.** Extend T154/T172/T183/T206/T207 with a stable agent-facing discovery surface that avoids one MCP tool per provider endpoint. Define compact `capability_search`, `capability_get` and exact `capability_call` semantics with static/versioned schemas, stable capability/offer IDs, read/write/open-world/destructive/idempotency annotations, audience-isolated OAuth/managed credentials where applicable and exact mapping into the canonical Effect path. Catalog growth changes data returned by discovery, not the trusted MCP tool list. Public/tool surface metadata cannot claim a private/unmodeled endpoint is a safe read merely from HTTP method or name.

### T208/T209 UX constraints

The primary product may expose:

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

These are product views over canonical state. They are not additional authority databases.

```text
ATTENTION_CARD != TASK_TRUTH
ACTION_PROPOSAL != AUTHORIZED_EFFECT
ASSOCIATION_CONFIDENCE != CANONICAL_RELATION
BRIEFING_SUMMARY != SOURCE_OF_TRUTH
LEARNED_RULE != ACTIVE_AUTHORITY
```

## Phase T2 — Attention budget and proactive autonomy discipline

- [ ] **T212 — Attention Budget and Proactive Autonomy Policy.** Refine T208/T209 with explicit interruption governance so proactive Golam does not become intelligent notification spam. Define deduplication/coalescing, freshness/expiry, quiet/defer policy, current-focus awareness where available, user-configured urgency classes, briefing-vs-immediate routing, reason-for-surfacing, correction feedback and a bounded daily/periodic interruption budget where useful. Evaluate precision/recall for important items together with unnecessary-interruption rate, duplicate surfacing, stale-card rate, deferred-item recovery and correction stability. Corrections produce candidate rules/routines and never silently mutate protected active policy. `ATTENTION_SCORE != USER_PRIORITY_TRUTH`; `HIGH_MODEL_CONFIDENCE != INTERRUPT_NOW`; `PROACTIVE != ALWAYS_INTERRUPTIVE`.

## Phase U — Owner portfolio governance and cross-fabric proof

- [ ] **T211 — Owner Portfolio Reuse Matrix / Internal Donor Bridge.** Maintain a confidentiality-safe inventory of the founder-owned GitHub portfolio and map selected public/private components to measured Golam gaps. The 2026-09-22 enumeration, reverified during the portfolio deep dive, observed 36 owner repositories: 29 public and 7 private. Public planning may name public sources; private names/content stay undisclosed unless separate publication authority exists. Reuse must route through the same T113/T165/T180/T197 exact-component Source Foundry record. For each selected component record current pin/tree, source role, selected paths, reuse strategy, dependency/runtime closure, rights/NOTICE, authority ceiling, TCB delta, benchmark/parity reason and removal/rollback path. Repository ownership or founder permission never auto-admits the code.

### T211 required role vocabulary

Use bounded roles rather than "merge everything":

```text
CANONICAL_GOLAM
PRIMARY_CAPABILITY_DONOR
BOUNDED_ADAPTER_CANDIDATE
HIGH_VALUE_ARCHITECTURE_REFERENCE
BENCHMARK_OR_METHOD_REFERENCE
DOMAIN_REFERENCE_ONLY
PROVENANCE_CAUTION_REFERENCE
NO_CURRENT_MEASURED_GAP
PRIVATE_CONSIDERED_UNDISCLOSED
```



## Phase V — Evidence fidelity, bounded delegation, skill replay and multidimensional proof

- [ ] **T213 — Evidence Fidelity, Coverage and Absence Contract.** Refine T149/T150/T156/T158/T177/T192 using the strongest owner-portfolio evidence semantics without creating a second Evidence Plane. Define one typed contract that separates: provider/vendor capability ceiling, adapter implementation fidelity, capture/observation activation, predicates actually observed, deterministic derivations, unsupported predicates, and explicit absence reasons. At minimum distinguish `NOT_OBSERVED` (observation was active and capable), `NOT_OBSERVABLE_AT_FIDELITY`, `CAPTURE_INACTIVE`, `UNSUPPORTED`, `FAILED`, `PARTIAL` and `UNKNOWN` where applicable. Evidence-producing adapters declare valid predicates/lifecycles by subject kind rather than forcing one universal lifecycle. Missing/failed/unsupported analysis never becomes a clean result by absence. Every nontrivial claim carries exact source/adapter/revision/fidelity evidence sufficient to explain what Golam could and could not know. This extends canonical Verification/Evidence semantics; it MUST NOT create a parallel finding/evidence authority.

- [ ] **T214 — Disclosure-Bound Agent Proposal and Review Checkpoint Contract.** Refine T120/T127/T133/T163/T167/T179/T192 with a bounded disclosure/proposal boundary for workers and external agents. Define an immutable `ContextDisclosureReceipt` binding exact disclosed object/source/revision identities, omissions/rejections, byte/token budget, sensitivity/taint, expiry, audience and capability ceiling. A returned `WorkerProposal` must cite the disclosure receipt and may target an existing canonical object only when that exact object/revision (or an explicitly permitted successor rule) was disclosed and the proposal carries current expected-revision preconditions. Disclosure exports no capability, approval, lease, secret or authority. Record proposal origin separately from owner/reviewer acceptance. Add an explicit `ReviewCheckpoint` / reviewed-through sequence marker so opening a view never implies review; resume projections are pinned to a stable canonical sequence and prioritize unresolved contradictions, stale evidence and blockers before ordinary continuation state.

- [ ] **T215 — Canonical Skill / Workflow IR, Deterministic Replay and Divergence Repair.** Refine T122/T123/T162/T163/T194/T199 before broad self-improving skill evolution. Define one versioned typed `WorkflowIR` / `SkillIR` carrying artifact/dataflow identities, capability requirements (never captured grants), disclosure constraints, side-effect/effect classes, preconditions, postconditions, verification obligations, retry/reconciliation semantics, compatibility assumptions and exact dependency/provider revisions. Separate `SkillCompiler` (human/verified trajectory -> authority-free candidate) from `DeterministicReplay` (fresh authorization, exact attempts/receipts) and from `DivergenceRepair` (failed-assumption detection, localized repair, downstream evidence/artifact invalidation, fresh authorization/re-verification, candidate version promotion). Successful exploratory work may reduce later model calls only when replay compatibility is proven. A syntactically/schema-valid compiled workflow is not automatically semantically faithful to the originating intent.

- [ ] **T216 — Multidimensional Verification and EvidenceBundle Contract.** Refine T149/T150/T157/T186/T193/T199 so consequential outcomes and durable artifacts are not collapsed into one green boolean. Define a versioned `EvidenceBundle` plus orthogonal verification dimensions suitable to the subject, such as source/input identity, output/content integrity, target/account identity, route/backend identity, authority/approval binding, operation/effect terminal state, constraint/postcondition satisfaction, verifier independence, coverage/fidelity, freshness/time basis, provider attestations and unresolved/unsupported dimensions. `VERIFIED_COMPLETE` requires the owning VerificationObligation to state which dimensions are mandatory and to fail closed when a mandatory dimension is invalid, missing or unknown. Prefer an independently implemented verifier/readback path for high-consequence artifacts/effects when practical; producer self-report alone cannot satisfy independent verification.

### T213–T216 hard invariants

```text
NOT_OBSERVED != NOT_POSSIBLE
NOT_OBSERVABLE != NOT_OCCURRED
CAPTURE_INACTIVE != NEGATIVE_EVIDENCE
NO_FINDING != CLEAN
CHECK_PASS != COMPLETE_COVERAGE
PARTIAL_ENFORCEMENT != ENFORCED
DISCLOSURE != AUTHORITY
UNDISCLOSED_OBJECT != VALID_PROPOSAL_TARGET
EXPORTED_CONTEXT != EXPORTED_CAPABILITY
AGENT_ORIGIN != OWNER_ACCEPTANCE
VIEW_OPENED != REVIEW_COMPLETED
SCHEMA_VALID != SEMANTICALLY_EQUIVALENT
COMPILABLE != FAITHFUL
REPLAY_COMPATIBLE != CURRENTLY_AUTHORIZED
REPAIR != SILENT_HISTORY_REWRITE
PRODUCER_SUCCESS != INDEPENDENT_VERIFICATION
EVIDENCE_BUNDLE != SINGLE_BOOLEAN
```

### T213–T216 acceptance direction

Future owning specs must prove, with bounded fixtures and at least one adversarial path per contract:

- unavailable capture/coverage cannot render as a negative fact or clean result;
- a worker cannot propose a mutation to undisclosed/stale protected state merely because it knows an identifier from another channel;
- exported context never carries live grants/approvals/secrets by implication;
- resume/checkpoint state is explicit and cannot be advanced by reading a screen;
- a compiled skill cannot retain demonstration-time authority and must acquire fresh authorization on replay;
- divergence invalidates only evidence/artifacts whose assumptions are affected, while preserving immutable prior history;
- a producer and independent verifier can disagree without the producer overwriting verification truth;
- mandatory unknown verification dimensions prevent `VERIFIED_COMPLETE`.

## Cross-fabric ownership rule

Before T203–T210 implementation, T198's Canonical Shared-Contract Ownership Matrix must name the sole owner/version source/migration authority for at least:

- TaskContract / Task-Session-Run-Worker identities;
- ExecutionEnvelope;
- Principal/capability lease;
- Operation/Effect ontology and Effect ledger;
- Route applicability;
- Egress/privacy;
- Secret/account binding;
- VerificationObligation/Receipt;
- canonical events;
- source/extension admission;
- ContextBundle/RetrievalReceipt;
- DecisionRequest/Receipt;
- CapabilityDefinition/Offer;
- AttentionItem/ActionProposal projections;
- EvidenceCoverage/Fidelity and absence semantics;
- ContextDisclosureReceipt / WorkerProposal / ReviewCheckpoint;
- WorkflowIR/SkillIR and replay/divergence semantics;
- EvidenceBundle / multidimensional verification projections.

No owning package may invent package-local protected truth for one of these concepts.

## Source-specific qualification requirements

### Google AX

Use as architecture/reference candidate first. Do not import Kubernetes, Redis or Agent Substrate into the local baseline merely to reproduce AX topology. Any bounded code reuse requires normal Source Foundry qualification.

### Treg

The public repository currently carries Apache-2.0 plus additional hosted-service restrictions. The founder separately asserts permission to use/copy the source. Any code admission for distributed/commercial embedding must bind documentary permission scope for the intended use in the exact component admission record. No implicit waiver is inferred.

### SemIf / Decider / Nimble

Treat code and model artifacts separately. A code license never auto-admits weights, base models, datasets, teacher outputs, quantizations or runtime backends. Model Artifact Foundry T175 applies to every selected artifact.

### Laya

Prefer behavior/UX and bounded component patterns. Its n8n/Python/Chroma topology is not a Golam trusted-path requirement.

### TinyFish / Desktop Commander

Adapters remain outside protected authority. Hosted variants are explicit non-strict capabilities. Anti-bot/stealth behavior is not a Golam product goal. Broad host/process capability does not bypass Golam isolation and Effect policy.

## Priority and sequencing

Recommended dependency order after an owning bounded lifecycle is authorized:

```text
P0_CANONICAL_PREREQUISITES:
  T112
  T149/T150
  T156/T177/T179
  T165/T180/T197
  T167
  T199
  T201

P0_NEW_SHARED_CONTRACTS:
  T203 Decision Provider Contract
  T206 Capability Catalog / Provider Offer Contract
  T213 Evidence Fidelity / Coverage / Absence Contract
  T214 Disclosure-Bound Agent Proposal / Review Checkpoint Contract
  T215 Canonical Skill / Workflow IR + Replay / Divergence Contract
  T216 Multidimensional Verification / EvidenceBundle Contract
  T211 Owner Portfolio Reuse Matrix (cross-cutting governance)
  T212 Attention Budget / Proactive Autonomy Policy

P1_NEW:
  T204 Decision Calibration / Escalation
  T205 ExecutionEnvelope / Reconciliation
  T207 Credential-Brokered Tool Relay

P1_PRODUCT_PROJECTION:
  T208 Proactive Attention / Action Proposal

P2_AFTER_FOUNDATIONS:
  T209 Cross-Source Coherence / Briefing
  T210 Compact MCP Capability Surface
```

This priority is architectural, not authorization. Live successor authority after Spec 006 closeout decides actual spec numbering and activation order.

## New hard invariants

```text
DECISION_PROBABILITY != AUTHORITY
DECISION_PROVIDER != POLICY_ENGINE
CALIBRATED_PROBABILITY != VERIFIED_FACT
DECISION_PROVIDER_OUTPUT != OWNER_APPROVAL
MODEL_ROUTE_RECOMMENDATION != EFFECT_PERMISSION
WORKLOAD_MANIFEST != CAPABILITY_GRANT
WORKSPACE_BINDING != FILESYSTEM_AUTHORITY
GATEWAY_DECLARATION != EGRESS_AUTHORIZATION
RUNTIME_READY != TASK_VERIFIED_COMPLETE
RUNTIME_RECONCILIATION != EFFECT_RETRY
CAPABILITY_CATALOG_ENTRY != TOOL_ADMISSION
CAPABILITY_OFFER != EFFECT_PERMISSION
CREDENTIAL_BINDING != CALL_AUTHORIZATION
TOOL_RELAY_SUCCESS != EFFECT_VERIFIED
ATTENTION_CARD != TASK_TRUTH
ACTION_PROPOSAL != AUTHORIZED_EFFECT
ASSOCIATION_CONFIDENCE != CANONICAL_RELATION
BRIEFING_SUMMARY != SOURCE_OF_TRUTH
LEARNED_RULE != ACTIVE_AUTHORITY
OWNER_REPOSITORY_ACCESS != SOURCE_ADMISSION
FIELD_PROBABILITY != JOINT_CONSISTENCY
STALE_WORKER != CURRENT_EFFECT_ACTOR
BUDGET_AUTHORIZATION != EFFECT_AUTHORIZATION
ATTENTION_SCORE != USER_PRIORITY_TRUTH
HIGH_MODEL_CONFIDENCE != INTERRUPT_NOW
NOT_OBSERVED != NOT_POSSIBLE
NO_FINDING != CLEAN
DISCLOSURE != AUTHORITY
UNDISCLOSED_OBJECT != VALID_PROPOSAL_TARGET
SCHEMA_VALID != SEMANTICALLY_EQUIVALENT
PRODUCER_SUCCESS != INDEPENDENT_VERIFICATION
```

## End-to-end proving journeys

Future GolamBench/Spec 010 should eventually include at least these cross-fabric journeys:

### Outcome request

```text
intent
-> TaskContract
-> capability discovery
-> bounded decision routing
-> isolated execution
-> evidence/context
-> ActionProposal
-> no unauthorized external write
-> VerificationReceipts
-> verified terminal state
```

### Proactive morning

```text
connector events
-> canonical ingest
-> triage
-> coherence/context
-> briefing/Attention
-> proposed actions
-> user approval
-> Effect Gate
-> dispatch/reconciliation
-> verification
```

### Provider failure

Preferred local/provider route becomes unavailable mid-task. Prove that fallback does not silently widen privacy, account, cost or authority and that dependent work does not proceed through ambiguous Effects.

### Resume after crash

Crash after an external request may have been accepted but before terminal evidence is persisted. Prove that Task/worker resume does not blind-retry and reconciles `UNKNOWN_OUTCOME`.

### Catalog poisoning/account confusion

Inject stale/malicious capability metadata and multiple provider accounts. Prove exact destination/account/effect semantics are independently bound and wrong-account dispatch is denied.

### Decision overconfidence

Feed an out-of-domain/high-confidence wrong DecisionProvider result. Prove deterministic policy and required authoritative verification prevent it from granting protected authority or verified completion.

## Current safe sequencing

1. Keep active Spec 006 PR #24 unchanged in scope.
2. Treat T203–T216 as planning-only extension tasks.
3. Qualify the planning PR on its exact new head after this extension.
4. Re-run independent architecture/security/governance review because the planning head changed.
5. Merge planning only after new-head findings and required checks are reconciled.
6. Do not change `specs/CURRENT.md` because no durable implementation lifecycle state changed.
7. After Spec 006 closes canonically, fetch live successor authority before creating any owning implementation spec.
8. Never use this file itself as implementation permission.

```text
ACTIVE_SPEC_006_PR_24_WIDENED=NO
CURRENT_POINTER_CHANGED=NO
CONSTITUTION_CHANGED=NO
NEW_PRODUCT_IMPLEMENTATION_STARTED=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
WAIVER_TAKEN=NO
```
