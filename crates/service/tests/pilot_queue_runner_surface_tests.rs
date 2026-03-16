//! Queue runner surface tests.
//!
//! Verifies that ops/run_campaign_queue.sh implements the required
//! lane-1 guard, dry-run, retry, blocked, max-count, summary, and
//! alert-hook behaviors by inspecting the script source.

const QUEUE_RUNNER: &str = include_str!("../../../ops/run_campaign_queue.sh");

// ── script exists ──────────────────────────────────────────────────────────────

#[test]
fn queue_runner_script_exists() {
    assert!(!QUEUE_RUNNER.is_empty(), "run_campaign_queue.sh must not be empty");
}

// ── argument flags ─────────────────────────────────────────────────────────────

#[test]
fn queue_runner_supports_dry_run_flag() {
    assert!(
        QUEUE_RUNNER.contains("--dry-run"),
        "run_campaign_queue.sh must support --dry-run flag"
    );
}

#[test]
fn queue_runner_supports_max_flag() {
    assert!(
        QUEUE_RUNNER.contains("--max"),
        "run_campaign_queue.sh must support --max flag"
    );
}

// ── dry-run mode ───────────────────────────────────────────────────────────────

#[test]
fn dry_run_prints_header() {
    assert!(
        QUEUE_RUNNER.contains("POSTCAD QUEUE RUNNER — DRY RUN"),
        "dry-run must print 'POSTCAD QUEUE RUNNER — DRY RUN' header"
    );
}

#[test]
fn dry_run_prints_completion_footer() {
    assert!(
        QUEUE_RUNNER.contains("DRY RUN COMPLETE"),
        "dry-run must print 'DRY RUN COMPLETE' footer"
    );
}

#[test]
fn dry_run_shows_guard_pass_label() {
    assert!(
        QUEUE_RUNNER.contains("Guard    : PASS"),
        "dry-run must print 'Guard    : PASS' when campaign passes guard"
    );
}

#[test]
fn dry_run_shows_guard_reject_label() {
    assert!(
        QUEUE_RUNNER.contains("Guard    : REJECT"),
        "dry-run must print 'Guard    : REJECT' when campaign fails guard"
    );
}

#[test]
fn dry_run_shows_would_invoke_run_campaign() {
    assert!(
        QUEUE_RUNNER.contains("Would invoke"),
        "dry-run must show which command would be invoked"
    );
}

#[test]
fn dry_run_exits_without_executing() {
    // dry-run block must reach exit 0 before live execution loop
    let dry_run_start = QUEUE_RUNNER
        .find("POSTCAD QUEUE RUNNER — DRY RUN")
        .expect("dry-run header must exist");
    // live execution loop starts after the dry-run fi block
    let live_start = QUEUE_RUNNER
        .find("live queue execution")
        .expect("live queue execution section must exist");
    let dry_run_block = &QUEUE_RUNNER[dry_run_start..live_start];
    assert!(
        dry_run_block.contains("exit 0"),
        "dry-run block must exit 0 before live execution"
    );
}

#[test]
fn dry_run_no_files_moved_annotation() {
    assert!(
        QUEUE_RUNNER.contains("no files moved"),
        "dry-run footer must state 'no files moved'"
    );
}

// ── forbidden path guard ───────────────────────────────────────────────────────

#[test]
fn queue_runner_defines_forbidden_kernel_paths_array() {
    assert!(
        QUEUE_RUNNER.contains("FORBIDDEN_KERNEL_PATHS"),
        "must define FORBIDDEN_KERNEL_PATHS array"
    );
}

#[test]
fn queue_runner_forbids_crates_core() {
    assert!(
        QUEUE_RUNNER.contains("\"crates/core\""),
        "FORBIDDEN_KERNEL_PATHS must include crates/core"
    );
}

#[test]
fn queue_runner_forbids_crates_routing() {
    assert!(
        QUEUE_RUNNER.contains("\"crates/routing\""),
        "FORBIDDEN_KERNEL_PATHS must include crates/routing"
    );
}

#[test]
fn queue_runner_forbids_crates_compliance() {
    assert!(
        QUEUE_RUNNER.contains("\"crates/compliance\""),
        "FORBIDDEN_KERNEL_PATHS must include crates/compliance"
    );
}

#[test]
fn queue_runner_forbids_crates_audit() {
    assert!(
        QUEUE_RUNNER.contains("\"crates/audit\""),
        "FORBIDDEN_KERNEL_PATHS must include crates/audit"
    );
}

