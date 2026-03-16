campaign name

pilot: add deterministic command map surface

objective

Add --command-map to run_pilot.sh — a static, deterministic navigation
surface that lists all pilot inspection commands with PURPOSE, FLOW,
COMMANDS, and START HERE sections. Requires no artifacts. Shell/docs/test only.

files changed

examples/pilot/run_pilot.sh
  - added --command-map block (after --pilot-demo, before --lab-entrypoint):
      POSTCAD PILOT COMMAND MAP header
      PURPOSE section: PostCAD routes cases deterministically, lists inspection surfaces
      FLOW section: 5 ordered stages (case intake, compliance, routing, dispatch, audit)
      COMMANDS section: 12 existing commands with one-line purpose each:
        --protocol-chain, --engineer-entrypoint, --business-entrypoint,
        --lab-entrypoint, --audit-receipt-view, --run-summary, --next-step,
        --operator-inbox, --timeline, --pilot-demo, --artifact-index,
        --trace-view, --run-fingerprint
      START HERE section: 3 recommendation paths
        (First-time demo, Operator review, Artifact review)
      no filesystem checks, no receipt.json required, no timestamps, exits 0
  - added --command-map entry to --help-surface mode listing

examples/pilot/README.md
  - added "## Command Map" section (after ## Pilot Demo, before ## Run Fingerprint):
      --command-map command
      four section descriptions
      note: does not require artifacts, read-only, no files written

crates/service/tests/pilot_command_map_surface_tests.rs (new)
  - 40 tests using command_map_block() helper for char-safe block extraction
  - covers:
    - flag in script, appears in help surface, exits 0
    - POSTCAD PILOT COMMAND MAP header
    - PURPOSE: section, mentions PostCAD, deterministic, routing
    - FLOW: section, all 5 stages (case intake, compliance, routing, dispatch, audit)
    - COMMANDS: section, all 12 listed commands
    - START HERE: section, first-time demo, operator review, artifact review paths
    - stable ordering: PURPOSE before FLOW before COMMANDS before START HERE
    - static: no receipt.json check, no filesystem checks
    - determinism: no $(date, no file writes
    - README: section, command, no-artifacts-required note

commands run

cargo test --test pilot_command_map_surface_tests -- --nocapture

result

All 40 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test -p service pilot_command_map_surface_tests -- --nocapture

commit message

pilot: add deterministic command map surface
