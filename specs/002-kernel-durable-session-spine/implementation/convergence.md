# Final Spec Kit Convergence — Spec 002

**Implementation branch**: `impl/002-kernel-durable-session-spine`  
**PR**: #3 — OPEN / DRAFT  
**Canonical base**: `main@cfcc90f452e7115bfb104f886e09c309a5d57a1c`  
**Latest behavioral repair head**: `acdce817dbd89d8286fc08b8821aded0b7dbf8f7`  
**Latest formatting repair head**: `1a0ef2c1a72056e42528521ec63e5acaabc297f0`  
**Pre-reconciliation head**: `b54abec064fd16074674a7c8c141f4bf3d69a245`  
**Final candidate head**: the commit containing this convergence package.  
**State**: `REPAIR_CONVERGED_PENDING_FINAL_EXACT_HEAD_CI_AND_POST_CI_QODO`.

Live GitHub truth overrides every recorded SHA. No PASS or review result transfers across a branch mutation.

## 1. Authority order re-read

The final repair convergence was checked against:

1. live GitHub branch/base/PR state;
2. `.specify/memory/constitution.md` v1.2.0 or later;
3. frozen Spec 001 architecture and GolamBench foundation gates;
4. Spec 002 spec, clarification closeout, research, donor qualification, plan, data model, contracts, quickstart, implementation-readiness checklist, tasks and analysis;
5. implementation evidence under `specs/002-kernel-durable-session-spine/implementation/`;
6. exact Rust/CI behavior on the implementation branch;
7. the authorized Qodo repair cycle.

No founder waiver is taken.

## 2. Converged implementation position

| Area | Current implementation position | State |
|---|---|---|
| Rust trusted path / seven-package spine | Rust 1.98, Golam product crates forbid unsafe, no future crate scaffolding | IMPLEMENTED |
| Small privileged authority boundary | protected authority subtree, sealed KernelApi authority, hostile-adapter probes | IMPLEMENTED |
| Local ownership / no cloud prerequisite | local SQLite/artifacts/credentials, Unix ownership checks, no model/provider dependency | IMPLEMENTED |
| Authenticated local IPC | UDS or Windows named pipe + OS peer checks + Ed25519 challenge authentication | IMPLEMENTED |
| IPC boundedness | frame/pending limits, local-ceiling handshake negotiation, daemon/CLI absolute deadlines | IMPLEMENTED |
| Session/fork/goal/checkpoint durability | transactional canonical order, atomic checkpoint metadata/event boundary, replay/checkpoint equivalence, immutable forks | IMPLEMENTED |
| Effect durability | frozen FSM, durable intent/attempt before dispatch proof, deterministic handlers | IMPLEMENTED |
| Ambiguous effects | UNKNOWN_OUTCOME blocks dependencies; interrupted executing enters reconciliation without redispatch; reconciliation resumes; manual review requires reconciling | IMPLEMENTED |
| Recovery | explicit Normal/RecoveryOnly/Quarantined scan; no silent authority reset | IMPLEMENTED |
| Disk pressure | real SQLite FULL regression plus deterministic fault injection | IMPLEMENTED |
| Strict-local egress | kernel hard deny + external no-network observation workflow | IMPLEMENTED |
| Security integrity | canonical event audit chain plus authority-security coverage for protected non-event records | IMPLEMENTED |
| Path security | canonical artifact receipt paths; traversal/symlink rejection for artifact and unprivileged paths | IMPLEMENTED |
| Protocol audit | repeated rejections receive unique durable incident identities | IMPLEMENTED |
| Source governance | no donor source code copied/ported; semantics-only evidence within recorded posture | IMPLEMENTED |
| Platform qualification | Windows, macOS, Ubuntu workflow includes platform-specific IPC/locality gates | IMPLEMENTED_PENDING_FINAL_RUN |

`IMPLEMENTED` above is not a final PASS claim. The commit containing this convergence record must still pass exact-head CI and the fresh post-CI authorized Qodo review.

## 3. Earlier convergence divergences retained as resolved

The earlier convergence cycle resolved:

