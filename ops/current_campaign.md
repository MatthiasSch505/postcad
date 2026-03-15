campaign name

pilot: add engineer entrypoint surface

objective

Add a --engineer-entrypoint mode to run_pilot.sh that prints a deterministic
technical entrypoint for engineers evaluating the protocol and workflow surfaces.
Shell/docs/test layer only.

files changed

examples/pilot/run_pilot.sh
  - added --engineer-entrypoint mode (before --protocol-chain block)
  - resolves run context from receipt.json (EE_RUN_ID, fallback "not detected")
  - prints 6 sections:
      POSTCAD ENGINEER ENTRYPOINT header
      WHAT TO LOOK AT FIRST   — 5 items: system overview, trace view, receipt replay,
                                 dispatch packet, protocol chain
      RECOMMENDED ORDER       — 5 exact commands in sequence
      WHAT EACH COMMAND SHOWS — one-line explanation per command
      CURRENT RUN CONTEXT     — Run ID
      WHY THIS IS TECHNICALLY INTERESTING — 4 bullets: deterministic routing artifacts,
        replayable receipt, verifiable workflow chain, audit-ready execution handoff
  - no timestamps, no colors, exits 0

examples/pilot/README.md
  - added "## Engineer Entrypoint" section (before ## Protocol Chain):
      --engineer-entrypoint command
      description: fastest technical introduction for engineers
      note: no commands executed, no files written

crates/service/tests/pilot_engineer_entrypoint_surface_tests.rs
  - 27 new tests covering:
    - --engineer-entrypoint flag exists, exits 0
    - POSTCAD ENGINEER ENTRYPOINT header
    - WHAT TO LOOK AT FIRST: section, all 5 items
    - RECOMMENDED ORDER: section, all 5 commands
    - WHAT EACH COMMAND SHOWS section
    - CURRENT RUN CONTEXT: section, EE_RUN_ID, "not detected" fallback
    - WHY THIS IS TECHNICALLY INTERESTING: section, all 4 bullets
    - no $(date) in block
    - README: section, command

commands run

cargo test --test pilot_engineer_entrypoint_tests

result

All 27 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_engineer_entrypoint_surface_tests

commit message

pilot: add engineer entrypoint surface
