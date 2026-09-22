# Golam Canonical Direction Task Extension — 2026-09-13

**Authority**: PROGRAM ORCHESTRATION ONLY — NO PRODUCT IMPLEMENTATION AUTHORITY

**Extends**:

- `program-superiority-tasks.md` T110–T165
- `program-superiority-gap-closure-tasks.md` T166–T184

**Direction review**: `program-direction-review-2026-09-13.md`

**Source review**: `competitive-source-register-supplement-2026-09-13.md`

These tasks close gaps revealed by the 2026-09-13 architecture/product/source review. They do not widen active Spec 006 PR #24 and do not authorize a successor implementation unit.

## Phase N — Experience projection and route-fabric hardening

- [ ] **T185 — Agent UI Projection Contract.** Define a sanitized, versioned `AgentUiEventProjection` for Desktop/TUI/Mobile/IDE and optional external UI consumers. Project Task/Run/Worker/Action/Effect/Verification lifecycle events without serializing protected capability leases, secrets, authority SQLite internals, native handles or raw protected evidence. Use CopilotKit/AG-UI as a behavior/protocol reference while preserving Golam canonical events as the source of truth. `UI_EVENT != CANONICAL_AUTHORITY_EVENT`.

- [ ] **T186 — Action, Egress and Uncertainty UX Contract.** Define cross-surface UX semantics for Action Preview, exact target/account/provider, route choice, data-leaving-device disclosure, irreversibility/retry class, approval reason, `UNKNOWN_OUTCOME`, reconciliation, verification and terminal outcome. A generic Retry affordance is forbidden while an effect outcome is unknown. Integrate with T149/T150/T179/T183.

- [ ] **T187 — Surface-Aware Route-Order Qualification.** Benchmark the constitutional global route order across browser, native application, document/artifact and generic desktop surfaces. Determine whether a surface-aware partial order can improve target identity/reliability without weakening fallback safety. If evidence supports a change, prepare a separate Constitution amendment proposal; until ratified, the current constitutional order remains binding. This task cannot silently reorder Spec 006.

- [ ] **T188 — Explicit Research/Web Provider Broker.** Extend T119/T152/T179 with a typed research-provider contract for local search/index providers and optional hosted Search/Fetch/Research/Browser providers. Bind account, destination, data class, purpose, budget, retention expectation where knowable, credential handle, privacy profile, model/provider identity and egress receipt. Hosted TinyFish/Perplexity-class providers are never strict-local capabilities and cannot silently replace a local route.

## Phase O — Knowledge ingestion, context honesty and semantic artifacts

- [ ] **T189 — Research Evidence Graph and Evaluation Profile.** Refine T136/T143/T145/T158 into a first-class research evidence graph: query/subquery, exact source URL/object identity, fetched revision/time, extracted claim, citation span, contradiction, source quality metadata, provider path, cost and freshness. Add provider-normalized resumable evaluation runs inspired by `perplexityai/search_evals`, with exact config/dataset fingerprints, failed-as-zero and failed-excluded reporting, trace retention and cost accounting. Provider graders are supporting evidence only.

- [ ] **T190 — Bounded Local Tool Session Contract.** Define process/file/tool-session semantics inspired by Desktop Commander ergonomics but backed by Golam authority/isolation: long-running process identity, bounded/paginated output, tail reads, cancellation/kill, session expiry, exact cwd/environment/profile, output truncation receipts and local audit projection. Treat stdout/stderr, file contents and process metadata as untrusted data; sanitize terminal-control/escape rendering at the Experience boundary and never allow tool output to become authority. Directory allowlists or command blocklists alone never qualify as containment. Integrate with T151/T159/T165/T179.

- [ ] **T191 — Knowledge Ingestion Foundry.** Define `SourceArtifact -> NormalizedDerivative -> KnowledgeRevisionProposal -> RetrievalDerivative` with exact content digests, converter identity/version, sandbox/isolation profile, taint class, network use, parse warnings and archive/path/size limits. Evaluate MarkItDown as a sandboxed narrow converter candidate and OpenRAG/Docling-class behavior as references. Cloud/OCR converters require explicit egress and remain disabled in strict-local mode. `NORMALIZED_MARKDOWN != TRUSTED_INSTRUCTION`.

