campaign name

pilot: harden verification operator verdict output

objective

Add a deterministic operator verdict block to verify.sh so the operator
gets an unambiguous VERIFICATION PASSED / VERIFICATION FAILED summary with
actionable next-step guidance after every verification run. Human inspection
layer only — no change to verification logic or exit codes.

files changed

examples/pilot/verify.sh
  - added verdict block to --inbound mode (after operator decision print):
      ════════════════════════════════════════
      VERIFICATION PASSED / VERIFICATION FAILED
      ════════════════════════════════════════
      Inbound : <file>
      Bundle  : <dir>
      Result  : verification passed / failed
      Next    : <guidance>
      ════════════════════════════════════════
  - failure guidance branches:
      unverifiable + not found       → "check inbound reply file path and rerun"
      unverifiable + not valid JSON  → "inspect inbound reply before verifying"
      unverifiable + bundle directory→ "confirm the pilot bundle path is correct"
      malformed                      → "ask the lab to resend a complete reply if fields are unreadable"
      run_mismatch                   → "confirm the lab returned the reply for the current run"
  - added verdict block to receipt verification mode (bottom of file):
      VERIFICATION PASSED / VERIFICATION FAILED block with Receipt path,
      Result, Next, and next-step instructions for reviewer shell

examples/pilot/README.md
  - added "## Verification Verdict Output" section with:
      PASSED example output
      FAILED example output
      failure guidance table

crates/service/tests/pilot_verify_operator_output_tests.rs
  - 20 new tests covering:
    - VERIFICATION PASSED / VERIFICATION FAILED wording
    - separator line presence
    - Result and Next field presence
    - inbound verdict Inbound/Bundle path fields
    - pass next action ("operator may export dispatch packet")
    - failure guidance: missing file, invalid JSON, missing bundle, malformed, run_mismatch
    - receipt verdict Receipt path field
    - receipt verdict fail guidance
    - README section coverage

commands run

cargo test --test pilot_verify_operator_output_tests

result

All 20 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_verify_operator_output_tests

commit message

pilot: harden verification operator verdict output
