//! Routing proof object tests.
//!
//! Proves that `RoutingProofObject` is deterministic, that every field maps
//! correctly from the receipt, and that any tampering is caught by
//! `verify_routing_proof`.

use postcad_cli::{
    build_routing_proof, route_case_from_policy_json, verify_routing_proof,
    verify_receipt_from_policy_json,
};

// ── Canonical fixtures ────────────────────────────────────────────────────────

const CASE_JSON: &str = r#"{
  "case_id": "a1b2c3d4-0000-0000-0000-000000000001",
  "jurisdiction": "DE",
  "routing_policy": "allow_domestic_and_cross_border",
  "patient_country": "germany",
  "manufacturer_country": "germany",
  "material": "zirconia",
  "procedure": "crown",
  "file_type": "stl"
}"#;

const POLICY_JSON: &str = r#"{
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
      "evidence_references": ["cert-001"],
      "attestation_statuses": ["verified"],
      "is_eligible": true
    }
  ]
}"#;

const REFUSED_POLICY_JSON: &str = r#"{
  "jurisdiction": "DE",
  "routing_policy": "allow_domestic_and_cross_border",
  "candidates": [],
  "snapshots": []
}"#;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn routed_receipt() -> postcad_cli::RoutingReceipt {
    route_case_from_policy_json(CASE_JSON, POLICY_JSON).expect("routing must succeed")
}

fn refused_receipt() -> postcad_cli::RoutingReceipt {
    route_case_from_policy_json(CASE_JSON, REFUSED_POLICY_JSON).expect("routing must succeed")
}

// ── Determinism tests ─────────────────────────────────────────────────────────

/// Same receipt must always produce the same proof object.
#[test]
fn proof_is_deterministic_for_routed_case() {
    let receipt = routed_receipt();
    let p1 = build_routing_proof(&receipt);
    let p2 = build_routing_proof(&receipt);
    assert_eq!(p1, p2, "build_routing_proof must be deterministic");
}

/// Proof generated from two independently routed receipts of the same case
/// must be identical — end-to-end determinism.
#[test]
fn proof_is_identical_for_same_inputs() {
    let r1 = routed_receipt();
    let r2 = routed_receipt();
    assert_eq!(
        build_routing_proof(&r1),
        build_routing_proof(&r2),
        "proof must be identical for identical routing inputs"
    );
}

/// Proof for a refused outcome must also be deterministic.
#[test]
fn proof_is_deterministic_for_refused_case() {
    let receipt = refused_receipt();
    let p1 = build_routing_proof(&receipt);
    let p2 = build_routing_proof(&receipt);
    assert_eq!(p1, p2);
}

// ── Field mapping tests ───────────────────────────────────────────────────────

/// Every proof field must equal the corresponding receipt field.
#[test]
fn proof_fields_match_receipt_fields() {
    let receipt = routed_receipt();
    let proof = build_routing_proof(&receipt);

    assert_eq!(proof.routing_kernel_version, receipt.routing_kernel_version);
    assert_eq!(proof.routing_input_hash,     receipt.routing_input_hash);
    assert_eq!(proof.registry_snapshot_hash, receipt.registry_snapshot_hash);
    assert_eq!(proof.candidate_pool_hash,    receipt.candidate_pool_hash);
    assert_eq!(proof.candidate_order_hash,   receipt.candidate_order_hash);
    assert_eq!(proof.routing_decision_hash,  receipt.routing_decision_hash);
    assert_eq!(proof.selected_candidate_id,  receipt.selected_candidate_id);
    assert_eq!(proof.receipt_hash,           receipt.receipt_hash);
    assert_eq!(proof.audit_entry_hash,       receipt.audit_entry_hash);
    assert_eq!(proof.audit_previous_hash,    receipt.audit_previous_hash);
}

/// For a refused receipt, `selected_candidate_id` in the proof must be `None`.
#[test]
fn proof_selected_candidate_is_none_for_refused_receipt() {
    let receipt = refused_receipt();
    let proof = build_routing_proof(&receipt);
    assert!(proof.selected_candidate_id.is_none());
    assert_eq!(proof.selected_candidate_id, receipt.selected_candidate_id);
}

/// `protocol_version` must equal the `PROTOCOL_VERSION` constant.
#[test]
fn proof_protocol_version_matches_constant() {
    let receipt = routed_receipt();
    let proof = build_routing_proof(&receipt);
    assert_eq!(proof.protocol_version, postcad_cli::PROTOCOL_VERSION);
}

// ── Verification tests ────────────────────────────────────────────────────────