#[test]
fn queue_runner_forbids_crates_registry() {
    assert!(
        QUEUE_RUNNER.contains("\"crates/registry\""),
        "FORBIDDEN_KERNEL_PATHS must include crates/registry"
    );
}

#[test]
fn queue_runner_has_is_kernel_path_function() {
    assert!(
        QUEUE_RUNNER.contains("is_kernel_path"),
        "must define is_kernel_path() guard function"
    );
}

#[test]
fn queue_runner_has_is_lane1_path_function() {
    assert!(
        QUEUE_RUNNER.contains("is_lane1_path"),
        "must define is_lane1_path() boundary check"
    );
}

#[test]
fn queue_runner_lane1_allows_examples_pilot() {
    let after = &QUEUE_RUNNER[QUEUE_RUNNER
        .find("is_lane1_path()")
        .expect("is_lane1_path() must exist")..];
    assert!(
        after.contains("examples/pilot/"),
        "is_lane1_path must allow examples/pilot/"
    );
}

#[test]
fn queue_runner_lane1_allows_ops() {
    let after = &QUEUE_RUNNER[QUEUE_RUNNER
        .find("is_lane1_path()")
        .expect("is_lane1_path() must exist")..];
    assert!(after.contains("ops/"), "is_lane1_path must allow ops/");
}

#[test]
fn queue_runner_lane1_allows_docs() {
    let after = &QUEUE_RUNNER[QUEUE_RUNNER
        .find("is_lane1_path()")
        .expect("is_lane1_path() must exist")..];
    assert!(after.contains("docs/"), "is_lane1_path must allow docs/");
}

#[test]
fn queue_runner_lane1_allows_surface_tests() {
    let after = &QUEUE_RUNNER[QUEUE_RUNNER
        .find("is_lane1_path()")
        .expect("is_lane1_path() must exist")..];
    assert!(
        after.contains("surface_tests"),
        "is_lane1_path must allow crates/service/tests/*surface_tests.rs"
    );
}

#[test]
fn queue_runner_emits_kernel_path_forbidden_on_rejection() {
    assert!(
        QUEUE_RUNNER.contains("KERNEL PATH FORBIDDEN"),
        "must emit 'KERNEL PATH FORBIDDEN' message on kernel path rejection"
    );
}

#[test]
fn queue_runner_emits_outside_lane1_on_rejection() {
    assert!(
        QUEUE_RUNNER.contains("OUTSIDE LANE-1 BOUNDARY"),
        "must emit 'OUTSIDE LANE-1 BOUNDARY' message on lane-1 boundary violation"
    );
}

#[test]
fn queue_runner_logs_rejected_status() {
    assert!(
        QUEUE_RUNNER.contains("\"REJECTED\""),
        "must log REJECTED status on guard failure"
    );
}

// ── success path ───────────────────────────────────────────────────────────────

#[test]
fn queue_runner_logs_started_before_execution() {
    assert!(
        QUEUE_RUNNER.contains("\"STARTED\""),
        "must log STARTED status before executing campaign"
    );
}

#[test]
fn queue_runner_copies_campaign_to_current_campaign() {
    assert!(
        QUEUE_RUNNER.contains("CURRENT_CAMPAIGN"),
        "must copy campaign file to CURRENT_CAMPAIGN (current_campaign.md)"
    );
}

#[test]
fn queue_runner_invokes_run_campaign_sh() {
    assert!(
        QUEUE_RUNNER.contains("RUN_CAMPAIGN"),
        "must invoke RUN_CAMPAIGN (ops/run_campaign.sh)"
    );
}

#[test]
fn queue_runner_moves_campaign_to_done_on_success() {
    assert!(
        QUEUE_RUNNER.contains("DONE_DIR"),
        "must move campaign file to DONE_DIR on success"
    );
}

#[test]
fn queue_runner_logs_passed_on_success() {
    assert!(
        QUEUE_RUNNER.contains("\"PASSED\""),
        "must log PASSED status on successful campaign execution"
    );
}

#[test]
fn queue_runner_writes_summary_on_success() {
    assert!(
        QUEUE_RUNNER.contains("write_summary"),
        "must call write_summary() to update last_result.md"
    );
}

#[test]
fn queue_runner_records_commit_hash_on_success() {
    assert!(
        QUEUE_RUNNER.contains("rev-parse --short HEAD"),
        "must record commit hash in summary via git rev-parse --short HEAD"
    );
}

