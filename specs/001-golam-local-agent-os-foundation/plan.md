# Implementation Plan: Golam Local Agent OS Foundation

**Branch**: `spec/001-golam-local-agent-os-foundation` | **Date**: 2026-08-24  
**Spec**: `spec.md`  
**Status**: `GLM_RECONCILED_PENDING_FINAL_CONSISTENCY_FREEZE`

## Summary

Build Golam as a clean Rust-first local Agent OS with a small authority-bearing privileged kernel, authenticated local clients, local canonical state, replaceable model/harness/tool adapters, semantic-first computer control, user-owned Markdown memory, and secure native remote control through GolamConnect.

The GLM-5.3 review returned `APPROVE_WITH_MANDATORY_CHANGES`. All 2 BLOCKER and 8 MAJOR findings are accepted; no founder waiver is taken. The binding corrections are expressed in the reconciled spec/data model and dedicated contracts under `contracts/`.

Spec 001 does not authorize implementing the entire product in one PR. It freezes the program architecture and decomposes implementation into bounded follow-on Spec Kit features.

### Additive program amendment PA-001 — Phone / Mobile / Voice / Channels

Founder direction on 2026-08-28 expands the future Spec 007 scope so Golam is first-class reachable from a phone. `program-amendments/PA-001-phone-channel-access.md` and `contracts/phone-channel-access-contract.md` are the binding additive planning artifacts for this change.

PA-001 supersedes only the previous deferral of **native mobile application** and **phone voice interaction**. It does not weaken existing authority/security contracts and it does not authorize implementation in the active earlier spec. Native Golam Mobile is a cryptographically paired GolamConnect device; Telegram/WhatsApp/WeChat/Slack/Discord/Matrix remain separately governed third-party channel transports.

## Technical Context

**Language**: Rust stable for trusted/runtime code; TypeScript/React only for untrusted Tauri renderer; optional Python/Node adapters outside trusted/privileged paths.  
**Runtime**: Tokio-based local daemon; exact crates pinned only after donor/dependency qualification.  
**Desktop**: Tauri 2 + React/TypeScript renderer, Rust backend.  
**Mobile**: future Spec 007 must preserve a Rust-owned GolamConnect client/protocol core and qualify Tauri 2 mobile versus a small native Swift/Kotlin shell around shared Rust before stack freeze.  
**Local state**: SQLite for operational state; Markdown/files for canonical human knowledge; content-addressed large artifacts.  
**Authorization**: Golam-owned capability/effect schema with Cedar as policy-engine candidate.  
**Local IPC**: authenticated OS-native IPC; no unauthenticated localhost control surface.  
**Sandbox**: explicit per-process/profile supervision; Wasmtime/WASI for bounded portable extensions, OS-native isolation for native tools.  
**Local inference**: `mistral.rs` primary candidate; `llama.cpp` compatibility sidecar; optional adapters later.  
**Networking**: kernel-authorized egress; Iroh/QUIC candidate for native GolamConnect; messaging and APNs/FCM are explicit non-strict-local egress capabilities.  
**Protocols**: ACP stable v1, MCP 2026-07-28 semantics, Agent Skills-compatible packages; A2A later for external federation.  
**Testing**: unit/property/fuzz/fault-injection/integration/on-device tests plus incremental GolamBench gates from Spec 002 onward.  
**Target platforms**: Windows 11, supported macOS, major Linux desktops, plus iOS/Android for the future Golam Mobile client, all with explicit capability matrices rather than false parity.  
**Constraints**: strict-local/no hidden fallback, small TCB, authenticated clients, crash-safe state, no blind duplicate effects, user-interruptible control.

## Constitution Check

| Gate | Result | Evidence |
|---|---|---|
| Local ownership/trust root | PASS | strict-local + mechanized egress contract |
| Rust trusted path | PASS | constitution 1.1.0 |
| Small privileged kernel | PASS_SPEC | kernel-boundary contract; runtime proof deferred to Spec 002 |
| Authenticated local control | PASS_SPEC | local-IPC contract |
| Explicit authority | PASS_SPEC | policy/protected-resource/approval contracts |
| Gated durable effects | PASS_SPEC | effect FSM + handler/reconciler contract |
| Secrets/taint | PASS_SPEC | broker + fallback + taint algebra contracts |
| User-owned governed memory | PASS_SPEC | Markdown/SQLite + memory-governance contract |
| Replaceable model/harness | PASS | expanded ExecutionProfile |
| Semantic-first control | PASS | platform-aware computer-control contract |
| GolamConnect security | PASS_SPEC | stable binding/per-message/reconnect rules |
| Clean-room donor governance | PASS | Source Foundry; Golam-Research reference-only |
| Verification over claims | PASS | incremental + release GolamBench |

