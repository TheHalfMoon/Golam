# Tasks: Golam Local Agent OS Foundation Program

**Spec**: 001 — Golam Local Agent OS Foundation  
**Generated**: 2026-08-24 after GLM-5.3 reconciliation and cross-artifact analysis  
**Authority**: PROGRAM ORCHESTRATION ONLY — this file does not authorize implementing the entire Golam product or skipping later Spec Kit gates.

## Task execution rule

Every implementation feature below MUST create its own Spec Kit package and complete:

`specify -> clarify -> research as needed -> plan -> data model/contracts -> checklist -> tasks -> analyze -> implement -> converge`

before that feature's product code is authorized.

No later feature may infer authority from this program task graph alone.

**Additive program amendments**:

- `PA-001-phone-channel-access.md` expands future Spec 007 to first-class native mobile, voice and governed channel access.
- `PA-002-memory-retrieval-learning-evals.md` strengthens future Specs 004/005/008/009/010 for governed memory candidates, multi-route context/retrieval, learning proposals, bounded autonomous experiments, provider conformance and trajectory-level evaluation.

Neither amendment changes the currently authorized bounded implementation scope by itself.

---

## Phase 0 — Close and preserve Spec 001

- [x] **T001** Establish clean Golam repository separated from Golam-Research.
- [x] **T002** Ratify local-first/Rust-first constitution.
- [x] **T003** Complete final donor/gap research and Source Foundry rules.
- [x] **T004** Define product spec, plan, data model, core contracts, Grok parity strategy and bounded follow-on sequence.
- [x] **T005** Obtain GLM-5.3 independent architecture review.
- [x] **T006** Reconcile BLK-001/BLK-002 and MAJ-001..008 with no founder waivers.
- [x] **T007** Add kernel boundary, authenticated IPC, effect handler, taint, memory governance, stable channel binding, sandbox, approvals, ledger/replay, worker supervision and egress contracts.
- [x] **T008** Run post-GLM cross-artifact consistency analysis with zero unresolved blocker/major findings.
- [x] **T009** Founder reviewed Draft PR #1, attested permission for the researched source universe, upgraded Golam-Research to high-value implementation evidence/authorized-source candidate, and explicitly approved merge/freeze of Spec 001 into `main`.

**Exit gate**: merge PR #1 at the exact reviewed head, then re-read exact live `main` before the next feature branch is created.

---

## Phase 1 — Spec 002: Kernel & Durable Session Spine

**Goal**: prove the smallest useful authority/durability spine before model/tool complexity.

- [ ] **T010** After PR #1 is merged, verify exact live `main` and create `spec/002-kernel-durable-session-spine` from that exact commit only.
- [ ] **T011** Start Spec 002 research by qualifying the relevant `Golam-Research` Grok Bot 0.18 runtime slices (`source/electron-main`, `source/host`, `source/node-agent-coordinator`, `source/shared`, protocol/tests) at exact head as high-value implementation evidence/authorized-source candidates. Record founder permission scope/evidence for any bounded component proposed for reuse. Mine behavior/contracts seriously; do not assume reconstructed names equal original upstream names.
- [ ] **T012** Write Spec 002 requirements for the <=8 initial Rust crate/binary boundary; prohibit empty crate scaffolding.
- [ ] **T013** Define the process-splittable privileged-kernel API and sealed/unforgeable authority types.
- [ ] **T014** Define authenticated local IPC implementation choices per platform and delegated client enrollment.
- [ ] **T015** Define versioned event/session/fork/goal ledger schemas, security-critical integrity chaining, global audit ordering, checkpoints and content-addressed artifact lifecycle.
- [ ] **T016** Define effect handler/executor/reconciler API, durable intent-before-dispatch and UNKNOWN_OUTCOME behavior for all five semantics classes.
- [ ] **T017** Define bootstrap `Authorize(principal, action, resource, context)` with deny-by-default semantics that Spec 003 can replace with Cedar without semantic rewrite.
- [ ] **T018** Define protected kernel state storage boundary and prove generic tools cannot mutate it.
- [ ] **T019** Add Spec 002 benchmark exit gates: BS-1 durability/fork/replay, BS-2 duplicate-effect prevention, BS-10 strict-local no-egress skeleton, kernel-boundary compromise probe and unauthenticated-local-client probe.
- [ ] **T020** Complete Spec 002 checklist/tasks/analyze before any Rust implementation.

