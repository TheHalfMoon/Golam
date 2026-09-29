# Golam Source Adoption Review — DBX, Paperclip and Synaplan — 2026-09-29

**Status:** PROGRAM RESEARCH / PLANNING ONLY  
**Authority:** No implementation, dependency, runtime, model or source admission. No Spec 006 scope expansion.

## 1. Exact reviewed source states

| Source | Exact reviewed revision | Public rights posture observed | Golam disposition |
| --- | --- | --- | --- |
| `t8y2/dbx` | `4269a61e2cf6c19afdcaba41fed1e57d6e5e3512` | Apache-2.0 | `HIGH_VALUE_STRUCTURED_DATA_CAPABILITY_DONOR_CANDIDATE` |
| `paperclipai/paperclip` | `24beb005755465f71a19ec92a85da0958d1b9740` | MIT | `HIGH_VALUE_WORK_ORCHESTRATION_AND_INTERACTION_DONOR_CANDIDATE` |
| `metadist/synaplan` | `e81eb3431deb3e242c3a114e8cbf08e2fbfd1e88` | Apache-2.0 | `HIGH_VALUE_WORKFLOW_DAG_READINESS_AND_PORTABILITY_DONOR_CANDIDATE` |

The founder explicitly states permission to use, copy, modify and selectively reuse the supplied source. That permission is a Source Foundry input, not automatic technical admission. Exact copied paths, generated/vendored code, dependency closure, assets, model artifacts, service terms and NOTICE obligations remain independently reconciled.

## 2. DBX — structured data access needs a first-class contract

DBX is useful to Golam primarily because it treats database access as a structured capability rather than an arbitrary shell/MCP call.

### High-value exact components

- `crates/dbx-core/src/ai/mcp_policy.rs`
  - hierarchical execution policy over global, group, connection and database scope;
  - stable connection/group identity;
  - current policy versions may opt into scoped overrides while legacy rules remain ceilings and cannot silently widen the global policy;
  - database-specific scope prevents cross-database writes/queries from bypassing narrower policy;
  - Salesforce DML requires an explicit per-connection opt-in in addition to ordinary write policy.
- `crates/dbx-sql-core/src/sql_risk.rs`
  - AST-based SQL classification into read-only, write, DDL and transaction classes;
  - conservative fallback for unknown statements;
  - detects dangerous/unbounded write shapes rather than treating every UPDATE/DELETE equally;
  - dialect-aware parsing and explicit transaction-control classification.
- `crates/dbx-core/assets/database-drivers.manifest.json` and related capability/trait projections
  - database-engine behavior is explicit metadata rather than guessed from a display name.
- `crates/dbx-core/src/query/two_phase_commit.rs`
  - useful only as an outcome-model/test reference for prepared/committing/committed/rolled-back/mixed/unknown states.

### Golam adoption

Golam should add one canonical structured-data capability family under T248 rather than exposing SQL as generic `OPEN_WORLD` text.

A provider may still be DBX, a native Golam adapter, a database-specific MCP server, or another qualified connector. The provider is replaceable; the protected semantics are Golam-owned.

Recommended core objects:

```text
DataSourceBinding
SchemaSnapshot
DataOperationPlan
DataScope
DataRiskAssessment
DataExecutionReceipt
DataTransactionReceipt
```

`DataSourceBinding` binds at minimum:

```text
source_id
provider/revision
connection/account binding
engine/dialect
host/destination identity where applicable
default database/schema
credential handle refs
locality/egress class
read/write ceiling
production/sensitivity classification
current generation
```

`DataOperationPlan` binds exact statement/query identity, parsed operation class, database/schema/object targets, parameter identities, row/byte/time limits, transaction semantics, expected schema revision and verification plan.

### DBX patterns to strengthen

- policy precedence is monotonic unless a current explicitly versioned rule is authorized to override at the narrower scope;
- stable connection identity is separate from display/group labels;
- cross-database qualified references require explicit scope evaluation;
- SQL parser failure is never interpreted as read-only;
- write risk is based on parsed semantics, not keyword substring alone;
- mutation with an obviously unbounded predicate receives higher consequence classification;
- database-native transaction state is evidence, not proof that every externally visible consequence was correctly applied.

