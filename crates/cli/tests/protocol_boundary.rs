//! Protocol boundary assertion tests.
//!
//! Verifies that the external protocol surface (receipts, proofs, manifests)
//! carries the required version fields and that verification correctly rejects
//! artifacts missing those fields.

use postcad_cli::{
    build_manifest, build_routing_proof, route_case_from_policy_json,
    verify_receipt_from_policy_json, POSTCAD_PROTOCOL_VERSION, PROTOCOL_VERSION,
    ROUTING_KERNEL_SEMVER,
};
use postcad_routing::ROUTING_KERNEL_VERSION;
use serde_json::Value;

// ── Frozen v1 demo fixtures ───────────────────────────────────────────────────

const CASE_JSON: &str = include_str!("../../../fixtures/case.json");
const POLICY_JSON: &str = include_str!("../../../fixtures/policy.json");

// ── Receipt boundary assertions ───────────────────────────────────────────────

/// Every routing receipt must carry routing_kernel_version.
#[test]
fn receipt_contains_kernel_version() {
    let receipt = route_case_from_policy_json(CASE_JSON, POLICY_JSON).expect("fixture must route");

    assert!(
        !receipt.routing_kernel_version.is_empty(),
        "routing_kernel_version must not be empty"
    );
    assert_eq!(
        receipt.routing_kernel_version, ROUTING_KERNEL_VERSION,
        "routing_kernel_version must equal the current kernel label"
    );
}

/// The proof object derived from a receipt must carry protocol_version.
#[test]
fn receipt_contains_protocol_version() {
    let receipt = route_case_from_policy_json(CASE_JSON, POLICY_JSON).expect("fixture must route");
    let proof = build_routing_proof(&receipt);

    assert_eq!(
        proof.protocol_version, PROTOCOL_VERSION,
        "proof protocol_version must equal the manifest PROTOCOL_VERSION label"
    );
    assert!(
        !proof.protocol_version.is_empty(),
        "proof protocol_version must not be empty"
    );
}

// ── Manifest boundary assertions ──────────────────────────────────────────────

/// Manifest must carry protocol_version.
#[test]
fn manifest_contains_protocol_version() {
    let m = build_manifest();
    assert_eq!(m.protocol_version, PROTOCOL_VERSION);
    assert!(!m.protocol_version.is_empty());
}

/// Manifest must carry routing_kernel_version.
#[test]
fn manifest_contains_kernel_version() {
    let m = build_manifest();
    assert_eq!(m.routing_kernel_version, ROUTING_KERNEL_VERSION);
    assert!(!m.routing_kernel_version.is_empty());
}

// ── Manifest completeness (STEP 4) ────────────────────────────────────────────

/// Protocol manifest must include all required freeze fields.
#[test]
fn protocol_manifest_complete() {
    let m = build_manifest();
    let v: Value = serde_json::to_value(&m).expect("manifest must serialize");

    let required_fields = [
        "protocol_version",
        "routing_kernel_version",
        "receipt_schema_hash",
        "proof_schema_hash",
        "refusal_code_set_hash",
        "manifest_fingerprint",
        "stable_error_codes",
    ];
    for field in &required_fields {
        assert!(
            !v[field].is_null(),
            "manifest must contain field {:?}",
            field
        );
    }

    // Verify the hash fields are 64-char hex digests.
    for hash_field in &[
        "receipt_schema_hash",
        "proof_schema_hash",
        "refusal_code_set_hash",
        "manifest_fingerprint",
    ] {
        let h = v[hash_field].as_str().unwrap_or("");
        assert_eq!(
            h.len(),
            64,
            "manifest field {:?} must be a 64-char hex digest, got {:?}",
            hash_field,
            h
        );
    }

    // stable_error_codes must be a non-empty array.
    assert!(
        v["stable_error_codes"]
            .as_array()
            .map_or(false, |a| !a.is_empty()),
        "stable_error_codes must be a non-empty array"
    );
}

// ── Verification rejects missing fields ──────────────────────────────────────

/// Removing routing_kernel_version from a receipt must cause verification to
/// fail (the kernel version field is required for replay verification).
#[test]
fn verification_rejects_missing_kernel_version() {
    let receipt = route_case_from_policy_json(CASE_JSON, POLICY_JSON).expect("fixture must route");
    let receipt_json = serde_json::to_string(&receipt).unwrap();

    let mut v: Value = serde_json::from_str(&receipt_json).unwrap();
    v.as_object_mut().unwrap().remove("routing_kernel_version");
    let broken_json = serde_json::to_string(&v).unwrap();

    let err = verify_receipt_from_policy_json(&broken_json, CASE_JSON, POLICY_JSON)
        .expect_err("receipt without routing_kernel_version must fail");
    assert!(
        !err.code.is_empty(),
        "failure must carry a stable error code"
    );
}

/// Removing schema_version from a receipt must fail with a stable schema
/// version error code (schema_version is the protocol anchor in the receipt).
#[test]
fn verification_rejects_missing_protocol_version() {
    let receipt = route_case_from_policy_json(CASE_JSON, POLICY_JSON).expect("fixture must route");
    let receipt_json = serde_json::to_string(&receipt).unwrap();

    let mut v: Value = serde_json::from_str(&receipt_json).unwrap();
    v.as_object_mut().unwrap().remove("schema_version");
    let broken_json = serde_json::to_string(&v).unwrap();

    let err = verify_receipt_from_policy_json(&broken_json, CASE_JSON, POLICY_JSON)
        .expect_err("receipt without schema_version must fail");
    assert_eq!(
        err.code, "missing_receipt_schema_version",
        "missing schema_version must produce missing_receipt_schema_version"
    );
}

// ── Semver sanity ─────────────────────────────────────────────────────────────

/// The semantic version constants must be consistent with each other.
#[test]
fn semver_constants_are_consistent() {
    assert_eq!(POSTCAD_PROTOCOL_VERSION, "1.0");
    assert_eq!(ROUTING_KERNEL_SEMVER, "1.0");
    // Label strings must remain unchanged (changing them breaks receipt hashes).
    assert_eq!(PROTOCOL_VERSION, "postcad-v1");
    assert_eq!(ROUTING_KERNEL_VERSION, "postcad-routing-v1");
}
