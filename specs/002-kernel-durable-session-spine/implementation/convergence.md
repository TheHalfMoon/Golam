# Interim Spec Kit Convergence — Spec 002

**Purpose**: keep the implementation branch self-contained so the plan, contracts, task state, exact evidence, review state, open divergences and next execution order are recoverable from the repository without chat history.  
**Reconciled against proven code head**: `29be235de00d853a205ae2f46add1d08b91c1796`  
**Proven tree**: `4915eb0ee62324ff5faf25184d33c5a13680e9b4`  
**Exact-head CI**: GitHub Actions `32800522051` / run `40` — Windows/macOS/Ubuntu PASS.  
**This is an interim convergence record**: final T002-077 remains required after all phases.

## 1. Authority order

Implementation continues to reconcile against `.specify/memory/constitution.md`, the full Spec 002 specification/research/plan/data-model/contracts/quickstart/checklists/donor package, `tasks.md`, and all files under `implementation/`.

`Constitution -> spec + clarification + contracts -> plan + data-model -> tasks -> implementation evidence`.

Implementation evidence may reveal a needed plan amendment, but it must not silently weaken a constitutional/spec/contract requirement.

## 2. Implementation-to-plan mapping

| Area | Current implementation | State |
|---|---|---|
| Seven-package Rust spine | bounded Rust workspace | PASS |
| Canonical explicit encoding | domain-separated Golam-owned bytes | PASS |
| SQLite authority schema | canonical operational tables | PASS |
| Global/per-session sequencing | transactional + hash-chain verified | PASS |
| Artifacts/checkpoints/forks/goals | durable/rebuildable semantics | PASS |
| IPC frame codec | bounded deterministic `GIPC` | PASS |
| Authenticated lifecycle | strict Ed25519 canonical transcript | PASS |
| Unix/macOS transport | private UDS + kernel peer credentials | PASS |
| Windows transport | current-user protected DACL + local-only named pipe + peer PID/session | PASS |
| Cross-platform directory privacy | Unix modes + Windows protected DACL | PASS |
| Dedicated authority-state subtree / generic-tool exclusion | not yet converged | PARTIAL |
| Persistent effect execution | initial vocabulary only | PENDING |
| Recovery-only serving mode | integrity fail-close exists; mode pending | PARTIAL |

## 3. Reconciled security details

### Authentication and transport identity

T002-031 cryptographic client authentication remains mandatory independently of T002-032/T002-033 OS transport identity. Same UID, current Windows SID, PID/session metadata, pipe name, or local transport connection alone never grants authority.

### Unix transport

Synchronous std UDS is confined to a user-private runtime directory and explicit platform path bounds. Exact-pinned target-Unix `nix` is used only for safe kernel credential queries.

### Windows protected directories

T002-033 closes the prior Windows ACL gap. Each protected RuntimeLayout directory receives a protected current-user DACL and is re-read/verified through the OS before `UserOnlyVerified` is returned. This was exercised on the exact Windows CI runner.

### Windows named pipe

The pipe uses a protected current-user DACL, local-only mode, non-inheritable handles and a bounded instance count. `interprocess` provides the safe named-pipe/security-descriptor and peer metadata surface; `windows-permissions` provides the narrow SID/DACL path-protection surface. Their internal Win32 unsafe boundaries are dependency boundaries and remain documented; Golam crates retain `unsafe_code = forbid`.

The SID included in the pipe name is only a per-user discovery namespace. It is not a security mechanism.

## 4. Material open convergence items

### C-001 — Dedicated authority-state boundary

**Related**: Constitution II/III, FR-004, T002-021, T002-042.  
**State**: OPEN / MATERIAL.

Cross-platform path permissions are now proven. The remaining issue is architectural: the plan names a dedicated protected authority-state subtree, while current SQLite authority paths remain caller-selected. T002-042 must make authority state inaccessible to future generic filesystem/storage helpers, or the plan must be formally amended with an equally strong tested boundary. This is why T002-021 remains PARTIAL even though Windows ACL enforcement now passes.

### C-002 — Explicit recovery-only/quarantine mode

**Related**: US5, FR-019/FR-021, T002-024/T002-060.  
**State**: OPEN / MATERIAL.

Startup integrity checks fail closed and never reset canonical state, but the explicit recovery-only/quarantine operational state/report path remains pending T002-060.

## 5. Review/provenance convergence

No donor source code has been copied, ported or vendored into Spec 002.

Official GitHub Codex Code Review was requested on PR #3 with explicit Rust IPC/security/platform instructions. Codex returned `usage limits reached`, so there is **no Codex finding set and no Codex PASS**.

A CodeRabbit manual review request returned `Action not completed — Head commit changed`; it is therefore not counted as a review. The next reviewer action is to re-trigger CodeRabbit on this stable post-T002-033 closeout head and resolve any findings before treating the external-review layer as clean.

## 6. Frozen remaining execution order

### Phase D
`T002-034 -> T002-035 -> T002-036`

### Phase E
`T002-040 -> T002-041 -> T002-042 -> T002-043 -> T002-044`

### Phase F
`T002-050 -> T002-051 -> T002-052 -> T002-053 -> T002-054 -> T002-055 -> T002-056`

### Phase G
`T002-060 -> T002-061 -> T002-062 -> T002-063`

### Phase H
`T002-070 -> T002-071 -> T002-072 -> T002-073 -> T002-074 -> T002-075 -> T002-076 -> T002-077 -> T002-078`

No task is PASS from design or a lower-level primitive alone; exact-head evidence is required.

## 7. Scope outside Spec 002

Models/providers, broad harness intelligence, arbitrary product tools, Desktop/computer control, GolamConnect, remote channels, real external effects and Spec 003 remain outside authority.

## 8. Next safe action

Proceed with **T002-034 only**: explicit local client enrollment/revocation, server-side public-key lookup/revocation state, and a qualified local private-key storage backend/fallback. Private key material must not enter canonical SQLite or model-visible history. Peer OS identity remains independent from cryptographic enrollment.
