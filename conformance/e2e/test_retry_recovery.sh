#!/bin/bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# Drop the first terminal response and verify client retries recover.
"$ROOT/conformance/e2e/run_topology.sh" "go,ts,rust" 3 "normal" 0 1 2 100 2000
"$ROOT/conformance/e2e/run_topology.sh" "rust,go,ts" 3 "normal" 0 1 2 100 2000
"$ROOT/conformance/e2e/run_topology.sh" "ts,rust,go" 3 "normal" 0 1 2 100 2000

echo "PASS retry recovery interop cases"
