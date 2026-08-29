# PA-002A — OpenClaw and OpenSandbox Source Foundry Addendum

**Status**: PROPOSED_FOR_REVIEW  
**Date**: 2026-08-28  
**Parent amendment**: `PA-002-memory-retrieval-learning-evals.md`  
**Implementation authorization**: NONE. This document strengthens future Spec 003/005/007/008/009/010 research and qualification requirements. It does not authorize adding either project as a runtime dependency or changing the active Spec 003 task order.

## 1. Executive decision

OpenClaw and OpenSandbox are both high-value Golam donors, but for different layers.

- **OpenClaw** is a high-value behavioral and product-architecture donor for personal-agent gateway UX, Markdown-first memory, device/node pairing, channels, companion devices, skill/plugin ergonomics, scheduled work, memory consolidation, and operator-facing security audits.
- **OpenSandbox** is a high-value sandbox/runtime substrate candidate for untrusted code, skills, MCP helpers, coding agents, browser/desktop workloads, experiments, evaluation trials, bounded remote execution, egress policy, and credential brokering.

Neither project becomes Golam's authority root.

OpenClaw's single-trusted-operator Gateway model is weaker than Golam's target authority isolation. OpenSandbox's lifecycle server, `execd`, egress sidecar, snapshots, or control APIs likewise remain subordinate execution infrastructure. Golam's kernel continues to own identity, capabilities, approval, secret authority, egress authorization, effect intent/reconciliation, and audit truth.

```text
Golam privileged kernel
  |
  +-- authorizes lease/effect/egress/secret handle
  |
  +--> Sandbox Provider Interface
         |
         +-- native OS sandbox / Wasmtime
         +-- Docker/Podman
         +-- OpenSandbox candidate
         +-- remote qualified provider

Sandbox provider may execute.
Sandbox provider may not mint authority.
```

## 2. Exact source snapshots

| ID | Source | Exact inspected state | License | Planning classification |
|---|---|---|---|---|
| SF2A-001 | `openclaw/openclaw` | `main@23a681efa6fc0e264e562c4249d8906c0785b5e4` | MIT | HIGH_VALUE_PRODUCT / MEMORY / CHANNEL / SECURITY REFERENCE; SELECTIVE_PORT candidate |
| SF2A-002 | `opensandbox-group/OpenSandbox` | `main@48b0215f1bd097b31d0f022a44640e00c11ac49d` | Apache-2.0 | HIGH_VALUE_SANDBOX_PLATFORM / PROTOCOL / DIRECT_PROVIDER candidate |

No code is admitted by this addendum. Future owning specs must re-pin exact source state and complete normal Source Foundry admission.

## 3. OpenClaw — useful mechanisms

### 3.1 Gateway and device model

The inspected architecture uses one long-lived Gateway as the local control plane for sessions, tools, events, channel connections, control clients, and paired nodes. Control clients and nodes connect over a typed WebSocket protocol. Nodes advertise capabilities/commands, device pairing is explicit, connection challenges are signed, and reconnect metadata is pinned.

Golam should absorb:

- typed client/node protocol discovery;
- explicit device identity and pairing state;
- reconnect metadata pinning;
- capability advertisement separated from capability authorization;
- control UI / CLI / TUI / companion-node continuity;
- idempotency keys for side-effecting API requests;
- health/presence/event surfaces useful for operator UX.

Golam divergence:

- `LOCALHOST != AUTHENTICATION` remains binding; Golam must not treat loopback locality as sufficient authority;
- device capability advertisement is data, not a lease;
- node pairing does not imply blanket machine authority;
- every protected request still revalidates current Golam capability/lease/policy state;
- remote-control generations, replay protection, approval freshness, and Effect Gate semantics remain Golam-owned.

### 3.2 Memory

The inspected OpenClaw memory design is unusually relevant to Golam:

