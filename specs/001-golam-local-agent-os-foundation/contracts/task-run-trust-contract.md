# Contract: Durable Task, Run, Control, and Trust Receipt

**Authority note**: This additive program contract is introduced by `program-amendments/PA-003-product-spine-golden-loop.md`. It governs future Specs 004–010 and does not authorize implementation in the active Spec 003 package.

## 1. Durable task identity

A `Task` is the durable user-intent identity for a unit of work.

A Task MUST survive ordinary changes in:

- interaction surface;
- session/thread;
- model/provider;
- daemon process lifetime;
- execution attempt;
- worker topology;
- device used by the same authorized user.

`SURFACE != TASK_IDENTITY`

## 2. Task, Session, Run, Worker are distinct

- `Task`: durable user intent and completion state.
- `Session`: conversational/navigation projection.
- `Run`: one execution attempt or continuation against a Task.
- `Worker`: bounded delegated executor acting inside a Run/Task causality graph.

`TASK != SESSION != RUN != WORKER`

A worker crash MUST NOT destroy the Task. Starting a new chat MUST NOT silently create a new task when the user is explicitly continuing an existing task. Switching model/provider MUST NOT fork authority or canonical task identity by itself.

## 3. Task Contract

Every non-trivial Task MUST have a current inspectable `TaskContract` projection derived from user intent and protected policy state.

It contains at least:

- `task_id`;
- goal;
- completion criteria;
- constraints;
- deliverables;
- scope/resource refs;
- locality posture;
- budget posture;
- approval/autonomy posture;
- expected effect classes where known;
- stop conditions;
- unresolved material ambiguities.

The Task Contract MUST NOT itself mint authority.

`TASK_CONTRACT != CAPABILITY_LEASE`

A model may propose amendments to a Task Contract but cannot unilaterally expand protected authority, data scope, locality, or effect permissions.

## 4. Goal Ledger relationship

The Goal Ledger remains canonical operational goal state outside ordinary compaction.

The Task Contract is the user-facing contract/projection; the Goal Ledger is the durable operational execution state. They MUST retain traceable correspondence but MAY evolve at different levels of detail.

A Goal Ledger update that materially changes user intent MUST be reflected to the user-facing Task Contract or require explicit user clarification/authorization as applicable.

## 5. Run lifecycle

A Run MUST have explicit durable states sufficient to represent at least:

- CREATED;
- ACTIVE;
- PAUSING;
- PAUSED;
- WAITING_USER;
- WAITING_EXTERNAL;
- BLOCKED_POLICY;
- BLOCKED_ENVIRONMENT;
- VERIFYING;
- PARTIAL;
- SUCCEEDED;
- FAILED;
- CANCELLED;
- UNKNOWN_EFFECT_BLOCKED;
- RECOVERING.

Exact implementation states may be refined by the owning spec.

A Run completion state MUST be derived from evidence and verification state, not only model text.

## 6. In-flight control semantics

The canonical runtime SHALL expose semantic operations equivalent to:

- Pause;
- Stop/Cancel;
- Steer;
- AddConstraint;
- ChangePriority;
- Inspect;
- TakeOver where computer/input authority exists;
- Resume.

These commands MUST be attributable to an authenticated principal and checked against the current authority applicable to that control operation.

User steering may narrow goal/scope or request narrower authority. Any widening request still passes normal policy/capability/approval rules.

A Task/Run control record or Task Contract revision MUST NOT directly mint, rewrite, revoke, or narrow protected authority objects merely because the requested direction is safer. If steering requires a capability lease to be narrowed/revoked, an approval/preauthorization to be invalidated, an egress permit to be reduced, or another protected authority object to change, that change proceeds through the owning typed protected mutation path with durable attribution/evidence. Planner/executor behavior may immediately respect the narrower Task scope while protected authority state converges, but stale broader authority MUST NOT be silently reused contrary to the current Task constraints.

`TASK_CONTROL != AUTHORITY_MUTATION`
`USER_STEERING_CAN_NARROW_BUT_NOT_SILENTLY_WIDEN_AUTHORITY`

Pause/Stop/Cancel MUST prevent new dispatch that is no longer authorized by the resulting Run/Task state. They MUST NOT fabricate cancellation of an external or irreversible effect that has already been dispatched or may have taken effect. Such work remains subject to handler-specific cancellation evidence, observation and normal Effect Gate reconciliation; if outcome is uncertain, the Run enters `UNKNOWN_EFFECT_BLOCKED` or an equivalent state rather than reporting successful cancellation.

