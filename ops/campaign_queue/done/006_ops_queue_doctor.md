campaign name

ops: add queue doctor diagnostic script

files allowed to change

ops/README.md
ops/queue_doctor.sh
crates/service/tests/queue_doctor_surface_tests.rs

Claude prompt

Implement a new lane-1-safe ops utility script called `ops/queue_doctor.sh`.

Goal:
Create a deterministic read-only diagnostic script that helps an operator understand why the campaign queue is idle, blocked, or ready.

Constraints:

* Do not modify any kernel crates or protocol semantics.
* Stay strictly inside:

  * `ops/README.md`
  * `ops/queue_doctor.sh`
  * `crates/service/tests/queue_doctor_surface_tests.rs`
* No tmux management.
* No auto-start.
* No file writes.
* No network calls.
* Script must be deterministic and read-only.
* Use `set -euo pipefail`.

Required behavior:

1. Add new script `ops/queue_doctor.sh`.
2. Script must refuse cleanly if not run inside the PostCAD repo.
3. Script must print sections:

   * `REPO`
   * `QUEUE`
   * `LAST RESULT`
   * `STATUS LOG`
   * `NEXT`
4. Report whether pending campaign files exist, whether `ops/last_result.md` exists, whether `ops/queue_status.log` exists, and short next-step commands.
5. If files are missing, report clearly and continue rather than crashing.
6. Add README documentation with example usage.
7. Add focused tests covering repo refusal, success path, missing files, deterministic output, and README mention.

Definition of done:

* `ops/queue_doctor.sh` is deterministic and read-only
* README mentions it
* tests pass

test command

cd ~/projects/postcad && cargo test -p service queue_doctor_surface_tests -- --nocapture

commit message

ops: add queue doctor diagnostic script
