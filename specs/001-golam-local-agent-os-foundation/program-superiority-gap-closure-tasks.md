# Program Superiority Gap-Closure Task Graph — 2026-09-08

**Authority**: PROGRAM ORCHESTRATION ONLY — NO PRODUCT IMPLEMENTATION AUTHORITY

**Extends**: `program-superiority-tasks.md` T110–T165

**Synthesis**: `source-synthesis-gap-closure-2026-09-08.md`

These tasks do not widen active Spec 006 PR #24. They become implementation authority only through a later bounded owning Spec Kit lifecycle after live successor-authority verification.

## Phase K — Planning consolidation and canonical product spine

- [ ] **T166 — Planning Consolidation Registry.** Reconcile historical planning PRs #6, #7, #8 and #22 into the current program overlay. Record each retained concept, owning future spec/task and any concept explicitly superseded. Once the overlay is canonical, historical planning branches remain provenance/research inputs rather than parallel live program authority.
- [ ] **T167 — Canonical Work Spine.** Define durable `Task`, `Session`, `Run` and `Worker` identities with `TASK != SESSION != RUN != WORKER`; versioned `TaskContract`; criterion-level Verification Plan; task-level pause/stop/steer/takeover/resume; `TrustReceipt`; and terminal distinction between verified `SATISFIED` and explicit unverified closure. Bind T149 VerificationReceipts so a successful run/worker/model statement cannot silently satisfy a task criterion.
- [ ] **T168 — Authority Host and Execution Nodes.** Define one active protected user-owned `AuthorityHost` by default and bounded `ExecutionNode`/`ComputeNode` roles. Specify migration, pairing, anti-fork recovery, fresh authentication root, recovery-authority freshness, external-credential quarantine, stale-backup effect uncertainty and current-authority revalidation before resumed work. `EXECUTION_NODE != AUTHORITY_HOST`, `COMPUTE_NODE != AUTHORITY_HOST`, `BACKUP_STATE != ACTIVE_AUTHORITY`.
- [ ] **T169 — Native Mobile and Channel Security Consolidation.** Carry forward the strongest PR #6 boundaries: Native Mobile is a paired GolamConnect device; messaging providers are lower-assurance transports; push is a wake hint; voice/media content is not authentication; offline signed request intent is not current authorization; remote-control media/input uses GolamConnect rather than messaging channels; provider identity is a binding key, not authority.

## Phase L — Compute, interoperability, containment and artifact trust

