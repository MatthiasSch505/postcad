#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
QUEUE_DIR="$ROOT/ops/campaign_queue"
STATUS_LOG="$ROOT/ops/queue_status.log"
LAST_RESULT="$ROOT/ops/last_result.md"
QUEUE_RUNNER="$ROOT/ops/run_campaign_queue.sh"

CMD="${1:-}"

case "$CMD" in

    # ── start: delegate to queue runner ────────────────────────────────────────
    start)
        shift
        exec bash "$QUEUE_RUNNER" "$@"
        ;;

    # ── status: compact one-screen summary from last_result.md ─────────────────
    status)
        if [[ ! -f "$LAST_RESULT" ]]; then
            echo "State     : NOT_RUN"
            echo "No summary found. Run: bash ops/queue_ctl.sh start"
            exit 0
        fi
        STATE=$(grep    "^Status"              "$LAST_RESULT" | head -1 | sed 's/.*: *//' | tr -d '\r')
        EXECUTED=$(grep "^Campaigns executed"  "$LAST_RESULT" | head -1 | sed 's/.*: *//' | tr -d '\r')
        BLOCKED=$(grep  "^Campaigns blocked"   "$LAST_RESULT" | head -1 | sed 's/.*: *//' | tr -d '\r')
        LAST_OK=$(grep  "^Last successful"     "$LAST_RESULT" | head -1 | sed 's/Last successful *: *//' | tr -d '\r')
        BLOCKED_C=$(grep "^Blocked campaign"   "$LAST_RESULT" | head -1 | sed 's/Blocked campaign *: *//' | tr -d '\r')
        PENDING=$(find "$QUEUE_DIR" -maxdepth 1 -name "*.md" 2>/dev/null | wc -l | tr -d '[:space:]')
        echo "State     : ${STATE:-unknown}"
        echo "Executed  : ${EXECUTED:-0}"
        echo "Blocked   : ${BLOCKED:-0}"
        echo "Last ok   : ${LAST_OK:-—}"
        [[ -n "$BLOCKED_C" && "$BLOCKED_C" != "—" ]] && echo "Blocked   : $BLOCKED_C"
        echo "Pending   : $PENDING"
        ;;

    # ── tail: recent queue log entries (phone-safe volume) ─────────────────────
    tail)
        if [[ ! -f "$STATUS_LOG" ]]; then
            echo "No status log found."
            exit 0
        fi
        tail -20 "$STATUS_LOG"
        ;;

    # ── pending: deterministic list of queued campaign files ───────────────────
    pending)
        mapfile -t PENDING_FILES < <(find "$QUEUE_DIR" -maxdepth 1 -name "*.md" 2>/dev/null | sort)
        if [[ ${#PENDING_FILES[@]} -eq 0 ]]; then
            echo "No pending campaigns."
        else
            echo "Pending (${#PENDING_FILES[@]}):"
            for f in "${PENDING_FILES[@]}"; do
                echo "  $(basename "$f")"
            done
        fi
        ;;

    # ── usage ──────────────────────────────────────────────────────────────────
    *)
        echo "Usage: bash ops/queue_ctl.sh <command>"
        echo ""
        echo "Commands:"
        echo "  start           run queue (all pending campaigns)"
        echo "  start --max N   run queue (at most N campaigns)"
        echo "  status          print compact queue summary"
        echo "  tail            print recent queue log entries"
        echo "  pending         list pending campaign files"
        exit 1
        ;;
esac
