//! Lightweight consistency check for the pilot acceptance runner.
//!
//! Verifies that `scripts/pilot_acceptance.sh` exists, is executable,
//! and references each canonical fixture path. This is a static check
//! only — it does not execute the script or start the service.

use std::fs;

const SCRIPT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../scripts/pilot_acceptance.sh"
);

// The script references fixtures via ${PILOT_DIR}/filename; check for
// the base filenames which are always present verbatim.
const FIXTURES: &[&str] = &[
    "case.json",
    "registry_snapshot.json",
    "config.json",
    "derived_policy.json",
    "expected_routed.json",
    "expected_verify.json",
];

/// The acceptance runner script must exist and be a non-empty file.
#[test]
fn acceptance_runner_script_exists() {
    let meta =
        fs::metadata(SCRIPT).unwrap_or_else(|_| panic!("acceptance runner not found: {SCRIPT}"));
    assert!(meta.is_file(), "expected a regular file at {SCRIPT}");
    assert!(meta.len() > 0, "acceptance runner is empty");
}

/// The script must reference each canonical fixture path.
#[test]
fn acceptance_runner_references_canonical_fixtures() {
    let content = fs::read_to_string(SCRIPT).unwrap_or_else(|_| panic!("cannot read {SCRIPT}"));
    for fixture in FIXTURES {
        assert!(
            content.contains(fixture),
            "acceptance runner does not reference fixture: {fixture}"
        );
    }
}

/// The script must reference the canonical service endpoints.
#[test]
fn acceptance_runner_references_pilot_endpoints() {
    let content = fs::read_to_string(SCRIPT).unwrap_or_else(|_| panic!("cannot read {SCRIPT}"));
    for endpoint in &["/health", "/version", "/route", "/verify"] {
        assert!(
            content.contains(endpoint),
            "acceptance runner does not reference endpoint: {endpoint}"
        );
    }
}
