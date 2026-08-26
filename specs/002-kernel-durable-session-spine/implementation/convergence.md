# Final Spec Kit Convergence — Spec 002

**Implementation branch**: `impl/002-kernel-durable-session-spine`  
**PR**: #3 — OPEN / DRAFT  
**Canonical base**: `main@cfcc90f452e7115bfb104f886e09c309a5d57a1c`  
**Last fully qualified code head before documentation reconciliation**: `29d54ca211e17c7bbcc0b2febfc2349d7b9ed2be`  
**Exact code-head CI**: GitHub Actions run ID `32958260286`, run number `225` — Windows/macOS/Ubuntu SUCCESS for the full qualification workflow.  
**State**: `CONVERGED_FOR_FINAL_EXACT_HEAD_QUALIFICATION`.

This record supersedes the earlier interim convergence snapshot. Live GitHub truth still overrides every recorded SHA if the branch moves.

## 1. Authority order re-read

Final convergence was checked against:

1. live GitHub branch/base/PR state;
2. `.specify/memory/constitution.md` v1.2.0;
3. frozen Spec 001 architecture and GolamBench foundation gates;
4. Spec 002 `spec.md`, clarification closeout, research, donor qualification, plan, data model, contracts, quickstart, implementation-readiness checklist, tasks and analysis;
5. implementation evidence under `specs/002-kernel-durable-session-spine/implementation/`;
6. exact Rust/CI behavior on the implementation branch.

No founder waiver is taken.

## 2. Constitution / Spec 002 convergence

| Area | Final implementation position | State |
|---|---|---|
| Rust trusted path / seven-package spine | Rust 1.98, Golam crates forbid unsafe, no future crate scaffolding | PASS |
| Small privileged authority boundary | protected authority subtree, sealed KernelApi grants, hostile-adapter tests | PASS |
| Local ownership / no cloud prerequisite | local SQLite/artifacts/credentials, no model/provider dependency | PASS |
| Authenticated local IPC | UDS or Windows named pipe + OS peer checks + Ed25519 challenge authentication | PASS |
| IPC boundedness | frame/pending limits, platform listener bounds, absolute accepted-connection deadline | PASS |
| Session/fork/goal/checkpoint durability | transactional canonical order, replay/checkpoint equivalence, immutable forks | PASS |
| Effect durability | durable intent/attempt before dispatch proof, CAS state, deterministic handlers | PASS |
| Ambiguous effects | UNKNOWN_OUTCOME blocks dependencies; no blind duplicate; reconciliation/manual review | PASS |
| Recovery | explicit Normal/RecoveryOnly/Quarantined scan; no silent authority reset | PASS |
| Disk pressure | real SQLite FULL regression; no dispatch authority after failed durable transaction | PASS |
| Strict-local egress | kernel hard deny + externally observed zero Golam Internet sockets | PASS |
| Security integrity | canonical event audit chain plus authority-security chain/coverage for protected non-event records | PASS |
| Source governance | no donor source code copied/ported; semantics-only evidence remains within recorded posture | PASS |
| Platform qualification | Windows, macOS, Ubuntu workflow steps explicitly exercise platform IPC/locality | PASS |

## 3. Material divergences found and resolved

### C-001 — authority-state boundary

**Resolved.** Authority data is under the protected authority subtree. Generic/unprivileged path admission rejects the authority root/database/credentials/reserved state and paths outside the admitted runtime area. Hostile-adapter tests prove the public non-kernel surface cannot mint authority or directly mutate the privileged ledger.

### C-002 — recovery-only / quarantine behavior

**Resolved.** `RecoveryScanner` distinguishes normal service, coherent attention, recovery-only conditions and canonical corruption quarantine. `KernelApi::open` cannot bypass the startup gate. `golamd` refuses privileged serving in RecoveryOnly/Quarantined state rather than silently resetting state.

### C-003 — accepted-connection deadline

**Resolved.** Final convergence found that a synchronous daemon could otherwise be held indefinitely by a silent same-user client. Accepted local streams are now nonblocking behind a bounded deadline wrapper. CI requalified Windows/macOS/Linux after the fix.

### C-004 — mandatory integrity for authorization/effect/client/recovery records

**Resolved.** The original security event chain protected canonical `SessionEvent` records but did not by itself authenticate every protected non-event authority row required by the constitution/event-ledger contract.

