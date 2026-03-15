campaign name

pilot: add inbound reply simulator for demo workflow

objective

Add a --simulate-inbound mode to run_pilot.sh that copies a deterministic
template reply into the inbound directory, enabling end-to-end demo without
a real external lab. Shell/testdata/docs/test layer only.

files changed

examples/pilot/run_pilot.sh
  - added --simulate-inbound mode (before --demo-surface block)
  - reads template from testdata/lab_reply_simulated.json (errors if missing)
  - if receipt.json present: resolves run_id, writes inbound/lab_reply_<run-id>.json
  - if no receipt: writes inbound/lab_reply_simulated.json
  - prints SIMULATED LAB REPLY GENERATED with File and Next step labels
  - next step shows both inspect and verify commands
  - exits 0 on success, exits 1 if template missing

examples/pilot/testdata/lab_reply_simulated.json (new)
  - minimal valid reply with all required fields:
    lab_response_schema, receipt_hash, dispatch_id, case_id,
    lab_acknowledged_at, lab_id (lab-simulator-001), status (accepted)
  - consistent with lab_reply_filled.json and lab_response_valid.json structure

examples/pilot/README.md
  - added "## Inbound Reply Simulator" section (before ## Demo Surface) with:
      --simulate-inbound command
      naming behavior (run-id vs simulated fallback)
      template reference
      note on next steps

crates/service/tests/pilot_inbound_simulator_tests.rs
  - 22 new tests covering:
    - --simulate-inbound flag exists, exits 0
    - template referenced in script
    - template fields: receipt_hash, lab_id, status, lab_acknowledged_at,
      lab_response_schema; status is "accepted"; valid JSON structure
    - output: SIMULATED LAB REPLY GENERATED, File label, Next step label,
      inspect/verify mentions
    - run-id named file when run exists, fallback filename without run
    - error if template missing
    - no $(date) in output block
    - README: section, command, template mention

commands run

cargo test --test pilot_inbound_simulator_tests

result

All 22 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_inbound_simulator_tests

commit message

pilot: add inbound reply simulator for demo workflow
