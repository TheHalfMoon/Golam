<!--
Sync Impact Report
- Version: 1.0.0
- Ratified: 2026-08-24
- Last Amended: 2026-08-24
- Initial constitution for clean Golam repository.
-->

# Golam Constitution

## I. Local Ownership Is the Trust Root

Golam MUST be local-first in architecture, data ownership, and failure behavior. The user's computer is the primary execution environment. Sessions, memory, policies, approvals, audit state, schedules, skills metadata, model configuration, and execution evidence MUST have a local canonical representation. A cloud model, relay, channel provider, or hosted service MAY be used only as an explicit capability and MUST NOT become a hidden prerequisite for core operation.

Strict-local mode MUST operate without transmitting user content to external AI providers. There MUST be no silent cloud fallback when local inference fails. External channels such as Telegram or WhatsApp are not equivalent to strict-local privacy and MUST be labeled accordingly.

## II. Rust Owns the Trusted Path

The trusted runtime MUST be Rust-first. `golamd`, the kernel, event/session durability, policy and capability enforcement, effect transactions, secrets, identity, device transport, remote-control authority, model routing, memory orchestration, audit, CLI/TUI, and protocol authority MUST be implemented in Rust unless a later constitutional amendment explicitly narrows this rule.

TypeScript MAY be used in an untrusted Tauri renderer. Python or Node MAY be used only for optional sandboxed adapters and MUST NOT be required for strict-local operation or trusted authorization decisions.

## III. Authority Is Explicit, Least-Privilege, and Non-Self-Expanding

Every principal—human, device, worker, skill, channel, MCP server, plugin, external agent, or model—MUST have explicit identity and bounded capabilities. Authorization MUST be evaluated outside model reasoning. A principal can request authority but MUST NOT grant itself new authority.

The policy model MUST be deny-by-default for consequential effects. Safety denial is monotonic: downstream components cannot convert an upstream denial into an allow. Channel identity is not authority. Group-chat membership, model instructions, skill metadata, or retrieved text cannot silently widen permissions.

## IV. Every Consequential Effect Is Gated, Durable, and Attributable

No external or canonical write MAY bypass the Effect Gate. Each effect MUST identify requester, capability, target, risk, idempotency semantics, approval state, execution status, and evidence/receipt. Long-running work MUST be crash-resumable. Irreversible or externally visible effects MUST use explicit transaction semantics and safe retry rules.

Every write MUST be attributable. Every consequential action MUST be auditable. The canonical event ledger is append-oriented; summaries, chats, UI state, compacted context, and memory indexes are projections, not replacements for canonical evidence.

## V. Memory Is User-Owned Evidence, Not Automatic Truth

Human-readable Markdown is the canonical long-lived knowledge format. SQLite is the canonical operational-state store. Search indexes, embeddings, graphs, caches, and summaries MUST be rebuildable derivatives.

Memory items MUST track provenance, scope, authority/confidence, time validity, ownership, and supersession when applicable. Remembered information MUST NOT outrank live repository, filesystem, device, or authoritative external state. Untrusted data remains tainted through summaries and model transformations unless independently verified.

## VI. The Model Is Replaceable; the Harness Is the Product

Golam MUST support multiple local and optional cloud models through `ExecutionProfile` contracts that separate model, inference backend, quantization, harness profile, context strategy, tool grammar, sampling, cache behavior, and resource limits.

No single provider or model-specific prompt may define core semantics. Harness quality MUST be measured separately from model quality. Local hardware calibration and explicit privacy/cost policy MUST inform routing.

## VII. Computer Control Is Semantic-First and Human-Interruptible

Golam MUST prefer stable semantic control paths in this order: domain/application API, native OS automation API, accessibility/semantic tree, browser DOM, deterministic keyboard/mouse control, then vision/pixel fallback.

Remote or autonomous control MUST be visible to the local user when an interactive session is active, MUST support immediate pause/stop/takeover, and MUST fail closed when required OS permissions or interactive-desktop conditions are unavailable. Golam MUST NOT implement stealth monitoring, hidden persistence, silent privilege escalation, or security-control bypass as product features.

## VIII. Open Protocols, Governed Skills, Replaceable Adapters

Golam MUST prefer open interoperability boundaries: ACP for IDE/client agent interaction, MCP for tools/resources, Agent Skills-compatible `SKILL.md` packaging, and A2A only for external independent-agent federation when needed. Internal workers SHOULD use native typed Rust scheduling rather than network protocols.

Skills and plugins are supply-chain inputs. Discovery, provenance, license review, capability inference, security scanning, sandbox testing, version locking, and deprecation MUST be first-class lifecycle steps. A skill is instruction and code; it is never authority.

## IX. Clean-Room and Donor Governance

`Golam-Research` and reconstructed Grok Bot artifacts are evidence/reference material only unless a specific component is independently proven redistributable. Golam MUST NOT copy proprietary reconstructed source, shipped renderer assets, private skills, trademarks, or unlicensed code.

Open-source donors MUST be qualified at an exact commit/tree before source admission. Qualification MUST record license/notices, vendored/generated code, dependency closure, network/telemetry/secrets behavior, unsafe/process boundaries, test posture, and an explicit reuse strategy: dependency, selective import/port, adapter, pattern/reference, or benchmark-only.

## X. Verification Beats Claims

No feature parity, benchmark, security, privacy, offline, platform, or reliability claim may be made without reproducible evidence. Exact-head tests are required for release-gating claims. Benchmarking MUST evaluate harness, tools, memory, context, durability, recovery, safety, cost, cache behavior, and user intervention—not only model accuracy.

Long-horizon evaluation MUST test premature stopping, stale-memory resistance, crash/restart recovery, idempotency, verification discipline, remote-control safety, and goal retention.

## Governance

This constitution supersedes ad-hoc conventions when they conflict.

- Principles I-X are binding gates for every Spec Kit plan and later implementation.
- Amendments require a pull request, rationale, founder approval, and semantic-version bump.
- MAJOR = backward-incompatible governance change or principle removal/redefinition.
- MINOR = new principle or materially expanded governance.
- PATCH = clarification without semantic change.
- Complexity that weakens a MUST requires an explicit constitutional amendment; it cannot be hidden in a plan exception.
- Every implementation PR MUST state the relevant constitution gates and exact verification evidence.

**Version**: 1.0.0 | **Ratified**: 2026-08-24 | **Last Amended**: 2026-08-24
