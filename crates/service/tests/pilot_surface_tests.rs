//! Pilot surface tests.
//!
//! Verifies that run_pilot.sh --operator-inbox implements the required
//! sections, DONE/PENDING markers, next-unresolved-item logic, complete-workflow
//! path, and deterministic behavior by inspecting the script source.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag support ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_operator_inbox_flag() {
    assert!(
        RUN_PILOT_SH.contains("--operator-inbox"),
        "run_pilot.sh must support --operator-inbox flag"
    );
}

#[test]
fn operator_inbox_block_exits_0_on_success() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("exit 0"),
        "--operator-inbox block must exit 0 on success"
    );
}

// ── header ─────────────────────────────────────────────────────────────────────

#[test]
fn operator_inbox_prints_postcad_operator_inbox_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD OPERATOR INBOX"),
        "run_pilot.sh --operator-inbox must print 'POSTCAD OPERATOR INBOX' header"
    );
}

// ── RUN CONTEXT section ────────────────────────────────────────────────────────

#[test]
fn operator_inbox_has_run_context_section() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("echo \"RUN CONTEXT\""),
        "--operator-inbox block must include 'RUN CONTEXT' section"
    );
}

#[test]
fn operator_inbox_run_context_shows_run_id() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("Run ID"),
        "'RUN CONTEXT' section must show 'Run ID'"
    );
}

#[test]
fn operator_inbox_run_context_shows_case_id() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("Case ID"),
        "'RUN CONTEXT' section must show 'Case ID'"
    );
}

#[test]
fn operator_inbox_run_context_shows_target_lab() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("Target lab"),
        "'RUN CONTEXT' section must show 'Target lab'"
    );
}

// ── ARTIFACT STATUS section ────────────────────────────────────────────────────

#[test]
fn operator_inbox_has_artifact_status_section() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("echo \"ARTIFACT STATUS\""),
        "--operator-inbox block must include 'ARTIFACT STATUS' section"
    );
}

#[test]
fn operator_inbox_artifact_status_shows_receipt() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("receipt.json"),
        "'ARTIFACT STATUS' must reference receipt.json"
    );
}

#[test]
fn operator_inbox_artifact_status_shows_dispatch_packet() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("export_packet.json"),
        "'ARTIFACT STATUS' must reference export_packet.json"
    );
}

#[test]
fn operator_inbox_artifact_status_shows_outbound_package() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("outbound package"),
        "'ARTIFACT STATUS' must reference 'outbound package'"
    );
}

#[test]
fn operator_inbox_artifact_status_shows_send_note() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("send note"),
        "'ARTIFACT STATUS' must reference 'send note'"
    );
}

#[test]
fn operator_inbox_artifact_status_shows_lab_reply() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("lab reply"),
        "'ARTIFACT STATUS' must reference 'lab reply'"
    );
}

#[test]
fn operator_inbox_artifact_status_shows_verification_record() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("verification record"),
        "'ARTIFACT STATUS' must reference 'verification record'"
    );
}

// ── DONE / PENDING markers ────────────────────────────────────────────────────

#[test]
fn operator_inbox_uses_done_marker() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("[DONE]"),
        "'ARTIFACT STATUS' must use [DONE] marker for present artifacts"
    );
}

#[test]
fn operator_inbox_uses_pending_marker() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("[PENDING]"),
        "'ARTIFACT STATUS' must use [PENDING] marker for absent artifacts"
    );
}

// ── partial workflow path ─────────────────────────────────────────────────────

#[test]
fn operator_inbox_partial_workflow_shows_next_unresolved_item() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("NEXT UNRESOLVED ITEM"),
        "partial workflow must show 'NEXT UNRESOLVED ITEM' section"
    );
}

#[test]
fn operator_inbox_partial_workflow_shows_pending_items() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("[PENDING]"),
        "partial workflow must show [PENDING] items for missing artifacts"
    );
}

