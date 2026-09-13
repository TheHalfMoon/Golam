# Competitive Source Register Supplement — 2026-09-13

**Status**: PROGRAM RESEARCH / SOURCE FOUNDRY INPUT — NO PRODUCT ADMISSION

**Live Golam base reviewed**: `TheHalfMoon/Golam@13a379ac478a3abaff7ed1da3db14ff9c1ac2188`

**Extends**:

- `competitive-source-register-2026.md`
- `competitive-source-register-supplement-2026-09-08.md`
- `source-permission-attestation.md`

**Founder permission**: explicitly reaffirmed on 2026-09-13 for the sources supplied in this review and the source universe already recorded in Golam. This is a rights input, not automatic technical admission.

```text
PERMISSION_TO_COPY != TECHNICAL_ADMISSION
PERMISSION_TO_COPY != DONOR_TRUST_MODEL_ADOPTED
SOURCE_PIN != FUTURE_SOURCE_STATE
SOURCE_FEATURE != GOLAM_REQUIREMENT
```

Every exact component still requires Source Foundry qualification, dependency/runtime closure, license/NOTICE reconciliation, network/telemetry/secrets analysis, authority-delta analysis, Golam-native tests, and an explicit reuse strategy.

## 1. CopilotKit — agent-facing experience and protocol projection

```text
SOURCE=CopilotKit/CopilotKit
PIN=abaec72f366d2346100b57ab951c2e0f70491fc9
PRIMARY_VALUE=AGENT_UI_EVENT_MODEL_GENERATIVE_UI_HITL_SHARED_STATE_MCP_APP_HOSTING
DISPOSITION=HIGH_VALUE_UX_PROTOCOL_REFERENCE_AND_SELECTIVE_COPY_CANDIDATE
PREFERRED_REUSE=REIMPLEMENT_BEHAVIOR_OR_SELECTIVE_COPY_OUTSIDE_TCB
TARGET=T183/T185/T186
```

Retain:

- event-driven agent/UI lifecycle rather than chat-only rendering;
- framework-independent interaction contracts with thin surface adapters;
- generative/declarative tool-result UI patterns;
- human-in-the-loop interaction ergonomics;
- shared agent/UI state projections;
- sandboxed MCP Apps rendering patterns and protocol single-sourcing.

Golam strengthening:

- UI events are sanitized projections of canonical Golam state, not authority-bearing state;
- a frontend tool call never authorizes an effect;
- an approval interaction is only an authenticated approval input to the Authority Kernel and cannot itself mint a capability;
- arbitrary generated UI cannot receive protected credentials, native handles, raw capability tokens, or authority SQLite access;
- renderer/MCP-App content is untrusted and tainted by default.

```text
AGENT_UI_EVENT != CANONICAL_EVENT
HITL_WIDGET_CONFIRMATION != AUTHORIZED_EFFECT
GENERATIVE_UI != TRUSTED_CODE
```

## 2. TinyFish organization — semantic web interaction and explicit network adapters

The organization was surveyed as an ecosystem, not treated as one undifferentiated dependency. Material sources for Golam are recorded below; unrelated repositories and forks are not admitted by association.

### AgentQL

```text
SOURCE=tinyfish-io/agentql
PIN=f561f147edf14293e5ef393e4026991d1d49f8a1
PRIMARY_VALUE=NATURAL_LANGUAGE_SEMANTIC_SELECTORS_PLAYWRIGHT_RESILIENT_WEB_EXTRACTION
DISPOSITION=HIGH_VALUE_BROWSER_SEMANTICS_REFERENCE_AND_ADAPTER_CANDIDATE
PREFERRED_REUSE=ADAPTER_OR_REIMPLEMENT_BEHAVIOR
TARGET=T115/T117/T179/T187
```

Retain:

- semantic element/data selection resilient to page structure changes;
- structured result schemas;
- existing-browser/session operation patterns;
- Playwright interoperability;
- debugger/evidence ergonomics around selector resolution.

Do not adopt:

- stealth or anti-bot evasion as a Golam product objective;
- selector/model confidence as target identity authority;
- hidden browser/session use that bypasses visible-channel, account, egress, or effect policy.

A natural-language selector may propose a candidate target. A stronger deterministic binding must identify the exact DOM/protocol/application target before a consequential effect whenever available.

### TinyFish MCP server

