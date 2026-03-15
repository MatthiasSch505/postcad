campaign name

pilot: add business entrypoint surface for external viewers

objective

Add a --business-entrypoint mode to run_pilot.sh that prints a deterministic
plain-text external entrypoint for non-technical viewers (investors, operators,
commercial partners, lab owners). Shell/docs/test layer only.

files changed

examples/pilot/run_pilot.sh
  - added --business-entrypoint mode (before --engineer-entrypoint block)
  - resolves run context from receipt.json (BE_RUN_ID, fallback "not detected")
  - prints 6 sections:
      POSTCAD BUSINESS ENTRYPOINT header
      WHAT THIS PILOT DOES   — 4 plain-language bullets: routing, receipt, lab reply, dispatch
      WHY IT MATTERS         — fewer ambiguities, verifiable handoff, audit-ready, accountability
      WHAT TO LOOK AT FIRST  — 4 commands: --demo-surface, --system-overview, --run-summary, --help-surface
      WHAT EACH COMMAND SHOWS — one-line per command
      CURRENT RUN CONTEXT    — Run ID
      WHY THIS IS STRATEGIC  — workflow infrastructure, traceable handoff, trusted routing
  - no timestamps, no colors, exits 0

examples/pilot/README.md
  - added "## Business Entrypoint" section (before ## Engineer Entrypoint):
      --business-entrypoint command
      description: fastest intro for non-technical external viewers
      note: no commands executed, no files written

crates/service/tests/pilot_business_entrypoint_surface_tests.rs
  - 29 new tests covering:
    - --business-entrypoint flag exists, exits 0
    - POSTCAD BUSINESS ENTRYPOINT header
    - WHAT THIS PILOT DOES: section, all 4 bullets
    - WHY IT MATTERS: section, workflow ambiguities, verifiable handoff,
      audit-ready, operational accountability
    - WHAT TO LOOK AT FIRST: section, --demo-surface, --system-overview,
      --run-summary, --help-surface
    - WHAT EACH COMMAND SHOWS section
    - CURRENT RUN CONTEXT: section, BE_RUN_ID, "not detected" fallback
    - WHY THIS IS STRATEGIC: section, workflow infrastructure, traceable handoff,
      trusted routing
    - no $(date) in block
    - README: section, command

commands run

cargo test --test pilot_business_entrypoint_surface_tests

result

All 29 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_business_entrypoint_surface_tests

commit message

pilot: add business entrypoint surface for external viewers
