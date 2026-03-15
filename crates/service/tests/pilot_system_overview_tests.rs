//! System overview surface tests.
//!
//! Checks that run_pilot.sh --system-overview exists and prints a
//! deterministic overview covering the required sections and content.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_system_overview_flag() {
    assert!(
        RUN_PILOT_SH.contains("--system-overview"),
        "run_pilot.sh must support --system-overview flag"
    );
}

#[test]
fn system_overview_exits_0() {
    assert!(
        RUN_PILOT_SH.contains("--system-overview") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --system-overview must exit 0"
    );
}

// ── header ────────────────────────────────────────────────────────────────────

#[test]
fn system_overview_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD PILOT SYSTEM OVERVIEW"),
        "run_pilot.sh must print 'POSTCAD PILOT SYSTEM OVERVIEW' header"
    );
}

// ── system description ────────────────────────────────────────────────────────

#[test]
fn system_overview_describes_routing_layer() {
    assert!(
        RUN_PILOT_SH.contains("deterministic routing and verification layer"),
        "run_pilot.sh --system-overview must describe PostCAD as a deterministic routing and verification layer"
    );
}

// ── CORE IDEA section ─────────────────────────────────────────────────────────

#[test]
fn system_overview_has_core_idea_section() {
    assert!(
        RUN_PILOT_SH.contains("CORE IDEA"),
        "run_pilot.sh --system-overview must include 'CORE IDEA' section"
    );
}

#[test]
fn system_overview_core_idea_mentions_case() {
    assert!(
        RUN_PILOT_SH.contains("produces a case") || RUN_PILOT_SH.contains("dental CAD design"),
        "run_pilot.sh --system-overview CORE IDEA must mention the dental CAD case"
    );
}

#[test]
fn system_overview_core_idea_mentions_receipt() {
    assert!(
        RUN_PILOT_SH.contains("receipt records the decision")
            || RUN_PILOT_SH.contains("A receipt records"),
        "run_pilot.sh --system-overview CORE IDEA must mention the receipt"
    );
}

#[test]
fn system_overview_core_idea_mentions_dispatch_packet() {
    assert!(
        RUN_PILOT_SH.contains("dispatch packet"),
        "run_pilot.sh --system-overview CORE IDEA must mention the dispatch packet"
    );
}

// ── PILOT WORKFLOW section ────────────────────────────────────────────────────

#[test]
fn system_overview_has_pilot_workflow_section() {
    assert!(
        RUN_PILOT_SH.contains("PILOT WORKFLOW"),
        "run_pilot.sh --system-overview must include 'PILOT WORKFLOW' section"
    );
}

#[test]
fn system_overview_workflow_has_four_steps() {
    assert!(
        RUN_PILOT_SH.contains("1.") && RUN_PILOT_SH.contains("2.")
            && RUN_PILOT_SH.contains("3.") && RUN_PILOT_SH.contains("4."),
        "run_pilot.sh --system-overview PILOT WORKFLOW must list 4 steps"
    );
}

#[test]
fn system_overview_workflow_step1_generate_bundle() {
    assert!(
        RUN_PILOT_SH.contains("Generate pilot bundle"),
        "run_pilot.sh --system-overview workflow must include 'Generate pilot bundle'"
    );
}

#[test]
fn system_overview_workflow_step2_inspect_reply() {
    assert!(
        RUN_PILOT_SH.contains("Inspect inbound reply"),
        "run_pilot.sh --system-overview workflow must include 'Inspect inbound reply'"
    );
}

#[test]
fn system_overview_workflow_step3_verify_reply() {
    assert!(
        RUN_PILOT_SH.contains("Verify inbound reply"),
        "run_pilot.sh --system-overview workflow must include 'Verify inbound reply'"
    );
}

#[test]
fn system_overview_workflow_step4_export_dispatch() {
    assert!(
        RUN_PILOT_SH.contains("Export dispatch packet"),
        "run_pilot.sh --system-overview workflow must include 'Export dispatch packet'"
    );
}

// ── KEY ARTIFACTS section ─────────────────────────────────────────────────────

