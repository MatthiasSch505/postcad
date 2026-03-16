//! Audit receipt viewer surface tests.
//!
//! Checks that run_pilot.sh --audit-receipt-view exists, prints the required
//! sections and command references, handles missing receipt deterministically,
//! and that README has the required section.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_audit_receipt_view_flag() {
    assert!(
        RUN_PILOT_SH.contains("--audit-receipt-view"),
        "run_pilot.sh must support --audit-receipt-view flag"
    );
}

#[test]
fn audit_receipt_view_exits_0() {
    assert!(
        RUN_PILOT_SH.contains("--audit-receipt-view") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --audit-receipt-view block must exit 0"
    );
}

// ── header ────────────────────────────────────────────────────────────────────

#[test]
fn audit_receipt_view_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD AUDIT RECEIPT VIEW"),
        "run_pilot.sh must print 'POSTCAD AUDIT RECEIPT VIEW' header"
    );
}

// ── receipt status section ────────────────────────────────────────────────────

#[test]
fn audit_receipt_view_has_receipt_status_section() {
    assert!(
        RUN_PILOT_SH.contains("RECEIPT STATUS"),
        "run_pilot.sh --audit-receipt-view must print 'RECEIPT STATUS' section"
    );
}

// ── fallback when receipt missing ─────────────────────────────────────────────

#[test]
fn audit_receipt_view_fallback_when_receipt_missing() {
    assert!(
        RUN_PILOT_SH.contains("no receipt detected"),
        "run_pilot.sh --audit-receipt-view must show 'no receipt detected' fallback when receipt is absent"
    );
}

// ── receipt path shown ────────────────────────────────────────────────────────

#[test]
fn audit_receipt_view_shows_receipt_path() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD AUDIT RECEIPT VIEW")
        .expect("POSTCAD AUDIT RECEIPT VIEW header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("examples/pilot/receipt.json"),
        "audit-receipt-view block must show receipt path"
    );
}

// ── receipt summary section ───────────────────────────────────────────────────

#[test]
fn audit_receipt_view_has_receipt_summary_section() {
    assert!(
        RUN_PILOT_SH.contains("RECEIPT SUMMARY"),
        "run_pilot.sh --audit-receipt-view must print 'RECEIPT SUMMARY' section"
    );
}

// ── why this matters section ──────────────────────────────────────────────────

#[test]
fn audit_receipt_view_has_why_this_matters_section() {
    assert!(
        RUN_PILOT_SH.contains("WHY THIS MATTERS"),
        "run_pilot.sh --audit-receipt-view must print 'WHY THIS MATTERS' section"
    );
}

// ── how to use section ────────────────────────────────────────────────────────

#[test]
fn audit_receipt_view_has_how_to_use_section() {
    assert!(
        RUN_PILOT_SH.contains("HOW TO USE"),
        "run_pilot.sh --audit-receipt-view must print 'HOW TO USE' section"
    );
}

#[test]
fn audit_receipt_view_how_to_use_mentions_run_fingerprint() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD AUDIT RECEIPT VIEW")
        .expect("POSTCAD AUDIT RECEIPT VIEW header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("--run-fingerprint"),
        "audit-receipt-view HOW TO USE must mention --run-fingerprint"
    );
}

#[test]
fn audit_receipt_view_how_to_use_mentions_protocol_chain() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD AUDIT RECEIPT VIEW")
        .expect("POSTCAD AUDIT RECEIPT VIEW header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("--protocol-chain"),
        "audit-receipt-view HOW TO USE must mention --protocol-chain"
    );
}

#[test]
fn audit_receipt_view_how_to_use_mentions_lab_entrypoint() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD AUDIT RECEIPT VIEW")
        .expect("POSTCAD AUDIT RECEIPT VIEW header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("--lab-entrypoint"),
        "audit-receipt-view HOW TO USE must mention --lab-entrypoint"
    );
}

// ── README ────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_audit_receipt_view_section() {
    assert!(
        README.contains("## Audit Receipt View"),
        "README must have '## Audit Receipt View' section"
    );
}

#[test]
fn readme_audit_receipt_view_shows_command() {
    assert!(
        README.contains("--audit-receipt-view"),
        "README must show --audit-receipt-view command"
    );
}
