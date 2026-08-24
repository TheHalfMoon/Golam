# Implementation Plan: Golam Local Agent OS Foundation

**Branch**: `spec/001-golam-local-agent-os-foundation` | **Date**: 2026-08-24  
**Spec**: `spec.md`  
**Status**: `PENDING_EXTERNAL_GLM_5_3_REVIEW` — technically planned, not frozen for implementation

## Summary

Build Golam as a clean Rust-first local Agent OS with a small privileged kernel, local canonical state, replaceable model/harness/tool adapters, semantic-first computer control, user-owned Markdown memory, and secure native remote control through GolamConnect.

Spec 001 does not authorize implementing the entire product in one PR. It establishes the platform spine and decomposes implementation into bounded follow-on specifications.

## Technical Context

**Language**: Rust stable for trusted/runtime code; TypeScript/React only for untrusted Tauri renderer; optional Python/Node adapters outside trusted path.  
**Runtime**: Tokio-based local daemon; exact crate choices pinned only after donor/dependency qualification.  
**Desktop**: Tauri 2 + React/TypeScript renderer, Rust backend.  
**Local state**: SQLite (WAL where appropriate) for operational state; Markdown/files for canonical human knowledge; content-addressed artifacts where useful.  
**Authorization**: Cedar candidate plus Golam capability/effect schema.  
**Extension sandbox**: Wasmtime/WASI candidate for bounded untrusted extensions; OS/native sandbox backends for native execution.  
**Local inference**: `mistral.rs` primary candidate; `llama.cpp` compatibility backend; other adapters optional.  
**Networking**: Iroh/QUIC candidate for native GolamConnect; encrypted relay fallback.  
**Protocols**: ACP stable v1 via current Rust SDK line; MCP 2026-07-28 semantics; Agent Skills-compatible packages; A2A later for external federation.  
**Testing**: `cargo test --workspace`, property tests for state/policy/effects, fuzzing for protocol parsers/state machines, integration tests with synthetic/fake adapters, platform on-device tests, benchmark harness.  
**Target platforms**: Windows 11, macOS current supported releases, major Linux desktop environments; explicit capability matrix.  
**Project type**: local daemon + CLI/TUI + desktop app + protocol/adapters + optional relay/mobile clients.  
**Performance goals**: local control loop responsive enough for interactive desktop use; compact semantic observations; bounded process output; measured prompt/cache efficiency; remote-control media target set per platform during implementation qualification.  
**Constraints**: strict-local mode, no hidden cloud dependency, small trusted computing base, crash-safe state, reversible/attributable effects, user-interruptible control.  
**Scale/scope**: single-user local-first P0 with multiple devices/workers/sessions; architecture must not require multi-tenant cloud infrastructure.

## Constitution Check

| Gate | Result | Plan evidence |
|---|---|---|
| Local ownership/trust root | PASS | local daemon/state/model paths; cloud optional |
| Rust trusted path | PASS | trusted components are Rust; TS only renderer |
| Explicit authority | PASS | identity/capability/Cedar/effect model |
| Gated durable effects | PASS | effect transaction contract + ledger |
| User-owned governed memory | PASS | Markdown canonical, SQLite operational |
| Replaceable model/harness | PASS | ExecutionProfile abstraction |
| Semantic-first control | PASS | DesktopController hierarchy + takeover |
| Open protocols/governed skills | PASS | ACP/MCP/Skills/A2A boundaries |
| Clean-room donor governance | PASS | Source Foundry and no reconstructed imports |
| Verification over claims | PASS | exact evidence and GolamBench required |

**External gate not yet passed**: GLM 5.3 architecture review.

## Architecture