// ── retry-then-pass path ───────────────────────────────────────────────────────

#[test]
fn queue_runner_has_attempt_1() {
    assert!(
        QUEUE_RUNNER.contains("Attempt 1"),
        "must label first attempt as 'Attempt 1'"
    );
}

#[test]
fn queue_runner_has_repair_pass_attempt_2() {
    assert!(
        QUEUE_RUNNER.contains("Attempt 2") && QUEUE_RUNNER.contains("repair pass"),
        "must label second attempt as 'Attempt 2 : repair pass'"
    );
}

#[test]
fn queue_runner_logs_passed_retry_on_repair_success() {
    assert!(
        QUEUE_RUNNER.contains("PASSED-RETRY"),
        "must log PASSED-RETRY when repair pass succeeds after first failure"
    );
}

// ── blocked-after-retry path ───────────────────────────────────────────────────

#[test]
fn queue_runner_logs_blocked_after_both_attempts_fail() {
    assert!(
        QUEUE_RUNNER.contains("\"BLOCKED\""),
        "must log BLOCKED status after both attempts fail"
    );
}

#[test]
fn queue_runner_exits_nonzero_on_blocked() {
    assert!(
        QUEUE_RUNNER.contains("exit 1"),
        "must exit 1 on BLOCKED to signal failure to caller"
    );
}

#[test]
fn queue_runner_reports_blocker_log_path() {
    assert!(
        QUEUE_RUNNER.contains("Blocker log"),
        "must report blocker log path on BLOCKED"
    );
}

// ── max-count behavior ─────────────────────────────────────────────────────────

#[test]
fn queue_runner_enforces_max_count() {
    assert!(
        QUEUE_RUNNER.contains("MAX_COUNT"),
        "must track and enforce MAX_COUNT limit"
    );
}

#[test]
fn queue_runner_max_count_prints_stop_message() {
    assert!(
        QUEUE_RUNNER.contains("Max count") && QUEUE_RUNNER.contains("reached"),
        "must print 'Max count ... reached' message when limit is hit"
    );
}

// ── per-campaign log directory ─────────────────────────────────────────────────

#[test]
fn queue_runner_defines_logs_dir() {
    assert!(
        QUEUE_RUNNER.contains("LOGS_DIR"),
        "must define LOGS_DIR for per-campaign log storage"
    );
}

#[test]
fn queue_runner_writes_per_campaign_log_file() {
    assert!(
        QUEUE_RUNNER.contains("log_file"),
        "must write a per-campaign log_file under LOGS_DIR"
    );
}

// ── non-interactive safety ─────────────────────────────────────────────────────

#[test]
fn queue_runner_is_noninteractive() {
    assert!(
        !QUEUE_RUNNER.contains("read -p"),
        "must not prompt for input — queue runner must be non-interactive"
    );
}

#[test]
fn queue_runner_uses_set_euo_pipefail() {
    assert!(
        QUEUE_RUNNER.contains("set -euo pipefail"),
        "must use 'set -euo pipefail' for safe unattended execution"
    );
}

// ── summary initialization ─────────────────────────────────────────────────────

#[test]
fn queue_runner_defines_write_summary_function() {
    assert!(
        QUEUE_RUNNER.contains("write_summary()"),
        "must define write_summary() to write last_result.md"
    );
}

#[test]
fn queue_runner_initializes_running_status_before_first_campaign() {
    // write_summary "RUNNING" must appear before the campaign loop body
    let running_call = QUEUE_RUNNER
        .find("write_summary \"RUNNING\"")
        .expect("write_summary \"RUNNING\" must exist");
    let loop_body = QUEUE_RUNNER
        .find("live queue execution")
        .expect("live queue execution section must exist");
    assert!(
        running_call > loop_body,
        "write_summary \"RUNNING\" must be called inside the live execution section, before the loop body"
    );
}

#[test]
fn queue_runner_summary_has_status_field() {
    let after = &QUEUE_RUNNER[QUEUE_RUNNER
        .find("write_summary()")
        .expect("write_summary() must exist")..];
    assert!(
        after.contains("Status               :"),
        "write_summary must emit 'Status               :' field"
    );
}

#[test]
fn queue_runner_summary_has_start_time_field() {
    let after = &QUEUE_RUNNER[QUEUE_RUNNER
        .find("write_summary()")
        .expect("write_summary() must exist")..];
    assert!(
        after.contains("Start time"),
        "write_summary must emit 'Start time' field"
    );
}

