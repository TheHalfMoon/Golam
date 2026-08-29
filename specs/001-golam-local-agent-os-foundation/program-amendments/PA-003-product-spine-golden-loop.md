# PA-003 — Product Spine, Golden Loop, Trust UX, and Release Sequencing

**Status**: PROPOSED_FOR_REVIEW  
**Date**: 2026-08-28  
**Founder input**: Perform a first-principles review of the combined Golam architecture after the Grok Bot, DeerFlow, OpenMausBot, Rakazo, Eino/Deer-Go, Mem0, LangGraph, LangChain, Qdrant, Firecrawl, Braintrust, LlamaIndex, Hermes Agent, autoresearch, DeepSeek Harness, OpenClaw, and OpenSandbox research waves.  
**Stacked planning base**: `plan/source-foundry-wave-002@5ca9ba3dda558f0ba8e08ef084a7706e0ab7350f`  
**Implementation authorization**: NONE. This amendment changes future product sequencing and acceptance gates only. It MUST NOT expand or reorder the active Spec 003 implementation package.

---

## 1. Executive decision

Golam MUST NOT try to win by accumulating the largest feature list.

The product moat is a **trustworthy universal action loop** that remains coherent across models, tools, computers, channels, workers, memory, and devices:

```text
INTENT
  -> TASK CONTRACT
  -> EVIDENCE
  -> PLAN / EXECUTION
  -> EFFECT GATE
  -> OBSERVE
  -> VERIFY
  -> RECOVER OR CONTINUE
  -> TRUST RECEIPT
  -> GOVERNED LEARNING
```

Every major surface in Golam exists to make this loop faster, safer, more capable, or more portable. A feature that does not improve this loop, a measured user workflow, or a release claim SHOULD NOT block an earlier useful release.

The program therefore adds an explicit **Golam Core Alpha gate after Spec 005**. Golam must become genuinely useful from CLI/TUI before Desktop, Mobile, large worker systems, broad parity, or ecosystem expansion can justify additional complexity.

---

## 2. Product thesis

Golam is not primarily:

- a chatbot;
- a coding agent;
- a browser agent;
- a memory product;
- a sandbox service;
- a multi-agent framework;
- a model launcher;
- a messaging bot;
- or a Grok Bot clone.

Golam is the user's **local-first personal action and intelligence operating layer**.

The user should be able to start on the terminal, continue on Desktop, approve from a phone, redirect through a channel, return the next day, switch models, recover after a crash, and still be interacting with the same durable task, evidence, authority, memory, and execution history.

The differentiator is not that every subsystem is novel. The differentiator is that the whole system shares one trustworthy semantic spine.

---

## 3. The Golden Loop

A release-worthy Golam task follows this lifecycle.

### 3.1 Capture intent

The user expresses a goal through any authenticated surface.

The surface is not the task. CLI, TUI, Desktop, Mobile, IDE, or a channel are projections over the same durable task semantics.

`SURFACE != TASK_IDENTITY`

### 3.2 Compile a Task Contract

Golam converts user intent into a durable, inspectable `TaskContract` containing at minimum:

- task identifier;
- goal;
- completion criteria;
- non-negotiable constraints;
- declared scope/resources;
- execution locality policy;
- allowed model/provider classes;
- initial capability ceiling;
- approval posture;
- budget constraints;
- notification/attention policy where applicable;
- expected deliverables;
- explicit stop conditions;
- unresolved ambiguities that materially affect safe execution.

The Task Contract is not model-authored authority. It is a user-intent projection compiled through Golam policy and protected-state rules.

`TASK_CONTRACT != CAPABILITY_LEASE`

### 3.3 Build evidence

The Context Compiler selects the cheapest sufficient authoritative evidence route under permission, freshness, locality, latency, and token constraints.

The task does not proceed from semantic similarity alone. Every important decision can retain evidence references, source authority, freshness, and taint.

### 3.4 Execute incrementally

The harness/worker runtime performs bounded steps. Long tasks remain interruptible and checkpointable.

Every consequential external effect uses the existing Effect Gate. Workflow progress never substitutes for external-effect durability.

### 3.5 Observe and verify

After actions, Golam observes the resulting environment and verifies the relevant success criteria.

A generated answer saying “done” is not verification.

### 3.6 Recover or continue

