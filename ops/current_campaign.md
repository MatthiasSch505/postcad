campaign name

pilot: add command safety guardrails for operator workflow

objective

Harden run_pilot.sh with structured guardrail blocks for three failure
paths: missing file argument for --inspect-inbound-reply, missing run
context for --export-dispatch, and unknown flags. All guardrail messages
are deterministic, plain text, non-interactive, exit non-zero. All existing
tests continue to pass.

files changed

examples/pilot/run_pilot.sh
  - --inspect-inbound-reply missing arg: replaced minimal error with
    INSPECT INBOUND REPLY — USAGE block (command form + example), then
    existing error line preserved (keeps old tests passing)
  - --export-dispatch no_receipt branch: added DISPATCH EXPORT — PRECONDITION
    NOT MET block with "A valid pilot run was not detected." and 3-step
    recommended steps after existing Reason/Next lines
  - added unknown argument handler before default block:
      if $1 non-empty → prints UNKNOWN COMMAND + --help-surface pointer + exit 1

examples/pilot/README.md
  - added "## Command Guardrails" section (before ## Run Summary) with:
      table of 3 guardrail situations and outputs
      note: exits non-zero so scripts can detect failure

crates/service/tests/pilot_command_guardrails_tests.rs
  - 18 new tests covering:
    - inspect usage header, command form, example, exit 1
    - dispatch precondition header, "A valid pilot run was not detected.",
      Recommended steps, steps 1/2/3
    - unknown command header, --help-surface pointer, exit 1
    - guardrail messages contain no ANSI color codes
    - README: section, inspect usage mention, dispatch precondition mention,
      unknown command mention

commands run

cargo test --test pilot_command_guardrails_tests
cargo test --all (all pass)

result

18 new tests pass. All existing tests pass. No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --test pilot_command_guardrails_tests

commit message

pilot: add command safety guardrails for operator workflow
