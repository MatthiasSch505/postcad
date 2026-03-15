//! Sendable lab trial package tests.
//!
//! Checks that run_pilot.sh --export-lab-trial-package mode produces
//! the correct file set, naming, instruction wording, and manifest content.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

const EXPECTED_MANIFEST_FIELDS: &str = include_str!(
    "../../../examples/pilot/testdata/expected_lab_trial_package_manifest_fields.txt"
);

// ── export command exists ─────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_export_lab_trial_package_flag() {
    assert!(
        RUN_PILOT_SH.contains("--export-lab-trial-package"),
        "run_pilot.sh must support --export-lab-trial-package flag"
    );
}

#[test]
fn export_command_errors_if_receipt_missing() {
    assert!(
        RUN_PILOT_SH.contains("receipt.json not found"),
        "run_pilot.sh --export-lab-trial-package must error if receipt.json missing"
    );
}

// ── package directory ─────────────────────────────────────────────────────────

#[test]
fn export_command_creates_outbound_lab_trial_dir() {
    assert!(
        RUN_PILOT_SH.contains("outbound/lab_trial_"),
        "run_pilot.sh must create outbound/lab_trial_<run-id> directory"
    );
}

#[test]
fn export_command_dir_includes_run_id() {
    assert!(
        RUN_PILOT_SH.contains("lab_trial_${EXP_RUN_ID}"),
        "run_pilot.sh package directory must include EXP_RUN_ID"
    );
}

// ── manifest ──────────────────────────────────────────────────────────────────

#[test]
fn export_command_writes_manifest_txt() {
    assert!(
        RUN_PILOT_SH.contains("manifest.txt"),
        "run_pilot.sh must write manifest.txt"
    );
}

#[test]
fn manifest_fixture_has_header() {
    assert!(
        EXPECTED_MANIFEST_FIELDS.contains("PostCAD Lab Trial Package"),
        "manifest fixture must have 'PostCAD Lab Trial Package' header"
    );
}

#[test]
fn manifest_fixture_has_run_id_field() {
    assert!(
        EXPECTED_MANIFEST_FIELDS.contains("run_id:"),
        "manifest fixture must have run_id: field"
    );
}

#[test]
fn manifest_fixture_has_receipt_hash_field() {
    assert!(
        EXPECTED_MANIFEST_FIELDS.contains("receipt_hash:"),
        "manifest fixture must have receipt_hash: field"
    );
}

#[test]
fn manifest_fixture_has_generated_at_field() {
    assert!(
        EXPECTED_MANIFEST_FIELDS.contains("generated_at:"),
        "manifest fixture must have generated_at: field"
    );
}

#[test]
fn manifest_fixture_lists_required_files() {
    for file in [
        "manifest.txt",
        "operator_instructions.txt",
        "lab_instructions.txt",
        "lab_reply_template.json",
        "receipt.json",
    ] {
        assert!(
            EXPECTED_MANIFEST_FIELDS.contains(file),
            "manifest fixture must list file: {file}"
        );
    }
}

// ── operator_instructions.txt ─────────────────────────────────────────────────

#[test]
fn export_command_writes_operator_instructions() {
    assert!(
        RUN_PILOT_SH.contains("operator_instructions.txt"),
        "run_pilot.sh must write operator_instructions.txt"
    );
}

#[test]
fn operator_instructions_describe_sendable_package() {
    assert!(
        RUN_PILOT_SH.contains("sendable lab trial package"),
        "operator_instructions.txt must describe this as a sendable lab trial package"
    );
}

#[test]
fn operator_instructions_mention_verify_command() {
    assert!(
        RUN_PILOT_SH.contains("verify.sh --inbound"),
        "operator_instructions.txt must reference the verify.sh --inbound command"
    );
}

#[test]
fn operator_instructions_include_run_id() {
    assert!(
        RUN_PILOT_SH.contains("Run ID:       $EXP_RUN_ID"),
        "operator_instructions.txt must include the current run id"
    );
}

// ── lab_instructions.txt ──────────────────────────────────────────────────────

