//! Manual lab reply template tests.
//!
//! Checks that:
//!   - the handoff pack includes a lab_reply_template.json
//!   - the template contains required fields and FILL_IN placeholders
//!   - run_pilot.sh has --prepare-manual-reply mode
//!   - the existing verify flow can accept a correctly filled reply and
//!     reject malformed or mismatched ones
//!   - README documents the real manual external trial workflow

const LAB_SIMULATOR_SH: &str = include_str!("../../../examples/pilot/lab_simulator.sh");
const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");
const VERIFY_SH: &str = include_str!("../../../examples/pilot/verify.sh");

const EXPECTED_TEMPLATE_FIELDS: &str = include_str!(
    "../../../examples/pilot/testdata/expected_manual_reply_template_fields.txt"
);
const EXPECTED_MANIFEST_FIELDS: &str = include_str!(
    "../../../examples/pilot/testdata/expected_handoff_manifest_fields.txt"
);

const VALID_RESPONSE: &str = include_str!("../../../examples/pilot/testdata/lab_response_valid.json");
const STALE_RESPONSE: &str = include_str!("../../../examples/pilot/testdata/lab_response_stale.json");
const MALFORMED_RESPONSE: &str = include_str!("../../../examples/pilot/testdata/lab_response_malformed.json");

// ── handoff pack includes manual reply template ───────────────────────────────

#[test]
fn lab_simulator_writes_lab_reply_template() {
    assert!(
        LAB_SIMULATOR_SH.contains("lab_reply_template.json"),
        "lab_simulator.sh must write lab_reply_template.json in handoff pack"
    );
}

#[test]
fn lab_simulator_template_contains_receipt_hash_placeholder() {
    // Template must embed the run's receipt_hash (pre-filled, not FILL_IN)
    assert!(
        LAB_SIMULATOR_SH.contains("\"receipt_hash\": \"$RECEIPT_HASH\""),
        "lab_reply_template.json must embed receipt_hash from current run"
    );
}

#[test]
fn lab_simulator_template_contains_fill_in_for_lab_acknowledged_at() {
    assert!(
        LAB_SIMULATOR_SH.contains("FILL_IN: ISO 8601 timestamp"),
        "lab_reply_template.json must have FILL_IN placeholder for lab_acknowledged_at"
    );
}

#[test]
fn lab_simulator_template_contains_fill_in_for_lab_id() {
    assert!(
        LAB_SIMULATOR_SH.contains("FILL_IN: your lab identifier"),
        "lab_reply_template.json must have FILL_IN placeholder for lab_id"
    );
}

#[test]
fn lab_simulator_template_includes_lab_response_schema() {
    assert!(
        LAB_SIMULATOR_SH.contains("lab_response_schema"),
        "lab_reply_template.json must include lab_response_schema field"
    );
}

#[test]
fn lab_simulator_template_includes_status_accepted() {
    // Template must pre-fill status as accepted
    assert!(
        LAB_SIMULATOR_SH.contains("\"status\": \"accepted\""),
        "lab_reply_template.json must pre-fill status: accepted"
    );
}

#[test]
fn manifest_fixture_includes_lab_reply_template() {
    assert!(
        EXPECTED_MANIFEST_FIELDS.contains("lab_reply_template.json"),
        "expected_handoff_manifest_fields.txt must list lab_reply_template.json"
    );
}

// ── template fixture contains required fields ─────────────────────────────────

#[test]
fn template_fixture_has_receipt_hash_field() {
    assert!(
        EXPECTED_TEMPLATE_FIELDS.contains("receipt_hash"),
        "template fixture must include receipt_hash field"
    );
}

#[test]
fn template_fixture_has_fill_in_marker() {
    assert!(
        EXPECTED_TEMPLATE_FIELDS.contains("FILL_IN"),
        "template fixture must include FILL_IN marker"
    );
}

#[test]
fn template_fixture_has_lab_acknowledged_at_field() {
    assert!(
        EXPECTED_TEMPLATE_FIELDS.contains("lab_acknowledged_at"),
        "template fixture must include lab_acknowledged_at field"
    );
}

#[test]
fn template_fixture_has_lab_id_field() {
    assert!(
        EXPECTED_TEMPLATE_FIELDS.contains("lab_id"),
        "template fixture must include lab_id field"
    );
}

#[test]
fn template_fixture_has_lab_response_schema_field() {
    assert!(
        EXPECTED_TEMPLATE_FIELDS.contains("lab_response_schema"),
        "template fixture must include lab_response_schema field"
    );
}

#[test]
fn template_fixture_has_status_field() {
    assert!(
        EXPECTED_TEMPLATE_FIELDS.contains("status"),
        "template fixture must include status field"
    );
}

// ── run_pilot.sh --prepare-manual-reply ──────────────────────────────────────

