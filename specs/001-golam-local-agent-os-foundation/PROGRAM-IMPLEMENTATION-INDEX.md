# Golam Program Implementation Index

**Latest planning overlay date:** 2026-09-30
**Status:** `PLAN_CONTENT_IMPLEMENTATION_READY / IMPLEMENTATION_NOT_YET_AUTHORIZED`
**Planning PR:** #28
**Task range:** T110–T252

This file is the single implementation entry index for the Golam program planning overlay. It does not replace the Constitution, `AGENTS.md`, `specs/CURRENT.md`, active Spec Kit authority, or live repository governance.

## 1. Read this first

At implementation time, always fetch live:

1. `AGENTS.md`;
2. Constitution / canonical governance documents;
3. `specs/CURRENT.md`;
4. current `main`;
5. current active PR/spec state;
6. this index;
7. `PROGRAM-IMPLEMENTATION-READINESS-CLOSEOUT-2026-09-30.md`;
8. the task and readiness packets named below.

If live canonical authority conflicts with this planning overlay, live canonical authority wins and the plan must be reconciled before implementation.

## 2. Planning task chain

Read task files in this order:

```text
program-superiority-tasks.md
  T110–T165

program-superiority-gap-closure-tasks.md
  T166–T184

program-direction-tasks-2026-09-13.md
  T185–T202

program-direction-tasks-2026-09-22.md
  T203–T247

program-direction-tasks-2026-09-29.md
  T248–T250

program-direction-tasks-2026-09-30.md
  T251–T252
```

Later files extend/refine earlier contracts; they do not silently revoke earlier invariants unless they explicitly say so.

## 3. Final implementation-readiness precedence

Read readiness material in this order:

```text
program-implementation-readiness-2026-09-29.md
  Packages A–C / T248–T250

program-implementation-readiness-2026-09-30.md
  Packages D–E / T251–T252
  Doop-derived Experience/Artifact refinements

PROGRAM-IMPLEMENTATION-READINESS-CLOSEOUT-2026-09-30.md
  final no-gap audit
  resolves the previously listed T251/T252 security/failure questions
  universal implementation-package Definition of Done
  cross-package integration journeys
```

The closeout is the newest planning overlay. Where an earlier readiness packet phrases an item as an open question, the closeout's frozen decision controls the planning handoff.

## 4. Current newest tasks

```text
T248 Structured Data Source / Query / Mutation Safety
T249 Work Graph / Ownership / Dependency / Liveness
T250 Workflow DAG Runtime / Trigger / Readiness / Portability
T251 Durable Supervision Event / Steering Inbox / Wake
T252 Verified Change Qualification / Publication / Repair Gate
```

No T253 is currently justified by a measured architecture or implementation-handoff gap.

## 5. Major architecture/source review packets

### Core direction

```text
program-direction-review-2026-09-13.md
program-direction-review-addendum-2026-09-22.md
program-major-review-2026-09-23.md
program-direction-review-addendum-2026-09-29.md
program-direction-review-addendum-2026-09-30.md
```

### Current source packets

```text
competitive-source-register-supplement-2026-09-13.md
competitive-source-register-supplement-2026-09-22.md
source-adoption-laya-coreml-jev-search-unreal-classifier-2026-09-23.md
source-adoption-dbx-paperclip-synaplan-2026-09-29.md
source-adoption-firstmate-no-mistakes-doop-2026-09-30.md
```

### Permission records

```text
source-permission-attestation.md
source-permission-attestation-supplement-2026-09-29.md
source-permission-attestation-supplement-2026-09-30.md
```

Permission records are Source Foundry inputs only. They never replace exact component/dependency/rights/NOTICE/security qualification.

## 6. High-level architecture spine

Every implementation package must consume the same protected spine:

```text
Input / User / Channel
-> InputEnvelope / authenticated surface binding
-> Task / Work Graph / Work Claim
-> optional Formation
-> durable Steering / Supervision
-> ExecutionEnvelope + generation fencing
-> Capability / Provider selection
-> OperationProposal
-> canonical Authority / Effect Gate
-> dispatch
-> Evidence / Verification
-> Experience projection
```

Optional domain/runtime layers such as voice, workflow DAGs, structured data, live artifacts or software-change publication fit into this spine; they do not create parallel authority.

## 7. Canonical cross-cutting owner rule

Before an implementation package starts, T198 must assign exactly one canonical owner/version/migration authority for every cross-cutting type consumed by that package.

Latest additions requiring explicit ownership include:

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

SteeringEnvelope
SteeringReceipt
SupervisionEvent
WakeQueueEntry
WakeReceipt
SupervisorAttentionProjection
SupervisionRouteBinding

ChangeSetCandidate
ChangeIntentRecord
QualificationPlan
QualificationStepReceipt
Finding
FindingDecision
RepairRound
ReviewedHeadBinding
PublicationPlan
PublicationReceipt
RecoveryAnchor
ChangeDeliveryOutcome
```

No provider, adapter, UI, donor package or runtime may define a competing protected truth for these concepts.

## 8. Source reuse discipline

For every donor component:

```text
REFERENCE
-> VERIFIED exact source revision
-> PERMISSION_RECORDED
-> exact component/path/blob selected
-> dependency closure known
-> rights / NOTICE known
-> authority ceiling known
-> TCB delta measured where relevant
-> tests / parity criteria defined
-> TECHNICALLY_QUALIFIED
-> ADMITTED only by existing Source Foundry authority
```

Source permission does not imply admission.

## 9. Current source-family placement

```text
Decision providers / semantic decisions:
  OpenJev
  convaiinnovations/laya
  laya-coreml backend
  SemIf / Decider / Nimble references

