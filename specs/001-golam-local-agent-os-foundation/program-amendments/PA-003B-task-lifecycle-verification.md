# PA-003B — Task Lifecycle and Verification Matrix

**Status**: PROPOSED_FOR_REVIEW  
**Date**: 2026-08-28  
**Parent**: `PA-003-product-spine-golden-loop.md`  
**Reason**: the PA-003 consistency review found two product-spine gaps that must be closed before future implementation: Task lifecycle must remain distinct from Run lifecycle, and completion criteria must have explicit verification state rather than relying on a terminal model claim.

## 1. Decision

Golam SHALL treat a durable `Task` as a lifecycle distinct from any particular `Run` and SHALL bind every non-trivial completion criterion to an inspectable verification strategy and evidence state.

The core invariants are:

```text
FAILED_RUN != FAILED_TASK
SUCCEEDED_RUN != SATISFIED_TASK
MODEL_SAYS_DONE != VERIFIED_CRITERION
VERIFICATION_PLAN != AUTHORITY
STALE_VERIFICATION != CURRENT_PROOF
USER_ACCEPTS_UNVERIFIED != VERIFIED_SUCCESS
```

This amendment adds no new product surface. It closes false-success and task-continuity gaps inside the Golden Loop.

## 2. Task lifecycle

A future owning spec MUST define an explicit durable Task lifecycle sufficient to represent at least:

- `DRAFT` — intent exists but the current Task Contract is not ready for execution;
- `READY` — Task Contract is executable but no current Run is actively advancing it;
- `IN_PROGRESS` — one or more current Runs/workers are legitimately advancing the Task;
- `SUSPENDED` — the Task is intentionally retained but execution is paused;
- `BLOCKED` — the Task cannot currently advance because of user input, policy, environment, node/resource availability, unknown external effect, or another explicit blocker;
- `SATISFIED` — all current required completion criteria are verified against non-stale evidence;
- `CLOSED_UNVERIFIED` — the user explicitly closes/accepts the Task while one or more required criteria remain unverified, unverifiable, stale, failed, or blocked;
- `CANCELLED` — the user or an authorized policy path intentionally terminates pursuit of the Task without satisfaction;
- `SUPERSEDED` — another explicitly linked Task replaces this Task while preserving provenance/history.

Exact names may be refined by the owning spec, but the semantic distinctions above are binding.

## 3. Task state is not inferred from one Run

A Run may fail, be cancelled, crash, pause, or complete its bounded execution plan without determining the terminal state of the Task.

Examples:

- a failed Run may be followed by a new Run against the same Task;
- a successful Run may leave completion criteria unsatisfied;
- cancelling a worker does not cancel the Task unless a separate attributable Task transition occurs;
- a process crash may move execution into recovery while the Task remains open;
- a Task may remain `BLOCKED` across multiple sessions and later resume without losing identity.

Every terminal Task transition MUST be attributable and durable.

## 4. Versioned Task Contract

Material Task Contract changes SHALL create a new version/revision reference rather than silently rewriting the semantic basis for prior Runs and verification evidence.

A material change includes changes to:

- goal;
- required completion criteria;
- non-negotiable constraints;
- declared scope/resources;
- locality posture;
- expected deliverables;
- explicit stop conditions.

Historical Runs and receipts retain the Task Contract version under which they operated.

A steering action that changes a completion criterion invalidates or re-evaluates prior verification for that criterion as appropriate.

## 5. Verification Plan / Matrix

Every non-trivial Task SHALL maintain an inspectable `VerificationPlan` or equivalent projection tied to the current Task Contract version.

For each required criterion it SHOULD record at least:

- stable `criterion_id`;
- criterion text/semantic form;
- source of the criterion (user, governing spec/policy, task compiler, etc.);
- required versus optional status;
- planned verification strategy;
- preferred verifier/tool/observation class;
- required evidence type/reference shape;
- freshness/validity requirements;
- independence requirement where applicable;
- current verification state;
- latest evidence references;
- reason when blocked, failed, stale, not applicable, or unverifiable.

The Verification Plan is a planning/evidence object. It never grants capability or effect authority.

`VERIFICATION_PLAN != AUTHORITY`

## 6. Criterion verification states

The owning spec MUST preserve semantic states sufficient to distinguish at least:

- `PENDING`;
- `VERIFYING`;
- `VERIFIED`;
- `FAILED`;
- `BLOCKED`;
- `STALE`;
- `UNVERIFIABLE`;
- `NOT_APPLICABLE` where a versioned Task Contract legitimately removes the requirement.

