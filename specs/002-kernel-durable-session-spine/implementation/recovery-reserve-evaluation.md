# Spec 002 Recovery Reserve Evaluation

## Decision

`NO_RECOVERY_RESERVE_GUARANTEE`

Spec 002 does not create, advertise, or rely on a preallocated `recovery-reserve.bin` as a disk-full recovery guarantee.

## Contract basis

The storage/recovery contract requires the implementation to evaluate a preallocated reserve and requires tests to prove it before it can be treated as a guarantee. It does not authorize treating a nominally sized file as guaranteed recoverable journal capacity.

## Evaluation

A portable Rust file length or ordinary write is not sufficient proof that physical blocks are durably reserved for the SQLite WAL/journal failure mode. Filesystems may use sparse allocation, delayed allocation, copy-on-write, quotas, compression, snapshots, or platform-specific reservation semantics. A deterministic in-memory capacity simulator would only prove the simulator, not Windows/macOS/Linux ENOSPC behavior.

The current CI matrix also does not provide an isolated bounded filesystem whose exhaustion can be induced without risking the runner outside the test sandbox. Therefore Spec 002 has no target-platform evidence that a reserve file would restore enough durable capacity for a post-dispatch authority transition under real disk exhaustion.

## Implemented invariant

Until such evidence exists:

- startup/recovery does not create `authority/recovery-reserve.bin`;
- recovery correctness does not depend on freeing reserved bytes;
- disk-full before durable intent must remain fail-closed with no dispatch;
- a post-dispatch durability failure remains an ambiguous effect requiring reconstruction/reconciliation/manual review;
- T002-063 may test bounded disk-full/corruption simulations, but simulator results alone do not upgrade this decision into a platform guarantee.

## Qualification

`crates/golam-ledger/tests/recovery_reserve_policy.rs` is a cross-platform regression that runs the real startup recovery scanner and proves the unproven reserve artifact is not created. Windows, macOS and Linux CI must pass before T002-061 is closed.

A future change that wants to claim reserve-backed recovery capacity must reopen this decision and add target-platform ENOSPC evidence before changing the guarantee.
