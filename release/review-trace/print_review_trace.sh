#!/usr/bin/env bash
# PostCAD pilot — print the review trace order and key checks.
#
# Usage (from repo root or release/review-trace/ directory):
#   ./release/review-trace/print_review_trace.sh
#
# Read-only. Does not start the service, run tests, generate evidence,
# or modify any file. Prints the numbered review order, path/command
# for each step, key stop conditions, and existence checks for trace
# resources.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

ok()      { echo "  [OK]  $*"; }
missing() { echo "  [--]  MISSING: $*"; }
hr()      { echo ""; echo "── $* ──────────────────────────────────────────────"; }

# ── header ─────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  PostCAD Pilot — Review Trace"
echo "══════════════════════════════════════════"
echo "  Repo root : $REPO_ROOT"
echo ""

# ── git state ───────────────────────────────────────────────────────────────

hr "Git state"
branch=$(git -C "$REPO_ROOT" branch --show-current 2>/dev/null || echo "(unknown)")
head=$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || echo "(unknown)")
clean=$(git -C "$REPO_ROOT" status --short 2>/dev/null || echo "?")
echo "  Branch : $branch"
echo "  HEAD   : $head"
[[ -z "$clean" ]] && echo "  Status : clean" || echo "  Status : DIRTY (run 'git status' for details)"

# ── review order ────────────────────────────────────────────────────────────

hr "Review order"
cat <<'ORDER'

  Step 1 — Inspect release index
    cat release/INDEX.md
    → What: package surfaces, classifications, recommended paths

  Step 2 — Inspect freeze manifest
    cat release/FREEZE_MANIFEST.md
    → What: frozen protocol values, surface classifications

  Step 3 — Run structural self-check
    ./release/selfcheck/run_release_selfcheck.sh
    → What: all release files present and executable

  Step 4 — Inspect walkthrough
    cat release/walkthrough/PILOT_WALKTHROUGH.md
    → What: exact operator sequence, expected outputs, failure modes

  Step 5 — Inspect review packet
    cat release/review/SYSTEM_OVERVIEW.md
    cat release/review/OPERATOR_FLOW.md
    cat release/review/ARTIFACT_GUIDE.md
    cat release/review/BOUNDARIES.md
    → What: system architecture, scope, artifact fields, frozen boundaries

  Step 6 — Inspect evidence bundle
    cat release/evidence/current/summary.txt
    cat release/evidence/current/06_verify.json
    cat release/evidence/current/04_receipt.json
    → What: pilot run result, verification outcome, receipt hash

  Step 7 — Inspect acceptance bundle
    ./release/acceptance/print_acceptance_summary.sh
    cat release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md
    → What: 33-item acceptance criteria, structural pre-check

  Step 8 — Inspect handoff packet
    ./release/handoff/print_handoff_index.sh
    cat release/handoff/KNOWN_GOOD_STATE.md
    → What: package completeness, known-good state match

ORDER

# ── key stop conditions ──────────────────────────────────────────────────────

hr "Key stop conditions"
cat <<'STOPS'

  S1  Release path missing          → run selfcheck; restore missing files
  S2  Self-check Missing items > 0  → resolve [--] before continuing
  S3  Evidence absent/incomplete    → generate: start_pilot.sh + generate_evidence_bundle.sh
  S4  Receipt hash mismatch         → verify fixtures unmodified; rebuild; re-run
  S5  Verification not VERIFIED     → reset data; regenerate evidence cleanly
  S6  Review packet references missing paths  → update doc to match reality
  S7  Acceptance pre-check [--]     → resolve missing acceptance inputs
  S8  Handoff index [--]            → resolve missing handoff resources
  S9  Freeze manifest contradicted  → reconcile diverged surfaces

  Full stop conditions: release/review-trace/STOP_POINTS.md

STOPS

# ── trace resource checks ────────────────────────────────────────────────────

hr "Trace resources"
for f in \
  "release/INDEX.md" \
  "release/FREEZE_MANIFEST.md" \
  "release/selfcheck/run_release_selfcheck.sh" \
  "release/walkthrough/PILOT_WALKTHROUGH.md" \
  "release/review/README.md" \
  "release/review/SYSTEM_OVERVIEW.md" \
  "release/review/BOUNDARIES.md" \
  "release/evidence/README.md" \
  "release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md" \
  "release/acceptance/print_acceptance_summary.sh" \
  "release/handoff/print_handoff_index.sh" \
  "release/handoff/KNOWN_GOOD_STATE.md" \
  "release/review-trace/REVIEW_TRACE.md" \
  "release/review-trace/STOP_POINTS.md"; do
  [[ -f "$REPO_ROOT/$f" || -x "$REPO_ROOT/$f" ]] && ok "$f" || missing "$f"
done

# ── evidence current ─────────────────────────────────────────────────────────

evidence="$REPO_ROOT/release/evidence/current"
if [[ -d "$evidence" ]] && [[ -f "$evidence/summary.txt" ]]; then
  grep -q "All 7 steps passed" "$evidence/summary.txt" \
    && ok "release/evidence/current/ — All 7 steps passed" \
    || echo "  [--]  release/evidence/current/ — present but summary check failed (stop point S3)"
else
  echo "  [--]  release/evidence/current/ — not present (stop point S3 if review requires evidence)"
fi

# ── frozen protocol values ────────────────────────────────────────────────────

hr "Frozen protocol values (expected)"
echo "  Protocol version   : postcad-v1"
echo "  Routing kernel     : postcad-routing-v1"
echo "  Receipt schema     : 1"
echo "  Receipt hash (det) : 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb"

# ── done ──────────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  Review trace printed."
echo "  Follow steps 1–8 in REVIEW_TRACE.md."
echo "  Resolve any stop condition before proceeding."
echo "══════════════════════════════════════════"
echo ""
