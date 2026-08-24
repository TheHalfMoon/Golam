# Dependency Qualification — Spec 002

**Checked**: 2026-08-24  
**Rust toolchain**: 1.98.0  
**Rule**: pin exact production versions during Spec 002; minimize the trusted dependency surface; dependencies do not receive Golam authority merely because they are Rust.

## Decisions

| Need | Candidate | Decision | Rationale / boundary |
|---|---|---|---|
| SQLite | `rusqlite 0.40.2` | QUALIFIED_CANDIDATE, admit at T002-022 | MIT wrapper; `libsqlite3-sys` is an explicit FFI boundary. Prefer `bundled` for deterministic cross-platform SQLite 3.53.2, with vendored SQLite/public-domain notice recorded. All DB authority semantics stay in `golam-ledger`. |
| Integrity hash | `blake3 1.8.7` | ADMITTED_THIS_SLICE | Official BLAKE3 implementation. Pin exact version and enable `pure` for the early spine to avoid assembly/C optimized paths while canonical vectors are frozen. Revisit performance only after correctness/security qualification. |
| Async runtime | `tokio 1.51.4` | QUALIFIED_CANDIDATE, defer to IPC transport slice | Current Tokio LTS line through March 2027; MIT; MSRV 1.71. Use a minimal feature set rather than `full`. |
| Generic serialization | `serde 1.0.229` | QUALIFIED_CANDIDATE | MIT OR Apache-2.0. May support typed IPC/domain values, but it does not define canonical ledger bytes. |
| IPC wire encoding | `postcard 1.1.3` | QUALIFIED_CANDIDATE | Stable documented wire format and Serde integration. Admission waits for the typed frame implementation. Canonical ledger integrity encoding remains an explicit Golam-owned format. |
| IDs | `uuid 1.25.0` | EVALUATED_NOT_ADMITTED | Golam Spec 002 already uses typed `u128` IDs and does not need a UUID dependency. Re-evaluate only if an external protocol requires UUID semantics. |
| Errors | `thiserror 2.0.20` | EVALUATED_NOT_ADMITTED | Core errors are small enough to implement with `std::error::Error`; avoid a derive dependency until error breadth justifies it. |
| Property tests | `proptest 1.11.0` | QUALIFIED_DEV_CANDIDATE | MIT OR Apache-2.0; Rust 1.85 MSRV; admit when replay/fork/effect properties land. |
| Fuzzing | `libfuzzer-sys 0.4.13` via `cargo fuzz` | QUALIFIED_TOOL_CANDIDATE | Apache-2.0/MIT wrapper plus NCSA libFuzzer material; Linux-only according to current docs. Tooling only, never a production dependency. |

## Current source evidence

- rusqlite 0.40.2: https://docs.rs/crate/rusqlite/0.40.2
- BLAKE3 1.8.7: https://docs.rs/crate/blake3/1.8.7
- Tokio 1.51.4 LTS context: https://docs.rs/crate/tokio/latest
- Serde 1.0.229: https://docs.rs/crate/serde/1.0.229
- Postcard 1.1.3: https://docs.rs/crate/postcard/1.1.3
- UUID 1.25.0: https://docs.rs/crate/uuid/1.25.0
- thiserror 2.0.20: https://docs.rs/crate/thiserror/2.0.20
- proptest 1.11.0: https://docs.rs/crate/proptest/1.11.0
- libfuzzer-sys 0.4.13: https://docs.rs/crate/libfuzzer-sys/0.4.13

## Security / unsafe boundary

`#![forbid(unsafe_code)]` remains mandatory in Golam crates. It does not imply transitive dependencies contain no unsafe/FFI. Every admitted dependency with native code, FFI, assembly, process spawning, or network behavior is recorded explicitly and wrapped behind the narrowest Golam crate boundary.

For Spec 002:

- SQLite C FFI belongs only behind `golam-ledger` storage APIs.
- BLAKE3 is initially configured with `pure` to shrink the native-code surface.
- Tokio will not be admitted until IPC requires async I/O.
- fuzzing dependencies are development tooling only.

## Canonical encoding decision

Security-critical ledger hashes MUST NOT depend on a serializer's incidental representation. Spec 002 owns an explicit versioned canonical encoding: domain separation, fixed field order, big-endian fixed-width integers, and u32 length-prefixed byte strings. Any future schema change must bump the schema/domain version and preserve frozen compatibility vectors.
