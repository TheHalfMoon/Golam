# Golam Program Superiority Roadmap — 2026

**Status**: PROGRAM-LEVEL PROPOSAL — NO PRODUCT IMPLEMENTATION AUTHORITY

**Canonical base reviewed**: `main@c85b4b8f0d6ffccb039645803542d75b3bd47f29`

**Purpose**: strengthen the frozen Spec 001 program so Golam can exceed current persistent-agent, coding-agent, desktop-agent and automation alternatives without weakening the Golam Constitution, current Spec 006 scope, or Source Foundry discipline.

This document is an overlay on the existing Spec 001 architecture. It does not reopen completed Specs 002–005, does not widen active Spec 006 implementation authority, and does not authorize implementation from this program file alone. Every new product unit still requires a bounded Spec Kit lifecycle and exact Source Foundry admission for every new dependency/runtime/source.

## 1. Competitive objective

Golam should not compete by copying the largest feature list. It should combine the strongest observed product properties of current alternatives with a materially stronger trust and evidence model.

Target product thesis:

> **The user-owned Agent OS that can work for hours or days across code, browser, apps, files and devices; learn repeatable work; collaborate through isolated workers; survive crashes; explain every consequential action; and remain locally authoritative even when optional cloud models or services are used.**

Golam should aim for the following combination:

- Grok Bot-class persistent named workers, persistent computers/workspaces, cross-device conversation continuity, parallel workers, handoffs, routines, and demonstration-derived workflows;
- Hermes-class skill learning, progressive-disclosure skills, session recall, schedules, messaging gateways, provider portability and multi-backend execution;
- Codex-class Rust-heavy local client, explicit sandbox/approval configuration, multi-agent session semantics, skills/MCP integration, and tight developer workflow;
- OpenHands-class multi-backend control surface and explicit separation between UI and execution backends;
- Browser-use/OpenClaw-class dedicated browser operation, while preserving Golam's stronger semantic-route ordering and authority boundaries;
- SWE-agent-class simple agent-computer interfaces and benchmarkability;
- Golam-only strengths: local canonical ownership, protected authority kernel, durable Effect Gate, `UNKNOWN_OUTCOME` reconciliation, strict-local hard denial, governed memory, provenance/taint, visible computer control, exact source admission and evidence-first release claims.

A competitor feature is not automatically a Golam requirement. It becomes a requirement only when it improves user value and can be reconciled with the Constitution.

## 2. Current source snapshots for comparison

The following states are research/reference anchors only. They are not dependency admission or permission to copy code.

| Source | Reviewed state | What Golam should learn | What Golam must not inherit blindly |
| --- | --- | --- | --- |
| `NousResearch/hermes-agent` | `03f3b09222b8f03becb203a6ebb9bac1f927b8b6` | learning loop, progressive-disclosure skills, session search, scheduling, multi-channel gateway, subagent lifecycle, portable execution backends | cloud/provider assumptions, model-owned authority, uncontrolled autonomous skill mutation |
| `openai/codex` | `4f1a2bb5ffed8fd28518925c1d4085ead158e304` | Rust local agent architecture, sandbox/approval configuration, multi-agent runtime/session metadata, MCP/skills/product surfaces | provider coupling or any semantics that bypass Golam's protected kernel/effect model |
| `OpenHands/OpenHands` | `f7fb0c4b21f5ed726edbba8a6309634ef434b004` | control-surface/backend separation, local/VM/cloud execution topology, automation backend separation | REST/host boundaries as authority merely because they are reachable |
| `redhat-et/ripwire` | `2848e64c16ea09022579d8862d6fe35dd9a1a58b` | deterministic local repository graph/context packing, blast-radius/test selection, token-budgeted context | C++ core as privileged dependency; graph output as truth or authority |
| Grok Bot public product | public 2026-08 docs; no internal source authority | persistent named teammates, shared durable computer state, parallel workers, handoffs, demonstration-to-routine UX, approval-driven unattended work | shared computer as an isolation boundary, cloud storage as mandatory trust root, product claims as implementation evidence |

Additional benchmark/reference candidates are listed below and require exact pinning in their owning spec before qualification use.

## 3. Differentiation pillars