```text
SOURCE=tinyfish-io/tinyfish-mcp-server
PIN=a838419a6d57163a08e452acdc62030ca9294978
PRIMARY_VALUE=MCP_PROXY_STREAMING_REMOTE_AUTOMATION_ERROR_UNKNOWN_RUN_HANDLING
DISPOSITION=NETWORK_ADAPTER_REFERENCE_ONLY_UNTIL_EXPLICIT_NON_STRICT_SPEC
PREFERRED_REUSE=REIMPLEMENT_BEHAVIOR
TARGET=T119/T172/T179/T188
```

Useful patterns:

- explicit upstream identity;
- SSE progress forwarding;
- clear mid-stream uncertainty warning instead of blind retry;
- origin validation for local HTTP exposure;
- local health/diagnostic surface.

Golam must not inherit its local trust assumption. The reviewed server documents that any local process able to reach the loopback port can use the server-held key. Golam Constitution requires authenticated local clients and explicitly rejects localhost as authentication.

```text
LOOPBACK_REACHABILITY != AUTHENTICATION
REMOTE_MCP_TOOL != STRICT_LOCAL_CAPABILITY
REMOTE_AUTOMATION_PROGRESS != EFFECT_SUCCESS
```

### TinyFish Cookbook

```text
SOURCE=tinyfish-io/tinyfish-cookbook
PIN=8615317f6db58ae776dd53817ac30668c1db5ef8
PRIMARY_VALUE=SEARCH_FETCH_AGENT_BROWSER_ROUTING_RECIPES_TOKEN_EFFICIENT_RESEARCH
DISPOSITION=BEHAVIORAL_REFERENCE_AND_RESEARCH_BENCHMARK_CORPUS
PREFERRED_REUSE=BENCHMARK_ONLY_OR_REIMPLEMENT_BEHAVIOR
TARGET=T136/T179/T188/T189
```

Retain the Search -> Fetch -> Agent/Browser escalation idea as a cost/strength ladder, but Golam must route by privacy, authority, egress and verification compatibility before speed/cost. Hosted Search/Fetch/Agent/Browser are explicit network capabilities only.

The documented stealth/proxy behavior is not a Golam target. Credential Vault ideas may inform the connector broker, but provider-held vault semantics cannot replace Golam's local secret broker or capability model.

## 3. Desktop Commander MCP — local tool ergonomics, not its trust model

```text
SOURCE=wonderwhy-er/DesktopCommanderMCP
PIN=74bca3d642dec0973e55db641dcaffd49a70ca40
PRIMARY_VALUE=FILES_PROCESS_SESSIONS_STREAMING_PAGINATION_DOCUMENT_TOOLING_PREVIEW_AUDIT_ERGONOMICS
DISPOSITION=HIGH_VALUE_TOOL_UX_AND_ADAPTER_SOURCE_CANDIDATE
PREFERRED_REUSE=SELECTIVE_COPY_OR_PORT_TO_RUST_BEHIND_GOLAM_GATES
TARGET=T137/T151/T165/T183/T190
```

Retain:

- long-running process sessions with bounded output retrieval;
- negative/tail reads and pagination to protect context budgets;
- rich file/document preview/edit UX;
- local tool-call history with bounded retention;
- explicit process lifecycle operations;
- format-specific operations where they are semantically stronger than generic pixel control.

Reject as Golam security posture:

- trusting the connected AI client as the security root;
- treating directory allowlists or command blocklists as containment;
- direct arbitrary shell execution outside qualified execution isolation and Golam Effect Gate.

Desktop Commander itself documents that its directory/blocklist controls are guardrails, not a sandbox. Golam should use this as a comparative proof point for why privileged execution belongs behind typed capabilities, isolation evidence, taint/egress policy and durable effects.

## 4. Perplexity organization — research/search interfaces and eval discipline

The organization contains product source, infrastructure, forks and unrelated libraries. Only material agent/search components are registered here.

### Perplexity MCP server

```text
SOURCE=perplexityai/modelcontextprotocol
PIN=c73c8561bbc2d9eb666334a53c311b50f4f4cf76
PRIMARY_VALUE=SEARCH_ASK_RESEARCH_REASON_TOOLS_STREAMING_REMOTE_MCP
DISPOSITION=EXPLICIT_NETWORK_RESEARCH_ADAPTER_REFERENCE
PREFERRED_REUSE=ADAPTER_OR_REIMPLEMENT_BEHAVIOR
TARGET=T119/T136/T172/T179/T188
```

