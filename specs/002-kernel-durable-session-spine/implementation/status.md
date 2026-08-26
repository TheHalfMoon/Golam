# Spec 002 Implementation Status

**Implementation branch**: `impl/002-kernel-durable-session-spine`  
**PR**: #3 — OPEN / DRAFT  
**Canonical base**: `main@cfcc90f452e7115bfb104f886e09c309a5d57a1c`  
**Base tree**: `da65a0ae907a53212bbfc7afed1a25e7f4aa4636`  
**Started**: 2026-08-24  
**Latest code repair head before this status mutation**: `77c2f160ca0cdc5cf60d3e91170c6a1472dbf05b`  
**State**: `SPEC_002_REPAIR_QUALIFICATION_IN_PROGRESS`.

> Exact live GitHub truth is authoritative. No prior CI or review result transfers across a branch mutation. The exact final Draft head containing all repair and closeout evidence must pass the complete qualification workflow and have no unresolved material authorized-review findings before Spec 002 implementation closeout may be claimed.

## Current gate summary

```text
T002-001..078=IMPLEMENTED_BUT_REPAIR_QUALIFICATION_REOPENED
WAIVER_TAKEN=NO

PR_3_DRAFT=YES
PR_READY=NO
MERGED=NO
SPEC_002_IMPLEMENTATION_COMPLETE=NO
SPEC_002_CLOSED_CANONICAL=NO
SPEC_003_AUTHORIZED=NO

LATEST_CODE_REPAIR_HEAD=77c2f160ca0cdc5cf60d3e91170c6a1472dbf05b
LATEST_CODE_REPAIR_CI=PENDING
AUTHORIZED_REVIEW_MATERIAL_FINDINGS=PENDING_REQUALIFICATION
CODEX_REVIEW_GATE=EXCLUDED_BY_FOUNDER_DIRECTION
```

## Why qualification was reopened

Fresh authorized review on the stable Draft implementation found material Spec 002 correctness, security and reliability gaps. Two test-only findings were rejected as non-actionable because the canonical test strategy explicitly requires a subprocess kill/restart harness and real disk-full scenarios in addition to deterministic fault injection. The material code findings were repaired on code head `77c2f160ca0cdc5cf60d3e91170c6a1472dbf05b`.

The repair set includes:

- enforce the frozen Spec 002 effect FSM at the generic `EffectStore::compare_and_swap` boundary so callers cannot persist arbitrary recognized-state edges;
- require exact content-addressed artifact receipt paths and reject traversal/symlink escapes;
- reject symlink components before granting an unprivileged runtime path;
- derive protocol incident identifiers from unique durable incident material so repeated rejections on one connection remain append-only and auditable;
- verify effective Unix ownership for the protected runtime tree in addition to permission bits;
- bound foreground bootstrap approval instead of permitting an unbounded stdin wait;
- bound CLI handshake/request/reply I/O with one absolute local IPC deadline;
- cap unauthenticated server-advertised handshake limits at the local client ceiling before those limits can influence subsequent frame allocation;
- admit an interrupted durable `executing` effect into `unknown_outcome -> reconciling` without redispatching the attempt;
- permit interrupted `reconciling` work to resume from durable context rather than becoming permanently stuck.

## Preserved Spec 002 behavior

### Protected local authority and canonical state

- Seven-package Rust 1.98 workspace; Golam product crates forbid unsafe code.
- Protected runtime/authority subtree with platform permission verification and generic/unprivileged path exclusion.
- SQLite WAL + `synchronous=FULL`, forward-schema refusal, quick-check and canonical integrity verification.
- Transactional canonical ordering, deterministic BLAKE3 event/session audit material, append-versioned goals, immutable forks, content-addressed artifacts and verified checkpoints.
- Authority corruption fails closed without silent reset.

### Mandatory security integrity

The security-critical canonical SessionEvent chain is reinforced by an independent `authority-security` chain for protected non-event authority state:

- client enrollment/revocation;
- authorization decisions;
- effect intents/transitions;
- effect attempt start/finish;
- recovery/protocol/manual-review incidents.

Authority-store open verifies complete source-row coverage, canonical source hashes, chain continuity and chain head. Protected source rows without required integrity coverage fail closed.

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
- reserved canonical event families remain owned by typed domain operations rather than an arbitrary caller-selected reserved-event append surface.

### Effects / recovery

- frozen Spec 002 effect FSM enforced at generic CAS and typed domain operations;
- deterministic handlers for all five execution semantics;
- durable intent and attempt/EXECUTING evidence before dispatch proof;
- `UNKNOWN_OUTCOME` dependency blocking and no blind duplicate for `AT_MOST_ONCE`/`IRREVERSIBLE`;
- interrupted `executing` state can be converted to durable unknown outcome for reconciliation without redispatch;
- reconciliation can resume after interruption and escalate durably to manual review;
- real OS process-kill/restart regression and real SQLite FULL rollback regression remain required substrate evidence;
- `RecoveryScanner` distinguishes Normal, RecoveryOnly and Quarantined states and blocks privileged service when required.

## Exact-head qualification required

The final Draft head must pass on Ubuntu, macOS and Windows:

- `cargo fmt --all -- --check`;
- `cargo clippy --locked --workspace --all-targets -- -D warnings`;
- `cargo test --locked --workspace --all-targets`;
- property qualification;
- bounded fuzz smoke;
- platform IPC qualification;
- authenticated daemon IPC qualification;
- adversarial authority qualification;
- daemon build and external strict-local no-network observation.

No `PASS`, `SPEC_002_IMPLEMENTATION_COMPLETE`, or closeout claim is valid until those gates succeed on the exact final head.

## Review policy

- Codex review is explicitly excluded from the Golam review workflow by founder direction and is not an implementation finding source or closeout gate.
- Authorized review findings are handled on their exact reviewed head; stale review results do not transfer to a new head.
- A fresh authorized external review may be requested only after the repair head is stable and exact-head CI succeeds.
- Any new material finding reopens repair and exact-head qualification.

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
