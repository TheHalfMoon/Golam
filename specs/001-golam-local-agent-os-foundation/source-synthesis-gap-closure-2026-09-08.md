# Golam Source Synthesis and Gap Closure — 2026-09-08

**Status**: PROGRAM-LEVEL PLANNING INPUT — NO PRODUCT IMPLEMENTATION AUTHORITY

**Parent overlay**: `program-superiority-2026.md`

**Task extension**: `program-superiority-gap-closure-tasks.md`

**Source supplement**: `competitive-source-register-supplement-2026-09-08.md`

**Active Spec 006 implementation PR widened**: `NO`

## 1. Purpose

This document reconciles three planning inputs into one future program direction:

1. the current Program Superiority overlay and T110–T165;
2. valuable but non-canonical concepts still stranded in historical planning PRs #6, #7, #8 and #22; and
3. newly reviewed protocol, compute, containment, formal-assurance, agent-security, owner-presence and observability sources.

The objective is not to maximize dependency count or copy donor architectures wholesale. The objective is to make Golam a materially stronger user-owned Agent OS while preserving the Constitution, protected authority kernel, Effect Gate, `UNKNOWN_OUTCOME` semantics, strict-local denial, Source Foundry and evidence-first release discipline.

## 2. Founder source-reuse posture

The founder reaffirmed on 2026-09-08 that Golam may copy, modify, port and reuse code from the sources supplied during this research and sources already recorded in the repository, subject to the exact rights/technical admission record for the selected component.

This changes the planning default from "reference behavior only when permission is unknown" to "reuse-eligible candidate when technically and legally qualified."

It does not change these invariants:

```text
PERMISSION_TO_COPY != TECHNICAL_ADMISSION
PERMISSION_TO_COPY != LICENSE_OBLIGATIONS_DISAPPEAR
DONOR_IMPLEMENTATION != DONOR_AUTHORITY_MODEL
SOURCE_FOUNDry_SUCCESS != PRODUCT_QUALIFICATION
```

Every owning spec chooses one reuse mode explicitly:

```text
COPY_AS_IS
SELECTIVE_COPY
PORT_TO_RUST
ADAPTER
REIMPLEMENT_BEHAVIOR
BENCHMARK_ONLY
REJECT
```

The decision must prefer the lowest attack surface and maintenance cost that preserves the desired behavior. Direct copy is appropriate only when it is better than a bounded port or adapter after dependency, FFI, network, telemetry, secrets, platform and long-term maintenance analysis.

## 3. Planning consolidation registry

The following open planning PRs are valuable historical inputs but must not remain parallel live roadmaps after this overlay becomes canonical.

### PR #6 — Native mobile and channel access

Carry forward:

- Native Golam Mobile as a cryptographically paired GolamConnect device;
- third-party messaging channels as lower-assurance untrusted transports;
- push notifications as wake/sync hints only;
- voice/media content as input, never authentication;
- signed offline requests as request-origin evidence, never current authorization;
- remote-control transport separated from messaging transports;
- stable provider identity as a binding key, never authority.

Owning future tasks: T141, T169, T174, Spec 007.

### PR #7 — Governed memory, retrieval, learning and execution providers

Carry forward:

- `MemoryCandidate != durable truth`;
- retrieval/index scores never raise source authority;
- Context Compiler routing by measured need;
- sandbox/provider state outside protected authority;
- learning as immutable proposals/candidates;
- deterministic/execution-grounded evaluation before LLM judging;
- OpenSandbox-class provider contracts without treating a sandbox provider as the authority root.

Owning future tasks: T120–T126, T151, T165, T173, T175, T181.

### PR #8 — Product spine, verified Task truth and Authority Host

Carry forward as high-priority architectural debt closure:

```text
TASK != SESSION != RUN != WORKER
SUCCEEDED_RUN != SATISFIED_TASK
MODEL_SAYS_DONE != VERIFIED_CRITERION
TRUST_RECEIPT != AUTHORITY_RECORD
EXECUTION_NODE != AUTHORITY_HOST
BACKUP_STATE != ACTIVE_AUTHORITY
```

