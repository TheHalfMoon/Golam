# Golam Canonical Direction Task Extension — 2026-09-30

**Authority:** PROGRAM ORCHESTRATION / IMPLEMENTATION-READINESS PLANNING ONLY
**Extends:** `program-direction-tasks-2026-09-29.md` through T250
**Source review:** `source-adoption-firstmate-no-mistakes-doop-2026-09-30.md`
**Permission record:** `source-permission-attestation-supplement-2026-09-30.md`

This extension adds only measured gaps exposed by the exact-pinned Firstmate / no-mistakes / Doop review. It does not widen active Spec 006, alter `specs/CURRENT.md`, amend the Constitution, admit source/dependencies/runtimes, or grant implementation authority.

## Phase AC — Durable supervision and verified change delivery

- [ ] **T251 — Durable Supervision Event / Steering Inbox / Wake Contract.** Extend T163/T167/T183/T205/T214/T221/T230/T247/T249 so supervisor-to-worker steering and worker/runtime-to-supervisor wake delivery are explicit durable contracts rather than model-polling conventions or backend-specific terminal text. Define canonical `SteeringEnvelope`, `SteeringReceipt`, `SupervisionEvent`, `WakeQueueEntry`, `WakeReceipt`, `SupervisorAttentionProjection` and `SupervisionRouteBinding`. Bind task/work claim, actor/run/runtime generation, source principal/supervisor, target worker/formation, message/event class, payload digest or canonical content reference, creation/expiry, ordering/dedup identity, privacy/taint, route/backend, delivery attempt, consumption state and recovery lineage. A supervisor may use deterministic watchers over canonical/observed worker state to wake model reasoning only when action is required; continuous model polling is not required for liveness. Steering content is durably persisted before an ephemeral terminal/IPC/remote “doorbell” is sent, so a short typed notification carries only a pointer/identity and cannot lose the durable message. Delivery and consumption are distinct: successful terminal keystroke/socket send is not proof the intended worker incorporated the steer. On restart/reconnect/replacement, only unconsumed steering targeted at a still-valid task/claim/generation may be replayed; stale-generation material is quarantined or re-routed through canonical reconciliation rather than injected into the successor blindly. Wake events distinguish actionable attention, declared wait, supervisor-owned decision, owner/user-held decision, worker-dead/missing, worker-stale/wedged, active-but-quiet, route unavailable and unknown. Every wait/attention event that defers escalation carries `{owner, condition/action, recheck/wake policy}` and freshness/expiry. Remote worker/supervisor route failure never silently becomes local execution. Use `kunchenguid/firstmate@23e5584714e6765cc223a1740d385d0e85f8ad8e` as the primary behavior/source donor for durable task inboxes, durable wake queues, event-driven zero-token supervision, generation-bound recovery, bounded escalation, restart reconciliation and isolated remote subordinate homes. Firstmate home/pane/status files remain donor implementation details, not Golam canonical truth.

- [ ] **T252 — Verified Change Qualification / Publication / Repair Gate Contract.** Extend T112/T143–T150/T167/T183/T205/T214/T216/T240/T245/T249/T250 for software/repository change delivery when Golam or a worker modifies code/configuration/content intended for publication. Define canonical `ChangeSetCandidate`, `ChangeIntentRecord`, `QualificationPlan`, `QualificationStepReceipt`, `Finding`, `FindingDecision`, `RepairRound`, `ReviewedHeadBinding`, `PublicationPlan`, `PublicationReceipt`, `RecoveryAnchor` and `ChangeDeliveryOutcome`. Bind exact repository/branch/base/head/tree identity, intended target, submitted intent, worktree/source revision, required qualification steps, exact tool/agent versions, finding severity/action, user/reviewer decisions, repair predecessor/successor heads, CI/check identities, remote target head/lease and final publication/PR identity. The minimum guarded pipeline is policy-owned and monotonic: configured repositories may add/tighten steps, but a lower-assurance local config, worker or donor cannot silently delete/reorder required review/test/document/lint/security/CI gates. Every qualification/attestation binds the exact head it describes; any repair that changes code creates a new head and either passes the owning revalidation policy or remains uncertified. “Safe auto-fix” is limited to bounded mechanical classes with explicit attempt budgets and cannot mutate user intent, authority/privacy/security policy or waive findings. User/reviewer findings are durable and decisions such as approve/fix/skip bind the exact finding + change revision. Publication checks the live remote target immediately before update and refuses to discard unincorporated commits; force-with-lease or equivalent concurrency control is necessary but not sufficient for content qualification. Preserve unpublished/at-risk heads before cleanup through an explicit recovery anchor; recovery preservation proves retention only, not that the content is reviewed or belongs in the published tree. CI repair uses the same guarded publication path and does not inherit review from a different head unless the owning policy explicitly proves an allowed equivalence/revalidation. Use `kunchenguid/no-mistakes@a1c06cdaefcefa7cbc6902ac507f68ce9eed14ad` as the primary source/donor for explicit local gate routing, disposable worktrees, fixed monotonic validation pipelines, finding/repair loops, exact-head attestations, remote data-loss guards, recovery anchors and crash-recoverable publication runs. The no-mistakes gate/daemon/database never becomes Golam repository authority, general Effect authority or merge authority.

