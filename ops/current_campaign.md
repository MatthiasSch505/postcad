campaign name

pilot: add protocol chain surface for workflow artifacts

objective

Add a --protocol-chain mode to run_pilot.sh that prints a deterministic
plain-text explanation of the ordered chain of protocol artifacts across
the pilot workflow: receipt → inbound reply → verification → dispatch packet.
Shell/docs/test layer only.

files changed

examples/pilot/run_pilot.sh
  - added --protocol-chain mode (before --dispatch-packet block)
  - resolves run context from receipt.json (PC_RUN_ID)
  - detects current state of each stage from filesystem artifacts:
      PC_RECEIPT      → receipt.json
      PC_INBOUND      → inbound/lab_reply_*.json
      PC_VERIFICATION → reports/decision_*.txt
      PC_DISPATCH     → export_packet.json
  - prints 6 sections:
      POSTCAD PROTOCOL CHAIN header
      RUN CONTEXT      — Run ID
      CHAIN            — 4 stages with short description each
      CURRENT STATE    — detected / not yet observed per stage
      WHY THIS MATTERS — deterministic chain, verifiable transition, audit-ready path
      HOW TO USE       — --receipt-replay, --dispatch-packet, --trace-view, --run-summary
      ENGINEER INTERPRETATION — deterministic / chained / audit-ready
  - no timestamps, no colors, exits 0

examples/pilot/README.md
  - added "## Protocol Chain" section (before ## Dispatch Packet):
      --protocol-chain command
      description: shows ordered four-stage artifact chain
      note: no commands executed, no files written

crates/service/tests/pilot_protocol_chain_surface_tests.rs
  - 34 new tests covering:
    - --protocol-chain flag exists, exits 0
    - POSTCAD PROTOCOL CHAIN header
    - RUN CONTEXT section, Run ID label
    - CHAIN: section, all 4 stage labels
    - stage descriptions: routing commitment, source of truth, execution-side handoff
    - CURRENT STATE: section, detected/not yet observed labels
    - PC_RECEIPT, PC_INBOUND, PC_VERIFICATION, PC_DISPATCH variables
    - WHY THIS MATTERS: section, deterministic chain, verifiable transition, audit-ready path
    - HOW TO USE: section, --receipt-replay, --dispatch-packet, --trace-view, --run-summary
    - ENGINEER INTERPRETATION: section, chained
    - no $(date) in block
    - README: section, command

commands run

cargo test --test pilot_protocol_chain_surface_tests

result

All 34 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_protocol_chain_surface_tests

commit message

pilot: add protocol chain surface for workflow artifacts