### 3.1 Owner-held authority

Golam's local canonical state and protected kernel remain the primary differentiator. Optional cloud models, relays, connectors, browsers or workers may contribute intelligence or execution capacity but may not become the only copy of authority, goals, approvals, memory governance or effect truth.

Required product property:

```text
OPTIONAL_SERVICE_LOSS != LOSS_OF_CANONICAL_USER_STATE
MODEL_PROVIDER != AUTHORITY_ROOT
WORKER != SELF_EXPANDING_AUTHORITY
```

### 3.2 Verifiable autonomy

Long-running work must be more trustworthy than competitors, not merely longer.

Every long-running goal should expose:

- immutable goal/non-negotiable constraints;
- current plan and dependency graph;
- evidence/proof obligations;
- budget/resource state;
- durable blockers and `UNKNOWN_OUTCOME` effects;
- what changed since the user last looked;
- next safe action;
- confidence/verification class for completion claims.

A worker may report `COMPLETE` only when required verification obligations are satisfied or the user explicitly accepts a documented unverified result.

### 3.3 Capability fabric instead of tool sprawl

Golam should converge all action surfaces into a typed capability fabric:

```text
domain/app API
-> native app/OS automation API
-> accessibility/semantic tree
-> browser DOM/protocol
-> deterministic input
-> bounded vision/pixel candidate
```

Introduce a kernel-owned `ControlRouteProviderRegistry` and `RouteApplicabilityEvidence` contract. Providers report applicability, availability, freshness, exact target identity, supported operations and failure class. Providers cannot grant themselves authority. Unknown/stale stronger-route state does not silently authorize a weaker route.

This closes the current cross-spec ownership gap between browser/application routes and desktop raw fallback.

### 3.4 Safe learning loop

Golam should exceed autonomous skill-learning systems by separating **learning** from **activation**.

Proposed lifecycle:

```text
trajectory/evidence
-> repeated-pattern detector
-> SkillCandidate / RoutineCandidate
-> provenance + taint + authority analysis
-> deterministic replay/simulation where possible
-> test generation
-> user or policy approval
-> immutable versioned activation
-> bounded production use
-> outcome evidence
-> candidate improvement
```

Rules:

- model-generated skill/routine content is untrusted candidate data;
- an active skill cannot rewrite itself in place;
- every improvement creates a new version with exact diff/provenance;
- authority never grows merely because a learned workflow previously succeeded;
- stale/revoked skill versions invalidate queued/cached authority;
- destructive or external-publication routines require explicit approval classes even after repeated success.

### 3.5 Demonstration-to-routine compiler

Add a later bounded feature for teaching Golam by demonstration without treating screen recording as executable truth.

Pipeline:

```text
human demonstration
-> local event/semantic observation
-> redact/separate secret material
-> infer candidate steps
-> map each step to strongest stable capability route
-> identify variables/preconditions/postconditions
-> generate RoutineCandidate
-> dry-run/simulation
-> user review
-> activate with explicit schedule/trigger authority
```

Prefer semantic actions learned from demonstrations. Recorded coordinates are hints only and cannot become durable authority.

### 3.6 Secure worker collaboration

Grok Bot-style multiple persistent workers are valuable, but shared state must not be mistaken for isolation.

Golam worker topology should support:

- one durable named worker identity per role;
- separate narrow child capability leases;
- separate worktree/workspace by default for coding workers;
- explicit shared project artifacts rather than ambient shared credentials;
- typed handoff bundles carrying goal, evidence, constraints, unresolved questions and bounded artifact refs;
- conflict detection for concurrent writes;
- join/cancel/crash-adopt semantics;
- optional collaborative group threads without collapsing authority identities;
- worker-specific memory views with explicit promotion to shared project memory.

### 3.7 Best-in-class repository intelligence

Keep the current L0-first policy but formalize a measured evaluation ladder:

```text
L0 bounded text/git
-> structural search / AST
-> LSP / semantic index
-> Ripwire-style deterministic graph/context
-> heavier dataflow/runtime/vector methods only when measured need remains
```

For every promotion, measure:

