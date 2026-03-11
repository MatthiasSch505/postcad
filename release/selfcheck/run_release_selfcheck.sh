#!/usr/bin/env bash
# PostCAD pilot — read-only structural release self-check.
#
# Usage (from repo root or release/selfcheck/ directory):
#   ./release/selfcheck/run_release_selfcheck.sh
#
# Read-only. Does not start the service, run tests, generate evidence,
# or modify any file. Prints [OK]/[--] for expected release surfaces.
# See SELFCHECK_SCOPE.md for the complete scope definition.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

MISSING=0

ok()      { echo "  [OK]  $*"; }
missing() { echo "  [--]  MISSING: $*"; MISSING=$((MISSING + 1)); }
info()    { echo "  [--]  $*"; }
runok()   { echo "  [OK]  $* (executable)"; }
runmiss() { echo "  [--]  NOT EXECUTABLE: $*"; MISSING=$((MISSING + 1)); }
hr()      { echo ""; echo "── $* ──────────────────────────────────────────────"; }

check_file() {
  local f="$1"
  [[ -f "$REPO_ROOT/$f" ]] && ok "$f" || missing "$f"
}

check_exec() {
  local f="$1"
  local p="$REPO_ROOT/$f"
  if [[ -x "$p" ]]; then
    runok "$f"
  elif [[ -f "$p" ]]; then
    runmiss "$f (not executable)"
  else
    missing "$f"
  fi
}

# ── header ─────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  PostCAD Pilot — Release Self-Check"
echo "══════════════════════════════════════════"
echo "  Repo root : $REPO_ROOT"
echo "  Scope     : structural file-presence only"
echo "  See       : release/selfcheck/SELFCHECK_SCOPE.md"
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

hr "Operator scripts"
check_exec "release/start_pilot.sh"
check_exec "release/reset_pilot_data.sh"
check_exec "release/smoke_test.sh"
check_exec "release/generate_evidence_bundle.sh"
check_exec "demo/run_demo.sh"

# ── release index and runbook ─────────────────────────────────────────────────

hr "Release index and runbook"
check_file "release/INDEX.md"
check_file "release/README.md"
check_exec "release/print_release_index.sh"

# ── walkthrough bundle ────────────────────────────────────────────────────────

hr "Walkthrough bundle (release/walkthrough/)"
check_file "release/walkthrough/README.md"
check_file "release/walkthrough/PILOT_WALKTHROUGH.md"
check_exec "release/walkthrough/print_walkthrough.sh"

# ── evidence bundle ───────────────────────────────────────────────────────────

hr "Evidence bundle (release/evidence/)"
check_file "release/evidence/README.md"
evidence="$REPO_ROOT/release/evidence/current"
if [[ -d "$evidence" ]] && [[ -f "$evidence/summary.txt" ]]; then
  if grep -q "All 7 steps passed" "$evidence/summary.txt"; then
    ok "release/evidence/current/ — summary: All 7 steps passed"
  else
    ok "release/evidence/current/ — present (summary does not contain 'All 7 steps passed')"
  fi
else
  info "release/evidence/current/ — not present (gitignored; generate with generate_evidence_bundle.sh)"
fi

# ── review packet ─────────────────────────────────────────────────────────────

hr "Review packet (release/review/)"
check_file "release/review/README.md"
check_file "release/review/SYSTEM_OVERVIEW.md"
check_file "release/review/OPERATOR_FLOW.md"
check_file "release/review/ARTIFACT_GUIDE.md"
check_file "release/review/BOUNDARIES.md"

# ── acceptance bundle ─────────────────────────────────────────────────────────

hr "Acceptance bundle (release/acceptance/)"
check_file "release/acceptance/README.md"
check_file "release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md"
check_file "release/acceptance/REVIEW_WORKSHEET.md"
check_exec "release/acceptance/print_acceptance_summary.sh"

# ── handoff packet ────────────────────────────────────────────────────────────

hr "Handoff packet (release/handoff/)"
check_file "release/handoff/README.md"
check_file "release/handoff/FIRST_HOUR_GUIDE.md"
check_file "release/handoff/KNOWN_GOOD_STATE.md"
check_file "release/handoff/HANDOFF_CHECKLIST.md"
check_exec "release/handoff/print_handoff_index.sh"

# ── self-check bundle ─────────────────────────────────────────────────────────

hr "Self-check bundle (release/selfcheck/)"
check_file "release/selfcheck/README.md"
check_file "release/selfcheck/SELFCHECK_SCOPE.md"

# ── canonical fixtures ────────────────────────────────────────────────────────

hr "Canonical fixtures (examples/pilot/)"
check_file "examples/pilot/case.json"
check_file "examples/pilot/registry_snapshot.json"
check_file "examples/pilot/config.json"

# ── cross-references ──────────────────────────────────────────────────────────

hr "Cross-references"
# Paths cited in INDEX.md and README.md that must exist
check_exec "demo/run_demo.sh"
[[ -d "$REPO_ROOT/examples/pilot" ]] \
  && ok "examples/pilot/ — directory present" \
  || missing "examples/pilot/ — directory missing"

# ── summary ───────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  Structural self-check completed."
if [[ "$MISSING" -eq 0 ]]; then
  echo "  Missing items: 0 — package structure intact."
else
  echo "  Missing items: $MISSING — see [--] lines above."
fi
echo "  This check covers file presence only."
echo "  See SELFCHECK_SCOPE.md for full scope."
echo "══════════════════════════════════════════"
echo ""
