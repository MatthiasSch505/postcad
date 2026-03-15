campaign name

pilot: add operator walkthrough mode for pilot workflow

objective

Add a guided walkthrough mode to the pilot script so an external operator
can see the complete 4-step pilot workflow with one command. Documentation
and shell-layer usability only — no protocol or service code changed.

files changed

examples/pilot/run_pilot.sh
  - added --walkthrough mode (before default echo block)
  - prints POSTCAD PILOT WALKTHROUGH header with separator
  - Step 1: Generate pilot bundle — run_pilot.sh, creates receipt.json
  - Step 2: Inspect inbound lab reply — --inspect-inbound-reply command
  - Step 3: Verify inbound reply — verify.sh --inbound, mentions PASSED/FAILED
  - Step 4: Export dispatch packet — --export-lab-trial-package
  - exits 0 after printing; no commands executed

examples/pilot/README.md
  - added "## Pilot Walkthrough" section with:
      --walkthrough command
      full expected output (all 4 steps)
      clarification that no commands are executed, no files written

crates/service/tests/pilot_walkthrough_tests.rs
  - 22 new tests covering:
    - --walkthrough flag exists
    - exits 0 after printing
    - POSTCAD PILOT WALKTHROUGH header
    - Step 1 title, command, artifact
    - Step 2 title, --inspect-inbound-reply command
    - Step 3 title, verify.sh --inbound command, VERIFICATION PASSED/FAILED mention
    - Step 4 title, --export-lab-trial-package command
    - walkthrough block does not invoke cargo
    - README section: heading, command, all 4 steps, no-commands explanation

commands run

cargo test --test pilot_walkthrough_tests

result

All 22 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_walkthrough_tests

commit message

pilot: add operator walkthrough mode for pilot workflow
