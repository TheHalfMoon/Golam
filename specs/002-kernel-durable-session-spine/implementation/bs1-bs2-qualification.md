# BS-1 / BS-2 Qualification — Spec 002

**Qualification code head**: `29d54ca211e17c7bbcc0b2febfc2349d7b9ed2be`  
**GitHub Actions**: run ID `32958260286`, run number `225`  
**Matrix**: Windows / macOS / Ubuntu — SUCCESS  
**Scope**: Spec 002 deterministic local storage/effect simulators only; no external network or real consequential effect.

## Decision

```text
BS-1=PASS
BS-2=PASS
WAIVER=NO
```

This record summarizes executable evidence; it is not a substitute for exact-head CI.

## BS-1 — crash / replay / fork / checkpoint / disk failure

BS-1 starts in Spec 002 and requires the durable session spine to preserve canonical truth or fail closed across crash/restart, replay/checkpoint/fork behavior, disk pressure, and authority corruption.

Evidence exercised by the workspace/qualification matrix includes:

- `golam-kernel/tests/process_kill_recovery.rs`
  - real OS child-process kill after durable dispatch;
  - restart refuses blind redispatch and preserves the durable attempt/state evidence.
- `golam-ledger/tests/disk_full_fail_closed.rs`
  - real SQLite `SQLITE_FULL` before durable dispatch authority;
  - failed transaction rolls back the attempt/state mutation so no dispatch authority leaks from a failed commit.
- `golam-ledger/tests/qualification_properties.rs`
  - replay from full history and verified checkpoint remain equivalent across deterministic prefix lengths;
  - fork anchors remain immutable across multiple parent prefixes;
  - canonical hash chaining is deterministic and parent-sensitive.
- checkpoint/storage/recovery workspace tests
  - missing/corrupt checkpoint material falls back or enters the appropriate fail-closed recovery state;
  - canonical authority corruption is quarantined without silent reset;
  - incomplete/incoherent effect state is surfaced as recovery attention/recovery-only rather than invented completion.
- `golam-ledger/tests/recovery_reserve_policy.rs`
  - no unproven preallocated recovery-reserve guarantee is created or relied upon.

Result: the supported Spec 002 failure model has executable evidence for a valid canonical prefix, deterministic replay/fork/checkpoint behavior, fail-closed disk-full handling, and fail-closed authority corruption.

## BS-2 — duplicate-effect / UNKNOWN_OUTCOME

BS-2 starts in Spec 002 and requires ambiguous effects to remain durable/reconcilable without blind duplicate execution.

Evidence exercised by the workspace/qualification matrix includes:

- `golam-kernel/tests/effect_restart_safety.rs`
  - `AT_MOST_ONCE` does not blind redispatch after daemon restart;
  - `IRREVERSIBLE` does not blind redispatch after daemon restart.
- effect dispatch/completion tests
  - durable intent and attempt/`EXECUTING` transition commit before a dispatch proof is returned;
  - dependency state `UNKNOWN_OUTCOME` blocks dispatch without creating another attempt;
  - attempt finish and terminal transition commit atomically.
- deterministic simulator/fault tests
  - post-accept ambiguous failures are reconcilable without redispatch;
  - pre-accept failures do not invent remote acceptance;
  - idempotent semantics reuse stable keys/receipts;
  - at-most-once and irreversible simulators reject duplicate acceptance.
- manual-review/reconciliation tests
  - unresolved ambiguity may remain `UNKNOWN_OUTCOME`/`RECONCILING` and escalate to durable `MANUAL_REVIEW` evidence;
  - dependent effects remain blocked until a permitted definitive resolution exists.

Result: Spec 002 proves durable intent-before-dispatch plus no blind duplicate for the dangerous semantics represented by deterministic simulators.

## Integrity reinforcement

The same qualified code head includes the mandatory `authority-security` integrity chain. Client enrollment/revocation, authorization decisions, effect intent/transitions/attempt starts/finishes, and recovery incidents have coverage checks and source-row hash verification; tampering or missing audit coverage blocks authority-store reopen.

## CI gate contents

Run #225 passed on all three operating-system jobs:

- `Format`
- `Clippy` with `-D warnings`
- full workspace `Test`
- `Property qualification`
- `Bounded fuzz smoke`
- platform-specific `IPC transport qualification`
- `Authenticated daemon IPC qualification`
- `Adversarial authority qualification`
- daemon build + externally observed strict-local no-network qualification.

No PASS in this artifact should be carried to a later branch head without a fresh exact-head gate.
