# Golam Source Adoption Review — Firstmate, no-mistakes, Doop — 2026-09-30

**Status:** PROGRAM RESEARCH / PLANNING ONLY
**Authority:** No implementation, dependency, runtime, source-component, model, Constitution, `specs/CURRENT.md`, or Spec 006 scope authority is granted by this document.

## 1. Reviewed exact source states

| Source | Exact reviewed state | Public rights posture observed | Golam disposition |
| --- | --- | --- | --- |
| `kunchenguid/firstmate` | `23e5584714e6765cc223a1740d385d0e85f8ad8e` | MIT | `HIGH_VALUE_AGENT_SUPERVISION / STEERING_DONOR` |
| `kunchenguid/no-mistakes` | `a1c06cdaefcefa7cbc6902ac507f68ce9eed14ad` | MIT | `HIGH_VALUE_CHANGE_QUALIFICATION / PUBLICATION_DONOR` |
| `kgoedecke/doop` | `77cb306aad47b9c901979c9d246105958458d7fb` | AGPL-3.0 public repository; founder separately asserts reuse permission | `HIGH_VALUE_LIVE_ARTIFACT_COLLABORATION_REFERENCE / SELECTIVE_DONOR_AFTER_RIGHTS_CLOSURE` |
| `mrmps/classifier-dev` / `classifier.dev` | prior exact review retained | MIT repository; hosted service terms separate | existing T246 source/reference; no new task |
| `caio0452/jev_search` | prior exact review retained | no public license file observed at prior reviewed pin; founder permission recorded | existing T246 source/reference; no new task |
| `unreallabsai/unreal-agent` | prior exact review retained | MIT | existing T247 source/reference; no new task |

Founder permission is an admission input, not automatic technical or transitive-rights admission. Exact-component Source Foundry records remain mandatory before copied source enters an implementation branch.

## 2. Firstmate — durable steering and event-driven supervision

Firstmate is valuable because it treats multi-agent supervision as a durable systems problem rather than continuous model polling.

The reviewed source exposes several high-value patterns:

- one operator-facing supervisor over a bounded crew;
- disposable per-task worktrees;
- persistent local or remote subordinate supervisors with separate homes/state;
- durable task steering inboxes;
- durable wake queues;
- generation-bound recovery/wake evidence;
- event-driven supervision that wakes the supervising agent only when action is required;
- explicit distinction between declared wait, captain-held decision, parked validation gate, active work, stale/wedged work, dead/missing worker, and unknown state;
- bounded re-surfacing / escalation rather than infinite alert storms;
- restart reconciliation from disk;
- remote-route failure that does not silently become local execution;
- pointer/doorbell delivery patterns that avoid inlining arbitrary message content into typed terminal channels;
- extensive portable backend/recovery regression fixtures.

### 2.1 What Golam should reuse

Priority component/source shortlist for future Source Foundry evaluation:

```text
bin/fm-task-inbox-lib.sh
bin/fm-wake-lib.sh
bin/fm-watch.sh
bin/fm-procevent.sh
remote-secondmate routing/restart/job libraries
related task-inbox / wake / relaunch / remote-secondmate tests
docs/architecture.md
docs/remote-secondmates.md
```

Preferred reuse mode:

```text
REIMPLEMENT_BEHAVIOR_IN_CANONICAL_GOLAM_CONTRACTS
-> SELECTIVE_COPY / PORT for proven pure queue/recovery primitives
-> NEVER transplant the Firstmate home/task state as canonical Golam authority
```

### 2.2 Gap exposed

Golam already has Task, WorkClaim, worker generation, InputEnvelope, formation and liveness contracts, but it does not yet give **internal supervisor-to-worker steering and worker-to-supervisor wake delivery** one explicit durable contract.

This is distinct from:

- T247 external inbound redelivery idempotency;
- T205 runtime generation fencing;
- T249 task ownership/liveness;
- T230 team formation.

The missing seam is the durable control/event channel connecting those contracts.

This becomes **T251 — Durable Supervision Event / Steering Inbox / Wake Contract**.

### 2.3 Non-adoptions

