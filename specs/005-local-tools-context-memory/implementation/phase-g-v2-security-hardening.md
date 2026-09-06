---
task_id: T005-072/T005-076
outcome: HARDENING_IMPLEMENTED_PENDING_EXACT_HEAD_CI_AND_INDEPENDENT_REVIEW
recorded_on: 2026-09-05
v1_ipc_hardening_head: ac5199977ef759b1d03417e85b5a07a48dbdd4de
rejected_candidate_profile: platform:linux-x86_64-landlock-v4-seccomp-v1
replacement_candidate_profile: platform:linux-x86_64-landlock-v4-seccomp-v2
source_foundry_dependency_change: false
production_native_executor_admitted: false
process_launch_enabled: false
waiver_taken: false
next_gate: T005-077 exact-head CI plus independent semantic/security review
---

# Phase G v2 containment security hardening disposition

## Decision

The original Phase G identity `platform:linux-x86_64-landlock-v4-seccomp-v1` remains immutable historical evidence and is **not eligible for T005-077 admission**.

CI #1193 / run `33968106942` was fully successful on Windows, macOS and Ubuntu for head `5cb4ac2814e8cb94771fa1e4046e8359a9633e9a`. Subsequent internal adversarial review correctly identified an empty-IPC syscall gap. Forward-only commit `ac5199977ef759b1d03417e85b5a07a48dbdd4de` hardened v1 by denying SysV/POSIX IPC, cross-process handles/memory/signals, keyrings, io_uring and related kernel escape surfaces. That repair is retained and is not reverted.

A second independent material gap remains: v1 filesystem read/write roots are path-only `PathBuf` values and can be directory roots. The frozen T005-070 requirements require identity-bound filesystem rights and empty device/IPC rights to remain deny-all. A generic admitted directory can contain FIFO/device/socket special files, so v1 cannot honestly prove that boundary merely by rejecting nonempty device/IPC request vectors.

No waiver is taken. v1 remains `NOT_ADMITTED` even after its valid IPC hardening.

## Replacement identity

The corrected candidate is a new immutable profile identity:

```text
PROFILE=platform:linux-x86_64-landlock-v4-seccomp-v2
PLATFORM=linux-x86_64
LANDLOCK_REQUIRED_ABI=V4
SECCOMP_REQUIRED=YES
SPAWN_RULE=DENY
STRICT_LOCAL=YES
PRODUCTION_ADMISSION=NO_PENDING_T005_077
```

The dependency closure does not change. T005-071 Source Foundry remains the dependency authority for exact `landlock 0.4.7`, `seccompiler 0.5.0`, `libc 0.2.189`, and the already-admitted `nix 0.31.3` resource/fs posture. v2 adds no dependency, feature, build script, native launcher, network library, shell, or donor runtime.

## Hardened filesystem boundary

v2 changes data filesystem rights from path-only roots to exact `NativeObjectIdentity` bindings containing canonical path, device, inode and mode. Every executable, cwd and read/write object is revalidated immediately before restriction installation.

The first v2 profile intentionally supports the narrowest data-root shape:

```text
READ_ROOT=EXACT_EXISTING_REGULAR_FILE_ONLY
WRITE_ROOT=EXACT_EXISTING_REGULAR_FILE_ONLY
READ_WRITE_ROOT_OVERLAP=DENIED
CWD=EXACT_IDENTITY_BOUND_DIRECTORY_WITH_READDIR_ONLY
EXECUTABLE=EXACT_IDENTITY_BOUND_REGULAR_FILE
DIRECTORY_DATA_ROOT=DENIED
DEVICE_NODE_DATA_ROOT=DENIED
FIFO_DATA_ROOT=DENIED_BY_REGULAR_FILE_REQUIREMENT
SOCKET_DATA_ROOT=DENIED_BY_REGULAR_FILE_REQUIREMENT
```

This prevents an allowed data directory from silently donating FIFO/device/socket authority. Broader directory-backed process filesystem access is not admitted by this profile and requires a future separately qualified design.

## Hardened IPC/cross-process/kernel boundary

v2 preserves the valid v1 IPC hardening from `ac519997...` and additionally binds that posture into the v2 containment receipt and parent supervisor contract. The seccomp candidate denies exact syscall families for:

- network and local socket creation;
- process/thread creation;
- SysV message queues, semaphores and shared memory;
- POSIX message queues;
- pipe/eventfd/signalfd/memfd IPC creation;
- cross-process memory access, pidfd access/signalling, process signalling and `kcmp`;
- mount/namespace escape;
- open-by-handle and filesystem event-monitor surfaces;
- device-node creation;
- keyring access;
- BPF/perf/userfaultfd kernel instrumentation;
- io_uring setup/operation surfaces outside the reviewed synchronous syscall boundary.

The hardened profile rejects every nonempty device/IPC/inherited-handle request and rejects any inherited non-stdio descriptor before untrusted execution.

## Resource and terminal semantics

The v2 parent supervisor binds the exact v2 receipt and refuses to supervise unless identity-bound regular-file roots, IPC denial, cross-process denial and credential/kernel-surface denial are all present. Wall-time and combined stdout/stderr limits remain parent-enforced; termination requests remain non-terminal until exact root terminal observation. Spawn denial keeps the owned process tree at one root process for this first profile.

## Qualification posture

Neither v1 CI #1193 nor the unqualified v1 IPC repair qualifies v2. The replacement head must obtain fresh:

1. formatting and Clippy with warnings denied;
2. full workspace tests and existing adversarial/property/fuzz qualifications;
3. Linux x86_64 v2 hostile containment execution;
4. external descendant-aware strict-local observation;
5. exact-head Windows/macOS/Ubuntu CI success;
6. substantive independent semantic/security review on the unchanged v2 head;
7. reconciliation of every material finding before any `ADMITTED` record.

```text
V1_BASE_CI_RUN=33968106942
V1_BASE_CI_RESULT=SUCCESS_BUT_NOT_ADMISSION_SUFFICIENT
V1_IPC_HARDENING_HEAD=ac5199977ef759b1d03417e85b5a07a48dbdd4de
V1_IPC_HARDENING=RETAINED_FORWARD_ONLY
V1_PROFILE_ADMITTED=NO
V2_PROFILE_ADMITTED=NO_PENDING_T005_077
SOURCE_FOUNDRY_NEW_DEPENDENCY=NO
PROCESS_LAUNCH_ENABLED=NO
SHELL_ENABLED=NO
LOCAL_MCP_EXECUTION_ENABLED=NO
EXECUTABLE_SKILLS_ENABLED=NO
WAIVER_TAKEN=NO
NEXT_GATE=FRESH_V2_EXACT_HEAD_CI_THEN_INDEPENDENT_SEMANTIC_SECURITY_REVIEW
```
