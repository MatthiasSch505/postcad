//! Pilot operator cheatsheet surface tests.
//!
//! Verifies that run_pilot.sh --operator-cheatsheet is present, deterministic,
//! static, and contains the required sections and command names. Also checks
//! that README documents the command.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

/// Extract the --operator-cheatsheet handler block from the script.
///
/// Anchors on the `== "--operator-cheatsheet" ]]; then` line — this is unique
/// to the if-block handler and cannot be confused with echo statements in
/// earlier sections. Ends at the first `exit 0\nfi\n` that closes the block.
fn cheatsheet_block() -> &'static str {
    let start = RUN_PILOT_SH
        .find("== \"--operator-cheatsheet\" ]]; then")
        .expect("--operator-cheatsheet if-block must exist in run_pilot.sh");
    let after = &RUN_PILOT_SH[start..];
    let relative_end = after
        .find("exit 0\nfi\n")
        .map(|i| i + "exit 0\nfi\n".len())
        .unwrap_or(after.len());
    &RUN_PILOT_SH[start..start + relative_end]
}

// ── flag exists in script ──────────────────────────────────────────────────

#[test]
fn run_pilot_supports_operator_cheatsheet_flag() {
    assert!(
        RUN_PILOT_SH.contains("--operator-cheatsheet"),
        "run_pilot.sh must support --operator-cheatsheet flag"
    );
}

#[test]
fn operator_cheatsheet_exits_0() {
    let block = cheatsheet_block();
    assert!(
        block.contains("exit 0"),
        "run_pilot.sh --operator-cheatsheet must exit 0"
    );
}

// ── required sections ─────────────────────────────────────────────────────

#[test]
fn operator_cheatsheet_has_what_this_shell_does_section() {
    let block = cheatsheet_block();
    assert!(
        block.contains("WHAT THIS SHELL DOES"),
        "run_pilot.sh --operator-cheatsheet must contain 'WHAT THIS SHELL DOES' section"
    );
}

#[test]
fn operator_cheatsheet_has_core_commands_section() {
    let block = cheatsheet_block();
    assert!(
        block.contains("CORE COMMANDS"),
        "run_pilot.sh --operator-cheatsheet must contain 'CORE COMMANDS' section"
    );
}

#[test]
fn operator_cheatsheet_has_when_to_use_them_section() {
    let block = cheatsheet_block();
    assert!(
        block.contains("WHEN TO USE THEM"),
        "run_pilot.sh --operator-cheatsheet must contain 'WHEN TO USE THEM' section"
    );
}

#[test]
fn operator_cheatsheet_has_safe_review_path_section() {
    let block = cheatsheet_block();
    assert!(
        block.contains("SAFE REVIEW PATH"),
        "run_pilot.sh --operator-cheatsheet must contain 'SAFE REVIEW PATH' section"
    );
}

// ── core commands completeness ─────────────────────────────────────────────

#[test]
fn operator_cheatsheet_core_commands_includes_default() {
    let block = cheatsheet_block();
    assert!(
        block.contains("Generate pilot bundle"),
        "CORE COMMANDS must include 'Generate pilot bundle'"
    );
}

#[test]
fn operator_cheatsheet_core_commands_includes_inspect_inbound_reply() {
    let block = cheatsheet_block();
    assert!(
        block.contains("--inspect-inbound-reply"),
        "CORE COMMANDS must include --inspect-inbound-reply"
    );
}

#[test]
fn operator_cheatsheet_core_commands_includes_export_dispatch() {
    let block = cheatsheet_block();
    assert!(
        block.contains("--export-dispatch"),
        "CORE COMMANDS must include --export-dispatch"
    );
}

#[test]
fn operator_cheatsheet_core_commands_includes_artifact_index() {
    let block = cheatsheet_block();
    assert!(
        block.contains("--artifact-index"),
        "CORE COMMANDS must include --artifact-index"
    );
}

#[test]
fn operator_cheatsheet_core_commands_includes_walkthrough() {
    let block = cheatsheet_block();
    assert!(
        block.contains("--walkthrough"),
        "CORE COMMANDS must include --walkthrough"
    );
}

