//! Pilot input normalization integration tests.
//!
//! Proves that:
//!  1. Normalized pilot input routes successfully end-to-end.
//!  2. Equivalent normalized inputs produce deterministic output.
//!  3. Invalid or missing fields fail with clear, stable error codes.
//!  4. Kernel semantics are unchanged — the pilot path and the direct
//!     `CaseInput` path produce the same receipt hash for the same case.

use postcad_cli::{normalize_pilot_case_json, route_case_from_registry_json};

// ── Shared fixtures ───────────────────────────────────────────────────────────

/// Canonical pilot case in the normalized pilot shape.
const PILOT_CASE_JSON: &str = r#"{
    "case_id": "f1000001-0000-0000-0000-000000000001",
    "restoration_type": "crown",
    "material": "zirconia",
    "jurisdiction": "DE"
}"#;

/// Equivalent case in the raw CaseInput shape — same semantic content as above.
const RAW_CASE_JSON: &str = r#"{
    "case_id": "f1000001-0000-0000-0000-000000000001",
    "jurisdiction": "DE",
    "routing_policy": "allow_domestic_and_cross_border",
    "patient_country": "germany",
    "manufacturer_country": "germany",
    "material": "zirconia",
    "procedure": "crown",
    "file_type": "stl"
}"#;

/// One eligible German manufacturer for DE jurisdiction.
const REGISTRY_JSON: &str = r#"[
  {
    "attestation_statuses": ["verified"],
    "capabilities": ["crown"],
    "country": "germany",
    "display_name": "Pilot GmbH",
    "is_active": true,
    "jurisdictions_served": ["germany"],
    "manufacturer_id": "pilot-de-001",
    "materials_supported": ["zirconia"],
    "sla_days": 5
  }
]"#;

const DE_CONFIG: &str = r#"{"jurisdiction": "DE", "routing_policy": "allow_domestic_and_cross_border"}"#;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn route_via_pilot(pilot_json: &str) -> postcad_cli::RegistryRoutingResult {
    let case_json =
        normalize_pilot_case_json(pilot_json).expect("normalization must succeed");
    route_case_from_registry_json(&case_json, REGISTRY_JSON, DE_CONFIG)
        .expect("routing must succeed")
}

// ── Test 1: normalized input routes successfully ──────────────────────────────

#[test]
fn pilot_input_routes_successfully() {
    let result = route_via_pilot(PILOT_CASE_JSON);
    assert_eq!(
        result.receipt.outcome, "routed",
        "pilot case must route to a manufacturer"
    );
    assert_eq!(
        result.receipt.selected_candidate_id.as_deref(),
        Some("pilot-de-001"),
        "must select the only eligible candidate"
    );
    assert!(
        result.receipt.refusal_code.is_none(),
        "must not have a refusal code on a successful route"
    );
}

// ── Test 2: determinism ───────────────────────────────────────────────────────

#[test]
fn pilot_input_routing_is_deterministic() {
    let r1 = route_via_pilot(PILOT_CASE_JSON);
    let r2 = route_via_pilot(PILOT_CASE_JSON);
    assert_eq!(
        r1.receipt.receipt_hash, r2.receipt.receipt_hash,
        "identical pilot inputs must produce identical receipt hashes"
    );
}

// ── Test 3: kernel semantics unchanged ───────────────────────────────────────

#[test]
fn pilot_path_and_direct_path_produce_same_receipt_hash() {
    // Route via the pilot normalization adapter.
    let case_json_via_pilot =
        normalize_pilot_case_json(PILOT_CASE_JSON).expect("normalization must succeed");
    let via_pilot = route_case_from_registry_json(&case_json_via_pilot, REGISTRY_JSON, DE_CONFIG)
        .expect("routing via pilot path must succeed");

    // Route via the direct CaseInput shape (bypasses normalization).
    let via_direct =
        route_case_from_registry_json(RAW_CASE_JSON, REGISTRY_JSON, DE_CONFIG)
            .expect("routing via direct path must succeed");

    assert_eq!(
        via_pilot.receipt.receipt_hash, via_direct.receipt.receipt_hash,
        "pilot normalization must produce byte-identical receipts to the direct path"
    );
}

// ── Test 4: case-insensitive restoration_type and material ────────────────────

#[test]
fn pilot_input_is_case_insensitive_for_restoration_type_and_material() {
    let mixed_case = r#"{
        "case_id": "f1000001-0000-0000-0000-000000000001",
        "restoration_type": "Crown",
        "material": "Zirconia",
        "jurisdiction": "de"
    }"#;

    let lower = route_via_pilot(PILOT_CASE_JSON);
    let mixed = route_via_pilot(mixed_case);

    assert_eq!(
        lower.receipt.receipt_hash, mixed.receipt.receipt_hash,
        "casing of restoration_type, material, and jurisdiction must not affect receipt hash"
    );
}

// ── Test 5: unknown jurisdiction fails clearly ────────────────────────────────

#[test]
fn unknown_jurisdiction_fails_with_invalid_field_error() {
    let bad_jurisdiction = r#"{
        "restoration_type": "crown",
        "material": "zirconia",
        "jurisdiction": "XX"
    }"#;
    let err = normalize_pilot_case_json(bad_jurisdiction)
        .expect_err("unknown jurisdiction must fail");
    assert_eq!(
        err.code(),
        "parse_error",
        "error code must be stable `parse_error`"
    );
    assert!(
        err.to_string().contains("XX"),
        "error message must identify the bad value"
    );
}

// ── Test 6: missing required field fails clearly ──────────────────────────────

#[test]
fn missing_restoration_type_fails_with_parse_error() {
    let missing_field = r#"{"material": "zirconia", "jurisdiction": "DE"}"#;
    let err = normalize_pilot_case_json(missing_field)
        .expect_err("missing field must fail");
    assert_eq!(err.code(), "parse_error");
}

#[test]
fn missing_material_fails_with_parse_error() {
    let missing_field = r#"{"restoration_type": "crown", "jurisdiction": "DE"}"#;
    let err = normalize_pilot_case_json(missing_field)
        .expect_err("missing field must fail");
    assert_eq!(err.code(), "parse_error");
}

#[test]
fn missing_jurisdiction_fails_with_parse_error() {
    let missing_field = r#"{"restoration_type": "crown", "material": "zirconia"}"#;
    let err = normalize_pilot_case_json(missing_field)
        .expect_err("missing field must fail");
    assert_eq!(err.code(), "parse_error");
}

// ── Test 7: verify round-trip ─────────────────────────────────────────────────

#[test]
fn pilot_routed_receipt_is_independently_verifiable() {
    use postcad_cli::verify_receipt_from_policy_json;

    let case_json =
        normalize_pilot_case_json(PILOT_CASE_JSON).expect("normalization must succeed");
    let result = route_case_from_registry_json(&case_json, REGISTRY_JSON, DE_CONFIG)
        .expect("routing must succeed");

    let receipt_json =
        serde_json::to_string(&result.receipt).expect("receipt must serialize");

    verify_receipt_from_policy_json(&receipt_json, &case_json, &result.derived_policy_json)
        .expect("receipt produced by the pilot path must independently verify");
}
