//! Business entrypoint surface tests.
//!
//! Checks that run_pilot.sh --business-entrypoint exists, prints the required
//! sections and command references, handles run context deterministically,
//! and that README has the required section.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_business_entrypoint_flag() {
    assert!(
        RUN_PILOT_SH.contains("--business-entrypoint"),
        "run_pilot.sh must support --business-entrypoint flag"
    );
}

#[test]
fn business_entrypoint_exits_0() {
    assert!(
        RUN_PILOT_SH.contains("--business-entrypoint") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --business-entrypoint block must exit 0"
    );
}

// ── header ────────────────────────────────────────────────────────────────────

#[test]
fn business_entrypoint_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD BUSINESS ENTRYPOINT"),
        "run_pilot.sh must print 'POSTCAD BUSINESS ENTRYPOINT' header"
    );
}

// ── what this pilot does section ──────────────────────────────────────────────

#[test]
fn business_entrypoint_has_what_this_pilot_does_section() {
    assert!(
        RUN_PILOT_SH.contains("WHAT THIS PILOT DOES"),
        "run_pilot.sh --business-entrypoint must print 'WHAT THIS PILOT DOES' section"
    );
}

#[test]
fn business_entrypoint_pilot_does_mentions_routing() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHAT THIS PILOT DOES")
        .expect("section must exist")..];
    assert!(
        after.contains("routes a dental manufacturing case deterministically"),
        "WHAT THIS PILOT DOES must mention deterministic routing"
    );
}

#[test]
fn business_entrypoint_pilot_does_mentions_receipt() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHAT THIS PILOT DOES")
        .expect("section must exist")..];
    assert!(
        after.contains("records a receipt for the routing decision"),
        "WHAT THIS PILOT DOES must mention receipt recording"
    );
}

#[test]
fn business_entrypoint_pilot_does_mentions_lab_reply() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHAT THIS PILOT DOES")
        .expect("section must exist")..];
    assert!(
        after.contains("accepts and verifies a lab reply"),
        "WHAT THIS PILOT DOES must mention accepting and verifying a lab reply"
    );
}

#[test]
fn business_entrypoint_pilot_does_mentions_dispatch_packet() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHAT THIS PILOT DOES")
        .expect("section must exist")..];
    assert!(
        after.contains("exports a dispatch packet for execution handoff"),
        "WHAT THIS PILOT DOES must mention dispatch packet export"
    );
}

// ── why it matters section ────────────────────────────────────────────────────

#[test]
fn business_entrypoint_has_why_it_matters_section() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD BUSINESS ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("WHY IT MATTERS"),
        "run_pilot.sh --business-entrypoint must print 'WHY IT MATTERS' section"
    );
}

#[test]
fn business_entrypoint_why_mentions_workflow_ambiguities() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD BUSINESS ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("fewer workflow ambiguities"),
        "WHY IT MATTERS must mention 'fewer workflow ambiguities'"
    );
}

#[test]
fn business_entrypoint_why_mentions_verifiable_handoff() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD BUSINESS ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("verifiable handoff between clinic and lab"),
        "WHY IT MATTERS must mention 'verifiable handoff between clinic and lab'"
    );
}

#[test]
fn business_entrypoint_why_mentions_audit_ready() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD BUSINESS ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("audit-ready process"),
        "WHY IT MATTERS must mention 'audit-ready process'"
    );
}

#[test]
fn business_entrypoint_why_mentions_operational_accountability() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD BUSINESS ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("clearer operational accountability"),
        "WHY IT MATTERS must mention 'clearer operational accountability'"
    );
}

// ── what to look at first section ────────────────────────────────────────────

#[test]
fn business_entrypoint_has_what_to_look_at_first_section() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD BUSINESS ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("WHAT TO LOOK AT FIRST"),
        "run_pilot.sh --business-entrypoint must print 'WHAT TO LOOK AT FIRST' section"
    );
}