- task success delta;
- changed-file recall/precision;
- test-selection recall;
- token reduction;
- index latency and peak memory;
- deterministic-output stability;
- stale-index failure behavior;
- false-authority risk.

Ripwire remains optional sidecar/MCP/selective-port candidate, never privileged authority.

### 3.8 Browser/application semantic control

Create a bounded successor unit that owns the missing browser/application route before broad release of weaker desktop fallbacks.

Preferred standards/evidence:

- W3C WebDriver + WebDriver BiDi for standardized browser semantic/protocol control;
- browser-native/CDP support only behind a Golam-owned provider contract and exact browser/version qualification;
- BrowserGym/WebArena-Verified/WorkArena-style benchmark tasks for browser evaluation;
- dedicated agent browser profiles by default rather than silently taking control of a user's personal browser session;
- personal-session attachment only as a separate explicit capability with visible control and credential-disclosure rules.

Browser login/session state remains data/secret state, not authority.

### 3.9 Trusted time and automation semantics

Spec 008 must introduce trusted scheduler time semantics instead of relying on cron strings alone.

Required cases:

- wall-clock timezone identity;
- monotonic elapsed time inside a boot/session where applicable;
- DST fold/gap policy;
- clock rollback/large jump detection;
- system sleep/suspend/resume;
- missed-run catch-up window;
- overlap/concurrency policy;
- deduplication identity;
- stale-source/no-data policy;
- user-presence and unattended-risk policy;
- model/profile/skill/version drift between scheduled runs.

Every schedule-triggered consequential effect still uses ordinary current authority and Effect Gate semantics.

### 3.10 Resource Governor

Add a Golam-owned unprivileged resource governor for models, workers, browser sessions, capture and tool processes.

It should model:

- CPU/GPU/memory reservations and observed pressure;
- model residency and eviction priority;
- worker concurrency budgets;
- disk/artifact quotas;
- battery/thermal state where available;
- bandwidth/network budget for non-strict profiles;
- foreground interactive latency priority;
- graceful degradation and cancellation order;
- resource evidence in benchmark/receipts.

Resource pressure may reduce capability or delay work but cannot weaken security gates.

### 3.11 Connector and credential broker

Move beyond ad-hoc connector tokens toward explicit connector identities and brokered credentials.

Each connector requires:

- stable provider/account identity;
- exact granted scopes;
- local secret handle;
- OAuth/session refresh evidence;
- origin/destination binding;
- revocation state;
- read/write capability separation;
- approval class for consequential mutations;
- taint/provenance on returned data;
- no automatic cross-connector credential forwarding.

Browser credentials and connector credentials remain separate authority domains.

### 3.12 Release trust chain

Supply-chain integrity must become a product feature before stable release.

Required release pipeline:

- exact dependency/source policy (`cargo vet`/equivalent audit evidence, advisories, license/source bans);
- exact action/workflow pinning policy for security-critical release jobs;
- reproducible or independently comparable release builds where feasible;
- SBOM for shipped artifacts;
- signed build provenance/artifact attestations;
- platform signing/notarization requirements;
- Tauri updater signature enforcement if updater is admitted;
- update key generation/storage/rotation/recovery procedure;
- rollback/anti-downgrade policy for authority schema and signed binaries;
- shipped third-party notices/source obligations;
- release artifact hashes bound into the release evidence ledger.

Dependency lockfiles are resolution evidence, not the entire trust claim.

### 3.13 Backup, restore and portability

Add product-wide backup/restore/export instead of treating recovery as individual database behavior.

Backup design must distinguish:

- canonical human memory;
- authority database metadata;
- encrypted secret material/keys;
- client/device identities;
- model/cache artifacts;
- optional browser/app session material;
- derived indexes that can be rebuilt.

Restore requires schema/version validation, integrity verification, anti-replay handling for authority state, explicit device/secret posture, and a dry-run report before destructive replacement.

### 3.14 Local diagnostics and explainability

Add a local-only diagnostics surface that does not expose secrets.

`golam doctor` should eventually report:

- canonical schema/integrity status;
- client/device enrollment status;
- current strict-local and egress posture;
- model/backend availability;
- sandbox/platform capability matrix;
- pending `UNKNOWN_OUTCOME` effects;
- stale/failed schedules;
- memory reconciliation conflicts;
- worker/resource state;
- platform permission state;
- recent bounded error classes.

