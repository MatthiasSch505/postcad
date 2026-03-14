#!/usr/bin/env bash
# PostCAD Protocol v1 — Pilot Receipt & Inbound Lab Response Verification
#
# Usage:
#   ./verify.sh                                        # verify receipt (human-readable)
#   ./verify.sh --json                                 # verify receipt (JSON output)
#   ./verify.sh --inbound <lab_response.json>          # verify single inbound response
#   ./verify.sh --inbound <lab_response.json> \
#               --bundle  <bundle_dir>                 # verify against specific bundle
#   ./verify.sh --batch-inbound <dir>                  # batch intake triage of all *.json in dir
#   ./verify.sh --batch-inbound <dir> \
#               --bundle  <bundle_dir> \
#               --report  <report_file>                # batch triage with written report
#
# Inbound verification outcomes (single):
#   response verified for current run
#   response belongs to different run
#   response missing required artifact/field
#   response cannot be verified
#
# Batch triage classifications:
#   accepted        — receipt hash matches current run
#   mismatch        — receipt hash belongs to different run
#   malformed       — missing required field (receipt_hash)
#   unverifiable    — file not valid JSON or no bundle receipt
#   duplicate       — identical receipt_hash already accepted in this batch
#
# Exit codes:
#   0  verification passed / at least one artifact accepted in batch
#   1  verification failed, artifact missing, or no accepted artifacts in batch

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BIN="${REPO_ROOT}/target/debug/postcad-cli"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BOLD='\033[1m'; RESET='\033[0m'

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

# ── Parse arguments ────────────────────────────────────────────────────────────

MODE="receipt"
JSON_FLAG=""
INBOUND_FILE=""
BATCH_DIR=""
BUNDLE_DIR="$SCRIPT_DIR"
REPORT_FILE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --inbound)
      MODE="inbound"
      INBOUND_FILE="${2:?--inbound requires a file argument}"
      shift 2
      ;;
    --batch-inbound)
      MODE="batch"
      BATCH_DIR="${2:?--batch-inbound requires a directory argument}"
      shift 2
      ;;
    --bundle)
      BUNDLE_DIR="${2:?--bundle requires a directory argument}"
      shift 2
      ;;
    --report)
      REPORT_FILE="${2:?--report requires a file argument}"
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

# ── Shared: resolve bundle receipt hash ───────────────────────────────────────

_bundle_receipt_hash() {
  local dir="$1"
  local h=""
  if [[ -f "$dir/receipt.json" && -s "$dir/receipt.json" ]]; then
    h=$(_field "$dir/receipt.json" "receipt_hash")
  fi
  if [[ -z "$h" && -f "$dir/export_packet.json" && -s "$dir/export_packet.json" ]]; then
    h=$(_field "$dir/export_packet.json" "receipt_hash")
  fi
  echo "$h"
}

_bundle_dispatch_id() {
  local dir="$1"
  if [[ -f "$dir/export_packet.json" && -s "$dir/export_packet.json" ]]; then
    _field "$dir/export_packet.json" "dispatch_id"
  else
    echo ""
  fi
}

# ── Mode: single inbound lab response verification ────────────────────────────

if [[ "$MODE" == "inbound" ]]; then

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

  # 5. Find bundle receipt hash
  BUNDLE_HASH=""
  BUNDLE_DISPATCH_ID=""
  BUNDLE_HASH=$(_bundle_receipt_hash "$BUNDLE_DIR")
  BUNDLE_DISPATCH_ID=$(_bundle_dispatch_id "$BUNDLE_DIR")

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
  [[ -n "$RESP_CASE_ID"     ]] && echo "  Case ID      : $RESP_CASE_ID"
  [[ -n "$RESP_DISPATCH_ID" ]] && echo "  Dispatch ID  : $RESP_DISPATCH_ID"
  echo ""
  exit 0
fi

# ── Mode: batch intake triage ─────────────────────────────────────────────────

