# Spec 002 Implementation Status

**Implementation branch**: `impl/002-kernel-durable-session-spine`  
**PR**: #3 — OPEN / DRAFT  
**Canonical base**: `main@cfcc90f452e7115bfb104f886e09c309a5d57a1c`  
**Base tree**: `da65a0ae907a53212bbfc7afed1a25e7f4aa4636`  
**Started**: 2026-08-24  
**Last exact convergence head proven before the final closeout-ledger package**: `a814e7d6a2b8610c9a54b96ae05c3df85335cee1`  
**Convergence CI**: GitHub Actions run ID `32958907240`, run number `233` — Windows/macOS/Ubuntu complete qualification workflow SUCCESS.  
**State**: `SPEC_002_IMPLEMENTATION_COMPLETE`.

> Exact-head rule: the branch head containing `implementation/closeout.md`, the final `tasks.md`, and this status file must also have a successful complete CI workflow before the Draft PR evidence may claim `FINAL_EXACT_HEAD_CI=PASS`. Live GitHub truth is authoritative.

## Gate summary

```text
T002-001..078=IMPLEMENTED
WAIVER_TAKEN=NO

BS-1=PASS
BS-2=PASS
BS-10_FOUNDATION=PASS_EXTERNALLY_OBSERVED_NO_NETWORK

CODEX_REVIEW_RESULT=BLOCKED_USAGE_LIMIT_NO_REVIEW
CODERABBIT_PREVIOUS_RESULT=NOT_COMPLETED_HEAD_CHANGED
EXTERNAL_REVIEW_PASS_CLAIMED=NO

SPEC_002_IMPLEMENTATION_COMPLETE=YES
PR_3_DRAFT=YES
PR_READY=NO
MERGED=NO
SPEC_002_CLOSED_CANONICAL=NO
SPEC_003_AUTHORIZED=NO
```

## Implemented behavior

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

Authority-store open verifies complete source-row coverage, canonical source hashes, chain continuity and chain head. Tampering or missing coverage rejects the authority store. The logical model explicitly records that the companion audit table may be absent only on a completely empty protected state; protected source rows without the table/coverage fail closed.

### Authenticated local IPC / daemon

- Versioned bounded `GIPC` framing and strict Hello -> Challenge -> Authenticate -> Ready lifecycle.
- Ed25519 transcript authentication plus independent OS-local peer checks.
- Unix/macOS private UDS with peer credentials; Windows current-user ACL named pipe with peer metadata.
- request/reply IDs, cancellation settlement, bounded pending requests and accepted-connection deadline.
- no HTTP/TCP control listener.

### Kernel / bootstrap authority

- KernelApi is the privileged mutation boundary; authority-bearing implementation types remain sealed.
- bootstrap authorization is deny-by-default and decisions are durable/audited.
- strict-local network egress is a monotonic hard denial.
- enrolled CLI is explicitly authorized for the bounded checkpoint/replay/synthetic-effect/reconciliation operations it exposes, but not client enrollment/revocation or network authority.
- reserved canonical event families are emitted through typed owning domain operations; no public arbitrary reserved `EventKind` append primitive is exposed.

### Effects / recovery

- full bounded Spec 002 effect FSM and compare-and-swap transitions;
- deterministic handlers for all five execution semantics;
- durable intent and attempt/EXECUTING evidence before dispatch proof;
- UNKNOWN_OUTCOME dependency blocking and no blind duplicate for AT_MOST_ONCE/IRREVERSIBLE;
- read-only reconciliation and durable manual-review escalation;
- real OS process-kill/restart regression and real SQLite FULL rollback regression;
- RecoveryScanner distinguishes Normal, RecoveryOnly and Quarantined states and blocks privileged service when required.

### CLI

The bounded CLI includes:

```text
golam client enroll <client-id>
golam sessions
golam session open ...
golam session create ...
golam session fork ...
golam goal append ...
golam checkpoint create ...
golam checkpoint verify ...
golam replay ...
golam effect simulate ...
golam effect reconcile ...
golam doctor
```

Normal commands cross authenticated local IPC. First enrollment uses explicit foreground bootstrap approval. `golam doctor` reads recovery status while privileged serving is allowed; RecoveryOnly/Quarantined startup reports its blocking state rather than exposing an unauthenticated recovery bypass.

## Qualification evidence

Run #233 on convergence head `a814e7d6...` passed on Windows, macOS and Ubuntu:

- Format;
- Clippy with `-D warnings`;
- full workspace Test;
- Property qualification;
- Bounded fuzz smoke;
- platform IPC qualification;
- authenticated daemon IPC qualification;
- adversarial authority qualification;
- daemon build and externally observed strict-local no-network qualification.

See:

- `implementation/bs1-bs2-qualification.md` for BS-1/BS-2 evidence;
- `implementation/convergence.md` for final constitutional/spec/contract reconciliation;
- `implementation/closeout.md` for T002-078 implementation closeout;
- `implementation/recovery-reserve-evaluation.md` for `NO_RECOVERY_RESERVE_GUARANTEE`.

## Review state

There are no submitted GitHub PR reviews and no inline review threads at this status snapshot.

A GitHub Codex review request was blocked by usage limits; there is no Codex PASS. A prior CodeRabbit request was not completed because the head changed; that is not a PASS. A fresh external review may be requested on the stable final Draft head. Any material finding must be resolved and all exact-head gates rerun.

## Lifecycle state

Spec 002 implementation work is complete, but ordinary implementation authority stops here.

```text
PR_READY_AUTHORITY=NOT_TAKEN
MERGE_AUTHORITY=NOT_TAKEN
CLOSED_CANONICAL=NO
SPEC_003_START=BLOCKED
```

Updating the Draft PR body/comment with exact final-head evidence and requesting a non-mutating review do not make the PR Ready and do not merge it. Any later branch mutation reopens exact-head qualification.

## Hard scope boundary

Spec 002 remains model-free, cloud-free and real-external-effect-free. It does not authorize broad product tools, Desktop/computer control, GolamConnect, external channels, model/provider integration, real secrets, or Spec 003 policy/secrets/sandbox implementation.
