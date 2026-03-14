#!/usr/bin/env bash
# lab_simulator.sh — simulate an external lab response or generate a handoff pack.
#
# Modes:
#
#   Response simulation (default):
#     ./examples/pilot/lab_simulator.sh [BUNDLE_DIR] [OUTPUT_FILE]
#
#     BUNDLE_DIR   source bundle directory   (default: pilot_bundle)
#     OUTPUT_FILE  output lab response file  (default: lab_response.json)
#
#     Generates a lab_response.json artifact bound to the current run.
#
#   External handoff pack:
#     ./examples/pilot/lab_simulator.sh --handoff-pack <output_dir> \
#                                       [--bundle <bundle_dir>]
#
#     Generates a directory with artifacts and plain-text instructions
#     for sending to a real external lab for trial runs.
#
#     Pack structure:
#       <output_dir>/<run-id>/
#         manifest.txt
#         operator_instructions.txt
#         lab_response_instructions.txt
#         artifacts/
#           receipt.json
#           export_packet.json  (if present)
#
# Exit codes:
#   0  success
#   1  bundle directory missing or required artifacts not found

set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'
BOLD='\033[1m'; RESET='\033[0m'

# ── Parse arguments ────────────────────────────────────────────────────────────

MODE="simulate"
HANDOFF_DIR=""
BUNDLE_DIR=""

# Detect --handoff-pack mode
if [[ "${1:-}" == "--handoff-pack" ]]; then
  MODE="handoff"
  HANDOFF_DIR="${2:?--handoff-pack requires an output directory argument}"
  shift 2
  # Parse remaining flags
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --bundle)
        BUNDLE_DIR="${2:?--bundle requires a directory argument}"
        shift 2
        ;;
      *)
        echo "error: unknown argument: $1" >&2
        exit 1
        ;;
    esac
  done
  BUNDLE_DIR="${BUNDLE_DIR:-pilot_bundle}"
else
  # Simulation mode — positional args
  BUNDLE_DIR="${1:-pilot_bundle}"
  OUTPUT_FILE="${2:-lab_response.json}"
fi

# ── Shared JSON field extractor ───────────────────────────────────────────────

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

# ── Shared: extract run identifiers from bundle ───────────────────────────────

_extract_bundle_ids() {
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
    [[ -z "$RECEIPT_HASH"          ]] && RECEIPT_HASH=$(_field "$BUNDLE_DIR/export_packet.json" "receipt_hash")
    [[ -z "$CASE_ID"               ]] && CASE_ID=$(_field "$BUNDLE_DIR/export_packet.json" "case_id")
    [[ -z "$SELECTED_CANDIDATE_ID" ]] && SELECTED_CANDIDATE_ID=$(_field "$BUNDLE_DIR/export_packet.json" "selected_candidate_id")
  fi
}

# ── Mode: external handoff pack ───────────────────────────────────────────────

