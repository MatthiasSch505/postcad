//! Lightweight consistency tests for the pilot maturity docs.
//!
//! These tests confirm that the maturity check and gap list files exist and
//! contain the fields expected by the assessment. They do not validate prose;
//! they verify the machine-readable parts are structurally coherent.
//!
//! Using `include_str!` means a missing file is a **compile error**, not a
//! runtime failure — matching the repo's pattern for frozen fixtures.

use serde_json::Value;

const MATURITY_CHECK_MD: &str = include_str!("../../../docs/pilot_maturity_check.md");
const GAP_LIST_JSON: &str = include_str!("../../../docs/pilot_gap_list.json");

/// The maturity check markdown must contain all required section headings.
#[test]
fn maturity_check_contains_required_sections() {
    let required = [
        "Purpose",
        "Current Implemented Surfaces",
        "Pass Conditions",
        "Remaining Gaps",
        "Explicit Non-Gaps",
        "Readiness Verdict",
        "Next Locked Phase",
    ];
    for heading in required {
        assert!(
            MATURITY_CHECK_MD.contains(heading),
            "pilot_maturity_check.md must contain section '{heading}'"
        );
    }
}

/// The gap list must be valid JSON with the required top-level fields.
#[test]
fn gap_list_is_valid_json_with_required_fields() {
    let v: Value =
        serde_json::from_str(GAP_LIST_JSON).expect("docs/pilot_gap_list.json must be valid JSON");

    for field in [
        "phase",
        "verdict",
        "completed_surfaces",
        "remaining_gaps",
        "next_phase",
    ] {
        assert!(
            !v[field].is_null(),
            "pilot_gap_list.json must contain field '{field}'"
        );
    }
}

/// The gap list verdict must be one of the four allowed values.
#[test]
fn gap_list_verdict_is_allowed_value() {
    let v: Value = serde_json::from_str(GAP_LIST_JSON).unwrap();
    let verdict = v["verdict"].as_str().expect("verdict must be a string");
    let allowed = [
        "not ready",
        "internally pilotable",
        "externally demoable",
        "pilot-ready with supervision",
    ];
    assert!(
        allowed.contains(&verdict),
        "verdict '{verdict}' is not one of the allowed values: {allowed:?}"
    );
}

/// The maturity check must reference the verdict it uses so the markdown and
/// JSON are consistent.
#[test]
fn maturity_check_verdict_matches_gap_list() {
    let v: Value = serde_json::from_str(GAP_LIST_JSON).unwrap();
    let verdict = v["verdict"].as_str().unwrap();
    assert!(
        MATURITY_CHECK_MD.contains(verdict),
        "pilot_maturity_check.md must contain the same verdict string as pilot_gap_list.json ('{verdict}')"
    );
}

/// The gap list must list at least the core completed surfaces.
#[test]
fn gap_list_completed_surfaces_covers_core() {
    let v: Value = serde_json::from_str(GAP_LIST_JSON).unwrap();
    let surfaces: Vec<&str> = v["completed_surfaces"]
        .as_array()
        .expect("completed_surfaces must be an array")
        .iter()
        .filter_map(|s| s.as_str())
        .collect();

    for required in [
        "routing_kernel",
        "verification_path",
        "http_service",
        "dispatch",
        "dispatch_verification_gate",
        "operator_ui",
        "acceptance_runner",
    ] {
        assert!(
            surfaces.contains(&required),
            "completed_surfaces must include '{required}'"
        );
    }
}
