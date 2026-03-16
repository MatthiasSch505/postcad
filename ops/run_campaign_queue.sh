#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
QUEUE_DIR="$ROOT/ops/campaign_queue"
DONE_DIR="$QUEUE_DIR/done"
LOGS_DIR="$QUEUE_DIR/logs"
STATUS_LOG="$ROOT/ops/queue_status.log"
LAST_RESULT="$ROOT/ops/last_result.md"
CURRENT_CAMPAIGN="$ROOT/ops/current_campaign.md"
RUN_CAMPAIGN="$ROOT/ops/run_campaign.sh"

DRY_RUN=false
MAX_COUNT=0
PROCESSED=0

# ── run counters ────────────────────────────────────────────────────────────────

COUNT_DISCOVERED=0
COUNT_EXECUTED=0
COUNT_PASSED=0
COUNT_RETRY=0
COUNT_BLOCKED=0
LAST_SUCCESS=""
BLOCKED_CAMPAIGN=""
LATEST_LOG=""
QUEUE_START_TIME="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
QUEUE_FINAL_STATUS="NOT_RUN"

# ── argument parsing ────────────────────────────────────────────────────────────

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run) DRY_RUN=true; shift ;;
        --max)
            MAX_COUNT="${2:?--max requires a numeric value}"
            shift 2
            ;;
        *) echo "ERROR: unknown flag: $1" >&2; exit 1 ;;
    esac
done

# ── lane-1 guard configuration ──────────────────────────────────────────────────

# Kernel crates — never touched in unattended mode
FORBIDDEN_KERNEL_PATHS=(
    "crates/core"
    "crates/routing"
    "crates/compliance"
    "crates/audit"
    "crates/registry"
)

# Returns 0 (true) if path touches a forbidden kernel crate
is_kernel_path() {
    local path="$1"
    for forbidden in "${FORBIDDEN_KERNEL_PATHS[@]}"; do
        if [[ "$path" == *"${forbidden}"* ]]; then
            return 0
        fi
    done
    return 1
}