#[test]
fn operator_cheatsheet_core_commands_includes_quickstart() {
    let block = cheatsheet_block();
    assert!(
        block.contains("--quickstart"),
        "CORE COMMANDS must include --quickstart"
    );
}

#[test]
fn operator_cheatsheet_core_commands_includes_verify_sh() {
    let block = cheatsheet_block();
    assert!(
        block.contains("verify.sh"),
        "CORE COMMANDS must include verify.sh"
    );
}

// ── safe review path ───────────────────────────────────────────────────────

#[test]
fn operator_cheatsheet_safe_review_path_starts_with_generate() {
    let block = cheatsheet_block();
    let srp_start = block
        .find("SAFE REVIEW PATH")
        .expect("SAFE REVIEW PATH must exist in the cheatsheet block");
    let srp_block = &block[srp_start..];
    assert!(
        srp_block.contains("Generate pilot bundle"),
        "SAFE REVIEW PATH must begin with 'Generate pilot bundle' step"
    );
}

#[test]
fn operator_cheatsheet_safe_review_path_includes_verify() {
    let block = cheatsheet_block();
    let srp_start = block
        .find("SAFE REVIEW PATH")
        .expect("SAFE REVIEW PATH must exist in the cheatsheet block");
    let srp_block = &block[srp_start..];
    assert!(
        srp_block.contains("verify.sh"),
        "SAFE REVIEW PATH must include a verify.sh step"
    );
}

#[test]
fn operator_cheatsheet_safe_review_path_is_numbered() {
    let block = cheatsheet_block();
    let srp_start = block
        .find("SAFE REVIEW PATH")
        .expect("SAFE REVIEW PATH must exist in the cheatsheet block");
    let srp_block = &block[srp_start..];
    assert!(
        srp_block.contains("1.") && srp_block.contains("2."),
        "SAFE REVIEW PATH must present steps as a numbered sequence"
    );
}

// ── static / deterministic behaviour ─────────────────────────────────────

#[test]
fn operator_cheatsheet_block_has_no_date_command() {
    let block = cheatsheet_block();
    assert!(
        !block.contains("$(date"),
        "operator-cheatsheet block must not embed wall-clock timestamps"
    );
}

#[test]
fn operator_cheatsheet_block_has_no_network_calls() {
    let block = cheatsheet_block();
    assert!(
        !block.contains("curl ") && !block.contains("wget "),
        "operator-cheatsheet block must not make network calls"
    );
}

#[test]
fn operator_cheatsheet_flag_appears_in_help_surface_listing() {
    // Ensure --operator-cheatsheet is documented inside the --help-surface block.
    let help_start = RUN_PILOT_SH
        .find("\"--help-surface\"")
        .expect("--help-surface block must exist");
    let help_block = &RUN_PILOT_SH[help_start..];
    assert!(
        help_block.contains("--operator-cheatsheet"),
        "--operator-cheatsheet must appear in the --help-surface listing"
    );
}

// ── README coverage ────────────────────────────────────────────────────────

#[test]
fn readme_mentions_operator_cheatsheet_command() {
    assert!(
        README.contains("--operator-cheatsheet"),
        "README must mention --operator-cheatsheet"
    );
}

#[test]
fn readme_has_operator_cheatsheet_section() {
    assert!(
        README.contains("## Operator Cheatsheet"),
        "README must have '## Operator Cheatsheet' section"
    );
}

#[test]
fn readme_operator_cheatsheet_describes_sections() {
    assert!(
        README.contains("WHAT THIS SHELL DOES")
            && README.contains("CORE COMMANDS")
            && README.contains("WHEN TO USE THEM")
            && README.contains("SAFE REVIEW PATH"),
        "README Operator Cheatsheet section must describe all four cheatsheet sections"
    );
}

#[test]
fn readme_operator_cheatsheet_notes_static_behavior() {
    // The README must clarify that this is a static/deterministic read-only surface.
    assert!(
        README.contains("static") || README.contains("deterministic"),
        "README Operator Cheatsheet section must note that output is static/deterministic"
    );
}
