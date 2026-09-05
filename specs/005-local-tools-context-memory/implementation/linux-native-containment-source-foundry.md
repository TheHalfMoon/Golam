---
task_id: T005-071
outcome: PASS
recorded_on: 2026-09-05
first_platform: linux-x86_64
production_profile_admitted: false
process_launch_enabled: false
waiver_taken: false
next_task: T005-072
---

# Source Foundry — first production native containment primitives

## Decision

T005-071 admits a minimal Rust dependency surface for implementing and qualifying the first production native containment profile on **Linux x86_64 only**:

- `landlock = 0.4.7` for unprivileged kernel-enforced filesystem restrictions and ABI-v4 TCP bind/connect restriction support;
- `seccompiler = 0.5.0` for in-process construction and installation of Linux seccomp-BPF filters;
- the already-admitted exact `nix = 0.31.3` dependency may be feature-expanded only when T005-072 demonstrates a concrete need for safe process/resource primitives and records the exact added features before code use.

This Source Foundry decision admits primitives/dependencies for implementation work only. It does **not** admit a production containment profile, a process executor, shell execution, an external search binary, local MCP execution or executable skills.

## First platform identity

The first implementation target is not generic "Linux". It is:

```text
os=linux
arch=x86_64
landlock_required_abi=V4
seccomp_required=YES
production_profile_token=platform:linux-x86_64-landlock-v4-seccomp-v1
```

Runtime admission MUST query actual Landlock support and fail closed unless all required rights can be enforced. Distribution/kernel naming is supporting evidence only, never runtime authority.

Ubuntu 24.04 LTS is the first official CI target because its GA kernel family is Linux 6.8 and upstream Linux 6.8 documents Landlock network rules from ABI v4. This does not authorize kernels merely because they report an Ubuntu release string.

macOS, Windows, other architectures and Linux systems lacking the exact required kernel features remain unsupported/denied until separate qualification.

## `landlock 0.4.7`

### Exact source identity

- registry package: `landlock`
- exact version: `0.4.7`
- crates.io checksum: `4cca98e95f35b29d469dade6724c6f96cec9236640f745a0e99b0334ec320ab1`
- upstream repository: `https://github.com/landlock-lsm/rust-landlock`
- upstream release tag: `v0.4.7`
- annotated tag object: `0b5cef9c77fda372ba3d4c235b6814c216bf1e7a`
- tagged source commit: `62fec0f1521e4ab7f697752c1f324b725fe643d5`
- GitHub reports the annotated release tag signature as verified.
- source SPDX: `Apache-2.0 OR MIT`

### Exact normal dependency closure selected by the registry release

`landlock 0.4.7` has these normal dependencies:

- `enumflags2 ^0.7`;
- `libc ^0.2.186`;
- `thiserror ^2.0`.

The exact closure selected for Golam's current lock universe is:

- `enumflags2 0.7.12`, checksum `1027f7680c853e056ebcec683615fb6fbbc07dbaa13b4d5d9442b146ded4ecef`;
- `enumflags2_derive 0.7.12`, checksum `67c78a4d8fdf9953a5c9d458f9efe940fd97a0cab0941c075a813ac594733827`;
- existing `libc 0.2.189`, checksum `3eaf3ede3fee6db1a4c2ee091bf8a8b4dccdc6d17f656fb07896ee72867612f2`;
- existing `thiserror 2.0.20`, checksum `ec86235f5fcc2a73650310756d2ac5b138a5780bbbdfae3eeccec992c435ba4f`;
- existing `thiserror-impl 2.0.20`, checksum `bc04cd3e1236dd4a98afca4569f2deb3f120e5422a4023be2cb683f8486292af`;
- existing procedural-macro closure already present in the lockfile (`proc-macro2`, `quote`, `syn`).

Dev dependencies published by the crate are not part of Golam's runtime closure.

### Admitted API/security posture

Landlock is a safe Rust abstraction over the Linux Landlock syscalls and is usable by unprivileged processes. Golam may use only an exact **hard requirement** compatibility posture for required rights. `BestEffort`/silent rights downgrade is forbidden for production admission.

The selected profile requires Landlock ABI v4 because:

- ABI v3 is required for enforceable truncate control;
- ABI v4 adds TCP bind/connect control;
- Ubuntu 24.04's Linux 6.8 baseline is compatible with ABI v4;
- ABI v5 device-IOCTL control was added only in Linux 6.10 and is therefore not required or claimed by the first profile.

Because ABI v4 does not control every network family/protocol, Landlock network rights are defense-in-depth for TCP. The production strict-local guarantee also requires a seccomp syscall boundary plus external descendant-aware observation. No UDP, UNIX-socket or universal network-isolation claim is derived from Landlock v4.

Landlock does not retroactively restrict file descriptors opened before restriction. T005-072 must therefore apply descriptor/handle hygiene and install the production restrictions before untrusted payload execution.

