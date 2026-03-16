//! Pilot artifact index surface tests.
//!
//! Verifies that run_pilot.sh --artifact-index has the required RUN, ARTIFACTS,
//! MISSING, and NEXT sections; uses [exists]/[missing] markers; fails cleanly
//! when no pilot outputs exist; and that the README documents the command.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

/// Find the artifact-index block as a &str from the header to the end of the
/// block (the next top-level mode comment or end of file). Uses only ASCII
/// search anchors so the slice boundaries are always valid char boundaries.
fn artifact_index_block() -> &'static str {
    // Anchor on ASCII-safe start of the if-block
    let start = RUN_PILOT_SH
        .find("\"--artifact-index\"")
        .expect("--artifact-index if-block must exist in run_pilot.sh");
    // End at the start of the next top-level mode section (starts with "\nif [[")
    // after our block's own "exit 0\nfi"
    let after = &RUN_PILOT_SH[start..];
    // Find "exit 0\nfi\n" which closes the block, then take a bit past it
    let relative_end = after
        .find("exit 0\nfi\n")
        .map(|i| i + "exit 0\nfi\n".len())
        .unwrap_or(after.len());
    &RUN_PILOT_SH[start..start + relative_end]
}

// ── command in help text ───────────────────────────────────────────────────────

#[test]
fn artifact_index_appears_in_help_surface() {
    let help_start = RUN_PILOT_SH
        .find("--help-surface")
        .expect("--help-surface must exist in run_pilot.sh");
    let tail = &RUN_PILOT_SH[help_start..];
    assert!(
        tail.contains("--artifact-index"),
        "--artifact-index must appear in help surface output"
    );
}

#[test]
fn artifact_index_flag_present_in_script() {
    assert!(
        RUN_PILOT_SH.contains("--artifact-index"),
        "--artifact-index flag must be present in run_pilot.sh"
    );
}

// ── RUN section ───────────────────────────────────────────────────────────────

#[test]
fn artifact_index_has_run_section() {
    let block = artifact_index_block();
    assert!(
        block.contains("echo \"RUN\""),
        "--artifact-index must print 'RUN' section"
    );
}

#[test]
fn artifact_index_run_section_shows_run_id() {
    let block = artifact_index_block();
    assert!(block.contains("Run ID"), "RUN section must show 'Run ID'");
}

#[test]
fn artifact_index_run_section_shows_case_id() {
    let block = artifact_index_block();
    assert!(block.contains("Case ID"), "RUN section must show 'Case ID'");
}

#[test]
fn artifact_index_run_section_shows_root_dir() {
    let block = artifact_index_block();
    assert!(block.contains("Root dir"), "RUN section must show 'Root dir'");
}

// ── ARTIFACTS section ─────────────────────────────────────────────────────────

#[test]
fn artifact_index_has_artifacts_section() {
    let block = artifact_index_block();
    assert!(
        block.contains("echo \"ARTIFACTS\""),
        "--artifact-index must print 'ARTIFACTS' section"
    );
}

#[test]
fn artifact_index_artifacts_uses_exists_marker() {
    let block = artifact_index_block();
    assert!(
        block.contains("[exists]"),
        "ARTIFACTS section must use '[exists]' marker for present artifacts"
    );
}

#[test]
fn artifact_index_artifacts_uses_missing_marker() {
    let block = artifact_index_block();
    assert!(
        block.contains("[missing]"),
        "ARTIFACTS section must use '[missing]' marker for absent artifacts"
    );
}

#[test]
fn artifact_index_artifacts_shows_input_case() {
    let block = artifact_index_block();
    assert!(
        block.contains("input case"),
        "ARTIFACTS section must list 'input case' artifact"
    );
}

#[test]
fn artifact_index_artifacts_shows_registry_snapshot() {
    let block = artifact_index_block();
    assert!(
        block.contains("registry snapshot"),
        "ARTIFACTS section must list 'registry snapshot' artifact"
    );
}