`PASS_SPEC` means the architecture contract is frozen but implementation evidence belongs to the owning follow-on spec.

## Enforceable Architecture

```text
       Desktop / CLI / IDE                    Native Golam Mobile
               |                                      |
       Authenticated Local IPC                  GolamConnect transport
               |                                      |
               +------------------+-------------------+
                                  |
                                golamd
                                  ^
                                  |
                 normalized untrusted ChannelEnvelope
                                  |
              Telegram / WhatsApp / WeChat-WeCom /
                Slack / Discord / Matrix / later
                                  |
                        Channel Adapter boundary
                                  |
               unprivileged/replaceable runtime services
          +-----------+-----------+-----------+-------------+
          |           |           |           |             |
       Harness      Context      Memory     Tools/Control  Scheduler
          |           |           |           |             |
          +-----------+-----------+-----------+-------------+
                                  |
                         typed kernel requests
                                  |
                    +---------------------------+
                    | PRIVILEGED RUST KERNEL    |
                    | identity + IPC auth       |
                    | capability/lease minting  |
                    | Cedar policy/protected DB |
                    | approvals                |
                    | effect journal/reconcile  |
                    | secret broker/redaction   |
                    | egress authorization      |
                    | audit/receipt integrity   |
                    | pairing/revocation        |
                    +-------------+-------------+
                                  |
                   leases/decisions/secret handles
                                  |
          +-----------------------+------------------------+
          |                        |                       |
      Sandboxes                OS executors            Model sidecars
   MCP/skills/helpers        browser/computer          local inference
```

Native mobile and third-party channels deliberately do not share the same trust path. A mobile device may hold an explicit cryptographic GolamConnect device lease; channel content remains channel-untrusted even after account binding.

### Trusted path vs privileged kernel

Rust trusted-path code may perform important local runtime work. Only the privileged kernel owns authority-bearing state/keys and may mint/validate capabilities, authorize effects/egress, broker secrets, authenticate clients, or commit/sign security-critical records. This distinction is binding.

A single-process v1 is allowed only with sealed authority types, protected kernel state, explicit kernel APIs, isolated parser/adaptor surfaces, and a process-splittable interface. The security design MUST NOT rely on crate naming as an isolation mechanism.

## Core invariants

- `MODEL_VISIBLE => LOGGED`, except secret-ingestion redaction/tombstone rules prevent accidental plaintext secret persistence.
- `NO_EXTERNAL_EFFECT_WITHOUT_EFFECT_GATE`
- `NO_GOLAM_MANAGED_EGRESS_WITHOUT_EGRESS_GATE`
- `AGENT_CANNOT_EXPAND_OWN_AUTHORITY`
- `CHANNEL != AUTHORITY`
- `PUSH_DELIVERY != AUTHORITY_OR_CANONICAL_ORDER`
- `VOICE_CONTENT != AUTHENTICATION`
- `LOCALHOST != AUTHENTICATION`
- `UNTRUSTED_DATA != INSTRUCTION_AUTHORITY`
- `SAFETY_DENIAL_IS_MONOTONIC`
- `MEMORY != TRUTH`
- `FULL_CANONICAL_HISTORY_SURVIVES_COMPACTION`
- `REAL_SECRETS_STAY_OUT_OF_MODEL_CONTEXT_WHEN_BROKERABLE`
- `PROTECTED_AUTHORITY_STATE_IS_NOT_GENERIC_FILESYSTEM_STATE`
- `EVERY_WRITE_IS_ATTRIBUTABLE`
- `EVERY_LONG_RUN_IS_CRASH_RESUMABLE`
- `UNKNOWN_EFFECT_OUTCOME_BLOCKS_DEPENDENT_EFFECTS`

## Initial Rust workspace — binding simplification

Do NOT start by creating the full target crate grid. Spec 002 begins with at most eight real crates/binaries, splitting only when ownership/testing boundaries are proven.

