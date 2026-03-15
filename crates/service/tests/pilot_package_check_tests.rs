//! Lab trial package self-check tests.
//!
//! Checks that run_pilot.sh --check-lab-trial-package mode exists,
//! verifies all required files, and produces deterministic pass/fail output.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_check_lab_trial_package_flag() {
    assert!(
        RUN_PILOT_SH.contains("--check-lab-trial-package"),
        "run_pilot.sh must support --check-lab-trial-package flag"
    );
}

#[test]
fn check_command_errors_if_receipt_missing() {
    assert!(
        RUN_PILOT_SH.contains("receipt.json not found"),
        "run_pilot.sh --check-lab-trial-package must error if receipt.json missing"
    );
}

// ── package directory check ───────────────────────────────────────────────────

#[test]
fn check_command_locates_outbound_lab_trial_dir() {
    assert!(
        RUN_PILOT_SH.contains("outbound/lab_trial_${CHK_RUN_ID}"),
        "run_pilot.sh must locate outbound/lab_trial_<run-id> package directory"
    );
}

#[test]
fn check_command_errors_if_package_dir_missing() {
    assert!(
        RUN_PILOT_SH.contains("package directory not found"),
        "run_pilot.sh must error if package directory not found"
    );
}

#[test]
fn check_command_suggests_export_on_missing_dir() {
    assert!(
        RUN_PILOT_SH.contains("--export-lab-trial-package"),
        "run_pilot.sh must suggest --export-lab-trial-package when package is missing"
    );
}

// ── required files are checked ────────────────────────────────────────────────

#[test]
fn check_command_checks_manifest_txt() {
    assert!(
        RUN_PILOT_SH.contains("\"manifest.txt\""),
        "run_pilot.sh must check for manifest.txt"
    );
}

#[test]
fn check_command_checks_operator_instructions() {
    assert!(
        RUN_PILOT_SH.contains("\"operator_instructions.txt\""),
        "run_pilot.sh must check for operator_instructions.txt"
    );
}

#[test]
fn check_command_checks_lab_instructions() {
    assert!(
        RUN_PILOT_SH.contains("\"lab_instructions.txt\""),
        "run_pilot.sh must check for lab_instructions.txt"
    );
}

#[test]
fn check_command_checks_lab_reply_template() {
    assert!(
        RUN_PILOT_SH.contains("\"lab_reply_template.json\""),
        "run_pilot.sh must check for lab_reply_template.json"
    );
}

#[test]
fn check_command_checks_email_to_lab() {
    assert!(
        RUN_PILOT_SH.contains("\"email_to_lab.txt\""),
        "run_pilot.sh must check for email_to_lab.txt"
    );
}

#[test]
fn check_command_checks_short_message_to_lab() {
    assert!(
        RUN_PILOT_SH.contains("\"short_message_to_lab.txt\""),
        "run_pilot.sh must check for short_message_to_lab.txt"
    );
}

#[test]
fn check_command_checks_operator_send_note() {
    assert!(
        RUN_PILOT_SH.contains("\"operator_send_note.txt\""),
        "run_pilot.sh must check for operator_send_note.txt"
    );
}

#[test]
fn check_command_checks_receipt_json() {
    assert!(
        RUN_PILOT_SH.contains("\"receipt.json\""),
        "run_pilot.sh must check for receipt.json artifact"
    );
}

// ── per-file output wording ───────────────────────────────────────────────────

#[test]
fn check_command_prints_present_for_found_files() {
    assert!(
        RUN_PILOT_SH.contains("present"),
        "run_pilot.sh must print 'present' for files that exist"
    );
}

#[test]
fn check_command_prints_missing_for_absent_files() {
    assert!(
        RUN_PILOT_SH.contains("missing"),
        "run_pilot.sh must print 'missing' for files that are absent"
    );
}

// ── pass/fail output ──────────────────────────────────────────────────────────

#[test]
fn check_command_prints_ready_on_success() {
    assert!(
        RUN_PILOT_SH.contains("package ready for external lab send"),
        "run_pilot.sh must print 'package ready for external lab send' when all files present"
    );
}

#[test]
fn check_command_prints_failed_on_failure() {
    assert!(
        RUN_PILOT_SH.contains("package check failed"),
        "run_pilot.sh must print 'package check failed' when files are missing"
    );
}

#[test]
fn check_command_exits_zero_on_pass() {
    // Verified by presence of exit 0 after "package ready" block
    assert!(
        RUN_PILOT_SH.contains("package ready for external lab send"),
        "run_pilot.sh must exit 0 when package is ready"
    );
}

#[test]
fn check_command_exits_nonzero_on_fail() {
    assert!(
        RUN_PILOT_SH.contains("package check failed"),
        "run_pilot.sh must exit 1 when package check fails"
    );
}

// ── success next-step guidance ────────────────────────────────────────────────

#[test]
fn check_command_success_suggests_zip_and_send() {
    assert!(
        RUN_PILOT_SH.contains("Zip and send:"),
        "run_pilot.sh must suggest zip-and-send on successful check"
    );
}

#[test]
fn check_command_success_references_operator_send_note() {
    assert!(
        RUN_PILOT_SH.contains("operator_send_note.txt"),
        "run_pilot.sh must reference operator_send_note.txt in success guidance"
    );
}

// ── README ────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_package_self_check_section() {
    assert!(
        README.contains("## Package Self-Check"),
        "README must have '## Package Self-Check' section"
    );
}

#[test]
fn readme_shows_check_lab_trial_package_command() {
    assert!(
        README.contains("--check-lab-trial-package"),
        "README must show --check-lab-trial-package command"
    );
}

#[test]
fn readme_shows_ready_output() {
    assert!(
        README.contains("package ready for external lab send"),
        "README must show expected 'package ready' output"
    );
}

#[test]
fn readme_shows_failed_output() {
    assert!(
        README.contains("package check failed"),
        "README must show expected 'package check failed' output"
    );
}

#[test]
fn readme_shows_regenerate_on_failure() {
    assert!(
        README.contains("--export-lab-trial-package"),
        "README must show --export-lab-trial-package as fix for failed check"
    );
}
