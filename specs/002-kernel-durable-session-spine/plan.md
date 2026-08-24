# Implementation Plan — Spec 002 Kernel & Durable Session Spine

**Branch**: `spec/002-kernel-durable-session-spine`  
**Base**: `7f3e7f8d6fe75f96b0190cb856a84aa54700ff38`  
**Status**: PLANNED — NO RUST IMPLEMENTATION IN THIS PLANNING PR

## Summary

Build a model-free Rust daemon/CLI spine with authenticated local IPC, a small process-splittable privileged kernel API, canonical SQLite session/event/goal/effect state, immutable forks, verified checkpoints, deterministic effect simulators and fail-closed recovery.

## Constitution check

| Gate | Result |
|---|---|
| Local-first/no hidden cloud | PASS — no cloud/model/network dependency |
| Rust trusted path | PASS — all Spec 002 product components Rust |
| Smaller privileged kernel | PASS — explicit `KernelApi`, protected state, transport outside authority modules |
| Authenticated local IPC | PASS — OS peer + enrolled challenge/response |
| Gated durable effects | PASS — intent-before-dispatch + reconciliation |
| Clean source permission/provenance | PASS — donor evidence pinned; no code admitted yet |
| Verification over claims | PASS — fault/property/fuzz/adversarial exit gates |

## Initial workspace — maximum seven crates/binaries

```text
crates/
  golam-core/      # pure IDs/types/errors/version contracts
  golam-ledger/    # sessions/events/goals/forks/checkpoints/artifact refs/SQLite
  golam-effects/   # effect FSM + handler/reconciler contracts
  golam-ipc/       # local framing/protocol/client auth transport adapters
  golam-kernel/    # authority API, protected state ownership, bootstrap authorization
apps/
  golamd/          # composition root
  golam/           # minimal CLI client
```

Do not create empty crates for future specs.

Dependency direction:

```text
golam-core
   ↑    ↑    ↑
ledger effects ipc
   \     |    /
     golam-kernel
          ↑
        golamd

 golam CLI -> golam-core + golam-ipc
```

`golam-kernel` owns authority semantics; `golamd` owns process composition/transport lifecycle. `golam-ipc` cannot mint kernel tokens.

## Storage layout

```text
<GolamData>/
  authority/
    golam.db
    recovery-reserve.bin
    quarantine/
  artifacts/
    blake3/<prefix>/<hash>
  runtime/
    socket-or-pipe-metadata
```

- parent directories are user-private;
- authority directory is never exposed through future generic filesystem tools;
- SQLite schema migrations are embedded/forward-only and backed up before destructive changes;
- authority DB corruption enters recovery-only mode, never automatic reset.

## Local IPC

### Wire state machine

```text
CONNECTED
 -> HELLO(protocol, client_id)
 -> CHALLENGE(server_nonce, server_epoch)
 -> AUTHENTICATE(client_key_id, signed_challenge, client_nonce)
 -> READY(session_token, limits)
 -> REQUEST / CANCEL / EVENT / REPLY
 -> SHUTDOWN
```

Protocol violations close the session and emit a bounded audit record.

### Transport

- Windows named pipe with current-user ACL and peer metadata.
- Unix-domain socket on macOS/Linux in a 0700 directory / 0600 socket with peer credentials.
- no TCP/HTTP control endpoint in Spec 002.

### Enrollment

First-party CLI enrollment creates a client keypair after explicit local bootstrap approval. Private material uses the strongest available OS user credential facility selected in implementation qualification; file fallback must be 0600 and clearly lower assurance. Revocation is kernel-owned.

## Canonical ledger

SQLite canonical tables/logs:

- `sessions`
- `session_events`
- `goal_versions`
- `checkpoints`
- `effects`
- `effect_transitions`
- `client_identities`
- `audit_chain_heads`
- `schema_migrations`

Every canonical mutation occurs in a transaction that also appends its corresponding event/audit transition where applicable.

Global order uses a monotonically assigned database sequence. Per-session sequence is unique. Timestamps are descriptive; sequence defines order.

## Integrity

Use domain-separated BLAKE3 over versioned canonical bytes. Freeze test vectors before broad implementation. Parent/fork anchors include exact parent event hash. Security-critical records are hash chained; checkpoint verification includes prefix hash.

Signatures are deferred to Spec 003 identity/key infrastructure unless needed for local client authentication; hash integrity is mandatory here.

## Goal ledger

Goals are append-versioned canonical rows linked to an event ID/global sequence. A current-goal projection is disposable/rebuildable. Later compaction/model context cannot mutate historical goal versions.

## Effect engine

```text
PROPOSED
 -> DENIED
 -> AUTHORIZED
    -> APPROVAL_REQUIRED -> AUTHORIZED | DENIED
    -> EXECUTING
       -> SUCCEEDED
       -> FAILED
       -> UNKNOWN_OUTCOME -> RECONCILING -> SUCCEEDED | FAILED | MANUAL_REVIEW
```

Spec 002 handlers are simulators only. Each implements:

```text
metadata()
derive_idempotency_key(intent)
execute(authorized_intent)
reconcile(intent, prior_attempt)
```

Rules:
- persist + durable commit intent before `execute`;
- at-most-once/irreversible never blind retry;
- unknown prerequisite blocks dependents;
- transitions use compare-and-swap expected state;
- crash tests instrument every boundary.

## Bootstrap authorization

Stable interface:

`Authorize(principal, action, resource, context) -> Allow | Deny(reason)`

Spec 002 bootstrap policy allows only enrolled local owner/client operations required by this slice and synthetic effects. Everything else denies. Spec 003 replaces evaluation with Cedar without changing call semantics.

## Recovery

Startup sequence:
1. secure data/runtime directories;
2. open DB with expected settings;
3. verify schema/migrations;
4. SQLite integrity/quick check;
5. verify event/audit chains;
6. resolve incomplete local transactions/state transitions;
7. identify `UNKNOWN_OUTCOME` effects for reconciliation/manual review;
8. validate checkpoints/artifact hashes;
9. enter serving mode only if authority state is coherent.

On authority corruption: close write path, quarantine evidence, emit local recovery report, refuse privileged service.

The design reserves a small preallocated recovery file so disk-full handling can release space for critical reconciliation metadata; implementation must test whether this mechanism actually survives target-platform failure conditions before relying on it.

## Donor use

`Golam-Research` is read first for relevant behavior. Port semantics where useful; do not carry Electron/cloud assumptions. `grok-build`/Goose provide Rust references. DeepSeek Harness provides session-log semantics. No code is imported by the planning PR.

## Test strategy

- unit tests for state transitions/parsers;
- property tests for replay/fork/idempotency/hash-chain invariants;
- fuzz IPC frame parser, event decoder and migration input;
- fault injection at every SQLite/effect/checkpoint boundary;
- subprocess kill/restart harness;
- Windows/macOS/Linux IPC integration tests;
- adversarial unauthenticated/replayed/oversized/malformed client tests;
- disk-full and corruption scenarios;
- external network listener/egress probes.

## CI target

When implementation begins, add GitHub Actions for Windows/macOS/Linux with:

- `cargo fmt --check`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo test --workspace`;
- deterministic property tests;
- bounded fuzz smoke corpus;
- platform IPC integration gates where runner capability permits.

Do not claim CI PASS until runs exist on exact head.

## Exit gate

Spec 002 implementation may close only when:
- all tasks complete;
- exact-head tests pass;
- durability/duplicate-effect/IPC/boundary/no-egress evidence exists;
- implementation `converge` finds no material divergence from spec/plan/tasks;
- PR is reviewed/merged before Spec 003 begins.
