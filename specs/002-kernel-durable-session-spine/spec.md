# Feature Specification: Kernel & Durable Session Spine

**Feature Branch**: `spec/002-kernel-durable-session-spine`  
**Base**: `main@7f3e7f8d6fe75f96b0190cb856a84aa54700ff38`  
**Created**: 2026-08-24  
**Status**: PLANNING_COMPLETE_PENDING_PR_REVIEW

## Purpose

Spec 002 builds the smallest trustworthy local spine on which all later Golam features depend. It proves that a local daemon can authenticate clients, own authority-bearing state, maintain crash-resumable session/goal history, gate effects through durable transaction semantics, fork/replay sessions, and recover safely from process/storage failure before models or broad tools are introduced.

This is deliberately model-free. A deterministic test driver stands in for later harness/model code.

## Product slice

At the end of implementation, a user/developer can run `golamd` locally and use the Rust `golam` CLI to:

- enroll/authenticate as a local client;
- create/list/open/fork sessions;
- append typed events and goal versions through kernel APIs;
- create/verify checkpoints and replay canonical state;
- execute synthetic effect handlers covering all five execution-semantics classes;
- crash/restart at injected boundaries and recover without silent duplicate effects;
- inspect an audit/recovery report;
- prove the daemon exposes no unauthenticated network listener.

## User stories

### US1 — Authenticated local daemon

As a user, I can start `golamd` and connect through the CLI without exposing an unauthenticated localhost service.

Acceptance:
- Windows uses a user-scoped named-pipe transport; macOS/Linux use a user-scoped Unix-domain socket transport.
- Transport identity and cryptographic client enrollment are both checked before privileged requests are accepted.
- Malformed, unauthenticated, revoked, or replayed client sessions fail closed and are auditable.
- No non-loopback listener exists, and Spec 002 does not require HTTP.

### US2 — Durable canonical session history

As a user, I can resume a session after daemon restart without relying on transient chat/model context.

Acceptance:
- events are append-oriented, schema-versioned and ordered globally and per session;
- goal versions are protected durable records;
- canonical history survives compaction/checkpoint creation;
- replay from genesis and replay from a verified checkpoint produce equivalent projected state.

### US3 — Safe session fork

As a user, I can branch a session at a previous canonical point without mutating the parent history.

Acceptance:
- a fork records parent session, parent sequence and parent event hash;
- the parent prefix remains immutable;
- child events have their own sequence;
- replay/audit can prove the shared prefix and divergent suffix.

### US4 — Crash-safe effect semantics

As a user, I can trust Golam not to blindly repeat externally meaningful work after a crash.

Acceptance:
- effect intent is durably committed before handler dispatch;
- every handler declares semantics and reconciliation behavior;
- `UNKNOWN_OUTCOME` blocks dependent effects;
- `AT_MOST_ONCE` and `IRREVERSIBLE` effects do not blind retry;
- deterministic simulators prove crash windows around dispatch/ack/commit.

### US5 — Fail-closed storage recovery

As a user, corruption or disk pressure does not silently reset authority/audit history.

Acceptance:
- kernel DB integrity is checked at startup;
- authority-ledger corruption causes quarantine/fail-closed recovery mode, not best-effort row dropping;
- non-authority content-addressed artifacts may use bounded salvage policies later, but cannot redefine authority truth;
- disk-full before intent commit prevents dispatch;
- a reserved recovery mechanism permits recording/reconciling critical ambiguity where practical.

## Functional requirements

