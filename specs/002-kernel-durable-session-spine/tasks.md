# Tasks — Spec 002 Kernel & Durable Session Spine

**Status**: `TASK_IMPLEMENTATION_COMPLETE_QUALIFICATION_OPEN`  
**Implementation branch**: `impl/002-kernel-durable-session-spine`  
**PR**: `#3` — OPEN / DRAFT  
**Canonical implementation base**: `main@cfcc90f452e7115bfb104f886e09c309a5d57a1c`  
**Final candidate head**: the commit containing this reconciled task ledger.

Legend:
- `[x]` means the bounded task implementation exists in repository evidence.
- Task implementation completion does **not** equal `SPEC_002_IMPLEMENTATION_COMPLETE` while final exact-head CI and the post-CI authorized Qodo review remain open.
- No PASS transfers across a branch mutation.

## Phase A — Exact-head/bootstrap

- [x] **T002-001** Verify exact live `main` after planning PR merge; create implementation branch from that exact commit. — Base remains `cfcc90f452e7115bfb104f886e09c309a5d57a1c`.
- [x] **T002-002** Create the bounded Rust workspace with only `golam-core`, `golam-ledger`, `golam-effects`, `golam-ipc`, `golam-kernel`, `golamd`, `golam`; pin stable toolchain and forbid unsafe Golam code. — Rust 1.98.0; seven-package spine retained.
- [x] **T002-003** Add baseline CI for fmt/clippy/test on Windows/macOS/Linux. — Final workflow includes the complete qualification matrix.

## Phase B — Donor admission/evidence

- [x] **T002-010** Create bounded Source Foundry admission record before donor source-code reuse. — N/A for Spec 002 implementation because no donor source code was copied/ported/vendored; gate reopens before future reuse.
- [x] **T002-011** Map selected Golam-Research protocol/recovery behaviors to Rust tests before implementation-detail reuse. — See `implementation/source-foundry/golam-research-semantics-map.md`.
- [x] **T002-012** Qualify exact Rust dependency versions and unsafe/FFI/platform boundaries. — See `implementation/dependency-qualification.md`.

## Phase C — Core types + protected storage

- [x] **T002-020** Implement IDs, protocol/schema versions, bounded errors and canonical byte encoding.
- [x] **T002-021** Implement protected Golam runtime/data/authority paths and per-platform permission/ownership verification; exclude authority from generic path admission.
- [x] **T002-022** Implement SQLite authority schema/tables for sessions/events/goals/forks/checkpoints/effects/transitions/clients/audit/recovery with future-version refusal and mandatory `authority-security` coverage.
- [x] **T002-023** Implement transactional global/per-session sequencing and deterministic event/hash-chain vectors.
- [x] **T002-024** Implement startup integrity checks and explicit fail-closed RecoveryOnly/Quarantined behavior; never silently reset canonical state.
- [x] **T002-025** Implement content-addressed artifact temp-write/hash/atomic-install/cleanup with canonical receipt-path validation.
- [x] **T002-026** Implement checkpoint creation/verification/fallback and replay equivalence; checkpoint event/metadata/session-head/security evidence commit atomically.
- [x] **T002-027** Implement immutable session fork anchors and property qualification.
- [x] **T002-028** Implement append-versioned Goal Ledger + rebuildable current projection.

## Phase D — IPC authentication

- [x] **T002-030** Implement typed/versioned bounded IPC frame codec/parser.
- [x] **T002-031** Implement Hello -> Challenge -> Authenticate -> Ready/Shutdown lifecycle, Ed25519 transcript authentication, local-ceiling negotiation and server epoch binding.
- [x] **T002-032** Implement private Unix/macOS local transport with socket/path bounds and OS peer credential checks.
- [x] **T002-033** Implement Windows local named-pipe transport with protected current-user ACL, local-only mode, instance bound and peer metadata.
- [x] **T002-034** Implement explicit local client enrollment/revocation and protected client-key storage fallback with assurance class.
- [x] **T002-035** Implement request/reply IDs, cancellation, pending-call bounds and close-on-protocol-breach settlement.
- [x] **T002-036** Add adversarial authentication/protocol/resource probes.

## Phase E — Kernel + bootstrap authorization

- [x] **T002-040** Implement sealed/process-splittable KernelApi and prevent external construction of authority-bearing tokens.
- [x] **T002-041** Implement auditable deny-by-default `Authorize(principal, action, resource, context)` bootstrap engine.
- [x] **T002-042** Implement protected-resource checks so generic file/storage helpers cannot target kernel state; hostile adapter has no direct privileged-ledger authority.
- [x] **T002-043** Implement strict-local egress authorization as a monotonic hard denial for Spec 002 product behavior.
- [x] **T002-044** Add hostile-adapter compromise tests for authority minting/protected-state/event/client mutation boundaries.

