//! Run summary surface tests.
//!
//! Verifies that run_pilot.sh --run-summary --run-id <id> implements the required
//! sections, inspection command references, error paths, and deterministic behavior
//! by inspecting the script source.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag support ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_run_summary_with_run_id() {
    assert!(
        RUN_PILOT_SH.contains("--run-id"),
        "run_pilot.sh --run-summary must support --run-id argument"
    );
}

#[test]
fn run_summary_run_id_block_exits_0_on_success() {
    // The --run-id variant must have an exit 0 at the end of its block
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("exit 0"),
        "--run-summary --run-id block must exit 0 on success"
    );
}

// ── header ─────────────────────────────────────────────────────────────────────

#[test]
fn run_summary_run_id_prints_postcad_run_summary_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD RUN SUMMARY"),
        "run_pilot.sh --run-summary --run-id must print 'POSTCAD RUN SUMMARY' header"
    );
}

// ── RUN section ────────────────────────────────────────────────────────────────

#[test]
fn run_summary_run_id_has_run_section() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("RUN\n") || block.contains("echo \"RUN\""),
        "--run-summary --run-id block must include 'RUN' section"
    );
}

#[test]
fn run_summary_run_id_run_section_shows_run_id_label() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("Run ID"),
        "'RUN' section must show 'Run ID' label"
    );
}

#[test]
fn run_summary_run_id_run_section_shows_fingerprint() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("Fingerprint"),
        "'RUN' section must show 'Fingerprint' when available"
    );
}

// ── BUSINESS section ───────────────────────────────────────────────────────────

#[test]
fn run_summary_run_id_has_business_section() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("BUSINESS"),
        "--run-summary --run-id block must include 'BUSINESS' section"
    );
}

#[test]
fn run_summary_run_id_business_shows_case_id() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("Case ID"),
        "'BUSINESS' section must show 'Case ID'"
    );
}

#[test]
fn run_summary_run_id_business_shows_jurisdiction() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("Jurisdiction"),
        "'BUSINESS' section must show 'Jurisdiction'"
    );
}

// ── ROUTING section ────────────────────────────────────────────────────────────

#[test]
fn run_summary_run_id_has_routing_section() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("ROUTING"),
        "--run-summary --run-id block must include 'ROUTING' section"
    );
}

#[test]
fn run_summary_run_id_routing_shows_target_lab() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("Target lab"),
        "'ROUTING' section must show 'Target lab'"
    );
}

#[test]
fn run_summary_run_id_routing_shows_decision() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("Decision"),
        "'ROUTING' section must show 'Decision' (accepted or refused path)"
    );
}

// ── ARTIFACTS section ──────────────────────────────────────────────────────────

#[test]
fn run_summary_run_id_has_artifacts_section() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("ARTIFACTS"),
        "--run-summary --run-id block must include 'ARTIFACTS' section"
    );
}

#[test]
fn run_summary_run_id_artifacts_shows_receipt() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("RSI_RECEIPT_FILE"),
        "'ARTIFACTS' section must reference RSI_RECEIPT_FILE (receipt.json path)"
    );
}

#[test]
fn run_summary_run_id_artifacts_shows_inbound_reply() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("inbound reply"),
        "'ARTIFACTS' section must show 'inbound reply'"
    );
}

#[test]
fn run_summary_run_id_artifacts_shows_verification_record() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("verification record"),
        "'ARTIFACTS' section must show 'verification record'"
    );
}

#[test]
fn run_summary_run_id_artifacts_shows_dispatch_packet() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("dispatch packet"),
        "'ARTIFACTS' section must show 'dispatch packet'"
    );
}

// ── NEXT INSPECTION COMMANDS section ──────────────────────────────────────────

#[test]
fn run_summary_run_id_has_next_inspection_commands_section() {
    assert!(
        RUN_PILOT_SH.contains("NEXT INSPECTION COMMANDS"),
        "--run-summary --run-id block must include 'NEXT INSPECTION COMMANDS' section"
    );
}