The implementation now adds the independent `authority-security` BLAKE3 chain. It covers:

- client enrollment and revocation;
- authorization decisions;
- effect intents;
- effect transitions;
- effect attempt start and finish;
- recovery/protocol/manual-review incidents.

Coverage and source-row hashes are verified on authority-store open. Missing audit coverage, row tampering, broken chain linkage or a mismatched chain head fails canonical integrity. Tamper integration tests exercise authorization, effect, client and recovery records.

### C-005 — enrolled CLI checkpoint/reconcile authority

**Resolved.** The daemon authenticates a CLI as `EnrolledClient`, not `LocalOwner` or `KernelService`. Final convergence found that `checkpoint.create` and `effect.reconcile` were missing from the enrolled-client bootstrap allow set even though T002-062 exposes those authenticated CLI commands and the bootstrap contract admits checkpoint/synthetic-effect operations.

`BootstrapPolicy` now explicitly permits those two actions for an enrolled local client while client enrollment/revocation and network egress remain denied. Regression tests prove both the intended permits and the non-expansion guardrails.

### C-006 — generic `append_session_event` contract wording

**Resolved by security narrowing of the conceptual contract, not by adding a dangerous generic primitive.**

A public API accepting caller-selected `(EventKind, bytes)` would allow an adapter to claim reserved system families such as checkpoint/effect/goal lifecycle evidence without the owning protected domain record. Spec 002 therefore exposes typed domain mutations; those operations emit canonical events where required. A future general product event family must receive its own typed request under its owning spec.

`kernel-api-contract.md` and `quickstart.md` now state this explicitly.

### C-007 — stale quickstart / AGENTS / implementation status

**Resolved in closeout documentation.** The old quickstart was explicitly a target sketch and used command shapes not implemented by the bounded CLI. It now records the actual parser grammar and recovery behavior. `AGENTS.md` now records the implementation-qualification phase and preserves Draft/no-merge/no-Spec003 guardrails.

## 4. T002-061 recovery reserve decision

`implementation/recovery-reserve-evaluation.md` records the tested decision: `NO_RECOVERY_RESERVE_GUARANTEE`.

Spec 002 does not fabricate a platform guarantee that was not proven. The regression ensures startup does not create or rely on an unproven reserve; disk-full behavior instead fails closed before dispatch authority when the durable transaction cannot commit.

## 5. Phase H evidence

The exact code-head qualification run #225 exercises:

- fmt and Clippy with warnings denied;
- full workspace tests;
- deterministic property qualification;
- bounded IPC/event/migration fuzz smoke;
- Windows/macOS/Linux platform IPC qualification;
- authenticated daemon IPC and hostile/adversarial authority probes;
- externally observed strict-local no-network checks.

`implementation/bs1-bs2-qualification.md` records BS-1 and BS-2 evidence. BS-10 is represented directly by the external locality observation workflow steps.

No PASS from run #225 is automatically inherited by a later documentation/task head. The final closeout head must run the same required matrix again.

## 6. Review state

- A prior GitHub Codex review request was blocked by usage limits; there is no Codex finding set and no Codex PASS.
- A prior CodeRabbit request was not completed because the head changed; that is not a review and not a PASS.
- PR #3 remains Draft. External bot review is therefore not silently treated as a completed gate.

Review state does not authorize making the PR Ready or merging it. If repository/founder policy requires a fresh external review before Ready/merge, request it only on a stable final head and resolve material findings before lifecycle promotion.

## 7. Scope boundary remains intact

Spec 002 has not admitted:

- model inference/download/providers;
- broad filesystem/shell/browser/MCP/skills product tools;
- Desktop/computer control;
- GolamConnect/channels;
- real consequential external effects;
- external network behavior;
- Spec 003 policy/secrets/sandbox implementation.

## 8. Closeout condition

T002-077 is implementation-converged when this reconciliation package itself receives a green exact-head CI matrix and no newly introduced material divergence exists.

T002-078 then records the exact final Draft PR head/evidence. Spec 002 may be implementation-complete while PR #3 remains Draft; it is **not** `CLOSED_CANONICAL` until separately authorized lifecycle actions occur and the implementation is merged/canonicalized. Spec 003 remains unauthorized until that canonical closure.
