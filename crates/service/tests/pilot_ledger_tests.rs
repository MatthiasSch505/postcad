//! Trial receipt ledger tests.
//!
//! Checks that run_pilot.sh, lab_simulator.sh, and verify.sh each write
//! deterministic ledger entries with the correct event names, fields, and
//! stable output wording.
//!
//! Uses include_str! so missing files are compile errors, not runtime failures.

const RUN_PILOT_SH: &str = include_str!("../../../examples/pilot/run_pilot.sh");
const LAB_SIMULATOR_SH: &str = include_str!("../../../examples/pilot/lab_simulator.sh");
const VERIFY_SH: &str = include_str!("../../../examples/pilot/verify.sh");
const README: &str = include_str!("../../../examples/pilot/README.md");
const EXPECTED_LEDGER_FORMAT: &str =
    include_str!("../../../examples/pilot/testdata/expected_ledger_entry_format.txt");

// ── Shared ledger helper ──────────────────────────────────────────────────────

#[test]
fn run_pilot_defines_append_ledger() {
    assert!(
        RUN_PILOT_SH.contains("_append_ledger"),
        "run_pilot.sh must define/call _append_ledger"
    );
}

#[test]
fn lab_simulator_defines_append_ledger() {
    assert!(
        LAB_SIMULATOR_SH.contains("_append_ledger"),
        "lab_simulator.sh must define/call _append_ledger"
    );
}

#[test]
fn verify_sh_defines_append_ledger() {
    assert!(
        VERIFY_SH.contains("_append_ledger"),
        "verify.sh must define/call _append_ledger"
    );
}

#[test]
fn ledger_helper_uses_deterministic_sequence_numbering() {
    for script in [RUN_PILOT_SH, LAB_SIMULATOR_SH, VERIFY_SH] {
        assert!(
            script.contains("_ledger_next_seq"),
            "each script must use _ledger_next_seq for deterministic sequencing"
        );
    }
}

// ── run_pilot.sh — outbound_bundle_created ────────────────────────────────────

#[test]
fn run_pilot_writes_outbound_bundle_created_event() {
    assert!(
        RUN_PILOT_SH.contains("outbound_bundle_created"),
        "run_pilot.sh must write 'outbound_bundle_created' ledger event"
    );
}

#[test]
fn run_pilot_computes_run_id_for_ledger() {
    assert!(
        RUN_PILOT_SH.contains("PILOT_RUN_ID"),
        "run_pilot.sh must compute PILOT_RUN_ID for the ledger file name"
    );
}

#[test]
fn run_pilot_uses_reports_dir_for_ledger() {
    assert!(
        RUN_PILOT_SH.contains("REPORTS_DIR"),
        "run_pilot.sh must use REPORTS_DIR for ledger file path"
    );
}

#[test]
fn run_pilot_prints_ledger_path() {
    assert!(
        RUN_PILOT_SH.contains("Ledger:"),
        "run_pilot.sh must print the ledger file path"
    );
}

// ── lab_simulator.sh — handoff_pack_created ───────────────────────────────────

#[test]
fn lab_simulator_writes_handoff_pack_created_event() {
    assert!(
        LAB_SIMULATOR_SH.contains("handoff_pack_created"),
        "lab_simulator.sh must write 'handoff_pack_created' ledger event"
    );
}

#[test]
fn lab_simulator_writes_ledger_in_reports_dir() {
    assert!(
        LAB_SIMULATOR_SH.contains("REPORTS_DIR"),
        "lab_simulator.sh must use REPORTS_DIR for ledger path"
    );
}

#[test]
fn lab_simulator_prints_ledger_path() {
    assert!(
        LAB_SIMULATOR_SH.contains("Ledger:"),
        "lab_simulator.sh must print the ledger file path"
    );
}

// ── verify.sh --inbound — inbound + verification + decision events ─────────────

#[test]
fn verify_sh_writes_inbound_artifact_processed_event() {
    assert!(
        VERIFY_SH.contains("inbound_artifact_processed"),
        "verify.sh must write 'inbound_artifact_processed' ledger event"
    );
}

