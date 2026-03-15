//! Quickstart command sheet tests.
//!
//! Checks that run_pilot.sh --quickstart exists, prints a deterministic
//! command sheet with all required workflow steps, and each step includes
//! the exact command and a one-line explanation.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_quickstart_flag() {
    assert!(
        RUN_PILOT_SH.contains("--quickstart"),
        "run_pilot.sh must support --quickstart flag"
    );
}

#[test]
fn quickstart_exits_0() {
    assert!(
        RUN_PILOT_SH.contains("--quickstart") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --quickstart must exit 0"
    );
}

// ── header ────────────────────────────────────────────────────────────────────

#[test]
fn quickstart_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("PostCAD Pilot — Quickstart Command Sheet"),
        "run_pilot.sh must print 'PostCAD Pilot — Quickstart Command Sheet' header"
    );
}

// ── step: generate pilot bundle ───────────────────────────────────────────────

#[test]
fn quickstart_has_generate_bundle_step() {
    assert!(
        RUN_PILOT_SH.contains("Generate pilot bundle"),
        "run_pilot.sh --quickstart must include 'Generate pilot bundle' step"
    );
}

#[test]
fn quickstart_generate_bundle_shows_command() {
    assert!(
        RUN_PILOT_SH.contains("./examples/pilot/run_pilot.sh"),
        "run_pilot.sh --quickstart must show run_pilot.sh as the generate command"
    );
}

#[test]
fn quickstart_generate_bundle_has_explanation() {
    assert!(
        RUN_PILOT_SH.contains("Routes the dental case and writes a cryptographic receipt"),
        "run_pilot.sh --quickstart generate step must have a one-line explanation"
    );
}

// ── step: inspect inbound lab reply ──────────────────────────────────────────

#[test]
fn quickstart_has_inspect_reply_step() {
    assert!(
        RUN_PILOT_SH.contains("Inspect inbound lab reply"),
        "run_pilot.sh --quickstart must include 'Inspect inbound lab reply' step"
    );
}

#[test]
fn quickstart_inspect_reply_shows_command() {
    assert!(
        RUN_PILOT_SH.contains("--inspect-inbound-reply inbound/lab_reply_<run-id>.json"),
        "run_pilot.sh --quickstart must show --inspect-inbound-reply command with placeholder"
    );
}

#[test]
fn quickstart_inspect_reply_has_explanation() {
    assert!(
        RUN_PILOT_SH.contains("Checks that all required fields are present before verification"),
        "run_pilot.sh --quickstart inspect step must have a one-line explanation"
    );
}

// ── step: verify inbound reply ────────────────────────────────────────────────

#[test]
fn quickstart_has_verify_reply_step() {
    assert!(
        RUN_PILOT_SH.contains("Verify inbound reply"),
        "run_pilot.sh --quickstart must include 'Verify inbound reply' step"
    );
}

#[test]
fn quickstart_verify_reply_shows_command() {
    assert!(
        RUN_PILOT_SH.contains(
            "./examples/pilot/verify.sh --inbound inbound/lab_reply_<run-id>.json --bundle examples/pilot"
        ),
        "run_pilot.sh --quickstart must show full verify.sh command with placeholders"
    );
}

#[test]
fn quickstart_verify_reply_has_explanation() {
    assert!(
        RUN_PILOT_SH.contains("Cryptographically binds the reply to the current run"),
        "run_pilot.sh --quickstart verify step must have a one-line explanation"
    );
}

// ── step: export dispatch packet ──────────────────────────────────────────────

#[test]
fn quickstart_has_export_dispatch_step() {
    assert!(
        RUN_PILOT_SH.contains("Export dispatch packet"),
        "run_pilot.sh --quickstart must include 'Export dispatch packet' step"
    );
}

#[test]
fn quickstart_export_dispatch_shows_command() {
    assert!(
        RUN_PILOT_SH.contains("./examples/pilot/run_pilot.sh --export-dispatch"),
        "run_pilot.sh --quickstart must show --export-dispatch command"
    );
}

#[test]
fn quickstart_export_dispatch_has_explanation() {
    assert!(
        RUN_PILOT_SH.contains("Confirms the dispatch packet is ready"),
        "run_pilot.sh --quickstart export step must have a one-line explanation"
    );
}

// ── step: show artifact index ─────────────────────────────────────────────────

#[test]
fn quickstart_has_artifact_index_step() {
    assert!(
        RUN_PILOT_SH.contains("Show artifact index"),
        "run_pilot.sh --quickstart must include 'Show artifact index' step"
    );
}

#[test]
fn quickstart_artifact_index_shows_command() {
    assert!(
        RUN_PILOT_SH.contains("./examples/pilot/run_pilot.sh --artifact-index"),
        "run_pilot.sh --quickstart must show --artifact-index command"
    );
}

#[test]
fn quickstart_artifact_index_has_explanation() {
    assert!(
        RUN_PILOT_SH.contains("Prints the artifact map for the current run"),
        "run_pilot.sh --quickstart artifact-index step must have a one-line explanation"
    );
}

// ── step: show walkthrough ────────────────────────────────────────────────────

#[test]
fn quickstart_has_walkthrough_step() {
    assert!(
        RUN_PILOT_SH.contains("Show walkthrough"),
        "run_pilot.sh --quickstart must include 'Show walkthrough' step"
    );
}

#[test]
fn quickstart_walkthrough_shows_command() {
    assert!(
        RUN_PILOT_SH.contains("./examples/pilot/run_pilot.sh --walkthrough"),
        "run_pilot.sh --quickstart must show --walkthrough command"
    );
}

#[test]
fn quickstart_walkthrough_has_explanation() {
    assert!(
        RUN_PILOT_SH.contains("Prints the full 4-step pilot workflow guide"),
        "run_pilot.sh --quickstart walkthrough step must have a one-line explanation"
    );
}

// ── determinism ───────────────────────────────────────────────────────────────

#[test]
fn quickstart_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("Quickstart Command Sheet")
        .expect("quickstart header must exist");
    let block_end = RUN_PILOT_SH[block_start..]
        .find("exit 0")
        .map(|i| block_start + i + 6)
        .unwrap_or(block_start + 1000);
    let block = &RUN_PILOT_SH[block_start..block_end];
    assert!(
        !block.contains("$(date"),
        "quickstart block must not embed timestamps"
    );
}

// ── README section ────────────────────────────────────────────────────────────

#[test]
fn readme_has_quickstart_section() {
    assert!(
        README.contains("## Quickstart"),
        "README must have '## Quickstart' section"
    );
}

#[test]
fn readme_shows_quickstart_command() {
    assert!(
        README.contains("--quickstart"),
        "README must show --quickstart command"
    );
}

#[test]
fn readme_quickstart_describes_purpose() {
    assert!(
        README.contains("new operator") || README.contains("fastest way"),
        "README Quickstart section must describe its purpose for new operators"
    );
}