Suggested initial spine:

```text
crates/
  golam-kernel
  golam-events-session
  golam-effects
  golam-policy-identity
  golam-secrets
  golam-harness
  golam-models
apps/
  golamd
  golam
```

Later target decomposition may split memory/context/tools/control/connect/ACP/MCP/audit/bench and OS-specific crates. Empty architectural crates are forbidden as planning theater.

## Component Decisions

### 1. `golamd` and local IPC

`golamd` is the long-lived local coordinator, not synonymous with the privileged kernel. Desktop/CLI/IDE clients authenticate over OS-native local IPC. Windows uses user-SID-restricted named pipes or stronger; Unix uses owner-only UDS + peer credentials or stronger. IDE/ACP delegated clients receive explicit scoped enrollment credentials.

No unauthenticated control HTTP/WS surface is permitted, even on loopback. Any loopback HTTP use requires authentication plus Origin/Host/CSRF/DNS-rebinding protections.

### 2. Session/Event and Goal Ledgers

Canonical events are append-oriented and versioned. Chat/context/UI are projections. Retry/rewind/model alternatives create immutable forks referencing parent prefixes; history is not rewritten. Cross-session causality is explicit. Security-critical families use mandatory integrity chaining. Large artifacts are content-addressed and governed by retention/GC. Checkpoints accelerate replay but never replace canonical history.

Goal Ledger remains outside ordinary compaction and carries goal, criteria, non-negotiable constraints, scope, proven facts, authoritative refs, blockers, completed work, and next safe action.

### 3. Effect Gate + Handler/Reconciler

Every consequential effect is proposed, authorized, durably journaled, executed, reconciled when ambiguous, verified, and receipted.

Semantics: READ_ONLY, IDEMPOTENT_AT_LEAST_ONCE, AT_MOST_ONCE, COMPENSATABLE, IRREVERSIBLE.

Each handler declares idempotency derivation, `execute`, read-only/safe `reconcile`, timeout/ambiguity policy, compensation, and evidence. The intent must be fsync-persistent before external dispatch. AT_MOST_ONCE/IRREVERSIBLE ambiguity never blind-retries; dependent effects wait on UNKNOWN_OUTCOME. MANUAL_REVIEW is first-class.

### 4. Identity / Policy / Protected Resources / Approvals

Policy input is `(principal, action, resource, context)`. Spec 002 defines the interface and a deny-by-default bootstrap evaluator; Spec 003 supplies Cedar integration without changing semantics.

Capability leases narrow only. Protected kernel resources include policy/principals/leases/approvals/secrets/effect journal/audit/pairing/egress/skill lock/schedule authority and cannot be mutated by generic file tools.

Approval classes: ONCE, SESSION_SCOPED, TIME_BOXED, OPERATION_PATTERN, RUN_PREAUTHORIZATION. Approval freshness is checked at execution. Unattended IRREVERSIBLE effects require explicit bounded per-run preauthorization.

### 5. Secrets and Taint

Secrets are handles. Broker at egress/client boundary when possible. Unbrokerable use requires explicit class approval, isolated process injection (not argv), no ambient inheritance, canary/value-aware redaction, and bounded retention. User-pasted secrets are redacted/tombstoned at ingestion; vault encrypted at rest.

Taint propagates through summaries, memory candidates, code/scripts/files/artifacts. Downgrade only by human approval or deterministic registered authoritative verification. Model self-assertion never downgrades. SECRET_DERIVED never enters long-term canonical memory.

### 6. Strict-Local Egress

Network capability is denied by default in strict-local mode for every Golam-managed process: models, tools, MCP, skills, browser helpers, telemetry/update checks, adapters, sidecars, channel bridges and push providers. Loopback is separately scoped. Components that need forbidden egress fail clearly. This is tested from outside Golam with sinkhole/network observation.

Strict-local may still permit an explicitly qualified direct local/LAN native phone path if its transport profile is locally scoped and the owning spec proves the boundary; it never silently enables APNs, FCM, Telegram, WhatsApp, WeChat, Slack, Discord or other external provider traffic.

### 7. Harness and ExecutionProfile