Batch/context narrowing:
  classifier.dev
  jev_search

Input/tool translation:
  unreal-agent

Work / organization / workflow:
  Paperclip
  Synaplan

Structured data:
  DBX

Agent supervision / steering:
  Firstmate

Change qualification / publication:
  no-mistakes

Live artifact collaboration / Experience:
  Doop
  OpenMuse

External Agent-OS completeness / channels / skills:
  AutoClaw / Z.AI
  OpenClaw / GLM-skills references
```

This placement is not authority. Exact component admission remains separate.

## 10. Implementation package order

Current recommended order after live authority is granted:

```text
P0_CANONICAL_PREREQUISITES
  active Spec 006 canonical closeout
  live successor authority
  T198 shared ownership freeze
  required existing Authority / Effect / Verification / Egress / Source Foundry prerequisites

P1_PARALLEL_FOUNDATIONS
  Package A / T248 Structured Data
  Package B / T249 Work Graph / Liveness
  Package D / T251 Durable Supervision
  Package E / T252 Verified Change Delivery

P2_AFTER_T215_AND_T249
  Package C / T250 Workflow DAG Runtime

P2_PLUS_EXPERIENCE_WHEN_OWNING_PACKAGE_EXISTS
  Doop/OpenMuse-inspired live artifact collaboration projections
```

Package ordering may be tightened by live dependencies; it may not be loosened by this index.

## 11. Universal package Definition of Done

Every future owning package for T185–T252 must instantiate the universal checklist in `PROGRAM-IMPLEMENTATION-READINESS-CLOSEOUT-2026-09-30.md` in addition to task-specific fixtures.

The mandatory categories are:

```text
scope / canonical ownership
serialization / compatibility / migration
retention / compaction / privacy
security / authority / negative testing
concurrency / crash / fault recovery
performance / resource envelope
platform / environment support
observability / operator recovery
supply chain / SBOM / build provenance
update / rollback / configuration revision
verification / exact-head release evidence
documentation / user and operator lifecycle
```

A package is not implementation-complete merely because its happy-path tests pass.

## 12. Implementation entry checklist

Before implementation of any T185–T252 task:

```text
[ ] live main fetched
[ ] live AGENTS / Constitution / CURRENT fetched
[ ] planning PR canonical and qualified under repository policy
[ ] active predecessor spec canonically closed when required
[ ] live successor owning spec authorized
[ ] exact task scope-in/out frozen
[ ] T198 shared owner/version/migration map complete
[ ] exact threat/failure model frozen
[ ] task-specific acceptance fixtures frozen
[ ] universal package Definition of Done instantiated
[ ] donor Source Foundry records complete for copied code
[ ] no unadmitted runtime/model/hosted service required
[ ] no material exact-head review finding unresolved
[ ] supported platform/runtime matrix frozen
[ ] retention/compaction policy bound to canonical privacy/data classes
[ ] performance/resource envelope frozen
[ ] migration/update/rollback behavior frozen
[ ] operator runbook/diagnostics/removal path defined
[ ] supply-chain/dependency/build provenance requirements frozen
[ ] no hidden Spec 006 scope widening
[ ] normal repository merge/history policy preserved
```

If a box is false, planning may continue but implementation remains blocked.

## 13. Stable-release cross-package journeys

Before the first stable program release combining the new packages, the closeout requires integrated evidence for at least:

```text
input redelivery
-> one canonical task
-> work claim
-> worker generation
-> durable steering
-> protected Effect
-> crash/restart
-> verification
-> exact-head change qualification/publication where relevant
```

```text
workflow duplicate/replayed trigger
-> one admitted occurrence
-> structured-data node
-> ambiguous external outcome
-> downstream block
-> supervisor attention
-> reconciliation
-> resume without Effect replay
```

```text
provider/account/config generation change
-> stale prepared work rejected
-> no stale secret release
-> refreshed capability readiness
-> queued work revalidated
-> explicit safe continuation/remediation
```

Individual green package tests alone are not sufficient for stable release.

## 14. Current live governance observation

At this planning audit:

```text
CANONICAL_MAIN=13a379ac478a3abaff7ed1da3db14ff9c1ac2188
ACTIVE_SPEC_006_PR=24
ACTIVE_SPEC_006_STATE=OPEN_DRAFT
SPEC_006_CLOSED_CANONICAL=NO
PLANNING_PR_28=OPEN
```

These are observations, not timeless constants. Re-fetch them before implementation.

## 15. Current disposition

```text
PROGRAM_PLAN_INDEX_PRESENT=YES
PROGRAM_IMPLEMENTATION_READINESS_CLOSEOUT_PRESENT=YES
TASK_RANGE=T110-T252
LATEST_TASK_EXTENSION=2026-09-30
LATEST_IMPLEMENTATION_READINESS=2026-09-30

NO_UNRESOLVED_PLANNING_DESIGN_QUESTION=YES
UNIVERSAL_PACKAGE_DEFINITION_OF_DONE_PRESENT=YES
CROSS_PACKAGE_INTEGRATION_JOURNEYS_PRESENT=YES
PLAN_CONTENT_IMPLEMENTATION_READY=YES

IMPLEMENTATION_AUTHORIZED=NO
T253_REQUIRED=NO
ACTIVE_SPEC_006_PR_24_WIDENED=NO
CURRENT_POINTER_CHANGED=NO
CONSTITUTION_CHANGED=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_SOURCE_COMPONENT_ADMITTED=NO
NEW_MODEL_ADMITTED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
WAIVER_TAKEN=NO
```