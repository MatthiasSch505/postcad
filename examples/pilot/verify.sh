#!/usr/bin/env bash
# PostCAD Protocol v1 — Pilot Receipt & Inbound Lab Response Verification
#
# Usage:
#   ./verify.sh                                        # verify receipt (human-readable)
#   ./verify.sh --json                                 # verify receipt (JSON output)
#   ./verify.sh --inbound <lab_response.json>          # verify inbound lab response
#   ./verify.sh --inbound <lab_response.json> \
#               --bundle  <bundle_dir>                 # verify against specific bundle
#
# Inbound verification outcomes:
#   response verified for current run
#   response belongs to different run
#   response missing required artifact/field
#   response cannot be verified
#
# Exit codes:
#   0  verification passed
#   1  verification failed or artifact missing

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BIN="${REPO_ROOT}/target/debug/postcad-cli"

RED='\033[0;31m'; GREEN='\033[0;32m'
BOLD='\033[1m'; RESET='\033[0m'

# ── Parse arguments ────────────────────────────────────────────────────────────

MODE="receipt"
JSON_FLAG=""
INBOUND_FILE=""
BUNDLE_DIR="$SCRIPT_DIR"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --inbound)
      MODE="inbound"
      INBOUND_FILE="${2:?--inbound requires a file argument}"
      shift 2
      ;;
    --bundle)
      BUNDLE_DIR="${2:?--bundle requires a directory argument}"
      shift 2
      ;;
    --json)
      JSON_FLAG="--json"
      shift
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

# ── Mode: inbound lab response verification ────────────────────────────────────

if [[ "$MODE" == "inbound" ]]; then

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
  echo -e "${BOLD}PostCAD — Inbound Lab Response Verification${RESET}"
  echo "  ────────────────────────────────────────"
  echo ""

  # 1. Check response file exists
  if [[ ! -f "$INBOUND_FILE" ]]; then
    echo -e "  ${RED}response cannot be verified${RESET}"
    echo ""
    echo "  Reason: lab response file not found: $INBOUND_FILE"
    echo ""
    exit 1
  fi

  # 2. Check response is valid JSON
  if command -v python3 &>/dev/null; then
    if ! python3 -c "import json; json.load(open('$INBOUND_FILE'))" 2>/dev/null; then
      echo -e "  ${RED}response cannot be verified${RESET}"
      echo ""
      echo "  Reason: lab response file is not valid JSON: $INBOUND_FILE"
      echo ""
      exit 1
    fi
  fi

  # 3. Extract fields from lab response
  RESP_HASH=""
  RESP_DISPATCH_ID=""
  RESP_CASE_ID=""
  RESP_HASH=$(_field "$INBOUND_FILE" "receipt_hash")
  RESP_DISPATCH_ID=$(_field "$INBOUND_FILE" "dispatch_id")
  RESP_CASE_ID=$(_field "$INBOUND_FILE" "case_id")

  # 4. Check required field
  if [[ -z "$RESP_HASH" ]]; then
    echo -e "  ${RED}response missing required artifact/field${RESET}"
    echo ""
    echo "  Reason: lab response is missing required field: receipt_hash"
    echo "  File:   $INBOUND_FILE"
    echo ""
    exit 1
  fi

  # 5. Find bundle receipt hash — prefer receipt.json, fall back to export_packet.json
  BUNDLE_HASH=""
  BUNDLE_DISPATCH_ID=""
  if [[ -f "$BUNDLE_DIR/receipt.json" && -s "$BUNDLE_DIR/receipt.json" ]]; then
    BUNDLE_HASH=$(_field "$BUNDLE_DIR/receipt.json" "receipt_hash")
  fi
  if [[ -z "$BUNDLE_HASH" && -f "$BUNDLE_DIR/export_packet.json" && -s "$BUNDLE_DIR/export_packet.json" ]]; then
    BUNDLE_HASH=$(_field "$BUNDLE_DIR/export_packet.json" "receipt_hash")
  fi
  if [[ -f "$BUNDLE_DIR/export_packet.json" && -s "$BUNDLE_DIR/export_packet.json" ]]; then
    BUNDLE_DISPATCH_ID=$(_field "$BUNDLE_DIR/export_packet.json" "dispatch_id")
  fi

  if [[ -z "$BUNDLE_HASH" ]]; then
    echo -e "  ${RED}response cannot be verified${RESET}"
    echo ""
    echo "  Reason: no receipt artifact found in bundle directory: $BUNDLE_DIR"
    echo "          expected receipt.json or export_packet.json with receipt_hash"
    echo ""
    exit 1
  fi

  # 6. Compare receipt hashes
  if [[ "$RESP_HASH" != "$BUNDLE_HASH" ]]; then
    echo -e "  ${RED}response belongs to different run${RESET}"
    echo ""
    echo "  Receipt hash mismatch:"
    echo "    bundle   : $BUNDLE_HASH"
    echo "    response : $RESP_HASH"
    echo ""
    exit 1
  fi

  # 7. Check dispatch_id consistency if both are present and non-empty
  if [[ -n "$RESP_DISPATCH_ID" && -n "$BUNDLE_DISPATCH_ID" && "$RESP_DISPATCH_ID" != "$BUNDLE_DISPATCH_ID" ]]; then
    echo -e "  ${RED}response belongs to different run${RESET}"
    echo ""
    echo "  Dispatch ID mismatch:"
    echo "    bundle   : $BUNDLE_DISPATCH_ID"
    echo "    response : $RESP_DISPATCH_ID"
    echo ""
    exit 1
  fi

  # 8. Verified
  echo -e "  ${GREEN}response verified for current run${RESET}"
  echo ""
  echo "  Receipt hash : $BUNDLE_HASH"
  [[ -n "$RESP_CASE_ID"       ]] && echo "  Case ID      : $RESP_CASE_ID"
  [[ -n "$RESP_DISPATCH_ID"   ]] && echo "  Dispatch ID  : $RESP_DISPATCH_ID"
  echo ""
  exit 0
fi

# ── Mode: receipt verification (original) ─────────────────────────────────────

if [[ ! -x "$BIN" ]]; then
  echo "Building postcad-cli..."
  cargo build --bin postcad-cli --quiet --manifest-path "${REPO_ROOT}/Cargo.toml"
fi

RECEIPT="${SCRIPT_DIR}/receipt.json"

if [[ ! -f "$RECEIPT" ]]; then
  echo "error: receipt.json not found — run run_pilot.sh first" >&2
  exit 1
fi

echo "PostCAD Protocol v1 — Pilot Receipt Verification"
echo "=================================================="
echo ""

"$BIN" verify-receipt $JSON_FLAG \
  --receipt    "${SCRIPT_DIR}/receipt.json" \
  --case       "${SCRIPT_DIR}/case.json" \
  --policy     "${SCRIPT_DIR}/derived_policy.json" \
  --candidates "${SCRIPT_DIR}/candidates.json"

echo ""
echo "Verification complete — receipt is authentic, dispatch is safe to proceed."
echo ""
echo "Next step: open the reviewer shell to create and approve a dispatch commitment."
echo "  cargo run -p postcad-service"
echo "  # then open http://localhost:8080/reviewer"
