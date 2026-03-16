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
