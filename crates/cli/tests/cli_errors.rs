/// Subprocess integration tests for the postcad-cli error envelope.
///
/// Each test spawns the real binary and asserts that the stdout JSON matches
/// the expected stable error shape: {"outcome": "error", "code": ..., "message": ...}.
/// Routing refusals are domain outcomes (outcome = "refused"), not errors.
use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_postcad-cli"))
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn scenario(name: &str) -> PathBuf {
    fixtures_dir().join("scenarios").join(name)
}

fn run(args: &[&str]) -> (bool, Value) {
    let out = Command::new(bin())
        .args(args)
        .output()
        .expect("failed to spawn postcad-cli");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|_| panic!("stdout was not valid JSON:\n{}", stdout));
    (out.status.success(), json)
}

// ── Missing required arguments ────────────────────────────────────────────────

#[test]
fn missing_case_arg_returns_invalid_arguments_envelope() {
    let (success, json) = run(&["route-case", "--json"]);
    assert!(!success);
    assert_eq!(json["outcome"], "error");
    assert_eq!(json["code"], "invalid_arguments");
    assert!(json["message"].as_str().unwrap().contains("--case"));
}

#[test]
fn missing_candidates_arg_returns_invalid_arguments_envelope() {
    let case = scenario("routed_domestic_allowed").join("case.json");
    let (success, json) = run(&[
        "route-case",
        "--json",
        "--case",
        case.to_str().unwrap(),
    ]);
    assert!(!success);
    assert_eq!(json["outcome"], "error");
    assert_eq!(json["code"], "invalid_arguments");
    assert!(json["message"].as_str().unwrap().contains("--candidates"));
}

#[test]
fn missing_snapshot_arg_returns_invalid_arguments_envelope() {
    let case = scenario("routed_domestic_allowed").join("case.json");
    let candidates = scenario("routed_domestic_allowed").join("candidates.json");
    let (success, json) = run(&[
        "route-case",
        "--json",
        "--case",
        case.to_str().unwrap(),
        "--candidates",
        candidates.to_str().unwrap(),
    ]);
    assert!(!success);
    assert_eq!(json["outcome"], "error");
    assert_eq!(json["code"], "invalid_arguments");
    assert!(json["message"].as_str().unwrap().contains("--snapshot"));
}

// ── Unknown flag ──────────────────────────────────────────────────────────────

#[test]
fn unknown_flag_returns_invalid_arguments_envelope() {
    let (success, json) = run(&["route-case", "--json", "--unknown-flag"]);
    assert!(!success);
    assert_eq!(json["outcome"], "error");
    assert_eq!(json["code"], "invalid_arguments");
}

// ── File I/O error ────────────────────────────────────────────────────────────

#[test]
fn nonexistent_file_returns_io_error_envelope() {
    let (success, json) = run(&[
        "route-case",
        "--json",
        "--case",
        "/nonexistent/path/case.json",
        "--candidates",
        "/nonexistent/path/candidates.json",
        "--snapshot",
        "/nonexistent/path/snapshot.json",
    ]);
    assert!(!success);
    assert_eq!(json["outcome"], "error");
    assert_eq!(json["code"], "io_error");
}

// ── Parse error ───────────────────────────────────────────────────────────────

#[test]
fn malformed_case_json_returns_parse_error_envelope() {
    let invalid = fixtures_dir().join("invalid.json");
    let candidates = scenario("routed_domestic_allowed").join("candidates.json");
    let snapshot = scenario("routed_domestic_allowed").join("snapshot.json");
    let (success, json) = run(&[
        "route-case",
        "--json",
        "--case",
        invalid.to_str().unwrap(),
        "--candidates",
        candidates.to_str().unwrap(),
        "--snapshot",
        snapshot.to_str().unwrap(),
    ]);
    assert!(!success);
    assert_eq!(json["outcome"], "error");
    assert_eq!(json["code"], "parse_error");
}

// ── Refusal is a domain outcome, not an error ─────────────────────────────────

#[test]
fn refusal_returns_refused_domain_json_not_error() {
    let s = scenario("refused_no_eligible_candidates");
    let (success, json) = run(&[
        "route-case",
        "--json",
        "--case",
        s.join("case.json").to_str().unwrap(),
        "--candidates",
        s.join("candidates.json").to_str().unwrap(),
        "--snapshot",
        s.join("snapshot.json").to_str().unwrap(),
    ]);
    // Routing refusals exit 0 — they are valid domain outcomes.
    assert!(success);
    assert_eq!(json["outcome"], "refused");
    assert_ne!(json["outcome"], "error");
    // Receipt fields are present on refused outcomes.
    assert!(json["routing_proof_hash"].is_string());
    assert!(json["refusal_code"].is_string());
    assert!(json["audit_entry_hash"].is_string());
}

// ── verify-receipt ────────────────────────────────────────────────────────────

