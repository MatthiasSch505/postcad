//! Next-step surface tests.
//!
//! Verifies that run_pilot.sh --next-step --run-id <id> implements the required
//! sections, state paths, inspection commands, error paths, and deterministic
//! behavior by inspecting the script source.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag support ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_next_step_flag() {
    assert!(
        RUN_PILOT_SH.contains("--next-step"),
        "run_pilot.sh must support --next-step flag"
    );
}

#[test]
fn run_pilot_next_step_requires_run_id_flag() {
    assert!(
        RUN_PILOT_SH.contains("--next-step requires --run-id"),
        "must emit error when --run-id is missing"
    );
}

// ── header ─────────────────────────────────────────────────────────────────────

#[test]
fn next_step_prints_postcad_next_step_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD NEXT STEP"),
        "run_pilot.sh --next-step must print 'POSTCAD NEXT STEP' header"
    );
}

#[test]
fn next_step_block_exits_0_on_success() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("exit 0"),
        "--next-step block must exit 0 on success"
    );
}

// ── RUN section ────────────────────────────────────────────────────────────────

#[test]
fn next_step_has_run_section() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("echo \"RUN\""),
        "--next-step block must include 'RUN' section"
    );
}

#[test]
fn next_step_run_section_shows_run_id() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("Run ID"),
        "'RUN' section must show 'Run ID' label"
    );
}

#[test]
fn next_step_run_section_shows_case_id() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("Case ID"),
        "'RUN' section must show 'Case ID' label"
    );
}

// ── CURRENT STATE section ──────────────────────────────────────────────────────

#[test]
fn next_step_has_current_state_section() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("echo \"CURRENT STATE\""),
        "--next-step block must include 'CURRENT STATE' section"
    );
}

#[test]
fn next_step_current_state_shows_state_label() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("State"),
        "'CURRENT STATE' section must include 'State' label"
    );
}

// ── state paths ───────────────────────────────────────────────────────────────

#[test]
fn next_step_handles_routing_refused_state() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("routing refused"),
        "must handle 'routing refused' state for refused cases"
    );
}

#[test]
fn next_step_handles_awaiting_lab_reply_state() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("awaiting lab reply"),
        "must handle 'awaiting lab reply' state"
    );
}

#[test]
fn next_step_handles_awaiting_verification_state() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("awaiting verification"),
        "must handle 'awaiting verification' state"
    );
}

#[test]
fn next_step_handles_complete_state() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("complete"),
        "must handle 'complete' state when all artifacts present"
    );
}

// ── RECOMMENDED HUMAN ACTION section ──────────────────────────────────────────

#[test]
fn next_step_has_recommended_action_section() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("echo \"RECOMMENDED HUMAN ACTION\""),
        "--next-step block must include 'RECOMMENDED HUMAN ACTION' section"
    );
}

// ── FILES TO REVIEW section ────────────────────────────────────────────────────

#[test]
fn next_step_has_files_to_review_section() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("echo \"FILES TO REVIEW\""),
        "--next-step block must include 'FILES TO REVIEW' section"
    );
}

#[test]
fn next_step_files_shows_receipt() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("receipt.json"),
        "'FILES TO REVIEW' must reference receipt.json"
    );
}

#[test]
fn next_step_files_shows_inbound_reply() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("inbound reply"),
        "'FILES TO REVIEW' must reference 'inbound reply'"
    );
}

#[test]
fn next_step_files_shows_verification_record() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("verification record"),
        "'FILES TO REVIEW' must reference 'verification record'"
    );
}

#[test]
fn next_step_files_shows_dispatch_packet() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("dispatch packet"),
        "'FILES TO REVIEW' must reference 'dispatch packet'"
    );
}

// ── COMMANDS section ───────────────────────────────────────────────────────────

#[test]
fn next_step_has_commands_section() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("echo \"COMMANDS\""),
        "--next-step block must include 'COMMANDS' section"
    );
}

#[test]
fn next_step_commands_includes_run_summary() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("--run-summary"),
        "'COMMANDS' must include --run-summary"
    );
}

#[test]
fn next_step_commands_includes_protocol_chain() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("--protocol-chain"),
        "'COMMANDS' must include --protocol-chain"
    );
}

#[test]
fn next_step_commands_includes_business_entrypoint() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("--business-entrypoint"),
        "'COMMANDS' must include --business-entrypoint"
    );
}