- protected authority-state boundary and generic-path exclusion;
- RecoveryOnly / Quarantined startup behavior;
- accepted-connection deadline;
- mandatory authority-security integrity for authorization/effect/client/recovery records;
- enrolled CLI checkpoint/reconcile bootstrap authority;
- unsafe generic caller-selected canonical `EventKind` contract wording;
- stale quickstart/AGENTS/implementation-status text.

Those repairs remain part of the implementation baseline.

## 4. Authorized Qodo repair cycle

A later Qodo review found material correctness, security and reliability gaps. The repair sequence through `acdce817dbd89d8286fc08b8821aded0b7dbf8f7`, followed by formatting head `1a0ef2c1a72056e42528521ec63e5acaabc297f0`, addresses:

1. invalid effect FSM transitions accepted by generic CAS;
2. non-atomic checkpoint event/metadata/session-pointer boundary;
3. artifact path traversal;
4. symlink escape in unprivileged path admission;
5. protocol incident ID collision;
6. missing Unix ownership verification;
7. unbounded foreground bootstrap approval;
8. unbounded CLI IPC waits;
9. unauthenticated server challenge limits influencing client allocation;
10. interrupted executing effects lacking a reconciliation entry path;
11. durable reconciliation becoming permanently stuck after interruption;
12. direct `unknown_outcome -> manual_review` bypass discovered during independent repair re-read.

Two Qodo rule findings about the real process-kill and real SQLite FULL tests were rejected as non-actionable because the canonical Spec 002 test strategy explicitly requires those substrate-level proofs in addition to deterministic fault injection.

All Qodo repair-cycle threads are resolved or outdated on the pre-reconciliation head. Because this closeout package mutates the branch, those results are repair evidence only and are not a final exact-head review PASS.

## 5. T002-061 recovery reserve decision

`implementation/recovery-reserve-evaluation.md` records `NO_RECOVERY_RESERVE_GUARANTEE`.

Spec 002 does not claim an unproven cross-platform reserve. Disk-full handling instead fails closed before dispatch authority when the durable transaction cannot commit.

## 6. Qualification evidence policy

Prior successful workflow runs remain historical implementation evidence, but none transfers PASS to the commit containing this final reconciliation package.

The final candidate head must run the complete CI matrix:

- format;
- Clippy with warnings denied;
- full workspace tests;
- deterministic property qualification;
- bounded fuzz smoke;
- Windows/macOS/Linux platform IPC gates;
- authenticated daemon IPC qualification;
- adversarial authority probes;
- daemon build;
- externally observed strict-local no-network checks.

Only after that exact-head CI succeeds may the fresh authorized Qodo review be requested on the unchanged final candidate head.

## 7. Review policy

- **Codex review is explicitly excluded from the Golam review workflow by founder direction. It is not a finding source, fallback reviewer, or closeout gate.**
- CodeRabbit is not substituted for the authorized Qodo gate.
- Qodo is the authorized external repair/review source for this closeout sequence.
- Any material post-CI Qodo finding reopens repair and invalidates the current exact-head qualification.
- PR #3 remains Draft throughout this qualification sequence.

## 8. Scope boundary remains intact

Spec 002 has not admitted:

- model inference/download/providers;
- broad filesystem/shell/browser/MCP/skills product tools;
- Desktop/computer control;
- GolamConnect/channels;
- real consequential external effects;
- external network behavior;
- Spec 003 policy/secrets/sandbox implementation.

## 9. Closeout condition

```text
T002_001_TO_078=IMPLEMENTED
TASK_IMPLEMENTATION=COMPLETE
FINAL_CANDIDATE_HEAD=THIS_COMMIT
FINAL_EXACT_HEAD_CI=PENDING
FINAL_POST_CI_QODO=PENDING
SPEC_002_IMPLEMENTATION_COMPLETE=NO
WAIVER_TAKEN=NO
PR_DRAFT=YES
PR_READY=NO
MERGED=NO
SPEC_002_CLOSED_CANONICAL=NO
SPEC_003_AUTHORIZED=NO
```

When both final gates succeed on the same unchanged head, the PR evidence may truthfully change `SPEC_002_IMPLEMENTATION_COMPLETE` to `YES` without implying Ready, merge, or canonical closure. Spec 002 is not `CLOSED_CANONICAL` until separately authorized lifecycle actions occur and the implementation is merged into canonical `main`.
