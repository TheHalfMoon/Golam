# Tasks — Spec 002 Kernel & Durable Session Spine

**Status**: GENERATED_AFTER_SPEC/RESEARCH/PLAN/CONTRACTS  
**Implementation authority**: begins only after this planning package is reviewed/merged from exact live truth.

## Phase A — Exact-head/bootstrap

- [ ] **T002-001** Verify exact live `main` after planning PR merge; create implementation branch from that exact commit.
- [ ] **T002-002** Create the Rust workspace with only `golam-core`, `golam-ledger`, `golam-effects`, `golam-ipc`, `golam-kernel`, `golamd`, `golam`; pin current stable toolchain and forbid unsafe Golam code.
- [ ] **T002-003** Add baseline CI for fmt/clippy/test on Windows/macOS/Linux; do not claim green until runs exist.

## Phase B — Donor admission/evidence

- [ ] **T002-010** Create bounded Source Foundry admission record for any Golam-Research files whose code (not only semantics) will be ported/copied; record exact permission evidence/scope and obligations.
- [ ] **T002-011** Map selected Golam-Research protocol/recovery behaviors to Rust tests before porting implementation details.
- [ ] **T002-012** Qualify exact Rust dependency versions for SQLite binding, BLAKE3, async runtime, serialization, IDs/errors and property/fuzz testing; record unsafe/FFI boundaries.

## Phase C — Core types + protected storage

- [ ] **T002-020** Implement IDs, protocol/schema versions, bounded errors and canonical byte-encoding primitives in `golam-core`.
- [ ] **T002-021** Implement protected Golam data/runtime directory creation and permission checks per platform.
- [ ] **T002-022** Implement SQLite migrations/tables for sessions/events/goals/forks/checkpoints/effects/transitions/clients/audit/recovery.
- [ ] **T002-023** Implement transactional global/per-session sequence assignment and deterministic event/hash-chain vectors.
- [ ] **T002-024** Implement authority DB startup integrity checks and fail-closed recovery-only mode; never silently reset.
- [ ] **T002-025** Implement content-addressed artifact temp-write/hash/atomic-install/cleanup.
- [ ] **T002-026** Implement checkpoint creation/verification/fallback and replay equivalence tests.
- [ ] **T002-027** Implement immutable session fork anchors and property tests.
- [ ] **T002-028** Implement append-versioned Goal Ledger + rebuildable current projection.

## Phase D — IPC authentication

- [ ] **T002-030** Implement typed/versioned IPC frame codec and parser with size/depth/resource bounds.
- [ ] **T002-031** Implement lifecycle handshake `hello/challenge/authenticate/ready/shutdown`, transcript signature and server epoch.
- [ ] **T002-032** Implement Unix-domain-socket transport with private runtime dir/socket + peer credential checks.
- [ ] **T002-033** Implement Windows named-pipe transport with user SID ACL + peer metadata where available.
- [ ] **T002-034** Implement explicit local client enrollment/revocation and qualified client-key storage backend/fallback.
- [ ] **T002-035** Implement request/reply IDs, cancellation, bounded pending calls and protocol-breach settlement.
- [ ] **T002-036** Add adversarial tests for unauthenticated client, wrong key, replay, stale epoch, malformed/repeated lifecycle, oversized frame, request-before-ready and resource exhaustion.

## Phase E — Kernel + bootstrap authorization

- [ ] **T002-040** Implement sealed/process-splittable KernelApi and prevent external construction of authority-bearing tokens.
- [ ] **T002-041** Implement bootstrap `Authorize(principal, action, resource, context)` deny-by-default engine with auditable decisions.
- [ ] **T002-042** Implement protected-resource checks so generic file/storage helpers cannot target kernel state.
- [ ] **T002-043** Implement strict-local egress authorization interface as deny-by-default; Spec 002 itself has no production egress client.
- [ ] **T002-044** Add hostile-adapter boundary test: cannot mint authority, modify policy-reserved state, append canonical audit or enroll/revoke clients without KernelApi.

## Phase F — Effect engine

- [ ] **T002-050** Implement effect FSM and compare-and-swap transitions.
- [ ] **T002-051** Implement EffectHandler metadata/execute/reconcile interfaces and persistent attempt records.
- [ ] **T002-052** Implement deterministic simulator handlers for all five execution semantics.
- [ ] **T002-053** Enforce durable intent-before-dispatch and dependent-effect blocking on UNKNOWN_OUTCOME.
- [ ] **T002-054** Build fault injector for every transition and simulated remote accept/ack boundary.
- [ ] **T002-055** Prove at-most-once/irreversible handlers do not blind duplicate across daemon kill/restart.
- [ ] **T002-056** Implement manual-review state/reporting for unreconcilable ambiguity.

## Phase G — Recovery + CLI

- [ ] **T002-060** Implement startup recovery scan for incomplete effects/checkpoints/hash chains.
- [ ] **T002-061** Evaluate/implement preallocated disk recovery reserve; prove or remove the guarantee based on tests.
- [ ] **T002-062** Implement minimal CLI commands for client enroll, sessions, replay/fork/checkpoint, effect simulator/reconcile and doctor/recovery report.
- [ ] **T002-063** Add process-kill/restart integration harness and disk-full/corruption simulations.

## Phase H — Qualification

- [ ] **T002-070** Run cargo fmt/clippy/test exact-head gates.
- [ ] **T002-071** Run property tests for replay/forks/hash chains/effect state/idempotency.
- [ ] **T002-072** Run fuzz smoke/corpus for IPC/event/migration decoders.
- [ ] **T002-073** Run Windows/macOS/Linux IPC integration matrix; explicitly record unsupported runner gaps.
- [ ] **T002-074** Run external listener scan and strict-local sinkhole/no-egress proof.
- [ ] **T002-075** Run BS-1 durability and BS-2 duplicate-effect qualification artifacts.
- [ ] **T002-076** Run kernel-boundary and unauthenticated-local-client adversarial probes.
- [ ] **T002-077** Run Spec Kit converge against spec/plan/tasks and resolve every material divergence.
- [ ] **T002-078** Prepare exact-head closeout report; do not start Spec 003 until Spec 002 is merged/closed canonical.
