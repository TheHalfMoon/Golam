# Program Superiority Task Graph — 2026 Overlay

**Authority**: PROGRAM ORCHESTRATION ONLY — NO PRODUCT IMPLEMENTATION AUTHORITY

**Parent**: `program-superiority-2026.md`

Every product task below requires its own bounded Spec Kit lifecycle and exact live predecessor verification before implementation. This task graph does not widen active Spec 006 PR #24.

## Phase A — Governance and release trust prerequisites

- [ ] **T110** Create and maintain one canonical durable lifecycle pointer (`specs/CURRENT.md`) and eliminate stale duplicated phase/PR state from root governance documents.
- [ ] **T111** Add a repository-state consistency check that fails when canonical lifecycle declarations contradict each other; live PR head/check/review state remains fetched rather than cached.
- [ ] **T112** Configure or formally require a GitHub `main` ruleset/branch-protection posture: PR-only updates, protected history, required qualification checks, no force-push/delete, and review requirements appropriate to repository governance. Record evidence or an explicit external-administration blocker; do not pretend documentation is enforcement.
- [ ] **T113** Define release supply-chain governance: dependency audit evidence, advisories/licenses/sources, action pinning, SBOM, artifact provenance/attestations, platform signing/notarization, update signature/key rotation/recovery, rollback/anti-downgrade and third-party notice obligations.
- [ ] **T114** Define real-device qualification as a separate evidence class from hosted CI and create a machine-readable platform/capability/permission fixture manifest.

## Phase B — Unified action/capability fabric

- [ ] **T115** Define `ControlRouteProvider` and kernel-owned `RouteApplicabilityEvidence` spanning domain/app API, native API, accessibility, browser DOM/protocol, deterministic input and bounded vision/pixel evidence without letting providers mint authority.
- [ ] **T116** Require stronger-route provider freshness/identity/disposition before weaker fallback; unknown/stale stronger-route state must not silently authorize weaker routes.
- [ ] **T117** Create a bounded Browser/Application Semantic Control planning unit after Spec 006 closes; prefer WebDriver/WebDriver BiDi and native application APIs over generic computer input where available.
- [ ] **T118** Correct the current OCR ownership mismatch: raw screenshot OCR/visual semantic extraction is not Spec 007 GolamConnect. Create a future bounded Visual Semantics unit only after canonical successor authorization.
- [ ] **T119** Define a connector/account capability broker separating API connector identity/scopes/secret handles from browser login/session state and from ordinary egress permission.

## Phase C — Context, memory and safe learning

- [ ] **T120** Formalize repository-intelligence promotion gates: L0 text/git -> structural/AST -> LSP -> Ripwire-style deterministic graph/context -> heavier dataflow/runtime/vector only after measured residual need.
- [ ] **T121** Benchmark `redhat-et/ripwire` under Issue #25 as an external local tool first, then optional sandboxed sidecar/MCP; measure task success, changed-file/test-selection quality, token savings, latency, memory and stale-index behavior before any port/admission.
- [ ] **T122** Add a safe learning compiler: trajectory -> `SkillCandidate`/`RoutineCandidate` -> provenance/taint/authority analysis -> deterministic replay/tests -> approval -> immutable versioned activation -> outcome evidence -> candidate improvement.
- [ ] **T123** Prohibit in-place autonomous mutation of active skills/routines; every learned revision is a new immutable version and stale queued/cached authority is invalidated on replacement/revocation.
- [ ] **T124** Add governed demonstration-to-routine planning: semantic observation first, secret separation, variable/precondition/postcondition inference, dry-run/test mode and explicit activation authority; coordinates remain untrusted hints only.
- [ ] **T125** Add external memory-quality qualification using exact-pinned LongMemEval-V2 plus bounded LoCoMo/BEAM-style cases and Golam-specific stale-memory/contradiction/provenance/abstention scenarios.
- [ ] **T126** Add product UX for inspecting/editing/promoting/forgetting memory and reviewing skill/routine provenance/version history without bypassing canonical writer governance.

## Phase D — Workers, automations and resource governance

