# Feature Specification: Golam Local Agent OS Foundation

**Feature Branch**: `spec/001-golam-local-agent-os-foundation`  
**Created**: 2026-08-24  
**Status**: GLM-5.3 reviewed; mandatory architecture findings reconciled; pending final consistency freeze  
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
- Strict-local egress is mechanically denied outside explicitly local/loopback needs and can be externally verified.
- User can inspect what models, files, tools, and network destinations were used.

### US2 — Use one agent from Desktop and CLI (P0)

As a user, I can start or resume the same Golam session from Desktop or CLI/TUI and see consistent goals, progress, approvals, files/actions, memory, and verification evidence.

**Acceptance**
- Desktop and CLI are authenticated clients of the same local daemon/state.
- Localhost/same-machine presence is never treated as authentication.
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
- Clipboard read is separately gated; camera/microphone are deny-by-default.

### US4 — Reach and control Golam remotely (P0/P1)

As a user, I can securely reach my Golam from another trusted device and, when explicitly allowed, view/control my laptop remotely or ask Golam to act on it.

**Acceptance**
- Native GolamConnect uses device identity, signed requests, end-to-end encrypted transport, NAT traversal, relay fallback, short-lived capability grants, replay protection, and host-side per-message enforcement.
- Remote screen, keyboard/mouse, clipboard, files, multi-monitor, reconnect, generation-based control arbitration, human takeover, and emergency stop have explicit contracts.
- Telegram/WhatsApp/Slack/Discord are command/notification adapters, not the trust root or native remote-desktop transport.
- Channel bindings use provider-stable identifiers, never usernames/display names.
- Third-party channel privacy is never mislabeled as strict local.

### US5 — Own durable memory (P0)

As a user, I can inspect and edit Golam's long-term knowledge as Markdown, while Golam maintains derived search/graph/vector indexes that can be rebuilt.

**Acceptance**
- Markdown is canonical long-lived knowledge.
- SQLite is canonical operational state.
- Memory tracks provenance, time validity, confidence/authority, owner/scope, and supersession/contradiction.
- ADD/UPDATE/SUPERSEDE/CONTRADICT/MERGE/EXPIRE/FORGET/REDACT have governed semantics.
- Golam-generated managed-vault writes use a single governed writer; user hand-edits are detected/reconciled.
- FORGET/REDACT clears affected derived indexes/caches.
- Live state outranks stale memory.

### US6 — Use the best model for my machine and task (P0)

As a user, Golam detects my hardware, recommends local inference profiles, and routes tasks through explicit `ExecutionProfile`s.

**Acceptance**
- Model/revision, tokenizer/chat template, runtime, locality, quantization, hardware mapping, harness, reasoning mode, tool-call conformance, context/cache strategy, warm residency, workload class, sampling, multimodal capability, resource/latency/quality budgets, and privacy/network policy are separable.
- Local model support is first-class; cloud providers are optional and explicit.
- Router decisions are inspectable and overrideable.

### US7 — Install and create governed skills (P1)

As a user, I can use built-in Golam skills, Agent Skills-compatible packages, MCP tools, and custom skills without allowing a skill to silently grant itself authority.

**Acceptance**
- Skill provenance, hash/version, license, requested capabilities, scripts, dependencies, and test status are tracked.
- Install/upgrade is reviewable and lockable.
- Executable skill scripts/MCP servers run under explicit sandbox profiles.
- Golam provides independently implemented equivalents for publicly documented Grok Bot built-in skills: Documents, Presentations, Spreadsheets, PDFs, and Skill Creator.

### US8 — Run durable workers and automations (P1)

As a user, I can create named workers with roles, memory loadouts, capabilities, schedules/triggers, and evaluation records, and let them resume after crashes/reboots.

**Acceptance**
- Workers cannot expand their own capability set.
- Scheduled/event work produces normal event/effect/audit records.
- Long-running work is checkpointed and recoverable.
- Spawn/join/cancel/crash-adopt semantics and capability inheritance are explicit.

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
- **FR-007**: System MUST propagate trust/taint provenance through retrieved/model-generated derivatives and generated artifacts.
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

### GLM-5.3 mandatory reconciliation requirements

