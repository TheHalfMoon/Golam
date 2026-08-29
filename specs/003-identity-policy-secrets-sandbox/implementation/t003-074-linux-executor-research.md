# T003-074 Linux Executor Primitive Research

## Disposition

Ubuntu-hosted qualification proved a privileged setup / unprivileged payload Bubblewrap plus seccomp primitive suitable only for the minimum native untrusted-process test executor required by T003-074.

The GitHub-hosted runner does not support the unprivileged user/network namespace paths attempted during qualification. The admitted test-only primitive therefore deliberately does not claim user-namespace or network-namespace isolation. The trusted qualification launcher creates an isolated mount/PID/IPC/UTS/session boundary, installs seccomp, and then reduces the payload to uid/gid 65534 with empty capability sets and `no_new_privs`.

The host network namespace remains shared, but a fixed x86_64 seccomp filter returns EPERM for socket/connect/listen/bind/accept/socketpair syscalls before payload execution.

This record does not admit Bubblewrap or sudo as a product sandbox dependency, does not claim macOS/Windows parity, and does not alter the production `native:unqualified` capability baseline.

## Qualified primitive properties

- Bubblewrap and setpriv versions are captured in the run log.
- Mount/PID/IPC/UTS/session/seccomp setup occurs in the trusted qualification launcher.
- The child payload runs as uid/gid 65534 with cleared environment, empty capability sets, and `no_new_privs`.
- A host-only forbidden file outside explicit read-only runtime mounts is not visible.
- Socket creation is denied before payload use despite sharing the host network namespace.
- A descendant process executes within the managed PID/session boundary and the parent-death boundary remains active.

## Boundaries

- Linux x86_64 qualification only; this is not a product executor admission.
- User and network namespace isolation are not claimed.
- macOS, Windows, non-x86_64 Linux, external-network profiles, arbitrary requester executable paths, and universal native isolation remain unsupported.
- No network-capable Golam-managed native child was launched.

```text
T003_074_LINUX_PRIMITIVE_RESEARCH=PASS
EXECUTOR_PRIMITIVE=PRIVILEGED_SETUP_BWRAP_SECCOMP_TEST_ONLY
QUALIFICATION_LAUNCHER_PRIVILEGED=YES
PAYLOAD_UID_GID_UNPRIVILEGED=YES
USER_NAMESPACE_ISOLATION_CLAIMED=NO
NETWORK_NAMESPACE_ISOLATION_CLAIMED=NO
ENVIRONMENT_CLEARED=YES
FORBIDDEN_HOST_FS_VISIBLE=NO
NETWORK_SOCKET_SYSCALL_DENIED=YES
PAYLOAD_CAPABILITIES_DROPPED=YES
MANAGED_DESCENDANT_EXECUTED=YES
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
MACOS_EXECUTOR_SUPPORTED=NO
WINDOWS_EXECUTOR_SUPPORTED=NO
UNIVERSAL_NATIVE_SANDBOX_CLAIMED=NO
NETWORK_CAPABLE_MANAGED_CHILD_LAUNCHED=NO
```
