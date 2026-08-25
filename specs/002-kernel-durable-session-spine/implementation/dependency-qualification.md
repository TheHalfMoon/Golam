# Dependency Qualification — Spec 002

**Checked**: 2026-08-25  
**Rust toolchain**: 1.98.0  
**Rule**: pin exact production versions during Spec 002; minimize the trusted dependency surface; dependencies do not receive Golam authority merely because they are Rust.

## Decisions

| Need | Candidate | Decision | Rationale / boundary |
|---|---|---|---|
| SQLite | `rusqlite 0.40.2` | ADMITTED | MIT wrapper; `libsqlite3-sys` is an explicit FFI boundary. `bundled` is enabled for deterministic cross-platform SQLite 3.53.2; vendored SQLite is public domain. All DB authority semantics remain in `golam-ledger`. |
| Integrity hash | `blake3 1.8.7` | ADMITTED | Official BLAKE3 implementation. Exact pin with `pure` avoids assembly/C optimized paths while canonical vectors are frozen. Revisit performance only after correctness/security qualification. |
| IPC transcript signatures | `ed25519-dalek 3.0.0` | ADMITTED_T002_031 | BSD-3-Clause, pure-Rust Ed25519 implementation, edition 2024, MSRV 1.85. Exact pin; default features disabled; only `signature` + `zeroize` enabled. Golam uses `VerifyingKey::verify_strict`; `fast`, `rand_core`, `hazmat`, `legacy_compatibility`, `serde`, `pem`, `pkcs8`, and batch verification are not enabled. T002-031 verifies caller-supplied enrolled public keys only; secure key generation/storage/enrollment remains T002-034. |
| Async runtime | `tokio 1.51.4` | QUALIFIED_CANDIDATE, defer to IPC transport slice | Current Tokio LTS line through March 2027; MIT; MSRV 1.71. Use a minimal feature set rather than `full`. |
| Generic serialization | `serde 1.0.229` | QUALIFIED_CANDIDATE | MIT OR Apache-2.0. May support later typed values, but it does not define canonical ledger or authentication transcript bytes. |
| IPC wire encoding | `postcard 1.1.3` | EVALUATED_NOT_NEEDED_T002_030_031 | Stable documented wire format and Serde integration, but current IPC frame/lifecycle payloads use an explicit Golam-owned fixed binary format. Re-evaluate only if later message breadth justifies it. |
| IDs | `uuid 1.25.0` | EVALUATED_NOT_ADMITTED | Golam Spec 002 already uses typed `u128` IDs and does not need a UUID dependency. Re-evaluate only if an external protocol requires UUID semantics. |
| Errors | `thiserror 2.0.20` | EVALUATED_NOT_ADMITTED | Core errors are small enough to implement with `std::error::Error`; avoid a derive dependency until error breadth justifies it. |
| Property tests | `proptest 1.11.0` | QUALIFIED_DEV_CANDIDATE | MIT OR Apache-2.0; Rust 1.85 MSRV; admit when final replay/fork/effect properties land. |
| Fuzzing | `libfuzzer-sys 0.4.13` via `cargo fuzz` | QUALIFIED_TOOL_CANDIDATE | Apache-2.0/MIT wrapper plus NCSA libFuzzer material; Linux-only according to current docs. Tooling only, never a production dependency. |

## Current source evidence

- rusqlite 0.40.2: https://docs.rs/crate/rusqlite/0.40.2
- BLAKE3 1.8.7: https://docs.rs/crate/blake3/1.8.7
- ed25519-dalek 3.0.0 package/MSRV/license/dependencies: https://docs.rs/crate/ed25519-dalek/3.0.0/source/Cargo.toml
- ed25519-dalek 3.0.0 features/safety: https://docs.rs/crate/ed25519-dalek/3.0.0
- Tokio 1.51.4 LTS context: https://docs.rs/crate/tokio/latest
- Serde 1.0.229: https://docs.rs/crate/serde/1.0.229
- Postcard 1.1.3: https://docs.rs/crate/postcard/1.1.3
- UUID 1.25.0: https://docs.rs/crate/uuid/1.25.0
- thiserror 2.0.20: https://docs.rs/crate/thiserror/2.0.20
- proptest 1.11.0: https://docs.rs/crate/proptest/1.11.0
- libfuzzer-sys 0.4.13: https://docs.rs/crate/libfuzzer-sys/0.4.13

## Security / unsafe boundary

`#![forbid(unsafe_code)]` remains mandatory in Golam crates. It does not imply transitive dependencies contain no unsafe/FFI. Every admitted dependency with native code, FFI, assembly, process spawning, network behavior, or cryptographic authority is recorded explicitly and wrapped behind the narrowest Golam crate boundary.

For Spec 002:

- SQLite C FFI belongs only behind `golam-ledger::storage`.
- BLAKE3 is configured with `pure` to shrink the native-code surface.
- `ed25519-dalek` is confined to `golam-ipc` transcript verification. Golam enables strict verification and does not enable the crate's explicitly hazardous/legacy compatibility surfaces. No RNG or key generation dependency is admitted by T002-031.
- Tokio will not be admitted until T002-032/T002-033 require async transport I/O.
- fuzzing dependencies are development tooling only.

## Canonical encoding decision

Security-critical ledger hashes and IPC authentication transcripts MUST NOT depend on a serializer's incidental representation. Spec 002 owns explicit versioned canonical encodings with domain separation, fixed field order, big-endian fixed-width integers, and explicit byte-string length encoding where variable data exists. Any future schema/protocol change must bump the relevant version/domain and preserve frozen compatibility vectors.

For T002-031, the signed transcript is domain-separated and binds at least the contract-required protocol version, client ID, client nonce, server nonce, and server epoch. Golam additionally binds the negotiated resource limits and client key ID so a pre-auth intermediary cannot silently alter those values while retaining a valid signature.

## SQLite durability posture

The implemented SQLite spine configures WAL + FULL synchronization, forward-only schema version refusal, startup quick-check and canonical event/audit integrity verification. Event sequencing, checkpoint replay/fallback, immutable fork anchors and append-versioned goals are now implemented. Explicit recovery-only/quarantine serving mode, disk-full reserve qualification and effect crash ambiguity remain separate tasks and are not implied by database integrity PASS.
