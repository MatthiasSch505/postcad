campaign name

pilot: add package self-check for real lab send readiness

objective

Add a deterministic lab trial package self-check so the operator can verify,
before sending, that the exported package is complete and ready for a real external
lab trial.

files changed

examples/pilot/run_pilot.sh
  - added --check-lab-trial-package mode
  - reads current run_id from receipt.json
  - locates outbound/lab_trial_<run-id>/
  - checks presence of all 8 required files:
      manifest.txt, operator_instructions.txt, lab_instructions.txt,
      lab_reply_template.json, email_to_lab.txt, short_message_to_lab.txt,
      operator_send_note.txt, receipt.json
  - prints per-file "present" / "missing" checklist
  - prints "package ready for external lab send" on pass (exit 0)
  - prints "package check failed" on fail (exit 1)
  - on fail: suggests --export-lab-trial-package to regenerate
  - on pass: suggests zip-and-send steps and references operator_send_note.txt

examples/pilot/README.md
  - added "## Package Self-Check" section with:
      3-step workflow: export → check → zip/send or regenerate
      expected ready output
      expected failed output

crates/service/tests/pilot_package_check_tests.rs
  - 26 new tests covering:
    - flag exists
    - error if receipt.json missing
    - package directory lookup uses run_id
    - error if package directory missing + suggests export command
    - all 8 required files are checked
    - "present" / "missing" per-file output wording
    - "package ready for external lab send" on pass
    - "package check failed" on fail
    - success guidance includes zip-and-send and operator_send_note.txt
    - README section coverage

commands run

cargo test --all

result

All tests pass. 26 new tests in pilot_package_check_tests.rs.
No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --all

commit message

pilot: add lab trial package self-check
