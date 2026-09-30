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

Detailed permission record: `source-permission-attestation-supplement-2026-09-29.md`.

## 2. DBX — structured data access needs a first-class contract

DBX is valuable because it models database access as a typed capability instead of arbitrary shell/MCP text.

### High-value exact components

- `crates/dbx-core/src/ai/mcp_policy.rs`
  - global/group/connection/database policy layers;
  - stable connection/group identity;
  - versioned narrower-scope override semantics;
  - legacy rules remain ceilings and cannot silently widen a global restriction;
  - cross-database SQL and Mongo output scope checks;
  - explicit per-connection DML opt-in for special providers such as Salesforce.
- `crates/dbx-sql-core/src/sql_risk.rs`
  - AST classification into read-only, write, DDL and transaction control;
  - dialect-aware parsing;
  - conservative unknown-statement handling;
  - unbounded/high-blast-radius UPDATE/DELETE-style risk detection.
- `crates/dbx-core/assets/database-drivers.manifest.json` plus capability/trait projections
  - provider behavior is explicit metadata rather than inferred from display names.
- `crates/dbx-core/src/query/two_phase_commit.rs`
  - useful as a transaction-outcome and failure-fixture reference only by default.

### Golam adoption

Golam should add one canonical structured-data family under T248 instead of treating SQL/database work as generic `OPEN_WORLD` text.

Canonical concepts include:

```text
DataSourceBinding
SchemaSnapshot
DataOperationPlan
DataScope
DataRiskAssessment
DataExecutionReceipt
DataTransactionReceipt
```

A provider may be a bounded DBX-derived adapter, a native Golam adapter, a database-specific MCP server or another qualified connector. The provider is replaceable; protected semantics remain Golam-owned.

Key retained patterns:

- policy precedence is monotonic unless a current explicit rule is permitted to override at a narrower scope;
- connection identity is separate from display/group labels;
- cross-database/schema references are evaluated independently;
- parser failure never becomes read-only;
- mutation consequence comes from parsed semantics, not model prose or keyword substring alone;
- sensitive reads still require privacy/egress authorization;
- transaction/provider state is evidence, not Golam authority.

### Explicit DBX non-adoption

Do not import DBX two-phase commit as the generic Golam Effect engine.

The reviewed implementation may retry participant `commit()` calls. Such retries are acceptable only for a future narrowly qualified provider protocol whose commit operation is proven safely repeatable. They must not weaken Golam's general at-most-once / `UNKNOWN_OUTCOME` rules.

```text
SQL_PARSE_SUCCESS != QUERY_AUTHORIZATION
READ_ONLY_CLASSIFICATION != DATA_EGRESS_AUTHORIZATION
DATABASE_CONNECTION != ACCOUNT_AUTHORITY
SCHEMA_SNAPSHOT != CURRENT_SCHEMA_TRUTH_FOREVER
TRANSACTION_COMMIT_RETURN != VERIFIED_BUSINESS_OUTCOME
DBX_2PC != GOLAM_EFFECT_ENGINE
MIXED_TRANSACTION != SAFE_AUTOMATIC_RETRY
```

## 3. Paperclip — separate structure, dependency, ownership and execution

Paperclip's strongest contribution is its explicit separation of concepts commonly collapsed by agent systems:

```text
STRUCTURE
DEPENDENCY
OWNERSHIP
EXECUTION
```

This closes a material gap between T167 canonical Task identity and T205 runtime execution.

### High-value exact components and patterns

- `doc/execution-semantics.md`
  - parent/sub-item structure is not automatically a blocker dependency;
  - agent ownership and live execution are separate;
  - blocked/review states require a routable next owner/action rather than prose-only waiting;
  - checkout ownership and active execution use distinct identities;
  - stale-lock recovery is reconciliation, not a retry loop;
  - known missing configuration becomes a pre-dispatch blocker instead of a doomed dispatched run;
  - goal ancestry supplies context without becoming authority.
- `server/src/services/execution-control-reconciliation.ts`
  - bounded, lock-aware reconciliation;
  - transactionally rechecks current owner/run binding;
  - explicit recovery action for uncertain external work;
  - no silent replay of ambiguous external action.
- rich ACP foundation at the reviewed commit
  - pending question/permission state becomes durable before presentation;
  - response must match an exact outstanding request and offered action/schema;
  - settlement persists before successful publication/replay;
  - persistence failure fences the executor;
  - provider replacement does not inherit unresolved approval promises;
  - lost transport acknowledgment does not prove provider application exactly once;
  - oversized plans/questions fail rather than hide unseen content.
- product/runtime patterns
  - atomic work checkout;
  - hard budget gates;
  - persistent context across wake/restart;
  - explicit blockers;
  - portable templates with secret scrubbing and collision handling.

### Golam adoption

T249 defines a canonical work/liveness layer over existing T167/T205 semantics:

```text
WorkRelation
WorkClaimReceipt
WaitDescriptor
GoalRef / GoalPath
WorkLivenessProjection
```

A relation is explicitly structural, blocking, derivational, review or coordination. Only declared dependency semantics gate execution.

