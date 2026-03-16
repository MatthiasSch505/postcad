#!/usr/bin/env bash
# PostCAD Protocol v1 — Pilot Workflow
#
# Runs the full registry-backed routing and verification flow:
#   1. Build the CLI binary (if needed).
#   2. Route the pilot case against the registry snapshot.
#      Self-verification runs automatically inside the routing step;
#      the command exits non-zero if the receipt fails to verify.
#   3. Save the receipt to receipt.json.
#   4. Print the routing result.
#
# Usage: ./run_pilot.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BIN="${REPO_ROOT}/target/debug/postcad-cli"
REPORTS_DIR="$SCRIPT_DIR/reports"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BOLD='\033[1m'; RESET='\033[0m'

# ── Trial receipt ledger helpers ───────────────────────────────────────────────

_ledger_next_seq() {
  local ledger="$1"
  if [[ ! -f "$ledger" ]]; then printf "%03d" 1; return; fi
  local last
  last=$(grep "^sequence:" "$ledger" 2>/dev/null | tail -1 | sed 's/sequence: *//' | sed 's/^0*//')
  [[ -z "$last" ]] && last=0
  printf "%03d" $((last + 1))
}

_append_ledger() {
  local ledger="$1" event="$2" run_id="$3" result="$4" artifact="${5:-}"
  mkdir -p "$(dirname "$ledger")"
  local seq
  seq=$(_ledger_next_seq "$ledger")
  {
    echo "sequence: $seq"
    echo "event: $event"
    echo "run_id: $run_id"
    [[ -n "$artifact" ]] && echo "artifact: $artifact"
    echo "result: $result"
    echo "timestamp: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    echo ""
  } >> "$ledger"
}

# ── Mode: full trial run ───────────────────────────────────────────────────────

if [[ "${1:-}" == "--trial-run" ]]; then

  echo ""
  echo -e "${BOLD}PostCAD — Full Trial Run${RESET}"
  echo "  ────────────────────────────────────────"
  echo ""
  echo -e "  ${BOLD}Starting PostCAD trial run${RESET}"
  echo ""

  # Build
  if [[ ! -x "$BIN" ]]; then
    echo "  Building postcad-cli..."
    cargo build --bin postcad-cli --quiet --manifest-path "${REPO_ROOT}/Cargo.toml"
  fi

  # Route
  TRIAL_RECEIPT_JSON=$("$BIN" route-case-from-registry --json \
    --case     "${SCRIPT_DIR}/case.json" \
    --registry "${SCRIPT_DIR}/registry_snapshot.json" \
    --config   "${SCRIPT_DIR}/config.json")
  echo "$TRIAL_RECEIPT_JSON" > "${SCRIPT_DIR}/receipt.json"

  # Compute run ID
  TRIAL_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${SCRIPT_DIR}/receipt.json').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
  TRIAL_RECEIPT_HASH=$(echo "$TRIAL_RECEIPT_JSON" | grep -o '"receipt_hash": *"[^"]*"' | head -1 | sed 's/.*: *"\(.*\)"/\1/')
  TRIAL_RUN_ID="${TRIAL_CASE_ID:-${TRIAL_RECEIPT_HASH:0:12}}"
  TRIAL_LEDGER_FILE="$REPORTS_DIR/ledger_${TRIAL_RUN_ID}.txt"

  # Ledger: outbound_bundle_created
  if [[ -n "$TRIAL_RUN_ID" ]]; then
    _append_ledger "$TRIAL_LEDGER_FILE" "outbound_bundle_created" "$TRIAL_RUN_ID" "recorded" "${SCRIPT_DIR}/receipt.json"
  fi

  echo -e "  ${GREEN}Outbound bundle created${RESET}"

  # Handoff pack
  mkdir -p "${SCRIPT_DIR}/handoff"
  "${SCRIPT_DIR}/lab_simulator.sh" --handoff-pack "${SCRIPT_DIR}/handoff" \
    --bundle "${SCRIPT_DIR}" > /dev/null 2>&1

  echo -e "  ${GREEN}External handoff pack created${RESET}"

  # Simulate lab response
  mkdir -p "${SCRIPT_DIR}/inbound"
  "${SCRIPT_DIR}/lab_simulator.sh" "${SCRIPT_DIR}" \
    "${SCRIPT_DIR}/inbound/trial_response.json" > /dev/null 2>&1

  echo -e "  ${GREEN}Simulated lab response generated${RESET}"

  # Verify inbound response — capture exit code without triggering set -e
  TRIAL_VERIFY_EXIT=0
  "${SCRIPT_DIR}/verify.sh" --inbound "${SCRIPT_DIR}/inbound/trial_response.json" \
    --bundle "${SCRIPT_DIR}" > /dev/null 2>&1 || TRIAL_VERIFY_EXIT=$?

  echo -e "  ${GREEN}Inbound response verified${RESET}"

  # Operator decision
  if [[ $TRIAL_VERIFY_EXIT -eq 0 ]]; then
    echo -e "  ${GREEN}Operator decision: ACCEPTED${RESET}"
    TRIAL_DECISION="ACCEPTED"
  else
    echo -e "  ${RED}Operator decision: REJECTED${RESET}"
    TRIAL_DECISION="REJECTED"
  fi

  echo -e "  ${GREEN}Trial ledger updated${RESET}"
  echo ""
  echo -e "  ${BOLD}Trial run completed${RESET}"
  echo ""
  echo "  Run ID : ${TRIAL_RUN_ID}"
  echo "  Ledger : ${TRIAL_LEDGER_FILE}"
  echo "  Receipt: ${SCRIPT_DIR}/receipt.json"
  echo ""

  if [[ "$TRIAL_DECISION" == "ACCEPTED" ]]; then
    exit 0
  else
    exit 1
  fi
fi

# ── Mode: prepare manual reply template ───────────────────────────────────────

