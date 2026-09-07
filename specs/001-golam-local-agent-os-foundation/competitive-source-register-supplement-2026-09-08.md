# Competitive Source Register Supplement — 2026-09-08

**Status**: PROGRAM RESEARCH / SOURCE FOUNDRY INPUT — NO PRODUCT ADMISSION

**Extends**: `competitive-source-register-2026.md`

**Synthesis**: `source-synthesis-gap-closure-2026-09-08.md`

Exact pins below are research anchors for this planning revision. Every implementation use must reverify the live source state and perform exact component-level Source Foundry qualification.

## 1. Protocol and interoperability sources

### Model Context Protocol

```text
SOURCE=modelcontextprotocol/modelcontextprotocol
PIN=e76e9c572c6f2bfcb730357101acc90f2f802e02
DISPOSITION=STANDARD_PROFILE_AND_CONFORMANCE_REFERENCE
TARGET=T172
```

Planning value:

- explicit current MCP protocol/profile evolution;
- self-describing request semantics and protocol migration;
- Tasks/Extensions and current authorization behavior;
- deprecation-aware compatibility rather than frozen historical assumptions.

Non-adoption:

- MCP capability advertisement is not Golam authority;
- MCP server/tool output is not trusted instruction or verified truth;
- no protocol extension may bypass Golam's ordinary extension/security/effect gates.

### Agent2Agent Protocol

```text
SOURCE=a2aproject/A2A
PIN=98853be376c88df25e1704771cd3ea9ef8823a96
DISPOSITION=EXTERNAL_AGENT_INTEROP_REFERENCE_AND_ADAPTER_CANDIDATE
TARGET=T171
```

Planning value:

- interoperable external-agent discovery/task/message semantics;
- AgentCard/version-negotiation reference;
- clear product boundary between opaque external agents and governed internal Golam workers.

Non-adoption:

- AgentCard/skill advertisement is not capability authority;
- external agents cannot satisfy human approval;
- external-agent completion is unverified until Golam verification;
- Golam capability tokens are not forwarded to arbitrary remote agents.

## 2. Local compute and execution sources

### NVIDIA Personal AI Router

```text
SOURCE=NVIDIA/Personal-AI-Router
PIN=13b68115fa2c9c1d94f1ead1358f8d5a527cfecf
LICENSE_POSTURE_AT_REVIEWED_STATE=Apache-2.0
DISPOSITION=HIGH_VALUE_LOCAL_COMPUTE_MESH_SOURCE_CANDIDATE
TARGET=T170
```

Planning value:

- paired LAN compute nodes;
- mTLS identity and node readiness;
- model/backend/GPU-aware routing;
- local Ollama/LM Studio-class inference topology;
- cross-platform local-node product behavior.

Golam strengthening:

- `COMPUTE_NODE != AUTHORITY_HOST`;
- locality/privacy/authority compatibility precedes resource/cost/quality optimization;
- exact model/backend/node identity is receipt-bound;
- hidden model install/download is denied;
- no implication of pooled VRAM or distributed sharding when the underlying route is one-node inference.

### Kubernetes Agent Sandbox

```text
SOURCE=kubernetes-sigs/agent-sandbox
PIN=e5e2831295a789299722a9f4d922250167b1e879
DISPOSITION=EXECUTION_ISOLATION_REFERENCE
TARGET=T173
```

Planning value:

- explicit sandbox lifecycle;
- strong isolation options including gVisor/Kata-class backends;
- useful reference for agent-specific execution containment.

Non-adoption:

- Kubernetes is not a required Golam runtime;
- container identity is not evidence of strong containment;
- sandbox provider capability advertisement cannot raise Golam authority.

### GitHub Agentic Workflows

```text
SOURCE=github/gh-aw
PIN=b52dd75307b4233bd710ce0a0afd96766c2a48e3
LICENSE_POSTURE_AT_REVIEWED_STATE=MIT
DISPOSITION=HIGH_VALUE_AGENTIC_WORKFLOW_GUARDRAIL_AND_ISOLATION_REFERENCE
TARGET=T133/T151/T173/T179/T182
```

Planning value:

- source Markdown compiled into a hardened/reviewable execution workflow rather than interpreted as unrestricted runtime authority;
- deterministic CI remains separate from agentic reasoning;
- host-side network/API/MCP gateway patterns;
- explicit cost/permission controls;
- current stronger-isolation direction around hardware-virtualized microVM execution.

Golam strengthening:

- compiled workflow artifacts are still candidate/execution inputs, not protected authority;
- isolation class is admitted through `ExecutionIsolationProfile` evidence rather than a runtime label;
- Golam must remain usable outside GitHub and without mandatory cloud orchestration;
- no workflow engine can bypass Effect Gate or strict-local/egress policy.