A diagnostic support bundle must be explicitly generated, redacted, content-bounded and never silently uploaded.

## 4. Program roadmap amendment

### Active Spec 006

Do **not** widen PR #24 with unrelated superiority work. Finish the existing bounded Desktop Computer Control scope under its current task graph.

Before final broad release of raw/vision fallback, ensure the cross-spec stronger-route contract exists so browser/application applicability cannot be silently assumed false.

Correct the current planning-language mismatch that says raw screenshot OCR is owned by Spec 007. Spec 007 is GolamConnect. OCR/visual semantic extraction belongs to a future bounded **Visual Semantics** unit whose number is assigned only after canonical successor authorization.

### Spec 007 — GolamConnect

Keep the current purpose. Strengthen it with:

- explicit metadata/privacy matrix for direct vs relay paths;
- device-loss and key-rotation recovery;
- user-verifiable remote-control indicator parity with local desktop control;
- reconnect resumption that never revives stale effect/control generations;
- file-transfer content hash + quarantine/scanning hook contracts;
- cross-device handoff bundles rather than implicit shared authority.

### Spec 008 — Workers & Automations

Expand mandatory scope to include:

- trusted time model and missed/overlap/catch-up semantics;
- durable worker identities and isolated workspaces;
- typed handoff bundles;
- Resource Governor integration;
- per-worker memory views and explicit shared-memory promotion;
- schedule trigger version binding to exact skill/model/profile/connector state;
- no-agent deterministic automation mode where an LLM is unnecessary;
- outcome-driven SkillCandidate/RoutineCandidate generation, but activation stays governed;
- demonstration-to-routine compiler late in the spec after deterministic scheduler reliability.

### Spec 009 — Superset Product Capabilities

Rename the product goal conceptually from **parity** to **verified superset**. Public Grok behavior remains a floor, not the architecture target.

Add measurable domains:

- browser/application semantic provider;
- first-class deep research with evidence graph/citation provenance;
- office artifact generation/editing for documents, presentations, spreadsheets and PDFs;
- connector broker and account-scoped capability management;
- persistent named worker UX and groups/handoffs;
- routine authoring/testing/history/rollback;
- local project/user memory inspection, conflict resolution and forget/export UX;
- optional voice interaction plan with local-first STT/TTS candidates, while real-time voice may ship in a later bounded unit if qualification would delay the trusted core;
- mobile/remote companion UX over GolamConnect, without moving the authority root to a cloud service.

### Spec 010 — GolamBench & Release Qualification

Spec 010 is an aggregator, not the first place safety/reliability defects are discovered.

Mandatory benchmark families:

- deterministic Golam hard gates;
- OSWorld V2 exact version for desktop tasks;
- WindowsAgentArena subset where supportable;
- BrowserGym/WebArena-Verified/WorkArena-style browser tasks;
- SWE-bench Verified/Pro or equivalent coding benchmark with exact harness disclosure;
- LongMemEval-V2 plus bounded LoCoMo/BEAM-style memory cases;
- crash/restart/fault injection;
- strict-local externally observed no-egress;
- connector/credential redirection and impersonation;
- long-running premature-stop/goal-drift tests;
- multi-worker conflict/handoff/recovery tests;
- real-device platform matrix for TCC/UIA/Wayland/interactive desktop behavior;
- release artifact signing/update/restore drills.

Hosted CI and real-device qualification are separate evidence classes.

## 5. Hard superiority metrics

The following are release gates, not weighted scores that can compensate for each other:

```text
UNAUTHORIZED_EFFECT_COUNT=0
BLIND_RETRY_OF_AT_MOST_ONCE_OR_IRREVERSIBLE=0
STRICT_LOCAL_UNEXPECTED_EGRESS=0
PROTECTED_AUTHORITY_FORGERY_SUCCESS=0
SECRET_PLAINTEXT_IN_ORDINARY_MODEL_VISIBLE_LEDGER=0
SILENT_CONTROL_WITHOUT_REQUIRED_VISIBLE_CHANNEL=0
STALE_CONTROL_GENERATION_ACCEPTED=0
UNVERIFIED_COMPLETION_REPORTED_AS_VERIFIED=0
```

