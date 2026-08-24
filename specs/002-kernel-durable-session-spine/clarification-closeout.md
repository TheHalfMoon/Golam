# Clarification Closeout — Spec 002

**Date**: 2026-08-24  
**Decision**: CLARIFIED_FOR_PLAN

## C1 — Does v1 require a separate privileged kernel process?

**Decision**: No. Spec 002 permits a single `golamd` process, but the authority boundary MUST be enforceable in code/module ownership and exposed through a process-splittable typed `KernelApi`. Network/protocol parsers and clients remain outside authority-bearing modules. A later process split must not require semantic redesign.

## C2 — What is the initial crate boundary?

**Decision**: seven maximum, with six/seven expected:

- `golam-core` — pure IDs/types/errors/version contracts;
- `golam-ledger` — SQLite-backed sessions/events/goals/forks/checkpoints/artifact references;
- `golam-effects` — effect FSM/handler/reconciliation semantics;
- `golam-ipc` — local wire protocol, framing, transport/auth handshake;
- `golam-kernel` — protected authority API, bootstrap authorization, client enrollment/revocation and egress deny interface;
- `golamd` binary — composition root;
- `golam` binary — minimal CLI client.

No placeholder future crates.

## C3 — Local IPC transport

**Decision**: no HTTP/TCP control surface in Spec 002.

- Windows: named pipe scoped to the interactive user SID with peer process metadata checks where available.
- macOS/Linux: Unix-domain socket under a user-private runtime directory, mode 0600/parent 0700, with peer credential checks.
- All platforms: application-layer cryptographic challenge/response using enrolled client credentials; transport identity alone is insufficient.

Exact OS key storage backend is qualified during implementation. If a platform cannot protect a persistent client private key strongly enough, enrollment falls back to explicit per-run approval rather than unauthenticated convenience.

## C4 — Same-user malware threat

**Decision**: Spec 002 MUST block browser/DNS-rebinding, stray scripts and unenrolled local processes. It does not claim immunity to arbitrary code execution that has already fully compromised the user's OS account/keychain. The security claim must state this boundary honestly.

## C5 — Operational store

**Decision**: one protected SQLite authority/operational database for Spec 002 plus a protected content-addressed artifact directory. SQLite uses WAL where supported, foreign keys, busy timeout and strong synchronous mode for security/effect-intent transactions. Exact pragmas are verified by crash tests, not assumed safe by name alone.

## C6 — Corruption behavior

**Decision**: authority state is fail-closed. Unlike recoverable non-authority blob stores, Golam MUST NOT silently salvage/drop arbitrary authority rows and continue. On failed integrity/hash-chain verification, quarantine the store/read-only evidence, stop privileged operation, and require a verified recovery path.

## C7 — Hash chain

**Decision**: BLAKE3 is the initial candidate for deterministic event/hash-chain integrity. Hash input uses a domain-separated, versioned canonical byte encoding with frozen test vectors; do not hash ad-hoc debug JSON.

## C8 — Effect execution in this spec

**Decision**: deterministic simulator handlers only. They emulate remote accept/ack, idempotency support, query/reconcile support, ambiguous outcomes and irreversible operations without touching external services.

## C9 — Goal ledger

**Decision**: append-versioned goal records are canonical and transactionally linked to the event that created them. A current-goal projection may be rebuilt.

## C10 — Checkpoints

**Decision**: checkpoints optimize replay but never replace canonical history. A checkpoint contains/refers to projection bytes, event prefix cursor/hash and schema version; verification failure falls back to an earlier valid checkpoint/full replay.

## C11 — `Golam-Research`

**Decision**: treat exact Grok Bot 0.18 reconstruction as high-value implementation evidence. Mine protocol/lifecycle/persistence/recovery semantics. Because Golam is Rust/local-first and has a stronger authority model, prefer semantic ports/adaptations rather than line-by-line TypeScript translation. Record permission scope/evidence before any bounded code admission.

## C12 — Merge/implementation discipline

**Decision**: this planning PR remains separate from Rust implementation. `tasks.md` authorizes implementation only after the Spec 002 planning package is reviewed/merged from exact live truth.
