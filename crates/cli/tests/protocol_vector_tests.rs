//! Protocol conformance test vectors.
//!
//! Each vector in `tests/protocol_vectors/` contains:
//!   - `case.json`              — CaseInput
//!   - `registry_snapshot.json` — Vec<ManufacturerRecord> (typed registry)
//!   - `policy.json`            — RegistryRoutingConfig (jurisdiction + routing_policy)
//!   - `expected_receipt.json`  — frozen RoutingReceipt output (generated on first run)
//!
//! # Seeding
//!
//! When `expected_receipt.json` is absent the test runner generates it from the
//! current routing output and passes.  On all subsequent runs the frozen receipt
//! is loaded and compared field-for-field; any drift fails the test.
//!
//! # Verification
//!
//! After the routing check every vector also runs `verify_receipt_from_policy_json`
//! against the expected receipt using the derived policy bundle returned by
//! `route_case_from_registry_json`.  This proves the receipt is independently
//! verifiable — not just that the output is stable.

use std::path::{Path, PathBuf};

use postcad_cli::{route_case_from_registry_json, verify_receipt_from_policy_json};
use serde_json::Value;

// ── Directory helpers ─────────────────────────────────────────────────────────

fn vectors_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR is crates/cli; vectors live two levels up at
    // <workspace_root>/tests/protocol_vectors/
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/protocol_vectors")
}

fn read_vector_file(vector: &str, file: &str) -> String {
    let path = vectors_dir().join(vector).join(file);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read vector file {}/{}: {}", vector, file, e))
}

fn expected_receipt_path(vector: &str) -> PathBuf {
    vectors_dir().join(vector).join("expected_receipt.json")
}

// ── Core runner ───────────────────────────────────────────────────────────────

/// Runs one protocol vector and verifies conformance.
///
/// 1. Route the case via the registry-backed path.
/// 2. If `expected_receipt.json` does not yet exist, write it (first-run seed).
/// 3. Otherwise compare the actual receipt to the frozen one field-for-field.
/// 4. Run `verify_receipt_from_policy_json` on the expected receipt.
fn run_vector(vector: &str) {
    let case_json = read_vector_file(vector, "case.json");
    let registry_json = read_vector_file(vector, "registry_snapshot.json");
    let config_json = read_vector_file(vector, "policy.json");

    let result = route_case_from_registry_json(&case_json, &registry_json, &config_json)
        .unwrap_or_else(|e| panic!("vector '{}' routing failed: {}", vector, e));

    let actual_value: Value =
        serde_json::to_value(&result.receipt).expect("receipt must serialise to JSON");

    let expected_path = expected_receipt_path(vector);

    if !expected_path.exists() {
        // First run: seed the expected receipt.
        let pretty = serde_json::to_string_pretty(&result.receipt)
            .expect("receipt must serialise to pretty JSON");
        std::fs::write(&expected_path, pretty)
            .unwrap_or_else(|e| panic!("cannot write {}: {}", expected_path.display(), e));
        eprintln!(
            "protocol_vector '{}': generated expected_receipt.json (commit this file)",
            vector
        );
    } else {
        // Subsequent runs: compare against the frozen artifact.
        let frozen_json = std::fs::read_to_string(&expected_path)
            .unwrap_or_else(|e| panic!("cannot read {}: {}", expected_path.display(), e));
        let expected_value: Value = serde_json::from_str(&frozen_json).unwrap_or_else(|e| {
            panic!(
                "vector '{}' expected_receipt.json is not valid JSON: {}",
                vector, e
            )
        });

        assert_eq!(
            actual_value, expected_value,
            "protocol vector '{}' output drifted from expected_receipt.json",
            vector,
        );
    }

    // Always verify the expected receipt against the derived policy bundle.
    // On the first run this verifies the freshly generated receipt; on
    // subsequent runs it proves the frozen artifact is still independently
    // verifiable.
    let expected_json = std::fs::read_to_string(&expected_path)
        .expect("expected_receipt.json must be present at this point");
    verify_receipt_from_policy_json(&expected_json, &case_json, &result.derived_policy_json)
        .unwrap_or_else(|reason| {
            panic!(
                "protocol vector '{}' verify-receipt failed: {}",
                vector, reason
            )
        });
}

// ── Vector tests ──────────────────────────────────────────────────────────────

/// v01 — Single eligible domestic manufacturer is routed successfully.
///
/// Invariant: outcome == "routed", selected_candidate_id == "mfr-de-001".
#[test]
fn vector_v01_basic_routing() {
    run_vector("v01_basic_routing");

    // Structural assertions on top of the frozen-receipt check.
    let case_json = read_vector_file("v01_basic_routing", "case.json");
    let registry_json = read_vector_file("v01_basic_routing", "registry_snapshot.json");
    let config_json = read_vector_file("v01_basic_routing", "policy.json");

    let result = route_case_from_registry_json(&case_json, &registry_json, &config_json).unwrap();
    assert_eq!(result.receipt.outcome, "routed");
    assert_eq!(
        result.receipt.selected_candidate_id.as_deref(),
        Some("mfr-de-001"),
        "v01: must select the only eligible manufacturer"
    );
    assert!(result.receipt.refusal_code.is_none());
}

