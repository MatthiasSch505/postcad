#!/usr/bin/env bash
# start_night_queue.sh — one-command unattended lane-1 night queue launcher
# Usage: bash ops/start_night_queue.sh
set -euo pipefail

SESSION="postcad-night"

# ── resolve repo root (works from any CWD inside the repo) ─────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ── verify required scripts exist ──────────────────────────────────────────
REQUIRED_SCRIPTS=(
    "$ROOT/ops/run_campaign.sh"
    "$ROOT/ops/run_campaign_queue.sh"
)
for f in "${REQUIRED_SCRIPTS[@]}"; do
    if [[ ! -f "$f" ]]; then
        echo "ERROR: required script not found: $f" >&2
        exit 1
    fi
done

# ── refuse duplicate starts ─────────────────────────────────────────────────
if tmux has-session -t "$SESSION" 2>/dev/null; then
    echo "ERROR: tmux session '$SESSION' already exists." >&2
    echo "       Inspect : tmux attach -t $SESSION" >&2
    echo "       Stop    : tmux kill-session -t $SESSION" >&2
    exit 1
fi

# ── launch detached tmux session ────────────────────────────────────────────
tmux new-session -d -s "$SESSION" \
    "cd '$ROOT' && bash ./ops/run_campaign_queue.sh"

# ── success summary ─────────────────────────────────────────────────────────
LOG_DIR="$ROOT/ops/logs"
echo ""
echo "Night queue started."
echo ""
echo "  Session : $SESSION"
echo "  Logs    : $LOG_DIR/"
echo "  Attach  : tmux attach -t $SESSION"
echo "  Stop    : tmux kill-session -t $SESSION"
echo ""
