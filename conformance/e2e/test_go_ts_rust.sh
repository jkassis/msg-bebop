#!/bin/bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_PORT="${RUST_PORT:-47103}"
TS_PORT="${TS_PORT:-47102}"

cleanup() {
  jobs -pr | xargs -r kill 2>/dev/null || true
}
trap cleanup EXIT

cd "$ROOT"

cargo run --quiet --manifest-path rust/Cargo.toml --bin interop -- \
  --mode server --node rust --listen "127.0.0.1:${RUST_PORT}" --once &
RUST_PID=$!

sleep 0.3

(cd typescript && npx tsx src/cmd/interop.ts \
  --mode server \
  --node ts \
  --listen "127.0.0.1:${TS_PORT}" \
  --next "127.0.0.1:${RUST_PORT}" \
  --once) &
TS_PID=$!

sleep 0.3

(cd golang/src && env -u GOROOT go run ./cmd/interop \
  --mode client \
  --node go \
  --addr "127.0.0.1:${TS_PORT}" \
  --expect-hops "go,ts,rust")

wait "$TS_PID"
wait "$RUST_PID"

echo "PASS go->ts->rust interop"
