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
    echo "error: --inspect-inbound-reply requires a file argument" >&2
    echo "  usage: ./examples/pilot/run_pilot.sh --inspect-inbound-reply <reply_file>" >&2
    exit 1
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