- **FR-001**: Implement a long-lived Rust `golamd` and Rust `golam` CLI with authenticated local IPC.
- **FR-002**: Initial workspace MUST contain at most seven real crates/binaries and no empty future scaffolding.
- **FR-003**: Privileged authority MUST be exposed only through an explicit process-splittable `KernelApi`; generic library access MUST NOT mint authority.
- **FR-004**: Kernel-owned storage paths/tables MUST be inaccessible through any generic data/tool API introduced in this spec.
- **FR-005**: Local IPC MUST use versioned typed frames with lifecycle handshake, challenge/authentication, request/reply IDs, cancellation, events and protocol-breach shutdown.
- **FR-006**: IPC MUST bound frame size, pending requests and concurrent clients; malformed or unsupported frames MUST fail closed.
- **FR-007**: The daemon MUST NOT expose unauthenticated HTTP/TCP control surfaces.
- **FR-008**: Canonical `SessionEvent` records MUST have global audit order, per-session order, schema version and deterministic integrity material.
- **FR-009**: Security-critical event families MUST participate in mandatory tamper-evident hash chaining.
- **FR-010**: Session forks MUST reference an immutable parent prefix rather than copying/mutating history.
- **FR-011**: Goal state MUST be append-versioned and linked to canonical events.
- **FR-012**: Checkpoints MUST reference a verified event prefix and content-addressed projection blob; corrupt checkpoints fall back to earlier valid checkpoint/full replay.
- **FR-013**: Effects MUST implement the Spec 001 state machine and five execution semantics classes.
- **FR-014**: Every effect handler MUST provide `prepare/idempotency`, `execute`, `reconcile` and timeout/retry policy metadata.
- **FR-015**: Effect intent MUST commit durably before external/simulated dispatch.
- **FR-016**: `UNKNOWN_OUTCOME` MUST be durable and MUST block dependent effects until reconciled/manual resolution.
- **FR-017**: Spec 002 MUST expose `Authorize(principal, action, resource, context)` with a deny-by-default bootstrap policy and stable semantics for Spec 003/Cedar replacement.
- **FR-018**: Strict-local egress MUST have a kernel-owned deny-by-default interface even though Spec 002 production code requires no external network.
- **FR-019**: Kernel DB startup MUST perform integrity/recovery checks and fail closed on unrecoverable authority corruption.
- **FR-020**: Event/effect writes MUST be transactional and crash-safe; no dispatch may occur after a failed intent commit.
- **FR-021**: Audit/recovery tooling MUST distinguish verified healthy state, recoverable checkpoint failure, ambiguous effect outcomes and unrecoverable authority-store corruption.
- **FR-022**: `Golam-Research` exact source evidence MUST be mined and mapped, but no bounded source component is admitted without a Source Foundry record.

## Non-functional requirements

- **NFR-001 Rust**: Spec 002 product code is Rust; `#![forbid(unsafe_code)]` in Golam crates unless an explicit reviewed exception exists. SQLite FFI is isolated behind the ledger/storage crate as a qualified dependency boundary.
- **NFR-002 Locality**: no cloud/model/service dependency.
- **NFR-003 Durability**: security/effect intent commits use durability settings strong enough that process crash/reboot cannot acknowledge a commit that was never made durable under the supported failure model.
- **NFR-004 Determinism**: replay from identical canonical bytes yields identical state.
- **NFR-005 Bounded resources**: IPC frames, event payloads, artifacts, checkpoints and queued requests have explicit limits.
- **NFR-006 Fail closed**: corruption/authentication/protocol uncertainty does not silently widen authority or reset canonical state.
- **NFR-007 Portability**: Windows 11, current macOS and major Linux distributions are test targets for local IPC/daemon lifecycle; platform-specific limitations are explicit.
- **NFR-008 No premature abstraction**: only the interfaces needed by Specs 003+ are created; no model/tool/desktop/connect crates are scaffolded.

## Success criteria

- **SC-001**: exact-head `cargo fmt`, `clippy -D warnings`, workspace tests and required property/fuzz tests pass on supported CI matrix when implementation is complete.
- **SC-002**: unauthenticated and replayed local client probes are rejected and audited.
- **SC-003**: fault injection at every effect transition proves no blind duplicate for at-most-once/irreversible simulators.
- **SC-004**: kill/restart during session/event/checkpoint writes preserves a valid prefix and deterministic replay.
- **SC-005**: fork property tests prove parent immutability and child causal anchor correctness.
- **SC-006**: corrupted checkpoint falls back safely; corrupted authority DB enters fail-closed recovery mode without silent reset.
- **SC-007**: external network scan shows no unexpected listening control ports; strict-local sinkhole test observes no Golam-managed egress.
- **SC-008**: a fault-injected unprivileged component cannot mint authority, alter protected kernel state or forge canonical audit entries.

## Out of scope

- Cedar policy engine and full capability schema (Spec 003).
- real secret vault/credential injection (Spec 003).
- arbitrary sandbox/plugin execution (Spec 003+).
- model inference/harness/context (Spec 004).
- filesystem/shell/git/browser product tools (Spec 005).
- Desktop/computer control (Spec 006).
- GolamConnect/channels (Spec 007).
- workers/scheduler product behavior (Spec 008).
- real payments/email/deployments or other consequential integrations.