**Required implementation evidence before Spec 002 closes**: crash fault injection at every effect transition; fsync ordering; no blind duplicate effect; unauthenticated local IPC rejected/audited; compromised unprivileged component cannot mint authority/read vault/write policy/forge audit; exact-head cargo fmt/clippy/test/fuzz/property evidence.

---

## Phase 2 — Spec 003: Identity, Policy, Secrets & Sandbox

- [ ] **T030** Create bounded Spec 003 only after Spec 002 is closed canonical.
- [ ] **T031** Qualify Cedar exact dependency state and map Golam principals/actions/resources/context without delegating Golam semantics.
- [ ] **T032** Implement protected-resource policy, lease narrowing/expiry/revocation and approval classes/freshness.
- [ ] **T033** Implement taint label algebra and allowed downgrade mechanisms; propagate labels to artifacts/effects.
- [ ] **T034** Implement secret vault/broker, encrypted at rest, pasted-secret ingest redaction/tombstones, bounded unbrokerable injection and canary redaction tests.
- [ ] **T035** Implement strict-local egress gate across Golam-managed processes with external no-egress verification.
- [ ] **T036** Define/qualify sandbox profiles for MCP/skill/native helpers; Wasmtime only when justified.
- [ ] **T037** Complete prompt-injection/taint, secret isolation, policy self-modification and no-egress adversarial gates before close.

**PA-002 boundary**: no memory/retrieval framework, vector database, learning loop, worker graph or evaluation platform introduced by PA-002 is part of Spec 003. Existing kernel/taint/approval/effect/egress semantics remain prerequisites those later systems must consume.

---

## Phase 3 — Spec 004: Harness & Local Intelligence

- [ ] **T040** Create bounded Spec 004 after 003 closes. Read PA-002 and `contracts/behavior-evaluation-contract.md` before freezing harness semantics.
- [ ] **T041** Define model-independent harness/session-visible-history/tool/cancellation/retry/compaction contracts, explicitly comparing authorized Golam-Research, grok-build, Goose and exact PA-002 DeepSeek Harness/LangGraph evidence. Make `Turn`, `Step`, `RequestSeries`, `InboxItem`, `Interrupt` and `Continuation` explicit while preserving `MODEL_VISIBLE => LOGGED` from canonical events.
- [ ] **T042** Qualify exact `mistral.rs` version/API/hardware/network behavior as primary local backend candidate.
- [ ] **T043** Qualify `llama.cpp` sidecar compatibility path and keep unsafe C FFI outside `golamd`.
- [ ] **T044** Implement expanded ExecutionProfile and HardwareProfile calibration/routing without hidden locality downgrade. Runtime profiles may replace model/tool/context providers but MUST NOT replace privileged kernel authority services.
- [ ] **T045** Build shared conformance suites for model/harness/provider seams and benchmark model vs harness separately: TTFT/TPS/load/warm residency/cache/tool-call validity/repair/task success/resource use. Provider support claims require exact conformance evidence rather than interface compilation alone.

**Spec 004 exit additions from PA-002**: prove interrupt/resume without history loss, request-series/cache envelope correctness, cancellation behavior, provider capability truth, and inability of a runtime profile/plugin/adapter to mount or replace a protected authority service.

---

## Phase 4 — Spec 005: Local Tools, Context & Memory

