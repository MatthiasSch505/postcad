//! Compile-time consistency checks for the pilot preflight script and demo scripts.
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
//!   - run_pilot.sh and verify.sh exist and have strict mode
//!   - CLI help text lists route-case-from-registry (used by run_pilot.sh)

const PREFLIGHT_SH: &str = include_str!("../../../examples/pilot/preflight.sh");
const DEMO_SH: &str = include_str!("../../../examples/pilot/demo.sh");
const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const VERIFY_SH: &str = include_str!("../../../examples/pilot/verify.sh");
const CLI_MAIN: &str = include_str!("../../cli/src/main.rs");

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

/// run_pilot.sh must exist and use strict mode.
#[test]
fn run_pilot_sh_has_strict_mode() {
    assert!(
        RUN_PILOT_SH.contains("set -euo pipefail"),
        "run_pilot.sh must use 'set -euo pipefail'"
    );
}

/// run_pilot.sh must call the route-case-from-registry subcommand.
#[test]
fn run_pilot_sh_uses_route_case_from_registry() {
    assert!(
        RUN_PILOT_SH.contains("route-case-from-registry"),
        "run_pilot.sh must call route-case-from-registry"
    );
}

/// run_pilot.sh must reference the three canonical pilot fixture files.
#[test]
fn run_pilot_sh_references_pilot_fixtures() {
    for fixture in ["case.json", "registry_snapshot.json", "config.json"] {
        assert!(
            RUN_PILOT_SH.contains(fixture),
            "run_pilot.sh must reference pilot fixture '{fixture}'"
        );
    }
}

/// verify.sh must exist and use strict mode.
#[test]
fn verify_sh_has_strict_mode() {
    assert!(
        VERIFY_SH.contains("set -euo pipefail"),
        "verify.sh must use 'set -euo pipefail'"
    );
}

/// verify.sh must call verify-receipt and pass derived_policy.json + candidates.json.
#[test]
fn verify_sh_uses_verify_receipt_with_correct_files() {
    assert!(
        VERIFY_SH.contains("verify-receipt"),
        "verify.sh must call verify-receipt subcommand"
    );
    assert!(
        VERIFY_SH.contains("derived_policy.json"),
        "verify.sh must pass derived_policy.json as --policy"
    );
    assert!(
        VERIFY_SH.contains("candidates.json"),
        "verify.sh must pass candidates.json as --candidates"
    );
}

/// CLI help must list route-case-from-registry — it is the subcommand used by run_pilot.sh.
/// A first-time reviewer running `postcad-cli help` must see it.
#[test]
fn cli_help_lists_route_case_from_registry() {
    assert!(
        CLI_MAIN.contains("route-case-from-registry"),
        "CLI help must list route-case-from-registry — it is used by run_pilot.sh"
    );
}
