#!/usr/bin/env bash
set -euo pipefail

binary="${1:-target/debug/golam-native-containment-hostile-probe}"
if [[ ! -x "$binary" ]]; then
  echo "native containment hostile probe is missing or not executable: $binary" >&2
  exit 2
fi
for command in ps lsof realpath; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "native containment qualification requires $command" >&2
    exit 2
  fi
done

binary="$(realpath "$binary")"
root="$(mktemp -d "${TMPDIR:-/tmp}/golam-native-containment.XXXXXX")"
stdout_log="$root/stdout.log"
stderr_log="$root/stderr.log"
pid=""
cancel_pid=""
resource_pid=""

cleanup() {
  for cleanup_pid in "$pid" "$cancel_pid" "$resource_pid"; do
    if [[ -n "$cleanup_pid" ]] && kill -0 "$cleanup_pid" 2>/dev/null; then
      kill "$cleanup_pid" 2>/dev/null || true
      wait "$cleanup_pid" 2>/dev/null || true
    fi
  done
  rm -rf "$root"
}
trap cleanup EXIT

descendants_from_snapshot() {
  local root_pid="$1"
  local snapshot="$2"
  local pids="$root_pid"
  local previous=""
  local child=""
  local parent=""
  while [[ "$pids" != "$previous" ]]; do
    previous="$pids"
    while read -r child parent; do
      [[ -z "${child:-}" || -z "${parent:-}" ]] && continue
      if printf '%s\n' "$pids" | grep -qx "$parent" \
        && ! printf '%s\n' "$pids" | grep -qx "$child"; then
        pids="${pids}"$'\n'"${child}"
      fi
    done <<< "$snapshot"
  done
  printf '%s\n' "$pids" | sed '/^$/d' | sort -n -u
}

assert_single_pid_tree() {
  local root_pid="$1"
  local snapshot
  local managed
  local count
  snapshot="$(ps -Ao pid=,ppid=)"
  managed="$(descendants_from_snapshot "$root_pid" "$snapshot")"
  count="$(printf '%s\n' "$managed" | sed '/^$/d' | wc -l | tr -d ' ')"
  if [[ "$count" -ne 1 ]]; then
    echo "spawn-denied profile exposed unexpected descendants:" >&2
    printf '%s\n' "$managed" >&2
    return 1
  fi
  printf '%s\n' "$managed"
}

assert_no_inet_sockets() {
  local managed="$1"
  local observed_pid
  local inet
  while read -r observed_pid; do
    [[ -z "${observed_pid:-}" ]] && continue
    inet="$(lsof -nP -a -p "$observed_pid" -i 2>/dev/null || true)"
    if [[ -n "$(printf '%s\n' "$inet" | sed -n '2,$p')" ]]; then
      echo "native containment process owns an unexpected Internet socket:" >&2
      printf '%s\n' "$inet" >&2
      return 1
    fi
  done <<< "$managed"
}

# The trusted launcher boundary must actually remove ambient descriptors before starting the
# containment helper. `env -i` clears environment variables only; GitHub-hosted runners may
# legitimately carry unrelated high-numbered descriptors in the invoking shell. This function
# is always backgrounded, so Bash already gives it the single process that must become the probe;
# introducing another subshell here would create a wrapper descendant and falsify tree evidence.
run_clean_probe() {
  local fd_path
  local fd
  shopt -s nullglob
  for fd_path in "/proc/${BASHPID}/fd/"*; do
    fd="${fd_path##*/}"
    if [[ "$fd" =~ ^[0-9]+$ ]] && (( fd > 2 )); then
      exec {fd}>&-
    fi
  done
  exec env -i "$binary" "$@"
}

run_clean_probe_to_completion() {
  local stdout_path="$1"
  local stderr_path="$2"
  shift 2
  run_clean_probe "$@" >"$stdout_path" 2>"$stderr_path" &
  resource_pid=$!
  if ! wait "$resource_pid"; then
    resource_pid=""
    echo "native supervisor hostile probe failed: $*" >&2
    cat "$stdout_path" >&2 || true
    cat "$stderr_path" >&2 || true
    return 1
  fi
  resource_pid=""
  if [[ -s "$stderr_path" ]]; then
    echo "native supervisor hostile probe emitted unexpected stderr: $*" >&2
    cat "$stderr_path" >&2
    return 1
  fi
}

# Non-empty ambient environment must fail before untrusted execution.
if env -i GOLAM_HOSTILE_ENV_CANARY=1 "$binary" >"$root/env.stdout" 2>"$root/env.stderr"; then
  echo "non-empty ambient environment unexpectedly passed containment admission" >&2
  exit 1
