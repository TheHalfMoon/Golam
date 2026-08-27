# Spec 002 Implementation Closeout

**Spec**: `002-kernel-durable-session-spine`  
**Implementation branch**: `impl/002-kernel-durable-session-spine`  
**PR**: #3 — OPEN / DRAFT  
**Canonical base**: `main@cfcc90f452e7115bfb104f886e09c309a5d57a1c`  
**Base tree**: `da65a0ae907a53212bbfc7afed1a25e7f4aa4636`  
**Final candidate head**: the commit containing this reconciled closeout package.

## Closeout decision

```text
T002_001_TO_078=IMPLEMENTED
TASK_IMPLEMENTATION=COMPLETE
SPEC_002_IMPLEMENTATION_COMPLETE=NO
FINAL_EXACT_HEAD_CI=PENDING
FINAL_POST_CI_QODO=PENDING
WAIVER_TAKEN=NO
PR_DRAFT=YES
PR_READY=NO
PR_MERGED=NO
SPEC_002_CLOSED_CANONICAL=NO
SPEC_003_AUTHORIZED=NO
CODEX_REVIEW_GATE=EXCLUDED_BY_FOUNDER_DIRECTION
```

The bounded Spec 002 task implementation exists, but implementation closeout is intentionally **not** claimed until the same final candidate head passes the complete cross-platform CI matrix and then receives a fresh authorized Qodo review with no unresolved material finding.

No prior CI or review result transfers across the documentation mutation containing this file.

## Repair history retained as evidence

The final implementation includes the earlier convergence fixes plus the authorized Qodo repair cycle. The latest behavioral repair head is `acdce817dbd89d8286fc08b8821aded0b7dbf8f7`; the formatting-only repair head is `1a0ef2c1a72056e42528521ec63e5acaabc297f0`.

The repair set closes the material boundaries around:

- frozen effect FSM enforcement;
- atomic checkpoint event/metadata/session-head/security evidence;
- artifact traversal and symlink containment;
- unprivileged path symlink escape;
- protocol incident uniqueness;
- Unix protected-path ownership;
- foreground bootstrap approval boundedness;
- CLI IPC boundedness;
- unauthenticated challenge allocation limits;
- interrupted executing-effect reconciliation entry;
- durable reconciliation resume;
- manual-review transition discipline.

Two Qodo test-only rule findings were resolved as non-actionable because Spec 002 explicitly requires real subprocess kill/restart and real SQLite `SQLITE_FULL` substrate qualification in addition to deterministic fault injection.

## Constitutional / scope closure

### Local ownership and strict locality

- core authority/session/effect state is local canonical state;
- no model, provider, cloud account or external service is required;
- no TCP/HTTP control listener is introduced;
- kernel strict-local egress authorization is a hard deny in Spec 002;
- the CI workflow includes external observation for absence of Golam Internet sockets while local IPC is serving.

### Rust trusted path / privileged kernel

- the implementation remains within the seven-package Rust spine;
- Golam product crates forbid unsafe code;
- SQLite/OS unsafe boundaries remain qualified dependency boundaries;
- KernelApi owns privileged authority mutation and returns outcomes/proofs rather than public mintable grants;
- generic/unprivileged path admission cannot address the protected authority subtree or follow unsafe symlink components.

### Authentication / IPC

- local transport identity and cryptographic authentication remain independent requirements;
- UDS/macOS peer credentials and Windows current-user named-pipe ACL/peer metadata are platform-specific qualification boundaries;
- Hello -> Challenge -> Authenticate -> Ready is enforced;
- unauthenticated challenge limits are bounded by local client ceilings before subsequent frame allocation;
- request/reply IDs, cancellation and pending limits fail closed;
- daemon and CLI sides both have bounded local IPC waits.

### Durable canonical session spine

- sessions/events/goals/forks/checkpoints use deterministic canonical material and transactional order;
- checkpoint canonical event, artifact metadata, checkpoint row, session head and security audit evidence are one SQLite transactional boundary;
- checkpoints remain accelerators, never replacements for canonical history;
- replay/checkpoint equivalence and fork-anchor immutability are property-qualified;
- reserved system event families are emitted only through owning typed domain paths rather than a public arbitrary reserved `EventKind` append surface.