- [ ] **T127** Expand Spec 008 with durable named worker identities, narrow child leases, isolated worktree/workspace by default, typed handoff bundles and explicit shared-artifact channels.
- [ ] **T128** Define conflict detection and merge/reconciliation for concurrent worker writes; shared workspace is never presented as a security boundary.
- [ ] **T129** Define trusted time semantics: timezone identity, DST fold/gap, clock jumps, suspend/resume, missed-run catch-up, overlap policy, dedup identity and stale/no-data policy.
- [ ] **T130** Bind every scheduled run to exact current skill/routine/model/profile/connector versions and re-evaluate authority at run time; drift cannot silently inherit old approval.
- [ ] **T131** Add deterministic no-agent automation mode where an LLM is unnecessary; it still uses Golam scheduling/effect/authority semantics.
- [ ] **T132** Implement a Golam Resource Governor for CPU/GPU/memory/model residency, worker concurrency, disk/artifact quotas, bandwidth, battery/thermal state where available, foreground latency priority and graceful degradation. Resource pressure never weakens security gates.

## Phase E — Persistent teammate and product-superset UX

- [ ] **T133** Define persistent named teammate UX: durable role, goal/status/evidence/blockers, clear current activity, explicit authority and per-worker memory view.
- [ ] **T134** Define typed multi-worker handoff/group semantics so workers may coordinate without collapsing identities, approvals, memory provenance or effect attribution.
- [ ] **T135** Add routine authoring/test/history/pause/resume/rollback UX with explicit unattended-risk controls and recent-run evidence.
- [ ] **T136** Add first-class deep-research evidence graph/citation provenance and exact-source freshness semantics rather than answer-only citations.
- [ ] **T137** Complete built-in governed artifact workflows for documents, presentations, spreadsheets and PDFs with deterministic validation, provenance and file/effect safety.
- [ ] **T138** Add product-wide encrypted backup/export/restore planning for canonical memory, authority metadata, secret/device posture and rebuildable derivatives; restore requires integrity/schema/anti-replay checks and dry-run reporting.
- [ ] **T139** Add `golam doctor`/diagnostic bundle design with local-only structured diagnostics, bounded/redacted logs, pending uncertainty, platform capability state, model/worker/resource status and no silent upload.

## Phase F — Voice, multimodal and cross-device superiority

- [ ] **T140** Plan optional real-time voice after trusted core maturity: local-first STT/TTS candidates, explicit cloud capability when used, interruption/barge-in, secret/privacy boundary and no always-listening default.
- [ ] **T141** Plan mobile/remote companion UX over GolamConnect while keeping local owner authority canonical; remote clients receive explicit short-lived capabilities rather than becoming cloud authority roots.
- [ ] **T142** Add governed multimodal artifact/image/PDF understanding with exact provenance and privacy controls; visual perception remains evidence, never authority.

## Phase G — GolamBench and verified release supremacy

- [ ] **T143** Pin and qualify external benchmark fixtures: OSWorld V2, WindowsAgentArena where feasible, BrowserGym/WebArena-Verified/WorkArena, SWE-bench-class coding suites and LongMemEval-V2. Never use floating benchmark state for release claims.
- [ ] **T144** Define hard non-compensating release gates: zero unauthorized effects, zero blind retry of at-most-once/irreversible effects, zero strict-local unexpected egress, zero protected-authority forgery success, zero silent control without required visibility, zero stale control generation accepted and zero unverified completion reported as verified.
- [ ] **T145** Define comparative metrics across verified task success, intervention rate, unnecessary approvals, takeover latency, crash recovery, wrong-target actions, memory stale/false recall, context/token cost, model cost, resources, route strength, scheduler reliability and worker handoff conflicts.
- [ ] **T146** Separate hosted CI, real-device platform qualification and signed release-artifact qualification; all three must be explicitly represented in final release evidence.
- [ ] **T147** Require backup/restore drill, updater/signature drill, device-loss/key-rotation drill and strict-local external observation against the exact release candidate artifacts before stable release.
- [ ] **T148** Publish no “best”, “parity”, “superset”, security, offline or reliability claim unless the exact benchmark/release evidence is revision-bound and reproducible.

## Phase H — Proof, execution-fabric and ecosystem superiority