Carry forward:

- versioned `TaskContract` and criterion-level Verification Plan;
- `TrustReceipt` projecting evidence, egress, effects and uncertainty;
- task controls that do not rewrite authority;
- one active user-owned Authority Host by default;
- bounded Execution Nodes;
- anti-fork recovery, fresh authentication root and external-credential quarantine;
- lost-host recovery preserving effect/scheduler uncertainty.

Owning future tasks: T149–T153, T167, T168, T174, T182.

### PR #22 — Agent Operating Kernel roadmap v2

Preserve the useful six-plane conceptual decomposition as an architectural lens, not a second runtime stack:

```text
Experience Plane
Harness Plane
Capability Plane
Context Plane
Evidence Plane
Authority Plane
```

Map the planes onto current Golam ownership instead of recreating parallel APIs. In particular, the Evidence Plane maps to verification/effect/receipt structures while the Authority Plane remains protected and non-model-controlled.

Historical references from PR #22 — Semantica, Multica, OpenWork, OpenClaw, WorldMonitor, Paperclip, Prime Agent, DeepSeek Harness, ZeroClaw, OpenManus and OpenSEO — remain research inputs and require current requalification before implementation reuse.

## 4. Canonical topology model

Future distributed/cross-device work should converge on four explicit trust-domain roles.

### AuthorityHost

The user-owned node that holds the active protected authority domain, durable Effect truth, canonical task/verification state and memory-governance roots.

```text
AUTHORITY_HOST_COUNT_ACTIVE_BY_DEFAULT=1
AUTHORITY_HOST != CLOUD_REQUIRED
AUTHORITY_HOST_LOSS != PERMISSION_TO_FORK_AUTHORITY
```

### ExecutionNode / ComputeNode

A paired node that executes bounded jobs, models, tools or isolated environments under current authority projections/leases.

```text
EXECUTION_NODE != AUTHORITY_HOST
COMPUTE_NODE != AUTHORITY_HOST
WORKER_PLACEMENT != AUTHORITY_TRANSFER
MODEL_PLACEMENT != AUTHORITY_TRANSFER
```

### ExternalA2AAgent

An external agent reachable through an interoperability protocol. It is outside Golam's trust domain unless separately enrolled as a Golam-controlled worker/node through a Golam protocol.

```text
EXTERNAL_AGENT_OUTPUT=UNTRUSTED_EVIDENCE_OR_CONTENT
AGENT_CARD != CAPABILITY_LEASE
REMOTE_AGENT_SUCCESS != VERIFIED_COMPLETION
```

### ChannelTransport

Telegram/WhatsApp/Slack/Discord/Matrix/email/push or equivalent transport. It transports content and may carry a bound provider identity, but does not become Golam authority.

```text
CHANNEL != AUTHORITY
VOICE_CONTENT != AUTHENTICATION
PUSH_DELIVERY != CANONICAL_ORDER
```

## 5. Protocol responsibility matrix

Golam must avoid protocol confusion as the ecosystem expands.

| Protocol/domain | Golam responsibility | Explicit non-responsibility |
| --- | --- | --- |
| MCP | Tool/context/resource interoperability and compatible extension surfaces | Does not grant Golam capability/approval/authority |
| A2A | Interoperability with opaque external agents/tasks/messages | AgentCard/task acceptance is not trust-domain membership or authority |
| ACP | User/client-to-agent product integration where selected | Does not replace kernel authorization or protected task truth |
| GolamConnect | Authenticated user-owned device/node identity, transport and bounded remote projections | Does not make relay/cloud infrastructure the authority root |
| Internal Worker Protocol | Governed Golam worker membership, handoff, leases, checkpoints and verification | Worker membership does not imply shared unrestricted credentials/authority |

