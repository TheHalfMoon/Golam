# Golam Program Implementation Readiness — 2026-09-29

**Status:** `CONTENT_IMPLEMENTATION_READY / EXECUTION_NOT_YET_AUTHORIZED`  
**Planning PR:** #28  
**Active product implementation:** Spec 006 / PR #24  
**Reviewed canonical main:** `13a379ac478a3abaff7ed1da3db14ff9c1ac2188`

## 1. Meaning of readiness

This document distinguishes two states that must never be conflated:

```text
CONTENT_IMPLEMENTATION_READY
!=
IMPLEMENTATION_AUTHORIZED
```

`CONTENT_IMPLEMENTATION_READY` means the planning contract now specifies:

- bounded scope;
- canonical ownership dependencies;
- required data/contracts;
- invariants;
- failure semantics;
- source-donor posture;
- acceptance evidence;
- implementation package ordering;
- removal/fallback boundaries.

`IMPLEMENTATION_AUTHORIZED` requires live repository governance at implementation start.

As of this review:

```text
CANONICAL_MAIN=13a379ac478a3abaff7ed1da3db14ff9c1ac2188
ACTIVE_SPEC_006_PR=24
ACTIVE_SPEC_006_STATE=OPEN_DRAFT
SPEC_006_CLOSED_CANONICAL=NO
PLANNING_PR_28=OPEN
```

Therefore T248–T250 are specification-ready but must not be implemented as authorized program work yet.

## 2. Implementation entry gates

Before the first implementation commit for T248/T249/T250, all must be true:

1. fetch live `main`, `AGENTS.md`, Constitution, `specs/CURRENT.md`, PR #28 and active Spec 006 state;
2. PR #28 planning overlay is merged canonically with exact-head CI and independent review evidence required by repository governance;
3. Spec 006 is `CLOSED_CANONICAL=YES` with post-merge canonical-main evidence;
4. live successor authority identifies the next bounded owning Spec Kit package;
5. T198 shared-contract ownership matrix assigns one owner/version/migration authority for every T248–T250 cross-cutting type;
6. the owning package freezes exact scope, scope-out, interfaces, threat model and acceptance fixtures before production code;
7. every copied/ported donor component has an exact Source Foundry record before its code enters the implementation branch;
8. no unresolved material architecture/security/review finding exists for the exact owning-plan head;
9. no implementation package depends on an unadmitted hosted service, model, runtime or secret;
10. implementation uses normal forward history and ordinary merge commits; no force-push/history rewriting.

If any gate fails, implementation remains blocked rather than inferring authority from this planning document.

## 3. Recommended bounded implementation packages

### Package A — Structured Data Capability

**Primary task:** T248  
**Purpose:** database/data-source operations with explicit target identity, parsed semantics, scoped policy, limits, privacy/egress and verifiable outcomes.

#### Scope-in

- canonical T248 pure contracts/types;
- one deterministic fake structured-data provider;
- SQL/parser/risk adapter boundary;
- exact data-source/schema/target identity;
- read/write/DDL/transaction consequence classification;
- scoped policy evaluation;
- row/byte/time limits;
- privacy/egress binding for reads;
- Effect mapping for writes;
- transaction outcome evidence including partial/mixed/unknown;
- provider-neutral receipts;
- source/provider removal behavior.

#### Scope-out for first package

- copying DBX's complete database-driver catalog;
- generic distributed transactions;
- a new SQL editor product;
- direct database credential exposure to models;
- automatically generated production mutations;
- cross-provider transaction retries;
- replacing T199 Effect semantics.

#### Donor admission order

1. implement provider-neutral contracts and deterministic fake first;
2. qualify DBX pure SQL risk/policy components against the contract;
3. prefer selective Rust copy/port where it materially reduces duplicate implementation risk;
4. add one real database provider only after exact dependency/driver qualification;
5. expand database-engine coverage only from measured user/product need.

#### Required qualification fixtures