Failures are classified rather than flattened into generic retry:

- task-understanding failure;
- missing/ambiguous user intent;
- stale or insufficient evidence;
- provider/model failure;
- tool/runtime failure;
- permission/policy denial;
- environment blocker;
- external-effect unknown outcome;
- verification failure;
- resource/budget exhaustion;
- user interruption/takeover;
- crash/restart recovery.

The recovery path must respect the effect semantics and current authority state.

### 3.7 Produce a Trust Receipt

Every meaningful task completion, partial completion, or blocked stop SHOULD project a user-readable `TrustReceipt` derived from canonical records.

The receipt answers:

- What did Golam do?
- What changed?
- What evidence supports the result?
- Which files/artifacts were created or modified?
- Which external actions occurred?
- What data left the device?
- Which models/providers/tools were used?
- Which approvals or preauthorizations were consumed?
- What remains unresolved or unknown?
- What did Golam propose to learn from this task?

`TRUST_RECEIPT != AUTHORITY_RECORD`

The receipt is a projection over event/effect/evidence/audit state and cannot rewrite those records.

### 3.8 Govern learning

Task-derived memories, skills, routing changes, and behavior improvements enter the PA-002 learning proposal path. Successful task execution does not grant self-modification authority.

---

## 4. Canonical product entities

The architecture MUST distinguish the following concepts.

### 4.1 Task

A durable unit of user intent that can outlive a conversation, process, model, device, or execution attempt.

### 4.2 Session

An interaction/history projection. A task may span multiple sessions; one session may navigate multiple tasks where explicitly supported.

### 4.3 Run

One execution attempt or continuation against a task. Runs may crash, pause, fork, resume, or be superseded without destroying the task.

### 4.4 Worker

A bounded executor acting on a delegated subgoal with narrower authority. A worker is not the user's durable task identity.

### 4.5 Goal Ledger

The durable operational statement of goal, constraints, evidence, blockers, and next safe action. It is closely related to but not identical to the user-facing Task Contract.

Binding relationships:

```text
Task 1 --- N SessionProjection
Task 1 --- N Run
Run  1 --- N WorkerExecution
Task 1 --- 1 current GoalLedger
Task 1 --- N TrustReceiptProjection
```

`TASK != SESSION != RUN != WORKER`

This distinction is required before future multi-worker or multi-surface complexity.

---

## 5. Progressive autonomy

Golam should make autonomy understandable rather than exposing a confusing set of independent toggles.

The product SHOULD project a small number of understandable autonomy postures while the kernel continues to enforce fine-grained policy/capabilities underneath.

A representative ladder is:

1. **Observe** — read/analyze only.
2. **Suggest** — plan and prepare changes, no consequential execution.
3. **Act Locally** — bounded reversible/local work under current leases.
4. **Act With Policy** — policy-authorized bounded external/reversible effects.
5. **Ask for Consequences** — sensitive/irreversible/high-impact effects require fresh approval or bounded run preauthorization.

This is product UX, not a replacement authorization model.

A “full access” UX MUST NOT become a universal irreversible always-allow rule.

`AUTONOMY_POSTURE != AUTHORITY`

---

## 6. In-flight human control is a runtime primitive

Golam MUST support active intervention during execution, not only post-hoc review.

Future harness/client surfaces SHALL converge on the following semantic controls:

- `Pause` — stop scheduling new work at a safe boundary;
- `Stop` — cancel the task/run according to effect semantics;
- `Steer` — add user direction while preserving history;
- `AddConstraint` — narrow the current task contract/goal constraints;
- `ChangePriority` — reorder bounded planned work without silently dropping obligations;
- `Inspect` — show current goal, evidence, planned/active effects, blockers, budgets, and authority posture;
- `TakeOver` — transfer interactive computer/input authority to the human;
- `Resume` — continue after revalidating authority, live state, and stale plans.

Model/provider native mid-turn steering may optimize the implementation but cannot own the semantics.

`USER_STEERING_CAN_NARROW_BUT_NOT_SILENTLY_WIDEN_AUTHORITY`

---

## 7. Initiative and attention budget

Persistent agents create a new product problem: a system that is technically proactive can become unusable if it constantly interrupts the user.

Spec 008 MUST therefore distinguish **initiative authority** from **attention authority**.

