//! Engineer entrypoint tests.
//!
//! Checks that run_pilot.sh --engineer-entrypoint exists, prints the required
//! sections and command references, handles run context deterministically,
//! and that README has the required section.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_engineer_entrypoint_flag() {
    assert!(
        RUN_PILOT_SH.contains("--engineer-entrypoint"),
        "run_pilot.sh must support --engineer-entrypoint flag"
    );
}

#[test]
fn engineer_entrypoint_exits_0() {
    assert!(
        RUN_PILOT_SH.contains("--engineer-entrypoint") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --engineer-entrypoint block must exit 0"
    );
}

// ── header ────────────────────────────────────────────────────────────────────

#[test]
fn engineer_entrypoint_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD ENGINEER ENTRYPOINT"),
        "run_pilot.sh must print 'POSTCAD ENGINEER ENTRYPOINT' header"
    );
}

// ── what to look at first section ────────────────────────────────────────────

#[test]
fn engineer_entrypoint_has_what_to_look_at_first_section() {
    assert!(
        RUN_PILOT_SH.contains("WHAT TO LOOK AT FIRST"),
        "run_pilot.sh --engineer-entrypoint must print 'WHAT TO LOOK AT FIRST' section"
    );
}

#[test]
fn engineer_entrypoint_look_first_mentions_system_overview() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHAT TO LOOK AT FIRST")
        .expect("section must exist")..];
    assert!(
        after.contains("system overview"),
        "WHAT TO LOOK AT FIRST must mention 'system overview'"
    );
}

#[test]
fn engineer_entrypoint_look_first_mentions_trace_view() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHAT TO LOOK AT FIRST")
        .expect("section must exist")..];
    assert!(
        after.contains("trace view"),
        "WHAT TO LOOK AT FIRST must mention 'trace view'"
    );
}

#[test]
fn engineer_entrypoint_look_first_mentions_receipt_replay() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHAT TO LOOK AT FIRST")
        .expect("section must exist")..];
    assert!(
        after.contains("receipt replay"),
        "WHAT TO LOOK AT FIRST must mention 'receipt replay'"
    );
}

#[test]
fn engineer_entrypoint_look_first_mentions_dispatch_packet() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHAT TO LOOK AT FIRST")
        .expect("section must exist")..];
    assert!(
        after.contains("dispatch packet"),
        "WHAT TO LOOK AT FIRST must mention 'dispatch packet'"
    );
}

#[test]
fn engineer_entrypoint_look_first_mentions_protocol_chain() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHAT TO LOOK AT FIRST")
        .expect("section must exist")..];
    assert!(
        after.contains("protocol chain"),
        "WHAT TO LOOK AT FIRST must mention 'protocol chain'"
    );
}

// ── recommended order section ─────────────────────────────────────────────────

#[test]
fn engineer_entrypoint_has_recommended_order_section() {
    assert!(
        RUN_PILOT_SH.contains("RECOMMENDED ORDER"),
        "run_pilot.sh --engineer-entrypoint must print 'RECOMMENDED ORDER' section"
    );
}

#[test]
fn engineer_entrypoint_recommended_order_shows_system_overview() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("RECOMMENDED ORDER")
        .expect("section must exist")..];
    assert!(
        after.contains("--system-overview"),
        "RECOMMENDED ORDER must include --system-overview command"
    );
}

#[test]
fn engineer_entrypoint_recommended_order_shows_trace_view() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("RECOMMENDED ORDER")
        .expect("section must exist")..];
    assert!(
        after.contains("--trace-view"),
        "RECOMMENDED ORDER must include --trace-view command"
    );
}

#[test]
fn engineer_entrypoint_recommended_order_shows_receipt_replay() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("RECOMMENDED ORDER")
        .expect("section must exist")..];
    assert!(
        after.contains("--receipt-replay"),
        "RECOMMENDED ORDER must include --receipt-replay command"
    );
}

#[test]
fn engineer_entrypoint_recommended_order_shows_dispatch_packet() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("RECOMMENDED ORDER")
        .expect("section must exist")..];
    assert!(
        after.contains("--dispatch-packet"),
        "RECOMMENDED ORDER must include --dispatch-packet command"
    );
}

