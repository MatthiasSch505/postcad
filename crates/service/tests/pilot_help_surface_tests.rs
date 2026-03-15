//! Consolidated help surface tests.
//!
//! Checks that run_pilot.sh --help-surface exists, lists all main pilot
//! operator modes, and includes a recommended order section.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_help_surface_flag() {
    assert!(
        RUN_PILOT_SH.contains("--help-surface"),
        "run_pilot.sh must support --help-surface flag"
    );
}

#[test]
fn help_surface_exits_0() {
    assert!(
        RUN_PILOT_SH.contains("--help-surface") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --help-surface must exit 0"
    );
}

// ── header ────────────────────────────────────────────────────────────────────

#[test]
fn help_surface_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("PostCAD Pilot — Operator Mode Reference"),
        "run_pilot.sh must print 'PostCAD Pilot — Operator Mode Reference' header"
    );
}

// ── mode entries ──────────────────────────────────────────────────────────────

#[test]
fn help_surface_includes_default_mode() {
    assert!(
        RUN_PILOT_SH.contains("(default)"),
        "run_pilot.sh --help-surface must document the default pilot bundle generation mode"
    );
}

#[test]
fn help_surface_includes_inspect_inbound_reply_mode() {
    assert!(
        RUN_PILOT_SH.contains("--inspect-inbound-reply <file>"),
        "run_pilot.sh --help-surface must document --inspect-inbound-reply mode"
    );
}

#[test]
fn help_surface_includes_export_dispatch_mode() {
    assert!(
        RUN_PILOT_SH.contains("--export-dispatch"),
        "run_pilot.sh --help-surface must document --export-dispatch mode"
    );
}

#[test]
fn help_surface_includes_walkthrough_mode() {
    assert!(
        RUN_PILOT_SH.contains("--walkthrough"),
        "run_pilot.sh --help-surface must document --walkthrough mode"
    );
}

#[test]
fn help_surface_includes_artifact_index_mode() {
    assert!(
        RUN_PILOT_SH.contains("--artifact-index"),
        "run_pilot.sh --help-surface must document --artifact-index mode"
    );
}

#[test]
fn help_surface_includes_quickstart_mode() {
    assert!(
        RUN_PILOT_SH.contains("--quickstart"),
        "run_pilot.sh --help-surface must document --quickstart mode"
    );
}

// ── purpose and use-when fields ───────────────────────────────────────────────

#[test]
fn help_surface_entries_have_purpose_field() {
    assert!(
        RUN_PILOT_SH.contains("Purpose :"),
        "run_pilot.sh --help-surface mode entries must include a Purpose field"
    );
}

#[test]
fn help_surface_entries_have_use_when_field() {
    assert!(
        RUN_PILOT_SH.contains("Use when:"),
        "run_pilot.sh --help-surface mode entries must include a Use when field"
    );
}

// ── recommended order section ─────────────────────────────────────────────────

#[test]
fn help_surface_has_recommended_order_section() {
    assert!(
        RUN_PILOT_SH.contains("Recommended order"),
        "run_pilot.sh --help-surface must include a 'Recommended order' section"
    );
}

#[test]
fn help_surface_recommended_order_step1_generate_bundle() {
    assert!(
        RUN_PILOT_SH.contains("generate pilot bundle"),
        "run_pilot.sh --help-surface recommended order must include 'generate pilot bundle'"
    );
}

#[test]
fn help_surface_recommended_order_step2_inspect_reply() {
    assert!(
        RUN_PILOT_SH.contains("inspect inbound reply"),
        "run_pilot.sh --help-surface recommended order must include 'inspect inbound reply'"
    );
}

#[test]
fn help_surface_recommended_order_step3_verify_reply() {
    assert!(
        RUN_PILOT_SH.contains("verify inbound reply"),
        "run_pilot.sh --help-surface recommended order must include 'verify inbound reply'"
    );
}

#[test]
fn help_surface_recommended_order_step4_export_dispatch() {
    assert!(
        RUN_PILOT_SH.contains("export dispatch packet"),
        "run_pilot.sh --help-surface recommended order must include 'export dispatch packet'"
    );
}

#[test]
fn help_surface_recommended_order_references_verify_sh() {
    assert!(
        RUN_PILOT_SH.contains("./examples/pilot/verify.sh"),
        "run_pilot.sh --help-surface recommended order must reference verify.sh for the verify step"
    );
}

// ── determinism ───────────────────────────────────────────────────────────────

#[test]
fn help_surface_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("Operator Mode Reference")
        .expect("help-surface header must exist");
    let block_end = RUN_PILOT_SH[block_start..]
        .find("exit 0")
        .map(|i| block_start + i + 6)
        .unwrap_or(block_start + 2000);
    let block = &RUN_PILOT_SH[block_start..block_end];
    assert!(
        !block.contains("$(date"),
        "help-surface block must not embed timestamps"
    );
}

// ── README section ────────────────────────────────────────────────────────────

#[test]
fn readme_has_help_surface_section() {
    assert!(
        README.contains("## Help Surface"),
        "README must have '## Help Surface' section"
    );
}

#[test]
fn readme_shows_help_surface_command() {
    assert!(
        README.contains("--help-surface"),
        "README must show --help-surface command"
    );
}

#[test]
fn readme_help_surface_describes_first_time_operator_use() {
    assert!(
        README.contains("first-time operator") || README.contains("best starting point"),
        "README Help Surface section must describe it as the starting point for first-time operators"
    );
}
