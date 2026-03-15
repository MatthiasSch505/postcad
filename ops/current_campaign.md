campaign name

pilot: add demo surface for external viewers

objective

Add a --demo-surface mode to run_pilot.sh that gives any external viewer
(lab, engineer, investor) a compact single-command narrative of what
PostCAD is, the end-to-end flow, what the operator sees, why it matters,
and the commands to explore further. Shell/docs/test layer only.

files changed

examples/pilot/run_pilot.sh
  - added --demo-surface mode (before --system-overview block)
  - prints "POSTCAD PILOT DEMO" header
  - one-line PostCAD description
  - END-TO-END FLOW: 4 steps (generate, inspect, verify, export)
  - WHAT THE OPERATOR SEES: receipt.json, inbound lab reply,
    verification outcome, dispatch packet
  - WHY THIS MATTERS: deterministic routing, verifiable replies,
    audit-ready dispatch workflow
  - TRY IT: 4 exact commands (--system-overview, --quickstart,
    --run-summary, --help-surface)
  - no timestamps, no colors, exits 0

examples/pilot/README.md
  - added "## Demo Surface" section (before ## System Overview) with:
      --demo-surface command
      description as "fastest single-command introduction to the pilot"
      note: no commands executed, no files written

crates/service/tests/pilot_demo_surface_tests.rs
  - 27 new tests covering:
    - --demo-surface flag exists, exits 0
    - POSTCAD PILOT DEMO header, PostCAD description
    - END-TO-END FLOW: all 4 steps
    - WHAT THE OPERATOR SEES: all 4 artifacts
    - WHY THIS MATTERS: deterministic routing, verifiable replies,
      audit-ready dispatch
    - TRY IT: all 4 exact commands
    - no $(date) in block
    - README: section, command, single-command-intro description

commands run

cargo test --test pilot_demo_surface_tests

result

All 27 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_demo_surface_tests

commit message

pilot: add demo surface for external viewers
