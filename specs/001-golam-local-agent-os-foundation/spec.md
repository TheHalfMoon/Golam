# Feature Specification: Golam Local Agent OS Foundation

**Feature Branch**: `spec/001-golam-local-agent-os-foundation`  
**Created**: 2026-08-24  
**Status**: Planning complete; external GLM 5.3 review pending  
**Input**: Build Golam as a Rust-first, local-first desktop + CLI Agent OS that can replace separate coding, research, browser, personal-assistant, automation, and remote-control agents for normal daily work, while preserving local ownership and secure remote access through GolamConnect.

## Product North Star

Golam is the autonomous operating layer for the user's computer.

A user should be able to use one local system for:

- everyday assistant work;
- coding and repository work;
- web research and browser workflows;
- document/PDF/spreadsheet/presentation work;
- file and application control;
- persistent memory;
- scheduled and event-driven automations;
- specialized workers/subagents;
- local model inference and optional explicit cloud models;
- secure remote access and full laptop control through GolamConnect;
- CLI/TUI, Desktop, IDE, and messaging-channel interaction.

The target is functional parity with publicly documented Grok Bot capabilities and skills, implemented independently under Golam's local-first/security constraints—not source or asset cloning.

## User Stories

### US1 — Run a useful agent fully locally (P0)

As a user, I can install Golam on my computer, choose strict-local mode, select or auto-configure a supported local model, and complete useful tasks through CLI or Desktop without a cloud model or Golam-hosted service.

**Acceptance**
- Core daemon, memory, policy, tools, schedules, and audit run locally.
- No outbound network is required after local model/assets are present.
- No hidden fallback sends prompts or files externally.
- User can inspect what models, files, tools, and network destinations were used.

### US2 — Use one agent from Desktop and CLI (P0)

As a user, I can start or resume the same Golam session from Desktop or CLI/TUI and see consistent goals, progress, approvals, files/actions, memory, and verification evidence.

**Acceptance**
- Desktop and CLI are clients of the same local daemon/state.
- Session continuity survives client restarts.
- Long-running work continues independently of the UI when policy permits.

### US3 — Let Golam control the computer (P0)

As a user, I can ask Golam to use applications, browser, terminal, files, clipboard, windows, and developer tools on my computer.

**Acceptance**
- Semantic/API/accessibility control is preferred over pixels.
- Keyboard/mouse/vision fallback exists for otherwise inaccessible UI.
- Every consequential action passes policy/effect gating.
- User can pause, stop, or take control immediately.
- Sensitive applications/resources can be blocked or scoped.

### US4 — Reach and control Golam remotely (P0/P1)

As a user, I can securely reach my Golam from another trusted device and, when explicitly allowed, view/control my laptop remotely or ask Golam to act on it.

**Acceptance**
- Native GolamConnect uses device identity, signed requests, end-to-end encrypted transport, NAT traversal, relay fallback, short-lived capability grants, replay protection, and host-side per-message enforcement.
- Remote screen, keyboard/mouse, clipboard, files, multi-monitor, reconnect, human takeover, and emergency stop have explicit contracts.
- Telegram/WhatsApp/Slack/Discord are command/notification adapters, not the trust root or native remote-desktop transport.
- Third-party channel privacy is never mislabeled as strict local.

### US5 — Own durable memory (P0)

As a user, I can inspect and edit Golam's long-term knowledge as Markdown, while Golam maintains derived search/graph/vector indexes that can be rebuilt.

**Acceptance**
- Markdown is canonical long-lived knowledge.
- SQLite is canonical operational state.
- Memory tracks provenance, time validity, confidence/authority, owner/scope, and supersession.
- Live state outranks stale memory.

### US6 — Use the best model for my machine and task (P0)

As a user, Golam detects my hardware, recommends local inference profiles, and routes tasks through explicit `ExecutionProfile`s.

**Acceptance**
- Model, runtime, quantization, harness, context/cache strategy, tool grammar, sampling, and resource budget are separable.
- Local model support is first-class; cloud providers are optional and explicit.
- Router decisions are inspectable and overrideable.

### US7 — Install and create governed skills (P1)

As a user, I can use built-in Golam skills, Agent Skills-compatible packages, MCP tools, and custom skills without allowing a skill to silently grant itself authority.

**Acceptance**
- Skill provenance, hash/version, license, requested capabilities, scripts, dependencies, and test status are tracked.
- Install/upgrade is reviewable and lockable.
- Golam provides independently implemented equivalents for publicly documented Grok Bot built-in skills: Documents, Presentations, Spreadsheets, PDFs, and Skill Creator.

### US8 — Run durable workers and automations (P1)

As a user, I can create named workers with roles, memory loadouts, capabilities, schedules/triggers, and evaluation records, and let them resume after crashes/reboots.

**Acceptance**
- Workers cannot expand their own capability set.
- Scheduled/event work produces normal event/effect/audit records.
- Long-running work is checkpointed and recoverable.

### US9 — Replace separate specialist agents in daily work (P1/P2)

As a user, I can use Golam rather than separate coding, research, browser, personal-assistant, memory, automation, and local-model frontends for normal tasks.

