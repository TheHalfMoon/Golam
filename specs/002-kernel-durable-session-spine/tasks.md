# Tasks — Spec 002 Kernel & Durable Session Spine

**Status**: IMPLEMENTATION_COMPLETE_PENDING_FINAL_EXACT_HEAD_CI_AND_PR_LIFECYCLE  
**Implementation branch**: `impl/002-kernel-durable-session-spine`  
**PR**: `#3` — OPEN / DRAFT  
**Canonical implementation base**: `main@cfcc90f452e7115bfb104f886e09c309a5d57a1c`  
**Last fully proven convergence head before closeout ledger mutation**: `a814e7d6a2b8610c9a54b96ae05c3df85335cee1`  
**Exact-head convergence CI**: run ID `32958907240` / run number `233` — Windows, macOS and Ubuntu full qualification workflow SUCCESS.

Legend:
- `[x]` = implementation/task requirement is satisfied by repository evidence.
- A `[x]` on this final documentation ledger remains valid only if the complete CI workflow attached to the commit containing this ledger also succeeds.
- `IMPLEMENTATION_COMPLETE` is not `CLOSED_CANONICAL`: PR #3 remains Draft/unmerged and Spec 003 remains unauthorized.

## Phase A — Exact-head/bootstrap

- [x] **T002-001** Verify exact live `main` after planning PR merge; create implementation branch from that exact commit. — Base remains `cfcc90f452e7115bfb104f886e09c309a5d57a1c`.
- [x] **T002-002** Create the bounded Rust workspace with only `golam-core`, `golam-ledger`, `golam-effects`, `golam-ipc`, `golam-kernel`, `golamd`, `golam`; pin stable toolchain and forbid unsafe Golam code. — Rust 1.98.0; seven-package spine retained.
- [x] **T002-003** Add baseline CI for fmt/clippy/test on Windows/macOS/Linux. — Expanded final workflow is qualified cross-platform.

## Phase B — Donor admission/evidence

- [x] **T002-010** Create bounded Source Foundry admission record before any donor source-code reuse. — SATISFIED N/A for Spec 002 implementation: no donor source code copied/ported/vendored; gate reopens before future code reuse.
- [x] **T002-011** Map selected Golam-Research protocol/recovery behaviors to Rust tests before implementation-detail reuse. — See `implementation/source-foundry/golam-research-semantics-map.md`.
- [x] **T002-012** Qualify exact Rust dependency versions and unsafe/FFI/platform boundaries. — See `implementation/dependency-qualification.md`.

## Phase C — Core types + protected storage

- [x] **T002-020** Implement IDs, protocol/schema versions, bounded errors and canonical byte encoding.
- [x] **T002-021** Implement protected Golam runtime/data/authority paths and per-platform permission verification; exclude authority from generic path admission.
- [x] **T002-022** Implement SQLite authority schema/tables for sessions/events/goals/forks/checkpoints/effects/transitions/clients/audit/recovery with future-version refusal. — `authority-security` is an integrity companion created transactionally on first covered protected record; absence with protected rows fails integrity.
- [x] **T002-023** Implement transactional global/per-session sequencing and deterministic event/hash-chain vectors.
- [x] **T002-024** Implement startup integrity checks and explicit fail-closed RecoveryOnly/Quarantined behavior; never silently reset canonical state.
- [x] **T002-025** Implement content-addressed artifact temp-write/hash/atomic-install/cleanup.
- [x] **T002-026** Implement checkpoint creation/verification/fallback and replay equivalence.
- [x] **T002-027** Implement immutable session fork anchors and property qualification.
- [x] **T002-028** Implement append-versioned Goal Ledger + rebuildable current projection.

## Phase D — IPC authentication

- [x] **T002-030** Implement typed/versioned bounded IPC frame codec/parser.
- [x] **T002-031** Implement Hello -> Challenge -> Authenticate -> Ready/Shutdown lifecycle, Ed25519 transcript authentication and server epoch binding.
- [x] **T002-032** Implement private Unix/macOS local transport with socket/path bounds and OS peer credential checks.
- [x] **T002-033** Implement Windows local named-pipe transport with protected current-user ACL, local-only mode, instance bound and peer metadata.
- [x] **T002-034** Implement explicit local client enrollment/revocation and protected client-key storage fallback with assurance class.
- [x] **T002-035** Implement request/reply IDs, cancellation, pending-call bounds and close-on-protocol-breach settlement.
- [x] **T002-036** Add adversarial authentication/protocol/resource probes. — Final CI runs explicit adversarial qualification.

## Phase E — Kernel + bootstrap authorization

- [x] **T002-040** Implement sealed/process-splittable KernelApi and prevent external construction of authority-bearing tokens.
- [x] **T002-041** Implement auditable deny-by-default `Authorize(principal, action, resource, context)` bootstrap engine.
- [x] **T002-042** Implement protected-resource checks so generic file/storage helpers cannot target kernel state; hostile adapter has no direct privileged-ledger authority.
- [x] **T002-043** Implement strict-local egress authorization as a monotonic hard denial for Spec 002 product behavior.
- [x] **T002-044** Add hostile-adapter compromise tests for authority minting/protected-state/event/client mutation boundaries.

