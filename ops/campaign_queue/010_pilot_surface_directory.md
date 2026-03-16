campaign name

pilot: add deterministic surface directory

files allowed to change

examples/pilot/README.md
examples/pilot/run_pilot.sh
crates/service/tests/pilot_surface_directory_surface_tests.rs

Claude prompt

Implement a new lane-1-safe pilot observability surface called `--surface-directory` inside `examples/pilot/run_pilot.sh`.

Goal:
Create a deterministic directory of all current pilot shell surfaces so a new operator can see what exists at a glance.

Constraints:

* Do not modify any kernel crates or protocol semantics.
* Stay strictly inside:

  * `examples/pilot/README.md`
  * `examples/pilot/run_pilot.sh`
  * `crates/service/tests/pilot_surface_directory_surface_tests.rs`
* No network calls.
* No wall-clock timestamps.
* Output must be deterministic.
* No artifact dependency checks.
* Do not change existing command behavior.

Required behavior:

1. Add `--surface-directory` to CLI help/usage output.
2. Implement `--surface-directory` as a static read-only surface with sections:

   * `ENTRYPOINTS`
   * `INSPECTION`
   * `ARTIFACTS`
   * `GUIDES`
3. Under each section, list existing pilot shell commands only, one per line with a short purpose.
4. Include current surfaces where available, including command-map, artifact-index, status-board, pilot-demo, operator-inbox, timeline, next-step, and other existing inspection views.
5. Command must work without pilot artifacts present.
6. Add README documentation with a short example.
7. Add focused tests covering help text, required sections, expected command names, deterministic output, and README mention.

Definition of done:

* `--surface-directory` works as a deterministic static directory
* README mentions it
* tests pass

test command

cd ~/projects/postcad && cargo test -p service pilot_surface_directory_surface_tests -- --nocapture

commit message

pilot: add deterministic surface directory
