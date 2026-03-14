#!/usr/bin/env bash
# package_run.sh — package the current PostCAD pilot run into a shareable artifact bundle.
#
# Usage:
#   ./examples/pilot/package_run.sh [SOURCE_DIR] [OUTPUT_DIR]
#
#   SOURCE_DIR  directory containing the run artifacts  (default: examples/pilot)
#   OUTPUT_DIR  destination bundle directory             (default: pilot_bundle)
#
# The script is idempotent: running it twice with the same inputs produces the
# same output directory. The output directory is created if it does not exist.
#
# Exit codes:
#   0  bundle created successfully
#   1  one or more required artifacts are missing

set -euo pipefail

SOURCE_DIR="${1:-examples/pilot}"
OUTPUT_DIR="${2:-pilot_bundle}"

# ── colour helpers ─────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RESET='\033[0m'
info()    { echo -e "${GREEN}[bundle]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[warn]${RESET}  $*"; }
fail()    { echo -e "${RED}[error]${RESET} $*" >&2; }

# ── resolve source directory ───────────────────────────────────────────────────
if [[ ! -d "$SOURCE_DIR" ]]; then
  fail "Source directory not found: $SOURCE_DIR"
  exit 1
fi

echo ""
info "PostCAD pilot run bundle packager"
info "Source  : $SOURCE_DIR"
info "Output  : $OUTPUT_DIR"
echo ""

# ── artifact manifest ──────────────────────────────────────────────────────────
# Maps bundle filename → source filename (relative to SOURCE_DIR).
# All five are required for a complete bundle.
declare -a BUNDLE_NAMES=(
  "route.json"
  "receipt.json"
  "verification.json"
  "export_packet.json"
  "reproducibility.json"
)
declare -A BUNDLE_SOURCES=(
  ["route.json"]="receipt.json"
  ["receipt.json"]="receipt.json"
  ["verification.json"]="verification.json"
  ["export_packet.json"]="export_packet.json"
  ["reproducibility.json"]="reproducibility.json"
)

# ── check required artifacts ───────────────────────────────────────────────────
MISSING=()
for bundle_name in "${BUNDLE_NAMES[@]}"; do
  src="${BUNDLE_SOURCES[$bundle_name]}"
  if [[ ! -f "$SOURCE_DIR/$src" ]]; then
    MISSING+=("$bundle_name (source: $SOURCE_DIR/$src)")
  fi
done

if [[ ${#MISSING[@]} -gt 0 ]]; then
  fail "Required artifacts missing — bundle cannot be created:"
  for m in "${MISSING[@]}"; do
    fail "  • $m"
  done
  echo ""
  fail "Complete the pilot run before packaging:"
  fail "  1. Generate a route       → receipt.json"
  fail "  2. Run verification       → verification.json"
  fail "  3. Export dispatch packet → export_packet.json"
  fail "  4. Run reproducibility    → reproducibility.json"
  echo ""
  exit 1
fi

# ── create output directory ────────────────────────────────────────────────────
mkdir -p "$OUTPUT_DIR"

# ── copy artifacts ─────────────────────────────────────────────────────────────
for bundle_name in "${BUNDLE_NAMES[@]}"; do
  src="$SOURCE_DIR/${BUNDLE_SOURCES[$bundle_name]}"
  dst="$OUTPUT_DIR/$bundle_name"
  cp "$src" "$dst"
  info "  ✓ $bundle_name"
done

# ── write bundle manifest ──────────────────────────────────────────────────────
MANIFEST="$OUTPUT_DIR/bundle_manifest.json"
CREATED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
cat > "$MANIFEST" <<JSON
{
  "bundle_type": "postcad_pilot_run",
  "created_at": "$CREATED_AT",
  "artifacts": [
    "route.json",
    "receipt.json",
    "verification.json",
    "export_packet.json",
    "reproducibility.json"
  ]
}
JSON
info "  ✓ bundle_manifest.json"

echo ""
info "Bundle complete → $OUTPUT_DIR/"
info "Artifacts:"
for f in "${BUNDLE_NAMES[@]}" bundle_manifest.json; do
  SIZE=$(wc -c < "$OUTPUT_DIR/$f" | tr -d ' ')
  info "  $f  (${SIZE} bytes)"
done
echo ""
