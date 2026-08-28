# Contract: Task Lifecycle and Verification

**Authority note**: This additive program contract is introduced by `program-amendments/PA-003B-task-lifecycle-verification.md`. It governs future Specs 004/005/008/010 and does not authorize implementation in the active Spec 003 package.

## 1. Task lifecycle is distinct from Run lifecycle

A durable `Task` represents user intent across execution attempts. A `Run` is one execution attempt or continuation.

`FAILED_RUN != FAILED_TASK`

`SUCCEEDED_RUN != SATISFIED_TASK`

No single Run terminal state may silently determine the Task terminal state.

## 2. Minimum Task states

The owning implementation SHALL preserve semantic states sufficient to distinguish:

- DRAFT;
- READY;
- IN_PROGRESS;
- SUSPENDED;
- BLOCKED;
- SATISFIED;
- CLOSED_UNVERIFIED;
- CANCELLED;
- SUPERSEDED.

Equivalent implementation names are allowed only if these distinctions remain inspectable and durable.

Every terminal Task transition MUST be attributable to a principal/system rule and recorded canonically.

## 3. Task Contract versioning

Material Task Contract changes create a new version or revision reference.

Material changes include goal, required completion criteria, non-negotiable constraints, declared scope/resources, locality posture, deliverables, and stop conditions.

Historical Runs, effects, evidence, and receipts MUST retain the Task Contract version under which they occurred.

A new Task Contract version MUST NOT rewrite prior execution history.

## 4. Verification Plan

Every non-trivial Task MUST maintain an inspectable `VerificationPlan` or equivalent projection bound to the current Task Contract version.

Each required criterion MUST have a stable identity and enough metadata to determine:

- what must be proven;
- whether it is required;
- how it will be verified;
- what evidence is sufficient;
- how fresh that evidence must be;
- whether independent observation is required;
- current verification state;
- latest evidence refs;
- why verification is blocked, failed, stale, or unavailable.

`VERIFICATION_PLAN != AUTHORITY`

The plan may request use of tools/capabilities but never grants them.

## 5. Criterion states

The owning implementation MUST preserve states sufficient to distinguish:

- PENDING;
- VERIFYING;
- VERIFIED;
- FAILED;
- BLOCKED;
- STALE;
- UNVERIFIABLE;
- NOT_APPLICABLE.

A required criterion becomes VERIFIED only from sufficient current evidence under its verification strategy.

## 6. No self-asserted success

Model prose, workflow completion, action-path success responses, or worker self-reports are not sufficient verification when a stronger deterministic or observational check is available.

`MODEL_SAYS_DONE != VERIFIED_CRITERION`

`WORKER_SAYS_DONE != VERIFIED_CRITERION`

`ACTION_RETURNED_OK != VERIFIED_EFFECT_OUTCOME`

The existing Effect Gate/reconciler semantics remain authoritative for external effects.

## 7. Pre-declared verification when practical

Where completion criteria are known before execution, Golam SHOULD select the verification strategy before consequential actions.

A verifier may be revised when evidence or constraints change, but the revision MUST be attributable and versioned so a weak post-hoc verifier cannot silently replace an earlier stronger requirement.

`POST_HOC_WEAK_VERIFIER != REQUIRED_PROOF`

## 8. Independent observation

Where practical, verification SHOULD use an evidence path meaningfully independent from the action path.

Examples include:

- re-reading changed files or parsing them after writes;
- running tests/lint/build required by repository criteria;
- observing application/semantic state after computer actions;
- reconciling remote provider state after effects;
- binding research claims to captured sources rather than model memory.

Independent observation does not grant additional authority.

## 9. Freshness and invalidation

Verification evidence MUST be marked stale or re-evaluated when a material dependency changes.

Potential invalidators include:

- Task Contract revision;
- content hash or repository/worktree change;
- external provider state change;
- application/device state change;
- relevant policy/lease/approval generation change;
- user steering that changes expected results.

`STALE_VERIFICATION != CURRENT_PROOF`

A stale criterion cannot support Task satisfaction until reverified.

## 10. Satisfaction rule

A Task MUST NOT enter SATISFIED while any current required criterion is PENDING, VERIFYING, FAILED, BLOCKED, STALE, or UNVERIFIABLE.

A required criterion may become NOT_APPLICABLE only through a legitimate current Task Contract revision that changes/removes that requirement while preserving history.

If a user explicitly closes or accepts work without complete verification, Golam MUST use CLOSED_UNVERIFIED or an equivalent visibly distinct terminal state.

`USER_ACCEPTS_UNVERIFIED != VERIFIED_SUCCESS`

This is a correctness invariant, not a performance target.

## 11. Run-to-Task mapping

A Run may:

- advance some Task criteria;
- fail without failing the Task;
- succeed operationally while leaving Task criteria unresolved;
- be cancelled while the Task remains open/suspended;
- produce evidence that a later Run reuses or invalidates.

Starting another Run MUST preserve Task identity, current Task Contract version, verification state, blockers, unknown effects, and prior Run history.

## 12. Worker-to-Task mapping

Worker/subgoal completion MUST map explicitly to parent Run/Task criteria or evidence refs.

A worker cannot mark the parent Task SATISFIED merely because its delegated subgoal completed.

`WORKER_SUCCESS != PARENT_TASK_SATISFACTION`

## 13. Trust Receipt binding

A terminal or meaningful-stop Trust Receipt MUST include:

- Task identifier;
- current Task Contract version;
- Task terminal/current state;
- criterion-level verification summary;
- verified/unverified/stale/blocked distinctions;
- evidence refs supporting verified claims;
- unresolved UNKNOWN effects where applicable.

Where canonical audit/evidence chains expose stable positions/digests, the receipt SHOULD include references sufficient to validate its evidence basis without falsely claiming a cryptographic signature that was not produced.

A receipt MUST NOT word CLOSED_UNVERIFIED as verified success.

## 14. Recovery

Restart/recovery MUST preserve:

- Task state;
- Task Contract version;
- Verification Plan/version;
- criterion states;
- evidence refs;
- known stale evidence;
- blockers;
- UNKNOWN external effects;
- prior Run history.

Recovery may re-run verification safely, but cannot erase prior failure/staleness evidence.

## 15. Core Alpha and release qualification

Core Alpha MUST prove at least:

1. failed Run followed by a new Run against the same Task;
2. successful Run that cannot satisfy a Task with an unverified required criterion;
3. steering that invalidates stale verification;
4. deterministic verification catching an action-path false success;
5. explicit CLOSED_UNVERIFIED distinct from SATISFIED;
6. Trust Receipt criterion summary matching canonical evidence;
7. restart persistence of Task and Verification Plan state.

Spec 010 SHALL re-test the same invariants against exact release heads and measure false-success/stale-verification behavior.