- [ ] **T050** Create bounded Spec 005 after 004 closes. Read PA-002 architecture, source research and `contracts/memory-retrieval-learning-contract.md` before requirements freeze.
- [ ] **T051** Re-pin and qualify Golam-Research plus PA-002 Mem0, Qdrant, LlamaIndex, LangChain, Firecrawl, Hermes and Braintrust context-route evidence at then-current exact source states. Reuse/port only bounded qualified mechanisms preserving Rust/local-first/security rules; no framework/cloud dependency is admitted by the program plan alone.
- [ ] **T052** Implement governed filesystem/shell/process/git/browser tool surfaces through kernel capability/effect gates. Web evidence remains tainted, attributable and egress-governed; Firecrawl-class hosted access is optional, explicit and never a strict-local fallback.
- [ ] **T053** Implement the Context Compiler as measured route selection over direct files/ripgrep/git, SQLite/structured query, FTS/BM25, justified L1 Tree-sitter/LSP, and optional dense/sparse/hybrid/entity routes. Preserve per-evidence source/hash/time/authority/permission/taint/score metadata; L2 graph/dataflow remains optional by measured need.
- [ ] **T054** Implement Markdown canonical vault + SQLite operational state plus explicit working/run/episodic/semantic/procedural/project/user/worker scopes. Add local SQLite FTS5 or equivalent session-search baseline before making semantic/vector retrieval mandatory.
- [ ] **T055** Implement `MemoryCandidate` extraction separate from promotion, ADD-first observation evidence, temporal metadata/entity links, single governed canonical memory writer and reconciliation of user hand-edited Markdown. Agent-generated assertions cannot self-promote.
- [ ] **T056** Implement approval/authoritative-verification promotion plus conflict/supersession/CONTRADICT/MERGE/EXPIRE/FORGET/REDACT. Derived lexical/vector/entity indexes MUST be deletable/rebuildable without canonical knowledge loss and a corrupt index cannot fabricate authority or canonical refs.
- [ ] **T057** Benchmark retrieval providers before admission: SQLite lexical/structured baseline versus any simple local vector baseline and Qdrant Edge exact candidate where still relevant. If Qdrant is admitted it remains a derived index; remote Qdrant is never required for strict-local core. Implement Agent Skills-compatible instructions and governed skill lifecycle on the same provenance/capability principles.
- [ ] **T058** Implement MCP/ACP and optional LangChain/LlamaIndex/Firecrawl-class compatibility adapters as untrusted interoperability boundaries. Add `LearningProposal` generation for memory/skill/behavior/routing/profile candidates, but canonical adoption remains governed and background learning cannot silently write memory or mutate installed skills.
- [ ] **T059** Run `ContextRouteBench` plus memory quality/security gates: compare filesystem/lexical/structured/vector/hybrid routes under equal model/budget conditions; run admissible LoCoMo/LongMemEval/BEAM subsets or full suites plus Golam-specific stale-live-state, contradiction, scope isolation, secret-derived rejection, prompt-injection, user-edit reconciliation, crash/promotion and index-corruption/rebuild tests. Report success with tokens and latency, not accuracy alone.

**Spec 005 exit additions from PA-002**: prove `MemoryCandidate != Durable Truth`, `RetrievalIndex != Canonical Memory`, live state outranks stale memory, web/provider evidence cannot grant authority, and useful local search remains available without remote embeddings/services.

---

## Phase 5 — Spec 006: Desktop & Computer Control

- [ ] **T060** Create bounded Spec 006 after 005 closes.
- [ ] **T061** Mine Golam-Research desktop/preload/RPC/settings/renderer contracts as authorized implementation evidence while building an independent Tauri/Rust desktop rather than carrying Electron into the trusted product architecture.
- [ ] **T062** Build Tauri Desktop as an authenticated client of `golamd`; renderer remains untrusted.
- [ ] **T063** Define/implement DesktopController semantic snapshot/ref/action contract.
- [ ] **T064** Implement Windows UIA-first path with locked/UAC/secure-desktop fail-closed behavior.
- [ ] **T065** Implement macOS AX/TCC path with explicit Accessibility/Screen Recording permission state.
- [ ] **T066** Implement Linux AT-SPI/X11/Wayland capability tiers with honest compositor/portal failures.
- [ ] **T067** Add input-injection and vision fallbacks only behind semantic failure/need plus post-action verification.
- [ ] **T068** Implement separate clipboard read/write and deny-by-default camera/mic capabilities.
- [ ] **T069** Implement human takeover at lease/input-authority layer and test takeover latency/stale refs/wrong-window hazards.

---

## Phase 6 — Spec 007: Phone, GolamConnect & Channel Access

**Binding amendment**: `program-amendments/PA-001-phone-channel-access.md` expands this phase. Native mobile and phone voice are no longer deferred through Spec 010. No implementation is authorized by this task graph; Spec 007 still requires its full Spec Kit lifecycle.