- [ ] **T149** Define a `VerificationObligation` / `VerificationReceipt` fabric separate from model self-assessment. Every long-running goal and high-risk task declares required evidence classes; `VERIFIED_COMPLETE` is permitted only when trusted verifiers satisfy them. Model confidence may inform prioritization but cannot certify completion.
- [ ] **T150** Add verifier diversity and independence rules: deterministic checks first, source-of-truth/API readback where available, environment observation second, independent model critique only as untrusted supporting evidence. Prevent the actor that produced an effect from being the sole verifier of the effect when a stronger verifier exists.
- [ ] **T151** Define a replaceable `ExecutionBackend` contract that separates agent logic from local process containment, local VM/container/microVM environments and optional remote sandbox providers. Evaluate SWE-ReX and E2B-like infrastructure as reference candidates; remote execution is an explicit non-strict capability and never becomes the canonical authority root.
- [ ] **T152** Extend resource routing into a `QualityCostPrivacyGovernor`: hard locality/privacy/authority compatibility first, then quality/latency/cost/resource optimization. Every provider/model/backend fallback is explicit, budgeted and attributable; an unavailable cheap/local path cannot silently widen privacy or authority.
- [ ] **T153** Define workspace snapshot/time-travel semantics for agent work: reproducible base identity, immutable checkpoints, worktree/VM snapshot refs, diff/artifact lineage, rollback as a new governed effect, and no claim that filesystem rollback reverses already-emitted external effects.
- [ ] **T154** Define a Golam Extension SDK + conformance kit for Skills/MCP/ACP/connectors/control-route providers and optional execution backends. Third-party extensions receive generated typed schemas, capability declarations, sandbox profiles, deterministic test fixtures and compatibility tests; extension conformance never implies authority admission.
- [ ] **T155** Add accessibility/internationalization/human-factors release gates for the desktop/TUI/approval/control surfaces: keyboard-only operation, screen-reader semantics where applicable, readable risk prompts, localization-safe identifiers, reduced-motion/high-contrast support and measured emergency-stop discoverability. Safety-critical controls cannot depend on color or pointer-only interaction.

## Phase I — System threat, compatibility and product-quality gates

- [ ] **T156** Maintain one system-level adversary/trust-zone model covering same-user malware, compromised model/backend, malicious MCP/Skill/extension, poisoned memory/context, hostile website/document, connector impersonation, stolen paired device, remote-execution provider compromise, update/supply-chain compromise and privileged-host administrator limits. Every owning spec maps new attack surfaces and mitigations back to this model.
- [ ] **T157** Define protocol/schema/migration compatibility policy for IPC, authority SQLite, canonical events, memory metadata, Skill/MCP/ACP bindings, GolamConnect, extension SDK and release artifacts: version negotiation, forward-only migration, backup-before-destructive migration, downgrade/rollback constraints, unsupported-future-version failure, and golden cross-version fixtures.
- [ ] **T158** Add benchmark-integrity policy: exact fixture hashes, held-out/private regression tasks where appropriate, contamination disclosure, no benchmark-specific prompt/tool hacks, anti-reward-hacking checks, evaluator version binding and separation of development tuning from final release qualification.
- [ ] **T159** Define product SLO/efficiency gates independent of benchmark intelligence: daemon cold/warm startup, idle CPU/RAM, IPC latency, interactive steering latency, emergency-stop latency, model-load overhead, disk growth/GC, sustained worker overhead, battery/thermal impact where measurable, and graceful degraded behavior under pressure.
- [ ] **T160** Define product data-lifecycle/privacy posture end to end: default retention by data class, local export/erase, cache/artifact cleanup, crash/log retention, remote execution/provider deletion semantics, connector/browser session cleanup, telemetry opt-in if ever admitted, and truthful limits for already-emitted external data.

## Phase J — Temporal knowledge, evolved skills, delivery graphs and extension security

Source qualification and design rationale for T161–T165 are recorded in `tencent-source-adoption-2026.md`. These tasks consume the Tencent sources only as explicitly bounded references/candidates; they do not widen active Spec 006.

