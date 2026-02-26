#!/bin/bash

set -euo pipefail

go_cmd() {
  local subcommand="${1:-}"
  shift || true

  local go_main="golang/src/cmd/main.go"
  local go_out_dir="golang/build"
  local go_bin_name="courier"
  local go_out="$go_out_dir/$go_bin_name"

  case "$subcommand" in
  build)
    local mode="${1:-debug}"
    shift || true

    mkdir -p "$go_out_dir"

    case "$mode" in
    debug)
      echo "→ Building Go (debug)..."
      env -u GOROOT go build -o "$go_out" "$go_main"
      ;;
    release)
      echo "→ Building Go (release)..."
      env -u GOROOT go build -ldflags="-s -w" -o "$go_out" "$go_main"
      ;;
    *)
      echo "✘ Unknown build mode: '$mode' (expected 'debug' or 'release')"
      return 1
      ;;
    esac

    echo "✔ Go binary: $go_out"
    ;;

  run)
    local mode="${1:-debug}"
    shift || true

    go_cmd build "$mode"
    echo "→ Running Go ($mode): $go_out"
    "$go_out" "$@"
    ;;

  test)
    echo "→ Running Go tests..."
    (cd golang/src && env -u GOROOT go test ./... ../tests "$@")
    ;;

  coverage)
    echo "→ Generating Go coverage report..."

    local go_src="golang/src"
    local go_build="golang/build"
    local cover_file="$go_build/coverage.out"
    local lcov_file="$go_build/lcov.info"

    mkdir -p "$go_build"

    # Run tests with coverage
    (cd "$go_src" && env -u GOROOT go test ./... -coverprofile=../../"$cover_file" -covermode=atomic)

    # Convert to LCOV for VSCode or unified tools
    if command -v gocov >/dev/null && command -v gocov-xml >/dev/null; then
      echo "→ Converting to lcov..."
      gocov convert "$cover_file" >"$lcov_file"
    fi

    echo "✔ Go coverage written to: $cover_file"

    gcov2lcov -infile $cover_file -outfile golang/build/lcov.info

    echo "✔ Go coverage converted for vscode from : $cover_file to golang/build/lconv.info"
    ;;

  *)
    echo "✘ Unknown subcommand for 'go': '$subcommand'"
    ;;
  esac
}

rust_cmd() {
  local subcommand="${1:-}"
  shift || true

  local rust_manifest="rust/Cargo.toml"
  local rust_bin_name="courier"
  local rust_out_dir="rust/build"

  case "$subcommand" in
  build)
    local mode="${1:-release}"
    shift || true

    case "$mode" in
    debug)
      local profile_dir="debug"
      local release_flag=""
      ;;
    release)
      local profile_dir="release"
      local release_flag="--release"
      ;;
    *)
      echo "✘ Unknown build mode: '$mode' (expected 'debug' or 'release')"
      return 1
      ;;
    esac

    echo "→ Building Rust ($mode)..."
    mkdir -p "$rust_out_dir"
    CARGO_TARGET_DIR="$rust_out_dir" cargo build --manifest-path="$rust_manifest" --bin "$rust_bin_name" $release_flag
    echo "✔ Rust binary: $rust_out_dir/$profile_dir/$rust_bin_name"
    ;;

  run)
    local mode="${1:-release}"
    shift || true

    # Build first
    rust_cmd build "$mode"

    local binary_path="$rust_out_dir/$mode/$rust_bin_name"
    echo "→ Running Rust ($mode): $binary_path"
    "$binary_path" "$@"
    ;;

  test)
    echo "→ Running Rust tests..."
    CARGO_TARGET_DIR="$rust_out_dir" cargo test --manifest-path="$rust_manifest" "$@"
    ;;

  coverage)
    echo "→ Running Rust coverage with tarpaulin..."
    (CARGO_TARGET_DIR=rust/coverage-target cargo tarpaulin \
      --manifest-path=rust/Cargo.toml \
      --out Lcov \
      --output-dir rust \
      --skip-clean)
    ;;

  *)
    echo "✘ Unknown subcommand for 'rust': '$subcommand'"
    ;;
  esac
}

