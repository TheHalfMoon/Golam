<!--
Sync Impact Report
- Version: 1.2.0
- Ratified: 2026-08-24
- Last Amended: 2026-08-24
- Amendment: incorporated founder-attested source permissions for all sources supplied by the founder and all sources identified during Spec 001 research. Such sources may become code donors after per-source Source Foundry admission records capture exact permission scope/evidence, source state, notices, dependency closure, and technical/security qualification. Golam-Research is upgraded from reference-only by default to high-value implementation evidence and an authorized-source candidate, while the immutable shipped Grok Bot 0.18 artifacts remain provenance anchors and proprietary branding/assets are not automatically admitted.
-->

# Golam Constitution

## I. Local Ownership Is the Trust Root

Golam MUST be local-first in architecture, data ownership, and failure behavior. The user's computer is the primary execution environment. Sessions, memory, policies, approvals, audit state, schedules, skills metadata, model configuration, and execution evidence MUST have a local canonical representation. A cloud model, relay, channel provider, or hosted service MAY be used only as an explicit capability and MUST NOT become a hidden prerequisite for core operation.

Strict-local mode MUST operate without transmitting user content to external AI providers. There MUST be no silent cloud fallback when local inference fails. Strict-local egress MUST be enforced at a single kernel-authorized network choke point for Golam-managed components; it is not merely a model-router preference. Unexpected egress attempts MUST fail closed and be auditable. External channels such as Telegram or WhatsApp are not equivalent to strict-local privacy and MUST be labeled accordingly.

## II. Rust Owns the Trusted Path; Authority Lives in a Smaller Privileged Kernel

The trusted runtime MUST be Rust-first. `golamd`, event/session durability, policy and capability enforcement, effect transactions, secrets, identity, device transport, remote-control authority, model routing, memory orchestration, audit, CLI/TUI, and protocol authority MUST be implemented in Rust unless a later constitutional amendment explicitly narrows this rule.

Rust trusted-path membership MUST NOT be confused with privileged authority. The privileged kernel is the strictly smaller subset that alone can read or mutate authority-bearing state, mint/verify capability material, authorize effects and egress, broker secrets, commit security-critical ledger records, authenticate local clients, maintain device pairing/revocation, or sign audit/receipts.

Kernel-owned state MUST be protected from generic filesystem, shell, worker, skill, MCP, plugin, browser, computer-control, and adapter write capabilities. Authority-bearing tokens/types MUST be unforgeable outside privileged kernel modules. The kernel API MUST be explicit and process-splittable so a single-process v1 does not make the whole daemon the de facto TCB.

Network/protocol-facing parsers and optional adapters SHOULD be isolated outside the privileged kernel and their outputs MUST be treated as untrusted input even when implemented in Rust.

TypeScript MAY be used in an untrusted Tauri renderer. Python or Node MAY be used only for optional sandboxed adapters and MUST NOT be required for strict-local operation or trusted authorization decisions.

## III. Authority Is Explicit, Least-Privilege, Authenticated, and Non-Self-Expanding

Every principal—human, device, worker, skill, channel, MCP server, plugin, external agent, client, or model—MUST have explicit identity and bounded capabilities. Authorization MUST be evaluated outside model reasoning. A principal can request authority but MUST NOT grant itself new authority.

Every local client of `golamd` MUST authenticate. Localhost, loopback, same-user-machine location, a display name, username, group membership, or successful transport connection MUST NOT be treated as authentication or authority. Remote/channel bindings MUST use stable provider/device identifiers and local user-approved binding.

The policy model MUST be deny-by-default for consequential effects. Safety denial is monotonic: downstream components cannot convert an upstream denial into an allow. Capability leases may only narrow parent authority and MUST honor expiry/revocation at the protected action boundary.

Policy, principal, capability/lease, approval, schedule-authority, Connect-pairing, and skill-lock changes are protected resources. Such mutations MUST themselves pass an elevated Effect Gate and cannot be performed through ordinary filesystem writes.

## IV. Every Consequential Effect Is Gated, Durable, Reconciled, and Attributable

No external or canonical write MAY bypass the Effect Gate. Each effect MUST identify requester, capability, target, risk, taint/provenance context, execution semantics, idempotency material, approval state, execution status, and evidence/receipt.