- [ ] **T192 — RetrievalReceipt / ContextBundle Epistemic Contract.** Extend T120/T125/T164 with one task-scoped contract reporting exact sources/revisions, retrieval strategy/version, freshness, token/byte budget, omitted/truncated sources, unresolved references, contradiction set, taint/data classes, network use, index identity and the next stronger retrieval step when context is thin. Do not invent one universal confidence score where no calibrated meaning exists. `THIN_CONTEXT != COMPLETE_CONTEXT`.

- [ ] **T193 — Semantic Artifact Provider Family.** Refine T137 so DOCX/XLSX/PPTX/PDF/structured-text operations prefer semantic file-format providers over GUI/pixel control. Define inspect/diff/patch/validate/render/write contracts, macro/external-link/embedded-object detection, exact output hashes, atomic replacement and rollback-as-new-effect semantics. Generic desktop input is a fallback only when the required behavior cannot be achieved through a stronger semantic route.

## Phase P — Long-horizon harness, voice and source-admission discipline

- [ ] **T194 — Checkpoint and Context Reconstruction Contract.** Refine T167/T127/T149/T163 with project/session/task checkpoints, task-tree progress, budgeted context reconstruction and resumable worker handoff. Borrow MiMo Code's useful checkpoint/context behaviors while requiring Golam provenance, exact source revisions and TaskContract criteria. Independent model judging may detect premature stop but cannot alone emit `VERIFIED_COMPLETE`.

- [ ] **T195 — Repository Context Compiler Honesty Profile.** Extend T120/T121 using Ripwire's deterministic/epistemic lessons. Repository context outputs must disclose indexed scope, ignored/skipped inputs, stale state, unresolved references, budget truncation and widening options. Keep external-tool-first qualification for Ripwire; do not place C++/tree-sitter runtime code inside the privileged kernel merely for token savings.

- [ ] **T196 — Voice Presence and Audio Authority Contract.** Refine T140/T169/T175 using VoiceStudio as a product/engine reference. Require explicit short-lived microphone authority, visible listening/recording state, immediate revoke/stop, push-to-talk or explicit start by default, local-first STT/TTS, barge-in, ephemeral raw audio default, transcript provenance/taint, engine/model artifact identity and explicit cloud capability when used. Treat emergency interruption as a protected control path independent of final transcription: an interrupt gesture/voice-activity event must be able to mute audible output and request cancellation immediately, without waiting for ASR to finalize the interrupting utterance; any later transcript is evidence/content, not the authority to stop. Treat voice cloning/speaker imitation as a separate bounded capability with source provenance, explicit consent/authorization policy and no authentication role; generated voice may never satisfy owner-presence or approval requirements. `VOICE_CONTENT != AUTHENTICATION`; `TRANSCRIPT != APPROVAL`; `SYNTHETIC_VOICE != OWNER_PRESENCE`; `FINAL_TRANSCRIPT != STOP_PREREQUISITE`.

- [ ] **T197 — Donor Firewall and TCB Budget.** Extend T113/T165/T180; do not create a second source-admission state machine, admission ledger, activation decision or competing definition of `ADMITTED`. Add the architectural-plane posture and privileged-TCB measurements as fields/evidence on the existing exact-component/version Source Foundry admission record. Bind every privileged-surface exception and every TCB-budget delta to the exact source pin/component, dependency closure, rights/NOTICE posture, architecture/security justification, measured benefit, rollback/removal plan and independent Golam qualification evidence. A source/version/component change reopens the affected admission evidence rather than inheriting the old exception. T180 extension revisions consume the same exact-component admission record for activation/revocation transparency. Track aggregate privileged dependency growth, unsafe/FFI/native parser surfaces, filesystem/network/device privileges, secret residency and process-spawn ability as governance metrics only: `TCB_BUDGET != ADMISSION_AUTHORITY`. Prefer ports/reimplementation in Authority/Evidence, bounded adapters in Capability/Context and broader selective reuse in Harness/Experience only after Source Foundry. A source permission grant never exempts the existing admission path or TCB budget.

