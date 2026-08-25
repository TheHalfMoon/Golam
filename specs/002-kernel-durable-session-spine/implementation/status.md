# Spec 002 Implementation Status

**Implementation branch**: `impl/002-kernel-durable-session-spine`  
**PR**: `#3` — OPEN / DRAFT  
**Canonical base**: `main@cfcc90f452e7115bfb104f886e09c309a5d57a1c`  
**Base tree**: `da65a0ae907a53212bbfc7afed1a25e7f4aa4636`  
**Started**: 2026-08-24  
**Last reconciled proven code head**: `13b222175eda9c760cd8581c879ccde1020af6f4`  
**Exact-head CI**: GitHub Actions run `32798308181` / run number `32` — SUCCESS on Windows, macOS, Ubuntu for `cargo fmt --check`, `clippy -D warnings`, and workspace tests.

> This status document is versioned on the same implementation branch. The live GitHub PR head always overrides the snapshot above if the branch advances after this document is committed.

## Gate summary

```text
T002-001_EXACT_MAIN_AND_BRANCH=PASS
T002-002_RUST_WORKSPACE=PASS
T002-003_BASELINE_CI=PASS
T002-010_SOURCE_CODE_ADMISSION=SATISFIED_NA_NO_SOURCE_CODE_COPIED
T002-011_GOLAM_RESEARCH_BEHAVIOR_MAP=PASS
T002-012_DEPENDENCY_QUALIFICATION=PASS

T002-020_CORE_TYPES_CANONICAL_ENCODING=PASS
T002-021_PROTECTED_RUNTIME_PATHS=PARTIAL_WINDOWS_ACL_PENDING
T002-022_SQLITE_SCHEMA=PASS
T002-023_SEQUENCE_AND_HASH_CHAINS=PASS
T002-024_STARTUP_INTEGRITY_RECOVERY_MODE=PARTIAL_EXPLICIT_RECOVERY_MODE_PENDING
T002-025_CONTENT_ADDRESSED_ARTIFACTS=PASS
T002-026_CHECKPOINT_VERIFY_FALLBACK=PASS
T002-027_IMMUTABLE_SESSION_FORKS=PASS
T002-028_APPEND_VERSIONED_GOALS=PASS

T002-030_IPC_FRAME_CODEC=PASS
T002-031_AUTHENTICATED_LIFECYCLE=PASS
T002-032_UNIX_SOCKET_PEER_AUTH=PENDING
T002-033_WINDOWS_PIPE_SID_ACL=PENDING
T002-034_CLIENT_ENROLLMENT=PENDING
T002-035_REQUEST_CANCEL_SETTLEMENT=PENDING
T002-036_IPC_ADVERSARIAL_SUITE=PENDING

SPEC_002_CLOSED=NO
PR_READY=NO
MERGE_AUTHORITY_TAKEN=NO
SPEC_003_AUTHORIZED=NO
```

## What is proven now

### Durable canonical session spine

- Seven-package Rust 1.98 workspace with `unsafe_code = forbid` in Golam code.
- SQLite schema v1 with future-schema refusal, WAL/FULL durability settings, `PRAGMA quick_check`, canonical event/audit verification and no silent reset.
- Transactional global/per-session sequencing and domain-separated deterministic BLAKE3 event/audit hashes.
- Content-addressed artifact store with temp-write, sync, verify, atomic no-clobber install and cleanup.
- Checkpoints anchored to a canonical prefix; checkpoint artifacts remain non-authority accelerators; missing/corrupt artifact falls back to canonical replay.
- Immutable fork anchors `(parent_session_id, parent_session_seq, parent_event_hash)` with parent continuation and DB mutation guard.
- Append-versioned Goal Ledger with stale-version/head rejection, atomic GoalVersion+canonical-event commit, append-only DB protection and canonical verification.

### IPC framing and authenticated lifecycle

- Fixed 20-byte `GIPC` header with explicit magic, protocol version, kind, flags, optional request id and payload length.
- Unknown kind/version/flags, non-canonical request-id bytes, missing/unexpected request IDs, truncated frames, trailing bytes and oversized declared frames fail closed.
- Fixed explicit lifecycle payload codecs for `HELLO`, `CHALLENGE`, `AUTHENTICATE`, `READY`, `SHUTDOWN`; no generic serializer defines the authentication bytes.
- Server lifecycle is fail-closed: repeated/out-of-order lifecycle, wrong signature, stale epoch, wrong client nonce or key ID cannot reach READY.
- Ed25519 transcript verification uses exact-pinned `ed25519-dalek 3.0.0` with `default-features = false`, `signature + zeroize`, and `verify_strict`.
- Signed canonical transcript binds protocol version, client ID, client nonce, server nonce, server epoch, negotiated resource limits and key ID.
- T002-031 accepts an already-enrolled verifying key; key generation/enrollment/storage remains T002-034.

## Reliability finding resolved during Phase D

macOS CI exposed test temp-directory collisions that could make otherwise-correct slices flaky. Test helpers in protected-path and artifact modules were hardened with PID + timestamp + atomic counter uniqueness. Exact-head run #29 proved the repair cross-platform before T002-031 was accepted. This is test isolation only; production path/artifact semantics were not weakened.

## Known open gaps — do not silently upgrade

### T002-021 Windows path protection

Unix/macOS directory privacy is enforced and verified with mode `0700`. Windows currently returns `AuthorityProtectionUnverified` for authority readiness. This is intentional fail-closed behavior. Current-user SID ACL enforcement/verification must be implemented with T002-033; parsing localized shell output is not an acceptable substitute.

The original plan also describes an explicit authority-state subdirectory. Current `RuntimeLayout` establishes root/data/runtime/artifact isolation but has not yet completed that authority-directory convergence. T002-021 remains PARTIAL until the plan/security boundary is fully satisfied.

### T002-024 recovery-only mode

Startup DB quick-check and canonical integrity verification already fail closed on corruption. An explicit recovery-only/quarantine operational mode is not yet implemented and is therefore not claimed. T002-060 owns that remaining behavior.

### Source reuse

No Golam-Research/donor source code has been copied or ported in Spec 002 so far. The Source Foundry artifact is a behavior/semantics map only. Before any source-code reuse, T002-010 must be re-entered for the exact files/components.

## Exact next execution order

1. **T002-032** Unix-domain socket transport, private socket, peer UID/PID checks.
2. **T002-033** Windows named pipe, current-user SID ACL and peer metadata; close the Windows half of T002-021 here.
3. **T002-034** local client enrollment/revocation and qualified key storage/fallback.
4. **T002-035** request/reply IDs, cancellation, bounded pending calls, protocol-breach settlement.
5. **T002-036** adversarial authenticated-IPC suite.
6. Phase E kernel/bootstrap authorization.
7. Phase F persistent effect engine and crash semantics.
8. Phase G recovery/CLI/process-kill/disk-full work.
9. Phase H final qualification, Spec Kit convergence and closeout.

## Hard scope boundary

Spec 002 remains model-free and external-effect-free except deterministic simulators later in Phase F. It does not authorize models, real model inference, broad filesystem/shell/browser product tools, Desktop, GolamConnect, cloud relays, real email/deploy/payment effects, or starting Spec 003.

PR #3 remains Draft. No merge/Ready transition is implied by a generic continuation instruction.
