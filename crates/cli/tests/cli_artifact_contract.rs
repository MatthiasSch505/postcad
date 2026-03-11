//! Subprocess tests that freeze the exact observable shell contract for the
//! canonical demo fixture set.
//!
//! Each test spawns the real binary and asserts:
//!   - the exact exit status (0 or 1)
//!   - the exact stdout JSON value (value equality, not string equality)
//!   - stderr is empty in --json mode
//!
//! These tests are the subprocess-level complement to the library-level golden
//! tests in `golden.rs`. They lock the CLI boundary itself, not just the
//! library functions.
//!
//! Canonical demo fixture set:
//!   fixtures/case.json
//!   fixtures/candidates.json
//!   fixtures/snapshot.json           (for route-case routed)
//!   fixtures/snapshot_refusal.json   (for route-case refusal)
//!   fixtures/policy.json             (for verify-receipt)
//!   fixtures/expected_routed.json    (frozen routed receipt)
//!   fixtures/expected_refused.json   (frozen refused receipt)

use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_postcad-cli"))
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn fixture(name: &str) -> PathBuf {
    fixtures_dir().join(name)
}

fn read_fixture(name: &str) -> String {
    let path = fixture(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read fixture {}: {}", path.display(), e))
}

/// Spawns the binary with the given args. Returns `(exit_code, stdout_json, stderr_string)`.
///
/// Panics if stdout is not valid JSON, surfacing the raw bytes for diagnosis.
fn run(args: &[&str]) -> (i32, Value, String) {
    let out = Command::new(bin())
        .args(args)
        .output()
        .expect("failed to spawn postcad-cli");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let json: Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|_| panic!("stdout was not valid JSON:\n{}", stdout));
    let code = out.status.code().unwrap_or(-1);
    (code, json, stderr)
}

/// Writes content to a temp file and returns the path. Each call uses a unique
/// tag so parallel tests do not collide.
fn write_tmp(tag: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir()
        .join(format!("postcad_artifact_contract_{}.json", tag));
    std::fs::write(&path, content).expect("failed to write tmp file");
    path
}

// ── Test 28: route-case success — exact frozen receipt ────────────────────────

/// `route-case --json` on the canonical demo inputs must exit 0, emit nothing
/// to stderr, and produce stdout that is value-identical to `expected_routed.json`.
///
/// This test locks the complete receipt artifact at the CLI boundary: any change
/// to a hash field, outcome field, audit field, or routing_input field is caught.
#[test]
fn cli_route_case_demo_success_exits_0_exact_frozen_receipt() {
    let (code, actual, stderr) = run(&[
        "route-case",
        "--json",
        "--case",       fixture("case.json").to_str().unwrap(),
        "--candidates", fixture("candidates.json").to_str().unwrap(),
        "--snapshot",   fixture("snapshot.json").to_str().unwrap(),
    ]);

    assert_eq!(code, 0, "route-case success must exit 0");
    assert!(
        stderr.is_empty(),
        "stderr must be empty in --json mode; got: {:?}",
        stderr
    );

    let expected: Value = serde_json::from_str(&read_fixture("expected_routed.json"))
        .expect("expected_routed.json must be valid JSON");
    assert_eq!(
        actual, expected,
        "route-case output on canonical demo inputs must exactly match expected_routed.json"
    );
}

// ── Test 29: verify-receipt success — exact verified envelope ─────────────────

/// `verify-receipt --json` on the frozen receipt must exit 0, emit nothing to
/// stderr, and produce stdout with exactly `{"result":"VERIFIED"}`.
///
/// The single-key assertion locks the shape: no extra fields may silently
/// appear in the VERIFIED envelope.
#[test]
fn cli_verify_receipt_demo_success_exits_0_verified_envelope() {
    let (code, json, stderr) = run(&[
        "verify-receipt",
        "--json",
        "--receipt",    fixture("expected_routed.json").to_str().unwrap(),
        "--case",       fixture("case.json").to_str().unwrap(),
        "--policy",     fixture("policy.json").to_str().unwrap(),
        "--candidates", fixture("candidates.json").to_str().unwrap(),
    ]);

    assert_eq!(code, 0, "verify-receipt success must exit 0");
    assert!(
        stderr.is_empty(),
        "stderr must be empty in --json mode; got: {:?}",
        stderr
    );
    assert_eq!(json["result"], "VERIFIED", "result must be VERIFIED");

    // Exactly one key — no accidental additions to the VERIFIED envelope.
    let keys: Vec<&str> = json
        .as_object()
        .expect("VERIFIED response must be a JSON object")
        .keys()
        .map(|k| k.as_str())
        .collect();
    assert_eq!(
        keys,
        vec!["result"],
        "VERIFIED envelope must contain exactly one key; got: {:?}",
        keys
    );
}

// ── Test 30: route-case deterministic refusal — exact frozen receipt ──────────