```text
                    Desktop / CLI / IDE / Mobile / Channels
                                   |
                              Golam Gateway
                                   |
                                golamd
                                   |
              +--------------------+--------------------+
              |                    |                    |
        Session/Event         Goal Ledger           Scheduler
              |                    |                    |
              +--------------------+--------------------+
                                   |
                            Harness Runtime
                                   |
             +---------------------+----------------------+
             |                     |                      |
      Context Compiler        Memory Brain           Skills/Tools
             |                     |                      |
             +---------------------+----------------------+
                                   |
                            ExecutionProfile
                     local inference / optional cloud
                                   |
                              Effect Intent
                                   |
                  +--------------------------------+
                  |       TRUSTED RUST KERNEL      |
                  | identity / policy / capability |
                  | taint / effect tx / secrets    |
                  | audit / receipts / authority   |
                  +---------------+----------------+
                                  |
              +-------------------+--------------------+
              |                   |                    |
          Sandbox            Computer Control      GolamConnect
              |                   |                    |
      WASI/native/remote     API/a11y/DOM/input      Iroh/QUIC
              |                   |               screen/input/files
              +-------------------+--------------------+
                                  |
                          USER'S COMPUTERS
```

## Core invariants

- `MODEL_VISIBLE => LOGGED`
- `NO_EXTERNAL_EFFECT_WITHOUT_EFFECT_GATE`
- `AGENT_CANNOT_EXPAND_OWN_AUTHORITY`
- `CHANNEL != AUTHORITY`
- `UNTRUSTED_DATA != INSTRUCTION_AUTHORITY`
- `SAFETY_DENIAL_IS_MONOTONIC`
- `MEMORY != TRUTH`
- `FULL_CANONICAL_HISTORY_SURVIVES_COMPACTION`
- `REAL_SECRETS_STAY_OUT_OF_MODEL_CONTEXT_WHEN_BROKERABLE`
- `EVERY_WRITE_IS_ATTRIBUTABLE`
- `EVERY_LONG_RUN_IS_CRASH_RESUMABLE`

## Rust workspace target

Start with a bounded spine; split only when ownership/testing boundaries are real.

```text
crates/
  golam-kernel
  golam-events
  golam-session
  golam-policy
  golam-effects
  golam-secrets

  golam-harness
  golam-models
  golam-context
  golam-memory
  golam-skills
  golam-workers

  golam-tools
  golam-browser
  golam-control
  golam-sandbox

  golam-connect
  golam-connect-protocol
  golam-connect-transport

  golam-acp
  golam-mcp
  golam-audit
  golam-bench

apps/
  golamd
  golam
  golam-desktop
```

OS-specific control/media/input crates SHOULD be split later when platform implementations become substantial, rather than creating empty abstractions up front.

## Component decisions

### 1. `golamd`

Long-lived local authority. Owns canonical sessions, scheduler, policy, worker lifecycle, model/runtime registry, local IPC, Connect endpoints, and audit. Clients are replaceable projections.

### 2. Session/Event Ledger

Append-oriented typed events. Chat transcript, compacted context, UI timeline, and audit views are projections. Recovery replays canonical events and checkpoints. Event schemas are versioned.

### 3. Goal Ledger

High-priority durable goal state outside ordinary compaction:
- goal;
- acceptance criteria;
- non-negotiable constraints;
- scope;
- proven facts;
- authoritative current-state refs;
- blockers;
- completed work;
- next safe action.

### 4. Effect Gateway

Every consequential effect is proposed then authorized then executed then verified/receipted. Effect semantics include:
- read-only;
- idempotent at-least-once;
- at-most-once;
- compensatable;
- irreversible.

The ledger records intent before execution and completion/evidence after execution. Ambiguous crash windows are resolved through effect-specific reconciliation rather than blind retry.

### 5. Identity/Policy/Capabilities

Principals: User, Device, Worker, Skill, Channel, Service, MCPServer, ExternalAgent. Policy requests are `(principal, action, resource, context)`. Capability leases can expire and narrow, never widen parent authority. Risk-based step-up approvals are supported.

### 6. Secret Broker

Secrets are references/handles. Where possible, credentials are injected at an egress/client boundary rather than exposed to model/tool process. OS keychain/keyring adapters are candidates. Logs and receipts record use metadata, never secret values.

### 7. Information-flow labels

