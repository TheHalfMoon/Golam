# Tasks — Spec 002 Kernel & Durable Session Spine

**Status**: IMPLEMENTATION_IN_PROGRESS  
**Implementation branch**: `impl/002-kernel-durable-session-spine`  
**PR**: `#3` — OPEN / DRAFT  
**Canonical implementation base**: `main@cfcc90f452e7115bfb104f886e09c309a5d57a1c`  
**Last reconciled proven code head**: `376c8d7439c7b6661f5fcb9d58887006fc0241ef`  
**Exact-head CI evidence**: run `32836135066` / run number `110` — Windows, macOS, Linux `fmt + clippy -D warnings + test` PASS

Legend:
- `[x]` = task requirement is satisfied by current implementation/evidence.
- `[ ] ... PARTIAL` = bounded implementation exists but the task is not complete.
- `[ ]` = not yet complete.
- A task is not promoted to PASS from intent, design, or an older head alone.

## Phase A — Exact-head/bootstrap

- [x] **T002-001** Verify exact live `main` after planning PR merge; create implementation branch from that exact commit. — PASS from `main@cfcc90f452e7115bfb104f886e09c309a5d57a1c`.
- [x] **T002-002** Create the Rust workspace with only `golam-core`, `golam-ledger`, `golam-effects`, `golam-ipc`, `golam-kernel`, `golamd`, `golam`; pin current stable toolchain and forbid unsafe Golam code. — PASS; Rust 1.98.0 and workspace `unsafe_code = forbid` are active.
- [x] **T002-003** Add baseline CI for fmt/clippy/test on Windows/macOS/Linux; do not claim green until runs exist. — PASS; exact-head matrix runs exist, latest proven code run `32836135066`.

## Phase B — Donor admission/evidence

- [x] **T002-010** Create bounded Source Foundry admission record for any Golam-Research files whose code (not only semantics) will be ported/copied; record exact permission evidence/scope and obligations. — SATISFIED AS NOT-APPLICABLE SO FAR: no Golam-Research source code has been copied/ported; semantics-only mapping is recorded. This task reopens before any source-code reuse.
- [x] **T002-011** Map selected Golam-Research protocol/recovery behaviors to Rust tests before porting implementation details. — PASS; see `implementation/source-foundry/golam-research-semantics-map.md`.
- [x] **T002-012** Qualify exact Rust dependency versions for SQLite binding, BLAKE3, async runtime, serialization, IDs/errors and property/fuzz testing; record unsafe/FFI boundaries. — PASS for qualification; see `implementation/dependency-qualification.md`. Candidate qualification does not equal production admission.

## Phase C — Core types + protected storage

- [x] **T002-020** Implement IDs, protocol/schema versions, bounded errors and canonical byte-encoding primitives in `golam-core`. — PASS.
- [x] **T002-021** Implement protected Golam data/runtime directory creation and permission checks per platform. — PASS: Unix/macOS private directory permissions and Windows current-user protected DACL application/re-verification are proven; authority state lives under an explicitly protected authority subtree; generic/unprivileged path admission rejects the authority root, DB, credential subtree, reserved audit/policy paths, traversal, and paths outside the runtime root. Exact-head cross-platform proof is run `32824677555`.
- [x] **T002-022** Implement SQLite migrations/tables for sessions/events/goals/forks/checkpoints/effects/transitions/clients/audit/recovery. — PASS for schema v1 tables and forward-version refusal.
- [x] **T002-023** Implement transactional global/per-session sequence assignment and deterministic event/hash-chain vectors. — PASS; canonical `global_seq` allocation is reconciled across session events, effect transitions, and authorization decisions, with a regression proving a session event advances past a prior authorization decision.
- [ ] **T002-024** Implement authority DB startup integrity checks and fail-closed recovery-only mode; never silently reset. — **PARTIAL**: startup quick-check + canonical event/audit integrity verification fail closed; explicit recovery-only/quarantine serving mode remains for T002-060.
- [x] **T002-025** Implement content-addressed artifact temp-write/hash/atomic-install/cleanup. — PASS.
- [x] **T002-026** Implement checkpoint creation/verification/fallback and replay equivalence tests. — PASS.
- [x] **T002-027** Implement immutable session fork anchors and property tests. — PASS for current bounded property coverage; final property-suite expansion remains T002-071.
- [x] **T002-028** Implement append-versioned Goal Ledger + rebuildable current projection. — PASS.

