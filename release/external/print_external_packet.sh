#!/usr/bin/env bash
# PostCAD pilot — print external delivery packet summary.
#
# Usage (from repo root or release/external/ directory):
#   ./release/external/print_external_packet.sh
#
# Read-only. Does not start the service, generate evidence, create tags,
# or modify any file. Prints the pilot label, key external packet
# resources, recommended review order, and scope reminders.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

PILOT_LABEL="pilot-local-v1"

ok()      { echo "  [OK]  $*"; }
missing() { echo "  [--]  MISSING: $*"; }
hr()      { echo ""; echo "── $* ──────────────────────────────────────────────"; }

# ── header ─────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  PostCAD Pilot — External Delivery Packet"
echo "══════════════════════════════════════════"
echo "  Pilot label : $PILOT_LABEL"
echo "  Repo root   : $REPO_ROOT"
echo "  Local pilot only. Not a production system."
echo ""

# ── git state ───────────────────────────────────────────────────────────────

hr "Git state"
branch=$(git -C "$REPO_ROOT" branch --show-current 2>/dev/null || echo "(unknown)")
head=$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || echo "(unknown)")
clean=$(git -C "$REPO_ROOT" status --short 2>/dev/null || echo "?")
echo "  Branch : $branch"
echo "  HEAD   : $head"
[[ -z "$clean" ]] && echo "  Status : clean" || echo "  Status : DIRTY (run 'git status' for details)"

# ── external packet resources ─────────────────────────────────────────────────

hr "External packet resources"
for f in \
  "release/external/README.md" \
  "release/external/EXTERNAL_DELIVERY_OVERVIEW.md" \
  "release/external/EXTERNAL_REVIEW_PATH.md" \
  "release/external/EXTERNAL_BOUNDARIES.md"; do
  [[ -f "$REPO_ROOT/$f" ]] && ok "$f" || missing "$f"
done

# ── key review surfaces ────────────────────────────────────────────────────────

hr "Key review surfaces"
for f in \
  "release/RELEASE_NOTES_PILOT.md" \
  "release/review/SYSTEM_OVERVIEW.md" \
  "release/review/ARTIFACT_GUIDE.md" \
  "release/review/BOUNDARIES.md" \
  "release/evidence/README.md" \
  "release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md" \
  "release/handoff/FIRST_HOUR_GUIDE.md" \
  "release/selfcheck/run_release_selfcheck.sh" \
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
  echo "  [--]  release/evidence/current/ — not present (generate with start_pilot.sh + generate_evidence_bundle.sh)"
fi

# ── recommended external review order ────────────────────────────────────────

hr "Recommended external review order"
cat <<'ORDER'

  Step 1  cat release/RELEASE_NOTES_PILOT.md
          → What surfaces are included? What is frozen?

  Step 2  cat release/review/SYSTEM_OVERVIEW.md
          → What does the system do?

  Step 3  ./release/selfcheck/run_release_selfcheck.sh
          → Is the package structurally complete?

  Step 4  cat release/review/ARTIFACT_GUIDE.md
          → What does each evidence file contain?

  Step 5  cat release/evidence/current/summary.txt
          cat release/evidence/current/06_verify.json
          → Did the pilot run succeed? Is verification VERIFIED?

  Step 6  cat release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md
          ./release/acceptance/print_acceptance_summary.sh
          → Do the outputs meet the acceptance criteria?

  Step 7  cat release/review/BOUNDARIES.md
          → What is frozen and what is out of scope?

  Full path with stop conditions: release/review-trace/REVIEW_TRACE.md

ORDER

# ── scope reminders ───────────────────────────────────────────────────────────

hr "Scope reminders"
cat <<'SCOPE'

  - Local pilot only — service runs on localhost:8080
  - No production deployment, no hosted service
  - No certification or regulatory claims
  - Protocol postcad-v1, kernel postcad-routing-v1, schema 1 — all frozen
  - One canonical pilot case: f1000001-0000-0000-0000-000000000001 (DE, zirconia crown)
  - Deterministic receipt hash: 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb

  Full boundaries: release/external/EXTERNAL_BOUNDARIES.md

SCOPE

# ── done ──────────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  External packet printed."
echo "  Start with EXTERNAL_DELIVERY_OVERVIEW.md"
echo "  then follow EXTERNAL_REVIEW_PATH.md."
echo "══════════════════════════════════════════"
echo ""
