//! Receipt replay surface tests.
//!
//! Checks that run_pilot.sh --receipt-replay exists, prints the required
//! sections and command references, handles missing receipt deterministically,
//! and that README has the required section.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_receipt_replay_flag() {
    assert!(
        RUN_PILOT_SH.contains("--receipt-replay"),
        "run_pilot.sh must support --receipt-replay flag"
    );
}

#[test]
fn receipt_replay_exits_0() {
    assert!(
        RUN_PILOT_SH.contains("--receipt-replay") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --receipt-replay block must exit 0"
    );
}

// ── header ────────────────────────────────────────────────────────────────────

#[test]
fn receipt_replay_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD RECEIPT REPLAY"),
        "run_pilot.sh must print 'POSTCAD RECEIPT REPLAY' header"
    );
}

// ── run context section ───────────────────────────────────────────────────────

#[test]
fn receipt_replay_has_run_context_section() {
    assert!(
        RUN_PILOT_SH.contains("RUN CONTEXT"),
        "run_pilot.sh --receipt-replay must print 'RUN CONTEXT' section"
    );
}

#[test]
fn receipt_replay_prints_run_id_label() {
    assert!(
        RUN_PILOT_SH.contains("Run ID  :"),
        "run_pilot.sh --receipt-replay must print 'Run ID  :' label"
    );
}

#[test]
fn receipt_replay_prints_receipt_label() {
    assert!(
        RUN_PILOT_SH.contains("Receipt :"),
        "run_pilot.sh --receipt-replay must print 'Receipt :' label"
    );
}

#[test]
fn receipt_replay_shows_not_found_fallback() {
    assert!(
        RUN_PILOT_SH.contains("(not found)"),
        "run_pilot.sh --receipt-replay must show '(not found)' when receipt is absent"
    );
}

// ── what the receipt commits section ─────────────────────────────────────────

#[test]
fn receipt_replay_has_what_commits_section() {
    assert!(
        RUN_PILOT_SH.contains("WHAT THE RECEIPT COMMITS"),
        "run_pilot.sh --receipt-replay must print 'WHAT THE RECEIPT COMMITS' section"
    );
}

#[test]
fn receipt_replay_commits_mentions_routing_candidate() {
    assert!(
        RUN_PILOT_SH.contains("selected routing candidate"),
        "run_pilot.sh --receipt-replay must mention 'selected routing candidate'"
    );
}

#[test]
fn receipt_replay_commits_mentions_deterministic_outcome() {
    assert!(
        RUN_PILOT_SH.contains("deterministic routing outcome"),
        "run_pilot.sh --receipt-replay must mention 'deterministic routing outcome'"
    );
}

#[test]
fn receipt_replay_commits_mentions_receipt_hash() {
    assert!(
        RUN_PILOT_SH.contains("receipt hash as the verification source of truth"),
        "run_pilot.sh --receipt-replay must mention receipt hash as source of truth"
    );
}

// ── replay idea section ───────────────────────────────────────────────────────

#[test]
fn receipt_replay_has_replay_idea_section() {
    assert!(
        RUN_PILOT_SH.contains("REPLAY IDEA"),
        "run_pilot.sh --receipt-replay must print 'REPLAY IDEA' section"
    );
}

#[test]
fn receipt_replay_idea_mentions_routing_commitment() {
    assert!(
        RUN_PILOT_SH.contains("routing commitment for the case"),
        "run_pilot.sh --receipt-replay must describe receipt as routing commitment for the case"
    );
}

// ── how to use section ────────────────────────────────────────────────────────

#[test]
fn receipt_replay_has_how_to_use_section() {
    assert!(
        RUN_PILOT_SH.contains("HOW TO USE"),
        "run_pilot.sh --receipt-replay must print 'HOW TO USE' section"
    );
}

