#!/usr/bin/env bash
# lab_simulator.sh — simulate an external lab acknowledging a pilot bundle.
#
# Usage:
#   ./examples/pilot/lab_simulator.sh [BUNDLE_DIR] [OUTPUT_FILE]
#
#   BUNDLE_DIR   source bundle directory   (default: pilot_bundle)
#   OUTPUT_FILE  output lab response file  (default: lab_response.json)
#
# Reads the exported run bundle and emits a deterministic lab_response.json
# artifact bound to the exact run (receipt_hash, dispatch_id, case_id).
#
# The response artifact is consumed by:
#   ./examples/pilot/verify.sh --inbound <lab_response.json> --bundle <bundle_dir>
#
# Exit codes:
#   0  lab_response.json written
#   1  bundle directory missing or required artifacts not found

set -euo pipefail

BUNDLE_DIR="${1:-pilot_bundle}"
OUTPUT_FILE="${2:-lab_response.json}"

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'
BOLD='\033[1m'; RESET='\033[0m'

_field() {
  local file="$1" key="$2"
  if command -v python3 &>/dev/null; then
    python3 -c "
import json
try:
    d = json.load(open('$file'))
    keys = '$key'.split('.')
    for k in keys:
        d = d.get(k,'') if isinstance(d,dict) else ''
    print(d)
except: print('')
" 2>/dev/null || echo ""
  elif command -v jq &>/dev/null; then
    jq -r ".$key // \"\"" "$file" 2>/dev/null || echo ""
  else
    echo ""
  fi
}

echo ""
echo -e "${BOLD}PostCAD — Lab Response Simulator${RESET}"
echo "  ────────────────────────────────────────"
echo ""

if [[ ! -d "$BUNDLE_DIR" ]]; then
  echo -e "  ${RED}error${RESET}: bundle directory not found: $BUNDLE_DIR" >&2
  exit 1
fi

# ── Extract run identifiers from bundle ───────────────────────────────────────

RECEIPT_HASH=""
DISPATCH_ID=""
CASE_ID=""
SELECTED_CANDIDATE_ID=""

if [[ -f "$BUNDLE_DIR/receipt.json" && -s "$BUNDLE_DIR/receipt.json" ]]; then
  RECEIPT_HASH=$(_field "$BUNDLE_DIR/receipt.json" "receipt_hash")
  CASE_ID=$(_field "$BUNDLE_DIR/receipt.json" "routing_input.case_id")
  SELECTED_CANDIDATE_ID=$(_field "$BUNDLE_DIR/receipt.json" "selected_candidate_id")
fi

if [[ -f "$BUNDLE_DIR/export_packet.json" && -s "$BUNDLE_DIR/export_packet.json" ]]; then
  DISPATCH_ID=$(_field "$BUNDLE_DIR/export_packet.json" "dispatch_id")
  [[ -z "$RECEIPT_HASH"           ]] && RECEIPT_HASH=$(_field "$BUNDLE_DIR/export_packet.json" "receipt_hash")
  [[ -z "$CASE_ID"                ]] && CASE_ID=$(_field "$BUNDLE_DIR/export_packet.json" "case_id")
  [[ -z "$SELECTED_CANDIDATE_ID" ]] && SELECTED_CANDIDATE_ID=$(_field "$BUNDLE_DIR/export_packet.json" "selected_candidate_id")
fi

if [[ -z "$RECEIPT_HASH" ]]; then
  echo -e "  ${RED}error${RESET}: receipt_hash not found in bundle artifacts" >&2
  echo "  Expected receipt.json or export_packet.json in: $BUNDLE_DIR" >&2
  exit 1
fi

# ── Write lab_response.json ───────────────────────────────────────────────────

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

cat > "$OUTPUT_FILE" <<EOF
{
  "lab_response_schema": "1",
  "receipt_hash": "$RECEIPT_HASH",
  "dispatch_id": "${DISPATCH_ID:-}",
  "case_id": "${CASE_ID:-}",
  "selected_candidate_id": "${SELECTED_CANDIDATE_ID:-}",
  "lab_acknowledged_at": "$TIMESTAMP",
  "lab_id": "lab-simulator-001",
  "status": "accepted"
}
EOF

echo -e "  ${GREEN}✓${RESET}  Lab response written: $OUTPUT_FILE"
echo ""
echo "  Receipt hash : $RECEIPT_HASH"
[[ -n "${CASE_ID:-}"      ]] && echo "  Case ID      : $CASE_ID"
[[ -n "${DISPATCH_ID:-}"  ]] && echo "  Dispatch ID  : $DISPATCH_ID"
echo ""
echo "  Verify inbound response:"
echo "    ./examples/pilot/verify.sh --inbound $OUTPUT_FILE --bundle $BUNDLE_DIR"
echo ""
