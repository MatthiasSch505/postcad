//! Contract matrix tests for the PostCAD public artifact schema.
//!
//! Each test treats `(case.json, policy.json, expected.json)` as an explicit
//! public contract and asserts two guarantees:
//!
//! 1. **Reproducibility** — routing `case.json` against `policy.json` produces
//!    output that is byte-for-byte identical to `expected.json`.
//! 2. **Verifiability** — `verify-receipt` accepts `expected.json` when given
//!    the same `(case.json, policy.json)` pair.
//!
//! These tests additionally assert that the fixture files parse cleanly as the
//! declared public artifact types ([`RoutingPolicyBundle`], [`RoutingReceipt`]),
//! locking in the stable JSON schema.

use postcad_cli::{
    route_case_from_policy_json, verify_receipt_from_policy_json, RoutingPolicyBundle,
    RoutingReceipt,
};
use serde_json::Value;

fn scenarios_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/scenarios")
}

fn read_scenario_file(scenario: &str, file: &str) -> String {
    let path = scenarios_dir().join(scenario).join(file);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}/{}: {}", scenario, file, e))
}

/// Core contract assertion for one scenario.
///
/// Checks:
/// - `policy.json` parses as [`RoutingPolicyBundle`]
/// - `expected.json` parses as [`RoutingReceipt`]
/// - Routing `case.json` + `policy.json` reproduces `expected.json` exactly
/// - `verify-receipt` accepts `expected.json` with `case.json` + `policy.json`
fn assert_contract(scenario: &str) {
    let case_json = read_scenario_file(scenario, "case.json");
    let policy_json = read_scenario_file(scenario, "policy.json");
    let expected_json = read_scenario_file(scenario, "expected.json");

    // Policy bundle parses as the stable public type.
    let _bundle: RoutingPolicyBundle = serde_json::from_str(&policy_json).unwrap_or_else(|e| {
        panic!(
            "[{}] policy.json failed to parse as RoutingPolicyBundle: {}",
            scenario, e
        )
    });

    // Receipt parses as the stable public type.
    let _receipt: RoutingReceipt = serde_json::from_str(&expected_json).unwrap_or_else(|e| {
        panic!(
            "[{}] expected.json failed to parse as RoutingReceipt: {}",
            scenario, e
        )
    });

    // Routing reproduces expected.json exactly.
    let actual = route_case_from_policy_json(&case_json, &policy_json)
        .unwrap_or_else(|e| panic!("[{}] route_case_from_policy_json failed: {}", scenario, e));
    let actual_value: Value = serde_json::to_value(&actual).unwrap();
    let expected_value: Value = serde_json::from_str(&expected_json).unwrap();
    assert_eq!(
        actual_value, expected_value,
        "[{}] routing output does not match expected.json",
        scenario
    );

    // verify-receipt accepts expected.json.
    verify_receipt_from_policy_json(&expected_json, &case_json, &policy_json)
        .unwrap_or_else(|f| panic!("[{}] verify-receipt failed: {}", scenario, f));
}

// ── Contract scenarios ─────────────────────────────────────────────────────────

#[test]
fn contract_routed_domestic_allowed() {
    assert_contract("routed_domestic_allowed");
}

#[test]
fn contract_routed_cross_border_allowed() {
    assert_contract("routed_cross_border_allowed");
}

#[test]
fn contract_refused_no_eligible_candidates() {
    assert_contract("refused_no_eligible_candidates");
}

#[test]
fn contract_refused_compliance_failed() {
    assert_contract("refused_compliance_failed");
}
