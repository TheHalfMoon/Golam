# Implementation Readiness / Closeout Checklist — Spec 002

## Scope

- [x] Model-free durable kernel slice is explicit.
- [x] The implementation remains within the seven-package Rust spine.
- [x] Out-of-scope future systems are explicit.
- [x] Golam-Research is treated as implementation evidence without forcing TypeScript architecture.

## Security boundary

- [x] Privileged KernelApi is process-splittable.
- [x] Protected-state enumeration and generic-path exclusion exist.
- [x] Local IPC transport + cryptographic enrollment are implemented independently.
- [x] No localhost HTTP/TCP control plane.
- [x] Bootstrap authorization is deny-by-default.
- [x] Strict-local egress interface is a hard deny for Spec 002 product behavior.
- [x] Security-critical canonical events and protected client/authorization/effect/recovery records have mandatory tamper-evident integrity coverage.
- [x] Unix protected runtime paths verify effective ownership as well as restrictive mode bits.
- [x] Artifact and unprivileged path admission reject traversal and symlink escapes.

## Durability

- [x] Global/per-session ordering implemented.
- [x] Fork semantics implemented and property-qualified.
- [x] Goal versions append.
- [x] Checkpoints do not replace canonical history and replay equivalence is qualified.
- [x] Checkpoint canonical event, artifact metadata, checkpoint row, session head and security audit head commit atomically.
- [x] Effect handler/reconciler contract implemented for deterministic simulators.
- [x] Frozen effect FSM is enforced at the generic CAS boundary.
- [x] Intent-before-dispatch implemented and crash-qualified.
- [x] Unknown outcome/dependency blocking implemented.
- [x] Interrupted executing effects enter reconciliation without redispatch.
- [x] Durable reconciling work can resume after interruption.
- [x] Manual review is admitted only from durable `reconciling` state.
- [x] Authority corruption fails closed without silent reset.
- [x] Disk-full before dispatch authority is exercised with real SQLite `SQLITE_FULL` behavior.
- [x] RecoveryOnly/Quarantined startup states block privileged service.

## IPC / reliability

- [x] Client-side unauthenticated challenge limits are bounded by local ceilings before subsequent frame allocation.
- [x] CLI handshake/request/reply operations have one absolute IPC deadline.
- [x] Foreground bootstrap approval has a bounded wait rather than an unbounded stdin block.
- [x] Repeated protocol rejections on one connection receive unique durable incident identities.

## Donor/provenance

- [x] Exact planning snapshots recorded for reviewed source candidates.
- [x] Founder permission posture recorded.
- [x] No donor source code admitted by the planning package.
- [x] Per-file Source Foundry admission is not applicable to Spec 002 implementation because no donor source code was copied, ported, vendored, or added as a donor dependency; this gate reopens before future source reuse.

## Verification substrate

- [x] Unit/property/fuzz/fault/platform tests implemented.
- [x] Unauthenticated local process/protocol probes implemented.
- [x] Kernel-boundary hostile-adapter probe implemented.
- [x] Duplicate-effect crash/restart suite implemented.
- [x] External no-network observation implemented on supported CI platforms.
- [x] Corruption/checkpoint/disk-full tests implemented.
- [x] Authority-security tamper and missing-coverage tests implemented.
- [x] Real subprocess kill/restart and real SQLite FULL qualification are retained alongside deterministic fault injection as required substrate evidence.

## Review / qualification state

- [x] Material Qodo findings from the repair cycle have been addressed or explicitly resolved as non-actionable where they conflict with the canonical substrate-test requirements.
- [x] Codex review is excluded from the Golam workflow by founder direction.
- [ ] Final exact-head CI succeeds on the commit containing this reconciled closeout package.
- [ ] Fresh authorized post-CI Qodo review on that unchanged exact head has no unresolved material finding.

## Closeout gate

```text
PLANNING_GATE=CLOSED_BY_MAIN_cfcc90f452e7115bfb104f886e09c309a5d57a1c
T002_001_TO_078=IMPLEMENTED
TASK_IMPLEMENTATION=COMPLETE
FINAL_CANDIDATE_HEAD=THIS_COMMIT
FINAL_EXACT_HEAD_CI=PENDING
FINAL_POST_CI_QODO=PENDING
SPEC_002_IMPLEMENTATION_COMPLETE=NO
PR_READY=NO
MERGED=NO
SPEC_002_CLOSED_CANONICAL=NO
SPEC_003_AUTHORIZED=NO
WAIVER_TAKEN=NO
```

A green prior code or documentation head does not qualify this later reconciled closeout mutation. The commit containing this checklist must pass the complete Windows/macOS/Ubuntu CI matrix, then receive the fresh authorized Qodo review on the unchanged head.

PR #3 remains Draft and requires separate founder authorization before Ready or merge. Spec 003 remains blocked until Spec 002 is merged and closed canonical.
