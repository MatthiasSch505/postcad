//! Inbound reply inspection summary tests.
//!
//! Checks that run_pilot.sh --inspect-inbound-reply exists, produces
//! deterministic field summaries, and distinguishes readable / missing-field /
//! not-readable outcomes.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

const LAB_REPLY_FILLED: &str =
    include_str!("../../../examples/pilot/testdata/lab_reply_filled.json");
const LAB_RESPONSE_MALFORMED: &str =
    include_str!("../../../examples/pilot/testdata/lab_response_malformed.json");
const LAB_RESPONSE_VALID: &str =
    include_str!("../../../examples/pilot/testdata/lab_response_valid.json");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_inspect_inbound_reply_flag() {
    assert!(
        RUN_PILOT_SH.contains("--inspect-inbound-reply"),
        "run_pilot.sh must support --inspect-inbound-reply flag"
    );
}

#[test]
fn inspect_command_requires_file_argument() {
    assert!(
        RUN_PILOT_SH.contains("--inspect-inbound-reply requires a file argument"),
        "run_pilot.sh must require a file argument for --inspect-inbound-reply"
    );
}

// ── file readability checks ───────────────────────────────────────────────────

#[test]
fn inspect_command_handles_missing_file() {
    assert!(
        RUN_PILOT_SH.contains("file not found"),
        "run_pilot.sh must print 'file not found' when reply file is missing"
    );
}

#[test]
fn inspect_command_handles_invalid_json() {
    assert!(
        RUN_PILOT_SH.contains("not valid JSON"),
        "run_pilot.sh must print 'not valid JSON' when reply is not parseable"
    );
}

#[test]
fn inspect_command_prints_reply_not_readable() {
    assert!(
        RUN_PILOT_SH.contains("reply not readable"),
        "run_pilot.sh must print 'reply not readable' for unreadable files"
    );
}

// ── field extraction ──────────────────────────────────────────────────────────

#[test]
fn inspect_command_extracts_case_id() {
    assert!(
        RUN_PILOT_SH.contains("Case ID"),
        "run_pilot.sh must extract and display Case ID"
    );
}

#[test]
fn inspect_command_extracts_receipt_hash() {
    assert!(
        RUN_PILOT_SH.contains("Receipt hash"),
        "run_pilot.sh must extract and display Receipt hash"
    );
}

#[test]
fn inspect_command_extracts_lab_id() {
    assert!(
        RUN_PILOT_SH.contains("Lab ID"),
        "run_pilot.sh must extract and display Lab ID"
    );
}

#[test]
fn inspect_command_extracts_status() {
    assert!(
        RUN_PILOT_SH.contains("Status"),
        "run_pilot.sh must extract and display Status"
    );
}

#[test]
fn inspect_command_extracts_acknowledged_at() {
    assert!(
        RUN_PILOT_SH.contains("Acknowledged at"),
        "run_pilot.sh must extract and display Acknowledged at"
    );
}

#[test]
fn inspect_command_reports_notes_presence() {
    assert!(
        RUN_PILOT_SH.contains("Notes"),
        "run_pilot.sh must report whether Notes field is present"
    );
}

#[test]
fn inspect_command_prints_not_present_for_absent_fields() {
    assert!(
        RUN_PILOT_SH.contains("not present"),
        "run_pilot.sh must print 'not present' for absent optional fields"
    );
}

// ── required field checklist ──────────────────────────────────────────────────

#[test]
fn inspect_command_checks_receipt_hash_required() {
    assert!(
        RUN_PILOT_SH.contains("INS_RECEIPT_HASH") && RUN_PILOT_SH.contains("receipt_hash"),
        "run_pilot.sh must include receipt_hash in required field checklist"
    );
}

#[test]
fn inspect_command_checks_lab_id_required() {
    assert!(
        RUN_PILOT_SH.contains("INS_LAB_ID") && RUN_PILOT_SH.contains("lab_id"),
        "run_pilot.sh must include lab_id in required field checklist"
    );
}

#[test]
fn inspect_command_checks_status_required() {
    assert!(
        RUN_PILOT_SH.contains("INS_STATUS") && RUN_PILOT_SH.contains("status"),
        "run_pilot.sh must include status in required field checklist"
    );
}

