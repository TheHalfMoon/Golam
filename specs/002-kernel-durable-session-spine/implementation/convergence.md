# Interim Spec Kit Convergence — Spec 002

**Purpose**: keep the implementation branch self-contained so the plan, contracts, task state, exact evidence, review state, open divergences and next execution order are recoverable from the repository without relying on chat history.  
**Reconciled against proven code head**: `e5845cfaa9ec9aa240afc92a61e0728c071722c7`  
**Exact-head CI**: GitHub Actions `32799215791` / run `36` — Windows/macOS/Ubuntu PASS.  
**This is an interim convergence record**: final T002-077 remains required after all implementation phases.

## 1. Canonical Spec Kit package

Implementation continues to reconcile against `.specify/memory/constitution.md`, the full `specs/002-kernel-durable-session-spine/` specification/research/plan/data-model/contracts/quickstart/checklists/donor package, `tasks.md`, and all files under `implementation/`.

Authority order:

`Constitution -> spec + clarification + contracts -> plan + data-model -> tasks -> implementation evidence`.

Implementation evidence may reveal a needed plan amendment, but it must not silently weaken a constitutional/spec/contract requirement.

## 2. Completed implementation-to-plan mapping

| Plan/spec area | Current implementation | State |
|---|---|---|
| Seven-package Rust spine | `golam-core`, `golam-ledger`, `golam-effects`, `golam-ipc`, `golam-kernel`, `golamd`, `golam` | PASS |
| Canonical explicit encoding | fixed-width/domain-separated Golam-owned bytes | PASS |
| SQLite authority schema | sessions/events/goals/checkpoints/artifacts/effects/clients/authorization/audit/recovery tables | PASS |
| Global/per-session sequencing | transactional assignment, stale-head rejection, hash-chain verification | PASS |
| Content-addressed artifacts | BLAKE3, temp write/sync, verify, atomic install, cleanup | PASS |
| Checkpoints | canonical prefix binding + verified artifact + replay fallback | PASS |
| Session forks | immutable parent anchor + independent child suffix + verifier | PASS |
| Goal ledger | append-versioned + atomic event link + append-only guard + verifier | PASS |
| IPC frame codec | bounded deterministic `GIPC` framing/parser | PASS |
| Authenticated IPC lifecycle | fixed wire payloads + canonical transcript + strict Ed25519 verification + fail-closed phase machine | PASS |
| Unix/macOS local transport | private UDS + peer UID/PID verification + path-bound handling | PASS |
| Windows named-pipe transport | not implemented yet | PENDING |
| Persistent effect execution semantics | only initial vocabulary exists | PENDING |
| Explicit recovery-only serving mode | integrity fail-close exists; mode/quarantine pending | PARTIAL |

## 3. Reconciled implementation details

### Workspace path illustration

The planning document illustrates `apps/golamd` and `apps/golam`. The actual seven-package workspace keeps both binaries under `crates/golamd` and `crates/golam`. This is a non-material repository-layout divergence: crate count, dependency direction and security boundaries are unchanged.

### SQLite names and schema versioning

Schema v1 uses concrete tables `clients`, `sessions`, `session_events`, `goal_versions`, `artifacts`, `checkpoints`, `effect_intents`, `effect_transitions`, `effect_attempts`, `authorization_decisions`, `audit_chain_heads`, `recovery_incidents`. Versioning uses SQLite `PRAGMA user_version` with forward-only migration/refusal rather than a dedicated migration journal. Reassess at final convergence if later migrations need a richer journal.

### Checkpoint authority

Checkpoint projection bytes and metadata are accelerators, not alternate authority. Missing/invalid accelerator state falls back to canonical replay.

### Authentication transcript

T002-031 uses Golam-owned canonical bytes, not serializer output. The transcript domain is versioned and binds the local-IPC contract-required protocol/client/nonces/server epoch plus negotiated resource limits and key ID. `ed25519-dalek` performs strict signature verification only; key generation/enrollment/storage remains T002-034.

### Unix transport

T002-032 keeps Unix transport synchronous and minimal: `std::os::unix::net` owns the listener/stream, while exact-pinned target-Unix `nix` is used only for safe kernel credential queries. The runtime directory is user-private (`0700`), the socket is `0600`, pre-existing paths fail closed without unlinking, and platform `sun_path` limits are checked explicitly before bind. Linux uses `SO_PEERCRED`; Apple uses `LOCAL_PEERCRED` plus `LOCAL_PEERPID`. Same-effective-UID is a transport identity input, not complete authorization; T002-031/T002-034 cryptographic client identity remains independently required.

## 4. Material open convergence items

### C-001 — Windows authority path protection

**Related**: Constitution II/III, FR-004, T002-021, T002-033.  
**State**: OPEN / MATERIAL.

Unix/macOS private directory enforcement is proven. Windows currently has path isolation only and `require_authority_ready()` fails closed. T002-033 must establish and verify current-user SID ACL behavior.

### C-002 — Planned authority directory boundary

**Related**: plan `Storage layout`, FR-004, T002-021/T002-042.  
**State**: OPEN / MATERIAL.

Current `RuntimeLayout` creates root/data/runtime/artifact paths while SQLite authority path remains caller-selected. Before protected-resource APIs are complete, implementation must conform to the planned authority subtree or formally amend the plan with an equally strong non-generic boundary and tests.

### C-003 — Explicit recovery-only/quarantine mode

**Related**: US5, FR-019/FR-021, T002-024/T002-060.  
**State**: OPEN / MATERIAL.

Current startup integrity checks return failure and never reset corrupt canonical state. Explicit recovery-only/quarantine operational state/report path remains pending T002-060.

## 5. Review/provenance convergence

No donor source code has been copied, ported or vendored into Spec 002. Current donor use remains semantics/behavior mapping only.

Official GitHub Codex Code Review was requested on PR #3 using `@codex review` with explicit Rust IPC/security/platform guidance. The Codex connector returned `usage limits reached`, therefore there is **no Codex finding set and no Codex PASS** to record. CodeRabbit manual review was subsequently triggered as an additional independent reviewer. External review findings must be reconciled when they exist; absence caused by quota is never treated as approval.

## 6. Frozen remaining execution order

### Phase D
`T002-033 -> T002-034 -> T002-035 -> T002-036`

### Phase E
`T002-040 -> T002-041 -> T002-042 -> T002-043 -> T002-044`

### Phase F
`T002-050 -> T002-051 -> T002-052 -> T002-053 -> T002-054 -> T002-055 -> T002-056`

### Phase G
`T002-060 -> T002-061 -> T002-062 -> T002-063`

### Phase H
`T002-070 -> T002-071 -> T002-072 -> T002-073 -> T002-074 -> T002-075 -> T002-076 -> T002-077 -> T002-078`

No task may be marked PASS merely because a lower-level primitive exists. Exact-head evidence is required.

## 7. Scope outside Spec 002

Models/model weights/providers, broader harness/context intelligence, arbitrary product tools, Desktop/computer control, GolamConnect/channel remote control, real external effects, and Spec 003 remain outside this implementation authority.

## 8. Next safe action

Proceed with **T002-033 only**: Windows named-pipe transport with a current-user SID ACL and peer metadata where safely available. It must close the Windows half of T002-021 without introducing a localhost/TCP substitute or weakening the cryptographic client-authentication requirement.