Future `InitiativePolicy` / `AttentionBudget` semantics SHOULD cover:

- silent internal maintenance that has no external effect and no sensitive data movement;
- notify-only events;
- propose-action events;
- policy-authorized bounded unattended actions;
- explicit escalation conditions;
- quiet hours / channel routing where supported;
- deduplication and batching of repeated notifications;
- urgency/severity;
- user feedback such as mute, defer, lower priority, or never notify for this class.

Receiving a trigger does not automatically authorize an effect, and authorization to perform an effect does not automatically authorize unlimited notifications.

`INITIATIVE_AUTHORITY != ATTENTION_AUTHORITY`

---

## 8. Trust UX as a product moat

Golam's security model must be visible enough that normal users benefit from it without reading policy schemas.

Every major client should eventually make the following questions easy to answer:

- **What are you doing now?**
- **Why are you allowed to do it?**
- **What are you waiting for?**
- **What data will leave my device?**
- **Which model/provider is seeing this context?**
- **What changed on my computer/accounts?**
- **What evidence supports this answer?**
- **What did you learn about me/project?**
- **How do I undo, revoke, forget, or stop this?**

This leads to a user-facing Trust Center projection over:

- current devices and sessions;
- active capability leases;
- approvals/preauthorizations;
- connected channels/providers;
- current egress destinations;
- secrets/credential bindings without plaintext;
- sandbox posture;
- memory candidates/promotions;
- scheduled/proactive work;
- security-audit findings;
- recent Trust Receipts.

Protected state remains kernel-owned; the Trust Center is an authenticated projection and control surface.

---

## 9. `golam doctor` and `golam security audit`

OpenClaw's current operational-security UX validates a missing Golam product requirement.

Golam SHOULD expose two distinct commands/surfaces:

### `golam doctor`

Operational diagnostics:

- daemon/IPC health;
- model/provider availability;
- sandbox/runtime compatibility;
- local database/index state;
- device/channel connectivity;
- stale migrations;
- dependency/runtime prerequisites;
- capability truth mismatches;
- recoverable configuration problems.

### `golam security audit`

Security posture analysis:

- unexpected network exposure;
- weak/unexpected client enrollment;
- broad capability grants;
- stale approvals/preauthorizations;
- secret-broker bypass paths;
- unsafe plugin/skill/MCP profiles;
- sandbox disabled or degraded from expected profile;
- dangerous mounts/host control paths;
- channel identity/approval configuration hazards;
- strict-local violations;
- unsigned/unqualified extension state;
- protection/audit integrity anomalies.

Supported modes SHOULD include human-readable, machine-readable, deep/live probes, and narrowly safe remediation where deterministic.

An audit result is evidence, not proof that the system is secure.

`SECURITY_AUDIT_FINDING != SECURITY_PROOF`

---

## 10. User model is separate from general memory

OpenClaw's explicit `USER.md` layer reinforces a useful distinction.

Golam SHOULD maintain a governed, compact `UserModel` projection for stable preferences and interaction constraints, separate from general episodic/project knowledge.

The UserModel MAY contain:

- communication/style preferences;
- stable tool/model preferences;
- approved recurring workflow preferences;
- accessibility/preferences relevant to UX;
- explicit user-set behavioral constraints.

It MUST retain provenance and supersession metadata and MUST NOT be a hidden behavioral profile built from unrestricted inference.

Sensitive inferred traits MUST NOT be silently promoted into a durable user model.

`USER_MODEL != ALL_USER_MEMORY`

---

## 11. Migration and portability as adoption infrastructure

A local-first product should make switching **into and out of Golam** easy.

Spec 005 SHOULD include a quarantined import pipeline for relevant portable artifacts from tools such as OpenClaw, Hermes, Claude Code, Codex, and other supported assistants where exact formats are safely detected.

Imports MUST:

- never import credentials/secrets by default;
- preserve source/provenance;
- stage imported memory separately;
- never auto-promote third-party memory into active canonical user/project truth;
- show conflicts before promotion;
- allow rollback/removal;
- preserve original source files.

Exports SHOULD support user-owned Markdown plus machine-readable task/evidence/receipt formats where appropriate.

Portability is a growth feature and a trust feature.

---

## 12. Locality is an explicit task policy

Golam should not reduce local/cloud decisions to model selection alone.

