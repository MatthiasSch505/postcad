//! Operator intake triage tests.
//!
//! Checks that verify.sh --batch-inbound mode has the correct classifications,
//! wording, and structural properties. Also validates the inbound testdata
//! fixtures used for batch triage testing.
//!
//! Uses include_str! so missing files are compile errors, not runtime failures.

const VERIFY_SH: &str = include_str!("../../../examples/pilot/verify.sh");

const INBOUND_A: &str =
    include_str!("../../../examples/pilot/testdata/inbound/response_a.json");
const INBOUND_B: &str =
    include_str!("../../../examples/pilot/testdata/inbound/response_b.json");
const INBOUND_C: &str =
    include_str!("../../../examples/pilot/testdata/inbound/response_c.json");
const INBOUND_DUP: &str =
    include_str!("../../../examples/pilot/testdata/inbound/response_dup.json");

const LOCKED_RECEIPT_HASH: &str =
    "0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb";

// ── verify.sh batch mode ──────────────────────────────────────────────────────

#[test]
fn verify_sh_supports_batch_inbound_mode() {
    assert!(
        VERIFY_SH.contains("--batch-inbound"),
        "verify.sh must support --batch-inbound flag"
    );
}

#[test]
fn verify_sh_batch_emits_accepted_classification() {
    assert!(
        VERIFY_SH.contains("\"accepted\""),
        "verify.sh must use 'accepted' classification string"
    );
}

#[test]
fn verify_sh_batch_emits_mismatch_classification() {
    assert!(
        VERIFY_SH.contains("\"mismatch\""),
        "verify.sh must use 'mismatch' classification string"
    );
}

#[test]
fn verify_sh_batch_emits_malformed_classification() {
    assert!(
        VERIFY_SH.contains("\"malformed\""),
        "verify.sh must use 'malformed' classification string"
    );
}

#[test]
fn verify_sh_batch_emits_unverifiable_classification() {
    assert!(
        VERIFY_SH.contains("\"unverifiable\""),
        "verify.sh must use 'unverifiable' classification string"
    );
}

#[test]
fn verify_sh_batch_emits_duplicate_classification() {
    assert!(
        VERIFY_SH.contains("\"duplicate\""),
        "verify.sh must use 'duplicate' classification string"
    );
}

#[test]
fn verify_sh_batch_has_intake_summary_heading() {
    assert!(
        VERIFY_SH.contains("Intake Summary"),
        "verify.sh must emit 'Intake Summary' heading in batch mode"
    );
}

#[test]
fn verify_sh_batch_reports_total_processed() {
    assert!(
        VERIFY_SH.contains("Total processed:"),
        "verify.sh must report 'Total processed:' count"
    );
}

#[test]
fn verify_sh_batch_reports_all_bucket_counts() {
    for label in ["Accepted:", "Mismatched:", "Malformed:", "Unverifiable:", "Duplicate:"] {
        assert!(
            VERIFY_SH.contains(label),
            "verify.sh must report '{label}' in intake summary"
        );
    }
}

#[test]
fn verify_sh_batch_processes_sorted_files() {
    assert!(
        VERIFY_SH.contains("| sort"),
        "verify.sh must sort inbound files for deterministic order"
    );
}

#[test]
fn verify_sh_batch_supports_report_flag() {
    assert!(
        VERIFY_SH.contains("--report"),
        "verify.sh must support --report flag for written output"
    );
    assert!(
        VERIFY_SH.contains("REPORT_FILE"),
        "verify.sh must use REPORT_FILE variable"
    );
}

#[test]
fn verify_sh_batch_exits_nonzero_when_no_accepted() {
    assert!(
        VERIFY_SH.contains("N_ACCEPTED -eq 0"),
        "verify.sh must exit non-zero when no artifacts are accepted"
    );
}

#[test]
fn verify_sh_preserves_inbound_single_mode() {
    assert!(
        VERIFY_SH.contains("response verified for current run"),
        "verify.sh must preserve single inbound verification wording"
    );
    assert!(
        VERIFY_SH.contains("response belongs to different run"),
        "verify.sh must preserve single inbound mismatch wording"
    );
}

// ── inbound testdata fixtures ─────────────────────────────────────────────────

#[test]
fn inbound_response_a_is_valid_json() {
    serde_json::from_str::<serde_json::Value>(INBOUND_A)
        .expect("inbound/response_a.json must be valid JSON");
}

#[test]
fn inbound_response_a_matches_locked_receipt_hash() {
    assert!(
        INBOUND_A.contains(LOCKED_RECEIPT_HASH),
        "response_a must contain the locked pilot receipt hash (accepted case)"
    );
}

#[test]
fn inbound_response_b_is_valid_json() {
    serde_json::from_str::<serde_json::Value>(INBOUND_B)
        .expect("inbound/response_b.json must be valid JSON");
}

#[test]
fn inbound_response_b_has_different_receipt_hash() {
    assert!(
        !INBOUND_B.contains(LOCKED_RECEIPT_HASH),
        "response_b must not contain the locked receipt hash (mismatch case)"
    );
    assert!(
        INBOUND_B.contains(r#""receipt_hash""#),
        "response_b must still have receipt_hash field"
    );
}

#[test]
fn inbound_response_c_is_valid_json() {
    serde_json::from_str::<serde_json::Value>(INBOUND_C)
        .expect("inbound/response_c.json must be valid JSON");
}

#[test]
fn inbound_response_c_is_missing_receipt_hash() {
    assert!(
        !INBOUND_C.contains(r#""receipt_hash""#),
        "response_c must not have receipt_hash field (malformed case)"
    );
}

#[test]
fn inbound_response_dup_is_valid_json() {
    serde_json::from_str::<serde_json::Value>(INBOUND_DUP)
        .expect("inbound/response_dup.json must be valid JSON");
}

#[test]
fn inbound_response_dup_matches_locked_receipt_hash() {
    assert!(
        INBOUND_DUP.contains(LOCKED_RECEIPT_HASH),
        "response_dup must contain the locked receipt hash (duplicate case)"
    );
}

#[test]
fn inbound_response_dup_differs_from_response_a_in_metadata() {
    // Same hash, different lab_id — confirms it's a duplicate not an identical file
    let a: serde_json::Value = serde_json::from_str(INBOUND_A).unwrap();
    let dup: serde_json::Value = serde_json::from_str(INBOUND_DUP).unwrap();
    assert_eq!(
        a["receipt_hash"], dup["receipt_hash"],
        "response_a and response_dup must share the same receipt_hash"
    );
    assert_ne!(
        a["lab_id"], dup["lab_id"],
        "response_dup must differ from response_a in lab_id to confirm it is a distinct file"
    );
}