- [ ] **T161** Define Golam Temporal Memory Semantics: explicit `observed_at` / `valid_from` / `valid_until` / source-time basis / contradiction lineage plus query-time temporal retrieval evidence. Evaluate temporal ranking against properly qualified temporal benchmarks and Golam-native fixtures. Recency/volatility may change ranking but can never raise source authority or destructively erase canonical history. `Tencent/RoMem` remains research-reference-only until repository/code/checkpoint/dataset rights are separately clear.
- [ ] **T162** Extend T122/T123 into Whole-Skill Evolution: atomic candidate revisions may change `SKILL.md`, scripts, references and assets together; held-out/private evaluation is separated from the improver by code/filesystem authority; every diagnosis → candidate → regression/safety evaluation → activation/rejection outcome becomes durable decision history. Activation is a separate governed effect and new dependencies still require Source Foundry. Use `Tencent/SkillHone` as a high-value method/source candidate, not authority.
- [ ] **T163** Define a resumable `DeliveryGraph` integrated with T127/T134/T149/T153: complexity-sensitive stage selection, immutable requirement/design inputs, separate implementation/review/test principals, stage entry predicates, verification obligations, durable checkpoints/resume cursors, abort semantics and artifact manifests. Resume always revalidates live repository/environment/authority state. Use `Tencent/LoopForge` as a workflow reference; its confirmation cadence does not override Golam risk/governance policy.
- [ ] **T164** Define a revisioned Knowledge Workspace distinct from canonical personal memory and protected authority: source documents, immutable `KnowledgeRevision`s, retrieval derivatives, exact citation/source bindings, connector-ingest receipts, stale/deletion semantics, manual edits and governed rollback. Borrow the strongest knowledge/revision/caller-bound-memory/sandbox-product patterns from `Tencent/WeKnora` while preserving Golam source-authority precedence, strict-local policy and execution/authority separation.
- [ ] **T165** Add Extension Security Admission before activation of any Skill/MCP/connector/control-route provider/execution backend revision. Deterministic pre-scan must cover hidden executable/bytecode surfaces, charset/encoding smuggling, archive/path hazards, download-and-execute/bootstrap behavior, credential/metadata access, persistence hooks, command construction, declared-vs-actual capabilities, dependency integrity/advisories/licenses, intent-vs-implementation mismatch, confusable/shadow tools and behavior/version drift. Optional LLM/dynamic scanners run only as sandboxed supporting evidence and cannot mint `SAFE` authority. Prefer SARIF-compatible stable findings/fingerprints. Use `Tencent/AI-Infra-Guard` as the primary security reference/source candidate and preserve all NOTICE obligations if code is ever reused.

## Competitor-reference rules

- Grok Bot public behavior is a product baseline, not internal architecture evidence.
- Hermes is a behavioral/implementation reference candidate for learning, schedules, skills, gateways and workers, subject to Source Foundry before code reuse/dependency use.
- Letta Code is a high-value reference for persistent agent identity, git-backed memory and self-evolving memory/skills/mods; Golam must retain candidate->verification->activation governance rather than allowing an active agent to rewrite trusted behavior in place.
- Codex is a behavioral/implementation reference candidate for local Rust agent UX, sandbox/approval, multi-agent/session and MCP/skills surfaces, subject to Source Foundry before reuse.
- OpenHands is a reference candidate for control-surface/backend topology.
- Browser Use/OpenClaw are browser behavior/reference candidates; they do not override Golam's semantic-route or credential/authority model.
- SWE-agent/SWE-ReX and E2B are coding/execution-infrastructure references; remote or cloud sandboxes remain optional execution capabilities, not Golam authority roots.
- Ripwire remains governed by Issue #25 and cannot enter the privileged kernel merely because it improves context quality.
- WeKnora is a high-value knowledge-workspace/memory UX/sandbox product reference; its multi-tenant/cloud/runtime assumptions do not define Golam authority.
- RoMem is a temporal-memory research reference only at the reviewed pin because root repository licensing is absent; no source/checkpoint/dataset reuse is admitted.
- SkillHone is a high-value governed skill-evolution reference/source candidate; runtime bypass behavior is explicitly not adopted.
- LoopForge is a resumable software-delivery workflow reference/source candidate; workflow artifacts remain evidence rather than authority.
- AI-Infra-Guard is a high-value Skill/MCP/agent security reference/source candidate; model-generated scanner verdicts are supporting evidence only and any code reuse must preserve exact Apache/NOTICE obligations.

## Current safe sequencing

1. Do not widen active PR #24 with this overlay.
2. Complete Spec 006 under its existing canonical task graph.
3. Merge this overlay only after exact-head planning/governance qualification and independent review.
4. After Spec 006 canonical closeout, re-read live successor authority and create only the next bounded authorized unit.
5. Future owning specs consume these tasks selectively in dependency order; this file never grants direct implementation authority.
