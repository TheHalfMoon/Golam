# Golam OpenJev Integration Plan — 2026-09-22

**Status:** PLANNING ONLY / FIRST-CLASS BOUNDED PROVIDER TARGET
**Source:** \`https://huggingface.co/AlexWortega/openjev\`
**Founder permission:** recorded in \`source-permission-attestation.md\`
**Authority:** no implementation/model admission is granted by this document

## 1. Decision

Golam SHOULD treat \`AlexWortega/openjev\` as a first-class, optional, replaceable local \`DecisionProvider\` target if it passes T175 artifact qualification and T204/T229 workload qualification.

OpenJev is valuable because many Golam decisions are bounded classification/ranking problems where a full generative LLM is unnecessary.

Current Hub metadata reviewed on 2026-09-22 identifies the supplied model as:

\`\`\`text
namespace: AlexWortega/openjev
task: text-classification
library: transformers
base model: Qwen/Qwen3.5-4B
tags: NLI / cross-encoder / reranker / text-classification
Hub license metadata: MIT
last observed update: 2026-09-21
\`\`\`

This metadata is discovery evidence, not immutable artifact identity. T175 must freeze exact model/config/tokenizer/safetensors file digests, runtime/backend identity, base-model/transitive rights and reproducible loading behavior before admission.

## 2. Golam role

OpenJev belongs in the **Decision Fabric**, not the Authority Plane.

Primary intended workloads:

### Voice / conversation

- \`COMMAND\` vs \`ASIDE\` vs \`QUESTION\` vs \`DICTATION\` vs \`CORRECTION\`;
- semantic turn-completeness hint: respond now vs keep listening;
- clarification-needed classification;
- conversational urgency/interrupt relevance;
- bounded candidate ranking for repository/project entities after deterministic extraction.

### Attention / proactive work

- AttentionItem triage;
- urgency/importance class proposal;
- duplicate/near-duplicate candidate ranking;
- whether a proactive item should be deferred to stronger reasoning.

### Capability / route selection

- rank already-admitted capability/provider candidates for a bounded task;
- classify whether the current request fits a capability family;
- recommend escalation to a stronger model/provider when the bounded classifier abstains.

### Retrieval / context

- rerank bounded retrieved candidates;
- classify relevance against an explicit criterion;
- decide whether additional retrieval may be useful, subject to deterministic budgets.

### Verification support

OpenJev may triage which verification rule/source should be consulted next. It MUST NOT itself turn evidence into verified truth or satisfy a VerificationObligation.

## 3. Explicit non-roles

\`\`\`text
OPENJEV != STT
OPENJEV != VAD
OPENJEV != TTS
OPENJEV != SPEAKER_IDENTITY
OPENJEV != OWNER_PRESENCE
OPENJEV != AUTHORITY
OPENJEV != POLICY_ENGINE
OPENJEV != EFFECT_GATE
OPENJEV != SOURCE_OF_TRUTH
OPENJEV != VERIFICATION_ORACLE
OPENJEV != COMPLETION_AUTHORITY
\`\`\`

A high OpenJev score can never:

- convert deny to allow;
- lower consequence class;
- approve egress;
- authorize spend;
- satisfy owner presence;
- approve an Effect;
- declare a source fact true;
- convert \`UNKNOWN\` to verified;
- emit \`VERIFIED_COMPLETE\`;
- silently replace deterministic routing or policy.

## 4. Adapter architecture

The integration target is a bounded \`OpenJevDecisionAdapter\` implementing T203.

\`\`\`text
Golam caller
  -> DecisionRequest
  -> applicability gate
  -> input budget / taint / locality checks
  -> OpenJevDecisionAdapter
  -> exact model/runtime
  -> raw scores
  -> calibrated workload profile
  -> cross-field consistency checks
  -> DecisionReceipt
  -> deterministic Golam policy / caller
\`\`\`

The adapter owns model translation only. It does not own routing policy, authorization, fallback policy, privacy policy or canonical state.

### DecisionRequest minimum binding

\`\`\`text
request_id
workload_id
state_digest
context_ref_or_bounded_text
question_schema_id
exact_option_ids
option_descriptions
abstain/no-match semantics
input_data_class
locality_requirement
latency_budget
resource_budget
calibration_profile_required
model/provider constraints
\`\`\`

### DecisionReceipt minimum binding

\`\`\`text
request_id
provider_id = openjev
provider_revision
model_artifact_ref
backend/runtime/device
input_digest
schema/prompt digest
exact option ids
raw option scores
calibrated probabilities if qualified
selected option or abstain
applicability/coverage
truncation/input-limit evidence
cross-field consistency result
latency
resource use
error/fallback state
\`\`\`

No free-form model prose is required for the protected contract.

## 5. Workload-specific qualification

OpenJev is admitted per workload profile, not globally.

At minimum qualify independent profiles for:

\`\`\`text
VOICE_UTTERANCE_CLASS
VOICE_TURN_COMPLETENESS
VOICE_CLARIFICATION_NEEDED
ATTENTION_TRIAGE
CAPABILITY_FIT
PROVIDER_CANDIDATE_RANK
RETRIEVAL_RERANK
VERIFICATION_NEXT_STEP_TRIAGE
\`\`\`

A profile that passes one workload does not inherit qualification for another.

## 6. Qualification metrics

T204/T229 must record, where applicable:

- accuracy / balanced accuracy;
- per-class precision/recall/F1;
- NLL/log loss;
- Brier score;
- ECE/reliability;
- selective risk / AURC;
- abstention coverage;
- false-safe / false-allow-like error rate for protected-adjacent tasks;
- option-order perturbation;
- wording/schema perturbation;
- adversarial/tainted-input sensitivity;
- truncation/context-limit sensitivity;
- cross-field contradiction rate;
- quantization/backend drift;
- warm/cold latency;
- CPU/GPU/NPU utilization;
- RAM/VRAM footprint;
- energy/thermal evidence where relevant;
- multilingual transfer limits;
- offline/network-denied startup behavior.

Voice-specific qualification must additionally measure:

- decision latency from partial and final transcript revisions;
- early wrong-commit rate;
- correction recovery;
- code-switch behavior;
- command/aside confusion;
- false "turn complete" rate;
- late-turn rate;
- robustness to ASR errors and transcript revisions.

## 7. Calibration and abstention

No generic threshold such as \`score > 0.8\` is canonical.

Each workload profile owns:

\`\`\`text
training/calibration/eval population
exact artifact/runtime
decision schema
calibration transform if any
accepted risk envelope
abstention policy
escalation path
expiry/requalification rule
\`\`\`

When applicability is weak, inputs are out-of-domain, truncation is material, contradiction exists, or calibration is stale, the provider must abstain/escalate.

\`\`\`text
HIGH_SCORE != CALIBRATED_PROBABILITY
CALIBRATED_PROBABILITY != AUTHORITY
OUT_OF_DOMAIN != BEST_EFFORT_ALLOW
ABSTAIN != FAILURE
\`\`\`

## 8. Escalation ladder

Preferred order:

\`\`\`text
exact deterministic rule / authoritative lookup
  -> OpenJev bounded DecisionProvider
  -> stronger qualified local DecisionProvider
  -> generative reasoning model
  -> independent verifier / authoritative source
  -> human review where required
\`\`\`

The ladder is workload-specific. OpenJev should reduce unnecessary large-model calls, not prevent escalation when uncertainty matters.

## 9. Voice integration

For voice, OpenJev receives text/context state after acoustic processing.

\`\`\`text
audio
-> VAD / endpoint evidence
-> streaming STT revisions
-> bounded transcript/context
-> OpenJev semantic decision
-> deterministic turn/conversation policy
\`\`\`

OpenJev cannot override:

- immediate stop/cancel controls;
- wake/source attribution;
- speaker identity rules;
- high-risk voice confirmation;
- Effect authorization;
- privacy/output-route policy.

Partial transcript decisions are provisional and revision-bound. A later transcript revision invalidates dependent provisional decisions according to the owning turn-state contract.

## 10. Failure and fallback

If OpenJev is:

- unavailable;
- corrupt;
- unqualified for the workload;
- too slow for the latency budget;
- out-of-domain;
- calibration-expired;
- contradictory;
- resource-incompatible;

Golam follows the owning escalation/fallback policy.

No fallback may silently widen:

- cloud/network use;
- provider/account;
- data retention;
- spend;
- authority;
- consequence class.

The system remains functional without OpenJev.

\`\`\`text
OPENJEV_UNAVAILABLE != GOLAM_UNAVAILABLE
OPENJEV_REMOVAL != AUTHORITY_SEMANTICS_CHANGE
\`\`\`

## 11. Artifact and supply-chain gate

Before any runtime admission, T175 must freeze:

- exact Hugging Face namespace;
- exact immutable revision;
- all relevant file SHA-256 digests;
- tokenizer/config/chat-template or classification-template identity;
- safetensors/config loader behavior;
- base-model identity and rights;
- model-card/license evidence;
- conversion/quantization provenance if used;
- exact Transformers/runtime/backend version;
- optional code/custom-code surface;
- offline startup/network behavior;
- supported hardware;
- rollback artifact.

\`\`\`text
MODEL_NAMESPACE != MODEL_ARTIFACT
MODEL_CARD_LICENSE != TRANSITIVE_RIGHTS_CLOSURE
MODEL_HASH != MODEL_TRUST
\`\`\`

## 12. Security tests

At minimum:

1. prompt/criterion injection inside untrusted transcript/context;
2. malicious option labels/descriptions;
3. option reorder;
4. impossible cross-field combination;
5. high-confidence out-of-domain input;
6. truncated input hiding a material qualifier;
7. negation/correction in speech;
8. stale partial transcript followed by correction;
9. adversarial media transcript pretending to be owner command;
10. attempt to use DecisionReceipt as approval;
11. model replacement with incompatible calibration profile;
12. corrupted/wrong artifact;
13. network-denied local operation;
14. resource exhaustion / timeout;
15. provider removal with deterministic fallback.

## 13. Success criteria

OpenJev becomes an admitted Golam provider for a workload only when:

\`\`\`text
FOUNDER_PERMISSION_RECORDED
AND EXACT_ARTIFACT_QUALIFIED_T175
AND RIGHTS_AND_OBLIGATIONS_RECONCILED
AND ADAPTER_CONFORMS_T203
AND WORKLOAD_PROFILE_PASSES_T204_T229
AND ABSTENTION_ESCALATION_PROVEN
AND AUTHORITY_CEILING_TESTED
AND STRICT_LOCAL_BEHAVIOR_PROVEN_WHERE_CLAIMED
AND REMOVAL_FALLBACK_PROVEN
AND ROLLBACK_PROVEN
\`\`\`

## 14. Integration disposition

\`\`\`text
OPENJEV_FIRST_CLASS_PROVIDER_TARGET=YES
OPENJEV_OPTIONAL_REPLACEABLE=YES
OPENJEV_MODEL_ADMITTED=NO
OPENJEV_RUNTIME_DEPENDENCY_ADMITTED=NO
OPENJEV_AUTHORITY_ROLE=NONE
OPENJEV_IMPLEMENTATION_AUTHORITY_GRANTED=NO
ACTIVE_SPEC_006_WIDENED=NO
\`\`\`
