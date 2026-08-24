# Tasks: Golam Local Agent OS Foundation Program

**Spec**: 001 — Golam Local Agent OS Foundation  
**Generated**: 2026-08-24 after GLM-5.3 reconciliation and cross-artifact analysis  
**Authority**: PROGRAM ORCHESTRATION ONLY — this file does not authorize implementing the entire Golam product or skipping later Spec Kit gates.

## Task execution rule

Every implementation feature below MUST create its own Spec Kit package and complete:

`specify -> clarify -> research as needed -> plan -> data model/contracts -> checklist -> tasks -> analyze -> implement -> converge`

before that feature's product code is authorized.

No later feature may infer authority from this program task graph alone.

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
- [ ] **T009** Founder reviews Draft PR #1 and explicitly decides whether to merge/freeze Spec 001 into `main`. Do not merge automatically from this task file.

**Exit gate**: Spec 001 merged/frozen on exact `main` truth before the next feature branch is created.

---

## Phase 1 — Spec 002: Kernel & Durable Session Spine

**Goal**: prove the smallest useful authority/durability spine before model/tool complexity.

- [ ] **T010** Create `spec/002-kernel-durable-session-spine` from exact frozen `main` only after T009.
- [ ] **T011** Write Spec 002 requirements for the <=8 initial Rust crate/binary boundary; prohibit empty crate scaffolding.
- [ ] **T012** Define the process-splittable privileged-kernel API and sealed/unforgeable authority types.
- [ ] **T013** Define authenticated local IPC implementation choices per platform and delegated client enrollment.
- [ ] **T014** Define versioned event/session/fork/goal ledger schemas, security-critical integrity chaining, global audit ordering, checkpoints and content-addressed artifact lifecycle.
- [ ] **T015** Define effect handler/executor/reconciler API, durable intent-before-dispatch and UNKNOWN_OUTCOME behavior for all five semantics classes.
- [ ] **T016** Define bootstrap `Authorize(principal, action, resource, context)` with deny-by-default semantics that Spec 003 can replace with Cedar without semantic rewrite.
- [ ] **T017** Define protected kernel state storage boundary and prove generic tools cannot mutate it.
- [ ] **T018** Add Spec 002 benchmark exit gates: BS-1 durability/fork/replay, BS-2 duplicate-effect prevention, BS-10 strict-local no-egress skeleton, kernel-boundary compromise probe and unauthenticated-local-client probe.
- [ ] **T019** Complete Spec 002 checklist/tasks/analyze before any Rust implementation.

**Required implementation evidence before Spec 002 closes**: crash fault injection at every effect transition; fsync ordering; no blind duplicate effect; unauthenticated local IPC rejected/audited; compromised unprivileged component cannot mint authority/read vault/write policy/forge audit; exact-head cargo fmt/clippy/test/fuzz/property evidence.

---

## Phase 2 — Spec 003: Identity, Policy, Secrets & Sandbox

- [ ] **T020** Create bounded Spec 003 only after Spec 002 is closed canonical.
- [ ] **T021** Qualify Cedar exact dependency state and map Golam principals/actions/resources/context without delegating Golam semantics.
- [ ] **T022** Implement protected-resource policy, lease narrowing/expiry/revocation and approval classes/freshness.
- [ ] **T023** Implement taint label algebra and allowed downgrade mechanisms; propagate labels to artifacts/effects.
- [ ] **T024** Implement secret vault/broker, encrypted at rest, pasted-secret ingest redaction/tombstones, bounded unbrokerable injection and canary redaction tests.
- [ ] **T025** Implement strict-local egress gate across Golam-managed processes with external no-egress verification.
- [ ] **T026** Define/qualify sandbox profiles for MCP/skill/native helpers; Wasmtime only when justified.
- [ ] **T027** Complete prompt-injection/taint, secret isolation, policy self-modification and no-egress adversarial gates before close.

---

## Phase 3 — Spec 004: Harness & Local Intelligence

- [ ] **T030** Create bounded Spec 004 after 003 closes.
- [ ] **T031** Define model-independent harness/session-visible-history/tool/cancellation/retry/compaction contracts.
- [ ] **T032** Qualify exact `mistral.rs` version/API/hardware/network behavior as primary local backend candidate.
- [ ] **T033** Qualify `llama.cpp` sidecar compatibility path and keep unsafe C FFI outside `golamd`.
- [ ] **T034** Implement expanded ExecutionProfile and HardwareProfile calibration/routing without hidden locality downgrade.
- [ ] **T035** Benchmark model vs harness separately: TTFT/TPS/load/warm residency/cache/tool-call validity/repair/task success/resource use.

---

## Phase 4 — Spec 005: Local Tools, Context & Memory

- [ ] **T040** Create bounded Spec 005 after 004 closes.
- [ ] **T041** Implement governed filesystem/shell/process/git/browser tool surfaces through kernel capability/effect gates.
- [ ] **T042** Implement context L0 (files/ripgrep/git) and justified L1 structural evidence (Tree-sitter/LSP); L2 graph/dataflow remains optional by measured need.
- [ ] **T043** Implement Markdown canonical vault + SQLite operational state and the full governed memory operation contract.
- [ ] **T044** Implement single Golam memory writer plus reconciliation of user hand-edited Markdown.
- [ ] **T045** Implement promotion/conflict/supersession/FORGET/REDACT and full derived-index rebuild tests.
- [ ] **T046** Implement Agent Skills-compatible instructions and governed lifecycle; executable scripts wait for qualified sandbox support.
- [ ] **T047** Implement MCP/ACP as untrusted interoperability boundaries; MCP processes are sandboxed and tainted.
- [ ] **T048** Run memory-poisoning, stale-memory, injection and strict-local regression gates.

