//! Operator walkthrough mode tests.
//!
//! Checks that run_pilot.sh --walkthrough exists, prints a deterministic
//! 4-step guide, and never invokes any runtime commands.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_walkthrough_flag() {
    assert!(
        RUN_PILOT_SH.contains("--walkthrough"),
        "run_pilot.sh must support --walkthrough flag"
    );
}

#[test]
fn walkthrough_mode_exits_after_printing() {
    assert!(
        RUN_PILOT_SH.contains("--walkthrough") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --walkthrough must exit 0 after printing"
    );
}

// ── header ────────────────────────────────────────────────────────────────────

#[test]
fn walkthrough_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD PILOT WALKTHROUGH"),
        "run_pilot.sh must print 'POSTCAD PILOT WALKTHROUGH' header"
    );
}

// ── step 1 ────────────────────────────────────────────────────────────────────

#[test]
fn walkthrough_has_step_1() {
    assert!(
        RUN_PILOT_SH.contains("Step 1"),
        "run_pilot.sh walkthrough must include Step 1"
    );
}

#[test]
fn walkthrough_step1_title_generate_bundle() {
    assert!(
        RUN_PILOT_SH.contains("Generate pilot bundle"),
        "run_pilot.sh walkthrough Step 1 must be titled 'Generate pilot bundle'"
    );
}

#[test]
fn walkthrough_step1_shows_run_pilot_command() {
    assert!(
        RUN_PILOT_SH.contains("./examples/pilot/run_pilot.sh"),
        "run_pilot.sh walkthrough Step 1 must show run_pilot.sh command"
    );
}

#[test]
fn walkthrough_step1_mentions_receipt_artifact() {
    assert!(
        RUN_PILOT_SH.contains("receipt.json"),
        "run_pilot.sh walkthrough Step 1 must mention receipt.json artifact"
    );
}

// ── step 2 ────────────────────────────────────────────────────────────────────

#[test]
fn walkthrough_has_step_2() {
    assert!(
        RUN_PILOT_SH.contains("Step 2"),
        "run_pilot.sh walkthrough must include Step 2"
    );
}

#[test]
fn walkthrough_step2_title_inspect_reply() {
    assert!(
        RUN_PILOT_SH.contains("Inspect inbound lab reply"),
        "run_pilot.sh walkthrough Step 2 must be titled 'Inspect inbound lab reply'"
    );
}

#[test]
fn walkthrough_step2_shows_inspect_command() {
    assert!(
        RUN_PILOT_SH.contains("--inspect-inbound-reply"),
        "run_pilot.sh walkthrough Step 2 must show --inspect-inbound-reply command"
    );
}

// ── step 3 ────────────────────────────────────────────────────────────────────

#[test]
fn walkthrough_has_step_3() {
    assert!(
        RUN_PILOT_SH.contains("Step 3"),
        "run_pilot.sh walkthrough must include Step 3"
    );
}

#[test]
fn walkthrough_step3_title_verify_reply() {
    assert!(
        RUN_PILOT_SH.contains("Verify inbound reply"),
        "run_pilot.sh walkthrough Step 3 must be titled 'Verify inbound reply'"
    );
}

#[test]
fn walkthrough_step3_shows_verify_sh_command() {
    assert!(
        RUN_PILOT_SH.contains("./examples/pilot/verify.sh --inbound"),
        "run_pilot.sh walkthrough Step 3 must show verify.sh --inbound command"
    );
}

#[test]
fn walkthrough_step3_mentions_verification_passed_failed() {
    assert!(
        RUN_PILOT_SH.contains("VERIFICATION PASSED / VERIFICATION FAILED"),
        "run_pilot.sh walkthrough Step 3 must mention VERIFICATION PASSED / VERIFICATION FAILED"
    );
}

// ── step 4 ────────────────────────────────────────────────────────────────────

#[test]
fn walkthrough_has_step_4() {
    assert!(
        RUN_PILOT_SH.contains("Step 4"),
        "run_pilot.sh walkthrough must include Step 4"
    );
}

#[test]
fn walkthrough_step4_title_export_dispatch() {
    assert!(
        RUN_PILOT_SH.contains("Export dispatch packet"),
        "run_pilot.sh walkthrough Step 4 must be titled 'Export dispatch packet'"
    );
}

#[test]
fn walkthrough_step4_shows_export_command() {
    assert!(
        RUN_PILOT_SH.contains("--export-lab-trial-package"),
        "run_pilot.sh walkthrough Step 4 must show --export-lab-trial-package command"
    );
}

// ── deterministic plain text — no runtime execution ──────────────────────────

#[test]
fn walkthrough_does_not_call_cargo() {
    // The walkthrough block must only echo — never invoke cargo or other scripts
    let walkthrough_start = RUN_PILOT_SH
        .find("--walkthrough")
        .expect("--walkthrough must exist");
    let walkthrough_end = RUN_PILOT_SH[walkthrough_start..]
        .find("exit 0")
        .map(|i| walkthrough_start + i + 6)
        .unwrap_or(walkthrough_start + 200);
    let block = &RUN_PILOT_SH[walkthrough_start..walkthrough_end];
    assert!(
        !block.contains("cargo "),
        "walkthrough block must not invoke cargo"
    );
}

// ── README section ────────────────────────────────────────────────────────────

#[test]
fn readme_has_pilot_walkthrough_section() {
    assert!(
        README.contains("## Pilot Walkthrough"),
        "README must have '## Pilot Walkthrough' section"
    );
}

#[test]
fn readme_shows_walkthrough_command() {
    assert!(
        README.contains("--walkthrough"),
        "README must show --walkthrough command"
    );
}

#[test]
fn readme_walkthrough_shows_all_four_steps() {
    for step in ["Step 1", "Step 2", "Step 3", "Step 4"] {
        assert!(
            README.contains(step),
            "README Pilot Walkthrough section must show {step}"
        );
    }
}

#[test]
fn readme_walkthrough_explains_no_commands_executed() {
    assert!(
        README.contains("no commands are executed") || README.contains("no files are written"),
        "README must clarify that walkthrough only prints — no commands executed"
    );
}
