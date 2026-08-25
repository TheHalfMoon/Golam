# Spec 002 Implementation Status

**Implementation branch**: `impl/002-kernel-durable-session-spine`  
**PR**: `#3` — OPEN / DRAFT  
**Canonical base**: `main@cfcc90f452e7115bfb104f886e09c309a5d57a1c`  
**Base tree**: `da65a0ae907a53212bbfc7afed1a25e7f4aa4636`  
**Started**: 2026-08-24  
**Last reconciled proven code head**: `29be235de00d853a205ae2f46add1d08b91c1796`  
**Proven code tree**: `4915eb0ee62324ff5faf25184d33c5a13680e9b4`  
**Exact-head CI**: GitHub Actions run `32800522051` / run number `40` — SUCCESS on Windows, macOS, Ubuntu for `cargo fmt --check`, `clippy -D warnings`, and workspace tests.

> This status document is versioned on the implementation branch. The live GitHub PR head overrides this snapshot if the branch advances after the document commit.

## Gate summary

```text
T002-001_EXACT_MAIN_AND_BRANCH=PASS
T002-002_RUST_WORKSPACE=PASS
T002-003_BASELINE_CI=PASS
T002-010_SOURCE_CODE_ADMISSION=SATISFIED_NA_NO_SOURCE_CODE_COPIED
T002-011_GOLAM_RESEARCH_BEHAVIOR_MAP=PASS
T002-012_DEPENDENCY_QUALIFICATION=PASS

T002-020_CORE_TYPES_CANONICAL_ENCODING=PASS
T002-021_PROTECTED_RUNTIME_PATHS=PARTIAL_AUTHORITY_SUBDIRECTORY_CONVERGENCE_ONLY
T002-022_SQLITE_SCHEMA=PASS
T002-023_SEQUENCE_AND_HASH_CHAINS=PASS
T002-024_STARTUP_INTEGRITY_RECOVERY_MODE=PARTIAL_EXPLICIT_RECOVERY_MODE_PENDING
T002-025_CONTENT_ADDRESSED_ARTIFACTS=PASS
T002-026_CHECKPOINT_VERIFY_FALLBACK=PASS
T002-027_IMMUTABLE_SESSION_FORKS=PASS
T002-028_APPEND_VERSIONED_GOALS=PASS

T002-030_IPC_FRAME_CODEC=PASS
T002-031_AUTHENTICATED_LIFECYCLE=PASS
T002-032_UNIX_SOCKET_PEER_AUTH=PASS
T002-033_WINDOWS_PIPE_SID_ACL=PASS
T002-034_CLIENT_ENROLLMENT=PENDING
T002-035_REQUEST_CANCEL_SETTLEMENT=PENDING
T002-036_IPC_ADVERSARIAL_SUITE=PENDING

CODEX_CODE_REVIEW_REQUESTED=YES
CODEX_CODE_REVIEW_RESULT=BLOCKED_USAGE_LIMIT_NO_REVIEW
CODERABBIT_PREVIOUS_MANUAL_REVIEW_RESULT=NOT_COMPLETED_HEAD_CHANGED
EXTERNAL_REVIEW_PASS_CLAIMED=NO

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
- Content-addressed artifacts, canonical checkpoints with replay fallback, immutable fork anchors and append-versioned goals.

### IPC framing and authentication

- Strict bounded `GIPC` framing and explicit lifecycle wire formats.
- Fail-closed lifecycle state machine with strict Ed25519 transcript verification.
- OS transport identity and cryptographic enrollment are independent required inputs; neither alone is authority.

### Unix/macOS transport

- `0700` runtime directory, `0600` UDS, no stale-path auto-unlink, explicit `sun_path` bounds.
- Linux `SO_PEERCRED`; macOS `LOCAL_PEERCRED` + `LOCAL_PEERPID`; same-effective-UID enforcement and valid peer PID.
- No TCP/HTTP and no Tokio.

### Windows transport and path protection

- Every protected RuntimeLayout directory on Windows receives a protected DACL whose only ACE grants inheritable file-all access to the current process SID.
- Verification re-reads the DACL through `GetNamedSecurityInfoW`, requires exactly one allow ACE for the expected SID with file-all rights, and verifies protected-DACL SDDL.
- Windows now reaches `ProtectionLevel::UserOnlyVerified`; the previous `AuthorityProtectionUnverified` Windows state is closed by exact Windows CI.
- The named pipe is local-only (`accept_remote=false`), non-inheritable, instance-bounded, and created with a protected DACL granting access only to the current SID.
- Kernel-reported client PID/session metadata is captured and tested. PID/session remains metadata/identity evidence, not standalone authority.
- The SID embedded in the pipe name is discovery only; security comes from the pipe DACL plus independent cryptographic client authentication.
- `windows-permissions 0.2.4`, `interprocess 2.4.3`, and `widestring 1.2.1` are exact-pinned target-Windows dependencies with their unsafe/Win32 boundaries recorded in dependency qualification. Golam itself still forbids unsafe code.

## Review state

Official GitHub Codex Code Review was requested with `@codex review` and an IPC/security-focused prompt. The Codex connector reported that the current code-review usage quota was exhausted; therefore **no Codex findings and no Codex PASS exist**.

A CodeRabbit manual review was also requested, but it returned `Action not completed — Head commit changed` because implementation advanced while the request was being processed. This is not a review and not a PASS. Re-trigger CodeRabbit only on a stable post-T002-033 documentation head.

## Reliability findings resolved during Phase D

- macOS test temp-root collisions were repaired with stronger test uniqueness.
- macOS `sockaddr_un.sun_path` limits are now explicitly validated and tested.
- Windows path protection moved from intentional fail-closed placeholder to real current-user DACL application + OS re-verification.
- Windows named-pipe ACL and peer metadata are exercised on the real Windows CI runner, not inferred from cross-compilation.

## Known open gaps — do not silently upgrade

### T002-021 authority-state boundary

Cross-platform directory privacy is now proven on Unix/macOS and Windows. The remaining T002-021 gap is different: the plan describes a dedicated protected authority-state subtree, while the current `RuntimeLayout` still permits callers to choose the SQLite authority path. T002-042 must establish generic-tool exclusion/protected-resource semantics or the plan must be formally amended to an equally strong tested boundary. Until then T002-021 remains PARTIAL.

### T002-024 recovery-only mode

Startup integrity verification fails closed, but explicit recovery-only/quarantine operational mode remains T002-060.

### Source reuse

No donor source code has been copied or ported; Source Foundry remains behavior/semantics evidence only.

## Exact next execution order

1. **T002-034** explicit local client enrollment/revocation + qualified private-key storage/fallback.
2. **T002-035** request/reply IDs, cancellation, bounded pending calls, protocol-breach settlement.
3. **T002-036** adversarial authenticated-IPC suite.
4. Phase E kernel/bootstrap authorization.
5. Phase F persistent effect engine and crash semantics.
6. Phase G recovery/CLI/process-kill/disk-full work.
7. Phase H final qualification, Spec Kit convergence and closeout.

## Hard scope boundary

Spec 002 remains model-free and external-effect-free except deterministic simulators later in Phase F. It does not authorize models, broad product tools, Desktop, GolamConnect, cloud relays, real external effects, or starting Spec 003. PR #3 remains Draft.
