//! Default inbound path resolution tests.
//!
//! Checks that run_pilot.sh --inspect-inbound-reply and verify.sh --bundle
//! auto-resolve the inbound reply path from receipt.json when no explicit
//! file argument is given.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const VERIFY_SH: &str = include_str!("../../../examples/pilot/verify.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── run_pilot.sh: auto-resolution variables ───────────────────────────────────

#[test]
fn run_pilot_auto_resolve_uses_ar_case_id() {
    assert!(
        RUN_PILOT_SH.contains("_AR_CASE_ID"),
        "run_pilot.sh must use _AR_CASE_ID variable for auto-resolution"
    );
}

#[test]
fn run_pilot_auto_resolve_uses_ar_receipt_hash() {
    assert!(
        RUN_PILOT_SH.contains("_AR_RECEIPT_HASH"),
        "run_pilot.sh must use _AR_RECEIPT_HASH variable for auto-resolution"
    );
}

#[test]
fn run_pilot_auto_resolve_uses_ar_run_id() {
    assert!(
        RUN_PILOT_SH.contains("_AR_RUN_ID"),
        "run_pilot.sh must use _AR_RUN_ID variable for auto-resolution"
    );
}

#[test]
fn run_pilot_auto_resolve_uses_ar_candidate() {
    assert!(
        RUN_PILOT_SH.contains("_AR_CANDIDATE"),
        "run_pilot.sh must use _AR_CANDIDATE variable to build expected inbound path"
    );
}

// ── run_pilot.sh: auto-resolution path pattern ────────────────────────────────

#[test]
fn run_pilot_auto_resolve_builds_lab_reply_path() {
    assert!(
        RUN_PILOT_SH.contains("inbound/lab_reply_"),
        "run_pilot.sh must build inbound/lab_reply_<run-id>.json path for auto-resolution"
    );
}

#[test]
fn run_pilot_auto_resolve_prints_resolved_path() {
    assert!(
        RUN_PILOT_SH.contains("auto-resolved inbound reply"),
        "run_pilot.sh must print 'auto-resolved inbound reply' when path is resolved"
    );
}

// ── run_pilot.sh: not-found guidance ──────────────────────────────────────────

#[test]
fn run_pilot_auto_resolve_prints_not_found_header() {
    assert!(
        RUN_PILOT_SH.contains("INBOUND REPLY NOT FOUND"),
        "run_pilot.sh must print 'INBOUND REPLY NOT FOUND' when auto-resolution fails"
    );
}

#[test]
fn run_pilot_auto_resolve_prints_current_run_label() {
    assert!(
        RUN_PILOT_SH.contains("Current run :"),
        "run_pilot.sh must print 'Current run :' label in not-found guidance"
    );
}

#[test]
fn run_pilot_auto_resolve_prints_expected_path_label() {
    assert!(
        RUN_PILOT_SH.contains("Expected    :"),
        "run_pilot.sh must print 'Expected    :' label in not-found guidance"
    );
}

#[test]
fn run_pilot_auto_resolve_not_found_suggests_simulate_inbound() {
    assert!(
        RUN_PILOT_SH.contains("--simulate-inbound"),
        "run_pilot.sh not-found guidance must suggest --simulate-inbound"
    );
}

// ── verify.sh: auto-resolution variables ─────────────────────────────────────

#[test]
fn verify_sh_auto_resolve_uses_bundle_explicit_flag() {
    assert!(
        VERIFY_SH.contains("BUNDLE_EXPLICIT"),
        "verify.sh must use BUNDLE_EXPLICIT flag to detect user-provided --bundle"
    );
}

#[test]
fn verify_sh_auto_resolve_uses_vr_candidate() {
    assert!(
        VERIFY_SH.contains("_VR_CANDIDATE"),
        "verify.sh must use _VR_CANDIDATE variable to build expected inbound path"
    );
}

#[test]
fn verify_sh_auto_resolve_prints_resolved_path() {
    assert!(
        VERIFY_SH.contains("auto-resolved inbound reply"),
        "verify.sh must print 'auto-resolved inbound reply' when path is resolved"
    );
}

#[test]
fn verify_sh_auto_resolve_prints_not_found_header() {
    assert!(
        VERIFY_SH.contains("INBOUND REPLY NOT FOUND"),
        "verify.sh must print 'INBOUND REPLY NOT FOUND' when auto-resolution fails"
    );
}

#[test]
fn verify_sh_auto_resolve_not_found_suggests_simulate_inbound() {
    assert!(
        VERIFY_SH.contains("--simulate-inbound"),
        "verify.sh not-found guidance must suggest --simulate-inbound"
    );
}

#[test]
fn verify_sh_bundle_explicit_set_true_on_bundle_flag() {
    assert!(
        VERIFY_SH.contains("BUNDLE_EXPLICIT=true"),
        "verify.sh must set BUNDLE_EXPLICIT=true when --bundle argument is parsed"
    );
}

// ── README ────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_default_inbound_path_resolution_section() {
    assert!(
        README.contains("## Default Inbound Path Resolution"),
        "README must have '## Default Inbound Path Resolution' section"
    );
}

#[test]
fn readme_default_path_section_mentions_inspect_inbound_reply() {
    assert!(
        README.contains("--inspect-inbound-reply"),
        "README Default Inbound Path Resolution section must mention --inspect-inbound-reply"
    );
}

#[test]
fn readme_default_path_section_mentions_bundle_flag() {
    let section_start = README
        .find("## Default Inbound Path Resolution")
        .expect("section must exist");
    let section_end = README[section_start..]
        .find("\n## ")
        .map(|i| section_start + i)
        .unwrap_or(README.len());
    let section = &README[section_start..section_end];
    assert!(
        section.contains("--bundle"),
        "Default Inbound Path Resolution section must mention --bundle flag for verify.sh"
    );
}

#[test]
fn readme_default_path_section_shows_not_found_message() {
    assert!(
        README.contains("INBOUND REPLY NOT FOUND"),
        "README must show the INBOUND REPLY NOT FOUND message in the default path section"
    );
}
