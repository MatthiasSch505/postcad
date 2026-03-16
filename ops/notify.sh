#!/usr/bin/env bash
# ops/notify.sh — best-effort notification helper for PostCAD queue events.
#
# Usage: bash ops/notify.sh <event> <campaign> <message>
#
# Always appends a formatted entry to ops/logs/notify.log and prints to stderr.
# If POSTCAD_NOTIFY_CMD is set, calls it with the three arguments.
# Never exits non-zero — notification failure must not break the queue.

set -uo pipefail

EVENT="${1:-unknown}"
CAMPAIGN="${2:-unknown}"
MESSAGE="${3:-}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
NOTIFY_LOG="$ROOT/ops/logs/notify.log"

# Ensure log directory exists (safe even on repeated calls)
mkdir -p "$(dirname "$NOTIFY_LOG")" 2>/dev/null || true

TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
LOG_LINE="[$TIMESTAMP] [$EVENT] $CAMPAIGN — $MESSAGE"

# Always append to local notification log
echo "$LOG_LINE" >> "$NOTIFY_LOG" 2>/dev/null || true

# Always print to stderr for terminal visibility
echo "$LOG_LINE" >&2

# Optional: delegate to external notifier if configured
if [[ -n "${POSTCAD_NOTIFY_CMD:-}" ]]; then
    if eval "$POSTCAD_NOTIFY_CMD" "$EVENT" "$CAMPAIGN" "$MESSAGE" >> "$NOTIFY_LOG" 2>&1; then
        echo "[NOTIFY-OK]   [$EVENT] $TIMESTAMP" >> "$NOTIFY_LOG" 2>/dev/null || true
    else
        echo "[NOTIFY-FAIL] [$EVENT] $TIMESTAMP (non-fatal)" >> "$NOTIFY_LOG" 2>/dev/null || true
    fi
fi

exit 0
