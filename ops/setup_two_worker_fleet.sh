#!/usr/bin/env bash
# ops/setup_two_worker_fleet.sh
#
# Bootstraps a 2-worker isolated worktree layout for safe parallel lane-1
# campaign execution.
#
# Workers are isolated git worktrees on dedicated branches.
# Nothing starts automatically. No tmux. No auto-merge. No auto-push.
# Each worker is entered manually and Claude is launched manually inside it.
#
# Usage:
#   bash ops/setup_two_worker_fleet.sh
#   bash ops/setup_two_worker_fleet.sh --base-dir /some/other/path
#
# Defaults:
#   Worker 1: ~/workers/postcad-w1  (branch: worker/w1)
#   Worker 2: ~/workers/postcad-w2  (branch: worker/w2)

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
      echo "Usage: bash ops/setup_two_worker_fleet.sh [--base-dir <path>]" >&2
      exit 1
      ;;
  esac
done

# ── Worker definitions ─────────────────────────────────────────────────────────

W1_PATH="${BASE_DIR}/postcad-w1"
W1_BRANCH="worker/w1"

W2_PATH="${BASE_DIR}/postcad-w2"
W2_BRANCH="worker/w2"

# ── Print plan ─────────────────────────────────────────────────────────────────

echo ""
echo "======================================"
echo "POSTCAD TWO-WORKER FLEET BOOTSTRAP"
echo "======================================"
echo ""
echo "Canonical repo : $REPO_ROOT"
echo "Worker base    : $BASE_DIR"
echo ""
echo "Worker 1 path  : $W1_PATH"
echo "Worker 1 branch: $W1_BRANCH"
echo ""
echo "Worker 2 path  : $W2_PATH"
echo "Worker 2 branch: $W2_BRANCH"
echo ""
echo "--------------------------------------"
echo ""

# ── Helper: setup one worker ───────────────────────────────────────────────────

setup_worker() {
  local label="$1"
  local wpath="$2"
  local wbranch="$3"

  echo "Setting up $label..."

  # Check if path already exists
  if [[ -e "$wpath" ]]; then
    # Verify it is a registered git worktree for this repo
    if git -C "$REPO_ROOT" worktree list --porcelain 2>/dev/null \
        | grep -qF "worktree $wpath"; then
      echo "  $label: worktree already registered at $wpath — reusing."
    elif [[ -d "$wpath/.git" ]] || [[ -f "$wpath/.git" ]]; then
      echo "  ERROR: $wpath exists and is a git repo but is not a registered" >&2
      echo "         worktree of this repo. Remove it manually before retrying." >&2
      exit 1
    else
      echo "  ERROR: $wpath exists but is not a git worktree." >&2
      echo "         Remove it manually before retrying." >&2
      exit 1
    fi
    echo "  $label: OK (pre-existing)"
    return
  fi

  # Check if branch already exists
  if git -C "$REPO_ROOT" rev-parse --verify "refs/heads/$wbranch" > /dev/null 2>&1; then
    echo "  $label: branch '$wbranch' already exists — reusing."
    git -C "$REPO_ROOT" worktree add "$wpath" "$wbranch"
  else
    echo "  $label: creating branch '$wbranch' from HEAD."
    git -C "$REPO_ROOT" worktree add -b "$wbranch" "$wpath"
  fi

  echo "  $label: worktree created at $wpath"
}

# ── Create base directory if needed ───────────────────────────────────────────

if [[ ! -d "$BASE_DIR" ]]; then
  echo "Creating base directory: $BASE_DIR"
  mkdir -p "$BASE_DIR"
fi

# ── Bootstrap workers ──────────────────────────────────────────────────────────

setup_worker "Worker 1" "$W1_PATH" "$W1_BRANCH"
echo ""
setup_worker "Worker 2" "$W2_PATH" "$W2_BRANCH"
echo ""

# ── Summary ────────────────────────────────────────────────────────────────────

echo "--------------------------------------"
echo "FLEET READY"
echo "--------------------------------------"
echo ""
echo "Canonical repo : $REPO_ROOT"
echo ""
echo "Worker 1"
echo "  Path  : $W1_PATH"
echo "  Branch: $W1_BRANCH"
echo "  Start : cd $W1_PATH && claude"
echo ""
echo "Worker 2"
echo "  Path  : $W2_PATH"
echo "  Branch: $W2_BRANCH"
echo "  Start : cd $W2_PATH && claude"
echo ""
echo "--------------------------------------"
echo "SAFETY RULES"
echo "--------------------------------------"
echo ""
echo "  - One campaign per worker at a time. No overlap."
echo "  - No two campaigns may touch the same files concurrently."
echo "  - Kernel/protocol files are off-limits in all workers."
echo "  - No auto-merge. Founder reviews and merges to main."
echo "  - No auto-push. Push only when the campaign is complete and verified."
echo ""
echo "To assign a campaign: copy the campaign file into the worker directory"
echo "and launch Claude manually inside that worker."
echo ""