Harness semantics remain Golam-owned and model/provider independent. ExecutionProfile includes model+revision, tokenizer/chat template, backend, locality, quantization, hardware mapping, harness, reasoning, native/grammar/text-fallback tool-call mode, schema mode, context, prefix/KV cache strategy, warm residency, sampling, workload class, multimodal flags, resource/latency/quality budgets, privacy/network constraints, fallback behavior, and benchmark refs.

`mistral.rs` is the primary candidate behind an adapter. `llama.cpp` is preferred as an out-of-process compatibility sidecar to keep unsafe C FFI outside `golamd`.

### 8. Context Compiler

`intent -> evidence requirements -> source routing -> retrieve -> authority/time/permission filter -> rank -> sufficiency -> replan -> ContextCapsule`.

Coding tiers: L0 files/ripgrep/git; L1 Tree-sitter/LSP; L2 graph/dataflow/runtime only by justified need. Graphify/code-graph systems are not P0 requirements. Initial assistant/research/document/browser capsules may use budgeted sufficiency heuristics rather than a research-heavy universal planner.

### 9. Memory Brain

Markdown canonical durable knowledge; SQLite operational state; derivatives rebuildable. Memory service is the single Golam writer for managed vault mutations while user hand-edits remain supported through hash/version reconciliation.

Governed operations: ADD, UPDATE, SUPERSEDE, CONTRADICT, MERGE, EXPIRE, FORGET, REDACT. Project/user promotion requires provenance plus user approval or deterministic authoritative verification. Contradictions are surfaced. FORGET/REDACT rewrites active canonical content and rebuilds indexes; already-sent external artifacts are not falsely claimed revoked. Backup/restore and disk-full fail-closed behavior are mandatory.

### 10. Skills / MCP / ACP / Sandbox

SKILL.md and MCP/ACP are interoperability surfaces, never authority. Skills-as-instructions may ship before executable skill scripts. Executable skills and MCP servers require explicit sandbox profiles with cleared env, bounded FS/network/process/resource access and supervised cancellation. Results remain untrusted/plugin-tainted.

Wasmtime is optional for portable bounded extension code; it is not a universal native-tool sandbox.

### 11. Computer Control

Hierarchy: `Domain/App API -> Native OS automation -> Accessibility/Semantic tree -> Browser DOM/protocol -> Input Injection -> Vision`.

Actions close the loop: BeforeState -> Intent -> Authorization -> Act -> ObservedAfter -> Verify. Stale refs fail/reobserve.

Windows: UIA first; input only on unlocked interactive desktop; UAC/secure desktop not bypassed. macOS: AX/TCC explicit. Linux: AT-SPI; X11/XTEST; pure Wayland only through supported portals/compositor capabilities; unsupported states fail closed.

Clipboard read is distinct. Camera/mic deny by default.

### 12. Phone, GolamConnect, Push and Messaging Channels

Native Connect is separate from messaging bridges. Pairing establishes cryptographic device identity. Every protected message is signed/replay-protected and checked against current short-lived lease, revocation, and generation. Newer control generation invalidates old input streams.

Iroh/QUIC is the P0 transport candidate; do not build a custom relay. Relay payloads are E2E encrypted but endpoint/timing/IP metadata exposure is documented; relay selection/self-host config may be offered later without making custom relay infrastructure a P0 deliverable.

**Native Golam Mobile** becomes part of future Spec 007 under PA-001. The phone is a GolamConnect `Device`, not a channel account. The mobile client reuses canonical sessions/tasks/workers and a Rust-owned protocol/crypto core. Pairing is local-user authorized; device key storage is platform protected; grants are bounded/revocable and revalidated at each protected request.

**Push** is optional convenience infrastructure. APNs/FCM payloads are opaque wake/sync hints only and contain no raw prompt/answer/approval/effect/file/memory/secret/screenshot content. Push delivery order/reliability is not canonical state. Strict-local disables cloud push.

**Voice** begins with push-to-talk/voice notes and governed ASR/TTS. Voice content is input, never identity or approval. Agent-initiated mic/camera access stays deny-by-default. Full-duplex call mode is evidence-gated after basic mobile reliability.

**Channel bridges** use one typed adapter/envelope boundary but preserve provider truth. Channel bindings use provider-stable IDs, never usernames/display names. Group/unbound participants have zero machine authority by default. Free-form `yes`/emoji/reaction/voice never authorizes consequential work. High-risk approvals step up to Native Mobile or an authenticated local client.