## Phase F — Effect engine

- [x] **T002-050** Implement and enforce the frozen effect FSM at the generic compare-and-swap boundary.
- [x] **T002-051** Implement EffectHandler metadata/execute/read-only-reconcile interfaces and persistent attempt records.
- [x] **T002-052** Implement deterministic simulator handlers for all five execution semantics.
- [x] **T002-053** Enforce durable intent/attempt/EXECUTING evidence before dispatch proof and block dependent effects on non-definitive prerequisites including UNKNOWN_OUTCOME.
- [x] **T002-054** Build deterministic fault injection around durable transitions and simulated remote accept/ack boundaries.
- [x] **T002-055** Prove AT_MOST_ONCE/IRREVERSIBLE handlers do not blind duplicate across process loss/restart.
- [x] **T002-056** Implement durable reconciliation/manual-review state and reporting; manual review is reachable only from durable `reconciling` state and interrupted reconciliation is resumable.

## Phase G — Recovery + CLI

- [x] **T002-060** Implement startup recovery scan for incomplete effects/checkpoints/hash/integrity state and explicit Normal/RecoveryOnly/Quarantined outcomes.
- [x] **T002-061** Evaluate preallocated disk recovery reserve; prove or remove the guarantee. — `NO_RECOVERY_RESERVE_GUARANTEE` is the tested decision.
- [x] **T002-062** Implement minimal authenticated CLI for client enroll, sessions/open/create/fork/goal, replay/checkpoint, deterministic effect simulate/reconcile and doctor, with an absolute IPC deadline.
- [x] **T002-063** Add process-kill/restart integration harness and disk-full/corruption qualification. — Real OS child-kill and real SQLite `SQLITE_FULL` tests complement deterministic fault injection and remain canonical substrate evidence.

## Phase H — Qualification / closeout

- [x] **T002-070** Implement exact-head fmt/clippy/test gates. — Final candidate head must still pass them; prior green runs are historical evidence only.
- [x] **T002-071** Implement deterministic property qualification for replay/checkpoints/forks/hash chains/effect FSM/idempotency.
- [x] **T002-072** Implement bounded fuzz smoke/corpus for IPC/event/migration decoders.
- [x] **T002-073** Implement Windows/macOS/Linux IPC integration matrix and platform-specific transport gates.
- [x] **T002-074** Implement external listener/strict-local no-egress proof.
- [x] **T002-075** Record BS-1 durability and BS-2 duplicate-effect qualification artifacts. — Prior evidence records `BS-1=PASS`, `BS-2=PASS`, waiver `NO`; final exact-head CI remains independently required.
- [x] **T002-076** Implement kernel-boundary and unauthenticated/adversarial local-client probes.
- [x] **T002-077** Re-run Spec Kit convergence against constitution/spec/research/plan/data-model/contracts/tasks and repair material divergences. — Post-review reconciliation is recorded in `implementation/convergence.md`.
- [x] **T002-078** Prepare final-candidate closeout records and keep Spec 003 blocked until Spec 002 is merged/closed canonical. — This task is implemented by the reconciled closeout package, but its qualification gate remains open until exact-head CI and post-CI Qodo complete.

## Qualification gate

```text
T002_001_TO_078=IMPLEMENTED
TASK_IMPLEMENTATION=COMPLETE
WAIVER_TAKEN=NO
FINAL_CANDIDATE_HEAD=THIS_COMMIT
FINAL_EXACT_HEAD_CI=PENDING
FINAL_POST_CI_QODO=PENDING
SPEC_002_IMPLEMENTATION_COMPLETE=NO
PR_3_STATE=DRAFT
PR_READY=NO
MERGED=NO
SPEC_002_CLOSED_CANONICAL=NO
SPEC_003_AUTHORIZED=NO
CODEX_REVIEW_GATE=EXCLUDED_BY_FOUNDER_DIRECTION
```

The complete CI workflow attached to the commit containing this ledger must succeed on Windows, macOS and Ubuntu. After that success, request a fresh authorized Qodo review on the unchanged head. Any material finding reopens repair and qualification.

No remaining Spec 002 task authorizes models, broad product tools, Desktop, GolamConnect, real external effects, external network behavior, marking PR #3 Ready, merging it, or starting Spec 003.
