//! Lab entrypoint surface tests.
//!
//! Checks that run_pilot.sh --lab-entrypoint exists, prints the required
//! sections and command references, handles run context deterministically,
//! and that README has the required section.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_lab_entrypoint_flag() {
    assert!(
        RUN_PILOT_SH.contains("--lab-entrypoint"),
        "run_pilot.sh must support --lab-entrypoint flag"
    );
}

#[test]
fn lab_entrypoint_exits_0() {
    assert!(
        RUN_PILOT_SH.contains("--lab-entrypoint") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --lab-entrypoint block must exit 0"
    );
}

// ── header ────────────────────────────────────────────────────────────────────

#[test]
fn lab_entrypoint_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD LAB ENTRYPOINT"),
        "run_pilot.sh must print 'POSTCAD LAB ENTRYPOINT' header"
    );
}

// ── what the lab receives section ────────────────────────────────────────────

#[test]
fn lab_entrypoint_has_what_lab_receives_section() {
    assert!(
        RUN_PILOT_SH.contains("WHAT THE LAB RECEIVES"),
        "run_pilot.sh --lab-entrypoint must print 'WHAT THE LAB RECEIVES' section"
    );
}

#[test]
fn lab_entrypoint_receives_mentions_routed_case() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHAT THE LAB RECEIVES")
        .expect("section must exist")..];
    assert!(
        after.contains("routed case context"),
        "WHAT THE LAB RECEIVES must mention 'routed case context'"
    );
}

#[test]
fn lab_entrypoint_receives_mentions_routing_receipt() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHAT THE LAB RECEIVES")
        .expect("section must exist")..];
    assert!(
        after.contains("routing receipt"),
        "WHAT THE LAB RECEIVES must mention 'routing receipt'"
    );
}

#[test]
fn lab_entrypoint_receives_mentions_inbound_reply_expectation() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHAT THE LAB RECEIVES")
        .expect("section must exist")..];
    assert!(
        after.contains("inbound reply expectation"),
        "WHAT THE LAB RECEIVES must mention 'inbound reply expectation'"
    );
}

#[test]
fn lab_entrypoint_receives_mentions_dispatch_handoff() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHAT THE LAB RECEIVES")
        .expect("section must exist")..];
    assert!(
        after.contains("dispatch-ready handoff artifact"),
        "WHAT THE LAB RECEIVES must mention 'dispatch-ready handoff artifact'"
    );
}

// ── what the lab is expected to do section ───────────────────────────────────

#[test]
fn lab_entrypoint_has_what_lab_expected_to_do_section() {
    assert!(
        RUN_PILOT_SH.contains("WHAT THE LAB IS EXPECTED TO DO"),
        "run_pilot.sh --lab-entrypoint must print 'WHAT THE LAB IS EXPECTED TO DO' section"
    );
}

#[test]
fn lab_entrypoint_expected_mentions_review_case() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHAT THE LAB IS EXPECTED TO DO")
        .expect("section must exist")..];
    assert!(
        after.contains("review the routed case"),
        "WHAT THE LAB IS EXPECTED TO DO must mention 'review the routed case'"
    );
}

#[test]
fn lab_entrypoint_expected_mentions_structured_reply() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHAT THE LAB IS EXPECTED TO DO")
        .expect("section must exist")..];
    assert!(
        after.contains("return a structured inbound reply"),
        "WHAT THE LAB IS EXPECTED TO DO must mention 'return a structured inbound reply'"
    );
}

#[test]
fn lab_entrypoint_expected_mentions_verifiable_workflow() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHAT THE LAB IS EXPECTED TO DO")
        .expect("section must exist")..];
    assert!(
        after.contains("participate in a verifiable workflow"),
        "WHAT THE LAB IS EXPECTED TO DO must mention 'participate in a verifiable workflow'"
    );
}

// ── why this matters to a lab section ────────────────────────────────────────

#[test]
fn lab_entrypoint_has_why_matters_to_lab_section() {
    assert!(
        RUN_PILOT_SH.contains("WHY THIS MATTERS TO A LAB"),
        "run_pilot.sh --lab-entrypoint must print 'WHY THIS MATTERS TO A LAB' section"
    );
}

#[test]
fn lab_entrypoint_why_mentions_clearer_handoff() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHY THIS MATTERS TO A LAB")
        .expect("section must exist")..];
    assert!(
        after.contains("clearer case handoff"),
        "WHY THIS MATTERS TO A LAB must mention 'clearer case handoff'"
    );
}

#[test]
fn lab_entrypoint_why_mentions_fewer_ambiguous_states() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHY THIS MATTERS TO A LAB")
        .expect("section must exist")..];
    assert!(
        after.contains("fewer ambiguous workflow states"),
        "WHY THIS MATTERS TO A LAB must mention 'fewer ambiguous workflow states'"
    );
}

