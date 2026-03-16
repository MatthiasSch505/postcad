//! Pilot command map surface tests.
//!
//! Verifies that run_pilot.sh --command-map exists, prints the required
//! PURPOSE, FLOW, COMMANDS, and START HERE sections, lists the expected
//! inspection commands, and is documented in the README.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

/// Extract the --command-map block using ASCII-safe anchors.
fn command_map_block() -> &'static str {
    let start = RUN_PILOT_SH
        .find("\"--command-map\"")
        .expect("--command-map if-block must exist in run_pilot.sh");
    let after = &RUN_PILOT_SH[start..];
    let relative_end = after
        .find("exit 0\nfi\n")
        .map(|i| i + "exit 0\nfi\n".len())
        .unwrap_or(after.len());
    &RUN_PILOT_SH[start..start + relative_end]
}

// ── flag and help surface ─────────────────────────────────────────────────────

#[test]
fn command_map_flag_present_in_script() {
    assert!(
        RUN_PILOT_SH.contains("--command-map"),
        "--command-map flag must be present in run_pilot.sh"
    );
}

#[test]
fn command_map_appears_in_help_surface() {
    let help_start = RUN_PILOT_SH
        .find("--help-surface")
        .expect("--help-surface must exist");
    let tail = &RUN_PILOT_SH[help_start..];
    assert!(
        tail.contains("--command-map"),
        "--command-map must appear in the help surface"
    );
}

#[test]
fn command_map_block_exits_0() {
    let block = command_map_block();
    assert!(block.contains("exit 0"), "--command-map block must exit 0");
}

// ── header ────────────────────────────────────────────────────────────────────

#[test]
fn command_map_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD PILOT COMMAND MAP"),
        "--command-map must print 'POSTCAD PILOT COMMAND MAP' header"
    );
}

// ── PURPOSE section ───────────────────────────────────────────────────────────

#[test]
fn command_map_has_purpose_section() {
    let block = command_map_block();
    assert!(
        block.contains("echo \"PURPOSE\""),
        "--command-map must print 'PURPOSE' section"
    );
}

#[test]
fn command_map_purpose_mentions_postcad() {
    let block = command_map_block();
    let p = block.find("echo \"PURPOSE\"").expect("PURPOSE must exist");
    let area = &block[p..p + 400];
    assert!(
        area.contains("PostCAD"),
        "PURPOSE must mention PostCAD"
    );
}

#[test]
fn command_map_purpose_mentions_deterministic() {
    let block = command_map_block();
    let p = block.find("echo \"PURPOSE\"").expect("PURPOSE must exist");
    let area = &block[p..p + 400];
    assert!(
        area.contains("deterministic"),
        "PURPOSE must describe deterministic routing"
    );
}

#[test]
fn command_map_purpose_mentions_routing() {
    let block = command_map_block();
    let p = block.find("echo \"PURPOSE\"").expect("PURPOSE must exist");
    let area = &block[p..p + 400];
    assert!(
        area.contains("routing") || area.contains("route"),
        "PURPOSE must mention routing"
    );
}

// ── FLOW section ──────────────────────────────────────────────────────────────

#[test]
fn command_map_has_flow_section() {
    let block = command_map_block();
    assert!(
        block.contains("echo \"FLOW\""),
        "--command-map must print 'FLOW' section"
    );
}

#[test]
fn command_map_flow_shows_case_intake() {
    let block = command_map_block();
    assert!(
        block.contains("case intake"),
        "FLOW section must show 'case intake' stage"
    );
}

#[test]
fn command_map_flow_shows_compliance() {
    let block = command_map_block();
    assert!(
        block.contains("compliance"),
        "FLOW section must show 'compliance' stage"
    );
}

#[test]
fn command_map_flow_shows_routing() {
    let block = command_map_block();
    assert!(
        block.contains("routing"),
        "FLOW section must show 'routing' stage"
    );
}

#[test]
fn command_map_flow_shows_dispatch() {
    let block = command_map_block();
    assert!(
        block.contains("dispatch"),
        "FLOW section must show 'dispatch' stage"
    );
}

#[test]
fn command_map_flow_shows_audit() {
    let block = command_map_block();
    assert!(
        block.contains("audit"),
        "FLOW section must show 'audit' stage"
    );
}

// ── COMMANDS section ──────────────────────────────────────────────────────────

#[test]
fn command_map_has_commands_section() {
    let block = command_map_block();
    assert!(
        block.contains("echo \"COMMANDS\""),
        "--command-map must print 'COMMANDS' section"
    );
}

#[test]
fn command_map_lists_protocol_chain() {
    let block = command_map_block();
    assert!(
        block.contains("--protocol-chain"),
        "COMMANDS must list --protocol-chain"
    );
}

#[test]
fn command_map_lists_engineer_entrypoint() {
    let block = command_map_block();
    assert!(
        block.contains("--engineer-entrypoint"),
        "COMMANDS must list --engineer-entrypoint"
    );
}

#[test]
fn command_map_lists_business_entrypoint() {
    let block = command_map_block();
    assert!(
        block.contains("--business-entrypoint"),
        "COMMANDS must list --business-entrypoint"
    );
}

#[test]
fn command_map_lists_lab_entrypoint() {
    let block = command_map_block();
    assert!(
        block.contains("--lab-entrypoint"),
        "COMMANDS must list --lab-entrypoint"
    );
}

