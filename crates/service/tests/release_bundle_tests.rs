//! Lightweight consistency tests for the pilot release bundle.
//!
//! These tests confirm that all release scripts exist and meet structural
//! requirements. Using `include_str!` means a missing file is a compile
//! error, not a runtime failure.

const START_PILOT_SH: &str = include_str!("../../../release/start_pilot.sh");
const RESET_PILOT_DATA_SH: &str = include_str!("../../../release/reset_pilot_data.sh");
const SMOKE_TEST_SH: &str = include_str!("../../../release/smoke_test.sh");
const RELEASE_README_MD: &str = include_str!("../../../release/README.md");

/// All three scripts must use strict mode.
#[test]
fn release_scripts_use_strict_mode() {
    for (name, src) in [
        ("start_pilot.sh", START_PILOT_SH),
        ("reset_pilot_data.sh", RESET_PILOT_DATA_SH),
        ("smoke_test.sh", SMOKE_TEST_SH),
    ] {
        assert!(
            src.contains("set -euo pipefail"),
            "release/{name} must contain 'set -euo pipefail'"
        );
    }
}

/// The start script must reference the service binary.
#[test]
fn start_script_references_service_binary() {
    assert!(
        START_PILOT_SH.contains("postcad-service"),
        "release/start_pilot.sh must reference postcad-service binary"
    );
}

/// The start script must announce data directories.
#[test]
fn start_script_prints_data_paths() {
    assert!(
        START_PILOT_SH.contains("data/"),
        "release/start_pilot.sh must print data directory paths"
    );
}

/// The reset script must reference each runtime data directory.
#[test]
fn reset_script_references_runtime_dirs() {
    for dir in ["cases", "receipts", "policies", "dispatch", "verification"] {
        assert!(
            RESET_PILOT_DATA_SH.contains(dir),
            "release/reset_pilot_data.sh must reference data dir '{dir}'"
        );
    }
}

/// The reset script must NOT reference canonical fixture directories.
#[test]
fn reset_script_does_not_touch_fixtures() {
    assert!(
        !RESET_PILOT_DATA_SH.contains("examples/pilot"),
        "release/reset_pilot_data.sh must not touch examples/pilot/"
    );
    assert!(
        !RESET_PILOT_DATA_SH.contains("protocol_vectors"),
        "release/reset_pilot_data.sh must not touch protocol_vectors/"
    );
}

/// The smoke test must reference all 7 required endpoints.
#[test]
fn smoke_test_references_required_endpoints() {
    let required = [
        "/health",
        "/cases",
        "/cases/",
        "/route",
        "/receipts/",
        "/dispatch/",
        "/routes",
    ];
    for endpoint in required {
        assert!(
            SMOKE_TEST_SH.contains(endpoint),
            "release/smoke_test.sh must reference endpoint '{endpoint}'"
        );
    }
}

/// The smoke test must assert VERIFIED result.
#[test]
fn smoke_test_asserts_verified_result() {
    assert!(
        SMOKE_TEST_SH.contains("VERIFIED"),
        "release/smoke_test.sh must assert VERIFIED result from dispatch verify"
    );
}

/// The smoke test must exit nonzero on failure.
#[test]
fn smoke_test_has_failure_exit() {
    assert!(
        SMOKE_TEST_SH.contains("exit 1") || SMOKE_TEST_SH.contains("exit nonzero"),
        "release/smoke_test.sh must exit nonzero on failure"
    );
}

/// The README must document the three commands in order.
#[test]
fn release_readme_documents_three_commands() {
    for cmd in ["reset_pilot_data.sh", "start_pilot.sh", "smoke_test.sh"] {
        assert!(
            RELEASE_README_MD.contains(cmd),
            "release/README.md must document command '{cmd}'"
        );
    }
}

/// The README must describe what is NOT changed.
#[test]
fn release_readme_describes_non_changes() {
    assert!(
        RELEASE_README_MD.contains("does NOT change")
            || RELEASE_README_MD.contains("does not change"),
        "release/README.md must describe what this release bundle does NOT change"
    );
}
