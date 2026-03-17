//! Surface tests for ops/night_tail.sh.
//!
//! Verifies repo-guard refusal, default tail behaviour, --lines N support,
//! missing-log handling, and README documentation by inspecting script and
//! README source.  No shell execution — all checks are static text assertions.

const NIGHT_TAIL_SH: &str = include_str!("../../../ops/night_tail.sh");
const README_NIGHT_MODE: &str = include_str!("../../../ops/README_night_mode.md");

mod night_tail_surface_tests {
    use super::{NIGHT_TAIL_SH, README_NIGHT_MODE};

    // ── safety / preamble ───────────────────────────────────────────────────

    #[test]
    fn uses_set_euo_pipefail() {
        assert!(
            NIGHT_TAIL_SH.contains("set -euo pipefail"),
            "script must use 'set -euo pipefail'"
        );
    }

    #[test]
    fn is_read_only_no_file_writes() {
        assert!(
            !NIGHT_TAIL_SH.contains("> \"${REPO_ROOT}"),
            "script must not write files into the repo"
        );
    }

    #[test]
    fn has_no_network_calls() {
        for cmd in &["curl", "wget", "ssh", "scp"] {
            assert!(
                !NIGHT_TAIL_SH.contains(cmd),
                "script must not make network calls; found forbidden command: {}",
                cmd
            );
        }
    }

    #[test]
    fn has_no_tmux_management() {
        assert!(
            !NIGHT_TAIL_SH.contains("tmux"),
            "script must not perform tmux management"
        );
    }

    // ── repo guard ──────────────────────────────────────────────────────────

    #[test]
    fn refuses_outside_postcad_repo_checks_marker() {
        assert!(
            NIGHT_TAIL_SH.contains("Post-CAD Layer"),
            "script must check for 'Post-CAD Layer' marker to refuse non-PostCAD repos"
        );
    }

    #[test]
    fn checks_claude_md_for_repo_identity() {
        assert!(
            NIGHT_TAIL_SH.contains("CLAUDE.md"),
            "script must check CLAUDE.md for repo identity"
        );
    }

    #[test]
    fn checks_git_repo_with_rev_parse() {
        assert!(
            NIGHT_TAIL_SH.contains("rev-parse --git-dir"),
            "script must verify it is inside a git repo via 'rev-parse --git-dir'"
        );
    }

    #[test]
    fn refusal_prints_to_stderr() {
        assert!(
            NIGHT_TAIL_SH.contains(">&2"),
            "refusal messages must be printed to stderr"
        );
    }

    #[test]
    fn refusal_exits_nonzero() {
        let guard_idx = NIGHT_TAIL_SH
            .find("Post-CAD Layer")
            .expect("repo guard must exist");
        let block = &NIGHT_TAIL_SH[guard_idx..guard_idx + 300];
        assert!(
            block.contains("exit 1"),
            "repo guard must exit 1 on refusal"
        );
    }

    // ── default behaviour ───────────────────────────────────────────────────

    #[test]
    fn uses_tail_command() {
        assert!(
            NIGHT_TAIL_SH.contains("tail -n"),
            "script must use 'tail -n' to output log lines"
        );
    }

    #[test]
    fn reads_queue_status_log() {
        assert!(
            NIGHT_TAIL_SH.contains("queue_status.log"),
            "script must target ops/queue_status.log"
        );
    }

    #[test]
    fn has_default_line_count() {
        assert!(
            NIGHT_TAIL_SH.contains("LINES=20"),
            "script must default to LINES=20"
        );
    }

    #[test]
    fn prints_header_line() {
        assert!(
            NIGHT_TAIL_SH.contains("night_tail"),
            "script must print a header identifying itself"
        );
    }

    // ── --lines N support ───────────────────────────────────────────────────

    #[test]
    fn accepts_lines_flag() {
        assert!(
            NIGHT_TAIL_SH.contains("--lines"),
            "script must accept --lines flag"
        );
    }

    #[test]
    fn lines_flag_shifts_argument() {
        let idx = NIGHT_TAIL_SH
            .find("--lines)")
            .expect("--lines case must exist");
        let block = &NIGHT_TAIL_SH[idx..idx + 200];
        assert!(
            block.contains("shift"),
            "--lines handler must shift the argument after consuming the value"
        );
    }

    #[test]
    fn lines_flag_validates_integer() {
        let idx = NIGHT_TAIL_SH
            .find("--lines)")
            .expect("--lines case must exist");
        let block = &NIGHT_TAIL_SH[idx..idx + 300];
        assert!(
            block.contains("[0-9]"),
            "--lines handler must validate that the value is a positive integer"
        );
    }

    #[test]
    fn unknown_arg_triggers_usage() {
        assert!(
            NIGHT_TAIL_SH.contains("unknown argument"),
            "script must print 'unknown argument' for unrecognised flags"
        );
    }

    // ── missing log file handling ───────────────────────────────────────────

    #[test]
    fn checks_log_file_exists() {
        assert!(
            NIGHT_TAIL_SH.contains("! -f"),
            "script must check whether the log file exists with '! -f'"
        );
    }

    #[test]
    fn missing_log_prints_clear_message() {
        assert!(
            NIGHT_TAIL_SH.contains("log file not found") || NIGHT_TAIL_SH.contains("not found"),
            "script must print a clear message when the log file is missing"
        );
    }

    #[test]
    fn missing_log_exits_nonzero() {
        // Anchor on the log-specific missing-file message, not the CLAUDE.md guard.
        // Clamp window to avoid overshooting the end of the script.
        let idx = NIGHT_TAIL_SH
            .find("log file not found")
            .expect("missing-file message must exist");
        let end = (idx + 150).min(NIGHT_TAIL_SH.len());
        let block = &NIGHT_TAIL_SH[idx..end];
        assert!(
            block.contains("exit 1"),
            "script must exit 1 when the log file is missing"
        );
    }

    #[test]
    fn missing_log_message_goes_to_stderr() {
        let idx = NIGHT_TAIL_SH
            .find("log file not found")
            .expect("missing-file message must exist");
        let end = (idx + 150).min(NIGHT_TAIL_SH.len());
        let block = &NIGHT_TAIL_SH[idx..end];
        assert!(
            block.contains(">&2"),
            "missing-log error message must be sent to stderr"
        );
    }

    // ── README documentation ────────────────────────────────────────────────

    #[test]
    fn readme_night_mode_mentions_night_tail_sh() {
        assert!(
            README_NIGHT_MODE.contains("night_tail.sh"),
            "ops/README_night_mode.md must mention night_tail.sh"
        );
    }

    #[test]
    fn readme_night_mode_documents_lines_flag() {
        assert!(
            README_NIGHT_MODE.contains("--lines"),
            "ops/README_night_mode.md must document the --lines flag"
        );
    }

    #[test]
    fn readme_night_mode_explains_refusal_behaviour() {
        assert!(
            README_NIGHT_MODE.contains("Refuses") || README_NIGHT_MODE.contains("refuses"),
            "ops/README_night_mode.md must document that the script refuses outside the repo"
        );
    }

    #[test]
    fn readme_night_mode_mentions_missing_log_behaviour() {
        assert!(
            README_NIGHT_MODE.contains("missing") || README_NIGHT_MODE.contains("non-zero"),
            "ops/README_night_mode.md must mention missing-log exit behaviour"
        );
    }
}