/// Route a scenario to produce a receipt, then verify it with the same inputs.
/// Exit code must be 0 and stdout must indicate success.
#[test]
fn verify_receipt_exits_0_when_verification_succeeds() {
    let s = scenario("routed_domestic_allowed");
    // Route to get the receipt JSON.
    let out = std::process::Command::new(bin())
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
        .expect("failed to spawn route-case");
    assert!(out.status.success(), "route-case must succeed");
    let receipt_json = String::from_utf8_lossy(&out.stdout).into_owned();

    // Write receipt to a temp file for the verify-receipt invocation.
    let tmp = std::env::temp_dir().join("postcad_test_verify_receipt_success.json");
    std::fs::write(&tmp, &receipt_json).expect("failed to write temp receipt");

    let (success, json) = run(&[
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
    assert!(success, "verify-receipt must exit 0 when verification succeeds");
    assert_eq!(json["result"], "VERIFIED");
}

/// Tamper one field in a valid receipt and verify it with the same inputs.
/// Exit code must be 1 and stdout must indicate failure.
#[test]
fn verify_receipt_exits_1_when_receipt_is_tampered() {
    let s = scenario("routed_domestic_allowed");
    let out = std::process::Command::new(bin())
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
        .expect("failed to spawn route-case");
    assert!(out.status.success());
    let receipt_json = String::from_utf8_lossy(&out.stdout);

    // Tamper the routing_proof_hash field.
    let mut receipt: serde_json::Value =
        serde_json::from_str(receipt_json.trim()).expect("receipt must be valid JSON");
    receipt["routing_proof_hash"] =
        serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
    let tampered = serde_json::to_string(&receipt).unwrap();

    let tmp = std::env::temp_dir().join("postcad_test_verify_receipt_tampered.json");
    std::fs::write(&tmp, &tampered).expect("failed to write temp receipt");

    let (success, json) = run(&[
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
    assert!(!success, "verify-receipt must exit 1 when receipt is tampered");
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert!(json["reason"].is_string(), "response must include a reason string");
}

/// Omitting --candidates must exit 1 with the stable invalid_arguments envelope.
#[test]
fn verify_receipt_missing_candidates_exits_1_with_invalid_arguments() {
    let s = scenario("routed_domestic_allowed");
    let tmp = std::env::temp_dir().join("postcad_test_dummy_receipt.json");
    std::fs::write(&tmp, "{}").expect("failed to write dummy receipt");

    let (success, json) = run(&[
        "verify-receipt",
        "--json",
        "--receipt",
        tmp.to_str().unwrap(),
        "--case",
        s.join("case.json").to_str().unwrap(),
        "--policy",
        s.join("policy.json").to_str().unwrap(),
        // --candidates intentionally omitted
    ]);
    assert!(!success, "verify-receipt must exit 1 when --candidates is missing");
    assert_eq!(json["outcome"], "error");
    assert_eq!(json["code"], "invalid_arguments");
    assert!(json["message"].as_str().unwrap().contains("--candidates"));
}

// ── Successful route ──────────────────────────────────────────────────────────

#[test]
fn successful_route_returns_routed_domain_json() {
    let s = scenario("routed_domestic_allowed");
    let (success, json) = run(&[
        "route-case",
        "--json",
        "--case",
        s.join("case.json").to_str().unwrap(),
        "--candidates",
        s.join("candidates.json").to_str().unwrap(),
        "--snapshot",
        s.join("snapshot.json").to_str().unwrap(),
    ]);
    assert!(success);
    assert_eq!(json["outcome"], "routed");
    // Receipt fields are present on routed outcomes.
    assert!(json["routing_proof_hash"].is_string());
    assert!(json["case_fingerprint"].is_string());
    assert!(json["audit_entry_hash"].is_string());
}

// ── Envelope contract tests ────────────────────────────────────────────────────

/// Recomputes `receipt_hash` for a tampered receipt JSON value.
///
/// Mirrors the production `hash_receipt_content` logic: serialize as compact
/// JSON (serde_json Value → BTreeMap → alphabetical keys), remove `receipt_hash`,
/// serialize again, SHA-256. Used by tests that tamper one field and then need a
/// plausible `receipt_hash` so the artifact-integrity check passes and the
/// field-specific check fires.
fn recompute_receipt_hash(receipt_val: &serde_json::Value) -> String {
    use sha2::{Digest, Sha256};
    let mut obj = receipt_val.clone();
    obj.as_object_mut().unwrap().remove("receipt_hash");
    let canonical = serde_json::to_string(&obj).unwrap();
    let digest = Sha256::digest(canonical.as_bytes());
    format!("{:x}", digest)
}

/// Helper: run route-case for a scenario directory and return (success, json).
fn route_scenario(s: &std::path::Path) -> (bool, serde_json::Value) {
    run(&[
        "route-case",
        "--json",
        "--case",
        s.join("case.json").to_str().unwrap(),
        "--candidates",
        s.join("candidates.json").to_str().unwrap(),
        "--snapshot",
        s.join("snapshot.json").to_str().unwrap(),
    ])
}

/// Helper: write content to a uniquely named temp file and return its path.
fn write_tmp(tag: &str, content: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("postcad_contract_{}.json", tag));
    std::fs::write(&path, content).expect("failed to write tmp file");
    path
}

fn is_64_char_hex(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// The routed receipt must contain all 14 stable fields with the correct shapes.
#[test]
fn route_case_routed_envelope_contract() {
    let s = scenario("routed_domestic_allowed");
    let (success, json) = route_scenario(&s);
    assert!(success, "route-case must exit 0 on a routed outcome");
    assert_eq!(json["outcome"], "routed");

    // schema_version must be the stable string "1".
    assert_eq!(json["schema_version"], "1", "schema_version must be \"1\"");

    // All hash fields must be 64-char lowercase hex strings.
    for field in &[
        "routing_proof_hash",
        "case_fingerprint",
        "policy_fingerprint",
        "candidate_pool_hash",
        "eligible_candidate_ids_hash",
        "selection_input_candidate_ids_hash",
        "audit_entry_hash",
        "audit_previous_hash",
        "receipt_hash",
    ] {
        let val = json[field].as_str().unwrap_or_else(|| {
            panic!("field '{}' must be a string, got: {:?}", field, json[field])
        });
        assert!(
            is_64_char_hex(val),
            "field '{}' must be 64 hex chars, got: {:?}",
            field,
            val
        );
    }

    // selected_candidate_id must be a non-null string on a routed outcome.
    assert!(
        json["selected_candidate_id"].is_string(),
        "selected_candidate_id must be a string on a routed outcome"
    );
    assert!(!json["selected_candidate_id"].as_str().unwrap().is_empty());

    // refusal_code must be null on a routed outcome.
    assert!(json["refusal_code"].is_null(), "refusal_code must be null on a routed outcome");

    // refusal object must be absent (null) on a routed outcome.
    assert!(
        json["refusal"].is_null(),
        "refusal must be null on a routed outcome"
    );

    // audit_seq must be a non-negative integer.
    assert!(json["audit_seq"].is_number(), "audit_seq must be a number");

    // Genesis previous_hash is 64 zeros.
    assert_eq!(
        json["audit_previous_hash"],
        "0000000000000000000000000000000000000000000000000000000000000000"
    );
}

/// The refused receipt must contain all stable fields with the correct shapes.
#[test]
fn route_case_refused_envelope_contract() {
    let s = scenario("refused_no_eligible_candidates");
    let (success, json) = route_scenario(&s);
    // Refusals exit 0 — they are valid domain outcomes.
    assert!(success, "route-case must exit 0 even on a refused outcome");
    assert_eq!(json["outcome"], "refused");

    // schema_version must be the stable string "1".
    assert_eq!(json["schema_version"], "1", "schema_version must be \"1\"");

    // All hash fields must be 64-char lowercase hex strings.
    for field in &[
        "routing_proof_hash",
        "case_fingerprint",
        "policy_fingerprint",
        "candidate_pool_hash",
        "eligible_candidate_ids_hash",
        "selection_input_candidate_ids_hash",
        "audit_entry_hash",
        "audit_previous_hash",
        "receipt_hash",
    ] {
        let val = json[field].as_str().unwrap_or_else(|| {
            panic!("field '{}' must be a string, got: {:?}", field, json[field])
        });
        assert!(
            is_64_char_hex(val),
            "field '{}' must be 64 hex chars, got: {:?}",
            field,
            val
        );
    }

    // selected_candidate_id must be null on a refused outcome.
    assert!(
        json["selected_candidate_id"].is_null(),
        "selected_candidate_id must be null on a refused outcome"
    );

    // refusal_code must be the stable string "no_eligible_candidates".
    assert_eq!(json["refusal_code"], "no_eligible_candidates");

    // refusal object must be present with required sub-fields.
    assert!(json["refusal"].is_object(), "refusal must be an object on a refused outcome");
    assert!(
        json["refusal"]["message"].is_string(),
        "refusal.message must be a string"
    );
    assert!(
        json["refusal"]["evaluated_candidate_ids"].is_array(),
        "refusal.evaluated_candidate_ids must be an array"
    );
    assert!(
        json["refusal"]["failed_constraint"].is_string(),
        "refusal.failed_constraint must be a string"
    );

    // audit_seq must be a non-negative integer.
    assert!(json["audit_seq"].is_number(), "audit_seq must be a number");

    // Genesis previous_hash is 64 zeros.
    assert_eq!(
        json["audit_previous_hash"],
        "0000000000000000000000000000000000000000000000000000000000000000"
    );
}

/// The refused_compliance_failed scenario (different case and snapshot from
/// refused_no_eligible_candidates) must produce the same structural refusal
/// envelope. This test locks the essential shape for the second refusal path
/// without duplicating the full field-by-field assertions of
/// route_case_refused_envelope_contract.
#[test]
fn route_case_refused_compliance_failed_basic_shape() {
    let s = scenario("refused_compliance_failed");
    let (success, json) = route_scenario(&s);
    assert!(success, "route-case must exit 0 on a refused outcome");
    assert_eq!(json["outcome"], "refused");
    assert_eq!(json["schema_version"], "1");
    // receipt_hash must be a 64-char hex string (artifact integrity commitment).
    assert!(is_64_char_hex(json["receipt_hash"].as_str().unwrap_or("")),
        "receipt_hash must be a 64-char hex string");
    // selected_candidate_id must be null on any refused outcome.
    assert!(json["selected_candidate_id"].is_null(),
        "selected_candidate_id must be null on a refused outcome");
    // refusal_code must be a non-null string.
    assert!(json["refusal_code"].is_string(),
        "refusal_code must be a string on a refused outcome");
    // refusal block must be present with its three required sub-fields.
    assert!(json["refusal"].is_object(), "refusal must be an object");
    assert!(json["refusal"]["message"].is_string(), "refusal.message must be a string");
    assert!(json["refusal"]["evaluated_candidate_ids"].is_array(),
        "refusal.evaluated_candidate_ids must be an array");
    assert!(json["refusal"]["failed_constraint"].is_string(),
        "refusal.failed_constraint must be a string");
}

/// The routed_cross_border_allowed scenario (different case and policy from
/// routed_domestic_allowed) must produce the same structural routed envelope.
/// This test locks the essential shape for the cross-border routing policy path
/// without duplicating the full field-by-field assertions of
/// route_case_routed_envelope_contract.
#[test]
fn route_case_routed_cross_border_basic_shape() {
    let s = scenario("routed_cross_border_allowed");
    let (success, json) = route_scenario(&s);
    assert!(success, "route-case must exit 0 on a routed outcome");
    assert_eq!(json["outcome"], "routed");
    assert_eq!(json["schema_version"], "1");
    // receipt_hash must be a 64-char hex string (artifact integrity commitment).
    assert!(is_64_char_hex(json["receipt_hash"].as_str().unwrap_or("")),
        "receipt_hash must be a 64-char hex string");
    // selected_candidate_id must be a non-empty string on any routed outcome.
    let selected = json["selected_candidate_id"].as_str()
        .expect("selected_candidate_id must be a string on a routed outcome");
    assert!(!selected.is_empty(), "selected_candidate_id must not be empty");
    // refusal_code must be null on a routed outcome.
    assert!(json["refusal_code"].is_null(), "refusal_code must be null on a routed outcome");
    // refusal block must be absent on a routed outcome.
    assert!(json["refusal"].is_null(), "refusal must be null on a routed outcome");
}

/// The verify-receipt VERIFIED envelope must be exactly {result: "VERIFIED"}.
#[test]
fn verify_receipt_verified_envelope_contract() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, receipt_json_val) = route_scenario(&s);
    assert!(route_ok, "route-case must succeed for envelope contract test");

    let receipt_content = serde_json::to_string(&receipt_json_val).unwrap();
    let tmp = write_tmp("verified_contract", &receipt_content);

    let (success, json) = run(&[
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
    assert!(success, "verify-receipt must exit 0 on VERIFIED");
    assert_eq!(json["result"], "VERIFIED");
    // The VERIFIED envelope must contain exactly one key: "result".
    // This locks the complete shape and catches any silent additions.
    let keys: Vec<&str> = json.as_object()
        .expect("VERIFIED response must be a JSON object")
        .keys()
        .map(|k| k.as_str())
        .collect();
    assert_eq!(keys, vec!["result"], "VERIFIED envelope must have exactly one key, got: {:?}", keys);
}

/// The verify-receipt VERIFICATION FAILED envelope must include result and reason,
/// and the reason must identify the tampered field.
#[test]
fn verify_receipt_failed_envelope_contract() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok, "route-case must succeed for envelope contract test");

    // Tamper routing_proof_hash and recompute receipt_hash so the artifact
    // check passes and the field-specific check fires.
    receipt_val["routing_proof_hash"] =
        serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("failed_contract", &tampered);

    let (success, json) = run(&[
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
    assert!(!success, "verify-receipt must exit 1 on VERIFICATION FAILED");
    assert_eq!(json["result"], "VERIFICATION FAILED");
    // code must be a stable machine-readable string identifying the failure.
    assert!(json["code"].is_string(), "VERIFICATION FAILED envelope must include a code field");
    // reason must be a string that identifies the tampered field.
    let reason = json["reason"].as_str().expect("reason must be a string");
    assert!(
        reason.contains("routing_proof_hash"),
        "reason must identify 'routing_proof_hash', got: {:?}",
        reason
    );
}

// ── Field semantics: route-case ───────────────────────────────────────────────

/// selected_candidate_id in a routed receipt must be one of the IDs present
/// in the candidates input file. This is a semantic cross-field consistency
/// check — not just presence.
#[test]
fn routed_selected_candidate_id_is_from_candidate_pool() {
    let s = scenario("routed_domestic_allowed");
    let (success, json) = route_scenario(&s);
    assert!(success);
    assert_eq!(json["outcome"], "routed");

    let selected = json["selected_candidate_id"]
        .as_str()
        .expect("selected_candidate_id must be a string on a routed outcome");

    let candidates_raw =
        std::fs::read_to_string(s.join("candidates.json")).expect("candidates.json must be readable");
    let candidates: Value =
        serde_json::from_str(&candidates_raw).expect("candidates.json must be valid JSON");

    let ids: Vec<&str> = candidates
        .as_array()
        .expect("candidates must be a JSON array")
        .iter()
        .filter_map(|c| c["id"].as_str())
        .collect();

    assert!(
        ids.contains(&selected),
        "selected_candidate_id '{}' must be one of the candidate IDs {:?}",
        selected,
        ids
    );
}

/// refused.evaluated_candidate_ids must be non-empty and each entry must be a
/// non-empty string. This confirms the refusal carry through the candidates
/// that were actually evaluated, not a synthetic empty list.
#[test]
fn refused_evaluated_candidate_ids_are_nonempty_strings() {
    let s = scenario("refused_no_eligible_candidates");
    let (success, json) = route_scenario(&s);
    assert!(success, "refused outcome must still exit 0");
    assert_eq!(json["outcome"], "refused");

    let evaluated = json["refusal"]["evaluated_candidate_ids"]
        .as_array()
        .expect("refusal.evaluated_candidate_ids must be an array");

    assert!(
        !evaluated.is_empty(),
        "refusal.evaluated_candidate_ids must not be empty: at least one candidate must have been evaluated"
    );
    for (i, entry) in evaluated.iter().enumerate() {
        let id = entry.as_str().unwrap_or_else(|| {
            panic!("refusal.evaluated_candidate_ids[{}] must be a string, got: {:?}", i, entry)
        });
        assert!(!id.is_empty(), "refusal.evaluated_candidate_ids[{}] must not be empty", i);
    }
}

/// A routed outcome must have no refusal fields populated, and a refused outcome
/// must have no selection fields populated. These invariants are mutually exclusive
/// by domain contract.
#[test]
fn routed_and_refused_outcome_fields_are_mutually_exclusive() {
    let routed_s = scenario("routed_domestic_allowed");
    let (routed_ok, routed_json) = route_scenario(&routed_s);
    assert!(routed_ok);
    assert_eq!(routed_json["outcome"], "routed");
    // Routed: selection present, refusal absent.
    assert!(
        routed_json["selected_candidate_id"].is_string(),
        "routed outcome must have a string selected_candidate_id"
    );
    assert!(
        routed_json["refusal_code"].is_null(),
        "routed outcome must have null refusal_code"
    );
    assert!(
        routed_json["refusal"].is_null(),
        "routed outcome must have null refusal block"
    );

    let refused_s = scenario("refused_no_eligible_candidates");
    let (refused_ok, refused_json) = route_scenario(&refused_s);
    assert!(refused_ok);
    assert_eq!(refused_json["outcome"], "refused");
    // Refused: refusal present, selection absent.
    assert!(
        refused_json["selected_candidate_id"].is_null(),
        "refused outcome must have null selected_candidate_id"
    );
    assert!(
        refused_json["refusal_code"].is_string(),
        "refused outcome must have a string refusal_code"
    );
    assert!(
        refused_json["refusal"].is_object(),
        "refused outcome must have a refusal block"
    );
}

// ── Field semantics: verify-receipt ───────────────────────────────────────────

/// verify-receipt must succeed for a refused outcome receipt, not just routed.
/// Refused receipts carry the same hash commitments and must be verifiable.
#[test]
fn verify_receipt_for_refused_outcome_exits_0_with_verified() {
    let s = scenario("refused_no_eligible_candidates");
    let (route_ok, receipt_val) = route_scenario(&s);
    assert!(route_ok, "route-case must succeed (exit 0) even for a refused outcome");

    let receipt_content = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("refused_verify", &receipt_content);

    let (success, json) = run(&[
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
    assert!(success, "verify-receipt must exit 0 for a valid refused receipt");
    assert_eq!(json["result"], "VERIFIED");
    assert!(json["reason"].is_null(), "VERIFIED envelope must not include a reason field");
}

/// Tampering candidate_pool_hash specifically must produce VERIFICATION FAILED
/// and the reason must identify "candidate_pool_hash". This is a regression
/// guard: the candidate pool commitment must be verified, not just the proof hash.
#[test]
fn candidate_pool_hash_tampering_produces_verification_failed() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    receipt_val["candidate_pool_hash"] =
        serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("pool_hash_tampered", &tampered);

    let (success, json) = run(&[
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
    assert!(!success, "verify-receipt must exit 1 when candidate_pool_hash is tampered");
    assert_eq!(json["result"], "VERIFICATION FAILED");
    let reason = json["reason"].as_str().expect("reason must be a string");
    assert!(
        reason.contains("candidate_pool_hash"),
        "reason must identify 'candidate_pool_hash', got: {:?}",
        reason
    );
}

// ── Receipt schema version ────────────────────────────────────────────────────

/// route-case --json output must include schema_version: "1" at the top level.
#[test]
fn route_case_json_output_includes_schema_version_1() {
    let s = scenario("routed_domestic_allowed");
    let (success, json) = route_scenario(&s);
    assert!(success);
    assert_eq!(json["schema_version"], "1");
}

/// verify-receipt must reject a receipt missing schema_version with the stable
/// code missing_receipt_schema_version.
#[test]
fn verify_receipt_rejects_missing_schema_version_with_stable_code() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    receipt_val.as_object_mut().unwrap().remove("schema_version");
    let tmp = write_tmp("missing_schema_version", &serde_json::to_string(&receipt_val).unwrap());

    let (success, json) = run(&[
        "verify-receipt", "--json",
        "--receipt", tmp.to_str().unwrap(),
        "--case", s.join("case.json").to_str().unwrap(),
        "--policy", s.join("policy.json").to_str().unwrap(),
        "--candidates", s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(!success);
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(json["code"], "missing_receipt_schema_version");
}

/// verify-receipt must reject a receipt with an unsupported schema_version
/// string with the stable code unsupported_receipt_schema_version.
#[test]
fn verify_receipt_rejects_unsupported_schema_version_with_stable_code() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    receipt_val["schema_version"] = serde_json::json!("2");
    let tmp = write_tmp("unsupported_schema_version", &serde_json::to_string(&receipt_val).unwrap());

    let (success, json) = run(&[
        "verify-receipt", "--json",
        "--receipt", tmp.to_str().unwrap(),
        "--case", s.join("case.json").to_str().unwrap(),
        "--policy", s.join("policy.json").to_str().unwrap(),
        "--candidates", s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(!success);
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(json["code"], "unsupported_receipt_schema_version");
    assert!(json["reason"].as_str().unwrap().contains("\"2\""));
}

/// verify-receipt must reject a receipt where schema_version is not a string
/// (malformed type) with the stable code invalid_receipt_schema_version.
#[test]
fn verify_receipt_rejects_invalid_schema_version_type_with_stable_code() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    receipt_val["schema_version"] = serde_json::json!(1); // number, not string
    let tmp = write_tmp("invalid_schema_version", &serde_json::to_string(&receipt_val).unwrap());

    let (success, json) = run(&[
        "verify-receipt", "--json",
        "--receipt", tmp.to_str().unwrap(),
        "--case", s.join("case.json").to_str().unwrap(),
        "--policy", s.join("policy.json").to_str().unwrap(),
        "--candidates", s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(!success);
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(json["code"], "invalid_receipt_schema_version");
}

// ── Stable failure code in verify-receipt JSON output ─────────────────────────

/// VERIFICATION FAILED JSON must include a stable machine-readable `code` field.
/// The `reason` field is human-readable and may change; `code` is the contract.
/// receipt_hash is recomputed so the artifact check passes and the field-specific
/// check fires.
#[test]
fn verify_receipt_failed_json_includes_stable_code_routing_proof_hash() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    receipt_val["routing_proof_hash"] =
        serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("code_routing_proof", &tampered);

    let (success, json) = run(&[
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
    assert!(!success);
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "routing_proof_hash_mismatch",
        "code must be the stable identifier for the failing check"
    );
    assert!(json["reason"].is_string(), "reason must also be present");
}

/// Same as above but for candidate_pool_hash tampering — verifies the code
/// field is field-specific, not a generic constant.
#[test]
fn verify_receipt_failed_json_includes_stable_code_candidate_pool_hash() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    receipt_val["candidate_pool_hash"] =
        serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("code_pool_hash", &tampered);

    let (success, json) = run(&[
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
    assert!(!success);
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "candidate_pool_hash_mismatch",
        "code must be field-specific, not a generic constant"
    );
    assert!(json["reason"].is_string(), "reason must also be present");
}

/// Tampering routing_proof_hash without updating receipt_hash must produce
/// receipt_hash_mismatch — the artifact-integrity layer fires first.
#[test]
fn tampering_field_without_updating_receipt_hash_yields_receipt_hash_mismatch() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    // Tamper a field but do NOT recompute receipt_hash.
    receipt_val["routing_proof_hash"] =
        serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("receipt_hash_first", &tampered);

    let (success, json) = run(&[
        "verify-receipt", "--json",
        "--receipt", tmp.to_str().unwrap(),
        "--case", s.join("case.json").to_str().unwrap(),
        "--policy", s.join("policy.json").to_str().unwrap(),
        "--candidates", s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(!success);
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(json["code"], "receipt_hash_mismatch",
        "artifact-integrity check must fire before field-specific checks");
}

/// VERIFIED result must not include a `code` field.
#[test]
fn verify_receipt_verified_json_has_no_code_field() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, receipt_val) = route_scenario(&s);
    assert!(route_ok);

    let receipt_content = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("code_verified", &receipt_content);

    let (success, json) = run(&[
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
    assert!(success);
    assert_eq!(json["result"], "VERIFIED");
    assert!(json["code"].is_null(), "VERIFIED envelope must not include a code field");
}

/// Directly zeroing the receipt_hash field (all other fields valid) must
/// produce exit 1, result "VERIFICATION FAILED", and code "receipt_hash_mismatch".
///
/// This is distinct from `tampering_field_without_updating_receipt_hash_yields_receipt_hash_mismatch`,
/// which reaches the same code via an *indirect* path (tampers another field and
/// leaves receipt_hash stale). This test targets the receipt_hash field itself.
#[test]
fn verify_receipt_rejects_zeroed_receipt_hash_field_with_stable_code() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    // Zero out receipt_hash directly; all other fields remain correct.
    receipt_val["receipt_hash"] =
        serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("zeroed_receipt_hash", &tampered);

    let (success, json) = run(&[
        "verify-receipt", "--json",
        "--receipt", tmp.to_str().unwrap(),
        "--case", s.join("case.json").to_str().unwrap(),
        "--policy", s.join("policy.json").to_str().unwrap(),
        "--candidates", s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(!success, "verify-receipt must exit 1 when receipt_hash is zeroed");
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(json["code"], "receipt_hash_mismatch");
    assert!(json["reason"].is_string(), "reason must be present");
}

/// A receipt JSON with an extra unknown top-level field must still verify
/// successfully. Extra fields are silently dropped during deserialization;
/// receipt_hash covers only the known struct fields, so the hash still matches.
///
/// This test locks the forward-compatibility behavior: unrecognised fields
/// introduced by a future schema version (or by tooling) do not break
/// verification against the current pipeline.
#[test]
fn verify_receipt_tolerates_extra_top_level_field() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    // Inject a synthetic unknown field that would appear in a hypothetical
    // future schema revision or third-party annotation.
    receipt_val["_unknown_future_field"] = serde_json::json!("should be ignored");
    let receipt_content = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("extra_field", &receipt_content);

    let (success, json) = run(&[
        "verify-receipt", "--json",
        "--receipt", tmp.to_str().unwrap(),
        "--case", s.join("case.json").to_str().unwrap(),
        "--policy", s.join("policy.json").to_str().unwrap(),
        "--candidates", s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(success, "verify-receipt must exit 0 when receipt has an extra unknown field");
    assert_eq!(json["result"], "VERIFIED");
}

/// A receipt JSON with the same semantic content but a different top-level key
/// order must still verify successfully.
///
/// serde_json uses BTreeMap (no preserve_order feature), so any JSON input is
/// normalised to alphabetical key order immediately on parse. hash_receipt_content
/// re-serialises from the RoutingReceipt struct through the same BTreeMap path,
/// so the computed hash is always identical regardless of the input key order.
///
/// This test constructs a JSON string where `schema_version` and `receipt_hash`
/// appear first (before the alphabetically-earlier `audit_*` fields), confirming
/// that the verifier is order-independent at the CLI boundary.
#[test]
fn verify_receipt_tolerates_different_key_order() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, receipt_val) = route_scenario(&s);
    assert!(route_ok);

    // Extract the two fields that will be moved to the front.
    let schema_version = receipt_val["schema_version"].to_string();
    let receipt_hash_val = receipt_val["receipt_hash"].to_string();

    // Build the remainder without those two fields (serde_json will still sort
    // the remaining keys, but schema_version and receipt_hash will now appear
    // before the alphabetically-earlier audit_* fields).
    let mut rest = receipt_val.clone();
    rest.as_object_mut().unwrap().remove("schema_version");
    rest.as_object_mut().unwrap().remove("receipt_hash");
    let rest_json = serde_json::to_string(&rest).unwrap();
    // Strip outer braces to inline as a continuation.
    let rest_inner = &rest_json[1..rest_json.len() - 1];

    let scrambled = format!(
        r#"{{"schema_version":{schema_version},"receipt_hash":{receipt_hash_val},{rest_inner}}}"#
    );

    let tmp = write_tmp("key_order", &scrambled);
    let (success, json) = run(&[
        "verify-receipt", "--json",
        "--receipt", tmp.to_str().unwrap(),
        "--case", s.join("case.json").to_str().unwrap(),
        "--policy", s.join("policy.json").to_str().unwrap(),
        "--candidates", s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(success, "verify-receipt must exit 0 when key order differs from alphabetical");
    assert_eq!(json["result"], "VERIFIED");
}

/// A receipt JSON that is pretty-printed (with newlines and indentation) must
/// still verify successfully.
///
/// serde_json::from_str is whitespace-agnostic; hash_receipt_content hashes
/// the re-serialised struct (compact JSON from BTreeMap), not the raw input
/// bytes. Insignificant whitespace in the input has no effect on the hash.
///
/// This test locks the pretty-print tolerance at the CLI boundary, ensuring
/// that receipts formatted for human readability are accepted by the verifier.
#[test]
fn verify_receipt_tolerates_pretty_printed_input() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, receipt_val) = route_scenario(&s);
    assert!(route_ok);

    // Serialize with indentation (serde_json pretty-print).
    let pretty = serde_json::to_string_pretty(&receipt_val).unwrap();
    let tmp = write_tmp("pretty_receipt", &pretty);

    let (success, json) = run(&[
        "verify-receipt", "--json",
        "--receipt", tmp.to_str().unwrap(),
        "--case", s.join("case.json").to_str().unwrap(),
        "--policy", s.join("policy.json").to_str().unwrap(),
        "--candidates", s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(success, "verify-receipt must exit 0 for a pretty-printed receipt");
    assert_eq!(json["result"], "VERIFIED");
}

/// Two separate route-case subprocess invocations for the same inputs must
/// produce identical receipt_hash values.
///
/// The unit-level test (receipt_hash_is_deterministic_for_same_inputs) covers
/// in-process determinism. This test locks the same invariant at the CLI
/// boundary: the full route-case binary, including I/O, serialisation, and
/// serde round-trips, must produce a stable receipt_hash across runs.
#[test]
fn route_case_receipt_hash_is_deterministic_across_subprocess_invocations() {
    let s = scenario("routed_domestic_allowed");
    let (ok_a, json_a) = route_scenario(&s);
    let (ok_b, json_b) = route_scenario(&s);
    assert!(ok_a && ok_b, "both route-case invocations must succeed");

    let hash_a = json_a["receipt_hash"].as_str()
        .expect("receipt_hash must be a string in first invocation");
    let hash_b = json_b["receipt_hash"].as_str()
        .expect("receipt_hash must be a string in second invocation");

    assert_eq!(
        hash_a, hash_b,
        "receipt_hash must be identical across separate route-case invocations for the same inputs"
    );
}

/// Tampering audit_entry_hash (the hash binding the routing decision to the
/// immutable audit log entry) must produce exit 1 and the stable code
/// "audit_entry_hash_mismatch".
///
/// receipt_hash is recomputed after the tamper so the artifact-integrity check
/// passes and the audit-binding check fires. This mirrors the pattern used for
/// routing_proof_hash and candidate_pool_hash tamper tests.
#[test]
fn verify_receipt_rejects_tampered_audit_entry_hash_with_stable_code() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    receipt_val["audit_entry_hash"] =
        serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("tampered_audit_entry_hash", &tampered);

    let (success, json) = run(&[
        "verify-receipt", "--json",
        "--receipt", tmp.to_str().unwrap(),
        "--case", s.join("case.json").to_str().unwrap(),
        "--policy", s.join("policy.json").to_str().unwrap(),
        "--candidates", s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(!success, "verify-receipt must exit 1 when audit_entry_hash is tampered");
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(json["code"], "audit_entry_hash_mismatch",
        "code must identify the audit binding failure");
    assert!(json["reason"].is_string(), "reason must be present");
}

/// Tampering audit_previous_hash (the chain-linkage field that binds this
/// audit entry to its predecessor) must produce exit 1 and the stable code
/// "audit_previous_hash_mismatch".
///
/// receipt_hash is recomputed after the tamper so the artifact-integrity check
/// passes and the audit-chain semantic check fires.
#[test]
fn verify_receipt_rejects_tampered_audit_previous_hash_with_stable_code() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    receipt_val["audit_previous_hash"] =
        serde_json::json!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("tampered_audit_previous_hash", &tampered);

    let (success, json) = run(&[
        "verify-receipt", "--json",
        "--receipt", tmp.to_str().unwrap(),
        "--case", s.join("case.json").to_str().unwrap(),
        "--policy", s.join("policy.json").to_str().unwrap(),
        "--candidates", s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(!success, "verify-receipt must exit 1 when audit_previous_hash is tampered");
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(json["code"], "audit_previous_hash_mismatch",
        "code must identify the audit chain-linkage failure");
    assert!(json["reason"].is_string(), "reason must be present");
}

/// A receipt with audit_entry_hash removed entirely (structurally broken audit
/// chain — missing required linkage field) must fail with the stable code
/// "receipt_parse_failed".
///
/// audit_entry_hash is a non-optional String in RoutingReceipt. Removing it
/// causes serde deserialization to fail before any semantic check fires.
#[test]
fn verify_receipt_rejects_missing_audit_entry_hash_field_with_stable_code() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    receipt_val.as_object_mut().unwrap().remove("audit_entry_hash");
    let broken = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("missing_audit_entry_hash", &broken);

    let (success, json) = run(&[
        "verify-receipt", "--json",
        "--receipt", tmp.to_str().unwrap(),
        "--case", s.join("case.json").to_str().unwrap(),
        "--policy", s.join("policy.json").to_str().unwrap(),
        "--candidates", s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(!success, "verify-receipt must exit 1 for a structurally broken receipt");
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(json["code"], "receipt_parse_failed",
        "missing required audit linkage field must produce receipt_parse_failed");
}

/// Tampering eligible_candidate_ids_hash (which commits to the sorted set of
/// candidates that survived compliance and policy filtering) must produce exit 1
/// and the stable code "eligible_candidate_ids_hash_mismatch".
///
/// In the current receipt design the eligible candidate set is stored as a
/// canonical sorted hash, not a raw array. Introducing any inconsistency —
/// including one that would arise from reordering the underlying IDs before
/// hashing — is represented by substituting a wrong hash value. receipt_hash
/// is recomputed so the artifact check passes and the field-specific check fires.
#[test]
fn verify_receipt_rejects_tampered_eligible_candidate_ids_hash_with_stable_code() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    receipt_val["eligible_candidate_ids_hash"] =
        serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("tampered_eligible_ids_hash", &tampered);

    let (success, json) = run(&[
        "verify-receipt", "--json",
        "--receipt", tmp.to_str().unwrap(),
        "--case", s.join("case.json").to_str().unwrap(),
        "--policy", s.join("policy.json").to_str().unwrap(),
        "--candidates", s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(!success, "verify-receipt must exit 1 when eligible_candidate_ids_hash is tampered");
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(json["code"], "eligible_candidate_ids_hash_mismatch",
        "code must identify the eligible candidate set commitment failure");
    assert!(json["reason"].is_string(), "reason must be present");
}

/// Tampering selection_input_candidate_ids_hash (which commits to the ordered
/// list of candidates presented to the deterministic selector) must produce
/// exit 1 and the stable code "selection_input_candidate_ids_hash_mismatch".
///
/// Unlike eligible_candidate_ids_hash, this hash is order-sensitive: any
/// reordering of the selector input produces a different value. receipt_hash
/// is recomputed so the artifact check passes and the field-specific check fires.
#[test]
fn verify_receipt_rejects_tampered_selection_input_candidate_ids_hash_with_stable_code() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok);

    receipt_val["selection_input_candidate_ids_hash"] =
        serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("tampered_selection_input_ids_hash", &tampered);

    let (success, json) = run(&[
        "verify-receipt", "--json",
        "--receipt", tmp.to_str().unwrap(),
        "--case", s.join("case.json").to_str().unwrap(),
        "--policy", s.join("policy.json").to_str().unwrap(),
        "--candidates", s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(!success, "verify-receipt must exit 1 when selection_input_candidate_ids_hash is tampered");
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(json["code"], "selection_input_candidate_ids_hash_mismatch",
        "code must identify the selector input ordering commitment failure");
    assert!(json["reason"].is_string(), "reason must be present");
}

/// Mutating `routing_input.procedure` (the domain analog of "device type")
/// without updating `routing_input_hash` must be detected as a
/// `routing_input_hash_mismatch` failure even when `receipt_hash` is recomputed.
///
/// The test:
///  1. Routes a case to produce a valid receipt.
///  2. Modifies `routing_input.procedure` to a different value.
///  3. Recomputes `receipt_hash` so the artifact-integrity check passes.
///  4. Calls `verify-receipt` and asserts failure with stable code
///     `routing_input_hash_mismatch`.
#[test]
fn verify_receipt_rejects_tampered_routing_input() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok, "route-case must succeed before tampering");

    // Mutate routing_input.procedure (analogous to "device_type" in the routing
    // input envelope) without updating routing_input_hash.
    receipt_val["routing_input"]["procedure"] = serde_json::json!("bridge");
    // Recompute receipt_hash so the artifact-integrity check passes and the
    // routing_input_hash_mismatch check fires instead.
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("tampered_routing_input", &tampered);

    let (success, json) = run(&[
        "verify-receipt", "--json",
        "--receipt", tmp.to_str().unwrap(),
        "--case", s.join("case.json").to_str().unwrap(),
        "--policy", s.join("policy.json").to_str().unwrap(),
        "--candidates", s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(!success, "verify-receipt must exit 1 when routing_input is tampered");
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "routing_input_hash_mismatch",
        "failure code must be routing_input_hash_mismatch, got: {:?}", json["code"]
    );
    let reason = json["reason"].as_str().expect("reason must be a string");
    assert!(
        reason.contains("routing_input_hash"),
        "reason must identify 'routing_input_hash', got: {:?}", reason
    );
}

/// Modifying `routing_kernel_version` to an unknown value and recomputing
/// `receipt_hash` must cause `verify-receipt` to exit 1 with stable code
/// `routing_kernel_version_mismatch`.
///
/// Test flow:
///  1. Route a case to produce a valid receipt.
///  2. Replace `routing_kernel_version` with a fictitious future version.
///  3. Recompute `receipt_hash` so the artifact-integrity check passes.
///  4. Assert `verify-receipt` fails with code `routing_kernel_version_mismatch`.
#[test]
fn verify_receipt_rejects_kernel_version_mismatch() {
    let s = scenario("routed_domestic_allowed");
    let (route_ok, mut receipt_val) = route_scenario(&s);
    assert!(route_ok, "route-case must succeed before tampering");

    receipt_val["routing_kernel_version"] = serde_json::json!("postcad-routing-v99");
    receipt_val["receipt_hash"] = serde_json::json!(recompute_receipt_hash(&receipt_val));
    let tampered = serde_json::to_string(&receipt_val).unwrap();
    let tmp = write_tmp("tampered_kernel_version", &tampered);

    let (success, json) = run(&[
        "verify-receipt", "--json",
        "--receipt", tmp.to_str().unwrap(),
        "--case", s.join("case.json").to_str().unwrap(),
        "--policy", s.join("policy.json").to_str().unwrap(),
        "--candidates", s.join("candidates.json").to_str().unwrap(),
    ]);
    assert!(!success, "verify-receipt must exit 1 on routing_kernel_version mismatch");
    assert_eq!(json["result"], "VERIFICATION FAILED");
    assert_eq!(
        json["code"], "routing_kernel_version_mismatch",
        "failure code must be routing_kernel_version_mismatch, got: {:?}", json["code"]
    );
    let reason = json["reason"].as_str().expect("reason must be a string");
    assert!(
        reason.contains("routing_kernel_version"),
        "reason must identify 'routing_kernel_version', got: {:?}", reason
    );
}
