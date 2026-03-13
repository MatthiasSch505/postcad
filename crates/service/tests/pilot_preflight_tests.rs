//! Compile-time consistency checks for the pilot preflight script.
//!
//! Follows the same pattern as demo_bundle_tests.rs: `include_str!` makes a
//! missing file a compile error, not a runtime failure.
//!
//! Checks:
//!   - preflight.sh exists and has strict mode
//!   - preflight.sh checks all required tools
//!   - preflight.sh checks all required fixture files
//!   - preflight.sh checks output directory writability
//!   - demo.sh invokes preflight.sh before proceeding

const PREFLIGHT_SH: &str = include_str!("../../../examples/pilot/preflight.sh");
const DEMO_SH: &str = include_str!("../../../examples/pilot/demo.sh");

/// preflight.sh must use strict mode (same requirement as demo.sh).
#[test]
fn preflight_has_strict_mode() {
    assert!(
        PREFLIGHT_SH.contains("set -euo pipefail"),
        "preflight.sh must use 'set -euo pipefail'"
    );
}

/// preflight.sh must check for all tools required by demo.sh.
#[test]
fn preflight_checks_required_tools() {
    for tool in ["cargo", "curl", "python3"] {
        assert!(
            PREFLIGHT_SH.contains(tool),
            "preflight.sh must check for required tool '{tool}'"
        );
    }
}

/// preflight.sh must check that all pilot fixture files exist.
#[test]
fn preflight_checks_required_fixtures() {
    for fixture in ["case.json", "registry_snapshot.json", "config.json", "demo.sh"] {
        assert!(
            PREFLIGHT_SH.contains(fixture),
            "preflight.sh must check for fixture '{fixture}'"
        );
    }
}

/// preflight.sh must check that the output directory is writable.
#[test]
fn preflight_checks_output_writability() {
    assert!(
        PREFLIGHT_SH.contains("-w "),
        "preflight.sh must check output directory writability with -w"
    );
}

/// preflight.sh must exit 0 on success and exit 1 on failure.
#[test]
fn preflight_has_explicit_exit_codes() {
    assert!(
        PREFLIGHT_SH.contains("exit 0"),
        "preflight.sh must exit 0 on success"
    );
    assert!(
        PREFLIGHT_SH.contains("exit 1"),
        "preflight.sh must exit 1 on failure"
    );
}

/// demo.sh must invoke preflight.sh so reviewers can't skip the check.
#[test]
fn demo_sh_invokes_preflight() {
    assert!(
        DEMO_SH.contains("preflight.sh"),
        "demo.sh must invoke preflight.sh"
    );
}