#[test]
fn next_step_commands_includes_lab_entrypoint() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("--lab-entrypoint"),
        "'COMMANDS' must include --lab-entrypoint"
    );
}

#[test]
fn next_step_commands_includes_audit_receipt_view() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        block.contains("--audit-receipt-view"),
        "'COMMANDS' must include --audit-receipt-view"
    );
}

// ── error paths ───────────────────────────────────────────────────────────────

#[test]
fn next_step_missing_run_id_shows_error() {
    assert!(
        RUN_PILOT_SH.contains("--next-step requires --run-id"),
        "must emit '--next-step requires --run-id' error when run id is missing"
    );
}

#[test]
fn next_step_missing_run_id_shows_usage() {
    assert!(
        RUN_PILOT_SH.contains("Usage: ./examples/pilot/run_pilot.sh --next-step --run-id"),
        "must print usage line when --run-id value is missing"
    );
}

#[test]
fn next_step_missing_run_id_exits_nonzero() {
    let block_start = RUN_PILOT_SH
        .find("--next-step requires --run-id")
        .expect("error text must exist");
    let after = &RUN_PILOT_SH[block_start..block_start + 300];
    assert!(
        after.contains("exit 1"),
        "must exit 1 when --run-id is missing"
    );
}

#[test]
fn next_step_missing_receipt_shows_error() {
    assert!(
        RUN_PILOT_SH.contains("no receipt.json found"),
        "must emit 'no receipt.json found' error message when receipt is absent"
    );
}

// ── stable section ordering ────────────────────────────────────────────────────

#[test]
fn next_step_stable_order_run_before_current_state() {
    let base = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[base..base + 3000];
    let run_pos = block.find("echo \"RUN\"").expect("RUN section must exist");
    let state_pos = block
        .find("echo \"CURRENT STATE\"")
        .expect("CURRENT STATE section must exist");
    assert!(run_pos < state_pos, "RUN section must appear before CURRENT STATE");
}

#[test]
fn next_step_stable_order_current_state_before_recommended_action() {
    let base = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[base..base + 3000];
    let state_pos = block
        .find("echo \"CURRENT STATE\"")
        .expect("CURRENT STATE section must exist");
    let action_pos = block
        .find("echo \"RECOMMENDED HUMAN ACTION\"")
        .expect("RECOMMENDED HUMAN ACTION section must exist");
    assert!(
        state_pos < action_pos,
        "CURRENT STATE must appear before RECOMMENDED HUMAN ACTION"
    );
}

#[test]
fn next_step_stable_order_recommended_action_before_files() {
    let base = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[base..base + 3000];
    let action_pos = block
        .find("echo \"RECOMMENDED HUMAN ACTION\"")
        .expect("RECOMMENDED HUMAN ACTION section must exist");
    let files_pos = block
        .find("echo \"FILES TO REVIEW\"")
        .expect("FILES TO REVIEW section must exist");
    assert!(
        action_pos < files_pos,
        "RECOMMENDED HUMAN ACTION must appear before FILES TO REVIEW"
    );
}

#[test]
fn next_step_stable_order_files_before_commands() {
    let base = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[base..base + 3000];
    let files_pos = block
        .find("echo \"FILES TO REVIEW\"")
        .expect("FILES TO REVIEW section must exist");
    let commands_pos = block
        .find("echo \"COMMANDS\"")
        .expect("COMMANDS section must exist");
    assert!(
        files_pos < commands_pos,
        "FILES TO REVIEW must appear before COMMANDS"
    );
}

// ── determinism ────────────────────────────────────────────────────────────────

#[test]
fn next_step_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        !block.contains("$(date"),
        "--next-step block must not embed timestamps via $(date"
    );
}

#[test]
fn next_step_block_is_readonly() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD NEXT STEP")
        .expect("POSTCAD NEXT STEP must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 3000];
    assert!(
        !block.contains("> \"${SCRIPT_DIR}"),
        "--next-step block must not write files"
    );
}

// ── README ─────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_next_step_section() {
    assert!(
        README.contains("## Next Step"),
        "README must have '## Next Step' section"
    );
}

#[test]
fn readme_shows_next_step_with_run_id() {
    assert!(
        README.contains("--next-step --run-id"),
        "README must show '--next-step --run-id' command example"
    );
}
