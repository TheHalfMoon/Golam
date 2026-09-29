# Golam Canonical Direction Task Extension — 2026-09-29

**Authority:** PROGRAM ORCHESTRATION / IMPLEMENTATION-READINESS PLANNING ONLY  
**Extends:** `program-direction-tasks-2026-09-22.md` through T247  
**Source review:** `source-adoption-dbx-paperclip-synaplan-2026-09-29.md`  
**Permission record:** `source-permission-attestation-supplement-2026-09-29.md`

This extension adds only measured gaps exposed by the exact-pinned DBX, Paperclip and Synaplan review. It does not widen active Spec 006, alter `specs/CURRENT.md`, amend the Constitution, admit source/dependencies/runtimes, or grant successor implementation authority.

## Phase AB — Structured data, work liveness and executable workflow closure

- [ ] **T248 — Structured Data Source / Query / Mutation Safety Contract.** Extend T119/T152/T165/T179/T190/T199/T201/T206/T207/T216/T238/T245 with one provider-neutral structured-data capability family for relational databases and database-like systems. Define canonical `DataSourceBinding`, `SchemaSnapshot`, `DataOperationPlan`, `DataScope`, `DataRiskAssessment`, `DataExecutionReceipt` and bounded `DataTransactionReceipt`. Bind exact provider/revision, connection/account identity, engine/dialect, endpoint/destination where applicable, default database/schema, secret handles, locality/egress class, read/write ceiling, production/sensitivity classification and current generation. Parse supported SQL/structured operations into explicit semantic classes such as read, write, DDL, transaction control and unknown; parser failure/unknown syntax fails closed to a conservative consequence class rather than read-only. Bind exact database/schema/object targets, parameters, row/byte/time limits, schema revision, transaction semantics and verification obligations before dispatch. Database-specific policy is monotonic unless a current explicitly versioned narrower-scope rule is authorized to override; legacy/unknown rules can only preserve or tighten a ceiling. Qualified references to another database/schema/tenant must be independently authorized and cannot inherit the active database's scope by textual qualification. Detect obviously unbounded/high-blast-radius mutations and escalate consequence/approval rather than trusting a model's intent description. Reads remain governed by data classification, privacy and egress; `SELECT` is not automatically safe to disclose. Use `t8y2/dbx@4269a61e2cf6c19afdcaba41fed1e57d6e5e3512` as the primary bounded donor/reference for SQL risk classification, connection/database scope and database-driver capability metadata. Treat DBX two-phase-commit only as an outcome/failure/test reference by default: its participant commit retries must never weaken Golam's at-most-once/`UNKNOWN_OUTCOME` semantics. Distributed/database transaction coordination is admitted only for a future exact provider protocol whose commit/retry semantics are independently proven safe.

- [ ] **T249 — Work Graph / Ownership / Dependency / Liveness Contract.** Refine T127/T129/T130/T132/T163/T167/T184/T205/T214/T216/T231/T247 so work structure, execution dependency, assignment, current work ownership and live execution are never conflated. Define canonical `WorkRelation`, `WorkClaimReceipt`, `WaitDescriptor`, `GoalRef` / `GoalPath` and `WorkLivenessProjection`. At minimum distinguish `STRUCTURAL_PARENT`, `BLOCKS`, `DERIVED_FROM`, `REVIEW_OF` and `COORDINATES_WITH`; only relations with explicit dependency semantics gate readiness. Assignment names the intended owner; a `WorkClaimReceipt` identifies the current actor/run permitted to advance work and is generation/revision-bound; it is not a capability lease, approval or Effect authorization. Runtime incarnation/execution remains T205-owned and distinct from the work claim. Define actionable states such as not-ready, ready, claimed/active, waiting/blocked, review/decision wait, paused, terminal-satisfied, terminal-unsatisfied/cancelled and attention-required without using UI labels as authority. Any non-terminal waiting state must have a routable continuation: unresolved blocker, exact pending approval/review participant, scheduled/monitored recheck, provider recovery action or structured `{owner, action}` remediation. Prose-only waiting is unhealthy and must surface attention rather than strand work. Known missing configuration/secret/workspace/provider prerequisites are pre-dispatch liveness gates and should not be represented as runtime execution failures. Stale claim recovery is reconciliation, not automatic Effect retry; a successor claim cannot erase `UNKNOWN_OUTCOME`. Goal ancestry is context/provenance for prioritization and explanation, not authority. Use `paperclipai/paperclip@24beb005755465f71a19ec92a85da0958d1b9740` as a high-value donor/reference for structure/dependency/ownership/execution separation, atomic checkout patterns, liveness/recovery, routed blockers, goal ancestry, budget gates and durable interaction settlement. Preserve Golam's T243 `PERSONAL_SINGLE_OWNER` baseline; Paperclip multi-organization behavior remains a future tenancy reference only.