/// `route-case --json` on the canonical refusal demo inputs must exit 0, emit
/// nothing to stderr, and produce stdout that is value-identical to
/// `expected_refused.json`.
///
/// Exit 0 is correct: a refusal is a valid domain outcome, not a CLI error.
/// The exact receipt equality locks the refusal code, refusal detail, all hash
/// fields, and the audit chain at the CLI boundary.
#[test]
fn cli_route_case_demo_refusal_exits_0_exact_frozen_receipt() {
    let (code, actual, stderr) = run(&[
        "route-case",
        "--json",
        "--case",       fixture("case.json").to_str().unwrap(),
        "--candidates", fixture("candidates.json").to_str().unwrap(),
        "--snapshot",   fixture("snapshot_refusal.json").to_str().unwrap(),
    ]);

    assert_eq!(code, 0, "route-case refusal must exit 0 (refusal is a valid domain outcome)");
    assert!(
        stderr.is_empty(),
        "stderr must be empty in --json mode; got: {:?}",
        stderr
    );

    let expected: Value = serde_json::from_str(&read_fixture("expected_refused.json"))
        .expect("expected_refused.json must be valid JSON");
    assert_eq!(
        actual, expected,
        "route-case refusal output on canonical demo inputs must exactly match expected_refused.json"
    );
}

// ── Test 31: verify-receipt failure — registry drift detected ─────────────────

/// `verify-receipt --json` on the frozen receipt with a semantically different
/// registry snapshot must exit 1 and report `registry_snapshot_hash_mismatch`.
///
/// The snapshot for `mfr-de-01` is changed from `is_eligible: true` /
/// `attestation_statuses: ["verified"]` to `is_eligible: false` /
/// `attestation_statuses: ["rejected"]`. The frozen receipt committed the
/// original hash; verification must catch the drift before the routing replay.
#[test]
fn cli_verify_receipt_demo_drift_exits_1_registry_snapshot_hash_mismatch() {
    let drifted_policy = r#"{
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
                "evidence_references": ["ISO-9001-2024"],
                "attestation_statuses": ["rejected"],
                "is_eligible": false
            }
        ]
    }"#;
    let drifted_policy_path = write_tmp("demo_drift_policy", drifted_policy);

    let (code, json, stderr) = run(&[
        "verify-receipt",
        "--json",
        "--receipt",    fixture("expected_routed.json").to_str().unwrap(),
        "--case",       fixture("case.json").to_str().unwrap(),
        "--policy",     drifted_policy_path.to_str().unwrap(),
        "--candidates", fixture("candidates.json").to_str().unwrap(),
    ]);

    assert_eq!(code, 1, "verify-receipt must exit 1 on VERIFICATION FAILED");
    assert!(
        stderr.is_empty(),
        "stderr must be empty in --json mode; got: {:?}",
        stderr
    );
    assert_eq!(
        json["result"], "VERIFICATION FAILED",
        "result must be VERIFICATION FAILED"
    );
    assert_eq!(
        json["code"], "registry_snapshot_hash_mismatch",
        "failure code must be registry_snapshot_hash_mismatch; got: {:?}",
        json["code"]
    );
}

// ── Test 32: protocol-manifest — exact frozen manifest ────────────────────────

/// `protocol-manifest --json` must exit 0, emit nothing to stderr, and produce
/// stdout that is value-identical to `fixtures/expected_manifest.json`.
///
/// Any protocol change (version bump, new committed field, new error code)
/// must update the fixture.  This test locks the CLI boundary for the manifest
/// command in the same way the route-case tests lock the receipt artifact.
#[test]
fn cli_protocol_manifest_exits_0_exact_frozen_manifest() {
    let (code, actual, stderr) = run(&["protocol-manifest", "--json"]);

    assert_eq!(code, 0, "protocol-manifest must exit 0");
    assert!(
        stderr.is_empty(),
        "stderr must be empty; got: {:?}",
        stderr
    );

    let expected: Value = serde_json::from_str(&read_fixture("expected_manifest.json"))
        .expect("expected_manifest.json must be valid JSON");
    assert_eq!(
        actual, expected,
        "protocol-manifest output must exactly match expected_manifest.json"
    );
}

/// `demo-run --json` must exit 0, emit `result == "VERIFIED"`, and produce
/// empty stderr.  The demo uses only embedded frozen fixtures so its output is
/// fully deterministic: same receipt_hash and selected_candidate_id on every run.
#[test]
fn cli_demo_run_exits_0_and_verified() {
    let (code, actual, stderr) = run(&["demo-run", "--json"]);

    assert_eq!(code, 0, "demo-run must exit 0");
    assert!(
        stderr.is_empty(),
        "stderr must be empty in --json mode; got: {:?}",
        stderr
    );
    assert_eq!(
        actual["result"], "VERIFIED",
        "demo-run must report result=VERIFIED"
    );
    assert_eq!(
        actual["outcome"], "routed",
        "demo-run frozen fixture must route successfully"
    );
    assert_eq!(
        actual["protocol_version"], "postcad-v1",
        "demo-run must report protocol_version=postcad-v1"
    );
    // receipt_hash must be a 64-character hex string (SHA-256).
    let hash = actual["receipt_hash"].as_str().unwrap_or("");
    assert_eq!(hash.len(), 64, "receipt_hash must be a 64-char hex digest");
    // selected_candidate_id must match the frozen fixture's single candidate id.
    assert_eq!(
        actual["selected_candidate_id"], "rc-de-01",
        "demo-run must select the frozen fixture's candidate"
    );
}