ts_cmd() {
  local subcommand="${1:-}"
  shift || true

  local ts_entry="typescript/src/cmd/main.ts"
  local ts_out_dir="typescript/build"
  local ts_out="$ts_out_dir/courier.js"
  local ts_config_debug="typescript/tsconfig.debug.json"
  local ts_config_release="typescript/tsconfig.release.json"

  case "$subcommand" in
  build)
    local mode="${1:-debug}"
    shift || true

    mkdir -p "$ts_out_dir"

    case "$mode" in
    debug)
      echo "→ Building TypeScript (debug)..."
      npx tsc --project "$ts_config_debug"
      cp "$ts_out_dir/cmd/main.js" "$ts_out"
      ;;
    release)
      echo "→ Building TypeScript (release)..."
      npx tsc --project "$ts_config_release"
      cp "$ts_out_dir/cmd/main.js" "$ts_out"
      ;;
    *)
      echo "✘ Unknown build mode: '$mode' (expected 'debug' or 'release')"
      return 1
      ;;
    esac

    echo "✔ TypeScript output: $ts_out"
    ;;

  run)
    local mode="${1:-debug}"
    shift || true

    ts_cmd build "$mode"
    echo "→ Running TypeScript ($mode): node $ts_out"
    node "$ts_out" "$@"
    ;;

  test)
    echo "→ Running TypeScript tests..."
    (cd typescript && npx jest "$@")
    ;;

  coverage)
    echo "→ Generating TypeScript coverage..."
    (cd typescript && npx jest --coverage "$@")
    echo "✔ Coverage written to typescript/build/coverage/"
    ;;

  *)
    echo "✘ Unknown subcommand for 'ts': '$subcommand'"
    ;;
  esac
}

