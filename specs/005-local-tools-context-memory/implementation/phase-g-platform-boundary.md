---
task_id: T005-075
outcome: IMPLEMENTED_PENDING_QUALIFICATION
recorded_on: 2026-09-05
profile: platform:linux-x86_64-landlock-v4-seccomp-v1
production_profile_admitted: false
waiver_taken: false
---

# Phase G exact native containment claim boundary

## Purpose

This record freezes the maximum claim surface for T005-075 before T005-077 admission. It is not profile-admission evidence and does not mark T005-075 PASS until the exact hostile qualification and repository CI are successful on an unchanged implementation head.

## Exact supported candidate boundary

The only production-containment candidate under qualification is:

```text
os=linux
arch=x86_64
profile=platform:linux-x86_64-landlock-v4-seccomp-v1
landlock_required_abi=V4
landlock_compatibility=HardRequirement
seccomp_required=YES
seccomp_installation=TSYNC
spawn_rule=DENY
network_posture=STRICT_LOCAL_DENY
production_admitted=NO
```

Runtime distribution names are not authority. The implementation must obtain fully enforced Landlock status at runtime and must successfully install the exact seccomp program. Failure or partial compatibility is denial.

## Qualified-claim candidates

Subject to successful T005-074/T005-076 execution and T005-077 review, this profile may claim only the controls implemented and externally observed for Linux x86_64:

- ambient process environment must be actually empty at the trusted child-side containment boundary;
- inherited descriptors beyond admitted standard descriptors are rejected before untrusted execution;
- standard descriptors are rejected when observed as inherited sockets;
- executable and cwd native object identities are revalidated immediately before restriction;
- filesystem access is Landlock V4 `HardRequirement` with explicit allowed roots; an empty write-root set means deny all new writes outside executable/cwd semantics;
- TCP bind/connect Landlock handling is defense in depth and is not treated as universal network isolation;
- seccomp denies socket/socketpair creation, process creation, ptrace, mount/unmount and namespace-changing syscall entry points listed by the exact implementation;
- process creation is denied by the first profile rather than supervised as a descendant tree;
- external process observation must therefore see exactly the owned root process while it is running;
- cancellation is non-terminal until the operating system reports the exact owned root terminal;
- resource claims are limited to exact RLIMIT controls configured by the implementation: core, CPU, address space, created-file size and open-file count;
- wall-time and captured-output limits remain parent-supervisor obligations and cannot be claimed from child-side RLIMIT evidence alone;
- device and arbitrary IPC access are not admitted by the first profile.

## Explicit non-claims

The following claims are forbidden for this profile unless separately implemented and qualified later:

```text
GENERIC_LINUX_EQUIVALENCE=NO
MACOS_EQUIVALENCE=NO
WINDOWS_EQUIVALENCE=NO
OTHER_ARCH_EQUIVALENCE=NO
UNIVERSAL_NAMESPACE_ISOLATION=NO
UNIVERSAL_NETWORK_ISOLATION_FROM_LANDLOCK_ALONE=NO
UDP_POLICY_FROM_LANDLOCK_V4=NO
UNIX_SOCKET_POLICY_FROM_LANDLOCK_V4=NO
DEVICE_IOCTL_ISOLATION=NO
MANAGED_DESCENDANT_SUPPORT=NO
PRODUCTION_SHELL=NO
PRODUCTION_EXECUTOR_ADMITTED=NO
```

Blocking `unshare`, `setns`, `mount`, `umount2` and process-creation syscalls through the exact seccomp policy does not justify a claim that Golam has created a separate namespace. The first profile instead prevents these escape/construction operations and relies on Landlock plus syscall denial inside the current Linux namespace context.

## Unsupported platform semantics

On macOS, Windows, non-x86_64 Linux and Linux hosts that cannot fully enforce the exact required Landlock/seccomp controls, the production native profile remains an explicit denial state. Repository compilation on those platforms proves portability of the surrounding code only; it does not prove equivalent containment.

## Admission ordering

T005-077 remains the sole admission gate. Before `production_admitted` can become true, all of the following must be clean on an unchanged candidate implementation head:

1. focused hostile containment qualification including external descendant-aware observation;
2. full repository CI for the applicable cross-platform compilation/regression boundary;
3. exact Linux x86_64 containment evidence for every claimed control;
4. substantive independent semantic/security review;
5. reconciliation of every material review finding.

Until then:

```text
T005_075=IMPLEMENTED_PENDING_QUALIFICATION
PRODUCTION_PROFILE_ADMITTED=NO
NATIVE_UNQUALIFIED_REMAINS_PRODUCTION_BASELINE=YES
WAIVER_TAKEN=NO
```
