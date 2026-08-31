#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" || "$(uname -m)" != "x86_64" ]]; then
  echo "T003-074 native executor qualification requires Linux x86_64" >&2
  exit 2
fi
for command in bwrap setpriv python3 sudo; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "T003-074 native executor qualification requires $command" >&2
    exit 2
  fi
done

root="$(mktemp -d "${TMPDIR:-/tmp}/golam-t003-074.XXXXXX")"
filter="$root/network-deny.bpf"
launcher="$root/launcher"
forbidden="$root/host-only-forbidden"
trap 'rm -rf "$root"' EXIT
printf 'host-only\n' > "$forbidden"

python3 - "$filter" <<'PY'
import struct
import sys
from pathlib import Path

BPF_LD_W_ABS = 0x20
BPF_JMP_JEQ_K = 0x15
BPF_RET_K = 0x06
AUDIT_ARCH_X86_64 = 0xC000003E
SECCOMP_RET_KILL_PROCESS = 0x80000000
SECCOMP_RET_ERRNO_EPERM = 0x00050001
SECCOMP_RET_ALLOW = 0x7FFF0000
blocked = [41, 42, 43, 49, 50, 53, 288]
insns = [
    (BPF_LD_W_ABS, 0, 0, 4),
    (BPF_JMP_JEQ_K, 1, 0, AUDIT_ARCH_X86_64),
    (BPF_RET_K, 0, 0, SECCOMP_RET_KILL_PROCESS),
    (BPF_LD_W_ABS, 0, 0, 0),
]
for syscall in blocked:
    insns.append((BPF_JMP_JEQ_K, 0, 1, syscall))
    insns.append((BPF_RET_K, 0, 0, SECCOMP_RET_ERRNO_EPERM))
insns.append((BPF_RET_K, 0, 0, SECCOMP_RET_ALLOW))
Path(sys.argv[1]).write_bytes(
    b"".join(struct.pack("<HBBI", *insn) for insn in insns)
)
PY

cat >"$launcher" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
filter="$1"
forbidden="$2"
exec 3<"$filter"
exec bwrap \
  --unshare-pid \
  --unshare-ipc \
  --unshare-uts \
  --unshare-cgroup-try \
  --die-with-parent \
  --new-session \
  --clearenv \
  --ro-bind /usr /usr \
  --ro-bind /bin /bin \
  --ro-bind-try /lib /lib \
  --ro-bind-try /lib64 /lib64 \
  --proc /proc \
  --dir /dev \
  --dev-bind /dev/null /dev/null \
  --tmpfs /tmp \
  --chmod 1777 /tmp \
  --seccomp 3 \
  --chdir /tmp \
  /usr/bin/setpriv \
    --reuid=65534 \
    --regid=65534 \
    --clear-groups \
    --bounding-set=-all \
    --inh-caps=-all \
    --ambient-caps=-all \
    --no-new-privs \
    /bin/bash -ceu '
      test "$(id -u)" = 65534
      test "$(id -g)" = 65534
      test -z "${GOLAM_AMBIENT_SECRET+x}"
      test ! -e "$1"
      test -e /dev/null
      test ! -e /dev/zero
      grep -q "^CapInh:[[:space:]]*0000000000000000$" /proc/self/status
      grep -q "^CapPrm:[[:space:]]*0000000000000000$" /proc/self/status
      grep -q "^CapEff:[[:space:]]*0000000000000000$" /proc/self/status
      grep -q "^CapBnd:[[:space:]]*0000000000000000$" /proc/self/status
      grep -q "^CapAmb:[[:space:]]*0000000000000000$" /proc/self/status
      grep -q "^NoNewPrivs:[[:space:]]*1$" /proc/self/status
      printf allowed > /tmp/allowed
      test "$(cat /tmp/allowed)" = allowed
      if printf forbidden > /usr/golam-forbidden 2>/dev/null; then
        echo "read-only runtime root write unexpectedly succeeded" >&2
        exit 31
      fi
      /bin/bash -ceu '\''
        test -z "${GOLAM_AMBIENT_SECRET+x}"
        if /usr/bin/timeout 2 /bin/bash -c "exec 4<>/dev/tcp/1.1.1.1/53" 2>/dev/null; then
          echo "managed descendant obtained external network capability" >&2
          exit 32
        fi
      '\''
    ' golam-native-test "$forbidden"
SH
chmod 0755 "$launcher"

sudo env GOLAM_AMBIENT_SECRET='must-not-cross-boundary' "$launcher" "$filter" "$forbidden"

echo "T003_074_NATIVE_EXECUTOR=PASS"
echo "QUALIFICATION_LAUNCHER_PRIVILEGED=YES"
echo "PAYLOAD_UID_GID_UNPRIVILEGED=YES"
echo "USER_NAMESPACE_ISOLATION_CLAIMED=NO"
echo "NETWORK_NAMESPACE_ISOLATION_CLAIMED=NO"
echo "ENVIRONMENT_CLEARED=YES"
echo "FORBIDDEN_HOST_FS_VISIBLE=NO"
echo "READONLY_RUNTIME_ROOT_WRITE_BLOCKED=YES"
echo "NETWORK_SOCKET_SYSCALL_DENIED=YES"
echo "PAYLOAD_CAPABILITIES_DROPPED=YES"
echo "MANAGED_DESCENDANT_EXECUTED=YES"
echo "PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO"
echo "UNIVERSAL_NATIVE_SANDBOX_CLAIMED=NO"
echo "NETWORK_CAPABLE_MANAGED_CHILD_LAUNCHED=NO"
