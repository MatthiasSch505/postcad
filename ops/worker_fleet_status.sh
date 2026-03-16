#!/usr/bin/env bash
# ops/worker_fleet_status.sh
#
# Read-only status inspector for the 2-worker fleet.
# Shows worker paths, git worktree registration, branch, and clean/dirty state.
# Does not start anything. Does not modify git state.
#
# Usage:
#   bash ops/worker_fleet_status.sh
#   bash ops/worker_fleet_status.sh --base-dir /some/other/path

set -euo pipefail

# ── Locate canonical repo root ─────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Refuse if not inside a git repo
if ! git -C "$REPO_ROOT" rev-parse --git-dir > /dev/null 2>&1; then
  echo "ERROR: $REPO_ROOT is not a git repository." >&2
  echo "Run this script from within the PostCAD repo." >&2
  exit 1
fi

# Refuse if not the PostCAD repo (check for a known marker)
if [[ ! -f "$REPO_ROOT/CLAUDE.md" ]] || ! grep -q "Post-CAD Layer" "$REPO_ROOT/CLAUDE.md" 2>/dev/null; then
  echo "ERROR: $REPO_ROOT does not appear to be the PostCAD repository." >&2
  echo "Expected CLAUDE.md with 'Post-CAD Layer' marker." >&2
  exit 1
fi

# ── Parse arguments ────────────────────────────────────────────────────────────

BASE_DIR="${HOME}/workers"

while [[ $# -gt 0 ]]; do
  case "${1:-}" in
    --base-dir)
      if [[ -z "${2:-}" ]]; then
        echo "ERROR: --base-dir requires a path argument." >&2
        exit 1
      fi
      BASE_DIR="$2"
      shift 2
      ;;
    *)
      echo "ERROR: Unknown argument: $1" >&2
      echo "Usage: bash ops/worker_fleet_status.sh [--base-dir <path>]" >&2
      exit 1
      ;;
  esac
done

# ── Worker definitions ─────────────────────────────────────────────────────────

WA_PATH="${BASE_DIR}/postcad-worker-a"
WB_PATH="${BASE_DIR}/postcad-worker-b"

# ── Helper: inspect one worker ─────────────────────────────────────────────────

# Prints status lines for a single worker. Read-only.
inspect_worker() {
  local label="$1"
  local wpath="$2"

  echo "$label"
  echo "  Path   : $wpath"

  if [[ ! -e "$wpath" ]]; then
    echo "  Status : missing"
    echo "  Branch : n/a"
    echo "  State  : n/a"
    return
  fi

  # Check git worktree registration
  if git -C "$REPO_ROOT" worktree list --porcelain 2>/dev/null \
      | grep -qF "worktree $wpath"; then
    echo "  Status : registered worktree"
  elif [[ -d "$wpath/.git" ]] || [[ -f "$wpath/.git" ]]; then
    echo "  Status : git repo (not a registered worktree of this repo)"
    echo "  Branch : n/a"
    echo "  State  : n/a"
    return
  else
    echo "  Status : not a git worktree"
    echo "  Branch : n/a"
    echo "  State  : n/a"
    return
  fi

  # Branch
  local branch
  branch=$(git -C "$wpath" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
  echo "  Branch : $branch"

  # Clean/dirty
  if git -C "$wpath" diff --quiet 2>/dev/null && git -C "$wpath" diff --cached --quiet 2>/dev/null; then
    echo "  State  : clean"
  else
    echo "  State  : dirty (uncommitted changes)"
  fi
}

# ── Check claude availability ──────────────────────────────────────────────────

CLAUDE_AVAILABLE="no"
if command -v claude > /dev/null 2>&1; then
  CLAUDE_AVAILABLE="yes"
fi

# ── Determine NEXT advice ──────────────────────────────────────────────────────

WA_EXISTS=false
WB_EXISTS=false
[[ -e "$WA_PATH" ]] && WA_EXISTS=true
[[ -e "$WB_PATH" ]] && WB_EXISTS=true

# ── Print report ───────────────────────────────────────────────────────────────

echo ""
echo "======================================"
echo "POSTCAD WORKER FLEET STATUS"
echo "======================================"
echo ""
echo "REPO"
echo "  Path   : $REPO_ROOT"
echo ""
echo "BASE DIR"
echo "  Path   : $BASE_DIR"
echo ""
inspect_worker "WORKER A" "$WA_PATH"
echo ""
inspect_worker "WORKER B" "$WB_PATH"
echo ""
echo "  claude : $CLAUDE_AVAILABLE (in PATH)"
echo ""
echo "--------------------------------------"
echo "NEXT"
echo "--------------------------------------"
echo ""

if [[ "$WA_EXISTS" == false || "$WB_EXISTS" == false ]]; then
  echo "  One or more workers are missing. Run bootstrap first:"
  echo "  bash ops/setup_two_worker_fleet.sh --base-dir $BASE_DIR"
  echo ""
fi

if [[ "$WA_EXISTS" == true ]]; then
  echo "  Enter worker A :"
  echo "    cd $WA_PATH && claude"
  echo ""
fi

if [[ "$WB_EXISTS" == true ]]; then
  echo "  Enter worker B :"
  echo "    cd $WB_PATH && claude"
  echo ""
fi