- [ ] **T170 — Local Compute Mesh.** Define a user-owned local inference/compute mesh inspired by `NVIDIA/Personal-AI-Router@13b68115fa2c9c1d94f1ead1358f8d5a527cfecf`, integrated with GolamConnect, T132 Resource Governor and T152 QualityCostPrivacyGovernor. Pair nodes cryptographically; advertise exact model/backend/hardware/readiness state; route locality/privacy/authority compatibility before resource/cost/quality; deny hidden model installation/download; bind exact model/backend/node identity into receipts; never place protected authority state on a compute node.
- [ ] **T171 — External Agent Interoperability Profile.** Define A2A interoperability using `a2aproject/A2A@98853be376c88df25e1704771cd3ea9ef8823a96` as the primary reference. AgentCards and advertised skills are descriptive only; inbound task/message content is untrusted; remote output requires Golam verification; arbitrary external agents never receive Golam capability tokens or satisfy human approval. Prefer GolamConnect/internal worker protocols when both endpoints are user-owned Golam members.
- [ ] **T172 — MCP 2026-07-28 Protocol Profile.** Create an explicit `McpProtocolProfile` and migration/conformance plan against `modelcontextprotocol/modelcontextprotocol@e76e9c572c6f2bfcb730357101acc90f2f802e02`. Track protocol/version negotiation, Tasks/Extensions, current authorization behavior, deprecated assumptions and cross-version fixtures. MCP remains tool/context/resource interoperability and never grants Golam authority by itself.
- [ ] **T173 — Execution Isolation Profile.** Replace generic `sandbox` claims with typed isolation classes and evidence. Cover ordinary process restrictions, OS sandbox/profile, container/runtime policy, syscall-interposer/gVisor-class containment, VM/microVM/Kata-class containment and optional remote sandbox. Use `kubernetes-sigs/agent-sandbox@e5e2831295a789299722a9f4d922250167b1e879` as a strong reference. Providers cannot self-raise isolation class. Qualification requires externally observed filesystem/network/device/process/syscall properties where applicable. `CONTAINER != SECURITY_BOUNDARY` without exact runtime/profile qualification.
- [ ] **T174 — High-Assurance Owner Presence.** Define an `OwnerPresenceReceipt` for selected high-risk approvals, device pairing and Authority Host recovery using qualified platform authenticators/WebAuthn Level 3 or native equivalents. `USER_VERIFIED != OPERATION_AUTHORIZED`; the receipt binds a fresh user-presence/authenticator event but ordinary current Golam authorization still binds the exact operation/effect. No mandatory biometric requirement.
- [ ] **T175 — Model Artifact Foundry.** Define exact model artifact qualification: weight/content digests, license, tokenizer bytes/hash, chat-template identity, quantization format/tool/version, conversion provenance, backend/runtime identity, supported hardware, local/remote/provider identity and optional code surfaces. Default `trust_remote_code` to false; deny undeclared model downloads; quarantine artifacts until admitted; bind exact model artifact/backend into routing and verification receipts. `MODEL_HASH != MODEL_TRUST` and model output never gains authority.
- [ ] **T176 — Formal Assurance Foundry.** Use Quint/TLA+-style executable/model-checkable specifications selectively for critical bounded state machines, with `quint-co/quint@6fb2924e00707cef6dbc5e30db606d555c447123` as a primary candidate/reference. Initial targets: Effect/`UNKNOWN_OUTCOME` FSM, capability lease generation/revocation, scheduler dedup/catch-up, Authority Host migration/recovery anti-fork, mobile approval anti-replay, worker handoff/resume and extension activation/revocation. Convert counterexamples into deterministic implementation regression fixtures. Never claim whole-product formal verification from a model-only result.

## Phase M — Agent security, observability, extensions and resilience

- [ ] **T177 — Agent Security Control Matrix.** Map T156's system threat model to NIST agent-security and OWASP Agentic Security Initiative threat categories. For every material threat record `threat -> Golam invariant -> owning contract/spec -> deterministic adversarial test -> evidence -> residual risk`. Cover goal hijacking, tool misuse, identity/privilege abuse, memory/context poisoning, insecure inter-agent communication, cascading failures, rogue agents and supply-chain/runtime compromise.
- [ ] **T178 — Privacy-Safe Observability Projection.** Define a local-first observability projection using OpenTelemetry GenAI semantic conventions where useful. Collect bounded model/provider latency, token/cost/resource/worker timings without prompt/tool contents by default. No silent upload; all remote telemetry requires explicit admission/consent. Observability supports `golam doctor` and performance analysis but remains separate from authority/evidence truth: `TRACE != EVIDENCE`, `TRACE != AUTHORITY`.
- [ ] **T179 — Instruction/Data Trust Zones and Egress Broker.** Create a cross-cutting trust-zone/egress contract so web pages, documents, email, connector/MCP/tool output, retrieved memory and external-agent output cannot become trusted instructions merely because a model reads them. Before sensitive egress bind data class/taint, exact destination/provider/account, purpose, retention expectation where knowable, redaction/secret posture, current egress capability and effect/approval class. Apply to model providers, sandboxes, A2A, connectors and browser/application routes.
- [ ] **T180 — Signed Extension Catalog and Revocation Transparency.** Extend T154/T165 with immutable package/version identity, publisher provenance key/signature, exact Source Foundry/security/conformance evidence, activation state, revocation state and a durable transparency/decision record. Publisher signature proves provenance only and cannot grant runtime authority. Require stale activation/cache/queue invalidation after revocation/replacement.
- [ ] **T181 — Shadow Activation and Rehearsal.** Add no-external-effect rehearsal/simulation/observe-only modes for SkillCandidate, RoutineCandidate, automation, extension and learned workflow activation where feasible. Compare proposed route/targets/effects against expected behavior and safety obligations before production activation. A successful rehearsal is evidence, not authority; activation remains a separate governed effect.
- [ ] **T182 — Chaos and Resilience Qualification.** Add exact fail-closed scenarios for disk-full, clock jump/rollback, suspend/resume, network partition, relay loss, compute-node loss, OOM/GPU eviction, connector-token expiry, browser/runtime update, daemon crash, extension revocation, Authority Host loss, paired-mobile loss and partial restore. Each fixture declares expected protected-state outcome, allowed degradation and recovery proof. No resilience score may compensate for an authority/effect-safety violation.
- [ ] **T183 — Surface Parity Contract.** Require CLI, TUI, Desktop, Mobile, IDE, MCP-facing product surfaces and remote projections to map the same semantic operation into the same typed kernel authority/effect path. UX may differ; privileged surface-only bypasses are forbidden. Add cross-surface conformance fixtures for identity, approval, effect, task-control and verification semantics.
- [ ] **T184 — Decision and Dissent Record.** Define durable multi-worker/agent decision provenance. Majority/model consensus cannot grant authority. Material dissent about safety, target identity, destructive effects, verification sufficiency or recovery remains visible and can trigger an independent verifier or human escalation. Preserve dissent across handoff/restart until explicitly resolved by an authorized decision process.

