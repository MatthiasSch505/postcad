campaign name

pilot: add deterministic flow explainer surface

files allowed to change

examples/pilot/README.md
examples/pilot/run_pilot.sh
crates/service/tests/pilot_flow_explainer_surface_tests.rs

Claude prompt

Implement a new lane-1-safe pilot observability surface called `--flow-explainer` inside `examples/pilot/run_pilot.sh`.

Goal:
Create a static deterministic explanation surface that describes the PostCAD pilot flow in plain operator language.

Constraints:

* Do not modify any kernel crates or protocol semantics.
* Stay strictly inside:

  * `examples/pilot/README.md`
  * `examples/pilot/run_pilot.sh`
  * `crates/service/tests/pilot_flow_explainer_surface_tests.rs`
* No network calls.
* No wall-clock timestamps.
* Output must be deterministic.
* No artifact dependency checks.
* Do not change existing command behavior.

Required behavior:

1. Add `--flow-explainer` to CLI help/usage output.
2. Implement `--flow-explainer` as a static read-only surface with sections:

   * `INPUT`
   * `CHECKS`
   * `DECISION`
   * `OUTPUTS`
   * `WHY THIS MATTERS`
3. Keep wording operational and boring, not marketing-heavy.
4. Command must work without pilot artifacts present.
5. Add README documentation with a short example.
6. Add focused tests covering help text, all required sections, static behavior, deterministic wording anchors, and README mention.

Definition of done:

* `--flow-explainer` works as a deterministic static explanation surface
* README mentions it
* tests pass

test command

cd ~/projects/postcad && cargo test -p service pilot_flow_explainer_surface_tests -- --nocapture

commit message

pilot: add deterministic flow explainer surface