/// v02 — Three eligible candidates; deterministic selection is stable.
///
/// Invariant: outcome == "routed", receipt_hash is identical on every run.
#[test]
fn vector_v02_multi_candidate() {
    run_vector("v02_multi_candidate");

    let case_json = read_vector_file("v02_multi_candidate", "case.json");
    let registry_json = read_vector_file("v02_multi_candidate", "registry_snapshot.json");
    let config_json = read_vector_file("v02_multi_candidate", "policy.json");

    let r1 = route_case_from_registry_json(&case_json, &registry_json, &config_json).unwrap();
    let r2 = route_case_from_registry_json(&case_json, &registry_json, &config_json).unwrap();

    assert_eq!(r1.receipt.outcome, "routed");
    assert_eq!(
        r1.receipt.receipt_hash, r2.receipt.receipt_hash,
        "v02: multi-candidate selection must be deterministic"
    );
    assert_eq!(
        r1.receipt.selected_candidate_id, r2.receipt.selected_candidate_id,
        "v02: same candidate must be selected on every call"
    );
}

/// v03 — Only US manufacturer available; DE case must be refused with
/// `no_jurisdiction_match`.
///
/// Invariant: outcome == "refused", refusal_code == "no_jurisdiction_match".
#[test]
fn vector_v03_jurisdiction_refusal() {
    run_vector("v03_jurisdiction_refusal");

    let case_json = read_vector_file("v03_jurisdiction_refusal", "case.json");
    let registry_json = read_vector_file("v03_jurisdiction_refusal", "registry_snapshot.json");
    let config_json = read_vector_file("v03_jurisdiction_refusal", "policy.json");

    let result = route_case_from_registry_json(&case_json, &registry_json, &config_json).unwrap();
    assert_eq!(result.receipt.outcome, "refused");
    assert_eq!(
        result.receipt.refusal_code.as_deref(),
        Some("no_jurisdiction_match"),
        "v03: must refuse with no_jurisdiction_match"
    );
    assert!(result.receipt.selected_candidate_id.is_none());
}

/// v04 — Manufacturer only supports implant + bridge; crown case must be
/// refused with `no_capability_match`.
///
/// Invariant: outcome == "refused", refusal_code == "no_capability_match".
#[test]
fn vector_v04_capability_refusal() {
    run_vector("v04_capability_refusal");

    let case_json = read_vector_file("v04_capability_refusal", "case.json");
    let registry_json = read_vector_file("v04_capability_refusal", "registry_snapshot.json");
    let config_json = read_vector_file("v04_capability_refusal", "policy.json");

    let result = route_case_from_registry_json(&case_json, &registry_json, &config_json).unwrap();
    assert_eq!(result.receipt.outcome, "refused");
    assert_eq!(
        result.receipt.refusal_code.as_deref(),
        Some("no_capability_match"),
        "v04: must refuse with no_capability_match"
    );
}

/// v05 — Both manufacturers have invalid attestations (expired + revoked);
/// case must be refused with `attestation_failed`.
///
/// Invariant: outcome == "refused", refusal_code == "attestation_failed".
#[test]
fn vector_v05_attestation_refusal() {
    run_vector("v05_attestation_refusal");

    let case_json = read_vector_file("v05_attestation_refusal", "case.json");
    let registry_json = read_vector_file("v05_attestation_refusal", "registry_snapshot.json");
    let config_json = read_vector_file("v05_attestation_refusal", "policy.json");

    let result = route_case_from_registry_json(&case_json, &registry_json, &config_json).unwrap();
    assert_eq!(result.receipt.outcome, "refused");
    assert_eq!(
        result.receipt.refusal_code.as_deref(),
        Some("attestation_failed"),
        "v05: must refuse with attestation_failed"
    );
}

// ── Cross-vector invariants ───────────────────────────────────────────────────

/// All 5 vectors must be deterministic: routing the same inputs twice yields
/// the same receipt_hash.
#[test]
fn all_vectors_are_deterministic() {
    let vectors = [
        "v01_basic_routing",
        "v02_multi_candidate",
        "v03_jurisdiction_refusal",
        "v04_capability_refusal",
        "v05_attestation_refusal",
    ];

    for vector in vectors {
        let case_json = read_vector_file(vector, "case.json");
        let registry_json = read_vector_file(vector, "registry_snapshot.json");
        let config_json = read_vector_file(vector, "policy.json");

        let r1 = route_case_from_registry_json(&case_json, &registry_json, &config_json).unwrap();
        let r2 = route_case_from_registry_json(&case_json, &registry_json, &config_json).unwrap();

        assert_eq!(
            r1.receipt.receipt_hash, r2.receipt.receipt_hash,
            "vector '{}' must be deterministic (receipt_hash differs between runs)",
            vector
        );
    }
}
