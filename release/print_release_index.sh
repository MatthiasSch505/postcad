#!/usr/bin/env bash
# PostCAD pilot — print top-level release index.
#
# Usage (from repo root or release/ directory):
#   ./release/print_release_index.sh
#
# Read-only. Does not start the service, run the smoke test,
# generate evidence, or modify any file.
# Prints: HEAD, git status, grouped release surfaces with [OK]/[--],
# recommended minimal path, and recommended full review path.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

ok()      { echo "  [OK]  $*"; }
missing() { echo "  [--]  MISSING: $*"; }
runok()   { echo "  [OK]  $* (executable)"; }
runmiss() { echo "  [--]  MISSING or not executable: $*"; }
hr()      { echo ""; echo "── $* ──────────────────────────────────────────────"; }

# ── header ────────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  PostCAD Pilot — Release Index"
echo "══════════════════════════════════════════"
echo "  Repo root : $REPO_ROOT"
echo ""

# ── git state ─────────────────────────────────────────────────────────────────

hr "Git state"
branch=$(git -C "$REPO_ROOT" branch --show-current 2>/dev/null || echo "(unknown)")
head=$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || echo "(unknown)")
clean=$(git -C "$REPO_ROOT" status --short 2>/dev/null || echo "?")
echo "  Branch : $branch"
echo "  HEAD   : $head"
[[ -z "$clean" ]] && echo "  Status : clean" || echo "  Status : DIRTY (run 'git status' for details)"

# ── operator scripts ───────────────────────────────────────────────────────────

hr "Operator scripts (executable)"
for f in \
  "release/start_pilot.sh" \
  "release/reset_pilot_data.sh" \
  "release/smoke_test.sh" \
  "release/generate_evidence_bundle.sh" \
  "demo/run_demo.sh"; do
  p="$REPO_ROOT/$f"
  if [[ -x "$p" ]]; then runok "$f"
  elif [[ -f "$p" ]]; then runmiss "$f (not executable)"
  else runmiss "$f"
  fi
done

# ── operator docs ──────────────────────────────────────────────────────────────

hr "Operator documentation"
for f in \
  "release/README.md" \
  "release/INDEX.md"; do
  [[ -f "$REPO_ROOT/$f" ]] && ok "$f" || missing "$f"
done

# ── walkthrough bundle ─────────────────────────────────────────────────────────

hr "Walkthrough bundle (release/walkthrough/)"
for f in \
  "release/walkthrough/README.md" \
  "release/walkthrough/PILOT_WALKTHROUGH.md"; do
  [[ -f "$REPO_ROOT/$f" ]] && ok "$f" || missing "$f"
done
f="release/walkthrough/print_walkthrough.sh"
p="$REPO_ROOT/$f"
if [[ -x "$p" ]]; then runok "$f"
elif [[ -f "$p" ]]; then runmiss "$f (not executable)"
else runmiss "$f"
fi

# ── evidence bundle ────────────────────────────────────────────────────────────

hr "Evidence bundle (release/evidence/)"
[[ -f "$REPO_ROOT/release/evidence/README.md" ]] && ok "release/evidence/README.md" || missing "release/evidence/README.md"
evidence="$REPO_ROOT/release/evidence/current"
if [[ -d "$evidence" ]] && [[ -f "$evidence/summary.txt" ]]; then
  grep -q "All 7 steps passed" "$evidence/summary.txt" \
    && ok "release/evidence/current/ — summary: All 7 steps passed" \
    || ok "release/evidence/current/ — present (summary check failed or incomplete)"
else
  echo "  [--]  release/evidence/current/ — not present"
  echo "        Generate: ./release/start_pilot.sh (Terminal A)"
  echo "                  ./release/generate_evidence_bundle.sh (Terminal B)"
fi

# ── review packet ──────────────────────────────────────────────────────────────

hr "Review packet (release/review/)"
for f in \
  "release/review/README.md" \
  "release/review/SYSTEM_OVERVIEW.md" \
  "release/review/OPERATOR_FLOW.md" \
  "release/review/ARTIFACT_GUIDE.md" \
  "release/review/BOUNDARIES.md"; do
  [[ -f "$REPO_ROOT/$f" ]] && ok "$f" || missing "$f"
done

# ── acceptance bundle ──────────────────────────────────────────────────────────

hr "Acceptance bundle (release/acceptance/)"
for f in \
  "release/acceptance/README.md" \
  "release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md" \
  "release/acceptance/REVIEW_WORKSHEET.md"; do
  [[ -f "$REPO_ROOT/$f" ]] && ok "$f" || missing "$f"
done
f="release/acceptance/print_acceptance_summary.sh"
p="$REPO_ROOT/$f"
if [[ -x "$p" ]]; then runok "$f"
elif [[ -f "$p" ]]; then runmiss "$f (not executable)"
else runmiss "$f"
fi

# ── handoff packet ─────────────────────────────────────────────────────────────

hr "Handoff packet (release/handoff/)"
for f in \
  "release/handoff/README.md" \
  "release/handoff/FIRST_HOUR_GUIDE.md" \
  "release/handoff/KNOWN_GOOD_STATE.md" \
  "release/handoff/HANDOFF_CHECKLIST.md"; do
  [[ -f "$REPO_ROOT/$f" ]] && ok "$f" || missing "$f"
done
f="release/handoff/print_handoff_index.sh"
p="$REPO_ROOT/$f"
if [[ -x "$p" ]]; then runok "$f"
elif [[ -f "$p" ]]; then runmiss "$f (not executable)"
else runmiss "$f"
fi

# ── recommended paths ──────────────────────────────────────────────────────────

hr "Recommended minimal path"
cat <<'MINIMAL'

  Two terminals required for steps 1–4.
  Run all commands from the repo root.

  Terminal A:
    ./release/start_pilot.sh                       (start service, leave running)

  Terminal B:
    ./release/reset_pilot_data.sh                  (clean slate)
    ./release/smoke_test.sh                        (7-step smoke test)
    ./demo/run_demo.sh                             (self-contained demo)
    ./release/generate_evidence_bundle.sh          (capture evidence)
    cat release/evidence/current/summary.txt       (confirm: All 7 steps passed.)

MINIMAL

hr "Recommended full review path"
cat <<'FULL'

  # Orient (read-only, no service needed)
    ./release/walkthrough/print_walkthrough.sh
    cat release/walkthrough/PILOT_WALKTHROUGH.md

  # Run the pilot (minimal path above)

  # Inspect
    cat release/review/README.md
    cat release/review/SYSTEM_OVERVIEW.md
    cat release/review/ARTIFACT_GUIDE.md

  # Acceptance pre-check
    ./release/acceptance/print_acceptance_summary.sh
    cat release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md

  # Handoff index
    ./release/handoff/print_handoff_index.sh
    cat release/handoff/FIRST_HOUR_GUIDE.md

FULL

# ── done ──────────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  Release index complete."
echo "  Items marked [--] require attention."
echo "  Read INDEX.md for the full reference table."
echo "══════════════════════════════════════════"
echo ""
