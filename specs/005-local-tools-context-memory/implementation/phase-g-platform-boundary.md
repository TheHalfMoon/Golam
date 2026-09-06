---
task_id: T005-075
outcome: IMPLEMENTED_PENDING_QUALIFICATION
recorded_on: 2026-09-05
profile: platform:linux-x86_64-landlock-v4-seccomp-v2
supersedes_profile_for_admission: platform:linux-x86_64-landlock-v4-seccomp-v1
production_profile_admitted: false
waiver_taken: false
---

# Phase G exact native containment claim boundary

## Purpose

This record freezes the maximum claim surface for T005-075 before T005-077 admission. It is not profile-admission evidence and does not mark T005-075 PASS until the exact v2 hostile qualification and repository CI are successful on an unchanged implementation head.

The earlier `platform:linux-x86_64-landlock-v4-seccomp-v1` identity remains immutable historical evidence and is not eligible for admission. The only candidate carried forward is v2.

## Exact supported candidate boundary

```text
os=linux
arch=x86_64
profile=platform:linux-x86_64-landlock-v4-seccomp-v2
landlock_required_abi=V4
landlock_compatibility=HardRequirement
seccomp_required=YES
seccomp_installation=TSYNC
spawn_rule=DENY
network_posture=STRICT_LOCAL_DENY
filesystem_data_root_shape=EXACT_EXISTING_REGULAR_FILE_ONLY
linux_capability_sets=EMPTY_INH_PRM_EFF_AMB
production_admitted=NO
```

Runtime distribution names are not authority. The implementation must obtain fully enforced Landlock status at runtime and must successfully install the exact seccomp program. Failure, missing `/proc` capability evidence, partial Landlock compatibility, or inability to apply the seccomp program is denial.

## Qualified-claim candidates

Subject to successful T005-073/T005-074/T005-076 execution and T005-077 review, v2 may claim only the controls implemented and externally observed for Linux x86_64:

- ambient process environment must be actually empty at the trusted child-side containment boundary;
- inherited descriptors beyond admitted standard descriptors are rejected before untrusted execution;
- standard descriptors are rejected when observed as inherited sockets;
- Linux `CapInh`, `CapPrm`, `CapEff`, and `CapAmb` must all be zero before untrusted execution;
- executable, cwd, and every admitted data-file native object identity are bound to canonical path/device/inode/mode and revalidated immediately before restriction;
- executable is an exact regular file; cwd is an exact identity-bound directory with `ReadDir` only;
- data filesystem read/write roots are exact existing regular files only; directory, device-node, FIFO, and socket data roots are denied;
- read and write data-root sets are disjoint in this first profile;
- filesystem access is Landlock V4 `HardRequirement`; no generic directory-backed data authority is admitted;
- Landlock TCP bind/connect handling is defense in depth and is not treated as universal network isolation;
- seccomp denies socket/socketpair creation and the exact process/thread, SysV/POSIX IPC, anonymous IPC, cross-process control/memory, mount/namespace, filesystem-monitor/handle, device-node creation, keyring, BPF/perf/userfaultfd, and io_uring syscall families listed by the exact v2 implementation;
- process creation is denied by the first profile rather than supported as a managed descendant tree;
- external process observation must therefore see exactly the owned root process while it is running;
- cancellation is non-terminal until the operating system reports the exact owned root terminal;
- resource claims are limited to exact RLIMIT controls configured by the implementation: core, CPU, address space, created-file size, and open-file count;
- wall-time and combined stdout/stderr limits are parent-supervisor obligations and cannot be claimed from child-side RLIMIT evidence alone;
- combined stdout/stderr accounting must terminate at the configured bound and require exact terminal reconciliation;
- device requests, IPC requests, and inherited-handle requests must be empty; nonempty requests are rejected before launch.

## Exact external observation boundary

The v2 hostile qualification is the applicable T005-074/T005-076 observation surface. It starts the exact v2 probe under a cleared environment and externally observes the running owned process tree using `ps` and Internet sockets using `lsof`. The child also attempts prohibited socket, local socketpair, spawn, filesystem-write, and device operations. The shell harness must fail if any required v2 evidence marker is absent.

The repository also retains daemon-level strict-local observation for regression coverage. That daemon observation is not a substitute for the v2-specific external hostile qualification.

## Explicit non-claims

The following claims remain forbidden unless separately implemented and qualified later:

```text
GENERIC_LINUX_EQUIVALENCE=NO
MACOS_EQUIVALENCE=NO
WINDOWS_EQUIVALENCE=NO
OTHER_ARCH_EQUIVALENCE=NO
UNIVERSAL_NAMESPACE_ISOLATION=NO
SEPARATE_NETWORK_NAMESPACE=NO
SEPARATE_USER_NAMESPACE=NO
UNIVERSAL_NETWORK_ISOLATION_FROM_LANDLOCK_ALONE=NO
UDP_POLICY_FROM_LANDLOCK_V4=NO
UNIX_SOCKET_POLICY_FROM_LANDLOCK_V4=NO
COMPLETE_DEVICE_IOCTL_MEDIATION=NO
MANAGED_DESCENDANT_SUPPORT=NO
PRODUCTION_SHELL=NO
PRODUCTION_EXECUTOR_ADMITTED=NO
```

Blocking `unshare`, `setns`, `mount`, `umount2`, process-creation, socket/IPC, and related syscall entry points does not justify a claim that Golam created a separate namespace. v2 instead prevents the explicitly listed construction/escape surfaces and relies on fully enforced Landlock plus the exact seccomp deny policy inside the current Linux namespace context.

## Unsupported platform semantics

On macOS, Windows, non-x86_64 Linux, and Linux hosts that cannot fully enforce the exact required Landlock/seccomp/capability-hygiene controls, the production native profile remains an explicit denial state. Repository compilation on those platforms proves portability of surrounding code only; it does not prove equivalent containment.

## Admission ordering

T005-077 remains the sole admission gate. Before `production_admitted` can become true, all of the following must be clean on one unchanged v2 candidate head:

1. T005-073 secret-safe process evidence qualification;
2. T005-074 external descendant-aware strict-local v2 observation;
3. T005-076 hostile v2 containment qualification;
4. full repository CI for the applicable cross-platform compilation/regression boundary;
5. exact Linux x86_64 containment evidence for every claimed control;
6. substantive independent semantic/security review;
7. reconciliation of every material review finding.

Until then:

```text
T005_075=IMPLEMENTED_PENDING_QUALIFICATION
PROFILE=platform:linux-x86_64-landlock-v4-seccomp-v2
V1_PROFILE_ADMISSION_ELIGIBLE=NO
PRODUCTION_PROFILE_ADMITTED=NO
NATIVE_UNQUALIFIED_REMAINS_PRODUCTION_BASELINE=YES
WAIVER_TAKEN=NO
```