### Mandatory integrity

Two integrity domains are explicit:

1. security-critical canonical `SessionEvent` chain;
2. `authority-security` chain for protected non-event authority records.

The authority-security chain covers client enrollment/revocation, authorization decisions, effect intents/transitions/attempt starts/finishes, and recovery/protocol/manual-review incidents. Authority-store open verifies complete source-row coverage, canonical hashes, chain linkage and chain head.

### Effect safety

- deterministic handlers cover the five Spec 002 execution semantics;
- generic CAS enforces the frozen FSM;
- effect intent and attempt/EXECUTING evidence commit before dispatch proof is returned;
- UNKNOWN_OUTCOME blocks dependent effects;
- AT_MOST_ONCE and IRREVERSIBLE do not blind-retry after ambiguous restart;
- interrupted executing effects are converted to durable unknown outcome before reconciliation without redispatch;
- durable `reconciling` context can resume after interruption;
- unresolved ambiguity may enter manual review only from `reconciling`.

### Recovery and disk pressure

- RecoveryScanner distinguishes normal service, recovery-only and quarantine conditions;
- privileged serving is blocked when recovery state requires it;
- authority corruption is not silently reset;
- real SQLite FULL and real process-kill/restart regressions are retained as required substrate evidence alongside deterministic fault injection;
- `NO_RECOVERY_RESERVE_GUARANTEE` remains the tested recovery-reserve decision.

## GolamBench foundation evidence

`implementation/bs1-bs2-qualification.md` contains historical BS-1/BS-2 evidence. Those results remain implementation evidence but do not substitute for final exact-head CI on this closeout package.

## Source / dependency posture

No donor source code was copied, ported, vendored or admitted as a donor dependency in Spec 002. Reviewed external projects remained semantics/architecture evidence only. Per-file Source Foundry admission therefore remained not applicable and reopens before any future source-code reuse.

The dependency qualification record remains the authority for exact third-party crate boundaries and unsafe/FFI/platform considerations.

## Review state

```text
QODO_REPAIR_CYCLE=RESOLVED_ON_PRE_RECONCILIATION_HEAD
QODO_FINAL_POST_CI_REVIEW=PENDING
CODEX_REVIEW=EXCLUDED_BY_FOUNDER_DIRECTION
CODERABBIT=NOT_AUTHORIZED_AS_QODO_REPLACEMENT
EXTERNAL_REVIEW_PASS_CLAIMED=NO
```

Codex review must not be requested or used as a fallback gate for Golam. The fresh final external review is Qodo and must occur only after exact-head CI succeeds on the unchanged final candidate head.

Any material Qodo finding reopens repair and requires another complete exact-head CI cycle after the fix.

## Final exact-head rule

The commit containing this file, the reconciled `tasks.md`, checklist, convergence record and status must pass the complete workflow on Windows, macOS and Ubuntu:

- format;
- Clippy with warnings denied;
- full workspace tests;
- property qualification;
- bounded fuzz smoke;
- platform IPC qualification;
- authenticated daemon IPC qualification;
- adversarial authority qualification;
- daemon build;
- external strict-local no-network observation.

Only after that run succeeds and the fresh post-CI Qodo review is clean may PR evidence state:

```text
SPEC_002_IMPLEMENTATION_COMPLETE=YES
FINAL_EXACT_HEAD_CI=PASS
FINAL_POST_CI_QODO=PASS
```

Even then:

```text
PR_DRAFT=YES
PR_READY=NO
MERGED=NO
SPEC_002_CLOSED_CANONICAL=NO
SPEC_003_AUTHORIZED=NO
```

until separate founder/bootstrap lifecycle authority is taken and the implementation is actually merged into canonical `main`.

## PR lifecycle guardrail

This closeout task does not authorize marking PR #3 Ready, merging it, deleting the implementation branch, declaring `CLOSED_CANONICAL`, or starting Spec 003.
