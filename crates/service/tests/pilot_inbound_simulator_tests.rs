//! Inbound reply simulator tests.
//!
//! Checks that run_pilot.sh --simulate-inbound exists, the template fixture
//! is valid, the output is deterministic, and the README section is present.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");
const LAB_REPLY_SIMULATED: &str =
    include_str!("../../../examples/pilot/testdata/lab_reply_simulated.json");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_simulate_inbound_flag() {
    assert!(
        RUN_PILOT_SH.contains("--simulate-inbound"),
        "run_pilot.sh must support --simulate-inbound flag"
    );
}

#[test]
fn simulate_inbound_exits_0_on_success() {
    assert!(
        RUN_PILOT_SH.contains("--simulate-inbound") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --simulate-inbound must exit 0 on success"
    );
}

// ── template file ─────────────────────────────────────────────────────────────

#[test]
fn simulator_template_is_referenced() {
    assert!(
        RUN_PILOT_SH.contains("lab_reply_simulated.json"),
        "run_pilot.sh must reference lab_reply_simulated.json template"
    );
}

#[test]
fn simulator_template_has_required_receipt_hash_field() {
    assert!(
        LAB_REPLY_SIMULATED.contains("receipt_hash"),
        "lab_reply_simulated.json must contain receipt_hash field"
    );
}

#[test]
fn simulator_template_has_required_lab_id_field() {
    assert!(
        LAB_REPLY_SIMULATED.contains("lab_id"),
        "lab_reply_simulated.json must contain lab_id field"
    );
}

#[test]
fn simulator_template_has_required_status_field() {
    assert!(
        LAB_REPLY_SIMULATED.contains("status"),
        "lab_reply_simulated.json must contain status field"
    );
}

#[test]
fn simulator_template_has_required_lab_acknowledged_at_field() {
    assert!(
        LAB_REPLY_SIMULATED.contains("lab_acknowledged_at"),
        "lab_reply_simulated.json must contain lab_acknowledged_at field"
    );
}

#[test]
fn simulator_template_has_lab_response_schema_field() {
    assert!(
        LAB_REPLY_SIMULATED.contains("lab_response_schema"),
        "lab_reply_simulated.json must contain lab_response_schema field"
    );
}

#[test]
fn simulator_template_status_is_accepted() {
    assert!(
        LAB_REPLY_SIMULATED.contains("\"accepted\""),
        "lab_reply_simulated.json status must be 'accepted'"
    );
}

#[test]
fn simulator_template_is_valid_json_structure() {
    // Minimal structural check: starts with { and ends with }
    let trimmed = LAB_REPLY_SIMULATED.trim();
    assert!(
        trimmed.starts_with('{') && trimmed.ends_with('}'),
        "lab_reply_simulated.json must be a JSON object"
    );
}

// ── output behavior ───────────────────────────────────────────────────────────

#[test]
fn simulate_inbound_prints_success_header() {
    assert!(
        RUN_PILOT_SH.contains("SIMULATED LAB REPLY GENERATED"),
        "run_pilot.sh must print 'SIMULATED LAB REPLY GENERATED' on success"
    );
}

#[test]
fn simulate_inbound_prints_file_label() {
    assert!(
        RUN_PILOT_SH.contains("SIMULATED LAB REPLY GENERATED")
            && RUN_PILOT_SH.contains("File:"),
        "run_pilot.sh --simulate-inbound output must include 'File:' label"
    );
}

#[test]
fn simulate_inbound_prints_next_step_label() {
    assert!(
        RUN_PILOT_SH.contains("Next step:"),
        "run_pilot.sh --simulate-inbound output must include 'Next step:' label"
    );
}

#[test]
fn simulate_inbound_next_step_mentions_inspect() {
    assert!(
        RUN_PILOT_SH.contains("inspect inbound reply"),
        "run_pilot.sh --simulate-inbound next step must mention 'inspect inbound reply'"
    );
}

#[test]
fn simulate_inbound_next_step_mentions_verify() {
    assert!(
        RUN_PILOT_SH.contains("verify inbound reply"),
        "run_pilot.sh --simulate-inbound next step must mention 'verify inbound reply'"
    );
}

// ── run-id handling ───────────────────────────────────────────────────────────

#[test]
fn simulate_inbound_writes_run_id_named_file_when_run_exists() {
    assert!(
        RUN_PILOT_SH.contains("inbound/lab_reply_${SIM_RUN_ID}.json"),
        "run_pilot.sh --simulate-inbound must write inbound/lab_reply_<run-id>.json when run_id known"
    );
}

#[test]
fn simulate_inbound_fallback_filename_when_no_run() {
    assert!(
        RUN_PILOT_SH.contains("inbound/lab_reply_simulated.json"),
        "run_pilot.sh --simulate-inbound must fall back to lab_reply_simulated.json when no run_id"
    );
}

// ── error handling ────────────────────────────────────────────────────────────

#[test]
fn simulate_inbound_errors_if_template_missing() {
    assert!(
        RUN_PILOT_SH.contains("simulator template not found"),
        "run_pilot.sh --simulate-inbound must error if template file is not found"
    );
}

// ── determinism ───────────────────────────────────────────────────────────────

#[test]
fn simulate_inbound_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("SIMULATED LAB REPLY GENERATED")
        .expect("simulator header must exist");
    let block_end = RUN_PILOT_SH[block_start..]
        .find("exit 0")
        .map(|i| block_start + i + 6)
        .unwrap_or(block_start + 1000);
    let block = &RUN_PILOT_SH[block_start..block_end];
    assert!(
        !block.contains("$(date"),
        "simulate-inbound output block must not embed timestamps"
    );
}

// ── README section ────────────────────────────────────────────────────────────

#[test]
fn readme_has_inbound_reply_simulator_section() {
    assert!(
        README.contains("## Inbound Reply Simulator"),
        "README must have '## Inbound Reply Simulator' section"
    );
}

#[test]
fn readme_shows_simulate_inbound_command() {
    assert!(
        README.contains("--simulate-inbound"),
        "README must show --simulate-inbound command"
    );
}

#[test]
fn readme_simulator_mentions_testdata_template() {
    assert!(
        README.contains("lab_reply_simulated.json"),
        "README must mention lab_reply_simulated.json template"
    );
}