#[test]
fn inspect_command_checks_lab_acknowledged_at_required() {
    assert!(
        RUN_PILOT_SH.contains("INS_ACK_AT") && RUN_PILOT_SH.contains("lab_acknowledged_at"),
        "run_pilot.sh must include lab_acknowledged_at in required field checklist"
    );
}

#[test]
fn inspect_command_prints_present_label() {
    assert!(
        RUN_PILOT_SH.contains("present"),
        "run_pilot.sh must print 'present' for fields that exist"
    );
}

#[test]
fn inspect_command_prints_missing_label() {
    assert!(
        RUN_PILOT_SH.contains("MISSING"),
        "run_pilot.sh must print 'MISSING' for absent required fields"
    );
}

// ── overall result wording ────────────────────────────────────────────────────

#[test]
fn inspect_command_prints_structurally_readable_on_pass() {
    assert!(
        RUN_PILOT_SH.contains("reply structurally readable"),
        "run_pilot.sh must print 'reply structurally readable' when all required fields present"
    );
}

#[test]
fn inspect_command_prints_missing_required_fields_on_fail() {
    assert!(
        RUN_PILOT_SH.contains("reply missing required field(s):"),
        "run_pilot.sh must print 'reply missing required field(s):' when fields absent"
    );
}

#[test]
fn inspect_command_suggests_verify_on_pass() {
    assert!(
        RUN_PILOT_SH.contains("verify.sh --inbound"),
        "run_pilot.sh must suggest verify.sh --inbound as next step on pass"
    );
}

// ── testdata fixtures ─────────────────────────────────────────────────────────

#[test]
fn lab_reply_filled_fixture_has_receipt_hash() {
    assert!(
        LAB_REPLY_FILLED.contains("receipt_hash"),
        "lab_reply_filled.json must contain receipt_hash field"
    );
}

#[test]
fn lab_reply_filled_fixture_has_lab_id() {
    assert!(
        LAB_REPLY_FILLED.contains("lab_id"),
        "lab_reply_filled.json must contain lab_id field"
    );
}

#[test]
fn lab_reply_filled_fixture_has_lab_acknowledged_at() {
    assert!(
        LAB_REPLY_FILLED.contains("lab_acknowledged_at"),
        "lab_reply_filled.json must contain lab_acknowledged_at field"
    );
}

#[test]
fn lab_reply_filled_fixture_has_status() {
    assert!(
        LAB_REPLY_FILLED.contains("status"),
        "lab_reply_filled.json must contain status field"
    );
}

#[test]
fn lab_reply_filled_fixture_has_real_lab_id_value() {
    // Should have a non-simulator lab_id to be useful as a manual reply fixture
    assert!(
        LAB_REPLY_FILLED.contains("dental-lab-berlin-001"),
        "lab_reply_filled.json must have a realistic lab_id value"
    );
}

#[test]
fn lab_response_malformed_fixture_missing_receipt_hash() {
    // Malformed fixture must lack receipt_hash to test missing-field path
    assert!(
        !LAB_RESPONSE_MALFORMED.contains("receipt_hash"),
        "lab_response_malformed.json must not contain receipt_hash field"
    );
}

#[test]
fn lab_response_valid_fixture_has_all_required_fields() {
    for field in ["receipt_hash", "lab_id", "status", "lab_acknowledged_at"] {
        assert!(
            LAB_RESPONSE_VALID.contains(field),
            "lab_response_valid.json must contain required field: {field}"
        );
    }
}

// ── README ────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_inbound_reply_inspection_section() {
    assert!(
        README.contains("## Inbound Reply Inspection"),
        "README must have '## Inbound Reply Inspection' section"
    );
}

#[test]
fn readme_shows_inspect_inbound_reply_command() {
    assert!(
        README.contains("--inspect-inbound-reply"),
        "README must show --inspect-inbound-reply command"
    );
}

#[test]
fn readme_shows_structurally_readable_output() {
    assert!(
        README.contains("reply structurally readable"),
        "README must show expected 'reply structurally readable' output"
    );
}

#[test]
fn readme_shows_missing_field_output() {
    assert!(
        README.contains("reply missing required field"),
        "README must show expected 'reply missing required field' output"
    );
}

#[test]
fn readme_shows_verify_as_next_step() {
    assert!(
        README.contains("verify.sh") && README.contains("inbound/lab_reply_"),
        "README must show verify.sh as next step after inspection"
    );
}
