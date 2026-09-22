# Golam Canonical Direction Task Extension — 2026-09-22

**Authority**: PROGRAM ORCHESTRATION ONLY — NO PRODUCT IMPLEMENTATION AUTHORITY

**Extends**:
- `program-superiority-tasks.md` T110–T165
- `program-superiority-gap-closure-tasks.md` T166–T184
- `program-direction-tasks-2026-09-13.md` T185–T202

**Direction review**: `program-direction-review-addendum-2026-09-22.md`

**Source review**: `competitive-source-register-supplement-2026-09-22.md`

These tasks capture gaps exposed by the 2026-09-22 source/portfolio review. They do not widen active Spec 006 PR #24, change `specs/CURRENT.md`, amend the Constitution, admit any dependency, or authorize a successor implementation unit.

## Phase R — Semantic decision and capability exchange foundations

- [ ] **T203 — Semantic Decision Provider Contract.** Define one provider-neutral `DecisionProvider` / `DecisionRequest` / `DecisionReceipt` family for bounded typed decisions. Support at minimum Choice, Boolean/Noul and ordered Score semantics where independently qualified. Bind exact state/context digest, schema/option identities, explicit no-match/abstain semantics, model artifact, backend/device/runtime, prompt/schema digest, probability output, calibration profile, input-limit/truncation evidence, latency/resource/cost and provider revision. Define deterministic cross-field constraints / bounded dependency graphs so independently scored fields cannot silently form an impossible or policy-inconsistent decision set; contradiction must cause abstention, deterministic repair only where specified, or escalation. SemIf, Decider, Nimble and the founder-supplied `AlexWortega/openjev` are candidate/reference implementations; none becomes the canonical policy engine. OpenJev is the first explicitly planned first-class bounded provider target under T229, subject to T175/T204 qualification. The contract must explicitly prohibit a provider from minting capability, lowering a trusted consequence class, authorizing egress, satisfying owner presence/approval, declaring source truth or emitting `VERIFIED_COMPLETE`.

- [ ] **T204 — Decision Calibration, Applicability and Escalation Qualification.** Extend T143/T145/T152/T158/T175/T203 with workload-specific qualification for decision providers. Measure appropriate accuracy/balanced accuracy, NLL/log loss, Brier, ECE/reliability, AURC/selective risk, abstention coverage, option-order/schema perturbation, cross-field contradiction/impossible-state rate, context-length and truncation sensitivity, quantization/backend drift, domain shift, tainted/adversarial input behavior, latency and resources. Define a deterministic escalation policy from exact rules/source truth -> bounded local decision provider -> stronger provider/generative reasoning -> independent verification/human review. Generic confidence thresholds are forbidden unless calibrated for the exact workload/configuration.

- [ ] **T206 — Capability Catalog and Provider Offer Contract.** Extend T119/T152/T154/T165/T180/T197/T199/T201 with provider-neutral `CapabilityDefinition`, exact `CapabilityOffer`, `ProviderRevision`, `AccountBinding`, `CapabilityAvailability`, `CapabilityQualification` and proposed `ToolCallPlan`. Discovery is by task/capability rather than provider name alone. Every offer must expose exact provider/revision, locality, account/credential requirements, operation/effect classes, egress destinations/data classes, retention expectation where knowable, cost model/estimate, latency/reliability evidence, availability freshness and Source Foundry/conformance references. Catalog state is a projection of admitted/observed provider state and must not become a second capability, identity, authorization, billing or Effect authority.

### T203 acceptance requirements

A future owning spec must prove:

1. provider-neutral fixtures work against at least two independently qualified providers or one real provider plus a deterministic fake;
2. provider output cannot lower deterministic risk/authority;
3. model/provider replacement changes no protected authority semantics;
4. exact artifact/backend/schema/calibration identities are attributable;
5. abstention and unavailable/not-qualified are distinct from a negative answer;
6. prompt/context truncation cannot be hidden.

### T206 acceptance requirements

A future owning spec must prove:

1. one stable capability ID may expose multiple exact provider offers without collapsing provider/account identity;
2. an offer cannot become callable merely by appearing in search;
3. stale/unqualified/unavailable offers fail closed;
4. strict-local search may display remote offers but cannot select/dispatch them;
5. provider fallback cannot silently change egress, account, spend or effect class;
6. arbitrary private/unmodeled tools remain `OPEN_WORLD` until qualified.

## Phase S — Reconciled execution and credential-brokered tool use

- [ ] **T205 — Agent Workload Manifest / Reconciled Execution Contract.** Refine T127/T129/T130/T151/T153/T159/T167/T168/T173 using Google AX as a reconciliation/lifecycle reference without importing Kubernetes/Redis as local baseline requirements. Define a versioned `ExecutionEnvelope` bound to a canonical `TaskContract` with workspace bindings, execution backend, isolation profile, compute/resource budget, egress policy reference, execution profile/model reference, capability set, secret-handle references, readiness conditions, observable endpoints, checkpoint/suspend/resume/timeout/cleanup policy and output-artifact policy. Separate desired runtime state from observed runtime state. Bind each runtime instance and every prepared privileged `ToolRequest`, prepared `Effect`, and Effect-dispatch authorization to the exact current execution incarnation/generation (or equivalent fencing token). Reconciliation must perform one protected atomic generation transition that advances the current generation and supersedes/revokes every prior generation before a replacement worker can become dispatch-eligible. The Effect Gate/kernel must perform final generation-equality validation immediately before brokered-secret release, egress/provider access, and `Effect` dispatch, failing closed on any mismatch even when the stale worker still possesses a previously prepared request/authorization. For process/container/tool execution, bind authorization across approval wait to exact executable/workload identity where observable: executable or artifact digest, workspace/source revision, backend identity, requested confinement profile and the trusted observed-confinement evidence available for the actual attempt. A backend label, requested sandbox profile, or pre-approval command name is not proof that the post-approval workload bytes or confinement remained identical. Runtime reconciliation may restart/reprovision a worker where allowed but must never become a blind retry path for ambiguous external Effects.