A work claim is separate from assignment, Worker identity, ExecutionEnvelope, capability lease and Effect authorization.

A wait is healthy only with a routable continuation: blocker, reviewer/approver, monitored recheck or structured owner/action remediation.

### Existing-task refinements from Paperclip

T183/T214/T247 should carry durable client/ACP interaction settlement: persist before presentation, exact request correlation, durable settlement before publication, and no unsafe provider-replacement inheritance.

Budget/resource exhaustion is a liveness gate consumed from existing resource/cost governance; it is never permission to change target/provider/privacy policy.

### Explicit Paperclip non-adoption

Golam remains personal/local single-owner by default under T243. Paperclip's multi-organization control plane is a future tenancy reference only.

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

Synaplan contributes bounded patterns that extend T206/T215/T244 without replacing them.

### High-value exact components

- `backend/src/Service/SavedTask/Graph/SavedTaskGraphValidator.php`
  - versioned graph schema;
  - bounded node count;
  - unique IDs;
  - capability allowlist;
  - explicit dependencies;
  - unknown/self dependency and cycle rejection;
  - input lineage restricted to trigger or declared dependencies;
  - approval may tighten only on nodes that can actually pause;
  - HTTPS/SSRF checks for outbound webhook nodes.
- `backend/src/Service/SavedTask/Graph/SavedTaskPlanFactory.php`
  - validates persisted graph before constructing an executable plan;
  - preserves capability/dependency/input/parameter identity;
  - supports explicit reply/output node identity.
- `backend/src/Service/SavedTask/Graph/SavedTaskGraphPortability.php`
  - strips secrets and owner-local IDs;
  - exports logical prompt/tool references using portable semantic identity;
  - rebinds only to owned/enabled destination resources;
  - returns a deterministic checklist for unresolved bindings rather than inventing replacements.
- `backend/src/Service/SelfAware/PlatformCapabilityInventory.php`
  - derives readiness from state that actually gates behavior;
  - distinguishes availability and deliberately unsupported capabilities;
  - gives remediation/alternatives instead of hallucinating capability;
  - optional modules may remain absent without exposing half-working controls.

### Golam adoption

T215 remains the canonical owner of portable `WorkflowIR` / `SkillIR` meaning. T250 owns runtime binding of one admitted revision to triggers, nodes, attempts, readiness and checkpoints.

Runtime concepts include:

```text
WorkflowRunPlan
WorkflowNodeBinding
TriggerBinding
NodeAttempt
WorkflowRunCheckpoint
WorkflowImportChecklist
CapabilityReadinessSnapshot
```

Required semantics include bounded acyclic graphs, declared data lineage, explicit outputs, non-authoritative capability references, approval that can only tighten, exact trigger/replay policy, overlap/catch-up/cancellation semantics, readiness revalidation, restart-safe checkpoints and authority-free portability.

### T206/T244 readiness refinement

A provider-neutral readiness snapshot should distinguish:

```text
AVAILABLE
DEGRADED
UNAVAILABLE
KNOWN_ABSENT
UNKNOWN
```

It records exact observations, freshness, dependencies, reason and remediation. It is a projection, not an authority decision.

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

```text
DBX
  -> Structured Data Capability patterns
  -> T248

Paperclip
  -> Work graph / ownership / liveness / durable interaction patterns
  -> T249 + T183/T214/T247 refinements

Synaplan
  -> Workflow DAG runtime / readiness / safe portability patterns
  -> T250 + T206/T244 refinements
```

None becomes an Authority Kernel, Task ledger, Effect ledger, Secret store, policy engine or baseline tenancy control plane.

## 6. Reuse strategy

### DBX

Preferred:

```text
PORT_TO_RUST / SELECTIVE_COPY
```

Golam is already Rust-first, so exact pure Rust SQL-risk/policy logic and tests are strong donor candidates after Source Foundry qualification. Database-driver breadth remains an adapter concern. `two_phase_commit.rs` is reference/test material by default.

### Paperclip

Preferred:

```text
REIMPLEMENT_BEHAVIOR / SELECTIVE_COPY_OF_PURE_BOUNDED_COMPONENTS
```

Do not import the Node/React/Postgres control plane as a Golam runtime dependency. Port semantic contracts/tests and narrowly useful pure components.

### Synaplan

Preferred:

```text
REIMPLEMENT_BEHAVIOR / SELECTIVE_PORT
```

Port validator, portability and readiness semantics into Golam's typed Rust contracts instead of importing Symfony/PHP runtime.

## 7. Measured gap disposition

Exactly three material new task gaps were identified:

```text
T248 Structured Data Source / Query / Mutation Safety Contract
T249 Work Graph / Ownership / Dependency / Liveness Contract
T250 Workflow DAG Runtime / Trigger / Readiness / Portability Contract
```

No fourth task is needed:

- ACP durable interaction settlement fits T183/T214/T247;
- capability readiness fits T206/T244;
- multi-tenant organization management remains behind T243;
- generic Effect ambiguity remains T199-owned;
- database distributed transaction behavior stays bounded under T248;
- workflow definition/compiler/replay remains T215-owned.

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
