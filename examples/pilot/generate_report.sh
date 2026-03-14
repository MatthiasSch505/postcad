#!/usr/bin/env bash
# generate_report.sh — generate a concise run report from a pilot bundle or run output directory.
#
# Usage:
#   ./examples/pilot/generate_report.sh [BUNDLE_DIR]
#
#   BUNDLE_DIR  bundle or run output directory  (default: pilot_bundle)
#
# Reads artifact files from the directory and prints a human-readable run report.
# Exit codes:
#   0  report generated
#   1  required artifacts missing

set -euo pipefail

BUNDLE_DIR="${1:-pilot_bundle}"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'
BOLD='\033[1m'; RESET='\033[0m'

field() {
  local file="$1" key="$2"
  if command -v python3 &>/dev/null; then
    python3 -c "
import json
try:
    d = json.load(open('$file'))
    keys = '$key'.split('.')
    for k in keys:
        d = d.get(k, '') if isinstance(d, dict) else ''
    print(d)
except: print('')
" 2>/dev/null || echo ""
  elif command -v jq &>/dev/null; then
    jq -r ".$key // \"\"" "$file" 2>/dev/null || echo ""
  else
    echo ""
  fi
}

hr() { echo "  ────────────────────────────────────────"; }
kv() { printf "  ${CYAN}%-26s${RESET} %s\n" "$1" "$2"; }
section() { echo -e "\n  ${BOLD}$*${RESET}"; hr; }

if [[ ! -d "$BUNDLE_DIR" ]]; then
  echo -e "${RED}[report] Bundle directory not found: $BUNDLE_DIR${RESET}" >&2
  exit 1
fi

echo ""
echo -e "${BOLD}  PostCAD Pilot Run Report${RESET}"
echo -e "  $BUNDLE_DIR"
hr

# ── route ──────────────────────────────────────────────────────────────────────
section "Route"
if [[ -f "$BUNDLE_DIR/route.json" || -f "$BUNDLE_DIR/receipt.json" ]]; then
  SRC="$BUNDLE_DIR/receipt.json"
  [[ -f "$BUNDLE_DIR/route.json" ]] && SRC="$BUNDLE_DIR/route.json"
  OUTCOME=$(field "$SRC" "outcome")
  SELECTED=$(field "$SRC" "selected_candidate_id")
  RECEIPT_HASH=$(field "$SRC" "receipt_hash")
  KVER=$(field "$SRC" "routing_kernel_version")
  CREATED=$(field "$SRC" "created_at")
  kv "Outcome:"       "${OUTCOME:-—}"
  kv "Selected:"      "${SELECTED:-(none — refused)}"
  kv "Kernel:"        "${KVER:-—}"
  kv "Created:"       "${CREATED:-—}"
  if [[ -n "$RECEIPT_HASH" ]]; then
    kv "Receipt hash:" "$RECEIPT_HASH"
  fi
else
  kv "Route:" "artifact not found"
fi

# ── verification ───────────────────────────────────────────────────────────────
section "Verification"
if [[ -f "$BUNDLE_DIR/verification.json" ]]; then
  VRESULT=$(field "$BUNDLE_DIR/verification.json" "result")
  if [[ "$VRESULT" == "VERIFIED" ]]; then
    kv "Result:" "${GREEN}VERIFIED ✓${RESET}"
  elif [[ -n "$VRESULT" ]]; then
    kv "Result:" "${RED}$VRESULT${RESET}"
  else
    kv "Result:" "—"
  fi
else
  kv "Status:" "${YELLOW}artifact not found${RESET}"
fi

# ── reproducibility ────────────────────────────────────────────────────────────
section "Reproducibility"
if [[ -f "$BUNDLE_DIR/reproducibility.json" ]]; then
  RSTATUS=$(field "$BUNDLE_DIR/reproducibility.json" "status")
  RMATCH=$(field "$BUNDLE_DIR/reproducibility.json" "match")
  if [[ "$RSTATUS" == "reproducible" || "$RMATCH" == "True" || "$RMATCH" == "true" ]]; then
    kv "Status:" "${GREEN}reproducible ✓${RESET}"
  elif [[ "$RSTATUS" == "mismatch" ]]; then
    kv "Status:" "${RED}mismatch ✗${RESET}"
  else
    kv "Status:" "${RSTATUS:-—}"
  fi
else
  kv "Status:" "${YELLOW}artifact not found${RESET}"
fi

# ── dispatch packet ────────────────────────────────────────────────────────────
section "Dispatch Packet"
if [[ -f "$BUNDLE_DIR/export_packet.json" ]]; then
  DSTATUS=$(field "$BUNDLE_DIR/export_packet.json" "status")
  DISPATCH_ID=$(field "$BUNDLE_DIR/export_packet.json" "dispatch_id")
  if [[ -n "$DSTATUS" && "$DSTATUS" != "{}" ]]; then
    kv "Status:"      "${GREEN}present ✓${RESET}"
    kv "Status field:" "$DSTATUS"
    [[ -n "$DISPATCH_ID" ]] && kv "Dispatch ID:" "$DISPATCH_ID"
  else
    kv "Status:" "${YELLOW}empty or not exported${RESET}"
  fi
else
  kv "Status:" "${YELLOW}artifact not found${RESET}"
fi

# ── bundle manifest ────────────────────────────────────────────────────────────
if [[ -f "$BUNDLE_DIR/bundle_manifest.json" ]]; then
  section "Bundle Manifest"
  BTIME=$(field "$BUNDLE_DIR/bundle_manifest.json" "created_at")
  kv "Created:" "${BTIME:-—}"
fi

# ── run summary ────────────────────────────────────────────────────────────────
if [[ -f "$BUNDLE_DIR/run_summary.json" ]]; then
  section "Run Summary"
  for k in outcome verification reproducibility; do
    val=$(field "$BUNDLE_DIR/run_summary.json" "$k")
    [[ -n "$val" ]] && kv "$k:" "$val"
  done
fi

echo ""
hr
echo -e "  Report complete."
echo ""
