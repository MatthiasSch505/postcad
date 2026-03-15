//! First-contact lab message kit tests.
//!
//! Checks that --export-lab-trial-package generates email_to_lab.txt,
//! short_message_to_lab.txt, and operator_send_note.txt with deterministic
//! wording and the current run id embedded.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

const EXPECTED_MANIFEST_FIELDS: &str = include_str!(
    "../../../examples/pilot/testdata/expected_lab_trial_package_manifest_fields.txt"
);

// ── manifest fixture lists all three message kit files ────────────────────────

#[test]
fn manifest_fixture_lists_email_to_lab() {
    assert!(
        EXPECTED_MANIFEST_FIELDS.contains("email_to_lab.txt"),
        "manifest fixture must list email_to_lab.txt"
    );
}

#[test]
fn manifest_fixture_lists_short_message_to_lab() {
    assert!(
        EXPECTED_MANIFEST_FIELDS.contains("short_message_to_lab.txt"),
        "manifest fixture must list short_message_to_lab.txt"
    );
}

#[test]
fn manifest_fixture_lists_operator_send_note() {
    assert!(
        EXPECTED_MANIFEST_FIELDS.contains("operator_send_note.txt"),
        "manifest fixture must list operator_send_note.txt"
    );
}

// ── run_pilot.sh writes all three files ──────────────────────────────────────

#[test]
fn export_command_writes_email_to_lab() {
    assert!(
        RUN_PILOT_SH.contains("email_to_lab.txt"),
        "run_pilot.sh must write email_to_lab.txt"
    );
}

#[test]
fn export_command_writes_short_message_to_lab() {
    assert!(
        RUN_PILOT_SH.contains("short_message_to_lab.txt"),
        "run_pilot.sh must write short_message_to_lab.txt"
    );
}

#[test]
fn export_command_writes_operator_send_note() {
    assert!(
        RUN_PILOT_SH.contains("operator_send_note.txt"),
        "run_pilot.sh must write operator_send_note.txt"
    );
}

// ── email_to_lab.txt content ──────────────────────────────────────────────────

#[test]
fn email_includes_run_id() {
    assert!(
        RUN_PILOT_SH.contains("run ${EXP_RUN_ID}"),
        "email_to_lab.txt must include current run id"
    );
}

#[test]
fn email_mentions_external_workflow_trial() {
    assert!(
        RUN_PILOT_SH.contains("external workflow trial"),
        "email_to_lab.txt must mention external workflow trial"
    );
}

#[test]
fn email_describes_reply_template_fields_to_fill() {
    assert!(
        RUN_PILOT_SH.contains("lab_acknowledged_at") && RUN_PILOT_SH.contains("lab_id"),
        "email_to_lab.txt must describe the two fields the lab must fill"
    );
}

#[test]
fn email_states_no_integration_required() {
    assert!(
        RUN_PILOT_SH.contains("No software integration is required"),
        "email_to_lab.txt must state that no software integration is required"
    );
}

#[test]
fn email_specifies_return_filename_with_run_id() {
    assert!(
        RUN_PILOT_SH.contains("lab_reply_${EXP_RUN_ID}.json"),
        "email_to_lab.txt must specify return filename containing run id"
    );
}

// ── short_message_to_lab.txt content ─────────────────────────────────────────

#[test]
fn short_message_includes_run_id() {
    assert!(
        RUN_PILOT_SH.contains("run ${EXP_RUN_ID}"),
        "short_message_to_lab.txt must include current run id"
    );
}

#[test]
fn short_message_mentions_reply_template() {
    assert!(
        RUN_PILOT_SH.contains("lab_reply_template.json"),
        "short_message_to_lab.txt must mention lab_reply_template.json"
    );
}

#[test]
fn short_message_states_no_integration() {
    assert!(
        RUN_PILOT_SH.contains("No integration needed"),
        "short_message_to_lab.txt must state no integration needed"
    );
}

// ── operator_send_note.txt content ────────────────────────────────────────────

#[test]
fn operator_send_note_has_checklist_header() {
    assert!(
        RUN_PILOT_SH.contains("Operator Send Checklist"),
        "operator_send_note.txt must have 'Operator Send Checklist' header"
    );
}

#[test]
fn operator_send_note_includes_run_id() {
    assert!(
        RUN_PILOT_SH.contains("Run ID: $EXP_RUN_ID"),
        "operator_send_note.txt must include current run id"
    );
}

#[test]
fn operator_send_note_step_zip_package() {
    assert!(
        RUN_PILOT_SH.contains("Zip the package"),
        "operator_send_note.txt must include step: zip the package"
    );
}

#[test]
fn operator_send_note_step_send_to_lab() {
    assert!(
        RUN_PILOT_SH.contains("Send the zip to the lab"),
        "operator_send_note.txt must include step: send the zip to the lab"
    );
}

#[test]
fn operator_send_note_step_wait_for_reply() {
    assert!(
        RUN_PILOT_SH.contains("Wait for the lab to return"),
        "operator_send_note.txt must include step: wait for lab reply"
    );
}

#[test]
fn operator_send_note_step_place_in_inbound() {
    assert!(
        RUN_PILOT_SH.contains("Place the returned file into your inbound directory"),
        "operator_send_note.txt must include step: place returned file into inbound"
    );
}

#[test]
fn operator_send_note_step_run_verification() {
    assert!(
        RUN_PILOT_SH.contains("Run verification and generate decision record"),
        "operator_send_note.txt must include step: run verification and decision"
    );
}

#[test]
fn operator_send_note_step_inspect_decision_record() {
    assert!(
        RUN_PILOT_SH.contains("Inspect the decision record"),
        "operator_send_note.txt must include step: inspect decision record"
    );
}

#[test]
fn operator_send_note_references_email_or_short_message() {
    assert!(
        RUN_PILOT_SH.contains("email_to_lab.txt") && RUN_PILOT_SH.contains("short_message_to_lab.txt"),
        "operator_send_note.txt must reference both email_to_lab.txt and short_message_to_lab.txt"
    );
}

// ── README ────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_first_contact_send_flow_section() {
    assert!(
        README.contains("## First-Contact Send Flow"),
        "README must have '## First-Contact Send Flow' section"
    );
}

#[test]
fn readme_documents_email_to_lab() {
    assert!(
        README.contains("email_to_lab.txt"),
        "README must document email_to_lab.txt"
    );
}

#[test]
fn readme_documents_short_message_to_lab() {
    assert!(
        README.contains("short_message_to_lab.txt"),
        "README must document short_message_to_lab.txt"
    );
}

#[test]
fn readme_documents_operator_send_note() {
    assert!(
        README.contains("operator_send_note.txt"),
        "README must document operator_send_note.txt"
    );
}

#[test]
fn readme_shows_checklist_steps() {
    assert!(
        README.contains("[ ] 1.") && README.contains("[ ] 5."),
        "README must show operator checklist with numbered steps"
    );
}