## Dependency guidance

Recommended dependency order after owning-spec authorization:

```text
T166
-> T167
-> T168
-> T169
-> T170/T171/T172/T173/T174/T175
-> T176/T177/T178/T179
-> T180/T181
-> T182/T183/T184
```

Cross-cutting dependencies:

- T170 depends on T132, T152, GolamConnect identity and T175 model identity.
- T171 depends on T119 connector/credential separation, T149 verification and T179 taint/egress.
- T172 depends on T157 protocol compatibility and T165 extension security.
- T173 refines T151 execution backend qualification.
- T174 must integrate with T156 threat model and T168 recovery.
- T175 integrates with T152 routing and T143/T145 benchmark attribution.
- T176 supplies additional evidence to owning specs but never bypasses exact implementation tests/review.
- T177 refines T156 rather than creating a second threat model.
- T178 supports T139/T159 but cannot replace evidence ledgers.
- T179 is required before broad external-agent/connector/cloud execution release.
- T180 extends T154/T165.
- T181 extends T122/T162 and applies before unattended learned behavior release.
- T182 is a Spec 010 release input but owning specs must create their failure fixtures earlier.
- T183 consumes the canonical work spine from T167.
- T184 consumes worker/handoff semantics from T127/T134/T163.

## Hard invariants added by this extension

```text
PERMISSION_TO_COPY != TECHNICAL_ADMISSION
EXECUTION_NODE != AUTHORITY_HOST
COMPUTE_NODE != AUTHORITY_HOST
AGENT_CARD != CAPABILITY_LEASE
EXTERNAL_AGENT_OUTPUT != VERIFIED_RESULT
MCP_CAPABILITY != GOLAM_AUTHORITY
CONTAINER != QUALIFIED_SECURITY_BOUNDARY
OWNER_PRESENCE != OPERATION_AUTHORIZATION
MODEL_HASH != MODEL_TRUST
FORMAL_MODEL_PASS != IMPLEMENTATION_PROOF
TRACE != EVIDENCE
PUBLISHER_SIGNATURE != AUTHORITY
REHEARSAL_SUCCESS != ACTIVATION_AUTHORITY
AGENT_CONSENSUS != HUMAN_APPROVAL
SURFACE_DIFFERENCE != AUTHORITY_DIFFERENCE
```

## Current sequencing

1. Do not widen PR #24.
2. Qualify this planning overlay on exact head.
3. Obtain fresh independent substantive architecture/security/governance review on the unchanged head.
4. Merge only after findings are reconciled and Ready is permitted.
5. After Spec 006 canonical closeout, re-read live successor authority before creating any implementation spec for T166+.

`ACTIVE_SPEC_006_PR_24_WIDENED=NO`

`NEW_PRODUCT_DEPENDENCY_ADMISSION=NO`

`FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO`

`WAIVER_TAKEN=NO`
