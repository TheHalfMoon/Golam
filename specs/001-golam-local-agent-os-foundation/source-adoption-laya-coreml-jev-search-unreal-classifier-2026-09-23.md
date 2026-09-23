# Golam Source Adoption Review — Laya CoreML, Jev Search, Unreal Agent and classifier.dev — 2026-09-23

**Status:** PROGRAM RESEARCH / PLANNING ONLY
**Authority:** No implementation, dependency, runtime, model or source admission. No Spec 006 scope expansion.

## 1. Reviewed source states

| Source | Exact reviewed state | Public rights posture observed | Golam disposition |
| --- | --- | --- | --- |
| `mizorewww/laya-coreml` | `4619e0483f07adf39068532e85b42ec2347edb83` | Apache-2.0 + NOTICE | HIGH_VALUE_APPLE_SILICON_DECISION_RUNTIME_DONOR_CANDIDATE |
| `caio0452/jev_search` | `ea073f6db48f5bff73ae4b9f2240d2d302fb9dc1` | no public license file observed at reviewed pin; founder separately asserts permission | BEHAVIORAL_REFERENCE / SELECTIVE_COPY_CANDIDATE_AFTER_EXACT_RIGHTS |
| `unreallabsai/unreal-agent` | `df8b0ba560da17fd705d941cbeb75eff86c74a1e` | MIT | HIGH_VALUE_HARNESS_DONOR_CANDIDATE |
| `mrmps/classifier-dev` / `classifier.dev` | `8f2bb2b84a0d51ad1c9ed3436b64155908354f75` | MIT repository; hosted service has separate terms | HIGH_VALUE_BATCH_DECISION / EVAL / PROVIDER_REFERENCE |

The founder explicitly states permission to copy/use the supplied source. That permission is recorded in `source-permission-attestation.md` but does not replace exact component, dependency, model-artifact or service-term admission.

## 2. Laya CoreML — strengthen T235, do not create another Decision Fabric

The repository is an independent Core ML port of Laya focused on typed decisions on Apple Silicon.

Useful properties:

- choice / score / noul typed outputs rather than generated JSON;
- Core ML execution and Apple Neural Engine-specific variants;
- local/offline prediction after artifact acquisition, including `local_files_only=True`;
- explicit short-context vs general-context artifact separation;
- measured latency and system-energy evidence;
- conversion-fidelity fixtures and published raw benchmark artifacts;
- tokenizer/config/model-card/checksum packaging;
- explicit calibration-temperature clamping after detecting an unsafe over-sharpened fitted bucket;
- honest distinction between conversion fidelity and task accuracy.

Golam adoption direction:

1. keep `LayaDecisionAdapter` provider-neutral;
2. treat Core ML / ANE as one backend implementation under T175/T235/T159;
3. qualify exact model + tokenizer + Core ML package + conversion tool/version + compute-unit policy;
4. measure cold initialization separately from steady-state latency;
5. measure energy/resource behavior on the exact target hardware;
6. preserve calibration provenance and make any runtime clamp/repair visible in the `DecisionReceipt`;
7. benchmark short-context ANE variants separately from long-context variants;
8. require strict-local artifact availability before selecting a no-network route;
9. retain a removable non-CoreML backend so Apple-specific runtime availability is never architectural authority.

Hard boundaries:

```text
COREML_BACKEND != DECISION_AUTHORITY
ANE_EXECUTION != POLICY_AUTHORITY
CONVERSION_FIDELITY != WORKLOAD_ACCURACY
CALIBRATION_CLAMP != CALIBRATION_PROOF
SHORT_CONTEXT_BENCHMARK != LONG_CONTEXT_QUALIFICATION
MODEL_PACKAGE_CHECKSUM != MODEL_TRUST
LOCAL_AFTER_DOWNLOAD != STRICT_LOCAL_ARTIFACT_ACQUISITION
```

No new task is required. T235 should explicitly include this backend candidate.

## 3. Jev Search — adopt the narrowing shape, not its production posture

The reviewed project is intentionally small and its README explicitly says it is fully AI-generated and should not be used in production.

Useful behavior:

- deterministic file discovery/filtering;
- lexical/keyword prioritization before semantic calls;
- text chunking;
- two-phase high-priority then low-priority search;
- progressive early results;
- boolean criteria AST with AND / OR / parentheses;
- parallel decision requests;
- file-level aggregation from chunk scores.

Problems that Golam must not copy blindly:

- OpenRouter is mandatory in the supplied implementation;
- an API key may be passed on the command line;
- a fixed global threshold is used;
- no workload calibration/abstention contract is demonstrated;
- file content can leave the device without Golam's T179/T201 policy;
- lexical priority can create false negatives if used as a hard exclusion;
- no canonical ContextBundle/RetrievalReceipt integration;
- no provenance-bound omission accounting.

Golam should keep lexical ranking as a cheap prioritizer, never as an unreported truth filter.

## 4. classifier.dev — batch classification and context-economy reference

The reviewed repository implements zero-shot classification over HTTP backed primarily by Jev and exposes REST, MCP, CLI, SDK and agent-skill surfaces.

High-value patterns:

- large batch classification rather than one reasoning call per item;
- stable per-item result identity;
- calibrated confidence and explicit escalation of uncertain items;
- multi-label classification;
- measured eval harnesses;
- actual answering-provider/fallback visibility;
- request-cost accounting;
- privacy-oriented analytics that avoid storing source text;
- health/alerting for silent provider fallback;
- candidate selection from real accessibility-tree elements in the computer-use action picker;
- context-economy framing: filter large result sets before spending expensive context on every item.

Golam strengthening:

- the hosted `classifier.dev` service is an explicit remote provider and therefore never strict-local;
- no hosted service may receive text without T179/T201 data/egress authorization;
- batch packing must preserve per-item source identity and provider identity;
- confidence thresholds are workload-calibrated under T204, never copied as universal constants;
- `smart` escalation patterns become a provider-neutral cascade, not a hard-coded cloud chain;
- privacy claims from a remote provider are routing metadata/evidence, not a substitute for local policy;
- accessibility-tree candidate ranking may recommend the next semantic action but cannot authorize the action.

```text
CLASSIFIER_CONFIDENCE != EFFECT_AUTHORIZATION
REMOTE_BATCH_PROVIDER != STRICT_LOCAL_CAPABILITY
PROVIDER_PRIVACY_CLAIM != EGRESS_AUTHORIZATION
ACCESSIBILITY_CANDIDATE_SCORE != TARGET_AUTHORITY
BATCH_SUCCESS != PER_ITEM_VERIFICATION
```

## 5. Unreal Agent — high-value harness donor with strict ownership boundaries

The reviewed harness contains several patterns that complement rather than replace Golam's stronger Effect spine.

Useful patterns:

- caller-supplied globally unique input IDs stable across redelivery;
- session-scoped input deduplication;
- append-only persisted session history with explicit forks;
- coordinator persists accepted inputs before subsequent model work;
- tool translators validate synchronously and perform no I/O;
- tool calls translate into serializable Operations;
- tool-call translation status and produced Operations are recorded atomically;
- operation execution state is separate from translation state;
- context building returns an explicit record of omitted/truncated/compacted material;
- serializable/versioned session and Operation formats;
- unsupported session versions fail explicitly on resume;
- swappable operation manager.

Golam must not copy these authority assumptions wholesale:

- Unreal Agent's session store must not become Golam canonical Task/Effect authority;
- its volatile Inbox is not sufficient for cross-restart redelivery idempotency;
- its Operation type maps into T199 and the canonical Effect path rather than creating a parallel executor truth;
- provider auth inside an LLM adapter must still obey Golam SecretBinding / egress rules;
- session forks cannot clone capability/approval/effect authority.

The most valuable gap exposed here is **inbound input redelivery idempotency before the Effect layer**, plus an explicit pure tool-translation commit boundary.

## 6. New contract — T246 Batch Semantic Narrowing and Decision Cascade

T246 should connect T192/T195/T203/T204/T229/T235/T179/T201.

Canonical batch request should include:

```text
BatchDecisionRequest
  request_id
  workload_id
  criteria_graph
  item[]
    item_id
    source_ref
    source_revision
    content_digest
    taint/data_class
    bounded payload/features
  recall_policy
  calibration_profile_ref
  locality/privacy constraints
  budget
```

Per-item output:

```text
BatchDecisionReceipt
  request_id
  item_id
  provider/artifact/backend identity
  label/score/probabilities
  applicability
  abstention/escalation
  truncation/packing evidence
  latency/resource/cost
  egress receipt if remote
```

Recommended narrowing ladder:

```text
exact IDs / metadata / deterministic filters
-> lexical / FTS / keyword priority
-> local batch DecisionProvider
-> optional stronger local provider
-> optional explicitly-authorized remote batch provider
-> expensive context / generative reasoning only for survivors
```

Recall-sensitive workloads default to **keep on uncertainty**, not silent drop.

Applications include:

