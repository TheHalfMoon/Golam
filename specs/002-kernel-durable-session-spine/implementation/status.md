# Spec 002 Implementation Status

**Implementation branch**: `impl/002-kernel-durable-session-spine`  
**PR**: #3 — OPEN / DRAFT  
**Canonical base**: `main@cfcc90f452e7115bfb104f886e09c309a5d57a1c`  
**Base tree**: `da65a0ae907a53212bbfc7afed1a25e7f4aa4636`  
**Started**: 2026-08-24  
**Latest behavioral repair head**: `acdce817dbd89d8286fc08b8821aded0b7dbf8f7`  
**Latest formatting-only repair head**: `1a0ef2c1a72056e42528521ec63e5acaabc297f0`  
**Pre-reconciliation qualification head**: `b54abec064fd16074674a7c8c141f4bf3d69a245`  
**State**: `SPEC_002_REPAIR_CONVERGED_PENDING_FINAL_EXACT_HEAD_CI_AND_POST_CI_QODO`.

> Exact live GitHub truth is authoritative. No CI or review result transfers across a branch mutation. The commit containing this reconciled closeout package is the final candidate head and must receive the complete qualification workflow and a fresh authorized Qodo review after CI succeeds.

## Current gate summary

```text
T002_001_TO_078=IMPLEMENTED
TASK_IMPLEMENTATION=COMPLETE
WAIVER_TAKEN=NO

PR_3_DRAFT=YES
PR_READY=NO
MERGED=NO
SPEC_002_IMPLEMENTATION_COMPLETE=NO
SPEC_002_CLOSED_CANONICAL=NO
SPEC_003_AUTHORIZED=NO

LATEST_BEHAVIORAL_REPAIR_HEAD=acdce817dbd89d8286fc08b8821aded0b7dbf8f7
LATEST_FORMATTING_REPAIR_HEAD=1a0ef2c1a72056e42528521ec63e5acaabc297f0
PRE_RECONCILIATION_HEAD=b54abec064fd16074674a7c8c141f4bf3d69a245
FINAL_CANDIDATE_HEAD=THIS_COMMIT
FINAL_EXACT_HEAD_CI=PENDING
FINAL_POST_CI_QODO=PENDING
AUTHORIZED_REVIEW_MATERIAL_FINDINGS=PENDING_FINAL_REVIEW
CODEX_REVIEW_GATE=EXCLUDED_BY_FOUNDER_DIRECTION
```

## Repair convergence

The authorized Qodo review identified correctness, security and reliability gaps. The material repairs now include:

- enforce the frozen Spec 002 effect FSM at the generic `EffectStore::compare_and_swap` boundary;
- require manual-review placement to originate from durable `reconciling` state;
- make checkpoint canonical event, artifact metadata, checkpoint row, session head and security audit commit atomically;
- require exact content-addressed artifact receipt paths and reject traversal/symlink escapes;
- reject symlink components before unprivileged runtime-path admission;
- derive protocol incident identifiers from unique durable incident material;
- verify effective Unix ownership for the protected runtime tree in addition to permission bits;
- bound foreground bootstrap approval;
- bound CLI handshake/request/reply I/O with one absolute local IPC deadline;
- cap unauthenticated server-advertised handshake limits at the local client ceiling;
- admit interrupted durable `executing` effects into `unknown_outcome -> reconciling` without redispatch;
- resume interrupted durable `reconciling` work from persisted context.

Two Qodo test-only findings were rejected as non-actionable because the canonical test strategy explicitly requires real subprocess kill/restart and real SQLite `SQLITE_FULL` substrate qualification in addition to deterministic fault injection. Those tests remain required evidence.

All Qodo threads from the repair cycle are resolved or outdated on the pre-reconciliation head. They do not transfer as a final-review PASS after this documentation mutation.

## Preserved Spec 002 behavior

### Protected local authority and canonical state

