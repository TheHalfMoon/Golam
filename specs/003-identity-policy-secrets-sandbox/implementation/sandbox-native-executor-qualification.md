# T003-074 Minimum Native Sandbox Executor Qualification

**Status**: PASS  
**Qualified implementation head**: `0d28681c971b3ae1f08504c7eb2448789ac8ed6e`  
**Qualified tree**: `32379e52964faca7db84b8354947be825ba2ef64`  
**Official qualification**: CI #592 / run `33247996101` — SUCCESS on Windows, macOS, and Ubuntu.  
**Focused native-executor qualification**: run `33247892761` — SUCCESS.

## Qualified boundary

T003-074 reuses the descendant-aware strict-local managed-process-tree observer already qualified by T003-064 and establishes only the minimum native untrusted-process test profile/executor required by the frozen contract.

The Rust qualification profile is test-only. It is intersected through the T003-072 enforcement descriptor, requires strict-local `DenyAll` networking, clears the environment, exposes only explicit filesystem/device rights, allows only managed descendants, and carries the exact platform requirement `platform:linux-x86_64-bwrap-seccomp-test-v1`.

The production T003-073 capability baseline remains `native:unqualified` with zero containment controls. The test profile therefore cannot become a production launch authority and continues to fail closed through normal production capability resolution.

## Linux x86_64 executor evidence

The durable qualification script `scripts/qualification/t003-074-native-executor.sh` proves the bounded OS boundary used by the test profile:

- trusted qualification setup creates mount, PID, IPC, UTS, session and parent-death boundaries;
- payload execution is reduced to uid/gid 65534;
- capability inheritable, permitted, effective, bounding and ambient sets are all empty;
- `no_new_privs` is set before untrusted payload execution;
- ambient environment is cleared;
- host paths outside explicit runtime mounts are not visible;
- runtime roots are read-only and sandbox-local `/tmp` is the only qualified writable root;
- `/dev/null` is the only explicitly exposed device;
- a fixed x86_64 seccomp filter denies socket/connect/listen/bind/accept/socketpair syscalls before payload execution;
- a managed descendant executes under the same no-network/no-ambient-environment boundary.

The GitHub-hosted Ubuntu runner does not support the attempted unprivileged user/network namespace paths. User-namespace and network-namespace isolation are therefore explicitly not claimed. The host network namespace is shared only inside this test harness while seccomp removes socket capability before payload execution.

## Honesty boundaries

- Bubblewrap and sudo are qualification-harness dependencies only; they are not admitted as Golam product runtime dependencies.
- macOS, Windows, non-x86_64 Linux, external-network profiles, arbitrary requester executable paths and universal native isolation remain unsupported by this concrete executor.
- No network-capable Golam-managed native child was launched.
- No platform containment claim transfers from this bounded test harness to the production `native:unqualified` baseline.
- `WASMTIME_DISPOSITION=NOT_ADMITTED_NOT_NEEDED` remains unchanged.

```text
T003_074=PASS
T003_074_QUALIFIED_HEAD=0d28681c971b3ae1f08504c7eb2448789ac8ed6e
T003_074_QUALIFIED_TREE=32379e52964faca7db84b8354947be825ba2ef64
T003_074_CI_RUN=33247996101
T003_074_FOCUSED_RUN=33247892761
MANAGED_PROCESS_TREE_OBSERVER=QUALIFIED
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
USER_NAMESPACE_ISOLATION_CLAIMED=NO
NETWORK_NAMESPACE_ISOLATION_CLAIMED=NO
UNIVERSAL_NATIVE_SANDBOX_CLAIMED=NO
NETWORK_CAPABLE_MANAGED_CHILD_LAUNCHED=NO
NEXT_TASK=T003-075
```
