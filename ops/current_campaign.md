campaign name

pilot: add quickstart command sheet for external operator

objective

Add a --quickstart mode to run_pilot.sh that prints the minimum command
sheet for the pilot workflow. Six steps, each with a label, exact command,
and one-line explanation. Deterministic, offline, plain text. Shell/docs/test
layer only — no protocol or service code changed.

files changed

examples/pilot/run_pilot.sh
  - added --quickstart mode (before --walkthrough block)
  - prints "PostCAD Pilot — Quickstart Command Sheet" header
  - six steps in order:
      1. Generate pilot bundle      — run_pilot.sh (plain)
      2. Inspect inbound lab reply  — --inspect-inbound-reply inbound/lab_reply_<run-id>.json
      3. Verify inbound reply       — verify.sh --inbound ... --bundle examples/pilot
      4. Export dispatch packet     — run_pilot.sh --export-dispatch
      5. Show artifact index        — run_pilot.sh --artifact-index
      6. Show walkthrough           — run_pilot.sh --walkthrough
  - each step: label + exact command + one-line explanation
  - no timestamps, no colors, exits 0

examples/pilot/README.md
  - added "## Quickstart" section (before ## Artifact Index) with:
      --quickstart command
      description as "fastest way for a new operator to understand the pilot commands"
      note: no commands executed, no files written

crates/service/tests/pilot_quickstart_tests.rs
  - 25 new tests covering:
    - --quickstart flag exists, exits 0
    - header "PostCAD Pilot — Quickstart Command Sheet"
    - each of 6 steps: label present, exact command present, explanation present
    - no $(date) in quickstart block
    - README: section heading, --quickstart command, purpose description

commands run

cargo test --test pilot_quickstart_tests

result

All 25 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_quickstart_tests

commit message

pilot: add quickstart command sheet for external operator