fi
if ! grep -q 'requires a cleared ambient environment' "$root/env.stderr"; then
  echo "ambient-environment denial evidence is missing" >&2
  cat "$root/env.stderr" >&2 || true
  exit 1
fi

# An inherited non-stdio descriptor must fail before untrusted execution.
if (
  exec 9</dev/null
  env -i "$binary" >"$root/fd.stdout" 2>"$root/fd.stderr"
); then
  echo "inherited descriptor unexpectedly passed containment admission" >&2
  exit 1
fi
if ! grep -q 'inherited an undeclared descriptor' "$root/fd.stderr"; then
  echo "inherited-descriptor denial evidence is missing" >&2
  cat "$root/fd.stderr" >&2 || true
  exit 1
fi

# Normal hostile payload: actual empty ambient environment, no inherited extra descriptors.
run_clean_probe >"$stdout_log" 2>"$stderr_log" &
pid=$!

ready=0
for _ in $(seq 1 100); do
  if ! kill -0 "$pid" 2>/dev/null; then
    echo "native containment hostile probe exited before readiness" >&2
    cat "$stdout_log" >&2 || true
    cat "$stderr_log" >&2 || true
    exit 1
  fi
  if grep -q '^HOSTILE_PROBE_READY=YES$' "$stdout_log" 2>/dev/null; then
    ready=1
    break
  fi
  sleep 0.03
done

if [[ "$ready" -ne 1 ]]; then
  echo "native containment hostile probe did not reach readiness" >&2
  cat "$stdout_log" >&2 || true
  cat "$stderr_log" >&2 || true
  exit 1
fi

managed="$(assert_single_pid_tree "$pid")"
managed_count="$(printf '%s\n' "$managed" | sed '/^$/d' | wc -l | tr -d ' ')"
assert_no_inet_sockets "$managed"

wait "$pid"
pid=""

for marker in \
  'CONTAINMENT_APPLIED=YES' \
  'SPAWN_DENIED=true' \
  'STRICT_LOCAL=true' \
  'LINUX_CAPABILITY_SETS_EMPTY=YES' \
  'IDENTITY_BOUND_REGULAR_FILE_ROOTS=YES' \
  'IPC_CREATION_DENIED=YES' \
  'CROSS_PROCESS_CONTROL_DENIED=YES' \
  'CREDENTIAL_KERNEL_SURFACES_DENIED=YES' \
  'NETWORK_ESCAPE_DENIED=YES' \
  'IPC_SOCKETPAIR_DENIED=YES' \
  'PROCESS_SPAWN_DENIED=YES' \
  'FILESYSTEM_WRITE_ESCAPE_DENIED=YES' \
  'DEVICE_ACCESS_DENIED=YES' \
  'HOSTILE_PROBE_COMPLETE=YES'; do
  if ! grep -q "^${marker}$" "$stdout_log"; then
    echo "native containment evidence marker missing: $marker" >&2
    cat "$stdout_log" >&2 || true
    cat "$stderr_log" >&2 || true
    exit 1
  fi
done

if [[ -s "$stderr_log" ]]; then
  echo "native containment hostile probe emitted unexpected stderr" >&2
  cat "$stderr_log" >&2
  exit 1
fi

# Cancellation is a request, not terminal proof. Observe the contained root, request termination,
# then require an actual OS terminal observation and no pre-cancel descendants.
cancel_stdout="$root/cancel.stdout"
cancel_stderr="$root/cancel.stderr"
run_clean_probe --cancel-hold >"$cancel_stdout" 2>"$cancel_stderr" &
cancel_pid=$!
cancel_ready=0
for _ in $(seq 1 100); do
  if ! kill -0 "$cancel_pid" 2>/dev/null; then
    echo "cancel-hold probe exited before cancellation readiness" >&2
    cat "$cancel_stdout" >&2 || true
    cat "$cancel_stderr" >&2 || true
    exit 1
  fi
  if grep -q '^CANCEL_HOLD_READY=YES$' "$cancel_stdout" 2>/dev/null; then
    cancel_ready=1
    break
  fi
  sleep 0.03
done
if [[ "$cancel_ready" -ne 1 ]]; then
  echo "cancel-hold probe did not reach readiness" >&2
  exit 1
fi

cancel_managed="$(assert_single_pid_tree "$cancel_pid")"
assert_no_inet_sockets "$cancel_managed"
kill "$cancel_pid"
if wait "$cancel_pid"; then
  echo "cancel-hold probe unexpectedly reported normal success after cancellation" >&2
  exit 1
fi
cancel_pid=""