Hosted Perplexity search/research is never part of the strict-local baseline. If admitted later, it must be an explicit connector capability with destination/account/budget/data-class binding, secret brokering, egress receipts and citation provenance.

### Search evaluations

```text
SOURCE=perplexityai/search_evals
PIN=cab4c5df36e5660dc73f5580352eb52962587f01
PRIMARY_VALUE=REPRODUCIBLE_DEEP_RESEARCH_HARNESSES_COST_ACCOUNTING_RESUME_TRACES_BENCHMARKS
DISPOSITION=HIGH_VALUE_EVALUATION_REFERENCE_AND_SELECTIVE_SOURCE_CANDIDATE
PREFERRED_REUSE=SELECTIVE_COPY_OR_REIMPLEMENT_BEHAVIOR
TARGET=T136/T143/T145/T158/T189
```

Retain:

- provider-normalized evaluation harnesses;
- resumable exact-config runs;
- task-level attempt traces;
- separate cost accounting;
- dataset-contract fingerprints;
- failed-as-zero and failed-excluded reporting;
- fixed benchmark suites and transparent grader traces.

Golam strengthening: final release claims must use Golam-owned verification and benchmark-integrity rules; paid provider graders are supporting evidence, not release authority.

### Perplexity CLI

```text
SOURCE=perplexityai/perplexity-cli
PIN=LIVE_COMPONENT_STATE_TO_REVERIFY_AT_ADMISSION
PRIMARY_VALUE=AGENT_FRIENDLY_JSON_SEARCH_SNIPPET_BUDGETING_PARTIAL_FAILURE_DISCLOSURE_UPDATE_INTEGRITY
DISPOSITION=CLI_UX_AND_RESEARCH_TOOL_REFERENCE
PREFERRED_REUSE=REIMPLEMENT_BEHAVIOR
TARGET=T136/T159/T183/T188
```

Retain machine-first JSON, bounded snippet/token budgets, per-result partial failure disclosure, exact build identity and checksum-verified updater patterns. Provider-specific networking remains an explicit connector.

Repositories in the organization that are unrelated forks/libraries are not product inputs merely because they are under `perplexityai`.

## 5. OpenRAG — knowledge workspace and retrieval product behavior

```text
SOURCE=langflow-ai/openrag
PIN=ffccefbd1b3f8eaebaac315d3cab27a94007cbfa
PRIMARY_VALUE=DOCUMENT_INGESTION_RERANKING_AGENTIC_RAG_KNOWLEDGE_UX_MCP
DISPOSITION=HIGH_VALUE_KNOWLEDGE_PRODUCT_REFERENCE_OPTIONAL_SIDECAR_CANDIDATE
PREFERRED_REUSE=REIMPLEMENT_BEHAVIOR_OR_BOUNDED_ADAPTER
TARGET=T136/T164/T191/T192
```

Retain:

- explicit ingest -> process -> index -> retrieve lifecycle;
- document collections/knowledge filtering;
- re-ranking and retrieval observability;
- user-facing knowledge management;
- MCP access as an interoperability surface.

Do not make FastAPI, Next.js, OpenSearch, Langflow or Docling mandatory dependencies of the Golam trusted/local baseline. For ordinary personal/local corpora, Golam should prefer a smaller rebuildable derivative index over canonical user-owned source artifacts. A heavier sidecar may be admitted only for measured scale/quality need.

```text
RETRIEVAL_INDEX != CANONICAL_MEMORY
RAG_RESULT != VERIFIED_FACT
DOCUMENT_TEXT != TRUSTED_INSTRUCTION
```

## 6. VoiceStudio — local voice engine/product reference

```text
SOURCE=debpalash/VoiceStudio
PIN=eaf8bb953855cab3b687d547b3835f86fa38b308
PRIMARY_VALUE=LOCAL_TTS_ASR_ENGINE_CATALOG_DEVICE_ROUTING_DICTATION_DIAGNOSTICS_REMOTE_WORKERS
LICENSE_POSTURE_AT_REVIEWED_STATE=AGPL-3.0_APPLICATION_UPSTREAM_MODEL_TERMS_SEPARATE
DISPOSITION=HIGH_VALUE_VOICE_PRODUCT_REFERENCE_AND_COMPONENT_CANDIDATE
PREFERRED_REUSE=ADAPTER_SELECTIVE_COPY_OR_REIMPLEMENT_BEHAVIOR_OUTSIDE_TCB
TARGET=T140/T170/T175/T193
```

