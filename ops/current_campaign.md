campaign name

pilot: harden dispatch export operator verdict output

objective

Add a new --export-dispatch mode to run_pilot.sh with a hardened operator
verdict block, mirroring the verification verdict hardening pattern. The
operator gets a clear DISPATCH EXPORT READY / DISPATCH EXPORT FAILED summary
with actionable next-step guidance. Documentation and shell-layer only —
no protocol or service code changed.

files changed

examples/pilot/run_pilot.sh
  - added --export-dispatch mode (before --walkthrough block)
  - checks: receipt.json present, export_packet.json present and non-empty
  - resolves run_id, receipt_hash, dispatch_id from current artifacts
  - success verdict block:
      DISPATCH EXPORT READY
      ════════════════════════════════════════
      Run ID  : <run-id>
      File    : examples/pilot/export_packet.json
      Result  : dispatch packet exported
      Next    : send packet to manufacturer / lab contact
      ════════════════════════════════════════
  - failure verdict block:
      DISPATCH EXPORT FAILED with Result, Reason, Next guidance
  - failure branches:
      no_receipt          → "generate or load a current pilot run before exporting"
      no_dispatch_packet  → "verify the current route before exporting dispatch"
                            + "approve dispatch via reviewer shell first"
      default             → "confirm the pilot bundle and current artifacts are present"
  - exits 0 on ready, exits 1 on failure

examples/pilot/README.md
  - added "## Dispatch Export Outcomes" section with:
      --export-dispatch command
      DISPATCH EXPORT READY example block
      DISPATCH EXPORT FAILED example block
      failure guidance table (3 failure causes + next actions)
      note on reviewer shell for dispatch packet creation

crates/service/tests/pilot_dispatch_operator_output_tests.rs
  - 20 new tests covering:
    - --export-dispatch flag exists
    - DISPATCH EXPORT READY / DISPATCH EXPORT FAILED wording
    - separator, Result field, Next field
    - success: File field, Run ID field, next action guidance
    - failure: no receipt guidance, no dispatch packet guidance + reviewer mention
    - generic fallback guidance
    - exit 0 on success, exit 1 on failure
    - README section, command, example outputs, guidance table

commands run

cargo test --test pilot_dispatch_operator_output_tests

result

All 20 tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_dispatch_operator_output_tests

commit message

pilot: harden dispatch export operator verdict output
