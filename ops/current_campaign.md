campaign name

pilot: add run summary surface for pilot workflow

objective

Add a --run-summary mode to run_pilot.sh that prints a compact, deterministic
summary of the current pilot run — run context, artifact status, suggested
next operator action, and command hints. Based only on existing filesystem
artifacts; no invented state. Shell/docs/test layer only.

files changed

examples/pilot/run_pilot.sh
  - added --run-summary mode (before --system-overview block)
  - RUN CONTEXT: resolves run_id from receipt.json, falls back to "not detected"
  - ARTIFACT STATUS: checks presence of receipt.json, inbound lab reply
    (inbound/lab_reply_*.json), verification result (reports/decision_*.txt),
    dispatch packet (export_packet.json); prints present/missing/not yet generated
  - NEXT OPERATOR ACTION: branches based on artifact presence:
      no receipt         → generate pilot bundle
      no inbound reply   → inspect inbound lab reply
      no decision record → verify inbound reply
      otherwise          → export dispatch packet
  - OPERATOR COMMAND HINTS: --quickstart, --artifact-index, --walkthrough
  - no timestamps, no colors, exits 0

examples/pilot/README.md
  - added "## Run Summary" section (before ## System Overview) with:
      --run-summary command
      description: shows current run state and recommended next step
      note: no commands executed, no files written

crates/service/tests/pilot_run_summary_tests.rs
  - 26 new tests covering:
    - --run-summary flag exists, exits 0
    - header "PostCAD — Pilot Run Summary"
    - RUN CONTEXT section: Run ID field, "not detected" fallback
    - ARTIFACT STATUS section: receipt.json, inbound lab reply,
      verification result, dispatch packet; present/missing/not yet generated labels
    - NEXT OPERATOR ACTION: all 4 suggestion branches
    - OPERATOR COMMAND HINTS: --quickstart, --artifact-index, --walkthrough
    - no $(date) in block
    - README: section heading, --run-summary command

commands run

cargo test --test pilot_run_summary_tests

result

All 26 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_run_summary_tests

commit message

pilot: add run summary surface for pilot workflow