if [[ "${1:-}" == "--prepare-manual-reply" ]]; then

  RECEIPT="${SCRIPT_DIR}/receipt.json"

  if [[ ! -f "$RECEIPT" ]]; then
    echo "error: receipt.json not found — run run_pilot.sh first" >&2
    exit 1
  fi

  # Compute run_id (same logic as routing modes)
  PR_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${RECEIPT}').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
  PR_RECEIPT_HASH=$(grep -o '"receipt_hash": *"[^"]*"' "$RECEIPT" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
  PR_RUN_ID="${PR_CASE_ID:-${PR_RECEIPT_HASH:0:12}}"

  if [[ -z "$PR_RUN_ID" ]]; then
    echo "error: could not determine run_id from receipt.json" >&2
    exit 1
  fi

  TEMPLATE_SRC="${SCRIPT_DIR}/handoff/${PR_RUN_ID}/lab_reply_template.json"

  if [[ ! -f "$TEMPLATE_SRC" ]]; then
    echo "error: handoff pack not found for run: $PR_RUN_ID" >&2
    echo "  expected: $TEMPLATE_SRC" >&2
    echo "  generate it first:" >&2
    echo "    ./examples/pilot/lab_simulator.sh --handoff-pack handoff/ --bundle examples/pilot" >&2
    exit 1
  fi

  mkdir -p "${SCRIPT_DIR}/inbound"
  DEST="${SCRIPT_DIR}/inbound/lab_reply_${PR_RUN_ID}.json"
  cp "$TEMPLATE_SRC" "$DEST"

  echo ""
  echo "PostCAD — Manual Reply Template"
  echo "  ────────────────────────────────────────"
  echo ""
  echo "  Reply template prepared for manual completion:"
  echo "    $DEST"
  echo ""
  echo "  Run ID      : $PR_RUN_ID"
  echo "  Receipt hash: $PR_RECEIPT_HASH"
  echo ""
  echo "  The lab must fill in:"
  echo "    lab_acknowledged_at  — ISO 8601 timestamp"
  echo "    lab_id               — lab identifier"
  echo ""
  echo "  Fields that must not be changed:"
  echo "    lab_response_schema, receipt_hash, dispatch_id, case_id, status"
  echo ""
  echo "  After the lab returns the filled file, verify it:"
  echo "    ./examples/pilot/verify.sh --inbound $DEST --bundle ${SCRIPT_DIR}"
  echo ""
  exit 0
fi

# ── Mode: export sendable lab trial package ───────────────────────────────────

if [[ "${1:-}" == "--export-lab-trial-package" ]]; then

  RECEIPT="${SCRIPT_DIR}/receipt.json"

  if [[ ! -f "$RECEIPT" ]]; then
    echo "error: receipt.json not found — run run_pilot.sh first" >&2
    exit 1
  fi

  # Compute run_id
  EXP_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${RECEIPT}').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
  EXP_RECEIPT_HASH=$(grep -o '"receipt_hash": *"[^"]*"' "$RECEIPT" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
  EXP_RUN_ID="${EXP_CASE_ID:-${EXP_RECEIPT_HASH:0:12}}"

  if [[ -z "$EXP_RUN_ID" ]]; then
    echo "error: could not determine run_id from receipt.json" >&2
    exit 1
  fi

  EXP_DISPATCH_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${SCRIPT_DIR}/export_packet.json').read())
    print(d.get('dispatch_id', ''))
except: print('')
" 2>/dev/null || echo "")

  PACKAGE_DIR="${SCRIPT_DIR}/outbound/lab_trial_${EXP_RUN_ID}"
  mkdir -p "$PACKAGE_DIR"

  # ── Copy receipt ────────────────────────────────────────────────────────────
  cp "$RECEIPT" "$PACKAGE_DIR/receipt.json"

  # ── Copy export_packet if present ───────────────────────────────────────────
  if [[ -f "${SCRIPT_DIR}/export_packet.json" && -s "${SCRIPT_DIR}/export_packet.json" ]]; then
    cp "${SCRIPT_DIR}/export_packet.json" "$PACKAGE_DIR/export_packet.json"
  fi

  # ── Write lab_reply_template.json ───────────────────────────────────────────
  {
    echo "{"
    echo "  \"lab_response_schema\": \"1\","
    echo "  \"receipt_hash\": \"$EXP_RECEIPT_HASH\","
    echo "  \"dispatch_id\": \"${EXP_DISPATCH_ID:-}\","
    echo "  \"case_id\": \"${EXP_CASE_ID:-}\","
    echo "  \"lab_acknowledged_at\": \"FILL_IN: ISO 8601 timestamp e.g. $(date -u +"%Y-%m-%d")T00:00:00Z\","
    echo "  \"lab_id\": \"FILL_IN: your lab identifier\","
    echo "  \"status\": \"accepted\""
    echo "}"
  } > "$PACKAGE_DIR/lab_reply_template.json"

  # ── Write manifest.txt ───────────────────────────────────────────────────────
  MANIFEST_FILES="  manifest.txt\n  operator_instructions.txt\n  lab_instructions.txt\n  lab_reply_template.json\n  receipt.json"
  [[ -f "$PACKAGE_DIR/export_packet.json" ]] && MANIFEST_FILES="${MANIFEST_FILES}\n  export_packet.json"
  MANIFEST_FILES="${MANIFEST_FILES}\n  email_to_lab.txt\n  short_message_to_lab.txt\n  operator_send_note.txt"

  {
    echo "PostCAD Lab Trial Package"
    echo "run_id: $EXP_RUN_ID"
    echo "receipt_hash: $EXP_RECEIPT_HASH"
    echo "generated_at: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    echo ""
    echo "files:"
    printf "%b\n" "$MANIFEST_FILES"
  } > "$PACKAGE_DIR/manifest.txt"

  # ── Write operator_instructions.txt ─────────────────────────────────────────
  {
    echo "PostCAD Lab Trial Package — Operator Instructions"
    echo "=================================================="
    echo ""
    echo "Run ID:       $EXP_RUN_ID"
    echo "Receipt hash: $EXP_RECEIPT_HASH"
    echo ""
    echo "This folder is a sendable lab trial package for a real external PostCAD routing trial."
    echo ""
    echo "Send the following files to the external lab:"
    echo "  lab_reply_template.json"
    echo "  lab_instructions.txt"
    echo "  receipt.json"
    echo ""
    echo "The lab must return the completed template named:"
    echo "  lab_reply_${EXP_RUN_ID}.json"
    echo ""
    echo "After receiving the lab reply, place it in your inbound directory and verify:"
    echo "  ./examples/pilot/verify.sh --inbound inbound/lab_reply_${EXP_RUN_ID}.json \\"
    echo "                             --bundle examples/pilot"
  } > "$PACKAGE_DIR/operator_instructions.txt"

  # ── Write lab_instructions.txt ───────────────────────────────────────────────
  {
    echo "PostCAD Lab Trial Package — Lab Instructions"
    echo "============================================="
    echo ""
    echo "Run ID:       $EXP_RUN_ID"
    echo "Receipt hash: $EXP_RECEIPT_HASH"
    echo ""
    echo "You have received a PostCAD routing case for a dental manufacturing trial."
    echo "The routing receipt is in receipt.json."
    echo ""
    echo "To acknowledge receipt of this case, fill in lab_reply_template.json:"
    echo ""
    echo "  Fields to fill in:"
    echo "    lab_acknowledged_at  — ISO 8601 timestamp when you received this case"
    echo "    lab_id               — your lab identifier"
    echo ""
    echo "  Fields that must not be changed:"
    echo "    lab_response_schema, receipt_hash, dispatch_id, case_id, status"
    echo ""
    echo "Return the completed file named:"
    echo "  lab_reply_${EXP_RUN_ID}.json"
    echo ""
    echo "The reply will be rejected if receipt_hash does not match exactly:"
    echo "  $EXP_RECEIPT_HASH"
  } > "$PACKAGE_DIR/lab_instructions.txt"

  # ── Write email_to_lab.txt ───────────────────────────────────────────────────
  {
    echo "Subject: External workflow trial — PostCAD routing package (run ${EXP_RUN_ID})"
    echo ""
    echo "Hi,"
    echo ""
    echo "I am running a small external workflow trial for a dental manufacturing routing system."
    echo ""
    echo "I have attached a package for run ${EXP_RUN_ID}."
    echo "The package contains a routing receipt and a reply template."
    echo ""
    echo "To participate in the trial, please:"
    echo ""
    echo "  1. Open lab_instructions.txt in the package."
    echo "  2. Fill in two fields in lab_reply_template.json:"
    echo "       lab_acknowledged_at — the current date and time (ISO 8601 format)"
    echo "       lab_id              — your lab name or identifier"
    echo "  3. Return the completed file as:"
    echo "       lab_reply_${EXP_RUN_ID}.json"
    echo ""
    echo "No software integration is required."
    echo "This is a plain JSON file you can edit in any text editor."
    echo ""
    echo "If you have questions about the format, see lab_instructions.txt."
    echo ""
    echo "Thank you for taking the time."
    echo ""
    echo "Best regards"
  } > "$PACKAGE_DIR/email_to_lab.txt"

  # ── Write short_message_to_lab.txt ───────────────────────────────────────────
  {
    echo "Hi — I am testing a small external workflow system for dental manufacturing routing."
    echo "I have sent you a package for run ${EXP_RUN_ID}."
    echo "It contains a short reply template (lab_reply_template.json) with two fields to fill in."
    echo "Could you take a quick look and send back the completed file?"
    echo "No integration needed — just a plain JSON file."
    echo "Instructions are in lab_instructions.txt."
    echo "Thanks a lot."
  } > "$PACKAGE_DIR/short_message_to_lab.txt"

  # ── Write operator_send_note.txt ─────────────────────────────────────────────
  {
    echo "PostCAD — Operator Send Checklist"
    echo "=================================="
    echo ""
    echo "Run ID: $EXP_RUN_ID"
    echo ""
    echo "Steps:"
    echo ""
    echo "  [ ] 1. Zip the package directory:"
    echo "           zip -r lab_trial_${EXP_RUN_ID}.zip $PACKAGE_DIR"
    echo ""
    echo "  [ ] 2. Send the zip to the lab."
    echo "         Use email_to_lab.txt or short_message_to_lab.txt as your message."
    echo ""
    echo "  [ ] 3. Wait for the lab to return:"
    echo "           lab_reply_${EXP_RUN_ID}.json"
    echo ""
    echo "  [ ] 4. Place the returned file into your inbound directory:"
    echo "           cp lab_reply_${EXP_RUN_ID}.json ${SCRIPT_DIR}/inbound/lab_reply_${EXP_RUN_ID}.json"
    echo ""
    echo "  [ ] 5. Run verification and generate decision record:"
    echo "           ./examples/pilot/verify.sh \\"
    echo "             --inbound ${SCRIPT_DIR}/inbound/lab_reply_${EXP_RUN_ID}.json \\"
    echo "             --bundle  ${SCRIPT_DIR}"
    echo ""
    echo "  [ ] 6. Inspect the decision record:"
    echo "           cat ${SCRIPT_DIR}/reports/decision_lab_reply_${EXP_RUN_ID}.txt"
  } > "$PACKAGE_DIR/operator_send_note.txt"

  # ── Print result ─────────────────────────────────────────────────────────────
  echo ""
  echo -e "${BOLD}PostCAD — Lab Trial Package${RESET}"
  echo "  ────────────────────────────────────────"
  echo ""
  echo -e "  ${GREEN}Package written: $PACKAGE_DIR${RESET}"
  echo ""
  echo "  Run ID      : $EXP_RUN_ID"
  echo "  Receipt hash: $EXP_RECEIPT_HASH"
  echo ""
  echo "  Contents:"
  printf "%b\n" "$MANIFEST_FILES" | sed 's/^/  /'
  echo ""
  echo "  Next steps:"
  echo "    1. Send $PACKAGE_DIR to the external lab."
  echo "    2. The lab fills lab_reply_template.json and returns it."
  echo "    3. Place the reply in your inbound directory:"
  echo "         cp lab_reply_returned.json ${SCRIPT_DIR}/inbound/lab_reply_${EXP_RUN_ID}.json"
  echo "    4. Verify:"
  echo "       ./examples/pilot/verify.sh --inbound ${SCRIPT_DIR}/inbound/lab_reply_${EXP_RUN_ID}.json \\"
  echo "                                  --bundle ${SCRIPT_DIR}"
  echo ""
  exit 0
fi

# ── Mode: lab trial package self-check ────────────────────────────────────────

if [[ "${1:-}" == "--check-lab-trial-package" ]]; then

  RECEIPT="${SCRIPT_DIR}/receipt.json"

  if [[ ! -f "$RECEIPT" ]]; then
    echo "error: receipt.json not found — run run_pilot.sh first" >&2
    exit 1
  fi

  # Compute run_id
  CHK_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${RECEIPT}').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
  CHK_RECEIPT_HASH=$(grep -o '"receipt_hash": *"[^"]*"' "$RECEIPT" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
  CHK_RUN_ID="${CHK_CASE_ID:-${CHK_RECEIPT_HASH:0:12}}"

  if [[ -z "$CHK_RUN_ID" ]]; then
    echo "error: could not determine run_id from receipt.json" >&2
    exit 1
  fi

  PACKAGE_DIR="${SCRIPT_DIR}/outbound/lab_trial_${CHK_RUN_ID}"

  echo ""
  echo -e "${BOLD}PostCAD — Lab Trial Package Self-Check${RESET}"
  echo "  ────────────────────────────────────────"
  echo ""
  echo "  Run ID  : $CHK_RUN_ID"
  echo "  Package : $PACKAGE_DIR"
  echo ""

  if [[ ! -d "$PACKAGE_DIR" ]]; then
    echo -e "  ${RED}package check failed${RESET}"
    echo "  Reason: package directory not found"
    echo "  Generate it first:"
    echo "    ./examples/pilot/run_pilot.sh --export-lab-trial-package"
    echo ""
    exit 1
  fi

  # Required files
  REQUIRED_FILES=(
    "manifest.txt"
    "operator_instructions.txt"
    "lab_instructions.txt"
    "lab_reply_template.json"
    "email_to_lab.txt"
    "short_message_to_lab.txt"
    "operator_send_note.txt"
    "receipt.json"
  )

  CHK_PASS=true

  echo "  File check:"
  echo ""
  for f in "${REQUIRED_FILES[@]}"; do
    if [[ -f "$PACKAGE_DIR/$f" && -s "$PACKAGE_DIR/$f" ]]; then
      echo -e "  ${GREEN}present${RESET}  $f"
    else
      echo -e "  ${RED}missing${RESET}  $f"
      CHK_PASS=false
    fi
  done

  echo ""

  if [[ "$CHK_PASS" == "true" ]]; then
    echo -e "  ${GREEN}package ready for external lab send${RESET}"
    echo ""
    echo "  Next steps:"
    echo "    1. Zip and send:"
    echo "         zip -r lab_trial_${CHK_RUN_ID}.zip $PACKAGE_DIR"
    echo "    2. Use email_to_lab.txt or short_message_to_lab.txt as your message."
    echo "    3. Follow operator_send_note.txt for the full send checklist."
    echo ""
    exit 0
  else
    echo -e "  ${RED}package check failed${RESET}"
    echo "  Regenerate the package:"
    echo "    ./examples/pilot/run_pilot.sh --export-lab-trial-package"
    echo ""
    exit 1
  fi
fi

# ── Mode: inbound reply inspection summary ────────────────────────────────────

if [[ "${1:-}" == "--inspect-inbound-reply" ]]; then

  INBOUND_FILE="${2:-}"

  if [[ -z "$INBOUND_FILE" ]]; then
    # Try to auto-resolve from current run context
    if [[ -f "${SCRIPT_DIR}/receipt.json" ]]; then
      _AR_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${SCRIPT_DIR}/receipt.json').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
      _AR_RECEIPT_HASH=$(grep -o '"receipt_hash": *"[^"]*"' "${SCRIPT_DIR}/receipt.json" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
      _AR_RUN_ID="${_AR_CASE_ID:-${_AR_RECEIPT_HASH:0:12}}"
      if [[ -n "$_AR_RUN_ID" ]]; then
        _AR_CANDIDATE="${SCRIPT_DIR}/inbound/lab_reply_${_AR_RUN_ID}.json"
        if [[ -f "$_AR_CANDIDATE" ]]; then
          INBOUND_FILE="$_AR_CANDIDATE"
          echo ""
          echo "auto-resolved inbound reply: $INBOUND_FILE"
          echo ""
        else
          echo ""
          echo "INBOUND REPLY NOT FOUND"
          echo ""
          echo "  Current run : $_AR_RUN_ID"
          echo "  Expected    : $_AR_CANDIDATE"
          echo ""
          echo "  Next step:"
          echo "    generate simulated reply:"
          echo "      ./examples/pilot/run_pilot.sh --simulate-inbound"
          echo "    or provide the file explicitly:"
          echo "      ./examples/pilot/run_pilot.sh --inspect-inbound-reply <file>"
          echo ""
          exit 1
        fi
      else
        echo ""
        echo "INSPECT INBOUND REPLY — USAGE"
        echo ""
        echo "  ./examples/pilot/run_pilot.sh --inspect-inbound-reply <file>"
        echo ""
        echo "Example:"
        echo "  ./examples/pilot/run_pilot.sh --inspect-inbound-reply inbound/lab_reply_<run-id>.json"
        echo ""
        echo "error: --inspect-inbound-reply requires a file argument" >&2
        exit 1
      fi
    else
      echo ""
      echo "INSPECT INBOUND REPLY — USAGE"
      echo ""
      echo "  ./examples/pilot/run_pilot.sh --inspect-inbound-reply <file>"
      echo ""
      echo "Example:"
      echo "  ./examples/pilot/run_pilot.sh --inspect-inbound-reply inbound/lab_reply_<run-id>.json"
      echo ""
      echo "error: --inspect-inbound-reply requires a file argument" >&2
      exit 1
    fi
  fi

  echo ""
  echo -e "${BOLD}PostCAD — Inbound Reply Inspection${RESET}"
  echo "  ────────────────────────────────────────"
  echo ""

  ARTIFACT_NAME=$(basename "$INBOUND_FILE")
  echo "  Artifact : $ARTIFACT_NAME"
  echo ""

  # Check file exists
  if [[ ! -f "$INBOUND_FILE" ]]; then
    echo -e "  ${RED}reply not readable${RESET}"
    echo "  Reason: file not found: $INBOUND_FILE"
    echo ""
    exit 1
  fi

  # Check valid JSON
  if ! python3 -c "import json; json.load(open('$INBOUND_FILE'))" 2>/dev/null; then
    echo -e "  ${RED}reply not readable${RESET}"
    echo "  Reason: not valid JSON: $INBOUND_FILE"
    echo ""
    exit 1
  fi

  # Extract fields
  INS_RECEIPT_HASH=$(python3 -c "
import json
try:
    d = json.load(open('$INBOUND_FILE'))
    print(d.get('receipt_hash', ''))
except: print('')
" 2>/dev/null || echo "")

  INS_CASE_ID=$(python3 -c "
import json
try:
    d = json.load(open('$INBOUND_FILE'))
    print(d.get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")

  INS_LAB_ID=$(python3 -c "
import json
try:
    d = json.load(open('$INBOUND_FILE'))
    print(d.get('lab_id', ''))
except: print('')
" 2>/dev/null || echo "")

  INS_STATUS=$(python3 -c "
import json
try:
    d = json.load(open('$INBOUND_FILE'))
    print(d.get('status', ''))
except: print('')
" 2>/dev/null || echo "")

  INS_ACK_AT=$(python3 -c "
import json
try:
    d = json.load(open('$INBOUND_FILE'))
    print(d.get('lab_acknowledged_at', ''))
except: print('')
" 2>/dev/null || echo "")

  INS_NOTES=$(python3 -c "
import json
try:
    d = json.load(open('$INBOUND_FILE'))
    print(d.get('notes', ''))
except: print('')
" 2>/dev/null || echo "")

  # Print field summary
  [[ -n "$INS_CASE_ID"      ]] && echo "  Case ID          : $INS_CASE_ID"      || echo "  Case ID          : not present"
  [[ -n "$INS_RECEIPT_HASH" ]] && echo "  Receipt hash     : $INS_RECEIPT_HASH" || echo "  Receipt hash     : not present"
  [[ -n "$INS_LAB_ID"       ]] && echo "  Lab ID           : $INS_LAB_ID"       || echo "  Lab ID           : not present"
  [[ -n "$INS_STATUS"       ]] && echo "  Status           : $INS_STATUS"       || echo "  Status           : not present"
  [[ -n "$INS_ACK_AT"       ]] && echo "  Acknowledged at  : $INS_ACK_AT"       || echo "  Acknowledged at  : not present"
  [[ -n "$INS_NOTES"        ]] && echo "  Notes            : $INS_NOTES"        || echo "  Notes            : not present"
  echo ""

  # Required field checklist
  echo "  Required fields:"
  [[ -n "$INS_RECEIPT_HASH" ]] && echo -e "    ${GREEN}present${RESET}  receipt_hash"        || echo -e "    ${RED}MISSING${RESET}  receipt_hash"
  [[ -n "$INS_LAB_ID"       ]] && echo -e "    ${GREEN}present${RESET}  lab_id"              || echo -e "    ${RED}MISSING${RESET}  lab_id"
  [[ -n "$INS_STATUS"       ]] && echo -e "    ${GREEN}present${RESET}  status"              || echo -e "    ${RED}MISSING${RESET}  status"
  [[ -n "$INS_ACK_AT"       ]] && echo -e "    ${GREEN}present${RESET}  lab_acknowledged_at" || echo -e "    ${RED}MISSING${RESET}  lab_acknowledged_at"
  echo ""

  # Determine overall result
  MISSING_FIELDS=""
  [[ -z "$INS_RECEIPT_HASH" ]] && MISSING_FIELDS="${MISSING_FIELDS} receipt_hash"
  [[ -z "$INS_LAB_ID"       ]] && MISSING_FIELDS="${MISSING_FIELDS} lab_id"
  [[ -z "$INS_STATUS"       ]] && MISSING_FIELDS="${MISSING_FIELDS} status"
  [[ -z "$INS_ACK_AT"       ]] && MISSING_FIELDS="${MISSING_FIELDS} lab_acknowledged_at"

  if [[ -z "$MISSING_FIELDS" ]]; then
    echo -e "  ${GREEN}reply structurally readable${RESET}"
    echo ""
    echo "  Next step — verify and generate decision record:"
    echo "    ./examples/pilot/verify.sh --inbound $INBOUND_FILE --bundle ${SCRIPT_DIR}"
    echo ""
    exit 0
  else
    echo -e "  ${RED}reply missing required field(s):${RESET}${MISSING_FIELDS}"
    echo ""
    exit 1
  fi
fi

# ── Mode: export dispatch ─────────────────────────────────────────────────────

if [[ "${1:-}" == "--export-dispatch" ]]; then

  RECEIPT="${SCRIPT_DIR}/receipt.json"
  DISPATCH_PKT="${SCRIPT_DIR}/export_packet.json"

  # Resolve run_id and receipt_hash from current receipt if available
  ED_RUN_ID=""
  ED_RECEIPT_HASH=""
  ED_DISPATCH_ID=""

  if [[ -f "$RECEIPT" ]]; then
    ED_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${RECEIPT}').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
    ED_RECEIPT_HASH=$(grep -o '"receipt_hash": *"[^"]*"' "$RECEIPT" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
    ED_RUN_ID="${ED_CASE_ID:-${ED_RECEIPT_HASH:0:12}}"
  fi

  if [[ -f "$DISPATCH_PKT" && -s "$DISPATCH_PKT" ]]; then
    ED_DISPATCH_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${DISPATCH_PKT}').read())
    print(d.get('dispatch_id', ''))
except: print('')
" 2>/dev/null || echo "")
  fi

  # Determine failure reason
  ED_FAIL_REASON=""
  if [[ ! -f "$RECEIPT" ]]; then
    ED_FAIL_REASON="no_receipt"
  elif [[ ! -f "$DISPATCH_PKT" || ! -s "$DISPATCH_PKT" ]]; then
    ED_FAIL_REASON="no_dispatch_packet"
  fi

  echo ""
  echo "PostCAD — Dispatch Export"
  echo "  ────────────────────────────────────────"
  echo ""

  if [[ -z "$ED_FAIL_REASON" ]]; then
    [[ -n "$ED_RUN_ID"      ]] && echo "  Run ID      : $ED_RUN_ID"
    [[ -n "$ED_DISPATCH_ID" ]] && echo "  Dispatch ID : $ED_DISPATCH_ID"
    echo "  File        : $DISPATCH_PKT"
    echo ""
    echo "  ════════════════════════════════════════"
    echo "  DISPATCH EXPORT READY"
    echo "  ════════════════════════════════════════"
    [[ -n "$ED_RUN_ID" ]] && echo "  Run ID  : $ED_RUN_ID"
    echo "  File    : $DISPATCH_PKT"
    echo "  Result  : dispatch packet exported"
    echo "  Next    : send packet to manufacturer / lab contact"
    echo "  ════════════════════════════════════════"
    echo ""
    exit 0
  else
    echo "  ════════════════════════════════════════"
    echo "  DISPATCH EXPORT FAILED"
    echo "  ════════════════════════════════════════"
    echo "  Result  : dispatch export failed"
    case "$ED_FAIL_REASON" in
      no_receipt)
        echo "  Reason  : no current pilot run found"
        echo "  Next    : generate or load a current pilot run before exporting"
        echo ""
        echo "DISPATCH EXPORT — PRECONDITION NOT MET"
        echo ""
        echo "A valid pilot run was not detected."
        echo ""
        echo "Recommended steps:"
        echo "  1  generate pilot bundle"
        echo "  2  verify inbound reply"
        echo "  3  export dispatch packet"
        ;;
      no_dispatch_packet)
        echo "  Reason  : dispatch packet not present"
        echo "  Next    : verify the current route before exporting dispatch"
        echo "            approve dispatch via reviewer shell first:"
        echo "              cargo run -p postcad-service"
        echo "              # then open http://localhost:8080/reviewer"
        ;;
      *)
        echo "  Reason  : export precondition not met"
        echo "  Next    : confirm the pilot bundle and current artifacts are present"
        ;;
    esac
    echo "  ════════════════════════════════════════"
    echo ""
    exit 1
  fi
fi

# ── Mode: artifact index ──────────────────────────────────────────────────────

if [[ "${1:-}" == "--artifact-index" ]]; then

  # Resolve current run context from receipt.json if present
  AI_RUN_ID=""
  AI_RECEIPT_HASH=""

  if [[ -f "${SCRIPT_DIR}/receipt.json" ]]; then
    AI_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${SCRIPT_DIR}/receipt.json').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
    AI_RECEIPT_HASH=$(grep -o '"receipt_hash": *"[^"]*"' "${SCRIPT_DIR}/receipt.json" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
    AI_RUN_ID="${AI_CASE_ID:-${AI_RECEIPT_HASH:0:12}}"
  fi

  echo ""
  echo "PostCAD — Pilot Artifact Index"
  echo "════════════════════════════════════════════════════════════"
  echo ""

  if [[ -n "$AI_RUN_ID" ]]; then
    echo "Current run : $AI_RUN_ID"
    echo ""
  fi

  echo "Pilot bundle"
  echo "  receipt.json       ${SCRIPT_DIR}/receipt.json"
  echo "  export_packet.json ${SCRIPT_DIR}/export_packet.json"
  echo ""

  echo "Inbound replies"
  echo "  directory  ${SCRIPT_DIR}/inbound/"
  if [[ -n "$AI_RUN_ID" ]]; then
    echo "  current    ${SCRIPT_DIR}/inbound/lab_reply_${AI_RUN_ID}.json"
  else
    echo "  pattern    ${SCRIPT_DIR}/inbound/lab_reply_<run-id>.json"
  fi
  echo ""

  echo "Outbound packages"
  echo "  directory  ${SCRIPT_DIR}/outbound/"
  if [[ -n "$AI_RUN_ID" ]]; then
    echo "  current    ${SCRIPT_DIR}/outbound/lab_trial_${AI_RUN_ID}/"
  else
    echo "  pattern    ${SCRIPT_DIR}/outbound/lab_trial_<run-id>/"
  fi
  echo ""

  echo "Decision records"
  echo "  directory  ${SCRIPT_DIR}/reports/"
  if [[ -n "$AI_RUN_ID" ]]; then
    echo "  ledger     ${SCRIPT_DIR}/reports/ledger_${AI_RUN_ID}.txt"
  else
    echo "  pattern    ${SCRIPT_DIR}/reports/ledger_<run-id>.txt"
  fi
  echo ""

  echo "Verification"
  if [[ -n "$AI_RUN_ID" ]]; then
    echo "  command    ./examples/pilot/verify.sh --inbound ${SCRIPT_DIR}/inbound/lab_reply_${AI_RUN_ID}.json --bundle ${SCRIPT_DIR}"
  else
    echo "  command    ./examples/pilot/verify.sh --inbound inbound/lab_reply_<run-id>.json --bundle examples/pilot"
  fi
  echo ""

  echo "────────────────────────────────────────"
  echo "Operator flow reminder"
  echo ""
  echo "  1. inspect inbound reply  — ./examples/pilot/run_pilot.sh --inspect-inbound-reply inbound/lab_reply_<run-id>.json"
  echo "  2. verify inbound reply   — ./examples/pilot/verify.sh --inbound inbound/lab_reply_<run-id>.json --bundle examples/pilot"
  echo "  3. export dispatch packet — ./examples/pilot/run_pilot.sh --export-dispatch"
  echo ""
  echo "════════════════════════════════════════════════════════════"
  echo ""
  exit 0
fi

# ── Mode: run summary ────────────────────────────────────────────────────────

if [[ "${1:-}" == "--run-summary" ]]; then

  # Resolve run context from receipt.json if present
  RS_RUN_ID="not detected"
  RS_RECEIPT_HASH=""

  if [[ -f "${SCRIPT_DIR}/receipt.json" ]]; then
    RS_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${SCRIPT_DIR}/receipt.json').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
    RS_RECEIPT_HASH=$(grep -o '"receipt_hash": *"[^"]*"' "${SCRIPT_DIR}/receipt.json" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
    _RS_ID="${RS_CASE_ID:-${RS_RECEIPT_HASH:0:12}}"
    [[ -n "$_RS_ID" ]] && RS_RUN_ID="$_RS_ID"
  fi

  # Detect inbound reply (any lab_reply_*.json in inbound/)
  RS_INBOUND_FILE=""
  if [[ -n "$(ls "${SCRIPT_DIR}/inbound/lab_reply_"*.json 2>/dev/null | head -1)" ]]; then
    RS_INBOUND_FILE=$(ls "${SCRIPT_DIR}/inbound/lab_reply_"*.json 2>/dev/null | head -1)
  fi

  # Detect verification decision record
  RS_DECISION_FILE=""
  if [[ -n "$(ls "${SCRIPT_DIR}/reports/decision_"*.txt 2>/dev/null | head -1)" ]]; then
    RS_DECISION_FILE=$(ls "${SCRIPT_DIR}/reports/decision_"*.txt 2>/dev/null | head -1)
  fi

  echo ""
  echo "PostCAD — Pilot Run Summary"
  echo "════════════════════════════════════════════════════════════"
  echo ""
  echo "RUN CONTEXT"
  echo ""
  echo "  Run ID : $RS_RUN_ID"
  echo ""

  echo "ARTIFACT STATUS"
  echo ""

  # receipt.json
  if [[ -f "${SCRIPT_DIR}/receipt.json" ]]; then
    echo "  receipt.json          present    ${SCRIPT_DIR}/receipt.json"
  else
    echo "  receipt.json          missing    ${SCRIPT_DIR}/receipt.json"
  fi

  # inbound lab reply
  if [[ -n "$RS_INBOUND_FILE" ]]; then
    echo "  inbound lab reply     present    $RS_INBOUND_FILE"
  elif [[ "$RS_RUN_ID" != "not detected" ]]; then
    echo "  inbound lab reply     missing    ${SCRIPT_DIR}/inbound/lab_reply_${RS_RUN_ID}.json"
  else
    echo "  inbound lab reply     not yet generated"
  fi

  # verification result
  if [[ -n "$RS_DECISION_FILE" ]]; then
    echo "  verification result   present    $RS_DECISION_FILE"
  else
    echo "  verification result   not yet generated"
  fi

  # dispatch packet
  if [[ -f "${SCRIPT_DIR}/export_packet.json" ]]; then
    echo "  dispatch packet       present    ${SCRIPT_DIR}/export_packet.json"
  else
    echo "  dispatch packet       not yet generated"
  fi

  echo ""

  echo "NEXT OPERATOR ACTION"
  echo ""

  if [[ ! -f "${SCRIPT_DIR}/receipt.json" ]]; then
    echo "  generate pilot bundle"
    echo "    ./examples/pilot/run_pilot.sh"
  elif [[ -z "$RS_INBOUND_FILE" ]]; then
    echo "  inspect inbound lab reply"
    echo "    ./examples/pilot/run_pilot.sh --inspect-inbound-reply inbound/lab_reply_<run-id>.json"
  elif [[ -z "$RS_DECISION_FILE" ]]; then
    echo "  verify inbound reply"
    echo "    ./examples/pilot/verify.sh --inbound $RS_INBOUND_FILE --bundle ${SCRIPT_DIR}"
  else
    echo "  export dispatch packet"
    echo "    ./examples/pilot/run_pilot.sh --export-dispatch"
  fi

  echo ""

  echo "OPERATOR COMMAND HINTS"
  echo ""
  echo "  --quickstart      quick reference for the most common commands"
  echo "  --artifact-index  artifact map for the current run"
  echo "  --walkthrough     full 4-step pilot workflow guide"
  echo ""
  echo "════════════════════════════════════════════════════════════"
  echo ""
  exit 0
fi

# ── Mode: trace view ─────────────────────────────────────────────────────────

if [[ "${1:-}" == "--trace-view" ]]; then

  # Resolve run_id from receipt.json if present
  TV_RUN_ID="not detected"
  if [[ -f "${SCRIPT_DIR}/receipt.json" ]]; then
    TV_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${SCRIPT_DIR}/receipt.json').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
    TV_RECEIPT_HASH=$(grep -o '"receipt_hash": *"[^"]*"' "${SCRIPT_DIR}/receipt.json" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
    _TV_ID="${TV_CASE_ID:-${TV_RECEIPT_HASH:0:12}}"
    [[ -n "$_TV_ID" ]] && TV_RUN_ID="$_TV_ID"
  fi

  # Detect events from filesystem artifacts
  TV_RECEIPT=""
  TV_INBOUND=""
  TV_VERIFICATION=""
  TV_DISPATCH=""

  [[ -f "${SCRIPT_DIR}/receipt.json" ]] && TV_RECEIPT="detected"

  if [[ "$TV_RUN_ID" != "not detected" ]]; then
    [[ -n "$(ls "${SCRIPT_DIR}/inbound/lab_reply_"*.json 2>/dev/null | head -1)" ]] && TV_INBOUND="detected"
  else
    [[ -n "$(ls "${SCRIPT_DIR}/inbound/lab_reply_"*.json 2>/dev/null | head -1)" ]] && TV_INBOUND="detected"
  fi

  [[ -n "$(ls "${SCRIPT_DIR}/reports/decision_"*.txt 2>/dev/null | head -1)" ]] && TV_VERIFICATION="detected"

  [[ -f "${SCRIPT_DIR}/export_packet.json" && -s "${SCRIPT_DIR}/export_packet.json" ]] && TV_DISPATCH="detected"

  echo ""
  echo "PostCAD — Pilot Trace View"
  echo "════════════════════════════════════════════════════════════"
  echo ""
  echo "Run ID : $TV_RUN_ID"
  echo ""

  if [[ -n "$TV_RECEIPT" ]]; then
    echo "  1  route decision generated       detected"
    echo "  2  receipt recorded               detected"
  else
    echo "  1  route decision generated       not yet observed"
    echo "  2  receipt recorded               not yet observed"
  fi

  if [[ -n "$TV_INBOUND" ]]; then
    echo "  3  inbound lab reply detected      detected"
  else
    echo "  3  inbound lab reply detected      not yet observed"
  fi

  if [[ -n "$TV_VERIFICATION" ]]; then
    echo "  4  verification step available     detected"
  else
    echo "  4  verification step available     not yet observed"
  fi

  if [[ -n "$TV_DISPATCH" ]]; then
    echo "  5  dispatch export available       detected"
  else
    echo "  5  dispatch export available       not yet observed"
  fi

  echo ""
  echo "════════════════════════════════════════════════════════════"
  echo ""
  exit 0
fi

# ── Mode: run fingerprint ────────────────────────────────────────────────────

if [[ "${1:-}" == "--run-fingerprint" ]]; then

  # Resolve run context
  RF_RUN_ID="not detected"
  if [[ -f "${SCRIPT_DIR}/receipt.json" ]]; then
    RF_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${SCRIPT_DIR}/receipt.json').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
    RF_RECEIPT_HASH=$(grep -o '"receipt_hash": *"[^"]*"' "${SCRIPT_DIR}/receipt.json" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
    _RF_ID="${RF_CASE_ID:-${RF_RECEIPT_HASH:0:12}}"
    [[ -n "$_RF_ID" ]] && RF_RUN_ID="$_RF_ID"
  fi

  # Collect artifact files that exist
  RF_RECEIPT_FILE="${SCRIPT_DIR}/receipt.json"
  RF_INBOUND_FILE=""
  RF_VERIFICATION_FILE=""
  RF_DISPATCH_FILE="${SCRIPT_DIR}/export_packet.json"

  if [[ "$RF_RUN_ID" != "not detected" ]]; then
    _RF_INBOUND="${SCRIPT_DIR}/inbound/lab_reply_${RF_RUN_ID}.json"
    [[ -f "$_RF_INBOUND" ]] && RF_INBOUND_FILE="$_RF_INBOUND"
    _RF_VERIFY=$(ls "${SCRIPT_DIR}/reports/decision_"*.txt 2>/dev/null | head -1 || true)
    [[ -n "$_RF_VERIFY" ]] && RF_VERIFICATION_FILE="$_RF_VERIFY"
  fi

  # Compute fingerprint from available artifact files
  RF_FINGERPRINT=""
  if [[ -f "$RF_RECEIPT_FILE" ]]; then
    RF_FINGERPRINT=$(python3 -c "
import hashlib, os
files = [
    '${RF_RECEIPT_FILE}',
    '${RF_INBOUND_FILE}',
    '${RF_VERIFICATION_FILE}',
    '${RF_DISPATCH_FILE}',
]
h = hashlib.sha256()
for f in files:
    if f and os.path.isfile(f):
        h.update(open(f, 'rb').read())
print(h.hexdigest())
" 2>/dev/null || echo "")
  fi

  echo ""
  echo "POSTCAD RUN FINGERPRINT"
  echo "════════════════════════════════════════════════════════════"
  echo ""
  echo "RUN CONTEXT"
  if [[ -f "$RF_RECEIPT_FILE" ]]; then
    echo "  Run ID       : $RF_RUN_ID"
    echo "  Receipt path : examples/pilot/receipt.json"
  else
    echo "  Run ID       : not detected"
  fi
  echo ""
  echo "FINGERPRINT COMPONENTS"
  if [[ -f "$RF_RECEIPT_FILE" ]];           then echo "  receipt.json                  included"; else echo "  receipt.json                  not present"; fi
  if [[ -n "$RF_INBOUND_FILE" ]];           then echo "  inbound reply                 included"; else echo "  inbound reply                 not present"; fi
  if [[ -n "$RF_VERIFICATION_FILE" ]];      then echo "  verification decision         included"; else echo "  verification decision         not present"; fi
  if [[ -f "$RF_DISPATCH_FILE" ]];          then echo "  dispatch packet               included"; else echo "  dispatch packet               not present"; fi
  echo ""
  if [[ -n "$RF_FINGERPRINT" ]]; then
    echo "FINGERPRINT"
    echo "  Run fingerprint : $RF_FINGERPRINT"
  else
    echo "FINGERPRINT"
    echo "  Run fingerprint : not available — generate a pilot bundle first"
  fi
  echo ""
  echo "WHY THIS MATTERS"
  echo "  - stable identifier for the workflow run"
  echo "  - derived from protocol artifacts"
  echo "  - useful for logs, tracing, and audits"
  echo ""
  echo "HOW TO USE"
  echo "  ./examples/pilot/run_pilot.sh --trace-view"
  echo "  ./examples/pilot/run_pilot.sh --protocol-chain"
  echo "  ./examples/pilot/run_pilot.sh --run-summary"
  echo ""
  echo "════════════════════════════════════════════════════════════"
  echo ""
  exit 0
fi

# ── Mode: lab entrypoint ─────────────────────────────────────────────────────

if [[ "${1:-}" == "--lab-entrypoint" ]]; then

  # Resolve run context
  LE_RUN_ID="not detected"
  if [[ -f "${SCRIPT_DIR}/receipt.json" ]]; then
    LE_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${SCRIPT_DIR}/receipt.json').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
    LE_RECEIPT_HASH=$(grep -o '"receipt_hash": *"[^"]*"' "${SCRIPT_DIR}/receipt.json" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
    _LE_ID="${LE_CASE_ID:-${LE_RECEIPT_HASH:0:12}}"
    [[ -n "$_LE_ID" ]] && LE_RUN_ID="$_LE_ID"
  fi

  echo ""
  echo "POSTCAD LAB ENTRYPOINT"
  echo "════════════════════════════════════════════════════════════"
  echo ""
  echo "WHAT THE LAB RECEIVES"
  echo "  - routed case context"
  echo "  - routing receipt"
  echo "  - inbound reply expectation"
  echo "  - dispatch-ready handoff artifact"
  echo ""
  echo "WHAT THE LAB IS EXPECTED TO DO"
  echo "  - review the routed case"
  echo "  - return a structured inbound reply"
  echo "  - participate in a verifiable workflow"
  echo "  - rely on dispatch packet after verified state"
  echo ""
  echo "WHY THIS MATTERS TO A LAB"
  echo "  - clearer case handoff"
  echo "  - fewer ambiguous workflow states"
  echo "  - verifiable reply path"
  echo "  - audit-ready execution handoff"
  echo ""
  echo "WHAT TO LOOK AT FIRST"
  echo "  ./examples/pilot/run_pilot.sh --demo-surface"
  echo "  ./examples/pilot/run_pilot.sh --artifact-index"
  echo "  ./examples/pilot/run_pilot.sh --trace-view"
  echo "  ./examples/pilot/run_pilot.sh --dispatch-packet"
  echo ""
  echo "WHAT EACH COMMAND SHOWS"
  echo "  --demo-surface     end-to-end pilot narrative and what the operator sees"
  echo "  --artifact-index   map of every artifact location in the current workflow"
  echo "  --trace-view       detection status of each workflow event artifact"
  echo "  --dispatch-packet  dispatch packet as execution-side protocol artifact"
  echo ""
  echo "CURRENT RUN CONTEXT"
  echo "  Run ID : $LE_RUN_ID"
  echo ""
  echo "LAB INTERPRETATION"
  echo "  - structured handoff"
  echo "  - verifiable reply"
  echo "  - execution-ready dispatch"
  echo ""
  echo "════════════════════════════════════════════════════════════"
  echo ""
  exit 0
fi

# ── Mode: business entrypoint ────────────────────────────────────────────────

if [[ "${1:-}" == "--business-entrypoint" ]]; then

  # Resolve run context
  BE_RUN_ID="not detected"
  if [[ -f "${SCRIPT_DIR}/receipt.json" ]]; then
    BE_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${SCRIPT_DIR}/receipt.json').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
    BE_RECEIPT_HASH=$(grep -o '"receipt_hash": *"[^"]*"' "${SCRIPT_DIR}/receipt.json" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
    _BE_ID="${BE_CASE_ID:-${BE_RECEIPT_HASH:0:12}}"
    [[ -n "$_BE_ID" ]] && BE_RUN_ID="$_BE_ID"
  fi

  echo ""
  echo "POSTCAD BUSINESS ENTRYPOINT"
  echo "════════════════════════════════════════════════════════════"
  echo ""
  echo "WHAT THIS PILOT DOES"
  echo "  - routes a dental manufacturing case deterministically"
  echo "  - records a receipt for the routing decision"
  echo "  - accepts and verifies a lab reply"
  echo "  - exports a dispatch packet for execution handoff"
  echo ""
  echo "WHY IT MATTERS"
  echo "  - fewer workflow ambiguities"
  echo "  - verifiable handoff between clinic and lab"
  echo "  - audit-ready process"
  echo "  - clearer operational accountability"
  echo ""
  echo "WHAT TO LOOK AT FIRST"
  echo "  ./examples/pilot/run_pilot.sh --demo-surface"
  echo "  ./examples/pilot/run_pilot.sh --system-overview"
  echo "  ./examples/pilot/run_pilot.sh --run-summary"
  echo "  ./examples/pilot/run_pilot.sh --help-surface"
  echo ""
  echo "WHAT EACH COMMAND SHOWS"
  echo "  --demo-surface      end-to-end pilot narrative and what the operator sees"
  echo "  --system-overview   what PostCAD does and how the workflow fits together"
  echo "  --run-summary       current run state and recommended next operator action"
  echo "  --help-surface      all available pilot commands and when to use each"
  echo ""
  echo "CURRENT RUN CONTEXT"
  echo "  Run ID : $BE_RUN_ID"
  echo ""
  echo "WHY THIS IS STRATEGIC"
  echo "  - workflow infrastructure, not just software"
  echo "  - traceable handoff layer"
  echo "  - foundation for trusted routing between clinics and manufacturers"
  echo ""
  echo "════════════════════════════════════════════════════════════"
  echo ""
  exit 0
fi

# ── Mode: engineer entrypoint ────────────────────────────────────────────────

if [[ "${1:-}" == "--engineer-entrypoint" ]]; then

  # Resolve run context
  EE_RUN_ID="not detected"
  if [[ -f "${SCRIPT_DIR}/receipt.json" ]]; then
    EE_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${SCRIPT_DIR}/receipt.json').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
    EE_RECEIPT_HASH=$(grep -o '"receipt_hash": *"[^"]*"' "${SCRIPT_DIR}/receipt.json" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
    _EE_ID="${EE_CASE_ID:-${EE_RECEIPT_HASH:0:12}}"
    [[ -n "$_EE_ID" ]] && EE_RUN_ID="$_EE_ID"
  fi

  echo ""
  echo "POSTCAD ENGINEER ENTRYPOINT"
  echo "════════════════════════════════════════════════════════════"
  echo ""
  echo "WHAT TO LOOK AT FIRST"
  echo "  system overview  — what PostCAD is and how the workflow fits together"
  echo "  trace view       — live event trace for the current run"
  echo "  receipt replay   — receipt as the replayable routing commitment"
  echo "  dispatch packet  — execution-side handoff artifact"
  echo "  protocol chain   — ordered artifact chain from routing to execution"
  echo ""
  echo "RECOMMENDED ORDER"
  echo "  ./examples/pilot/run_pilot.sh --system-overview"
  echo "  ./examples/pilot/run_pilot.sh --trace-view"
  echo "  ./examples/pilot/run_pilot.sh --receipt-replay"
  echo "  ./examples/pilot/run_pilot.sh --dispatch-packet"
  echo "  ./examples/pilot/run_pilot.sh --protocol-chain"
  echo ""
  echo "WHAT EACH COMMAND SHOWS"
  echo "  --system-overview   PostCAD components, workflow, and system properties"
  echo "  --trace-view        detection status of each workflow event artifact"
  echo "  --receipt-replay    receipt as routing commitment and source of truth"
  echo "  --dispatch-packet   dispatch packet as execution-side protocol artifact"
  echo "  --protocol-chain    four-stage artifact chain with current state"
  echo ""
  echo "CURRENT RUN CONTEXT"
  echo "  Run ID : $EE_RUN_ID"
  echo ""
  echo "WHY THIS IS TECHNICALLY INTERESTING"
  echo "  - deterministic routing artifacts"
  echo "  - replayable receipt"
  echo "  - verifiable workflow chain"
  echo "  - audit-ready execution handoff"
  echo ""
  echo "════════════════════════════════════════════════════════════"
  echo ""
  exit 0
fi

# ── Mode: protocol chain surface ─────────────────────────────────────────────

if [[ "${1:-}" == "--protocol-chain" ]]; then

  # Resolve run context
  PC_RUN_ID="not detected"
  if [[ -f "${SCRIPT_DIR}/receipt.json" ]]; then
    PC_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${SCRIPT_DIR}/receipt.json').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
    PC_RECEIPT_HASH=$(grep -o '"receipt_hash": *"[^"]*"' "${SCRIPT_DIR}/receipt.json" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
    _PC_ID="${PC_CASE_ID:-${PC_RECEIPT_HASH:0:12}}"
    [[ -n "$_PC_ID" ]] && PC_RUN_ID="$_PC_ID"
  fi

  # Detect current state of each chain stage
  PC_RECEIPT="not yet observed"
  PC_INBOUND="not yet observed"
  PC_VERIFICATION="not yet observed"
  PC_DISPATCH="not yet observed"

  [[ -f "${SCRIPT_DIR}/receipt.json" ]] && PC_RECEIPT="detected"
  [[ -n "$(ls "${SCRIPT_DIR}/inbound/lab_reply_"*.json 2>/dev/null | head -1)" ]] && PC_INBOUND="detected"
  [[ -n "$(ls "${SCRIPT_DIR}/reports/decision_"*.txt 2>/dev/null | head -1)" ]] && PC_VERIFICATION="detected"
  [[ -f "${SCRIPT_DIR}/export_packet.json" ]] && PC_DISPATCH="detected"

  echo ""
  echo "POSTCAD PROTOCOL CHAIN"
  echo "════════════════════════════════════════════════════════════"
  echo ""
  echo "RUN CONTEXT"
  echo "  Run ID : $PC_RUN_ID"
  echo ""
  echo "CHAIN"
  echo "  1  receipt"
  echo "       routing commitment"
  echo "       source of truth for the run"
  echo ""
  echo "  2  inbound reply"
  echo "       lab response tied to the routed case"
  echo ""
  echo "  3  verification"
  echo "       checks inbound reply against protocol expectations"
  echo ""
  echo "  4  dispatch packet"
  echo "       execution-side handoff artifact after verified workflow state"
  echo ""
  echo "CURRENT STATE"
  echo "  1  receipt              $PC_RECEIPT"
  echo "  2  inbound reply        $PC_INBOUND"
  echo "  3  verification         $PC_VERIFICATION"
  echo "  4  dispatch packet      $PC_DISPATCH"
  echo ""
  echo "WHY THIS MATTERS"
  echo "  - deterministic chain of workflow artifacts"
  echo "  - verifiable transition from routing to execution"
  echo "  - audit-ready protocol path"
  echo ""
  echo "HOW TO USE"
  echo "  ./examples/pilot/run_pilot.sh --receipt-replay"
  echo "  ./examples/pilot/run_pilot.sh --dispatch-packet"
  echo "  ./examples/pilot/run_pilot.sh --trace-view"
  echo "  ./examples/pilot/run_pilot.sh --run-summary"
  echo ""
  echo "ENGINEER INTERPRETATION"
  echo "  - deterministic"
  echo "  - chained"
  echo "  - audit-ready"
  echo ""
  echo "════════════════════════════════════════════════════════════"
  echo ""
  exit 0
fi

# ── Mode: dispatch packet surface ────────────────────────────────────────────

if [[ "${1:-}" == "--dispatch-packet" ]]; then

  # Resolve run context from receipt.json if present
  DP_RUN_ID="not detected"
  if [[ -f "${SCRIPT_DIR}/receipt.json" ]]; then
    DP_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${SCRIPT_DIR}/receipt.json').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
    DP_RECEIPT_HASH=$(grep -o '"receipt_hash": *"[^"]*"' "${SCRIPT_DIR}/receipt.json" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
    _DP_ID="${DP_CASE_ID:-${DP_RECEIPT_HASH:0:12}}"
    [[ -n "$_DP_ID" ]] && DP_RUN_ID="$_DP_ID"
  fi

  # Detect dispatch artifact
  DP_ARTIFACT_LINE="  not yet generated"
  if [[ -f "${SCRIPT_DIR}/export_packet.json" ]]; then
    DP_ARTIFACT_LINE="  examples/pilot/export_packet.json"
  fi

  echo ""
  echo "POSTCAD DISPATCH PACKET"
  echo "════════════════════════════════════════════════════════════"
  echo ""
  echo "RUN CONTEXT"
  echo "  Run ID : $DP_RUN_ID"
  echo ""
  echo "DISPATCH ARTIFACT"
  echo "  The dispatch packet represents the execution-side handoff artifact."
  echo "  It follows verified workflow state."
  echo "  It is what the operator exports when the run is ready."
  echo ""
  echo "  Dispatch packet : $DP_ARTIFACT_LINE"
  echo ""
  echo "WHY IT MATTERS"
  echo "  - binds execution to verified workflow state"
  echo "  - audit-ready handoff artifact"
  echo "  - operator-facing execution checkpoint"
  echo ""
  echo "HOW TO USE"
  echo "  ./examples/pilot/run_pilot.sh --export-dispatch"
  echo "  ./examples/pilot/run_pilot.sh --run-summary"
  echo "  ./examples/pilot/run_pilot.sh --trace-view"
  echo "  ./examples/pilot/run_pilot.sh --artifact-index"
  echo ""
  echo "ENGINEER INTERPRETATION"
  echo "  - deterministic"
  echo "  - exportable"
  echo "  - audit-ready"
  echo ""
  if [[ ! -f "${SCRIPT_DIR}/export_packet.json" ]]; then
    echo "  Export dispatch packet after verification to generate the artifact."
  fi
  echo ""
  echo "════════════════════════════════════════════════════════════"
  echo ""
  exit 0
fi

# ── Mode: receipt replay surface ─────────────────────────────────────────────

if [[ "${1:-}" == "--receipt-replay" ]]; then

  # Resolve run context from receipt.json if present
  RR_RUN_ID="not detected"
  RR_RECEIPT_STATUS="examples/pilot/receipt.json (not found)"
  if [[ -f "${SCRIPT_DIR}/receipt.json" ]]; then
    RR_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${SCRIPT_DIR}/receipt.json').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
    RR_RECEIPT_HASH=$(grep -o '"receipt_hash": *"[^"]*"' "${SCRIPT_DIR}/receipt.json" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
    _RR_ID="${RR_CASE_ID:-${RR_RECEIPT_HASH:0:12}}"
    [[ -n "$_RR_ID" ]] && RR_RUN_ID="$_RR_ID"
    RR_RECEIPT_STATUS="examples/pilot/receipt.json"
  fi

  echo ""
  echo "POSTCAD RECEIPT REPLAY"
  echo "════════════════════════════════════════════════════════════"
  echo ""
  echo "RUN CONTEXT"
  echo "  Run ID  : $RR_RUN_ID"
  echo "  Receipt : $RR_RECEIPT_STATUS"
  echo ""
  echo "WHAT THE RECEIPT COMMITS"
  echo "  - selected routing candidate for the case"
  echo "  - deterministic routing outcome bound to the input set"
  echo "  - receipt hash as the verification source of truth"
  echo ""
  echo "REPLAY IDEA"
  echo "  The receipt represents the routing commitment for the case."
  echo "  Replay verification checks that the stored routing result"
  echo "  is consistent with the recorded receipt."
  echo "  The receipt is the artifact an operator or engineer"
  echo "  should inspect first."
  echo ""
  echo "HOW TO USE"
  echo "  ./examples/pilot/run_pilot.sh"
  echo "  ./examples/pilot/verify.sh"
  echo "  ./examples/pilot/run_pilot.sh --run-summary"
  echo "  ./examples/pilot/run_pilot.sh --trace-view"
  echo ""
  echo "ENGINEER INTERPRETATION"
  echo "  - deterministic"
  echo "  - replayable"
  echo "  - audit-ready"
  echo ""
  if [[ -f "${SCRIPT_DIR}/receipt.json" ]]; then
    echo "  Current receipt detected for replay-oriented inspection."
  else
    echo "  Generate a pilot bundle first to create a receipt."
  fi
  echo ""
  echo "════════════════════════════════════════════════════════════"
  echo ""
  exit 0
fi

# ── Mode: simulate inbound reply ─────────────────────────────────────────────

if [[ "${1:-}" == "--simulate-inbound" ]]; then

  TEMPLATE="${SCRIPT_DIR}/testdata/lab_reply_simulated.json"

  if [[ ! -f "$TEMPLATE" ]]; then
    echo ""
    echo "error: simulator template not found: $TEMPLATE" >&2
    exit 1
  fi

  # Resolve run_id from receipt.json if present
  SIM_RUN_ID=""
  if [[ -f "${SCRIPT_DIR}/receipt.json" ]]; then
    SIM_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${SCRIPT_DIR}/receipt.json').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
    SIM_RECEIPT_HASH=$(grep -o '"receipt_hash": *"[^"]*"' "${SCRIPT_DIR}/receipt.json" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
    SIM_RUN_ID="${SIM_CASE_ID:-${SIM_RECEIPT_HASH:0:12}}"
  fi

  mkdir -p "${SCRIPT_DIR}/inbound"

  if [[ -n "$SIM_RUN_ID" ]]; then
    DEST="${SCRIPT_DIR}/inbound/lab_reply_${SIM_RUN_ID}.json"
  else
    DEST="${SCRIPT_DIR}/inbound/lab_reply_simulated.json"
  fi

  cp "$TEMPLATE" "$DEST"

  echo ""
  echo "SIMULATED LAB REPLY GENERATED"
  echo ""
  echo "File:"
  echo "  $DEST"
  echo ""
  echo "Next step:"
  echo "  inspect inbound reply"
  echo "    ./examples/pilot/run_pilot.sh --inspect-inbound-reply $DEST"
  echo "  verify inbound reply"
  echo "    ./examples/pilot/verify.sh --inbound $DEST --bundle ${SCRIPT_DIR}"
  echo ""
  exit 0
fi

# ── Mode: demo surface ───────────────────────────────────────────────────────

if [[ "${1:-}" == "--demo-surface" ]]; then
  echo ""
  echo "POSTCAD PILOT DEMO"
  echo "════════════════════════════════════════════════════════════"
  echo ""
  echo "PostCAD is a deterministic routing and verification layer for dental CAD"
  echo "manufacturing workflows."
  echo ""
  echo "END-TO-END FLOW"
  echo ""
  echo "  1. generate pilot bundle    route the dental case, write receipt.json"
  echo "  2. inspect inbound reply    check required fields before verification"
  echo "  3. verify inbound reply     bind the reply to the current run cryptographically"
  echo "  4. export dispatch packet   confirm dispatch ready, record next action"
  echo ""
  echo "WHAT THE OPERATOR SEES"
  echo ""
  echo "  receipt.json          cryptographic commitment to the routing decision"
  echo "  inbound lab reply     lab acknowledgement returned for the routed case"
  echo "  verification outcome  operator decision record bound to the receipt"
  echo "  dispatch packet       approved dispatch commitment (export_packet.json)"
  echo ""
  echo "WHY THIS MATTERS"
  echo ""
  echo "  - Deterministic routing: same case inputs always produce the same receipt."
  echo "  - Verifiable inbound replies: lab replies are cryptographically bound to the run."
  echo "  - Audit-ready dispatch workflow: every decision carries a receipt hash."
  echo ""
  echo "TRY IT"
  echo ""
  echo "  ./examples/pilot/run_pilot.sh --system-overview"
  echo "  ./examples/pilot/run_pilot.sh --quickstart"
  echo "  ./examples/pilot/run_pilot.sh --run-summary"
  echo "  ./examples/pilot/run_pilot.sh --help-surface"
  echo ""
  echo "════════════════════════════════════════════════════════════"
  echo ""
  exit 0
fi

# ── Mode: system overview ─────────────────────────────────────────────────────

if [[ "${1:-}" == "--system-overview" ]]; then
  echo ""
  echo "POSTCAD PILOT SYSTEM OVERVIEW"
  echo "════════════════════════════════════════════════════════════"
  echo ""
  echo "PostCAD is a deterministic routing and verification layer for dental CAD"
  echo "manufacturing workflows. Every routing decision is cryptographically"
  echo "committed to a receipt that can be independently verified."
  echo ""
  echo "CORE IDEA"
  echo ""
  echo "  A dental CAD design produces a case."
  echo "  PostCAD generates a routing decision — which manufacturer receives the case."
  echo "  A receipt records the decision as a cryptographic commitment."
  echo "  The lab returns a reply that can be verified against the receipt."
  echo "  A dispatch packet commits the approved workflow for audit."
  echo ""
  echo "PILOT WORKFLOW"
  echo ""
  echo "  1. Generate pilot bundle   — route the case, write receipt.json"
  echo "  2. Inspect inbound reply   — check required fields before verification"
  echo "  3. Verify inbound reply    — bind the reply to the current run cryptographically"
  echo "  4. Export dispatch packet  — confirm dispatch ready, record next action"
  echo ""
  echo "KEY ARTIFACTS"
  echo ""
  echo "  receipt.json          routing decision committed as a cryptographic receipt"
  echo "  inbound lab reply     lab acknowledgement of the routing case"
  echo "  verification result   operator decision record bound to the receipt"
  echo "  dispatch packet       approved dispatch commitment (export_packet.json)"
  echo ""
  echo "OPERATOR TOOLS"
  echo ""
  echo "  --quickstart          minimum command sheet for the pilot workflow"
  echo "  --walkthrough         full 4-step pilot workflow guide"
  echo "  --artifact-index      artifact map — where every file lives"
  echo "  --help-surface        consolidated overview of all operator modes"
  echo ""
  echo "PROPERTIES"
  echo ""
  echo "  - Deterministic routing: same case inputs always produce the same receipt."
  echo "  - Verifiable inbound replies: lab replies are cryptographically bound to the run."
  echo "  - Audit-ready dispatch packets: every decision carries a receipt hash and reason code."
  echo ""
  echo "════════════════════════════════════════════════════════════"
  echo ""
  exit 0
fi

# ── Mode: consolidated help surface ──────────────────────────────────────────

if [[ "${1:-}" == "--help-surface" ]]; then
  echo ""
  echo "PostCAD Pilot — Operator Mode Reference"
  echo "════════════════════════════════════════════════════════════"
  echo ""
  echo "Available modes"
  echo ""
  echo "(default)"
  echo "  ./examples/pilot/run_pilot.sh"
  echo "  Purpose : Generate a pilot bundle — route the dental case and write receipt.json."
  echo "  Use when: starting a new pilot run."
  echo ""
  echo "--inspect-inbound-reply <file>"
  echo "  ./examples/pilot/run_pilot.sh --inspect-inbound-reply inbound/lab_reply_<run-id>.json"
  echo "  Purpose : Check that all required fields are present in a returned lab reply."
  echo "  Use when: you have received a lab reply and want to inspect it before verification."
  echo ""
  echo "--export-dispatch"
  echo "  ./examples/pilot/run_pilot.sh --export-dispatch"
  echo "  Purpose : Confirm the dispatch packet is ready and show the next action."
  echo "  Use when: the routing receipt is ready and you want to check dispatch status."
  echo ""
  echo "--walkthrough"
  echo "  ./examples/pilot/run_pilot.sh --walkthrough"
  echo "  Purpose : Print the full 4-step pilot workflow guide."
  echo "  Use when: learning the pilot flow for the first time."
  echo ""
  echo "--artifact-index"
  echo "  ./examples/pilot/run_pilot.sh --artifact-index"
  echo "  Purpose : Print the artifact map for the current run — where every file lives."
  echo "  Use when: you want to see all artifact locations for the current run."
  echo ""
  echo "--quickstart"
  echo "  ./examples/pilot/run_pilot.sh --quickstart"
  echo "  Purpose : Print the minimum command sheet for the pilot workflow."
  echo "  Use when: you need a quick reference to the most common commands."
  echo ""
  echo "────────────────────────────────────────"
  echo "Recommended order"
  echo ""
  echo "  1. generate pilot bundle   ./examples/pilot/run_pilot.sh"
  echo "  2. inspect inbound reply   ./examples/pilot/run_pilot.sh --inspect-inbound-reply inbound/lab_reply_<run-id>.json"
  echo "  3. verify inbound reply    ./examples/pilot/verify.sh --inbound inbound/lab_reply_<run-id>.json --bundle examples/pilot"
  echo "  4. export dispatch packet  ./examples/pilot/run_pilot.sh --export-dispatch"
  echo ""
  echo "════════════════════════════════════════════════════════════"
  echo ""
  exit 0
fi

# ── Mode: quickstart command sheet ───────────────────────────────────────────

if [[ "${1:-}" == "--quickstart" ]]; then
  echo ""
  echo "PostCAD Pilot — Quickstart Command Sheet"
  echo "════════════════════════════════════════════════════════════"
  echo ""
  echo "Generate pilot bundle"
  echo "  ./examples/pilot/run_pilot.sh"
  echo "  Routes the dental case and writes a cryptographic receipt."
  echo ""
  echo "Inspect inbound lab reply"
  echo "  ./examples/pilot/run_pilot.sh --inspect-inbound-reply inbound/lab_reply_<run-id>.json"
  echo "  Checks that all required fields are present before verification."
  echo ""
  echo "Verify inbound reply"
  echo "  ./examples/pilot/verify.sh --inbound inbound/lab_reply_<run-id>.json --bundle examples/pilot"
  echo "  Cryptographically binds the reply to the current run and records a decision."
  echo ""
  echo "Export dispatch packet"
  echo "  ./examples/pilot/run_pilot.sh --export-dispatch"
  echo "  Confirms the dispatch packet is ready and tells you the next action."
  echo ""
  echo "Show artifact index"
  echo "  ./examples/pilot/run_pilot.sh --artifact-index"
  echo "  Prints the artifact map for the current run — where every file lives."
  echo ""
  echo "Show walkthrough"
  echo "  ./examples/pilot/run_pilot.sh --walkthrough"
  echo "  Prints the full 4-step pilot workflow guide."
  echo ""
  echo "════════════════════════════════════════════════════════════"
  echo ""
  exit 0
fi

# ── Mode: operator walkthrough ────────────────────────────────────────────────

# ── Mode: audit receipt view ──────────────────────────────────────────────────

if [[ "${1:-}" == "--audit-receipt-view" ]]; then

  ARV_RECEIPT="${SCRIPT_DIR}/receipt.json"

  echo ""
  echo "POSTCAD AUDIT RECEIPT VIEW"
  echo "════════════════════════════════════════════════════════════"
  echo ""
  echo "RECEIPT STATUS"
  if [[ -f "$ARV_RECEIPT" ]]; then
    echo "  - receipt detected"
    echo "  - receipt path : examples/pilot/receipt.json"
    echo ""

    ARV_RUN_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${ARV_RECEIPT}').read())
    print(d.get('routing_input', {}).get('case_id', '(not present)'))
except: print('(not present)')
" 2>/dev/null || echo "(not present)")

    ARV_DECISION=$(grep -o '"outcome": *"[^"]*"' "$ARV_RECEIPT" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
    [[ -z "$ARV_DECISION" ]] && ARV_DECISION="(not present)"

    ARV_JURISDICTION=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${ARV_RECEIPT}').read())
    print(d.get('routing_input', {}).get('jurisdiction', '(not present)'))
except: print('(not present)')
" 2>/dev/null || echo "(not present)")

    ARV_MFR_ID=$(grep -o '"selected_candidate_id": *"[^"]*"' "$ARV_RECEIPT" | head -1 | sed 's/.*: *"\(.*\)"/\1/')
    [[ -z "$ARV_MFR_ID" ]] && ARV_MFR_ID="(not present)"

    ARV_PROFILE=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${ARV_RECEIPT}').read())
    print(d.get('routing_input', {}).get('routing_policy', '(not present)'))
except: print('(not present)')
" 2>/dev/null || echo "(not present)")

    echo "RECEIPT SUMMARY"
    echo "  - run id          : $ARV_RUN_ID"
    echo "  - decision        : $ARV_DECISION"
    echo "  - jurisdiction    : $ARV_JURISDICTION"
    echo "  - manufacturer id : $ARV_MFR_ID"
    echo "  - profile         : $ARV_PROFILE"
    echo ""
  else
    echo "  - no receipt detected"
    echo "  - receipt path : examples/pilot/receipt.json (not present)"
    echo ""
    echo "RECEIPT SUMMARY"
    echo "  No receipt available. Run ./examples/pilot/run_pilot.sh to generate one."
    echo ""
  fi

  echo "WHY THIS MATTERS"
  echo "  - operator can inspect the canonical audit receipt"
  echo "  - read-only surface"
  echo "  - useful for handoff / review / audit trace"
  echo ""
  echo "HOW TO USE"
  echo "  ./examples/pilot/run_pilot.sh --run-fingerprint"
  echo "  ./examples/pilot/run_pilot.sh --protocol-chain"
  echo "  ./examples/pilot/run_pilot.sh --lab-entrypoint"
  echo ""
  echo "════════════════════════════════════════════════════════════"
  echo ""
  exit 0
fi

if [[ "${1:-}" == "--walkthrough" ]]; then
  echo ""
  echo "POSTCAD PILOT WALKTHROUGH"
  echo "════════════════════════════════════════════════════════════"
  echo ""
  echo "Step 1 — Generate pilot bundle"
  echo "  Command : ./examples/pilot/run_pilot.sh"
  echo "  Creates : examples/pilot/receipt.json"
  echo "  What    : Routes the dental case against the manufacturer registry."
  echo "            A cryptographic receipt is written and self-verified."
  echo "            The receipt hash is the verification source of truth for this run."
  echo ""
  echo "Step 2 — Inspect inbound lab reply"
  echo "  Command : ./examples/pilot/run_pilot.sh --inspect-inbound-reply inbound/lab_reply_<run-id>.json"
  echo "  Reads   : inbound/lab_reply_<run-id>.json"
  echo "  What    : Checks that all required fields are present in the returned reply"
  echo "            before running full cryptographic verification."
  echo "            Prints: reply structurally readable / reply missing required field(s)"
  echo ""
  echo "Step 3 — Verify inbound reply"
  echo "  Command : ./examples/pilot/verify.sh --inbound inbound/lab_reply_<run-id>.json --bundle examples/pilot"
  echo "  Reads   : inbound/lab_reply_<run-id>.json + examples/pilot/receipt.json"
  echo "  What    : Cryptographically binds the inbound reply to the current run."
  echo "            Writes a decision record to examples/pilot/reports/."
  echo "            Prints: VERIFICATION PASSED / VERIFICATION FAILED"
  echo ""
  echo "Step 4 — Export dispatch packet"
  echo "  Command : ./examples/pilot/run_pilot.sh --export-lab-trial-package"
  echo "  Creates : examples/pilot/outbound/lab_trial_<run-id>/"
  echo "  What    : Packages the routing receipt and lab reply template"
  echo "            into a sendable directory for the external lab."
  echo "            Includes operator instructions, message kit, and receipt."
  echo ""
  echo "════════════════════════════════════════════════════════════"
  echo "Run this walkthrough at any time:"
  echo "  ./examples/pilot/run_pilot.sh --walkthrough"
  echo ""
  exit 0
fi

# ── Unknown argument handler ──────────────────────────────────────────────────

if [[ -n "${1:-}" ]]; then
  echo ""
  echo "UNKNOWN COMMAND"
  echo ""
  echo "Use:"
  echo "  ./examples/pilot/run_pilot.sh --help-surface"
  echo ""
  exit 1
fi

echo "PostCAD Protocol v1 — Pilot Workflow"
echo "======================================"
echo ""

# ── Step 1: Build ──────────────────────────────────────────────────────────────
if [[ ! -x "$BIN" ]]; then
  echo "Building postcad-cli..."
  cargo build --bin postcad-cli --quiet --manifest-path "${REPO_ROOT}/Cargo.toml"
  echo ""
fi

# ── Step 2: Route ──────────────────────────────────────────────────────────────
echo "Routing case..."
echo ""

RECEIPT_JSON=$("$BIN" route-case-from-registry --json \
  --case    "${SCRIPT_DIR}/case.json" \
  --registry "${SCRIPT_DIR}/registry_snapshot.json" \
  --config  "${SCRIPT_DIR}/config.json")

# Exit non-zero propagates via set -e; reaching here means route + self-verify passed.

# ── Step 3: Save receipt ───────────────────────────────────────────────────────
echo "$RECEIPT_JSON" > "${SCRIPT_DIR}/receipt.json"

# ── Step 3a: Append ledger entry ───────────────────────────────────────────────
PILOT_CASE_ID=$(python3 -c "
import json, sys
try:
    d = json.loads(open('${SCRIPT_DIR}/receipt.json').read())
    print(d.get('routing_input', {}).get('case_id', ''))
except: print('')
" 2>/dev/null || echo "")
PILOT_RECEIPT_HASH=$(echo "$RECEIPT_JSON" | grep -o '"receipt_hash": *"[^"]*"' | head -1 | sed 's/.*: *"\(.*\)"/\1/')
PILOT_RUN_ID="${PILOT_CASE_ID:-${PILOT_RECEIPT_HASH:0:12}}"
if [[ -n "$PILOT_RUN_ID" ]]; then
  LEDGER_FILE="$REPORTS_DIR/ledger_${PILOT_RUN_ID}.txt"
  _append_ledger "$LEDGER_FILE" "outbound_bundle_created" "$PILOT_RUN_ID" "recorded" "${SCRIPT_DIR}/receipt.json"
fi

# ── Step 4: Print result ───────────────────────────────────────────────────────
# Extract key fields with pure-shell JSON parsing (no jq dependency).
OUTCOME=$(echo "$RECEIPT_JSON"       | grep -o '"outcome": *"[^"]*"'             | head -1 | sed 's/.*: *"\(.*\)"/\1/')
CANDIDATE=$(echo "$RECEIPT_JSON"     | grep -o '"selected_candidate_id": *"[^"]*"' | head -1 | sed 's/.*: *"\(.*\)"/\1/')
RECEIPT_HASH=$(echo "$RECEIPT_JSON"  | grep -o '"receipt_hash": *"[^"]*"'        | head -1 | sed 's/.*: *"\(.*\)"/\1/')
KERNEL=$(echo "$RECEIPT_JSON"        | grep -o '"routing_kernel_version": *"[^"]*"' | head -1 | sed 's/.*: *"\(.*\)"/\1/')

echo "Result:               ${OUTCOME}"
echo "Selected candidate:   ${CANDIDATE}"
echo "Receipt hash:         ${RECEIPT_HASH}"
echo "Kernel version:       ${KERNEL}"
echo ""
echo "Receipt written to:   examples/pilot/receipt.json"
if [[ -n "${PILOT_RUN_ID:-}" ]]; then
  echo "Ledger:               ${LEDGER_FILE:-}"
fi
echo ""
echo "Verification: OK"
echo ""
echo "  (Self-verification ran inside the routing step."
echo "   The receipt would not have been emitted if it failed to verify.)"
echo ""
echo "──────────────────────────────────────────────────────────────────────"
echo "Artifacts"
echo ""
echo "  examples/pilot/receipt.json      routing decision audit record"
echo "                                   the receipt hash above is the"
echo "                                   verification source of truth"
echo ""
echo "Next steps"
echo ""
echo "  1. Inspect the receipt (examples/pilot/receipt.json)."
echo "     Confirm: outcome=routed, selected_candidate_id, receipt_hash."
echo ""
echo "  2. Independent receipt verification (CLI, no service required):"
echo "       ./examples/pilot/verify.sh"
echo ""
echo "  3. Human review and dispatch (requires the HTTP service):"
echo "       cargo run -p postcad-service"
echo "       # then open http://localhost:8080/reviewer"
echo ""
echo "  4. Package a run bundle and simulate a lab response:"
echo "       ./examples/pilot/package_run.sh"
echo "       ./examples/pilot/lab_simulator.sh pilot_bundle lab_response.json"
echo ""
echo "  5. Verify the inbound lab response against the current run:"
echo "       ./examples/pilot/verify.sh --inbound lab_response.json --bundle pilot_bundle"
echo ""
echo "  6. Generate an external handoff pack for real lab trials:"
echo "       ./examples/pilot/lab_simulator.sh --handoff-pack handoff/ --bundle pilot_bundle"
echo "──────────────────────────────────────────────────────────────────────"
