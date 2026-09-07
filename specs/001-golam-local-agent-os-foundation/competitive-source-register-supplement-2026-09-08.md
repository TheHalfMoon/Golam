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

## 3. Formal-assurance source

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
DISPOSITION=OBSERVABILITY_SCHEMA_REFERENCE
TARGET=T178/T139/T159
```

Use for interoperable local diagnostic/latency/token/cost projections when useful. Prompt/tool contents remain disabled by default; remote export requires separate explicit admission/consent.

```text
TRACE != EVIDENCE
TRACE != AUTHORITY
SILENT_REMOTE_TELEMETRY=DENIED
```

## 5. Historical Golam planning corpus

The following PRs are valuable source material but are not parallel current program authority:

| PR | Exact historical head | Carry-forward value | Consolidation target |
| --- | --- | --- | --- |
| #6 `phone/channel access` | `7a93194ba1d1129358f0975d846a0e96b9001392` | native paired mobile, lower-assurance channels, push/voice/offline-intent boundaries | T169 / Spec 007 |
| #7 `memory/retrieval/learning/evals` | `a935f12adec031bfd47026f801bb3d1e20814c29` | memory promotion, retrieval authority, learning proposals, sandbox providers, eval governance | T120–T126/T151/T165/T173/T181 |
| #8 `product spine/Core Alpha` | `8f91db8cc10bff811631328104e48e9b664d9a15` | Task/Session/Run/Worker separation, TrustReceipt, Authority Host/Execution Nodes, anti-fork recovery | T167/T168/T149–T153/T182 |
| #22 `Agent Kernel roadmap v2` | `53fdf83d5dd850f3ec1e44d4f0b3381fb9db00c6` | six-plane architecture lens, evidence/capability concepts, reference systems | T166 + current overlay mapping |

Historical planning evidence remains provenance. After the current overlay is canonical, these branches must not be treated as independent successor authority merely because they remain open.

## 6. Existing PR #22 reference corpus retained for future requalification

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

## 7. Reuse decision vocabulary

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

## 8. Program disposition

```text
ACTIVE_SPEC_006_PR_24_WIDENED=NO
SOURCE_REUSE_PERMISSION_REAFFIRMED=YES
SOURCE_REUSE_AUTOMATICALLY_ADMITTED=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_AUTHORITY_GRANTED=NO
WAIVER_TAKEN=NO
```
