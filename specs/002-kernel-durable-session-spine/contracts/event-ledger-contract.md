# Contract: Session/Event/Goal Ledger

## Canonical principles

- events append; they are never rewritten for compaction;
- per-session and global order are explicit integers assigned transactionally;
- timestamps never determine canonical order;
- payload schema is versioned;
- replay must reject unknown required schema versions rather than misinterpret bytes;
- goal versions append and are tied to canonical events;
- checkpoints are optional accelerators, not canonical replacements.

## Session creation

Creates a session row and `SessionCreated` event in one transaction.

## Event append

Append transaction:
1. lock/verify expected session head;
2. assign next session seq/global seq;
3. canonicalize payload bytes;
4. compute payload/event/audit hashes;
5. insert event;
6. update session head/audit head;
7. commit.

A failed commit exposes no successful event to callers.

## Fork

`fork_session(parent, through_seq)` verifies that exact parent event/hash, creates child session with parent anchor, and emits child `SessionForked` event. It never copies or mutates the parent prefix.

## Goal version

Goal version insert and corresponding event are one transaction. Current-goal projection may be recomputed from highest valid version.

## Integrity

Use domain-separated versioned canonical bytes. BLAKE3 candidate is frozen by test vectors before release. Security-critical audit chain covers at least:
- client enrollment/revocation/auth failures;
- authorization decisions;
- effect proposals/transitions/attempts;
- goal changes;
- forks/checkpoints;
- recovery incidents.

## Checkpoint

Checkpoint artifact includes deterministic projection bytes. Record contains event prefix cursor/hash + artifact hash. Verify before use; otherwise discard/fallback.

## Replay equivalence

For a supported schema corpus:

`replay(genesis..head) == replay(valid_checkpoint..head)`

must hold for defined projections.
