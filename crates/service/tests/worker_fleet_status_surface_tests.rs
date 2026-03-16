//! Worker fleet status surface tests.
//!
//! Verifies that ops/worker_fleet_status.sh implements the required sections,
//! worker path names, repo-guard refusal, --base-dir handling, missing-worker
//! behavior, and README documentation by inspecting script and README source.

const FLEET_STATUS_SH: &str = include_str!("../../../ops/worker_fleet_status.sh");
const OPS_README: &str = include_str!("../../../ops/README.md");

// ── repo guard ─────────────────────────────────────────────────────────────────

#[test]
fn fleet_status_refuses_outside_postcad_repo() {
    assert!(
        FLEET_STATUS_SH.contains("Post-CAD Layer"),
        "script must check for Post-CAD Layer marker to refuse non-PostCAD repos"
    );
}

#[test]
fn fleet_status_checks_claude_md_marker() {
    assert!(
        FLEET_STATUS_SH.contains("CLAUDE.md"),
        "script must check CLAUDE.md for repo identity"
    );
}

#[test]
fn fleet_status_refuses_non_git_repo() {
    assert!(
        FLEET_STATUS_SH.contains("rev-parse --git-dir"),
        "script must check for git repo before proceeding"
    );
}

#[test]
fn fleet_status_refusal_prints_error_to_stderr() {
    assert!(
        FLEET_STATUS_SH.contains(">&2"),
        "refusal messages must be printed to stderr"
    );
}

// ── --base-dir argument ────────────────────────────────────────────────────────

#[test]
fn fleet_status_accepts_base_dir_argument() {
    assert!(
        FLEET_STATUS_SH.contains("--base-dir"),
        "script must accept --base-dir argument"
    );
}

#[test]
fn fleet_status_base_dir_has_default() {
    assert!(
        FLEET_STATUS_SH.contains("HOME}/workers") || FLEET_STATUS_SH.contains("~/workers"),
        "script must default BASE_DIR to ~/workers"
    );
}

#[test]
fn fleet_status_base_dir_requires_value() {
    // The script must check that --base-dir has a value argument (case block)
    let idx = FLEET_STATUS_SH
        .find("--base-dir)")
        .expect("--base-dir case entry must exist");
    let block = &FLEET_STATUS_SH[idx..idx + 300];
    assert!(
        block.contains("requires a path argument") || block.contains("shift 2"),
        "--base-dir must require a value and shift arguments"
    );
}

// ── worker path names ──────────────────────────────────────────────────────────

#[test]
fn fleet_status_uses_worker_a_path() {
    assert!(
        FLEET_STATUS_SH.contains("postcad-worker-a"),
        "script must use 'postcad-worker-a' as worker A path name"
    );
}

#[test]
fn fleet_status_uses_worker_b_path() {
    assert!(
        FLEET_STATUS_SH.contains("postcad-worker-b"),
        "script must use 'postcad-worker-b' as worker B path name"
    );
}

// ── output sections ────────────────────────────────────────────────────────────

#[test]
fn fleet_status_prints_header() {
    assert!(
        FLEET_STATUS_SH.contains("POSTCAD WORKER FLEET STATUS"),
        "script must print 'POSTCAD WORKER FLEET STATUS' header"
    );
}

#[test]
fn fleet_status_has_repo_section() {
    assert!(
        FLEET_STATUS_SH.contains("echo \"REPO\""),
        "script must print 'REPO' section"
    );
}

#[test]
fn fleet_status_has_base_dir_section() {
    assert!(
        FLEET_STATUS_SH.contains("echo \"BASE DIR\""),
        "script must print 'BASE DIR' section"
    );
}

#[test]
fn fleet_status_has_worker_a_section() {
    assert!(
        FLEET_STATUS_SH.contains("WORKER A"),
        "script must print 'WORKER A' section"
    );
}

#[test]
fn fleet_status_has_worker_b_section() {
    assert!(
        FLEET_STATUS_SH.contains("WORKER B"),
        "script must print 'WORKER B' section"
    );
}

#[test]
fn fleet_status_has_next_section() {
    assert!(
        FLEET_STATUS_SH.contains("echo \"NEXT\""),
        "script must print 'NEXT' section"
    );
}

// ── per-worker status fields ───────────────────────────────────────────────────

#[test]
fn fleet_status_reports_worker_path() {
    assert!(
        FLEET_STATUS_SH.contains("Path   :"),
        "script must report path for each worker"
    );
}

#[test]
fn fleet_status_reports_worker_branch() {
    assert!(
        FLEET_STATUS_SH.contains("Branch :"),
        "script must report branch for each worker"
    );
}

#[test]
fn fleet_status_reports_worker_state() {
    assert!(
        FLEET_STATUS_SH.contains("State  :"),
        "script must report clean/dirty state for each worker"
    );
}

// ── missing worker handling ────────────────────────────────────────────────────

#[test]
fn fleet_status_handles_missing_worker_path() {
    assert!(
        FLEET_STATUS_SH.contains("missing"),
        "script must report 'missing' when a worker path does not exist"
    );
}