# Returns 0 (true) if path is within lane-1 automation boundary
is_lane1_path() {
    local path="$1"
    [[ "$path" == examples/pilot/* ]] && return 0
    [[ "$path" == docs/* ]] && return 0
    [[ "$path" == ops/* ]] && return 0
    [[ "$path" =~ ^crates/service/tests/.*surface_tests\.rs$ ]] && return 0
    return 1
}

# ── campaign file parsers ───────────────────────────────────────────────────────

extract_campaign_name() {
    local file="$1"
    awk '
        /^campaign name[[:space:]]*$/ { in_section=1; next }
        in_section && /^[[:space:]]*$/ { next }
        in_section { print; exit }
    ' "$file" | tr -d '\r'
}

extract_allowed_files() {
    local file="$1"
    awk '
        /^files allowed to change[[:space:]]*$/ { in_section=1; next }
        in_section && /^(Claude prompt|test command|commit message|objective|commands run|result)[[:space:]]*$/ { in_section=0 }
        in_section && /^[^[:space:]]/ { print }
    ' "$file"
}

extract_test_command() {
    local file="$1"
    awk '
        /^test command[[:space:]]*$/ { in_section=1; next }
        in_section && /^[[:space:]]*$/ { next }
        in_section && /^```/ { next }
        in_section { print; exit }
    ' "$file" | tr -d '\r'
}

# ── guard: validates campaign file against lane-1 boundary ─────────────────────
# Prints rejection reason to stdout and returns 1 if rejected; returns 0 if safe
check_campaign_guard() {
    local campaign_file="$1"
    local files_ok=true
    local reason=""
    local path

    while IFS= read -r path; do
        # strip leading/trailing whitespace
        path="${path#"${path%%[![:space:]]*}"}"
        path="${path%"${path##*[![:space:]]}"}"
        [[ -z "$path" ]] && continue

        if is_kernel_path "$path"; then
            reason="KERNEL PATH FORBIDDEN: $path"
            files_ok=false
            break
        fi

        if ! is_lane1_path "$path"; then
            reason="PATH OUTSIDE LANE-1 BOUNDARY: $path"
            files_ok=false
            break
        fi
    done < <(extract_allowed_files "$campaign_file")

    if $files_ok; then
        return 0
    else
        echo "$reason"
        return 1
    fi
}

# ── status helpers ──────────────────────────────────────────────────────────────

log_status() {
    local tag="$1"
    local campaign_basename="$2"
    local timestamp
    timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "[$tag] $campaign_basename $timestamp" >> "$STATUS_LOG"
}

# Writes the full queue summary to ops/last_result.md.
# Usage: write_summary <STATUS>
write_summary() {
    local status="$1"
    local end_time_display="—"
    [[ "$status" != "RUNNING" ]] && end_time_display="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

    # Collect pending campaigns remaining in queue dir, in deterministic order
    local pending_lines=""
    while IFS= read -r f; do
        pending_lines+="  $(basename "$f")"$'\n'
    done < <(find "$QUEUE_DIR" -maxdepth 1 -name "*.md" | sort)

    {
        echo "# Last Queue Result"
        echo ""
        echo "Status               : $status"
        echo "Start time           : ${QUEUE_START_TIME:-—}"
        echo "End time             : $end_time_display"
        echo "Campaigns discovered : $COUNT_DISCOVERED"
        echo "Campaigns executed   : $COUNT_EXECUTED"
        echo "Campaigns passed     : $COUNT_PASSED"
        echo "Campaigns passed (retry) : $COUNT_RETRY"
        echo "Campaigns blocked    : $COUNT_BLOCKED"
        echo "Last successful      : ${LAST_SUCCESS:-—}"
        echo "Blocked campaign     : ${BLOCKED_CAMPAIGN:-—}"
        echo "Latest commit        : $(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo "unknown")"
        echo "Latest log           : ${LATEST_LOG:-—}"
        echo ""
        echo "Pending campaigns:"
        if [[ -n "$pending_lines" ]]; then
            printf '%s' "$pending_lines"
        else
            echo "  (none)"
        fi
    } > "$LAST_RESULT"
}

# Fires an optional alert hook defined by an environment variable. Non-fatal.
# Exports payload variables so the hook command can inspect queue state.
# Usage: fire_hook <ENV_VAR_NAME>
fire_hook() {
    local hook_var="$1"
    local hook_cmd="${!hook_var:-}"
    [[ -z "$hook_cmd" ]] && return 0

    local timestamp
    timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

    # Export payload variables for hook consumption
    export POSTCAD_QUEUE_STATUS="$QUEUE_FINAL_STATUS"
    export POSTCAD_QUEUE_EXECUTED="$COUNT_EXECUTED"
    export POSTCAD_QUEUE_PASSED="$COUNT_PASSED"
    export POSTCAD_QUEUE_BLOCKED="$COUNT_BLOCKED"
    export POSTCAD_QUEUE_LAST_CAMPAIGN="${LAST_SUCCESS:-}"
    export POSTCAD_QUEUE_LOG_PATH="${LATEST_LOG:-}"

    echo "[HOOK] $hook_var invoking: $hook_cmd" >> "$STATUS_LOG"
    if eval "$hook_cmd" >> "$STATUS_LOG" 2>&1; then
        echo "[HOOK-OK] $hook_var $timestamp" >> "$STATUS_LOG"
    else
        # Hook failure is non-fatal — log and continue
        echo "[HOOK-FAIL] $hook_var $timestamp (non-fatal)" >> "$STATUS_LOG"
    fi
    return 0
}

# ── notification helper ──────────────────────────────────────────────────────────
# Calls ops/notify.sh in a subshell with error suppression — completely non-fatal.
# Usage: notify_event <event> <campaign> <message>
notify_event() {
    local event="$1"
    local campaign="${2:-}"
    local message="${3:-}"
    bash "$ROOT/ops/notify.sh" "$event" "$campaign" "$message" 2>/dev/null || true
}

# ── ensure directories exist ────────────────────────────────────────────────────

mkdir -p "$DONE_DIR" "$LOGS_DIR"

# ── collect campaign files in lexicographic order ───────────────────────────────

mapfile -t CAMPAIGN_FILES < <(
    find "$QUEUE_DIR" -maxdepth 1 -name "*.md" | sort
)

COUNT_DISCOVERED=${#CAMPAIGN_FILES[@]}

if [[ $COUNT_DISCOVERED -eq 0 ]]; then
    QUEUE_FINAL_STATUS="NO_WORK"
    write_summary "NO_WORK"
    echo "[QUEUE-NO_WORK] queue $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$STATUS_LOG"
    echo "Queue is empty. No campaigns to process."
    exit 0
fi

# ── dry-run mode ────────────────────────────────────────────────────────────────

if $DRY_RUN; then
    echo "======================================"
    echo "POSTCAD QUEUE RUNNER — DRY RUN"
    echo "======================================"
    echo ""
    echo "Queue directory : $QUEUE_DIR"
    echo "Campaigns found : $COUNT_DISCOVERED"
    [[ $MAX_COUNT -gt 0 ]] && echo "Max count       : $MAX_COUNT"
    echo ""

    local_count=0
    for campaign_file in "${CAMPAIGN_FILES[@]}"; do
        if [[ $MAX_COUNT -gt 0 && $local_count -ge $MAX_COUNT ]]; then
            echo "(max count reached — remaining campaigns not shown)"
            break
        fi

        campaign_basename="$(basename "$campaign_file")"
        campaign_name="$(extract_campaign_name "$campaign_file")"
        test_cmd="$(extract_test_command "$campaign_file")"

        echo "--------------------------------------"
        echo "Campaign : $campaign_basename"
        echo "Name     : $campaign_name"

        guard_reason="$(check_campaign_guard "$campaign_file" 2>&1)" && guard_pass=true || guard_pass=false
        if $guard_pass; then
            echo "Guard    : PASS (lane-1 safe)"
        else
            echo "Guard    : REJECT — $guard_reason"
        fi

        echo "Would copy campaign to : $CURRENT_CAMPAIGN"
        echo "Would invoke           : $RUN_CAMPAIGN"
        echo "Test command           : ${test_cmd:-<not found>}"
        echo ""

        local_count=$((local_count + 1))
    done

    echo "======================================"
    echo "DRY RUN COMPLETE — no campaigns executed, no files moved"
    echo "======================================"
    exit 0
fi

# ── live queue execution ────────────────────────────────────────────────────────

QUEUE_FINAL_STATUS="RUNNING"

# Write RUNNING status before first campaign so the file is always current
write_summary "RUNNING"

for campaign_file in "${CAMPAIGN_FILES[@]}"; do
    if [[ $MAX_COUNT -gt 0 && $PROCESSED -ge $MAX_COUNT ]]; then
        echo "Max count ($MAX_COUNT) reached. Stopping queue."
        break
    fi

    campaign_basename="$(basename "$campaign_file")"
    campaign_name="$(extract_campaign_name "$campaign_file")"
    log_file="$LOGS_DIR/${campaign_basename%.md}_$(date -u +%Y%m%dT%H%M%SZ).log"
    LATEST_LOG="$log_file"

    echo "======================================"
    echo "CAMPAIGN  : $campaign_basename"
    echo "NAME      : $campaign_name"
    echo "LOG       : $log_file"
    echo "======================================"

    # Guard check
    guard_reason="$(check_campaign_guard "$campaign_file" 2>&1)" && guard_pass=true || guard_pass=false
    if ! $guard_pass; then
        echo "REJECTED  : $guard_reason"
        log_status "REJECTED" "$campaign_basename"
        BLOCKED_CAMPAIGN="$campaign_name"
        COUNT_BLOCKED=$((COUNT_BLOCKED + 1))
        QUEUE_FINAL_STATUS="BLOCKED"
        write_summary "BLOCKED"
        echo "[QUEUE-BLOCKED] queue $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$STATUS_LOG"
        notify_event "guard-blocked" "$campaign_name" "$guard_reason"
        fire_hook "POSTCAD_QUEUE_ON_BLOCKED"
        echo "Queue stopped. Guard rejected $campaign_basename."
        exit 1
    fi

    echo "Guard     : PASS"
    log_status "STARTED" "$campaign_basename"
    COUNT_EXECUTED=$((COUNT_EXECUTED + 1))

    cp "$campaign_file" "$CURRENT_CAMPAIGN"

    # Attempt 1
    echo "Attempt 1 : invoking $RUN_CAMPAIGN"
    if bash "$RUN_CAMPAIGN" > "$log_file" 2>&1; then
        attempt_passed=true
        attempt_tag="PASSED"
    else
        attempt_passed=false
        attempt_tag=""
    fi

    # Attempt 2 (repair pass)
    if ! $attempt_passed; then
        echo "Attempt 2 : repair pass"
        if bash "$RUN_CAMPAIGN" >> "$log_file" 2>&1; then
            attempt_passed=true
            attempt_tag="PASSED-RETRY"
        else
            attempt_passed=false
        fi
    fi

    if $attempt_passed; then
        log_status "$attempt_tag" "$campaign_basename"
        mv "$campaign_file" "$DONE_DIR/"
        LAST_SUCCESS="$campaign_name"
        COUNT_PASSED=$((COUNT_PASSED + 1))
        [[ "$attempt_tag" == "PASSED-RETRY" ]] && COUNT_RETRY=$((COUNT_RETRY + 1))
        echo "$attempt_tag : $campaign_basename"
        PROCESSED=$((PROCESSED + 1))
    else
        log_status "BLOCKED" "$campaign_basename"
        BLOCKED_CAMPAIGN="$campaign_name"
        COUNT_BLOCKED=$((COUNT_BLOCKED + 1))
        QUEUE_FINAL_STATUS="BLOCKED"
        write_summary "BLOCKED"
        echo "[QUEUE-BLOCKED] queue $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$STATUS_LOG"
        notify_event "campaign-failed" "$campaign_name" "failed after 2 attempts — see $log_file"
        fire_hook "POSTCAD_QUEUE_ON_BLOCKED"
        echo "BLOCKED   : $campaign_basename"
        echo "Blocker log: $log_file"
        echo "Queue stopped. Inspect the log above."
        exit 1
    fi
done

# ── terminal state ──────────────────────────────────────────────────────────────

# PARTIAL: --max was set and campaigns remain unprocessed in the queue
remaining_count="$(find "$QUEUE_DIR" -maxdepth 1 -name "*.md" | wc -l | tr -d '[:space:]')"
if [[ $MAX_COUNT -gt 0 && "$remaining_count" -gt 0 && $PROCESSED -gt 0 ]]; then
    QUEUE_FINAL_STATUS="PARTIAL"
else
    QUEUE_FINAL_STATUS="PASSED"
fi

write_summary "$QUEUE_FINAL_STATUS"
echo "[QUEUE-$QUEUE_FINAL_STATUS] queue $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$STATUS_LOG"
notify_event "queue-finished" "queue" "$QUEUE_FINAL_STATUS — $COUNT_PASSED passed, $COUNT_BLOCKED blocked"

if [[ "$QUEUE_FINAL_STATUS" == "PARTIAL" ]]; then
    fire_hook "POSTCAD_QUEUE_ON_PARTIAL"
else
    fire_hook "POSTCAD_QUEUE_ON_SUCCESS"
fi

echo "======================================"
echo "Queue complete. Status: $QUEUE_FINAL_STATUS. Processed: $PROCESSED campaign(s)."
echo "======================================"
