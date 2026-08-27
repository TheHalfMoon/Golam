# Dependency Qualification — Spec 002

**Checked**: 2026-08-25  
**Rust toolchain**: 1.98.0  
**Rule**: pin exact production versions during Spec 002; minimize the trusted dependency surface; dependencies do not receive Golam authority merely because they are Rust.

## Decisions

| Need | Candidate | Decision | Rationale / boundary |
|---|---|---|---|
| SQLite | `rusqlite 0.40.2` | ADMITTED | MIT wrapper; `libsqlite3-sys` is an explicit FFI boundary. `bundled` is enabled for deterministic cross-platform SQLite; vendored SQLite is public domain. All DB authority semantics remain in `golam-ledger`. |
| Integrity hash | `blake3 1.8.7` | ADMITTED | Official BLAKE3 implementation. Exact pin with `pure` avoids assembly/C optimized paths while canonical vectors are frozen. |
| IPC transcript signatures | `ed25519-dalek 3.0.0` | ADMITTED_T002_031_T002_034 | BSD-3-Clause, pure-Rust Ed25519, MSRV 1.85. Exact pin; defaults disabled; only `signature` + `zeroize`. Golam uses `verify_strict` for lifecycle authentication and `SigningKey` for local first-party credentials. Private seed storage remains outside SQLite/model state. |
| OS cryptographic randomness | `getrandom 0.4.3` | ADMITTED_T002_034 | MIT OR Apache-2.0, MSRV 1.85. Exact pin with `std`; used only to fill the 32-byte Ed25519 private seed from the operating-system random source. It does not define key storage, authorization, protocol bytes or model-visible state. |
| Unix peer credentials | `nix 0.31.3` | ADMITTED_T002_032_UNIX_ONLY | MIT, MSRV 1.69. Exact pin, defaults disabled, only `socket` + `user`. Target-Unix only. Linux uses `SO_PEERCRED`; Apple uses `LOCAL_PEERCRED` + `LOCAL_PEERPID`. |
| Windows named pipes | `interprocess 2.4.3` | ADMITTED_T002_033_WINDOWS_ONLY | `0BSD OR Apache-2.0`, Rust 1.75 MSRV. Exact pin with default features disabled. Target-Windows only in `golam-ipc`; synchronous named-pipe listener/stream, security descriptor injection, local-only pipe and peer PID/session metadata. |
| Windows SID/DACL wrappers | `windows-permissions 0.2.4` | ADMITTED_T002_033_T002_034_WINDOWS_ONLY_WITH_AGE_RISK | MIT safe wrapper crate from 2021, exact-pinned and target-Windows only in `golam-core`. Used narrowly for current-process SID, protected current-user DACL application and re-verification on runtime/authority/credential filesystem objects. Its age and transitive `winapi 0.3.9` unsafe boundary remain explicit debt. |
| Windows SDDL UTF-16 | `widestring 1.2.1` | ADMITTED_T002_033_WINDOWS_ONLY | MIT OR Apache-2.0, exact pin and target-Windows only in `golam-ipc`. Used only for a NUL-terminated SDDL string for the named-pipe security descriptor. |
| Async runtime | `tokio 1.51.4` | QUALIFIED_CANDIDATE_NOT_ADMITTED | Not needed by T002-032/T002-035 slices so far; local transports remain synchronous. Re-evaluate only when bounded daemon multiplexing requires async I/O. |
| Generic serialization | `serde 1.0.229` | QUALIFIED_CANDIDATE | Does not define canonical ledger/authentication bytes. |
| IPC wire encoding | `postcard 1.1.3` | EVALUATED_NOT_NEEDED | Current IPC frame/lifecycle payloads use explicit Golam-owned fixed binary formats. |
| IDs | `uuid 1.25.0` | EVALUATED_NOT_ADMITTED | Typed `u128` IDs remain sufficient. |
| Errors | `thiserror 2.0.20` | EVALUATED_NOT_ADMITTED | Current bounded error enums use std only. |
| Property tests | `proptest 1.11.0` | QUALIFIED_DEV_CANDIDATE | Admit when final property suite lands. |
| Fuzzing | `libfuzzer-sys 0.4.13` via `cargo fuzz` | QUALIFIED_TOOL_CANDIDATE | Tooling only, never production dependency. |

## Dependency reproducibility