#[test]
fn fleet_status_does_not_fail_on_missing_path() {
    // The script must use [[ ! -e "$wpath" ]] guard rather than hard-failing
    assert!(
        FLEET_STATUS_SH.contains("! -e"),
        "script must handle missing paths without failing (use -e check)"
    );
}

#[test]
fn fleet_status_suggests_bootstrap_when_worker_missing() {
    assert!(
        FLEET_STATUS_SH.contains("setup_two_worker_fleet.sh"),
        "NEXT section must suggest running setup_two_worker_fleet.sh when workers are missing"
    );
}

// ── git worktree detection ─────────────────────────────────────────────────────

#[test]
fn fleet_status_detects_registered_worktree() {
    assert!(
        FLEET_STATUS_SH.contains("registered worktree"),
        "script must report 'registered worktree' when worker is a known worktree"
    );
}

#[test]
fn fleet_status_reports_non_registered_git_repo() {
    assert!(
        FLEET_STATUS_SH.contains("not a registered worktree"),
        "script must report when path is a git repo but not a registered worktree"
    );
}

#[test]
fn fleet_status_reports_non_worktree_path() {
    assert!(
        FLEET_STATUS_SH.contains("not a git worktree"),
        "script must report when path exists but is not a git worktree at all"
    );
}

// ── clean/dirty detection ─────────────────────────────────────────────────────

#[test]
fn fleet_status_reports_clean_state() {
    assert!(
        FLEET_STATUS_SH.contains("clean"),
        "script must report 'clean' when working tree has no changes"
    );
}

#[test]
fn fleet_status_reports_dirty_state() {
    assert!(
        FLEET_STATUS_SH.contains("dirty"),
        "script must report 'dirty' when working tree has uncommitted changes"
    );
}

// ── claude availability ────────────────────────────────────────────────────────

#[test]
fn fleet_status_checks_claude_in_path() {
    assert!(
        FLEET_STATUS_SH.contains("command -v claude"),
        "script must check whether 'claude' command is available in PATH"
    );
}

#[test]
fn fleet_status_does_not_invoke_claude() {
    // Must check availability only — not run it
    // The only 'claude' usage should be in the 'command -v' check and echo lines
    let lines_with_bare_claude: Vec<&str> = FLEET_STATUS_SH
        .lines()
        .filter(|l| {
            let trimmed = l.trim();
            // Bare invocation: line that is just "claude" or starts with "claude " (not in echo/comment/command -v)
            !trimmed.starts_with('#')
                && !trimmed.contains("echo")
                && !trimmed.contains("command -v")
                && (trimmed == "claude" || trimmed.starts_with("claude "))
        })
        .collect();
    assert!(
        lines_with_bare_claude.is_empty(),
        "script must not invoke claude directly; found: {:?}",
        lines_with_bare_claude
    );
}

// ── NEXT section content ───────────────────────────────────────────────────────

#[test]
fn fleet_status_next_shows_enter_worker_a_command() {
    assert!(
        FLEET_STATUS_SH.contains("Enter worker A"),
        "NEXT section must show how to enter worker A"
    );
}

#[test]
fn fleet_status_next_shows_enter_worker_b_command() {
    assert!(
        FLEET_STATUS_SH.contains("Enter worker B"),
        "NEXT section must show how to enter worker B"
    );
}

#[test]
fn fleet_status_next_shows_claude_launch_command() {
    // The NEXT section must show cd ... && claude
    let next_idx = FLEET_STATUS_SH
        .find("NEXT")
        .expect("NEXT section must exist");
    let next_block = &FLEET_STATUS_SH[next_idx..];
    assert!(
        next_block.contains("&& claude"),
        "NEXT section must show 'cd ... && claude' launch command"
    );
}

// ── determinism / read-only ────────────────────────────────────────────────────

#[test]
fn fleet_status_has_no_date_command() {
    assert!(
        !FLEET_STATUS_SH.contains("$(date"),
        "script must not embed timestamps via $(date"
    );
}

#[test]
fn fleet_status_does_not_write_files() {
    // Must not redirect output to files in the repo
    assert!(
        !FLEET_STATUS_SH.contains("> \"${REPO_ROOT}"),
        "script must not write files to the repo"
    );
}

#[test]
fn fleet_status_does_not_modify_git_state() {
    // Must not run git commit, git push, git merge, git branch -D, git reset
    for forbidden in &["git commit", "git push", "git merge", "git branch -D", "git reset"] {
        assert!(
            !FLEET_STATUS_SH.contains(forbidden),
            "script must not modify git state; found forbidden command: {}",
            forbidden
        );
    }
}

#[test]
fn fleet_status_uses_set_euo_pipefail() {
    assert!(
        FLEET_STATUS_SH.contains("set -euo pipefail"),
        "script must use 'set -euo pipefail'"
    );
}

// ── README ─────────────────────────────────────────────────────────────────────

#[test]
fn ops_readme_mentions_fleet_status_script() {
    assert!(
        OPS_README.contains("worker_fleet_status.sh"),
        "ops/README.md must mention worker_fleet_status.sh"
    );
}

#[test]
fn ops_readme_has_fleet_status_usage() {
    assert!(
        OPS_README.contains("--base-dir") || OPS_README.contains("worker_fleet_status"),
        "ops/README.md must document fleet status usage"
    );
}