#[test]
fn engineer_entrypoint_recommended_order_shows_protocol_chain() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("RECOMMENDED ORDER")
        .expect("section must exist")..];
    assert!(
        after.contains("--protocol-chain"),
        "RECOMMENDED ORDER must include --protocol-chain command"
    );
}

// ── what each command shows section ──────────────────────────────────────────

#[test]
fn engineer_entrypoint_has_what_each_command_shows_section() {
    assert!(
        RUN_PILOT_SH.contains("WHAT EACH COMMAND SHOWS"),
        "run_pilot.sh --engineer-entrypoint must print 'WHAT EACH COMMAND SHOWS' section"
    );
}

// ── current run context section ───────────────────────────────────────────────

#[test]
fn engineer_entrypoint_has_current_run_context_section() {
    assert!(
        RUN_PILOT_SH.contains("CURRENT RUN CONTEXT"),
        "run_pilot.sh --engineer-entrypoint must print 'CURRENT RUN CONTEXT' section"
    );
}

#[test]
fn engineer_entrypoint_run_context_shows_run_id_label() {
    assert!(
        RUN_PILOT_SH.contains("EE_RUN_ID"),
        "run_pilot.sh --engineer-entrypoint must use EE_RUN_ID variable for run context"
    );
}

#[test]
fn engineer_entrypoint_run_context_not_detected_fallback() {
    assert!(
        RUN_PILOT_SH.contains("not detected"),
        "run_pilot.sh --engineer-entrypoint must show 'not detected' when run ID cannot be resolved"
    );
}

// ── why technically interesting section ──────────────────────────────────────

#[test]
fn engineer_entrypoint_has_why_technically_interesting_section() {
    assert!(
        RUN_PILOT_SH.contains("WHY THIS IS TECHNICALLY INTERESTING"),
        "run_pilot.sh --engineer-entrypoint must print 'WHY THIS IS TECHNICALLY INTERESTING' section"
    );
}

#[test]
fn engineer_entrypoint_why_mentions_deterministic_routing() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHY THIS IS TECHNICALLY INTERESTING")
        .expect("section must exist")..];
    assert!(
        after.contains("deterministic routing artifacts"),
        "WHY section must mention 'deterministic routing artifacts'"
    );
}

#[test]
fn engineer_entrypoint_why_mentions_replayable_receipt() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHY THIS IS TECHNICALLY INTERESTING")
        .expect("section must exist")..];
    assert!(
        after.contains("replayable receipt"),
        "WHY section must mention 'replayable receipt'"
    );
}

#[test]
fn engineer_entrypoint_why_mentions_verifiable_chain() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHY THIS IS TECHNICALLY INTERESTING")
        .expect("section must exist")..];
    assert!(
        after.contains("verifiable workflow chain"),
        "WHY section must mention 'verifiable workflow chain'"
    );
}

#[test]
fn engineer_entrypoint_why_mentions_audit_ready_handoff() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("WHY THIS IS TECHNICALLY INTERESTING")
        .expect("section must exist")..];
    assert!(
        after.contains("audit-ready execution handoff"),
        "WHY section must mention 'audit-ready execution handoff'"
    );
}

// ── determinism: no $(date) in block ─────────────────────────────────────────

#[test]
fn engineer_entrypoint_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD ENGINEER ENTRYPOINT")
        .expect("header must exist");
    let after = &RUN_PILOT_SH[block_start..];
    let block_end = after.find("exit 0").map(|i| i + 6).unwrap_or(3000);
    let block = &after[..block_end];
    assert!(
        !block.contains("$(date"),
        "engineer-entrypoint block must not embed timestamps"
    );
}

// ── README ────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_engineer_entrypoint_section() {
    assert!(
        README.contains("## Engineer Entrypoint"),
        "README must have '## Engineer Entrypoint' section"
    );
}

#[test]
fn readme_engineer_entrypoint_shows_command() {
    assert!(
        README.contains("--engineer-entrypoint"),
        "README must show --engineer-entrypoint command"
    );
}
