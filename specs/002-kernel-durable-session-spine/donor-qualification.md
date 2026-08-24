# Donor Qualification — Spec 002

**Status**: PLANNING QUALIFICATION; NO DONOR CODE ADMITTED BY THIS DOCUMENT

## Permission posture

Spec 001 records a founder attestation that permission has been obtained for the supplied/researched source universe. For any code actually reused during Spec 002 implementation, create a bounded admission record with the exact permission evidence/scope before copy/port/vendor/dependency admission.

## DQ-001 — Golam-Research / Grok Bot 0.18

- repo: `TheHalfMoon/Golam-research`
- commit: `a9f633e09d49a85829b8236331b9e21f7e612634`
- tree: `b68f24972427952c4934e4364736fec62661044f`
- status: `VERIFIED_SOURCE_STATE + FOUNDER_PERMISSION_ATTESTED`
- classification: `HIGH_VALUE_IMPLEMENTATION_EVIDENCE / SELECTIVE_PORT_CANDIDATE`

Selected evidence paths for Spec 002:
- `source/shared/rpc/coordinator-port.ts`
- `source/node-agent-coordinator/control-port-client.ts`
- `source/node-agent-coordinator/gateway/gateway-event-families.ts`
- `source/node-agent-coordinator/gateway/host-supervisor.ts`
- `source/electron-main/adapters/ipc.ts`
- `source/electron-main/adapters/main-rpc.ts`
- `source/shared/client-persistence-store.ts`
- `source/host/agent-isolation/conversation-blob-db.ts`
- `source/electron-main/box/box-recovery.ts`
- `source/host/gateway-protocol.ts`

Take/port concepts:
- protocol parser/lifecycle strictness;
- stale-attempt invalidation;
- ordered registration rollback;
- bounded atomic persistence;
- recovery intent/pending markers;
- explicit event-family and RPC dependency contracts.

Do not take unchanged:
- Electron trust assumptions;
- unauthenticated loopback assumptions;
- broad string→`any` command surface;
- cloud/box architecture as local core;
- best-effort salvage for authority state.

Admission required before code use:
- exact permission evidence scope for selected source files;
- NOTICE/attribution requirements;
- dependency/transitive-source closure if code rather than semantics is copied;
- independent Rust tests proving semantic equivalence where ported.

## DQ-002 — xai-org/grok-build

- commit: `07b2f7144fd5c5c9d3dd1966937a87852d2dbdb8`
- tree: `4251ed602dfcc5c739711d105493b042f57bd893`
- permission: founder-attested
- classification: `RUST SELECTIVE_PORT / REFERENCE`

Use for session/concurrency/lifecycle patterns only after exact file-level qualification. Do not wholesale fork.

## DQ-003 — DeepSeek Harness

- commit: `b150a551b8d465e31e418e1b2eaf5e79bbb7d28e`
- tree: `53915efe4e2126cc7779b73dfc8a3bcec5318c44`
- permission: founder-attested
- classification: `ARCHITECTURE REFERENCE / SELECTIVE PORT IF JUSTIFIED`

Take canonical log/fork/replay/event semantics. Do not adopt Cordis/plugin-everything as Golam authority architecture.

## DQ-004 — Goose

- commit: `3a65236210d7231923059ff1f954fd6a2d67591d`
- tree: `ef02f7fa2f1254a14b5b195d2635bd780c417ae5`
- permission: founder-attested
- classification: `RUST REFERENCE / SELECTIVE PORT`

Take Rust implementation lessons for session/compaction/error behavior after exact path qualification. Do not import unrelated provider/extension breadth into Spec 002.

## Admission decision for planning

```text
DONOR_CODE_ADMITTED=NO
SEMANTIC_RESEARCH_COMPLETE=YES
NEXT_ADMISSION_GATE=IMPLEMENTATION_TASK_WITH_EXACT_SELECTED_FILES_AND_PERMISSION_SCOPE
```
