# Golam Program Implementation Index

**Latest planning overlay date:** 2026-09-30
**Status:** `CONTENT_IMPLEMENTATION_READY / IMPLEMENTATION_NOT_YET_AUTHORIZED`
**Planning PR:** #28
**Task range:** T110–T252

This file is the implementation entry index for the Golam program planning overlay. It does not replace the Constitution, `AGENTS.md`, `specs/CURRENT.md`, active Spec Kit authority, or live repository governance.

## 1. Read this first

At implementation time, always fetch live:

1. `AGENTS.md`;
2. Constitution / canonical governance documents;
3. `specs/CURRENT.md`;
4. current `main`;
5. current active PR/spec state;
6. this index and the exact planning files below.

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

## 3. Current newest tasks

```text
T248 Structured Data Source / Query / Mutation Safety
T249 Work Graph / Ownership / Dependency / Liveness
T250 Workflow DAG Runtime / Trigger / Readiness / Portability
T251 Durable Supervision Event / Steering Inbox / Wake
T252 Verified Change Qualification / Publication / Repair Gate
```

No T253 is currently justified by measured gaps.

## 4. Current implementation-readiness packets

```text
program-implementation-readiness-2026-09-29.md
  Packages A–C
  T248–T250

program-implementation-readiness-2026-09-30.md
  Packages D–E
  T251–T252
  Doop-derived Experience/Artifact refinements
```

These packets define scope-in/out, donor admission order, required fixtures, shared ownership and package sequencing. They are the primary handoff for a future implementation agent after live authority is granted.

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

The latest additions requiring explicit ownership include:

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
  T248 Structured Data
  T249 Work Graph / Liveness
  T251 Durable Supervision
  T252 Verified Change Delivery

P2_AFTER_T215_AND_T249
  T250 Workflow DAG Runtime

P2_PLUS_EXPERIENCE_WHEN_OWNING_PACKAGE_EXISTS
  Doop/OpenMuse-inspired live artifact collaboration projections
```

Package ordering may be tightened by live dependencies; it may not be loosened by this index.

## 11. Implementation entry checklist

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
[ ] acceptance fixtures frozen
[ ] donor Source Foundry records complete for copied code
[ ] no unadmitted runtime/model/hosted service required
[ ] no material exact-head review finding unresolved
[ ] no hidden Spec 006 scope widening
[ ] normal repository merge/history policy preserved
```

If a box is false, planning may continue but implementation remains blocked.

## 12. Current disposition

```text
PROGRAM_PLAN_INDEX_PRESENT=YES
TASK_RANGE=T110-T252
LATEST_TASK_EXTENSION=2026-09-30
LATEST_IMPLEMENTATION_READINESS=2026-09-30
PLAN_CONTENT_IMPLEMENTATION_READY=YES
IMPLEMENTATION_AUTHORIZED=NO
T253_REQUIRED=NO
ACTIVE_SPEC_006_PR_24_WIDENED=NO
CURRENT_POINTER_CHANGED=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_SOURCE_COMPONENT_ADMITTED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
```
