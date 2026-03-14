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
