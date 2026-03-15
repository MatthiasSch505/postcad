campaign name

pilot: add sendable lab trial package export

objective

Create a deterministic sendable lab trial package export so the operator can generate
one clean directory for a real external lab trial in a single command. The package
contains everything the lab needs: manifest, operator instructions, lab instructions,
pre-filled reply template, and the routing receipt.

files changed

examples/pilot/run_pilot.sh
  - added --export-lab-trial-package mode
  - generates outbound/lab_trial_<run-id>/ with:
      manifest.txt (run_id, receipt_hash, file list)
      operator_instructions.txt (what to send, what to expect back, how to verify)
      lab_instructions.txt (which fields to fill, rejection rule, return filename)
      lab_reply_template.json (pre-filled receipt_hash, FILL_IN for lab_acknowledged_at/lab_id)
      receipt.json (copied from current run)
      export_packet.json (copied if present)
  - errors clearly if receipt.json not found

examples/pilot/.gitignore
  - added outbound/ (generated packages are runtime artifacts — do not commit)

examples/pilot/README.md
  - added "## Sendable Lab Trial Package" section with:
      generate command
      expected output
      package structure table
      zip and send steps
      verify step for returned reply

examples/pilot/testdata/expected_lab_trial_package_manifest_fields.txt
  - new fixture listing required manifest fields and file list

crates/service/tests/pilot_lab_trial_package_tests.rs
  - 31 new tests covering:
    - export flag exists
    - outbound/lab_trial_<run-id> directory naming
    - manifest.txt fields and file list
    - operator_instructions.txt wording and verify reference
    - lab_instructions.txt fill-in fields, rejection rule, return filename
    - lab_reply_template.json receipt_hash pre-filled, FILL_IN placeholders
    - receipt.json copied
    - stdout "Package written:" message
    - README section coverage

commands run

cargo test --all

result

All tests pass. 31 new tests in pilot_lab_trial_package_tests.rs.
No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --all

commit message

pilot: add sendable lab trial package export
