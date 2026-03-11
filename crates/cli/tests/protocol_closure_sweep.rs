//! Protocol closure sweep — final hardening pass.
//!
//! Closes the remaining subprocess-level contract gaps not covered by the
//! earlier test files (`cli_errors.rs`, `receipt_schema_contract.rs`,
//! `cli_artifact_contract.rs`).
//!
//! ## What this file locks
//!
//! 1. `case_parse_failed`     — malformed case.json passed to verify-receipt
//! 2. `policy_bundle_parse_failed` — malformed policy.json / candidates.json
//! 3. Refusal path tamper: `refusal_code` change → `routing_decision_hash_mismatch`
//! 4. Refusal path tamper: inject non-null `selected_candidate_id` into refused
//!    receipt (consistent decision hash) → `routing_decision_replay_mismatch`
//! 5. Routed path tamper: change `refusal_code` from null to non-null →
//!    `routing_decision_hash_mismatch`
//! 6. `routing_input` whole-object removal → `receipt_parse_failed`
//! 7. `audit_previous_hash` field removal → `receipt_parse_failed`
//! 8. Extra field inside `routing_input` sub-object → VERIFIED (tolerance)

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
    let path = std::env::temp_dir().join(format!("postcad_closure_sweep_{}.json", tag));
    std::fs::write(&path, content).expect("failed to write tmp file");
    path
}

/// Recomputes receipt_hash after in-place tampering. Removes receipt_hash from
/// the Value, serializes the remainder to compact JSON (serde alphabetises
/// keys), and returns the lowercase SHA-256 hex digest.
fn recompute_receipt_hash(receipt_val: &Value) -> String {
    use sha2::{Digest, Sha256};
    let mut obj = receipt_val.clone();
    obj.as_object_mut().unwrap().remove("receipt_hash");
    let canonical = serde_json::to_string(&obj).unwrap();
    format!("{:x}", Sha256::digest(canonical.as_bytes()))
}

// ── 1. case_parse_failed ──────────────────────────────────────────────────────

/// Passing malformed (non-JSON) bytes as `--case` to `verify-receipt` must
/// produce exit 1 with stable code `case_parse_failed`.
///
/// The verifier reads and parses case.json at step 2; invalid bytes cause a
/// parse error before any fingerprint comparison.
#[test]
fn verify_receipt_exits_1_with_case_parse_failed_on_malformed_case_json() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, receipt_val) = route_scenario(&s);
    assert!(route_ok);

    let receipt_path = write_tmp(
        "case_parse_receipt",
        &serde_json::to_string(&receipt_val).unwrap(),
    );
    let bad_case = write_tmp("case_parse_bad_case", "THIS IS NOT JSON {{{");

    let (success, json) = verify(&[
        "verify-receipt",
        "--json",
        "--receipt",
        receipt_path.to_str().unwrap(),
        "--case",
        bad_case.to_str().unwrap(),
        "--policy",
        s.join("policy.json").to_str().unwrap(),
        "--candidates",
        s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(
        !success,
        "verify-receipt must exit 1 when case.json is malformed"
    );
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "case_parse_failed",
        "malformed case.json must produce case_parse_failed; got: {:?}",
        json["code"]
    );
}

// ── 2a. policy_bundle_parse_failed — malformed policy.json ───────────────────

/// Passing malformed (non-JSON) bytes as `--policy` to `verify-receipt` must
/// produce exit 1 with stable code `policy_bundle_parse_failed`.
#[test]
fn verify_receipt_exits_1_with_policy_bundle_parse_failed_on_malformed_policy_json() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, receipt_val) = route_scenario(&s);
    assert!(route_ok);

    let receipt_path = write_tmp(
        "policy_parse_receipt",
        &serde_json::to_string(&receipt_val).unwrap(),
    );
    let bad_policy = write_tmp("policy_parse_bad_policy", "NOT JSON AT ALL }{");

    let (success, json) = verify(&[
        "verify-receipt",
        "--json",
        "--receipt",
        receipt_path.to_str().unwrap(),
        "--case",
        s.join("case.json").to_str().unwrap(),
        "--policy",
        bad_policy.to_str().unwrap(),
        "--candidates",
        s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(
        !success,
        "verify-receipt must exit 1 when policy.json is malformed"
    );
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "policy_bundle_parse_failed",
        "malformed policy.json must produce policy_bundle_parse_failed; got: {:?}",
        json["code"]
    );
}

// ── 2b. policy_bundle_parse_failed — malformed candidates.json ───────────────