#[test]
fn verify_sh_writes_verification_recorded_event() {
    assert!(
        VERIFY_SH.contains("verification_recorded"),
        "verify.sh must write 'verification_recorded' ledger event"
    );
}

#[test]
fn verify_sh_writes_operator_decision_recorded_event() {
    assert!(
        VERIFY_SH.contains("operator_decision_recorded"),
        "verify.sh must write 'operator_decision_recorded' ledger event"
    );
}

#[test]
fn verify_sh_computes_run_id_for_ledger() {
    assert!(
        VERIFY_SH.contains("INBOUND_RUN_ID") || VERIFY_SH.contains("BATCH_RUN_ID"),
        "verify.sh must compute a run ID for the ledger file name"
    );
}

#[test]
fn verify_sh_inbound_mode_prints_ledger_path() {
    assert!(
        VERIFY_SH.contains("Ledger:"),
        "verify.sh must print the ledger file path"
    );
}

// ── verify.sh --batch-inbound — per-artifact ledger entries ───────────────────

#[test]
fn verify_sh_batch_writes_inbound_artifact_processed_per_artifact() {
    assert!(
        VERIFY_SH.contains("BATCH_LEDGER_FILE"),
        "verify.sh batch mode must use a per-run ledger file for batch entries"
    );
}

// ── ledger entry format fixture ───────────────────────────────────────────────

#[test]
fn expected_ledger_format_has_sequence_field() {
    assert!(
        EXPECTED_LEDGER_FORMAT.contains("sequence: "),
        "ledger format fixture must have 'sequence:' field"
    );
}

#[test]
fn expected_ledger_format_has_event_field() {
    assert!(
        EXPECTED_LEDGER_FORMAT.contains("event: "),
        "ledger format fixture must have 'event:' field"
    );
}

#[test]
fn expected_ledger_format_has_run_id_field() {
    assert!(
        EXPECTED_LEDGER_FORMAT.contains("run_id: "),
        "ledger format fixture must have 'run_id:' field"
    );
}

#[test]
fn expected_ledger_format_has_result_field() {
    assert!(
        EXPECTED_LEDGER_FORMAT.contains("result: "),
        "ledger format fixture must have 'result:' field"
    );
}

#[test]
fn expected_ledger_format_has_timestamp_field() {
    assert!(
        EXPECTED_LEDGER_FORMAT.contains("timestamp: "),
        "ledger format fixture must have 'timestamp:' field"
    );
}

#[test]
fn expected_ledger_format_has_all_five_event_types() {
    for event in [
        "outbound_bundle_created",
        "handoff_pack_created",
        "inbound_artifact_processed",
        "verification_recorded",
        "operator_decision_recorded",
    ] {
        assert!(
            EXPECTED_LEDGER_FORMAT.contains(event),
            "ledger format fixture must include event: {event}"
        );
    }
}

#[test]
fn expected_ledger_format_has_sequential_numbering() {
    assert!(
        EXPECTED_LEDGER_FORMAT.contains("sequence: 001")
            && EXPECTED_LEDGER_FORMAT.contains("sequence: 002")
            && EXPECTED_LEDGER_FORMAT.contains("sequence: 003")
            && EXPECTED_LEDGER_FORMAT.contains("sequence: 004")
            && EXPECTED_LEDGER_FORMAT.contains("sequence: 005"),
        "ledger format fixture must show sequential 001-005 numbering"
    );
}

// ── README documents ledger ───────────────────────────────────────────────────

#[test]
fn readme_has_trial_receipt_ledger_section() {
    assert!(
        README.contains("## Trial Receipt Ledger"),
        "README must have '## Trial Receipt Ledger' section"
    );
}

#[test]
fn readme_shows_ledger_file_path_pattern() {
    assert!(
        README.contains("ledger_<run-id>.txt"),
        "README must show the ledger file path pattern"
    );
}

#[test]
fn readme_documents_all_ledger_events() {
    for event in [
        "outbound_bundle_created",
        "handoff_pack_created",
        "inbound_artifact_processed",
        "verification_recorded",
        "operator_decision_recorded",
    ] {
        assert!(
            README.contains(event),
            "README must document ledger event: {event}"
        );
    }
}
