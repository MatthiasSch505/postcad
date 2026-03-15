campaign name

pilot: add manual lab reply template for external trials

objective

Make the external handoff pack usable by a real human lab without relying on the
simulator. Add a deterministic manual lab reply template to the handoff pack, add
a --prepare-manual-reply command to materialize it into the inbound directory, and
validate that the existing verify flow accepts a filled reply and rejects malformed
or mismatched ones.

files changed

examples/pilot/lab_simulator.sh
  - added lab_reply_template.json to handoff pack (pre-filled with run identifiers,
    FILL_IN placeholders for lab_acknowledged_at and lab_id)
  - added lab_reply_template.json to ARTIFACT_LIST and manifest
  - updated operator_instructions.txt to mention the template

examples/pilot/run_pilot.sh
  - added --prepare-manual-reply mode
  - reads current run from receipt.json, locates handoff/<run-id>/lab_reply_template.json
  - copies template to inbound/lab_reply_<run-id>.json
  - prints: which fields lab must fill, which fields must not change, verify command

examples/pilot/README.md
  - added "## Real Manual External Trial" section
  - step-by-step: route → handoff pack → send → lab fills template → place in inbound
    → verify → decision record

examples/pilot/testdata/expected_handoff_manifest_fields.txt
  - added lab_reply_template.json entry

examples/pilot/testdata/expected_manual_reply_template_fields.txt
  - new fixture listing required template fields and FILL_IN marker

crates/service/tests/pilot_manual_reply_tests.rs
  - 30 new tests covering:
    - handoff pack writes lab_reply_template.json
    - template has receipt_hash pre-filled, FILL_IN placeholders for lab fields
    - run_pilot.sh --prepare-manual-reply mode exists and handles error cases
    - verify.sh accepts matching reply, rejects mismatched/malformed
    - README documents the real manual external trial section

commands run

cargo test --all

result

All tests pass. 30 new tests in pilot_manual_reply_tests.rs.
No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --all

commit message

pilot: add manual lab reply template for external trials