## `seccompiler 0.5.0`

### Exact source identity

- registry package: `seccompiler`
- exact version: `0.5.0`
- crates.io checksum: `a4ae55de56877481d112a559bbc12667635fdaf5e005712fd4e2b2fa50ffc884`
- upstream repository: `https://github.com/firecracker-microvm/firecracker`
- upstream crate path: `src/seccompiler`
- source SPDX: `Apache-2.0 OR BSD-3-Clause`

The crates.io registry checksum is the executable source identity used for Golam dependency admission; no mutable upstream branch is trusted as package identity.

### Exact normal dependency closure

With Golam constructing filters directly in Rust and **not** enabling the optional JSON feature, `seccompiler 0.5.0` requires only:

- existing `libc 0.2.189`, checksum `3eaf3ede3fee6db1a4c2ee091bf8a8b4dccdc6d17f656fb07896ee72867612f2`.

No serde/JSON parser is admitted for this boundary.

### Admitted API/security posture

The crate is Linux-specific and documents little-endian x86_64 as a supported host architecture. Golam may use its typed Rust filter representation and `apply_filter_all_threads`/TSYNC where applicable. JSON policy ingestion is out of scope.

T005-072 must define the syscall policy in Golam-owned typed code, bind the exact compiled policy identity to the production profile, and test the filter itself. A seccomp filter is a containment primitive, not authority and not terminal process evidence.

## Existing `nix 0.31.3`

The workspace already pins:

```toml
nix = { version = "=0.31.3", default-features = false, features = ["fs", "socket", "user"] }
```

T005-071 does not widen these features. T005-072 may add only exact safe features proven necessary for implementation, such as process/resource/signal primitives, after checking the `nix 0.31.3` feature/API closure. No `mount`, `sched`, `ptrace` or unrelated capability is pre-authorized by this record.

## Why the Spec 003 Bubblewrap harness is not selected

Canonical T003-074 used Bubblewrap, `setpriv`, Python and `sudo` inside a test-only qualification harness. Its own evidence explicitly states these are harness dependencies and do not become Golam product-runtime admissions.

T005-071 therefore does not admit system `bwrap`, `sudo`, a shell launcher or distribution package-version coupling for production. The selected Rust primitives allow T005-072 to build a narrow Golam-owned helper while keeping application code under `#![forbid(unsafe_code)]` and avoiding ambient external-launcher authority.

## Frozen first-profile implementation constraints

T005-072 MUST fail closed unless it can prove, before untrusted execution:

1. current platform is Linux x86_64;
2. Landlock supports every required v4 filesystem/network right under `HardRequirement` semantics;
3. filesystem read/write/execute roots are explicit and empty sets mean deny-all;
4. pre-existing descriptor inheritance is reduced to the exact admitted standard/IPC handles;
5. ambient environment is cleared;
6. seccomp filter installation succeeds for the exact expected architecture and policy;
7. external network socket creation/use and process escape are denied to the exact tested boundary;
8. nonempty device/IPC/handle requests are denied unless separately implemented and proven;
9. resource/timeout controls are either exactly enforced or the profile refuses the request;
10. cancellation cannot become terminal success without root/descendant terminal reconciliation.

T005-072 may initially implement the narrowest production profile whose `spawn_rule` is `Deny`, making descendant creation itself an enforced denial. Broader `ManagedDescendants` support requires separate ownership/discovery/reconciliation proof before admission.

## Explicit non-admissions

```text
SYSTEM_BWRAP_RUNTIME=NOT_ADMITTED
SUDO_RUNTIME=NOT_ADMITTED
SHELL_LAUNCH=NOT_ADMITTED
EXTERNAL_SEARCH_BINARY=NOT_ADMITTED
MCP_LOCAL_EXECUTION=NOT_ADMITTED
EXECUTABLE_SKILLS=NOT_ADMITTED
JSON_SECCOMP_POLICY=NOT_ADMITTED
MACOS_NATIVE_PROFILE=NOT_ADMITTED
WINDOWS_NATIVE_PROFILE=NOT_ADMITTED
CROSS_PLATFORM_EQUIVALENCE=NO
```

## Source Foundry result

```text
T005_071=PASS
LANDLOCK_0_4_7=ADMITTED_T005_PHASE_G_LINUX_X86_64_PRIMITIVE
SECCOMPILER_0_5_0=ADMITTED_T005_PHASE_G_LINUX_X86_64_PRIMITIVE
NIX_0_31_3_FEATURE_WIDENING=NOT_YET_ADMITTED
PRODUCTION_PROFILE_TOKEN=platform:linux-x86_64-landlock-v4-seccomp-v1
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
PROCESS_LAUNCH_ENABLED=NO
WAIVER_TAKEN=NO
NEXT_TASK=T005-072
```
