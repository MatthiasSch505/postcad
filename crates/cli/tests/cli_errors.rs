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
    assert!(json.get("audit").is_some());
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
    assert!(json["audit"]["proof"]["hash_hex"].is_string());
}
