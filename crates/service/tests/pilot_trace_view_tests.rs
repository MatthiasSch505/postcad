//! Trace view tests.
//!
//! Checks that run_pilot.sh --trace-view exists, prints a deterministic
//! 5-event workflow trace, detects artifacts correctly, and shows
//! "not yet observed" for missing artifacts.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");

// ── flag exists ───────────────────────────────────────────────────────────────

#[test]
fn run_pilot_supports_trace_view_flag() {
    assert!(
        RUN_PILOT_SH.contains("--trace-view"),
        "run_pilot.sh must support --trace-view flag"
    );
}

#[test]
fn trace_view_exits_0() {
    assert!(
        RUN_PILOT_SH.contains("--trace-view") && RUN_PILOT_SH.contains("exit 0"),
        "run_pilot.sh --trace-view must exit 0"
    );
}

// ── header ────────────────────────────────────────────────────────────────────

#[test]
fn trace_view_prints_header() {
    assert!(
        RUN_PILOT_SH.contains("PostCAD — Pilot Trace View"),
        "run_pilot.sh must print 'PostCAD — Pilot Trace View' header"
    );
}

// ── run id ────────────────────────────────────────────────────────────────────

#[test]
fn trace_view_prints_run_id_label() {
    assert!(
        RUN_PILOT_SH.contains("Run ID :"),
        "run_pilot.sh --trace-view must print 'Run ID :' line"
    );
}

#[test]
fn trace_view_run_id_fallback_not_detected() {
    assert!(
        RUN_PILOT_SH.contains("not detected"),
        "run_pilot.sh --trace-view must print 'not detected' when run ID cannot be resolved"
    );
}

// ── trace events ──────────────────────────────────────────────────────────────

#[test]
fn trace_view_has_event_1_route_decision() {
    assert!(
        RUN_PILOT_SH.contains("1  route decision generated"),
        "run_pilot.sh --trace-view must include event '1  route decision generated'"
    );
}

#[test]
fn trace_view_has_event_2_receipt_recorded() {
    assert!(
        RUN_PILOT_SH.contains("2  receipt recorded"),
        "run_pilot.sh --trace-view must include event '2  receipt recorded'"
    );
}

#[test]
fn trace_view_has_event_3_inbound_reply() {
    assert!(
        RUN_PILOT_SH.contains("3  inbound lab reply detected"),
        "run_pilot.sh --trace-view must include event '3  inbound lab reply detected'"
    );
}

#[test]
fn trace_view_has_event_4_verification() {
    assert!(
        RUN_PILOT_SH.contains("4  verification step available"),
        "run_pilot.sh --trace-view must include event '4  verification step available'"
    );
}

#[test]
fn trace_view_has_event_5_dispatch() {
    assert!(
        RUN_PILOT_SH.contains("5  dispatch export available"),
        "run_pilot.sh --trace-view must include event '5  dispatch export available'"
    );
}

// ── detected / not yet observed ───────────────────────────────────────────────

#[test]
fn trace_view_uses_detected_label() {
    assert!(
        RUN_PILOT_SH.contains("detected"),
        "run_pilot.sh --trace-view must use 'detected' label for present artifacts"
    );
}

#[test]
fn trace_view_uses_not_yet_observed_label() {
    assert!(
        RUN_PILOT_SH.contains("not yet observed"),
        "run_pilot.sh --trace-view must use 'not yet observed' label for missing artifacts"
    );
}

// ── artifact detection logic ──────────────────────────────────────────────────

#[test]
fn trace_view_detects_receipt_from_receipt_json() {
    assert!(
        RUN_PILOT_SH.contains("receipt.json") && RUN_PILOT_SH.contains("TV_RECEIPT"),
        "run_pilot.sh --trace-view must detect route/receipt events from receipt.json"
    );
}

#[test]
fn trace_view_detects_inbound_from_inbound_dir() {
    assert!(
        RUN_PILOT_SH.contains("inbound/lab_reply_") && RUN_PILOT_SH.contains("TV_INBOUND"),
        "run_pilot.sh --trace-view must detect inbound reply from inbound/lab_reply_*.json"
    );
}

#[test]
fn trace_view_detects_verification_from_reports() {
    assert!(
        RUN_PILOT_SH.contains("reports/decision_") && RUN_PILOT_SH.contains("TV_VERIFICATION"),
        "run_pilot.sh --trace-view must detect verification from reports/decision_*.txt"
    );
}

#[test]
fn trace_view_detects_dispatch_from_export_packet() {
    assert!(
        RUN_PILOT_SH.contains("export_packet.json") && RUN_PILOT_SH.contains("TV_DISPATCH"),
        "run_pilot.sh --trace-view must detect dispatch from export_packet.json"
    );
}

// ── determinism ───────────────────────────────────────────────────────────────

#[test]
fn trace_view_block_has_no_date_command() {
    let block_start = RUN_PILOT_SH
        .find("Pilot Trace View")
        .expect("trace view header must exist");
    let block_end = RUN_PILOT_SH[block_start..]
        .find("exit 0")
        .map(|i| block_start + i + 6)
        .unwrap_or(block_start + 2000);
    let block = &RUN_PILOT_SH[block_start..block_end];
    assert!(
        !block.contains("$(date"),
        "trace-view block must not embed timestamps"
    );
}

// ── README section ────────────────────────────────────────────────────────────

#[test]
fn readme_has_trace_view_section() {
    assert!(
        README.contains("## Trace View"),
        "README must have '## Trace View' section"
    );
}

#[test]
fn readme_shows_trace_view_command() {
    assert!(
        README.contains("--trace-view"),
        "README must show --trace-view command"
    );
}