```text
FIRSTMATE_HOME != GOLAM_CANONICAL_STATE
TERMINAL_PANE != WORKER_IDENTITY
PANE_HASH != WORK_TRUTH
WAKE_EVENT != TASK_AUTHORITY
STEERING_MESSAGE != EFFECT_AUTHORIZATION
WORKTREE_ACTIVITY != VERIFIED_PROGRESS
REMOTE_ROUTE_FAILURE != LOCAL_FALLBACK_PERMISSION
SUPERVISOR_WAKE != MODEL_POLLING_REQUIREMENT
```

Firstmate's shell/tmux/Herdr-specific mechanics are implementation references, not Golam's canonical runtime architecture.

## 3. no-mistakes — exact-head change qualification and publication safety

no-mistakes provides a strong local change-publication gate:

```text
working branch
-> explicit gate remote
-> admission hook
-> disposable worktree
-> intent
-> rebase
-> review
-> test
-> document
-> lint
-> guarded push
-> PR
-> CI
```

High-value patterns include:

- opt-in named gate rather than silently replacing `origin`;
- disposable worktree isolation;
- fixed minimum validation sequence that configuration can extend but not weaken/reorder;
- explicit `auto-fix` vs `ask-user` findings;
- exact-head-bound pipeline attestations;
- safe mechanical repair loops with bounded attempts;
- CI repairs that remain bound to guarded publication semantics;
- exact remote-head / force-with-lease checks;
- refusal to discard unknown private/upstream commits;
- recovery anchors preserving unpublished heads before cleanup;
- exact commit / patch / tree survival evidence rather than commit-message inference;
- daemon crash recovery and serialized same-branch runs;
- no blind assumption that a newer or different head inherits prior review;
- fork/parent routing without rewriting canonical `origin` semantics.

### 3.1 What Golam should reuse

Priority component/source shortlist:

```text
internal gate / reconciliation logic
disposable worktree lifecycle
review/test/document/lint/CI step contracts
finding/action model
auto-fix bounded-loop logic
CI repair exact-head/publication binding
remote data-loss guard
recovery-anchor preservation tests
exact-head pipeline attestation tests
```

Preferred reuse mode:

```text
PORT_TEST_PATTERNS + REIMPLEMENT_CANONICAL_CONTRACT
-> SELECTIVE_COPY pure Git/reconciliation helpers where measured
```

### 3.2 Gap exposed

Golam has strong verification/evidence contracts, but the program does not yet own one implementation-ready **software change publication gate** that binds:

- reviewed change identity;
- qualification evidence;
- repair rounds;
- publication target;
- remote-head lease;
- CI repair identity;
- preservation/recovery of unpublished heads.

This becomes **T252 — Verified Change Qualification / Publication / Repair Gate Contract**.

T252 is a development/change-delivery capability; it does not replace T149/T150/T216 verification generally and it does not authorize product merges by itself.

### 3.3 Non-adoptions

```text
PIPELINE_GREEN != MERGE_AUTHORIZATION
LOCAL_GATE_REMOTE != SOURCE_REPOSITORY_AUTHORITY
AUTO_FIX_FINDING != USER_INTENT_CHANGE_PERMISSION
CI_REPAIR != REVIEW_INHERITANCE
FORCE_WITH_LEASE != CONTENT_VERIFICATION
PATCH_ID_MATCH != FINAL_TREE_EQUIVALENCE
RECOVERY_ANCHOR != PUBLICATION_PROOF
CHANGE_GATE != GOLAM_EFFECT_GATE
```

Golam's repository governance, exact-head CI/review requirements, normal merge policy and human/owner approval remain authoritative.

## 4. Doop — live human/agent artifact collaboration reference

Doop is valuable primarily at the Experience / Artifact boundary, not as a control-plane donor.

The reviewed source demonstrates:

- human and agent edits streaming into one shared artifact workspace;
- live cursors/presence;
- per-artifact/frame editing indicators;
- comments pinned to exact elements;
- activity feed;
- explicit agent status and task panels;
- undo/redo;
- share/invite/access projection;
- MCP OAuth where the agent acts under the approving user's access;
- sandboxed iframe rendering of generated HTML frames;
- exemplar/decision memory plus a distiller that proposes durable style rules;
- local/self-hosted operation with optional provider integrations.

### 4.1 Golam placement

