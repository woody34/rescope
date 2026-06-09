#!/usr/bin/env bash
# coverage-integration.sh — Rust coverage from live HTTP integration tests.
#
# Strategy:
#   1. Build the emulator with -Cinstrument-coverage into a separate target dir
#      (avoids cargo-llvm-cov's EAGAIN issues with too many parallel processes).
#   2. Set DESCOPE_EMULATOR_BIN so Vitest global-setup uses the instrumented binary.
#      global-setup polls health, runs tests, then calls teardown(SIGTERM).
#      main.rs handles SIGTERM → std::process::exit(0) → LLVM flushes .profraw.
#   3. Merge profraw files and emit JSON + optional HTML report via LLVM tools.
#
# Requirements: llvm-tools-preview rustup component (already installed with cargo-llvm-cov).
#
# Usage:
#   npm run test:coverage-integration
#   bash scripts/coverage-integration.sh [--html]
#
# Output:
#   .coverage/integration-coverage.json
#   .coverage/integration-html/  (with --html)
set -eo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

HTML="${1:-}"
PROFILE_DIR="$ROOT/.coverage"
COV_BUILD="$ROOT/target/cov-build"
mkdir -p "$PROFILE_DIR"

# ── 0. Kill any stale emulator processes to avoid port conflicts ───────────────
PORT="${DESCOPE_EMULATOR_TEST_PORT:-4502}"
echo "Killing any stale emulators on port $PORT..."
lsof -ti:"$PORT" 2>/dev/null | xargs kill -9 2>/dev/null || true
pkill -f "descope-emulator" 2>/dev/null || true
sleep 0.5   # let the OS reclaim the port

# ── 1. Build instrumented binary ───────────────────────────────────────────────
echo "Building instrumented binary..."
unset RUSTC_WRAPPER CARGO_LLVM_COV __CARGO_LLVM_COV_RUSTC_WRAPPER_RUSTFLAGS 2>/dev/null || true
RUSTFLAGS="-Cinstrument-coverage" \
  cargo build --target-dir "$COV_BUILD" 2>&1 | grep -E "Compiling descope|Finished|^error" || true

BIN="$COV_BUILD/debug/descope-emulator"
SYMBOL_COUNT=$(nm "$BIN" 2>/dev/null | grep -c '__llvm_profile' 2>/dev/null || true)
SYMBOL_COUNT="${SYMBOL_COUNT//[^0-9]/}"   # trim whitespace/newlines
echo "  Binary: $BIN (${SYMBOL_COUNT:-0} llvm_profile symbols)"

if [ "${SYMBOL_COUNT:-0}" -eq 0 ]; then
  echo "ERROR: binary is not instrumented — check RUSTFLAGS" >&2
  exit 1
fi

# ── 2. Configure profraw output path and tell Vitest which binary to run ───────
rm -f "$PROFILE_DIR"/integration-*.profraw   # clear old data
export LLVM_PROFILE_FILE="$PROFILE_DIR/integration-%p.profraw"
export DESCOPE_EMULATOR_BIN="$BIN"
export DESCOPE_EMULATOR_TEST_PORT="${DESCOPE_EMULATOR_TEST_PORT:-4502}"
export EMULATOR_PROJECT_ID="${EMULATOR_PROJECT_ID:-emulator-project}"
export EMULATOR_MANAGEMENT_KEY="${EMULATOR_MANAGEMENT_KEY:-emulator-key}"

echo "  Profraw → $LLVM_PROFILE_FILE"
echo "  Port    → $DESCOPE_EMULATOR_TEST_PORT"

# ── 3. Run integration test suites ────────────────────────────────────────────
EXIT_CODE=0
echo "Running API integration tests..."
(cd "$ROOT/integration/api" && npm install --silent && npx vitest run) || EXIT_CODE=$?

echo "Running sdk-js integration tests..."
(cd "$ROOT/integration/sdk-js" && npm install --silent && npx vitest run) || EXIT_CODE=$?

echo "Running sdk-nodejs integration tests..."
(cd "$ROOT/integration/sdk-nodejs" && npm install --silent && npx vitest run) || EXIT_CODE=$?

# Give the binary a moment to finish flushing profraw after teardown SIGTERM
sleep 1

# ── 4. Merge profraw and generate report ──────────────────────────────────────
SYSROOT=$(rustc --print sysroot)
ARCH=$(rustc -vV | grep host | awk '{print $2}')
LLVM_PROFDATA="$SYSROOT/lib/rustlib/$ARCH/bin/llvm-profdata"
LLVM_COV="$SYSROOT/lib/rustlib/$ARCH/bin/llvm-cov"

PROFRAW_COUNT=$(find "$PROFILE_DIR" -name "integration-*.profraw" 2>/dev/null | wc -l | tr -d ' ')
echo "Found $PROFRAW_COUNT .profraw file(s) in .coverage/"

if [ "$PROFRAW_COUNT" -eq 0 ]; then
  echo "ERROR: no profraw files found — did the binary exit cleanly?" >&2
  exit 1
fi

echo "Merging profraw files..."
find "$PROFILE_DIR" -name "integration-*.profraw" | \
  xargs "$LLVM_PROFDATA" merge -sparse -o "$PROFILE_DIR/merged.profdata"

echo "Generating JSON report..."
"$LLVM_COV" export "$BIN" \
  --instr-profile="$PROFILE_DIR/merged.profdata" \
  --ignore-filename-regex='\.cargo/registry|\.rustup|/rustc/' \
  > "$PROFILE_DIR/integration-coverage.json"

echo "Coverage summary:"
"$LLVM_COV" report "$BIN" \
  --instr-profile="$PROFILE_DIR/merged.profdata" \
  --ignore-filename-regex='\.cargo/registry|\.rustup|/rustc/'

if [ "$HTML" = "--html" ]; then
  "$LLVM_COV" show "$BIN" \
    --instr-profile="$PROFILE_DIR/merged.profdata" \
    --format=html \
    --output-dir="$PROFILE_DIR/integration-html" \
    --ignore-filename-regex='\.cargo/registry|\.rustup|/rustc/'
  echo "HTML report: $PROFILE_DIR/integration-html/index.html"
  open "$PROFILE_DIR/integration-html/index.html" 2>/dev/null || true
fi

JSON_BYTES=$(wc -c < "$PROFILE_DIR/integration-coverage.json" | tr -d ' ')
echo "JSON report: $PROFILE_DIR/integration-coverage.json ($JSON_BYTES bytes)"
exit "$EXIT_CODE"
