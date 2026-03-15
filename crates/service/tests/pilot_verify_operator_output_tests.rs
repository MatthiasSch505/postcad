//! Verification operator verdict output tests.
//!
//! Checks that verify.sh produces deterministic verdict blocks for both
//! receipt verification and inbound lab response verification modes.

const VERIFY_SH: &str = include_str!("../../../examples/pilot/verify.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── verdict wording ───────────────────────────────────────────────────────────

#[test]
fn verify_sh_prints_verification_passed() {
    assert!(
        VERIFY_SH.contains("VERIFICATION PASSED"),
        "verify.sh must print 'VERIFICATION PASSED' on success"
    );
}

#[test]
fn verify_sh_prints_verification_failed() {
    assert!(
        VERIFY_SH.contains("VERIFICATION FAILED"),
        "verify.sh must print 'VERIFICATION FAILED' on failure"
    );
}

// ── verdict block structure ───────────────────────────────────────────────────

#[test]
fn verify_sh_verdict_block_has_separator() {
    assert!(
        VERIFY_SH.contains("════════════════════════════════════════"),
        "verify.sh must include separator line in verdict block"
    );
}

#[test]
fn verify_sh_verdict_block_has_result_field() {
    assert!(
        VERIFY_SH.contains("Result  : verification passed")
            && VERIFY_SH.contains("Result  : verification failed"),
        "verify.sh must print Result field for both passed and failed"
    );
}

#[test]
fn verify_sh_verdict_block_has_next_field() {
    assert!(
        VERIFY_SH.contains("Next    :"),
        "verify.sh must print Next field in verdict block"
    );
}

// ── inbound mode verdict ──────────────────────────────────────────────────────

#[test]
fn inbound_verdict_block_has_inbound_path_field() {
    assert!(
        VERIFY_SH.contains("Inbound : $INBOUND_FILE"),
        "verify.sh inbound verdict must print Inbound path"
    );
}

#[test]
fn inbound_verdict_block_has_bundle_path_field() {
    assert!(
        VERIFY_SH.contains("Bundle  : $BUNDLE_DIR"),
        "verify.sh inbound verdict must print Bundle path"
    );
}

#[test]
fn inbound_verdict_pass_next_action() {
    assert!(
        VERIFY_SH.contains("operator may export dispatch packet"),
        "verify.sh must tell operator they may export dispatch packet on pass"
    );
}

// ── inbound failure guidance ──────────────────────────────────────────────────

#[test]
fn inbound_verdict_unverifiable_missing_file_guidance() {
    assert!(
        VERIFY_SH.contains("check inbound reply file path and rerun"),
        "verify.sh must guide operator to check inbound reply file path when file not found"
    );
}

#[test]
fn inbound_verdict_unverifiable_invalid_json_guidance() {
    assert!(
        VERIFY_SH.contains("inspect inbound reply before verifying"),
        "verify.sh must suggest inspect command when reply is not valid JSON"
    );
}

#[test]
fn inbound_verdict_unverifiable_bundle_path_guidance() {
    assert!(
        VERIFY_SH.contains("confirm the pilot bundle path is correct"),
        "verify.sh must guide operator to confirm bundle path when bundle directory missing"
    );
}

#[test]
fn inbound_verdict_malformed_guidance() {
    assert!(
        VERIFY_SH.contains("ask the lab to resend a complete reply if fields are unreadable"),
        "verify.sh must guide operator to ask lab for resend when reply is malformed"
    );
}

#[test]
fn inbound_verdict_run_mismatch_guidance() {
    assert!(
        VERIFY_SH.contains("confirm the lab returned the reply for the current run"),
        "verify.sh must guide operator to confirm reply matches current run on run_mismatch"
    );
}

// ── receipt mode verdict ──────────────────────────────────────────────────────

#[test]
fn receipt_verdict_has_receipt_path_field() {
    assert!(
        VERIFY_SH.contains("Receipt : ${SCRIPT_DIR}/receipt.json"),
        "verify.sh receipt verdict must print Receipt path"
    );
}

#[test]
fn receipt_verdict_pass_next_action() {
    // Same string appears in both inbound and receipt modes
    assert!(
        VERIFY_SH.contains("operator may export dispatch packet"),
        "verify.sh receipt verdict must tell operator they may export dispatch packet on pass"
    );
}

#[test]
fn receipt_verdict_fail_guidance() {
    assert!(
        VERIFY_SH.contains("check receipt.json and input files, then rerun"),
        "verify.sh must guide operator to check receipt.json on failure"
    );
}

// ── README section ────────────────────────────────────────────────────────────

#[test]
fn readme_has_verification_verdict_section() {
    assert!(
        README.contains("## Verification Verdict Output"),
        "README must have '## Verification Verdict Output' section"
    );
}

#[test]
fn readme_shows_verification_passed_output() {
    assert!(
        README.contains("VERIFICATION PASSED"),
        "README must show 'VERIFICATION PASSED' example output"
    );
}

#[test]
fn readme_shows_verification_failed_output() {
    assert!(
        README.contains("VERIFICATION FAILED"),
        "README must show 'VERIFICATION FAILED' example output"
    );
}

#[test]
fn readme_shows_failure_guidance() {
    assert!(
        README.contains("check inbound reply file path")
            || README.contains("ask the lab to resend")
            || README.contains("confirm the lab returned"),
        "README must document at least one failure guidance message"
    );
}
