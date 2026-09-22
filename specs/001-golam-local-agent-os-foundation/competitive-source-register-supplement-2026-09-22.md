# Competitive Source Register Supplement — 2026-09-22

**Status**: PROGRAM RESEARCH / SOURCE FOUNDRY INPUT — NO PRODUCT ADMISSION

**Extends**:
- `competitive-source-register-2026.md`
- `competitive-source-register-supplement-2026-09-08.md`
- `competitive-source-register-supplement-2026-09-13.md`
- `source-permission-attestation.md`

**Founder direction**: the founder states permission to use, copy, modify, port and selectively reuse source material supplied for Golam and source material in the founder-owned GitHub portfolio. This is a permission input, not an admission decision.

```text
PERMISSION_TO_COPY != TECHNICAL_ADMISSION
SOURCE_VISIBILITY != SOURCE_ADMISSION
OWNER_REPOSITORY_ACCESS != SOURCE_ADMISSION
CATALOG_ENTRY != CAPABILITY_AUTHORITY
MODEL_PROBABILITY != AUTHORITY
```

All implementation reuse remains subject to the exact-component Source Foundry record, the T197 TCB budget, transitive rights/NOTICE closure, security qualification, dependency/runtime closure, exact version identity and the canonical Effect/Verification contracts.

## 1. External sources reverified for this pass

| Source | Exact reviewed revision/state | Primary value | Golam disposition |
| --- | --- | --- | --- |
| `google/ax` | `d8ed0fe38bceb7842d3c47817d53d16ccdfcb601` | Declarative Task/Workspace/Gateway/Model resources, reconciliation, readiness, suspend/resume, sandboxed agent workloads | HIGH_VALUE_EXECUTION_ORCHESTRATION_REFERENCE |
| `superdesigndev/treg` | `9ba0d6906d495cb2842a008c959983a4a536f256` | Capability catalog, search-by-job, credential-brokered relay, MCP catalog surface, explicit provider/cost selection | HIGH_VALUE_CAPABILITY_EXCHANGE_SOURCE_CANDIDATE |
| `TheoLeeCJ/SemIf` | `1f2dea3e25379f9dfc98cb83c324f00ab5deda37` | Local typed semantic decisions from option logits, shared-state scoring, calibration, reproducible evidence | HIGH_VALUE_DECISION_PROVIDER_CANDIDATE |
| `Mapika/decider` | `7557fe058e0b31ffcf754b7c5bb31a452454ce89` | Choice/Score/Noul typed decisions, calibrated probabilities, fast local serving, browser/vision experiments | HIGH_VALUE_DECISION_PROVIDER_CANDIDATE |
| `bespokelabsai/nimble` | `f136b3f75721fda4ea961f73993cc50b08488835` | Open Jev-style training/serving recipe and typed structured prediction | HIGH_VALUE_DECISION_TRAINING_AND_EVAL_REFERENCE |
| `bespokelabs/Bespoke-Nimble-9B` | Hub state reviewed 2026-09-22; Apache-2.0 tag; Qwen3.5-9B adapter | Candidate local typed-decision artifact | MODEL_ARTIFACT_CANDIDATE_ONLY |
| `aayushch/laya` | `5970a114241ee09cec09d571acf9ba52d27ae612` | Proactive action cards, cross-app attention, briefing, coherence, approval-first action UX, local-first desktop patterns | HIGH_VALUE_EXPERIENCE_AND_HARNESS_REFERENCE |
| `tinyfish-io/agentql` | `418ba8ad1c69dfac134a6833369a01dfba5a24a7` | Resilient semantic web selectors and structured extraction | HIGH_VALUE_BROWSER_SEMANTICS_REFERENCE |
| `tinyfish-io/tinyfish-mcp-server` | `5df1bbb8ff48bc282c59bbc00e8c8418cf791b62` | Hosted automation/MCP streaming and remote run behavior | NETWORK_ADAPTER_REFERENCE_ONLY |
| `tinyfish-io/tinyfish-cookbook` | `8615317f6db58ae776dd53817ac30668c1db5ef8` | Search/Fetch/Agent/Browser escalation recipes and research workflows | BEHAVIORAL_REFERENCE |
| `wonderwhy-er/DesktopCommanderMCP` | `75048278f4866f0d8bde26f6f9aa3b8d39dca870` | Files/process/PTY/document tooling, session UX, bounded output, local audit, remote MCP patterns | HIGH_VALUE_TOOL_PROVIDER_REFERENCE |