Comparative metrics should include:

- task success;
- verified task success;
- user intervention rate;
- unnecessary approval rate;
- human takeover latency;
- recovery success after process/host interruption;
- duplicate-effect rate;
- wrong-target action rate;
- memory stale/false recall rate;
- contradiction surfacing quality;
- context tokens and latency per successful task;
- model cost per verified successful task for optional cloud profiles;
- local resource use;
- browser/desktop route strength used;
- schedule reliability and duplicate/missed-run correctness;
- worker handoff loss/conflict rate.

A leaderboard improvement is not sufficient if a hard gate regresses.

## 6. External benchmark/source candidates

Owning specs must pin exact revisions/releases before use.

- W3C WebDriver and WebDriver BiDi — browser semantic/protocol architecture reference.
- BrowserGym, WebArena-Verified, VisualWebArena, WorkArena — browser evaluation candidates.
- OSWorld V2 — desktop/computer benchmark candidate; pin code/tasks/assets/websites as one versioned fixture.
- WindowsAgentArena — Windows-native task benchmark candidate.
- LongMemEval-V2 — long-horizon agentic memory benchmark candidate.
- SWE-bench / SWE-agent / SWE-ReX — coding evaluation and sandbox/ACI references.
- `NousResearch/hermes-agent` — learning/scheduler/skills/gateway/worker behavioral reference.
- `openai/codex` — local Rust client, sandbox/approval, multi-agent/session and developer UX reference.
- `OpenHands/OpenHands` and its SDK/server architecture — backend/control-surface reference.
- `block/goose` — local extensible MCP/model portability reference.
- `openclaw/openclaw` — dedicated agent browser/profile and local gateway behavioral reference.
- `browser-use/browser-use` — browser-agent behavior and benchmark reference.
- `redhat-et/ripwire` — repository-intelligence candidate already tracked separately.
- `cargo-vet`, `cargo-deny`, RustSec and GitHub artifact attestations/SLSA guidance — supply-chain qualification candidates/reference.
- Tauri updater/signing documentation — desktop release/update trust reference.

No source above receives automatic architecture authority or code admission.

## 7. Product experience principles

Golam should be easier to use than security-heavy architecture normally implies.

Required UX direction:

- one-step local onboarding that starts with least privilege;
- capability requests explained in human language with exact scope and duration;
- persistent named workers with clear state, ownership and current activity;
- visible timeline of goal progress/evidence/blockers;
- immediate steering, pause and takeover;
- routine test mode before unattended activation;
- actionable explanations for denied/unsupported operations;
- clear local/cloud/privacy indicator per active execution profile;
- easy model/backend switching without losing canonical state;
- inspectable memory and skill/routine versions;
- portable export/backup;
- recovery UI that tells the truth about uncertainty instead of silently resetting.

## 8. Architecture constraints that must not be traded away for parity

Do not weaken these to match a competitor:

- no mandatory cloud authority or hidden cloud fallback;
- no model-owned authorization;
- no worker self-expansion of authority;
- no shared worker environment presented as a security boundary;
- no blind effect retry after ambiguous side effects;
- no automatic memory promotion without governance;
- no self-modifying active skills without versioned review/activation;
- no browser/desktop login session interpreted as authorization to perform every action;
- no hidden computer control or lost human takeover;
- no generic filesystem/process access to protected authority state;
- no benchmark claim without exact reproducible evidence.

## 9. Entry and exit rules for this overlay

This overlay may become canonical only through its own planning/governance PR. It does not alter active Spec 006 implementation authority until merged and then explicitly consumed by an owning successor spec.

Before merge:

1. reconcile this overlay with Spec 001 `plan.md`, `tasks.md` and root governance;
2. ensure it creates no contradictory authority for active PR #24;
3. run exact-head repository CI if the branch touches executable/check-consumed artifacts;
4. obtain fresh independent architecture/security/governance review;
5. repair every material finding forward-only;
6. guarded merge only on the unchanged qualified head;
7. verify canonical `main` afterward.
