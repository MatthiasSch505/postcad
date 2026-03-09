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
