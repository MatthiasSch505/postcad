//! Artifact index summary tests.
//!
//! Checks that run_pilot.sh --artifact-index exists, prints a deterministic
//! artifact map covering all pilot workflow artifact categories, and includes
//! an operator flow reminder.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_artifact_index_flag() {
    assert!(
        RUN_PILOT_SH.contains("--artifact-index"),
        "run_pilot.sh must support --artifact-index flag"
    );
}

#[test]
fn artifact_index_exits_0() {
    assert!(
        RUN_PILOT_SH.contains("--artifact-index") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --artifact-index must exit 0"
    );
}

// ── header ────────────────────────────────────────────────────────────────────

#[test]
fn artifact_index_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("PostCAD — Pilot Artifact Index"),
        "run_pilot.sh must print 'PostCAD — Pilot Artifact Index' header"
    );
}

// ── artifact categories ───────────────────────────────────────────────────────

#[test]
fn artifact_index_shows_pilot_bundle_section() {
    assert!(
        RUN_PILOT_SH.contains("Pilot bundle"),
        "run_pilot.sh --artifact-index must show 'Pilot bundle' section"
    );
}

#[test]
fn artifact_index_shows_receipt_json() {
    assert!(
        RUN_PILOT_SH.contains("receipt.json"),
        "run_pilot.sh --artifact-index must show receipt.json"
    );
}

#[test]
fn artifact_index_shows_export_packet_json() {
    assert!(
        RUN_PILOT_SH.contains("export_packet.json"),
        "run_pilot.sh --artifact-index must show export_packet.json"
    );
}

#[test]
fn artifact_index_shows_inbound_replies_section() {
    assert!(
        RUN_PILOT_SH.contains("Inbound replies"),
        "run_pilot.sh --artifact-index must show 'Inbound replies' section"
    );
}

#[test]
fn artifact_index_shows_inbound_directory() {
    assert!(
        RUN_PILOT_SH.contains("inbound/"),
        "run_pilot.sh --artifact-index must show inbound/ directory"
    );
}

#[test]
fn artifact_index_shows_outbound_packages_section() {
    assert!(
        RUN_PILOT_SH.contains("Outbound packages"),
        "run_pilot.sh --artifact-index must show 'Outbound packages' section"
    );
}

#[test]
fn artifact_index_shows_outbound_directory() {
    assert!(
        RUN_PILOT_SH.contains("outbound/"),
        "run_pilot.sh --artifact-index must show outbound/ directory"
    );
}

#[test]
fn artifact_index_shows_decision_records_section() {
    assert!(
        RUN_PILOT_SH.contains("Decision records"),
        "run_pilot.sh --artifact-index must show 'Decision records' section"
    );
}

#[test]
fn artifact_index_shows_reports_directory() {
    assert!(
        RUN_PILOT_SH.contains("reports/"),
        "run_pilot.sh --artifact-index must show reports/ directory"
    );
}

#[test]
fn artifact_index_shows_verification_section() {
    assert!(
        RUN_PILOT_SH.contains("Verification"),
        "run_pilot.sh --artifact-index must show 'Verification' section"
    );
}

#[test]
fn artifact_index_shows_verify_sh_command() {
    assert!(
        RUN_PILOT_SH.contains("./examples/pilot/verify.sh"),
        "run_pilot.sh --artifact-index must reference verify.sh command"
    );
}

// ── operator flow reminder ────────────────────────────────────────────────────

#[test]
fn artifact_index_has_operator_flow_reminder() {
    assert!(
        RUN_PILOT_SH.contains("Operator flow reminder"),
        "run_pilot.sh --artifact-index must print 'Operator flow reminder'"
    );
}

#[test]
fn artifact_index_reminder_includes_inspect_step() {
    assert!(
        RUN_PILOT_SH.contains("inspect inbound reply"),
        "run_pilot.sh --artifact-index operator flow reminder must include 'inspect inbound reply'"
    );
}

#[test]
fn artifact_index_reminder_includes_verify_step() {
    assert!(
        RUN_PILOT_SH.contains("verify inbound reply"),
        "run_pilot.sh --artifact-index operator flow reminder must include 'verify inbound reply'"
    );
}

#[test]
fn artifact_index_reminder_includes_export_dispatch_step() {
    assert!(
        RUN_PILOT_SH.contains("export dispatch packet"),
        "run_pilot.sh --artifact-index operator flow reminder must include 'export dispatch packet'"
    );
}

// ── determinism — no non-deterministic content ────────────────────────────────

#[test]
fn artifact_index_does_not_use_date_command() {
    // The artifact-index block must not embed timestamps
    let block_start = RUN_PILOT_SH
        .find("--artifact-index")
        .expect("--artifact-index must exist");
    let block_end = RUN_PILOT_SH[block_start..]
        .find("exit 0")
        .map(|i| block_start + i + 6)
        .unwrap_or(block_start + 500);
    let block = &RUN_PILOT_SH[block_start..block_end];
    assert!(
        !block.contains("$(date"),
        "artifact-index block must not embed timestamps via $(date ...)"
    );
}

// ── README section ────────────────────────────────────────────────────────────

#[test]
fn readme_has_artifact_index_section() {
    assert!(
        README.contains("## Artifact Index"),
        "README must have '## Artifact Index' section"
    );
}

#[test]
fn readme_shows_artifact_index_command() {
    assert!(
        README.contains("--artifact-index"),
        "README must show --artifact-index command"
    );
}

#[test]
fn readme_shows_artifact_index_expected_output() {
    assert!(
        README.contains("PostCAD — Pilot Artifact Index"),
        "README must show expected output header for --artifact-index"
    );
}

#[test]
fn readme_artifact_index_mentions_no_files_written() {
    assert!(
        README.contains("No files are written") || README.contains("No commands are executed"),
        "README must clarify that --artifact-index only prints — no files written"
    );
}