- [ ] **T250 — Workflow DAG Runtime / Trigger / Readiness / Portability Contract.** Extend T122/T123/T129/T130/T162/T167/T180/T205/T206/T207/T215/T216/T233/T241/T244/T245/T249. T215 remains the sole canonical owner of `WorkflowIR` / `SkillIR` definition, compiler, replay and divergence-repair semantics; T250 defines the runtime binding of one immutable admitted workflow revision to an actual run. Define `WorkflowRunPlan`, `WorkflowNodeBinding`, `TriggerBinding`, `NodeAttempt`, `WorkflowRunCheckpoint`, `WorkflowImportChecklist` and provider-neutral `CapabilityReadinessSnapshot`. Validate bounded acyclic graphs, unique node identities, declared capabilities, dependency existence, input lineage, explicit output/artifact/reply nodes where relevant and compatibility/version constraints before activation/run. Every node input must derive from admitted trigger material, explicit workflow constants or an upstream dependency; undeclared cross-node ambient state is forbidden. A node-level approval setting may only preserve or tighten the enclosing authority/policy; it cannot promise an approval boundary for a node/runtime unable to pause safely. Trigger types such as manual, schedule, webhook/event, connector observation or future admitted sources are exact/versioned `InputEnvelope` producers and inherit T129/T130/T241/T247 replay, dedup, ordering and catch-up rules. Define concurrency, overlap/coalescing, catch-up, timeout, cancellation, retry and reconciliation per workflow/node class. Capability readiness is revalidated at run admission and immediately before protected node dispatch; cached availability is not dispatch proof. Pause/resume/checkpoint preserves node attempts, prior outputs, receipts and Effect uncertainty rather than restarting the graph. Workflow export/import strips secrets, account-local IDs, capability leases, approvals and active authority; portable logical references are rebound only to qualified destination resources and unresolved bindings produce a deterministic checklist instead of guessed replacements. Use `metadist/synaplan@e81eb3431deb3e242c3a114e8cbf08e2fbfd1e88` as the primary behavior/source reference for versioned graph validation, input/dependency validation, bounded node counts, approval tightening, SSRF-aware outbound nodes, plan construction, portability/rebinding and live capability inventory.

## T248 canonical contract minimums

### `DataSourceBinding`

```text
source_id
provider_id / provider_revision
connection_id / account_binding
engine / dialect
endpoint_or_destination_identity
locality / egress_class
default_database / default_schema
secret_handle_refs
read_write_ceiling
production_or_sensitivity_class
binding_generation
observed_at / freshness
```

### `SchemaSnapshot`

```text
source_id
binding_generation
schema_revision_or_digest
captured_at
catalog / database / schema / objects
observability_limits
provider_revision
```

The snapshot is evidence for planning and validation. It is not perpetual truth; current-target identity and relevant constraints must be revalidated at the protected boundary when mutation safety depends on them.

### `DataOperationPlan`

```text
operation_id
source_binding_ref
schema_snapshot_ref
statement_or_request_digest
parsed_operation_class
targets[]
parameter_identities
row / byte / time limits
read_write / destructive / open_world class
data_class / egress purpose
transaction_semantics
idempotency / reconciliation semantics
verification_obligations
```

### T248 hard invariants

