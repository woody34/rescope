#!/usr/bin/env bash
#
# scripts/release-build.sh — Build Rescope release binaries for multiple platforms.
#
# Usage:
#   ./scripts/release-build.sh              # Build all available targets
#   ./scripts/release-build.sh --target linux-amd64   # Build one target
#
# Requirements:
#   - Rust toolchain (rustup)
#   - Node.js + npm (for UI build)
#   - `cross` (for Linux cross-compilation): cargo install cross
#   - Docker (required by `cross` for Linux targets)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"

# ── Target definitions ─────────────────────────────────────────────────
# Format: "label rust-triple tool binary-suffix"
# tool: "cargo" for native builds, "cross" for cross-compilation
TARGETS=(
  "linux-amd64    x86_64-unknown-linux-gnu      cross  "
  "linux-arm64    aarch64-unknown-linux-gnu      cross  "
  "macos-amd64    x86_64-apple-darwin            cargo  "
  "macos-arm64    aarch64-apple-darwin            cargo  "
)

# ── Colors ──────────────────────────────────────────────────────────────
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

info()  { echo -e "${GREEN}▶${NC} $*"; }
warn()  { echo -e "${YELLOW}⚠${NC} $*"; }
error() { echo -e "${RED}✗${NC} $*"; }

# ── Parse args ──────────────────────────────────────────────────────────
FILTER_TARGET=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      FILTER_TARGET="$2"
      shift 2
      ;;
    *)
      echo "Usage: $0 [--target <label>]"
      exit 1
      ;;
  esac
done

# ── Step 1: Build UI ───────────────────────────────────────────────────
info "Building UI..."
cd "$ROOT_DIR/apps/ui"
npm run build
cd "$ROOT_DIR"
info "UI build complete"

# ── Step 2: Prepare output directory ───────────────────────────────────
mkdir -p "$DIST_DIR"

# ── Step 3: Build each target ──────────────────────────────────────────
BUILT=0
SKIPPED=0

for entry in "${TARGETS[@]}"; do
  read -r label triple tool suffix <<< "$entry"

  # Filter if --target was specified
  if [[ -n "$FILTER_TARGET" && "$label" != "$FILTER_TARGET" ]]; then
    continue
  fi

  output_name="rescope-${label}${suffix}"

  info "Building $output_name ($triple)..."

  # Check if the build tool is available
  if [[ "$tool" == "cross" ]]; then
    if ! command -v cross &>/dev/null; then
      warn "Skipping $label: 'cross' not installed (cargo install cross)"
      SKIPPED=$((SKIPPED + 1))
      continue
    fi
    if ! docker info &>/dev/null 2>&1; then
      warn "Skipping $label: Docker not running (required by cross)"
      SKIPPED=$((SKIPPED + 1))
      continue
    fi
  fi

  # Ensure the Rust target is installed (for native cargo builds)
  if [[ "$tool" == "cargo" ]]; then
    if ! rustup target list --installed | grep -q "$triple"; then
      info "  Installing Rust target $triple..."
      if ! rustup target add "$triple" 2>/dev/null; then
        warn "Skipping $label: could not install target $triple"
        SKIPPED=$((SKIPPED + 1))
        continue
      fi
    fi
  fi

  # Build
  if $tool build --release --target "$triple"; then
    # Copy binary to dist/
    src="$ROOT_DIR/target/$triple/release/rescope${suffix}"
    if [[ -f "$src" ]]; then
      cp "$src" "$DIST_DIR/$output_name"
      info "  ✓ $output_name ($(du -h "$DIST_DIR/$output_name" | cut -f1 | xargs))"
      BUILT=$((BUILT + 1))
    else
      error "  Binary not found at $src"
      SKIPPED=$((SKIPPED + 1))
    fi
  else
    warn "Skipping $label: build failed"
    SKIPPED=$((SKIPPED + 1))
  fi
done

# ── Step 4: Generate checksums ─────────────────────────────────────────
if [[ $BUILT -gt 0 ]]; then
  info "Generating checksums..."
  cd "$DIST_DIR"
  if command -v sha256sum &>/dev/null; then
    sha256sum rescope-* > checksums-sha256.txt
  elif command -v shasum &>/dev/null; then
    shasum -a 256 rescope-* > checksums-sha256.txt
  fi
  cd "$ROOT_DIR"
fi

# ── Summary ────────────────────────────────────────────────────────────
echo ""
info "Release build complete"
echo "  Built:   $BUILT"
echo "  Skipped: $SKIPPED"
echo "  Output:  $DIST_DIR/"
if [[ $BUILT -gt 0 ]]; then
  echo ""
  ls -lh "$DIST_DIR/"
fi
