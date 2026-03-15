//! Run fingerprint surface tests.
//!
//! Checks that run_pilot.sh --run-fingerprint exists, prints the required
//! sections, computes a fingerprint deterministically from protocol artifacts,
//! and that README has the required section.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_run_fingerprint_flag() {
    assert!(
        RUN_PILOT_SH.contains("--run-fingerprint"),
        "run_pilot.sh must support --run-fingerprint flag"
    );
}

#[test]
fn run_fingerprint_exits_0() {
    assert!(
        RUN_PILOT_SH.contains("--run-fingerprint") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --run-fingerprint block must exit 0"
    );
}

// ── header ────────────────────────────────────────────────────────────────────

#[test]
fn run_fingerprint_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("POSTCAD RUN FINGERPRINT"),
        "run_pilot.sh must print 'POSTCAD RUN FINGERPRINT' header"
    );
}

// ── run context section ───────────────────────────────────────────────────────

#[test]
fn run_fingerprint_has_run_context_section() {
    assert!(
        RUN_PILOT_SH.contains("RUN CONTEXT"),
        "run_pilot.sh --run-fingerprint must print 'RUN CONTEXT' section"
    );
}

#[test]
fn run_fingerprint_run_context_shows_run_id_label() {
    assert!(
        RUN_PILOT_SH.contains("RF_RUN_ID"),
        "run_pilot.sh --run-fingerprint must use RF_RUN_ID variable"
    );
}

#[test]
fn run_fingerprint_run_context_shows_receipt_path() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD RUN FINGERPRINT")
        .expect("header must exist")..];
    assert!(
        after.contains("Receipt path : examples/pilot/receipt.json"),
        "run_pilot.sh --run-fingerprint must show receipt path when receipt exists"
    );
}

#[test]
fn run_fingerprint_run_context_not_detected_fallback() {
    assert!(
        RUN_PILOT_SH.contains("not detected"),
        "run_pilot.sh --run-fingerprint must show 'not detected' fallback"
    );
}

// ── fingerprint components section ───────────────────────────────────────────

#[test]
fn run_fingerprint_has_fingerprint_components_section() {
    assert!(
        RUN_PILOT_SH.contains("FINGERPRINT COMPONENTS"),
        "run_pilot.sh --run-fingerprint must print 'FINGERPRINT COMPONENTS' section"
    );
}

#[test]
fn run_fingerprint_components_includes_receipt() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("FINGERPRINT COMPONENTS")
        .expect("section must exist")..];
    assert!(
        after.contains("receipt.json"),
        "FINGERPRINT COMPONENTS must reference receipt.json"
    );
}

#[test]
fn run_fingerprint_components_includes_inbound_reply() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("FINGERPRINT COMPONENTS")
        .expect("section must exist")..];
    assert!(
        after.contains("inbound reply"),
        "FINGERPRINT COMPONENTS must reference inbound reply"
    );
}

#[test]
fn run_fingerprint_components_includes_verification_decision() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("FINGERPRINT COMPONENTS")
        .expect("section must exist")..];
    assert!(
        after.contains("verification decision"),
        "FINGERPRINT COMPONENTS must reference verification decision"
    );
}

#[test]
fn run_fingerprint_components_includes_dispatch_packet() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("FINGERPRINT COMPONENTS")
        .expect("section must exist")..];
    assert!(
        after.contains("dispatch packet"),
        "FINGERPRINT COMPONENTS must reference dispatch packet"
    );
}

#[test]
fn run_fingerprint_components_shows_included_label() {
    assert!(
        RUN_PILOT_SH.contains("included"),
        "run_pilot.sh --run-fingerprint must show 'included' label for present artifacts"
    );
}

#[test]
fn run_fingerprint_components_shows_not_present_label() {
    assert!(
        RUN_PILOT_SH.contains("not present"),
        "run_pilot.sh --run-fingerprint must show 'not present' label for absent artifacts"
    );
}

// ── fingerprint computation ───────────────────────────────────────────────────

#[test]
fn run_fingerprint_uses_sha256_hash() {
    assert!(
        RUN_PILOT_SH.contains("hashlib.sha256"),
        "run_pilot.sh --run-fingerprint must use hashlib.sha256 for fingerprint computation"
    );
}

#[test]
fn run_fingerprint_uses_rf_fingerprint_variable() {
    assert!(
        RUN_PILOT_SH.contains("RF_FINGERPRINT"),
        "run_pilot.sh --run-fingerprint must use RF_FINGERPRINT variable"
    );
}

#[test]
fn run_fingerprint_prints_run_fingerprint_label() {
    assert!(
        RUN_PILOT_SH.contains("Run fingerprint :"),
        "run_pilot.sh --run-fingerprint must print 'Run fingerprint :' label"
    );
}