```text
SQL_PARSE_SUCCESS != QUERY_AUTHORIZATION
READ_ONLY_CLASSIFICATION != DATA_EGRESS_AUTHORIZATION
DATABASE_CONNECTION != ACCOUNT_AUTHORITY
DATABASE_NAME != DATABASE_SCOPE_PROOF
SCHEMA_SNAPSHOT != CURRENT_SCHEMA_TRUTH_FOREVER
MODEL_GENERATED_SQL != APPROVED_OPERATION
DDL != ORDINARY_WRITE
TRANSACTION_CONTROL != ORDINARY_QUERY
TRANSACTION_COMMIT_RETURN != VERIFIED_BUSINESS_OUTCOME
DBX_2PC != GOLAM_EFFECT_ENGINE
MIXED_TRANSACTION != SAFE_AUTOMATIC_RETRY
UNKNOWN_TRANSACTION_OUTCOME != RETRY_PERMISSION
```

### T248 acceptance requirements

A future owning spec must prove at least:

1. supported read/write/DDL/transaction cases are parsed across selected dialects and unknown/unparseable statements fail conservatively;
2. an apparently read-only request that causes mutation in the selected dialect/provider cannot be admitted as a simple read;
3. unbounded/high-blast-radius UPDATE/DELETE-like mutations are detected or conservatively escalated;
4. cross-database/schema references cannot bypass a narrower scoped rule;
5. stale/mismatched schema evidence invalidates or revalidates the operation rather than silently retargeting it;
6. data reads respect row/byte/time/resource and privacy/egress limits;
7. transaction rollback/partial/mixed/unknown states preserve exact participant evidence and never cause blind replay of an ambiguous mutation;
8. database/provider removal leaves canonical receipts/evidence readable and does not make donor-specific state authoritative.

## T249 work graph semantics

### `WorkRelation`

```text
relation_id
from_task
to_task
kind
revision
created_by / provenance
active_from / superseded_by
```

Structural parentage is not execution dependency unless a separate `BLOCKS` edge says so.

### `WorkClaimReceipt`

```text
claim_id
task_id
task_revision
claimant_principal / worker / run refs
claim_generation
claimed_at
expires_or_reconciliation_policy
expected_current_owner
supersedes_claim
```

The claim grants only the bounded right to coordinate/advance the task state permitted by current policy. Protected operations still require their ordinary capability/approval/Effect path.

### `WaitDescriptor`

```text
wait_kind
blocking_refs[]
owner_or_responder
action_or_condition
recheck_or_wake_policy
freshness / expiry
recovery_evidence
```

### T249 hard invariants

```text
STRUCTURAL_PARENT != EXECUTION_DEPENDENCY
ASSIGNMENT != WORK_CLAIM
WORK_CLAIM != EXECUTION_RUN
WORK_CLAIM != CAPABILITY_LEASE
EXECUTION_RUN != EFFECT_AUTHORIZATION
GOAL_ANCESTRY != AUTHORITY
HEARTBEAT != WORK_CLAIM
BLOCKED_WITHOUT_ROUTE != HEALTHY_WAIT
REVIEW_WAIT != EXECUTION_ACTIVE
CONFIGURATION_INCOMPLETE != RUNTIME_FAILURE
BUDGET_AVAILABLE != EFFECT_AUTHORIZATION
AGENT_ROLE != CAPABILITY_LEASE
PAPERCLIP_COMPANY != GOLAM_TENANT
STALE_CLAIM_RECOVERY != EFFECT_RETRY
```

### T249 acceptance requirements

A future owning spec must prove:

1. two workers racing to claim one task produce one current claim without double execution ownership;
2. stale claim cleanup cannot clear a live successor claim and does not replay protected work;
3. parent/child structure alone cannot make a child block a parent or vice versa;
4. an explicit blocker becoming terminal in a non-satisfying state does not silently satisfy the dependency;
5. waiting without an actionable route becomes `attention_required`/equivalent rather than a silent dead state;
6. pending reviewer/approver identity survives restart and is not inherited by an unrelated provider/worker replacement;
7. missing secrets/config/workspace/provider readiness prevents dispatch as a liveness gate with remediation evidence;
8. goal ancestry improves context/priority explanation but cannot grant authority;
9. resource/budget exhaustion pauses/prevents new work without mutating privacy/account/effect policy;
10. `UNKNOWN_OUTCOME` remains visible and blocks conflicting dependent work across claim handoff.

## T250 workflow runtime semantics