- **FR-024 Kernel Boundary**: System MUST distinguish the Rust trusted path from a smaller privileged kernel. Kernel-owned authority state MUST be protected from generic filesystem/tool/plugin/worker writes. Authority-bearing types/tokens MUST be constructible only by the privileged kernel API, which MUST be process-splittable.
- **FR-025 Local IPC Authentication**: Every local `golamd` client MUST authenticate. Windows named-pipe/ACL and Unix-domain-socket/peer-credential protections (or stronger equivalents) are required. `golamd` MUST NOT expose unauthenticated control listeners, including loopback HTTP.
- **FR-026 Effect Handlers**: Every effect family MUST declare execution semantics, stable idempotency behavior where applicable, `execute`, safe/read-only `reconcile`, timeout/ambiguity policy, and evidence. Intent MUST be durably persisted before execution. AT_MOST_ONCE/IRREVERSIBLE ambiguity MUST NOT blind-retry, and dependent effects MUST block on UNKNOWN_OUTCOME.
- **FR-027 Protected Authority Resources**: Policy, principals, leases, approvals, Connect pairing, secret state, effect journal, audit chain, schedule authority, and skill-lock/admission state MUST be protected resources. Mutating them is an elevated effect requiring policy plus appropriate user approval.
- **FR-028 Taint Downgrade**: Taint MAY be downgraded only by explicit human approval or deterministic pre-registered authoritative verification. Model/worker/skill/MCP assertions cannot self-clear taint. Artifacts inherit taint and SECRET_DERIVED content MUST NOT enter long-term canonical memory.
- **FR-029 Secret Fallback**: Unbrokerable secret use MUST require bounded user-approved injection into an isolated execution channel, never argv; ambient inheritance is denied; output/log redaction is mandatory. User-pasted secrets MUST be redacted/tombstoned at ingestion; vault storage is encrypted at rest; screenshot/support-bundle retention is minimized.
- **FR-030 Memory Governance**: System MUST implement the governed memory operation semantics defined in `memory-governance-contract.md`, including single-writer Golam mutation, external-edit reconciliation, promotion rules, contradiction surfacing, and FORGET/REDACT derived-state rebuild.
- **FR-031 Ledger/Fork/Integrity**: Session forks MUST reference immutable parent prefixes; cross-session causality/audit ordering MUST be explicit; security-critical event families MUST have mandatory integrity chaining/authentication; large artifacts are content-addressed with retention/GC and checkpoint rules.
- **FR-032 Stable Channel Binding**: Third-party channel binding MUST use provider-stable identifiers and explicit local user authorization/revocation. Group/unbound senders hold zero machine authority by default.
- **FR-033 Approval Classes**: System MUST define ONCE, session/time/pattern-scoped, and run-preauthorization classes with freshness, bounds, and revocation. Unattended IRREVERSIBLE effects require explicit bounded per-run preauthorization.
- **FR-034 Strict-Local Egress**: Strict-local network denial MUST be mechanized at the kernel egress choke point across all Golam-managed processes. Unexpected egress attempts are denied/audited and any incompatible component fails clearly instead of downgrading privacy.
- **FR-035 Sandbox Profiles**: MCP servers, executable skill scripts, browser/protocol helpers, optional language adapters, and sidecars MUST run under declared sandbox/supervision profiles with explicit FS/network/env/process budgets; outputs retain untrusted provenance.

## Non-Functional Requirements

- **NFR-001 Locality**: strict-local core works without cloud services and is externally testable for no unexpected egress.
- **NFR-002 Portability**: Windows, macOS, Linux are product targets; platform-specific limitations must fail explicitly.
- **NFR-003 Security**: no hidden control, silent privilege escalation, self-granted authority, unauthenticated local control surface, or generic write path to protected authority state.
- **NFR-004 Durability**: crash/restart must not corrupt canonical state or blindly duplicate external effects; disk-full/fsync failure fails closed.
- **NFR-005 Auditability**: privileged authority-bearing code remains small enough for focused review; unsafe Rust/FFI and network parser surfaces are isolated and justified.
- **NFR-006 Performance**: agent-facing observations should prefer compact semantic state over large screenshots/DOM dumps; context and prompt-prefix stability are measured.
- **NFR-007 Interoperability**: protocols and adapters remain replaceable; internal authority is not delegated to external frameworks.
- **NFR-008 Licensing**: only qualified compatible donor code may be admitted; proprietary/reconstructed material remains reference-only.
- **NFR-009 Privacy**: relay/channel metadata exposure and screenshot/clipboard/camera/mic sensitivity are explicitly represented; strict-local privacy is never claimed for third-party messaging paths.
- **NFR-010 Recoverability**: vault + SQLite support consistent backup/restore; checkpoints are optional accelerators and canonical replay remains possible.

## Success Criteria

- **SC-001**: A strict-local end-to-end task runs with zero unexpected external network/model calls after assets are present, verified from outside Golam-managed processes.
- **SC-002**: A session survives daemon restart and resumes from canonical state without goal drift in the recovery test corpus.
- **SC-003**: Effect replay tests prove defined behavior for read-only, idempotent-at-least-once, at-most-once, compensatable, and irreversible effects, including crash windows at remote acceptance/ack.
- **SC-004**: Unauthorized worker/channel/skill/model/client capability expansion is denied in adversarial tests, including attempted writes to protected authority state.
- **SC-005**: Desktop semantic-control tests complete representative native-app workflows with pixel/vision fallback only when semantic paths are unavailable and fail closed on protected/locked surfaces.
- **SC-006**: GolamConnect reconnect/lease/replay/generation-arbitration/emergency-stop tests pass across supported host platforms before remote-control release.
- **SC-007**: User can inspect/edit Markdown memory, surface contradictions, execute FORGET/REDACT, and rebuild derived indexes from canonical data.
- **SC-008**: ExecutionProfile benchmark records model/harness/runtime/resource/cache/warm-residency/workload metrics separately.
- **SC-009**: Public Grok Bot parity claims reference scenario evidence; unimplemented items remain visibly incomplete.
- **SC-010**: Unauthenticated/revoked local clients cannot issue daemon commands and no unexpected control listener is exposed.
- **SC-011**: Canary-secret tests prove secret values do not appear in prompts, durable logs, memory, support bundles, or unauthorized subprocess outputs; approved unbrokerable paths are bounded/redacted.
- **SC-012**: Taint survives web/MCP/channel -> summary -> memory/artifact -> later effect unless an auditable allowed downgrade occurs.

## Out of Scope for Spec 001 Implementation

Spec 001 defines the governed platform foundation and implementation sequence. It does not authorize implementing the complete product in one change. Application code begins only after this reconciled package is consistency-checked/frozen and `tasks.md` explicitly starts the next bounded Spec Kit slice.