- [ ] **T207 — Credential-Brokered Tool Relay Contract.** Extend T119/T165/T172/T179/T190/T199/T201/T206 with a Treg-inspired but Golam-authorized relay path. Support exact provider/account selection, multiple credential bindings, secret-handle injection, destination binding, control-header/cookie stripping, SSRF/private-address policy, raw path/query fidelity where required, bounded request/response sizes, streaming, cancellation, idempotency material, bounded pre-dispatch cost quote/reservation where a provider is paid, final cost receipt/reconciliation, and failure capture with redaction. When invoked by a reconciled worker, the relay must consume the T205 current-generation binding from the prepared `ToolRequest`, prepared `Effect` and dispatch authorization and revalidate final generation equality immediately before credential/secret release and before egress/upstream provider dispatch; a stale generation fails closed before either boundary even if it retained a previously prepared request. Cost overrun beyond the authorized budget must stop/fail closed or require a new budget decision; spend authorization is separate from Effect authorization. Authorization, Effect state, egress, identity, secret state and final verification remain canonical Golam concerns. A relay transport success is not an Effect verification result.

### T205 invariants

```text
WORKLOAD_MANIFEST != CAPABILITY_GRANT
WORKSPACE_BINDING != FILESYSTEM_AUTHORITY
GATEWAY_DECLARATION != EGRESS_AUTHORIZATION
RUNTIME_READY != TASK_VERIFIED_COMPLETE
RUNTIME_RECONCILIATION != EFFECT_RETRY
CHECKPOINT_STATE != CURRENT_AUTHORITY
APPROVED_COMMAND_NAME != APPROVED_EXECUTABLE_BYTES
SANDBOX_LABEL != CONFINEMENT_PROOF
BACKEND_CAPABILITY != OBSERVED_CONFINEMENT
APPROVAL_WAIT != WORKLOAD_IDENTITY_CONTINUITY
PREPARED_REQUEST != GENERATION_INDEPENDENT_AUTHORITY
STALE_GENERATION != SECRET_RELEASE_ELIGIBLE
STALE_GENERATION != EGRESS_DISPATCH_ELIGIBLE
```

### T207 invariants

```text
CREDENTIAL_BINDING != CALL_AUTHORIZATION
RELAY_TARGET_RESOLUTION != EFFECT_PERMISSION
HTTP_SUCCESS != EFFECT_VERIFIED
TOOL_METADATA != TRUSTED_OPERATION_CLASS
OPEN_WORLD_TOOL != SAFE_READ
```

### T205/T207 shared acceptance

A future owning spec must prove:

- resume revalidates TaskContract, current capability generations, workspace/source revisions, provider availability and pending Effect uncertainty;
- a lost worker cannot cause duplicate at-most-once/irreversible Effects;
- a partitioned old worker and its replacement can race on the same consequential action only with the current generation reaching secret release/egress/dispatch; after replacement activation the old generation must fail before brokered-secret release and before any external dispatch;
- setup/bootstrap authority is narrower than or equal to the authorized envelope and cannot install arbitrary software through hidden egress;
- credential values never need to enter model context when brokered use is possible;
- exact provider account is visible before consequential dispatch;
- cancellation has honest semantics when an upstream may already have received the request;
- `UNKNOWN_OUTCOME` blocks dependent/conflicting operations until reconciliation.

## Phase T — Proactive attention, coherence and compact agent-facing discovery

- [ ] **T208 — Proactive Attention and Action Proposal Fabric.** Extend T133/T135/T167/T179/T183/T185/T186/T201 with Laya-inspired proactive product semantics built strictly as projections over canonical Golam state. Define `AttentionItem` and `ActionProposal` objects sourced from connector observations, schedules, worker blockers, approval expiry, `UNKNOWN_OUTCOME`, verification gaps and user-governed opportunity rules. An ActionProposal must disclose exact target/account/provider/route, proposed payload or diff, operation/effect class, egress/data path, cost, irreversibility/retry semantics, source/context evidence, verification plan and freshness/expiry. UI approval is authenticated approval input only; the Effect Gate immediately revalidates live authority before dispatch.

- [ ] **T209 — Cross-Source Coherence and Briefing Projection.** Refine T120/T125/T126/T136/T161/T164/T189/T192 using Laya's Coherence/Omni ideas and Morize semantics. Define evidence-linked cross-source entity/relation candidates, temporal views and daily/periodic briefing projections across connectors, Tasks, Actions and Knowledge. Retrieval should prefer exact identity/metadata/time -> FTS/BM25 -> explicit relation graph -> optional local vectors/reranking -> synthesis. Where fusion such as Reciprocal Rank Fusion is used it remains ranking evidence, not truth. Briefings must disclose time window, included/offline/stale sources, omissions, unresolved contradictions and which statements are direct observations versus synthesis. Manual link/unlink/classification corrections create immutable rule/routine candidates; they cannot mutate active trusted policy in place.

- [ ] **T210 — Compact MCP Capability Surface.** Extend T154/T172/T183/T206/T207 with a stable agent-facing discovery surface that avoids one MCP tool per provider endpoint. Define compact `capability_search`, `capability_get` and exact `capability_call` semantics with static/versioned schemas, stable capability/offer IDs, read/write/open-world/destructive/idempotency annotations, audience-isolated OAuth/managed credentials where applicable and exact mapping into the canonical Effect path. Catalog growth changes data returned by discovery, not the trusted MCP tool list. Public/tool surface metadata cannot claim a private/unmodeled endpoint is a safe read merely from HTTP method or name.

### T208/T209 UX constraints

The primary product may expose:

```text
Attention
Work
  Tasks
  Sessions
  Workers
Actions
Knowledge
Routines
Capabilities
Activity / Evidence
Settings
```

These are product views over canonical state. They are not additional authority databases.

```text
ATTENTION_CARD != TASK_TRUTH
ACTION_PROPOSAL != AUTHORIZED_EFFECT
ASSOCIATION_CONFIDENCE != CANONICAL_RELATION
BRIEFING_SUMMARY != SOURCE_OF_TRUTH
LEARNED_RULE != ACTIVE_AUTHORITY
```

## Phase T2 — Attention budget and proactive autonomy discipline

- [ ] **T212 — Attention Budget and Proactive Autonomy Policy.** Refine T208/T209 with explicit interruption governance so proactive Golam does not become intelligent notification spam. Define deduplication/coalescing, freshness/expiry, quiet/defer policy, current-focus awareness where available, user-configured urgency classes, briefing-vs-immediate routing, reason-for-surfacing, correction feedback and a bounded daily/periodic interruption budget where useful. Evaluate precision/recall for important items together with unnecessary-interruption rate, duplicate surfacing, stale-card rate, deferred-item recovery and correction stability. Corrections produce candidate rules/routines and never silently mutate protected active policy. `ATTENTION_SCORE != USER_PRIORITY_TRUTH`; `HIGH_MODEL_CONFIDENCE != INTERRUPT_NOW`; `PROACTIVE != ALWAYS_INTERRUPTIVE`.