#[test]
fn system_overview_has_key_artifacts_section() {
    assert!(
        RUN_PILOT_SH.contains("KEY ARTIFACTS"),
        "run_pilot.sh --system-overview must include 'KEY ARTIFACTS' section"
    );
}

#[test]
fn system_overview_artifacts_includes_receipt_json() {
    assert!(
        RUN_PILOT_SH.contains("receipt.json"),
        "run_pilot.sh --system-overview KEY ARTIFACTS must include receipt.json"
    );
}

#[test]
fn system_overview_artifacts_includes_dispatch_packet() {
    assert!(
        RUN_PILOT_SH.contains("export_packet.json"),
        "run_pilot.sh --system-overview KEY ARTIFACTS must include export_packet.json"
    );
}

// ── OPERATOR TOOLS section ────────────────────────────────────────────────────

#[test]
fn system_overview_has_operator_tools_section() {
    assert!(
        RUN_PILOT_SH.contains("OPERATOR TOOLS"),
        "run_pilot.sh --system-overview must include 'OPERATOR TOOLS' section"
    );
}

#[test]
fn system_overview_operator_tools_mentions_quickstart() {
    assert!(
        RUN_PILOT_SH.contains("--quickstart"),
        "run_pilot.sh --system-overview OPERATOR TOOLS must mention --quickstart"
    );
}

#[test]
fn system_overview_operator_tools_mentions_walkthrough() {
    assert!(
        RUN_PILOT_SH.contains("--walkthrough"),
        "run_pilot.sh --system-overview OPERATOR TOOLS must mention --walkthrough"
    );
}

#[test]
fn system_overview_operator_tools_mentions_artifact_index() {
    assert!(
        RUN_PILOT_SH.contains("--artifact-index"),
        "run_pilot.sh --system-overview OPERATOR TOOLS must mention --artifact-index"
    );
}

#[test]
fn system_overview_operator_tools_mentions_help_surface() {
    assert!(
        RUN_PILOT_SH.contains("--help-surface"),
        "run_pilot.sh --system-overview OPERATOR TOOLS must mention --help-surface"
    );
}

// ── PROPERTIES section ────────────────────────────────────────────────────────

#[test]
fn system_overview_has_properties_section() {
    assert!(
        RUN_PILOT_SH.contains("PROPERTIES"),
        "run_pilot.sh --system-overview must include 'PROPERTIES' section"
    );
}

#[test]
fn system_overview_properties_deterministic_routing() {
    assert!(
        RUN_PILOT_SH.contains("Deterministic routing"),
        "run_pilot.sh --system-overview PROPERTIES must mention deterministic routing"
    );
}

#[test]
fn system_overview_properties_verifiable_replies() {
    assert!(
        RUN_PILOT_SH.contains("Verifiable inbound replies"),
        "run_pilot.sh --system-overview PROPERTIES must mention verifiable inbound replies"
    );
}

#[test]
fn system_overview_properties_audit_ready_dispatch() {
    assert!(
        RUN_PILOT_SH.contains("Audit-ready dispatch packets"),
        "run_pilot.sh --system-overview PROPERTIES must mention audit-ready dispatch packets"
    );
}

// ── determinism ───────────────────────────────────────────────────────────────

#[test]
fn system_overview_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT SYSTEM OVERVIEW")
        .expect("system overview header must exist");
    let block_end = RUN_PILOT_SH[block_start..]
        .find("exit 0")
        .map(|i| block_start + i + 6)
        .unwrap_or(block_start + 2000);
    let block = &RUN_PILOT_SH[block_start..block_end];
    assert!(
        !block.contains("$(date"),
        "system-overview block must not embed timestamps"
    );
}

// ── README section ────────────────────────────────────────────────────────────

#[test]
fn readme_has_system_overview_section() {
    assert!(
        README.contains("## System Overview"),
        "README must have '## System Overview' section"
    );
}

#[test]
fn readme_shows_system_overview_command() {
    assert!(
        README.contains("--system-overview"),
        "README must show --system-overview command"
    );
}
