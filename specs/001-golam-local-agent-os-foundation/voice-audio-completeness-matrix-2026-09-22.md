# Golam Voice / Audio Completeness Matrix — 2026-09-22

**Status:** PLANNING ONLY  
**Scope:** Completeness closure for the T196 umbrella and T217–T228 voice/audio program  
**Authority:** This document grants no implementation authority, dependency admission, model admission, cloud-provider admission, or Spec 006 scope expansion.

## 1. Product outcome

Golam voice is not a dictation feature and not a separate chatbot. Voice is a first-class input/output surface over the same canonical Conversation / Task / Effect / Authority / Evidence system used by text, UI, automation and other control surfaces.

The target experience is:

- user can speak or type naturally into the same durable work spine;
- Golam can begin understanding before an utterance fully ends;
- Golam can respond with low-latency streaming speech;
- the user can interrupt spoken output immediately;
- multiple user turns can steer, queue, branch or cancel long-running work;
- strict-local voice remains useful with networking disabled;
- optional remote/cloud speech never silently weakens privacy, authority or spend policy;
- system audio, media, remote participants and Golam's own TTS cannot become owner commands merely because they are transcribed;
- every consequential action still passes the ordinary Golam authority/effect path;
- every consequential voice control has a visible/text/touch/keyboard fallback.

## 2. Canonical voice pipeline

Two runtime shapes are permitted behind the same contracts.

### 2.1 Modular cascade

