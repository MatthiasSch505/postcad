campaign name

pilot: add deterministic review checklist surface

files allowed to change

examples/pilot/README.md
examples/pilot/run_pilot.sh
crates/service/tests/pilot_review_checklist_surface_tests.rs

Claude prompt

Implement a new lane-1-safe pilot observability surface called `--review-checklist` inside `examples/pilot/run_pilot.sh`.

Goal:
Create a deterministic operator checklist for reviewing a completed pilot run using existing observability surfaces.

Constraints:

* Do not modify any kernel crates or protocol semantics.
* Stay strictly inside:

  * `examples/pilot/README.md`
  * `examples/pilot/run_pilot.sh`
  * `crates/service/tests/pilot_review_checklist_surface_tests.rs`
* No network calls.
* No wall-clock timestamps.
* Output must be deterministic.
* This surface may be static and must not require artifacts.
* Do not change existing command behavior.

Required behavior:

1. Add `--review-checklist` to CLI help/usage output.
2. Implement `--review-checklist` as a static read-only surface with sections:

   * `BEFORE YOU START`
   * `REVIEW STEPS`
   * `EXPECTED OUTPUTS`
   * `STOP CONDITIONS`
3. Reference existing commands only.
4. Keep it practical and operator-safe.
5. Add README documentation with a short example.
6. Add focused tests covering help text, required sections, expected command references, deterministic output, and README mention.

Definition of done:

* `--review-checklist` works as a deterministic static review guide
* README mentions it
* tests pass

test command

cd ~/projects/postcad && cargo test -p service pilot_review_checklist_surface_tests -- --nocapture

commit message

pilot: add deterministic review checklist surface
