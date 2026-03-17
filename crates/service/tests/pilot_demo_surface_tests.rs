//! Demo surface tests.
//!
//! Checks that run_pilot.sh --demo-surface exists and prints a deterministic
//! end-to-end intro covering all required sections and exact commands.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_demo_surface_flag() {
    assert!(
        RUN_PILOT_SH.contains("--demo-surface"),
        "run_pilot.sh must support --demo-surface flag"
    );
}

#[test]
fn demo_surface_exits_0() {
    assert!(
        RUN_PILOT_SH.contains("--demo-surface") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --demo-surface must exit 0"
    );
}

// ── header and description ────────────────────────────────────────────────────

#[test]
fn demo_surface_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD PILOT DEMO"),
        "run_pilot.sh must print 'POSTCAD PILOT DEMO' header"
    );
}

#[test]
fn demo_surface_describes_postcad() {
    assert!(
        RUN_PILOT_SH.contains("deterministic routing and verification layer for dental CAD"),
        "run_pilot.sh --demo-surface must describe PostCAD as a deterministic routing and verification layer"
    );
}

// ── END-TO-END FLOW section ───────────────────────────────────────────────────

#[test]
fn demo_surface_has_end_to_end_flow_section() {
    assert!(
        RUN_PILOT_SH.contains("END-TO-END FLOW"),
        "run_pilot.sh --demo-surface must include 'END-TO-END FLOW' section"
    );
}

#[test]
fn demo_surface_flow_includes_generate_bundle() {
    assert!(
        RUN_PILOT_SH.contains("generate pilot bundle"),
        "run_pilot.sh --demo-surface END-TO-END FLOW must include 'generate pilot bundle'"
    );
}

#[test]
fn demo_surface_flow_includes_inspect_reply() {
    assert!(
        RUN_PILOT_SH.contains("inspect inbound reply"),
        "run_pilot.sh --demo-surface END-TO-END FLOW must include 'inspect inbound reply'"
    );
}

#[test]
fn demo_surface_flow_includes_verify_reply() {
    assert!(
        RUN_PILOT_SH.contains("verify inbound reply"),
        "run_pilot.sh --demo-surface END-TO-END FLOW must include 'verify inbound reply'"
    );
}

#[test]
fn demo_surface_flow_includes_export_dispatch() {
    assert!(
        RUN_PILOT_SH.contains("export dispatch packet"),
        "run_pilot.sh --demo-surface END-TO-END FLOW must include 'export dispatch packet'"
    );
}

// ── WHAT THE OPERATOR SEES section ───────────────────────────────────────────

#[test]
fn demo_surface_has_what_operator_sees_section() {
    assert!(
        RUN_PILOT_SH.contains("WHAT THE OPERATOR SEES"),
        "run_pilot.sh --demo-surface must include 'WHAT THE OPERATOR SEES' section"
    );
}

#[test]
fn demo_surface_operator_sees_receipt_json() {
    assert!(
        RUN_PILOT_SH.contains("receipt.json"),
        "run_pilot.sh --demo-surface must mention receipt.json"
    );
}

#[test]
fn demo_surface_operator_sees_inbound_lab_reply() {
    assert!(
        RUN_PILOT_SH.contains("inbound lab reply"),
        "run_pilot.sh --demo-surface must mention 'inbound lab reply'"
    );
}

#[test]
fn demo_surface_operator_sees_verification_outcome() {
    assert!(
        RUN_PILOT_SH.contains("verification outcome"),
        "run_pilot.sh --demo-surface must mention 'verification outcome'"
    );
}

#[test]
fn demo_surface_operator_sees_dispatch_packet() {
    assert!(
        RUN_PILOT_SH.contains("dispatch packet"),
        "run_pilot.sh --demo-surface must mention 'dispatch packet'"
    );
}

// ── WHY THIS MATTERS section ──────────────────────────────────────────────────

#[test]
fn demo_surface_has_why_this_matters_section() {
    assert!(
        RUN_PILOT_SH.contains("WHY THIS MATTERS"),
        "run_pilot.sh --demo-surface must include 'WHY THIS MATTERS' section"
    );
}

#[test]
fn demo_surface_why_mentions_deterministic_routing() {
    assert!(
        RUN_PILOT_SH.contains("Deterministic routing"),
        "run_pilot.sh --demo-surface WHY THIS MATTERS must mention deterministic routing"
    );
}

#[test]
fn demo_surface_why_mentions_verifiable_replies() {
    assert!(
        RUN_PILOT_SH.contains("Verifiable inbound replies"),
        "run_pilot.sh --demo-surface WHY THIS MATTERS must mention verifiable inbound replies"
    );
}

#[test]
fn demo_surface_why_mentions_audit_ready() {
    assert!(
        RUN_PILOT_SH.contains("Audit-ready dispatch workflow"),
        "run_pilot.sh --demo-surface WHY THIS MATTERS must mention audit-ready dispatch workflow"
    );
}

// ── TRY IT section ────────────────────────────────────────────────────────────

#[test]
fn demo_surface_has_try_it_section() {
    assert!(
        RUN_PILOT_SH.contains("TRY IT"),
        "run_pilot.sh --demo-surface must include 'TRY IT' section"
    );
}