Every protocol adapter maps inbound content to explicit taint/provenance and maps outbound consequential behavior through ordinary Golam authorization/effect/egress gates.

## 6. Newly qualified planning sources and exact dispositions

### MCP 2026-07-28 profile

Reviewed source anchor:

`modelcontextprotocol/modelcontextprotocol@e76e9c572c6f2bfcb730357101acc90f2f802e02`

Adopt as protocol/reference authority for an explicit future `McpProtocolProfile`, including protocol/version negotiation, extension/task semantics, authorization hardening and migration away from deprecated assumptions. Golam's existing MCP bindings must become version/profile explicit rather than silently assuming one historical MCP shape.

Disposition: `STANDARD_PROFILE_AND_CONFORMANCE_REFERENCE`.

### A2A v1.0

Reviewed source anchor:

`a2aproject/A2A@98853be376c88df25e1704771cd3ea9ef8823a96`

Adopt for an `ExternalAgentInteropProfile` only. Inbound tasks/messages are untrusted proposals/content, remote agent output remains unverified until Golam verification, and no Golam capability token is sent to an arbitrary external agent.

Disposition: `EXTERNAL_AGENT_INTEROP_REFERENCE_AND_ADAPTER_CANDIDATE`.

### NVIDIA Personal AI Router

Reviewed source anchor:

`NVIDIA/Personal-AI-Router@13b68115fa2c9c1d94f1ead1358f8d5a527cfecf`

License posture reviewed as Apache-2.0 at this source state. High-value design/source candidate for local paired inference nodes, mTLS identity, readiness/model/GPU-aware routing and local-LAN operation.

Golam should adopt the product value but make routing stricter:

```text
LOCALITY_PRIVACY_AUTHORITY_COMPATIBILITY_FIRST
RESOURCE_COST_QUALITY_OPTIMIZATION_SECOND
HIDDEN_MODEL_DOWNLOAD=DENIED
COMPUTE_NODE_AUTHORITY=NONE
```

Disposition: `HIGH_VALUE_LOCAL_COMPUTE_MESH_SOURCE_CANDIDATE`.

### Kubernetes Agent Sandbox

Reviewed source anchor:

`kubernetes-sigs/agent-sandbox@e5e2831295a789299722a9f4d922250167b1e879`

Use as containment/runtime-lifecycle reference, especially for gVisor/Kata-class isolation. Do not inherit Kubernetes as a mandatory Golam dependency or equate a container with a security boundary.

Disposition: `EXECUTION_ISOLATION_REFERENCE`.

### Quint / TLA+-style formal assurance

Reviewed source anchor:

`quint-co/quint@6fb2924e00707cef6dbc5e30db606d555c447123`

Use selectively for finite critical state machines: Effect/reconciliation, lease generation/revocation, scheduler dedup/catch-up, Authority Host migration/recovery anti-fork, mobile approval replay protection, worker handoff/resume and extension activation/revocation.

Counterexamples become deterministic regression fixtures. Formal models are evidence about modeled contracts, not claims of whole-product formal verification.

Disposition: `FORMAL_ASSURANCE_TOOL_REFERENCE_AND_CANDIDATE`.

### NIST AI agent security

Reference: NIST Trustworthy and Responsible AI 800-5, 2026 RFI summary on security considerations for AI agents.

Use as a taxonomy/reference input for the system threat model and control matrix, not as product certification.

Disposition: `SECURITY_TAXONOMY_REFERENCE`.

### OWASP Agentic Security Initiative

Use current agentic threat/control material to ensure explicit treatment of goal hijacking, tool misuse, identity/privilege abuse, memory/context poisoning, insecure inter-agent communication, cascading failures, rogue agents and agent supply-chain/runtime compromise.

Disposition: `ADVERSARIAL_CONTROL_REFERENCE`.

### WebAuthn Level 3

Use W3C WebAuthn Level 3 as one high-assurance owner-presence candidate for specific high-risk approvals, device pairing and Authority Host recovery.

