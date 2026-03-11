//! Subprocess tests that freeze the full receipt schema contract.
//!
//! These tests cover the remaining gaps in the committed-field tamper matrix
//! and the required-field enforcement contract. Each test:
//!   - spawns the real binary
//!   - asserts the exact exit code (0 or 1)
//!   - asserts the exact stable failure code when verification must fail
//!
//! ## What is already locked (elsewhere)
//! routing_proof_hash, candidate_pool_hash, eligible_candidate_ids_hash,
//! selection_input_candidate_ids_hash, candidate_order_hash,
//! routing_decision_hash (stale + replay), audit_entry_hash, audit_previous_hash,
//! registry_snapshot_hash, routing_kernel_version, routing_input sub-field tamper,
//! receipt_hash artifact integrity, schema_version checks, extra-field tolerance,
//! top-level key-order tolerance, pretty-print tolerance.
//!
//! ## What this file locks
//! Required-field removal: receipt_hash, routing_input_hash, routing_proof_hash.
//! Tamper gaps: case_fingerprint, policy_fingerprint, policy_version,
//!              routing_input_hash (direct hash tamper), outcome.
//! Sub-object key-order: routing_input with non-alphabetical key order.

use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_postcad-cli"))
}

fn scenario(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/scenarios")
        .join(name)
}

fn route_scenario(s: &std::path::Path) -> (bool, Value) {
    let out = Command::new(bin())
        .args([
            "route-case",
            "--json",
            "--case",
            s.join("case.json").to_str().unwrap(),
            "--candidates",
            s.join("candidates.json").to_str().unwrap(),
            "--snapshot",
            s.join("snapshot.json").to_str().unwrap(),
        ])
        .output()
        .expect("failed to spawn postcad-cli");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|_| panic!("stdout was not valid JSON:\n{}", stdout));
    (out.status.success(), json)
}

fn verify(args: &[&str]) -> (bool, Value) {
    let out = Command::new(bin())
        .args(args)
        .output()
        .expect("failed to spawn postcad-cli");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|_| panic!("stdout was not valid JSON:\n{}", stdout));
    (out.status.success(), json)
}

fn write_tmp(tag: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("postcad_schema_contract_{}.json", tag));
    std::fs::write(&path, content).expect("failed to write tmp file");
    path
}

/// Recomputes receipt_hash after in-place tampering. Removes receipt_hash from
/// the Value, serializes the remainder to compact JSON (serde alphabetises keys),
/// then returns the lowercase SHA-256 hex digest.
fn recompute_receipt_hash(receipt_val: &Value) -> String {
    use sha2::{Digest, Sha256};
    let mut obj = receipt_val.clone();
    obj.as_object_mut().unwrap().remove("receipt_hash");
    let canonical = serde_json::to_string(&obj).unwrap();
    format!("{:x}", Sha256::digest(canonical.as_bytes()))
}

// ── Required-field removal: receipt_parse_failed ──────────────────────────────

/// Removing `receipt_hash` (a non-Optional committed field) from the receipt
/// must produce `receipt_parse_failed`.
///
/// serde deserializes `receipt_hash: String` as required; a missing field causes
/// the parse step to fail before any semantic check is reached.
#[test]
fn verify_receipt_rejects_missing_receipt_hash_field_with_parse_failed() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    receipt_val.as_object_mut().unwrap().remove("receipt_hash");
    let broken = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("missing_receipt_hash", &broken);

    let (success, json) = verify(&[
        "verify-receipt",
        "--json",
        "--receipt",
        tmp.to_str().unwrap(),
        "--case",
        s.join("case.json").to_str().unwrap(),
        "--policy",
        s.join("policy.json").to_str().unwrap(),
        "--candidates",
        s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(
        !success,
        "verify-receipt must exit 1 when receipt_hash field is absent"
    );
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "receipt_parse_failed",
        "missing receipt_hash must produce receipt_parse_failed"
    );
}

/// Removing `routing_input_hash` (a non-Optional committed field) from the
/// receipt must produce `receipt_parse_failed`.
#[test]
fn verify_receipt_rejects_missing_routing_input_hash_field_with_parse_failed() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    receipt_val
        .as_object_mut()
        .unwrap()
        .remove("routing_input_hash");
    let broken = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("missing_routing_input_hash", &broken);

    let (success, json) = verify(&[
        "verify-receipt",
        "--json",
        "--receipt",
        tmp.to_str().unwrap(),
        "--case",
        s.join("case.json").to_str().unwrap(),
        "--policy",
        s.join("policy.json").to_str().unwrap(),
        "--candidates",
        s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(
        !success,
        "verify-receipt must exit 1 when routing_input_hash field is absent"
    );
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "receipt_parse_failed",
        "missing routing_input_hash must produce receipt_parse_failed"
    );
}

