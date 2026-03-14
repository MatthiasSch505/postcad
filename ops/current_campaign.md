campaign name

pilot: add full external trial workflow command

objective

Add a --trial-run mode to run_pilot.sh that orchestrates the complete external
pilot workflow in a single command: route → handoff pack → simulate lab response
→ verify inbound → operator decision → ledger update. Prints clean lifecycle
status lines. No new identity model, no protocol changes.

files changed

examples/pilot/run_pilot.sh
  - added --trial-run mode block (checked on first argument)
  - orchestrates: route → _append_ledger outbound_bundle_created → handoff pack
    → simulate lab response → verify inbound (TRIAL_VERIFY_EXIT capture) →
    operator decision print → Trial run completed summary
  - prints lifecycle: Starting PostCAD trial run / Outbound bundle created /
    External handoff pack created / Simulated lab response generated /
    Inbound response verified / Operator decision: ACCEPTED|REJECTED /
    Trial ledger updated / Trial run completed
  - suppresses subprocess output (> /dev/null 2>&1) for clean display
  - exits 0 on ACCEPTED, 1 on REJECTED

examples/pilot/README.md
  - added "## Running a full PostCAD pilot trial" section with command,
    expected output, and exit code description

crates/service/tests/pilot_trial_run_tests.rs   25 new tests

commands run

cargo test --all

result

All tests pass. 25 new tests in pilot_trial_run_tests.rs.

test command

cd ~/projects/postcad && cargo test --all

commit message

pilot: add full external trial workflow command