Official paths are binding defaults: Telegram Bot API; WhatsApp Business Platform/Cloud API rather than WhatsApp Web scraping; sanctioned WeChat/WeCom APIs rather than unsupported personal account automation; official Slack/Discord app APIs; Matrix as an open/self-hostable candidate. Every adapter publishes current identity/delivery/media/retry/privacy/relay/account constraints.

Screen/media, input, clipboard, file transfer, reconnect, multi-monitor, visible indicator, local emergency stop and human takeover are independently permissioned through native GolamConnect. Human takeover suspends agent/other-controller input at lease level. Reconnect fully reauthenticates/revalidates; it is not a new auth path.

See `program-amendments/PA-001-phone-channel-access.md`, `program-amendments/PA-001-provider-research.md`, and `contracts/phone-channel-access-contract.md` for the normative expansion.

### 13. Workers / Automations

Workers use typed Rust supervision, narrow child leases, explicit workspace/worktree isolation, spawn/join/cancel/crash-adopt semantics, and bounded budgets. Single-worker reliability precedes swarm/collaboration complexity. Groups and teach-by-demonstration are late Spec 008/009 work.

Phone/channel events may become worker/scheduler triggers only through typed policy-bound trigger definitions; receiving a message is not blanket automation authority. Worker/channel notifications remain ordinary external effects.

### 14. Verification / Receipts

Receipts minimize content while reporting profiles/models, locality, tools, files changed, network destinations, secret handles used, effects, approvals, verification, trace and integrity binding. No pass claim without exact-head evidence.

## Program Decomposition

Keep current order:

- **002 Kernel & Durable Session Spine** — initial Rust workspace; kernel API boundary; authenticated local IPC; session/fork/goal ledger; effect tx + handler/reconcile; audit integrity; bootstrap deny-by-default `Authorize`; BS-1/BS-2/BS-10 foundations.
- **003 Identity, Policy, Secrets & Sandbox** — Cedar/capabilities, protected resources, approvals, taint algebra, secret fallback/redaction, egress policy, sandbox profiles.
- **004 Harness & Local Intelligence** — harness, mistral.rs/llama.cpp adapters, calibration, expanded ExecutionProfile, early model/harness separation benchmarks.
- **005 Local Tools, Context & Memory** — filesystem/shell/git/browser, L0/L1 context, Markdown brain/governance, skills/MCP/ACP; injection/memory/no-egress gates. L2 graph intelligence justification-gated.
- **006 Desktop & Computer Control** — Tauri, semantic OS adapters, platform matrix, human takeover, vision fallback.
- **007 Phone, GolamConnect & Channel Access** — Connect Core/device pairing/Iroh; iOS/Android Golam Mobile; privacy-minimized push; mobile approval; push-to-talk/voice notes; native screen/input/files/clipboard/reconnect/control generations; common channel envelope; Telegram first; official WhatsApp Business and WeChat/WeCom paths; Slack/Discord/Matrix candidates; adversarial channel/mobile tests.
- **008 Workers & Automations** — scheduler, durable workers, triggers, worktrees, bounded parallelism; policy-bound phone/channel triggers and outbound notifications consume Spec 007 contracts.
- **009 Grok Public Parity** — complete evidence ledger and remaining independently implemented public domains/skills.
- **010 GolamBench & Release Qualification** — full long-horizon, computer, memory, security, offline/privacy, recovery, phone/mobile, channel, remote-control and parity qualification.

Specs 007/008 may swap if implementation evidence makes that useful. A2A federation, image/video generation, custom relay, multi-device CRDT memory sync, and always-on background wake-word/autonomous microphone listening remain deferred through 010 unless a later reviewed spec explicitly changes scope.

## Grok Public Parity Domains

Initial ledger includes persistent agents/workers; persistent computer/workspace; files/terminal/browser/apps; parallel/background work; memory; approvals/security; local computer execution; desktop/native-mobile/channel continuity; skills/plugins/MCP/connectors; routines/schedules; teach-by-demonstration; groups/handoffs; rich artifacts; Documents/Presentations/Spreadsheets/PDFs/Skill Creator; multimodal document/image/PDF input; deep research with citations; search/web connectors; phone push-to-talk/voice-note continuity. Image/video generation and always-on voice/wake-word behavior remain deferred or `NOT_APPLICABLE_WITH_RATIONALE` until a later spec.