/// Removing `routing_proof_hash` (a non-Optional committed field) from the
/// receipt must produce `receipt_parse_failed`.
#[test]
fn verify_receipt_rejects_missing_routing_proof_hash_field_with_parse_failed() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    receipt_val
        .as_object_mut()
        .unwrap()
        .remove("routing_proof_hash");
    let broken = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("missing_routing_proof_hash", &broken);

    let (success, json) = verify(&[
        "verify-receipt",
        "--json",
        "--receipt",
        tmp.to_str().unwrap(),
        "--case",
        s.join("case.json").to_str().unwrap(),
        "--policy",
        s.join("policy.json").to_str().unwrap(),
        "--candidates",
        s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(
        !success,
        "verify-receipt must exit 1 when routing_proof_hash field is absent"
    );
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "receipt_parse_failed",
        "missing routing_proof_hash must produce receipt_parse_failed"
    );
}

// ── Committed-field tamper: case_fingerprint ──────────────────────────────────

/// Tampering `case_fingerprint` (the SHA-256 of the canonical case payload) must
/// produce exit 1 with stable code `case_fingerprint_mismatch`.
///
/// The verifier recomputes the fingerprint from the provided `case.json` and
/// compares it to the stored value. Substituting a zeroed hash makes them
/// diverge at step 2 of the verification protocol.
///
/// `receipt_hash` is recomputed so that the artifact-integrity check (step 1b)
/// passes and the field-specific check fires.
#[test]
fn verify_receipt_rejects_tampered_case_fingerprint_with_stable_code() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok, "route-case must succeed before tampering");

    receipt_val["case_fingerprint"] =
        serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("tampered_case_fp", &tampered);

    let (success, json) = verify(&[
        "verify-receipt",
        "--json",
        "--receipt",
        tmp.to_str().unwrap(),
        "--case",
        s.join("case.json").to_str().unwrap(),
        "--policy",
        s.join("policy.json").to_str().unwrap(),
        "--candidates",
        s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(
        !success,
        "verify-receipt must exit 1 when case_fingerprint is tampered"
    );
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "case_fingerprint_mismatch",
        "code must be case_fingerprint_mismatch; got: {:?}",
        json["code"]
    );
    assert!(json["reason"].is_string(), "reason must be present");
}

// ── Committed-field tamper: policy_fingerprint ────────────────────────────────

/// Tampering `policy_fingerprint` (the SHA-256 of the canonical routing policy
/// configuration) must produce exit 1 with stable code `policy_fingerprint_mismatch`.
///
/// The verifier recomputes the fingerprint from `policy.json` and compares it
/// to the stored value. A zeroed hash diverges at step 4.
#[test]
fn verify_receipt_rejects_tampered_policy_fingerprint_with_stable_code() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok, "route-case must succeed before tampering");

    receipt_val["policy_fingerprint"] =
        serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("tampered_policy_fp", &tampered);

    let (success, json) = verify(&[
        "verify-receipt",
        "--json",
        "--receipt",
        tmp.to_str().unwrap(),
        "--case",
        s.join("case.json").to_str().unwrap(),
        "--policy",
        s.join("policy.json").to_str().unwrap(),
        "--candidates",
        s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(
        !success,
        "verify-receipt must exit 1 when policy_fingerprint is tampered"
    );
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "policy_fingerprint_mismatch",
        "code must be policy_fingerprint_mismatch; got: {:?}",
        json["code"]
    );
    assert!(json["reason"].is_string(), "reason must be present");
}

// ── Committed-field tamper: policy_version ────────────────────────────────────

