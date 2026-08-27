# Golam-Research Semantics Map — Spec 002 Implementation

**Source**: `TheHalfMoon/Golam-research`  
**Commit**: `a9f633e09d49a85829b8236331b9e21f7e612634`  
**Tree**: `b68f24972427952c4934e4364736fec62661044f`  
**Permission posture**: founder-attested; no source copied in this slice.  
**Reuse class in this slice**: behavior/test mapping only.

## Mapped implementation evidence

| Source path | Behavior mined | Golam Rust target | First verification |
|---|---|---|---|
| `source/shared/rpc/coordinator-port.ts` | versioned lifecycle/request/cancel/reply/event protocol; breach shutdown | `golam-ipc::FrameKind` and later authenticated lifecycle FSM | parser/lifecycle tests |
| `source/node-agent-coordinator/control-port-client.ts` | request IDs, pending-call settlement, single ready transition | `golam-ipc` request lifecycle | cancellation/pending bounds tests |
| `source/node-agent-coordinator/gateway/gateway-event-families.ts` | explicit event family catalog | `golam-ledger::EventKind` | schema/version tests |
| `source/node-agent-coordinator/gateway/host-supervisor.ts` | stale connection-attempt invalidation | later daemon supervisor | epoch/stale completion tests |
| `source/electron-main/adapters/ipc.ts` | ordered registration and rollback | later IPC service registration | partial-registration rollback test |
| `source/electron-main/adapters/main-rpc.ts` | explicit dependency joins/fail-fast missing ports | later daemon composition | missing-dependency startup test |
| `source/shared/client-persistence-store.ts` | bounded persistence + temp/rename | protected storage/artifact writes | bound + crash atomicity tests |
| `source/host/agent-isolation/conversation-blob-db.ts` | WAL/health/quarantine/recovery markers for conversation state | `golam-ledger` non-authority recovery patterns | corruption/checkpoint tests |
| `source/electron-main/box/box-recovery.ts` | explicit recovery commands/restart lifecycle | recovery-only mode | restart/recovery tests |
| `source/host/gateway-protocol.ts` | broad product protocol surface | evidence for future specs only | excluded from Spec 002 breadth |

## Deliberate Golam improvements

- lifecycle becomes `hello -> challenge -> authenticate -> ready`, not trusted-loopback/renderer authority;
- authority-bearing storage fails closed instead of best-effort row salvage;
- typed Rust enums/contracts replace string/`any` dispatch;
- local-first daemon semantics replace cloud/box defaults;
- Electron renderer trust assumptions do not enter `KernelApi`.

## Admission rule

If a later task copies or ports source text/structure rather than only independently implementing behavior, create a per-file Source Foundry admission record before the code enters Golam. Record exact permission evidence/scope, notices/obligations, dependency closure, modifications and independent Rust tests.