#[test]
fn operator_inbox_partial_workflow_first_pending_is_receipt() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("generate routing receipt"),
        "first pending step must direct operator to generate routing receipt"
    );
}

#[test]
fn operator_inbox_partial_workflow_awaiting_lab_reply_message() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("awaiting lab reply"),
        "must show 'awaiting lab reply' message when outbound package sent but no reply yet"
    );
}

// ── complete workflow path ─────────────────────────────────────────────────────

#[test]
fn operator_inbox_complete_workflow_shows_inbox_result() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("INBOX RESULT"),
        "complete workflow must show 'INBOX RESULT' section"
    );
}

#[test]
fn operator_inbox_complete_workflow_shows_workflow_complete_message() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("workflow complete"),
        "complete workflow must print 'workflow complete' message"
    );
}

#[test]
fn operator_inbox_complete_workflow_shows_no_unresolved_items_message() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("no unresolved operator items"),
        "complete workflow must print 'no unresolved operator items'"
    );
}

// ── stable section ordering ────────────────────────────────────────────────────

#[test]
fn operator_inbox_stable_order_run_context_before_artifact_status() {
    let base = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[base..base + 4000];
    let ctx_pos = block
        .find("echo \"RUN CONTEXT\"")
        .expect("RUN CONTEXT section must exist");
    let art_pos = block
        .find("echo \"ARTIFACT STATUS\"")
        .expect("ARTIFACT STATUS section must exist");
    assert!(
        ctx_pos < art_pos,
        "RUN CONTEXT section must appear before ARTIFACT STATUS"
    );
}

#[test]
fn operator_inbox_stable_order_artifact_status_before_result() {
    let base = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[base..base + 4000];
    let art_pos = block
        .find("echo \"ARTIFACT STATUS\"")
        .expect("ARTIFACT STATUS section must exist");
    let result_pos = block
        .find("INBOX RESULT")
        .expect("INBOX RESULT must exist");
    assert!(
        art_pos < result_pos,
        "ARTIFACT STATUS section must appear before INBOX RESULT"
    );
}

// ── determinism ────────────────────────────────────────────────────────────────

#[test]
fn operator_inbox_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        !block.contains("$(date"),
        "--operator-inbox block must not embed timestamps via $(date"
    );
}

#[test]
fn operator_inbox_block_is_readonly() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD OPERATOR INBOX")
        .expect("POSTCAD OPERATOR INBOX must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        !block.contains("> \"${SCRIPT_DIR}"),
        "--operator-inbox block must not write files"
    );
}

// ── README ─────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_operator_inbox_section() {
    assert!(
        README.contains("## Operator Inbox"),
        "README must have '## Operator Inbox' section"
    );
}

#[test]
fn readme_shows_operator_inbox_command() {
    assert!(
        README.contains("--operator-inbox"),
        "README must show '--operator-inbox' command"
    );
}

#[test]
fn readme_operator_inbox_mentions_done_pending() {
    let section_start = README
        .find("## Operator Inbox")
        .expect("'## Operator Inbox' section must exist");
    let section = &README[section_start..section_start + 800];
    assert!(
        section.contains("[DONE]") || section.contains("DONE"),
        "README Operator Inbox section must mention DONE/PENDING status markers"
    );
}

#[test]
fn readme_operator_inbox_mentions_workflow_complete() {
    let section_start = README
        .find("## Operator Inbox")
        .expect("'## Operator Inbox' section must exist");
    let section = &README[section_start..section_start + 800];
    assert!(
        section.contains("workflow complete"),
        "README Operator Inbox section must mention 'workflow complete' path"
    );
}

// ── Timeline: flag support ─────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_timeline_flag() {
    assert!(
        RUN_PILOT_SH.contains("--timeline"),
        "run_pilot.sh must support --timeline flag"
    );
}

#[test]
fn timeline_block_exits_0_on_success() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("exit 0"),
        "--timeline block must exit 0 on success"
    );
}

