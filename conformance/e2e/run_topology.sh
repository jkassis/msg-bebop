#!/bin/bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TOPOLOGY="${1:-go,ts,rust}"
COUNT="${2:-1}"
ACK_MODE="${3:-normal}"
EXPECT_FAILURE="${4:-0}"
DROP_FIRST="${5:-0}"
RETRIES="${6:-0}"
RETRY_DELAY_MS="${7:-100}"
TIMEOUT_MS="${8:-2000}"
RELAY_FAULT_MODE="${9:-normal}"
RELAY_DELAY_MS="${10:-0}"
HOST="127.0.0.1"
LOG_DIR="${ROOT}/conformance/e2e/logs"
mkdir -p "$LOG_DIR"
PORT_RANGE_START="${COURIER_INTEROP_PORT_START:-23000}"
PORT_RANGE_END="${COURIER_INTEROP_PORT_END:-23999}"

IFS=',' read -r CLIENT_LANG MID_LANG TERMINAL_LANG <<<"$TOPOLOGY"
if [[ -z "${CLIENT_LANG}" || -z "${MID_LANG}" || -z "${TERMINAL_LANG}" ]]; then
  echo "usage: $0 <client,mid,terminal>" >&2
  exit 1
fi
LOG_PREFIX="${LOG_DIR}/$(date +%Y%m%d-%H%M%S)-${CLIENT_LANG}-${MID_LANG}-${TERMINAL_LANG}"

port_in_use() {
  local port="$1"
  if command -v lsof >/dev/null 2>&1; then
    lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1
  elif command -v ss >/dev/null 2>&1; then
    ss -ltn "( sport = :${port} )" 2>/dev/null | awk 'NR>1 {found=1} END {exit !found}'
  elif command -v netstat >/dev/null 2>&1; then
    netstat -an 2>/dev/null | grep -E "[\.\:]${port}[[:space:]].*LISTEN" >/dev/null 2>&1
  else
    return 1
  fi
}

pick_port() {
  local span=$((PORT_RANGE_END - PORT_RANGE_START + 1))
  local attempts=200
  local port
  while ((attempts > 0)); do
    port=$((PORT_RANGE_START + RANDOM % span))
    if ! port_in_use "$port"; then
      echo "$port"
      return 0
    fi
    attempts=$((attempts - 1))
  done
  echo "no free port in range ${PORT_RANGE_START}-${PORT_RANGE_END}" >&2
  return 1
}

wait_for_server_ready() {
  local pid="$1"
  local logfile="$2"
  local retries=1200
  while ((retries > 0)); do
    if [[ -f "$logfile" ]] && grep -q "INTEROP_READY " "$logfile"; then
      return 0
    fi
    if ! kill -0 "$pid" 2>/dev/null; then
      echo "server pid ${pid} exited before signaling readiness" >&2
      if [[ -f "$logfile" ]]; then
        tail -n 40 "$logfile" >&2 || true
      fi
      return 1
    fi
    sleep 0.05
    retries=$((retries - 1))
  done
  echo "timeout waiting for server readiness (pid ${pid})" >&2
  if [[ -f "$logfile" ]]; then
    tail -n 40 "$logfile" >&2 || true
  fi
  return 1
}