#[test]
fn run_summary_run_id_next_includes_protocol_chain() {
    let block_start = RUN_PILOT_SH
        .find("NEXT INSPECTION COMMANDS")
        .expect("NEXT INSPECTION COMMANDS must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 500];
    assert!(
        block.contains("--protocol-chain"),
        "'NEXT INSPECTION COMMANDS' must include --protocol-chain"
    );
}

#[test]
fn run_summary_run_id_next_includes_engineer_entrypoint() {
    let block_start = RUN_PILOT_SH
        .find("NEXT INSPECTION COMMANDS")
        .expect("NEXT INSPECTION COMMANDS must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 500];
    assert!(
        block.contains("--engineer-entrypoint"),
        "'NEXT INSPECTION COMMANDS' must include --engineer-entrypoint"
    );
}

#[test]
fn run_summary_run_id_next_includes_business_entrypoint() {
    let block_start = RUN_PILOT_SH
        .find("NEXT INSPECTION COMMANDS")
        .expect("NEXT INSPECTION COMMANDS must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 500];
    assert!(
        block.contains("--business-entrypoint"),
        "'NEXT INSPECTION COMMANDS' must include --business-entrypoint"
    );
}

#[test]
fn run_summary_run_id_next_includes_lab_entrypoint() {
    let block_start = RUN_PILOT_SH
        .find("NEXT INSPECTION COMMANDS")
        .expect("NEXT INSPECTION COMMANDS must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 500];
    assert!(
        block.contains("--lab-entrypoint"),
        "'NEXT INSPECTION COMMANDS' must include --lab-entrypoint"
    );
}

#[test]
fn run_summary_run_id_next_includes_audit_receipt_view() {
    let block_start = RUN_PILOT_SH
        .find("NEXT INSPECTION COMMANDS")
        .expect("NEXT INSPECTION COMMANDS must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 500];
    assert!(
        block.contains("--audit-receipt-view"),
        "'NEXT INSPECTION COMMANDS' must include --audit-receipt-view"
    );
}

// ── error paths ────────────────────────────────────────────────────────────────

#[test]
fn run_summary_missing_run_id_value_shows_error() {
    assert!(
        RUN_PILOT_SH.contains("--run-id requires a run id value"),
        "must emit '--run-id requires a run id value' error when run id is missing"
    );
}

#[test]
fn run_summary_missing_run_id_shows_usage() {
    assert!(
        RUN_PILOT_SH.contains("Usage: ./examples/pilot/run_pilot.sh --run-summary --run-id"),
        "must print usage line when --run-id value is missing"
    );
}

#[test]
fn run_summary_missing_artifact_exits_nonzero() {
    // When receipt.json is absent the block must exit 1
    let block_start = RUN_PILOT_SH
        .find("--run-id requires a run id value")
        .expect("error text must exist");
    // The receipt-not-found guard and its exit 1 must both appear after that
    let after = &RUN_PILOT_SH[block_start..block_start + 600];
    assert!(
        after.contains("exit 1"),
        "must exit 1 when required receipt artifact is missing"
    );
}

#[test]
fn run_summary_missing_artifact_shows_clear_error() {
    assert!(
        RUN_PILOT_SH.contains("no receipt.json found"),
        "must emit 'no receipt.json found' error message when receipt is absent"
    );
}

// ── stable section ordering ────────────────────────────────────────────────────

#[test]
fn run_summary_run_id_stable_section_order_run_before_business() {
    let base = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[base..base + 4000];
    let run_pos = block.find("echo \"RUN\"").expect("RUN section must exist");
    let biz_pos = block.find("echo \"BUSINESS\"").expect("BUSINESS section must exist");
    assert!(run_pos < biz_pos, "RUN section must appear before BUSINESS section");
}

#[test]
fn run_summary_run_id_stable_section_order_business_before_routing() {
    let base = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[base..base + 4000];
    let biz_pos = block.find("echo \"BUSINESS\"").expect("BUSINESS section must exist");
    let rout_pos = block.find("echo \"ROUTING\"").expect("ROUTING section must exist");
    assert!(biz_pos < rout_pos, "BUSINESS section must appear before ROUTING section");
}

#[test]
fn run_summary_run_id_stable_section_order_routing_before_artifacts() {
    let base = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[base..base + 4000];
    let rout_pos = block.find("echo \"ROUTING\"").expect("ROUTING section must exist");
    let art_pos = block.find("echo \"ARTIFACTS\"").expect("ARTIFACTS section must exist");
    assert!(rout_pos < art_pos, "ROUTING section must appear before ARTIFACTS section");
}

#[test]
fn run_summary_run_id_stable_section_order_artifacts_before_next_inspection() {
    let base = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[base..base + 4000];
    let art_pos = block.find("echo \"ARTIFACTS\"").expect("ARTIFACTS section must exist");
    let next_pos = block
        .find("NEXT INSPECTION COMMANDS")
        .expect("NEXT INSPECTION COMMANDS must exist");
    assert!(
        art_pos < next_pos,
        "ARTIFACTS section must appear before NEXT INSPECTION COMMANDS"
    );
}

// ── determinism ────────────────────────────────────────────────────────────────

#[test]
fn run_summary_run_id_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        !block.contains("$(date"),
        "--run-summary --run-id block must not embed timestamps via $(date"
    );
}

#[test]
fn run_summary_run_id_is_readonly() {
    // Block must not write any files (no > redirection to script-dir paths)
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN SUMMARY")
        .expect("POSTCAD RUN SUMMARY must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        !block.contains("> \"${SCRIPT_DIR}"),
        "--run-summary --run-id block must not write files"
    );
}

// ── README ─────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_run_summary_section() {
    assert!(
        README.contains("## Run Summary"),
        "README must have '## Run Summary' section"
    );
}

#[test]
fn readme_shows_run_summary_with_run_id() {
    assert!(
        README.contains("--run-summary --run-id"),
        "README must show '--run-summary --run-id' command example"
    );
}
