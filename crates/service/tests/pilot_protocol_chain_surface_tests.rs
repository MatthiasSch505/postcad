//! Protocol chain surface tests.
//!
//! Checks that run_pilot.sh --protocol-chain exists, prints the required
//! sections and chain stages, handles missing artifacts deterministically,
//! and that README has the required section.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_protocol_chain_flag() {
    assert!(
        RUN_PILOT_SH.contains("--protocol-chain"),
        "run_pilot.sh must support --protocol-chain flag"
    );
}

#[test]
fn protocol_chain_exits_0() {
    assert!(
        RUN_PILOT_SH.contains("--protocol-chain") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --protocol-chain block must exit 0"
    );
}

// ── header ────────────────────────────────────────────────────────────────────

#[test]
fn protocol_chain_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD PROTOCOL CHAIN"),
        "run_pilot.sh must print 'POSTCAD PROTOCOL CHAIN' header"
    );
}

// ── run context section ───────────────────────────────────────────────────────

#[test]
fn protocol_chain_has_run_context_section() {
    assert!(
        RUN_PILOT_SH.contains("RUN CONTEXT"),
        "run_pilot.sh --protocol-chain must print 'RUN CONTEXT' section"
    );
}

#[test]
fn protocol_chain_prints_run_id_label() {
    assert!(
        RUN_PILOT_SH.contains("Run ID :"),
        "run_pilot.sh --protocol-chain must print 'Run ID :' label"
    );
}

// ── chain section ─────────────────────────────────────────────────────────────

#[test]
fn protocol_chain_has_chain_section() {
    assert!(
        RUN_PILOT_SH.contains("CHAIN"),
        "run_pilot.sh --protocol-chain must print 'CHAIN' section"
    );
}

#[test]
fn protocol_chain_has_stage_1_receipt() {
    assert!(
        RUN_PILOT_SH.contains("1  receipt"),
        "run_pilot.sh --protocol-chain must include stage '1  receipt'"
    );
}

#[test]
fn protocol_chain_has_stage_2_inbound_reply() {
    assert!(
        RUN_PILOT_SH.contains("2  inbound reply"),
        "run_pilot.sh --protocol-chain must include stage '2  inbound reply'"
    );
}

#[test]
fn protocol_chain_has_stage_3_verification() {
    assert!(
        RUN_PILOT_SH.contains("3  verification"),
        "run_pilot.sh --protocol-chain must include stage '3  verification'"
    );
}

#[test]
fn protocol_chain_has_stage_4_dispatch_packet() {
    assert!(
        RUN_PILOT_SH.contains("4  dispatch packet"),
        "run_pilot.sh --protocol-chain must include stage '4  dispatch packet'"
    );
}

#[test]
fn protocol_chain_stage_1_describes_routing_commitment() {
    assert!(
        RUN_PILOT_SH.contains("routing commitment"),
        "run_pilot.sh --protocol-chain stage 1 must describe 'routing commitment'"
    );
}

#[test]
fn protocol_chain_stage_1_describes_source_of_truth() {
    assert!(
        RUN_PILOT_SH.contains("source of truth for the run"),
        "run_pilot.sh --protocol-chain stage 1 must describe 'source of truth for the run'"
    );
}

#[test]
fn protocol_chain_stage_4_describes_execution_handoff() {
    assert!(
        RUN_PILOT_SH.contains("execution-side handoff artifact after verified workflow state"),
        "run_pilot.sh --protocol-chain stage 4 must describe execution-side handoff"
    );
}

// ── current state section ─────────────────────────────────────────────────────

#[test]
fn protocol_chain_has_current_state_section() {
    assert!(
        RUN_PILOT_SH.contains("CURRENT STATE"),
        "run_pilot.sh --protocol-chain must print 'CURRENT STATE' section"
    );
}

#[test]
fn protocol_chain_current_state_uses_detected_label() {
    assert!(
        RUN_PILOT_SH.contains("PC_RECEIPT") && RUN_PILOT_SH.contains("detected"),
        "run_pilot.sh --protocol-chain must use 'detected' label in CURRENT STATE"
    );
}

#[test]
fn protocol_chain_current_state_uses_not_yet_observed_label() {
    assert!(
        RUN_PILOT_SH.contains("not yet observed"),
        "run_pilot.sh --protocol-chain must use 'not yet observed' label in CURRENT STATE"
    );
}

#[test]
fn protocol_chain_detects_receipt_artifact() {
    assert!(
        RUN_PILOT_SH.contains("PC_RECEIPT"),
        "run_pilot.sh --protocol-chain must use PC_RECEIPT variable for receipt detection"
    );
}

