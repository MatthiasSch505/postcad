campaign name

ops: add monitor help script

files allowed to change

ops/README.md
ops/monitor_help.sh
crates/service/tests/monitor_help_surface_tests.rs

Claude prompt

Implement a new lane-1-safe ops utility script called `ops/monitor_help.sh`.

Goal:
Create a deterministic read-only helper that prints the standard monitoring commands for queue, workers, and recent results.

Constraints:

* Do not modify any kernel crates or protocol semantics.
* Stay strictly inside:

  * `ops/README.md`
  * `ops/monitor_help.sh`
  * `crates/service/tests/monitor_help_surface_tests.rs`
* No tmux management.
* No auto-start.
* No file writes.
* No network calls.
* Script must be deterministic and read-only.
* Use `set -euo pipefail`.

Required behavior:

1. Add new script `ops/monitor_help.sh`.
2. Script must refuse cleanly if not run inside the PostCAD repo.
3. Print sections:

   * `QUICK CHECK`
   * `QUEUE`
   * `WORKERS`
   * `RESULTS`
   * `STOP`
4. Each section should print exact shell commands only, short and copy-pasteable.
5. Add README documentation with example usage.
6. Add focused tests covering repo refusal, required sections, deterministic command anchors, and README mention.

Definition of done:

* `ops/monitor_help.sh` is deterministic and read-only
* README mentions it
* tests pass

test command

cd ~/projects/postcad && cargo test -p service monitor_help_surface_tests -- --nocapture

commit message

ops: add monitor help script