run_server() {
  local lang="$1"
  local node="$2"
  local listen="$3"
  local next="${4:-}"
  local max_requests="${5:-1}"
  local ack_mode="${6:-normal}"
  local drop_first="${7:-0}"
  local delay_ms="${8:-0}"
  case "$lang" in
    rust)
      local drop_flag=""
      if [[ "$drop_first" == "1" ]]; then drop_flag="--drop-first"; fi
      if [[ -n "$next" ]]; then
        cargo run --quiet --offline --manifest-path rust/Cargo.toml --bin interop -- \
          --mode server --node "$node" --listen "$listen" --next "$next" \
          --max-requests "$max_requests" --ack-mode "$ack_mode" --delay-ms "$delay_ms" $drop_flag
      else
        cargo run --quiet --offline --manifest-path rust/Cargo.toml --bin interop -- \
          --mode server --node "$node" --listen "$listen" \
          --max-requests "$max_requests" --ack-mode "$ack_mode" --delay-ms "$delay_ms" $drop_flag
      fi
      ;;
    go)
      local drop_flag=""
      if [[ "$drop_first" == "1" ]]; then drop_flag="--drop-first"; fi
      if [[ -n "$next" ]]; then
        (cd golang/src && env -u GOROOT go run ./cmd/interop \
          --mode server --node "$node" --listen "$listen" --next "$next" \
          --max-requests "$max_requests" --ack-mode "$ack_mode" --delay-ms "$delay_ms" $drop_flag)
      else
        (cd golang/src && env -u GOROOT go run ./cmd/interop \
          --mode server --node "$node" --listen "$listen" \
          --max-requests "$max_requests" --ack-mode "$ack_mode" --delay-ms "$delay_ms" $drop_flag)
      fi
      ;;
    ts)
      local drop_flag=""
      if [[ "$drop_first" == "1" ]]; then drop_flag="--drop-first"; fi
      if [[ -n "$next" ]]; then
        (cd typescript && npx tsx src/cmd/interop.ts \
          --mode server --node "$node" --listen "$listen" --next "$next" \
          --max-requests "$max_requests" --ack-mode "$ack_mode" --delay-ms "$delay_ms" $drop_flag)
      else
        (cd typescript && npx tsx src/cmd/interop.ts \
          --mode server --node "$node" --listen "$listen" \
          --max-requests "$max_requests" --ack-mode "$ack_mode" --delay-ms "$delay_ms" $drop_flag)
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
  local count="${6:-1}"
  local expect_failure="${7:-0}"
  local retries="${8:-0}"
  local retry_delay_ms="${9:-100}"
  local timeout_ms="${10:-2000}"
  case "$lang" in
    rust)
      if [[ "$expect_failure" == "1" ]]; then
        cargo run --quiet --offline --manifest-path rust/Cargo.toml --bin interop -- \
          --mode client --node "$node" --addr "$addr" --expect-hops "$hops" --expect-ack-from "$expect_ack_from" --count "$count" --retries "$retries" --retry-delay-ms "$retry_delay_ms" --timeout-ms "$timeout_ms" --expect-failure
      else
        cargo run --quiet --offline --manifest-path rust/Cargo.toml --bin interop -- \
          --mode client --node "$node" --addr "$addr" --expect-hops "$hops" --expect-ack-from "$expect_ack_from" --count "$count" --retries "$retries" --retry-delay-ms "$retry_delay_ms" --timeout-ms "$timeout_ms"
      fi
      ;;
    go)
      if [[ "$expect_failure" == "1" ]]; then
        (cd golang/src && env -u GOROOT go run ./cmd/interop \
          --mode client --node "$node" --addr "$addr" --expect-hops "$hops" --expect-ack-from "$expect_ack_from" --count "$count" --retries "$retries" --retry-delay-ms "$retry_delay_ms" --timeout-ms "$timeout_ms" --expect-failure)
      else
        (cd golang/src && env -u GOROOT go run ./cmd/interop \
          --mode client --node "$node" --addr "$addr" --expect-hops "$hops" --expect-ack-from "$expect_ack_from" --count "$count" --retries "$retries" --retry-delay-ms "$retry_delay_ms" --timeout-ms "$timeout_ms")
      fi
      ;;
    ts)
      if [[ "$expect_failure" == "1" ]]; then
        (cd typescript && npx tsx src/cmd/interop.ts \
          --mode client --node "$node" --addr "$addr" --expect-hops "$hops" --expect-ack-from "$expect_ack_from" --count "$count" --retries "$retries" --retry-delay-ms "$retry_delay_ms" --timeout-ms "$timeout_ms" --expect-failure)
      else
        (cd typescript && npx tsx src/cmd/interop.ts \
          --mode client --node "$node" --addr "$addr" --expect-hops "$hops" --expect-ack-from "$expect_ack_from" --count "$count" --retries "$retries" --retry-delay-ms "$retry_delay_ms" --timeout-ms "$timeout_ms")
      fi
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
MID_DROP_FIRST=0
if [[ "$RELAY_FAULT_MODE" == "drop_first_forward" ]]; then
  MID_DROP_FIRST=1
fi
TERM_DROP_FIRST="$DROP_FIRST"
TERM_EXTRA_REQUESTS=0
if [[ "$TERM_DROP_FIRST" == "1" ]]; then
  TERM_EXTRA_REQUESTS=1
fi
MID_EXTRA_REQUESTS=0
if [[ "$TERM_DROP_FIRST" == "1" ]]; then
  MID_EXTRA_REQUESTS=$((MID_EXTRA_REQUESTS + 1))
fi
if [[ "$MID_DROP_FIRST" == "1" ]]; then
  MID_EXTRA_REQUESTS=$((MID_EXTRA_REQUESTS + 1))
fi
MID_EXPECTED_REQUESTS=$((COUNT + MID_EXTRA_REQUESTS))
TERM_EXPECTED_REQUESTS=$((COUNT + TERM_EXTRA_REQUESTS))

run_server "$TERMINAL_LANG" "$TERMINAL_LANG" "$TERM_ADDR" "" "$TERM_EXPECTED_REQUESTS" "$ACK_MODE" "$TERM_DROP_FIRST" "0" >"${LOG_PREFIX}.terminal.log" 2>&1 &
TERM_PID=$!
wait_for_server_ready "$TERM_PID" "${LOG_PREFIX}.terminal.log"

run_server "$MID_LANG" "$MID_LANG" "$MID_ADDR" "$TERM_ADDR" "$MID_EXPECTED_REQUESTS" "normal" "$MID_DROP_FIRST" "$RELAY_DELAY_MS" >"${LOG_PREFIX}.mid.log" 2>&1 &
MID_PID=$!
wait_for_server_ready "$MID_PID" "${LOG_PREFIX}.mid.log"

run_client "$CLIENT_LANG" "$CLIENT_LANG" "$MID_ADDR" "$EXPECT_HOPS" "$TERMINAL_LANG" "$COUNT" "$EXPECT_FAILURE" "$RETRIES" "$RETRY_DELAY_MS" "$TIMEOUT_MS" >"${LOG_PREFIX}.client.log" 2>&1

wait "$MID_PID"
wait "$TERM_PID"

echo "PASS ${CLIENT_LANG}->${MID_LANG}->${TERMINAL_LANG} interop count=${COUNT} ack_mode=${ACK_MODE} drop_first=${DROP_FIRST} relay_fault_mode=${RELAY_FAULT_MODE} relay_delay_ms=${RELAY_DELAY_MS}"
echo "{\"topology\":\"${CLIENT_LANG},${MID_LANG},${TERMINAL_LANG}\",\"count\":${COUNT},\"ack_mode\":\"${ACK_MODE}\",\"drop_first\":${DROP_FIRST},\"relay_fault_mode\":\"${RELAY_FAULT_MODE}\",\"relay_delay_ms\":${RELAY_DELAY_MS},\"expect_failure\":${EXPECT_FAILURE},\"retries\":${RETRIES},\"status\":\"pass\"}" >"${LOG_PREFIX}.summary.json"
