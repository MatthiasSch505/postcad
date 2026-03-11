#!/usr/bin/env bash
# PostCAD pilot — print freeze manifest surfaces and classifications.
#
# Usage (from repo root or release/freeze/ directory):
#   ./release/freeze/print_freeze_manifest.sh
#
# Read-only. Does not start the service, run tests, generate evidence,
# or modify any file. Prints classified surface listing and key manifest
# file existence checks.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

ok()      { echo "  [OK]  $*"; }
missing() { echo "  [--]  MISSING: $*"; }
hr()      { echo ""; echo "── $* ──────────────────────────────────────────────"; }
row()     { printf "  %-12s  %s\n" "$1" "$2"; }

# ── header ─────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  PostCAD Pilot — Freeze Manifest"
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

# ── operator scripts ─────────────────────────────────────────────────────────

hr "Operator scripts  [executable]"
for f in \
  "release/start_pilot.sh" \
  "release/reset_pilot_data.sh" \
  "release/smoke_test.sh" \
  "release/generate_evidence_bundle.sh" \
  "demo/run_demo.sh"; do
  p="$REPO_ROOT/$f"
  if [[ -x "$p" ]]; then row "executable" "$f"
  elif [[ -f "$p" ]]; then missing "$f (not executable)"
  else missing "$f"
  fi
done

# ── release index / manifest ──────────────────────────────────────────────────

hr "Release index and manifest  [read-only / executable]"
row "read-only"  "release/README.md"
row "read-only"  "release/INDEX.md"
row "read-only"  "release/FREEZE_MANIFEST.md"
row "executable" "release/print_release_index.sh"
for f in \
  "release/README.md" \
  "release/INDEX.md" \
  "release/FREEZE_MANIFEST.md"; do
  [[ -f "$REPO_ROOT/$f" ]] || missing "$f"
done
[[ -x "$REPO_ROOT/release/print_release_index.sh" ]] || missing "release/print_release_index.sh (not executable or missing)"

# ── evidence bundle ────────────────────────────────────────────────────────────

hr "Evidence bundle  [read-only / runtime-generated]"
row "read-only"          "release/evidence/README.md"
row "runtime-generated"  "release/evidence/current/"
[[ -f "$REPO_ROOT/release/evidence/README.md" ]] || missing "release/evidence/README.md"
evidence="$REPO_ROOT/release/evidence/current"
if [[ -d "$evidence" ]] && [[ -f "$evidence/summary.txt" ]]; then
  grep -q "All 7 steps passed" "$evidence/summary.txt" \
    && echo "  [OK]  release/evidence/current/ — All 7 steps passed" \
    || echo "  [--]  release/evidence/current/ — present (summary check incomplete)"
else
  echo "  [--]  release/evidence/current/ — not present (gitignored; generate to populate)"
fi

# ── walkthrough / review / acceptance / handoff / selfcheck / freeze ──────────

hr "Walkthrough bundle  [read-only / executable]"
row "read-only"  "release/walkthrough/README.md"
row "read-only"  "release/walkthrough/PILOT_WALKTHROUGH.md"
row "executable" "release/walkthrough/print_walkthrough.sh"
for f in \
  "release/walkthrough/README.md" \
  "release/walkthrough/PILOT_WALKTHROUGH.md"; do
  [[ -f "$REPO_ROOT/$f" ]] || missing "$f"
done
[[ -x "$REPO_ROOT/release/walkthrough/print_walkthrough.sh" ]] || missing "release/walkthrough/print_walkthrough.sh"

hr "Review packet  [read-only]"
for f in \
  "release/review/README.md" \
  "release/review/SYSTEM_OVERVIEW.md" \
  "release/review/OPERATOR_FLOW.md" \
  "release/review/ARTIFACT_GUIDE.md" \
  "release/review/BOUNDARIES.md"; do
  row "read-only" "$f"
  [[ -f "$REPO_ROOT/$f" ]] || missing "$f"
done

hr "Acceptance bundle  [read-only / executable]"
for f in \
  "release/acceptance/README.md" \
  "release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md" \
  "release/acceptance/REVIEW_WORKSHEET.md"; do
  row "read-only" "$f"
  [[ -f "$REPO_ROOT/$f" ]] || missing "$f"
done
row "executable" "release/acceptance/print_acceptance_summary.sh"
[[ -x "$REPO_ROOT/release/acceptance/print_acceptance_summary.sh" ]] || missing "release/acceptance/print_acceptance_summary.sh"

hr "Handoff packet  [read-only / executable]"
for f in \
  "release/handoff/README.md" \
  "release/handoff/FIRST_HOUR_GUIDE.md" \
  "release/handoff/KNOWN_GOOD_STATE.md" \
  "release/handoff/HANDOFF_CHECKLIST.md"; do
  row "read-only" "$f"
  [[ -f "$REPO_ROOT/$f" ]] || missing "$f"
done
row "executable" "release/handoff/print_handoff_index.sh"
[[ -x "$REPO_ROOT/release/handoff/print_handoff_index.sh" ]] || missing "release/handoff/print_handoff_index.sh"

hr "Self-check bundle  [read-only / executable]"
for f in \
  "release/selfcheck/README.md" \
  "release/selfcheck/SELFCHECK_SCOPE.md"; do
  row "read-only" "$f"
  [[ -f "$REPO_ROOT/$f" ]] || missing "$f"
done
row "executable" "release/selfcheck/run_release_selfcheck.sh"
[[ -x "$REPO_ROOT/release/selfcheck/run_release_selfcheck.sh" ]] || missing "release/selfcheck/run_release_selfcheck.sh"

hr "Freeze bundle  [read-only / executable]"
for f in \
  "release/freeze/README.md" \
  "release/freeze/PILOT_SURFACES.md" \
  "release/freeze/FROZEN_BOUNDARIES.md"; do
  row "read-only" "$f"
  [[ -f "$REPO_ROOT/$f" ]] || missing "$f"
done
row "executable" "release/freeze/print_freeze_manifest.sh"
[[ -x "$REPO_ROOT/release/freeze/print_freeze_manifest.sh" ]] \
  || echo "  [--]  release/freeze/print_freeze_manifest.sh (not yet executable — run chmod +x)"

# ── canonical fixtures ─────────────────────────────────────────────────────────

hr "Canonical fixtures  [read-only / frozen]"
for f in \
  "examples/pilot/case.json" \
  "examples/pilot/registry_snapshot.json" \
  "examples/pilot/config.json"; do
  row "read-only" "$f"
  [[ -f "$REPO_ROOT/$f" ]] || missing "$f"
done

# ── frozen protocol values ─────────────────────────────────────────────────────

hr "Frozen protocol values"
echo "  Protocol version    : postcad-v1"
echo "  Routing kernel      : postcad-routing-v1"
echo "  Receipt schema      : 1"
echo "  Deterministic hash  : 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb"
echo "                        (canonical pilot inputs, every run)"

# ── done ──────────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  Freeze manifest printed."
echo "  This is a classification listing only."
echo "  See FROZEN_BOUNDARIES.md for full scope."
echo "══════════════════════════════════════════"
echo ""