---

## Phase 5 — Spec 006: Desktop & Computer Control

- [ ] **T050** Create bounded Spec 006 after 005 closes.
- [ ] **T051** Build Tauri Desktop as an authenticated client of `golamd`; renderer remains untrusted.
- [ ] **T052** Define/implement DesktopController semantic snapshot/ref/action contract.
- [ ] **T053** Implement Windows UIA-first path with locked/UAC/secure-desktop fail-closed behavior.
- [ ] **T054** Implement macOS AX/TCC path with explicit Accessibility/Screen Recording permission state.
- [ ] **T055** Implement Linux AT-SPI/X11/Wayland capability tiers with honest compositor/portal failures.
- [ ] **T056** Add input-injection and vision fallbacks only behind semantic failure/need plus post-action verification.
- [ ] **T057** Implement separate clipboard read/write and deny-by-default camera/mic capabilities.
- [ ] **T058** Implement human takeover at lease/input-authority layer and test takeover latency/stale refs/wrong-window hazards.

---

## Phase 6 — Spec 007: GolamConnect

- [ ] **T060** Create bounded Spec 007 after prerequisites 003+006 close; Spec 008 may precede 007 only with explicit reviewed dependency justification.
- [ ] **T061** Qualify exact Iroh dependency and relay metadata/privacy behavior.
- [ ] **T062** Qualify RASystem exact selected crates/files and independently review grants/nonces/control/audit/media; do not wholesale fork.
- [ ] **T063** Implement cryptographic device pairing/revocation and short-lived generation-based control leases.
- [ ] **T064** Implement signed/replay-protected per-message host authorization and reconnect full revalidation.
- [ ] **T065** Implement screen/media, input, multi-monitor, clipboard, file transfer, visible indicator, emergency stop and human/agent takeover arbitration.
- [ ] **T066** Implement Telegram first as command/notification bridge using provider-stable IDs; WhatsApp/Slack/Discord follow through the same channel-binding contract.
- [ ] **T067** Run two-machine NAT/relay/loss/replay/revocation/lease-expiry/channel-impersonation/emergency-stop tests on supported host platforms.

---

## Phase 7 — Spec 008: Workers & Automations

- [ ] **T070** Define typed worker lifecycle, spawn/join/cancel/crash-adopt, narrow lease inheritance and workspace/worktree isolation.
- [ ] **T071** Implement durable scheduler/triggers using canonical event/effect semantics; prove restart does not double-fire effects.
- [ ] **T072** Implement bounded parallelism and parent/child goal/causality tracking.
- [ ] **T073** Keep groups/collaboration and teach-by-demonstration late in the spec; single-worker reliability is the entry gate.

---

## Phase 8 — Spec 009: Grok Public Feature Parity

- [ ] **T080** Refresh public Grok Bot capability evidence and pin every parity record to source/version/date.
- [ ] **T081** Close independently implementable MUST-MATCH domains: persistent agents/computer/workspace, apps/browser/files/shell, long-running/background work, memory, approvals, local computer control, channels/connectors/MCP, routines/schedules, multimodal input, deep research/artifacts and built-in Documents/Presentations/Spreadsheets/PDFs/Skill Creator equivalents.
- [ ] **T082** Record deferred/non-applicable domains honestly rather than copying proprietary UX/prompts/assets.
- [ ] **T083** Require scenario evidence before `VERIFIED_EQUIVALENT` or `VERIFIED_SUPERSET` states.

---

## Phase 9 — Spec 010: GolamBench & Release Qualification

- [ ] **T090** Aggregate all incremental gates into a reproducible GolamBench release suite.
- [ ] **T091** Qualify long-horizon goal retention/premature termination/model-switch/compaction behavior.
- [ ] **T092** Qualify crash/recovery/idempotency/fork/checkpoint/disk failure.
- [ ] **T093** Qualify prompt injection/taint/memory poisoning/secret isolation/policy self-escalation/local IPC/channel impersonation.
- [ ] **T094** Qualify computer control and GolamConnect across the exact claimed platform matrix.
- [ ] **T095** Prove strict-local no-egress from outside the Golam process boundary.
- [ ] **T096** Produce exact-head evidence/receipts for every release/parity/security claim.

---

## Deferred through Spec 010 unless separately re-authorized

- native mobile application;
- voice-mode product experience;
- A2A external federation;
- image/video generation parity;
- custom Golam relay infrastructure;
- multi-device CRDT memory sync;
- mandatory Graphify/code-graph L2 stack;
- large swarm architecture.

## Next safe action

`T009` only: founder reviews Draft PR #1 and chooses whether to merge the frozen Spec 001 planning package. After merge, create Spec 002 from exact live `main` and run its Spec Kit lifecycle. Do not begin product implementation directly from this program task file.