/// Changing `policy_version` in the receipt from `null` to a non-null string
/// must produce exit 1 with stable code `policy_version_mismatch`.
///
/// The scenario's `policy.json` does not declare a `policy_version`, so
/// `policy_input.policy_version` is `None`. Changing the receipt's field to
/// `Some("v1")` creates a mismatch caught at step 4b of the verification
/// protocol.
///
/// `policy_version` is also committed inside `routing_decision_hash` (via the
/// `RoutingDecisionSnapshot`). To reach step 4b, `routing_decision_hash` must
/// be recomputed consistently with the tampered `policy_version`, so that step
/// 1e (the self-contained decision hash check) passes and the mismatch is
/// detected at the bundle comparison step.
#[test]
fn verify_receipt_rejects_tampered_policy_version_with_stable_code() {
    use sha2::{Digest, Sha256};

    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok, "route-case must succeed before tampering");
    assert!(
        receipt_val["policy_version"].is_null(),
        "scenario receipt must have null policy_version for this test"
    );

    let new_version = "v1-injected";

    // Tamper policy_version.
    receipt_val["policy_version"] = serde_json::json!(new_version);

    // Recompute routing_decision_hash consistently with the tampered version
    // so that step 1e (self-contained decision hash check) passes and the
    // policy_version_mismatch check at step 4b is reached instead.
    let decision_obj = serde_json::json!({
        "decision_type":        receipt_val["outcome"],
        "policy_version":       receipt_val["policy_version"],
        "reason":               receipt_val["refusal_code"],
        "routing_kernel_version": receipt_val["routing_kernel_version"],
        "selected_candidate_id":  receipt_val["selected_candidate_id"],
    });
    let new_decision_hash = format!(
        "{:x}",
        Sha256::digest(serde_json::to_string(&decision_obj).unwrap().as_bytes())
    );
    receipt_val["routing_decision_hash"] = serde_json::json!(new_decision_hash);

    // Recompute receipt_hash last so the artifact-integrity check passes.
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("tampered_policy_version", &tampered);

    let (success, json) = verify(&[
        "verify-receipt",
        "--json",
        "--receipt",
        tmp.to_str().unwrap(),
        "--case",
        s.join("case.json").to_str().unwrap(),
        "--policy",
        s.join("policy.json").to_str().unwrap(),
        "--candidates",
        s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(
        !success,
        "verify-receipt must exit 1 when policy_version is tampered"
    );
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "policy_version_mismatch",
        "code must be policy_version_mismatch; got: {:?}",
        json["code"]
    );
    assert!(json["reason"].is_string(), "reason must be present");
}

// ── Committed-field tamper: routing_input_hash (direct hash tamper) ───────────

/// Zeroing `routing_input_hash` (the stored digest of the routing input
/// envelope) without touching `routing_input` must produce exit 1 with stable
/// code `routing_input_hash_mismatch`.
///
/// The verifier recomputes `hash_routing_input(&receipt.routing_input)` at
/// step 1c and compares it to the stored hash. Substituting a zeroed value
/// for the hash (leaving `routing_input` intact) creates a mismatch.
///
/// This test is distinct from the `verify_receipt_rejects_tampered_routing_input`
/// test in `cli_errors.rs`, which tampers the *content* of `routing_input`
/// and leaves the hash stale. Here, the *hash field itself* is zeroed while
/// the underlying data is untouched.
#[test]
fn verify_receipt_rejects_tampered_routing_input_hash_directly_with_stable_code() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok, "route-case must succeed before tampering");

    // Zero out the hash field only; leave routing_input unchanged.
    receipt_val["routing_input_hash"] =
        serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("tampered_routing_input_hash_direct", &tampered);

    let (success, json) = verify(&[
        "verify-receipt",
        "--json",
        "--receipt",
        tmp.to_str().unwrap(),
        "--case",
        s.join("case.json").to_str().unwrap(),
        "--policy",
        s.join("policy.json").to_str().unwrap(),
        "--candidates",
        s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(
        !success,
        "verify-receipt must exit 1 when routing_input_hash is zeroed"
    );
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "routing_input_hash_mismatch",
        "code must be routing_input_hash_mismatch; got: {:?}",
        json["code"]
    );
    assert!(json["reason"].is_string(), "reason must be present");
}

// ── Committed-field tamper: outcome ──────────────────────────────────────────