### `CapabilityReadinessSnapshot`

Minimum state vocabulary:

```text
AVAILABLE
DEGRADED
UNAVAILABLE
KNOWN_ABSENT
UNKNOWN
```

Each snapshot binds:

```text
capability_id
selected_offer/provider/account/backend refs
state
reason
source observations / revision
observed_at / freshness
remediation
known limitations
```

It is a projection of actual gating state. It does not authorize a call.

### `WorkflowRunPlan`

```text
workflow_revision
run_id
trigger_binding
node_bindings[]
input_lineage
selected capability offers where frozen by policy
concurrency/catch_up policy
resource/time budget
checkpoint policy
verification obligations
```

### T250 hard invariants

```text
WORKFLOW_IR != ACTIVE_WORKFLOW_RUN
SAVED_GRAPH != CAPABILITY_GRANT
NODE_DEPENDENCY != DATA_AUTHORITY
NODE_OUTPUT != DOWNSTREAM_TRUTH_WITHOUT_PROVENANCE
APPROVAL_NODE != POLICY_WIDENING
CAPABILITY_ADVERTISED != CAPABILITY_READY
CAPABILITY_READY != EFFECT_AUTHORIZED
CACHED_READINESS != DISPATCH_READINESS_PROOF
KNOWN_ABSENT != FAILED
OPTIONAL_MODULE_ABSENT != PARTIAL_SUCCESS
WORKFLOW_IMPORT != SECRET_IMPORT
WORKFLOW_IMPORT != AUTHORITY_IMPORT
PAUSE_RESUME != GRAPH_REPLAY_FROM_ZERO
WORKFLOW_RETRY != EFFECT_RETRY
```

### T250 acceptance requirements

A future owning spec must prove:

1. duplicate IDs, missing dependencies, cycles, undeclared input lineage and incompatible workflow versions are rejected before activation/run;
2. graph/node limits are explicit and malicious expansion cannot exhaust the planner/runtime unchecked;
3. node-level approval can tighten but cannot widen the surrounding policy;
4. webhook/network trigger targets obey SSRF/egress and T241/T247 input-integrity rules;
5. schedule/event catch-up and overlap semantics are deterministic across restart/clock changes;
6. readiness changes between planning and dispatch are detected before protected node execution;
7. pause/restart resumes only eligible nodes and preserves already-attempted Effect uncertainty;
8. import/export contains no secret, approval, lease, account-local authority or active runtime identity;
9. destination rebinding requires an exact qualified destination or produces an unresolved checklist item;
10. optional/known-absent capabilities produce explicit degradation/unavailability and alternatives rather than exposing half-working controls;
11. T215 replay/divergence semantics remain canonical and T250 does not create a second workflow language.

## Refinements to existing tasks — no new IDs

### T183 / T214 / T247 — durable ACP/client interaction settlement

Paperclip's current rich ACP work strengthens the existing interoperability/input contracts:

- pending questions/permission requests must become durable before UI presentation when a later response is expected;
- response binds the exact outstanding request revision/session and one of the offered actions or typed-answer schema;
- settlement is persisted before successful publication/replay acknowledgment;
- failed persistence fences further command acceptance/publication for that executor until recovery;
- provider/runner replacement cannot inherit an unresolved approval promise;
- lost transport acknowledgment is not proof that the provider applied the reply exactly once;
- large plan/question content is bounded and must fail or explicitly truncate before approval rather than hide an unseen suffix.

These refine existing T183/T214/T247 surfaces and do not create `T251`.

### T206 / T244 — live capability readiness

Synaplan's live capability inventory strengthens existing capability/health projections. `CapabilityReadinessSnapshot` is owned by the T206/T244 contract family and consumed by T250. Availability must derive from the same observed provider/model/module/connection state that gates execution, with explicit `KNOWN_ABSENT`, `UNAVAILABLE`, `DEGRADED` and remediation semantics.

This does not create a separate capability registry or `T251`.

### T199 / Effect FSM — database transaction non-adoption

DBX transaction outcome vocabulary may inform T248 test fixtures, but generic DBX two-phase commit is not adopted into T199. Golam's existing Effect FSM / `UNKNOWN_OUTCOME` semantics remain authoritative.

