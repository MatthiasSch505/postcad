campaign name

pilot: add dispatch packet surface for protocol inspect

objective

Add a --dispatch-packet mode to run_pilot.sh that prints a deterministic
plain-text explanation of the dispatch packet as the execution-side protocol
artifact of the pilot workflow. Shell/docs/test layer only.

files changed

examples/pilot/run_pilot.sh
  - added --dispatch-packet mode (before --receipt-replay block)
  - resolves run context from receipt.json (DP_RUN_ID)
  - detects dispatch artifact: export_packet.json → path, else "not yet generated"
  - prints 5 sections:
      POSTCAD DISPATCH PACKET header
      RUN CONTEXT             — Run ID
      DISPATCH ARTIFACT       — execution-side handoff, verified state, artifact path
      WHY IT MATTERS          — verified workflow state, audit-ready, execution checkpoint
      HOW TO USE              — --export-dispatch, --run-summary, --trace-view, --artifact-index
      ENGINEER INTERPRETATION — deterministic / exportable / audit-ready
  - dispatch absent: "Export dispatch packet after verification to generate the artifact."
  - no timestamps, no colors, exits 0

examples/pilot/README.md
  - added "## Dispatch Packet" section (before ## Receipt Replay):
      --dispatch-packet command
      description: shows dispatch packet as execution-side artifact
      note: no commands executed, no files written

crates/service/tests/pilot_dispatch_packet_surface_tests.rs
  - 27 new tests covering:
    - --dispatch-packet flag exists, exits 0
    - POSTCAD DISPATCH PACKET header
    - RUN CONTEXT section, Run ID label
    - DISPATCH ARTIFACT: execution-side handoff, verified workflow state,
      export_packet.json path, "not yet generated" fallback
    - WHY IT MATTERS: verified workflow state, audit-ready, execution checkpoint
    - HOW TO USE: section, --export-dispatch, --run-summary, --trace-view, --artifact-index
    - ENGINEER INTERPRETATION: deterministic, exportable, audit-ready
    - absent-artifact export guidance message
    - no $(date) in block
    - README: section, command
  - fixed char-boundary issue in ENGINEER INTERPRETATION tests:
    uses two-stage find() instead of fixed byte offsets

commands run

cargo test --test pilot_dispatch_packet_surface_tests

result

All 27 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_dispatch_packet_surface_tests

commit message

pilot: add dispatch packet surface for protocol inspect
