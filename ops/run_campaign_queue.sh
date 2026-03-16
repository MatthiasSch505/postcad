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

write_last_result() {
    local campaign_name="$1"
    local status="$2"
    local detail="${3:-}"
    {
        echo "# Last Queue Result"
        echo ""
        echo "Campaign : $campaign_name"
        echo "Status   : $status"
        echo "Time     : $(date -u +%Y-%m-%dT%H:%M:%SZ)"
        [[ -n "$detail" ]] && echo "Detail   : $detail"
    } > "$LAST_RESULT"
}

# ── ensure directories exist ────────────────────────────────────────────────────

mkdir -p "$DONE_DIR" "$LOGS_DIR"

# ── collect campaign files in lexicographic order ───────────────────────────────

mapfile -t CAMPAIGN_FILES < <(
    find "$QUEUE_DIR" -maxdepth 1 -name "*.md" | sort
)

if [[ ${#CAMPAIGN_FILES[@]} -eq 0 ]]; then
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
    echo "Campaigns found : ${#CAMPAIGN_FILES[@]}"
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

for campaign_file in "${CAMPAIGN_FILES[@]}"; do
    if [[ $MAX_COUNT -gt 0 && $PROCESSED -ge $MAX_COUNT ]]; then
        echo "Max count ($MAX_COUNT) reached. Stopping queue."
        break
    fi

    campaign_basename="$(basename "$campaign_file")"
    campaign_name="$(extract_campaign_name "$campaign_file")"
    log_file="$LOGS_DIR/${campaign_basename%.md}_$(date -u +%Y%m%dT%H%M%SZ).log"

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
        write_last_result "$campaign_name" "REJECTED" "$guard_reason"
        echo "Queue stopped. Guard rejected $campaign_basename."
        exit 1
    fi

    echo "Guard     : PASS"
    log_status "STARTED" "$campaign_basename"

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
        commit_hash="$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo "unknown")"
        log_status "$attempt_tag" "$campaign_basename"
        write_last_result "$campaign_name" "$attempt_tag" "commit=$commit_hash log=$(basename "$log_file")"
        mv "$campaign_file" "$DONE_DIR/"
        echo "$attempt_tag : $campaign_basename"
        PROCESSED=$((PROCESSED + 1))
    else
        log_status "BLOCKED" "$campaign_basename"
        write_last_result "$campaign_name" "BLOCKED" "log=$(basename "$log_file")"
        echo "BLOCKED   : $campaign_basename"
        echo "Blocker log: $log_file"
        echo "Queue stopped. Inspect the log above."
        exit 1
    fi
done

echo "======================================"
echo "Queue complete. Processed: $PROCESSED campaign(s)."
echo "======================================"