**Acceptance**
- Feature-parity ledger tracks publicly documented target behaviors.
- Parity requires scenario evidence, not UI resemblance or copied internals.
- Missing parity remains explicit rather than being marketed as complete.

## Functional Requirements

- **FR-001**: System MUST provide a long-lived local Rust daemon (`golamd`) as canonical runtime authority.
- **FR-002**: System MUST provide Rust CLI/TUI (`golam`) and a Tauri Desktop client using the same daemon.
- **FR-003**: System MUST maintain an append-oriented canonical event/session ledger and separate immutable-priority Goal Ledger.
- **FR-004**: System MUST model every consequential external effect as a typed transaction with retry/idempotency semantics.
- **FR-005**: System MUST enforce identity/capabilities/policy outside model reasoning and deny by default where authority is absent.
- **FR-006**: System MUST provide a credential broker that avoids placing real secrets in model context or untrusted execution environments when brokered use is possible.
- **FR-007**: System MUST propagate trust/taint provenance through retrieved/model-generated derivatives.
- **FR-008**: System MUST support sandboxed execution and bounded subprocess supervision.
- **FR-009**: System MUST provide local inference via at least one Rust-native engine plus a broad compatibility backend.
- **FR-010**: System MUST provide hardware calibration and `ExecutionProfile` routing.
- **FR-011**: System MUST provide context compilation over files, search, syntax, LSP, git/history, optional code graphs, optional semantic retrieval, and external evidence when allowed.
- **FR-012**: System MUST provide user-owned Markdown memory with SQLite operational indexes/state.
- **FR-013**: System MUST provide filesystem, terminal/process, git, browser, and desktop-control capabilities.
- **FR-014**: Computer control MUST use semantic-first hierarchy with deterministic refs/state verification and vision fallback.
- **FR-015**: System MUST support ACP stable wire v1 through a current Rust SDK and MCP 2026-07-28 semantics with compatibility handling as needed.
- **FR-016**: System MUST support Agent Skills-compatible packaging and governed skill lifecycle.
- **FR-017**: System MUST provide durable workers, scheduler, checkpoints, and bounded parallelism.
- **FR-018**: System MUST provide GolamConnect native device pairing, signed command envelopes, P2P/relay transport, remote screen/control, reconnect, and takeover semantics.
- **FR-019**: System MUST support third-party channel adapters including Telegram and a standards-compliant WhatsApp path without making unofficial account automation a core dependency.
- **FR-020**: System MUST produce execution/privacy receipts for significant tasks.
- **FR-021**: System MUST expose observability/replay without requiring cloud telemetry.
- **FR-022**: System MUST maintain a Grok Bot public feature/skill parity ledger with evidence states.
- **FR-023**: System MUST have benchmark suites for long-horizon coding, computer use, memory, security, crash/recovery, remote control, and local/offline behavior.

## Non-Functional Requirements

- **NFR-001 Locality**: strict-local core works without cloud services.
- **NFR-002 Portability**: Windows, macOS, Linux are product targets; platform-specific limitations must fail explicitly.
- **NFR-003 Security**: no hidden control, silent privilege escalation, or self-granted authority.
- **NFR-004 Durability**: crash/restart must not corrupt canonical state or blindly duplicate external effects.
- **NFR-005 Auditability**: security-sensitive trusted code remains small enough for focused review; unsafe Rust/FFI is isolated and justified.
- **NFR-006 Performance**: agent-facing observations should prefer compact semantic state over large screenshots/DOM dumps; context and prompt-prefix stability are measured.
- **NFR-007 Interoperability**: protocols and adapters remain replaceable; internal authority is not delegated to external frameworks.
- **NFR-008 Licensing**: only qualified compatible donor code may be admitted; proprietary/reconstructed material remains reference-only.

## Success Criteria

- **SC-001**: A strict-local end-to-end task runs with zero external model/network calls after assets are present.
- **SC-002**: A session survives daemon restart and resumes from canonical state without goal drift in the recovery test corpus.
- **SC-003**: Effect replay tests prove defined behavior for read-only, idempotent-at-least-once, at-most-once, compensatable, and irreversible effects.
- **SC-004**: Unauthorized worker/channel/skill/model capability expansion is denied in adversarial tests.
- **SC-005**: Desktop semantic-control tests complete representative native-app workflows with pixel/vision fallback only when semantic paths are unavailable.
- **SC-006**: GolamConnect reconnect/lease/replay/emergency-stop tests pass across supported host platforms before remote-control release.
- **SC-007**: User can inspect/edit Markdown memory and rebuild derived indexes from canonical data.
- **SC-008**: ExecutionProfile benchmark records model/harness/runtime/resource/cache metrics separately.
- **SC-009**: Public Grok Bot parity claims reference scenario evidence; unimplemented items remain visibly incomplete.

## Out of Scope for Spec 001 Implementation

Spec 001 defines the governed platform foundation and implementation sequence. It does not authorize implementing the complete product in one change. Application code begins only after review and task generation, then proceeds through bounded follow-on specs.
