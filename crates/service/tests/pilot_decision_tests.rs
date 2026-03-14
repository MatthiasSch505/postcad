//! Operator decision artifact tests.
//!
//! Checks that verify.sh produces decision artifacts with deterministic
//! structure, correct decision mapping, and stable output wording.
//!
//! Uses include_str! so missing files are compile errors, not runtime failures.

const VERIFY_SH: &str = include_str!("../../../examples/pilot/verify.sh");

const EXPECTED_ACCEPTED: &str =
    include_str!("../../../examples/pilot/testdata/expected_decision_accepted.txt");
const EXPECTED_REJECTED_MISMATCH: &str =
    include_str!("../../../examples/pilot/testdata/expected_decision_rejected_mismatch.txt");
const EXPECTED_REJECTED_MALFORMED: &str =
    include_str!("../../../examples/pilot/testdata/expected_decision_rejected_malformed.txt");

const LOCKED_RECEIPT_HASH: &str =
    "0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb";

// ── verify.sh decision artifact generation ───────────────────────────────────

#[test]
fn verify_sh_has_write_decision_function() {
    assert!(
        VERIFY_SH.contains("_write_decision"),
        "verify.sh must define _write_decision function"
    );
}

#[test]
fn verify_sh_decision_artifact_contains_run_id_field() {
    assert!(
        VERIFY_SH.contains("run_id:"),
        "verify.sh must write 'run_id:' field in decision artifact"
    );
}

#[test]
fn verify_sh_decision_artifact_contains_artifact_field() {
    assert!(
        VERIFY_SH.contains("artifact:"),
        "verify.sh must write 'artifact:' field in decision artifact"
    );
}

#[test]
fn verify_sh_decision_artifact_contains_verification_result_field() {
    assert!(
        VERIFY_SH.contains("verification_result:"),
        "verify.sh must write 'verification_result:' field in decision artifact"
    );
}

#[test]
fn verify_sh_decision_artifact_contains_operator_decision_field() {
    assert!(
        VERIFY_SH.contains("operator_decision:"),
        "verify.sh must write 'operator_decision:' field in decision artifact"
    );
}

#[test]
fn verify_sh_decision_artifact_contains_timestamp_field() {
    assert!(
        VERIFY_SH.contains("timestamp:"),
        "verify.sh must write 'timestamp:' field in decision artifact"
    );
}

#[test]
fn verify_sh_emits_operator_decision_accepted_output() {
    assert!(
        VERIFY_SH.contains("Operator decision: ACCEPTED"),
        "verify.sh must print 'Operator decision: ACCEPTED' on success"
    );
}

#[test]
fn verify_sh_emits_operator_decision_rejected_output() {
    assert!(
        VERIFY_SH.contains("Operator decision: REJECTED"),
        "verify.sh must print 'Operator decision: REJECTED' on failure"
    );
}

#[test]
fn verify_sh_emits_decision_record_path() {
    assert!(
        VERIFY_SH.contains("Decision record:"),
        "verify.sh must print 'Decision record:' path after decision"
    );
}

#[test]
fn verify_sh_uses_reports_dir_variable() {
    assert!(
        VERIFY_SH.contains("REPORTS_DIR"),
        "verify.sh must use REPORTS_DIR variable for decision artifact output"
    );
}

#[test]
fn verify_sh_supports_reports_dir_flag() {
    assert!(
        VERIFY_SH.contains("--reports-dir"),
        "verify.sh must support --reports-dir flag"
    );
}

// ── Decision mapping — verified → accepted ───────────────────────────────────

#[test]
fn verify_sh_maps_verified_to_accepted() {
    assert!(
        VERIFY_SH.contains("verified_for_current_run") && VERIFY_SH.contains("\"accepted\""),
        "verify.sh must map verified_for_current_run to accepted decision"
    );
}

#[test]
fn verify_sh_maps_run_mismatch_to_rejected() {
    assert!(
        VERIFY_SH.contains("belongs_to_different_run") && VERIFY_SH.contains("run_mismatch"),
        "verify.sh must map belongs_to_different_run to rejected with run_mismatch reason"
    );
}

#[test]
fn verify_sh_maps_malformed_to_rejected() {
    assert!(
        VERIFY_SH.contains("\"malformed\""),
        "verify.sh must map malformed to rejected decision"
    );
}

#[test]
fn verify_sh_maps_unverifiable_to_rejected() {
    assert!(
        VERIFY_SH.contains("\"unverifiable\""),
        "verify.sh must map unverifiable to rejected decision"
    );
}

// ── Expected decision fixture content ────────────────────────────────────────

#[test]
fn expected_accepted_decision_has_correct_fields() {
    assert!(
        EXPECTED_ACCEPTED.contains("operator_decision: accepted"),
        "accepted decision fixture must have operator_decision: accepted"
    );
    assert!(
        EXPECTED_ACCEPTED.contains("verification_result: verified_for_current_run"),
        "accepted decision fixture must have verification_result: verified_for_current_run"
    );
    assert!(
        EXPECTED_ACCEPTED.contains(LOCKED_RECEIPT_HASH),
        "accepted decision fixture must contain the locked pilot receipt hash as run_id"
    );
}

#[test]
fn expected_accepted_decision_has_no_reason_field() {
    assert!(
        !EXPECTED_ACCEPTED.contains("reason:"),
        "accepted decision fixture must not have a reason field"
    );
}

#[test]
fn expected_rejected_mismatch_has_correct_fields() {
    assert!(
        EXPECTED_REJECTED_MISMATCH.contains("operator_decision: rejected"),
        "mismatch decision fixture must have operator_decision: rejected"
    );
    assert!(
        EXPECTED_REJECTED_MISMATCH.contains("verification_result: belongs_to_different_run"),
        "mismatch decision fixture must have verification_result: belongs_to_different_run"
    );
    assert!(
        EXPECTED_REJECTED_MISMATCH.contains("reason: run_mismatch"),
        "mismatch decision fixture must have reason: run_mismatch"
    );
}

#[test]
fn expected_rejected_malformed_has_correct_fields() {
    assert!(
        EXPECTED_REJECTED_MALFORMED.contains("operator_decision: rejected"),
        "malformed decision fixture must have operator_decision: rejected"
    );
    assert!(
        EXPECTED_REJECTED_MALFORMED.contains("verification_result: malformed"),
        "malformed decision fixture must have verification_result: malformed"
    );
    assert!(
        EXPECTED_REJECTED_MALFORMED.contains("reason: malformed"),
        "malformed decision fixture must have reason: malformed"
    );
}

#[test]
fn expected_rejected_malformed_uses_unknown_run_id() {
    assert!(
        EXPECTED_REJECTED_MALFORMED.contains("run_id: unknown"),
        "malformed decision fixture must have run_id: unknown (no bundle hash available)"
    );
}

// ── Batch mode also writes decision records ───────────────────────────────────

#[test]
fn verify_sh_batch_writes_decision_records_per_artifact() {
    assert!(
        VERIFY_SH.contains("_write_decision \"$REPORTS_DIR\""),
        "verify.sh batch mode must call _write_decision for each artifact"
    );
}

#[test]
fn verify_sh_batch_prints_decision_records_path_in_summary() {
    assert!(
        VERIFY_SH.contains("Decision records:"),
        "verify.sh batch mode must print 'Decision records:' path in summary"
    );
}
