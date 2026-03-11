use postcad_cli::{
    build_manifest, route_case_from_json, route_case_from_policy_json,
    verify_receipt_from_policy_json,
};
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

    verify_receipt_from_policy_json(&receipt_json, &case_json, &policy_json).unwrap_or_else(
        |reason| panic!("verify-receipt for scenario '{}' failed: {}", name, reason),
    );
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

// ── End-to-end golden demo artifact tests ─────────────────────────────────────
//
// These three tests lock the canonical demo fixture set
// (fixtures/case.json + fixtures/policy.json + fixtures/expected_routed.json)
// as a frozen end-to-end contract.

/// Golden route-case test: policy-form path produces the exact frozen receipt.
///
/// Uses route_case_from_policy_json (the 2-artifact public API), exercising the
/// same code path as the `contract_*` scenario tests. Assert is byte-for-byte
/// (via JSON value equality) against fixtures/expected_routed.json.
#[test]
fn golden_demo_route_case_policy_form_matches_frozen_receipt() {
    let actual =
        route_case_from_policy_json(&read_fixture("case.json"), &read_fixture("policy.json"))
            .expect("routing should succeed");

    let actual_value: Value = serde_json::to_value(&actual).unwrap();
    let expected_value: Value = as_json_value(&read_fixture("expected_routed.json"));

    assert_eq!(
        actual_value, expected_value,
        "policy-form route-case output does not match frozen expected_routed.json"
    );
}

/// Golden verify-receipt test: verify-receipt accepts the frozen receipt.
///
/// Proves that verify_receipt_from_policy_json succeeds on the exact artifact
/// stored in expected_routed.json when given the canonical demo inputs.
#[test]
fn golden_demo_verify_receipt_accepts_frozen_receipt() {
    verify_receipt_from_policy_json(
        &read_fixture("expected_routed.json"),
        &read_fixture("case.json"),
        &read_fixture("policy.json"),
    )
    .expect("verify-receipt should accept the frozen demo receipt");
}

/// Drift-detection test: a real semantic change in the registry snapshot causes
/// deterministic verification failure with `registry_snapshot_hash_mismatch`.
///
/// The snapshot for mfr-de-01 is changed from eligible+verified to
/// ineligible+rejected. The frozen receipt's registry_snapshot_hash was
/// committed against the original data, so verification must reject the
/// tampered policy.
#[test]
fn golden_demo_drift_detection_snapshot_change_fails_verify() {
    // Registry snapshot with a different semantic value: manufacturer is now
    // ineligible with a rejected attestation instead of verified+eligible.
    let drifted_policy = r#"{
        "jurisdiction": "DE",
        "routing_policy": "allow_domestic_and_cross_border",
        "candidates": [
            {
                "id": "rc-de-01",
                "manufacturer_id": "mfr-de-01",
                "location": "domestic",
                "accepts_case": true,
                "eligibility": "eligible"
            }
        ],
        "snapshots": [
            {
                "manufacturer_id": "mfr-de-01",
                "evidence_references": ["ISO-9001-2024"],
                "attestation_statuses": ["rejected"],
                "is_eligible": false
            }
        ]
    }"#;

    let err = verify_receipt_from_policy_json(
        &read_fixture("expected_routed.json"),
        &read_fixture("case.json"),
        drifted_policy,
    )
    .expect_err("verify-receipt must fail when registry snapshot has drifted");

    assert_eq!(
        err.code, "registry_snapshot_hash_mismatch",
        "expected registry_snapshot_hash_mismatch, got: {:?}",
        err
    );
}

// ── Protocol manifest golden test ─────────────────────────────────────────────

/// `build_manifest()` must produce a value that is byte-for-byte (via JSON
/// value equality) identical to `fixtures/expected_manifest.json`.
///
/// Any change to the manifest — a new field, a renamed error code, a version
/// bump — requires regenerating the fixture and is therefore immediately
/// visible in version control.
#[test]
fn golden_protocol_manifest_matches_frozen_fixture() {
    let actual: Value =
        serde_json::to_value(build_manifest()).expect("build_manifest must serialise to JSON");
    let expected: Value = as_json_value(&read_fixture("expected_manifest.json"));
    assert_eq!(
        actual, expected,
        "protocol manifest does not match fixtures/expected_manifest.json"
    );
}
