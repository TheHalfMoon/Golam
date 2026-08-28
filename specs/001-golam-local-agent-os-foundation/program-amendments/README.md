# Golam Spec 001 — Additive Program Amendments

Spec 001 is a frozen program foundation. Founder-directed scope or architecture refinements after that freeze are recorded here rather than silently rewriting the original historical design record.

An amendment is **planning authority for future owning specs only** unless it explicitly states otherwise. It does not authorize implementation in an earlier/current bounded Spec Kit feature.

## Mandatory read order by owning spec

### Before Spec 003 sandbox/provider closure

1. canonical active Spec 003 package and closed predecessors;
2. `PA-002A-openclaw-opensandbox-source-foundry.md` as additional research evidence only;
3. existing Spec 003 identity/policy/secrets/sandbox contracts remain authoritative.

PA-002A does **not** authorize changing the active Spec 003 task order or adding OpenSandbox/OpenClaw dependencies. It only requires the owning spec to re-pin and consider the new sandbox/secret-broker evidence before final provider architecture is frozen where still legitimately possible.

PA-003/PA-003A do not alter active Spec 003 implementation scope. Their product/topology sequencing begins to bind future Specs 004+ and adds a Core Alpha gate only after Spec 005 closes.

### Before Spec 004 — Harness & Local Intelligence

1. canonical Spec 001 foundation and contracts;
2. canonical closed predecessors;
3. `PA-002-memory-retrieval-learning-evals.md`;
4. `PA-002-source-foundry-research.md`;
5. `PA-002A-openclaw-opensandbox-source-foundry.md` for provider/runtime boundary evidence;
6. `PA-003-product-spine-golden-loop.md`;
7. `PA-003-task-delta.md`;
8. `../contracts/task-run-trust-contract.md`;
9. `../contracts/behavior-evaluation-contract.md`.

Spec 004 must preserve the PA-003 distinction `TASK != SESSION != RUN != WORKER` and expose runtime-level pause/stop/steer/inspect/resume semantics sufficient for the later Core Alpha gate.

### Before Spec 005 — Local Tools, Context & Memory

1. canonical Spec 001 foundation and contracts;
2. canonical closed predecessors;
3. `PA-002-memory-retrieval-learning-evals.md`;
4. `PA-002-source-foundry-research.md`;
5. `PA-002A-openclaw-opensandbox-source-foundry.md`;
6. `PA-003-product-spine-golden-loop.md`;
7. `PA-003-task-delta.md`;
8. `../contracts/task-run-trust-contract.md`;
9. `../contracts/memory-retrieval-learning-contract.md`;
10. `../contracts/behavior-evaluation-contract.md`.

Spec 005 owns the PA-003 **Golam Core Alpha** product gate: useful CLI/TUI Golden Loop, Trust Receipt projection, UserModel baseline, portability/import staging, recovery/steering, and strict-local end-to-end proof before Desktop becomes the next product blocker.

### Before Spec 006 — Desktop & Computer Control

1. canonical Spec 001 foundation and closed predecessors;
2. `PA-003-product-spine-golden-loop.md`;
3. `PA-003-task-delta.md`;
4. `../contracts/task-run-trust-contract.md`;
5. PA-002/PA-002A where Desktop surfaces memory, providers, sandbox/tool evidence, or security posture.

Desktop must remain a client/projection of the same durable Task/Run/Trust semantics rather than creating a separate desktop-only agent state.

### Before Spec 007 — Phone, GolamConnect & Channel Access

1. canonical Spec 001 foundation and contracts;
2. canonical closed predecessors;
3. `PA-001-phone-channel-access.md`;
4. `PA-001-provider-research.md`;
5. `PA-003-product-spine-golden-loop.md`;
6. `PA-003A-always-on-host-topology.md`;
7. `PA-003-task-delta.md`;
8. `../contracts/task-run-trust-contract.md`;
9. `../contracts/phone-channel-access-contract.md`;
10. `PA-002A-openclaw-opensandbox-source-foundry.md` for OpenClaw Gateway/node/channel evidence;
11. PA-002 Hermes/channel-related evidence only where the owning Spec 007 research requalifies it.

Native Mobile/channels project the same durable task identity; channel/device identity never becomes task or authority identity. PA-003A additionally requires an optional user-owned Authority Host / Execution Node topology so phone continuity and schedules do not require a sleeping work laptop or vendor cloud.

### Before Spec 008 — Workers, Durable Graphs, Learning & Automations

