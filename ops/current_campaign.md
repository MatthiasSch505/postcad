campaign name

pilot: add workflow trace view for pilot runs

objective

Add a --trace-view mode to run_pilot.sh that prints a deterministic 5-event
workflow trace for the current pilot run, inferring event presence from
existing filesystem artifacts. Shell/docs/test layer only.

files changed

examples/pilot/run_pilot.sh
  - added --trace-view mode (before --simulate-inbound block)
  - resolves run_id from receipt.json (falls back to "not detected")
  - detects 5 events from filesystem artifacts:
      1 route decision generated  → receipt.json
      2 receipt recorded          → receipt.json
      3 inbound lab reply         → inbound/lab_reply_*.json
      4 verification step         → reports/decision_*.txt
      5 dispatch export           → export_packet.json
  - prints "detected" or "not yet observed" per event
  - no timestamps, no colors, exits 0

examples/pilot/README.md
  - added "## Trace View" section (before ## Inbound Reply Simulator) with:
      --trace-view command
      description: shows event trace, infers from filesystem only
      note: no commands executed, no files written

crates/service/tests/pilot_trace_view_tests.rs
  - 19 new tests covering:
    - --trace-view flag exists, exits 0
    - header "PostCAD — Pilot Trace View"
    - Run ID label, "not detected" fallback
    - all 5 events present in output
    - "detected" and "not yet observed" labels
    - artifact detection variables: TV_RECEIPT, TV_INBOUND,
      TV_VERIFICATION, TV_DISPATCH
    - no $(date) in block
    - README: section, command

commands run

cargo test --test pilot_trace_view_tests

result

All 19 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_trace_view_tests

commit message

pilot: add workflow trace view for pilot runs