The product SHOULD support an understandable locality policy such as:

- `STRICT_LOCAL` — no external provider/channel/push/remote egress except explicitly qualified local/LAN paths allowed by policy;
- `LOCAL_PREFERRED` — start locally and ask/route only when a permitted escalation is required;
- `CLOUD_ALLOWED` — eligible configured cloud providers may be used under data/egress policy;
- `REMOTE_EXECUTION_ALLOWED` — authorized remote computer/sandbox providers may execute work under bounded leases.

The exact kernel representation may be more granular. The user-facing posture must never permit hidden locality downgrade.

---

## 13. Capability Truth Matrix

Every model, tool provider, sandbox provider, platform, channel, and device adapter SHOULD publish a machine-readable capability descriptor and conformance state.

The UI/harness MUST NOT advertise or plan around unsupported capabilities as if they existed.

Examples include:

- model: vision, native tool calls, schema mode, context, local/cloud, streaming, steer support;
- sandbox: pause/resume, snapshot, PTY, browser, desktop, GPU, egress enforcement, credential brokerage;
- desktop: accessibility, input injection, secure desktop restrictions;
- channel: edits, threads, reactions, attachments, identity guarantees, delivery semantics;
- mobile/device: background execution, push, camera/mic, remote control, secure key storage.

`DECLARED_CAPABILITY + CONFORMANCE_EVIDENCE -> CLAIMED_CAPABILITY`

A declaration without evidence is not enough for release claims.

---

## 14. Early product release gate: Golam Core Alpha

The current program sequence is technically sound but risks delaying real product learning until too many subsystems exist.

After Spec 005 closes canonically, and before Spec 006 complexity becomes a release blocker, Golam MUST pass a **Core Alpha product gate** through CLI/TUI.

Core Alpha does not require Desktop, Mobile, broad channels, workers/groups, or Grok parity.

It must prove one coherent local-first product loop across representative workflows.

### Required Core Alpha scenarios

1. **Repository task**
   - inspect a real local repository;
   - form/retain a goal;
   - edit through governed tools;
   - run deterministic verification;
   - report exact evidence and changed artifacts.

2. **Research/evidence task**
   - retrieve permitted web/local evidence;
   - preserve source provenance and taint;
   - distinguish evidence from authority;
   - produce an attributable artifact/answer.

3. **Filesystem/document task**
   - inspect and transform local files;
   - preserve source files unless authorized to change them;
   - show resulting artifact and Trust Receipt.

4. **Cross-session memory task**
   - persist approved memory;
   - restart the daemon/new session;
   - retrieve the right memory under correct scope;
   - handle stale/conflicting live state correctly.

5. **Interrupt/recovery task**
   - pause/steer/stop/resume a non-trivial run;
   - survive process restart;
   - avoid duplicate protected effects;
   - preserve task/goal/evidence continuity.

6. **Strict-local task**
   - perform a useful end-to-end task with external network egress disabled;
   - prove externally that no hidden cloud/model/vector/eval/telemetry fallback occurred.

### Core Alpha product evidence

Measure and report at least:

- time to first useful action;
- task completion and partial-completion quality;
- false-success/verification failures;
- interruptions/recovery success;
- approval count and approval repetition;
- context/retrieval token use where applicable;
- local resource usage;
- model/provider switches/fallbacks;
- user-visible unresolved/unknown outcomes;
- data-egress events;
- memory retrieval/promotion correctness.

Exact thresholds belong to the owning release package after baseline measurement. PA-003 intentionally does not invent target numbers without evidence.

---

## 15. Release ladder

The program SHOULD optimize for these product checkpoints rather than waiting for one giant final release.

### Gate A — Trusted Spine

Specs 002–003: durable kernel/effects + identity/policy/secrets/sandbox foundations.

### Gate B — Intelligence Spine

Spec 004: model-independent harness, local intelligence, interrupt/resume, provider truth.

### Gate C — **Golam Core Alpha**

Spec 005: useful CLI/TUI product with local tools, context, governed memory, evidence, Trust Receipts, portability baseline, and strict-local proof.

### Gate D — Computer Product

Spec 006: first-class Desktop/computer-control experience with human takeover.

### Gate E — Everywhere Golam

Spec 007: native mobile + secure remote GolamConnect + voice + official channels.

