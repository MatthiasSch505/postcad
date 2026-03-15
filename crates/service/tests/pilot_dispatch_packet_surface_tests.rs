//! Dispatch packet surface tests.
//!
//! Checks that run_pilot.sh --dispatch-packet exists, prints the required
//! sections and command references, handles missing dispatch artifact
//! deterministically, and that README has the required section.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_dispatch_packet_flag() {
    assert!(
        RUN_PILOT_SH.contains("--dispatch-packet"),
        "run_pilot.sh must support --dispatch-packet flag"
    );
}

#[test]
fn dispatch_packet_exits_0() {
    assert!(
        RUN_PILOT_SH.contains("--dispatch-packet") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --dispatch-packet block must exit 0"
    );
}

// ── header ────────────────────────────────────────────────────────────────────

#[test]
fn dispatch_packet_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD DISPATCH PACKET"),
        "run_pilot.sh must print 'POSTCAD DISPATCH PACKET' header"
    );
}

// ── run context section ───────────────────────────────────────────────────────

#[test]
fn dispatch_packet_has_run_context_section() {
    assert!(
        RUN_PILOT_SH.contains("RUN CONTEXT"),
        "run_pilot.sh --dispatch-packet must print 'RUN CONTEXT' section"
    );
}

#[test]
fn dispatch_packet_prints_run_id_label() {
    assert!(
        RUN_PILOT_SH.contains("Run ID :"),
        "run_pilot.sh --dispatch-packet must print 'Run ID :' label"
    );
}

// ── dispatch artifact section ─────────────────────────────────────────────────

#[test]
fn dispatch_packet_has_dispatch_artifact_section() {
    assert!(
        RUN_PILOT_SH.contains("DISPATCH ARTIFACT"),
        "run_pilot.sh --dispatch-packet must print 'DISPATCH ARTIFACT' section"
    );
}

#[test]
fn dispatch_packet_artifact_describes_execution_handoff() {
    assert!(
        RUN_PILOT_SH.contains("execution-side handoff artifact"),
        "run_pilot.sh --dispatch-packet must describe dispatch packet as 'execution-side handoff artifact'"
    );
}

#[test]
fn dispatch_packet_artifact_describes_verified_state() {
    assert!(
        RUN_PILOT_SH.contains("follows verified workflow state"),
        "run_pilot.sh --dispatch-packet must mention 'follows verified workflow state'"
    );
}

#[test]
fn dispatch_packet_artifact_shows_export_packet_path() {
    assert!(
        RUN_PILOT_SH.contains("export_packet.json"),
        "run_pilot.sh --dispatch-packet must reference export_packet.json"
    );
}

#[test]
fn dispatch_packet_artifact_shows_not_yet_generated() {
    assert!(
        RUN_PILOT_SH.contains("not yet generated"),
        "run_pilot.sh --dispatch-packet must print 'not yet generated' when dispatch artifact is absent"
    );
}

// ── why it matters section ────────────────────────────────────────────────────

#[test]
fn dispatch_packet_has_why_it_matters_section() {
    assert!(
        RUN_PILOT_SH.contains("WHY IT MATTERS"),
        "run_pilot.sh --dispatch-packet must print 'WHY IT MATTERS' section"
    );
}