## Canonical shared-contract ownership extension

Before T248–T250 implementation, T198's shared-contract ownership matrix must assign exactly one owner/version/migration authority for:

```text
DataSourceBinding
SchemaSnapshot
DataOperationPlan
DataRiskAssessment
DataExecutionReceipt
DataTransactionReceipt
WorkRelation
WorkClaimReceipt
WaitDescriptor
GoalRef / GoalPath
WorkLivenessProjection
WorkflowRunPlan
WorkflowNodeBinding
TriggerBinding
NodeAttempt
WorkflowRunCheckpoint
WorkflowImportChecklist
CapabilityReadinessSnapshot
```

No adapter/provider/UI package may invent package-local protected truth for these concepts.

## Dependency and implementation order

After a future owning lifecycle is authorized:

```text
P0_SHARED_OWNERSHIP:
  T198 update / shared-contract owner freeze

P1_PARALLEL_FOUNDATIONS:
  T248 Structured Data Source / Query / Mutation Safety
  T249 Work Graph / Ownership / Dependency / Liveness

P1_REFINEMENTS:
  T183/T214/T247 durable interaction settlement
  T206/T244 live CapabilityReadinessSnapshot

P2_AFTER_T215_AND_T249:
  T250 Workflow DAG Runtime / Trigger / Readiness / Portability
```

T250 depends on the canonical T215 workflow definition semantics and T249 liveness/dependency semantics. T248 is independently implementable after the shared ownership and data/egress/secret prerequisites are frozen.

## No-gap source disposition

| Source capability | Existing owner | New work |
| --- | --- | --- |
| DBX MCP/database policy hierarchy | T206/T207/T245 insufficient for SQL semantics | T248 |
| DBX SQL AST/risk | no structured-data contract | T248 |
| DBX 2PC | T199/Effect FSM already stronger for general external effects | reference only; T248 fixtures |
| Paperclip task/session runtime | T167/T205 | T249 refinement, no donor ledger |
| Paperclip structure vs dependency vs ownership vs execution | not explicit | T249 |
| Paperclip durable ACP questions/permissions | T183/T214/T247 | refine existing tasks |
| Paperclip multi-org | T243 future boundary | no baseline adoption |
| Paperclip budget gates | T132/T207 | liveness integration in T249 |
| Synaplan workflow graph | T215 definition exists but runtime binding incomplete | T250 |
| Synaplan graph portability | T215/T245 partially cover | T250 |
| Synaplan live capability inventory | T206/T244 | refine readiness projection |
| Synaplan air-gap/self-host patterns | T201/T237–T245 already cover lifecycle | reference/acceptance evidence |

No additional material task gap was identified after allocating these concerns.

## Current safe sequencing

1. Keep active Spec 006 PR #24 unchanged in scope.
2. Treat T248–T250 as planning-only until this planning overlay is canonical and live successor authority permits owning specs.
3. Do not change `specs/CURRENT.md` from this planning extension.
4. Qualify PR #28 on its exact final head after this extension.
5. Run fresh independent Jev + Alibaba Open Code Review / equivalent repository-authorized review on the exact head and reconcile material findings without waiver.
6. Merge the planning PR only after exact-head CI/review gates satisfy repository governance.
7. After Spec 006 implementation closes canonical, fetch live successor authority before opening implementation specs for T248+.
8. Source copying requires component-level Source Foundry admission even though founder permission is recorded.

```text
TASK_GRAPH_EXTENDS_THROUGH_T250=YES
MEASURED_NEW_GAPS=3
T248_DEFINED=YES
T249_DEFINED=YES
T250_DEFINED=YES
NEW_PARALLEL_AUTHORITY_SYSTEM=NO
NEW_PARALLEL_TASK_LEDGER=NO
NEW_PARALLEL_EFFECT_LEDGER=NO
NEW_PARALLEL_WORKFLOW_LANGUAGE=NO
ACTIVE_SPEC_006_PR_24_WIDENED=NO
CURRENT_POINTER_CHANGED=NO
CONSTITUTION_CHANGED=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_PRODUCT_IMPLEMENTATION_STARTED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
WAIVER_TAKEN=NO
```