Initial trust labels include user/local trusted, local unverified, web untrusted, channel untrusted, MCP/plugin untrusted, model generated, and secret-derived. Derived content inherits source taint unless independently verified under an explicit rule.

### 8. Harness Runtime

Provider/model-independent agent loop with explicit model-visible history, typed tools, bounded context, cancellation, retries, compaction/reset strategy, checkpointing and evaluator separation. DeepSeek Harness/Grok Build/Goose patterns are references, not semantic dependencies.

### 9. ExecutionProfile Router

An ExecutionProfile binds model + inference backend + quantization + harness + context/cache strategy + sampling + tool schema mode + resource policy. Hardware calibration produces candidate local profiles. Routing is observable and user-overridable.

### 10. Context Compiler

Evidence pipeline:

`intent -> evidence requirements -> source routing -> retrieve -> authority/time/permission filter -> rank -> sufficiency check -> replan -> ContextCapsule`

Coding evidence tiers:
- L0: filesystem, ripgrep, git;
- L1: Tree-sitter, LSP, structural graph;
- L2: deep semantic/dataflow/runtime analysis when justified.

Prompt-cache planner preserves stable prefixes/tool ordering when beneficial.

### 11. Memory Brain

Canonical Markdown vault plus SQLite operational index/state. Memory assets include Working, Run, Project, User, and Verified Repository Knowledge. Every promoted memory has provenance/scope/time/authority metadata. Graph/vector/search indexes are rebuildable.

### 12. Skills OS

Agent Skills-compatible `SKILL.md` as interoperability surface. Lifecycle: discover -> provenance/license -> normalize -> infer capabilities -> scan -> sandbox -> test -> benchmark -> sign -> lock -> install -> upgrade/deprecate. Skill metadata cannot grant authority.

### 13. Browser

Prefer browser protocol/DOM/accessibility over vision. Maintain user-controlled browser profiles. No credential scraping. Downloads/uploads and external form submissions are effects with policy/evidence.

### 14. Computer Control

`Domain API -> Native OS API -> Accessibility -> Browser DOM -> Input Injection -> Vision`.

Observation returns compact state + stable refs when possible. Actions use before/expected/after/verification state. Platform adapters must fail explicitly on locked desktops/UAC/secure surfaces where injection is impossible.

### 15. GolamConnect

Native Connect is a protocol, not a Telegram bot. Devices pair cryptographically. Requests are signed and replay-protected. Host is authority. Transport uses Iroh/QUIC candidate with direct path and encrypted relay fallback. Remote control adds screen/media, input, clipboard/files, multi-monitor, reconnect, lease renewal, visible indicator, takeover and emergency stop.

Telegram/WhatsApp/Slack/Discord normalize messages into untrusted or authenticated channel requests mapped to principals. They never bypass policy.

### 16. Workers/Automations

Worker definition includes version, behavior contract, capability manifest, memory loadout, harness profile, evaluation record and signature/provenance. Internal scheduling uses typed Rust state/DAGs. Bounded parallel workers get isolated workspaces/worktrees/sandboxes where needed.

### 17. Verification/Receipts

Significant task receipt reports models, local/cloud calls, tools, files changed, network destinations, approvals, secret handles used, external effects, tests/verifications and trace ID. Content exposure is minimized.

## Program decomposition into follow-on Spec Kit features

Implementation should proceed as separately reviewable specs:

- **002 Kernel & Durable Session Spine** — Rust workspace, events/session, goal ledger, effect tx, audit.
- **003 Identity, Policy, Secrets & Sandbox** — Cedar/capabilities, taint, credential broker, Wasmtime/native sandbox interfaces.
- **004 Harness & Local Intelligence** — model adapters, mistral.rs/llama.cpp, hardware calibration, ExecutionProfile, context/cache base.
- **005 Local Tools, Context & Memory** — filesystem/shell/git/browser, context compiler, Markdown brain, skills/MCP/ACP.
- **006 Desktop & Computer Control** — Tauri app, semantic control adapters, app/window state, human takeover, vision fallback.
- **007 GolamConnect** — pairing, Iroh transport, remote screen/control/files/clipboard/reconnect, Telegram and other channel bridges.
- **008 Workers & Automations** — scheduler, durable workers, triggers, worktrees, bounded parallelism.
- **009 Grok Public Parity** — complete parity ledger, built-in skill equivalents, routines/teach-by-demonstration/groups/connectors/artifacts and remaining product behaviors.
- **010 GolamBench & Release Qualification** — long-horizon, computer, memory, security, offline/privacy, recovery and parity qualification.

