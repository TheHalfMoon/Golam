# Tasks — Spec 002 Kernel & Durable Session Spine

**Status**: IMPLEMENTATION_IN_PROGRESS  
**Implementation branch**: `impl/002-kernel-durable-session-spine`  
**PR**: `#3` — OPEN / DRAFT  
**Canonical implementation base**: `main@cfcc90f452e7115bfb104f886e09c309a5d57a1c`  
**Last reconciled proven code head**: `13b222175eda9c760cd8581c879ccde1020af6f4`  
**Exact-head CI evidence**: run `32798308181` / run number `32` — Windows, macOS, Linux `fmt + clippy -D warnings + test` PASS

Legend:
- `[x]` = task requirement is satisfied by current implementation/evidence.
- `[ ] ... PARTIAL` = bounded implementation exists but the task is not complete.
- `[ ]` = not yet complete.
- A task is not promoted to PASS from intent, design, or an older head alone.

## Phase A — Exact-head/bootstrap

- [x] **T002-001** Verify exact live `main` after planning PR merge; create implementation branch from that exact commit. — PASS from `main@cfcc90f452e7115bfb104f886e09c309a5d57a1c`.
- [x] **T002-002** Create the Rust workspace with only `golam-core`, `golam-ledger`, `golam-effects`, `golam-ipc`, `golam-kernel`, `golamd`, `golam`; pin current stable toolchain and forbid unsafe Golam code. — PASS; Rust 1.98.0 and workspace `unsafe_code = forbid` are active.
- [x] **T002-003** Add baseline CI for fmt/clippy/test on Windows/macOS/Linux; do not claim green until runs exist. — PASS; exact-head matrix runs exist, latest proven code run `32798308181`.

## Phase B — Donor admission/evidence

- [x] **T002-010** Create bounded Source Foundry admission record for any Golam-Research files whose code (not only semantics) will be ported/copied; record exact permission evidence/scope and obligations. — SATISFIED AS NOT-APPLICABLE SO FAR: no Golam-Research source code has been copied/ported; semantics-only mapping is recorded. This task reopens before any source-code reuse.
- [x] **T002-011** Map selected Golam-Research protocol/recovery behaviors to Rust tests before porting implementation details. — PASS; see `implementation/source-foundry/golam-research-semantics-map.md`.
- [x] **T002-012** Qualify exact Rust dependency versions for SQLite binding, BLAKE3, async runtime, serialization, IDs/errors and property/fuzz testing; record unsafe/FFI boundaries. — PASS for qualification; see `implementation/dependency-qualification.md`. Candidate qualification does not equal production admission.

## Phase C — Core types + protected storage

- [x] **T002-020** Implement IDs, protocol/schema versions, bounded errors and canonical byte-encoding primitives in `golam-core`. — PASS.
- [ ] **T002-021** Implement protected Golam data/runtime directory creation and permission checks per platform. — **PARTIAL**: Unix/macOS user-only `0700` path protection is verified; Windows authority readiness deliberately fails closed until current-user SID ACL verification is implemented with T002-033. Planned authority-directory separation also remains an explicit convergence item.
- [x] **T002-022** Implement SQLite migrations/tables for sessions/events/goals/forks/checkpoints/effects/transitions/clients/audit/recovery. — PASS for schema v1 tables and forward-version refusal.
- [x] **T002-023** Implement transactional global/per-session sequence assignment and deterministic event/hash-chain vectors. — PASS.
- [ ] **T002-024** Implement authority DB startup integrity checks and fail-closed recovery-only mode; never silently reset. — **PARTIAL**: startup quick-check + canonical event/audit integrity verification fail closed; explicit recovery-only/quarantine serving mode remains for T002-060.
- [x] **T002-025** Implement content-addressed artifact temp-write/hash/atomic-install/cleanup. — PASS.
- [x] **T002-026** Implement checkpoint creation/verification/fallback and replay equivalence tests. — PASS.
- [x] **T002-027** Implement immutable session fork anchors and property tests. — PASS for current bounded property coverage; final property-suite expansion remains T002-071.
- [x] **T002-028** Implement append-versioned Goal Ledger + rebuildable current projection. — PASS.

## Phase D — IPC authentication