## Phase D — IPC authentication

- [x] **T002-030** Implement typed/versioned IPC frame codec and parser with size/depth/resource bounds. — PASS; deterministic bounded `GIPC` framing/parser.
- [x] **T002-031** Implement lifecycle handshake `hello/challenge/authenticate/ready/shutdown`, transcript signature and server epoch. — PASS at `13b222175eda9c760cd8581c879ccde1020af6f4`, CI `32798308181`: fixed lifecycle payload codecs; fail-closed lifecycle state machine; Ed25519 strict transcript verification; transcript binds protocol/client/nonces/server epoch plus negotiated limits and client key ID; wrong signature, stale epoch, nonce/key mismatch, malformed payload and out-of-order/repeated lifecycle tests.
- [x] **T002-032** Implement Unix-domain-socket transport with private runtime dir/socket + peer credential checks. — PASS at `e5845cfaa9ec9aa240afc92a61e0728c071722c7`, CI `32799215791`: parent runtime dir `0700`, socket `0600`, no stale-path auto-unlink, explicit platform socket-path byte bound, Linux `SO_PEERCRED`, macOS `LOCAL_PEERCRED` + `LOCAL_PEERPID`, same-effective-UID enforcement, valid peer PID, safe Rust wrapper boundary via exact-pinned `nix`, and no TCP/HTTP listener.
- [x] **T002-033** Implement Windows named-pipe transport with user SID ACL + peer metadata where available. — PASS at `29be235de00d853a205ae2f46add1d08b91c1796`, tree `4915eb0ee62324ff5faf25184d33c5a13680e9b4`, CI `32800522051`: protected current-user DACLs are applied and re-read/verified for Golam runtime/data/artifact directories; named pipe uses a protected current-user DACL, `accept_remote=false`, non-inheritable handles, bounded instance count, kernel-reported client PID/session metadata, and synchronous local transport. Windows CI performs the real ACL + pipe connect + peer metadata tests. SID in the pipe name is discovery only and is not treated as authority; T002-031 cryptographic client authentication remains independently required.
- [x] **T002-034** Implement explicit local client enrollment/revocation and qualified client-key storage backend/fallback. — PASS: Ed25519 credentials use protected per-user file fallback with explicit assurance class; client registry is durable; enrollment/revocation and registered-client authentication are now kernel-owned, with unknown/wrong/revoked keys closed and durably audited.
- [x] **T002-035** Implement request/reply IDs, cancellation, bounded pending calls and protocol-breach settlement. — PASS: bounded request tracker enforces request IDs, request/reply direction, exact payload length, pending limits, cancellation/reply settlement, duplicate/unknown IDs and close-on-breach behavior.
- [x] **T002-036** Add adversarial tests for unauthenticated client, wrong key, replay, stale epoch, malformed/repeated lifecycle, oversized frame, request-before-ready and resource exhaustion. — PASS: wire/lifecycle probes remain in `golam-ipc`; authority-dependent unknown/wrong/revoked/replay/pre-READY probes execute inside the kernel boundary and verify durable rejection reasons. Latest exact-head cross-platform proof is run `32824677555`.

## Phase E — Kernel + bootstrap authorization

