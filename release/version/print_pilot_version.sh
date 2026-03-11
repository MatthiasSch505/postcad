#!/usr/bin/env bash
# PostCAD pilot — print pilot version and release anchoring information.
#
# Usage (from repo root or release/version/ directory):
#   ./release/version/print_pilot_version.sh
#
# Read-only. Does not create a git tag, modify git state, start the
# service, generate evidence, or modify any file.
# Prints: current HEAD, pilot label, key version/release files,
# verification instructions, and the optional git tag command as text.

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
echo "  PostCAD Pilot — Version"
echo "══════════════════════════════════════════"
echo "  Pilot label : $PILOT_LABEL"
echo "  Repo root   : $REPO_ROOT"
echo ""

# ── git state ───────────────────────────────────────────────────────────────

hr "Git state"
branch=$(git -C "$REPO_ROOT" branch --show-current 2>/dev/null || echo "(unknown)")
head=$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || echo "(unknown)")
clean=$(git -C "$REPO_ROOT" status --short 2>/dev/null || echo "?")
echo "  Branch : $branch"
echo "  HEAD   : $head"
[[ -z "$clean" ]] && echo "  Status : clean" || echo "  Status : DIRTY (run 'git status' for details)"

# Check if tag already exists
if git -C "$REPO_ROOT" tag -l "$PILOT_LABEL" 2>/dev/null | grep -q "$PILOT_LABEL"; then
  tag_commit=$(git -C "$REPO_ROOT" rev-list -n1 "$PILOT_LABEL" 2>/dev/null || echo "(unknown)")
  echo "  Tag    : $PILOT_LABEL → $tag_commit"
else
  echo "  Tag    : $PILOT_LABEL — not yet created (see optional command below)"
fi

# ── version and release files ─────────────────────────────────────────────────

hr "Version and release files"
for f in \
  "release/version/PILOT_VERSION.md" \
  "release/version/README.md" \
  "release/RELEASE_NOTES_PILOT.md" \
  "release/FREEZE_MANIFEST.md" \
  "release/readiness/READINESS_SNAPSHOT.md" \
  "release/freeze/FROZEN_BOUNDARIES.md"; do
  [[ -f "$REPO_ROOT/$f" ]] && ok "$f" || missing "$f"
done

# ── frozen protocol values ────────────────────────────────────────────────────

hr "Frozen protocol values"
echo "  Protocol version   : postcad-v1"
echo "  Routing kernel     : postcad-routing-v1"
echo "  Receipt schema     : 1"
echo "  Receipt hash (det) : 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb"
echo "  Pilot case ID      : f1000001-0000-0000-0000-000000000001"

# ── verification instructions ─────────────────────────────────────────────────

hr "Verification instructions"
cat <<'VERIFY'

  To confirm you are on the reviewed pilot state:

    git branch --show-current               # expected: main
    git log --oneline -1                    # review HEAD
    git status                              # expected: clean
    cargo test --workspace                  # all suites pass, 0 failures
    ./release/selfcheck/run_release_selfcheck.sh   # Missing items: 0

VERIFY

# ── optional git tag command (printed only — not executed) ────────────────────

hr "Optional git tag command (documentation only — not run by this script)"
echo ""
echo "  To anchor the current HEAD as the named pilot state:"
echo ""
echo "    git tag -a $PILOT_LABEL \$(git rev-parse HEAD) -m \"PostCAD local pilot v1\""
echo ""
echo "  Run only when explicitly instructed and after confirming the commit"
echo "  is the correct reviewed state. This script does NOT create the tag."

# ── done ──────────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  Pilot label  : $PILOT_LABEL"
echo "  HEAD         : $head"
echo "  Tag created  : $(git -C "$REPO_ROOT" tag -l "$PILOT_LABEL" 2>/dev/null | grep -q "$PILOT_LABEL" && echo "yes" || echo "no")"
echo "══════════════════════════════════════════"
echo ""