Retain:

- local-first multi-engine STT/TTS routing;
- explicit model catalogue/install/load/unload state;
- hardware-aware CUDA/MPS/ROCm/CPU routing;
- dictation and interruption UX;
- diagnostics/self-check and scrubbed support bundles;
- remote worker concepts only when integrated with Golam identity/compute governance.

Golam strengthening:

- microphone access requires an explicit short-lived capability/lease and visible active indicator;
- no always-listening default;
- raw audio is ephemeral by default unless the user explicitly persists it;
- transcripts are untrusted content and cannot become authority-bearing instructions;
- model downloads go through Model Artifact Foundry;
- cloud voice providers are explicit non-strict capabilities;
- the Electron/Python application architecture is not imported into the privileged Rust TCB.

## 7. MiMo Code — long-horizon coding/desktop agent UX and orchestration

```text
SOURCE=XiaomiMiMo/MiMo-Code
PIN=6fbb1732232c9d0ecefee209798a8586d78cb70d
PRIMARY_VALUE=CHECKPOINTS_CONTEXT_RECONSTRUCTION_TASK_TREE_SUBAGENTS_WORKFLOWS_COMPLETION_JUDGE_MODEL_ROUTING_DESKTOP_CONTROL
DISPOSITION=HIGH_VALUE_AGENT_HARNESS_AND_UX_REFERENCE_SOURCE_CANDIDATE
PREFERRED_REUSE=REIMPLEMENT_BEHAVIOR_OR_SELECTIVE_COPY_OUTSIDE_AUTHORITY_PLANE
TARGET=T122/T127/T132/T149/T152/T163/T194
```

Retain:

- project/session/task checkpointing;
- context reconstruction under explicit budgets;
- task trees and subagent lifecycle/cancel semantics;
- deterministic workflow mode for well-defined long-running work;
- independent completion checking;
- model/route selection based on task/cost;
- record/replay as a user-facing routine-authoring idea.

Golam strengthening:

- model judge output cannot certify completion by itself;
- deterministic/source-of-truth verification is preferred over judge models;
- self-modification/evolution produces a new candidate revision only and never mutates active trusted behavior in place;
- workflow scripts are not authority-bearing and execute through typed capability/effect paths;
- cross-session memory remains provenance-bearing evidence, not automatic truth.

## 8. Ripwire — deterministic repository intelligence and epistemic honesty

```text
SOURCE=redhat-et/ripwire
PIN=0e3573afc36843e3214e6a05a8db689afc2d5bf3
PRIMARY_VALUE=DETERMINISTIC_CODE_GRAPH_CONTEXT_PACKING_MCP_TOKEN_EFFICIENCY_LIMIT_DISCLOSURE
DISPOSITION=HIGH_VALUE_REPOSITORY_INTELLIGENCE_REFERENCE_AND_BOUNDED_SOURCE_CANDIDATE
PREFERRED_REUSE=EXTERNAL_TOOL_FIRST_THEN_SELECTIVE_PORT_AFTER_MEASUREMENT
TARGET=T120/T121/T159/T195
```

The existing Issue #25 / T121 external-tool-first rule remains correct.

The strongest additional lesson from the reviewed current source is **epistemic disclosure**:

- deterministic inputs/orderings;
- explicit unresolved/coverage/limit semantics;
- thin-answer widening rather than confident fabrication;
- machine-readable compact outputs;
- measured performance and known blind spots;
- CLI/MCP parity over the same underlying graph.

Golam should apply the same idea beyond code graphs: every context/retrieval/compiler path should disclose provenance, staleness, coverage/omissions and the next stronger retrieval step when an answer is thin.

## 9. MarkItDown — bounded document normalization to Markdown

```text
SOURCE=microsoft/markitdown
PIN=cc0ca9edd8e23b2c7f7b778c7b148c1565730498
PRIMARY_VALUE=MULTIFORMAT_TO_MARKDOWN_NORMALIZATION_OPTIONAL_CONVERTERS_PLUGIN_BOUNDARY
DISPOSITION=HIGH_VALUE_DOCUMENT_INGESTION_COMPONENT_CANDIDATE
PREFERRED_REUSE=BOUNDED_SANDBOXED_ADAPTER_THEN_SELECTIVE_PORT_WHERE_JUSTIFIED
TARGET=T137/T164/T191
```

