#!/usr/bin/env bash
# PostCAD pilot — print readiness snapshot summary.
#
# Usage (from repo root or release/readiness/ directory):
#   ./release/readiness/print_readiness_snapshot.sh
#
# Read-only. Does not start the service, run tests, generate evidence,
# or modify any file. Prints the readiness surfaces, recommended review
# path, out-of-scope reminder, and existence checks for readiness resources.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

ok()      { echo "  [OK]  $*"; }
missing() { echo "  [--]  MISSING: $*"; }
hr()      { echo ""; echo "── $* ──────────────────────────────────────────────"; }

# ── header ─────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  PostCAD Pilot — Readiness Snapshot"
echo "══════════════════════════════════════════"
echo "  Repo root : $REPO_ROOT"
echo "  This is a description only, not a certification."
echo ""

# ── git state ───────────────────────────────────────────────────────────────

hr "Git state"
branch=$(git -C "$REPO_ROOT" branch --show-current 2>/dev/null || echo "(unknown)")
head=$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || echo "(unknown)")
clean=$(git -C "$REPO_ROOT" status --short 2>/dev/null || echo "?")
echo "  Branch : $branch"
echo "  HEAD   : $head"
[[ -z "$clean" ]] && echo "  Status : clean" || echo "  Status : DIRTY (run 'git status' for details)"

# ── release surfaces ──────────────────────────────────────────────────────────

hr "Release surfaces"
for f in \
  "release/README.md" \
  "release/INDEX.md" \
  "release/FREEZE_MANIFEST.md" \
  "release/start_pilot.sh" \
  "release/reset_pilot_data.sh" \
  "release/smoke_test.sh" \
  "release/generate_evidence_bundle.sh" \
  "demo/run_demo.sh" \
  "release/evidence/README.md" \
  "release/walkthrough/PILOT_WALKTHROUGH.md" \
  "release/review/SYSTEM_OVERVIEW.md" \
  "release/review/BOUNDARIES.md" \
  "release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md" \
  "release/handoff/FIRST_HOUR_GUIDE.md" \
  "release/selfcheck/run_release_selfcheck.sh" \
  "release/freeze/FROZEN_BOUNDARIES.md" \
  "release/review-trace/REVIEW_TRACE.md" \
  "release/readiness/READINESS_SNAPSHOT.md" \
  "release/readiness/OUT_OF_SCOPE.md" \
  "examples/pilot/case.json" \
  "examples/pilot/registry_snapshot.json" \
  "examples/pilot/config.json"; do
  [[ -f "$REPO_ROOT/$f" || -x "$REPO_ROOT/$f" ]] && ok "$f" || missing "$f"
done

# ── evidence current ─────────────────────────────────────────────────────────

evidence="$REPO_ROOT/release/evidence/current"
if [[ -d "$evidence" ]] && [[ -f "$evidence/summary.txt" ]]; then
  grep -q "All 7 steps passed" "$evidence/summary.txt" \
    && ok "release/evidence/current/ — All 7 steps passed" \
    || echo "  [--]  release/evidence/current/ — present but summary check failed"
else
  echo "  [--]  release/evidence/current/ — not present (generate with generate_evidence_bundle.sh)"
fi

# ── recommended review path ───────────────────────────────────────────────────

hr "Recommended review path (8 steps)"
cat <<'PATH'

  Step 1  cat release/INDEX.md
  Step 2  cat release/FREEZE_MANIFEST.md
  Step 3  ./release/selfcheck/run_release_selfcheck.sh
  Step 4  cat release/walkthrough/PILOT_WALKTHROUGH.md
  Step 5  cat release/review/SYSTEM_OVERVIEW.md
          cat release/review/OPERATOR_FLOW.md
          cat release/review/ARTIFACT_GUIDE.md
          cat release/review/BOUNDARIES.md
  Step 6  cat release/evidence/current/summary.txt
          cat release/evidence/current/06_verify.json
          cat release/evidence/current/04_receipt.json
  Step 7  ./release/acceptance/print_acceptance_summary.sh
  Step 8  ./release/handoff/print_handoff_index.sh

  Full trace with stop conditions: release/review-trace/REVIEW_TRACE.md

PATH

# ── frozen values reminder ────────────────────────────────────────────────────

hr "Frozen protocol values"
echo "  Protocol version   : postcad-v1"
echo "  Routing kernel     : postcad-routing-v1"
echo "  Receipt schema     : 1"
echo "  Receipt hash (det) : 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb"
echo "  Pilot case ID      : f1000001-0000-0000-0000-000000000001"
echo "  Full boundaries    : release/freeze/FROZEN_BOUNDARIES.md"

# ── out-of-scope reminder ─────────────────────────────────────────────────────

hr "Out of scope (summary)"
cat <<'OOS'

  - No protocol redesign
  - No routing redesign
  - No schema changes
  - No kernel changes
  - No certification or regulatory claims
  - No production deployment
  - No hosted or remote operations
  - No coverage beyond the single canonical pilot case

  Full list: release/readiness/OUT_OF_SCOPE.md

OOS

# ── done ──────────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  Readiness snapshot printed."
echo "  This is a description only."
echo "  See READINESS_SNAPSHOT.md for full detail."
echo "  See OUT_OF_SCOPE.md for explicit exclusions."
echo "══════════════════════════════════════════"
echo ""