### Explicit DBX non-adoptions

Do **not** import DBX two-phase commit as Golam's generic Effect engine.

The reviewed DBX implementation retries participant `commit()` calls. That may be appropriate only for a narrowly qualified participant protocol with proven idempotent commit semantics. It conflicts with Golam's general rule that an ambiguous external Effect cannot be blindly retried.

Use its `Mixed`/`Unknown` ideas as failure/test references only unless a future bounded database transaction spec proves exact safe retry semantics for the selected database/provider.

```text
SQL_PARSE_SUCCESS != QUERY_AUTHORIZATION
READ_ONLY_CLASSIFICATION != DATA_EGRESS_AUTHORIZATION
DATABASE_CONNECTION != ACCOUNT_AUTHORITY
SCHEMA_SNAPSHOT != CURRENT_SCHEMA_TRUTH_FOREVER
TRANSACTION_COMMIT_RETURN != VERIFIED_BUSINESS_OUTCOME
DBX_2PC != GOLAM_EFFECT_ENGINE
MIXED_TRANSACTION != SAFE_AUTOMATIC_RETRY
```

## 3. Paperclip — separate work structure, dependency, ownership and execution

Paperclip's strongest contribution is not its company metaphor. It is its explicit separation of concepts that are often collapsed in agent systems.

The reviewed `doc/execution-semantics.md` separates:

```text
STRUCTURE
DEPENDENCY
OWNERSHIP
EXECUTION
```

This closes a real Golam planning gap between T167's canonical Task identity and T205's runtime execution envelope.

### High-value exact components and patterns

- `doc/execution-semantics.md`
  - parent/sub-item structure is not automatically a blocker dependency;
  - agent ownership and live execution are distinct;
  - blocked/review states require a routable next owner/action rather than prose-only waiting;
  - checkout ownership and active execution use distinct identities;
  - stale-lock recovery is not a retry loop;
  - pre-dispatch configuration failures are surfaced as blockers rather than dispatched-then-failed runs;
  - work can carry goal ancestry without that ancestry becoming authority.
- `server/src/services/execution-control-reconciliation.ts`
  - deadline-driven reconciliation is bounded and lock-aware;
  - recovery does not silently retry uncertain external actions;
  - current owner/run bindings are rechecked transactionally;
  - a timed-out finalization produces an explicit recovery action and requires reconciliation of uncertain external work.
- current rich ACP foundation at `24beb005...`
  - durable questions/permission requests are persisted before presentation;
  - a response must match an outstanding exact request and one of the offered actions/typed-answer schemas;
  - successful settlement is persisted before publication/replay;
  - failed persistence fences the executor rather than publishing in-memory state;
  - replacement provider processes do not inherit unresolved approval promises;
  - bounded plan/question envelopes fail rather than hide an unseen suffix.
- product-level patterns
  - atomic task checkout;
  - hard budget gates;
  - persistent context across heartbeats/restarts;
  - goal ancestry;
  - explicit blocker dependencies;
  - portable templates with secret scrubbing and collision handling.

### Golam adoption

T249 should refine T167/T205 with one canonical work-graph and liveness contract.

Recommended additional objects/projections:

```text
WorkRelation
WorkClaimReceipt
WaitDescriptor
GoalRef / GoalPath
WorkLivenessProjection
```

A `WorkRelation` is explicitly one of:

```text
STRUCTURAL_PARENT
BLOCKS
DERIVED_FROM
REVIEW_OF
COORDINATES_WITH
```

Only relations with dependency semantics can gate execution.

`WorkClaimReceipt` identifies who currently owns the right to advance a Task. It is separate from:

- Task assignment;
- Worker identity;
- ExecutionEnvelope/runtime incarnation;
- capability lease;
- Effect authorization.

A wait is healthy only when it has a routable continuation such as a blocker, approval/review participant, scheduled recheck or explicit owner/action descriptor.

### Paperclip patterns to refine existing Golam tasks without a new authority system