- Seven-package Rust 1.98 workspace; Golam product crates forbid unsafe code.
- Protected runtime/authority subtree with per-platform verification and generic/unprivileged path exclusion.
- SQLite WAL + `synchronous=FULL`, forward-schema refusal, quick-check and canonical integrity verification.
- Transactional canonical ordering, deterministic BLAKE3 audit material, append-versioned goals, immutable forks, content-addressed artifacts and verified checkpoints.
- Authority corruption fails closed without silent reset.

### Mandatory security integrity

The canonical `SessionEvent` chain is reinforced by the independent `authority-security` chain for protected non-event authority records, including client enrollment/revocation, authorization decisions, effect intents/transitions/attempts, and recovery/protocol/manual-review incidents. Authority-store open verifies source-row coverage, canonical hashes, chain linkage and the chain head.

### Authenticated local IPC / daemon

- Versioned bounded `GIPC` framing and strict Hello -> Challenge -> Authenticate -> Ready lifecycle.
- Ed25519 transcript authentication plus independent OS-local peer checks.
- Unix/macOS private UDS with peer credentials; Windows current-user ACL named pipe with peer metadata.
- request/reply IDs, cancellation settlement, bounded pending requests, daemon connection deadline and CLI-side absolute IPC deadline.
- no HTTP/TCP control listener.

### Kernel / bootstrap authority

- `KernelApi` remains the privileged mutation boundary; authority-bearing implementation types remain sealed.
- bootstrap authorization is deny-by-default and decisions are durable/audited.
- strict-local network egress is a monotonic hard denial.
- reserved canonical event families are emitted through their owning typed domain operations rather than a generic caller-selected reserved-event append surface.

### Effects / recovery

- frozen Spec 002 effect FSM enforced at generic CAS and typed operations;
- deterministic handlers for all five execution semantics;
- durable intent and attempt/EXECUTING evidence before dispatch proof;
- `UNKNOWN_OUTCOME` dependency blocking and no blind duplicate for `AT_MOST_ONCE`/`IRREVERSIBLE`;
- interrupted `executing` can become durable unknown outcome for reconciliation without redispatch;
- reconciliation can resume after interruption and escalate to manual review only from `reconciling`;
- `RecoveryScanner` distinguishes Normal, RecoveryOnly and Quarantined states and blocks privileged service when required.

## Final exact-head qualification required

The commit containing this file must pass on Ubuntu, macOS and Windows:

- `cargo +1.98.0 fmt --all -- --check`;
- `cargo +1.98.0 clippy --locked --workspace --all-targets -- -D warnings`;
- `cargo +1.98.0 test --locked --workspace --all-targets`;
- property qualification;
- bounded fuzz smoke;
- platform IPC qualification;
- authenticated daemon IPC qualification;
- adversarial authority qualification;
- daemon build and external strict-local no-network observation.

After that exact-head CI succeeds, request a fresh authorized Qodo review on the unchanged head. Any material finding reopens repair and exact-head qualification.

No `PASS`, `SPEC_002_IMPLEMENTATION_COMPLETE`, or closeout claim is valid before both gates succeed on the same final candidate head.

## Review policy

- Codex review is explicitly excluded from the Golam workflow by founder direction and is not a finding source or gate.
- CodeRabbit is not substituted for the authorized Qodo gate.
- Prior Qodo results are repair evidence only after a branch mutation; they are not final exact-head review evidence.
- No waiver is taken.

## Lifecycle state

```text
PR_READY_AUTHORITY=NOT_TAKEN
MERGE_AUTHORITY=NOT_TAKEN
CLOSED_CANONICAL=NO
SPEC_003_START=BLOCKED
```

PR #3 must remain Draft. Do not merge it or start Spec 003 without separate explicit founder/bootstrap authorization and canonical post-merge evidence.

## Hard scope boundary

Spec 002 remains model-free, cloud-free and real-external-effect-free. It does not authorize broad product tools, Desktop/computer control, GolamConnect, external channels, model/provider integration, real secrets, or Spec 003 policy/secrets/sandbox implementation.