The supplied Bespoke Hugging Face discussion is not treated as architecture authority. Its material program value is the Nimble model/repository and the reproducible typed-decision method, not the discussion thread itself.

## 2. Google AX: adopt the reconciliation model, not the cluster requirement

AX contributes four especially strong concepts:

- `Task`: isolated unit with compute/resource constraints;
- `Workspace`: reusable materialized repositories/tools/skills;
- `Gateway`: explicit network boundary;
- `Model`: named model configuration;
- readiness conditions plus suspend/resume/checkpoint behavior;
- a controller that reconciles desired state toward observed runtime state.

Golam should not make Kubernetes, Redis, Agent Substrate or a cluster control plane mandatory for local use.

Recommended translation:

```text
Golam TaskContract
  -> ExecutionEnvelope
     -> WorkspaceBindings
     -> ExecutionBackend / IsolationProfile
     -> EgressPolicyRef
     -> ExecutionProfileRef
     -> CapabilitySetRef
     -> ReadinessConditions
     -> CheckpointPolicy
```

The envelope is declarative execution input. It does not grant authority.

```text
WORKLOAD_MANIFEST != CAPABILITY_GRANT
WORKSPACE_BINDING != FILESYSTEM_AUTHORITY
GATEWAY_DECLARATION != EGRESS_AUTHORIZATION
READINESS_CONDITION != VERIFIED_COMPLETION
```

AX reconciliation is valuable for workers and long-running tasks because it separates desired state from runtime truth. Golam should preserve this distinction locally first and allow cluster/remote backends later through T151/T173.

## 3. Treg: capability exchange without a second authority system

Treg demonstrates that an agent should ask for a capability rather than carry thousands of provider-specific tool schemas.

Retain:

- capability/job-oriented search;
- stable exact endpoint/tool identity;
- provider alternatives with price and behavior metadata;
- server-side credential injection;
- multiple credential bindings where an upstream requires them;
- compact MCP discovery plus exact call instead of thousands of tools;
- structured authorization remediation;
- faithful relay behavior;
- explicit cost/balance information;
- provider/account selection visible to the caller;
- no silent provider failover for ordinary exact calls.

Golam strengthening:

- the catalog is a projection of admitted/available capability offers, never authority;
- credentials use Golam secret handles and the existing connector/account broker;
- every call is mapped to T199 Operation/Effect semantics before dispatch;
- arbitrary or insufficiently modeled tools remain `OPEN_WORLD`, potentially consequential and non-idempotent;
- T179 owns taint/egress; T152 owns routing constraints; T165/T180/T197 own source/extension admission;
- tool relay cannot create its own Effect ledger, authorization state, identity root or truth about successful completion;
- exact cost, locality, destination and account are known before a route is selected when possible.

A future `CapabilityOffer` should carry at least:

```text
capability_id
provider_id
provider_revision
operation_classes
read_write_open_world_class
locality
account_binding
credential_binding_refs
egress_destinations
data_classes
retention_expectation_if_known
price_model
estimated_cost
latency/reliability evidence
idempotency/retry semantics
source_admission_ref
conformance_ref
availability/freshness
```

### Treg license and permission boundary

The reviewed repository license is Apache-2.0 plus additional terms that restrict using the public-licensed software as a hosted, managed or embedded third-party commercial service without explicit prior written authorization.

The founder separately states that Golam has permission to use/copy the Treg source. That statement is recorded as founder-attested permission input. Before any Treg code is copied into a distributed/commercial Golam component, the exact Source Foundry record must bind documentary permission scope sufficient for the intended use and preserve any continuing notice/attribution obligations.

```text
FOUNDER_ATTESTED_PERMISSION != VERIFIED_SCOPE_OF_UPSTREAM_COMMERCIAL_GRANT
PUBLIC_REPO_ACCESS != COMMERCIAL_EMBEDDING_RIGHT
```

This is not a reason to avoid Treg; it is a reason to keep rights proof exact.

## 4. SemIf + Decider + Nimble: a provider-neutral System-1 decision fabric

These sources reveal a product opportunity larger than choosing one model.

Many agent decisions are bounded:

- classify attention;
- choose one route/provider;
- check whether evidence supports a criterion;
- score urgency/risk;
- decide whether to escalate;
- select a workflow branch;
- estimate whether more retrieval is needed.