- [x] **T002-030** Implement typed/versioned IPC frame codec and parser with size/depth/resource bounds. — PASS; deterministic bounded `GIPC` framing/parser.
- [x] **T002-031** Implement lifecycle handshake `hello/challenge/authenticate/ready/shutdown`, transcript signature and server epoch. — PASS at `13b222175eda9c760cd8581c879ccde1020af6f4`, CI `32798308181`: fixed lifecycle payload codecs; fail-closed lifecycle state machine; Ed25519 strict transcript verification; transcript binds protocol/client/nonces/server epoch plus negotiated limits and client key ID; wrong signature, stale epoch, nonce/key mismatch, malformed payload and out-of-order/repeated lifecycle tests.
- [ ] **T002-032** Implement Unix-domain-socket transport with private runtime dir/socket + peer credential checks.
- [ ] **T002-033** Implement Windows named-pipe transport with user SID ACL + peer metadata where available. — This task must also close the Windows side of T002-021; no path-only substitute counts.
- [ ] **T002-034** Implement explicit local client enrollment/revocation and qualified client-key storage backend/fallback.
- [ ] **T002-035** Implement request/reply IDs, cancellation, bounded pending calls and protocol-breach settlement.
- [ ] **T002-036** Add adversarial tests for unauthenticated client, wrong key, replay, stale epoch, malformed/repeated lifecycle, oversized frame, request-before-ready and resource exhaustion.

## Phase E — Kernel + bootstrap authorization

- [ ] **T002-040** Implement sealed/process-splittable KernelApi and prevent external construction of authority-bearing tokens. — Bootstrap boundary exists, but full task remains pending adversarial/process-split qualification.
- [ ] **T002-041** Implement bootstrap `Authorize(principal, action, resource, context)` deny-by-default engine with auditable decisions.
- [ ] **T002-042** Implement protected-resource checks so generic file/storage helpers cannot target kernel state.
- [ ] **T002-043** Implement strict-local egress authorization interface as deny-by-default; Spec 002 itself has no production egress client.
- [ ] **T002-044** Add hostile-adapter boundary test: cannot mint authority, modify policy-reserved state, append canonical audit or enroll/revoke clients without KernelApi.

## Phase F — Effect engine

- [ ] **T002-050** Implement effect FSM and compare-and-swap transitions. — Initial effect vocabulary exists; persistent CAS engine is not yet complete.
- [ ] **T002-051** Implement EffectHandler metadata/execute/reconcile interfaces and persistent attempt records.
- [ ] **T002-052** Implement deterministic simulator handlers for all five execution semantics.
- [ ] **T002-053** Enforce durable intent-before-dispatch and dependent-effect blocking on UNKNOWN_OUTCOME.
- [ ] **T002-054** Build fault injector for every transition and simulated remote accept/ack boundary.
- [ ] **T002-055** Prove at-most-once/irreversible handlers do not blind duplicate across daemon kill/restart.
- [ ] **T002-056** Implement manual-review state/reporting for unreconcilable ambiguity.

## Phase G — Recovery + CLI

- [ ] **T002-060** Implement startup recovery scan for incomplete effects/checkpoints/hash chains and explicit recovery-only/quarantine mode. — Must close the remaining T002-024 gap.
- [ ] **T002-061** Evaluate/implement preallocated disk recovery reserve; prove or remove the guarantee based on tests.
- [ ] **T002-062** Implement minimal CLI commands for client enroll, sessions, replay/fork/checkpoint, effect simulator/reconcile and doctor/recovery report.
- [ ] **T002-063** Add process-kill/restart integration harness and disk-full/corruption simulations.

## Phase H — Qualification

- [ ] **T002-070** Run cargo fmt/clippy/test exact-head gates. — Slice-level exact-head gates are continuously green when claimed; final Spec 002 gate remains pending completion of all phases.
- [ ] **T002-071** Run property tests for replay/forks/hash chains/effect state/idempotency.
- [ ] **T002-072** Run fuzz smoke/corpus for IPC/event/migration decoders.
- [ ] **T002-073** Run Windows/macOS/Linux IPC integration matrix; explicitly record unsupported runner gaps.
- [ ] **T002-074** Run external listener scan and strict-local sinkhole/no-egress proof.
- [ ] **T002-075** Run BS-1 durability and BS-2 duplicate-effect qualification artifacts.
- [ ] **T002-076** Run kernel-boundary and unauthenticated-local-client adversarial probes.
- [ ] **T002-077** Run final Spec Kit converge against constitution/spec/research/plan/data-model/contracts/tasks and resolve every material divergence. — Interim convergence is recorded in `implementation/convergence.md`; final gate remains pending.
- [ ] **T002-078** Prepare exact-head closeout report; do not start Spec 003 until Spec 002 is merged/closed canonical.

## Execution-order guardrail

Continue in task order unless an earlier task is required to satisfy a later platform gate:

`T002-032 -> T002-033 -> T002-034 -> T002-035 -> T002-036 -> Phase E -> Phase F -> Phase G -> Phase H`.

Do not start Spec 003, models, broad tools, Desktop, GolamConnect, real external effects, or external network behavior from Spec 002 authority.