// ── Timeline: header ───────────────────────────────────────────────────────────

#[test]
fn timeline_prints_postcad_workflow_timeline_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD WORKFLOW TIMELINE"),
        "run_pilot.sh --timeline must print 'POSTCAD WORKFLOW TIMELINE' header"
    );
}

// ── Timeline: RUN CONTEXT section ─────────────────────────────────────────────

#[test]
fn timeline_has_run_context_section() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("echo \"RUN CONTEXT\""),
        "--timeline block must include 'RUN CONTEXT' section"
    );
}

#[test]
fn timeline_run_context_shows_run_id() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("Run ID"),
        "'RUN CONTEXT' section must show 'Run ID'"
    );
}

#[test]
fn timeline_run_context_shows_case_id() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("Case ID"),
        "'RUN CONTEXT' section must show 'Case ID'"
    );
}

#[test]
fn timeline_run_context_shows_target_lab() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("Target lab"),
        "'RUN CONTEXT' section must show 'Target lab'"
    );
}

// ── Timeline: TIMELINE section ────────────────────────────────────────────────

#[test]
fn timeline_has_timeline_section() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("echo \"TIMELINE\""),
        "--timeline block must include 'TIMELINE' section"
    );
}

#[test]
fn timeline_uses_done_marker() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("[DONE]"),
        "'TIMELINE' must use [DONE] marker for completed stages"
    );
}

#[test]
fn timeline_uses_pending_marker() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("[PENDING]"),
        "'TIMELINE' must use [PENDING] marker for missing stages"
    );
}

// ── Timeline: artifact stages ─────────────────────────────────────────────────

#[test]
fn timeline_includes_routing_receipt_stage() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("routing receipt"),
        "'TIMELINE' must include routing receipt stage"
    );
}

#[test]
fn timeline_includes_outbound_package_stage() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("outbound package"),
        "'TIMELINE' must include outbound package stage"
    );
}

#[test]
fn timeline_includes_send_log_stage() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("send log"),
        "'TIMELINE' must include send log stage"
    );
}

#[test]
fn timeline_includes_lab_reply_stage() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("lab reply"),
        "'TIMELINE' must include lab reply stage"
    );
}

#[test]
fn timeline_includes_verification_stage() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("verification"),
        "'TIMELINE' must include verification stage"
    );
}

#[test]
fn timeline_includes_dispatch_packet_stage() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("dispatch packet"),
        "'TIMELINE' must include dispatch packet stage"
    );
}

// ── Timeline: partial workflow path ───────────────────────────────────────────

#[test]
fn timeline_partial_workflow_shows_current_stage() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("CURRENT STAGE"),
        "partial workflow must show 'CURRENT STAGE' section"
    );
}

#[test]
fn timeline_partial_workflow_shows_next_missing_stage() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("NEXT MISSING STAGE"),
        "partial workflow must show 'NEXT MISSING STAGE' section"
    );
}

#[test]
fn timeline_partial_workflow_awaiting_lab_reply_message() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("awaiting lab reply"),
        "must show 'awaiting lab reply' message when outbound sent but no reply"
    );
}

// ── Timeline: complete workflow path ──────────────────────────────────────────

#[test]
fn timeline_complete_workflow_shows_timeline_result() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("TIMELINE RESULT"),
        "complete workflow must show 'TIMELINE RESULT' section"
    );
}

#[test]
fn timeline_complete_workflow_shows_workflow_complete_message() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("workflow complete"),
        "complete workflow must print 'workflow complete' message"
    );
}

#[test]
fn timeline_complete_workflow_shows_all_stages_message() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        block.contains("all stages evidenced by artifacts"),
        "complete workflow must print 'all stages evidenced by artifacts'"
    );
}

// ── Timeline: stable section ordering ────────────────────────────────────────