A full text-generating LLM is often unnecessary.

Golam should define one replaceable `DecisionProvider` contract with candidates such as SemIf, Decider, Nimble-derived artifacts and future qualified local providers. The founder-supplied `AlexWortega/openjev` is the first explicitly planned first-class bounded provider target under T229, while remaining optional and replaceable.

A `DecisionRequest` should bind:

```text
request_id
state_digest
bounded_state_or_context_ref
question_schema
exact_option_ids
criterion_descriptions
explicit_no_match_or_abstain_semantics
data_class
locality_requirement
latency_budget
resource_budget
calibration_profile_requirement
model/provider constraints
```

A `DecisionReceipt` should bind:

```text
provider_id
provider_revision
model_artifact_ref
backend/device/runtime
prompt_or_schema_digest
option_probabilities
selected_option
abstention_state
calibration_profile
applicability/coverage evidence
latency
resource/cost evidence
input_truncation_or_limit evidence
```

SemIf contributes reproducible prompt hashes, local backends and workload calibration. Decider contributes richer typed decision families and high-throughput serving. Nimble contributes an independently reproducible training/data recipe and strong warning that option probabilities are conditional rather than guarantees of correctness.

The decision fabric is advisory to deterministic policy and verification.

```text
DECISION_PROBABILITY != AUTHORITY
DECISION_PROVIDER != POLICY_ENGINE
CALIBRATED_PROBABILITY != VERIFIED_FACT
DECISION_PROVIDER_OUTPUT != OWNER_APPROVAL
MODEL_ROUTE_RECOMMENDATION != EFFECT_PERMISSION
```

Security-sensitive authorization remains deterministic and kernel-owned. A decision provider may raise attention or request review; it must never lower a trusted consequence class or transform deny into allow.

## 5. Laya: make Golam proactive without creating a second event system

Laya's strongest contribution is product behavior, not its n8n/Python/Chroma implementation stack.

Retain:

- proactive action cards;
- pre-research before the user opens an item;
- daily briefings;
- cross-platform entity coherence;
- temporal rollups;
- explicit preview-before-send;
- worker session launched from a card;
- local/cloud model selection;
- budget visibility;
- editable learned grouping/routing rules.

Golam strengthening:

```text
External Event / Observation
  -> canonical connector ingest
  -> deterministic normalization
  -> DecisionProvider triage where useful
  -> ContextBundle / research evidence
  -> AttentionItem
  -> ActionProposal
  -> authenticated user input
  -> canonical Effect flow
  -> VerificationReceipt
```

There is no second notification/event database with separate authority semantics. `AttentionItem` and `ActionProposal` are projections over canonical Task/Event/Knowledge/Effect/Evidence state.

Corrections to grouping, classification or routing produce immutable `RoutineCandidate` or rule-candidate revisions. They do not mutate trusted active behavior in place.

```text
ATTENTION_CARD != TASK_TRUTH
ACTION_PROPOSAL != AUTHORIZED_EFFECT
ASSOCIATION_CONFIDENCE != CANONICAL_RELATION
BRIEFING_SUMMARY != SOURCE_OF_TRUTH
LEARNED_RULE != ACTIVE_AUTHORITY
```

## 6. TinyFish + Desktop Commander: providers under the same fabric

TinyFish/AgentQL remains valuable for semantic browser target discovery, especially where deterministic selectors alone are brittle.

The route remains:

```text
strong deterministic browser/application identity when available
-> semantic candidate discovery
-> exact target binding
-> canonical Effect Gate
-> dispatch
-> observation/reconciliation
```

Natural-language selector confidence does not establish target authority.

Desktop Commander contributes strong UX and provider patterns for:

- process sessions;
- bounded/tail output;
- filesystem tooling;
- PDF/DOCX/XLSX operations;
- local call history;
- remote machine access.

Golam must keep these behind typed capabilities, isolation, secret/egress controls and the Effect ledger. Command blocklists and directory allowlists are guardrails, not containment.

## 7. Founder-owned GitHub portfolio review

### Canonical owner-enumeration record

This is the single population record consumed by T211 and later owner-portfolio planning for this review revision.