- [ ] **T070** Create bounded Spec 007 after prerequisites 003+006 close; Spec 008 may precede 007 only with explicit reviewed dependency justification. Read PA-001 and `contracts/phone-channel-access-contract.md` before scope freeze.
- [ ] **T071** Qualify exact Iroh dependency and relay metadata/privacy behavior; define Connect Core device identity, pairing, signed/encrypted envelopes, replay protection, reconnect and generation semantics before channel/mobile UI work.
- [ ] **T072** Qualify RASystem exact selected crates/files plus relevant mobile/channel donors and provider SDK/source candidates. Independently review grants/nonces/control/audit/media/provenance; permission does not bypass technical/security admission.
- [ ] **T073** Implement GolamConnect cryptographic device pairing/revocation, short-lived generation-based capability leases, host-side per-message authorization and reconnect full revalidation.
- [ ] **T074** Implement **Golam Mobile** for iOS/Android as a client of the canonical daemon: shared Rust protocol/crypto core, secure device-key storage, session/task/worker continuity, mobile approvals, privacy-minimized push wake/sync, file/photo/voice-note input, pause/stop controls, and explicit strict-local behavior. Qualify Tauri 2 mobile versus a native Swift/Kotlin shell around shared Rust before freezing the UI stack.
- [ ] **T075** Implement native Connect screen/media, input, multi-monitor, clipboard, file transfer, visible indicator, emergency stop and human/agent takeover arbitration after Spec 006 computer-control prerequisites exist. Add mobile remote-view/control UX without routing protected control through messaging providers.
- [ ] **T076** Implement early voice interaction on mobile: push-to-talk/voice notes, governed ASR/TTS provider selection, interruption/cancel, bounded audio/transcript retention, and tests proving voice cannot act as authentication or bypass approval. Full-duplex “call Golam” follows only after basic mobile reliability/safety gates.
- [ ] **T077** Implement the common `ChannelAdapterDescriptor` + normalized `ChannelEnvelope`, stable binding/revocation generations, narrow ingress modes, attachment quarantine, edit/delete/replay semantics, causality/hop-loop protection and outbound Effect Gate. Ship Telegram first through the official Bot API, preferring local polling for the initial local-first path; optional webhook mode must authenticate and dedupe.
- [ ] **T078** Add channels only through current official paths: WhatsApp Business Platform/Cloud API (not unofficial WhatsApp Web/personal-account automation); qualified official WeChat/WeCom robot/application APIs (not unsupported consumer WeChat automation); Slack official Events/Socket Mode; Discord official bot/Gateway; Matrix Application Service as an open/self-hostable candidate. Every adapter publishes an exact capability/privacy matrix and current official-source qualification.
- [ ] **T079** Run phone/channel release-entry gates: two-device pairing/revoke/reconnect; push-payload privacy/reorder/collapse; stale mobile approval; voice approval-bypass; provider webhook forgery/replay/duplicate/out-of-order; spoofed/recycled identity; group injection; cross-channel replay; message edit/delete; attachment fuzz/quarantine; provider outage/rate limits; channel loops; strict-local no channel/push egress; and mobile remote-control emergency-stop/takeover races on claimed platforms.

---

## Phase 7 — Spec 008: Workers, Durable Graphs, Learning & Automations

- [ ] **T080** Define typed worker lifecycle, spawn/join/cancel/crash-adopt, narrow lease inheritance and workspace/worktree isolation after comparing qualified worker/subagent implementations from authorized sources. Re-pin PA-002 LangGraph/DeepSeek Harness/Hermes/autoresearch evidence before freezing worker orchestration.
- [ ] **T081** Implement durable scheduler/triggers using canonical event/effect semantics; prove restart does not double-fire effects. Phone/channel inputs may become typed triggers only through explicit policy-bound rules defined by Spec 007; receiving a message is not blanket automation authority.
- [ ] **T082** Implement bounded parallelism and parent/child goal/causality tracking plus an event-sourced `WorkerGraph` with deterministic/model/tool/worker/wait/interrupt/join/verification nodes. Bind `WORKFLOW_CHECKPOINT != EXTERNAL_EFFECT_COMMIT`: graph replay cannot blind-retry protected external effects and resume revalidates current authority/live state.
- [ ] **T083** After single-worker reliability is proven, add groups/collaboration/teach-by-demonstration and bounded `LearningReview`/versioned skill-improvement proposals. Add an autoresearch-inspired `ExperimentProgram` only for declared non-authority mutable scopes with immutable evaluators, fixed budgets, sandbox/worktree, keep/discard evidence, guardrails and separate adoption. Ordinary experiments cannot autonomously adopt privileged-kernel/authority changes.