- [ ] **T198 — Spec 009 Bounded-Package Decomposition and Shared-Contract Ownership.** Keep Spec 009 as an umbrella program target but prohibit one monolithic implementation authority package. Before any Spec 009 package receives implementation authority, produce one canonical ownership/dependency matrix for both feature packages and every shared cross-cutting contract they consume. At minimum map: Control Route Fabric + Browser/Application Semantics; Knowledge Ingestion + Research Evidence; Document/Artifact Semantics; Agent Experience Projection + Task/Routine UX; Voice + Presence; Context Compiler/Repository Intelligence; plus T167 TaskContract/work identities, T115/T116 route/applicability, T149/T150 verification, T157 schema/migration, T179 trust-zone/egress, T197 exact-component source-admission evidence, T199 Operation/Effect ontology and T201 privacy profiles. For each shared contract assign exactly one canonical owner/version source and migration authority; enumerate package read/write boundaries and compatibility gates. Feature packages may define provider-specific schemas but MUST consume canonical shared semantics and MUST NOT create package-local Effect ledgers, authorization state, capability-lease truth, verification truth, egress decisions, source-admission state, canonical-event truth or privacy-profile interpretation. The decomposition may be designed in parallel, but no first package is authorized until the shared-contract ownership/version/migration map required for that package is complete. Exact spec numbers/order are assigned only after live successor-authority verification.

## Phase Q — Capability ontology and product convergence

- [ ] **T199 — Unified Operation/Effect Ontology.** Define one typed operation/effect classification used across native desktop, browser, filesystem, process, Git, document, MCP, connector, research, voice, mobile and worker surfaces. The ontology must express read/observe vs mutate, externality, reversibility, idempotency/retry class, target identity strength, secret/taint/egress implications, required visibility, verification obligations and approval policy inputs. This extends the existing canonical Effect path and classification semantics; adapters map into it and do not create a second Effect ledger or package-local authority semantics.

- [ ] **T200 — Product Command Center Information Architecture.** Define the primary product navigation around durable Tasks, Sessions, Workers, Actions, Knowledge and Routines rather than chat history alone. Specify cross-links from an action to its task, actor, capability route, Effect record, egress receipt and VerificationReceipt; from knowledge to source/provenance/contradictions; and from routines to immutable versions/rehearsals/run history. Keep the frontend a projection/client of canonical state.

- [ ] **T201 — Privacy Profile Contract.** Make `STRICT_LOCAL`, `LOCAL_PLUS_APPROVED_CONNECTORS`, and `HYBRID_MANAGED` (names provisional) explicit top-level privacy profiles derived into capability/egress/model/execution policy. Profiles may narrow or preconfigure allowed capabilities but cannot create a second egress-policy engine, reinterpret strict-local independently, or bypass per-effect authority/secret policy. Switching to a more permissive profile is itself a governed configuration effect with clear data-path disclosure. T179/T152 remain the canonical egress/privacy-policy path consumed by this profile contract.

- [ ] **T202 — Heavy-Sidecar Admission Gate.** Require measured residual need before admitting always-on or heavyweight sidecars such as broad RAG/search services, browser farms, document services or model managers. Evaluate task-quality gain, startup/idle RAM/CPU/disk, attack surface, strict-local behavior, upgrade burden and failure modes against the smallest viable built-in/optional alternative. Heavy providers should load on demand where practical. Any admitted sidecar uses the same T197 exact-component Source Foundry record rather than a separate sidecar-admission path.

## Priority order

These tasks are not all equally urgent. Recommended planning/implementation dependency once a bounded owning spec is actually authorized:

```text
P0_EXISTING: T112 + T149/T150 + T156/T177/T179 + T115/T116 + T167

P0_NEW:
  T197 Donor Firewall / existing Source Foundry record + TCB Budget
  T199 Unified Operation/Effect Ontology on the existing Effect path
  T198 Spec 009 decomposition + shared-contract ownership matrix

P1_NEW:
  T185 Agent UI Projection
  T186 Action/Egress/Uncertainty UX
  T191 Knowledge Ingestion Foundry
  T192 RetrievalReceipt / ContextBundle
  T194 Checkpoint / Context Reconstruction
  T195 Repository Context Honesty
  T201 Privacy Profiles

P2_NEW_AFTER_FOUNDATIONS:
  T187 Surface-aware route-order qualification
  T188 Research Provider Broker
  T189 Research Evidence Graph/Evals
  T190 Local Tool Sessions
  T193 Semantic Artifact Providers
  T196 Voice Presence
  T200 Product Command Center
  T202 Heavy-Sidecar Admission
```

## Dependency notes