```text
ENUMERATION_DATE = 2026-09-22
ENUMERATION_METHOD = authenticated GitHub repositories-by-affiliation(owner), first page sized above observed population
INCLUSION = every repository returned for the founder as owner
PRIVATE_REPOSITORIES_INCLUDED_IN_COUNT = YES
PRIVATE_REPOSITORY_NAMES_PUBLISHED_HERE = NO
ARCHIVED_REPOSITORIES = included if returned; not excluded by policy
FORKS = included if returned; fork status is not an exclusion rule
EMPTY_REPOSITORIES = included if returned
NAME_OR_SIZE_FILTER = NONE
OWNER_REPOSITORIES = 36
PUBLIC_REPOSITORIES = 29
PRIVATE_REPOSITORIES = 7
```

The current connector payload exposes archived state and no returned repository was archived at this revalidation. Fork status is not relied upon by this enumeration record; forks are not excluded if the owner-affiliation enumeration returns them. Counts are therefore defined by the authenticated returned population plus repository visibility, not by a hand-maintained source table.

Any later count change requires refreshing this record and then updating consumers that cite it; consumers MUST NOT independently infer population size from the subset of repositories promoted as Golam sources.

Private repository names/content are deliberately not copied into this public Golam planning artifact. They were available to the discovery pass but remain confidentiality-preserving candidate context. Any private component selected later requires an exact Source Foundry record and a separate decision about what provenance may be published.

### Primary internal architecture/donor families

| Source | Reviewed current pin | Golam value |
| --- | --- | --- |
| `TheHalfMoon/kernux` | `18dd95f7fa07a3c2b9fbd1b9583a044b8384c020` | Capability kernel, real browser/computer/process/runtime execution, grants/evidence, local/remote runtime patterns |
| `TheHalfMoon/Ascout` | `20818ea52568c3955654236994de28a00f47fb26` | Unified assurance, reality verification, owner-source inventory, evidence/finding/coverage discipline |
| `TheHalfMoon/Sentrdel` | `f5747319a50831ef7cee983d253c0ca5503c9a64` | Security evidence, invariants, coverage honesty, reachability and retest lifecycle |
| `TheHalfMoon/wepld` | `666e62d7d9e040505baca277a474d734afcc07a0` | Agentic engineering control plane, semantic/browser/computer research, terminal/process ownership |
| `TheHalfMoon/Morize` | `62fc04d01d398da93b4dacf0ab2f33e3f1440462` | Governed memory OS, bi-temporal truth, ContextBundle, memory firewall, retrieval hierarchy |
| `TheHalfMoon/Winds` | `3e637403accadc0f446d596f4411c5c9da371e89` | Exact Git/worktree isolation, typed execution/evidence, recovery |
| `TheHalfMoon/SpecGrain` | `5de7d6499bb0a9e3a191fc0934399cf099d1980a` | Bounded dependency-ordered delivery |
| `TheHalfMoon/Diffcipline` | `1e6d14f77b95bb132b42276f10d67f1018ab5bb6` | Challenge/prove-before-done discipline |
| `TheHalfMoon/Delethos` | `5b8f5f28239bbd0b2a366824fd14c1b6c01058b8` | Verified delegation and independent review/isolation |
| `TheHalfMoon/Tarif` | `b1b3cecc7c2de32a4ecdba02a6bb752ae7a050c5` | Action/resource/parameter authority model reference |
| `TheHalfMoon/Ecra` | `0e2ff8c687c93e6f158da6984a7a6915339b5f3f` | Browser/workspace/trusted execution UX reference |

### Model, benchmark and evidence families

| Source | Reviewed current pin | Golam value |
| --- | --- | --- |
| `TheHalfMoon/MESC` | `ab29ca613cd1ec19460809a2708d2c315d191b9f` | Validator-grounded outputs and provider qualification |
| `TheHalfMoon/MedScale` | `ff2e677294b1ea8704bbeefa605d129c1a99e8f2` | Local privacy, source registers, benchmark/evidence planning |
| `TheHalfMoon/MSTR` | `e87328872232471fa0e1eb05d74223bc0aeaafd3` | Local/offline model/runtime qualification and efficiency |
| `TheHalfMoon/commandMed` | `ce5dc8f4dda49270e5bb4bebc936ce7514a9b9e9` | Abstention, evidence/safety/resource constrained evaluation |
| `TheHalfMoon/commandF` | `18819c7fdaee618f4aa86c6c3279b1acb70ec9f2` | SHA-bound package identity, deterministic diff/check/SARIF patterns |

### Human control, multimodal and product-harness references