#[test]
fn receipt_replay_how_to_use_shows_run_pilot() {
    let block_start = RUN_PILOT_SH
        .find("HOW TO USE")
        .expect("HOW TO USE section must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 300];
    assert!(
        block.contains("./examples/pilot/run_pilot.sh"),
        "HOW TO USE must reference ./examples/pilot/run_pilot.sh"
    );
}

#[test]
fn receipt_replay_how_to_use_shows_verify_sh() {
    let block_start = RUN_PILOT_SH
        .find("HOW TO USE")
        .expect("HOW TO USE section must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 300];
    assert!(
        block.contains("./examples/pilot/verify.sh"),
        "HOW TO USE must reference ./examples/pilot/verify.sh"
    );
}

#[test]
fn receipt_replay_how_to_use_shows_run_summary() {
    assert!(
        RUN_PILOT_SH.contains("--run-summary"),
        "run_pilot.sh --receipt-replay HOW TO USE must reference --run-summary"
    );
}

#[test]
fn receipt_replay_how_to_use_shows_trace_view() {
    assert!(
        RUN_PILOT_SH.contains("--trace-view"),
        "run_pilot.sh --receipt-replay HOW TO USE must reference --trace-view"
    );
}

// ── engineer interpretation section ──────────────────────────────────────────

#[test]
fn receipt_replay_has_engineer_interpretation_section() {
    assert!(
        RUN_PILOT_SH.contains("ENGINEER INTERPRETATION"),
        "run_pilot.sh --receipt-replay must print 'ENGINEER INTERPRETATION' section"
    );
}

#[test]
fn receipt_replay_engineer_interpretation_deterministic() {
    let block_start = RUN_PILOT_SH
        .find("ENGINEER INTERPRETATION")
        .expect("ENGINEER INTERPRETATION section must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 200];
    assert!(
        block.contains("deterministic"),
        "ENGINEER INTERPRETATION must include 'deterministic'"
    );
}

#[test]
fn receipt_replay_engineer_interpretation_replayable() {
    let block_start = RUN_PILOT_SH
        .find("ENGINEER INTERPRETATION")
        .expect("ENGINEER INTERPRETATION section must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 200];
    assert!(
        block.contains("replayable"),
        "ENGINEER INTERPRETATION must include 'replayable'"
    );
}

#[test]
fn receipt_replay_engineer_interpretation_audit_ready() {
    let block_start = RUN_PILOT_SH
        .find("ENGINEER INTERPRETATION")
        .expect("ENGINEER INTERPRETATION section must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 200];
    assert!(
        block.contains("audit-ready"),
        "ENGINEER INTERPRETATION must include 'audit-ready'"
    );
}

// ── receipt-present / receipt-absent messages ─────────────────────────────────

#[test]
fn receipt_replay_detected_message_when_receipt_present() {
    assert!(
        RUN_PILOT_SH.contains("Current receipt detected for replay-oriented inspection."),
        "run_pilot.sh must print receipt-detected message when receipt.json exists"
    );
}

#[test]
fn receipt_replay_generate_bundle_message_when_absent() {
    assert!(
        RUN_PILOT_SH.contains("Generate a pilot bundle first to create a receipt."),
        "run_pilot.sh must print generate-bundle message when receipt.json is absent"
    );
}

// ── determinism: no $(date) in block ─────────────────────────────────────────

#[test]
fn receipt_replay_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RECEIPT REPLAY")
        .expect("POSTCAD RECEIPT REPLAY header must exist");
    let block_end = RUN_PILOT_SH[block_start..]
        .find("exit 0")
        .map(|i| block_start + i + 6)
        .unwrap_or(block_start + 3000);
    let block = &RUN_PILOT_SH[block_start..block_end];
    assert!(
        !block.contains("$(date"),
        "receipt-replay block must not embed timestamps"
    );
}

// ── README ────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_receipt_replay_section() {
    assert!(
        README.contains("## Receipt Replay"),
        "README must have '## Receipt Replay' section"
    );
}

#[test]
fn readme_receipt_replay_shows_command() {
    assert!(
        README.contains("--receipt-replay"),
        "README must show --receipt-replay command"
    );
}
