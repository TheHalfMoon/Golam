# Golam Program Implementation Readiness — 2026-09-30

**Status:** `CONTENT_IMPLEMENTATION_READY / EXECUTION_NOT_YET_AUTHORIZED`
**Planning PR:** #28
**Extends:** `program-implementation-readiness-2026-09-29.md`
**New tasks:** T251–T252

## 1. Readiness meaning

```text
CONTENT_IMPLEMENTATION_READY
!=
IMPLEMENTATION_AUTHORIZED
```

This packet makes T251/T252 specification-ready by freezing bounded scope, canonical dependencies, donor posture, failure semantics, acceptance fixtures and removal/fallback requirements. Implementation still requires live repository governance and an owning spec after the current active frontier permits it.

## 2. Entry gates shared by T251/T252

Before first implementation code:

1. fetch live `main`, `AGENTS.md`, Constitution, `specs/CURRENT.md`, PR #28 and active Spec 006 state;
2. PR #28 planning overlay is canonical/qualified under repository policy;
3. Spec 006 is canonically closed and successor authority is re-fetched;
4. T198 names one owner/version/migration authority for all T251/T252 shared contracts;
5. the owning spec freezes scope-in/out, threat model and exact fixtures before production code;
6. copied donor components have exact Source Foundry records including rights/NOTICE/dependency closure;
7. no unadmitted hosted service/runtime/model/secret is required;
8. no unresolved material security/architecture review finding exists on the exact owning-plan head;
9. normal forward history remains the repository governance default.

## 3. Package D — Durable Supervision and Steering

**Primary task:** T251
**Primary donor/reference:** `kunchenguid/firstmate@23e5584714e6765cc223a1740d385d0e85f8ad8e`
**Depends on:** T205 + T230 + T247 + T249

### Purpose

Provide a provider-neutral durable internal control/event channel between supervisor and workers so Golam can supervise long-running multi-agent work without continuous model polling and without losing steering/wake events across crashes, reconnects or worker replacement.

### Scope-in

- `SteeringEnvelope`;
- `SteeringReceipt`;
- `SupervisionEvent`;
- `WakeQueueEntry`;
- `WakeReceipt`;
- `SupervisorAttentionProjection`;
- `SupervisionRouteBinding`;
- durable persist-before-doorbell semantics;
- exact worker/task/claim/runtime-generation binding;
- delivery vs consumption settlement;
- deduplication of repeated doorbells/wakes;
- restart recovery;
- stale-generation quarantine/reconciliation;
- bounded escalation / re-surfacing;
- declared wait / owner-held / supervisor-held distinctions;
- dead/missing worker once-per-incarnation attention;
- remote route failure without local fallback;
- deterministic watchers capable of sleeping model reasoning between actionable events.

### Scope-out for first package

- copying Firstmate's full shell/tmux/Herdr control plane;
- treating pane text as canonical worker state;
- generic remote-shell product functionality;
- automatic worker restart after ambiguous Effects;
- model-authored liveness truth;
- replacing T205 execution generation;
- replacing T249 work ownership/liveness;
- replacing T247 external InputEnvelope;
- cluster consensus / distributed queue infrastructure.

### Donor admission order

1. implement Golam provider-neutral contracts and deterministic fake backend;
2. port Firstmate task-inbox/wake/relaunch regression fixtures into Golam-shaped tests;
3. compare a minimal Rust/local-file or existing canonical-store implementation against donor behavior;
4. selectively copy/port pure queue/generation primitives only if they reduce risk/complexity measurably;
5. backend-specific terminal/remote adapters remain outside canonical supervision state.

### Required qualification fixtures

```text
persist_before_doorbell
crash_after_persist_before_send
send_success_no_consumption_not_settled
duplicate_doorbell_one_message
restart_recovers_unconsumed_valid_steer
replacement_fences_old_generation_steer
stale_steer_not_injected_into_successor
declared_wait_no_false_wedge
undeclared_quiet_worker_resurfaces
proven_dead_once_per_incarnation
active_worker_no_polling_token_loop
remote_route_loss_no_local_fallback
unknown_worker_state_fails_conservative
expired_steer_not_delivered
supervision_event_projection_no_authority
firstmate_removed_state_still_readable
```