#[test]
fn demo_surface_try_it_shows_system_overview_command() {
    assert!(
        RUN_PILOT_SH.contains("./examples/pilot/run_pilot.sh --system-overview"),
        "run_pilot.sh --demo-surface TRY IT must show --system-overview command"
    );
}

#[test]
fn demo_surface_try_it_shows_quickstart_command() {
    assert!(
        RUN_PILOT_SH.contains("./examples/pilot/run_pilot.sh --quickstart"),
        "run_pilot.sh --demo-surface TRY IT must show --quickstart command"
    );
}

#[test]
fn demo_surface_try_it_shows_run_summary_command() {
    assert!(
        RUN_PILOT_SH.contains("./examples/pilot/run_pilot.sh --run-summary"),
        "run_pilot.sh --demo-surface TRY IT must show --run-summary command"
    );
}

#[test]
fn demo_surface_try_it_shows_help_surface_command() {
    assert!(
        RUN_PILOT_SH.contains("./examples/pilot/run_pilot.sh --help-surface"),
        "run_pilot.sh --demo-surface TRY IT must show --help-surface command"
    );
}

// ── determinism ───────────────────────────────────────────────────────────────

#[test]
fn demo_surface_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("demo surface header must exist");
    let block_end = RUN_PILOT_SH[block_start..]
        .find("exit 0")
        .map(|i| block_start + i + 6)
        .unwrap_or(block_start + 2000);
    let block = &RUN_PILOT_SH[block_start..block_end];
    assert!(
        !block.contains("$(date"),
        "demo-surface block must not embed timestamps"
    );
}

// ── README section ────────────────────────────────────────────────────────────

#[test]
fn readme_has_demo_surface_section() {
    assert!(
        README.contains("## Demo Surface"),
        "README must have '## Demo Surface' section"
    );
}

#[test]
fn readme_shows_demo_surface_command() {
    assert!(
        README.contains("--demo-surface"),
        "README must show --demo-surface command"
    );
}

#[test]
fn readme_demo_surface_describes_single_command_intro() {
    assert!(
        README.contains("fastest single-command introduction")
            || README.contains("single-command intro"),
        "README Demo Surface section must describe it as the fastest single-command introduction"
    );
}

// ── pilot-demo pipeline output ────────────────────────────────────────────────

#[test]
fn pilot_demo_prints_postcad_pipeline_header() {
    assert!(
        RUN_PILOT_SH.contains("PostCAD Pipeline"),
        "run_pilot.sh --pilot-demo must print 'PostCAD Pipeline'"
    );
}

#[test]
fn pilot_demo_uses_down_arrows() {
    // Locate the --pilot-demo block and verify it contains the ↓ arrow
    let block_start = RUN_PILOT_SH
        .find("\"--pilot-demo\"")
        .expect("--pilot-demo flag must exist in run_pilot.sh");
    let block_end = RUN_PILOT_SH[block_start..]
        .find("\nfi\n")
        .map(|i| block_start + i + 4)
        .unwrap_or(block_start + 500);
    let block = &RUN_PILOT_SH[block_start..block_end];
    assert!(
        block.contains('↓'),
        "run_pilot.sh --pilot-demo block must use '↓' arrows between pipeline stages"
    );
}

#[test]
fn pilot_demo_includes_postcad_routing_stage() {
    assert!(
        RUN_PILOT_SH.contains("PostCAD routing"),
        "run_pilot.sh --pilot-demo must include 'PostCAD routing' stage"
    );
}

#[test]
fn pilot_demo_includes_compliance_verification_stage() {
    assert!(
        RUN_PILOT_SH.contains("compliance verification"),
        "run_pilot.sh --pilot-demo must include 'compliance verification' stage"
    );
}

#[test]
fn pilot_demo_includes_manufacturing_dispatch_stage() {
    assert!(
        RUN_PILOT_SH.contains("manufacturing dispatch"),
        "run_pilot.sh --pilot-demo must include 'manufacturing dispatch' stage"
    );
}

#[test]
fn pilot_demo_includes_audit_receipt_stage() {
    assert!(
        RUN_PILOT_SH.contains("audit receipt"),
        "run_pilot.sh --pilot-demo must include 'audit receipt' stage"
    );
}

#[test]
fn pilot_demo_block_has_no_timestamps() {
    let block_start = RUN_PILOT_SH
        .find("\"--pilot-demo\"")
        .expect("--pilot-demo flag must exist in run_pilot.sh");
    let block_end = RUN_PILOT_SH[block_start..]
        .find("\nfi\n")
        .map(|i| block_start + i + 4)
        .unwrap_or(block_start + 500);
    let block = &RUN_PILOT_SH[block_start..block_end];
    assert!(
        !block.contains("$(date"),
        "run_pilot.sh --pilot-demo block must not embed timestamps"
    );
}

#[test]
fn pilot_demo_block_has_no_ansi_codes() {
    let block_start = RUN_PILOT_SH
        .find("\"--pilot-demo\"")
        .expect("--pilot-demo flag must exist in run_pilot.sh");
    let block_end = RUN_PILOT_SH[block_start..]
        .find("\nfi\n")
        .map(|i| block_start + i + 4)
        .unwrap_or(block_start + 500);
    let block = &RUN_PILOT_SH[block_start..block_end];
    assert!(
        !block.contains("\\033[") && !block.contains("\\e["),
        "run_pilot.sh --pilot-demo block must not contain ANSI color codes"
    );
}
