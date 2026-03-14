//! External handoff pack tests.
//!
//! Checks that lab_simulator.sh --handoff-pack mode produces the correct
//! structure, file names, and stable instruction wording.
//!
//! Uses include_str! so missing files are compile errors, not runtime failures.

const LAB_SIMULATOR_SH: &str = include_str!("../../../examples/pilot/lab_simulator.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");
const EXPECTED_MANIFEST_FIELDS: &str = include_str!(
    "../../../examples/pilot/testdata/expected_handoff_manifest_fields.txt"
);

const LOCKED_RECEIPT_HASH: &str =
    "0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb";

// ── lab_simulator.sh --handoff-pack mode ─────────────────────────────────────

#[test]
fn lab_simulator_supports_handoff_pack_flag() {
    assert!(
        LAB_SIMULATOR_SH.contains("--handoff-pack"),
        "lab_simulator.sh must support --handoff-pack flag"
    );
}

#[test]
fn lab_simulator_creates_artifacts_directory() {
    assert!(
        LAB_SIMULATOR_SH.contains("artifacts/"),
        "lab_simulator.sh must create artifacts/ subdirectory in handoff pack"
    );
}

#[test]
fn lab_simulator_writes_manifest_txt() {
    assert!(
        LAB_SIMULATOR_SH.contains("manifest.txt"),
        "lab_simulator.sh must write manifest.txt"
    );
}

#[test]
fn lab_simulator_writes_operator_instructions() {
    assert!(
        LAB_SIMULATOR_SH.contains("operator_instructions.txt"),
        "lab_simulator.sh must write operator_instructions.txt"
    );
}

#[test]
fn lab_simulator_writes_lab_response_instructions() {
    assert!(
        LAB_SIMULATOR_SH.contains("lab_response_instructions.txt"),
        "lab_simulator.sh must write lab_response_instructions.txt"
    );
}

#[test]
fn lab_simulator_includes_receipt_hash_in_instructions() {
    assert!(
        LAB_SIMULATOR_SH.contains("receipt_hash"),
        "lab_simulator.sh must include receipt_hash in handoff pack instructions"
    );
}

#[test]
fn lab_simulator_includes_run_id_in_pack_path() {
    assert!(
        LAB_SIMULATOR_SH.contains("RUN_ID"),
        "lab_simulator.sh must use RUN_ID as the pack subdirectory name"
    );
}

#[test]
fn lab_simulator_manifest_includes_run_id_field() {
    assert!(
        LAB_SIMULATOR_SH.contains("run_id:"),
        "lab_simulator.sh manifest must include run_id: field"
    );
}

#[test]
fn lab_simulator_manifest_includes_receipt_hash_field() {
    assert!(
        LAB_SIMULATOR_SH.contains("receipt_hash:"),
        "lab_simulator.sh manifest must include receipt_hash: field"
    );
}

#[test]
fn lab_simulator_manifest_includes_generated_at_field() {
    assert!(
        LAB_SIMULATOR_SH.contains("generated_at:"),
        "lab_simulator.sh manifest must include generated_at: field"
    );
}

#[test]
fn lab_simulator_operator_instructions_mention_verify() {
    assert!(
        LAB_SIMULATOR_SH.contains("verify.sh"),
        "lab_simulator.sh operator instructions must reference verify.sh"
    );
}

#[test]
fn lab_simulator_lab_instructions_include_response_template() {
    assert!(
        LAB_SIMULATOR_SH.contains("lab_response_schema"),
        "lab_simulator.sh lab_response_instructions must include response JSON template"
    );
}

#[test]
fn lab_simulator_lab_instructions_state_rejection_rule() {
    assert!(
        LAB_SIMULATOR_SH.contains("will be rejected if receipt_hash does not match"),
        "lab_simulator.sh lab_response_instructions must state rejection rule"
    );
}

#[test]
fn lab_simulator_emits_handoff_pack_written_message() {
    assert!(
        LAB_SIMULATOR_SH.contains("Handoff pack written:"),
        "lab_simulator.sh must print 'Handoff pack written:' on success"
    );
}

#[test]
fn lab_simulator_preserves_simulation_mode() {
    // Original simulation mode must still be present
    assert!(
        LAB_SIMULATOR_SH.contains("Lab Response Simulator"),
        "lab_simulator.sh must preserve the original simulation mode"
    );
    assert!(
        LAB_SIMULATOR_SH.contains("lab_response_schema"),
        "lab_simulator.sh must still write lab_response_schema field in simulation mode"
    );
}

#[test]
fn lab_simulator_handoff_pack_exits_nonzero_on_missing_bundle() {
    assert!(
        LAB_SIMULATOR_SH.contains("exit 1"),
        "lab_simulator.sh must exit 1 when bundle is missing"
    );
}

// ── expected manifest fixture ─────────────────────────────────────────────────

#[test]
fn expected_manifest_has_header() {
    assert!(
        EXPECTED_MANIFEST_FIELDS.contains("PostCAD External Handoff Pack"),
        "manifest fixture must have 'PostCAD External Handoff Pack' header"
    );
}

#[test]
fn expected_manifest_lists_required_files() {
    for file in [
        "artifacts/receipt.json",
        "manifest.txt",
        "operator_instructions.txt",
        "lab_response_instructions.txt",
    ] {
        assert!(
            EXPECTED_MANIFEST_FIELDS.contains(file),
            "manifest fixture must list file: {file}"
        );
    }
}

// ── README documents external lab trial ──────────────────────────────────────

#[test]
fn readme_documents_external_lab_trial_section() {
    assert!(
        README.contains("## External Lab Trial"),
        "README must have '## External Lab Trial' section"
    );
}

#[test]
fn readme_shows_handoff_pack_command() {
    assert!(
        README.contains("--handoff-pack"),
        "README must show --handoff-pack command"
    );
}

#[test]
fn readme_shows_receive_and_verify_flow() {
    assert!(
        README.contains("Receive response and verify"),
        "README must show receive-and-verify flow for real lab trials"
    );
}

#[test]
fn readme_documents_handoff_pack_structure() {
    assert!(
        README.contains("manifest.txt") && README.contains("lab_response_instructions.txt"),
        "README must document handoff pack structure including manifest and instructions"
    );
}