## T251 canonical contract minimums

### `SteeringEnvelope`

```text
steering_id
task_id / task_revision
work_claim_ref
formation / worker / run target
source supervisor/principal
source runtime generation
target runtime generation if bound
message_class
content_ref_or_bounded_payload
payload_digest
taint / data_class
created_at / expires_at
ordering / dedup identity
route preference / constraints
```

### `SteeringReceipt`

```text
steering_id
persisted_at
route_binding
attempt_id
sent_at
transport_result
consumed_at / consumer_generation
settlement
recovery_or_quarantine_reason
```

### `SupervisionEvent`

At minimum distinguish:

```text
ACTIONABLE_WORKER_RESULT
ACTIONABLE_FAILURE
ACTIONABLE_USER_OR_REVIEW_DECISION
DECLARED_WAIT
SUPERVISOR_OWNED_WAIT
USER_HELD_WAIT
WORKER_ACTIVE
WORKER_STALE_SUSPECTED
WORKER_DEAD_OR_MISSING
ROUTE_UNAVAILABLE
UNKNOWN
```

### T251 hard invariants

```text
STEERING_MESSAGE != EFFECT_AUTHORIZATION
STEERING_PERSISTED != STEERING_CONSUMED
TRANSPORT_SEND_SUCCESS != WORKER_APPLIED_MESSAGE
TERMINAL_PANE != WORKER_IDENTITY
PANE_TEXT != TASK_TRUTH
WAKE_EVENT != TASK_AUTHORITY
MODEL_POLLING != SUPERVISION_REQUIREMENT
WORKTREE_ACTIVITY != VERIFIED_PROGRESS
STALE_WORKER_STEER != SUCCESSOR_INPUT
REMOTE_ROUTE_FAILURE != LOCAL_FALLBACK_PERMISSION
SUPERVISION_EVENT != VERIFIED_OUTCOME
```

### T251 acceptance requirements

A future owning spec must prove at least:

1. a steer persisted immediately before supervisor crash is recovered and delivered at most once to the still-valid target generation or explicitly reconciled/quarantined;
2. transport “send succeeded” without worker consumption cannot be reported as consumed;
3. worker replacement between persist and delivery fences the stale target generation and does not inject the message into the successor without reconciliation;
4. duplicate doorbells/wake notifications deduplicate against one durable event/message identity;
5. a declared wait with an explicit owner/action/recheck policy does not escalate as a wedge while the declaration remains fresh;
6. an undeclared quiet live worker eventually surfaces bounded attention instead of being silently ignored forever;
7. a proven dead/missing worker does not generate infinite repeated alarms for the same incarnation;
8. remote route loss surfaces remediation/attention and never silently reroutes execution locally;
9. one supervisor can sleep without model-token polling while deterministic watchers still surface an actionable completion/failure/decision;
10. removing the Firstmate donor implementation leaves canonical steering/wake state readable and provider-neutral.

## T252 canonical contract minimums

### `ChangeSetCandidate`

```text
change_id
repository / project identity
base_ref / base_sha
submitted_head_sha / tree_sha
intent_record_ref
source workspace/worktree/run
actor / worker refs
created_at
publication_target
```

