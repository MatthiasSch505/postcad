//! Lightweight consistency tests for the external demo bundle.
//!
//! These tests confirm that the demo script and case file exist and reference
//! the expected endpoints. They use `include_str!` so a missing file is a
//! compile error, not a runtime failure.

const RUN_DEMO_SH: &str = include_str!("../../../demo/run_demo.sh");
const CASE_DEMO_JSON: &str = include_str!("../../../demo/case_demo.json");
const DEMO_RUN_MD: &str = include_str!("../../../docs/demo_run.md");

/// The demo script must reference all 8 required endpoints.
#[test]
fn demo_script_references_required_endpoints() {
    let required = [
        "/health",
        "/cases",
        "/route",
        "/receipts/",
        "/dispatch/",
        "/dispatch/",
        "/routes",
    ];
    for endpoint in required {
        assert!(
            RUN_DEMO_SH.contains(endpoint),
            "demo/run_demo.sh must reference endpoint '{endpoint}'"
        );
    }
}

/// The demo script must use set -euo pipefail.
#[test]
fn demo_script_has_strict_mode() {
    assert!(
        RUN_DEMO_SH.contains("set -euo pipefail"),
        "demo/run_demo.sh must use 'set -euo pipefail'"
    );
}

/// The demo script must trap EXIT for cleanup.
#[test]
fn demo_script_traps_exit() {
    assert!(
        RUN_DEMO_SH.contains("trap") && RUN_DEMO_SH.contains("EXIT"),
        "demo/run_demo.sh must trap EXIT for cleanup"
    );
}

/// The demo case must be valid JSON with required fields.
#[test]
fn demo_case_is_valid_json_with_required_fields() {
    let v: serde_json::Value =
        serde_json::from_str(CASE_DEMO_JSON).expect("demo/case_demo.json must be valid JSON");
    for field in [
        "case_id",
        "jurisdiction",
        "material",
        "procedure",
        "file_type",
    ] {
        assert!(
            !v[field].is_null(),
            "demo/case_demo.json must contain field '{field}'"
        );
    }
}

/// The demo case must use a distinct case_id (not the pilot fixture).
#[test]
fn demo_case_uses_distinct_case_id() {
    let v: serde_json::Value = serde_json::from_str(CASE_DEMO_JSON).unwrap();
    let case_id = v["case_id"].as_str().expect("case_id must be a string");
    assert!(
        !case_id.starts_with("f1000001"),
        "demo case_id must be distinct from the pilot fixture case_id (f1000001-…)"
    );
}

/// The demo run doc must reference the single command.
#[test]
fn demo_run_doc_references_run_command() {
    assert!(
        DEMO_RUN_MD.contains("run_demo.sh"),
        "docs/demo_run.md must reference ./demo/run_demo.sh"
    );
}