if [[ "$MODE" == "batch" ]]; then

  echo ""
  echo -e "${BOLD}PostCAD — Operator Intake Triage${RESET}"
  echo "  ────────────────────────────────────────"
  echo ""

  if [[ ! -d "$BATCH_DIR" ]]; then
    echo -e "  ${RED}error${RESET}: inbound directory not found: $BATCH_DIR" >&2
    exit 1
  fi

  # Resolve bundle identifiers
  BUNDLE_HASH=""
  BUNDLE_DISPATCH_ID=""
  BUNDLE_HASH=$(_bundle_receipt_hash "$BUNDLE_DIR")
  BUNDLE_DISPATCH_ID=$(_bundle_dispatch_id "$BUNDLE_DIR")

  if [[ -z "$BUNDLE_HASH" ]]; then
    echo -e "  ${RED}error${RESET}: no receipt artifact found in bundle directory: $BUNDLE_DIR" >&2
    echo "  expected receipt.json or export_packet.json with receipt_hash" >&2
    exit 1
  fi

  echo "  Bundle directory : $BUNDLE_DIR"
  echo "  Receipt hash     : $BUNDLE_HASH"
  echo "  Inbound directory: $BATCH_DIR"
  echo ""

  # Collect inbound files in deterministic (sorted) order
  mapfile -t FILES < <(find "$BATCH_DIR" -maxdepth 1 -name "*.json" | sort)

  if [[ ${#FILES[@]} -eq 0 ]]; then
    echo "  No inbound artifacts found in: $BATCH_DIR"
    echo ""
    exit 0
  fi

  echo -e "  ${BOLD}Per-Artifact Results${RESET}"
  echo "  ────────────────────────────────────────"
  echo ""

  # Counters
  N_ACCEPTED=0
  N_MISMATCH=0
  N_MALFORMED=0
  N_UNVERIFIABLE=0
  N_DUPLICATE=0

  # Track accepted hashes for duplicate detection within this batch
  declare -a SEEN_HASHES=()

  # Report buffer (plain text lines)
  REPORT_LINES=""

  for FILE in "${FILES[@]}"; do
    BASENAME=$(basename "$FILE")
    CLASS=""
    REASON=""
    RESP_HASH=""
    RESP_DISPATCH_ID=""
    RESP_CASE_ID=""

    # 1. Check valid JSON
    if command -v python3 &>/dev/null; then
      if ! python3 -c "import json; json.load(open('$FILE'))" 2>/dev/null; then
        CLASS="unverifiable"
        REASON="not valid JSON"
        N_UNVERIFIABLE=$((N_UNVERIFIABLE + 1))
      fi
    fi

    if [[ -z "$CLASS" ]]; then
      RESP_HASH=$(_field "$FILE" "receipt_hash")
      RESP_DISPATCH_ID=$(_field "$FILE" "dispatch_id")
      RESP_CASE_ID=$(_field "$FILE" "case_id")

      # 2. Check required field
      if [[ -z "$RESP_HASH" ]]; then
        CLASS="malformed"
        REASON="missing required field: receipt_hash"
        N_MALFORMED=$((N_MALFORMED + 1))
      else
        # 3. Check duplicate (within this batch run)
        IS_DUP=false
        if [[ ${#SEEN_HASHES[@]} -gt 0 ]]; then
          for h in "${SEEN_HASHES[@]}"; do
            if [[ "$h" == "$RESP_HASH" ]]; then
              IS_DUP=true
              break
            fi
          done
        fi

        if [[ "$IS_DUP" == "true" ]]; then
          CLASS="duplicate"
          REASON="receipt_hash already accepted in this batch"
          N_DUPLICATE=$((N_DUPLICATE + 1))

        # 4. Check hash match
        elif [[ "$RESP_HASH" != "$BUNDLE_HASH" ]]; then
          CLASS="mismatch"
          REASON="receipt_hash does not match current run"
          N_MISMATCH=$((N_MISMATCH + 1))

        # 5. Check dispatch_id consistency if both present and non-empty
        elif [[ -n "$RESP_DISPATCH_ID" && -n "$BUNDLE_DISPATCH_ID" && "$RESP_DISPATCH_ID" != "$BUNDLE_DISPATCH_ID" ]]; then
          CLASS="mismatch"
          REASON="dispatch_id does not match current run"
          N_MISMATCH=$((N_MISMATCH + 1))

        else
          CLASS="accepted"
          REASON="receipt_hash matches current run"
          N_ACCEPTED=$((N_ACCEPTED + 1))
          SEEN_HASHES+=("$RESP_HASH")
        fi
      fi
    fi

    # Print per-artifact result
    case "$CLASS" in
      accepted)     CLR="$GREEN"  ;;
      mismatch)     CLR="$RED"    ;;
      malformed)    CLR="$RED"    ;;
      unverifiable) CLR="$YELLOW" ;;
      duplicate)    CLR="$YELLOW" ;;
      *)            CLR="$RESET"  ;;
    esac

    printf "  ${CLR}%-14s${RESET}  %s\n" "$CLASS" "$BASENAME"
    printf "  %-14s  Reason  : %s\n" "" "$REASON"
    [[ -n "$RESP_HASH"    ]] && printf "  %-14s  Hash    : %s\n" "" "$RESP_HASH"
    [[ -n "$RESP_CASE_ID" ]] && printf "  %-14s  Case ID : %s\n" "" "$RESP_CASE_ID"
    echo ""

    REPORT_LINES="${REPORT_LINES}${CLASS}  ${BASENAME}  reason=${REASON}  hash=${RESP_HASH:-—}\n"
  done

  N_TOTAL=${#FILES[@]}

  echo "  ────────────────────────────────────────"
  echo -e "  ${BOLD}Intake Summary${RESET}"
  echo ""
  printf "  %-20s %d\n" "Total processed:"  "$N_TOTAL"
  printf "  %-20s %d\n" "Accepted:"         "$N_ACCEPTED"
  printf "  %-20s %d\n" "Mismatched:"       "$N_MISMATCH"
  printf "  %-20s %d\n" "Malformed:"        "$N_MALFORMED"
  printf "  %-20s %d\n" "Unverifiable:"     "$N_UNVERIFIABLE"
  printf "  %-20s %d\n" "Duplicate:"        "$N_DUPLICATE"
  echo ""

  # Write report file if requested
  if [[ -n "$REPORT_FILE" ]]; then
    {
      echo "PostCAD Intake Triage Report"
      echo "bundle_dir=${BUNDLE_DIR}"
      echo "inbound_dir=${BATCH_DIR}"
      echo "receipt_hash=${BUNDLE_HASH}"
      echo "generated_at=$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
      echo ""
      echo "--- per-artifact results ---"
      printf "%b" "$REPORT_LINES"
      echo ""
      echo "--- summary ---"
      echo "total=${N_TOTAL}"
      echo "accepted=${N_ACCEPTED}"
      echo "mismatch=${N_MISMATCH}"
      echo "malformed=${N_MALFORMED}"
      echo "unverifiable=${N_UNVERIFIABLE}"
      echo "duplicate=${N_DUPLICATE}"
    } > "$REPORT_FILE"
    echo "  Report written: $REPORT_FILE"
    echo ""
  fi

  if [[ $N_ACCEPTED -eq 0 ]]; then
    exit 1
  fi
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