- `USER.md` for stable user-profile directives;
- `MEMORY.md` for curated durable memory;
- dated `memory/YYYY-MM-DD*.md` working notes;
- optional `DREAMS.md` human-review surface;
- SQLite-backed local memory/search engine;
- hybrid semantic + keyword retrieval;
- explicit memory import from other assistants;
- action-sensitive memories carrying timing/authority/expiry context;
- background "dreaming" that scores and consolidates candidates;
- taint gating before durable promotion;
- a provenance-rich Memory Wiki layer with claims, evidence, contradiction, freshness, dashboards and deterministic page structure;
- pre-compaction memory flush.

These validate and strengthen PA-002's Markdown-first canonical design.

Golam should absorb the best UX concepts while keeping stricter governance:

```text
Observation / conversation / tool result
  -> MemoryCandidate
  -> provenance + taint + scope + temporal validity
  -> promotion review
  -> canonical Markdown / SQLite evidence
  -> derived indexes / wiki / graph
```

OpenClaw's `memory-wiki` concept is a strong candidate for Golam's human-auditable knowledge projection, but a compiled wiki remains derived unless the owning Spec explicitly promotes a human-authored canonical page.

`DREAMS.md` inspires a Golam **Learning Journal** / **Memory Review Ledger** where background learning proposals remain inspectable before or after promotion according to policy.

### 3.3 Channels and companion devices

OpenClaw supports a broad channel/gateway model plus nodes on macOS/iOS/Android/headless systems. This is high-value implementation evidence for future Spec 007 UX and adapter ergonomics.

Golam must preserve PA-001's stronger invariants:

- `CHANNEL != AUTHORITY`;
- messaging account possession cannot bootstrap native device pairing;
- free-form message/reaction/voice is not consequential-effect approval;
- high-risk approvals step up to Native Golam Mobile or authenticated local clients;
- protected remote control stays on GolamConnect, not ordinary messaging transports.

### 3.4 Security audit as product feature

OpenClaw's `security audit` / `--deep` / `--fix` pattern is worth adopting as a first-class Golam operator surface.

Future Golam should provide a deterministic security posture command, for example:

```text
golam security audit
golam security audit --deep
golam security audit --json
golam doctor --security
```

The audit should inspect at minimum:

- unauthenticated local/remote control surfaces;
- stale or broad capability leases;
- irreversible-effect preauthorization scope;
- secret broker fallbacks;
- egress policy and strict-local drift;
- sandbox/provider escape surfaces;
- host filesystem mounts;
- plugin/MCP/skill trust state;
- channel identity/pairing state;
- memory/index provenance and poisoning indicators;
- browser/computer-control exposure;
- local file/DB permissions;
- unverified or unsigned provider artifacts;
- hidden telemetry/update/network paths.

Unlike an advisory-only linter, findings touching protected authority must map to deterministic Golam invariants and exact remediation evidence.

### 3.5 OpenClaw sandbox lessons

OpenClaw's current sandbox system supports Docker, Podman, SSH, and OpenShell backends with per-agent/session/shared scope, workspace `none/ro/rw`, Docker defaults such as `network:none`, read-only root, dropped capabilities, tmpfs, and no-new-privileges.

Useful lessons:

- sandbox scope is an explicit product concept;
- creator-required sandboxing should be immutable/fail-closed for a session;
- workspace identity must participate in sandbox identity;
- sandbox provider capabilities need a published matrix;
- unavailable required sandbox backends must fail closed;
- host engine sockets/credentials must never be mounted into agent sandboxes.

Golam divergence:

- sandboxing for untrusted/native execution should be deny-by-default where the owning capability requires it, not "off by default" as a general product posture;
- native plugins cannot automatically share the privileged kernel trust boundary;
- "elevated" execution cannot become a generic escape from an authority-required sandbox;
- policy/effect/secret/egress checks occur outside the untrusted sandbox and cannot be replaced by sandbox configuration.

## 4. OpenSandbox — useful mechanisms

### 4.1 Public protocol and provider separation

OpenSandbox separates:

- language SDK / CLI / MCP clients;
- OpenAPI protocol contracts;
- lifecycle control plane;
- Docker and Kubernetes runtime providers;
- in-sandbox `execd` data plane;
- ingress/egress/security components.

This decomposition is directly useful for a Golam `SandboxProvider` contract.