Doop should strengthen existing contracts rather than add a new control-plane task:

- **T185 Agent UI Projection** — add live presence, actor status, artifact edit/activity projections;
- **T193 Semantic Artifact Provider Family** — add sandboxed rendered preview and element-level diff/comment anchors;
- **T200 Product Command Center** — add live human/agent collaboration surfaces over canonical Tasks/Artifacts/Actions;
- **T214 Review Checkpoints** — comments/feedback bind exact artifact revision/element identity;
- **T232 Learning/Evolution** — distillation may create rule/style candidates only, never active policy;
- **T243 Deployment/Tenancy Boundary** — multi-human collaboration remains a separately qualified future tenancy mode; personal single-owner baseline is unchanged.

### 4.2 AGPL / permission firewall

The public repository is AGPL-3.0. The founder separately asserts permission to reuse supplied source code. Before copied Doop code enters Golam:

1. bind exact component/path/blob;
2. record the direct permission evidence/scope relied upon if Golam will not distribute that component under AGPL terms;
3. reconcile third-party/transitive licenses;
4. record NOTICE/source-availability obligations where applicable;
5. prefer behavior reimplementation when rights scope is unclear or the donor would unnecessarily pull an AGPL/network-server closure into Golam.

```text
DOOP_PUBLIC_AGPL != AUTOMATIC_GOLAM_LICENSE
FOUNDER_PERMISSION_ASSERTION != TRANSITIVE_RIGHTS_CLOSURE
LIVE_PRESENCE != PRINCIPAL_AUTHORITY
MCP_OAUTH_IDENTITY != EFFECT_AUTHORIZATION
ACTIVITY_FEED != CANONICAL_EVENT_LEDGER
SANDBOXED_IFRAME != TRUSTED_CONTENT
DISTILLED_STYLE_RULE != ACTIVE_POLICY
```

No T253 is created from Doop in this review.

## 5. Revalidated prior sources

The supplied repeated sources retain their prior disposition:

### classifier.dev

- remains a T246 batch-decision/evaluation/provider reference;
- hosted classifier paths are explicit remote egress;
- confidence never becomes target/Effect authority.

### jev_search

- remains a T246 two-phase narrowing behavior/source candidate;
- lexical priority is not exclusion authority;
- OpenRouter/API-key/global-threshold defaults are not Golam policy.

### unreal-agent

- remains a T247 donor/reference for stable input IDs, redelivery handling, pure tool translation, atomic OperationProposal persistence and context-omission evidence;
- donor session/operation stores do not become Golam Task/Effect authority.

No duplicate task is created for these three.

## 6. New task summary

```text
T251 Durable Supervision Event / Steering Inbox / Wake Contract
T252 Verified Change Qualification / Publication / Repair Gate Contract
```

No T253 is justified by this source set.

## 7. Cross-source synthesis

The combined source set sharpens Golam's execution chain:

```text
User / channel input
-> canonical InputEnvelope                    [T247]
-> Work Graph / Claim / Liveness              [T249]
-> Formation / delegation                     [T230]
-> durable supervisor steering / wake         [T251]
-> worker execution generation                [T205]
-> ToolTranslation / Operation / Effect       [T247/T199]
-> Evidence / Verification                    [T149/T150/T216]
-> optional software-change qualification     [T252]
-> publication / PR / CI evidence
-> Experience projections / live artifacts    [T185/T193/T200 + Doop patterns]
```

No donor owns canonical authority.

## 8. Planning disposition

```text
FIRSTMATE_SOURCE_REVIEWED=YES
NO_MISTAKES_SOURCE_REVIEWED=YES
DOOP_SOURCE_REVIEWED=YES
CLASSIFIER_DEV_REVALIDATED=YES
JEV_SEARCH_REVALIDATED=YES
UNREAL_AGENT_REVALIDATED=YES

NEW_TASKS=T251,T252
T253_REQUIRED=NO
NEW_PARALLEL_TASK_LEDGER=NO
NEW_PARALLEL_EFFECT_LEDGER=NO
NEW_PARALLEL_IDENTITY_AUTHORITY=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_SOURCE_COMPONENT_ADMITTED=NO
ACTIVE_SPEC_006_PR_24_WIDENED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
```
