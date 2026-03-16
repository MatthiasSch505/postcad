//! Verification view surface tests.
//!
//! Checks that run_pilot.sh --verification-view exists, prints the required
//! sections and field labels, handles missing artifact deterministically,
//! and that README has the required section.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_verification_view_flag() {
    assert!(
        RUN_PILOT_SH.contains("--verification-view"),
        "run_pilot.sh must support --verification-view flag"
    );
}

#[test]
fn verification_view_exits_0() {
    assert!(
        RUN_PILOT_SH.contains("--verification-view") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --verification-view block must exit 0"
    );
}

// ── header ────────────────────────────────────────────────────────────────────

#[test]
fn verification_view_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD VERIFICATION VIEW"),
        "run_pilot.sh must print 'POSTCAD VERIFICATION VIEW' header"
    );
}

// ── verification status section ───────────────────────────────────────────────

#[test]
fn verification_view_has_verification_status_section() {
    assert!(
        RUN_PILOT_SH.contains("VERIFICATION STATUS"),
        "run_pilot.sh --verification-view must print 'VERIFICATION STATUS' section"
    );
}

#[test]
fn verification_view_detected_message() {
    assert!(
        RUN_PILOT_SH.contains("verification detected"),
        "run_pilot.sh --verification-view must print 'verification detected' when artifact present"
    );
}

#[test]
fn verification_view_not_detected_fallback() {
    assert!(
        RUN_PILOT_SH.contains("no verification detected"),
        "run_pilot.sh --verification-view must print 'no verification detected' fallback"
    );
}

// ── verification path shown ───────────────────────────────────────────────────

#[test]
fn verification_view_shows_verification_path() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD VERIFICATION VIEW")
        .expect("POSTCAD VERIFICATION VIEW header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("examples/pilot/reports/"),
        "verification-view block must show verification artifact path"
    );
}

// ── verification summary section ─────────────────────────────────────────────

#[test]
fn verification_view_has_verification_summary_section() {
    assert!(
        RUN_PILOT_SH.contains("VERIFICATION SUMMARY"),
        "run_pilot.sh --verification-view must print 'VERIFICATION SUMMARY' section"
    );
}

#[test]
fn verification_view_summary_has_run_id_label() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD VERIFICATION VIEW")
        .expect("header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("run id"),
        "VERIFICATION SUMMARY must include 'run id' field label"
    );
}

#[test]
fn verification_view_summary_has_decision_label() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD VERIFICATION VIEW")
        .expect("header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("decision"),
        "VERIFICATION SUMMARY must include 'decision' field label"
    );
}

#[test]
fn verification_view_summary_has_manufacturer_id_label() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD VERIFICATION VIEW")
        .expect("header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("manufacturer id"),
        "VERIFICATION SUMMARY must include 'manufacturer id' field label"
    );
}

#[test]
fn verification_view_summary_has_jurisdiction_label() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD VERIFICATION VIEW")
        .expect("header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("jurisdiction"),
        "VERIFICATION SUMMARY must include 'jurisdiction' field label"
    );
}

#[test]
fn verification_view_summary_has_reason_label() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD VERIFICATION VIEW")
        .expect("header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("reason"),
        "VERIFICATION SUMMARY must include 'reason' field label"
    );
}

// ── why this matters section ──────────────────────────────────────────────────

#[test]
fn verification_view_has_why_this_matters_section() {
    assert!(
        RUN_PILOT_SH.contains("WHY THIS MATTERS"),
        "run_pilot.sh --verification-view must print 'WHY THIS MATTERS' section"
    );
}

// ── how to use section ────────────────────────────────────────────────────────

#[test]
fn verification_view_has_how_to_use_section() {
    assert!(
        RUN_PILOT_SH.contains("HOW TO USE"),
        "run_pilot.sh --verification-view must print 'HOW TO USE' section"
    );
}

#[test]
fn verification_view_how_to_use_mentions_audit_receipt_view() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD VERIFICATION VIEW")
        .expect("header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("--audit-receipt-view"),
        "HOW TO USE must mention --audit-receipt-view"
    );
}

#[test]
fn verification_view_how_to_use_mentions_run_fingerprint() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD VERIFICATION VIEW")
        .expect("header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("--run-fingerprint"),
        "HOW TO USE must mention --run-fingerprint"
    );
}

#[test]
fn verification_view_how_to_use_mentions_protocol_chain() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD VERIFICATION VIEW")
        .expect("header must exist");
    let block = &RUN_PILOT_SH[block_start..block_start + 4000];
    assert!(
        block.contains("--protocol-chain"),
        "HOW TO USE must mention --protocol-chain"
    );
}

// ── no $(date) in block ───────────────────────────────────────────────────────

#[test]
fn verification_view_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD VERIFICATION VIEW")
        .expect("POSTCAD VERIFICATION VIEW header must exist");
    let after = &RUN_PILOT_SH[block_start..];
    let block_end = after.find("exit 0").map(|i| i + 6).unwrap_or(4000);
    let block = &after[..block_end];
    assert!(
        !block.contains("$(date"),
        "verification-view block must not embed timestamps"
    );
}

// ── README ────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_verification_view_section() {
    assert!(
        README.contains("## Verification View"),
        "README must have '## Verification View' section"
    );
}

#[test]
fn readme_verification_view_shows_command() {
    assert!(
        README.contains("--verification-view"),
        "README must show --verification-view command"
    );
}
