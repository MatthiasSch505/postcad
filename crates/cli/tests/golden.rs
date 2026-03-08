use postcad_cli::{route_case_from_json, verify_receipt_from_policy_json};
use serde_json::Value;

fn fixtures_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn scenarios_dir() -> std::path::PathBuf {
    fixtures_dir().join("scenarios")
}

fn read_fixture(name: &str) -> String {
    let path = fixtures_dir().join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read fixture {}: {}", path.display(), e))
}

fn read_scenario_file(scenario: &str, file: &str) -> String {
    let path = scenarios_dir().join(scenario).join(file);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read scenario file {}/{}: {}", scenario, file, e))
}

fn as_json_value(json_str: &str) -> Value {
    serde_json::from_str(json_str).expect("expected valid JSON")
}

/// Runs a scenario through the real routing path and returns the result as a
/// JSON value. Success outputs are serialized directly; errors are converted to
/// a stable `{"code": ..., "message": ...}` object matching the CLI error shape.
fn run_scenario(name: &str) -> Value {
    let case_json = read_scenario_file(name, "case.json");
    let candidates_json = read_scenario_file(name, "candidates.json");
    let snapshot_json = read_scenario_file(name, "snapshot.json");

    match route_case_from_json(&case_json, &candidates_json, &snapshot_json) {
        Ok(output) => serde_json::to_value(&output).unwrap(),
        Err(e) => serde_json::json!({
            "outcome": "error",
            "code": e.code(),
            "message": e.to_string(),
        }),
    }
}

fn assert_scenario(name: &str) {
    let actual = run_scenario(name);
    let expected: Value = as_json_value(&read_scenario_file(name, "expected.json"));
    assert_eq!(actual, expected, "scenario '{}' output mismatch", name);
}

// ── Original golden tests (root fixtures) ─────────────────────────────────────

#[test]
fn golden_routed_output_matches_expected() {
    let output = route_case_from_json(
        &read_fixture("case.json"),
        &read_fixture("candidates.json"),
        &read_fixture("snapshot.json"),
    )
    .expect("routing should succeed");

    let actual: Value = serde_json::to_value(&output).unwrap();
    let expected: Value = as_json_value(&read_fixture("expected_routed.json"));

    assert_eq!(actual, expected);
}

#[test]
fn golden_refused_output_matches_expected() {
    let output = route_case_from_json(
        &read_fixture("case.json"),
        &read_fixture("candidates.json"),
        &read_fixture("snapshot_refusal.json"),
    )
    .expect("parse should succeed");

    let actual: Value = serde_json::to_value(&output).unwrap();
    let expected: Value = as_json_value(&read_fixture("expected_refused.json"));

    assert_eq!(actual, expected);
}

// ── verify-receipt golden tests ───────────────────────────────────────────────

fn verify_scenario(name: &str) {
    let receipt_json = read_scenario_file(name, "expected.json");
    let case_json = read_scenario_file(name, "case.json");
    let policy_json = read_scenario_file(name, "policy.json");

    verify_receipt_from_policy_json(&receipt_json, &case_json, &policy_json)
        .unwrap_or_else(|reason| {
            panic!("verify-receipt for scenario '{}' failed: {}", name, reason)
        });
}

#[test]
fn verify_receipt_routed_domestic_allowed() {
    verify_scenario("routed_domestic_allowed");
}

#[test]
fn verify_receipt_routed_cross_border_allowed() {
    verify_scenario("routed_cross_border_allowed");
}

#[test]
fn verify_receipt_refused_no_eligible_candidates() {
    verify_scenario("refused_no_eligible_candidates");
}

#[test]
fn verify_receipt_refused_compliance_failed() {
    verify_scenario("refused_compliance_failed");
}

// ── Scenario corpus ───────────────────────────────────────────────────────────

#[test]
fn scenario_routed_domestic_allowed() {
    assert_scenario("routed_domestic_allowed");
}

#[test]
fn scenario_refused_no_eligible_candidates() {
    assert_scenario("refused_no_eligible_candidates");
}

#[test]
fn scenario_refused_compliance_failed() {
    assert_scenario("refused_compliance_failed");
}

#[test]
fn scenario_refused_invalid_snapshot() {
    assert_scenario("refused_invalid_snapshot");
}

#[test]
fn scenario_routed_cross_border_allowed() {
    assert_scenario("routed_cross_border_allowed");
}