### Gate F — Persistent Team

Spec 008: durable workers, routines, initiative/attention policy, learning and bounded experiments.

### Gate G — Public Parity / Superset

Spec 009: close public Grok Bot MUST-MATCH domains and document deliberate supersets.

### Gate H — Release Qualification

Spec 010: GolamBench, long-horizon/hybrid-interface verification, security/recovery/platform matrix and exact-head release evidence.

---

## 16. Benchmark implications

Recent long-horizon agent research reinforces three Golam decisions:

1. **Outcome-only scoring is insufficient.** Hybrid GUI/CLI/code tasks expose fabricated or shortcut success that trajectory-aware judges catch.
2. **Process and outcome verification are different signals.** GolamBench should score both and distinguish controllable agent failures from external blockers.
3. **Human steering materially matters.** Planning/runtime APIs should expose intervention rather than requiring a full stop/restart to correct direction.

Future Spec 010 research SHOULD re-pin and qualify then-current sources such as:

- WeaveBench (long-horizon hybrid GUI/CLI/code evaluation);
- Universal Verifier / CUAVerifierBench process-vs-outcome verifier research;
- LongCLI-Bench or equivalent long-horizon CLI engineering benchmarks;
- other exact-head, rights-qualified benchmark artifacts relevant at implementation time.

These are benchmark/research targets, not implicit dependency admissions.

---

## 17. Explicit simplifications

To protect time-to-useful-product, the following MUST NOT become Core Alpha blockers unless a bounded owning spec produces measured evidence that they are necessary:

- mandatory graph database;
- large swarm architecture;
- native mobile;
- broad messaging provider matrix;
- custom relay infrastructure;
- cloud control plane;
- universal hosted observability;
- remote vector database;
- autonomous always-on microphone;
- generalized A2A federation;
- image/video generation;
- every donor framework compatibility adapter;
- marketplace/discovery ecosystem;
- arbitrary self-modification.

The architecture may preserve clean seams for these capabilities without implementing them early.

---

## 18. New binding invariants

```text
SURFACE != TASK_IDENTITY
TASK != SESSION != RUN != WORKER
TASK_CONTRACT != CAPABILITY_LEASE
AUTONOMY_POSTURE != AUTHORITY
INITIATIVE_AUTHORITY != ATTENTION_AUTHORITY
TRUST_RECEIPT != AUTHORITY_RECORD
USER_MODEL != ALL_USER_MEMORY
SECURITY_AUDIT_FINDING != SECURITY_PROOF
USER_STEERING_CAN_NARROW_BUT_NOT_SILENTLY_WIDEN_AUTHORITY
DECLARED_CAPABILITY + CONFORMANCE_EVIDENCE -> CLAIMED_CAPABILITY
```

Existing constitutional and earlier amendment invariants remain stronger where applicable.

---

## 19. Owning-spec changes

### Spec 004

Must establish explicit Task/Run/Interrupt/Continuation runtime semantics, provider capability truth, and in-flight steering primitives sufficient for Core Alpha.

### Spec 005

Must deliver the Core Alpha product gate, Trust Receipt projection, governed UserModel baseline, portability/import staging, and usable local CLI/TUI workflows before Desktop becomes the next product blocker.

### Spec 006

Must project the same Task/Run/Trust semantics into Desktop and provide human takeover/inspect/steer without introducing a separate desktop-only agent state.

### Spec 007

Must preserve task continuity across Native Mobile/channels and keep phone/channel identity distinct from task/authority identity.

### Spec 008

Must add InitiativePolicy/AttentionBudget semantics before proactive workers/routines are considered product-complete.

### Spec 009

Parity work should prefer Golam `VERIFIED_SUPERSET` where Trust Receipts, user-owned governed memory, in-flight control, locality guarantees, or security posture are stronger.

### Spec 010

Must evaluate the Golden Loop, trajectory/process correctness, false-success rate, hybrid-interface tasks, Trust Receipt completeness, human steering/recovery, and the release ladder's exact claimed surfaces.

---

## 20. Final product principle

The shortest statement of the intended product is:

> **One Golam. One durable task spine. Any model. Any surface. Any computer. User-owned memory. Explicit authority. Verifiable actions.**

Feature breadth is valuable only after this sentence is true in practice.
