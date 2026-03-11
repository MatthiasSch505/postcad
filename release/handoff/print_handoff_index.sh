#!/usr/bin/env bash
# PostCAD pilot handoff — print compact index of expected resources.
#
# Usage (from repo root or release/ directory):
#   ./release/handoff/print_handoff_index.sh
#
# Read-only. Does not modify anything. Does not declare acceptance.
# Prints current HEAD, git status summary, and whether each expected
# handoff/release resource exists.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

ok()      { echo "  [OK]  $*"; }
missing() { echo "  [--]  MISSING: $*"; }
hr()      { echo ""; echo "── $* ──────────────────────────────────────────────"; }

echo ""
echo "══════════════════════════════════════════"
echo "  PostCAD Pilot — Handoff Index"
echo "══════════════════════════════════════════"
echo "  Repo root : $REPO_ROOT"
echo ""

# ── Git state ─────────────────────────────────────────────────────────────────

hr "Git state"
branch=$(git -C "$REPO_ROOT" branch --show-current 2>/dev/null || echo "(unknown)")
head=$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || echo "(unknown)")
status=$(git -C "$REPO_ROOT" status --short 2>/dev/null || echo "(unknown)")
echo "  Branch : $branch"
echo "  HEAD   : $head"
if [[ -z "$status" ]]; then
  echo "  Status : clean"
else
  echo "  Status : DIRTY"
  echo "$status" | sed 's/^/    /'
fi

# ── Operator scripts ──────────────────────────────────────────────────────────

hr "Operator scripts"
for f in \
  "release/start_pilot.sh" \
  "release/reset_pilot_data.sh" \
  "release/smoke_test.sh" \
  "release/generate_evidence_bundle.sh" \
  "demo/run_demo.sh"; do
  p="$REPO_ROOT/$f"
  if [[ -x "$p" ]]; then ok "$f"; elif [[ -f "$p" ]]; then missing "$f (not executable)"; else missing "$f"; fi
done

# ── Canonical fixtures ────────────────────────────────────────────────────────

hr "Canonical fixtures"
for f in \
  "examples/pilot/case.json" \
  "examples/pilot/registry_snapshot.json" \
  "examples/pilot/config.json"; do
  [[ -f "$REPO_ROOT/$f" ]] && ok "$f" || missing "$f"
done
fixture_diff=$(git -C "$REPO_ROOT" diff examples/pilot/ 2>/dev/null || echo "")
[[ -z "$fixture_diff" ]] && ok "examples/pilot/ — no uncommitted changes" \
  || missing "examples/pilot/ — HAS UNCOMMITTED CHANGES"

# ── Review packet ─────────────────────────────────────────────────────────────

hr "Review packet (release/review/)"
for f in README.md SYSTEM_OVERVIEW.md OPERATOR_FLOW.md ARTIFACT_GUIDE.md BOUNDARIES.md; do
  [[ -f "$REPO_ROOT/release/review/$f" ]] && ok "release/review/$f" || missing "release/review/$f"
done

# ── Acceptance bundle ─────────────────────────────────────────────────────────

hr "Acceptance bundle (release/acceptance/)"
for f in README.md PILOT_ACCEPTANCE_CHECKLIST.md REVIEW_WORKSHEET.md print_acceptance_summary.sh; do
  p="$REPO_ROOT/release/acceptance/$f"
  if [[ "$f" == *.sh ]]; then
    if [[ -x "$p" ]]; then ok "release/acceptance/$f"; elif [[ -f "$p" ]]; then missing "release/acceptance/$f (not executable)"; else missing "release/acceptance/$f"; fi
  else
    [[ -f "$p" ]] && ok "release/acceptance/$f" || missing "release/acceptance/$f"
  fi
done

# ── Handoff packet ────────────────────────────────────────────────────────────

hr "Handoff packet (release/handoff/)"
for f in README.md HANDOFF_CHECKLIST.md FIRST_HOUR_GUIDE.md KNOWN_GOOD_STATE.md; do
  [[ -f "$REPO_ROOT/release/handoff/$f" ]] && ok "release/handoff/$f" || missing "release/handoff/$f"
done

# ── Evidence bundle ───────────────────────────────────────────────────────────

hr "Evidence bundle (release/evidence/current/)"
EVIDENCE="$REPO_ROOT/release/evidence/current"
if [[ -d "$EVIDENCE" ]]; then
  for f in summary.txt git_head.txt commands.txt \
            01_health.json 02_store_case.json 03_route_case.json \
            04_receipt.json 05_dispatch.json 06_verify.json 07_route_history.json; do
    [[ -f "$EVIDENCE/$f" ]] && ok "$f" || missing "$f"
  done
  if [[ -f "$EVIDENCE/summary.txt" ]]; then
    grep -q "All 7 steps passed" "$EVIDENCE/summary.txt" \
      && ok "summary.txt — All 7 steps passed" \
      || missing "summary.txt — does NOT contain 'All 7 steps passed.'"
  fi
else
  echo "  [--]  release/evidence/current/ not present"
  echo "        Generate with: ./release/start_pilot.sh (Terminal A)"
  echo "                       ./release/generate_evidence_bundle.sh (Terminal B)"
fi

# ── done ──────────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  Handoff index complete."
echo "  Items marked [--] require attention."
echo "  Next: read FIRST_HOUR_GUIDE.md"
echo "══════════════════════════════════════════"
echo ""