#[test]
fn queue_runner_summary_has_end_time_field() {
    let after = &QUEUE_RUNNER[QUEUE_RUNNER
        .find("write_summary()")
        .expect("write_summary() must exist")..];
    assert!(
        after.contains("End time"),
        "write_summary must emit 'End time' field"
    );
}

#[test]
fn queue_runner_summary_tracks_discovered() {
    assert!(
        QUEUE_RUNNER.contains("COUNT_DISCOVERED"),
        "must track COUNT_DISCOVERED"
    );
}

#[test]
fn queue_runner_summary_tracks_executed() {
    assert!(
        QUEUE_RUNNER.contains("COUNT_EXECUTED"),
        "must track COUNT_EXECUTED"
    );
}

#[test]
fn queue_runner_summary_tracks_passed() {
    assert!(
        QUEUE_RUNNER.contains("COUNT_PASSED"),
        "must track COUNT_PASSED"
    );
}

#[test]
fn queue_runner_summary_tracks_retry() {
    assert!(
        QUEUE_RUNNER.contains("COUNT_RETRY"),
        "must track COUNT_RETRY for repair-pass successes"
    );
}

#[test]
fn queue_runner_summary_tracks_blocked() {
    assert!(
        QUEUE_RUNNER.contains("COUNT_BLOCKED"),
        "must track COUNT_BLOCKED"
    );
}

#[test]
fn queue_runner_summary_tracks_last_successful_campaign() {
    assert!(
        QUEUE_RUNNER.contains("LAST_SUCCESS"),
        "must track LAST_SUCCESS campaign name"
    );
}

#[test]
fn queue_runner_summary_tracks_blocked_campaign_name() {
    assert!(
        QUEUE_RUNNER.contains("BLOCKED_CAMPAIGN"),
        "must track BLOCKED_CAMPAIGN name"
    );
}

#[test]
fn queue_runner_summary_lists_pending_campaigns() {
    let after = &QUEUE_RUNNER[QUEUE_RUNNER
        .find("write_summary()")
        .expect("write_summary() must exist")..];
    assert!(
        after.contains("Pending campaigns"),
        "write_summary must list 'Pending campaigns' in deterministic order"
    );
}

#[test]
fn queue_runner_summary_pending_list_is_sorted() {
    // pending campaigns are built from `find ... | sort`
    let after = &QUEUE_RUNNER[QUEUE_RUNNER
        .find("Pending campaigns")
        .expect("Pending campaigns section must exist")..];
    assert!(
        after.contains("sort"),
        "pending campaign list must be produced by a sorted find"
    );
}

#[test]
fn queue_runner_writes_passed_summary() {
    assert!(
        QUEUE_RUNNER.contains("write_summary \"$QUEUE_FINAL_STATUS\""),
        "must call write_summary with final status at queue completion"
    );
}

#[test]
fn queue_runner_writes_blocked_summary() {
    assert!(
        QUEUE_RUNNER.contains("write_summary \"BLOCKED\""),
        "must call write_summary \"BLOCKED\" when queue is blocked"
    );
}

#[test]
fn queue_runner_partial_status_on_max_with_remaining() {
    assert!(
        QUEUE_RUNNER.contains("PARTIAL"),
        "must set PARTIAL status when --max stops the queue with campaigns remaining"
    );
}

// ── alert hooks ────────────────────────────────────────────────────────────────

#[test]
fn queue_runner_defines_fire_hook_function() {
    assert!(
        QUEUE_RUNNER.contains("fire_hook()"),
        "must define fire_hook() function for optional alert hooks"
    );
}

#[test]
fn queue_runner_supports_on_success_hook_var() {
    assert!(
        QUEUE_RUNNER.contains("POSTCAD_QUEUE_ON_SUCCESS"),
        "must support POSTCAD_QUEUE_ON_SUCCESS environment variable hook"
    );
}

#[test]
fn queue_runner_supports_on_blocked_hook_var() {
    assert!(
        QUEUE_RUNNER.contains("POSTCAD_QUEUE_ON_BLOCKED"),
        "must support POSTCAD_QUEUE_ON_BLOCKED environment variable hook"
    );
}

#[test]
fn queue_runner_supports_on_partial_hook_var() {
    assert!(
        QUEUE_RUNNER.contains("POSTCAD_QUEUE_ON_PARTIAL"),
        "must support POSTCAD_QUEUE_ON_PARTIAL environment variable hook"
    );
}