#[test]
fn lab_entrypoint_why_mentions_verifiable_reply_path() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHY THIS MATTERS TO A LAB")
        .expect("section must exist")..];
    assert!(
        after.contains("verifiable reply path"),
        "WHY THIS MATTERS TO A LAB must mention 'verifiable reply path'"
    );
}

// ── what to look at first section ────────────────────────────────────────────

#[test]
fn lab_entrypoint_has_what_to_look_at_first_section() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD LAB ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("WHAT TO LOOK AT FIRST"),
        "run_pilot.sh --lab-entrypoint must print 'WHAT TO LOOK AT FIRST' section"
    );
}

#[test]
fn lab_entrypoint_look_first_shows_demo_surface() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD LAB ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("--demo-surface"),
        "WHAT TO LOOK AT FIRST must include --demo-surface"
    );
}

#[test]
fn lab_entrypoint_look_first_shows_artifact_index() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD LAB ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("--artifact-index"),
        "WHAT TO LOOK AT FIRST must include --artifact-index"
    );
}

#[test]
fn lab_entrypoint_look_first_shows_trace_view() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD LAB ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("--trace-view"),
        "WHAT TO LOOK AT FIRST must include --trace-view"
    );
}

#[test]
fn lab_entrypoint_look_first_shows_dispatch_packet() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD LAB ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("--dispatch-packet"),
        "WHAT TO LOOK AT FIRST must include --dispatch-packet"
    );
}

// ── what each command shows section ──────────────────────────────────────────

#[test]
fn lab_entrypoint_has_what_each_command_shows_section() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD LAB ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("WHAT EACH COMMAND SHOWS"),
        "run_pilot.sh --lab-entrypoint must print 'WHAT EACH COMMAND SHOWS' section"
    );
}

// ── current run context section ───────────────────────────────────────────────

#[test]
fn lab_entrypoint_has_current_run_context_section() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD LAB ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("CURRENT RUN CONTEXT"),
        "run_pilot.sh --lab-entrypoint must print 'CURRENT RUN CONTEXT' section"
    );
}

#[test]
fn lab_entrypoint_run_context_uses_le_run_id() {
    assert!(
        RUN_PILOT_SH.contains("LE_RUN_ID"),
        "run_pilot.sh --lab-entrypoint must use LE_RUN_ID variable"
    );
}

#[test]
fn lab_entrypoint_run_context_not_detected_fallback() {
    assert!(
        RUN_PILOT_SH.contains("not detected"),
        "run_pilot.sh --lab-entrypoint must show 'not detected' fallback"
    );
}

// ── lab interpretation section ────────────────────────────────────────────────

#[test]
fn lab_entrypoint_has_lab_interpretation_section() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD LAB ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("LAB INTERPRETATION"),
        "run_pilot.sh --lab-entrypoint must print 'LAB INTERPRETATION' section"
    );
}

#[test]
fn lab_entrypoint_interpretation_structured_handoff() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("LAB INTERPRETATION")
        .expect("section must exist")..];
    assert!(
        after.contains("structured handoff"),
        "LAB INTERPRETATION must include 'structured handoff'"
    );
}

#[test]
fn lab_entrypoint_interpretation_verifiable_reply() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("LAB INTERPRETATION")
        .expect("section must exist")..];
    assert!(
        after.contains("verifiable reply"),
        "LAB INTERPRETATION must include 'verifiable reply'"
    );
}

#[test]
fn lab_entrypoint_interpretation_execution_ready_dispatch() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("LAB INTERPRETATION")
        .expect("section must exist")..];
    assert!(
        after.contains("execution-ready dispatch"),
        "LAB INTERPRETATION must include 'execution-ready dispatch'"
    );
}

// ── determinism: no $(date) in block ─────────────────────────────────────────

#[test]
fn lab_entrypoint_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD LAB ENTRYPOINT")
        .expect("header must exist");
    let after = &RUN_PILOT_SH[block_start..];
    let block_end = after.find("exit 0").map(|i| i + 6).unwrap_or(3000);
    let block = &after[..block_end];
    assert!(
        !block.contains("$(date"),
        "lab-entrypoint block must not embed timestamps"
    );
}

// ── README ────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_lab_entrypoint_section() {
    assert!(
        README.contains("## Lab Entrypoint"),
        "README must have '## Lab Entrypoint' section"
    );
}

#[test]
fn readme_lab_entrypoint_shows_command() {
    assert!(
        README.contains("--lab-entrypoint"),
        "README must show --lab-entrypoint command"
    );
}