#[test]
fn timeline_stable_order_run_context_before_timeline() {
    let base = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[base..base + 4500];
    let ctx_pos = block
        .find("echo \"RUN CONTEXT\"")
        .expect("RUN CONTEXT section must exist");
    let tl_pos = block
        .find("echo \"TIMELINE\"")
        .expect("TIMELINE section must exist");
    assert!(
        ctx_pos < tl_pos,
        "RUN CONTEXT section must appear before TIMELINE"
    );
}

#[test]
fn timeline_stable_order_timeline_before_result() {
    let base = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[base..base + 4500];
    let tl_pos = block
        .find("echo \"TIMELINE\"")
        .expect("TIMELINE section must exist");
    let result_pos = block
        .find("TIMELINE RESULT")
        .expect("TIMELINE RESULT must exist");
    assert!(
        tl_pos < result_pos,
        "TIMELINE section must appear before TIMELINE RESULT"
    );
}

// ── Timeline: determinism ─────────────────────────────────────────────────────

#[test]
fn timeline_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        !block.contains("$(date"),
        "--timeline block must not embed timestamps via $(date"
    );
}

#[test]
fn timeline_block_is_readonly() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD WORKFLOW TIMELINE")
        .expect("POSTCAD WORKFLOW TIMELINE must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4500];
    assert!(
        !block.contains("> \"${SCRIPT_DIR}"),
        "--timeline block must not write files"
    );
}

// ── Timeline: README ──────────────────────────────────────────────────────────

#[test]
fn readme_has_timeline_section() {
    assert!(
        README.contains("## Timeline"),
        "README must have '## Timeline' section"
    );
}

#[test]
fn readme_shows_timeline_command() {
    assert!(
        README.contains("--timeline"),
        "README must show '--timeline' command"
    );
}

// ── Pilot Demo: flag support ──────────────────────────────────────────────────

#[test]
fn run_pilot_supports_pilot_demo_flag() {
    assert!(
        RUN_PILOT_SH.contains("--pilot-demo"),
        "run_pilot.sh must support --pilot-demo flag"
    );
}

#[test]
fn pilot_demo_block_exits_0_on_success() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("exit 0"),
        "--pilot-demo block must exit 0 on success"
    );
}

// ── Pilot Demo: header ────────────────────────────────────────────────────────

#[test]
fn pilot_demo_prints_postcad_pilot_demo_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD PILOT DEMO"),
        "run_pilot.sh --pilot-demo must print 'POSTCAD PILOT DEMO' header"
    );
}

// ── Pilot Demo: RUN CONTEXT section ──────────────────────────────────────────

#[test]
fn pilot_demo_has_run_context_section() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("echo \"RUN CONTEXT\""),
        "--pilot-demo block must include 'RUN CONTEXT' section"
    );
}

#[test]
fn pilot_demo_run_context_shows_run_id() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("Run ID"),
        "'RUN CONTEXT' section must show 'Run ID'"
    );
}

#[test]
fn pilot_demo_run_context_shows_case_id() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("Case ID"),
        "'RUN CONTEXT' section must show 'Case ID'"
    );
}

#[test]
fn pilot_demo_run_context_shows_target_lab() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("Target lab"),
        "'RUN CONTEXT' section must show 'Target lab'"
    );
}

// ── Pilot Demo: STAGE FLOW section ───────────────────────────────────────────

#[test]
fn pilot_demo_has_stage_flow_section() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("echo \"STAGE FLOW\""),
        "--pilot-demo block must include 'STAGE FLOW' section"
    );
}

#[test]
fn pilot_demo_uses_done_marker() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("[DONE]"),
        "'STAGE FLOW' must use [DONE] marker for completed stages"
    );
}

#[test]
fn pilot_demo_uses_pending_marker() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("[PENDING]"),
        "'STAGE FLOW' must use [PENDING] marker for missing stages"
    );
}

// ── Pilot Demo: stage labels ──────────────────────────────────────────────────

#[test]
fn pilot_demo_includes_cad_case_stage() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("CAD case available"),
        "'STAGE FLOW' must include 'CAD case available' stage"
    );
}