| Source | Reviewed current pin | Golam value |
| --- | --- | --- |
| `TheHalfMoon/Himsat` | `cc1c1c38bf07fe6c28d6ce919d5d773f41d23d62` | Evidence-linked conversation memory, voice/capture privacy and temporal provenance |
| `TheHalfMoon/Wispral` | `edacdf7504302cc91ff7138bc6ac2d391e4df1f4` | Voice control, interruption, visible permissions |
| `TheHalfMoon/Inercative` | `fd3fd806b8941a03fc96ddfcc395f2da57622a5a` | Product-building harness, typed generation/test/policy/evidence loops |
| `TheHalfMoon/Golam-research` | `a9f633e09d49a85829b8236331b9e21f7e612634` | High-value reconstruction/protocol/product evidence with special provenance handling |

Domain repositories remain useful for patterns, datasets, testing methods or domain-specific examples, but they are not promoted into Golam runtime dependencies merely because they are owner-accessible.



### Additional owner-portfolio dispositions from the deep dive

The re-enumeration was followed by a repository-content deep dive, not only a name/README scan. Public sources that expose a measured Golam gap are promoted below; private repositories remain `PRIVATE_CONSIDERED_UNDISCLOSED` in this public artifact and require a private exact-component Source Foundry record before any reuse.

| Source | Reviewed current pin | Disposition | Measured Golam value |
| --- | --- | --- | --- |
| `TheHalfMoon/Kodac` | `406b335277f2df1e3dedf24cdb45847dff919d44` | `HIGH_VALUE_ARCHITECTURE_REFERENCE / BOUNDED_ADAPTER_CANDIDATE` | Exact workload identity across approval wait; requested-vs-observed confinement; backend capability is not confinement proof; one-shot approval binding |
| `TheHalfMoon/Pluma` | `f38a3e743a32342f1c25e7a7e606eff6a8c8edb2` | `HIGH_VALUE_ARCHITECTURE_REFERENCE / PORT_OR_ADAPT_DOMAIN_CONTRACTS` | DisclosureReceipt -> bounded AgentProposal, explicit review checkpoints, valid-time vs recorded-time decision resolution, agent-independent resume/continuity |
| `TheHalfMoon/Signthos` | `f945f12fd1a2b600c2c61493162e3654b4d5b50c` | `HIGH_VALUE_ARCHITECTURE_REFERENCE` | Orthogonal verification dimensions, versioned EvidenceBundle, immutable artifact revisions and independent verification |
| `TheHalfMoon/Ecra` | `0e2ff8c687c93e6f158da6984a7a6915339b5f3f` | `HIGH_VALUE_ARCHITECTURE_REFERENCE` | Typed Skill IR -> candidate compiler -> deterministic replay -> divergence/repair -> downstream invalidation |
| `TheHalfMoon/Inercative` | `fd3fd806b8941a03fc96ddfcc395f2da57622a5a` | `HIGH_VALUE_ARCHITECTURE_REFERENCE` | Failure Ledger, finite evidence-driven repair, Context Continuation Policy, completeness manifests and model/tool/budget routing |
| `TheHalfMoon/Wispral` | `edacdf7504302cc91ff7138bc6ac2d391e4df1f4` | `HIGH_VALUE_ARCHITECTURE_REFERENCE` | Typed voice lifecycle and interruption/cancellation that must not wait for final transcription |

The private portfolio deep dive exposed additional high-value patterns in evidence fidelity/absence semantics and semantic-faithfulness verification. Those patterns informed T213 and T215, but this public repository intentionally does not publish private repository names, paths, content or pins. Exact private-source identity is retained only when/where an authorized private Source Foundry record is created.

Public sources reviewed with **no current measured Golam gap** are not promoted merely because they are available:

| Source | Current disposition | Reason |
| --- | --- | --- |
| `TheHalfMoon/acarat` | `NO_CURRENT_MEASURED_GAP` | Explainable uncertainty/projection lessons are already covered by Decision/Evidence contracts; no Golam runtime dependency justified |
| `TheHalfMoon/Balott` | `DOMAIN_REFERENCE_ONLY` | Game/ranking/voice domain does not expose a unique Golam platform gap |
| `TheHalfMoon/Qdrat` | `DOMAIN_REFERENCE_ONLY` | Application-event automation is useful product evidence but must not replace Golam durable Effect/scheduler semantics |
| `TheHalfMoon/Zyara` | `DOMAIN_REFERENCE_ONLY` | Adapter/voice/provider lessons overlap stronger Golam/Wispral/connector contracts |
| `TheHalfMoon/Trcel` | `NO_CURRENT_MEASURED_GAP` | Empty repository at review time |