- T185 depends on T167/T183/T157 and must consume projections rather than create a new authority/event database.
- T186 depends on Effect/`UNKNOWN_OUTCOME`, T149/T150 and T179.
- T187 depends on T115/T116 and benchmark evidence; a Constitution amendment is a separate lifecycle.
- T188 depends on T119/T152/T179/T201.
- T189 depends on T188 for network providers and T192 for context/evidence disclosure.
- T190 depends on T151 isolation, T179 taint handling and T199 operation/effect classes.
- T191 depends on T165 extension security, T173 isolation and T179 instruction/data trust zones.
- T192 consumes T191/T164 sources and must remain provider-independent.
- T193 depends on T115/T116/T191/T199.
- T194 depends on T167/T149/T150/T163.
- T195 retains T121 external-tool-first qualification.
- T196 depends on T174/T175/T179/T199/T201 and does not belong to Spec 006.
- T197 extends T113/T165/T180 and stores posture/exceptions/TCB deltas on the same exact-component Source Foundry admission record; it is a governance/source-admission prerequisite for broad donor reuse, not a new admission authority.
- T198 consumes T157/T167/T115/T116/T149/T150/T179/T197/T199/T201 as shared contracts or ownership inputs. It is an implementation-scope prerequisite before any Spec 009 package authorization, and the first package cannot proceed until its shared-contract owner/version/migration map is complete.
- T199 refines existing Effect/capability semantics; it must not create a second Effect ledger.
- T200 depends on T185/T186/T167/T126/T135.
- T201 refines T179/T152 strict-local/egress policy; strict local remains fail closed and there is one canonical policy interpretation.
- T202 consumes T159 SLO/resource evidence and the same T197 exact-component Source Foundry admission evidence.

## Hard invariants added by this extension

```text
UI_EVENT != CANONICAL_AUTHORITY_EVENT
HITL_CONFIRMATION != EFFECT_AUTHORIZATION
LOOPBACK_REACHABILITY != AUTHENTICATION
SEMANTIC_SELECTOR_MATCH != TARGET_AUTHORITY
REMOTE_RESEARCH_PROVIDER != STRICT_LOCAL_CAPABILITY
RESEARCH_RESULT != VERIFIED_FACT
NORMALIZED_DOCUMENT != TRUSTED_INSTRUCTION
RETRIEVAL_INDEX != CANONICAL_MEMORY
THIN_CONTEXT != COMPLETE_CONTEXT
MODEL_JUDGE_PASS != VERIFIED_COMPLETE
VOICE_CONTENT != AUTHENTICATION
TRANSCRIPT != APPROVAL
SYNTHETIC_VOICE != OWNER_PRESENCE
FINAL_TRANSCRIPT != STOP_PREREQUISITE
DONOR_PERMISSION != TCB_ADMISSION
TCB_BUDGET != ADMISSION_AUTHORITY
SOURCE_ADMISSION_POSTURE != SECOND_ADMISSION_LEDGER
SPEC_009_UMBRELLA != MONOLITHIC_IMPLEMENTATION_AUTHORITY
PACKAGE_OWNERSHIP != SHARED_CONTRACT_OWNERSHIP
PACKAGE_LOCAL_EFFECT_MODEL != CANONICAL_EFFECT_MODEL
PACKAGE_LOCAL_PRIVACY_INTERPRETATION != CANONICAL_EGRESS_POLICY
PRIVACY_PROFILE != EFFECT_CAPABILITY
HEAVY_PROVIDER_AVAILABILITY != ARCHITECTURAL_REQUIREMENT
```

## Current safe sequencing

1. Keep active Spec 006 PR #24 unchanged in scope.
2. Qualify this planning branch independently on its exact head.
3. Merge only after material architecture/security/governance findings are reconciled under repository policy.
4. Do not change `specs/CURRENT.md` because no durable lifecycle state changed.
5. After Spec 006 canonical closeout, re-fetch live successor authority before creating or implementing any T185+ owning spec.
6. Before the first future Spec 009 package receives implementation authority, complete T198's canonical shared-contract ownership/read-write/version/migration map for that package and verify that it consumes the existing Authority/Effect/Verification/Egress/Source-Admission paths rather than creating local substitutes.
7. Decompose future capability work into bounded independently reviewable units; never treat this task file as direct implementation permission.

```text
ACTIVE_SPEC_006_PR_24_WIDENED=NO
CURRENT_POINTER_CHANGED=NO
NEW_PRODUCT_IMPLEMENTATION_STARTED=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
WAIVER_TAKEN=NO
```