#[test]
fn business_entrypoint_look_first_shows_demo_surface() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD BUSINESS ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("--demo-surface"),
        "WHAT TO LOOK AT FIRST must include --demo-surface"
    );
}

#[test]
fn business_entrypoint_look_first_shows_system_overview() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD BUSINESS ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("--system-overview"),
        "WHAT TO LOOK AT FIRST must include --system-overview"
    );
}

#[test]
fn business_entrypoint_look_first_shows_run_summary() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD BUSINESS ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("--run-summary"),
        "WHAT TO LOOK AT FIRST must include --run-summary"
    );
}

#[test]
fn business_entrypoint_look_first_shows_help_surface() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD BUSINESS ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("--help-surface"),
        "WHAT TO LOOK AT FIRST must include --help-surface"
    );
}

// ── what each command shows section ──────────────────────────────────────────

#[test]
fn business_entrypoint_has_what_each_command_shows_section() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD BUSINESS ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("WHAT EACH COMMAND SHOWS"),
        "run_pilot.sh --business-entrypoint must print 'WHAT EACH COMMAND SHOWS' section"
    );
}

// ── current run context section ───────────────────────────────────────────────

#[test]
fn business_entrypoint_has_current_run_context_section() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD BUSINESS ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("CURRENT RUN CONTEXT"),
        "run_pilot.sh --business-entrypoint must print 'CURRENT RUN CONTEXT' section"
    );
}

#[test]
fn business_entrypoint_run_context_uses_be_run_id() {
    assert!(
        RUN_PILOT_SH.contains("BE_RUN_ID"),
        "run_pilot.sh --business-entrypoint must use BE_RUN_ID variable"
    );
}

#[test]
fn business_entrypoint_run_context_not_detected_fallback() {
    assert!(
        RUN_PILOT_SH.contains("not detected"),
        "run_pilot.sh --business-entrypoint must show 'not detected' fallback"
    );
}

// ── why this is strategic section ────────────────────────────────────────────

#[test]
fn business_entrypoint_has_why_strategic_section() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD BUSINESS ENTRYPOINT")
        .expect("header must exist")..];
    assert!(
        after.contains("WHY THIS IS STRATEGIC"),
        "run_pilot.sh --business-entrypoint must print 'WHY THIS IS STRATEGIC' section"
    );
}

#[test]
fn business_entrypoint_strategic_mentions_workflow_infrastructure() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHY THIS IS STRATEGIC")
        .expect("section must exist")..];
    assert!(
        after.contains("workflow infrastructure, not just software"),
        "WHY THIS IS STRATEGIC must mention 'workflow infrastructure, not just software'"
    );
}

#[test]
fn business_entrypoint_strategic_mentions_traceable_handoff() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHY THIS IS STRATEGIC")
        .expect("section must exist")..];
    assert!(
        after.contains("traceable handoff layer"),
        "WHY THIS IS STRATEGIC must mention 'traceable handoff layer'"
    );
}

#[test]
fn business_entrypoint_strategic_mentions_trusted_routing() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHY THIS IS STRATEGIC")
        .expect("section must exist")..];
    assert!(
        after.contains("foundation for trusted routing between clinics and manufacturers"),
        "WHY THIS IS STRATEGIC must mention trusted routing foundation"
    );
}

// ── determinism: no $(date) in block ─────────────────────────────────────────

#[test]
fn business_entrypoint_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD BUSINESS ENTRYPOINT")
        .expect("header must exist");
    let after = &RUN_PILOT_SH[block_start..];
    let block_end = after.find("exit 0").map(|i| i + 6).unwrap_or(3000);
    let block = &after[..block_end];
    assert!(
        !block.contains("$(date"),
        "business-entrypoint block must not embed timestamps"
    );
}

// ── README ────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_business_entrypoint_section() {
    assert!(
        README.contains("## Business Entrypoint"),
        "README must have '## Business Entrypoint' section"
    );
}

#[test]
fn readme_business_entrypoint_shows_command() {
    assert!(
        README.contains("--business-entrypoint"),
        "README must show --business-entrypoint command"
    );
}
