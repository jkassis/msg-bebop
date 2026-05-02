#!/bin/bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# Relay drops first forward; clients recover via retries.
"$ROOT/conformance/e2e/run_topology.sh" "go,ts,rust" 3 "normal" 0 0 2 100 2000 "drop_first_forward" 0
"$ROOT/conformance/e2e/run_topology.sh" "rust,go,ts" 3 "normal" 0 0 2 100 2000 "drop_first_forward" 0
"$ROOT/conformance/e2e/run_topology.sh" "ts,rust,go" 3 "normal" 0 0 2 100 2000 "drop_first_forward" 0

# Relay delay fault; clients tolerate latency with timeout budget.
"$ROOT/conformance/e2e/run_topology.sh" "go,ts,rust" 2 "normal" 0 0 1 50 3000 "normal" 200
"$ROOT/conformance/e2e/run_topology.sh" "rust,go,ts" 2 "normal" 0 0 1 50 3000 "normal" 200
"$ROOT/conformance/e2e/run_topology.sh" "ts,rust,go" 2 "normal" 0 0 1 50 3000 "normal" 200

echo "PASS relay fault interop cases"