#[test]
fn artifact_index_artifacts_shows_routing_decision() {
    let block = artifact_index_block();
    assert!(
        block.contains("routing decision"),
        "ARTIFACTS section must list 'routing decision' artifact"
    );
}

#[test]
fn artifact_index_artifacts_shows_dispatch_artifact() {
    let block = artifact_index_block();
    assert!(
        block.contains("dispatch artifact"),
        "ARTIFACTS section must list 'dispatch artifact'"
    );
}

#[test]
fn artifact_index_artifacts_shows_outbound_package() {
    let block = artifact_index_block();
    assert!(
        block.contains("outbound package"),
        "ARTIFACTS section must list 'outbound package' artifact"
    );
}

#[test]
fn artifact_index_artifacts_shows_lab_reply() {
    let block = artifact_index_block();
    assert!(
        block.contains("lab reply"),
        "ARTIFACTS section must list 'lab reply' artifact"
    );
}

#[test]
fn artifact_index_artifacts_shows_audit_receipt() {
    let block = artifact_index_block();
    assert!(
        block.contains("audit receipt"),
        "ARTIFACTS section must list 'audit receipt' artifact"
    );
}

// ── MISSING section ───────────────────────────────────────────────────────────

#[test]
fn artifact_index_has_missing_section() {
    let block = artifact_index_block();
    assert!(
        block.contains("echo \"MISSING\""),
        "--artifact-index must print 'MISSING' section"
    );
}

#[test]
fn artifact_index_missing_section_prints_none_when_complete() {
    let block = artifact_index_block();
    // The MISSING block must have a "none" path for when all artifacts are present
    let missing_start = block
        .find("echo \"MISSING\"")
        .expect("MISSING section must exist");
    let missing_area = &block[missing_start..missing_start + 1000];
    assert!(
        missing_area.contains("none"),
        "MISSING section must print 'none' when all expected artifacts are present"
    );
}

#[test]
fn artifact_index_missing_section_lists_missing_artifacts() {
    let block = artifact_index_block();
    let missing_start = block
        .find("echo \"MISSING\"")
        .expect("MISSING section must exist");
    let missing_area = &block[missing_start..missing_start + 600];
    assert!(
        missing_area.contains("export_packet.json") || missing_area.contains("case.json"),
        "MISSING section must list individual artifact names for missing items"
    );
}

// ── NEXT section ──────────────────────────────────────────────────────────────

#[test]
fn artifact_index_has_next_section() {
    let block = artifact_index_block();
    assert!(
        block.contains("echo \"NEXT\""),
        "--artifact-index must print 'NEXT' section"
    );
}

#[test]
fn artifact_index_next_section_shows_inspection_commands() {
    let block = artifact_index_block();
    let next_start = block.find("echo \"NEXT\"").expect("NEXT section must exist");
    let next_area = &block[next_start..next_start + 300];
    assert!(
        next_area.contains("--run-summary")
            || next_area.contains("--operator-inbox")
            || next_area.contains("--timeline"),
        "NEXT section must show pilot inspection commands"
    );
}

// ── stable section ordering ───────────────────────────────────────────────────

#[test]
fn artifact_index_run_before_artifacts() {
    let block = artifact_index_block();
    let run_pos = block.find("echo \"RUN\"").expect("RUN section must exist");
    let art_pos = block
        .find("echo \"ARTIFACTS\"")
        .expect("ARTIFACTS section must exist");
    assert!(run_pos < art_pos, "RUN must appear before ARTIFACTS");
}

#[test]
fn artifact_index_artifacts_before_missing() {
    let block = artifact_index_block();
    let art_pos = block
        .find("echo \"ARTIFACTS\"")
        .expect("ARTIFACTS section must exist");
    let miss_pos = block
        .find("echo \"MISSING\"")
        .expect("MISSING section must exist");
    assert!(art_pos < miss_pos, "ARTIFACTS must appear before MISSING");
}