- repository/context search;
- research-result filtering;
- logs/events;
- knowledge ingestion;
- notification/attention triage;
- candidate capability/provider selection;
- accessibility-tree action candidates;
- bulk document/entity triage.

## 7. New contract — T247 Inbound Input Envelope and Pure Tool Translation Commit

T247 should connect T167/T169/T183/T199/T205/T214/T241.

Canonical input acceptance should distinguish **input duplication** from **Effect duplication**.

```text
InputEnvelope
  input_id
  source_kind
  source_principal/account/channel
  provider_event_id if available
  provider_delivery_attempt if available
  source_timestamp
  observed_at
  payload_digest
  taint/data_class
  ordering/sequence evidence if available
  schema_version
```

Required behavior:

- if a provider supplies a stable event ID, persist it before the input can influence protected work;
- same stable event ID + same payload digest is a redelivery, not a new user intent;
- same stable event ID + different digest is an integrity/security anomaly;
- absence of a provider-stable event ID is represented honestly; a locally generated receipt ID does not prove cross-delivery deduplication;
- channel reconnect, retry and webhook redelivery cannot create duplicate canonical intent silently;
- input dedup survives process restart where the source protocol claims replay/redelivery semantics;
- dedup state has bounded retention appropriate to the source protocol and cannot erase legitimate repeated user intents with different IDs.

Tool translation boundary:

```text
ModelToolCallCandidate
-> deterministic/schema validation
-> pure ToolTranslationReceipt + OperationProposal[]
-> atomic durable commit
-> canonical policy/capability/Effect evaluation
-> dispatch
```

The translator performs no network/file/secret/provider I/O. If the translation/operation commit fails, nothing dispatches.

This does not replace the Effect Gate. It ensures the model-facing tool-call interpretation is durable and inspectable before the existing authority path begins protected execution.

## 8. Source reuse posture

### Laya CoreML

Preferred order:

```text
ADAPTER / SELECTIVE_COPY
-> exact CoreML packaging/runtime components only if measured
-> PORT_TO_RUST where privileged integration would otherwise expand Python TCB
```

### Jev Search

Preferred order:

```text
REIMPLEMENT_BEHAVIOR
-> SELECTIVE_COPY only after exact permission/right record
```

Do not copy its cloud/API-key/threshold defaults as policy.

### Unreal Agent

Preferred order:

```text
SELECTIVE_COPY / PORT_PATTERNS
```

Priority components are input identity/dedup, pure translation, atomic operation recording, versioned serialization tests and context omission receipts. Do not transplant the session store as Golam authority.

### classifier.dev

Preferred order:

```text
REIMPLEMENT_BEHAVIOR / SELECTIVE_COPY
```

Priority components are batch packing, eval/cascade methodology, actual-provider/fallback reporting, privacy-safe metrics patterns and candidate-ranking fixtures. Cloudflare-specific deployment is optional and non-canonical.

## 9. Required acceptance evidence

Before any owning implementation admits these patterns:

- T235 demonstrates Laya CoreML artifact/runtime fidelity, workload calibration, cold/warm latency, energy/resource measurements and removable fallback on qualified Apple hardware;
- T246 shows higher context efficiency without unacceptable recall loss against a no-prefilter baseline;
- T246 proves remote batch providers never run under strict-local and every remote item has explicit egress/provider evidence;
- T246 reports omitted/rejected items and keeps uncertain items in recall-sensitive profiles;
- accessibility candidate scoring never bypasses route applicability, target identity or Effect authorization;
- T247 survives duplicate/redelivered channel/webhook inputs across restart without duplicating canonical intent or Effects;
- T247 detects same provider-event ID with a different payload digest;
- T247 tool translation is deterministic/IO-free and atomically persisted before any Operation can dispatch;
- removing any one donor/provider leaves canonical state readable and the relevant fallback path explicit.

## 10. Planning disposition

```text
LAYA_COREML_SOURCE_REVIEWED=YES
JEV_SEARCH_SOURCE_REVIEWED=YES
UNREAL_AGENT_SOURCE_REVIEWED=YES
CLASSIFIER_DEV_SOURCE_REVIEWED=YES

Laya_CoreML -> T235/T175/T159
Jev_Search -> T246 behavior/reference
classifier.dev -> T246 donor/reference/provider candidate
Unreal_Agent -> T247 donor/reference

NEW_TASKS=T246,T247
NEW_PARALLEL_EFFECT_LEDGER=NO
NEW_PARALLEL_SESSION_AUTHORITY=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_MODEL_ADMITTED=NO
ACTIVE_SPEC_006_PR_24_WIDENED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
```