`Cargo.lock` is committed for the Spec 002 workspace. CI uses `cargo ... --locked` for clippy and tests on Windows/macOS/Linux. Direct dependencies remain exact-pinned in `Cargo.toml`, while the lockfile freezes the currently qualified transitive graph so a new compatible transitive release cannot silently change an exact-head qualification run.

The lockfile is dependency-resolution evidence, not a supply-chain trust claim. Source/license/SBOM review remains separate.

## Security / unsafe boundary

`#![forbid(unsafe_code)]` remains mandatory in Golam crates. Transitive/native unsafe boundaries are recorded and minimized.

- SQLite C FFI belongs only behind `golam-ledger::storage`.
- BLAKE3 uses `pure`.
- `ed25519-dalek` is confined to local IPC authentication/credential cryptography; Golam does not implement Ed25519 arithmetic.
- `getrandom` is confined to private-seed generation; it receives no authority policy or persistent-state role.
- `nix` is confined to Unix peer-credential queries and target-Unix only.
- `windows-permissions` is confined to Windows filesystem SID/DACL application and verification in `golam-core`; Golam contains no direct Win32 FFI or unsafe block.
- `interprocess` is confined to target-Windows named-pipe creation/accept/connect and peer PID/session metadata in `golam-ipc`; remote pipe access is disabled.
- `widestring` only encodes named-pipe SDDL.
- Tokio remains unadmitted.

## T002-034 credential-storage decision

Spec 002 does not pretend the current filesystem fallback is an OS keychain. The first implemented credential backend is explicitly `FilesystemUserPrivateV1`, a lower-assurance fallback:

- credential files live only under the canonical protected `<GolamData>/authority/client-credentials/` subtree;
- Unix files are created `0600`; parent authority directories are `0700`;
- Windows files receive and re-verify a protected current-user-only DACL;
- `create_new` prevents silent overwrite of a prior credential;
- the private seed is never stored in the `clients` SQLite table and is not model-visible state;
- the credential envelope binds magic/version/client ID/key fingerprint/private seed/public key;
- load re-derives the public key and fingerprint and fails closed on corruption/mismatch;
- temporary seed/envelope buffers are zeroed where the Rust types expose mutable bytes, while `ed25519-dalek` has `zeroize` enabled;
- credential deletion is logical filesystem deletion, **not** a secure-erase guarantee on SSD/COW/journaled filesystems.

A future qualified OS credential facility may use `OsProtectedV1`; T002-034 does not claim that stronger assurance today.

## Canonical / authentication decision

Security-critical ledger hashes and IPC authentication transcripts remain Golam-owned explicit encodings. OS peer identity and filesystem/named-pipe ACLs are independent transport-security inputs; neither replaces cryptographic enrollment. Client IDs cannot silently acquire a different key, duplicate client/key IDs are rejected, and revoked/unknown/mismatched credentials fail before READY.

## Protected authority path decision

The canonical protected authority subtree is `<GolamData>/authority/`; its SQLite authority database path is `<GolamData>/authority/golam.db`, matching the Spec 002 plan. Client credential fallback files are direct children of `<GolamData>/authority/client-credentials/`. Authority paths are not future generic-tool paths; T002-042 still owns the hostile/generic-resource API enforcement proof.

## Windows named-pipe decision

The SID embedded in the pipe name is discovery data, not the security boundary. Security comes from the protected named-pipe DACL, `accept_remote=false`, non-inheritable handles, peer metadata, and the independent Ed25519 lifecycle. PID/session are audit/identity inputs and not standalone authority.

## Source evidence

- `interprocess 2.4.3` package/features/license/MSRV and named-pipe listener/stream APIs.
- `windows-permissions 0.2.4` MIT wrapper and current-process SID / named-security APIs.
- `widestring 1.2.1` UTF-16 handling.
- `getrandom 0.4.3` exact package used for OS random bytes.
- committed `Cargo.lock` plus locked three-OS CI for the qualified graph.

## SQLite durability posture

The SQLite spine remains fail-closed with canonical event/audit verification. Client enrollment/revocation persistence is a low-level authority-store primitive at T002-034; T002-044 remains responsible for proving unprivileged adapters cannot call those mutations without `KernelApi`. Explicit recovery-only/quarantine serving mode, disk-full reserve qualification and effect crash ambiguity remain separate tasks.