Parity evidence is public-behavior/scenario based. No Grok internals/assets/prompts/reconstruction enter Golam.

## Incremental GolamBench Gates

Do not defer safety evidence to Spec 010.

- **BS-1** crash/replay/fork/checkpoint/disk failure starts in 002.
- **BS-2** duplicate-effect/UNKNOWN_OUTCOME starts in 002.
- **BS-10** strict-local externally observed no-egress starts in 002/003 and remains a regression gate.
- IPC compromise/kernel-boundary probes start in 002.
- Taint/prompt-injection/secret canaries start in 003 and expand in 005.
- Model/harness separation begins in 004.
- Memory poisoning/FORGET starts in 005.
- Computer-control safety starts in 006.
- Connect replay/revocation/takeover/channel impersonation starts in 007.
- Native mobile adds device-key/reconnect/stale-approval/push-content-leak/voice-approval-isolation gates in 007.
- Provider adapters add webhook/stream/poll duplicate/reorder/edit/delete/outage/rate-limit/attachment/loop gates in 007.
- Spec 010 aggregates full release qualification.

## Donor Strategy After GLM Review

Classifications are not code-admission approvals. Source Foundry exact qualification remains required.

- Iroh: `DIRECT_DEPENDENCY` candidate for Connect transport.
- RASystem: `SELECTIVE_PORT` candidate; independently qualify grants/control/audit/media and Windows/Linux behavior.
- Cedar: `DIRECT_DEPENDENCY` candidate for policy evaluation; Golam owns semantics.
- Wasmtime: `DIRECT_DEPENDENCY` candidate when executable portable extensions are introduced.
- mistral.rs: `DIRECT_DEPENDENCY` candidate behind model adapter.
- llama.cpp: `ADAPTER`, preferred sidecar.
- grok-build/goose: `SELECTIVE_PORT` or reference after exact qualification.
- DeepSeek Harness/OpenBot: `REFERENCE_ONLY` architecture/UX patterns.
- winappCli: `REFERENCE_ONLY` behavioral spec unless a later admission justifies code use.
- agent-desktop: `SELECTIVE_PORT` candidate for snapshot/ref concepts only after exact license/commit qualification.
- RustDesk/OpenControl/other reciprocal sources: code `REJECT`/behavior `REFERENCE_ONLY` by default.
- Graphify/code-graph-rag: optional adapters; no mandatory graph DB.
- Restate/Temporal: durability pattern references only; no server dependency.
- Golam-Research/Grok reconstruction: code `REJECT`, behavioral evidence only.

Phone/mobile/channel architecture references such as Tauri mobile, Telegram/Meta/WeCom/Slack/Discord/Matrix official APIs and any donor client/channel implementations remain reference candidates until the exact Spec 007 Source Foundry/dependency qualification is performed.

## Complexity Discipline

Accepted: small privileged kernel, canonical event/effect model, platform-specific control, native Connect protocol, first-class mobile client, bounded official messaging adapters.

Binding simplifications: <=8 initial crates; no custom relay; no mandatory L2 graph; no CRDT memory in P0; no huge swarm; no executable skill runtime before sandbox profile; no always-on background microphone/wake-word through Spec 010; no unofficial WhatsApp Web/personal WeChat automation; no A2A/media-generation scope creep through Spec 010; no wholesale donor forks.

## Phase Exit Criteria for Spec 001

Spec 001 may be frozen and program `tasks.md` generated only when:

1. GLM review result/finding ledger is committed;
2. BLK-001/BLK-002 are resolved in normative artifacts;
3. MAJ-001..008 are incorporated or founder-waived (current decision: incorporated, no waivers);
4. mandatory strict-local egress requirement is normative;
5. constitution/spec/plan/data model/contracts/checklist are cross-artifact consistent;
6. finalization status records the GLM source-tail truncation honestly;
7. live branch head is reverified;
8. `tasks.md` authorizes only bounded next Spec Kit work, not an unreviewed all-product implementation.

PA-001 is an additive post-freeze founder-scope amendment. Its own review/merge evidence governs the mobile/voice/channel expansion and does not retroactively alter the historical Spec 001 closeout evidence.