#[test]
fn queue_runner_fires_hook_on_success() {
    // fire_hook must be called after the PASSED/PARTIAL terminal state
    let terminal = QUEUE_RUNNER
        .find("terminal state")
        .expect("terminal state section must exist");
    let after = &QUEUE_RUNNER[terminal..];
    assert!(
        after.contains("fire_hook"),
        "fire_hook must be called in the terminal state section"
    );
}

#[test]
fn queue_runner_fires_hook_on_blocked() {
    // fire_hook "POSTCAD_QUEUE_ON_BLOCKED" must appear in blocked paths
    assert!(
        QUEUE_RUNNER.contains("fire_hook \"POSTCAD_QUEUE_ON_BLOCKED\""),
        "must call fire_hook \"POSTCAD_QUEUE_ON_BLOCKED\" when campaign is blocked"
    );
}

#[test]
fn queue_runner_fires_hook_on_partial() {
    assert!(
        QUEUE_RUNNER.contains("fire_hook \"POSTCAD_QUEUE_ON_PARTIAL\""),
        "must call fire_hook \"POSTCAD_QUEUE_ON_PARTIAL\" when queue stops at max"
    );
}

#[test]
fn queue_runner_hook_is_nonfatal() {
    // hook failure must be caught and logged, not propagated
    let after = &QUEUE_RUNNER[QUEUE_RUNNER
        .find("fire_hook()")
        .expect("fire_hook() must exist")..];
    assert!(
        after.contains("non-fatal"),
        "fire_hook must treat hook execution failure as non-fatal"
    );
}

#[test]
fn queue_runner_hook_does_not_change_exit_code() {
    // fire_hook must end with `return 0` so it can never fail the caller
    let after = &QUEUE_RUNNER[QUEUE_RUNNER
        .find("fire_hook()")
        .expect("fire_hook() must exist")..];
    assert!(
        after.contains("return 0"),
        "fire_hook must return 0 so hook failure never affects queue exit code"
    );
}

#[test]
fn queue_runner_hook_exports_status_var() {
    assert!(
        QUEUE_RUNNER.contains("export POSTCAD_QUEUE_STATUS"),
        "fire_hook must export POSTCAD_QUEUE_STATUS for hook commands"
    );
}

#[test]
fn queue_runner_hook_exports_executed_var() {
    assert!(
        QUEUE_RUNNER.contains("export POSTCAD_QUEUE_EXECUTED"),
        "fire_hook must export POSTCAD_QUEUE_EXECUTED for hook commands"
    );
}

#[test]
fn queue_runner_hook_exports_passed_var() {
    assert!(
        QUEUE_RUNNER.contains("export POSTCAD_QUEUE_PASSED"),
        "fire_hook must export POSTCAD_QUEUE_PASSED for hook commands"
    );
}

#[test]
fn queue_runner_hook_exports_blocked_var() {
    assert!(
        QUEUE_RUNNER.contains("export POSTCAD_QUEUE_BLOCKED"),
        "fire_hook must export POSTCAD_QUEUE_BLOCKED for hook commands"
    );
}

#[test]
fn queue_runner_hook_exports_last_campaign_var() {
    assert!(
        QUEUE_RUNNER.contains("export POSTCAD_QUEUE_LAST_CAMPAIGN"),
        "fire_hook must export POSTCAD_QUEUE_LAST_CAMPAIGN for hook commands"
    );
}

#[test]
fn queue_runner_hook_exports_log_path_var() {
    assert!(
        QUEUE_RUNNER.contains("export POSTCAD_QUEUE_LOG_PATH"),
        "fire_hook must export POSTCAD_QUEUE_LOG_PATH for hook commands"
    );
}

#[test]
fn queue_runner_hook_logs_invocation_to_status_log() {
    let after = &QUEUE_RUNNER[QUEUE_RUNNER
        .find("fire_hook()")
        .expect("fire_hook() must exist")..];
    assert!(
        after.contains("STATUS_LOG"),
        "fire_hook must log hook invocation to STATUS_LOG"
    );
}

#[test]
fn queue_runner_hook_logs_hook_ok() {
    assert!(
        QUEUE_RUNNER.contains("HOOK-OK"),
        "fire_hook must log HOOK-OK on successful hook execution"
    );
}

#[test]
fn queue_runner_hook_logs_hook_fail() {
    assert!(
        QUEUE_RUNNER.contains("HOOK-FAIL"),
        "fire_hook must log HOOK-FAIL on failed hook execution"
    );
}