#[test]
fn pilot_demo_includes_routing_decision_stage() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("routing decision"),
        "'STAGE FLOW' must include routing decision stage"
    );
}

#[test]
fn pilot_demo_includes_compliance_stage() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("compliance"),
        "'STAGE FLOW' must include compliance/eligibility stage"
    );
}

#[test]
fn pilot_demo_includes_outbound_package_stage() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("outbound package"),
        "'STAGE FLOW' must include outbound package stage"
    );
}

#[test]
fn pilot_demo_includes_send_log_stage() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("send log"),
        "'STAGE FLOW' must include send log stage"
    );
}

#[test]
fn pilot_demo_includes_lab_reply_stage() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("lab reply"),
        "'STAGE FLOW' must include lab reply stage"
    );
}

#[test]
fn pilot_demo_includes_verification_stage() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("verification artifact"),
        "'STAGE FLOW' must include verification artifact stage"
    );
}

#[test]
fn pilot_demo_includes_audit_receipt_stage() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("audit receipt"),
        "'STAGE FLOW' must include audit receipt stage"
    );
}

// ── Pilot Demo: complete workflow path ────────────────────────────────────────

#[test]
fn pilot_demo_complete_workflow_shows_demo_complete_message() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("Demo complete"),
        "complete workflow must print 'Demo complete' message"
    );
}

#[test]
fn pilot_demo_complete_workflow_shows_end_to_end_evidence_message() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("end-to-end protocol evidence present"),
        "complete workflow must print 'end-to-end protocol evidence present'"
    );
}

// ── Pilot Demo: partial workflow path ─────────────────────────────────────────

#[test]
fn pilot_demo_partial_workflow_shows_demo_in_progress_message() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("Demo in progress"),
        "partial workflow must print 'Demo in progress' message"
    );
}

#[test]
fn pilot_demo_partial_workflow_shows_next_stage_label() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        block.contains("next stage:"),
        "partial workflow must show 'next stage:' label"
    );
}

// ── Pilot Demo: stable section ordering ──────────────────────────────────────

#[test]
fn pilot_demo_stable_order_run_context_before_stage_flow() {
    let base = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[base..base + 5000];
    let ctx_pos = block
        .find("echo \"RUN CONTEXT\"")
        .expect("RUN CONTEXT section must exist");
    let sf_pos = block
        .find("echo \"STAGE FLOW\"")
        .expect("STAGE FLOW section must exist");
    assert!(
        ctx_pos < sf_pos,
        "RUN CONTEXT section must appear before STAGE FLOW"
    );
}

#[test]
fn pilot_demo_stable_order_stage_flow_before_summary() {
    let base = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[base..base + 5000];
    let sf_pos = block
        .find("echo \"STAGE FLOW\"")
        .expect("STAGE FLOW section must exist");
    let summary_pos = block
        .find("Demo complete")
        .expect("Demo complete message must exist");
    assert!(
        sf_pos < summary_pos,
        "STAGE FLOW section must appear before summary line"
    );
}

// ── Pilot Demo: determinism ───────────────────────────────────────────────────

#[test]
fn pilot_demo_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        !block.contains("$(date"),
        "--pilot-demo block must not embed timestamps via $(date"
    );
}

#[test]
fn pilot_demo_block_is_readonly() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD PILOT DEMO")
        .expect("POSTCAD PILOT DEMO must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 5000];
    assert!(
        !block.contains("> \"${SCRIPT_DIR}"),
        "--pilot-demo block must not write files"
    );
}

// ── Pilot Demo: README ────────────────────────────────────────────────────────

#[test]
fn readme_has_pilot_demo_section() {
    assert!(
        README.contains("## Pilot Demo"),
        "README must have '## Pilot Demo' section"
    );
}

#[test]
fn readme_shows_pilot_demo_command() {
    assert!(
        README.contains("--pilot-demo"),
        "README must show '--pilot-demo' command"
    );
}
