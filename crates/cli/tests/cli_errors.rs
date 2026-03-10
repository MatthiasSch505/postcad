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
    // The VERIFIED envelope must not include extraneous fields.
    assert!(json["reason"].is_null(), "VERIFIED envelope must not include a reason field");
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
