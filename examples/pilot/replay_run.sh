#!/usr/bin/env bash
# replay_run.sh — print a deterministic summary of a PostCAD pilot run bundle.
#
# Usage:
#   ./examples/pilot/replay_run.sh [BUNDLE_DIR]
#
#   BUNDLE_DIR  bundle directory to replay  (default: pilot_bundle)
#
# Reads route.json, receipt.json, and verification.json from the bundle and
# prints a concise human-readable run summary. Performs a cross-artifact
# consistency check to confirm that all artifacts belong to the same run.
#
# Exit codes:
#   0  summary printed; artifacts are consistent
#   1  required artifact missing or cross-artifact consistency check failed

set -euo pipefail

BUNDLE_DIR="${1:-pilot_bundle}"

# ── colour helpers ─────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; RESET='\033[0m'
info()  { echo -e "${GREEN}[replay]${RESET} $*"; }
label() { echo -e "  ${CYAN}$1${RESET}"; }
kv()    { printf "  %-28s %s\n" "$1" "$2"; }
warn()  { echo -e "  ${YELLOW}⚠${RESET}  $*"; }
fail()  { echo -e "  ${RED}✗${RESET}  $*" >&2; }

# ── locate json extractor ──────────────────────────────────────────────────────
extract_field() {
  local file="$1" field="$2"
  if command -v python3 &>/dev/null; then
    python3 -c "
import json, sys
data = json.load(open('$file'))
keys = '$field'.split('.')
for k in keys:
    data = data.get(k, '') if isinstance(data, dict) else ''
print(data)
" 2>/dev/null || echo ""
  elif command -v jq &>/dev/null; then
    jq -r ".$field // \"\"" "$file" 2>/dev/null || echo ""
  else
    echo ""
  fi
}

# ── check required files ───────────────────────────────────────────────────────
REQUIRED=("route.json" "receipt.json" "verification.json")
for f in "${REQUIRED[@]}"; do
  if [[ ! -f "$BUNDLE_DIR/$f" ]]; then
    fail "Required artifact missing: $BUNDLE_DIR/$f"
    exit 1
  fi
done

echo ""
info "PostCAD pilot run replay — $BUNDLE_DIR"
echo "  ─────────────────────────────────────────"

# ── route summary ──────────────────────────────────────────────────────────────
label "Route"
OUTCOME=$(extract_field "$BUNDLE_DIR/route.json" "outcome")
SELECTED=$(extract_field "$BUNDLE_DIR/route.json" "selected_candidate_id")
JURISDICTION=$(extract_field "$BUNDLE_DIR/route.json" "routing_input.jurisdiction")
KVER=$(extract_field "$BUNDLE_DIR/route.json" "routing_kernel_version")
kv "Outcome:"      "${OUTCOME:-—}"
kv "Selected:"     "${SELECTED:-—}"
kv "Jurisdiction:" "${JURISDICTION:-—}"
kv "Kernel:"       "${KVER:-—}"
echo ""

# ── receipt summary ────────────────────────────────────────────────────────────
label "Receipt"
RECEIPT_HASH=$(extract_field "$BUNDLE_DIR/receipt.json" "receipt_hash")
ROUTE_HASH=$(extract_field "$BUNDLE_DIR/route.json"    "receipt_hash")
CREATED_AT=$(extract_field "$BUNDLE_DIR/receipt.json"  "created_at")
kv "Receipt hash:"  "${RECEIPT_HASH:-—}"
kv "Created at:"    "${CREATED_AT:-—}"
echo ""

# ── verification summary ───────────────────────────────────────────────────────
label "Verification"
VERIFY_RESULT=$(extract_field "$BUNDLE_DIR/verification.json" "result")
kv "Result:" "${VERIFY_RESULT:-—}"
echo ""

# ── cross-artifact consistency check ──────────────────────────────────────────
label "Consistency"
CONSISTENT=1

if [[ -n "$RECEIPT_HASH" && -n "$ROUTE_HASH" ]]; then
  if [[ "$RECEIPT_HASH" == "$ROUTE_HASH" ]]; then
    kv "Receipt hash (route vs receipt):" "match ✓"
  else
    kv "Receipt hash (route vs receipt):" "MISMATCH ✗"
    warn "route.json receipt_hash does not match receipt.json receipt_hash"
    CONSISTENT=0
  fi
else
  kv "Receipt hash cross-check:" "skipped (hash not found in one or both files)"
fi

# check verification result is consistent with outcome
if [[ "$OUTCOME" == "routed" && "$VERIFY_RESULT" == "VERIFIED" ]]; then
  kv "Outcome / verification agreement:" "consistent ✓"
elif [[ -z "$OUTCOME" || -z "$VERIFY_RESULT" ]]; then
  kv "Outcome / verification agreement:" "skipped (fields not found)"
elif [[ "$OUTCOME" != "routed" ]]; then
  kv "Outcome / verification note:" "route did not produce routed outcome"
else
  kv "Outcome / verification agreement:" "attention — routed but not VERIFIED"
  warn "Route shows 'routed' but verification did not pass"
fi

echo ""
echo "  ─────────────────────────────────────────"

if [[ $CONSISTENT -eq 1 ]]; then
  info "Run summary complete — artifacts are consistent"
else
  echo -e "${RED}[replay] Cross-artifact consistency check FAILED${RESET}" >&2
  exit 1
fi
echo ""
