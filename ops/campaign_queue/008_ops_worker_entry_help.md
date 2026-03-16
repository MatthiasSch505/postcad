campaign name

ops: add worker entry help script

files allowed to change

ops/README.md
ops/worker_entry_help.sh
crates/service/tests/worker_entry_help_surface_tests.rs

Claude prompt

Implement a new lane-1-safe ops utility script called `ops/worker_entry_help.sh`.

Goal:
Create a deterministic read-only helper that prints the exact manual commands an operator should use to enter worker A or worker B safely.

Constraints:

* Do not modify any kernel crates or protocol semantics.
* Stay strictly inside:

  * `ops/README.md`
  * `ops/worker_entry_help.sh`
  * `crates/service/tests/worker_entry_help_surface_tests.rs`
* No tmux management.
* No auto-start.
* No file writes.
* No network calls.
* Script must be deterministic and read-only.
* Use `set -euo pipefail`.

Required behavior:

1. Add new script `ops/worker_entry_help.sh`.
2. Script must refuse cleanly if not run inside the PostCAD repo.
3. Support optional `--base-dir <path>` with default `~/workers`.
4. Print sections:

   * `BASE DIR`
   * `WORKER A`
   * `WORKER B`
   * `SAFETY`
5. For each worker print exact manual `cd` and `claude` commands, whether or not the directory exists.
6. Add README documentation with example usage.
7. Add focused tests covering repo refusal, default and override base dir, deterministic command text, and README mention.

Definition of done:

* `ops/worker_entry_help.sh` is deterministic and read-only
* README mentions it
* tests pass

test command

cd ~/projects/postcad && cargo test -p service worker_entry_help_surface_tests -- --nocapture

commit message

ops: add worker entry help script
