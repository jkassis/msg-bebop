#!/bin/bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

"$ROOT/conformance/e2e/run_topology.sh" "go,ts,rust"
"$ROOT/conformance/e2e/run_topology.sh" "rust,go,ts"
"$ROOT/conformance/e2e/run_topology.sh" "ts,rust,go"

echo "PASS all interop topologies"
