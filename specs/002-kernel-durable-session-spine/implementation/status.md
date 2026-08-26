# Spec 002 Implementation Status

**Implementation branch**: `impl/002-kernel-durable-session-spine`  
**PR**: #3 — OPEN / DRAFT  
**Canonical base**: `main@cfcc90f452e7115bfb104f886e09c309a5d57a1c`  
**Base tree**: `da65a0ae907a53212bbfc7afed1a25e7f4aa4636`  
**Started**: 2026-08-24  
**Last fully qualified code head before closeout-document reconciliation**: `29d54ca211e17c7bbcc0b2febfc2349d7b9ed2be`  
**Exact code-head CI**: GitHub Actions run ID `32958260286`, run number `225` — SUCCESS on Windows, macOS and Ubuntu for the complete qualification workflow.  
**Current state**: `IMPLEMENTATION_COMPLETE_PENDING_FINAL_DOCUMENTATION_HEAD_CI_AND_PR_LIFECYCLE`.

> Live GitHub truth overrides this file if the branch advances. A later documentation/task head requires its own exact-head CI before final PASS is recorded.

## Gate summary

```text
T002-001..060=PASS
T002-061_RECOVERY_RESERVE_EVALUATION=PASS_NO_UNPROVEN_RESERVE_GUARANTEE
T002-062_MINIMAL_AUTHENTICATED_CLI=PASS_CODE_HEAD_29d54ca
T002-063_PROCESS_KILL_DISK_FULL_CORRUPTION=PASS_CODE_HEAD_29d54ca

T002-070_FMT_CLIPPY_TEST=PASS_CODE_HEAD_29d54ca
T002-071_PROPERTY_QUALIFICATION=PASS_CODE_HEAD_29d54ca
T002-072_BOUNDED_FUZZ_SMOKE=PASS_CODE_HEAD_29d54ca
T002-073_PLATFORM_IPC_MATRIX=PASS_CODE_HEAD_29d54ca
T002-074_EXTERNAL_NO_NETWORK=PASS_CODE_HEAD_29d54ca
T002-075_BS1_BS2_ARTIFACT=RECORDED_PENDING_FINAL_DOC_HEAD_CI
T002-076_ADVERSARIAL_BOUNDARY=PASS_CODE_HEAD_29d54ca
T002-077_FINAL_CONVERGENCE=RECORDED_PENDING_FINAL_DOC_HEAD_CI
T002-078_CLOSEOUT=PENDING_FINAL_EXACT_HEAD

CODEX_REVIEW_RESULT=BLOCKED_USAGE_LIMIT_NO_REVIEW
CODERABBIT_PREVIOUS_RESULT=NOT_COMPLETED_HEAD_CHANGED
EXTERNAL_REVIEW_PASS_CLAIMED=NO

SPEC_002_IMPLEMENTATION_COMPLETE=YES_PENDING_FINAL_DOC_HEAD_CI
SPEC_002_CLOSED_CANONICAL=NO
PR_READY=NO
MERGE_AUTHORITY_TAKEN=NO
SPEC_003_AUTHORIZED=NO
```

## Implemented and qualified behavior

### Protected local authority and canonical state

- Seven-package Rust 1.98 workspace with `unsafe_code = forbid` in Golam product crates.
- Protected runtime/authority subtree with platform permission verification and generic/unprivileged path exclusion.
- SQLite WAL + `synchronous=FULL`, forward-schema refusal, quick-check and canonical integrity verification.
- Transactional global/per-session canonical ordering, deterministic BLAKE3 event/session audit material, append-versioned goals, immutable forks, content-addressed artifacts and verified checkpoints.
- Authority corruption fails closed without silent reset.

### Mandatory security integrity

The canonical session-event audit chain is reinforced by an independent `authority-security` chain for protected non-event state:

- client enrollment/revocation;
- authorization decisions;
- effect intents/transitions;
- effect attempt start/finish;
- recovery/protocol/manual-review incidents.