#[test]
fn protocol_chain_detects_inbound_artifact() {
    assert!(
        RUN_PILOT_SH.contains("PC_INBOUND"),
        "run_pilot.sh --protocol-chain must use PC_INBOUND variable for inbound detection"
    );
}

#[test]
fn protocol_chain_detects_verification_artifact() {
    assert!(
        RUN_PILOT_SH.contains("PC_VERIFICATION"),
        "run_pilot.sh --protocol-chain must use PC_VERIFICATION variable for verification detection"
    );
}

#[test]
fn protocol_chain_detects_dispatch_artifact() {
    assert!(
        RUN_PILOT_SH.contains("PC_DISPATCH"),
        "run_pilot.sh --protocol-chain must use PC_DISPATCH variable for dispatch detection"
    );
}

// ── why this matters section ──────────────────────────────────────────────────

#[test]
fn protocol_chain_has_why_this_matters_section() {
    assert!(
        RUN_PILOT_SH.contains("WHY THIS MATTERS"),
        "run_pilot.sh --protocol-chain must print 'WHY THIS MATTERS' section"
    );
}

#[test]
fn protocol_chain_why_mentions_deterministic_chain() {
    assert!(
        RUN_PILOT_SH.contains("deterministic chain of workflow artifacts"),
        "WHY THIS MATTERS must mention 'deterministic chain of workflow artifacts'"
    );
}

#[test]
fn protocol_chain_why_mentions_verifiable_transition() {
    assert!(
        RUN_PILOT_SH.contains("verifiable transition from routing to execution"),
        "WHY THIS MATTERS must mention 'verifiable transition from routing to execution'"
    );
}

#[test]
fn protocol_chain_why_mentions_audit_ready_path() {
    assert!(
        RUN_PILOT_SH.contains("audit-ready protocol path"),
        "WHY THIS MATTERS must mention 'audit-ready protocol path'"
    );
}

// ── how to use section ────────────────────────────────────────────────────────

#[test]
fn protocol_chain_has_how_to_use_section() {
    assert!(
        RUN_PILOT_SH.contains("HOW TO USE"),
        "run_pilot.sh --protocol-chain must print 'HOW TO USE' section"
    );
}

#[test]
fn protocol_chain_how_to_use_shows_receipt_replay() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD PROTOCOL CHAIN")
        .expect("header must exist")..];
    assert!(
        after.contains("--receipt-replay"),
        "HOW TO USE must reference --receipt-replay"
    );
}

#[test]
fn protocol_chain_how_to_use_shows_dispatch_packet() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD PROTOCOL CHAIN")
        .expect("header must exist")..];
    assert!(
        after.contains("--dispatch-packet"),
        "HOW TO USE must reference --dispatch-packet"
    );
}

#[test]
fn protocol_chain_how_to_use_shows_trace_view() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD PROTOCOL CHAIN")
        .expect("header must exist")..];
    assert!(
        after.contains("--trace-view"),
        "HOW TO USE must reference --trace-view"
    );
}

#[test]
fn protocol_chain_how_to_use_shows_run_summary() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD PROTOCOL CHAIN")
        .expect("header must exist")..];
    assert!(
        after.contains("--run-summary"),
        "HOW TO USE must reference --run-summary"
    );
}

// ── engineer interpretation section ──────────────────────────────────────────

#[test]
fn protocol_chain_has_engineer_interpretation_section() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD PROTOCOL CHAIN")
        .expect("header must exist")..];
    assert!(
        after.contains("ENGINEER INTERPRETATION"),
        "run_pilot.sh --protocol-chain must print 'ENGINEER INTERPRETATION' section"
    );
}

#[test]
fn protocol_chain_engineer_interpretation_chained() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD PROTOCOL CHAIN")
        .expect("header must exist")..];
    let after_eng = &after[after
        .find("ENGINEER INTERPRETATION")
        .expect("ENGINEER INTERPRETATION must exist")..];
    assert!(
        after_eng.contains("chained"),
        "ENGINEER INTERPRETATION must include 'chained'"
    );
}

// ── determinism: no $(date) in block ─────────────────────────────────────────

#[test]
fn protocol_chain_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PROTOCOL CHAIN")
        .expect("header must exist");
    let after = &RUN_PILOT_SH[block_start..];
    let block_end = after
        .find("exit 0")
        .map(|i| i + 6)
        .unwrap_or(3000);
    let block = &after[..block_end];
    assert!(
        !block.contains("$(date"),
        "protocol-chain block must not embed timestamps"
    );
}

// ── README ────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_protocol_chain_section() {
    assert!(
        README.contains("## Protocol Chain"),
        "README must have '## Protocol Chain' section"
    );
}

#[test]
fn readme_protocol_chain_shows_command() {
    assert!(
        README.contains("--protocol-chain"),
        "README must show --protocol-chain command"
    );
}