```text
parse_known_read
parse_known_write
parse_ddl
parse_transaction_control
parse_unknown_fail_conservative
unbounded_update_escalates
unbounded_delete_escalates
cross_database_scope_denied
stale_schema_revalidation
read_limit_enforced
sensitive_read_egress_denied
write_requires_effect_gate
mixed_transaction_no_blind_retry
unknown_transaction_blocks_dependents
provider_removed_receipt_still_readable
```

### Package B — Work Graph and Liveness

**Primary task:** T249  
**Purpose:** one durable interpretation of work structure, blockers, ownership/claims, waiting/review, liveness and handoff.

#### Scope-in

- `WorkRelation`;
- `WorkClaimReceipt`;
- `WaitDescriptor`;
- `GoalRef` / `GoalPath`;
- `WorkLivenessProjection`;
- structural-vs-blocker relationship validation;
- atomic current-claim semantics;
- stale claim reconciliation;
- routed waiting/review/remediation;
- pre-dispatch configuration/readiness blockers;
- task-level budget/resource liveness gates;
- handoff preserving `UNKNOWN_OUTCOME`;
- projections consumable by Desktop/CLI/TUI/Mobile without surface-local task truth.

#### Scope-out for first package

- Paperclip company/org database;
- enterprise tenancy/SSO/RBAC beyond existing T243 baseline;
- organization billing;
- agent employment metaphors as security identities;
- a second scheduler;
- a second Task ledger;
- implicit authority from goals/roles.

#### Required qualification fixtures

```text
two_workers_one_claim
successor_claim_not_cleared_by_stale_finalizer
structural_parent_not_blocker
cancelled_blocker_not_satisfied
blocker_resolution_wakes_once
prose_only_blocked_rejected_or_attention
review_wait_not_execution_active
missing_secret_prevents_dispatch
missing_workspace_prevents_dispatch
budget_stop_no_policy_widening
claim_handoff_preserves_unknown_effect
crash_restart_current_claim_recovered
surface_projection_no_local_authority
```

### Package C — Workflow DAG Runtime

**Primary task:** T250  
**Depends on:** canonical T215 definitions + canonical T249 liveness/dependency semantics  
**Purpose:** safely bind admitted `WorkflowIR` to actual trigger-driven execution without inventing another workflow language.

#### Scope-in

- `WorkflowRunPlan`;
- `WorkflowNodeBinding`;
- `TriggerBinding`;
- `NodeAttempt`;
- `WorkflowRunCheckpoint`;
- `WorkflowImportChecklist`;
- `CapabilityReadinessSnapshot` integration;
- bounded DAG validation;
- node input lineage;
- explicit output nodes;
- manual/schedule/webhook-event trigger adapters using canonical InputEnvelope;
- overlap/concurrency/catch-up policy;
- node-level timeout/cancellation/reconciliation;
- protected-node readiness revalidation;
- pause/resume/checkpoint;
- safe workflow export/import/rebinding;
- explicit degraded/unavailable/known-absent capability behavior.

#### Scope-out for first package

- a second workflow DSL;
- model-generated graph auto-activation;
- arbitrary dynamic code nodes;
- capturing capability/approval/secret material inside workflow files;
- cloud-only scheduler/control plane;
- implicit retry of ambiguous Effects;
- silent provider/account substitution.

#### Required qualification fixtures

```text
duplicate_node_rejected
unknown_dependency_rejected
cycle_rejected
undeclared_input_lineage_rejected
unsupported_version_rejected
node_limit_enforced
approval_only_tightens
webhook_ssrf_denied
duplicate_trigger_deduped
clock_jump_catchup_deterministic
overlap_policy_deterministic
readiness_stale_before_dispatch_denied
pause_resume_no_completed_node_replay
unknown_effect_blocks_downstream
export_has_no_secret_or_lease
import_missing_binding_checklist
qualified_rebind_only
known_absent_is_not_success
optional_provider_removed_graceful
```

## 4. Shared contract ownership freeze required before Packages A–C

T198 must name one owner, serialization/version source and migration authority for at least:

### Structured data

```text
DataSourceBinding
SchemaSnapshot
DataOperationPlan
DataScope
DataRiskAssessment
DataExecutionReceipt
DataTransactionReceipt
```

### Work/liveness

```text
WorkRelation
WorkClaimReceipt
WaitDescriptor
GoalRef
GoalPath
WorkLivenessProjection
```

### Workflow runtime

```text
WorkflowRunPlan
WorkflowNodeBinding
TriggerBinding
NodeAttempt
WorkflowRunCheckpoint
WorkflowImportChecklist
CapabilityReadinessSnapshot
```

Likely ownership should minimize privilege:

- pure immutable protocol/value types in an unprivileged shared/core package;
- canonical durable state in the existing canonical ledger/task/state owner;
- authority/effect decisions exclusively in the existing protected kernel/Effect Gate;
- UI/provider adapters consume projections and may not own protected state.

The exact crate/package names are left to the future owning Spec after live repository verification; this planning file does not invent package names that may become stale.

## 5. Existing task refinements that must be folded into owning specs

### T183 / T214 / T247 — durable client/ACP interactions

The owning work/client package must include:

- pending interaction persistence before presentation;
- exact request/revision/session correlation;
- offered-option/schema validation;
- durable settlement before success publication;
- recovery behavior for crash between settlement and publication;
- no provider-replacement inheritance of unresolved approvals;
- bounded payload/plan/question sizes;
- lost transport ACK not treated as exactly-once provider application.

Paperclip is the main current donor/reference for these fixtures.

### T206 / T244 — live readiness

The owning capability/health package must include:

- readiness derived from the same observations/config/provider state that actually gate execution;
- freshness and source evidence;
- `AVAILABLE`, `DEGRADED`, `UNAVAILABLE`, `KNOWN_ABSENT`, `UNKNOWN`;
- user/operator remediation;
- no hand-maintained feature claim overriding current observed state;
- no readiness state authorizing an Effect.

Synaplan is the main current donor/reference for this refinement.

### T199 — transaction safety

The owning structured-data package must map every external mutation into T199/Effect Gate semantics. Database transaction state is target/provider evidence; it never creates a second generic Effect FSM.

## 6. Source Foundry component shortlist

The future owning specs should evaluate these exact paths first rather than wholesale repository copying.

### DBX shortlist

```text
crates/dbx-core/src/ai/mcp_policy.rs
crates/dbx-sql-core/src/sql_risk.rs
crates/dbx-core/assets/database-drivers.manifest.json
apps/desktop/src/lib/database/databaseDriverManifest.ts
apps/desktop/src/lib/database/databaseCapabilitySets.ts
crates/dbx-core/src/query/two_phase_commit.rs   # reference/test by default
```

### Paperclip shortlist

```text
doc/execution-semantics.md
server/src/services/execution-control-reconciliation.ts
packages/paperclip-runner/                     # only exact bounded rich-ACP components selected by owning spec
```

The current rich-ACP commit/report is architecture/test input. Provider-specific Cursor/Copilot/Pi components remain separately qualified and are not admitted from this plan.

### Synaplan shortlist

```text
backend/src/Service/SavedTask/Graph/SavedTaskGraphValidator.php
backend/src/Service/SavedTask/Graph/SavedTaskPlanFactory.php
backend/src/Service/SavedTask/Graph/SavedTaskGraphPortability.php
backend/src/Service/SelfAware/PlatformCapabilityInventory.php
```

## 7. Threat / failure checklist before implementation closeout

### T248

- SQL injection through generated query text;
- parser/dialect mismatch;
- query classified read but provider executes mutation;
- cross-database/schema escape;
- stale schema/renamed target;
- overly broad row disclosure;
- sensitive data sent to remote model/provider;
- credential/account confusion;
- long-running query resource exhaustion;
- lock/transaction deadlock;
- ambiguous commit after network/process loss;
- partial/mixed transaction outcome;
- verification reads wrong account/database.