#[test]
fn export_command_writes_lab_instructions() {
    assert!(
        RUN_PILOT_SH.contains("lab_instructions.txt"),
        "run_pilot.sh must write lab_instructions.txt"
    );
}

#[test]
fn lab_instructions_list_fill_in_fields() {
    assert!(
        RUN_PILOT_SH.contains("Fields to fill in:"),
        "lab_instructions.txt must list fields the lab must fill in"
    );
}

#[test]
fn lab_instructions_list_must_not_change_fields() {
    assert!(
        RUN_PILOT_SH.contains("Fields that must not be changed:"),
        "lab_instructions.txt must list fields that must not be changed"
    );
}

#[test]
fn lab_instructions_state_rejection_rule() {
    assert!(
        RUN_PILOT_SH.contains("will be rejected if receipt_hash does not match exactly"),
        "lab_instructions.txt must state rejection rule for mismatched receipt_hash"
    );
}

#[test]
fn lab_instructions_specify_return_filename() {
    assert!(
        RUN_PILOT_SH.contains("lab_reply_${EXP_RUN_ID}.json"),
        "lab_instructions.txt must specify the filename the lab should return"
    );
}

// ── lab_reply_template.json ───────────────────────────────────────────────────

#[test]
fn export_command_writes_lab_reply_template() {
    assert!(
        RUN_PILOT_SH.contains("lab_reply_template.json"),
        "run_pilot.sh must write lab_reply_template.json"
    );
}

#[test]
fn lab_reply_template_has_receipt_hash_pre_filled() {
    assert!(
        RUN_PILOT_SH.contains("receipt_hash") && RUN_PILOT_SH.contains("EXP_RECEIPT_HASH"),
        "lab_reply_template.json must pre-fill receipt_hash from current run"
    );
}

#[test]
fn lab_reply_template_has_fill_in_for_lab_acknowledged_at() {
    assert!(
        RUN_PILOT_SH.contains("FILL_IN: ISO 8601 timestamp"),
        "lab_reply_template.json must have FILL_IN placeholder for lab_acknowledged_at"
    );
}

#[test]
fn lab_reply_template_has_fill_in_for_lab_id() {
    assert!(
        RUN_PILOT_SH.contains("FILL_IN: your lab identifier"),
        "lab_reply_template.json must have FILL_IN placeholder for lab_id"
    );
}

#[test]
fn lab_reply_template_has_status_accepted() {
    assert!(
        RUN_PILOT_SH.contains(r#"\"status\": \"accepted\""#),
        "lab_reply_template.json must pre-fill status as accepted"
    );
}

// ── receipt.json is copied ────────────────────────────────────────────────────

#[test]
fn export_command_copies_receipt_json() {
    assert!(
        RUN_PILOT_SH.contains("cp \"$RECEIPT\" \"$PACKAGE_DIR/receipt.json\""),
        "run_pilot.sh must copy receipt.json into the package"
    );
}

// ── stdout output ─────────────────────────────────────────────────────────────

#[test]
fn export_command_prints_package_written() {
    assert!(
        RUN_PILOT_SH.contains("Package written:"),
        "run_pilot.sh must print 'Package written:' on success"
    );
}

// ── README ────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_sendable_lab_trial_package_section() {
    assert!(
        README.contains("## Sendable Lab Trial Package"),
        "README must have '## Sendable Lab Trial Package' section"
    );
}

#[test]
fn readme_shows_export_lab_trial_package_command() {
    assert!(
        README.contains("--export-lab-trial-package"),
        "README must show --export-lab-trial-package command"
    );
}

#[test]
fn readme_shows_package_structure() {
    assert!(
        README.contains("outbound/lab_trial_"),
        "README must show outbound/lab_trial_<run-id> package structure"
    );
}

#[test]
fn readme_shows_verify_step_for_returned_reply() {
    assert!(
        README.contains("verify.sh") && README.contains("inbound/lab_reply_"),
        "README must show verify.sh step for the returned lab reply"
    );
}

#[test]
fn readme_documents_zip_and_send() {
    assert!(
        README.contains("zip") && README.contains("lab_trial_"),
        "README must document zipping and sending the package"
    );
}