- T183/T214/T247 ACP/client interaction should persist pending questions/permission requests before UI presentation and settle them durably before publication;
- a lost transport acknowledgment is not proof that a provider applied a permission reply;
- a provider replacement invalidates unresolved interaction promises rather than replaying them;
- task budget/resource exhaustion is a liveness gate consumed from T132/T207, not permission to change target/provider/privacy policy;
- portable templates may carry goals/roles/workflows but never secret values, capability leases, approvals or active authority.

### Explicit Paperclip non-adoptions

Golam remains personal/local single-owner by default under T243. Paperclip's multi-organization control plane is a future tenancy reference only.

Do not introduce an organization/company database as a prerequisite for ordinary Golam use. Do not equate an agent org-chart role with Golam authority.

```text
STRUCTURAL_PARENT != EXECUTION_DEPENDENCY
ASSIGNMENT != WORK_CLAIM
WORK_CLAIM != EXECUTION_RUN
EXECUTION_RUN != EFFECT_AUTHORIZATION
GOAL_ANCESTRY != AUTHORITY
HEARTBEAT != WORK_CLAIM
BLOCKED_WITHOUT_ROUTE != HEALTHY_WAIT
REVIEW_WAIT != EXECUTION_ACTIVE
BUDGET_AVAILABLE != EFFECT_AUTHORIZATION
AGENT_ROLE != CAPABILITY_LEASE
PAPERCLIP_COMPANY != GOLAM_TENANT
```

## 4. Synaplan — executable workflow DAG, readiness and portability

Synaplan contributes three bounded patterns that should extend, not replace, T206/T215/T244.

### High-value exact components

- `backend/src/Service/SavedTask/Graph/SavedTaskGraphValidator.php`
  - versioned graph schema;
  - bounded node count;
  - unique node IDs;
  - capability allowlist;
  - explicit dependencies;
  - self-dependency/cycle rejection;
  - graph inputs must come from trigger or declared dependencies;
  - approval controls may tighten behavior but cannot be promised on a node that cannot actually pause;
  - outbound webhook validation includes HTTPS and SSRF screening.
- `backend/src/Service/SavedTask/Graph/SavedTaskPlanFactory.php`
  - validated persisted graph becomes an executable task plan;
  - each node keeps capability, dependency, input and params identity;
  - reply/output node is explicit rather than inferred from model prose when declared.
- `backend/src/Service/SavedTask/Graph/SavedTaskGraphPortability.php`
  - export strips secrets and owner-bound IDs;
  - logical references are exported by stable semantic names/topics rather than local numeric IDs;
  - import returns an explicit checklist for missing mailbox/MCP/prompt bindings instead of fabricating replacements;
  - connection/tool references are rebound only to owned, enabled destination resources.
- `backend/src/Service/SelfAware/PlatformCapabilityInventory.php`
  - live capability report derives from the same runtime/configuration sources that actually gate behavior;
  - available, unavailable and deliberately unsupported capabilities are explicit;
  - remediation/alternative text is surfaced instead of hallucinating a capability;
  - optional modules can be absent without exposing half-working controls.

### Golam adoption

T250 should define the execution binding from T215 `WorkflowIR`/`SkillIR` to an actual bounded workflow run.

T215 remains the canonical workflow-definition/compiler owner. T250 must not create another workflow language.

Recommended runtime objects:

```text
WorkflowRunPlan
WorkflowNodeBinding
TriggerBinding
NodeAttempt
WorkflowRunCheckpoint
WorkflowImportChecklist
CapabilityReadinessSnapshot
```

Required graph semantics:

- immutable workflow revision identity;
- bounded node/edge count appropriate to the owning profile;
- acyclic dependency graph;
- every data input originates from trigger input or an explicitly declared upstream dependency;
- explicit output/reply/artifact node(s);
- capability/provider requirements are references only, never captured grants;
- node-level approval may only tighten the enclosing policy;
- triggers are explicit and versioned: manual, schedule, webhook/event or other separately admitted source;
- concurrency, catch-up, timeout and cancellation semantics are explicit;
- capability/readiness is revalidated at run start and protected node dispatch;
- pause/resume retains node attempts and Effect uncertainty rather than re-running the graph from scratch;
- import/export strips secret/account/lease/approval material and returns a deterministic unresolved-binding checklist;
- missing optional capability yields unavailable/degraded state and a remediation path, never a fabricated successful route.