### T249

- two active workers believe they own the same task;
- stale worker clears successor claim;
- parent relation incorrectly acts as blocker;
- terminal-cancelled blocker incorrectly unblocks dependent;
- blocked state with no owner/recheck path;
- reviewer/approver process dies;
- missing configuration repeatedly dispatches doomed runs;
- budget loop repeatedly wakes a blocked agent;
- handoff loses unresolved Effect state;
- goal/role text is interpreted as authority;
- task projection diverges across surfaces.

### T250

- graph cycle/expansion DoS;
- duplicate node identity;
- hidden input dependency;
- stale capability readiness;
- trigger replay/duplicate webhook;
- schedule catch-up storm;
- overlapping workflow executions race on Effects;
- provider/account silently substituted;
- node retry repeats ambiguous Effect;
- pause/resume replays completed nodes;
- import carries secrets/approval/leases;
- imported logical resource binds to wrong account;
- optional module missing but UI/runtime claims success.

## 8. No-gap matrix

| Concern | Owner after this review | Gap status |
| --- | --- | --- |
| typed decisions | T203/T204/T229/T235/T246 | covered |
| capability catalog/relay | T206/T207/T210 | covered |
| structured database/query semantics | **T248** | closed in plan |
| Task/Session/Run/Worker identities | T167 | covered |
| structure/dependency/claim/liveness | **T249** | closed in plan |
| workload reconciliation/fencing | T205 | covered |
| inbound redelivery | T247 | covered |
| durable ACP questions/permissions | T183/T214/T247 refinement | closed in plan |
| WorkflowIR/compiler/replay | T215 | covered |
| workflow DAG runtime/triggers/checkpoints | **T250** | closed in plan |
| live capability readiness | T206/T244 refinement consumed by T250 | closed in plan |
| workflow portability/rebinding | T250 | closed in plan |
| secret lifecycle | T238 | covered |
| configuration revision | T245 | covered |
| connector auth/webhooks/sync | T241 | covered |
| Effect ambiguity | T199 + existing Effect FSM | covered; donor 2PC rejected as replacement |
| multi-tenant enterprise | T243 future-mode gate | explicitly not baseline |
| lifecycle/update/backup/decommission | T237–T245 | covered |
| verification/evidence | T149/T150/T213/T216 | covered |

No additional material planning task was identified after this allocation.

## 9. Definition of implementation-ready for this extension

The extension is considered planning-content ready when all are true on one exact PR head:

```text
T248_CONTRACT_COMPLETE=YES
T249_CONTRACT_COMPLETE=YES
T250_CONTRACT_COMPLETE=YES
SOURCE_PINS_RECORDED=YES
SOURCE_PERMISSION_RECORDED=YES
LICENSE_POSTURE_RECORDED=YES
REUSE_STRATEGY_RECORDED=YES
SHARED_OWNERSHIP_REQUIREMENTS_RECORDED=YES
FAILURE_FIXTURES_DEFINED=YES
ACCEPTANCE_FIXTURES_DEFINED=YES
PACKAGE_ORDER_DEFINED=YES
NON_ADOPTIONS_RECORDED=YES
NO_KNOWN_MATERIAL_GAP_AFTER_SELF_REVIEW=YES
```

Exact-head CI and independent review are separate repository evidence gates and must still complete before the planning PR can be called qualified/canonical.

## 10. Current disposition

```text
PLAN_CONTENT_READY_FOR_FUTURE_OWNING_SPECS=YES
IMPLEMENTATION_AUTHORIZED_NOW=NO
REASON_SPEC_006_NOT_CLOSED_CANONICAL=YES
REASON_PLANNING_PR_28_NOT_CANONICAL=YES

ACTIVE_SPEC_006_PR_24_WIDENED=NO
CURRENT_POINTER_CHANGED=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_SOURCE_COMPONENT_ADMITTED=NO
NEW_PRODUCT_IMPLEMENTATION_STARTED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
WAIVER_TAKEN=NO
```