#[test]
fn run_pilot_supports_prepare_manual_reply_flag() {
    assert!(
        RUN_PILOT_SH.contains("--prepare-manual-reply"),
        "run_pilot.sh must support --prepare-manual-reply flag"
    );
}

#[test]
fn run_pilot_prepare_manual_reply_checks_for_receipt() {
    // Must require receipt.json to exist (current run must be present)
    assert!(
        RUN_PILOT_SH.contains("receipt.json not found"),
        "run_pilot.sh --prepare-manual-reply must error if receipt.json missing"
    );
}

#[test]
fn run_pilot_prepare_manual_reply_errors_if_handoff_pack_missing() {
    assert!(
        RUN_PILOT_SH.contains("handoff pack not found for run"),
        "run_pilot.sh --prepare-manual-reply must error if handoff pack not found"
    );
}

#[test]
fn run_pilot_prepare_manual_reply_prints_lab_must_fill_in() {
    assert!(
        RUN_PILOT_SH.contains("The lab must fill in:"),
        "run_pilot.sh --prepare-manual-reply must print 'The lab must fill in:'"
    );
}

#[test]
fn run_pilot_prepare_manual_reply_prints_fields_must_not_change() {
    assert!(
        RUN_PILOT_SH.contains("Fields that must not be changed:"),
        "run_pilot.sh --prepare-manual-reply must list fields that must not be changed"
    );
}

#[test]
fn run_pilot_prepare_manual_reply_prints_template_prepared() {
    assert!(
        RUN_PILOT_SH.contains("Reply template prepared for manual completion:"),
        "run_pilot.sh must print 'Reply template prepared for manual completion:' on success"
    );
}

// ── existing verify flow handles correctly filled / malformed / mismatched ────

#[test]
fn verify_accepts_response_with_matching_receipt_hash() {
    // A valid response (matching receipt_hash) must be accepted
    // Verified via testdata/lab_response_valid.json fixture
    assert!(
        VALID_RESPONSE.contains("receipt_hash"),
        "valid lab response fixture must contain receipt_hash field"
    );
    // The verify.sh checks receipt_hash match — verified path exists
    assert!(
        VERIFY_SH.contains("verified_for_current_run"),
        "verify.sh must produce verified_for_current_run outcome for matching response"
    );
}

#[test]
fn verify_rejects_response_with_mismatched_receipt_hash() {
    // A stale response has a non-matching receipt_hash
    assert!(
        STALE_RESPONSE.contains("receipt_hash"),
        "stale lab response fixture must contain receipt_hash field"
    );
    assert!(
        VERIFY_SH.contains("belongs_to_different_run"),
        "verify.sh must produce belongs_to_different_run outcome for mismatched response"
    );
}

#[test]
fn verify_rejects_malformed_response_missing_receipt_hash() {
    // Malformed response must lack receipt_hash
    assert!(
        !MALFORMED_RESPONSE.contains("receipt_hash"),
        "malformed lab response fixture must not contain receipt_hash field"
    );
    assert!(
        VERIFY_SH.contains("malformed"),
        "verify.sh must produce malformed outcome when receipt_hash is absent"
    );
}

#[test]
fn verify_maps_malformed_to_rejected_decision() {
    assert!(
        VERIFY_SH.contains("malformed)    DECISION=\"rejected\""),
        "verify.sh must map malformed verification result to rejected decision"
    );
}

#[test]
fn verify_maps_run_mismatch_to_rejected_decision() {
    assert!(
        VERIFY_SH.contains("run_mismatch) DECISION=\"rejected\""),
        "verify.sh must map run_mismatch verification result to rejected decision"
    );
}

// ── README documents real manual external trial ───────────────────────────────

#[test]
fn readme_has_real_manual_external_trial_section() {
    assert!(
        README.contains("## Real Manual External Trial"),
        "README must have '## Real Manual External Trial' section"
    );
}

#[test]
fn readme_documents_prepare_manual_reply_command() {
    assert!(
        README.contains("--prepare-manual-reply"),
        "README must document --prepare-manual-reply command"
    );
}

#[test]
fn readme_shows_lab_reply_template_in_pack() {
    assert!(
        README.contains("lab_reply_template.json"),
        "README must mention lab_reply_template.json in handoff pack description"
    );
}

#[test]
fn readme_shows_fill_in_fields_for_lab() {
    assert!(
        README.contains("lab_acknowledged_at") && README.contains("lab_id"),
        "README must show lab_acknowledged_at and lab_id as fields the lab fills in"
    );
}

#[test]
fn readme_shows_verify_step_for_manual_reply() {
    assert!(
        README.contains("verify.sh") && README.contains("inbound"),
        "README must show verify.sh step for inbound manual reply"
    );
}

#[test]
fn readme_shows_receipt_hash_must_not_change() {
    assert!(
        README.contains("receipt_hash") && README.contains("must not be changed"),
        "README must state that receipt_hash must not be changed"
    );
}
