# Interim Spec Kit Convergence — Spec 002

**Purpose**: keep the implementation branch self-contained so the plan, contracts, task state, exact evidence, open divergences and next execution order are recoverable from the repository without relying on chat history.  
**Reconciled against proven code head**: `4e189db48cd0d5d2ddd3ec2679ac72e6fb253a97`  
**Exact-head CI**: GitHub Actions `32797277680` / run `26` — Windows/macOS/Ubuntu PASS.  
**This is an interim convergence record**: final T002-077 remains required after all implementation phases.

## 1. Canonical Spec Kit package

The implementation must continue to reconcile against all of the following repository-owned inputs:

1. `.specify/memory/constitution.md`
2. `specs/002-kernel-durable-session-spine/spec.md`
3. `specs/002-kernel-durable-session-spine/clarification-closeout.md`
4. `specs/002-kernel-durable-session-spine/research.md`
5. `specs/002-kernel-durable-session-spine/plan.md`
6. `specs/002-kernel-durable-session-spine/data-model.md`
7. `specs/002-kernel-durable-session-spine/contracts/event-ledger-contract.md`
8. `specs/002-kernel-durable-session-spine/contracts/storage-recovery-contract.md`
9. `specs/002-kernel-durable-session-spine/contracts/local-ipc-contract.md`
10. `specs/002-kernel-durable-session-spine/contracts/kernel-api-contract.md`
11. `specs/002-kernel-durable-session-spine/contracts/bootstrap-authorization-contract.md`
12. `specs/002-kernel-durable-session-spine/contracts/effect-handler-contract.md`
13. `specs/002-kernel-durable-session-spine/quickstart.md`
14. `specs/002-kernel-durable-session-spine/checklists/implementation-readiness.md`
15. `specs/002-kernel-durable-session-spine/donor-qualification.md`
16. `specs/002-kernel-durable-session-spine/tasks.md`
17. `specs/002-kernel-durable-session-spine/implementation/dependency-qualification.md`
18. `specs/002-kernel-durable-session-spine/implementation/source-foundry/golam-research-semantics-map.md`
19. `specs/002-kernel-durable-session-spine/implementation/status.md`
20. this convergence record.

Authority order for implementation decisions is:

`Constitution -> spec + clarification + contracts -> plan + data-model -> tasks -> implementation evidence`.

Implementation evidence may reveal a needed plan amendment, but it must not silently weaken a constitutional/spec/contract requirement.

## 2. Completed implementation-to-plan mapping

| Plan/spec area | Current implementation | State |
|---|---|---|
| Seven-package Rust spine | `golam-core`, `golam-ledger`, `golam-effects`, `golam-ipc`, `golam-kernel`, `golamd`, `golam` | PASS |
| Canonical explicit encoding | big-endian fixed-width ints + bounded length-prefixed bytes + domain separation | PASS |
| SQLite authority schema | sessions/events/goals/checkpoints/artifacts/effects/clients/authorization/audit/recovery tables | PASS |
| Global/per-session sequencing | transactional assignment, stale-head rejection, hash-chain verification | PASS |
| Content-addressed artifacts | BLAKE3, temp write/sync, verify, atomic install, cleanup | PASS |
| Checkpoints | canonical prefix binding + verified artifact + replay fallback | PASS |
| Session forks | immutable parent anchor + independent child suffix + verifier | PASS |
| Goal ledger | append-versioned + atomic event link + append-only guard + verifier | PASS |
| IPC frame codec | bounded deterministic `GIPC` binary framing/parser | PASS |
| Authenticated IPC lifecycle/transports | not implemented yet | PENDING |
| Persistent effect execution semantics | only initial vocabulary exists | PENDING |
| Explicit recovery-only serving mode | integrity fail-close exists; mode/quarantine pending | PARTIAL |

## 3. Reconciled implementation details

### Workspace path illustration

The planning document illustrates `apps/golamd` and `apps/golam`. The actual seven-package workspace keeps both binaries under `crates/golamd` and `crates/golam`. This is a **non-material repository-layout divergence**: crate count, dependency direction and security boundaries are unchanged. Final T002-077 should either update the illustration or retain this record as the accepted path mapping.

### SQLite names

