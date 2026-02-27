#!/bin/bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

"$ROOT/conformance/e2e/run_topology.sh" "go,ts,rust" 2 "missing_ack_msg_id" 1
"$ROOT/conformance/e2e/run_topology.sh" "rust,go,ts" 2 "bad_ack_version" 1
"$ROOT/conformance/e2e/run_topology.sh" "ts,rust,go" 2 "missing_ack_msg_id" 1

echo "PASS negative interop cases"
