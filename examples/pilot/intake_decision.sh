#!/usr/bin/env bash
# intake_decision.sh — print an intake decision summary for a pilot bundle.
#
# Usage:
#   ./examples/pilot/intake_decision.sh [BUNDLE_DIR]
#
#   BUNDLE_DIR  pilot bundle directory  (default: pilot_bundle)
#
# Reads artifact files and produces a three-verdict intake decision:
#   accepted for review    — all required artifacts present and valid
#   requires clarification — bundle present but one or more artifacts degraded
#   rejected               — critical artifacts missing or failed
#
# Exit codes:
#   0  decision printed
#   1  bundle directory not found

set -euo pipefail

BUNDLE_DIR="${1:-pilot_bundle}"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'
BOLD='\033[1m'; RESET='\033[0m'
hr()  { echo "  ────────────────────────────────────────"; }
kv()  { printf "  ${CYAN}%-30s${RESET} %s\n" "$1" "$2"; }
ok()  { echo -e "  ${GREEN}✓${RESET}  $*"; }
no()  { echo -e "  ${RED}✗${RESET}  $*"; }
unk() { echo -e "  ${YELLOW}?${RESET}  $*"; }

field() {
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

if [[ ! -d "$BUNDLE_DIR" ]]; then
  echo -e "${RED}[decision] Bundle directory not found: $BUNDLE_DIR${RESET}" >&2
  exit 1
fi

echo ""
echo -e "${BOLD}  PostCAD Pilot Bundle — Intake Decision${RESET}"
hr

# ── artifact checks ────────────────────────────────────────────────────────────
SCORE=0      # 0–5, one point per passing check
CRITICAL=0   # critical failures that force rejection

# 1. route artifact
echo ""
echo -e "  ${BOLD}Artifact Checklist${RESET}"
SRC=""
[[ -f "$BUNDLE_DIR/route.json"   ]] && SRC="$BUNDLE_DIR/route.json"
[[ -f "$BUNDLE_DIR/receipt.json" ]] && SRC="${SRC:-$BUNDLE_DIR/receipt.json}"

if [[ -n "$SRC" && -s "$SRC" ]]; then
  OUTCOME=$(field "$SRC" "outcome")
  SELECTED=$(field "$SRC" "selected_candidate_id")
  RHASH=$(field "$SRC" "receipt_hash")
  ok "Route artifact present (outcome: ${OUTCOME:-—}, selected: ${SELECTED:-(none)})"
  SCORE=$((SCORE + 1))
else
  no "Route artifact missing"
  CRITICAL=$((CRITICAL + 1))
fi

# 2. receipt
if [[ -f "$BUNDLE_DIR/receipt.json" && -s "$BUNDLE_DIR/receipt.json" ]]; then
  ok "Receipt present"
  SCORE=$((SCORE + 1))
else
  no "Receipt missing"
  CRITICAL=$((CRITICAL + 1))
fi

# 3. verification
if [[ -f "$BUNDLE_DIR/verification.json" && -s "$BUNDLE_DIR/verification.json" ]]; then
  VRESULT=$(field "$BUNDLE_DIR/verification.json" "result")
  if [[ "$VRESULT" == "VERIFIED" ]]; then
    ok "Verification: VERIFIED"
    SCORE=$((SCORE + 1))
  else
    unk "Verification present but result: ${VRESULT:-unknown}"
  fi
else
  no "Verification artifact missing"
  CRITICAL=$((CRITICAL + 1))
fi

# 4. reproducibility
if [[ -f "$BUNDLE_DIR/reproducibility.json" && -s "$BUNDLE_DIR/reproducibility.json" ]]; then
  RSTATUS=$(field "$BUNDLE_DIR/reproducibility.json" "status")
  RMATCH=$(field "$BUNDLE_DIR/reproducibility.json" "match")
  if [[ "$RSTATUS" == "reproducible" || "$RMATCH" == "True" || "$RMATCH" == "true" ]]; then
    ok "Reproducibility: passed"
    SCORE=$((SCORE + 1))
  elif [[ "$RSTATUS" == "mismatch" ]]; then
    unk "Reproducibility: mismatch detected"
  else
    unk "Reproducibility: status unknown (${RSTATUS:-—})"
  fi
else
  unk "Reproducibility artifact missing"
fi

# 5. dispatch packet
if [[ -f "$BUNDLE_DIR/export_packet.json" && -s "$BUNDLE_DIR/export_packet.json" ]]; then
  DSTATUS=$(field "$BUNDLE_DIR/export_packet.json" "status")
  if [[ -n "$DSTATUS" && "$DSTATUS" != "{}" ]]; then
    ok "Dispatch packet present (status: $DSTATUS)"
    SCORE=$((SCORE + 1))
  else
    unk "Dispatch packet file present but appears empty or unexported"
  fi
else
  no "Dispatch packet missing"
  CRITICAL=$((CRITICAL + 1))
fi

# ── receipt hash summary ───────────────────────────────────────────────────────
echo ""
echo -e "  ${BOLD}Receipt Hash${RESET}"
if [[ -n "${RHASH:-}" ]]; then
  kv "Hash:" "$RHASH"
else
  kv "Hash:" "—"
fi

# ── intake decision ────────────────────────────────────────────────────────────
echo ""
echo -e "  ${BOLD}Intake Decision${RESET}"
hr

if [[ $CRITICAL -gt 0 ]]; then
  DECISION="rejected"
  DECISION_COLOR="$RED"
  REASON="$CRITICAL critical artifact(s) missing — bundle cannot be accepted for review."
elif [[ $SCORE -ge 4 ]]; then
  DECISION="accepted for review"
  DECISION_COLOR="$GREEN"
  REASON="All critical artifacts present and valid. Bundle is suitable for external review."
else
  DECISION="requires clarification"
  DECISION_COLOR="$YELLOW"
  REASON="Bundle is present but one or more artifacts are degraded or missing. Operator should clarify before review."
fi

echo ""
echo -e "  ${DECISION_COLOR}${BOLD}  $DECISION${RESET}"
echo ""
echo -e "  $REASON"
echo ""
hr
echo ""