Hard boundary:

```text
USER_VERIFIED != OPERATION_AUTHORIZED
OWNER_PRESENCE_RECEIPT != CAPABILITY_LEASE
BIOMETRICS_MANDATORY=NO
```

Disposition: `OWNER_PRESENCE_STANDARD_REFERENCE`.

### OpenTelemetry GenAI semantic conventions

Use as an interoperability reference for a privacy-safe local observability projection: model/provider latency, tokens/cost where available, worker/resource timing and diagnostics. Prompt/tool content collection is disabled by default and ordinary telemetry never becomes protected evidence or authority.

```text
TRACE != EVIDENCE
TRACE != AUTHORITY
SILENT_REMOTE_TELEMETRY=DENIED
```

Disposition: `OBSERVABILITY_SCHEMA_REFERENCE`.

## 7. Gap closure decisions

### Gap A — Local compute is under-modeled

Current Resource Governor covers one host well but does not define a user-owned local compute mesh. Add a topology where paired compute nodes advertise exact model/backend/resource state and receive bounded work without receiving authority state.

### Gap B — External agent interoperability is absent

MCP is not agent-to-agent governance. Add A2A support as an external interoperability layer while preserving internal Golam worker membership as a stronger separate trust-domain contract.

### Gap C — MCP evolution is under-specified

Current MCP work must become explicit about protocol profile/version and conformance. Deprecated/changed MCP behaviors cannot remain implicit compatibility assumptions.

### Gap D — "Sandbox" is too ambiguous

Introduce `ExecutionIsolationProfile` with explicit classes, evidence and externally observed containment. Providers cannot self-claim a stronger isolation class.

Candidate classes:

1. ordinary process restrictions;
2. OS sandbox/profile;
3. container with exact runtime policy;
4. syscall-interposer/gVisor-class containment;
5. VM/microVM/Kata-class containment;
6. remote sandbox with explicit provider trust/egress posture.

`CONTAINER != SECURITY_BOUNDARY` unless the exact runtime/profile is qualified.

### Gap E — Model artifacts need their own Foundry

Add `ModelArtifactFoundry` for exact weights digest, license, tokenizer bytes, chat-template identity, quantization/conversion provenance, backend/runtime identity and hardware compatibility.

```text
MODEL_HASH != MODEL_TRUST
MODEL_OUTPUT != AUTHORITY
TRUST_REMOTE_CODE_DEFAULT=DENIED
UNDECLARED_MODEL_DOWNLOAD=DENIED
```

### Gap F — Critical state machines lack formal model evidence

Add `FormalAssuranceFoundry` for selected bounded state machines. Do not attempt whole-product formal verification before the implementation architecture stabilizes.

### Gap G — Agent security controls are distributed but not mapped

Add an `AgentSecurityControlMatrix`:

```text
threat
-> Golam invariant
-> owning contract/spec
-> deterministic adversarial test
-> evidence class
-> residual-risk statement
```

Map the existing T156 threat model to NIST/OWASP agent-security categories.

### Gap H — Instruction/data trust and egress need a single cross-cutting broker

External web pages, documents, email, MCP output, connector data, remote-agent output and retrieved memory never become trusted instructions merely because the model reads them.

Before sensitive egress, bind:

- data class/taint;
- destination/provider/account;
- purpose;
- retention expectation where knowable;
- redaction/secret posture;
- current egress capability;
- effect/approval class where consequential.

### Gap I — Extensions need publisher/revocation lifecycle beyond static scanning

T165 covers security admission. Add an immutable signed extension catalog with package identity, publisher provenance, version, Source Foundry/security/conformance evidence, activation state and revocation transparency.

Publisher signature is provenance, not authority.

### Gap J — Candidate activation needs shadow rehearsal

Skills, routines, automations, extensions and learned workflows should support no-external-effect rehearsal/simulation/observe-only activation gates before production where possible.

### Gap K — Resilience testing is too benchmark-oriented