Each later spec must run clarify/plan/checklist/tasks/analyze independently and remain bounded.

## Grok Bot parity strategy

Parity is black-box/public-behavior oriented:

`public feature -> Golam scenario -> expected evidence -> status`.

Do not preserve Grok implementation details, trademarks, UI assets, private skill prompts, or reconstructed source. Equivalent capability may use a different local architecture.

Initial parity domains:
- persistent named agents/workers;
- persistent computer/workspace state;
- files/terminal/browser/app use;
- parallel agents and collaboration;
- long-running/background work;
- memory and continuity;
- approvals/security controls;
- local computer execution;
- Desktop/mobile/channel continuity;
- skills/plugins/MCP/connectors;
- routines/schedules;
- teach-by-demonstration;
- groups/handoffs;
- artifacts/files;
- built-in Documents, Presentations, Spreadsheets, PDFs, Skill Creator equivalents.

## Testing strategy

- Unit tests for typed state and deterministic logic.
- Property tests for event replay, policy monotonicity, capability narrowing, idempotency state machines and parsers.
- Fuzz protocol/event/skill/package/parsing surfaces.
- Synthetic adapters for deterministic CI.
- On-device Windows/macOS/Linux control tests.
- Two-machine GolamConnect tests including NAT/reconnect/relay conditions.
- Fault injection at every durable-effect boundary.
- Security adversarial tests for prompt injection, tool poisoning, stale refs, replay, capability escalation, confused deputy, SSRF and secret exfiltration.
- Exact-head benchmark artifacts and machine metadata.

## GolamBench dimensions

External suites are selected at implementation time based on availability/license and supplemented with Golam-native scenarios. Required dimensions:

- long-horizon coding and terminal work;
- whole-repository implementation/refactoring;
- computer-use workflows;
- memory/state/workflow recall;
- prompt injection/tool poisoning;
- crash/reboot/resume;
- effect idempotency;
- stale memory resistance;
- goal retention after compaction/reset;
- remote-control reconnect/takeover/emergency-stop;
- strict-local/no-egress proof;
- context bytes, cache hit ratio, tokens, wall time, resource use, interventions and verified success.

## Source Foundry implementation rule

Do not start by forking all donors. For each candidate:

1. pin exact source state;
2. qualify rights/dependencies/security;
3. build adapter or benchmark first where practical;
4. measure value;
5. import/port only the smallest justified surface;
6. preserve notices and modification records;
7. own Golam-facing contracts so donors stay replaceable.

## Complexity tracking

Accepted complexity:
- separate privileged kernel because pluggable-everything would weaken security;
- canonical event/effect model because durable autonomy requires replay/reconciliation;
- platform-specific computer-control implementations because OS semantics materially differ;
- native Connect protocol because messaging bridges cannot safely represent full remote control.

Rejected premature complexity:
- mandatory graph database;
- custom universal agent DSL;
- A2A for internal workers;
- cloud control plane required for local use;
- huge swarm before single-worker reliability;
- wholesale donor forks;
- mandatory container/runtime for all tools.

## Phase exit criteria for Spec 001

Spec 001 may be frozen and `tasks.md` generated only when:

1. founder accepts constitution/spec/research/plan/contracts;
2. GLM 5.3 external review is completed with no unresolved BLOCKER finding;
3. MAJOR findings are either incorporated or explicitly founder-waived with rationale;
4. finalization status is changed to `READY_FOR_TASK_GENERATION`;
5. repository live head is re-verified before any follow-up mutation.