### Voice/audio and personal-agent follow-up sources

| Source | Reviewed state | Disposition | Golam value / boundary |
| --- | --- | --- | --- |
| `AlexWortega/openjev` (Hugging Face) | `https://huggingface.co/AlexWortega/openjev`; Hub state reviewed 2026-09-22; updated 2026-09-21; task `text-classification`; Transformers; Qwen3.5-4B base; NLI/cross-encoder/reranker tags; Hub license metadata MIT; exact immutable artifact digests still require T175 | `FIRST_CLASS_BOUNDED_DECISION_PROVIDER_TARGET / MODEL_ARTIFACT_CANDIDATE` | Optional replaceable local T203 provider target under T229 for voice semantic decisions, attention triage, capability-fit/provider ranking and retrieval reranking; **not** STT, VAD, TTS, policy, authority, speaker identity or verification truth |
| `TheHalfMoon/Golam-research` | `a9f633e09d49a85829b8236331b9e21f7e612634` | `HIGH_VALUE_IMPLEMENTATION_EVIDENCE / BOUNDED_PORT_CANDIDATE` | Grok Bot 0.18 recovered push-to-talk voice controller, transcript cards, permission/auto-review UX, coordinator/session fences; whole-clip STT is a baseline to surpass, not the target full-duplex design |
| `TheHalfMoon/Wispral` | `edacdf7504302cc91ff7138bc6ac2d391e4df1f4` | `PRIMARY_VOICE_ARCHITECTURE_REFERENCE` | Event-driven voice control plane, independent cancellation, provenance, push-to-talk baseline, streaming STT evidence and measurable interruption/latency semantics |
| `TheHalfMoon/Himsat` | `cc1c1c38bf07fe6c28d6ce919d5d773f41d23d62` | `PRIMARY_AUDIO_RUNTIME_AND_BENCHMARK_REFERENCE` | Capture health, conditioning/VAD, Voice Runtime Router, local speech challenger matrix, Arabic/English/code-switch, device/clock/long-session evidence |
| Meta Muse public product/security documentation | official public material reviewed 2026-09-22: `https://about.fb.com/news/2026/09/introducing-muse-personal-ai-agent/`, `https://research.meta.ai/blog/security-and-safety-for-ai-agents-our-approach-with-muse`, `https://introducing.muse.ai/` | `BEHAVIOR_SECURITY_REFERENCE_ONLY` | Long-running non-turn-locked conversation, side chats, background goals/work, proactive sparse updates, activity transparency, artifacts, deterministic approvals; isolated runtime + independent permission/egress authority + credential surrogation + tainted egress. No proprietary source-code reuse inferred. |

| Meta Muse Voice Transcribe | official Meta AI Research material reviewed 2026-09-22 | `REMOTE_SPEECH_BENCHMARK / OPTIONAL_PROVIDER_REFERENCE` | Streaming ASR, endpointing, 20+ speaker diarization, multilingual/code-switch and context-bias behavior; no strict-local equivalence inferred and no hidden cloud fallback permitted |
| `meta-models/Muse-Glimmer-30B` | official Meta/Hugging Face model state reviewed 2026-09-22; Apache-2.0 open weights; exact files/digests/runtime still require T175 | `LOCAL_AGENT_MODEL_ARTIFACT_CANDIDATE` | Always-on local agent/tool-use/multimodal candidate; **not a speech engine** |
| NemotronLabs VoiceChat paper/model family | public research state reviewed 2026-09-22 | `NATIVE_DUPLEX_BENCHMARK_CANDIDATE` | Full-duplex speech-to-speech, incremental transcript, streaming TTS and structured tool calls; benchmark interruption, tool-selection and argument correctness separately; no automatic code/model admission |

#### OpenJev placement rule

OpenJev must not be mislabeled as a voice model. This review refers specifically to the founder-supplied `AlexWortega/openjev` namespace; similarly named Hub repositories are separate source/model objects and may carry different licenses or artifacts. Exact namespace/revision/file digests are therefore mandatory. It may consume transcript/context state to answer bounded typed semantic questions. Acoustic endpointing, speaker/source attribution, microphone control, VAD, STT and TTS remain separate replaceable capabilities.

