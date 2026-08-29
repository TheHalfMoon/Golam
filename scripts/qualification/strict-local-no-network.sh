#!/usr/bin/env bash
set -euo pipefail

binary="${1:-target/debug/golamd}"
if ! command -v lsof >/dev/null 2>&1; then
  echo "strict-local qualification requires lsof on Unix runners" >&2
  exit 2
fi
if ! command -v ps >/dev/null 2>&1; then
  echo "strict-local qualification requires ps on Unix runners" >&2
  exit 2
fi

root="$(mktemp -d "${TMPDIR:-/tmp}/golam-net.XXXXXX")"
stderr_log="$root/golamd.stderr"
stdout_log="$root/golamd.stdout"
pid=""
max_managed_pids=0

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

observer_self_test() {
  local synthetic
  local observed
  local expected
  synthetic=$'200 100\n300 200\n400 999\n500 300'
  observed="$(descendants_from_snapshot 100 "$synthetic" | tr '\n' ' ' | sed 's/ $//')"
  expected="100 200 300 500"
  if [[ "$observed" != "$expected" ]]; then
    echo "process-tree observer self-test failed: expected '$expected', got '$observed'" >&2
    exit 1
  fi
}

managed_pids() {
  local snapshot
  snapshot="$(ps -Ao pid=,ppid=)"
  descendants_from_snapshot "$pid" "$snapshot"
}

cleanup() {
  if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
    local observed=""
    observed="$(managed_pids 2>/dev/null || true)"
    if [[ -n "$observed" ]]; then
      # Every PID here was captured as the daemon or one of its live descendants.
      # Terminate the observed tree so a reparented descendant cannot outlive qualification cleanup.
      # shellcheck disable=SC2086
      kill $observed 2>/dev/null || true
    fi
    wait "$pid" 2>/dev/null || true
  fi
  rm -rf "$root"
}
trap cleanup EXIT

observer_self_test

GOLAM_ROOT="$root" "$binary" --foreground </dev/null >"$stdout_log" 2>"$stderr_log" &
pid=$!

observe_no_inet_sockets() {
  local observed
  local count
  local observed_pid
  local output
  observed="$(managed_pids)"
  count="$(printf '%s\n' "$observed" | sed '/^$/d' | wc -l | tr -d ' ')"
  if [[ "$count" -gt "$max_managed_pids" ]]; then
    max_managed_pids="$count"
  fi

  while read -r observed_pid; do
    [[ -z "${observed_pid:-}" ]] && continue
    output="$(lsof -nP -a -p "$observed_pid" -i 2>/dev/null || true)"
    if [[ -n "$(printf '%s\n' "$output" | sed -n '2,$p')" ]]; then
      echo "Golam managed process tree owns an unexpected Internet socket (pid=$observed_pid):" >&2
      printf '%s\n' "$output" >&2
      return 1
    fi
  done <<< "$observed"
}

ready=0
for _ in $(seq 1 100); do
  if ! kill -0 "$pid" 2>/dev/null; then
    echo "golamd exited before local IPC readiness" >&2
    cat "$stderr_log" >&2 || true
    exit 1
  fi
  observe_no_inet_sockets
  if grep -q "golamd: listening on" "$stderr_log" 2>/dev/null; then
    ready=1
    break
  fi
  sleep 0.05
done

if [[ "$ready" -ne 1 ]]; then
  echo "golamd did not reach local IPC readiness" >&2
  cat "$stderr_log" >&2 || true
  exit 1
fi

for _ in $(seq 1 40); do
  observe_no_inet_sockets
  sleep 0.05
done

if ! grep -q "golamd: listening on" "$stderr_log"; then
  echo "local IPC listener evidence is missing" >&2
  exit 1
fi

if [[ "$max_managed_pids" -lt 1 ]]; then
  echo "managed process-tree observer did not capture golamd" >&2
  exit 1
fi

echo "PROCESS_TREE_TRAVERSAL_SELF_TEST=PASS"
echo "MANAGED_PROCESS_TREE_OBSERVER=ENABLED"
echo "MAX_MANAGED_PIDS_OBSERVED=$max_managed_pids"
echo "STRICT_LOCAL_INET_SOCKETS=0"
echo "LOCAL_IPC_LISTENER=OBSERVED"
