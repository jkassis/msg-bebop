#!/bin/bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
"$ROOT/conformance/e2e/run_topology.sh" "go,ts,rust" 5 "normal" 0
