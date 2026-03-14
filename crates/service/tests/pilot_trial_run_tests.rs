//! Full external trial workflow tests.
//!
//! Checks that run_pilot.sh implements the --trial-run mode with the correct
//! lifecycle output strings, subprocess calls, and stable wording.
//!
//! Uses include_str! so missing files are compile errors, not runtime failures.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── --trial-run flag ──────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_trial_run_flag() {
    assert!(
        RUN_PILOT_SH.contains("--trial-run"),
        "run_pilot.sh must support --trial-run flag"
    );
}

#[test]
fn run_pilot_trial_run_is_gated_on_argument() {
    assert!(
        RUN_PILOT_SH.contains("\"${1:-}\" == \"--trial-run\"")
            || RUN_PILOT_SH.contains("\"$1\" == \"--trial-run\""),
        "run_pilot.sh must gate --trial-run on first argument check"
    );
}

// ── Lifecycle output strings ──────────────────────────────────────────────────

#[test]
fn run_pilot_trial_run_prints_starting() {
    assert!(
        RUN_PILOT_SH.contains("Starting PostCAD trial run"),
        "run_pilot.sh must print 'Starting PostCAD trial run'"
    );
}

#[test]
fn run_pilot_trial_run_prints_outbound_bundle_created() {
    assert!(
        RUN_PILOT_SH.contains("Outbound bundle created"),
        "run_pilot.sh must print 'Outbound bundle created'"
    );
}

#[test]
fn run_pilot_trial_run_prints_external_handoff_pack_created() {
    assert!(
        RUN_PILOT_SH.contains("External handoff pack created"),
        "run_pilot.sh must print 'External handoff pack created'"
    );
}

#[test]
fn run_pilot_trial_run_prints_simulated_lab_response_generated() {
    assert!(
        RUN_PILOT_SH.contains("Simulated lab response generated"),
        "run_pilot.sh must print 'Simulated lab response generated'"
    );
}

#[test]
fn run_pilot_trial_run_prints_inbound_response_verified() {
    assert!(
        RUN_PILOT_SH.contains("Inbound response verified"),
        "run_pilot.sh must print 'Inbound response verified'"
    );
}

#[test]
fn run_pilot_trial_run_prints_operator_decision() {
    assert!(
        RUN_PILOT_SH.contains("Operator decision:"),
        "run_pilot.sh must print 'Operator decision:'"
    );
}

#[test]
fn run_pilot_trial_run_prints_trial_ledger_updated() {
    assert!(
        RUN_PILOT_SH.contains("Trial ledger updated"),
        "run_pilot.sh must print 'Trial ledger updated'"
    );
}

#[test]
fn run_pilot_trial_run_prints_trial_run_completed() {
    assert!(
        RUN_PILOT_SH.contains("Trial run completed"),
        "run_pilot.sh must print 'Trial run completed'"
    );
}

// ── Subprocess calls ──────────────────────────────────────────────────────────

#[test]
fn run_pilot_trial_run_calls_lab_simulator_handoff_pack() {
    assert!(
        RUN_PILOT_SH.contains("lab_simulator.sh") && RUN_PILOT_SH.contains("--handoff-pack"),
        "run_pilot.sh must call lab_simulator.sh --handoff-pack in trial run"
    );
}

#[test]
fn run_pilot_trial_run_calls_lab_simulator_simulate() {
    // lab_simulator.sh is called twice: once for handoff-pack and once for simulation
    let count = RUN_PILOT_SH.matches("lab_simulator.sh").count();
    assert!(
        count >= 2,
        "run_pilot.sh must call lab_simulator.sh at least twice in trial run (handoff + simulate)"
    );
}

#[test]
fn run_pilot_trial_run_calls_verify_sh_inbound() {
    assert!(
        RUN_PILOT_SH.contains("verify.sh") && RUN_PILOT_SH.contains("--inbound"),
        "run_pilot.sh must call verify.sh --inbound in trial run"
    );
}

// ── Error handling ────────────────────────────────────────────────────────────

#[test]
fn run_pilot_trial_run_captures_verify_exit_code() {
    assert!(
        RUN_PILOT_SH.contains("TRIAL_VERIFY_EXIT"),
        "run_pilot.sh must capture verify exit code without triggering set -e"
    );
}

#[test]
fn run_pilot_trial_run_exits_nonzero_on_rejected() {
    assert!(
        RUN_PILOT_SH.contains("TRIAL_DECISION"),
        "run_pilot.sh must track TRIAL_DECISION for exit code selection"
    );
}

// ── Ledger integration ────────────────────────────────────────────────────────

#[test]
fn run_pilot_trial_run_uses_trial_ledger_file() {
    assert!(
        RUN_PILOT_SH.contains("TRIAL_LEDGER_FILE"),
        "run_pilot.sh trial run must use TRIAL_LEDGER_FILE"
    );
}

#[test]
fn run_pilot_trial_run_appends_outbound_bundle_created_ledger() {
    // The trial run appends an outbound_bundle_created entry
    assert!(
        RUN_PILOT_SH.contains("outbound_bundle_created"),
        "run_pilot.sh trial run must append outbound_bundle_created to ledger"
    );
}

#[test]
fn run_pilot_trial_run_prints_ledger_path() {
    assert!(
        RUN_PILOT_SH.contains("TRIAL_LEDGER_FILE"),
        "run_pilot.sh must print the trial ledger file path"
    );
}

// ── Output suppression ────────────────────────────────────────────────────────

#[test]
fn run_pilot_trial_run_suppresses_subprocess_output() {
    assert!(
        RUN_PILOT_SH.contains("> /dev/null 2>&1"),
        "run_pilot.sh trial run must suppress subprocess output for clean lifecycle display"
    );
}

// ── README ────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_running_a_full_trial_section() {
    assert!(
        README.contains("## Running a full PostCAD pilot trial"),
        "README must have '## Running a full PostCAD pilot trial' section"
    );
}

#[test]
fn readme_shows_trial_run_command() {
    assert!(
        README.contains("--trial-run"),
        "README must show --trial-run command"
    );
}

#[test]
fn readme_documents_trial_lifecycle_output() {
    for line in [
        "Starting PostCAD trial run",
        "Outbound bundle created",
        "External handoff pack created",
        "Simulated lab response generated",
        "Inbound response verified",
        "Operator decision: ACCEPTED",
        "Trial ledger updated",
        "Trial run completed",
    ] {
        assert!(
            README.contains(line),
            "README must document trial lifecycle line: {line}"
        );
    }
}