An effect intent MUST be durably persisted before external execution begins. Every effect handler MUST declare its execution semantics and reconciliation behavior. AT_MOST_ONCE and IRREVERSIBLE effects MUST NOT blind-retry after ambiguous outcomes. Dependent effects MUST wait while a prerequisite remains `UNKNOWN_OUTCOME`. Manual resolution is a first-class auditable state.

Long-running work MUST be crash-resumable. Every write MUST be attributable. The canonical event ledger is append-oriented; summaries, chats, UI state, compacted context, and memory indexes are projections, not replacements for canonical evidence.

Security-critical authorization/effect/connect/memory-governance records MUST have mandatory integrity chaining or an equivalently strong authenticated integrity mechanism; this cannot be optional release behavior.

## V. Secrets and Information Flow Are Explicit Security State

Real secrets SHOULD stay outside model context and untrusted execution whenever brokered use is possible. Secret handles, not raw values, are the default interface.

When a use cannot be brokered, it MUST follow an explicitly approved bounded fallback: inject only into the authorized sandbox/process channel, never command-line arguments; prevent ambient inheritance; apply value-aware output/log redaction; and keep secret material out of durable model-visible history where possible. User-pasted secrets MUST be redacted/tombstoned at ingestion rather than turned into a plaintext permanent ledger entry. The secret vault MUST be encrypted at rest.

Taint/provenance labels MUST survive derivation into summaries, memory candidates, scripts, code/files, and other artifacts. A model or tainted worker cannot declare its own input verified. Taint MAY be downgraded only by explicit human approval or deterministic verification against a pre-registered authoritative source/rule, and every downgrade MUST be auditable. SECRET_DERIVED content MUST NOT enter canonical long-term memory.

## VI. Memory Is User-Owned Evidence, Not Automatic Truth

Human-readable Markdown is the canonical long-lived knowledge format. SQLite is the canonical operational-state store. Search indexes, embeddings, graphs, caches, and summaries MUST be rebuildable derivatives.

Memory governance MUST define ADD, UPDATE, SUPERSEDE, CONTRADICT, MERGE, EXPIRE, FORGET, and REDACT semantics. Golam-generated managed-vault mutation MUST pass through a single governed memory writer; user hand-edits remain allowed and MUST be detected/reconciled rather than silently overwritten.

Promotion to durable project/user scope MUST retain provenance and require explicit approval or deterministic authoritative verification according to policy. Contradictions MUST be surfaced rather than silently erased. FORGET/REDACT MUST remove affected canonical content from active knowledge and rebuild derived indexes/caches; already-emitted external artifacts cannot be retroactively made unseen and the product MUST state that honestly.

Remembered information MUST NOT outrank live repository, filesystem, device, or authoritative external state.

## VII. The Model Is Replaceable; the Harness Is the Product

Golam MUST support multiple local and optional cloud models through `ExecutionProfile` contracts that separate model/revision, tokenizer/chat template, inference backend, locality, quantization, hardware mapping, harness profile, reasoning mode, tool-call conformance, context strategy, sampling, prompt/KV cache behavior, warm-residency policy, workload class, multimodal capabilities, budgets, and privacy/network constraints.

No single provider or model-specific prompt may define core semantics. Harness quality MUST be measured separately from model quality. Local hardware calibration and explicit privacy/cost policy MUST inform routing.

## VIII. Computer Control Is Semantic-First and Human-Interruptible

Golam MUST prefer stable semantic control paths in this order: domain/application API, native OS automation API, accessibility/semantic tree, browser DOM/protocol, deterministic keyboard/mouse control, then vision/pixel fallback.

Remote or autonomous control MUST be visible to the local user when an interactive session is active, MUST support immediate pause/stop/takeover, and MUST fail closed when required OS permissions or interactive-desktop conditions are unavailable. Secure desktop/UAC/TCC/Wayland boundaries MUST NOT be bypassed as product behavior.

Clipboard read is a distinct protected capability. Camera and microphone are deny-by-default capabilities. Human takeover suspends conflicting agent input at the authorization/lease layer rather than relying on UI convention.

Golam MUST NOT implement stealth monitoring, hidden persistence, silent privilege escalation, or security-control bypass as product features.