## Phase U — Owner portfolio governance and cross-fabric proof

- [ ] **T211 — Owner Portfolio Reuse Matrix / Internal Donor Bridge.** Maintain a confidentiality-safe inventory of the founder-owned GitHub portfolio and map selected public/private components to measured Golam gaps. Consume the canonical owner-enumeration record in `competitive-source-register-supplement-2026-09-22.md` rather than maintaining an independent population count. At the current reverified record that population is 36 owner repositories: 29 public and 7 private. Public planning may name public sources; private names/content stay undisclosed unless separate publication authority exists. Reuse must route through the same T113/T165/T180/T197 exact-component Source Foundry record. For each selected component record current pin/tree, source role, selected paths, reuse strategy, dependency/runtime closure, rights/NOTICE, authority ceiling, TCB delta, benchmark/parity reason and removal/rollback path. Repository ownership or founder permission never auto-admits the code.

### T211 required role vocabulary

Use bounded roles rather than "merge everything":

```text
CANONICAL_GOLAM
PRIMARY_CAPABILITY_DONOR
BOUNDED_ADAPTER_CANDIDATE
HIGH_VALUE_ARCHITECTURE_REFERENCE
BENCHMARK_OR_METHOD_REFERENCE
DOMAIN_REFERENCE_ONLY
PROVENANCE_CAUTION_REFERENCE
NO_CURRENT_MEASURED_GAP
PRIVATE_CONSIDERED_UNDISCLOSED
```



## Phase V — Evidence fidelity, bounded delegation, skill replay and multidimensional proof

- [ ] **T213 — Evidence Fidelity, Coverage and Absence Contract.** Refine T149/T150/T156/T158/T177/T192 using the strongest owner-portfolio evidence semantics without creating a second Evidence Plane. Define one typed contract that separates: provider/vendor capability ceiling, adapter implementation fidelity, capture/observation activation, predicates actually observed, deterministic derivations, unsupported predicates, and explicit absence reasons. At minimum distinguish `NOT_OBSERVED` (observation was active and capable), `NOT_OBSERVABLE_AT_FIDELITY`, `CAPTURE_INACTIVE`, `UNSUPPORTED`, `FAILED`, `PARTIAL` and `UNKNOWN` where applicable. Evidence-producing adapters declare valid predicates/lifecycles by subject kind rather than forcing one universal lifecycle. Missing/failed/unsupported analysis never becomes a clean result by absence. Every nontrivial claim carries exact source/adapter/revision/fidelity evidence sufficient to explain what Golam could and could not know. This extends canonical Verification/Evidence semantics; it MUST NOT create a parallel finding/evidence authority.

- [ ] **T214 — Disclosure-Bound Agent Proposal and Review Checkpoint Contract.** Refine T120/T127/T133/T163/T167/T179/T192 with a bounded disclosure/proposal boundary for workers and external agents. Define an immutable `ContextDisclosureReceipt` binding exact disclosed object/source/revision identities, omissions/rejections, byte/token budget, sensitivity/taint, expiry, audience and capability ceiling. A returned `WorkerProposal` must cite the disclosure receipt and may target an existing canonical object only when that exact object/revision (or an explicitly permitted successor rule) was disclosed and the proposal carries current expected-revision preconditions. Disclosure exports no capability, approval, lease, secret or authority. Record proposal origin separately from owner/reviewer acceptance. Add an explicit `ReviewCheckpoint` / reviewed-through sequence marker so opening a view never implies review; resume projections are pinned to a stable canonical sequence and prioritize unresolved contradictions, stale evidence and blockers before ordinary continuation state.

- [ ] **T215 — Canonical Skill / Workflow IR, Deterministic Replay and Divergence Repair.** Refine T122/T123/T162/T163/T194/T199 before broad self-improving skill evolution. Define one versioned typed `WorkflowIR` / `SkillIR` carrying artifact/dataflow identities, capability requirements (never captured grants), disclosure constraints, side-effect/effect classes, preconditions, postconditions, verification obligations, retry/reconciliation semantics, compatibility assumptions and exact dependency/provider revisions. Separate `SkillCompiler` (human/verified trajectory -> authority-free candidate) from `DeterministicReplay` (fresh authorization, exact attempts/receipts) and from `DivergenceRepair` (failed-assumption detection, localized repair, downstream evidence/artifact invalidation, fresh authorization/re-verification, candidate version promotion). Successful exploratory work may reduce later model calls only when replay compatibility is proven. A syntactically/schema-valid compiled workflow is not automatically semantically faithful to the originating intent.

- [ ] **T216 — Multidimensional Verification and EvidenceBundle Contract.** Refine T149/T150/T157/T186/T193/T199 so consequential outcomes and durable artifacts are not collapsed into one green boolean. Define a versioned `EvidenceBundle` plus orthogonal verification dimensions suitable to the subject, such as source/input identity, output/content integrity, target/account identity, route/backend identity, authority/approval binding, operation/effect terminal state, constraint/postcondition satisfaction, verifier independence, coverage/fidelity, freshness/time basis, provider attestations and unresolved/unsupported dimensions. `VERIFIED_COMPLETE` requires the owning VerificationObligation to state which dimensions are mandatory and to fail closed when a mandatory dimension is invalid, missing or unknown. Prefer an independently implemented verifier/readback path for high-consequence artifacts/effects when practical; producer self-report alone cannot satisfy independent verification.

### T213–T216 hard invariants

