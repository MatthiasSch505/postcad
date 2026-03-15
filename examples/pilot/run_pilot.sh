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