A required criterion may be marked `VERIFIED` only when its verification strategy has sufficient current evidence.

A model's prose assertion, an action-path success response, or a workflow checkpoint is not independent proof when a stronger deterministic or observational check is available.

## 7. Verification is planned before claiming success

Where criteria are known before execution, Golam SHOULD select the verification strategy before consequential action. This reduces evaluator drift and prevents the system from inventing a weak success test after seeing its own output.

The plan may evolve when new evidence or constraints appear, but changes are versioned and attributable.

`POST_HOC_WEAK_VERIFIER != REQUIRED_PROOF`

## 8. Independence and observation

When practical, the verification path SHOULD be meaningfully independent from the action path.

Examples:

- after writing a file, re-read or run a deterministic parser/test rather than trusting the write call response alone;
- after changing repository code, execute the declared test/lint/build checks needed by the criterion;
- after a remote/external effect, use the effect reconciler or independent provider state where available;
- after computer-control action, observe the resulting semantic/application state rather than trusting input injection completion;
- after retrieval/research, bind claims to captured sources/evidence rather than model recollection.

Independence is evidence quality, not extra authority.

## 9. Freshness and invalidation

Verification evidence can become stale.

Golam MUST invalidate or mark stale verification when a material dependency changes, including where relevant:

- Task Contract revision;
- source file/content hash;
- repository head/worktree state;
- external provider state;
- device/application state;
- policy/approval/lease generation when the criterion depends on it;
- user steering that changes the expected result.

`STALE_VERIFICATION != CURRENT_PROOF`

## 10. Task satisfaction rule

A Task MUST NOT enter `SATISFIED` while any current required completion criterion is `PENDING`, `VERIFYING`, `FAILED`, `BLOCKED`, `STALE`, or `UNVERIFIABLE`.

A criterion removed through a legitimate versioned Task Contract change may become `NOT_APPLICABLE` to the new version; the history must remain visible.

If the user explicitly accepts or closes work despite incomplete verification, Golam records `CLOSED_UNVERIFIED` or equivalent—not `SATISFIED`.

`USER_ACCEPTS_UNVERIFIED != VERIFIED_SUCCESS`

This is a correctness invariant, not a benchmark target.

## 11. Trust Receipt binding

A Trust Receipt for a terminal/meaningful stop SHALL include the current Task Contract version and criterion-level verification summary.

For verified claims, it MUST reference sufficient canonical/captured evidence to reconstruct why the claim is marked verified.

Where the existing integrity/audit ledger exposes canonical chain positions or digests, the receipt SHOULD include stable references/digests sufficient to validate that its evidence basis corresponds to canonical records without claiming a signature that was not actually produced.

A receipt cannot convert `CLOSED_UNVERIFIED` into apparent success by wording.

## 12. Recovery and new Runs

Starting a recovery/new Run against an existing Task MUST preserve:

- Task identity;
- current Task Contract version;
- criterion states and evidence references;
- known stale verification;
- blockers/unknown effects;
- prior failed/cancelled Run history.

A new Run may re-verify stale/failed criteria but cannot erase the prior evidence trail.

## 13. Core Alpha implications

Core Alpha MUST include exact tests/scenarios proving:

1. one Run fails and a later Run continues the same Task without identity loss;
2. a Run completes but the Task cannot become `SATISFIED` while a required criterion is unverified;
3. steering changes a criterion and invalidates stale verification;
4. a deterministic verifier catches an action-path false success;
5. explicit user acceptance of unverified work is represented separately from verified satisfaction;
6. a Trust Receipt's criterion summary matches canonical evidence/verification state;
7. process restart preserves Task and Verification Plan state.

## 14. Owning specs

### Spec 004

Freeze durable Task lifecycle, Task Contract versioning, Verification Plan primitives, Run/Task transition semantics, restart persistence, and in-flight invalidation rules.

### Spec 005

Expose criterion/verification state in CLI/TUI, produce criterion-bound Trust Receipts, and include the PA-003B scenarios in Core Alpha.

### Spec 008

Worker/subgoal completion must map back to parent Task criteria explicitly; worker success does not imply parent Task satisfaction.

### Spec 010

Measure false-success and stale-verification behavior; verify Task terminal states and Trust Receipts against canonical evidence at exact release heads.

## 15. Non-goals

PA-003B does not authorize:

- a universal theorem prover;
- mandatory LLM judges;
- redundant verification for every trivial read-only response;
- new provider/cloud dependencies;
- changing active Spec 003 implementation scope;
- treating verification policy as effect authority.