## 3. Formal-assurance and workflow-verification sources

### Quint

```text
SOURCE=quint-co/quint
PIN=6fb2924e00707cef6dbc5e30db606d555c447123
DISPOSITION=FORMAL_ASSURANCE_TOOL_REFERENCE_AND_CANDIDATE
TARGET=T176
```

Planning value:

- executable specifications grounded in TLA-style state-machine reasoning;
- simulation/model checking;
- counterexample generation useful for deterministic regression fixtures.

Initial bounded targets:

- Effect/`UNKNOWN_OUTCOME` reconciliation;
- capability-lease generation/revocation;
- scheduler dedup/catch-up;
- Authority Host migration/recovery anti-fork;
- mobile approval replay protection;
- worker handoff/resume;
- extension activation/revocation.

Non-claim:

`FORMAL_MODEL_PASS != WHOLE_PRODUCT_FORMAL_VERIFICATION`.

### Guardians of the Agents implementation

```text
SOURCE=metareflection/guardians
PIN=59e52d9f1cbefdb344245304e260e647489a647b
LICENSE_POSTURE_AT_REVIEWED_STATE=MIT
DISPOSITION=HIGH_VALUE_PRE_EXECUTION_WORKFLOW_VERIFICATION_REFERENCE_AND_SELECTIVE_PORT_CANDIDATE
TARGET=T149/T176/T177/T179/T181
```

Planning value:

- structured workflow AST with symbolic references separating plan structure from attacker-controlled runtime data;
- static taint analysis from source to sink;
- security-automata checks over action sequences;
- theorem/constraint checks for preconditions and frame conditions;
- verifier-before-executor architecture with model adapters outside the verification core.

Golam adoption direction:

- reuse the **generate/verify/execute separation** for workflows that can be safely predeclared;
- selectively port compact deterministic verifier ideas/tests to Rust where Source Foundry shows a net benefit;
- integrate with existing Golam taint, ToolRequest, capability, Effect and VerificationReceipt structures instead of creating a second authority system;
- preserve runtime immediate revalidation because external state can change after static verification;
- adaptive/interactive work may require incremental verified plan fragments rather than one immutable whole-session plan.

Hard boundary:

```text
STATIC_PLAN_VERIFIED != CURRENT_RUNTIME_AUTHORIZED
STATIC_PLAN_VERIFIED != EXTERNAL_EFFECT_SUCCEEDED
```

## 4. Security and trust standards

### NIST Trustworthy and Responsible AI 800-5

```text
SOURCE=NIST_AI_AGENT_SECURITY_2026
DISPOSITION=SECURITY_TAXONOMY_REFERENCE
TARGET=T177/T156
```

Use as a taxonomy/reference input for system-level AI-agent threat analysis and control mapping. It is not product certification or release authority.

### OWASP Agentic Security Initiative

```text
SOURCE=OWASP_AGENTIC_SECURITY_INITIATIVE
DISPOSITION=ADVERSARIAL_CONTROL_REFERENCE
TARGET=T177/T179/T165
```

Ensure explicit treatment of:

- goal/instruction hijacking;
- tool misuse;
- identity/privilege abuse;
- memory/context poisoning;
- insecure inter-agent communication;
- cascading failures;
- rogue agents;
- extension/supply-chain/runtime compromise.

Scanner or checklist output remains supporting evidence; deterministic Golam invariants/tests are authoritative within the product.

### WebAuthn Level 3

```text
SOURCE=W3C_WEBAUTHN_LEVEL_3
REVIEWED_CR_SNAPSHOT=2026-05-26
DISPOSITION=OWNER_PRESENCE_STANDARD_REFERENCE
TARGET=T174/T168/T169
```

Use as one qualified option for high-assurance owner-presence evidence during selected approvals, pairing and recovery.

Hard boundary:

```text
USER_VERIFIED != OPERATION_AUTHORIZED
OWNER_PRESENCE_RECEIPT != CAPABILITY_LEASE
MANDATORY_BIOMETRIC_REQUIREMENT=NO
```

### OpenTelemetry GenAI semantic conventions

```text
SOURCE=OPENTELEMETRY_GENAI_SEMANTIC_CONVENTIONS
REVIEWED_CORE_SEMCONV_SERIES=1.44.x
GENAI_CONVENTIONS_LOCATION=DEDICATED_GENAI_SEMANTIC_CONVENTIONS_PROJECT
DISPOSITION=OBSERVABILITY_SCHEMA_REFERENCE
TARGET=T178/T139/T159
```

