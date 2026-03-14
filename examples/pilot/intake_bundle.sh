#!/usr/bin/env bash
# intake_bundle.sh — simulate receiving-side intake of a PostCAD pilot bundle.
#
# Usage:
#   ./examples/pilot/intake_bundle.sh [BUNDLE_DIR]
#
#   BUNDLE_DIR  pilot bundle directory to intake  (default: pilot_bundle)
#
# The script validates the bundle using existing tooling, prints an intake
# summary, and exits non-zero if the bundle is incomplete or invalid.
#
# Exit codes:
#   0  bundle accepted for intake processing
#   1  bundle invalid, incomplete, or missing

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUNDLE_DIR="${1:-pilot_bundle}"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'
BOLD='\033[1m'; RESET='\033[0m'
info()  { echo -e "${GREEN}[intake]${RESET} $*"; }
fail()  { echo -e "${RED}[intake]${RESET} $*" >&2; }
warn()  { echo -e "${YELLOW}[intake]${RESET} $*"; }
hr()    { echo "  ────────────────────────────────────────"; }

echo ""
echo -e "${BOLD}  PostCAD Pilot Bundle — Lab Intake${RESET}"
hr
info "Bundle path : $BUNDLE_DIR"
info "Received at : $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo ""

# ── step 1: directory check ────────────────────────────────────────────────────
if [[ ! -d "$BUNDLE_DIR" ]]; then
  fail "Bundle directory not found: $BUNDLE_DIR"
  fail "Provide the path to a valid pilot_bundle directory."
  echo ""
  exit 1
fi
echo -e "  ${GREEN}✓${RESET} Bundle directory found"

# ── step 2: validate bundle using existing tooling ─────────────────────────────
VALIDATE_SCRIPT="$SCRIPT_DIR/validate_bundle.sh"
if [[ -x "$VALIDATE_SCRIPT" ]]; then
  echo ""
  info "Running bundle validation..."
  if "$VALIDATE_SCRIPT" "$BUNDLE_DIR"; then
    echo -e "  ${GREEN}✓${RESET} Validation passed"
  else
    echo ""
    fail "Bundle validation FAILED — intake rejected"
    exit 1
  fi
else
  warn "validate_bundle.sh not found — skipping automated validation"
fi

# ── step 3: presence check for intake-critical files ──────────────────────────
echo ""
info "Checking intake-critical artifacts..."
INTAKE_ERRORS=0
for f in receipt.json verification.json export_packet.json; do
  if [[ -f "$BUNDLE_DIR/$f" && -s "$BUNDLE_DIR/$f" ]]; then
    echo -e "  ${GREEN}✓${RESET} $f"
  else
    echo -e "  ${RED}✗${RESET} $f — missing or empty"
    INTAKE_ERRORS=$((INTAKE_ERRORS + 1))
  fi
done

if [[ $INTAKE_ERRORS -gt 0 ]]; then
  echo ""
  fail "$INTAKE_ERRORS intake-critical artifact(s) missing — intake cannot proceed"
  exit 1
fi

# ── step 4: intake summary ─────────────────────────────────────────────────────
echo ""
info "Generating intake summary..."
"$SCRIPT_DIR/intake_decision.sh" "$BUNDLE_DIR"