if [[ "$MODE" == "handoff" ]]; then

  echo ""
  echo -e "${BOLD}PostCAD — External Handoff Pack${RESET}"
  echo "  ────────────────────────────────────────"
  echo ""

  if [[ ! -d "$BUNDLE_DIR" ]]; then
    echo -e "  ${RED}error${RESET}: bundle directory not found: $BUNDLE_DIR" >&2
    exit 1
  fi

  _extract_bundle_ids

  if [[ -z "$RECEIPT_HASH" ]]; then
    echo -e "  ${RED}error${RESET}: receipt_hash not found in bundle artifacts" >&2
    echo "  Expected receipt.json or export_packet.json in: $BUNDLE_DIR" >&2
    exit 1
  fi

  # Use case_id as run-id for the pack directory name; fall back to first 12 chars of receipt_hash
  RUN_ID="${CASE_ID:-${RECEIPT_HASH:0:12}}"
  PACK_DIR="$HANDOFF_DIR/$RUN_ID"
  ARTIFACTS_DIR="$PACK_DIR/artifacts"
  TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

  mkdir -p "$ARTIFACTS_DIR"

  # ── Copy artifacts ──────────────────────────────────────────────────────────

  ARTIFACT_LIST=""

  if [[ -f "$BUNDLE_DIR/receipt.json" && -s "$BUNDLE_DIR/receipt.json" ]]; then
    cp "$BUNDLE_DIR/receipt.json" "$ARTIFACTS_DIR/receipt.json"
    ARTIFACT_LIST="${ARTIFACT_LIST}  artifacts/receipt.json\n"
  fi

  if [[ -f "$BUNDLE_DIR/export_packet.json" && -s "$BUNDLE_DIR/export_packet.json" ]]; then
    cp "$BUNDLE_DIR/export_packet.json" "$ARTIFACTS_DIR/export_packet.json"
    ARTIFACT_LIST="${ARTIFACT_LIST}  artifacts/export_packet.json\n"
  fi

  ARTIFACT_LIST="${ARTIFACT_LIST}  manifest.txt\n"
  ARTIFACT_LIST="${ARTIFACT_LIST}  operator_instructions.txt\n"
  ARTIFACT_LIST="${ARTIFACT_LIST}  lab_response_instructions.txt\n"

  # ── Write manifest.txt ──────────────────────────────────────────────────────

  {
    echo "PostCAD External Handoff Pack"
    echo "run_id: $RUN_ID"
    echo "receipt_hash: $RECEIPT_HASH"
    [[ -n "${DISPATCH_ID:-}"           ]] && echo "dispatch_id: $DISPATCH_ID"
    [[ -n "${SELECTED_CANDIDATE_ID:-}" ]] && echo "selected_candidate: $SELECTED_CANDIDATE_ID"
    echo "generated_at: $TIMESTAMP"
    echo ""
    echo "files:"
    printf "%b" "$ARTIFACT_LIST"
  } > "$PACK_DIR/manifest.txt"

  # ── Write operator_instructions.txt ────────────────────────────────────────

  {
    echo "PostCAD External Handoff Pack — Operator Instructions"
    echo "======================================================"
    echo ""
    echo "Run ID:       $RUN_ID"
    echo "Receipt hash: $RECEIPT_HASH"
    [[ -n "${DISPATCH_ID:-}" ]] && echo "Dispatch ID:  $DISPATCH_ID"
    echo ""
    echo "This pack contains the routing receipt and dispatch packet for the above run."
    echo "Send the complete handoff pack directory to the external lab."
    echo ""
    echo "The lab must return a lab_response.json artifact acknowledging the run."
    echo "See lab_response_instructions.txt for the expected response format."
    echo ""
    echo "After receiving the lab response:"
    echo ""
    echo "  1. Place the response file into your inbound directory:"
    echo "       cp lab_response.json inbound/lab_response_<run-id>.json"
    echo ""
    echo "  2. Run single-artifact verification:"
    echo "       ./examples/pilot/verify.sh --inbound inbound/lab_response_<run-id>.json \\"
    echo "                                  --bundle $BUNDLE_DIR"
    echo ""
    echo "  3. Or run batch intake triage for all inbound responses:"
    echo "       ./examples/pilot/verify.sh --batch-inbound inbound/ --bundle $BUNDLE_DIR"
    echo ""
    echo "The response will be REJECTED if the receipt_hash does not match exactly."
  } > "$PACK_DIR/operator_instructions.txt"

  # ── Write lab_response_instructions.txt ─────────────────────────────────────

  {
    echo "PostCAD External Handoff Pack — Lab Response Instructions"
    echo "========================================================="
    echo ""
    echo "Run ID:       $RUN_ID"
    echo "Receipt hash: $RECEIPT_HASH"
    [[ -n "${DISPATCH_ID:-}" ]] && echo "Dispatch ID:  $DISPATCH_ID"
    echo ""
    echo "You have received a PostCAD routing decision for a dental case."
    echo "The artifacts/ directory contains the routing receipt and dispatch packet."
    echo ""
    echo "To acknowledge receipt of this case, return a response file named:"
    echo "  lab_response.json"
    echo ""
    echo "Required content:"
    echo ""
    echo "  {"
    echo "    \"lab_response_schema\": \"1\","
    echo "    \"receipt_hash\": \"$RECEIPT_HASH\","
    [[ -n "${DISPATCH_ID:-}" ]] && \
    echo "    \"dispatch_id\": \"$DISPATCH_ID\","
    echo "    \"case_id\": \"${CASE_ID:-}\","
    echo "    \"lab_acknowledged_at\": \"<ISO 8601 timestamp>\","
    echo "    \"lab_id\": \"<your lab identifier>\","
    echo "    \"status\": \"accepted\""
    echo "  }"
    echo ""
    echo "The receipt_hash field must match exactly:"
    echo "  $RECEIPT_HASH"
    echo ""
    echo "The response will be rejected if receipt_hash does not match."
  } > "$PACK_DIR/lab_response_instructions.txt"

  # ── Print result ────────────────────────────────────────────────────────────

  echo -e "  ${GREEN}✓${RESET}  Handoff pack written: $PACK_DIR"
  echo ""
  echo "  Run ID       : $RUN_ID"
  echo "  Receipt hash : $RECEIPT_HASH"
  [[ -n "${DISPATCH_ID:-}" ]] && echo "  Dispatch ID  : $DISPATCH_ID"
  echo ""
  echo "  Contents:"
  printf "%b" "$ARTIFACT_LIST" | sed 's/^/  /'
  echo ""
  echo "  Next steps:"
  echo "    1. Send $PACK_DIR to the external lab."
  echo "    2. The lab returns lab_response.json."
  echo "    3. Place the response in your inbound directory."
  echo "    4. Verify:"
  echo "       ./examples/pilot/verify.sh --inbound <lab_response.json> --bundle $BUNDLE_DIR"
  echo ""
  exit 0
fi

# ── Mode: lab response simulation (original) ──────────────────────────────────

echo ""
echo -e "${BOLD}PostCAD — Lab Response Simulator${RESET}"
echo "  ────────────────────────────────────────"
echo ""

if [[ ! -d "$BUNDLE_DIR" ]]; then
  echo -e "  ${RED}error${RESET}: bundle directory not found: $BUNDLE_DIR" >&2
  exit 1
fi

_extract_bundle_ids

if [[ -z "$RECEIPT_HASH" ]]; then
  echo -e "  ${RED}error${RESET}: receipt_hash not found in bundle artifacts" >&2
  echo "  Expected receipt.json or export_packet.json in: $BUNDLE_DIR" >&2
  exit 1
fi

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
[[ -n "${CASE_ID:-}"     ]] && echo "  Case ID      : $CASE_ID"
[[ -n "${DISPATCH_ID:-}" ]] && echo "  Dispatch ID  : $DISPATCH_ID"
echo ""
echo "  Verify inbound response:"
echo "    ./examples/pilot/verify.sh --inbound $OUTPUT_FILE --bundle $BUNDLE_DIR"
echo ""