### Security/failure questions that must have explicit answers

- What proves the target worker generation is still current at delivery and consumption?
- How is a message settled if transport ACK exists but the worker crashes before applying it?
- What prevents repeated wake notifications from spawning repeated supervisor turns?
- How is a supervisor-owned decision distinguished from an owner/user-held decision?
- How does quarantine/reconciliation handle a steer whose target task was cancelled/replaced?
- What bounded retention/compaction preserves auditability without infinite queue growth?
- What happens if a route reports `alive` but the underlying worker is unreachable?

## 4. Package E — Verified Change Delivery Gate

**Primary task:** T252
**Primary donor/reference:** `kunchenguid/no-mistakes@a1c06cdaefcefa7cbc6902ac507f68ce9eed14ad`
**Depends on:** T149/T150 + T214/T216 + T240 + T249

### Purpose

Provide one exact-head-bound software/change qualification and publication capability for code/config/content changes produced by Golam or its workers.

### Scope-in

- `ChangeSetCandidate`;
- `ChangeIntentRecord`;
- `QualificationPlan`;
- `QualificationStepReceipt`;
- `Finding` / `FindingDecision`;
- `RepairRound`;
- `ReviewedHeadBinding`;
- `PublicationPlan` / `PublicationReceipt`;
- `RecoveryAnchor`;
- `ChangeDeliveryOutcome`;
- disposable worktree execution;
- monotonic required gate sequence;
- exact-head review/test/docs/lint/security/CI evidence;
- bounded safe mechanical fixes;
- human/owner/reviewer escalation for intent-changing findings;
- repair lineage;
- live remote-head verification;
- force-with-lease/equivalent concurrency protection;
- preservation of unpublished heads before cleanup;
- crash recovery without duplicate publication;
- PR/publication evidence linked to the qualified head.

### Scope-out for first package

- adopting no-mistakes' bare Git remote/daemon/SQLite as canonical Golam state;
- changing Golam repository governance to force-push/rebase workflows;
- automatic merge authority;
- allowing an AI reviewer to waive repository policy;
- treating patch-ID equality alone as semantic review equivalence;
- generic binary release publication;
- package registry/container registry release management;
- bypassing GitHub/forge branch/ruleset protections;
- automatic intent-changing repairs.

### Donor admission order

1. define canonical Golam change/qualification/publication records;
2. implement deterministic fake forge and local disposable-worktree harness;
3. port no-mistakes exact-head, private-content preservation, recovery-anchor, repair and publication-race fixtures;
4. selectively reuse pure Git reconciliation helpers after independent review;
5. add one real GitHub adapter only after provider/account/secret/effect boundaries are explicit;
6. other forges remain separate adapters.

### Required qualification fixtures

```text
head_a_review_does_not_cover_head_b
required_step_cannot_be_removed
required_step_cannot_be_reordered
mechanical_fix_creates_new_head
intent_change_requires_review
ask_user_not_consumed_as_autofix
repair_attempt_budget_enforced
remote_head_advanced_publication_denied
force_with_lease_not_content_proof
private_unincorporated_commit_preserved
recovery_anchor_not_review_proof
ci_repair_exact_head_bound
crash_before_publication_resume_once
crash_after_remote_update_reconcile_no_duplicate
cleanup_preserves_only_unpublished_copy
stale_pr_attestation_rejected
forge_provider_removed_records_still_readable
no_mistakes_removed_contract_still_works
```

### Security/failure questions that must have explicit answers

- Which step classes may auto-fix, and which always require reviewer/user judgment?
- What exact evidence proves a repair remains within reviewed intent?
- How does publication distinguish remote-head concurrency from content verification?
- What survives if the run crashes after remote push but before local settlement?
- How are unpublished/private heads preserved without making preservation equal approval?
- How does a provider-side CI rerun map to the exact candidate head?
- What is the rollback/reconciliation behavior if PR creation succeeds but local receipt persistence fails?

## 5. Doop-derived Experience / Artifact refinement readiness

**Source/reference:** `kgoedecke/doop@77cb306aad47b9c901979c9d246105958458d7fb`
**No new task ID.**

