# Golam Strategic Agent Reference Corpus

**Status**: REFERENCE-ONLY RESEARCH INPUT

**Purpose**: retain the exact founder-supplied public reference set that informed `golam-agent-kernel-roadmap-v2.md` and record the architectural lesson to evaluate without implying code admission.

No source listed here is admitted as code, dependency, binary, model, asset, or redistribution input merely by appearing in this file. Any reuse still requires the exact Source Foundry lifecycle required by the Golam Constitution.

## References

### Semantica

- Repository: https://github.com/semantica-agi/semantica
- Research focus: provenance, knowledge/context graphs, temporal facts, causal and decision relationships, reasoning, conflict handling.
- Golam lesson: record graph semantics and decision/evidence lineage while keeping canonical evidence independent of a graph database.

### Multica

- Repository: https://github.com/multica-ai/multica
- Research focus: multi-agent operations, task ownership, execution visibility, human review, reusable skills, local/remote runtimes.
- Golam lesson: adopt durable work/review primitives later; do not couple trusted core semantics to an organizational metaphor.

### OpenWork

- Repository: https://github.com/different-ai/openwork
- Research focus: capability discovery/execution, connected-service governance, MCP-facing control-plane abstractions.
- Golam lesson: converge heterogeneous powers behind a first-class capability plane with exact provider binding and kernel authorization.

### OpenClaw

- Repository: https://github.com/openclaw/openclaw
- Research focus: long-lived gateway, typed client/node protocol, device pairing, channels, session continuity, idempotent side-effect requests.
- Golam lesson: use a typed gateway/principal model while retaining stricter authentication, lease, protected-state, and effect boundaries.

### WorldMonitor

- Repository: https://github.com/koala73/worldmonitor
- Research focus: product packaging, desktop/web/API/MCP/SDK surfaces, machine discoverability.
- Golam lesson: architecture superiority must eventually become excellent distribution, diagnostics, discoverability, and dual human/agent UX.

### Paperclip

- Repository: https://github.com/paperclipai/paperclip
- Research focus: tasks, heartbeats, budgets, audit logs, execution locks, review/governance primitives.
- Golam lesson: build atomic WorkLease, budget, heartbeat, checkpoint, artifact, and review primitives; render organizational views only at product level.

### Prime Agent

- Repository: https://github.com/PrimeIntellect-ai/prime-agent
- Research focus: persistent agent sessions, supervisor/worker separation, long-running/background execution, goals, scheduling, recursive agents.
- Golam lesson: adopt continuity and lifecycle architecture while keeping optional Python/model-facing kernels outside privileged authority.

### DeepSeek Harness

- Repository: https://github.com/deepseek-ai/deepseek-harness
- Research focus: everything-is-a-plugin composition, typed events, reversible registrations, provider seams, durable session-event history.
- Golam lesson: adopt governed typed capability seams and preserve `MODEL_VISIBLE => LOGGED`; do not permit unrestricted privileged dynamic plugins.

### ZeroClaw

- Repository: https://github.com/zeroclaw-labs/zeroclaw
- Research focus: Rust local runtime, supervised autonomy, OS sandboxing, gateway/dashboard, channels, tool receipts.
- Golam lesson: preserve Rust/local-first posture and exceed ephemeral tool receipts with durable EvidenceReceipt + Effect Gate + target identity + readback binding.

### OpenManus

- Repository: https://github.com/FoundationAgents/OpenManus
- Research focus: simple general-agent loop, MCP, browser automation, tool-call agents, optional sandboxed agent paths.
- Golam lesson: keep the user/developer experience simple even when the trusted internals are substantially stricter.

### OpenSEO

- Repository: https://github.com/every-app/open-seo
- Research focus: vertical workflow product usable by humans and agents through UI, MCP, and reusable skills.
- Golam lesson: useful subsystems should become coherent agent-consumable capabilities without creating alternative authority paths.

## Cross-reference synthesis

The corpus informs six strategic planes:

```text
Experience
Harness
Capability
Context
Evidence
Authority
```

The central Golam differentiator remains:

> Models may propose actions, but models never own authority. Every consequential action is bounded, attributable, durable, and verifiable.

## Reuse gate

Before copying, porting, vendoring, forking, adding a direct dependency, executing a donor binary, or redistributing donor material, create a per-source Source Foundry record that binds at minimum:

- exact repository/artifact and commit/tree/version;
- permission/license evidence and scope;
- notices and redistribution/modification obligations;
- selected files/crates/features;
- dependency/transitive/generated/native closure;
- unsafe/FFI/process/network/telemetry/secrets behavior;
- platform posture;
- reuse strategy;
- independent Golam verification.

Until that record reaches `ADMITTED`, the source remains architectural/research evidence only.