Add explicit chaos/resilience qualification for disk full, clock jump, suspend/resume, network partition, relay loss, compute-node loss, OOM/GPU eviction, connector expiry, browser update, daemon crash, extension revocation, Authority Host loss, mobile loss and partial restore.

### Gap L — Product surfaces must converge on the same authority path

Add `SurfaceParityContract`: CLI, TUI, Desktop, Mobile, IDE, MCP-facing products and remote surfaces may differ in UX but cannot create a privileged surface-only bypass. The same semantic operation maps to the same kernel authorization/effect semantics.

### Gap M — Multi-worker consensus can hide dissent

Add durable `DecisionRecord` / dissent semantics. Majority vote or model consensus never grants authority. Material dissent about safety, target identity, destructive effects or verification remains visible and escalates to a stronger verifier or user when policy requires.

## 8. Model and execution trust lattice

Future planning should use this ordering:

```text
USER/OWNER AUTHORITY
  > PROTECTED KERNEL AUTHORITY STATE
  > VERIFIED EFFECT/IDENTITY EVIDENCE
  > QUALIFIED EXECUTION/CONTROL PROVIDER OBSERVATION
  > EXTERNAL TOOL/AGENT/MODEL OUTPUT
```

Higher model intelligence never raises the authority of its output. A remote execution provider, external agent or model can produce useful observations/artifacts while remaining outside the protected trust root.

## 9. Reuse strategy by source class

### Prefer selective copy/port

Use for compact deterministic algorithms, codecs, scheduling primitives, graph/ranking logic, parsers or tests where donor behavior is well-bounded and dependencies are acceptable.

### Prefer adapter

Use for standardized protocols or large independent runtimes such as A2A, MCP, browser protocols, sandbox providers and model servers.

### Prefer behavioral reimplementation

Use when donor architecture couples useful UX/behavior to an incompatible authority model, cloud requirement, runtime language/dependency closure or broad privileged surface.

### Benchmark only

Use where legal posture, architecture, maintenance cost or attack surface makes runtime reuse unattractive but the source is valuable for evaluation.

## 10. Non-adoptions

The following remain explicitly rejected unless a later bounded spec proves a different safe posture:

- model-owned or external-agent-owned authority;
- automatic self-rewriting active skills;
- shared computer/workspace presented as a security boundary;
- remote compute as the only canonical authority copy;
- protocol capability advertisement treated as authorization;
- container label treated as proof of containment;
- telemetry as proof of success;
- owner-presence authentication treated as operation authorization;
- silent cloud/model fallback;
- hidden model/package downloads during runtime;
- extension publisher signatures treated as trusted execution permission;
- consensus of agents treated as human approval;
- stale backup recovery that clears unknown external-effect or credential state.

## 11. Sequencing

1. Keep PR #24 limited to canonical Spec 006 implementation.
2. Qualify and merge this planning overlay only after exact-head CI and fresh independent substantive review.
3. Consolidate historical planning PR concepts into owning future specs rather than merging the stale planning stacks as parallel authority.
4. After Spec 006 canonical closeout, re-read live successor authority before creating any implementation unit from T166+.
5. Each source/code reuse decision still requires exact current Source Foundry admission at implementation time.

## 12. Proposed program outcome

Golam should target a product that combines persistent teammates, governed learning, local/cross-device autonomy, semantic computer/browser control, local compute routing, external-agent interoperability and a rich extension ecosystem while retaining stronger guarantees than current alternatives:

```text
INTELLIGENCE_IS_REPLACEABLE
AUTHORITY_IS_NOT
EXECUTION_IS_PORTABLE
EVIDENCE_IS_DURABLE
LEARNING_IS_GOVERNED
COMPLETION_IS_PROVED
EGRESS_IS_EXPLICIT
RECOVERY_DOES_NOT_REWRITE_HISTORY
```

This is the target superiority claim architecture. Public superiority claims remain prohibited until GolamBench and exact release evidence support them.