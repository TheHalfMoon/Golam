# Source Foundry — `nix 0.31.3` filesystem mutation primitives

**Spec**: 005 — Local Tools, Context & Memory  
**Scope**: T005-060..T005-062 only  
**Candidate**: existing exact-pinned `nix = 0.31.3` dependency, feature expansion `fs`  
**Decision**: `ADMITTED_T005_PHASE_F_UNIX_FS_ONLY`

## Exact source identity

- Package: `nix`
- Version: `0.31.3`
- Upstream repository: `https://github.com/nix-rust/nix`
- Release documentation: `https://docs.rs/nix/0.31.3/nix/`
- Source documentation: `https://docs.rs/crate/nix/0.31.3/source/`
- License: MIT
- Rust MSRV reported by the already-canonical Spec 002 qualification: 1.69
- Golam workspace pin before this admission: `=0.31.3`, `default-features = false`, features `socket`, `user`

This record does not admit a new package version or a new dependency family. It admits only the `fs` feature of the already exact-pinned `nix 0.31.3` package for a narrow Unix filesystem-mutation boundary in `golamd`.

## Selected exact surfaces

The selected `fs` feature provides safe Rust wrapper surfaces needed for descriptor-relative mutation:

- `nix::fcntl::open` / `nix::fcntl::openat` with owned file descriptors and filesystem flags;
- `nix::sys::stat::fstat` for descriptor-backed identity revalidation;
- `nix::fcntl::renameat` for descriptor-relative rename/replace transitions;
- `nix::unistd::linkat` for no-overwrite descriptor-relative install/rollback transitions between regular-file names under the same retained parent authority;
- `nix::unistd::unlinkat` for descriptor-relative deletion.

Golam may use only the minimum subset needed to retain an already checked parent/target identity through the mutation boundary. `linkat` is admitted only as a no-overwrite hard-link transition for already-opened regular files; it does not widen authority to arbitrary link creation. `nix` does not decide authorization, capability, target scope, preconditions, Effect Gate state, reconciliation, or terminal success.

## Security boundary

Admission is constrained as follows:

1. Unix only. No Windows equivalence is inferred.
2. `fs` is the only newly admitted feature for this use. This admission does not authorize `process`, `net`, `mount`, `sched`, `ptrace`, shell launch, or any other capability.
3. Golam crates remain `#![forbid(unsafe_code)]`; any internal libc/unsafe implementation remains inside the reviewed upstream dependency boundary.
4. Generic filesystem authority still excludes protected Golam state before and at commit time.
5. A path string never becomes authority. Mutation must bind an authorized root, exact operation, retained parent descriptor, expected target state, and Effect Gate preparation.
6. Existing targets must be revalidated after identity-preserving relocation and before irreversible deletion/replacement. If identity cannot be preserved or proven, the operation fails closed or remains reconciling/`UNKNOWN_OUTCOME` as applicable.
7. Failed validation must preserve user data; rollback/reconciliation is mandatory when a prior target has been displaced.
8. No external executable, shell, network access, or process launch is introduced by this admission.

## Why `std::fs` alone is insufficient

The existing read resolver correctly rejects lexical traversal, aliases, mount/device changes and protected paths, but a path-only check followed later by a path-based mutation leaves a rename/swap window. Phase F explicitly requires checked identity to survive to commit or the operation to be refused.

Descriptor-relative `*at` operations let Golam retain the authorized parent directory identity across the final mutation boundary. This closes the parent-path rebinding class without widening into native process execution.

## Dependency and feature disposition

Allowed Cargo change:

```toml
nix = { version = "=0.31.3", default-features = false, features = ["fs", "socket", "user"] }
```

`golamd` may consume the workspace dependency only under `cfg(unix)` for Phase F filesystem mutation primitives.

No other dependency or feature is admitted by this record.

## Qualification requirements

Before T005-068 may pass, exact-head qualification must prove at least:

- retained-parent descriptor prevents parent rename/rebinding from widening authority;
- symlink target and alias insertion are denied;
- stale parent/target/content expectations deny mutation;
- create cannot overwrite an unexpected target;
- replace preserves/restores user data when the checked target changes;
- rename/delete verify the exact displaced identity before commit;
- protected Golam paths remain unreachable through generic mutation authority;
- ambiguous completion is not reported as success;
- Windows compiles with the mutation provider remaining an explicit unsupported/denial state rather than inferred equivalent support.

## Admission result

`NIX_0_31_3_FS_FEATURE=ADMITTED_T005_PHASE_F_UNIX_FS_ONLY`

`NEW_DEPENDENCY_FAMILY=NO`

`PROCESS_OR_NETWORK_AUTHORITY=NO`

`CROSS_PLATFORM_EQUIVALENCE=NO`

`WAIVER_TAKEN=NO`
