#!/usr/bin/env bash
# validate_bundle.sh — validate a PostCAD pilot run bundle directory.
#
# Usage:
#   ./examples/pilot/validate_bundle.sh [BUNDLE_DIR]
#
#   BUNDLE_DIR  bundle directory to validate  (default: pilot_bundle)
#
# Checks:
#   - all required files are present
#   - all files are non-empty
#   - all files are valid JSON
#   - filenames match expected bundle structure
#
# Exit codes:
#   0  bundle is valid
#   1  one or more validation checks failed

set -euo pipefail

BUNDLE_DIR="${1:-pilot_bundle}"

# ── colour helpers ─────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RESET='\033[0m'
ok()   { echo -e "  ${GREEN}✓${RESET} $*"; }
fail() { echo -e "  ${RED}✗${RESET} $*" >&2; }
info() { echo -e "${GREEN}[validate]${RESET} $*"; }

# ── locate json validator ──────────────────────────────────────────────────────
json_valid() {
  local file="$1"
  if command -v python3 &>/dev/null; then
    python3 -m json.tool "$file" >/dev/null 2>&1
  elif command -v jq &>/dev/null; then
    jq empty "$file" >/dev/null 2>&1
  elif command -v python &>/dev/null; then
    python -m json.tool "$file" >/dev/null 2>&1
  else
    echo -e "${YELLOW}[warn]${RESET}  no JSON validator found (python3/jq) — skipping JSON check for $file" >&2
    return 0
  fi
}

# ── required bundle files ──────────────────────────────────────────────────────
REQUIRED_FILES=(
  "route.json"
  "receipt.json"
  "verification.json"
  "export_packet.json"
  "reproducibility.json"
)

echo ""
info "Validating bundle: $BUNDLE_DIR"
echo ""

ERRORS=0

# ── check directory exists ─────────────────────────────────────────────────────
if [[ ! -d "$BUNDLE_DIR" ]]; then
  fail "Bundle directory not found: $BUNDLE_DIR"
  echo ""
  echo -e "${RED}[validate] FAILED${RESET} — bundle directory does not exist" >&2
  exit 1
fi

# ── check each required file ───────────────────────────────────────────────────
for fname in "${REQUIRED_FILES[@]}"; do
  fpath="$BUNDLE_DIR/$fname"

  # presence
  if [[ ! -f "$fpath" ]]; then
    fail "$fname — missing"
    ERRORS=$((ERRORS + 1))
    continue
  fi

  # non-empty
  if [[ ! -s "$fpath" ]]; then
    fail "$fname — file is empty"
    ERRORS=$((ERRORS + 1))
    continue
  fi

  # valid JSON
  if ! json_valid "$fpath"; then
    fail "$fname — invalid JSON"
    ERRORS=$((ERRORS + 1))
    continue
  fi

  ok "$fname"
done

# ── check for unexpected files ─────────────────────────────────────────────────
echo ""
info "Checking bundle structure..."
KNOWN_FILES=("${REQUIRED_FILES[@]}" "bundle_manifest.json")
while IFS= read -r -d '' entry; do
  bname="$(basename "$entry")"
  found=0
  for k in "${KNOWN_FILES[@]}"; do
    [[ "$k" == "$bname" ]] && found=1 && break
  done
  if [[ $found -eq 0 ]]; then
    echo -e "  ${YELLOW}?${RESET} $bname (unexpected file — not part of standard bundle)"
  fi
done < <(find "$BUNDLE_DIR" -maxdepth 1 -type f -print0)

# ── result ─────────────────────────────────────────────────────────────────────
echo ""
if [[ $ERRORS -eq 0 ]]; then
  info "PASSED — bundle is valid (${#REQUIRED_FILES[@]} required files present and valid)"
  echo ""
  exit 0
else
  echo -e "${RED}[validate] FAILED${RESET} — $ERRORS validation error(s)" >&2
  echo ""
  exit 1
fi
