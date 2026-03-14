//! Inbound lab response verification tests.
//!
//! Checks that lab_simulator.sh and verify.sh --inbound have the correct
//! content, wording, and structural properties. Uses include_str! so
//! missing files are compile errors, not runtime failures.

const LAB_SIMULATOR_SH: &str = include_str!("../../../examples/pilot/lab_simulator.sh");
const VERIFY_SH: &str = include_str!("../../../examples/pilot/verify.sh");
const LAB_RESPONSE_VALID: &str =
    include_str!("../../../examples/pilot/testdata/lab_response_valid.json");
const LAB_RESPONSE_STALE: &str =
    include_str!("../../../examples/pilot/testdata/lab_response_stale.json");
const LAB_RESPONSE_MALFORMED: &str =
    include_str!("../../../examples/pilot/testdata/lab_response_malformed.json");

const LOCKED_RECEIPT_HASH: &str =
    "0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb";

// ── lab_simulator.sh ──────────────────────────────────────────────────────────

#[test]
fn lab_simulator_has_strict_mode() {
    assert!(
        LAB_SIMULATOR_SH.contains("set -euo pipefail"),
        "lab_simulator.sh must use 'set -euo pipefail'"
    );
}

#[test]
fn lab_simulator_emits_lab_response_schema_field() {
    assert!(
        LAB_SIMULATOR_SH.contains("lab_response_schema"),
        "lab_simulator.sh must write lab_response_schema field"
    );
}

#[test]
fn lab_simulator_binds_receipt_hash_to_output() {
    assert!(
        LAB_SIMULATOR_SH.contains("receipt_hash"),
        "lab_simulator.sh must include receipt_hash in output"
    );
    assert!(
        LAB_SIMULATOR_SH.contains("RECEIPT_HASH"),
        "lab_simulator.sh must use RECEIPT_HASH variable"
    );
}

#[test]
fn lab_simulator_references_verify_inbound() {
    assert!(
        LAB_SIMULATOR_SH.contains("--inbound"),
        "lab_simulator.sh must reference --inbound in next-step instructions"
    );
}

#[test]
fn lab_simulator_reads_from_bundle_dir() {
    assert!(
        LAB_SIMULATOR_SH.contains("BUNDLE_DIR"),
        "lab_simulator.sh must accept a bundle directory argument"
    );
}

#[test]
fn lab_simulator_exits_nonzero_on_missing_bundle() {
    assert!(
        LAB_SIMULATOR_SH.contains("exit 1"),
        "lab_simulator.sh must exit 1 when bundle is missing"
    );
}

// ── verify.sh --inbound mode ──────────────────────────────────────────────────

#[test]
fn verify_sh_supports_inbound_mode() {
    assert!(
        VERIFY_SH.contains("--inbound"),
        "verify.sh must support --inbound flag"
    );
}

#[test]
fn verify_sh_emits_verified_outcome_wording() {
    assert!(
        VERIFY_SH.contains("response verified for current run"),
        "verify.sh must emit exact wording: 'response verified for current run'"
    );
}

#[test]
fn verify_sh_emits_different_run_wording() {
    assert!(
        VERIFY_SH.contains("response belongs to different run"),
        "verify.sh must emit exact wording: 'response belongs to different run'"
    );
}

#[test]
fn verify_sh_emits_missing_field_wording() {
    assert!(
        VERIFY_SH.contains("response missing required artifact/field"),
        "verify.sh must emit exact wording: 'response missing required artifact/field'"
    );
}

#[test]
fn verify_sh_emits_cannot_verify_wording() {
    assert!(
        VERIFY_SH.contains("response cannot be verified"),
        "verify.sh must emit exact wording: 'response cannot be verified'"
    );
}

#[test]
fn verify_sh_preserves_receipt_verification_mode() {
    assert!(
        VERIFY_SH.contains("verify-receipt"),
        "verify.sh must still call verify-receipt for the original receipt mode"
    );
}

#[test]
fn verify_sh_has_strict_mode() {
    assert!(
        VERIFY_SH.contains("set -euo pipefail"),
        "verify.sh must use 'set -euo pipefail'"
    );
}

// ── testdata fixtures ─────────────────────────────────────────────────────────

#[test]
fn valid_lab_response_contains_locked_receipt_hash() {
    assert!(
        LAB_RESPONSE_VALID.contains(LOCKED_RECEIPT_HASH),
        "valid lab response must contain the locked pilot receipt hash"
    );
}

#[test]
fn valid_lab_response_has_required_fields() {
    assert!(
        LAB_RESPONSE_VALID.contains(r#""receipt_hash""#),
        "valid lab response must have receipt_hash"
    );
    assert!(
        LAB_RESPONSE_VALID.contains(r#""dispatch_id""#),
        "valid lab response must have dispatch_id"
    );
    assert!(
        LAB_RESPONSE_VALID.contains(r#""case_id""#),
        "valid lab response must have case_id"
    );
    assert!(
        LAB_RESPONSE_VALID.contains(r#""lab_response_schema""#),
        "valid lab response must have lab_response_schema"
    );
}

#[test]
fn stale_lab_response_has_different_receipt_hash() {
    assert!(
        !LAB_RESPONSE_STALE.contains(LOCKED_RECEIPT_HASH),
        "stale lab response must not contain the locked pilot receipt hash"
    );
    assert!(
        LAB_RESPONSE_STALE.contains(r#""receipt_hash""#),
        "stale lab response must still have receipt_hash field (just wrong value)"
    );
}

#[test]
fn malformed_lab_response_is_missing_receipt_hash() {
    assert!(
        !LAB_RESPONSE_MALFORMED.contains(r#""receipt_hash""#),
        "malformed lab response must not have a receipt_hash field"
    );
}

#[test]
fn all_testdata_fixtures_are_valid_json() {
    serde_json::from_str::<serde_json::Value>(LAB_RESPONSE_VALID)
        .expect("lab_response_valid.json must be valid JSON");
    serde_json::from_str::<serde_json::Value>(LAB_RESPONSE_STALE)
        .expect("lab_response_stale.json must be valid JSON");
    serde_json::from_str::<serde_json::Value>(LAB_RESPONSE_MALFORMED)
        .expect("lab_response_malformed.json must be valid JSON");
}
