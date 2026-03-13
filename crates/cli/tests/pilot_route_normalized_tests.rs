//! Subprocess tests for `postcad-cli pilot-route-normalized`.
//!
//! Verifies:
//!  1. Command exits 0 and emits valid JSON with expected fields (--json mode).
//!  2. Command exits 0 and prints human-readable output (plain mode).
//!  3. receipt_hash is stable across two consecutive runs (determinism).

use std::process::Command;

use serde_json::Value;

fn bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_postcad-cli"))
}

/// Spawns `postcad-cli pilot-route-normalized [extra_args]`.
/// Returns `(exit_code, stdout_string, stderr_string)`.
fn run(extra_args: &[&str]) -> (i32, String, String) {
    let mut args = vec!["pilot-route-normalized"];
    args.extend_from_slice(extra_args);
    let out = Command::new(bin())
        .args(&args)
        .output()
        .expect("failed to spawn postcad-cli");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (code, stdout, stderr)
}

// ── Test 1: --json mode exits 0 and returns expected fields ──────────────────

#[test]
fn pilot_route_normalized_json_exits_0_and_returns_expected_fields() {
    let (code, stdout, stderr) = run(&["--json"]);

    assert_eq!(code, 0, "command must exit 0; stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr must be empty in --json mode; got: {stderr}");

    let json: Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|_| panic!("stdout was not valid JSON:\n{stdout}"));

    assert_eq!(json["result"], "VERIFIED", "result must be VERIFIED");
    assert_eq!(json["outcome"], "routed", "outcome must be routed");
    assert!(
        json["selected_candidate_id"].is_string(),
        "selected_candidate_id must be present"
    );
    assert!(
        json["receipt_hash"].is_string(),
        "receipt_hash must be present"
    );
}

// ── Test 2: plain mode exits 0 and prints human-readable output ───────────────

#[test]
fn pilot_route_normalized_plain_exits_0_and_prints_readable_output() {
    let (code, stdout, _stderr) = run(&[]);

    assert_eq!(code, 0, "command must exit 0");
    assert!(
        stdout.contains("Selected Candidate:"),
        "output must contain 'Selected Candidate:'"
    );
    assert!(
        stdout.contains("Receipt Hash:"),
        "output must contain 'Receipt Hash:'"
    );
    assert!(
        stdout.contains("Verification:"),
        "output must contain 'Verification:'"
    );
    assert!(
        stdout.contains("VERIFIED"),
        "output must contain 'VERIFIED'"
    );
}

// ── Test 3: determinism — same receipt_hash on consecutive runs ───────────────

#[test]
fn pilot_route_normalized_is_deterministic() {
    let (code1, stdout1, _) = run(&["--json"]);
    let (code2, stdout2, _) = run(&["--json"]);

    assert_eq!(code1, 0);
    assert_eq!(code2, 0);

    let j1: Value = serde_json::from_str(stdout1.trim()).unwrap();
    let j2: Value = serde_json::from_str(stdout2.trim()).unwrap();

    assert_eq!(
        j1["receipt_hash"], j2["receipt_hash"],
        "receipt_hash must be identical on consecutive runs"
    );
}