```text
OPENJEV != STT
OPENJEV != VAD
OPENJEV != TTS
OPENJEV_DECISION != EFFECT_AUTHORIZATION
MODEL_DISPLAY_NAME != MODEL_ARTIFACT_IDENTITY
MUSE_GLIMMER != SPEECH_ENGINE
REMOTE_STREAMING_STT != STRICT_LOCAL_ROUTE
NATIVE_DUPLEX_TOOL_CALL != EFFECT_AUTHORIZATION
```

#### OpenJev first-class provider admission path

OpenJev has moved from general candidate/reference status to a bounded first-class provider **target**, not to admitted runtime status.

The required path is:

```text
founder permission
-> exact HF namespace/revision/file digest freeze
-> transitive/base-model rights closure
-> T175 artifact/runtime qualification
-> T203 adapter conformance
-> T204/T229 workload-specific calibration/applicability
-> adversarial + strict-local + replacement tests
-> workload-specific ADMITTED or REJECTED result
```

Admission is per workload. A passing voice-turn profile does not automatically admit attention triage, routing, reranking or verification-support usage.

```text
OPENJEV_PROVIDER_TARGET=YES
OPENJEV_GLOBAL_ADMISSION=NO
OPENJEV_WORKLOAD_ADMISSION_IS_INDEPENDENT=YES
OPENJEV_DISABLED_PATH_REQUIRED=YES
```

#### Muse placement rule

Muse is not an admitted dependency or donor implementation. Its public architecture is useful because it independently reinforces Golam's existing direction: a model/runtime should not own permission, credentials or egress. Golam maps those lessons onto its existing Authority/Effect/Secret/Egress contracts rather than reproducing Meta's cloud VM topology.



### AutoClaw / Z.AI follow-up sources

The founder explicitly states that Z.AI granted permission to copy/use available Z.AI AutoClaw-related source code. This permission is recorded as founder-attested input and does not bypass exact-component Source Foundry, transitive rights/NOTICE closure, T165 security admission or T197 TCB budgeting.

| Source | Reviewed state | Disposition | Golam value / boundary |
| --- | --- | --- | --- |
| AutoClaw official product (`https://autoclaw.z.ai/`) | official product/blog/changelog reviewed 2026-09-22; public desktop source repo not identified | `HIGH_VALUE_PRODUCT_BEHAVIOR_REFERENCE / AUTHORIZED_SOURCE_IF_SEPARATELY_SUPPLIED` | one-message execution UX, Cluster Mode, multi-agent isolation, Hermes self-evolution, IM control, professional deliverables, model switching; product behavior does not become Golam authority |
| `openclaw/openclaw` | `e9df70639592b85e6c6f15e609b5a4a1c53b1b18`; MIT + third-party notices | `HIGH_VALUE_UPSTREAM_SOURCE_CANDIDATE` | channels, automation/cron/hooks/tasks, sessions/memory/context, multi-agent operator lifecycle, plugins/skills, sandbox/browser/security and completeness rubrics; independent upstream rights, not covered by Z.AI permission assertion |
| `zai-org/GLM-skills` | `2ecd31c37e75671a4767342ba3a68a84c8f1b848`; Apache-2.0 | `HIGH_VALUE_SKILL_PACK_SOURCE_CANDIDATE` | official skill packaging/prerequisite patterns and bounded OCR/multimodal/document/PRD workflows; use through T233/Source Foundry |
| `zai-org/Open-AutoGLM` | `86f55382982fb054e8fc98ca80609dff8a2cdc3c`; Apache-2.0 | `BOUNDED_MOBILE_ACTION_PROVIDER_CANDIDATE` | Android/HarmonyOS/iOS visual-control adapters, screenshots, normalized coordinates, takeover/confirmation patterns; model->action loop must be re-mapped through Golam T199/Effect Gate |
| `zai-org/ZCode` | `872ad960de7ec172591f7e1952f7849229f94521`; Apache-2.0 | `HIGH_VALUE_HARNESS_AND_TOOLING_REFERENCE / BOUNDED_SOURCE_CANDIDATE` | coding harness, browser skill, artifact/UI patterns, architecture governance and feature-boundary tooling |
| `zai-org/Synapse` | `651d39d92ff08beff0868f991c36a27c20191726`; Apache-2.0 | `HIGH_VALUE_COLLABORATION_AND_LEARNING_REFERENCE / BOUNDED_SOURCE_CANDIDATE` | shared conversations/memory/plugins plus OpenClaw self-improvement hook/promotion patterns; active Golam behavior remains governed |