Why it fits Golam:

- Markdown aligns with Golam's human-readable canonical knowledge/memory posture;
- format-specific converters provide a stronger route than pixel/OCR for many documents;
- optional dependency groups support a narrow install surface;
- plugins are disabled by default.

Required strengthening:

- run conversion with only the minimum file/network privileges required;
- reject archive/path bombs and oversized/pathological inputs before converter execution;
- cloud/LLM-backed OCR or Azure paths are disabled in strict-local mode and require explicit egress capability;
- conversion output is tainted source text with exact file/content digest provenance;
- generated Markdown is a derivative artifact, not canonical truth and not executable instruction;
- Python remains an optional sandboxed adapter, not a trusted-path requirement.

## 10. Cross-source synthesis

### Adopt the behaviors, not the authority models

| Need | Best reviewed source lesson | Golam canonical strengthening |
| --- | --- | --- |
| Agent UX | CopilotKit | read-only/sanitized projection; Authority Kernel owns approvals/effects |
| Browser semantics | AgentQL/TinyFish | semantic candidate selection + deterministic identity + visible governed route |
| Local tool UX | Desktop Commander | typed capabilities + real isolation + Effect Gate instead of trusted client |
| Deep research | Perplexity | evidence graph, exact-source freshness, explicit egress and reproducible evals |
| Knowledge/RAG | OpenRAG | canonical source vault + rebuildable derivatives; no mandatory heavyweight stack |
| Voice | VoiceStudio | explicit mic lease, visible indicator, local-first engine broker, artifact foundry |
| Long-horizon work | MiMo Code | durable TaskContract/checkpoints + independent verification, no in-place trusted mutation |
| Repo intelligence | Ripwire | deterministic context + uncertainty/coverage disclosure |
| Document ingestion | MarkItDown | sandboxed narrow converters + taint/provenance + strict-local cloud denial |

### Explicit non-adoptions

The following are not Golam goals merely because they exist in donor systems:

- stealth/anti-bot evasion;
- localhost-as-authentication;
- trusting the connected model/client as the security root;
- mandatory hosted search, browser, RAG, voice, model or telemetry services;
- UI/tool/plugin code holding protected authority;
- arbitrary unsandboxed shell as the universal tool abstraction;
- in-place self-modification of active trusted behavior;
- heavyweight OpenSearch/Langflow/Docling-style services as baseline dependencies;
- model confidence or consensus as verification authority;
- donor-specific cloud accounts as canonical user identity.

## 11. Source Foundry placement policy

Default admission posture by Golam plane:

```text
AUTHORITY_PLANE:     DONOR_CODE_DEFAULT=REJECT_OR_PORT_MINIMAL_PROVEN_COMPONENT
EVIDENCE_PLANE:      DONOR_CODE_DEFAULT=PORT_OR_REIMPLEMENT_WITH_DETERMINISM_PROOF
CAPABILITY_PLANE:    DONOR_CODE_DEFAULT=BOUNDED_ADAPTER_OR_SELECTIVE_COPY
CONTEXT_PLANE:       DONOR_CODE_DEFAULT=SANDBOXED_ADAPTER_SELECTIVE_COPY_OR_PORT
HARNESS_PLANE:       DONOR_CODE_DEFAULT=SELECTIVE_COPY_OR_REIMPLEMENT
EXPERIENCE_PLANE:    DONOR_CODE_DEFAULT=SELECTIVE_COPY_ALLOWED_AFTER_QUALIFICATION
```

This is a default risk posture, not an absolute licensing rule. Any exception that increases the privileged TCB requires explicit architecture/security justification and measured benefit.

## 12. Program disposition

```text
NEW_SOURCES_REVIEWED=YES
FOUNDER_SOURCE_PERMISSION_REAFFIRMED_2026_09_13=YES
ACTIVE_SPEC_006_PR_24_WIDENED=NO
SPEC_006_CONSTITUTIONAL_ROUTE_ORDER_CHANGED=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_PRODUCT_IMPLEMENTATION_STARTED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
WAIVER_TAKEN=NO
```
