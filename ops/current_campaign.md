campaign name

pilot: add run fingerprint surface for protocol observability

objective

Add a --run-fingerprint mode to run_pilot.sh that computes and prints a
deterministic SHA-256 fingerprint derived from available protocol artifacts
(receipt, inbound reply, verification decision, dispatch packet).
Shell/docs/test layer only.

files changed

examples/pilot/run_pilot.sh
  - added --run-fingerprint mode (before --lab-entrypoint block)
  - resolves run context from receipt.json (RF_RUN_ID)
  - collects artifact files: RF_RECEIPT_FILE, RF_INBOUND_FILE,
    RF_VERIFICATION_FILE, RF_DISPATCH_FILE
  - computes RF_FINGERPRINT via python3 hashlib.sha256 over all present files
  - prints 5 sections:
      POSTCAD RUN FINGERPRINT header
      RUN CONTEXT        — Run ID, receipt path (or "not detected")
      FINGERPRINT COMPONENTS — included / not present per artifact
      FINGERPRINT        — "Run fingerprint : <hash>" or not-available fallback
      WHY THIS MATTERS   — stable identifier, derived from artifacts, logs/tracing/audits
      HOW TO USE         — --trace-view, --protocol-chain, --run-summary
  - no timestamps, no colors, exits 0

examples/pilot/README.md
  - added "## Run Fingerprint" section (before ## Lab Entrypoint):
      --run-fingerprint command
      description: deterministic SHA-256 identifier derived from protocol artifacts
      note: no commands executed, no files written

crates/service/tests/pilot_run_fingerprint_surface_tests.rs
  - 33 new tests covering:
    - --run-fingerprint flag exists, exits 0
    - POSTCAD RUN FINGERPRINT header
    - RUN CONTEXT: section, RF_RUN_ID, receipt path, "not detected" fallback
    - FINGERPRINT COMPONENTS: section, all 4 artifacts, included/not present labels
    - FINGERPRINT section: echo "FINGERPRINT" heading, RF_FINGERPRINT variable,
      "Run fingerprint :" label, sha256 usage, per-file variables
    - not-available fallback when no receipt
    - WHY THIS MATTERS: section, stable identifier, derived from artifacts,
      logs/tracing/audits
    - HOW TO USE: section, --trace-view, --protocol-chain, --run-summary
    - no $(date) in block
    - README: section, command
  - fixed fingerprint section heading test: uses echo "FINGERPRINT" pattern
    instead of raw newline match

commands run

cargo test --test pilot_run_fingerprint_surface_tests

result

All 33 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_run_fingerprint_surface_tests

commit message

pilot: add run fingerprint surface for protocol observability
