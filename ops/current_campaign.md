campaign name

pilot: add artifact index summary for operator workflow

objective

Add a --artifact-index mode to run_pilot.sh so the operator can see the
complete pilot artifact layout in one deterministic, offline, plain-text
command. Includes current run context if receipt.json is present.
Shell/docs/test layer only — no protocol or service code changed.

files changed

examples/pilot/run_pilot.sh
  - added --artifact-index mode (before --walkthrough block)
  - if receipt.json present: resolves run_id, shows specific paths
  - if no receipt: shows generic patterns
  - sections printed:
      Current run (if known)
      Pilot bundle: receipt.json, export_packet.json
      Inbound replies: directory + current/pattern path
      Outbound packages: directory + current/pattern path
      Decision records: directory + ledger path
      Verification: verify.sh command
      Operator flow reminder: inspect → verify → export dispatch
  - no timestamps, no colors, no network calls, exits 0

examples/pilot/README.md
  - added "## Artifact Index" section with:
      --artifact-index command
      explanation that no commands executed, no files written
      full expected output example (with current run)

crates/service/tests/pilot_artifact_index_tests.rs
  - 23 new tests covering:
    - --artifact-index flag exists
    - exits 0
    - header "PostCAD — Pilot Artifact Index"
    - Pilot bundle section: receipt.json, export_packet.json
    - Inbound replies section + inbound/ directory
    - Outbound packages section + outbound/ directory
    - Decision records section + reports/ directory
    - Verification section + verify.sh command reference
    - Operator flow reminder: inspect/verify/export steps
    - no $(date) in block (determinism check)
    - README: section heading, command, expected output, no-files note

commands run

cargo test --test pilot_artifact_index_tests

result

All 23 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_artifact_index_tests

commit message

pilot: add artifact index summary for operator workflow
