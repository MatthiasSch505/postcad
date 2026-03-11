#!/usr/bin/env bash
# PostCAD pilot acceptance — print structural pre-check summary.
#
# Usage (from repo root or release/ directory):
#   ./release/acceptance/print_acceptance_summary.sh
#
# Read-only. Does not start or stop the service.
# Does not modify any files or declare the system accepted.
# Prints what is present or missing for each acceptance section.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
EVIDENCE="$REPO_ROOT/release/evidence/current"
REVIEW="$REPO_ROOT/release/review"
RECEIPT_HASH="0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb"

ok()      { echo "  [OK]  $*"; }
missing() { echo "  [--]  MISSING: $*"; }
found()   { echo "  [OK]  found: $*"; }
hr()      { echo ""; echo "── $* ──────────────────────────────────────────────"; }

echo ""
echo "══════════════════════════════════════════"
echo "  PostCAD Pilot — Acceptance Pre-Check"
echo "══════════════════════════════════════════"
echo "  Repo root : $REPO_ROOT"
echo "  Evidence  : $EVIDENCE"
echo ""

# ── Section 2: operator scripts ───────────────────────────────────────────────

hr "Section 2 — Operator scripts"
for f in \
  "release/reset_pilot_data.sh" \
  "release/start_pilot.sh" \
  "release/smoke_test.sh" \
  "release/generate_evidence_bundle.sh" \
  "demo/run_demo.sh"; do
  p="$REPO_ROOT/$f"
  if [[ -x "$p" ]]; then
    ok "$f (executable)"
  elif [[ -f "$p" ]]; then
    missing "$f (exists but not executable)"
  else
    missing "$f"
  fi
done

for f in \
  "examples/pilot/case.json" \
  "examples/pilot/registry_snapshot.json" \
  "examples/pilot/config.json"; do
  [[ -f "$REPO_ROOT/$f" ]] && ok "$f" || missing "$f"
done

# ── Section 3: API response files ─────────────────────────────────────────────

hr "Section 3 — API response files"
for f in \
  "01_health.json" \
  "02_store_case.json" \
  "03_route_case.json" \
  "04_receipt.json" \
  "05_dispatch.json" \
  "06_verify.json" \
  "07_route_history.json"; do
  [[ -f "$EVIDENCE/$f" ]] && ok "$f" || missing "$f"
done

# ── Section 4: data artifacts ─────────────────────────────────────────────────

hr "Section 4 — Data artifacts"
for sub in cases receipts policies dispatch verification; do
  d="$EVIDENCE/data_artifacts/$sub"
  if [[ -d "$d" ]] && compgen -G "$d/*.json" > /dev/null 2>&1; then
    count=$(find "$d" -maxdepth 1 -name "*.json" | wc -l | tr -d ' ')
    ok "data_artifacts/$sub/ ($count file(s))"
  else
    missing "data_artifacts/$sub/"
  fi
done

# ── Section 5: evidence bundle context files ──────────────────────────────────

hr "Section 5 — Evidence bundle context"
for f in "summary.txt" "commands.txt" "git_head.txt"; do
  [[ -f "$EVIDENCE/$f" ]] && ok "$f" || missing "$f"
done
for f in "inputs/case.json" "inputs/registry_snapshot.json" "inputs/config.json"; do
  [[ -f "$EVIDENCE/$f" ]] && ok "$f" || missing "$f"
done

# quick summary.txt check
if [[ -f "$EVIDENCE/summary.txt" ]]; then
  if grep -q "All 7 steps passed" "$EVIDENCE/summary.txt"; then
    ok "summary.txt — contains 'All 7 steps passed.'"
  else
    missing "summary.txt — does NOT contain 'All 7 steps passed.'"
  fi
fi

# ── Section 6: review packet ──────────────────────────────────────────────────

hr "Section 6 — Review packet"
for f in \
  "README.md" \
  "SYSTEM_OVERVIEW.md" \
  "OPERATOR_FLOW.md" \
  "ARTIFACT_GUIDE.md" \
  "BOUNDARIES.md"; do
  [[ -f "$REVIEW/$f" ]] && ok "release/review/$f" || missing "release/review/$f"
done

# ── Section 7: canonical fixture hashes ──────────────────────────────────────

hr "Section 7 — Frozen boundaries"
if [[ -f "$EVIDENCE/04_receipt.json" ]]; then
  rh=$(python3 -c "
import json, sys
d = json.load(open(sys.argv[1]))
print(d.get('receipt_hash', ''))
" "$EVIDENCE/04_receipt.json" 2>/dev/null || echo "")
  if [[ "$rh" == "$RECEIPT_HASH" ]]; then
    ok "receipt_hash matches canonical value ($RECEIPT_HASH)"
  else
    missing "receipt_hash mismatch: got '$rh', expected '$RECEIPT_HASH'"
  fi

  kv=$(python3 -c "
import json, sys
d = json.load(open(sys.argv[1]))
print(d.get('routing_kernel_version', ''))
" "$EVIDENCE/04_receipt.json" 2>/dev/null || echo "")
  [[ "$kv" == "postcad-routing-v1" ]] \
    && ok "routing_kernel_version = $kv" \
    || missing "routing_kernel_version: got '$kv'"

  sv=$(python3 -c "
import json, sys
d = json.load(open(sys.argv[1]))
print(d.get('schema_version', ''))
" "$EVIDENCE/04_receipt.json" 2>/dev/null || echo "")
  [[ "$sv" == "1" ]] \
    && ok "schema_version = $sv" \
    || missing "schema_version: got '$sv'"
else
  missing "04_receipt.json not found — cannot check frozen values"
fi

# ── gitignore check ───────────────────────────────────────────────────────────

hr "Gitignore"
if grep -q "release/evidence/current" "$REPO_ROOT/.gitignore" 2>/dev/null; then
  ok ".gitignore excludes release/evidence/current/"
else
  missing ".gitignore does not exclude release/evidence/current/"
fi

# ── done ──────────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  Pre-check complete."
echo "  Review items marked [--] before accepting."
echo "  Use PILOT_ACCEPTANCE_CHECKLIST.md for full criteria."
echo "  Record findings in REVIEW_WORKSHEET.md."
echo "══════════════════════════════════════════"
echo ""