### T206/T244 readiness refinement

Add a provider-neutral `CapabilityReadinessSnapshot` projection with at least:

```text
AVAILABLE
DEGRADED
UNAVAILABLE
KNOWN_ABSENT
UNKNOWN
```

A snapshot names the source observations, freshness, exact provider/account/backend dependencies, reason and remediation. It is a projection over actual gating state, not a hand-maintained promise and not authority.

```text
WORKFLOW_IR != ACTIVE_WORKFLOW_RUN
SAVED_GRAPH != CAPABILITY_GRANT
NODE_DEPENDENCY != DATA_AUTHORITY
APPROVAL_NODE != POLICY_WIDENING
CAPABILITY_ADVERTISED != CAPABILITY_READY
CAPABILITY_READY != EFFECT_AUTHORIZED
KNOWN_ABSENT != FAILED
OPTIONAL_MODULE_ABSENT != PARTIAL_SUCCESS
WORKFLOW_IMPORT != SECRET_IMPORT
WORKFLOW_IMPORT != AUTHORITY_IMPORT
```

## 5. Combined architecture placement

These sources fit Golam as three bounded additions around the existing protected spine:

```text
DBX
  -> Structured Data Capability Provider patterns
  -> T248

Paperclip
  -> Work graph / ownership / liveness / durable interaction patterns
  -> T249 + refinements to T183/T214/T247

Synaplan
  -> Workflow DAG runtime / capability readiness / safe portability patterns
  -> T250 + refinements to T206/T244
```

None becomes a new Authority Kernel, Task ledger, Effect ledger, Secret store or policy engine.

## 6. Reuse strategy

### DBX

Preferred order:

```text
PORT_TO_RUST / SELECTIVE_COPY
```

Golam is already Rust-first, so exact bounded Rust components/tests around SQL parsing/risk and policy scope are strong donor candidates after Source Foundry qualification. Database-driver breadth should remain an adapter/provider concern; do not copy 100+ drivers into the privileged path.

`two_phase_commit.rs` is `BENCHMARK_OR_METHOD_REFERENCE` by default, not a generic runtime donor.

### Paperclip

Preferred order:

```text
REIMPLEMENT_BEHAVIOR / SELECTIVE_COPY_OF_PURE_BOUNDED_COMPONENTS
```

The Node/React/Postgres control plane is not a Golam runtime dependency. Port semantic contracts, tests and small pure helpers where valuable. Keep task/effect/identity authority Golam-owned.

### Synaplan

Preferred order:

```text
REIMPLEMENT_BEHAVIOR / SELECTIVE_PORT
```

Port validator, portability and readiness semantics into Golam's typed Rust contracts rather than importing the Symfony/PHP runtime.

## 7. Implementation-readiness consequences

The new source set exposes exactly three material gaps:

```text
T248 Structured Data Source / Query / Mutation Safety Contract
T249 Work Graph / Ownership / Dependency / Liveness Contract
T250 Workflow DAG Runtime / Trigger / Readiness / Portability Contract
```

No fourth task is needed. ACP durable interaction settlement fits T183/T214/T247. Capability readiness fits T206/T244. Multi-tenant organization management remains behind T243 future-mode qualification. Database distributed transaction behavior is bounded inside T248 and does not modify the generic Effect FSM.

## 8. Planning disposition

```text
DBX_SOURCE_REVIEWED=YES
PAPERCLIP_SOURCE_REVIEWED=YES
SYNAPLAN_SOURCE_REVIEWED=YES
FOUNDER_PERMISSION_ASSERTED_FOR_ALL_THREE=YES

NEW_TASKS=T248,T249,T250
NEW_PARALLEL_TASK_LEDGER=NO
NEW_PARALLEL_EFFECT_LEDGER=NO
NEW_PARALLEL_POLICY_ENGINE=NO
NEW_TENANCY_BASELINE=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_MODEL_ADMITTED=NO
ACTIVE_SPEC_006_PR_24_WIDENED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
WAIVER_TAKEN=NO
```
