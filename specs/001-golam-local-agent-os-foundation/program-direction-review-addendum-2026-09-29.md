# Golam Program Direction Review Addendum — 2026-09-29

**Status:** PROGRAM REVIEW / PLANNING ONLY

## 1. Review question

Do DBX, Paperclip and Synaplan materially improve Golam, and if so, can their strongest patterns be adopted without creating duplicate authority, task, workflow or Effect systems?

## 2. Answer

Yes. The source review exposed exactly three material gaps:

```text
T248 Structured Data Source / Query / Mutation Safety
T249 Work Graph / Ownership / Dependency / Liveness
T250 Workflow DAG Runtime / Trigger / Readiness / Portability
```

The rest of their useful behavior maps into existing Golam owners and therefore does not justify more task IDs.

## 3. Source-to-architecture mapping

### DBX

Primary value:

- structured database capability metadata;
- AST-based SQL consequence/risk classification;
- monotonic scoped execution policy;
- stable connection/database identity;
- cross-database scope protection;
- conservative handling of dangerous/unbounded writes.

Placement:

```text
Capability Plane
-> Structured Data provider family
-> T248
-> canonical T199 Effect Gate for mutation
```

Explicit rejection:

```text
DBX_2PC != GOLAM_EFFECT_ENGINE
```

DBX two-phase-commit retries are not imported into generic Golam Effect semantics. Its mixed/unknown outcome vocabulary may inform bounded database transaction fixtures only.

### Paperclip

Primary value:

- explicit separation of work structure, dependency, ownership and execution;
- atomic current work checkout/claim patterns;
- execution-backed liveness;
- routable blockers/reviews;
- pre-dispatch configuration blockers;
- goal ancestry without authority;
- durable rich-ACP question/permission settlement;
- budget hard stops and recovery patterns.

Placement:

```text
Task / Work Plane
-> T167 canonical Task identity
-> T249 work graph + claim + liveness
-> T205 runtime reconciliation
-> existing Effect Gate
```

Explicit rejection:

```text
PAPERCLIP_COMPANY != GOLAM_TENANT
PAPERCLIP_TASK_DB != GOLAM_TASK_AUTHORITY
```

Golam remains personal/local single-owner by default. Paperclip's multi-organization control plane is not a baseline dependency.

### Synaplan

Primary value:

- bounded/versioned DAG validation;
- explicit input lineage and dependencies;
- trigger/runtime graph construction;
- safe approval tightening;
- SSRF-aware outbound workflow nodes;
- workflow export/import with secret stripping and deterministic unresolved-binding checklists;
- live capability/readiness inventory derived from actual gating state;
- explicit known-absent capabilities and remediation.

Placement:

```text
T215 WorkflowIR / SkillIR definition
-> T250 admitted runtime binding
-> T249 liveness/dependency semantics
-> T206/T244 readiness projection
-> T199 Effect Gate for consequential nodes
```

Explicit rejection:

```text
SYNAPLAN_GRAPH != NEW_GOLAM_WORKFLOW_LANGUAGE
SYNAPLAN_CAPABILITY_INVENTORY != GOLAM_AUTHORITY
```

## 4. Cross-source synthesis

The three sources reinforce one architecture rather than creating separate products:

```text
USER / TRIGGER
-> canonical InputEnvelope
-> TaskContract
-> Work Graph / Work Claim / Liveness                [T249]
-> WorkflowIR -> bounded WorkflowRunPlan             [T215 + T250]
-> live CapabilityReadiness / exact CapabilityOffer  [T206/T244]
-> Structured Data / other typed capability plan     [T248 or existing providers]
-> Authority + Effect Gate                           [canonical]
-> dispatch
-> target/provider receipts
-> EvidenceBundle / verification                     [canonical]
```

No donor owns the protected center.

## 5. Important new design decisions

### 5.1 Structured data is not generic text tooling

SQL/database operations get an explicit semantic contract because consequences depend on parsed operation type, database/schema/object scope, row/data exposure, transaction semantics and target identity.

A database connector can be offered through MCP or another provider, but protocol transport does not replace structured-data safety.

