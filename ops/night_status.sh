#!/usr/bin/env bash
set -euo pipefail

# ops/night_status.sh — compact night-session status for operators
# Usage: bash ops/night_status.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

QUEUE_LOG="${REPO_ROOT}/ops/queue_status.log"
LAST_RESULT="${REPO_ROOT}/ops/last_result.md"
SESSION="postcad-night"
TAIL_LINES=5

echo "=== PostCAD Night Status ($(date '+%Y-%m-%d %H:%M:%S')) ==="
echo ""

# --- Session status ---
if tmux has-session -t "${SESSION}" 2>/dev/null; then
    echo "  Session : RUNNING  (tmux: ${SESSION})"
else
    echo "  Session : STOPPED  (tmux: ${SESSION})"
fi

echo ""

# --- Recent queue log ---
if [[ -f "${QUEUE_LOG}" ]]; then
    echo "--- queue_status.log (last ${TAIL_LINES} lines) ---"
    tail -n "${TAIL_LINES}" "${QUEUE_LOG}"
else
    echo "--- queue_status.log : not found ---"
fi

echo ""

# --- Recent last result ---
if [[ -f "${LAST_RESULT}" ]]; then
    echo "--- last_result.md (last ${TAIL_LINES} lines) ---"
    tail -n "${TAIL_LINES}" "${LAST_RESULT}"
else
    echo "--- last_result.md : not found ---"
fi

echo ""
echo "=== end ==="