/// Changing `outcome` from `"routed"` to `"refused"` without updating
/// `routing_decision_hash` must produce exit 1 with stable code
/// `routing_decision_hash_mismatch`.
///
/// `routing_decision_hash` commits to `outcome` (serialized as `decision_type`
/// in the canonical decision snapshot). Leaving the hash stale while changing
/// `outcome` causes step 1e (self-contained decision hash check) to fire before
/// the routing replay.
///
/// `receipt_hash` is recomputed so that the artifact-integrity check passes
/// and the field-specific check fires.
#[test]
fn verify_receipt_rejects_tampered_outcome_field_with_stable_code() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok, "route-case must succeed before tampering");
    assert_eq!(
        receipt_val["outcome"], "routed",
        "scenario must produce a routed outcome for this test"
    );

    // Change outcome; leave routing_decision_hash stale (not updated).
    receipt_val["outcome"] = serde_json::json!("refused");
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("tampered_outcome", &tampered);

    let (success, json) = verify(&[
        "verify-receipt",
        "--json",
        "--receipt",
        tmp.to_str().unwrap(),
        "--case",
        s.join("case.json").to_str().unwrap(),
        "--policy",
        s.join("policy.json").to_str().unwrap(),
        "--candidates",
        s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(
        !success,
        "verify-receipt must exit 1 when outcome is tampered"
    );
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "routing_decision_hash_mismatch",
        "code must be routing_decision_hash_mismatch; got: {:?}",
        json["code"]
    );
    assert!(json["reason"].is_string(), "reason must be present");
}

// ── Key-order independence: routing_input sub-object ─────────────────────────

/// A receipt whose `routing_input` sub-object has keys in non-alphabetical
/// (reversed) order must still verify successfully.
///
/// serde_json deserialises object keys by name regardless of their order in
/// the raw JSON bytes. The verifier therefore receives the same
/// `RoutingInputEnvelope` struct regardless of key order, and both
/// `routing_input_hash` and `receipt_hash` remain valid.
///
/// This test complements `verify_receipt_tolerates_different_key_order` (which
/// scrambles top-level keys) by targeting the nested sub-object specifically.
#[test]
fn verify_receipt_tolerates_routing_input_with_reversed_key_order() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, receipt_val) = route_scenario(&s);
    assert!(route_ok, "route-case must succeed");

    // Extract routing_input field values.
    let ri = &receipt_val["routing_input"];
    let case_id = ri["case_id"].as_str().unwrap();
    let file_type = ri["file_type"].as_str().unwrap();
    let jurisdiction = ri["jurisdiction"].as_str().unwrap();
    let manufacturer_ctry = ri["manufacturer_country"].as_str().unwrap();
    let material = ri["material"].as_str().unwrap();
    let patient_ctry = ri["patient_country"].as_str().unwrap();
    let procedure = ri["procedure"].as_str().unwrap();
    let routing_policy = ri["routing_policy"].as_str().unwrap();

    // Build routing_input JSON with keys in REVERSED alphabetical order.
    // Standard (alphabetical): case_id, file_type, jurisdiction, ...
    // Reversed:                routing_policy, procedure, patient_country, ...
    let reversed_ri = format!(
        concat!(
            r#"{{"routing_policy":"{routing_policy}","procedure":"{procedure}","#,
            r#""patient_country":"{patient_ctry}","material":"{material}","#,
            r#""manufacturer_country":"{manufacturer_ctry}","jurisdiction":"{jurisdiction}","#,
            r#""file_type":"{file_type}","case_id":"{case_id}"}}"#
        ),
        routing_policy = routing_policy,
        procedure = procedure,
        patient_ctry = patient_ctry,
        material = material,
        manufacturer_ctry = manufacturer_ctry,
        jurisdiction = jurisdiction,
        file_type = file_type,
        case_id = case_id,
    );

    // Build the receipt JSON: serialize all other fields (serde alphabetises
    // them), then append routing_input with the reversed key order.
    let mut rest = receipt_val.clone();
    rest.as_object_mut().unwrap().remove("routing_input");
    let rest_str = serde_json::to_string(&rest).unwrap();
    // rest_str is a compact JSON object ending with "}". Strip the closing
    // brace and inject routing_input before re-closing.
    let prefix = &rest_str[..rest_str.len() - 1];
    let scrambled = format!("{},\"routing_input\":{}}}", prefix, reversed_ri);

    let tmp = write_tmp("reversed_routing_input_keys", &scrambled);

    let (success, json) = verify(&[
        "verify-receipt",
        "--json",
        "--receipt",
        tmp.to_str().unwrap(),
        "--case",
        s.join("case.json").to_str().unwrap(),
        "--policy",
        s.join("policy.json").to_str().unwrap(),
        "--candidates",
        s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(
        success,
        "verify-receipt must exit 0 when routing_input keys are in reversed order; got: {:?}",
        json
    );
    assert_eq!(
        json["result"], "VERIFIED",
        "reversed routing_input key order must produce VERIFIED; got: {:?}",
        json
    );
}
