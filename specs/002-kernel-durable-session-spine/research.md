# Research — Spec 002 Kernel & Durable Session Spine

**Date**: 2026-08-24  
**Decision**: RESEARCH_COMPLETE_FOR_PLAN

## 1. Research objective

Find the minimum durable/authenticated Rust spine by comparing:

- working Grok Bot 0.18 reconstruction in `TheHalfMoon/Golam-research`;
- current Rust `xai-org/grok-build`;
- DeepSeek Harness session/runtime architecture;
- current Rust Goose;
- Spec 001/GLM security findings.

The goal is not framework stacking. Golam owns its contracts.

## 2. Exact source snapshots

### Golam-Research / Grok Bot 0.18

- repository: `TheHalfMoon/Golam-research`
- head: `a9f633e09d49a85829b8236331b9e21f7e612634`
- tree: `b68f24972427952c4934e4364736fec62661044f`
- founder permission posture: `FOUNDER_PERMISSION_ATTESTED`
- classification for Spec 002: `HIGH_VALUE_IMPLEMENTATION_EVIDENCE / AUTHORIZED_SOURCE_CANDIDATE`

Relevant observed mechanisms:

1. `source/shared/rpc/coordinator-port.ts` defines a versioned lifecycle/request/cancel/reply/event protocol, explicit parser rejection and protocol-error shutdown.
2. `source/node-agent-coordinator/control-port-client.ts` tracks request IDs/pending calls, rejects pending work on settlement, validates one `ready`, and closes on protocol breach.
3. `source/node-agent-coordinator/gateway/gateway-event-families.ts` has explicit event-family→channel mapping for transcript/tools/agents/workflows/subagents/async tasks/automation/computer action/etc.
4. `source/node-agent-coordinator/gateway/host-supervisor.ts` uses health epochs, retires stale connection attempts and prevents superseded async results from winning.
5. `source/electron-main/adapters/ipc.ts` registers IPC surfaces in an explicit order, rejects duplicate channels and rolls back registrations on partial failure.
6. `source/electron-main/adapters/main-rpc.ts` builds an explicit RPC dependency graph and fails fast on missing ports.
7. `source/shared/client-persistence-store.ts` bounds per-key/total storage and uses temp-file→rename mutation.
8. `source/host/agent-isolation/conversation-blob-db.ts` uses SQLite WAL, health checks, corruption quarantine, replacement files and resumable recovery markers for non-authority conversation blobs.
9. `source/electron-main/box/box-recovery.ts` separates recovery commands, migration state relay, restart and lifecycle disposal.
10. `source/host/gateway-protocol.ts` exposes a very broad product command surface: transcripts, agents, groups, memory, automations, workflows, skills, channels, subagents, async tasks, forever-box lifecycle, settings/secrets/MCP, attachments and computer-related state.

### What Golam should take

- versioned typed IPC lifecycle and explicit breach semantics;
- request IDs/cancellation/pending-call settlement;
- explicit event-family catalog;
- lifecycle supervisors that invalidate stale async attempts;
- ordered registration + rollback;
- atomic bounded file writes;
- corruption quarantine/recovery markers for non-authority stores;
- explicit dependency joins instead of ambient globals.

### What Golam must improve

- authentication is a first-class kernel requirement, not merely trusted Electron sender/loopback assumptions;
- authority state cannot use best-effort salvage that may drop policy/effect/audit rows;
- `any`/JSON command dispatch becomes typed Rust enums/versioned contracts;
- product command breadth does not belong in Spec 002;
- cloud/box defaults do not define Golam's local-first architecture;
- renderer/Electron IPC is not the Golam trust root.

## 3. xAI grok-build

- repository: `xai-org/grok-build`
- head: `07b2f7144fd5c5c9d3dd1966937a87852d2dbdb8`
- tree: `4251ed602dfcc5c739711d105493b042f57bd893`
- source revision reported by sync commit: `956313d459bee15ae8f17bf73e0633605e18dddd`
- language direction: Rust-heavy harness/tooling

Current head changelog reinforces:
- stale-instance supersession;
- non-blocking startup and deferred transcript replay under fan-out;
- permission rules that are trust-gated;
- typed sandbox quota denials;
- scheduler tools and workflows;
- retry/backoff under rate limiting;
- compaction instrumentation and honest failure reporting.

Spec 002 lesson: concurrency/staleness and lifecycle states must be explicit from day one. Do not let old async completions mutate new authority/session state.

## 4. DeepSeek Harness

- repository: `deepseek-ai/deepseek-harness`
- head: `b150a551b8d465e31e418e1b2eaf5e79bbb7d28e`
- tree: `53915efe4e2126cc7779b73dfc8a3bcec5318c44`
- release line: `dsh@0.1.1-rc.2`

Relevant architecture retained from Spec 001 review:
- canonical session/log orientation;
- typed events/services;
- replay/fork/resume semantics;
- model-visible state tied to logged state;
- replaceable execution services.

Golam difference: the privileged kernel is not a pluggable service and authority cannot be delegated to a plugin framework.

## 5. Goose

- repository: `aaif-goose/goose`
- head: `3a65236210d7231923059ff1f954fd6a2d67591d`
- tree: `ef02f7fa2f1254a14b5b195d2635bd780c417ae5`
- current head specifically fixes compaction failure messaging and fast-fail behavior when required tool responses do not exist.

Spec 002 lesson: corrupted/incomplete projected state should fail honestly and early rather than pretending replay/compaction succeeded.

## 6. Storage decision

Spec 001 fixes SQLite as canonical operational state. Spec 002 uses a protected SQLite DB for authority/session/effect state and a content-addressed artifact directory.

Key adaptation from Golam-Research:
- use WAL/checkpoint/recovery patterns;
- preserve atomic rename patterns for artifact writes;
- **do not** automatically salvage authority rows and continue after corruption.

Authority corruption -> quarantine + recovery-only mode.

## 7. IPC decision

Grok Bot's coordinator protocol demonstrates the value of:

`hello -> ready -> request/cancel/reply/event -> shutdown`

Golam strengthens it to:

`hello -> challenge -> authenticate -> ready -> request/cancel/reply/event -> shutdown`

with protocol version, client ID/key, nonce binding, bounded frame sizes and OS peer identity.

No loopback HTTP control plane in Spec 002.

## 8. Effect durability decision

The GLM review correctly identified `UNKNOWN_OUTCOME` reconciliation as load-bearing. Spec 002 therefore tests effect handlers against deterministic simulators with controllable crash points:

- before durable intent;
- after durable intent/before dispatch;
- remote accept before local ack;
- local ack before success commit;
- during reconcile;
- during checkpoint/daemon restart.

## 9. Research stop rule

Spec 002 source discovery is complete. Additional research is justified only if an implementation dependency choice is unresolved, a selected donor fails admission, or a test exposes a semantic gap.