all_cmd() {
  local subcommand="${1:-}"
  shift || true
  case "$subcommand" in
  build)
    go_cmd build "$@"
    rust_cmd build "$@"
    ts_cmd build "$@"
    ;;

  test)
    echo "→ Running all tests..."
    local failed=0

    echo ""
    echo "🧪 Rust tests:"
    if ! rust_cmd test "$@"; then
      echo "❌ Rust tests failed"
      failed=1
    fi

    echo ""
    echo "🧪 Go tests:"
    if ! go_cmd test "$@"; then
      echo "❌ Go tests failed"
      failed=1
    fi

    echo ""
    echo "🧪 TypeScript tests:"
    if ! ts_cmd test "$@"; then
      echo "❌ TypeScript tests failed"
      failed=1
    fi

    echo ""
    echo "🧪 Cross-language interop e2e:"
    if ! interop_cmd test "$@"; then
      echo "❌ Interop e2e test failed"
      failed=1
    fi

    if [[ "$failed" -ne 0 ]]; then
      echo ""
      echo "❌ One or more test suites failed."
      exit 1
    else
      echo ""
      echo "✅ All tests passed."
    fi
    ;;

  coverage)
    echo "→ Running coverage for all languages..."
    local failed=0

    echo ""
    echo "📊 Rust coverage:"
    if ! rust_cmd coverage "$@"; then
      echo "❌ Rust coverage failed"
      failed=1
    fi

    echo ""
    echo "📊 Go coverage:"
    if ! go_cmd coverage "$@"; then
      echo "❌ Go coverage failed"
      failed=1
    fi

    echo ""
    echo "📊 TypeScript coverage:"
    if ! ts_cmd coverage "$@"; then
      echo "❌ TypeScript coverage failed"
      failed=1
    fi

    if [[ "$failed" -ne 0 ]]; then
      echo ""
      echo "❌ One or more coverage reports failed."
      exit 1
    else
      echo ""
      echo "✅ All coverage reports completed successfully."
    fi
    ;;

  coverage-dashboard)
    echo "→ Generating unified coverage dashboard..."

    # Step 1: Run per-language coverage
    all_cmd coverage

    # Step 2: Convert Go's coverage.out to lcov
    if ! command -v gcov2lcov >/dev/null; then
      echo "⚠️  Missing gcov2lcov. Install with:"
      echo "   go install github.com/jandelgado/gcov2lcov@latest"
      exit 1
    fi

    gcov2lcov \
      -infile golang/build/coverage.out \
      -outfile build/coverage/go_raw.lcov

    # Step 3: Rewrite LCOV paths for VSCode + genhtml
    mkdir -p build/coverage

    # TypeScript: prefix with typescript/
    sed 's|^SF:src/|SF:typescript/src/|' typescript/build/coverage/lcov.info >build/coverage/ts_fixed.lcov

    # Go: prefix with golang/src/
    sed 's|^SF:cmd/|SF:golang/src/cmd/|' build/coverage/go_raw.lcov >build/coverage/go_fixed.lcov

    # Rust: assumed already correct; copy as-is
    cp rust/lcov.info build/coverage/rust_fixed.lcov

    # Step 4: Merge
    cat \
      build/coverage/rust_fixed.lcov \
      build/coverage/go_fixed.lcov \
      build/coverage/ts_fixed.lcov \
      >build/coverage/merged.info

    # Step 5: Generate HTML report
    if ! command -v genhtml >/dev/null; then
      echo "⚠️  Missing genhtml. Install with: brew install lcov (mac) or apt install lcov (Linux)"
      exit 1
    fi

    # Generate HTML report
    if command -v genhtml >/dev/null; then
      genhtml build/coverage/merged.info --output-directory build/coverage/html

      local report_path="build/coverage/html/index.html"
      local abs_path="file://$(pwd)/$report_path"

      echo ""
      echo "📊 Coverage dashboard available at:"
      echo "👉 $abs_path"

      # Auto-open browser on supported platforms
      if [[ "$OSTYPE" == "darwin"* ]]; then
        open "$report_path"
      elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v xdg-open >/dev/null; then
          xdg-open "$report_path"
        fi
      fi

    else
      echo "⚠️  genhtml not found. Install via: brew install lcov (mac) or apt install lcov (Linux)"
      exit 1
    fi

    # Generate Coverage Summary
    echo ""
    echo "📊 Coverage Summary:"
    printf "┌ %-13s ┬ %-10s ┐\n" "Language" "Coverage"
    printf "├───────────────┼────────────┤\n"

    # Rust
    if [[ -f rust/lcov.info ]]; then
      rust_covered=$(grep -c "^DA:[0-9]*,[1-9][0-9]*" rust/lcov.info)
      rust_total=$(grep -c "^DA:" rust/lcov.info)
      rust_pct=$(awk "BEGIN { if ($rust_total == 0) print \"0.0\"; else printf \"%.1f\", ($rust_covered / $rust_total) * 100 }")
      printf "│ %-13s │ %5s%%     │\n" "Rust" "$rust_pct"
    fi

    # Go
    if [[ -f golang/build/coverage.out ]]; then
      go_pct=$(env -u GOROOT go tool cover -func=golang/build/coverage.out | grep total: | awk '{print $3}' | tr -d '%')
      printf "│ %-13s │ %5s%%     │\n" "Go" "$go_pct"
    fi

    # TypeScript
    if [[ -f typescript/build/coverage/lcov.info ]]; then
      ts_covered=$(grep -c "^DA:[0-9]*,[1-9][0-9]*" typescript/build/coverage/lcov.info)
      ts_total=$(grep -c "^DA:" typescript/build/coverage/lcov.info)
      ts_pct=$(awk "BEGIN { if ($ts_total == 0) print \"0.0\"; else printf \"%.1f\", ($ts_covered / $ts_total) * 100 }")
      printf "│ %-13s │ %5s%%     │\n" "TypeScript" "$ts_pct"
    fi

    printf "└───────────────┴────────────┘\n"

    ;;

  *)
    echo "✘ Unknown subcommand for 'all': '$subcommand'"
    ;;
  esac
}

interop_cmd() {
  local subcommand="${1:-}"
  shift || true

  case "$subcommand" in
  test)
    echo "→ Running go->ts->rust interop e2e..."
    ./conformance/e2e/test_go_ts_rust.sh "$@"
    ;;
  *)
    echo "✘ Unknown subcommand for 'interop': '$subcommand'"
    ;;
  esac
}

main() {
  local target="${1:-}"
  shift || true

  case "$target" in
  go) go_cmd "$@" ;;
  rust) rust_cmd "$@" ;;
  ts | typescript) ts_cmd "$@" ;;
  interop) interop_cmd "$@" ;;
  all) all_cmd "$@" ;;
  *)
    echo "Usage: $0 {go|rust|ts|interop|all} <subcommand>"
    echo "Example: $0 rust build"
    exit 1
    ;;
  esac
}

main "$@"
