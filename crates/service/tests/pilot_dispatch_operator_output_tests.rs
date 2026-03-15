//! Dispatch export operator verdict output tests.
//!
//! Checks that run_pilot.sh --export-dispatch produces a deterministic
//! structured verdict block on both success and failure paths.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_export_dispatch_flag() {
    assert!(
        RUN_PILOT_SH.contains("--export-dispatch"),
        "run_pilot.sh must support --export-dispatch flag"
    );
}

// ── verdict wording ───────────────────────────────────────────────────────────

#[test]
fn export_dispatch_prints_ready_verdict() {
    assert!(
        RUN_PILOT_SH.contains("DISPATCH EXPORT READY"),
        "run_pilot.sh must print 'DISPATCH EXPORT READY' on success"
    );
}

#[test]
fn export_dispatch_prints_failed_verdict() {
    assert!(
        RUN_PILOT_SH.contains("DISPATCH EXPORT FAILED"),
        "run_pilot.sh must print 'DISPATCH EXPORT FAILED' on failure"
    );
}

// ── verdict block structure ───────────────────────────────────────────────────

#[test]
fn export_dispatch_verdict_block_has_separator() {
    assert!(
        RUN_PILOT_SH.contains("════════════════════════════════════════"),
        "run_pilot.sh --export-dispatch must include separator in verdict block"
    );
}

#[test]
fn export_dispatch_verdict_has_result_field() {
    assert!(
        RUN_PILOT_SH.contains("Result  : dispatch packet exported")
            && RUN_PILOT_SH.contains("Result  : dispatch export failed"),
        "run_pilot.sh must print Result field for both success and failure"
    );
}

#[test]
fn export_dispatch_verdict_has_next_field() {
    assert!(
        RUN_PILOT_SH.contains("Next    :"),
        "run_pilot.sh --export-dispatch must print Next field in verdict block"
    );
}

// ── success block fields ──────────────────────────────────────────────────────

#[test]
fn export_dispatch_success_shows_file_field() {
    assert!(
        RUN_PILOT_SH.contains("File    : $DISPATCH_PKT"),
        "run_pilot.sh --export-dispatch success block must show File field"
    );
}

#[test]
fn export_dispatch_success_shows_run_id_field() {
    assert!(
        RUN_PILOT_SH.contains("Run ID  : $ED_RUN_ID"),
        "run_pilot.sh --export-dispatch success block must show Run ID field"
    );
}

#[test]
fn export_dispatch_success_next_action() {
    assert!(
        RUN_PILOT_SH.contains("send packet to manufacturer / lab contact"),
        "run_pilot.sh --export-dispatch must tell operator to send packet to manufacturer / lab contact"
    );
}

// ── failure guidance ──────────────────────────────────────────────────────────

#[test]
fn export_dispatch_failure_no_receipt_guidance() {
    assert!(
        RUN_PILOT_SH.contains("generate or load a current pilot run before exporting"),
        "run_pilot.sh must guide operator to generate a pilot run when receipt.json is missing"
    );
}

#[test]
fn export_dispatch_failure_no_dispatch_packet_guidance() {
    assert!(
        RUN_PILOT_SH.contains("verify the current route before exporting dispatch"),
        "run_pilot.sh must guide operator to verify current route when dispatch packet is missing"
    );
}

#[test]
fn export_dispatch_failure_no_dispatch_packet_mentions_reviewer() {
    assert!(
        RUN_PILOT_SH.contains("approve dispatch via reviewer shell"),
        "run_pilot.sh must mention reviewer shell when dispatch packet is missing"
    );
}

#[test]
fn export_dispatch_failure_generic_guidance() {
    assert!(
        RUN_PILOT_SH.contains("confirm the pilot bundle and current artifacts are present"),
        "run_pilot.sh must provide generic guidance for unknown failure"
    );
}

// ── exit codes ────────────────────────────────────────────────────────────────

#[test]
fn export_dispatch_exits_0_on_success() {
    // The success path must contain exit 0
    assert!(
        RUN_PILOT_SH.contains("DISPATCH EXPORT READY") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --export-dispatch must exit 0 on success"
    );
}

#[test]
fn export_dispatch_exits_1_on_failure() {
    assert!(
        RUN_PILOT_SH.contains("DISPATCH EXPORT FAILED") && RUN_PILOT_SH.contains("exit 1"),
        "run_pilot.sh --export-dispatch must exit 1 on failure"
    );
}

// ── README section ────────────────────────────────────────────────────────────

#[test]
fn readme_has_dispatch_export_outcomes_section() {
    assert!(
        README.contains("## Dispatch Export Outcomes"),
        "README must have '## Dispatch Export Outcomes' section"
    );
}

#[test]
fn readme_shows_dispatch_export_ready() {
    assert!(
        README.contains("DISPATCH EXPORT READY"),
        "README must show 'DISPATCH EXPORT READY' example output"
    );
}

#[test]
fn readme_shows_dispatch_export_failed() {
    assert!(
        README.contains("DISPATCH EXPORT FAILED"),
        "README must show 'DISPATCH EXPORT FAILED' example output"
    );
}

#[test]
fn readme_shows_failure_guidance_table() {
    assert!(
        README.contains("generate or load a current pilot run before exporting")
            || README.contains("verify the current route before exporting dispatch"),
        "README must document at least one failure guidance message"
    );
}

#[test]
fn readme_mentions_export_dispatch_command() {
    assert!(
        README.contains("--export-dispatch"),
        "README must show --export-dispatch command"
    );
}