1. canonical Spec 001 foundation and contracts;
2. canonical closed predecessors;
3. `PA-002-memory-retrieval-learning-evals.md`;
4. `PA-002-source-foundry-research.md`;
5. `PA-002A-openclaw-opensandbox-source-foundry.md`;
6. `PA-003-product-spine-golden-loop.md`;
7. `PA-003A-always-on-host-topology.md`;
8. `PA-003-task-delta.md`;
9. `../contracts/task-run-trust-contract.md`;
10. `../contracts/memory-retrieval-learning-contract.md`;
11. `../contracts/behavior-evaluation-contract.md`;
12. PA-001 when workers interact with phone/channel triggers or delivery.

Spec 008 must distinguish initiative authority from attention authority before proactive workers/routines are considered product-complete, and worker placement on a remote/node/sandbox must never transfer kernel authority.

### Before Spec 009 — Grok Public Feature Parity

Read PA-001, PA-002, PA-002A, PA-003, PA-003A, PA-003 task delta, and the task/run/trust contract because they materially strengthen phone continuity, governed cross-session memory, proactive learning, durable workers, sandboxed execution, user-owned always-on topology, multi-channel/companion-device UX, Trust Receipts, locality controls, in-flight user control, deep research/evidence and parity-superset requirements.

### Before Spec 010 — GolamBench & Release Qualification

Read all amendments and additive contracts because release qualification must cover phone/channel security; PA-002 trajectory behavior, memory/retrieval, learning/experiment evidence; PA-002A sandbox-provider/credential-broker/snapshot-resume qualification; PA-003 Golden Loop, hybrid-interface/process verification, false-success, steering/recovery, Trust Receipt, capability-truth and staged product-release claims; and PA-003A Authority Host/node offline/migration/fencing/locality claims where implemented.

## Amendment register

| Amendment | Status | Scope | Does not do |
|---|---|---|---|
| `PA-001-phone-channel-access.md` | proposed in its planning PR until merged | native iOS/Android Golam Mobile, voice, push, official messaging channels, future Spec 007 | does not authorize current Spec 003 implementation; channels do not become authority |
| `PA-002-memory-retrieval-learning-evals.md` | proposed in its stacked planning PR until merged | memory candidates, retrieval/context, harness seams, worker graphs, learning, autonomous experiments, trajectory evaluation, future Specs 004/005/008/009/010 | does not admit listed donors/dependencies; does not authorize current Spec 003 implementation; does not make plugins/frameworks authority |
| `PA-002A-openclaw-opensandbox-source-foundry.md` | proposed in the same stacked planning PR until merged | OpenClaw product/memory/channel/security evidence; OpenSandbox sandbox protocol/runtime/egress/credential-broker evidence for future Specs 003/005/007/008/009/010 | does not admit either dependency; does not make sandbox/provider state authority; does not change active Spec 003 task order |
| `PA-003-product-spine-golden-loop.md` | proposed in a stacked planning PR until merged | durable Task/Run product spine, Task Contract, Golden Loop, Trust Receipts, progressive autonomy UX, in-flight control, Initiative/Attention semantics, UserModel, portability, capability truth, Core Alpha and staged release ladder | does not weaken kernel authority; does not reorder active Spec 003; does not make UI/autonomy settings authority; does not require deferred breadth before Core Alpha |
| `PA-003A-always-on-host-topology.md` | proposed with PA-003 until merged | optional user-owned Authority Host, paired Execution Nodes, offline-node/wait semantics, protected host migration, data/secret placement, always-on phone/routine continuity without vendor-cloud trust root | does not authorize multi-master authority, automatic cloud failover, CRDT protected-state sync, or copying all node data/secrets to the host |

`PA-003-task-delta.md` is the execution-oriented companion to PA-003. `../contracts/task-run-trust-contract.md` is its normative task/run/control/receipt contract. Neither changes current implementation authority by itself.

## Source Foundry rule

Amendment research may upgrade a source from `UNVERIFIED_REFERENCE` to an exact `VERIFIED_SNAPSHOT` or `PARTIALLY_VERIFIED` planning state. It still cannot mark code `ADMITTED`.

Every owning implementation spec must re-pin the exact source state it proposes to use and close permission/license/dependency/security/platform/benchmark/conformance gates before copying, porting, vendoring or adding a dependency.

## Product sequencing rule

After PA-003, product usefulness is an explicit gate rather than an implied by-product of architecture completion.

Specs MUST preserve future extensibility, but no deferred subsystem may become a blocker for the **Golam Core Alpha** checkpoint after Spec 005 unless the owning spec records measured evidence that the subsystem is required for the Golden Loop scenarios.

## Conflict rule

If an additive amendment conflicts with a constitutional invariant, the constitution wins unless separately amended through its governance process.

If live canonical repository truth has evolved beyond an amendment, live canonical truth and later reviewed amendments/specs win. Never use an old amendment to override newer protected state or exact repository evidence.