```text
NOT_OBSERVED != NOT_POSSIBLE
NOT_OBSERVABLE != NOT_OCCURRED
CAPTURE_INACTIVE != NEGATIVE_EVIDENCE
NO_FINDING != CLEAN
CHECK_PASS != COMPLETE_COVERAGE
PARTIAL_ENFORCEMENT != ENFORCED
DISCLOSURE != AUTHORITY
UNDISCLOSED_OBJECT != VALID_PROPOSAL_TARGET
EXPORTED_CONTEXT != EXPORTED_CAPABILITY
AGENT_ORIGIN != OWNER_ACCEPTANCE
VIEW_OPENED != REVIEW_COMPLETED
SCHEMA_VALID != SEMANTICALLY_EQUIVALENT
COMPILABLE != FAITHFUL
REPLAY_COMPATIBLE != CURRENTLY_AUTHORIZED
REPAIR != SILENT_HISTORY_REWRITE
PRODUCER_SUCCESS != INDEPENDENT_VERIFICATION
EVIDENCE_BUNDLE != SINGLE_BOOLEAN
```

### T213–T216 acceptance direction

Future owning specs must prove, with bounded fixtures and at least one adversarial path per contract:

- unavailable capture/coverage cannot render as a negative fact or clean result;
- a worker cannot propose a mutation to undisclosed/stale protected state merely because it knows an identifier from another channel;
- exported context never carries live grants/approvals/secrets by implication;
- resume/checkpoint state is explicit and cannot be advanced by reading a screen;
- a compiled skill cannot retain demonstration-time authority and must acquire fresh authorization on replay;
- divergence invalidates only evidence/artifacts whose assumptions are affected, while preserving immutable prior history;
- a producer and independent verifier can disagree without the producer overwriting verification truth;
- mandatory unknown verification dimensions prevent `VERIFIED_COMPLETE`.



## Phase W — Realtime voice, audio and conversational presence

T196 remains the umbrella Voice Presence / Audio Authority contract. T217–T228 decompose it into bounded shared contracts so voice does not become one monolithic implementation package or a second authority path. The companion `voice-audio-completeness-matrix-2026-09-22.md` is the closure checklist for lifecycle ownership and acceptance.

- [ ] **T217 — Realtime Audio Session and Duplex Control Contract.** Refine T140/T169/T196/T199/T201 into one event-driven `AudioSession` contract with independently cancellable lifecycles for microphone capture, conditioning/VAD, STT, agent streaming, TTS synthesis and audio playback. Define typed events for microphone authority grant/revoke, device/route state, audio-source identity, speech start/end, partial/final transcript, turn candidate/commit, agent acknowledgement/stream, TTS first-audio/playback state, barge-in, mute/duck, cancellation request/settlement, route change and failure. Push-to-talk remains the reliability baseline; hands-free/full-duplex is explicit opt-in and MUST NOT imply always-listening. Where microphone capture remains active during TTS, bind acoustic echo cancellation/playback-reference behavior so Golam's own output cannot silently become user intent. Raw audio is ephemeral by default unless a separate recording/retention capability is explicitly authorized.

- [ ] **T218 — Speech Runtime Router, Capture Health and Model/Engine Qualification.** Refine T132/T152/T159/T170/T175/T191/T196 using the strongest Himsat/Wispral voice-runtime evidence. Define provider-neutral replaceable lanes such as `LIVE_GENERAL`, `LOW_RESOURCE`, `MULTILINGUAL_CODE_SWITCH`, `HIGH_ACCURACY`, `LONG_FORM` and an optional `SPEECH_NATIVE_DUPLEX` lane, with exact STT/VAD/conditioning/TTS or native speech-to-speech engine+artifact identity, device/backend/hardware, locality, privacy profile, latency/resource envelope and health evidence. Preserve capture/source health separately from recognition quality: attachment, route, silence/clipping/drop/backpressure, clock discontinuity, self-capture and device-change evidence. Route selection is locked for a bounded utterance/segment with explicit transition points and hysteresis; no hidden local->cloud fallback or mid-utterance model thrash. Candidate families from Himsat/Wispral remain challengers only until exact T175/Source Foundry qualification. Meta Muse Voice Transcribe is a current remote/hosted benchmark/provider reference for streaming ASR+endpointing+diarization+code-switch behavior, not a strict-local dependency and not proof that an equivalent local route exists. Any future local/open artifact is a new exact T175 object.

- [ ] **T219 — Streaming STT, Turn Detection and Semantic Speech Interpretation Contract.** Refine T192/T196/T203/T204/T213. Preserve separate representations for source/audio metadata or explicit non-persistence, partial transcript revisions, final raw transcript, normalized transcript, entity candidates/bindings, utterance semantic class, turn-decision evidence and final Task/Conversation input. Acoustic endpointing/VAD/prosody and semantic completeness are separate signals. A native speech-to-speech provider may emit its own incremental transcript/turn/tool-call stream, but Golam must project those outputs into the same transcript/turn/Task/Effect contracts and retain enough aligned evidence to audit consequential actions; speech-native tool output never bypasses the canonical Effect Gate. A T203 `DecisionProvider` such as a qualified OpenJev-style model may provide bounded textual decisions such as `COMMAND`/`ASIDE`/`QUESTION`/`DICTATION`/`CORRECTION`, semantic "respond now vs keep listening" hints, urgency or routing, but it is not an audio model and cannot replace VAD, authorize an Effect, authenticate a speaker, or make a high-risk target safe. Deterministic stop/cancel controls and explicit UI gestures bypass semantic-model latency. Partial/final transcript revisions must retain lineage rather than silently rewriting what the user said.

- [ ] **T220 — Streaming TTS, Barge-In and Voice Presence Contract.** Refine T196/T199/T201 around low-latency, interruptible speech output. Define `VoiceOutputSession` with exact TTS engine/model/voice/style identity, text/source revision, output data class, device route identity, route-policy generation, synthesis chunks, first-audio/last-audio timing, playback state, cancellation and provenance. Bind audible playback to the canonical T199 operation/effect classification and T201/T179 privacy policy for the approved output data class + exact route + route-policy generation. Immediately before each audible playback dispatch/chunk boundary, the protected playback path must revalidate that the current route still matches the permitted binding. If headphones disconnect, the route changes, the policy generation changes, or the new route no longer satisfies the data-class policy, Golam must mute/block pending audio and require a newly permitted route or fresh disclosure/approval; the TTS engine/playback adapter cannot decide this locally. Permit phrase/sentence streaming where bounded, but never speak unverified hidden reasoning or secrets. Sensitive output may require a safer route (screen/headset) or explicit disclosure/approval rather than ambient speaker playback. Barge-in must mute/duck audible output immediately on the protected control path and request agent/TTS cancellation without waiting for final transcription. Voice cloning/speaker imitation remains a separate consented capability and can never establish owner presence or authentication.