### `QualificationPlan`

```text
plan_revision
required_steps[]
step_order / dependency
required_reviewers / tools where policy-bound
repair_policy
finding_policy
CI/check policy
publication_policy
recovery policy
```

### `QualificationStepReceipt`

```text
change_id
head_sha / tree_sha
step_id
exact tool/agent/runtime revision
started_at / finished_at
inputs / command digest
status
findings[]
artifacts / logs / evidence refs
```

### `ReviewedHeadBinding`

```text
head_sha / tree_sha
review_revision
reviewer/tool identity
finding set digest
decision set digest
validity / expiry
superseded_by
```

### T252 hard invariants

```text
PIPELINE_GREEN != MERGE_AUTHORIZATION
QUALIFICATION_RECEIPT != DIFFERENT_HEAD_QUALIFICATION
CI_REPAIR != REVIEW_INHERITANCE
AUTO_FIX != USER_INTENT_MUTATION_AUTHORITY
FORCE_WITH_LEASE != CONTENT_VERIFICATION
PATCH_ID_MATCH != FINAL_TREE_EQUIVALENCE
RECOVERY_ANCHOR != PUBLICATION_PROOF
LOCAL_GATE_REMOTE != SOURCE_REPOSITORY_AUTHORITY
CHANGE_GATE != GOLAM_EFFECT_GATE
PUBLICATION_SUCCESS != VERIFIED_PRODUCT_OUTCOME
PR_CREATED != CHANGE_ACCEPTED
```

### T252 acceptance requirements

A future owning spec must prove:

1. qualification evidence for head A cannot satisfy publication of changed head B without explicit revalidation/equivalence policy evidence;
2. review/test/document/lint/security/CI required by policy cannot be removed or reordered by a lower-scope configuration;
3. a bounded mechanical auto-fix produces a new head and a visible repair lineage; an intent-changing repair requires escalation/review;
4. stale or provider-attributed `ask-user` findings do not get consumed as auto-fix attempts;
5. publication refuses when the live remote target contains unincorporated commits not present in the candidate/equivalence proof;
6. recovery anchors preserve unpublished work across cleanup/crash but never count as review or publication evidence;
7. CI repair after a PR is opened cannot overwrite unrelated/newer remote work and remains exact-head bound;
8. crash/restart recovers run state, findings, repair rounds and pending publication without duplicate publication;
9. detached/disposable worktree cleanup cannot remove the only preserved copy of an unpublished candidate;
10. no-mistakes donor code can be removed without making Golam qualification/publication records unreadable.

## Refinements to existing tasks — no new IDs

### T185 / T193 / T200 — live human-agent artifact collaboration projection

Use `kgoedecke/doop@77cb306aad47b9c901979c9d246105958458d7fb` as a behavior/source reference for live collaborative artifact UX:

- live actor/presence projection;
- explicit agent working/status projection;
- exact artifact/frame/element edit indicators;
- comments/feedback bound to exact artifact revision + element identity;
- activity feed derived from canonical events;
- live rendered previews in a sandboxed content boundary;
- undo/redo as new governed edits, not authority-history rewrite;
- MCP/OAuth agent access projected as the approving principal's bounded capability context rather than standalone authority.

The Experience layer may stream updates aggressively, but durable/canonical artifact truth remains T193/provider-owned and authorization remains canonical Golam authority.

### T214 — artifact comment / review binding

A review/comment that expects later action must bind:

```text
artifact_id
artifact_revision / content digest
element/selection identity if applicable
actor/principal
comment/review identity
requested action / question
created_at / expiry
settlement
```

A comment on revision N cannot silently approve or reject materially different revision N+1.

### T232 — distilled design/behavior memory

Doop-style distillation is accepted only as a source of `PreferenceRuleCandidate` / style/workflow candidates. A distiller may propose durable rules from examples/decisions, but promotion still requires T232 evidence/scope/conflict/activation policy.

### T243 — collaboration / tenancy boundary

Doop multi-user workspaces remain a future tenancy/collaboration reference. Golam's current baseline stays `PERSONAL_SINGLE_OWNER`. Adding live presence or remote collaborators to a surface does not prove tenant isolation, role security or enterprise readiness.

