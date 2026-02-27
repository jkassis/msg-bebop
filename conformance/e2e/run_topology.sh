#!/bin/bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TOPOLOGY="${1:-go,ts,rust}"
HOST="127.0.0.1"

IFS=',' read -r CLIENT_LANG MID_LANG TERMINAL_LANG <<<"$TOPOLOGY"
if [[ -z "${CLIENT_LANG}" || -z "${MID_LANG}" || -z "${TERMINAL_LANG}" ]]; then
  echo "usage: $0 <client,mid,terminal>" >&2
  exit 1
fi

port_in_use() {
  local port="$1"
  if command -v lsof >/dev/null 2>&1; then
    lsof -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1
  else
    (echo >"/dev/tcp/${HOST}/${port}") >/dev/null 2>&1
  fi
}

pick_port() {
  local port
  while true; do
    port=$((20000 + RANDOM % 20000))
    if ! port_in_use "$port"; then
      echo "$port"
      return 0
    fi
  done
}

run_server() {
  local lang="$1"
  local node="$2"
  local listen="$3"
  local next="${4:-}"
  case "$lang" in
    rust)
      if [[ -n "$next" ]]; then
        cargo run --quiet --manifest-path rust/Cargo.toml --bin interop -- \
          --mode server --node "$node" --listen "$listen" --next "$next" --once
      else
        cargo run --quiet --manifest-path rust/Cargo.toml --bin interop -- \
          --mode server --node "$node" --listen "$listen" --once
      fi
      ;;
    go)
      if [[ -n "$next" ]]; then
        (cd golang/src && env -u GOROOT go run ./cmd/interop \
          --mode server --node "$node" --listen "$listen" --next "$next" --once)
      else
        (cd golang/src && env -u GOROOT go run ./cmd/interop \
          --mode server --node "$node" --listen "$listen" --once)
      fi
      ;;
    ts)
      if [[ -n "$next" ]]; then
        (cd typescript && npx tsx src/cmd/interop.ts \
          --mode server --node "$node" --listen "$listen" --next "$next" --once)
      else
        (cd typescript && npx tsx src/cmd/interop.ts \
          --mode server --node "$node" --listen "$listen" --once)
      fi
      ;;
    *)
      echo "unsupported server lang: $lang" >&2
      return 1
      ;;
  esac
}

run_client() {
  local lang="$1"
  local node="$2"
  local addr="$3"
  local hops="$4"
  local expect_ack_from="$5"
  case "$lang" in
    rust)
      cargo run --quiet --manifest-path rust/Cargo.toml --bin interop -- \
        --mode client --node "$node" --addr "$addr" --expect-hops "$hops" --expect-ack-from "$expect_ack_from"
      ;;
    go)
      (cd golang/src && env -u GOROOT go run ./cmd/interop \
        --mode client --node "$node" --addr "$addr" --expect-hops "$hops" --expect-ack-from "$expect_ack_from")
      ;;
    ts)
      (cd typescript && npx tsx src/cmd/interop.ts \
        --mode client --node "$node" --addr "$addr" --expect-hops "$hops" --expect-ack-from "$expect_ack_from")
      ;;
    *)
      echo "unsupported client lang: $lang" >&2
      return 1
      ;;
  esac
}

cleanup() {
  jobs -pr | xargs -r kill 2>/dev/null || true
}
trap cleanup EXIT

cd "$ROOT"

MID_PORT="$(pick_port)"
TERM_PORT="$(pick_port)"
while [[ "$TERM_PORT" == "$MID_PORT" ]]; do TERM_PORT="$(pick_port)"; done

TERM_ADDR="${HOST}:${TERM_PORT}"
MID_ADDR="${HOST}:${MID_PORT}"
EXPECT_HOPS="${CLIENT_LANG},${MID_LANG},${TERMINAL_LANG}"

run_server "$TERMINAL_LANG" "$TERMINAL_LANG" "$TERM_ADDR" &
TERM_PID=$!
sleep 0.4

run_server "$MID_LANG" "$MID_LANG" "$MID_ADDR" "$TERM_ADDR" &
MID_PID=$!
sleep 0.4

run_client "$CLIENT_LANG" "$CLIENT_LANG" "$MID_ADDR" "$EXPECT_HOPS" "$TERMINAL_LANG"

wait "$MID_PID"
wait "$TERM_PID"

echo "PASS ${CLIENT_LANG}->${MID_LANG}->${TERMINAL_LANG} interop"
