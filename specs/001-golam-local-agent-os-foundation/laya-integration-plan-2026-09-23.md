# Golam Laya Integration Plan — 2026-09-23

**Status:** PLANNING ONLY / FIRST-CLASS LOW-RESOURCE DECISION PROVIDER TARGET
**Source:** `https://huggingface.co/convaiinnovations/laya`
**Founder permission:** explicitly reaffirmed 2026-09-23
**Authority:** no model/runtime admission or implementation authority is granted by this document

## 1. Decision

Golam SHOULD treat `convaiinnovations/laya` as a first-class, optional, replaceable low-resource T203 `DecisionProvider` target if it passes T175 artifact qualification and T204/T235 workload qualification.

Current Hub metadata reviewed 2026-09-23 reports:

```text
namespace: convaiinnovations/laya
task: text-classification
library: transformers
parameters: ~421.3M
tags:
  system-one
  calibrated-decisions
  classification
  routing
  scoring
  guardrails
  moderation
  rlcd
Hub license metadata: Apache-2.0
last observed update: 2026-09-23
```

Hub metadata is discovery evidence, not immutable artifact identity. T175 must freeze exact files/digests/runtime and reconcile model/data/base-model/transitive rights before admission.

## 2. Why Laya is distinct from OpenJev

Laya and OpenJev are complementary candidates, not aliases.

```text
Laya
  -> small/fast System-1 classification, routing, scoring, risk/guardrail flagging

OpenJev
  -> larger NLI/cross-encoder/reranker semantic decision candidate
```

Golam must not globally declare one better. Qualification is workload + hardware + latency + privacy + calibration specific.

Potential route shape:

```text
very high-frequency / low-latency bounded triage
  -> Laya if qualified

deeper semantic relation / NLI / reranking
  -> OpenJev if qualified

ambiguous / out-of-domain / high-consequence
  -> stronger provider / exact rule / verifier / human
```

## 3. Intended workloads

Candidate workload profiles include:

- attention triage;
- capability-fit classification;
- provider/route candidate scoring;
- retrieval relevance/reranking where its scoring semantics fit;
- urgency/risk flags;
- moderation/guardrail escalation flags;
- voice utterance/turn semantic classification only if separately qualified;
- "more retrieval/reasoning needed?" bounded decisions;
- lightweight duplicate/near-duplicate candidate scoring.

Guardrail/moderation output can raise scrutiny or request review. It MUST NOT silently convert deny to allow or satisfy protected policy.

## 4. Adapter contract

Implement only through the canonical T203 contract:

```text
DecisionRequest
  -> applicability / input-budget / taint / locality checks
  -> LayaDecisionAdapter
  -> exact qualified model/runtime
  -> scores/classes
  -> workload calibration
  -> cross-field consistency
  -> DecisionReceipt
  -> deterministic Golam caller
```

The adapter owns translation to/from the model only. It does not own policy, authority, consequence classification, egress, approval, secret release, Effect dispatch, verification truth or fallback policy.

## 5. Cross-provider tournament

T235 must compare Laya against OpenJev and deterministic/generative baselines on matched held-out workloads.

Record:

- exact artifact/runtime/backend/device;
- accuracy / balanced accuracy / per-class metrics;
- log loss / Brier / ECE where applicable;
- AURC/selective risk and abstention coverage;
- OOD behavior;
- option/schema/wording perturbation;
- contradiction/impossible-state rate;
- tainted/adversarial input behavior;
- latency cold/warm;
- throughput;
- RAM/VRAM;
- quantization/backend drift;
- energy/thermal evidence where meaningful;
- strict-local network-denied behavior;
- workload-specific multilingual limits.

A smaller model wins a route only when the measured objective supports it.

## 6. Safety ceiling

```text
LAYA != AUTHORITY
LAYA != POLICY_ENGINE
LAYA != EFFECT_GATE
LAYA != OWNER_PRESENCE
LAYA != VERIFICATION_ORACLE
LAYA_GUARDRAIL_SCORE != POLICY_DECISION
LAYA_MODERATION_SCORE != EFFECT_PERMISSION
LAYA_HIGH_SCORE != ALLOW
LAYA_UNAVAILABLE != GOLAM_UNAVAILABLE
```

For protected-adjacent workloads, model output may only flag, rank, request review, recommend escalation, or abstain. It cannot lower deterministic safeguards.

## 7. Calibration and applicability

Every admitted workload has its own:

```text
workload_id
population / held-out fixtures
artifact/runtime
input schema
output schema
calibration profile
abstention policy
accepted risk envelope
latency/resource envelope
expiry/requalification rule
escalation path
```

No universal score threshold is canonical.

## 8. Model artifact gate

T175 must record at minimum:

- exact namespace and immutable revision;
- relevant file SHA-256 digests;
- config/tokenizer/processor identity;
- safetensors/model file identity;
- architecture/custom-code requirements;
- runtime/Transformers version;
- conversion/quantization provenance;
- license/NOTICE and transitive rights;
- network behavior/offline startup;
- supported hardware;
- rollback artifact.

## 9. Removal and fallback

Golam must remain correct with Laya disabled.

```text
LAYA_REMOVED
-> exact deterministic rule if applicable
-> another admitted DecisionProvider
-> stronger reasoning route
-> human/authoritative source where required
```

Fallback cannot silently widen privacy, cloud use, provider/account, spend or authority.

## 10. Disposition

```text
LAYA_FIRST_CLASS_LOW_RESOURCE_PROVIDER_TARGET=YES
LAYA_OPTIONAL_REPLACEABLE=YES
LAYA_MODEL_ADMITTED=NO
LAYA_RUNTIME_DEPENDENCY_ADMITTED=NO
LAYA_AUTHORITY_ROLE=NONE
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
```