### 5.2 Work ownership is not task assignment

A task may have an intended assignee while no live actor currently owns the right to advance it. Conversely, a current work claim does not authorize protected external effects.

This prevents stale workers, UI task labels and assignment changes from becoming implicit execution authority.

### 5.3 Waiting must be routable

A task is not considered healthily blocked merely because a model wrote "waiting for X". It must carry a first-class blocker, pending reviewer/approver, monitored recheck or structured owner/action remediation.

### 5.4 Workflow definition and workflow runtime remain separate

T215 owns portable workflow meaning. T250 owns the runtime binding of one admitted revision to triggers, nodes, attempts, readiness and checkpoints.

This separation prevents runtime state, provider IDs or secrets from contaminating the portable workflow definition.

### 5.5 Readiness is observed state, not catalog marketing

A capability may exist in the catalog but be unavailable because of missing credentials, model artifacts, OS permissions, provider outage or optional-module absence.

`CapabilityReadinessSnapshot` reports current observed usability and remediation; it does not grant authority.

## 6. Interaction with previous tasks

No existing owner is replaced:

```text
T167  canonical Task/Session/Run/Worker identity       retained
T183  surface parity                                  refined
T198  shared contract ownership                       extended
T199  Operation / Effect semantics                    retained
T205  runtime reconciliation/fencing                  retained
T206  capability catalog                              refined with readiness
T207  credential-brokered relay                       retained
T214  disclosure/review checkpoint                    refined
T215  WorkflowIR / SkillIR                            retained as definition owner
T216  multidimensional verification                   retained
T238  secret lifecycle                                retained
T241  connector/webhook lifecycle                     retained
T243  personal/enterprise boundary                    retained
T244  diagnostics/health                              refined with readiness
T245  configuration revision                          retained
T247  inbound redelivery/tool-translation commit      refined with durable interactions
```

## 7. Implementation-readiness conclusion

The implementation package is now decomposed as:

```text
Shared owner freeze / T198
   |
   +--> Package A: T248 Structured Data
   |
   +--> Package B: T249 Work Graph + Liveness
                |
                +--> Package C: T250 Workflow DAG Runtime
                     (also requires canonical T215)
```

The detailed future implementation entry gates and fixture lists are in:

- `program-implementation-readiness-2026-09-29.md`.

This is sufficient to write bounded implementation specs without another architectural discovery pass for these source families, unless live repository/source state changes or an owning-spec review exposes a new material issue.

## 8. Remaining program gates are governance, not undefined architecture

As of the live review:

```text
CANONICAL_MAIN=13a379ac478a3abaff7ed1da3db14ff9c1ac2188
PR_28_PLANNING=OPEN
PR_24_SPEC_006_IMPLEMENTATION=OPEN_DRAFT
SPEC_006_CLOSED_CANONICAL=NO
```

Therefore:

```text
PLAN_CONTENT_IMPLEMENTATION_READY=YES
IMPLEMENTATION_AUTHORIZED_NOW=NO
```

The next required work is exact-head planning qualification and canonical closeout, not further speculative task expansion.

## 9. Review disposition

```text
DBX_MATERIAL_GAP_COUNT=1
PAPERCLIP_MATERIAL_GAP_COUNT=1
SYNAPLAN_MATERIAL_GAP_COUNT=1
TOTAL_NEW_TASKS=3
TASK_GRAPH_EXTENDS_THROUGH_T250=YES

NEW_PARALLEL_AUTHORITY_SYSTEM=NO
NEW_PARALLEL_TASK_LEDGER=NO
NEW_PARALLEL_EFFECT_LEDGER=NO
NEW_PARALLEL_WORKFLOW_LANGUAGE=NO
NEW_BASELINE_TENANCY_SYSTEM=NO

NO_ADDITIONAL_MATERIAL_GAP_IDENTIFIED_AFTER_ALLOCATION=YES
ACTIVE_SPEC_006_PR_24_WIDENED=NO
CURRENT_POINTER_CHANGED=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_PRODUCT_IMPLEMENTATION_STARTED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
WAIVER_TAKEN=NO
```
