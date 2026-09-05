---
task_id: T005-078
scope: linux_x86_64_parent_death_lifecycle_feature_addendum
outcome: ADMITTED_FOR_GOVERNED_PROCESS_IMPLEMENTATION
recorded_on: 2026-09-05
package: nix
version: 0.31.3
new_feature: signal
implied_feature: process
production_profile_predecessor: platform:linux-x86_64-landlock-v4-seccomp-v2
production_profile_admitted_predecessor: true
process_launch_enabled_by_this_record: false
waiver_taken: false
---

# Source Foundry addendum — `nix 0.31.3` process-lifecycle surface

## Decision

T005-078 must not leave a launched payload alive if the trusted Golam parent dies after process creation but before ordinary wall-time/cancellation supervision completes. Parent-side observation alone cannot close that daemon-crash window.

The workspace therefore widens the already exact-pinned `nix = 0.31.3` dependency by enabling exactly the `signal` feature:

```toml
nix = { version = "=0.31.3", default-features = false, features = ["fs", "resource", "signal", "socket", "user"] }
```

For `nix 0.31.3`, `signal` depends on the crate's `process` feature. Neither feature adds a new normal package dependency. The package version and registry identity do not change.

Existing lock identity:

```text
package=nix
version=0.31.3
registry_checksum=9fb9654ba8355388abeb8dcb4fc62f511300867002afc858860463bdd9fe0c44
```

## Selected API surface

Only the following safe wrappers/types are admitted by this addendum for the Linux x86_64 governed executor:

- `nix::sys::prctl::set_pdeathsig` to bind kernel-delivered parent-death termination;
- `nix::sys::prctl::get_pdeathsig` for read-back verification of that binding;
- `nix::sys::signal::Signal::SIGKILL` as the non-catchable parent-death disposition;
- `nix::unistd::getppid` to close the documented parent-death race by comparing the helper's observed parent with the exact expected launcher PID after `PR_SET_PDEATHSIG` succeeds.

The trusted helper must fail closed unless all of the following hold before containment and one-way payload exec:

1. the expected parent PID is nonzero;
2. `set_pdeathsig(SIGKILL)` succeeds;
3. `get_pdeathsig()` reads back exactly `SIGKILL`;
4. `getppid()` still equals the expected parent PID.

If the parent died before the binding, the post-bind parent check detects reparenting and the helper exits without executing the payload. If the parent dies after the binding, the Linux kernel delivers `SIGKILL`. This complements, rather than replaces, the admitted parent-side wall-time/output/cancellation and terminal process-tree reconciliation boundary.

## Security posture

This addendum does not admit `fork`, `clone`, process groups, sessions, arbitrary signaling, signal handlers, ptrace, sched, mount, namespace manipulation or shell launch. Golam code remains `#![forbid(unsafe_code)]`; the syscall implementation stays inside the exact reviewed upstream dependency boundary.

The `signal` feature is admitted only because the typed `Signal::SIGKILL` value used by the safe `prctl` wrapper is feature-gated there, and that feature in turn enables `process`. No Golam signal handler is installed.

Parent-death binding is not terminal process evidence. T005-080 must still prove that normal termination, timeout, cancellation, daemon-parent loss and restart ambiguity fail closed and that success is never emitted without exact root/process-tree reconciliation.

## Non-admissions

```text
NIX_FORK=NOT_ADMITTED
NIX_CLONE=NOT_ADMITTED
ARBITRARY_SIGNAL_CONTROL=NOT_ADMITTED
SIGNAL_HANDLERS=NOT_ADMITTED
PROCESS_GROUP_WIDENING=NOT_ADMITTED
SHELL_LAUNCH=NOT_ADMITTED
EXTERNAL_SEARCH_BINARY=NOT_ADMITTED
LOCAL_MCP_EXECUTION=NOT_ADMITTED_BY_THIS_RECORD
EXECUTABLE_SKILLS=NOT_ADMITTED_BY_THIS_RECORD
MACOS_PROCESS_PROFILE=NOT_ADMITTED
WINDOWS_PROCESS_PROFILE=NOT_ADMITTED
```

## Result

```text
T005_078_NIX_PROCESS_LIFECYCLE_SOURCE_FOUNDRY=PASS
NIX_0_31_3_SIGNAL_FEATURE=ADMITTED_T005_078_LINUX_X86_64
NIX_0_31_3_PROCESS_FEATURE=IMPLIED_BY_SIGNAL
NEW_PACKAGE_VERSION=NO
NEW_NORMAL_PACKAGE_DEPENDENCY=NO
PDEATHSIG_REQUIRED=YES
POST_BIND_PARENT_RECHECK_REQUIRED=YES
PROCESS_LAUNCH_ENABLED=NO
WAIVER_TAKEN=NO
```