Authority-store open checks chain continuity, source-row hashes, chain head and complete coverage. Tampering or missing audit coverage is an integrity failure. Integration tests directly mutate authorization/effect/client/recovery rows and prove reopen is rejected.

### Authenticated local IPC / daemon

- Versioned bounded framing and strict Hello -> Challenge -> Authenticate -> Ready lifecycle.
- Ed25519 transcript authentication plus independent OS-local peer checks.
- Unix/macOS private UDS and peer credentials; Windows current-user ACL named pipe and peer metadata.
- request/reply IDs, cancellation settlement and bounded pending requests.
- accepted-connection deadline prevents a silent local peer from monopolizing the synchronous daemon indefinitely.
- no HTTP/TCP control listener.

### Kernel / bootstrap authority

- KernelApi is the privileged mutation boundary; sealed authority-bearing implementation types are not public.
- bootstrap authorization is deny-by-default and every decision is durable/audited.
- strict-local network egress is a hard monotonic denial.
- enrolled CLI can perform the bounded Spec 002 session/checkpoint/replay/synthetic-effect/reconciliation operations required by T002-062, but cannot enroll/revoke clients or acquire network authority.
- canonical events are emitted through typed domain operations; no public caller-selected reserved `EventKind` append API is exposed.

### Effects / recovery

- full Spec 002 effect state vocabulary with compare-and-swap transitions;
- deterministic handlers for five execution semantics;
- durable effect intent and attempt/EXECUTING transition before a dispatch proof is returned;
- UNKNOWN_OUTCOME dependency blocking and no blind duplicate for AT_MOST_ONCE/IRREVERSIBLE;
- read-only reconciliation and durable manual-review escalation;
- real process kill/restart regression and SQLite FULL rollback regression;
- RecoveryScanner reports Normal, RecoveryOnly or Quarantined states and blocks privileged service when required.

### CLI

The implemented CLI is deliberately bounded and low-level:

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

All normal commands cross authenticated local IPC. First client enrollment is an explicit foreground bootstrap flow.

`golam doctor` reads the kernel recovery report while privileged serving is allowed. RecoveryOnly/Quarantined startup itself reports the blocking state and does not create an unauthenticated diagnostic control plane.

## Qualification evidence

Exact code-head run #225 passed on Windows, macOS and Ubuntu:

- Format;
- Clippy with `-D warnings`;
- full workspace Test;
- Property qualification;
- Bounded fuzz smoke;
- platform IPC qualification;
- authenticated daemon IPC qualification;
- adversarial authority qualification;
- daemon build and externally observed strict-local no-network qualification.

See `implementation/bs1-bs2-qualification.md` for BS-1/BS-2 evidence and `implementation/convergence.md` for final cross-artifact reconciliation.

## Recovery reserve decision

`implementation/recovery-reserve-evaluation.md` records `NO_RECOVERY_RESERVE_GUARANTEE`. Spec 002 does not claim a cross-platform preallocated reserve that was not proven. The implementation instead proves fail-closed SQLite FULL behavior before dispatch authority and explicit recovery/quarantine behavior.

## Review state

A GitHub Codex review request was blocked by usage limits. There is no Codex finding set and no Codex PASS.

A CodeRabbit request was not completed because the PR head changed. That result is not a review and not a PASS. Any fresh external review required for Ready/merge must be performed on a stable final head; material findings must be resolved before lifecycle promotion.

## Remaining actions within current authorization

1. Qualify the final documentation/convergence head with the complete CI matrix.
2. Reconcile `tasks.md` only from that exact-head evidence.
3. Write the T002-078 closeout record and update the Draft PR body/comment with exact evidence.
4. Re-run exact-head CI after any final closeout-only mutation.
5. Keep PR #3 Draft and unmerged unless separate explicit founder authorization changes its lifecycle.

Spec 003 is not authorized until Spec 002 is merged and closed canonical.

## Hard scope boundary

Spec 002 remains model-free, cloud-free and real-external-effect-free. It does not authorize broad product tools, Desktop/computer control, GolamConnect, external channels, model/provider integration, real secrets, or Spec 003 policy/secrets/sandbox implementation.
