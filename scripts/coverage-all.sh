#!/usr/bin/env bash
# coverage-all.sh — merged coverage: unit tests + integration tests in one report.
#
# Strategy:
#   1. Build everything with -Cinstrument-coverage into target/cov-build/
#   2. Run `cargo test --lib` → unit test binary writes profraw for library code
#   3. Run integration Vitest suites → server binary writes profraw for HTTP handlers
#   4. Merge all profraw → .coverage/merged-all.profdata
#   5. Pass BOTH binaries (unit-test exe + server exe) to llvm-cov via --object
#      so all function hashes resolve and coverage is attributed to the right source.
#
# Usage:
#   npm run test:coverage-all
#   npm run test:coverage-all-html
#   bash scripts/coverage-all.sh [--html]
#
# Output:
#   .coverage/coverage-all.json
#   .coverage/coverage-all-html/  (with --html)
set -eo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

HTML="${1:-}"
PROFILE_DIR="$ROOT/.coverage"
COV_BUILD="$ROOT/target/cov-build"
PORT="${DESCOPE_EMULATOR_TEST_PORT:-4502}"

mkdir -p "$PROFILE_DIR"

# ── 0. Clean up stale processes and old profraw ────────────────────────────────
echo "Cleaning up stale processes and old profraw data..."
lsof -ti:"$PORT" 2>/dev/null | xargs kill -9 2>/dev/null || true
pkill -f "descope-emulator" 2>/dev/null || true
rm -f "$PROFILE_DIR"/merged-all-*.profraw
sleep 0.3

# ── 1. Build everything instrumented ──────────────────────────────────────────
echo "Building with -Cinstrument-coverage..."
unset RUSTC_WRAPPER CARGO_LLVM_COV __CARGO_LLVM_COV_RUSTC_WRAPPER_RUSTFLAGS 2>/dev/null || true
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="$PROFILE_DIR/merged-all-%p.profraw"

cargo build --target-dir "$COV_BUILD" 2>&1 | grep -E "Compiling descope|Finished|^error" || true

BIN="$COV_BUILD/debug/descope-emulator"
SYMBOL_COUNT=$(nm "$BIN" 2>/dev/null | grep -c '__llvm_profile' 2>/dev/null || true)
SYMBOL_COUNT="${SYMBOL_COUNT//[^0-9]/}"
echo "  Server binary: $BIN (${SYMBOL_COUNT:-0} llvm_profile symbols)"

# ── 2. Run unit tests ──────────────────────────────────────────────────────────
# The test binary is a separate ELF — we need its path later for llvm-cov --object
echo "Running unit tests..."
cargo test --lib --target-dir "$COV_BUILD" -- --test-threads=1 \
  2>&1 | grep -E "^test |^running|^test result" || true

# Find the freshest unit-test binary (cargo names it descope_emulator-<hash>)
TEST_BIN=$(find "$COV_BUILD/debug/deps" -name "descope_emulator-*" \
  -not -name "*.d" -not -name "*.rlib" -perm +0111 \
  2>/dev/null | sort -t- -k2 | tail -1)
echo "  Test binary:   ${TEST_BIN:-not found}"

# ── 3. Run integration tests against the instrumented server binary ────────────
export DESCOPE_EMULATOR_BIN="$BIN"
export DESCOPE_EMULATOR_TEST_PORT="$PORT"
export EMULATOR_PROJECT_ID="${EMULATOR_PROJECT_ID:-emulator-project}"
export EMULATOR_MANAGEMENT_KEY="${EMULATOR_MANAGEMENT_KEY:-emulator-key}"

EXIT_CODE=0
echo "Running API integration tests..."
(cd "$ROOT/integration/api" && npm install --silent && npx vitest run) || EXIT_CODE=$?

echo "Running sdk-js integration tests..."
(cd "$ROOT/integration/sdk-js" && npm install --silent && npx vitest run) || EXIT_CODE=$?

echo "Running sdk-nodejs integration tests..."
(cd "$ROOT/integration/sdk-nodejs" && npm install --silent && npx vitest run) || EXIT_CODE=$?

# Wait for server SIGTERM → process::exit(0) → LLVM flush
sleep 1

# ── 4. Merge all profraw ───────────────────────────────────────────────────────
SYSROOT=$(rustc --print sysroot)
ARCH=$(rustc -vV | grep host | awk '{print $2}')
LLVM_PROFDATA="$SYSROOT/lib/rustlib/$ARCH/bin/llvm-profdata"
LLVM_COV="$SYSROOT/lib/rustlib/$ARCH/bin/llvm-cov"

PROFRAW_COUNT=$(find "$PROFILE_DIR" -name "merged-all-*.profraw" 2>/dev/null | wc -l | tr -d ' ')
echo "Found $PROFRAW_COUNT .profraw file(s) — merging..."

if [ "$PROFRAW_COUNT" -eq 0 ]; then
  echo "ERROR: no profraw files found" >&2
  exit 1
fi

find "$PROFILE_DIR" -name "merged-all-*.profraw" | \
  xargs "$LLVM_PROFDATA" merge -sparse -o "$PROFILE_DIR/merged-all.profdata"

# ── 5. Build llvm-cov --object list (server binary + unit-test binary) ─────────
OBJECTS="$BIN"
if [ -n "$TEST_BIN" ] && [ -f "$TEST_BIN" ]; then
  OBJECTS="$OBJECTS -object $TEST_BIN"
  echo "  Using both binaries for full symbol resolution"
else
  echo "  Warning: unit-test binary not found; report may miss unit-only coverage"
fi

# ── 6. Generate report ────────────────────────────────────────────────────────
IGNORE_RE='\.cargo/registry|\.rustup|/rustc/'

echo "Generating merged JSON report..."
# shellcheck disable=SC2086
"$LLVM_COV" export $OBJECTS \
  --instr-profile="$PROFILE_DIR/merged-all.profdata" \
  --ignore-filename-regex="$IGNORE_RE" \
  > "$PROFILE_DIR/coverage-all.json"

echo ""
echo "Merged coverage summary (unit tests + integration tests):"
# shellcheck disable=SC2086
"$LLVM_COV" report $OBJECTS \
  --instr-profile="$PROFILE_DIR/merged-all.profdata" \
  --ignore-filename-regex="$IGNORE_RE"

if [ "$HTML" = "--html" ]; then
  # shellcheck disable=SC2086
  "$LLVM_COV" show $OBJECTS \
    --instr-profile="$PROFILE_DIR/merged-all.profdata" \
    --format=html \
    --output-dir="$PROFILE_DIR/coverage-all-html" \
    --ignore-filename-regex="$IGNORE_RE"
  echo "HTML report: $PROFILE_DIR/coverage-all-html/index.html"
  open "$PROFILE_DIR/coverage-all-html/index.html" 2>/dev/null || true
fi

JSON_BYTES=$(wc -c < "$PROFILE_DIR/coverage-all.json" | tr -d ' ')
echo "JSON report: $PROFILE_DIR/coverage-all.json ($JSON_BYTES bytes)"
exit "$EXIT_CODE"