- [ ] **T221 — Conversational Work, Multi-Task Voice UX and Background Presence.** Refine T185/T200/T208/T209/T212 using Golam-research/Grok Bot and Meta Muse as product-behavior references without importing their authority models. Text and voice consume one canonical conversation/work spine: one main long-running conversation may accept additional user messages while work is active; explicit side chats/project scopes may isolate context; users can steer, interrupt, cancel, queue or branch tasks without waiting for the previous assistant turn to finish. Background work, Goals/Tasks, proactive updates, generated Artifacts and activity history are projections over canonical state. Reuse Grok-style transcript/permission-card lessons and stale-approval handling, but all consequential approvals remain deterministic Golam controls. Muse-style proactivity must obey T212 attention budgets; voice output is not used for unsolicited always-on announcements by default.

- [ ] **T222 — Golam VoiceBench, Multilingual Safety and Accessibility Qualification.** Extend T143–T150/T158/T178/T213/T216 with a reproducible voice/audio qualification profile before any "best voice" claim. Measure at minimum: STT WER/CER and entity accuracy; Arabic, English and code-switch conditions where qualified; first-partial/finalization latency; transcript revision stability; VAD miss/false activation/boundary error; semantic endpoint false-stop/late-stop; interruption-to-mute and interruption-to-agent-cancel latency; false barge-in from Golam playback/echo; TTS first-audio latency/RTF/intelligibility; route/device/Bluetooth changes; CPU/RAM/accelerator/battery/thermal behavior where applicable; long-session stability; network-denied strict-local behavior; and exact model/runtime provenance. Benchmark both modular cascade and native speech-to-speech candidates where eligible, keeping their interaction semantics visibly distinct; use current public references such as Muse Voice Transcribe for streaming-ASR/endpointing/diarization parity and NemotronLabs VoiceChat-class systems for native full-duplex/tool-calling behavior where lawful and reproducible. Include tool-selection/argument correctness, argument accuracy, interruption takeover and post-interruption recovery for any speech-native tool-calling path. Include adversarial source-channel cases proving system audio, media playback, remote-party speech and Golam's own TTS cannot become owner commands merely because they are transcribed. Every consequential voice control path retains a keyboard/touch/text accessibility fallback.



- [ ] **T223 — Activation, Wake, Source Attribution and Floor-Control Contract.** Refine T169/T196/T217/T219/T226 around who/what is currently allowed to open a voice turn. Push-to-talk remains the deterministic baseline. Optional hands-free/wake operation must be explicit opt-in, local-first by default, visibly enabled, independently revocable, bounded by a non-durable pre-roll/ring buffer, and qualified for false activation/rejection, accent/language, noise and device-resource cost. Every captured segment carries source-channel evidence when available so microphone speech, system audio, imported media, remote participants and Golam playback remain distinguishable. Diarization, speaker embeddings and floor ownership are contextual evidence only: they cannot authenticate a principal or mint authority. Define explicit floor/turn takeover, overlapping-speaker and unknown-speaker handling for meetings and shared spaces.

- [ ] **T224 — Critical Utterance Binding, Corrections and High-Risk Voice Confirmation.** Refine T174/T199/T219/T213/T216 for speech-specific ambiguity. Preserve raw and normalized transcript revisions, exact recognized number/name/path/hash/account/amount/date tokens, entity-resolution evidence, negation/correction lineage and uncertainty. Before a consequential voice-triggered action, bind the current interpretation to the exact pending operation/Effect, target/account/workspace/provider context and expected revisions. Any required voice confirmation is a bounded challenge tied to exactly that pending operation, session generation and expiry; a generic/replayed "yes", Golam TTS, media audio or another speaker cannot approve it. High-risk ambiguity fails closed or requests explicit read-back/text confirmation according to policy.

- [ ] **T225 — Audio Transport, Clock, Device and Remote-Stream Resilience.** Refine T169/T217/T218/T220/T205 for sample/transport/session integrity. Define canonical audio-frame metadata for source, device, codec/PCM format, sample rate/channels, monotonic timestamps, session generation, packet/buffer sequence and discontinuities. Cover local device hot-plug, Bluetooth/profile switches, sleep/wake, capture/playback underrun/overrun, bounded backpressure, remote GolamConnect audio jitter/loss/reorder/reconnect, source-clock drift and resynchronization. A reconnect, engine restart or route change must not replay buffered audio into a new session, revive a cancelled utterance or silently rewrite transcript/source-time truth.

- [ ] **T226 — Voice Privacy, Consent, Retention, Biometrics and Abuse-Safety Contract.** Refine T174/T179/T196/T201/T217/T220. Separate microphone permission, active-listening visibility, optional wake buffer, raw-audio retention, transcript retention, cloud processing, speaker embeddings/biometrics, diagnostic recordings and voice cloning/imitation into independently governed capabilities. Raw audio is ephemeral by default; persistence requires explicit purpose, duration, data-class policy, export/delete/reset behavior and exact storage boundary. Speaker similarity is never sole authentication. Add adversarial coverage for replayed owner recordings, deepfake/clone speech, malicious media/TV/video audio, remote-party commands, Golam self-TTS, malformed/corrupt audio, stale wake buffers and resource-exhaustion streams. No hidden recording or content-bearing remote telemetry.

- [ ] **T227 — Language, Lexicon, Pronunciation, Voice Persona and Accessibility Contract.** Refine T192/T196/T219/T220/T222 for user-specific speech quality without weakening truth. Support evidence-scoped English, Arabic and Arabic/English code-switch qualification, with dialect/accent claims only where a corpus exists. Define bounded user lexicons/hotwords/pronunciation dictionaries for names, organizations, repository identifiers and technical vocabulary; preserve raw ASR and measure false-insertion cost. TTS preferences such as language, voice, pace and pronunciation are user-owned presentation configuration. Voice cloning/imitation requires explicit consent/provenance and remains non-identity. Require captions/live transcript where available, text/keyboard/touch alternatives, adjustable playback and non-audio presentation for critical confirmations; no consequential feature may be voice-only.

