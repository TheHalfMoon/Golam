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
| Unix peer credentials | `nix 0.31.3` | ADMITTED_T002_032_UNIX_ONLY | MIT, MSRV 1.69. Exact pin, defaults disabled, only `socket` + `user`. Target-Unix only. Linux uses `SO_PEERCRED`; Apple uses `LOCAL_PEERCRED` + `LOCAL_PEERPID`. |
| Windows named pipes | `interprocess 2.4.3` | ADMITTED_T002_033_WINDOWS_ONLY | `0BSD OR Apache-2.0`, Rust 1.75 MSRV, actively maintained. Exact pin with default features disabled: no Tokio/async feature. Target-Windows only in `golam-ipc`. Provides synchronous named-pipe listener/stream, explicit security descriptor injection, `accept_remote=false`, non-inheritable handles, and kernel-reported peer PID/session metadata. Its internal `windows-sys`/unsafe Win32 boundary remains outside Golam crates. |
| Windows SID/DACL wrappers | `windows-permissions 0.2.4` | ADMITTED_T002_033_WINDOWS_ONLY_WITH_AGE_RISK | MIT safe wrapper crate from 2021, exact-pinned and target-Windows only in `golam-core`. Used narrowly for current-process SID, SDDL parsing, `SetNamedSecurityInfoW`/`GetNamedSecurityInfoW`, ACE verification and protected DACL verification. Its age and transitive `winapi 0.3.9` unsafe boundary are explicit debt; no broad Windows abstraction authority is granted. Exact Windows CI must prove Rust 1.98 compatibility and actual ACL behavior. |
| Windows SDDL UTF-16 | `widestring 1.2.1` | ADMITTED_T002_033_WINDOWS_ONLY | MIT OR Apache-2.0, exact pin and target-Windows only in `golam-ipc`. Used only to produce a NUL-terminated UTF-16 SDDL string for `interprocess::SecurityDescriptor::deserialize`; it receives no authority semantics. |
| Async runtime | `tokio 1.51.4` | QUALIFIED_CANDIDATE_NOT_ADMITTED | Not needed by T002-032/T002-033; local transports remain synchronous. Re-evaluate only when bounded daemon multiplexing requires async I/O. |
| Generic serialization | `serde 1.0.229` | QUALIFIED_CANDIDATE | Does not define canonical ledger/authentication bytes. |
| IPC wire encoding | `postcard 1.1.3` | EVALUATED_NOT_NEEDED_T002_030_033 | Current IPC frame/lifecycle payloads use explicit Golam-owned fixed binary formats. |
| IDs | `uuid 1.25.0` | EVALUATED_NOT_ADMITTED | Typed `u128` IDs remain sufficient. |
| Errors | `thiserror 2.0.20` | EVALUATED_NOT_ADMITTED | Current bounded error enums use std only. |
| Property tests | `proptest 1.11.0` | QUALIFIED_DEV_CANDIDATE | Admit when final property suite lands. |
| Fuzzing | `libfuzzer-sys 0.4.13` via `cargo fuzz` | QUALIFIED_TOOL_CANDIDATE | Tooling only, never production dependency. |

## Security / unsafe boundary

`#![forbid(unsafe_code)]` remains mandatory in Golam crates. Transitive/native unsafe boundaries are recorded and minimized.

- SQLite C FFI belongs only behind `golam-ledger::storage`.
- BLAKE3 uses `pure`.
- `ed25519-dalek` is confined to `golam-ipc` transcript verification with strict verification and no hazardous/legacy/RNG surfaces enabled.
- `nix` is confined to Unix peer-credential queries and target-Unix only.
- `windows-permissions` is confined to Windows filesystem SID/DACL application and verification in `golam-core`; Golam contains no direct Win32 FFI or unsafe block.
- `interprocess` is confined to target-Windows named-pipe creation/accept/connect and peer PID/session metadata in `golam-ipc`; its security descriptor is supplied by Golam-owned SDDL and remote connections are disabled.
- `widestring` only encodes that SDDL for the safe interprocess API.
- Tokio remains unadmitted.

## Canonical / authentication decision

Security-critical ledger hashes and IPC authentication transcripts remain Golam-owned explicit encodings. T002-033 does not alter T002-031 authentication bytes. Windows OS identity and pipe DACL are independent transport-security inputs; they never replace cryptographic enrollment/verification.

## Windows protected path decision

On Windows, every Golam protected runtime/data/artifact directory receives a protected DACL whose only ACE grants inheritable file-all access to the current process SID. Verification re-reads the DACL from the OS as a file object, requires exactly one allow ACE for that SID with file-all access, and requires SDDL `D:P` protection. `ProtectionLevel::UserOnlyVerified` is not returned until those checks pass.

## Windows named-pipe decision

The pipe name is per-user for discovery, but the SID embedded in the name is **not** treated as a security boundary. Security comes from the protected named-pipe DACL supplied at kernel object creation, `accept_remote=false`, non-inheritable handles, plus the independent T002-031 cryptographic lifecycle. Server-side accepted streams capture kernel-reported client PID/session metadata; PID/session are audit/identity inputs and not standalone authority.

The configured instance limit must be 2..=254 because `interprocess` documents that a limit of 1 breaks `.accept()` and Windows reserves 255 as the unlimited sentinel.

## Current source evidence

- `interprocess 2.4.3` package/features/license/MSRV: upstream `Cargo.toml`.
- `interprocess` `PipeListenerOptions`: explicit `security_descriptor`, `accept_remote`, `instance_limit`, `inheritable`.
- `interprocess` `PipeStream`: `client_process_id`, `peer_process_id`, `client_session_id`, `peer_session_id`.
- `windows-permissions 0.2.4`: MIT, safe Windows permissions API wrapper over `winapi 0.3.9`.
- `windows-permissions::utilities::current_process_sid`, safe wrappers for SID conversion and named security info.
- `widestring 1.2.1`: UTF-16 string handling only.

## SQLite durability posture

The SQLite spine remains fail-closed with canonical event/audit verification. Explicit recovery-only/quarantine serving mode, disk-full reserve qualification and effect crash ambiguity remain separate tasks.