The plan describes semantic table families, while schema v1 uses concrete names:

- `clients`
- `sessions`
- `session_events`
- `goal_versions`
- `artifacts`
- `checkpoints`
- `effect_intents`
- `effect_transitions`
- `effect_attempts`
- `authorization_decisions`
- `audit_chain_heads`
- `recovery_incidents`

Schema versioning currently uses SQLite `PRAGMA user_version` with forward-only migration/refusal rather than a dedicated `schema_migrations` table. This is currently treated as a **non-material implementation choice** because version refusal/migration authority remains explicit. Reassess at final convergence if later migrations need a richer journal.

### Checkpoint authority

Checkpoint projection bytes and checkpoint metadata are accelerators, not alternate authority. The canonical `CheckpointCreated` event binds the checkpoint id, session prefix, prefix hash and artifact hash. If accelerator metadata/artifact is missing or invalid, reconstruction uses canonical replay. This preserves the plan/constitution rule that full canonical history outranks compaction/checkpoint state.

## 4. Material open convergence items

### C-001 — Windows authority path protection

**Related**: Constitution II/III, FR-004, T002-021, T002-033.  
**State**: OPEN / MATERIAL.

Unix/macOS private directory enforcement is proven. Windows currently has path isolation only and `require_authority_ready()` fails closed. T002-033 must establish and verify current-user SID ACL behavior; the task cannot be closed merely because a path is under the user's profile.

### C-002 — Planned authority directory boundary

**Related**: plan `Storage layout`, FR-004, T002-021/T002-042.  
**State**: OPEN / MATERIAL.

The plan calls out a protected authority-state subtree. Current `RuntimeLayout` creates root/data/runtime/artifact paths, while the SQLite authority path remains caller-selected. Before protected-resource APIs are considered complete, the implementation must either conform to the planned authority subtree or formally amend the plan with an equally strong non-generic boundary and tests.

### C-003 — Explicit recovery-only/quarantine mode

**Related**: US5, FR-019/FR-021, T002-024/T002-060.  
**State**: OPEN / MATERIAL.

Current startup integrity checks return failure and never reset corrupt canonical state. The explicit recovery-only/quarantine operational state/report path is still pending and must be implemented in T002-060.

## 5. Source/provenance convergence

No source code from Golam-Research or another donor has been copied, ported or vendored into the Spec 002 implementation to date. Current donor use is semantics/behavior mapping only. Therefore no per-file code-admission record is required yet.

If implementation later reuses donor source, the sequence is mandatory before reuse:

`exact source -> exact commit/tree -> permission/license scope -> notices/dependency closure -> unsafe/FFI/network/security review -> selected files -> reuse mode -> tests -> Source Foundry ADMITTED`.

## 6. Frozen remaining execution order

The remaining implementation plan is not kept in chat; it is the repository task graph below:

### Phase D
`T002-031 -> T002-032 -> T002-033 -> T002-034 -> T002-035 -> T002-036`

### Phase E
`T002-040 -> T002-041 -> T002-042 -> T002-043 -> T002-044`

### Phase F
`T002-050 -> T002-051 -> T002-052 -> T002-053 -> T002-054 -> T002-055 -> T002-056`

### Phase G
`T002-060 -> T002-061 -> T002-062 -> T002-063`

### Phase H
`T002-070 -> T002-071 -> T002-072 -> T002-073 -> T002-074 -> T002-075 -> T002-076 -> T002-077 -> T002-078`

No task may be marked PASS merely because a lower-level primitive exists. Exact-head evidence is required for gate claims.

## 7. Scope that remains explicitly outside Spec 002

- model inference / model weights / model providers;
- harness/context/memory intelligence beyond this durable session spine;
- arbitrary filesystem/shell/git/browser product tools;
- Desktop/computer control;
- GolamConnect/channel adapters/remote control;
- real external effects such as email, deployments, payments or production mutations;
- starting Spec 003 before Spec 002 is reviewed, merged and CLOSED_CANONICAL.

## 8. Next safe action

Proceed with **T002-031 only**: authenticated lifecycle state machine and transcript binding/server epoch. Keep transport-specific Unix socket and Windows named-pipe behavior in T002-032/T002-033 so each security claim can be independently proven.
