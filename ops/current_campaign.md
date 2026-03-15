campaign name

pilot: add consolidated help surface for operators

objective

Add a --help-surface mode to run_pilot.sh that prints a consolidated
overview of all pilot operator modes, their purpose, when to use them,
and a recommended operator order. Best starting point for first-time
operators. Shell/docs/test layer only — no protocol or service code changed.

files changed

examples/pilot/run_pilot.sh
  - added --help-surface mode (before --quickstart block)
  - prints "PostCAD Pilot — Operator Mode Reference" header
  - documents 6 modes, each with Purpose and Use when fields:
      (default) — generate pilot bundle
      --inspect-inbound-reply — check reply fields before verification
      --export-dispatch — confirm dispatch packet ready
      --walkthrough — full 4-step guide
      --artifact-index — artifact map for current run
      --quickstart — minimum command sheet
  - "Recommended order" section: 4 steps including verify.sh reference
  - no timestamps, no colors, exits 0

examples/pilot/README.md
  - added "## Help Surface" section (before ## Quickstart) with:
      --help-surface command
      description as "best starting point for a first-time operator"
      note: no commands executed, no files written

crates/service/tests/pilot_help_surface_tests.rs
  - 21 new tests covering:
    - --help-surface flag exists, exits 0
    - header "PostCAD Pilot — Operator Mode Reference"
    - all 6 modes documented: default, --inspect-inbound-reply,
      --export-dispatch, --walkthrough, --artifact-index, --quickstart
    - Purpose and Use when fields present
    - Recommended order: all 4 steps + verify.sh reference
    - no $(date) in block
    - README: section heading, command, first-time operator description

commands run

cargo test --test pilot_help_surface_tests

result

All 21 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_help_surface_tests

commit message

pilot: add consolidated help surface for operators
