//! Run summary surface tests.
//!
//! Checks that run_pilot.sh --run-summary exists and prints a deterministic
//! summary with run context, artifact status, next operator action, and
//! command hints.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_run_summary_flag() {
    assert!(
        RUN_PILOT_SH.contains("--run-summary"),
        "run_pilot.sh must support --run-summary flag"
    );
}

#[test]
fn run_summary_exits_0() {
    assert!(
        RUN_PILOT_SH.contains("--run-summary") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --run-summary must exit 0"
    );
}

// ── header ────────────────────────────────────────────────────────────────────

#[test]
fn run_summary_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("PostCAD — Pilot Run Summary"),
        "run_pilot.sh must print 'PostCAD — Pilot Run Summary' header"
    );
}

// ── RUN CONTEXT section ───────────────────────────────────────────────────────

#[test]
fn run_summary_has_run_context_section() {
    assert!(
        RUN_PILOT_SH.contains("RUN CONTEXT"),
        "run_pilot.sh --run-summary must include 'RUN CONTEXT' section"
    );
}

#[test]
fn run_summary_run_context_shows_run_id() {
    assert!(
        RUN_PILOT_SH.contains("Run ID :"),
        "run_pilot.sh --run-summary must print 'Run ID :' in RUN CONTEXT"
    );
}

#[test]
fn run_summary_run_context_fallback_not_detected() {
    assert!(
        RUN_PILOT_SH.contains("not detected"),
        "run_pilot.sh --run-summary must print 'not detected' when run ID cannot be resolved"
    );
}

// ── ARTIFACT STATUS section ───────────────────────────────────────────────────

#[test]
fn run_summary_has_artifact_status_section() {
    assert!(
        RUN_PILOT_SH.contains("ARTIFACT STATUS"),
        "run_pilot.sh --run-summary must include 'ARTIFACT STATUS' section"
    );
}

#[test]
fn run_summary_artifact_status_shows_receipt_json() {
    assert!(
        RUN_PILOT_SH.contains("receipt.json"),
        "run_pilot.sh --run-summary ARTIFACT STATUS must show receipt.json"
    );
}

#[test]
fn run_summary_artifact_status_shows_inbound_lab_reply() {
    assert!(
        RUN_PILOT_SH.contains("inbound lab reply"),
        "run_pilot.sh --run-summary ARTIFACT STATUS must show 'inbound lab reply'"
    );
}

#[test]
fn run_summary_artifact_status_shows_verification_result() {
    assert!(
        RUN_PILOT_SH.contains("verification result"),
        "run_pilot.sh --run-summary ARTIFACT STATUS must show 'verification result'"
    );
}

#[test]
fn run_summary_artifact_status_shows_dispatch_packet() {
    assert!(
        RUN_PILOT_SH.contains("dispatch packet"),
        "run_pilot.sh --run-summary ARTIFACT STATUS must show 'dispatch packet'"
    );
}

#[test]
fn run_summary_artifact_status_uses_present_label() {
    assert!(
        RUN_PILOT_SH.contains("present"),
        "run_pilot.sh --run-summary must use 'present' status label"
    );
}

#[test]
fn run_summary_artifact_status_uses_missing_label() {
    assert!(
        RUN_PILOT_SH.contains("missing"),
        "run_pilot.sh --run-summary must use 'missing' status label"
    );
}

#[test]
fn run_summary_artifact_status_uses_not_yet_generated_label() {
    assert!(
        RUN_PILOT_SH.contains("not yet generated"),
        "run_pilot.sh --run-summary must use 'not yet generated' status label"
    );
}

// ── NEXT OPERATOR ACTION section ──────────────────────────────────────────────

#[test]
fn run_summary_has_next_operator_action_section() {
    assert!(
        RUN_PILOT_SH.contains("NEXT OPERATOR ACTION"),
        "run_pilot.sh --run-summary must include 'NEXT OPERATOR ACTION' section"
    );
}

#[test]
fn run_summary_next_action_generate_bundle_when_no_receipt() {
    assert!(
        RUN_PILOT_SH.contains("generate pilot bundle"),
        "run_pilot.sh --run-summary must suggest 'generate pilot bundle' when no receipt"
    );
}

#[test]
fn run_summary_next_action_inspect_reply() {
    assert!(
        RUN_PILOT_SH.contains("inspect inbound lab reply"),
        "run_pilot.sh --run-summary must suggest 'inspect inbound lab reply'"
    );
}

#[test]
fn run_summary_next_action_verify_reply() {
    assert!(
        RUN_PILOT_SH.contains("verify inbound reply"),
        "run_pilot.sh --run-summary must suggest 'verify inbound reply'"
    );
}

#[test]
fn run_summary_next_action_export_dispatch() {
    assert!(
        RUN_PILOT_SH.contains("export dispatch packet"),
        "run_pilot.sh --run-summary must suggest 'export dispatch packet'"
    );
}

// ── OPERATOR COMMAND HINTS section ────────────────────────────────────────────

#[test]
fn run_summary_has_operator_command_hints_section() {
    assert!(
        RUN_PILOT_SH.contains("OPERATOR COMMAND HINTS"),
        "run_pilot.sh --run-summary must include 'OPERATOR COMMAND HINTS' section"
    );
}

#[test]
fn run_summary_hints_mentions_quickstart() {
    assert!(
        RUN_PILOT_SH.contains("--quickstart"),
        "run_pilot.sh --run-summary command hints must mention --quickstart"
    );
}

#[test]
fn run_summary_hints_mentions_artifact_index() {
    assert!(
        RUN_PILOT_SH.contains("--artifact-index"),
        "run_pilot.sh --run-summary command hints must mention --artifact-index"
    );
}

#[test]
fn run_summary_hints_mentions_walkthrough() {
    assert!(
        RUN_PILOT_SH.contains("--walkthrough"),
        "run_pilot.sh --run-summary command hints must mention --walkthrough"
    );
}

// ── determinism ───────────────────────────────────────────────────────────────

#[test]
fn run_summary_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("Pilot Run Summary")
        .expect("run summary header must exist");
    let block_end = RUN_PILOT_SH[block_start..]
        .find("exit 0")
        .map(|i| block_start + i + 6)
        .unwrap_or(block_start + 3000);
    let block = &RUN_PILOT_SH[block_start..block_end];
    assert!(
        !block.contains("$(date"),
        "run-summary block must not embed timestamps"
    );
}

// ── README section ────────────────────────────────────────────────────────────

#[test]
fn readme_has_run_summary_section() {
    assert!(
        README.contains("## Run Summary"),
        "README must have '## Run Summary' section"
    );
}

#[test]
fn readme_shows_run_summary_command() {
    assert!(
        README.contains("--run-summary"),
        "README must show --run-summary command"
    );
}