- [ ] **T228 — Voice Reliability, Graceful Degradation, Model Lifecycle and Release Gates.** Refine T132/T143–T150/T175/T178/T218/T222. Every qualified voice route publishes exact artifact/runtime identity plus measured latency, accuracy, resource, stability and privacy behavior by hardware/language/profile. Define deterministic degradation: TTS failure -> text; unreliable AEC -> push-to-talk/headphones profile; preferred local STT unavailable -> another admitted local route or explicit unavailable; cloud denied/network down -> no cloud fallback; device loss -> stop/mute/rebind; resource/thermal pressure -> admitted lower-resource route or explicit pause. Updates require benchmark delta and rollback to the last qualified route. Release evidence must include long-session/crash/recovery, device changes, strict-local network denial and zero-tolerance authority/privacy gates; no aggregate quality score may compensate for an unauthorized Effect or privacy violation.

### T217–T228 hard invariants

```text
AUDIO_SESSION != AUTHORITY
MICROPHONE_AUDIO != OWNER_IDENTITY
SYSTEM_AUDIO != OWNER_COMMAND
REMOTE_PARTY_SPEECH != OWNER_COMMAND
TTS_OUTPUT != USER_INTENT
FULL_DUPLEX != ALWAYS_LISTENING
PARTIAL_TRANSCRIPT != FINAL_USER_INTENT
NORMALIZED_TRANSCRIPT != SOURCE_AUDIO_TRUTH
TEXT_DECISION != ACOUSTIC_ENDPOINT
DECISION_PROVIDER != VOICE_AUTHORITY
VAD_EVENT != COMMAND_AUTHORIZATION
FINAL_TRANSCRIPT != STOP_PREREQUISITE
VOICE_ROUTE_FALLBACK != PRIVACY_DOWNGRADE
PREPARED_TTS_ROUTE != CURRENT_PLAYBACK_PERMISSION
OUTPUT_ROUTE_CHANGE != DISCLOSURE_CONTINUITY
SPEECH_NATIVE_TOOL_CALL != EFFECT_AUTHORIZATION
NATIVE_DUPLEX_OUTPUT != VERIFIED_TRANSCRIPT
VOICE_CLONE != OWNER_PRESENCE
CHAT_BUBBLE != TASK_BOUNDARY
PROACTIVE_VOICE != UNSOLICITED_ALWAYS_ON_AUDIO
WAKE_DETECTED != OWNER_AUTHENTICATED
SPEAKER_MATCH != OPERATION_AUTHORIZED
DIARIZATION_LABEL != PRINCIPAL_IDENTITY
GENERIC_YES != BOUND_OPERATION_APPROVAL
RECONNECTED_STREAM != SAME_AUDIO_SESSION
BUFFERED_AUDIO != CURRENT_USER_INTENT
RAW_AUDIO_RETENTION != TRANSCRIPT_RETENTION
PERSONAL_LEXICON != SOURCE_TRANSCRIPT_TRUTH
DEGRADED_ROUTE != POLICY_RELAXATION
```

### T217–T228 acceptance direction

Future owning specs must prove, with exact instrumentation rather than subjective demos:

- microphone capture, STT, agent work, TTS and playback can be cancelled independently without leaking stale callbacks across account/agent/session fences;
- in hands-free mode, Golam can hear a real user interruption while rejecting/discounting its own playback echo, and the stop path does not wait for final ASR;
- sensitive audio prepared for a permitted headphone route emits no sensitive samples after a pre-playback route change to ambient speakers until the new route independently satisfies policy or receives fresh disclosure/approval;
- a stale/partial transcript revision cannot authorize or target a consequential Effect;
- semantic turn models can be removed/replaced without changing deterministic authority semantics;
- local speech routes stay local under network denial and never silently fail over to cloud;
- system audio/media/remote participants cannot be promoted to owner commands by transcript content alone;
- multiple user utterances/messages arriving while Golam is working are attributed to explicit steer/queue/branch semantics rather than flattened into one ambiguous turn;
- a screen-only/text fallback exists for every consequential voice interaction;
- voice model/runtime updates preserve exact artifact identity, benchmark deltas and rollback to the last qualified local route;
- wake-word activation cannot authenticate the speaker or authorize an Effect;
- a replayed/generic confirmation cannot approve an unrelated or stale operation;
- reconnect/restart/device-switch cannot replay stale buffered audio into a new session;
- raw audio cannot persist beyond declared policy by crash/restart or provider behavior;
- language/lexicon personalization cannot silently rewrite raw transcript evidence;
- graceful degradation never widens privacy, provider, account, authority or Effect policy.



- [ ] **T229 — OpenJev First-Class Decision Provider Qualification and Adapter.** Execute the bounded integration plan in `openjev-integration-plan-2026-09-22.md` after an owning implementation lifecycle is authorized. Treat exactly `AlexWortega/openjev` as an optional, replaceable T203 `DecisionProvider` target. T175 must first freeze exact immutable model/config/tokenizer/safetensors identities, base-model/transitive rights, runtime/backend, offline behavior and rollback artifact. Then implement one bounded `OpenJevDecisionAdapter` that accepts canonical `DecisionRequest` and emits `DecisionReceipt` only; it MUST NOT own routing policy, authority, egress, Effect decisions, owner presence, verification truth or fallback policy. Qualify OpenJev independently for bounded workloads such as voice utterance class, semantic turn-completeness, clarification-needed, Attention triage, capability fit, provider-candidate ranking and retrieval reranking. Each workload gets its own calibration/applicability/abstention profile and can be rejected independently. Prove replacement/removal, out-of-domain abstention, transcript-revision invalidation for voice, adversarial/tainted-input handling, quantization/backend drift, resource/latency envelopes, strict-local network-denied behavior where claimed, and deterministic escalation to stronger providers/human review where required.

### T229 hard invariants

```text
OPENJEV != AUTHORITY
OPENJEV != POLICY_ENGINE
OPENJEV != EFFECT_GATE
OPENJEV != OWNER_PRESENCE
OPENJEV != VERIFICATION_ORACLE
OPENJEV != STT
OPENJEV != VAD
OPENJEV != TTS
OPENJEV_HIGH_SCORE != ALLOW
OPENJEV_DECISION != VERIFIED_FACT
OPENJEV_UNAVAILABLE != GOLAM_UNAVAILABLE
OPENJEV_REMOVAL != AUTHORITY_SEMANTICS_CHANGE
```

### T229 acceptance direction

A future owning spec must prove:

- exact artifact and runtime identity are frozen and independently reproducible;
- founder permission plus transitive rights/NOTICE obligations are reconciled for the exact artifact/runtime;
- at least one bounded workload beats or materially improves the baseline on a preregistered latency/quality/resource objective without violating safety/authority gates;
- every admitted workload has explicit calibration/applicability/abstention evidence rather than a generic confidence threshold;
- a high-confidence wrong/out-of-domain result cannot grant authority, lower consequence, satisfy approval or create verified completion;
- stale partial-transcript decisions are invalidated when the transcript revision they depend on changes;
- removal/unavailability falls back or escalates without widening locality, privacy, spend, account or authority;
- Golam remains correct when OpenJev is disabled entirely.

## Cross-fabric ownership rule

Before T203–T229 implementation, T198's Canonical Shared-Contract Ownership Matrix must name the sole owner/version source/migration authority for at least:

- TaskContract / Task-Session-Run-Worker identities;
- ExecutionEnvelope;
- Principal/capability lease;
- Operation/Effect ontology and Effect ledger;
- Route applicability;
- Egress/privacy;
- Secret/account binding;
- VerificationObligation/Receipt;
- canonical events;
- source/extension admission;
- ContextBundle/RetrievalReceipt;
- DecisionRequest/Receipt;
- CapabilityDefinition/Offer;
- AttentionItem/ActionProposal projections;
- EvidenceCoverage/Fidelity and absence semantics;
- ContextDisclosureReceipt / WorkerProposal / ReviewCheckpoint;
- WorkflowIR/SkillIR and replay/divergence semantics;
- EvidenceBundle / multidimensional verification projections;
- AudioSession / voice event and source-channel semantics;
- SpeechRuntimeRoute / CaptureHealth / engine-model identity;
- SpeechInterpretation / turn-decision receipts;
- VoiceOutputSession / playback and barge-in state;
- conversational steer/queue/branch projection semantics;
- VoiceActivation / WakeSession / source-channel / floor-control semantics;
- CriticalUtteranceBinding / VoiceConfirmationChallenge semantics;
- AudioTransport / clock / stream-generation semantics;
- VoiceRetention / biometric / consent semantics;
- VoiceProfile / lexicon / pronunciation semantics;
- VoiceRouteDegradation / release-gate semantics;
- OpenJevDecisionAdapter / workload qualification / calibration-profile semantics.

No owning package may invent package-local protected truth for one of these concepts.

## Source-specific qualification requirements

### Google AX

Use as architecture/reference candidate first. Do not import Kubernetes, Redis or Agent Substrate into the local baseline merely to reproduce AX topology. Any bounded code reuse requires normal Source Foundry qualification.

### Treg

The public repository currently carries Apache-2.0 plus additional hosted-service restrictions. The founder separately asserts permission to use/copy the source. Any code admission for distributed/commercial embedding must bind documentary permission scope for the intended use in the exact component admission record. No implicit waiver is inferred.

### SemIf / Decider / Nimble

Treat code and model artifacts separately. A code license never auto-admits weights, base models, datasets, teacher outputs, quantizations or runtime backends. Model Artifact Foundry T175 applies to every selected artifact.

### Laya

Prefer behavior/UX and bounded component patterns. Its n8n/Python/Chroma topology is not a Golam trusted-path requirement.

### TinyFish / Desktop Commander

Adapters remain outside protected authority. Hosted variants are explicit non-strict capabilities. Anti-bot/stealth behavior is not a Golam product goal. Broad host/process capability does not bypass Golam isolation and Effect policy.

### OpenJev / Jev-style decision models

Treat OpenJev as a T203/T219 decision-provider candidate, not as STT, VAD, TTS or speaker authentication. Its current public artifact is a text-classification / NLI-style Qwen3.5-derived model and may also expose multimodal decision behavior, but voice use must consume transcript/context state through the typed DecisionProvider contract. T175 applies to exact model weights/tokenizer/config/runtime; no current Hub branch name substitutes for immutable artifact digests.

### Golam-research / Grok Bot 0.18 reconstruction

The recovered voice surface is a useful push-to-talk and permission/transcript UX baseline: bounded recording, microphone permission handling, cancellation, agent/account scoping, transcript cards and stale approval handling. It is not the target full-duplex architecture because its recovered voice path records a clip and transcribes after stop. Reuse remains subject to the repository's special provenance handling and normal Source Foundry qualification.

### Meta Muse

Muse is a behavior/security architecture reference only; no Meta proprietary source-code reuse is inferred. Keep the Muse family separated by role: Muse Agent/Spark behavior is a collaboration/harness reference; Muse Voice Transcribe is a streaming speech benchmark/provider reference; Muse Glimmer is an Apache-2.0 open-weight local agent-model candidate for ordinary T175/T152 qualification, not a speech engine. Retain the product lessons that fit Golam: long-running conversation that accepts new input while work continues, explicit side chats, background goals/work, proactive-but-sparse updates, activity transparency, rich artifacts and deterministic approval controls. Retain the security lessons conceptually: runtime isolation, a permission/egress authority outside the model, credential surrogation/just-in-time insertion and tainted-untrusted external data. Map these onto Golam's existing Authority Kernel, Effect Gate, Secret/Account broker, T179 taint/egress and T173 isolation rather than creating a second Sentinel authority.

## Priority and sequencing

Recommended dependency order after an owning bounded lifecycle is authorized:

```text
P0_CANONICAL_PREREQUISITES:
  T112
  T149/T150
  T156/T177/T179
  T165/T180/T197
  T167
  T199
  T201

P0_NEW_SHARED_CONTRACTS:
  T203 Decision Provider Contract
  T206 Capability Catalog / Provider Offer Contract
  T213 Evidence Fidelity / Coverage / Absence Contract
  T214 Disclosure-Bound Agent Proposal / Review Checkpoint Contract
  T215 Canonical Skill / Workflow IR + Replay / Divergence Contract
  T216 Multidimensional Verification / EvidenceBundle Contract
  T211 Owner Portfolio Reuse Matrix (cross-cutting governance)
  T212 Attention Budget / Proactive Autonomy Policy

P1_NEW:
  T204 Decision Calibration / Escalation
  T205 ExecutionEnvelope / Reconciliation
  T207 Credential-Brokered Tool Relay
  T229 OpenJev First-Class Decision Provider Qualification / Adapter

P1_PRODUCT_PROJECTION:
  T208 Proactive Attention / Action Proposal

P1_VOICE_FOUNDATION:
  T217 Realtime Audio Session / Duplex Control
  T218 Speech Runtime Router / Capture Health
  T219 Streaming STT / Turn Detection / Semantic Speech Interpretation

P2_VOICE_EXPERIENCE:
  T220 Streaming TTS / Barge-In / Voice Presence
  T221 Conversational Work / Multi-Task Voice UX
  T222 Golam VoiceBench / Multilingual Safety / Accessibility
  T223 Activation / Wake / Source Attribution / Floor Control
  T224 Critical Utterance Binding / Voice Confirmation
  T225 Audio Transport / Clock / Device Resilience
  T226 Privacy / Consent / Retention / Biometrics / Abuse Safety
  T227 Language / Lexicon / Pronunciation / Accessibility
  T228 Reliability / Degradation / Model Lifecycle / Release Gates

P2_AFTER_FOUNDATIONS:
  T209 Cross-Source Coherence / Briefing
  T210 Compact MCP Capability Surface
```