#[test]
fn command_map_lists_audit_receipt_view() {
    let block = command_map_block();
    assert!(
        block.contains("--audit-receipt-view"),
        "COMMANDS must list --audit-receipt-view"
    );
}

#[test]
fn command_map_lists_run_summary() {
    let block = command_map_block();
    assert!(
        block.contains("--run-summary"),
        "COMMANDS must list --run-summary"
    );
}

#[test]
fn command_map_lists_next_step() {
    let block = command_map_block();
    assert!(
        block.contains("--next-step"),
        "COMMANDS must list --next-step"
    );
}

#[test]
fn command_map_lists_operator_inbox() {
    let block = command_map_block();
    assert!(
        block.contains("--operator-inbox"),
        "COMMANDS must list --operator-inbox"
    );
}

#[test]
fn command_map_lists_timeline() {
    let block = command_map_block();
    assert!(
        block.contains("--timeline"),
        "COMMANDS must list --timeline"
    );
}

#[test]
fn command_map_lists_pilot_demo() {
    let block = command_map_block();
    assert!(
        block.contains("--pilot-demo"),
        "COMMANDS must list --pilot-demo"
    );
}

#[test]
fn command_map_lists_artifact_index() {
    let block = command_map_block();
    assert!(
        block.contains("--artifact-index"),
        "COMMANDS must list --artifact-index"
    );
}

// ── START HERE section ────────────────────────────────────────────────────────

#[test]
fn command_map_has_start_here_section() {
    let block = command_map_block();
    assert!(
        block.contains("START HERE"),
        "--command-map must print 'START HERE' section"
    );
}

#[test]
fn command_map_start_here_has_first_time_demo_path() {
    let block = command_map_block();
    let s = block.find("START HERE").expect("START HERE must exist");
    let area = &block[s..s + 300];
    assert!(
        area.contains("First-time demo") || area.contains("first-time demo"),
        "START HERE must include a first-time demo recommendation"
    );
}

#[test]
fn command_map_start_here_has_operator_review_path() {
    let block = command_map_block();
    let s = block.find("START HERE").expect("START HERE must exist");
    let area = &block[s..s + 300];
    assert!(
        area.contains("Operator review") || area.contains("operator review"),
        "START HERE must include an operator review path"
    );
}

#[test]
fn command_map_start_here_has_artifact_review_path() {
    let block = command_map_block();
    let s = block.find("START HERE").expect("START HERE must exist");
    let area = &block[s..s + 300];
    assert!(
        area.contains("Artifact review") || area.contains("artifact review"),
        "START HERE must include an artifact review path"
    );
}

// ── stable section ordering ───────────────────────────────────────────────────

#[test]
fn command_map_purpose_before_flow() {
    let block = command_map_block();
    let p = block.find("echo \"PURPOSE\"").expect("PURPOSE must exist");
    let f = block.find("echo \"FLOW\"").expect("FLOW must exist");
    assert!(p < f, "PURPOSE must appear before FLOW");
}

#[test]
fn command_map_flow_before_commands() {
    let block = command_map_block();
    let f = block.find("echo \"FLOW\"").expect("FLOW must exist");
    let c = block.find("echo \"COMMANDS\"").expect("COMMANDS must exist");
    assert!(f < c, "FLOW must appear before COMMANDS");
}

#[test]
fn command_map_commands_before_start_here() {
    let block = command_map_block();
    let c = block.find("echo \"COMMANDS\"").expect("COMMANDS must exist");
    let s = block.find("START HERE").expect("START HERE must exist");
    assert!(c < s, "COMMANDS must appear before START HERE");
}

// ── static — does not require artifacts ──────────────────────────────────────

#[test]
fn command_map_does_not_check_for_receipt() {
    let block = command_map_block();
    assert!(
        !block.contains("receipt.json"),
        "--command-map must not check for receipt.json (must work without artifacts)"
    );
}

#[test]
fn command_map_does_not_check_filesystem() {
    let block = command_map_block();
    // No -f or -d filesystem checks inside this static block
    assert!(
        !block.contains("-f \"${SCRIPT_DIR}") && !block.contains("-d \"${SCRIPT_DIR}"),
        "--command-map must not perform filesystem checks"
    );
}

// ── determinism ───────────────────────────────────────────────────────────────

#[test]
fn command_map_has_no_date_command() {
    let block = command_map_block();
    assert!(
        !block.contains("$(date"),
        "--command-map must not embed timestamps via $(date"
    );
}

#[test]
fn command_map_is_readonly() {
    let block = command_map_block();
    assert!(
        !block.contains("> \"${SCRIPT_DIR}"),
        "--command-map must not write files"
    );
}

// ── README ────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_command_map_section() {
    assert!(
        README.contains("## Command Map"),
        "README must have '## Command Map' section"
    );
}

#[test]
fn readme_shows_command_map_command() {
    assert!(
        README.contains("--command-map"),
        "README must show '--command-map' command"
    );
}

#[test]
fn readme_command_map_mentions_no_artifacts_required() {
    let s = README
        .find("## Command Map")
        .expect("'## Command Map' section must exist");
    let section = &README[s..s + 800];
    assert!(
        section.contains("not require") || section.contains("Does not require") || section.contains("without"),
        "README must clarify that --command-map works without pilot artifacts"
    );
}