#[test]
fn run_fingerprint_has_fingerprint_section() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD RUN FINGERPRINT")
        .expect("header must exist")..];
    assert!(
        after.contains("echo \"FINGERPRINT\""),
        "run_pilot.sh --run-fingerprint must have 'FINGERPRINT' section heading"
    );
}

#[test]
fn run_fingerprint_includes_receipt_in_hash_input() {
    assert!(
        RUN_PILOT_SH.contains("RF_RECEIPT_FILE"),
        "run_pilot.sh --run-fingerprint must include RF_RECEIPT_FILE in hash computation"
    );
}

#[test]
fn run_fingerprint_includes_inbound_in_hash_input() {
    assert!(
        RUN_PILOT_SH.contains("RF_INBOUND_FILE"),
        "run_pilot.sh --run-fingerprint must include RF_INBOUND_FILE in hash computation"
    );
}

#[test]
fn run_fingerprint_includes_dispatch_in_hash_input() {
    assert!(
        RUN_PILOT_SH.contains("RF_DISPATCH_FILE"),
        "run_pilot.sh --run-fingerprint must include RF_DISPATCH_FILE in hash computation"
    );
}

#[test]
fn run_fingerprint_not_available_fallback_when_no_receipt() {
    assert!(
        RUN_PILOT_SH.contains("not available — generate a pilot bundle first"),
        "run_pilot.sh --run-fingerprint must show fallback when no receipt exists"
    );
}

// ── why this matters section ──────────────────────────────────────────────────

#[test]
fn run_fingerprint_has_why_this_matters_section() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD RUN FINGERPRINT")
        .expect("header must exist")..];
    assert!(
        after.contains("WHY THIS MATTERS"),
        "run_pilot.sh --run-fingerprint must print 'WHY THIS MATTERS' section"
    );
}

#[test]
fn run_fingerprint_why_mentions_stable_identifier() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD RUN FINGERPRINT")
        .expect("header must exist")..];
    assert!(
        after.contains("stable identifier for the workflow run"),
        "WHY THIS MATTERS must mention 'stable identifier for the workflow run'"
    );
}

#[test]
fn run_fingerprint_why_mentions_derived_from_artifacts() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD RUN FINGERPRINT")
        .expect("header must exist")..];
    assert!(
        after.contains("derived from protocol artifacts"),
        "WHY THIS MATTERS must mention 'derived from protocol artifacts'"
    );
}

#[test]
fn run_fingerprint_why_mentions_logs_tracing_audits() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD RUN FINGERPRINT")
        .expect("header must exist")..];
    assert!(
        after.contains("useful for logs, tracing, and audits"),
        "WHY THIS MATTERS must mention 'useful for logs, tracing, and audits'"
    );
}

// ── how to use section ────────────────────────────────────────────────────────

#[test]
fn run_fingerprint_has_how_to_use_section() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD RUN FINGERPRINT")
        .expect("header must exist")..];
    assert!(
        after.contains("HOW TO USE"),
        "run_pilot.sh --run-fingerprint must print 'HOW TO USE' section"
    );
}

#[test]
fn run_fingerprint_how_to_use_shows_trace_view() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD RUN FINGERPRINT")
        .expect("header must exist")..];
    assert!(
        after.contains("--trace-view"),
        "HOW TO USE must reference --trace-view"
    );
}

#[test]
fn run_fingerprint_how_to_use_shows_protocol_chain() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD RUN FINGERPRINT")
        .expect("header must exist")..];
    assert!(
        after.contains("--protocol-chain"),
        "HOW TO USE must reference --protocol-chain"
    );
}

#[test]
fn run_fingerprint_how_to_use_shows_run_summary() {
    let after = &RUN_PILOT_SH[RUN_PILOT_SH
        .find("POSTCAD RUN FINGERPRINT")
        .expect("header must exist")..];
    assert!(
        after.contains("--run-summary"),
        "HOW TO USE must reference --run-summary"
    );
}

// ── determinism: no $(date) in block ─────────────────────────────────────────

#[test]
fn run_fingerprint_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("POSTCAD RUN FINGERPRINT")
        .expect("header must exist");
    let after = &RUN_PILOT_SH[block_start..];
    let block_end = after.find("exit 0").map(|i| i + 6).unwrap_or(3000);
    let block = &after[..block_end];
    assert!(
        !block.contains("$(date"),
        "run-fingerprint block must not embed timestamps"
    );
}

// ── README ────────────────────────────────────────────────────────────────────

#[test]
fn readme_has_run_fingerprint_section() {
    assert!(
        README.contains("## Run Fingerprint"),
        "README must have '## Run Fingerprint' section"
    );
}

#[test]
fn readme_run_fingerprint_shows_command() {
    assert!(
        README.contains("--run-fingerprint"),
        "README must show --run-fingerprint command"
    );
}
