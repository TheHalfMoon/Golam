#!/usr/bin/env bash
set -euo pipefail

binary="${1:-target/debug/golam-native-containment-hostile-probe}"
if [[ ! -x "$binary" ]]; then
  echo "native containment hostile probe is missing or not executable: $binary" >&2
  exit 2
fi
if ! command -v ps >/dev/null 2>&1; then
  echo "native containment qualification requires ps" >&2
  exit 2
fi
if ! command -v lsof >/dev/null 2>&1; then
  echo "native containment qualification requires lsof" >&2
  exit 2
fi

binary="$(realpath "$binary")"
root="$(mktemp -d "${TMPDIR:-/tmp}/golam-native-containment.XXXXXX")"
stdout_log="$root/stdout.log"
stderr_log="$root/stderr.log"
pid=""

cleanup() {
  if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  fi
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

# The helper itself requires an actually cleared ambient environment.
env -i "$binary" >"$stdout_log" 2>"$stderr_log" &
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

snapshot="$(ps -Ao pid=,ppid=)"
managed="$(descendants_from_snapshot "$pid" "$snapshot")"
managed_count="$(printf '%s\n' "$managed" | sed '/^$/d' | wc -l | tr -d ' ')"
if [[ "$managed_count" -ne 1 ]]; then
  echo "spawn-denied profile exposed unexpected descendants:" >&2
  printf '%s\n' "$managed" >&2
  exit 1
fi

while read -r observed_pid; do
  [[ -z "${observed_pid:-}" ]] && continue
  inet="$(lsof -nP -a -p "$observed_pid" -i 2>/dev/null || true)"
  if [[ -n "$(printf '%s\n' "$inet" | sed -n '2,$p')" ]]; then
    echo "native containment process owns an unexpected Internet socket:" >&2
    printf '%s\n' "$inet" >&2
    exit 1
  fi
done <<< "$managed"

wait "$pid"
pid=""

for marker in \
  'CONTAINMENT_APPLIED=YES' \
  'SPAWN_DENIED=true' \
  'STRICT_LOCAL=true' \
  'NETWORK_ESCAPE_DENIED=YES' \
  'PROCESS_SPAWN_DENIED=YES' \
  'FILESYSTEM_WRITE_ESCAPE_DENIED=YES' \
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

echo "NATIVE_CONTAINMENT_PROFILE_APPLIED=YES"
echo "NATIVE_CONTAINMENT_EXTERNAL_PROCESS_TREE_COUNT=$managed_count"
echo "NATIVE_CONTAINMENT_INET_SOCKETS=0"
echo "NATIVE_CONTAINMENT_SOCKET_ESCAPE=DENIED"
echo "NATIVE_CONTAINMENT_PROCESS_SPAWN=DENIED"
echo "NATIVE_CONTAINMENT_FORBIDDEN_WRITE=DENIED"
echo "PRODUCTION_PROFILE_ADMITTED=NO"
