# Golam Spec 001 — Additive Program Amendments

Spec 001 is a frozen program foundation. Founder-directed scope or architecture refinements after that freeze are recorded here rather than silently rewriting the original historical design record.

An amendment is **planning authority for future owning specs only** unless it explicitly states otherwise. It does not authorize implementation in an earlier/current bounded Spec Kit feature.

## Mandatory read order by owning spec

### Before Spec 003 sandbox/provider closure

1. canonical active Spec 003 package and closed predecessors;
2. `PA-002A-openclaw-opensandbox-source-foundry.md` as additional research evidence only;
3. existing Spec 003 identity/policy/secrets/sandbox contracts remain authoritative.

PA-002A does **not** authorize changing the active Spec 003 task order or adding OpenSandbox/OpenClaw dependencies. It only requires the owning spec to re-pin and consider the new sandbox/secret-broker evidence before final provider architecture is frozen where still legitimately possible.

### Before Spec 004 — Harness & Local Intelligence

1. canonical Spec 001 foundation and contracts;
2. canonical closed predecessors;
3. `PA-002-memory-retrieval-learning-evals.md`;
4. `PA-002-source-foundry-research.md`;
5. `PA-002A-openclaw-opensandbox-source-foundry.md` for provider/runtime boundary evidence;
6. `../contracts/behavior-evaluation-contract.md`.

### Before Spec 005 — Local Tools, Context & Memory

1. canonical Spec 001 foundation and contracts;
2. canonical closed predecessors;
3. `PA-002-memory-retrieval-learning-evals.md`;
4. `PA-002-source-foundry-research.md`;
5. `PA-002A-openclaw-opensandbox-source-foundry.md`;
6. `../contracts/memory-retrieval-learning-contract.md`;
7. `../contracts/behavior-evaluation-contract.md`.

### Before Spec 007 — Phone, GolamConnect & Channel Access

1. canonical Spec 001 foundation and contracts;
2. canonical closed predecessors;
3. `PA-001-phone-channel-access.md`;
4. `PA-001-provider-research.md`;
5. `../contracts/phone-channel-access-contract.md`;
6. `PA-002A-openclaw-opensandbox-source-foundry.md` for OpenClaw Gateway/node/channel evidence;
7. PA-002 Hermes/channel-related evidence only where the owning Spec 007 research requalifies it.

### Before Spec 008 — Workers, Durable Graphs, Learning & Automations

1. canonical Spec 001 foundation and contracts;
2. canonical closed predecessors;
3. `PA-002-memory-retrieval-learning-evals.md`;
4. `PA-002-source-foundry-research.md`;
5. `PA-002A-openclaw-opensandbox-source-foundry.md`;
6. `../contracts/memory-retrieval-learning-contract.md`;
7. `../contracts/behavior-evaluation-contract.md`;
8. PA-001 when workers interact with phone/channel triggers or delivery.

### Before Spec 009 — Grok Public Feature Parity

Read PA-001, PA-002, and PA-002A because they materially strengthen phone continuity, governed cross-session memory, proactive learning, durable workers, sandboxed execution, multi-channel/companion-device UX, deep research/evidence and parity-superset requirements.

### Before Spec 010 — GolamBench & Release Qualification

Read all amendments and additive contracts because release qualification must cover phone/channel security plus PA-002 trajectory behavior, memory/retrieval, learning/experiment evidence, and PA-002A sandbox-provider/credential-broker/snapshot-resume qualification.

## Amendment register

| Amendment | Status | Scope | Does not do |
|---|---|---|---|
| `PA-001-phone-channel-access.md` | proposed in its planning PR until merged | native iOS/Android Golam Mobile, voice, push, official messaging channels, future Spec 007 | does not authorize current Spec 003 implementation; channels do not become authority |
| `PA-002-memory-retrieval-learning-evals.md` | proposed in its stacked planning PR until merged | memory candidates, retrieval/context, harness seams, worker graphs, learning, autonomous experiments, trajectory evaluation, future Specs 004/005/008/009/010 | does not admit listed donors/dependencies; does not authorize current Spec 003 implementation; does not make plugins/frameworks authority |
| `PA-002A-openclaw-opensandbox-source-foundry.md` | proposed in the same stacked planning PR until merged | OpenClaw product/memory/channel/security evidence; OpenSandbox sandbox protocol/runtime/egress/credential-broker evidence for future Specs 003/005/007/008/009/010 | does not admit either dependency; does not make sandbox/provider state authority; does not change active Spec 003 task order |

## Source Foundry rule

Amendment research may upgrade a source from `UNVERIFIED_REFERENCE` to an exact `VERIFIED_SNAPSHOT` or `PARTIALLY_VERIFIED` planning state. It still cannot mark code `ADMITTED`.

Every owning implementation spec must re-pin the exact source state it proposes to use and close permission/license/dependency/security/platform/benchmark/conformance gates before copying, porting, vendoring or adding a dependency.

## Conflict rule

If an additive amendment conflicts with a constitutional invariant, the constitution wins unless separately amended through its governance process.

If live canonical repository truth has evolved beyond an amendment, live canonical truth and later reviewed amendments/specs win. Never use an old amendment to override newer protected state or exact repository evidence.