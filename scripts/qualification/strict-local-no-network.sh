#!/usr/bin/env bash
set -euo pipefail

binary="${1:-target/debug/golamd}"
if ! command -v lsof >/dev/null 2>&1; then
  echo "strict-local qualification requires lsof on Unix runners" >&2
  exit 2
fi

root="$(mktemp -d "${TMPDIR:-/tmp}/golam-net.XXXXXX")"
stderr_log="$root/golamd.stderr"
stdout_log="$root/golamd.stdout"
pid=""

cleanup() {
  if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  fi
  rm -rf "$root"
}
trap cleanup EXIT

GOLAM_ROOT="$root" "$binary" --foreground </dev/null >"$stdout_log" 2>"$stderr_log" &
pid=$!

observe_no_inet_sockets() {
  local output
  output="$(lsof -nP -a -p "$pid" -i 2>/dev/null || true)"
  if [[ -n "$(printf '%s\n' "$output" | sed -n '2,$p')" ]]; then
    echo "Golam process owns an unexpected Internet socket:" >&2
    printf '%s\n' "$output" >&2
    return 1
  fi
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

echo "STRICT_LOCAL_INET_SOCKETS=0"
echo "LOCAL_IPC_LISTENER=OBSERVED"