\`\`\`text
explicit activation / mic capability
-> source + device binding
-> capture health
-> conditioning / AEC / NS / AGC
-> VAD / acoustic endpoint evidence
-> streaming STT revisions
-> entity + language + semantic-turn interpretation
-> Conversation / Task spine
-> agent / worker / tool proposal
-> Authority / Effect Gate
-> agent response stream
-> streaming TTS
-> protected output-route revalidation
-> audio playback
\`\`\`

### 2.2 Native duplex

\`\`\`text
explicit activation / mic capability
-> qualified speech-native duplex provider
-> incremental transcript / turn / response / proposed-tool stream
-> canonical Golam transcript + Conversation / Task projections
-> Authority / Effect Gate for every consequential tool/effect
-> protected playback
\`\`\`

A native duplex model may collapse compute stages, but it MUST NOT collapse canonical authority, Effect, privacy, provenance or verification boundaries.

## 3. End-to-end lifecycle ownership matrix

| Lifecycle concern | Canonical owner(s) | Required behavior | Minimum proof |
| --- | --- | --- | --- |
| Activation / push-to-talk | T217, T223 | explicit visible activation; push-to-talk baseline; session-scoped mic capability | permission-denied, revoke, stale-session and rapid start/stop tests |
| Optional wake word | T223, T226 | opt-in, local-first detector, bounded non-durable buffer, visible enabled state | false activation/rejection, disable/reset, privacy and battery evidence |
| Microphone capability | T196, T217, T226 | short-lived least privilege; no ambient authority | revocation before/while capture and session fence tests |
| Source identity | T217, T223 | distinguish mic, system audio, remote participant, imported media and Golam playback | source-spoof and channel-mix adversarial fixtures |
| Speaker / floor attribution | T223 | diarization/floor evidence may guide UX; never authentication | overlapping-speaker, speaker-switch and unknown-speaker cases |
| Capture health | T218, T225 | attachment, silence, clipping, drops, backpressure, device route, discontinuity | hot-plug, sleep/wake, Bluetooth and buffer-overrun tests |
| Clock / timestamp integrity | T225 | monotonic timestamps, drift detection, discontinuity markers and bounded resync | long-session drift and multi-source clock tests |
| Conditioning / AEC / NS / AGC | T217, T218, T225 | replaceable; measurable speech damage; playback-reference aware | quiet-speech/noise/echo benchmark and self-TTS rejection |
| VAD / endpointing | T218, T219, T222 | acoustic evidence separated from semantic turn completion | false activation/miss/boundary, late-stop/false-stop metrics |
| Streaming STT | T218, T219, T222 | partial revisions + final transcript with lineage | first-partial/final latency, revision stability, WER/CER |
| Language / code-switch | T218, T227, T222 | route and transcript language state; bounded switching; no mid-utterance thrash | Arabic/English/code-switch panels where qualified |
| Custom lexicon / names / repo entities | T219, T224, T227 | bounded vocabulary hints; preserve raw ASR; measure false insertion | entity recall gain vs collateral false-insertion benchmark |
| Normalization | T219, T224 | raw transcript retained; normalized form is derivative | numbers, units, paths, hashes, dates and punctuation fixtures |
| Semantic turn classification | T203, T204, T219 | bounded decision provider; abstain/escalate; no authority | out-of-domain high-confidence and replacement-provider tests |
| COMMAND / ASIDE / QUESTION / CORRECTION | T219, T223 | semantic labels inform UX only; deterministic policy remains authoritative | ambiguity and mid-turn correction fixtures |
| Critical entity binding | T224 | exact target/value revision for risky actions | homophone, near-match, stale-target and ambiguity tests |
| Voice confirmation | T174, T199, T224 | confirmation binds exact pending operation, scope, generation and expiry | unrelated "yes", replayed audio and stale-confirmation rejection |
| Conversation concurrency | T167, T221 | explicit steer / queue / branch / cancel semantics | overlapping input while worker active and reconnect/resume tests |
| Background goals/tasks | T167, T208, T212, T221 | voice is projection/control surface, not separate task truth | task-state reconciliation and stale projection tests |
| Tool proposals | T199, T219 | speech-native/model tool call is proposal only | forced tool-call injection and wrong-argument denial |
| Consequential Effect | T149, T150, T199 | normal Effect Gate + verification; voice grants no bypass | unauthorized effect = zero-tolerance release gate |
| TTS synthesis | T220, T227 | exact engine/model/voice/style/text revision; no hidden reasoning/secrets | provenance, cancellation and secret-redaction tests |
| Output privacy | T179, T201, T220, T226 | bind data class + exact route + policy generation; revalidate at playback | headphones->speaker zero-sensitive-sample test |
| Barge-in / emergency stop | T196, T217, T220 | immediate mute/duck + cancellation request without final transcript | interruption-to-mute/cancel latency and race tests |
| Self-echo / playback injection | T217, T222, T223 | Golam TTS cannot become owner intent | speaker playback, echo path and replay attack fixtures |
| Remote participant speech | T223, T225 | participant speech stays untrusted/context unless explicitly selected | meeting/video/remote-party imperative-command fixtures |
| Raw audio retention | T217, T226 | ephemeral default; recording is separate explicit capability | crash cleanup, expiry, export/delete and retention-policy tests |
| Transcript retention | T192, T201, T226 | separate from raw-audio retention; privacy/profile governed | delete/export, profile switch and cloud-retention tests |
| Speaker embeddings / biometrics | T174, T226 | opt-in, encrypted/resettable/deletable if ever used; never sole auth | spoof/replay/deepfake and reset/delete tests |
| Cloud speech | T179, T201, T218, T226 | explicit non-strict route; exact provider/account/retention disclosure | network-denied strict-local and no-silent-fallback tests |
| Remote device audio | T169, T225 | GolamConnect-bound device/session/source identity | disconnect/reconnect, stale device and route-confusion tests |
| Device hot-plug | T220, T225 | safe route change; no stale playback/capture binding | mic/headset/Bluetooth insertion/removal races |
| Network jitter / packet loss | T225 | bounded buffers, cancellation, no stale replay | loss/reorder/reconnect/backpressure fixtures |
| Resource pressure | T132, T218, T228 | deterministic downgrade/stop by qualified route; no privacy downgrade | CPU/RAM/thermal/battery pressure tests |
| Model/runtime update | T175, T218, T228 | exact artifacts; qualification delta; rollback to prior working route | corrupted artifact, incompatible update and rollback tests |
| Crash / restart | T167, T205, T217, T225, T228 | no stale callbacks, duplicated Effects or phantom playback | crash during partial STT, approval, TTS and barge-in |
| Personal pronunciation / voice style | T227 | user-owned profile; presentation only; consent for imitation | reset/export and provider replacement tests |
| Accessibility | T222, T227 | captions, text/keyboard/touch alternatives; adjustable pace and output | no voice-only consequential path |
| Observability | T178, T213, T222, T228 | typed latency/health events without raw-content telemetry by default | trace privacy and missing-capture honesty tests |
| Release claims | T216, T222, T228 | route/hardware/language-specific evidence; no global "best" without scope | reproducible exact-artifact benchmark bundle |

## 4. Activation, wake and floor-control requirements

Push-to-talk is the deterministic reliability baseline. Hands-free and wake-word operation are optional capabilities.

An optional wake-word path must provide:

- local processing by default;
- explicit enable/disable and visible state;
- bounded, non-durable pre-roll/ring buffer;
- exact detector/model identity;
- false-accept / false-reject evidence;
- accent/language and noise evidence;
- battery/thermal evidence on mobile devices;
- immediate disable/reset;
- no implicit authority elevation after activation.

Wake detection opens an input session. It does not authenticate the speaker or authorize an operation.

For multi-person environments, every audio frame/segment should carry source-channel evidence when available. Speaker diarization, speaker embeddings and floor ownership are contextual evidence only.

\`\`\`text
WAKE_DETECTED != OWNER_AUTHENTICATED
SPEAKER_MATCH != OPERATION_AUTHORIZED
DIARIZATION_LABEL != PRINCIPAL_IDENTITY
CURRENT_FLOOR != CAPABILITY_GRANT
\`\`\`

## 5. Critical utterance and confirmation requirements

Voice has failure modes that text does not: homophones, numbers, paths, hashes, amounts, names, negation and corrections.

Before consequential execution, the interpretation layer must preserve and bind:

- raw transcript revision;
- normalized transcript revision;
- exact recognized target/value tokens;
- entity-resolution evidence;
- ambiguity/uncertainty;
- current pending Effect/operation identity;
- current workspace/account/provider/route context;
- confirmation challenge identity where required.

High-risk confirmation must bind to one exact pending operation and expire. A free-floating "yes", replayed audio, TTS output or a response from another speaker cannot approve a different operation.

Examples needing explicit uncertainty handling include:

- file/repository/path names;
- shell commands and flags;
- package/crate/module names;
- contacts/accounts/destinations;
- money/quantities/dates/times;
- medication-like numbers/units in any future sensitive domain;
- irreversible delete/send/publish/purchase actions;
- branch/SHA/version identifiers.

\`\`\`text
TRANSCRIPT_CONFIDENCE != TARGET_CERTAINTY
GENERIC_YES != BOUND_OPERATION_APPROVAL
NORMALIZED_NUMBER != SOURCE_NUMBER_TRUTH
ENTITY_CANDIDATE != EXACT_TARGET_BINDING
\`\`\`

## 6. Audio transport and resilience requirements

The voice plane must make transport and timing explicit rather than hiding them inside an SDK.

Required evidence includes:

- sample rate / channels / encoding;
- source-device identity;
- monotonic source timestamps;
- buffering and backpressure;
- packet loss/reorder for remote streams;
- capture/playback underrun/overrun;
- device hot-plug;
- Bluetooth/profile switching;
- suspend/resume;
- clock drift and resynchronization;
- route changes;
- cancellation/drain state;
- session generation.

A reconnect or engine restart cannot replay stale audio into a new session or revive a cancelled utterance.

\`\`\`text
RECONNECTED_STREAM != SAME_AUDIO_SESSION
BUFFERED_AUDIO != CURRENT_USER_INTENT
DEVICE_ROUTE_NAME != ROUTE_CONTINUITY
CLOCK_RESYNC != TRANSCRIPT_REWRITE_AUTHORITY
\`\`\`

## 7. Privacy, consent, retention and abuse safety

Voice privacy covers more than network egress.

The plan must distinguish:

- microphone permission;
- active listening visibility;
- optional wake-word buffering;
- raw audio retention;
- transcript retention;
- TTS output disclosure;
- optional cloud processing;
- optional speaker biometrics;
- optional voice cloning/imitation;
- diagnostic/benchmark recordings.

Raw audio remains ephemeral by default. Retention requires explicit purpose, duration, deletion/export behavior and data-class policy.

No hidden always-listening mode. No covert recording. No use of a cloned/generated voice as identity proof. No silent upload of raw audio/transcripts for telemetry.

Adversarial tests should include:

- replayed owner recordings;
- synthetic/deepfake owner-like speech;
- media/TV/video commands;
- remote participant commands;
- Golam's own TTS;
- very quiet / distorted / overlapping speech;
- malicious spoken prompt injection in untrusted media;
- stale wake-word buffers;
- malformed/corrupt audio;
- resource-exhaustion audio streams.

## 8. Language, personalization and accessibility

Language behavior is a route qualification problem, not a marketing label.

The plan must support evidence-scoped qualification for:

- English;
- Arabic;
- Arabic dialect variation where corpus evidence exists;
- Arabic/English code-switch;
- user names, organizations and project terminology;
- repository identifiers and technical vocabulary;
- numbers, units and dates;
- configurable TTS language, voice, pace and pronunciation.

Personal lexicons/hotwords are user-owned derivative configuration. They must be reversible and auditable, and false-insertion costs must be measured.

Voice persona and cloning are presentation features only. Speaker imitation requires explicit consent/provenance.

Accessibility requirements include:

- live transcript/captions where available;
- text fallback;
- keyboard/touch stop and approval controls;
- adjustable playback rate;
- non-audio presentation of critical confirmations;
- no consequential feature available only by speech.

## 9. Graceful degradation

Voice should degrade by capability, not by hidden policy relaxation.

Examples:

\`\`\`text
TTS unavailable          -> text response
streaming STT unavailable -> explicit push-to-talk / bounded batch route if qualified
AEC unreliable           -> push-to-talk or headphones-required profile
preferred local model unavailable -> another admitted local route or explicit unavailable
network unavailable      -> strict-local route only
cloud denied             -> no cloud fallback
device lost              -> stop/mute, require rebind
thermal/resource pressure -> admitted lower-resource route or explicit pause
\`\`\`

A downgrade cannot silently change:

- privacy profile;
- destination/provider;
- data retention;
- account;
- authority;
- Effect class;
- verification requirements.

## 10. Voice model and runtime lifecycle

Every speech artifact follows T175 and Source Foundry.

Record at minimum:

- source/repository/model namespace;
- immutable revision/file hashes;
- license and NOTICE;
- tokenizer/config/voice assets;
- quantization/conversion provenance;
- runtime/backend/native-library identity;
- supported hardware;
- model/parser/custom-code attack surface;
- locality/network requirements;
- measured language/capability scope;
- benchmark bundle;
- rollback predecessor.

Do not treat a model display name, provider marketing name or hosted endpoint version as immutable identity.

## 11. Voice SLO and benchmark dimensions

Exact numeric release thresholds belong to the future owning spec and hardware tier, but every qualified route must publish measured distributions for relevant dimensions.

At minimum:

- activation-to-listening;
- speech-start detection;
- first partial transcript;
- stable/final transcript;
- turn-commit;
- agent acknowledgement;
- first semantic response;
- first TTS audio;
- interruption-to-mute;
- interruption-to-agent-cancel;
- cancellation settlement;
- STT WER/CER/entity accuracy;
- VAD false accept/reject/boundary error;
- semantic endpoint false-stop/late-stop;
- TTS intelligibility and real-time factor;
- tool selection / argument accuracy for native-duplex paths;
- false self-echo command rate;
- CPU/RAM/GPU/NPU/battery/thermal;
- long-session stability;
- device/route recovery;
- privacy/egress violations;
- unauthorized Effect count.

No single aggregate score may hide a zero-tolerance authority/privacy failure.

## 12. Zero-tolerance voice release gates

A release claiming a voice capability must show zero observed cases in the qualified adversarial suite of:

- system audio / remote speaker / media / Golam TTS promoted directly to owner authority;
- stale partial transcript authorizing an Effect;
- generic/replayed confirmation approving the wrong Effect;
- hidden strict-local -> cloud fallback;
- sensitive TTS emitted after output route becomes disallowed;
- cancelled/stale audio session dispatching a consequential action;
- native-duplex tool call bypassing the Effect Gate;
- raw audio retained beyond declared policy;
- hidden always-listening activation;
- speaker similarity accepted as sole owner authentication;
- voice-only consequential control without a non-voice fallback.

## 13. Explicit non-goals

Golam voice does not require:

- biometric voice authentication;
- covert recording;
- hidden always-listening;
- stealth impersonation;
- one mandatory speech vendor;
- one mandatory speech architecture;
- cloud availability for core voice;
- accepting a model's own confidence as authority;
- treating diarization as identity;
- storing raw audio by default;
- making unverified medical/legal/financial decisions merely because the interface is voice.

## 14. Program closure checklist

Voice planning is not considered complete until the future owning lifecycle has explicit acceptance for all of these:

- [ ] activation / push-to-talk;
- [ ] optional wake word;
- [ ] source attribution;
- [ ] speaker/floor handling;
- [ ] capture health;
- [ ] conditioning/AEC;
- [ ] VAD/endpointing;
- [ ] streaming STT;
- [ ] Arabic/English/code-switch scope;
- [ ] entity and critical-number binding;
- [ ] corrections/disfluency handling;
- [ ] bounded semantic turn decisions;
- [ ] conversation steer/queue/branch/cancel;
- [ ] deterministic tool/Effect authority;
- [ ] bound high-risk confirmation;
- [ ] streaming TTS;
- [ ] output-route privacy revalidation;
- [ ] barge-in/emergency stop;
- [ ] self-echo rejection;
- [ ] raw-audio/transcript retention policy;
- [ ] cloud/local boundary;
- [ ] remote device transport;
- [ ] device/clock/jitter resilience;
- [ ] model/runtime integrity and rollback;
- [ ] personalization / lexicon / pronunciation;
- [ ] accessibility;
- [ ] observability without hidden content telemetry;
- [ ] benchmark/SLO evidence;
- [ ] long-session/crash/recovery;
- [ ] adversarial security fixtures;
- [ ] zero-tolerance release gates.

\`\`\`text
VOICE_PLAN_COMPLETENESS_MATRIX_PRESENT=YES
VOICE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
SPEECH_MODEL_ADMITTED=NO
CLOUD_SPEECH_ADMITTED=NO
ACTIVE_SPEC_006_WIDENED=NO
\`\`\`