### AGPL / permission Source Foundry rule

Doop's public AGPL-3.0 source is not copied into Golam by default merely because the behavior is useful. For every copied Doop component, Source Foundry must record the exact permission basis used (founder-supplied direct permission or public AGPL compliance), plus transitive rights and distribution/source-offer obligations. If exact alternate permission scope cannot be demonstrated, prefer behavior reimplementation or comply with AGPL as applicable.

## Canonical shared-contract ownership extension

Before T251/T252 implementation, T198 must assign exactly one owner/version/migration authority for:

```text
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

No runtime/backend/UI/git-adapter package may create package-local protected truth for these concepts.

## Dependency and implementation order

After a future owning lifecycle is authorized:

```text
P0_SHARED:
  T198 shared owner/version/migration freeze

P1_SUPERVISION:
  T251 Durable Supervision Event / Steering Inbox / Wake
    depends on T205 + T230 + T247 + T249

P1_CHANGE_DELIVERY:
  T252 Verified Change Qualification / Publication / Repair Gate
    depends on T149/T150 + T214/T216 + T240 + T249

P2_EXPERIENCE_REFINEMENTS:
  T185/T193/T200/T214/T232 Doop-derived live collaboration patterns
```

T251 and T252 are independent of one another after their shared prerequisites are frozen.

## Source reuse priority

### Firstmate

```text
1. reimplement canonical contracts/provider-neutral fake
2. port/selectively copy queue/generation/recovery primitives only when tests prove value
3. reuse portable regression fixtures aggressively
4. do not import Firstmate home/task state as canonical authority
```

### no-mistakes

```text
1. define canonical change qualification/publication records first
2. port exact-head/reconciliation/recovery tests
3. selectively reuse pure Git helpers where measured
4. do not import its SQLite/run database as Golam canonical state
```

### Doop

```text
1. use behavior/UX patterns first
2. prefer reimplementation at Experience/Artifact boundary
3. selectively copy only exact components with Source Foundry rights closure
4. never import Doop auth/workspace/activity state as Golam authority
```

## No-gap source disposition

| Source capability | Existing owner | New work |
| --- | --- | --- |
| Firstmate formation / crew dispatch | T230/T249 | strengthen only |
| Firstmate durable steering inbox | no explicit owner | T251 |
| Firstmate durable wake/event supervision | no explicit owner | T251 |
| Firstmate worker generation/restart | T205/T249 | consume, do not duplicate |
| Firstmate remote subordinate homes | T205/T231/T249 | reference only; T251 route behavior |
| no-mistakes exact-head qualification | T149/T150/T216 insufficient for code-publication lifecycle | T252 |
| no-mistakes repair/publication guard | no dedicated owner | T252 |
| no-mistakes recovery anchors | no dedicated software-change owner | T252 |
| Doop live presence/activity | T185/T200 | refine |
| Doop sandboxed rendered artifacts | T193 | refine |
| Doop comments/review anchors | T214 | refine |
| Doop distilled style memory | T232 | refine |
| Doop team workspaces | T243 | future reference only |
| classifier.dev / jev_search | T246 | no new work |
| unreal-agent | T247 | no new work |

No additional material gap from this source set justifies T253.

## Current safe sequencing

1. Keep active Spec 006 PR #24 unchanged in scope.
2. Qualify this planning head independently.
3. Do not infer implementation permission from this document.
4. After Spec 006 canonical closeout, re-fetch live successor authority.
5. Before T251/T252 implementation, freeze T198 ownership and create bounded owning specs with exact scope-out/threat/failure fixtures.
6. Require exact Source Foundry records before copied donor code enters implementation.
7. Preserve normal merge history; no force-push/rebase is introduced as a Golam governance default by the no-mistakes source review.

```text
TASK_GRAPH_EXTENDS_THROUGH_T252=YES
T253_REQUIRED=NO
PLAN_CONTENT_IMPLEMENTATION_READY=YES
IMPLEMENTATION_AUTHORIZED=NO
ACTIVE_SPEC_006_PR_24_WIDENED=NO
CURRENT_POINTER_CHANGED=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_SOURCE_COMPONENT_ADMITTED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
WAIVER_TAKEN=NO
```