This priority is architectural, not authorization. Live successor authority after Spec 006 closeout decides actual spec numbering and activation order.

## New hard invariants

```text
DECISION_PROBABILITY != AUTHORITY
DECISION_PROVIDER != POLICY_ENGINE
CALIBRATED_PROBABILITY != VERIFIED_FACT
DECISION_PROVIDER_OUTPUT != OWNER_APPROVAL
MODEL_ROUTE_RECOMMENDATION != EFFECT_PERMISSION
WORKLOAD_MANIFEST != CAPABILITY_GRANT
WORKSPACE_BINDING != FILESYSTEM_AUTHORITY
GATEWAY_DECLARATION != EGRESS_AUTHORIZATION
RUNTIME_READY != TASK_VERIFIED_COMPLETE
RUNTIME_RECONCILIATION != EFFECT_RETRY
CAPABILITY_CATALOG_ENTRY != TOOL_ADMISSION
CAPABILITY_OFFER != EFFECT_PERMISSION
CREDENTIAL_BINDING != CALL_AUTHORIZATION
TOOL_RELAY_SUCCESS != EFFECT_VERIFIED
ATTENTION_CARD != TASK_TRUTH
ACTION_PROPOSAL != AUTHORIZED_EFFECT
ASSOCIATION_CONFIDENCE != CANONICAL_RELATION
BRIEFING_SUMMARY != SOURCE_OF_TRUTH
LEARNED_RULE != ACTIVE_AUTHORITY
OWNER_REPOSITORY_ACCESS != SOURCE_ADMISSION
FIELD_PROBABILITY != JOINT_CONSISTENCY
STALE_WORKER != CURRENT_EFFECT_ACTOR
BUDGET_AUTHORIZATION != EFFECT_AUTHORIZATION
ATTENTION_SCORE != USER_PRIORITY_TRUTH
HIGH_MODEL_CONFIDENCE != INTERRUPT_NOW
NOT_OBSERVED != NOT_POSSIBLE
NO_FINDING != CLEAN
DISCLOSURE != AUTHORITY
UNDISCLOSED_OBJECT != VALID_PROPOSAL_TARGET
SCHEMA_VALID != SEMANTICALLY_EQUIVALENT
PRODUCER_SUCCESS != INDEPENDENT_VERIFICATION
AUDIO_SESSION != AUTHORITY
MICROPHONE_AUDIO != OWNER_IDENTITY
SYSTEM_AUDIO != OWNER_COMMAND
FULL_DUPLEX != ALWAYS_LISTENING
PARTIAL_TRANSCRIPT != FINAL_USER_INTENT
TEXT_DECISION != ACOUSTIC_ENDPOINT
VAD_EVENT != COMMAND_AUTHORIZATION
VOICE_ROUTE_FALLBACK != PRIVACY_DOWNGRADE
PREPARED_TTS_ROUTE != CURRENT_PLAYBACK_PERMISSION
OUTPUT_ROUTE_CHANGE != DISCLOSURE_CONTINUITY
SPEECH_NATIVE_TOOL_CALL != EFFECT_AUTHORIZATION
NATIVE_DUPLEX_OUTPUT != VERIFIED_TRANSCRIPT
```

## End-to-end proving journeys

Future GolamBench/Spec 010 should eventually include at least these cross-fabric journeys:

### Outcome request

```text
intent
-> TaskContract
-> capability discovery
-> bounded decision routing
-> isolated execution
-> evidence/context
-> ActionProposal
-> no unauthorized external write
-> VerificationReceipts
-> verified terminal state
```

### Proactive morning

```text
connector events
-> canonical ingest
-> triage
-> coherence/context
-> briefing/Attention
-> proposed actions
-> user approval
-> Effect Gate
-> dispatch/reconciliation
-> verification
```

### Provider failure

Preferred local/provider route becomes unavailable mid-task. Prove that fallback does not silently widen privacy, account, cost or authority and that dependent work does not proceed through ambiguous Effects.

### Resume after crash

Crash after an external request may have been accepted but before terminal evidence is persisted. Prove that Task/worker resume does not blind-retry and reconciles `UNKNOWN_OUTCOME`.

### Catalog poisoning/account confusion

Inject stale/malicious capability metadata and multiple provider accounts. Prove exact destination/account/effect semantics are independently bound and wrong-account dispatch is denied.

### Decision overconfidence

Feed an out-of-domain/high-confidence wrong DecisionProvider result. Prove deterministic policy and required authoritative verification prevent it from granting protected authority or verified completion.

## Current safe sequencing

1. Keep active Spec 006 PR #24 unchanged in scope.
2. Treat T203–T229 as planning-only extension tasks.
3. Qualify the planning PR on its exact new head after this extension.
4. Re-run independent architecture/security/governance review because the planning head changed.
5. Merge planning only after new-head findings and required checks are reconciled.
6. Do not change `specs/CURRENT.md` because no durable implementation lifecycle state changed.
7. After Spec 006 closes canonically, fetch live successor authority before creating any owning implementation spec.
8. Never use this file itself as implementation permission.

```text
ACTIVE_SPEC_006_PR_24_WIDENED=NO
CURRENT_POINTER_CHANGED=NO
CONSTITUTION_CHANGED=NO
NEW_PRODUCT_IMPLEMENTATION_STARTED=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
WAIVER_TAKEN=NO
```
