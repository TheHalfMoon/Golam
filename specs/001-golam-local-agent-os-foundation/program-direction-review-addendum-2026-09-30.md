# Golam Program Direction Review Addendum — 2026-09-30

**Status:** PLANNING REVIEW / NO IMPLEMENTATION AUTHORITY

## 1. Source-set conclusion

The 2026-09-30 exact-pinned review of Firstmate, no-mistakes and Doop found two material planning gaps and one valuable Experience refinement.

### Gap A — durable internal supervision delivery

Existing Golam planning had external `InputEnvelope` redelivery protection (T247), worker generation fencing (T205), team formation (T230) and work ownership/liveness (T249), but no first-class durable contract for the internal control/event channel between supervisor and worker.

Firstmate demonstrated the value of:

- persistent task steering inboxes;
- persistent wake queues;
- persist-before-doorbell delivery;
- event-driven supervision with no mandatory model polling;
- generation-bound replay/recovery;
- explicit wait/decision/wedge/dead/unknown classes;
- bounded escalation and remote-route honesty.

This becomes T251.

### Gap B — exact-head software change publication

Golam had general verification and evidence contracts but no one bounded contract covering the whole software-change publication lifecycle:

```text
intent
-> isolated candidate
-> exact-head qualification
-> findings / decisions
-> repair lineage
-> remote-head reconciliation
-> publication / PR
-> CI
-> CI repair / revalidation
-> final exact-head evidence
```

no-mistakes provides strong donor/reference patterns for this seam. This becomes T252.

### Doop refinement — live human/agent artifact workspace

Doop does not justify another authority/control-plane task. Its strongest Golam value is Experience/Artifact behavior:

- live human/agent presence;
- actor working state;
- streaming artifact edits;
- exact element comments;
- activity feed;
- sandboxed previews;
- MCP-authenticated agent access under human access;
- distillation of examples/decisions into proposed durable style rules.

These strengthen T185/T193/T200/T214/T232/T243 only.

## 2. Architectural effect

The updated program chain is:

```text
External input
  -> T247 InputEnvelope
  -> T249 Work Graph / WorkClaim / Liveness
  -> T230 Formation / delegation
  -> T251 durable supervisor steering + wake
  -> T205 execution generation
  -> T199 Operation / Effect
  -> T149/T150/T216 evidence + verification
  -> T252 optional software-change qualification/publication
  -> T185/T193/T200 live Experience/artifact projection
```

No donor becomes canonical authority.

## 3. Source/legal posture

- Firstmate exact reviewed pin is MIT.
- no-mistakes exact reviewed pin is MIT.
- Doop exact reviewed pin is publicly AGPL-3.0.
- The founder separately asserts permission to reuse the supplied source set.
- Exact Source Foundry component records remain mandatory.
- For Doop, any reliance on direct permission rather than public AGPL terms must bind exact permission evidence/scope before copied component admission/distribution.

## 4. New hard invariants

```text
STEERING_MESSAGE != EFFECT_AUTHORIZATION
STEERING_PERSISTED != STEERING_CONSUMED
TRANSPORT_SEND_SUCCESS != WORKER_APPLIED_MESSAGE
WAKE_EVENT != TASK_AUTHORITY
REMOTE_ROUTE_FAILURE != LOCAL_FALLBACK_PERMISSION
MODEL_POLLING != SUPERVISION_REQUIREMENT

PIPELINE_GREEN != MERGE_AUTHORIZATION
QUALIFICATION_RECEIPT != DIFFERENT_HEAD_QUALIFICATION
CI_REPAIR != REVIEW_INHERITANCE
AUTO_FIX != USER_INTENT_MUTATION_AUTHORITY
RECOVERY_ANCHOR != PUBLICATION_PROOF
CHANGE_GATE != GOLAM_EFFECT_GATE

LIVE_PRESENCE != PRINCIPAL_AUTHORITY
ACTIVITY_FEED != CANONICAL_EVENT_LEDGER
SANDBOXED_IFRAME != TRUSTED_CONTENT
DISTILLED_STYLE_RULE != ACTIVE_POLICY
```

## 5. Task-graph result

```text
T251 Durable Supervision Event / Steering Inbox / Wake Contract
T252 Verified Change Qualification / Publication / Repair Gate Contract
T253_REQUIRED=NO
```

The repeated classifier.dev / jev_search / unreal-agent sources remain correctly placed in T246/T247 and do not justify duplicate tasks.

## 6. Implementation-readiness result

The companion `program-implementation-readiness-2026-09-30.md` freezes:

- scope-in/scope-out;
- canonical contracts;
- donor admission order;
- failure/security questions;
- acceptance fixtures;
- T198 shared-owner requirements;
- package order;
- donor removal behavior.

Therefore the planning content is implementation-ready once live governance authorizes bounded owning specs.

```text
TASK_GRAPH_EXTENDS_THROUGH_T252=YES
PLAN_CONTENT_IMPLEMENTATION_READY=YES
IMPLEMENTATION_AUTHORIZED=NO
NEW_PARALLEL_AUTHORITY_SYSTEM=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_SOURCE_COMPONENT_ADMITTED=NO
ACTIVE_SPEC_006_PR_24_WIDENED=NO
CURRENT_POINTER_CHANGED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
WAIVER_TAKEN=NO
```