while read -r observed_pid; do
  [[ -z "${observed_pid:-}" ]] && continue
  if kill -0 "$observed_pid" 2>/dev/null; then
    echo "managed pid persisted after terminal cancellation observation: $observed_pid" >&2
    exit 1
  fi
done <<< "$cancel_managed"

if [[ -s "$cancel_stderr" ]]; then
  echo "cancel-hold probe emitted unexpected stderr before termination" >&2
  cat "$cancel_stderr" >&2
  exit 1
fi

# Parent-side resource enforcement is part of the same profile. These qualification-only parent
# modes launch the exact contained helper, feed real monotonic time/output observations through
# the production supervisor primitive, request termination at the bound, and require exact
# terminal reconciliation. They do not admit or expose a production process-launch path.
wall_stdout="$root/wall.stdout"
wall_stderr="$root/wall.stderr"
run_clean_probe_to_completion "$wall_stdout" "$wall_stderr" --supervisor-wall-time
for marker in \
  'SUPERVISOR_WALL_TIME_ENFORCED=YES' \
  'SUPERVISOR_WALL_TIME_TERMINAL_RECONCILED=YES'; do
  if ! grep -q "^${marker}$" "$wall_stdout"; then
    echo "wall-time supervisor evidence marker missing: $marker" >&2
    cat "$wall_stdout" >&2 || true
    exit 1
  fi
done

output_stdout="$root/output.stdout"
output_stderr="$root/output.stderr"
run_clean_probe_to_completion "$output_stdout" "$output_stderr" --supervisor-output-limit
for marker in \
  'SUPERVISOR_OUTPUT_LIMIT_ENFORCED=YES' \
  'SUPERVISOR_OUTPUT_COMBINED_STDOUT_STDERR=YES' \
  'SUPERVISOR_OUTPUT_TERMINAL_RECONCILED=YES'; do
  if ! grep -q "^${marker}$" "$output_stdout"; then
    echo "output supervisor evidence marker missing: $marker" >&2
    cat "$output_stdout" >&2 || true
    exit 1
  fi
done
accepted_output="$(sed -n 's/^SUPERVISOR_OUTPUT_ACCEPTED_BYTES=//p' "$output_stdout")"
if [[ -z "$accepted_output" || ! "$accepted_output" =~ ^[0-9]+$ || "$accepted_output" -gt 1024 ]]; then
  echo "output supervisor retained bytes beyond the hostile bound: ${accepted_output:-missing}" >&2
  exit 1
fi

echo "NATIVE_CONTAINMENT_PROFILE_APPLIED=YES"
echo "NATIVE_CONTAINMENT_EXTERNAL_PROCESS_TREE_COUNT=$managed_count"
echo "NATIVE_CONTAINMENT_INET_SOCKETS=0"
echo "NATIVE_CONTAINMENT_SOCKET_ESCAPE=DENIED"
echo "NATIVE_CONTAINMENT_LOCAL_IPC_SOCKETPAIR=DENIED"
echo "NATIVE_CONTAINMENT_PROCESS_SPAWN=DENIED"
echo "NATIVE_CONTAINMENT_FORBIDDEN_WRITE=DENIED"
echo "NATIVE_CONTAINMENT_DEVICE_ACCESS=DENIED"
echo "NATIVE_CONTAINMENT_AMBIENT_ENVIRONMENT=DENIED"
echo "NATIVE_CONTAINMENT_INHERITED_DESCRIPTOR=DENIED"
echo "NATIVE_CONTAINMENT_LINUX_CAPABILITIES=EMPTY"
echo "NATIVE_CONTAINMENT_IDENTITY_BOUND_REGULAR_FILE_ROOTS=YES"
echo "NATIVE_CONTAINMENT_IPC_CREATION=DENIED"
echo "NATIVE_CONTAINMENT_CROSS_PROCESS_CONTROL=DENIED"
echo "NATIVE_CONTAINMENT_CREDENTIAL_KERNEL_SURFACES=DENIED"
echo "NATIVE_CONTAINMENT_CANCEL_REQUEST=NON_TERMINAL"
echo "NATIVE_CONTAINMENT_CANCEL_TERMINAL_OBSERVED=YES"
echo "NATIVE_CONTAINMENT_DESCENDANT_PERSISTENCE=0"
echo "NATIVE_CONTAINMENT_WALL_TIME_LIMIT=ENFORCED"
echo "NATIVE_CONTAINMENT_OUTPUT_LIMIT=ENFORCED"
echo "NATIVE_CONTAINMENT_OUTPUT_COMBINED_STDOUT_STDERR=YES"
echo "NATIVE_CONTAINMENT_OUTPUT_ACCEPTED_BYTES=$accepted_output"
echo "PRODUCTION_PROFILE_ADMITTED=NO"