- [x] **T002-040** Implement sealed/process-splittable KernelApi and prevent external construction of authority-bearing tokens. — PASS: authority grants and client-authority implementation are private modules/types; callers receive typed outcomes rather than constructible grants; `compile_fail` boundary probes enforce sealed paths; the public call shape remains future-IPC-compatible.
- [x] **T002-041** Implement bootstrap `Authorize(principal, action, resource, context)` deny-by-default engine with auditable decisions. — PASS: explicit owner/client/kernel/test bootstrap policy, deny-by-default fallback, durable authorization-decision rows, stable decision IDs/reason codes, and a canonical global sequence shared with other authority records.
- [x] **T002-042** Implement protected-resource checks so generic file/storage helpers cannot target kernel state. — PASS: unprivileged path admission rejects the authority root/DB/credential/audit/policy state, traversal, and external paths; product crates outside `golam-kernel` do not directly link the privileged ledger.
- [x] **T002-043** Implement strict-local egress authorization interface as deny-by-default; Spec 002 itself has no production egress client. — PASS: `network.egress*` is a hard monotonic denial before replaceable policy evaluation and is covered even under a permissive test policy.
- [x] **T002-044** Add hostile-adapter boundary test: cannot mint authority, modify policy-reserved state, append canonical audit or enroll/revoke clients without KernelApi. — PASS: hostile-adapter qualification proves protected-path rejection, denied client enrollment/revocation, denied egress, sealed grant/client-authority modules, and no direct privileged-ledger dependency from non-kernel product crates. Exact-head cross-platform proof is run `32824677555`.

## Phase F — Effect engine

- [x] **T002-050** Implement effect FSM and compare-and-swap transitions. — PASS at `34e6b9b4922c2b6a92e18416d6a0bdb8b0425135`, CI `32832568236`: full planned state vocabulary includes DENIED and APPROVAL_REQUIRED; `golam-effects` validates declared/forbidden FSM edges and blind-retry semantics; `golam-ledger::effects` durably commits effect intent plus PROPOSED transition and applies expected-current-state CAS transitions under `BEGIN IMMEDIATE`; stale CAS does not consume canonical `global_seq`; reopen tests prove durable current state and transition history.
- [x] **T002-051** Implement EffectHandler metadata/execute/reconcile interfaces and persistent attempt records. — PASS at `ba1dc799099db59e3b4c85cc67ee446ecc568c98`, CI `32833294311`: handler metadata covers supported actions/resources, execution semantics, idempotency support, reconciliation class, timeouts and manual-review capability; the trait exposes stable `derive_idempotency_key`, mutable `execute`, and read-only `reconcile`; durable attempts persist handler/version/dispatch token/start anchor and support write-once finish with success/failure/unknown outcomes, reopen verification, duplicate rejection and refinish rejection.
- [x] **T002-052** Implement deterministic simulator handlers for all five execution semantics. — PASS at `31cd4061b4d69bd77593f14f7f802ab37268b85a`, CI `32833866835`: deterministic in-memory simulators cover pure read, idempotent-at-least-once keyed write, at-most-once write with queryable status, compensatable write with compensation record, and irreversible write with an intentionally ambiguous acknowledgement path; tests prove stable receipts, idempotent key lookup, redispatch rejection, compensation replay safety and reconciliation behavior without external network/effects.
- [x] **T002-053** Enforce durable intent-before-dispatch and dependent-effect blocking on UNKNOWN_OUTCOME. — PASS at `376c8d7439c7b6661f5fcb9d58887006fc0241ef`, CI `32836135066`: canonical bounded dependency encoding is fail-closed; `prepare_dispatch` requires the effect to be AUTHORIZED and every dependency to be definitively SUCCEEDED, so UNKNOWN_OUTCOME/missing/nonterminal dependencies block before any attempt exists; one `BEGIN IMMEDIATE` transaction writes the durable attempt and AUTHORIZED→EXECUTING transition before returning; `KernelApi` returns a sealed `PreparedEffectDispatch` proof rather than exposing a constructible dispatch authority token.
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

`T002-034 -> T002-035 -> T002-036 -> Phase E -> Phase F -> Phase G -> Phase H`.

Do not start Spec 003, models, broad tools, Desktop, GolamConnect, real external effects, or external network behavior from Spec 002 authority.
