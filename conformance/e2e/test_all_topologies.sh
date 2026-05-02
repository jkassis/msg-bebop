#!/bin/bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

"$ROOT/conformance/e2e/run_topology.sh" "go,ts,rust" 3 "normal" 0
"$ROOT/conformance/e2e/run_topology.sh" "rust,go,ts" 3 "normal" 0
"$ROOT/conformance/e2e/run_topology.sh" "ts,rust,go" 3 "normal" 0

echo "PASS all interop topologies"
