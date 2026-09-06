---
task_id: T005-072
scope: linux-x86_64_resource_primitive_addendum
outcome: ADMITTED_FOR_CANDIDATE_IMPLEMENTATION
recorded_on: 2026-09-05
package: nix
version: 0.31.3
new_feature: resource
production_profile_admitted: false
waiver_taken: false
---

# Source Foundry addendum — `nix 0.31.3` `resource` feature

## Decision

T005-072 requires child-side operating-system resource ceilings without adding unsafe Golam code. The already exact-pinned `nix = 0.31.3` dependency is therefore widened by exactly one feature:

```toml
nix = { version = "=0.31.3", default-features = false, features = ["fs", "resource", "socket", "user"] }
```

No package version changes. The `resource` feature has no additional feature dependency closure in `nix 0.31.3`.

## Selected surface

Only the safe `nix::sys::resource::setrlimit` wrapper and `Resource` identifiers are admitted for the first Linux x86_64 containment candidate.

The child-side candidate uses exact hard ceilings for:

- `RLIMIT_CORE=0`;
- `RLIMIT_CPU` for CPU-time consumption;
- `RLIMIT_AS` for address-space consumption;
- `RLIMIT_FSIZE` for created regular-file size;
- `RLIMIT_NOFILE` for descriptor-count ceiling.

`RLIMIT_CPU` is explicitly **not** treated as a wall-clock timeout. Wall-time termination and stdout/stderr byte ceilings require the trusted parent supervisor and terminal reconciliation boundary before production admission.

## Non-admissions

This feature widening does not admit `nix` `process`, `signal`, `sched`, `mount`, `ptrace`, `net`, `ioctl` or other features. It does not enable process launch, cancellation authority, shell execution, network access or a production executor.

The underlying `nix` implementation may use unsafe system calls internally; that unsafe code remains inside the already-reviewed upstream dependency boundary. Golam crates remain `#![forbid(unsafe_code)]`.

## Result

```text
NIX_0_31_3_RESOURCE_FEATURE=ADMITTED_T005_PHASE_G_LINUX_X86_64_CANDIDATE
NEW_PACKAGE_VERSION=NO
NEW_TRANSITIVE_FEATURE_DEPENDENCY=NO
WALL_TIME_ENFORCEMENT_CLAIMED=NO
OUTPUT_CAPTURE_BOUND_CLAIMED=NO
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
WAIVER_TAKEN=NO
```
