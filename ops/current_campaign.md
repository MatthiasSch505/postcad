campaign name

pilot: add first-contact lab message kit

objective

Reduce first-contact friction with real external labs by adding a deterministic
message kit to the sendable lab trial package. The operator no longer needs to
improvise any wording when sending the package to a real lab.

files changed

examples/pilot/run_pilot.sh
  - extended --export-lab-trial-package to write three message kit files:
      email_to_lab.txt: boring first-contact email draft with run id, what to fill,
                        what to return, no-integration statement, return filename
      short_message_to_lab.txt: one-paragraph WhatsApp/Signal/LinkedIn message
                                with run id and reply template reference
      operator_send_note.txt: 6-step operator checklist — zip → send → wait →
                              place in inbound → verify → inspect decision record
  - updated MANIFEST_FILES to include all three new files

examples/pilot/README.md
  - added "## First-Contact Send Flow" section with:
      message kit file table
      how to use email or short message
      operator checklist display

examples/pilot/testdata/expected_lab_trial_package_manifest_fields.txt
  - added email_to_lab.txt, short_message_to_lab.txt, operator_send_note.txt

crates/service/tests/pilot_message_kit_tests.rs
  - 28 new tests covering:
    - manifest fixture lists all three message kit files
    - export command writes all three files
    - email_to_lab.txt: run id, external trial wording, fill-in fields,
      no-integration statement, return filename
    - short_message_to_lab.txt: run id, reply template reference, no-integration
    - operator_send_note.txt: checklist header, run id, all 6 steps
    - README section coverage

commands run

cargo test --all

result

All tests pass. 28 new tests in pilot_message_kit_tests.rs.
No protocol/core code changed.

test command

cd ~/projects/postcad && cargo test --all

commit message

pilot: add first-contact lab message kit