#[test]
fn dispatch_packet_why_mentions_verified_workflow_state() {
    let block_start = RUN_PILOT_SH
        .find("WHY IT MATTERS")
        .expect("WHY IT MATTERS section must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 300];
    assert!(
        block.contains("verified workflow state"),
        "WHY IT MATTERS must mention 'verified workflow state'"
    );
}

#[test]
fn dispatch_packet_why_mentions_audit_ready() {
    let block_start = RUN_PILOT_SH
        .find("WHY IT MATTERS")
        .expect("WHY IT MATTERS section must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 300];
    assert!(
        block.contains("audit-ready handoff artifact"),
        "WHY IT MATTERS must mention 'audit-ready handoff artifact'"
    );
}

#[test]
fn dispatch_packet_why_mentions_execution_checkpoint() {
    let block_start = RUN_PILOT_SH
        .find("WHY IT MATTERS")
        .expect("WHY IT MATTERS section must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 300];
    assert!(
        block.contains("execution checkpoint"),
        "WHY IT MATTERS must mention 'execution checkpoint'"
    );
}

// ── how to use section ────────────────────────────────────────────────────────

#[test]
fn dispatch_packet_has_how_to_use_section() {
    assert!(
        RUN_PILOT_SH.contains("HOW TO USE"),
        "run_pilot.sh --dispatch-packet must print 'HOW TO USE' section"
    );
}

#[test]
fn dispatch_packet_how_to_use_shows_export_dispatch() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD DISPATCH PACKET")
        .expect("header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 1000];
    assert!(
        block.contains("--export-dispatch"),
        "HOW TO USE must reference --export-dispatch"
    );
}

#[test]
fn dispatch_packet_how_to_use_shows_run_summary() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD DISPATCH PACKET")
        .expect("header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 1000];
    assert!(
        block.contains("--run-summary"),
        "HOW TO USE must reference --run-summary"
    );
}

#[test]
fn dispatch_packet_how_to_use_shows_trace_view() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD DISPATCH PACKET")
        .expect("header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 1000];
    assert!(
        block.contains("--trace-view"),
        "HOW TO USE must reference --trace-view"
    );
}

#[test]
fn dispatch_packet_how_to_use_shows_artifact_index() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD DISPATCH PACKET")
        .expect("header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 1000];
    assert!(
        block.contains("--artifact-index"),
        "HOW TO USE must reference --artifact-index"
    );
}

// ── engineer interpretation section ──────────────────────────────────────────

#[test]
fn dispatch_packet_has_engineer_interpretation_section() {
    let after_header = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD DISPATCH PACKET")
        .expect("header must exist")..];
    assert!(
        after_header.contains("ENGINEER INTERPRETATION"),
        "run_pilot.sh --dispatch-packet must print 'ENGINEER INTERPRETATION' section"
    );
}

#[test]
fn dispatch_packet_engineer_interpretation_deterministic() {
    let after_header = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD DISPATCH PACKET")
        .expect("header must exist")..];
    let after_eng = &after_header[after_header
        .find("ENGINEER INTERPRETATION")
        .expect("ENGINEER INTERPRETATION must exist")..];
    assert!(
        after_eng.contains("deterministic"),
        "ENGINEER INTERPRETATION must include 'deterministic'"
    );
}

#[test]
fn dispatch_packet_engineer_interpretation_exportable() {
    let after_header = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD DISPATCH PACKET")
        .expect("header must exist")..];
    let after_eng = &after_header[after_header
        .find("ENGINEER INTERPRETATION")
        .expect("ENGINEER INTERPRETATION must exist")..];
    assert!(
        after_eng.contains("exportable"),
        "ENGINEER INTERPRETATION must include 'exportable'"
    );
}

#[test]
fn dispatch_packet_engineer_interpretation_audit_ready() {
    let after_header = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD DISPATCH PACKET")
        .expect("header must exist")..];
    let after_eng = &after_header[after_header
        .find("ENGINEER INTERPRETATION")
        .expect("ENGINEER INTERPRETATION must exist")..];
    assert!(
        after_eng.contains("audit-ready"),
        "ENGINEER INTERPRETATION must include 'audit-ready'"
    );
}

// ── absent-artifact message ───────────────────────────────────────────────────

#[test]
fn dispatch_packet_absent_suggests_export_dispatch() {
    assert!(
        RUN_PILOT_SH.contains("Export dispatch packet after verification to generate the artifact."),
        "run_pilot.sh must print export guidance when dispatch artifact is absent"
    );
}

// ── determinism: no $(date) in block ─────────────────────────────────────────

#[test]
fn dispatch_packet_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD DISPATCH PACKET")
        .expect("POSTCAD DISPATCH PACKET header must exist");
    let block_end = RUN_PILOT_SH[block_start..]
        .find("exit 0")
        .map(|i| block_start + i + 6)
        .unwrap_or(block_start + 3000);
    let block = &RUN_PILOT_SH[block_start..block_end];
    assert!(
        !block.contains("$(date"),
        "dispatch-packet block must not embed timestamps"
    );
}

// ── README ────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_dispatch_packet_section() {
    assert!(
        README.contains("## Dispatch Packet"),
        "README must have '## Dispatch Packet' section"
    );
}

#[test]
fn readme_dispatch_packet_shows_command() {
    assert!(
        README.contains("--dispatch-packet"),
        "README must show --dispatch-packet command"
    );
}