/// Passing malformed (non-JSON) bytes as `--candidates` to `verify-receipt`
/// must produce exit 1 with stable code `policy_bundle_parse_failed`.
///
/// The verifier merges `policy.json` + `candidates.json` into a
/// `RoutingPolicyBundle`; a parse failure in either source maps to the same
/// `policy_bundle_parse_failed` code.
#[test]
fn verify_receipt_exits_1_with_policy_bundle_parse_failed_on_malformed_candidates_json() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, receipt_val) = route_scenario(&s);
    assert!(route_ok);

    let receipt_path = write_tmp(
        "cands_parse_receipt",
        &serde_json::to_string(&receipt_val).unwrap(),
    );
    let bad_candidates = write_tmp("cands_parse_bad_cands", "{{not valid json}}");

    let (success, json) = verify(&[
        "verify-receipt",
        "--json",
        "--receipt",
        receipt_path.to_str().unwrap(),
        "--case",
        s.join("case.json").to_str().unwrap(),
        "--policy",
        s.join("policy.json").to_str().unwrap(),
        "--candidates",
        bad_candidates.to_str().unwrap(),
    ]);
    assert!(
        !success,
        "verify-receipt must exit 1 when candidates.json is malformed"
    );
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "policy_bundle_parse_failed",
        "malformed candidates.json must produce policy_bundle_parse_failed; got: {:?}",
        json["code"]
    );
}

// ── 3. Refusal path tamper: refusal_code change ───────────────────────────────

/// Changing `refusal_code` on a REFUSED receipt from its original value to a
/// different string (without updating `routing_decision_hash`) must produce
/// exit 1 with stable code `routing_decision_hash_mismatch`.
///
/// `refusal_code` is committed inside `routing_decision_hash` (serialised as
/// the `reason` field of `RoutingDecisionSnapshot`). Leaving the hash stale
/// causes step 1e (self-contained decision hash check) to fire before the
/// routing replay.
#[test]
fn verify_receipt_rejects_tampered_refusal_code_on_refused_receipt() {
    let s = scenario("refused_no_eligible_candidates");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok, "route-case must exit 0 for a refused outcome");
    assert_eq!(
        receipt_val["outcome"], "refused",
        "scenario must produce a refused outcome"
    );

    // Change refusal_code; leave routing_decision_hash stale.
    receipt_val["refusal_code"] = serde_json::json!("injected_refusal_code");
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("tampered_refusal_code_refused", &tampered);

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
        "verify-receipt must exit 1 when refusal_code is tampered on refused receipt"
    );
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "routing_decision_hash_mismatch",
        "tampered refusal_code must produce routing_decision_hash_mismatch; got: {:?}",
        json["code"]
    );
}

// ── 4. Refusal path tamper: inject selected_candidate_id into refused receipt ─

/// Injecting a non-null `selected_candidate_id` into a REFUSED receipt (while
/// recomputing `routing_decision_hash` consistently so step 1e passes) must
/// produce exit 1 with stable code `routing_decision_replay_mismatch`.
///
/// The replay deterministically produces `None` for the refused scenario
/// (no eligible candidates), but the tampered receipt claims a specific
/// candidate was selected. The verifier detects this at step 5e.
#[test]
fn verify_receipt_replay_detects_candidate_injected_into_refused_receipt() {
    use sha2::{Digest, Sha256};

    let s = scenario("refused_no_eligible_candidates");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok, "route-case must exit 0 for a refused outcome");
    assert_eq!(
        receipt_val["outcome"], "refused",
        "scenario must produce a refused outcome"
    );
    assert!(
        receipt_val["selected_candidate_id"].is_null(),
        "refused receipt must have null selected_candidate_id"
    );

    let injected_id = "rc-injected-candidate";

    // Inject selected_candidate_id.
    receipt_val["selected_candidate_id"] = serde_json::json!(injected_id);

    // Recompute routing_decision_hash consistently with the tampered
    // selected_candidate_id so step 1e passes and the replay check fires.
    let decision_obj = serde_json::json!({
        "decision_type":          receipt_val["outcome"],
        "policy_version":         receipt_val["policy_version"],
        "reason":                 receipt_val["refusal_code"],
        "routing_kernel_version": receipt_val["routing_kernel_version"],
        "selected_candidate_id":  receipt_val["selected_candidate_id"],
    });
    let new_decision_hash = format!(
        "{:x}",
        Sha256::digest(serde_json::to_string(&decision_obj).unwrap().as_bytes())
    );
    receipt_val["routing_decision_hash"] = serde_json::json!(new_decision_hash);

    // Recompute receipt_hash so the artifact-integrity check (step 1b) passes.
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("injected_candidate_refused", &tampered);

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
        "verify-receipt must exit 1 when selected_candidate_id is injected into refused receipt"
    );
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(json["code"], "routing_decision_replay_mismatch",
        "injected candidate in refused receipt must produce routing_decision_replay_mismatch; got: {:?}",
        json["code"]);
}

// ── 5. Routed path tamper: inject non-null refusal_code ───────────────────────

