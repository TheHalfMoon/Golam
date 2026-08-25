# Dependency Qualification — Spec 002

**Checked**: 2026-08-25  
**Rust toolchain**: 1.98.0  
**Rule**: pin exact production versions during Spec 002; minimize the trusted dependency surface; dependencies do not receive Golam authority merely because they are Rust.

## Decisions

| Need | Candidate | Decision | Rationale / boundary |
|---|---|---|---|
| SQLite | `rusqlite 0.40.2` | ADMITTED | MIT wrapper; `libsqlite3-sys` is an explicit FFI boundary. `bundled` is enabled for deterministic cross-platform SQLite; vendored SQLite is public domain. All DB authority semantics remain in `golam-ledger`. |
| Integrity hash | `blake3 1.8.7` | ADMITTED | Official BLAKE3 implementation. Exact pin with `pure` avoids assembly/C optimized paths while canonical vectors are frozen. |
| IPC transcript signatures | `ed25519-dalek 3.0.0` | ADMITTED_T002_031 | BSD-3-Clause, pure-Rust Ed25519, MSRV 1.85. Exact pin; defaults disabled; only `signature` + `zeroize`. Golam uses `verify_strict`; no RNG/key generation/storage is admitted here. |
| Unix peer credentials | `nix 0.31.3` | ADMITTED_T002_032_UNIX_ONLY | MIT, MSRV 1.69, actively maintained safe wrappers around Unix APIs. Exact pin, defaults disabled, only `socket` + `user`. Declared as a target-`cfg(unix)` dependency of `golam-ipc`; Windows does not build/link it. Linux uses kernel `SO_PEERCRED` through `PeerCredentials`; Apple uses `LOCAL_PEERCRED` + `LOCAL_PEERPID`. Golam contains no direct `unsafe`/`libc` peer-credential calls. |
| Async runtime | `tokio 1.51.4` | QUALIFIED_CANDIDATE_NOT_ADMITTED | Not needed by T002-032: the first local transport is deliberately synchronous `std::UnixListener/UnixStream`. Re-evaluate only when bounded daemon multiplexing requires async I/O. |
| Generic serialization | `serde 1.0.229` | QUALIFIED_CANDIDATE | Does not define canonical ledger/authentication bytes. |
| IPC wire encoding | `postcard 1.1.3` | EVALUATED_NOT_NEEDED_T002_030_032 | Current IPC frame/lifecycle payloads use explicit Golam-owned fixed binary formats. |
| IDs | `uuid 1.25.0` | EVALUATED_NOT_ADMITTED | Typed `u128` IDs remain sufficient. |
| Errors | `thiserror 2.0.20` | EVALUATED_NOT_ADMITTED | Current bounded error enums use std only. |
| Property tests | `proptest 1.11.0` | QUALIFIED_DEV_CANDIDATE | Admit when final property suite lands. |
| Fuzzing | `libfuzzer-sys 0.4.13` via `cargo fuzz` | QUALIFIED_TOOL_CANDIDATE | Tooling only, never production dependency. |

## Current source evidence

- rusqlite 0.40.2: https://docs.rs/crate/rusqlite/0.40.2
- BLAKE3 1.8.7: https://docs.rs/crate/blake3/1.8.7
- ed25519-dalek 3.0.0: https://docs.rs/crate/ed25519-dalek/3.0.0
- nix 0.31.3 package/MSRV/license/features: https://docs.rs/crate/nix/0.31.3/source/Cargo.toml
- nix Apple/Linux socket options: https://docs.rs/crate/nix/0.31.3/source/src/sys/socket/sockopt.rs
- Rust 1.98 UnixStream peer credentials remain nightly-only: https://doc.rust-lang.org/beta/std/os/unix/net/struct.UnixStream.html
- Tokio 1.51.4: https://docs.rs/crate/tokio/latest
- Serde 1.0.229: https://docs.rs/crate/serde/1.0.229
- Postcard 1.1.3: https://docs.rs/crate/postcard/1.1.3
- proptest 1.11.0: https://docs.rs/crate/proptest/1.11.0
- libfuzzer-sys 0.4.13: https://docs.rs/crate/libfuzzer-sys/0.4.13

## Security / unsafe boundary

`#![forbid(unsafe_code)]` remains mandatory in Golam crates. Transitive/native unsafe boundaries are still recorded and minimized.

- SQLite C FFI belongs only behind `golam-ledger::storage`.
- BLAKE3 uses `pure`.
- `ed25519-dalek` is confined to `golam-ipc` transcript verification with strict verification and no hazardous/legacy/RNG surfaces enabled.
- `nix` is confined to the Unix IPC transport adapter. Golam itself does not call `libc` or write unsafe peer-credential code. Only the `socket` and `user` feature sets are admitted, and the dependency is target-Unix only.
- Tokio remains unadmitted because T002-032 can be proven synchronously.
- fuzzing dependencies are development tooling only.

## Canonical encoding decision

Security-critical ledger hashes and IPC authentication transcripts MUST NOT depend on serializer incidental representation. Spec 002 owns explicit versioned canonical encodings with domain separation, fixed field order, big-endian fixed-width integers, and explicit byte-string length encoding where variable data exists.

T002-031 transcript binds protocol/client/nonces/server epoch plus negotiated limits and client key ID. T002-032 does not alter those authentication bytes; OS peer identity is an independent input that must be combined with cryptographic enrollment by later connection composition.

## Unix transport decision

T002-032 uses standard-library Unix stream/listener I/O and a narrow safe `nix` credential adapter. Socket path is fixed under the verified private runtime directory; a pre-existing path is not automatically unlinked; socket mode is set and checked at `0600`; parent runtime directory remains `0700`. Accepted peers must match the daemon's effective UID, and Linux/macOS must supply a positive kernel-reported PID where supported.

The std `UnixStream::peer_cred` API is not used because it remains nightly-only in Rust 1.98. No TCP/HTTP listener and no async runtime are introduced by this task.

## SQLite durability posture

The SQLite spine remains fail-closed with canonical event/audit verification. Explicit recovery-only/quarantine serving mode, disk-full reserve qualification and effect crash ambiguity remain separate tasks.