**Spec 008 exit additions from PA-002**: crash/restart preserves graph causality without duplicate effects; interrupts resume safely; learning remains proposals until governed adoption; skill patches cannot self-expand capabilities; experiment candidates cannot rewrite evaluators/guardrails or escape declared resource bounds.

---

## Phase 8 — Spec 009: Grok Public Feature Parity

- [ ] **T090** Refresh public Grok Bot capability evidence AND mine the pinned Golam-Research/Grok Bot 0.18 implementation evidence at exact source state. Keep source-derived implementation evidence distinct from public behavior evidence and from any later Grok releases.
- [ ] **T091** Close independently implementable MUST-MATCH domains: persistent agents/computer/workspace, apps/browser/files/shell, long-running/background work, governed cross-session memory/search, proactive learning/skill improvement, approvals, local computer control, native mobile continuity, channels/connectors/MCP, routines/schedules, multimodal input, deep research/evidence-rich artifacts and built-in Documents/Presentations/Spreadsheets/PDFs/Skill Creator equivalents.
- [ ] **T092** Use authorized source code/behavior where technically justified but preserve Golam's Rust/local-first/security architecture rather than mechanically cloning Electron/cloud/framework assumptions. Prefer `VERIFIED_SUPERSET` where Golam's user-owned memory, authority-aware retrieval, learning governance and local/offline behavior are stronger.
- [ ] **T093** Require atomic scenario evidence before `VERIFIED_EQUIVALENT` or `VERIFIED_SUPERSET` states. A parity scenario that succeeds by violating Golam authority/privacy/durability invariants is a failure, not parity.

---

## Phase 9 — Spec 010: GolamBench & Release Qualification

- [ ] **T100** Aggregate all incremental gates into a reproducible local-first GolamBench release suite. Add typed EvalDataset/EvalCase/ReferenceEvidence/Scorer/BehaviorSpec/TrajectoryTrace/ExperimentRun records and optional compatibility with open `BEHAVIOR.md`; no hosted eval/telemetry service is mandatory.
- [ ] **T101** Qualify long-horizon goal retention/premature termination/model-switch/compaction behavior with trajectory-level behaviors: evidence gathering, live-state reread, verification discipline, interruption, recovery, abstention and no fabricated PASS/completion claims. Outcome success alone is insufficient.
- [ ] **T102** Qualify crash/recovery/idempotency/fork/checkpoint/disk failure across session ledger and WorkerGraph. Explicitly prove graph/checkpoint replay never substitutes for effect reconciliation or duplicates AT_MOST_ONCE/IRREVERSIBLE effects.
- [ ] **T103** Qualify prompt injection/taint/memory poisoning/secret isolation/policy self-escalation/local IPC/channel impersonation plus PA-002 memory/index/eval threats: false memory promotion, stale-live conflicts, cross-scope leakage, corrupt derived indexes, judge/evaluator injection and skill/experiment self-expansion.
- [ ] **T104** Qualify computer control, native Golam Mobile, push privacy, voice approval isolation and GolamConnect across the exact claimed platform/device matrix. Include user interruption/takeover as trajectory behaviors, not only final-state tests.
- [ ] **T105** Prove strict-local no-egress from outside the Golam process boundary, including no third-party channel/APNs/FCM traffic and no hidden Mem0/Firecrawl/Braintrust/LangSmith/vector/eval provider calls when strict-local is active. Run ContextRouteBench and admissible memory benchmarks under equal budget/model constraints, reporting tokens/latency/resources with quality.
- [ ] **T106** Produce exact-head evidence/receipts for every release/parity/security/performance claim. Deterministic/execution-grounded scorers take precedence where possible; every LLM judge result records exact judge/provider/prompt-rubric/version/cost/variance provenance and cannot alone close a security gate.

---

## Deferred through Spec 010 unless separately re-authorized

- always-on background wake-word / autonomous microphone listening;
- A2A external federation;
- image/video generation parity;
- custom Golam relay infrastructure;
- multi-device CRDT memory sync;
- mandatory Graphify/code-graph L2 stack;
- large swarm architecture.

## Next safe action

Follow exact live repository truth and the currently active bounded Spec Kit feature. PA-001 and PA-002 are future-program planning amendments only: neither authorizes leapfrogging the current implementation sequence. Phone/channel work begins only when Spec 007 is legitimately opened; PA-002 memory/retrieval/harness/worker/eval work begins only inside its owning Specs 004/005/008/010 after their prerequisites and planning gates close.