/// Changing `refusal_code` from `null` to a non-null string on a ROUTED
/// receipt (without updating `routing_decision_hash`) must produce exit 1
/// with stable code `routing_decision_hash_mismatch`.
///
/// `refusal_code` is committed in the decision hash as the `reason` field.
/// A stale hash diverges at step 1e.
#[test]
fn verify_receipt_rejects_injected_refusal_code_on_routed_receipt() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok, "route-case must succeed");
    assert_eq!(
        receipt_val["outcome"], "routed",
        "scenario must produce a routed outcome"
    );
    assert!(
        receipt_val["refusal_code"].is_null(),
        "routed receipt must have null refusal_code"
    );

    // Inject a non-null refusal_code; leave routing_decision_hash stale.
    receipt_val["refusal_code"] = serde_json::json!("injected_refusal");
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("injected_refusal_code_routed", &tampered);

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
        "verify-receipt must exit 1 when refusal_code is injected into routed receipt"
    );
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(json["code"], "routing_decision_hash_mismatch",
        "injected refusal_code on routed receipt must produce routing_decision_hash_mismatch; got: {:?}",
        json["code"]);
}

// ── 6. routing_input whole-object removal → receipt_parse_failed ──────────────

/// Removing the entire `routing_input` object from the receipt must produce
/// exit 1 with stable code `receipt_parse_failed`.
///
/// `routing_input` is a required non-Optional struct field in `RoutingReceipt`;
/// serde fails the parse before any semantic check is reached.
#[test]
fn verify_receipt_rejects_missing_routing_input_object_with_parse_failed() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    receipt_val.as_object_mut().unwrap().remove("routing_input");
    let broken = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("missing_routing_input_object", &broken);

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
        "verify-receipt must exit 1 when routing_input object is absent"
    );
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "receipt_parse_failed",
        "missing routing_input object must produce receipt_parse_failed; got: {:?}",
        json["code"]
    );
}

// ── 7. audit_previous_hash removal → receipt_parse_failed ────────────────────

/// Removing `audit_previous_hash` (a non-Optional committed field) from the
/// receipt must produce exit 1 with stable code `receipt_parse_failed`.
///
/// `audit_previous_hash: String` is required by serde; its absence fails the
/// parse step before any audit chain verification.
#[test]
fn verify_receipt_rejects_missing_audit_previous_hash_field_with_parse_failed() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    receipt_val
        .as_object_mut()
        .unwrap()
        .remove("audit_previous_hash");
    let broken = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("missing_audit_previous_hash", &broken);

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
        "verify-receipt must exit 1 when audit_previous_hash field is absent"
    );
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "receipt_parse_failed",
        "missing audit_previous_hash must produce receipt_parse_failed; got: {:?}",
        json["code"]
    );
}

// ── 8. Extra field inside routing_input sub-object → VERIFIED ────────────────

/// A receipt whose `routing_input` sub-object contains an unrecognised extra
/// field must still verify successfully (exit 0, result VERIFIED).
///
/// `RoutingInputEnvelope` does not carry `#[serde(deny_unknown_fields)]`, so
/// extra fields are silently ignored during deserialisation. Neither the
/// `routing_input_hash` nor the `receipt_hash` covers unknown keys (both are
/// computed from the re-serialised struct, not the raw bytes), so the verifier
/// sees the same canonical data as if the field were absent.
///
/// This test locks the tolerance contract for forward-compatible receipt
/// additions inside the `routing_input` sub-object.
#[test]
fn verify_receipt_tolerates_extra_field_in_routing_input_sub_object() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, receipt_val) = route_scenario(&s);
    assert!(route_ok, "route-case must succeed");

    // Inject an extra field into the routing_input sub-object by rebuilding
    // the JSON string with the extra key manually inserted.
    let ri = serde_json::to_string(&receipt_val["routing_input"]).unwrap();
    // ri ends with "}"; insert an extra field before the closing brace.
    let ri_with_extra = format!(
        "{},\"unknown_future_field\":\"ignored_value\"}}",
        &ri[..ri.len() - 1]
    );

    let mut rest = receipt_val.clone();
    rest.as_object_mut().unwrap().remove("routing_input");
    let rest_str = serde_json::to_string(&rest).unwrap();
    let prefix = &rest_str[..rest_str.len() - 1];
    let with_extra = format!("{},\"routing_input\":{}}}", prefix, ri_with_extra);

    let tmp = write_tmp("extra_field_routing_input", &with_extra);

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
        "verify-receipt must exit 0 when routing_input has extra unknown field; got: {:?}",
        json
    );
    assert_eq!(
        json["result"], "VERIFIED",
        "extra field in routing_input must produce VERIFIED; got: {:?}",
        json
    );
}
