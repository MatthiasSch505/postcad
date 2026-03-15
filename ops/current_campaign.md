campaign name

pilot: add receipt replay surface for protocol inspect

objective

Add a --receipt-replay mode to run_pilot.sh that prints a deterministic
plain-text explanation of the receipt replay concept for the current pilot run.
Shell/docs/test layer only.

files changed

examples/pilot/run_pilot.sh
  - added --receipt-replay mode (before --simulate-inbound block)
  - resolves run context from receipt.json (RR_RUN_ID, RR_RECEIPT_STATUS)
  - prints 5 sections:
      POSTCAD RECEIPT REPLAY header
      RUN CONTEXT             — Run ID, Receipt path (or "(not found)")
      WHAT THE RECEIPT COMMITS — candidate, deterministic outcome, receipt hash
      REPLAY IDEA             — routing commitment, replay verification concept
      HOW TO USE              — 4 exact pilot commands
      ENGINEER INTERPRETATION — deterministic / replayable / audit-ready
  - receipt present: "Current receipt detected for replay-oriented inspection."
  - receipt absent:  "Generate a pilot bundle first to create a receipt."
  - no timestamps, no colors, exits 0

examples/pilot/README.md
  - added "## Receipt Replay" section (before ## Default Inbound Path Resolution):
      --receipt-replay command
      description: shows receipt as replayable routing commitment
      note: no commands executed, no files written

crates/service/tests/pilot_receipt_replay_surface_tests.rs
  - 27 new tests covering:
    - --receipt-replay flag exists, exits 0
    - POSTCAD RECEIPT REPLAY header
    - RUN CONTEXT section, Run ID / Receipt labels, (not found) fallback
    - WHAT THE RECEIPT COMMITS: routing candidate, deterministic outcome, receipt hash
    - REPLAY IDEA section, routing commitment phrasing
    - HOW TO USE: section, run_pilot.sh, verify.sh, --run-summary, --trace-view
    - ENGINEER INTERPRETATION: deterministic, replayable, audit-ready
    - receipt-present / receipt-absent conditional messages
    - no $(date) in block
    - README: section, command

commands run

cargo test --test pilot_receipt_replay_surface_tests

result

All 27 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_receipt_replay_surface_tests

commit message

pilot: add receipt replay surface for protocol inspect