## IX. Open Protocols, Governed Skills, Replaceable Adapters

Golam MUST prefer open interoperability boundaries: ACP for IDE/client agent interaction, MCP for tools/resources, Agent Skills-compatible `SKILL.md` packaging, and A2A only for external independent-agent federation when needed. Internal workers SHOULD use native typed Rust scheduling rather than network protocols.

Skills, MCP servers, plugins, and adapters are supply-chain inputs. Discovery, provenance, permission/license review, capability inference, security scanning, sandbox testing, version locking, and deprecation MUST be first-class lifecycle steps. Executable skill scripts and MCP processes MUST run under bounded sandbox supervision with explicit FS/network/environment limits. A skill is instruction/code; it is never authority.

## X. Source Permission, Provenance, and Donor Governance

The founder has attested that permission exists for all source projects supplied by the founder and all source projects identified during Spec 001 research. This attestation makes those sources eligible for Source Foundry admission; it does NOT by itself mean that every file, trademark, binary asset, dependency, model weight, service credential, or redistribution mode is automatically covered.

Before any source code is copied, ported, vendored, forked, or made a direct dependency, the implementation spec MUST create a per-source admission record containing: source repository/artifact; exact commit/tree/version; permission/license evidence reference and scope; redistribution/modification obligations; notices; vendored/generated code boundaries; dependency closure; reciprocal-license closure where relevant; network/telemetry/secrets behavior; unsafe/FFI/process boundaries; platform test posture; selected files/crates; reuse strategy; and independent Golam verification.

`Golam-Research` and the reconstructed Grok Bot 0.18 codebase MUST be treated as high-value implementation evidence and an authorized-source candidate because the founder states permission has been obtained. Its pinned public release artifacts, hashes, reconstructed runtime boundaries, tests, protocol contracts, and working behavior SHOULD be mined seriously during donor qualification. However, the reconstruction itself records that it is not Anysphere's original monorepo and that historical upstream source licensing was not asserted in that repository; therefore Golam MUST record the founder's permission scope/evidence before admitting any bounded Grok reconstruction component. Shipped renderer assets, trademarks/branding, original installers, and other binary assets require their own explicit scope and MUST NOT be assumed admitted merely because runtime source use is permitted.

Permission does not force reuse. Rust/local-first architecture, security boundaries, technical quality, dependency risk, and maintainability remain independent admission gates. A TypeScript/Python/Node donor may be ported into Rust, wrapped as an isolated adapter, or rejected even when permission exists.

Unverified donor claims MUST be labeled unverified. A source may move through `REFERENCE -> VERIFIED -> PERMISSION_RECORDED -> TECHNICALLY_QUALIFIED -> ADMITTED`; code admission requires the final state.

## XI. Verification Beats Claims

No feature parity, benchmark, security, privacy, offline, platform, or reliability claim may be made without reproducible evidence. Exact-head tests are required for release-gating claims. Benchmarking MUST evaluate harness, tools, memory, context, durability, recovery, safety, cost, cache behavior, IPC security, egress, and user intervention—not only model accuracy.

Long-horizon evaluation MUST test premature stopping, stale-memory resistance, crash/restart recovery, duplicate-effect prevention, verification discipline, remote-control safety, goal retention, prompt-injection/taint survival, secret isolation, channel impersonation, and strict-local no-egress behavior.

Durability/idempotency/no-egress/injection gates MUST begin in the implementing specs; they MUST NOT be deferred entirely to a final benchmark phase.

## Governance

This constitution supersedes ad-hoc conventions when they conflict.

- Principles I-XI are binding gates for every Spec Kit plan and later implementation.
- Amendments require a pull request, rationale, founder approval, and semantic-version bump.
- MAJOR = backward-incompatible governance change or principle removal/redefinition.
- MINOR = new principle or materially expanded governance.
- PATCH = clarification without semantic change.
- Complexity that weakens a MUST requires an explicit constitutional amendment; it cannot be hidden in a plan exception.
- Every implementation PR MUST state the relevant constitution gates and exact verification evidence.

**Version**: 1.2.0 | **Ratified**: 2026-08-24 | **Last Amended**: 2026-08-24
