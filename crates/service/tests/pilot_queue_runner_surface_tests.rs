//! Queue runner surface tests.
//!
//! Verifies that ops/run_campaign_queue.sh implements the required
//! lane-1 guard, dry-run, retry, blocked, and max-count behaviors
//! by inspecting the script source.

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
fn queue_runner_writes_last_result_on_success() {
    assert!(
        QUEUE_RUNNER.contains("write_last_result"),
        "must call write_last_result() to update last_result.md"
    );
}

#[test]
fn queue_runner_records_commit_hash_on_success() {
    assert!(
        QUEUE_RUNNER.contains("commit_hash"),
        "must record commit hash in last result on success"
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
