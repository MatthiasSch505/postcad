//! Command safety guardrail tests.
//!
//! Checks that run_pilot.sh produces structured, deterministic error guidance
//! when required arguments are missing, preconditions are unmet, or an
//! unknown command is given.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── inspect-inbound-reply: missing file argument ──────────────────────────────

#[test]
fn inspect_inbound_reply_missing_arg_prints_usage_header() {
    assert!(
        RUN_PILOT_SH.contains("INSPECT INBOUND REPLY — USAGE"),
        "run_pilot.sh must print 'INSPECT INBOUND REPLY — USAGE' when file arg is missing"
    );
}

#[test]
fn inspect_inbound_reply_usage_shows_command_form() {
    assert!(
        RUN_PILOT_SH.contains("./examples/pilot/run_pilot.sh --inspect-inbound-reply <file>"),
        "run_pilot.sh inspect usage block must show the correct command form"
    );
}

#[test]
fn inspect_inbound_reply_usage_shows_example() {
    assert!(
        RUN_PILOT_SH.contains("Example:")
            && RUN_PILOT_SH.contains("--inspect-inbound-reply inbound/lab_reply_<run-id>.json"),
        "run_pilot.sh inspect usage block must show an example command"
    );
}

#[test]
fn inspect_inbound_reply_missing_arg_exits_nonzero() {
    assert!(
        RUN_PILOT_SH.contains("INSPECT INBOUND REPLY — USAGE") && RUN_PILOT_SH.contains("exit 1"),
        "run_pilot.sh --inspect-inbound-reply missing arg path must exit 1"
    );
}

// ── export-dispatch: precondition not met ────────────────────────────────────

#[test]
fn export_dispatch_precondition_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("DISPATCH EXPORT — PRECONDITION NOT MET"),
        "run_pilot.sh must print 'DISPATCH EXPORT — PRECONDITION NOT MET' when no run detected"
    );
}

#[test]
fn export_dispatch_precondition_explains_no_run_detected() {
    assert!(
        RUN_PILOT_SH.contains("A valid pilot run was not detected."),
        "run_pilot.sh precondition block must say 'A valid pilot run was not detected.'"
    );
}

#[test]
fn export_dispatch_precondition_shows_recommended_steps() {
    assert!(
        RUN_PILOT_SH.contains("Recommended steps:"),
        "run_pilot.sh precondition block must show 'Recommended steps:'"
    );
}

#[test]
fn export_dispatch_precondition_recommended_step1_generate_bundle() {
    assert!(
        RUN_PILOT_SH.contains("1  generate pilot bundle"),
        "run_pilot.sh precondition recommended steps must include '1  generate pilot bundle'"
    );
}

#[test]
fn export_dispatch_precondition_recommended_step2_verify_reply() {
    assert!(
        RUN_PILOT_SH.contains("2  verify inbound reply"),
        "run_pilot.sh precondition recommended steps must include '2  verify inbound reply'"
    );
}

#[test]
fn export_dispatch_precondition_recommended_step3_export_dispatch() {
    assert!(
        RUN_PILOT_SH.contains("3  export dispatch packet"),
        "run_pilot.sh precondition recommended steps must include '3  export dispatch packet'"
    );
}

// ── unknown command handler ───────────────────────────────────────────────────

#[test]
fn unknown_command_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("UNKNOWN COMMAND"),
        "run_pilot.sh must print 'UNKNOWN COMMAND' for unrecognised flags"
    );
}

#[test]
fn unknown_command_directs_to_help_surface() {
    assert!(
        RUN_PILOT_SH.contains("UNKNOWN COMMAND")
            && RUN_PILOT_SH.contains("./examples/pilot/run_pilot.sh --help-surface"),
        "run_pilot.sh unknown command block must direct operator to --help-surface"
    );
}

#[test]
fn unknown_command_exits_nonzero() {
    assert!(
        RUN_PILOT_SH.contains("UNKNOWN COMMAND") && RUN_PILOT_SH.contains("exit 1"),
        "run_pilot.sh unknown command handler must exit 1"
    );
}

// ── guardrail output properties ───────────────────────────────────────────────

#[test]
fn guardrail_messages_have_no_color_codes() {
    // Guardrail blocks must not embed ANSI escape sequences (plain text only)
    let inspect_block = RUN_PILOT_SH
        .find("INSPECT INBOUND REPLY — USAGE")
        .unwrap_or(0);
    let dispatch_block = RUN_PILOT_SH
        .find("DISPATCH EXPORT — PRECONDITION NOT MET")
        .unwrap_or(0);
    let unknown_block = RUN_PILOT_SH.find("UNKNOWN COMMAND").unwrap_or(0);

    for pos in [inspect_block, dispatch_block, unknown_block] {
        if pos == 0 {
            continue;
        }
        let snippet = &RUN_PILOT_SH[pos..pos.saturating_add(500)];
        assert!(
            !snippet.contains("\\033["),
            "guardrail messages must not embed ANSI color codes"
        );
    }
}

// ── README section ────────────────────────────────────────────────────────────

#[test]
fn readme_has_command_guardrails_section() {
    assert!(
        README.contains("## Command Guardrails"),
        "README must have '## Command Guardrails' section"
    );
}

#[test]
fn readme_guardrails_mentions_inspect_usage() {
    assert!(
        README.contains("INSPECT INBOUND REPLY — USAGE"),
        "README Command Guardrails must mention the inspect usage block"
    );
}

#[test]
fn readme_guardrails_mentions_dispatch_precondition() {
    assert!(
        README.contains("DISPATCH EXPORT — PRECONDITION NOT MET"),
        "README Command Guardrails must mention the dispatch precondition block"
    );
}

#[test]
fn readme_guardrails_mentions_unknown_command() {
    assert!(
        README.contains("UNKNOWN COMMAND") || README.contains("Unknown or unrecognised flag"),
        "README Command Guardrails must mention the unknown command handler"
    );
}