## Phase F — Effect engine

- [x] **T002-050** Implement effect FSM and compare-and-swap transitions.
- [x] **T002-051** Implement EffectHandler metadata/execute/read-only-reconcile interfaces and persistent attempt records.
- [x] **T002-052** Implement deterministic simulator handlers for all five execution semantics.
- [x] **T002-053** Enforce durable intent/attempt/EXECUTING evidence before dispatch proof and block dependent effects on non-definitive prerequisites including UNKNOWN_OUTCOME.
- [x] **T002-054** Build deterministic fault injection around durable transitions and simulated remote accept/ack boundaries.
- [x] **T002-055** Prove AT_MOST_ONCE/IRREVERSIBLE handlers do not blind duplicate across process loss/restart.
- [x] **T002-056** Implement durable manual-review state/reporting for unresolved ambiguity.

## Phase G — Recovery + CLI

- [x] **T002-060** Implement startup recovery scan for incomplete effects/checkpoints/hash/integrity state and explicit Normal/RecoveryOnly/Quarantined outcomes.
- [x] **T002-061** Evaluate/implement preallocated disk recovery reserve; prove or remove the guarantee based on tests. — PASS with `NO_RECOVERY_RESERVE_GUARANTEE`; see `implementation/recovery-reserve-evaluation.md` and `recovery_reserve_policy` regression. Spec 002 does not claim an unproven cross-platform reserve.
- [x] **T002-062** Implement minimal authenticated CLI for client enroll, sessions/open/create/fork/goal, replay/checkpoint, deterministic effect simulate/reconcile and doctor. — Real CLI -> authenticated OS-local IPC -> daemon -> KernelApi path; enrolled-client bootstrap authority explicitly permits required checkpoint/reconcile operations without gaining client-management or network authority.
- [x] **T002-063** Add process-kill/restart integration harness and disk-full/corruption simulations. — Real OS child kill regression, real SQLite FULL rollback regression, and authority corruption/recovery qualification are in the workspace suite.

## Phase H — Qualification

- [x] **T002-070** Run cargo fmt/clippy/test exact-head gates. — Convergence head `a814e7d6...`, CI #233, all platforms SUCCESS; final closeout-ledger head must also pass.
- [x] **T002-071** Run deterministic property qualification for replay/checkpoints/forks/hash chains/effect FSM/idempotency. — Dedicated CI step SUCCESS on #233.
- [x] **T002-072** Run bounded fuzz smoke/corpus for IPC/event/migration decoders. — Dedicated CI step SUCCESS on #233.
- [x] **T002-073** Run Windows/macOS/Linux IPC integration matrix and platform-specific transport gates. — Dedicated OS steps SUCCESS on #233 where applicable.
- [x] **T002-074** Run external listener/strict-local no-egress proof. — Daemon is built and observed from outside the Golam-managed process; no Internet socket is observed while the local IPC listener is present.
- [x] **T002-075** Record BS-1 durability and BS-2 duplicate-effect qualification artifacts. — See `implementation/bs1-bs2-qualification.md`; `BS-1=PASS`, `BS-2=PASS`, waiver `NO`, backed by #225/#233 workspace qualification.
- [x] **T002-076** Run kernel-boundary and unauthenticated/adversarial local-client probes. — Dedicated adversarial authority/IPC qualification SUCCESS on #233.
- [x] **T002-077** Run final Spec Kit convergence against constitution/spec/research/plan/data-model/contracts/tasks and resolve every material divergence. — See `implementation/convergence.md`; resolved accepted-connection deadline, mandatory non-event authority integrity, enrolled-client checkpoint/reconcile authority, stale quickstart/AGENTS/checklist/data-model text, and unsafe generic reserved-event append wording. Convergence head #233 is green.
- [x] **T002-078** Prepare exact-head closeout report and keep Spec 003 blocked until Spec 002 is merged/closed canonical. — See `implementation/closeout.md`. The commit containing this final ledger/report must receive the complete CI matrix before the final PR evidence may claim exact-head PASS.

## Closeout guardrail

```text
SPEC_002_IMPLEMENTATION=COMPLETE
WAIVER_TAKEN=NO
PR_3_STATE=DRAFT
PR_READY=NO
MERGED=NO
SPEC_002_CLOSED_CANONICAL=NO
SPEC_003_AUTHORIZED=NO
```

No remaining Spec 002 implementation task authorizes models, broad product tools, Desktop, GolamConnect, real external effects, external network behavior, marking PR #3 Ready, merging it, or starting Spec 003. A final green exact-head CI run on this closeout ledger is required before the Draft PR is presented as implementation-complete.
