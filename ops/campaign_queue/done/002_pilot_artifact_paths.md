campaign name

pilot: add deterministic artifact paths surface

files allowed to change

examples/pilot/README.md
examples/pilot/run_pilot.sh
crates/service/tests/pilot_artifact_paths_surface_tests.rs

Claude prompt

Implement a new lane-1-safe pilot observability surface called `--artifact-paths` inside `examples/pilot/run_pilot.sh`.

Goal:
Create a deterministic path-focused surface that tells an operator where the latest pilot artifacts live and what each path represents.

Constraints:

* Do not modify any kernel crates or protocol semantics.
* Stay strictly inside:

  * `examples/pilot/README.md`
  * `examples/pilot/run_pilot.sh`
  * `crates/service/tests/pilot_artifact_paths_surface_tests.rs`
* No network calls.
* No wall-clock timestamps.
* Output must be deterministic from existing pilot artifact locations only.
* Do not change existing command behavior.

Required behavior:

1. Add `--artifact-paths` to CLI help/usage output.
2. Implement `--artifact-paths` as a read/inspect surface with sections:

   * `RUN`
   * `DIRECTORIES`
   * `FILES`
   * `NEXT`
3. Show root output directory and key file paths for the latest run.
4. If no latest run artifacts exist, fail cleanly with a deterministic error telling the operator to run the pilot first.
5. Keep it path-centric and minimal; do not duplicate full artifact-index purposes.
6. Add README documentation with a short example.
7. Add focused tests covering help text, success path, missing-artifacts failure, deterministic ordering, and README mention.

Definition of done:

* `--artifact-paths` works end-to-end from existing pilot outputs
* README mentions it
* tests pass

test command

cd ~/projects/postcad && cargo test -p service pilot_artifact_paths_surface_tests -- --nocapture

commit message

pilot: add deterministic artifact paths surface
