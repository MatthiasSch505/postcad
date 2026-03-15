campaign name

pilot: add system overview surface for external operators

objective

Add a --system-overview mode to run_pilot.sh that explains what the
PostCAD pilot system is and does — deterministic, plain text, offline.
Five sections: description, CORE IDEA, PILOT WORKFLOW, KEY ARTIFACTS,
OPERATOR TOOLS, PROPERTIES. Shell/docs/test layer only.

files changed

examples/pilot/run_pilot.sh
  - added --system-overview mode (before --help-surface block)
  - prints "POSTCAD PILOT SYSTEM OVERVIEW" header
  - CORE IDEA: case → routing decision → receipt → lab reply → dispatch packet
  - PILOT WORKFLOW: 4 steps (generate, inspect, verify, export)
  - KEY ARTIFACTS: receipt.json, inbound lab reply, verification result,
    dispatch packet (export_packet.json)
  - OPERATOR TOOLS: --quickstart, --walkthrough, --artifact-index, --help-surface
  - PROPERTIES: deterministic routing, verifiable replies, audit-ready dispatch
  - no timestamps, no colors, exits 0

examples/pilot/README.md
  - added "## System Overview" section (before ## Help Surface) with:
      --system-overview command
      description of what it prints
      note: no commands executed, no files written

crates/service/tests/pilot_system_overview_tests.rs
  - 29 new tests covering:
    - --system-overview flag exists, exits 0
    - POSTCAD PILOT SYSTEM OVERVIEW header
    - routing layer description
    - CORE IDEA section: case, receipt, dispatch packet
    - PILOT WORKFLOW section: 4 steps
    - KEY ARTIFACTS: receipt.json, export_packet.json
    - OPERATOR TOOLS: all 4 helper modes
    - PROPERTIES: deterministic routing, verifiable replies, audit-ready dispatch
    - no $(date) in block
    - README: section heading, --system-overview command

commands run

cargo test --test pilot_system_overview_tests

result

All 29 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_system_overview_tests

commit message

pilot: add system overview surface for external operators