Future Golam should define a provider-neutral sandbox descriptor with capabilities such as:

- create/start/wait;
- pause/resume;
- renew/expire;
- kill/delete;
- snapshot/restore where supported;
- command exec and PTY;
- files/directories;
- browser/desktop endpoints;
- resource limits and metrics;
- network policy;
- credential broker support;
- workspace/volume bindings;
- secure-runtime/isolation class;
- diagnostic and evidence export.

Required/optional capabilities should have conformance tests, similar to the provider-conformance principle already introduced by PA-002.

### 4.2 Runtime diversity

OpenSandbox supports local Docker and Kubernetes runtime backends, including stronger isolation runtimes such as gVisor, Kata Containers and Firecracker-oriented configurations.

Golam should not standardize one universal sandbox. Future qualification should match workload to isolation class:

- pure portable extension -> Wasmtime/WASI where sufficient;
- ordinary disposable local tool -> hardened container where sufficient;
- untrusted native/code execution -> stronger OS/container isolation;
- high-risk multi-tenant/remote evaluation -> microVM/Kata/Firecracker-class boundary where justified;
- remote provider -> explicit authenticated provider with equivalent policy/evidence contract.

### 4.3 Egress policy

OpenSandbox's egress sidecar offers DNS/FQDN/CIDR policy, default-deny patterns, platform `always-deny` / `always-allow` overlays, and runtime policy inspection.

This is valuable implementation evidence for Golam sandbox egress, but the authority direction is reversed:

```text
Golam kernel EgressDecision
   -> compiled provider policy
   -> OpenSandbox/native provider enforcement
   -> external observation / receipt
```

The sandbox must not widen egress beyond the lease/policy minted by Golam. Runtime patch APIs cannot become an agent-facing authority-expansion path.

`STRICT_LOCAL` must still be proven from outside the sandbox/provider process boundary.

### 4.4 Credential Vault / secret brokerage

OpenSandbox's Credential Vault is especially relevant. The real credential remains in a host/sidecar control path while the sandbox receives fake/placeholder values. Outbound HTTPS is matched against precise bindings and the real credential is injected only at the network boundary.

This strongly validates Golam's existing secret-broker architecture.

Future Spec 003/005 should evaluate a provider interface capable of:

- secret handle, never plaintext tool argument;
- exact scheme/host/port/method/path binding;
- optional header/query/path/body substitution only where explicitly required;
- default-deny egress prerequisite;
- no secret in sandbox env/argv/fs/logs/snapshots;
- write-only secret provisioning;
- redaction of raw and encoded forms;
- broker state outside restorable untrusted snapshot;
- mandatory re-injection/revalidation after sandbox/provider restart/resume;
- audit receipt linking secret handle, binding, effect, destination and result without plaintext.

Golam should prefer end-to-end TLS-preserving upstream credential brokers where possible. Transparent HTTPS MITM is an implementation technique requiring explicit trust/certificate/security review, not a default architectural requirement.

### 4.5 Snapshots and pause/resume

OpenSandbox supports pause/resume and snapshot patterns, including container/image/rootfs persistence.

For Golam:

`SANDBOX_SNAPSHOT != AUTHORITY_SNAPSHOT`

A restored sandbox must never restore stale capability leases, approvals, secret plaintext, egress grants or protected effect state as authority. Resume requires a fresh kernel authorization envelope and secret-broker provisioning.

### 4.6 Network-isolation caveats

OpenSandbox documents important runtime compatibility limits: egress interception depends on runtime networking features and conflicts can exist with gVisor netstack or transparent service-mesh sidecars.

Golam must therefore publish an exact sandbox capability matrix and never claim a security property merely because the provider exposes a knob. Qualification must verify the actual runtime/platform combination.

## 5. Binding new invariants

The following are additive to PA-002 and existing contracts:

- `SANDBOX_PROVIDER != AUTHORITY_ROOT`
- `SANDBOX_SNAPSHOT != AUTHORITY_SNAPSHOT`
- `SANDBOX_CAPABILITY_ADVERTISEMENT != CAPABILITY_LEASE`
- `SANDBOX_NETWORK_POLICY <= KERNEL_EGRESS_AUTHORITY`
- `SANDBOX_RESUME_REQUIRES_AUTHORITY_REVALIDATION`
- `SECRET_BROKER_BINDING != GENERAL_NETWORK_PERMISSION`
- `DEVICE_PAIRING != BLANKET_EFFECT_AUTHORITY`
- `SECURITY_AUDIT_FINDING != SECURITY_PROOF`

## 6. Future owning-spec changes

### Spec 003 — Identity, Policy, Secrets & Sandbox

Re-pin OpenSandbox and OpenClaw sandbox evidence before final sandbox-provider freeze. Add provider requirements for secret brokering, egress compilation, snapshot/resume authority invalidation, provider capability truth, workspace/mount policy and fail-closed required isolation.

No OpenSandbox dependency is authorized by this program amendment while Spec 003 is already active.

### Spec 005 — Local Tools, Context & Memory

Re-pin OpenClaw memory and OpenSandbox execution evidence. Evaluate:

- action-sensitive memory metadata;
- Memory Wiki / Learning Journal projection concepts;
- imported-memory quarantine and provenance;
- sandboxed tool execution through the provider seam;
- credential-brokered external tools;
- sandbox command/file/browser/desktop evidence capture.

### Spec 007 — Phone, GolamConnect & Channels

Re-pin OpenClaw Gateway/node/channel implementations as UX/protocol evidence. Preserve PA-001's stronger cryptographic device and approval boundaries.

### Spec 008 — Workers & Automations

Evaluate OpenSandbox as a worker/experiment isolation provider and OpenClaw scheduled/companion-agent patterns. Worker sandbox identity must bind worker/task/workspace and capability generation.

### Spec 009 — Grok parity

OpenClaw may provide independent implementation evidence for multi-channel continuity, companion devices, memory UX, scheduled work, plugins/skills and local operator surfaces. It is not parity evidence for Grok by itself.

### Spec 010 — GolamBench

Add sandbox-provider qualification suites:

- command/files/PTY lifecycle;
- cancel/kill/timeout;
- resource enforcement;
- workspace `none/ro/rw` isolation;
- mount traversal/symlink attacks;
- default-deny egress;
- provider policy cannot widen kernel egress;
- credential exfiltration canaries;
- restart/pause/resume secret re-provisioning;
- snapshot does not restore authority;
- sandbox-to-sandbox isolation;
- compromised workload cannot access provider control plane;
- secure-runtime capability truth;
- strict-local external observation;
- signed/pinned provider artifact verification where used.

## 7. Admission posture

### OpenClaw

Default posture: **high-value behavioral/source reference and selective-port candidate**. Its MIT license permits serious bounded reuse consideration, but the owning spec must still isolate selected components, dependency closure, Node/native/plugin behavior, network surfaces, telemetry/update behavior, secrets, and platform assumptions.

Golam must not import OpenClaw's single-operator trust assumptions as its own security model.

### OpenSandbox

Default posture: **high-value direct sandbox-provider/protocol candidate**, not a privileged-core dependency. Apache-2.0 makes bounded reuse/integration feasible, but future admission must requalify exact components and deployment modes.

A preferred integration shape is a replaceable out-of-process provider behind Golam-owned Rust contracts. Direct reuse of protocol/schema concepts or a Rust client may be considered if available and qualified; the Python FastAPI lifecycle server is not required to become part of Golam's trusted local core.

## 8. Review gate

Before this addendum can be treated as accepted planning evidence, verify:

1. no active Spec 003 implementation authority was expanded;
2. OpenClaw remains a product/reference donor, not a security trust root;
3. OpenSandbox remains an execution provider, not policy/secret/effect authority;
4. sandbox snapshot/resume cannot restore stale authority;
5. sandbox egress cannot exceed kernel authority;
6. credential injection cannot expose plaintext to workload state, logs or snapshots;
7. provider capability claims are conformance-tested per exact runtime/platform;
8. strict-local remains externally verifiable independent of provider self-reporting.