Use for interoperable local diagnostic/latency/token/cost projections when useful. Prompt/tool contents remain disabled by default; remote export requires separate explicit admission/consent. Reverify the dedicated GenAI semantic-conventions project/version in the owning implementation spec rather than assuming core semantic-convention version identity.

```text
TRACE != EVIDENCE
TRACE != AUTHORITY
SILENT_REMOTE_TELEMETRY=DENIED
```

## 5. Direct security-first competitor/reference

### Comis

```text
SOURCE=comisai/comis
PIN=641d4af18428ae35a6120304f871a8106f332b77
LICENSE_POSTURE_AT_REVIEWED_STATE=Apache-2.0
DISPOSITION=HIGH_VALUE_SECURITY_FIRST_AGENT_RUNTIME_COMPARATOR_AND_SELECTIVE_SOURCE_CANDIDATE
TARGET=T120/T122/T127/T132/T149/T152/T156/T177
```

Why it matters:

- persistent agents with governed learning;
- capability/origin/tool/credential/approval/budget/lease controls outside model output;
- source-linked correctable learned guidance;
- typed parallel orchestration and autonomy budgets;
- execution/security audit evidence and offline explanation surfaces;
- explicit threat model and documented limitations.

Golam differentiation target:

- stronger durable Effect/`UNKNOWN_OUTCOME` semantics for consequential external mutation;
- Rust protected authority kernel rather than TypeScript runtime as the primary authority root;
- stricter fail-closed isolation/egress defaults and explicit Source Foundry admission;
- independent VerificationReceipts before verified task completion;
- user-owned canonical state and Authority Host topology;
- model/learned guidance never self-expands authority.

Use Comis as a serious comparator and possible bounded source of tests/behavioral ideas. Do not blindly copy its default tool-policy, sandbox fallback, retention or learning-evidence choices.

## 6. Historical Golam planning corpus

The following PRs are valuable source material but are not parallel current program authority:

| PR | Exact historical head | Carry-forward value | Consolidation target |
| --- | --- | --- | --- |
| #6 `phone/channel access` | `7a93194ba1d1129358f0975d846a0e96b9001392` | native paired mobile, lower-assurance channels, push/voice/offline-intent boundaries | T169 / Spec 007 |
| #7 `memory/retrieval/learning/evals` | `a935f12adec031bfd47026f801bb3d1e20814c29` | memory promotion, retrieval authority, learning proposals, sandbox providers, eval governance | T120–T126/T151/T165/T173/T181 |
| #8 `product spine/Core Alpha` | `8f91db8cc10bff811631328104e48e9b664d9a15` | Task/Session/Run/Worker separation, TrustReceipt, Authority Host/Execution Nodes, anti-fork recovery | T167/T168/T149–T153/T182 |
| #22 `Agent Kernel roadmap v2` | `53fdf83d5dd850f3ec1e44d4f0b3381fb9db00c6` | six-plane architecture lens, evidence/capability concepts, reference systems | T166 + current overlay mapping |

Historical planning evidence remains provenance. After the current overlay is canonical, these branches must not be treated as independent successor authority merely because they remain open.

## 7. Existing PR #22 reference corpus retained for future requalification

The current overlay should retain research awareness of:

- `semantica-agi/semantica`;
- `multica-ai/multica`;
- `different-ai/openwork`;
- `openclaw/openclaw`;
- `koala73/worldmonitor`;
- `paperclipai/paperclip`;
- `PrimeIntellect-ai/prime-agent`;
- `deepseek-ai/deepseek-harness`;
- `zeroclaw-labs/zeroclaw`;
- `FoundationAgents/OpenManus`;
- `every-app/open-seo`.

Do not reuse their old historical pins automatically. Re-fetch exact source state and license/NOTICE/dependency/runtime behavior in the owning spec.

## 8. Reuse decision vocabulary

Every donor/component disposition in an owning spec should choose exactly one primary reuse strategy:

```text
COPY_AS_IS
SELECTIVE_COPY
PORT_TO_RUST
ADAPTER
REIMPLEMENT_BEHAVIOR
BENCHMARK_ONLY
REJECT
```

Required accompanying evidence:

- exact source commit/tree/version;
- selected file/component boundary;
- permission evidence and legal/license/NOTICE posture;
- dependency closure and native/FFI surfaces;
- network/telemetry/secrets behavior;
- authority delta analysis;
- modifications and provenance;
- independent Golam tests/security/benchmarks;
- removal/rollback plan where practical.

## 9. Program disposition

```text
ACTIVE_SPEC_006_PR_24_WIDENED=NO
SOURCE_REUSE_PERMISSION_REAFFIRMED=YES
SOURCE_REUSE_AUTOMATICALLY_ADMITTED=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_AUTHORITY_GRANTED=NO
WAIVER_TAKEN=NO
```