#### AutoClaw placement rule

AutoClaw should influence Golam at the Harness/Experience/Capability boundary, not replace the protected Authority/Evidence spine.

```text
AUTOCLAW_CLUSTER_MODE != NEW_TASK_AUTHORITY
AUTOCLAW_MULTI_AGENT != SHARED_AMBIENT_MEMORY
HERMES_LEARNING != ACTIVE_RULE
AUTOCLAW_CHANNEL != GOLAM_PRINCIPAL
OPEN_AUTOGLM_ACTION != AUTHORIZED_EFFECT
SKILL_PACK != ADMITTED_CAPABILITY
```

The reviewed adoption plan is `autoclaw-zai-adoption-2026-09-22.md`.

## 8. Portfolio synthesis: do not create a collage

The combined source universe points to five reusable fabrics, not dozens of embedded applications:

```text
1. Authority + Evidence Core        -> Golam canonical kernel
2. Execution Fabric                -> Kernux + AX + Desktop Commander + TinyFish
3. Capability Exchange             -> Treg + Golam connectors/extensions/secrets
4. Decision Fabric                 -> OpenJev first-class bounded target + SemIf + Decider + Nimble + internal model qualification
5. Attention / Knowledge Fabric    -> Laya + Morize + Golam Experience/Context
6. Voice / Conversational Presence -> Wispral + Himsat + Golam-research + replaceable speech engines + T203 decision providers
7. Adaptive Delivery / Channels / Skills -> AutoClaw behavior + OpenClaw + Z.AI GLM-skills + Synapse/ZCode, under T163/T169/T180/T230–T234
```

Ascout/Winds/SpecGrain/Diffcipline/MESC/commandF provide cross-cutting proof, source identity, bounded delivery and independent verification. Kodac/Pluma/Signthos/Ecra/Inercative/Wispral add bounded contracts for workload identity, disclosure-bound proposals, review checkpoints, multidimensional proof, skill replay/repair, failure continuity and immediate human interruption without becoming separate authority systems.

The key design constraint is that these fabrics consume the same canonical contracts. None becomes a second authority root.

## 9. Source intake rule

A source becomes implementation material only when all are true:

```text
MEASURED_GAP
AND BOUNDED_COMPONENT_SELECTED
AND EXACT_SOURCE_PINNED
AND PERMISSION_RIGHTS_OBLIGATIONS_RECONCILED
AND DEPENDENCY_RUNTIME_CLOSURE_RECORDED
AND TCB_DELTA_ACCEPTABLE
AND AUTHORITY_EFFECT_MAPPING_DEFINED
AND BENCHMARK_OR_PARITY_NEED_DEFINED
AND FAILURE_REMOVAL_PATH_DEFINED
```

Otherwise the source remains a reference.

## 10. Program disposition

```text
NEW_SOURCES_REVIEWED_2026_09_22=YES
OWNER_PORTFOLIO_REENUMERATED_2026_09_22=YES
OWNER_PORTFOLIO_COUNT_CORRECTED_TO_36_2026_09_22=YES
FOUNDER_SOURCE_PERMISSION_REAFFIRMED_2026_09_22=YES
ACTIVE_SPEC_006_PR_24_WIDENED=NO
CURRENT_POINTER_CHANGED=NO
CONSTITUTION_CHANGED=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_PRODUCT_IMPLEMENTATION_STARTED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
PORTFOLIO_DEEP_DIVE_COMPLETED_2026_09_22=YES
VOICE_AUDIO_DEEP_DIVE_COMPLETED_2026_09_22=YES
OPENJEV_CLASSIFIED_AS_DECISION_NOT_SPEECH=YES
OPENJEV_FIRST_CLASS_PROVIDER_TARGET=YES
OPENJEV_RUNTIME_ADMITTED=NO
MUSE_REFERENCE_ONLY_NO_CODE_ADMISSION=YES
AUTOCLAW_ZAI_DEEP_DIVE_COMPLETED_2026_09_22=YES
ZAI_AVAILABLE_SOURCE_PERMISSION_ATTESTED=YES
AUTOCLAW_PUBLIC_DESKTOP_SOURCE_REPO_IDENTIFIED=NO
PRIVATE_SOURCE_NAMES_PUBLISHED=NO
WAIVER_TAKEN=NO
```