/// A proof freshly built from a receipt must verify against that receipt.
#[test]
fn proof_verifies_against_its_source_receipt() {
    let receipt = routed_receipt();
    let proof = build_routing_proof(&receipt);
    verify_routing_proof(&proof, &receipt).expect("proof must verify against source receipt");
}

/// A proof from a refused receipt must also verify.
#[test]
fn proof_verifies_for_refused_receipt() {
    let receipt = refused_receipt();
    let proof = build_routing_proof(&receipt);
    verify_routing_proof(&proof, &receipt).expect("refused proof must verify");
}

/// `verify_routing_proof` succeeding implies `verify_receipt_from_policy_json`
/// also succeeds — the proof covers the same commitments as receipt verification.
#[test]
fn proof_verification_consistent_with_receipt_verification() {
    let receipt = routed_receipt();
    let proof = build_routing_proof(&receipt);

    let proof_result = verify_routing_proof(&proof, &receipt);
    let receipt_json = serde_json::to_string(&receipt).unwrap();
    let receipt_result = verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_JSON);

    assert!(proof_result.is_ok(), "proof verification must succeed");
    assert!(receipt_result.is_ok(), "receipt verification must succeed");
}

// ── Tamper detection tests ────────────────────────────────────────────────────

/// Tampering `routing_input_hash` in the proof must be caught.
#[test]
fn tampered_routing_input_hash_fails_proof_verification() {
    let receipt = routed_receipt();
    let mut proof = build_routing_proof(&receipt);
    proof.routing_input_hash = "0".repeat(64);
    let err = verify_routing_proof(&proof, &receipt).expect_err("tampered proof must fail");
    assert_eq!(err.code, "proof_field_mismatch");
    assert!(err.message.contains("routing_input_hash"));
}

/// Tampering `registry_snapshot_hash` in the proof must be caught.
#[test]
fn tampered_registry_snapshot_hash_fails_proof_verification() {
    let receipt = routed_receipt();
    let mut proof = build_routing_proof(&receipt);
    proof.registry_snapshot_hash = "f".repeat(64);
    let err = verify_routing_proof(&proof, &receipt).expect_err("tampered proof must fail");
    assert_eq!(err.code, "proof_field_mismatch");
    assert!(err.message.contains("registry_snapshot_hash"));
}

/// Tampering `routing_decision_hash` in the proof must be caught.
#[test]
fn tampered_routing_decision_hash_fails_proof_verification() {
    let receipt = routed_receipt();
    let mut proof = build_routing_proof(&receipt);
    proof.routing_decision_hash = "a".repeat(64);
    let err = verify_routing_proof(&proof, &receipt).expect_err("tampered proof must fail");
    assert_eq!(err.code, "proof_field_mismatch");
    assert!(err.message.contains("routing_decision_hash"));
}

/// Tampering `receipt_hash` in the proof must be caught.
#[test]
fn tampered_receipt_hash_fails_proof_verification() {
    let receipt = routed_receipt();
    let mut proof = build_routing_proof(&receipt);
    proof.receipt_hash = "b".repeat(64);
    let err = verify_routing_proof(&proof, &receipt).expect_err("tampered proof must fail");
    assert_eq!(err.code, "proof_field_mismatch");
    assert!(err.message.contains("receipt_hash"));
}

/// Tampering `audit_entry_hash` in the proof must be caught.
#[test]
fn tampered_audit_entry_hash_fails_proof_verification() {
    let receipt = routed_receipt();
    let mut proof = build_routing_proof(&receipt);
    proof.audit_entry_hash = "c".repeat(64);
    let err = verify_routing_proof(&proof, &receipt).expect_err("tampered proof must fail");
    assert_eq!(err.code, "proof_field_mismatch");
    assert!(err.message.contains("audit_entry_hash"));
}

/// A stale or forged `protocol_version` must be rejected.
#[test]
fn wrong_protocol_version_fails_proof_verification() {
    let receipt = routed_receipt();
    let mut proof = build_routing_proof(&receipt);
    proof.protocol_version = "postcad-v0".to_string();
    let err = verify_routing_proof(&proof, &receipt).expect_err("wrong protocol version must fail");
    assert_eq!(err.code, "proof_protocol_version_mismatch");
}

/// A proof built from a different (unrelated) receipt must not verify against
/// the original receipt.
#[test]
fn proof_from_different_receipt_fails_verification() {
    let receipt_a = routed_receipt();
    let receipt_b = refused_receipt();
    let proof_b = build_routing_proof(&receipt_b);
    // proof_b was built from receipt_b; verifying it against receipt_a must fail
    // because the hash commitments differ.
    let result = verify_routing_proof(&proof_b, &receipt_a);
    assert!(result.is_err(), "proof from a different receipt must not verify");
}