`STOP_REQUESTED != EXTERNAL_EFFECT_CANCELLED`

TakeOver transfers only the explicitly governed interactive/input-control lease or equivalent bounded control needed for the takeover. It MUST NOT silently grant general Task, filesystem, secret, network, or effect authority.

Resume MUST re-read current protected state, relevant live environment state, stale references, expiries, approvals, and UNKNOWN external effects before continuing. A paused/stopped Task Contract or cached pre-pause capability view is not authority to resume protected work.

## 7. Progressive autonomy projection

User-facing autonomy postures are UX projections over protected policy/capability state.

They MAY simplify common modes such as observe, suggest, bounded local action, policy-bounded external action, and fresh approval for consequential action.

They MUST NOT create a hidden super-capability or universal irreversible allow rule.

`AUTONOMY_POSTURE != AUTHORITY`

## 8. Trust Receipt

Every meaningful terminal Task/Run state SHOULD expose a `TrustReceipt` projection.

A receipt SHOULD include, where applicable:

- task/run identifiers;
- outcome state;
- completion criteria status;
- verified versus unverified claims;
- evidence references;
- files/artifacts created or modified;
- external effects and receipts;
- UNKNOWN outcomes;
- network/egress destinations used;
- model/provider/harness profile identifiers;
- tools/sandbox providers used;
- approvals/preauthorizations consumed;
- user interventions/takeover events;
- memory/learning candidates created;
- unresolved questions/blockers;
- rollback/compensation state where relevant.

A Trust Receipt is derived from canonical records and cannot rewrite or replace them.

`TRUST_RECEIPT != AUTHORITY_RECORD`

## 9. Evidence binding

A receipt claim marked verified MUST point to sufficient canonical/captured verification evidence.

Model prose, self-reported tool success, or workflow checkpoint state alone cannot satisfy a deterministic verification requirement where an independent check is available.

`CLAIMED_SUCCESS != VERIFIED_SUCCESS`

## 10. Data-egress accounting

Where external data movement occurs, the receipt MUST preserve enough information to answer what class of data left the device, through which authorized destination/provider, for which task/run, under which egress decision, without persisting secret plaintext.

Strict-local runs MUST report zero unauthorized external egress and must be externally testable.

## 11. UserModel boundary

Stable user preferences MAY be projected into a compact governed `UserModel`, separate from general user/project/episodic memory.

UserModel entries require provenance and supersession semantics. Sensitive inferred traits MUST NOT silently become durable profile facts.

`USER_MODEL != ALL_USER_MEMORY`

## 12. Initiative and attention

Future proactive workers/routines MUST represent two separate questions:

1. Is Golam authorized to perform or propose this work?
2. Is Golam authorized/appropriate to interrupt or notify the user now through this surface?

`INITIATIVE_AUTHORITY != ATTENTION_AUTHORITY`

Notification policy never grants effect authority, and effect authority never implies unlimited notification rights.

## 13. Capability truth

A provider/platform capability may be exposed as supported only when the adapter declaration and owning-spec conformance evidence agree.

`DECLARED_CAPABILITY + CONFORMANCE_EVIDENCE -> CLAIMED_CAPABILITY`

Unsupported or degraded capabilities MUST fail honestly rather than trigger silent insecure fallback.

## 14. Portability

Task/evidence/receipt/user-owned memory formats SHOULD have stable export paths where practical.

Imports from external agents MUST remain quarantined/provenanced until governed promotion. Imported credentials and protected authority state are denied by default.

## 15. Core Alpha requirement

After Spec 005, CLI/TUI MUST be able to demonstrate the complete Golden Loop without Desktop, Mobile, broad channels, or worker swarms being required.

The owning product gate must prove representative repository, research/evidence, filesystem/document, cross-session memory, interrupt/recovery, and strict-local tasks with exact evidence. It MUST also include control-path cases proving that steering cannot mutate protected authority by TaskContract side effect, stop/cancel cannot falsely claim an already-dispatched external effect was cancelled, UNKNOWN outcomes block honest completion, takeover remains bounded to its control lease, and resume revalidates current authority rather than reusing stale pre-pause state.
