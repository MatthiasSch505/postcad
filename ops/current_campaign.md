campaign name

pilot: add default inbound path resolution for current run

objective

Add auto-resolution of the inbound reply path in both run_pilot.sh
(--inspect-inbound-reply with no argument) and verify.sh (--bundle with no
--inbound). Both scripts compute the expected inbound path from receipt.json
and use it automatically if the file exists, or print structured
"INBOUND REPLY NOT FOUND" guidance if it does not. Shell/docs/test layer only.

files changed

examples/pilot/run_pilot.sh
  - --inspect-inbound-reply: if no file argument given, reads receipt.json,
    extracts case_id (fallback: first 12 chars of receipt_hash) as _AR_RUN_ID,
    builds _AR_CANDIDATE = inbound/lab_reply_<run-id>.json
  - if candidate exists: sets INBOUND_FILE and prints "auto-resolved inbound reply: <path>"
  - if receipt exists but candidate missing: prints INBOUND REPLY NOT FOUND
    with "Current run :", "Expected :", and --simulate-inbound guidance, exits 1
  - if no receipt: falls back to original INSPECT INBOUND REPLY — USAGE block

examples/pilot/verify.sh
  - added BUNDLE_EXPLICIT=false variable declaration
  - set BUNDLE_EXPLICIT=true when --bundle argument is parsed
  - new block after arg-parse loop: if MODE==receipt and BUNDLE_EXPLICIT==true,
    reads bundle receipt.json, computes _VR_RUN_ID, builds _VR_CANDIDATE
  - if candidate exists: switches MODE=inbound, INBOUND_FILE=<path>,
    prints "auto-resolved inbound reply: <path>"
  - if candidate missing: prints INBOUND REPLY NOT FOUND with guidance, exits 1

examples/pilot/README.md
  - added "## Default Inbound Path Resolution" section (before ## Trace View):
    - describes auto-resolution for --inspect-inbound-reply (no file arg)
    - describes auto-resolution for verify.sh --bundle (no --inbound)
    - shows INBOUND REPLY NOT FOUND message with example
    - shows --simulate-inbound as the recommended next step

crates/service/tests/pilot_inbound_default_path_tests.rs
  - 20 new tests covering:
    - run_pilot.sh: _AR_CASE_ID, _AR_RECEIPT_HASH, _AR_RUN_ID, _AR_CANDIDATE variables
    - run_pilot.sh: inbound/lab_reply_ path pattern
    - run_pilot.sh: "auto-resolved inbound reply" print
    - run_pilot.sh: INBOUND REPLY NOT FOUND, "Current run :", "Expected    :" labels
    - run_pilot.sh: --simulate-inbound suggestion in not-found guidance
    - verify.sh: BUNDLE_EXPLICIT flag, BUNDLE_EXPLICIT=true, _VR_CANDIDATE
    - verify.sh: "auto-resolved inbound reply", INBOUND REPLY NOT FOUND, --simulate-inbound
    - README: section present, --inspect-inbound-reply, --bundle, INBOUND REPLY NOT FOUND

commands run

cargo test --test pilot_inbound_default_path_tests

result

All 20 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_inbound_default_path_tests

commit message

pilot: add default inbound path resolution for current run