### Target owners

```text
T185 Agent UI Projection
T193 Semantic Artifact Provider Family
T200 Product Command Center
T214 Review Checkpoints
T232 Governed Learning/Evolution
T243 Deployment/Tenancy Boundary
```

### Safe first implementation slice

A future Experience/Artifact package may implement provider-neutral projections for:

- actor presence;
- agent working state;
- artifact revision/edit activity;
- exact revision/element comment anchors;
- activity feed;
- sandboxed rendered preview;
- streamed artifact update projection;
- review feedback settlement.

It must **not** implement multi-tenant security merely by copying Doop workspaces/auth, and it must not admit Doop AGPL code without an exact Source Foundry rights record.

### Required fixtures

```text
presence_disconnect_expires
agent_status_not_task_truth
comment_revision_mismatch_rejected_or_stale
comment_element_missing_after_revision_surfaces_conflict
activity_feed_rebuilds_from_canonical_events
preview_sandbox_blocks_parent_privilege
untrusted_rendered_content_not_instruction
undo_creates_new_revision
mcp_user_access_not_effect_authority
distilled_rule_stays_candidate
single_owner_baseline_unchanged
```

## 6. Shared contract ownership freeze

Before Packages D/E, T198 must identify one owner/version/migration authority for:

### Supervision

```text
SteeringEnvelope
SteeringReceipt
SupervisionEvent
WakeQueueEntry
WakeReceipt
SupervisorAttentionProjection
SupervisionRouteBinding
```

### Change delivery

```text
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

No donor/backend/UI/provider may own a second authoritative copy.

## 7. Implementation sequence

```text
P0:
  canonical PR #28 qualification / merge under repository policy
  active Spec 006 canonical closeout
  live successor authority
  T198 shared owner/version/migration freeze

P1_PARALLEL:
  Package D / T251 Durable Supervision
  Package E / T252 Verified Change Delivery

P2_WHEN_RELEVANT_ARTIFACT_EXPERIENCE_PACKAGE_EXISTS:
  Doop-derived projection refinements under T185/T193/T200/T214/T232/T243
```

T251 and T252 do not depend on one another.

## 8. No-gap checklist for the new source set

```text
FIRSTMATE_EXACT_PIN_RECORDED=YES
FIRSTMATE_LICENSE_RECORDED=YES
FIRSTMATE_REUSE_MODE_RECORDED=YES
FIRSTMATE_CONTROL_PLANE_NON_ADOPTION=YES

NO_MISTAKES_EXACT_PIN_RECORDED=YES
NO_MISTAKES_LICENSE_RECORDED=YES
NO_MISTAKES_REUSE_MODE_RECORDED=YES
NO_MISTAKES_GATE_NON_ADOPTION=YES

DOOP_EXACT_PIN_RECORDED=YES
DOOP_PUBLIC_AGPL_RECORDED=YES
DOOP_PERMISSION_FIREWALL_RECORDED=YES
DOOP_REUSE_MODE_RECORDED=YES
DOOP_CONTROL_PLANE_NON_ADOPTION=YES

CLASSIFIER_DEV_PRIOR_ROLE_REVALIDATED=YES
JEV_SEARCH_PRIOR_ROLE_REVALIDATED=YES
UNREAL_AGENT_PRIOR_ROLE_REVALIDATED=YES

T251_SCOPE_READY=YES
T251_ACCEPTANCE_FIXTURES_READY=YES
T252_SCOPE_READY=YES
T252_ACCEPTANCE_FIXTURES_READY=YES
DOOP_REFINEMENT_FIXTURES_READY=YES
T253_REQUIRED=NO
```

## 9. Readiness disposition

```text
PLAN_CONTENT_IMPLEMENTATION_READY=YES
TASK_GRAPH_EXTENDS_THROUGH_T252=YES
IMPLEMENTATION_AUTHORIZED=NO
NEW_PARALLEL_AUTHORITY_SYSTEM=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_SOURCE_COMPONENT_ADMITTED=NO
ACTIVE_SPEC_006_PR_24_WIDENED=NO
CURRENT_POINTER_CHANGED=NO
WAIVER_TAKEN=NO
```