#[test]
fn artifact_index_missing_before_next() {
    let block = artifact_index_block();
    let miss_pos = block
        .find("echo \"MISSING\"")
        .expect("MISSING section must exist");
    let next_pos = block.find("echo \"NEXT\"").expect("NEXT section must exist");
    assert!(miss_pos < next_pos, "MISSING must appear before NEXT");
}

// ── clean failure when pilot outputs missing ──────────────────────────────────

#[test]
fn artifact_index_fails_cleanly_when_no_receipt() {
    let block = artifact_index_block();
    assert!(
        block.contains("no pilot run artifacts found"),
        "--artifact-index must print 'no pilot run artifacts found' when receipt.json is absent"
    );
}

#[test]
fn artifact_index_failure_exits_nonzero() {
    let block = artifact_index_block();
    let err_start = block
        .find("no pilot run artifacts found")
        .expect("no-artifacts error message must exist");
    let err_area = &block[err_start..err_start + 200];
    assert!(
        err_area.contains("exit 1"),
        "clean failure path must exit 1"
    );
}

#[test]
fn artifact_index_failure_prints_to_stderr() {
    let block = artifact_index_block();
    let err_start = block
        .find("no pilot run artifacts found")
        .expect("no-artifacts error message must exist");
    let err_area = &block[err_start..err_start + 200];
    assert!(
        err_area.contains(">&2"),
        "clean failure message must be printed to stderr"
    );
}

#[test]
fn artifact_index_failure_tells_operator_to_run_pilot() {
    let block = artifact_index_block();
    let err_start = block
        .find("no pilot run artifacts found")
        .expect("no-artifacts error message must exist");
    let err_area = &block[err_start..err_start + 300];
    assert!(
        err_area.contains("run_pilot.sh"),
        "clean failure must direct operator to run run_pilot.sh"
    );
}

// ── missing optional artifacts handled cleanly ────────────────────────────────

#[test]
fn artifact_index_missing_optional_uses_presence_checks() {
    let block = artifact_index_block();
    assert!(
        block.contains("AI_OUTBOUND_PRESENT") && block.contains("AI_INBOUND_PRESENT"),
        "optional artifact presence must be checked before printing [exists]/[missing]"
    );
}

#[test]
fn artifact_index_does_not_crash_on_missing_run_id() {
    let block = artifact_index_block();
    assert!(
        block.contains("<run-id>"),
        "script must use pattern paths like lab_reply_<run-id>.json when run id is unavailable"
    );
}

// ── determinism ───────────────────────────────────────────────────────────────

#[test]
fn artifact_index_surface_block_has_no_date_command() {
    let block = artifact_index_block();
    assert!(
        !block.contains("$(date"),
        "--artifact-index block must not embed timestamps via $(date"
    );
}

#[test]
fn artifact_index_surface_block_is_readonly() {
    let block = artifact_index_block();
    assert!(
        !block.contains("> \"${SCRIPT_DIR}"),
        "--artifact-index block must not write files"
    );
}

// ── README ────────────────────────────────────────────────────────────────────

#[test]
fn readme_artifact_index_section_exists() {
    assert!(
        README.contains("## Artifact Index"),
        "README must have '## Artifact Index' section"
    );
}

#[test]
fn readme_artifact_index_shows_command() {
    assert!(
        README.contains("--artifact-index"),
        "README must show '--artifact-index' command"
    );
}

#[test]
fn readme_artifact_index_mentions_exists_missing_markers() {
    let section_start = README
        .find("## Artifact Index")
        .expect("'## Artifact Index' section must exist");
    let section = &README[section_start..section_start + 1000];
    assert!(
        section.contains("[exists]") || section.contains("exists") || section.contains("missing"),
        "README Artifact Index section must describe [exists]/[missing] status markers"
    );
}
