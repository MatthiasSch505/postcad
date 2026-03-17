campaign name

ops: add focused night log tail command

files allowed to change

ops/README_night_mode.md
ops/night_tail.sh
crates/service/tests/night_tail_surface_tests.rs

Claude prompt

Implement a new lane-1-safe ops utility script called `ops/night_tail.sh`.

Goal:
Create a deterministic read-only helper for night-mode monitoring that tails `ops/queue_status.log` with an optional bounded line count.

Constraints:
- Do not modify any kernel crates or protocol semantics.
- Stay strictly inside:
  - `ops/README_night_mode.md`
  - `ops/night_tail.sh`
  - `crates/service/tests/night_tail_surface_tests.rs`
- No tmux management.
- No auto-start.
- No file writes.
- No network calls.
- Script must be deterministic and read-only.
- Use `set -euo pipefail`.

Required behavior:
1. Add new script `ops/night_tail.sh`.
2. Script must refuse cleanly if not run inside the PostCAD repo.
3. Default behavior: print the tail of `ops/queue_status.log`.
4. Support `--lines N` to override the default number of lines.
5. If the log file is missing, print a clear deterministic message and exit non-zero.
6. Document the command in `ops/README_night_mode.md`.
7. Add focused tests covering:
   - repo refusal
   - default behavior
   - `--lines N`
   - missing log handling
   - README mention

Definition of done:
- `ops/night_tail.sh` exists
- README_night_mode mentions it
- tests pass

test command

cd ~/projects/postcad && cargo test -p service night_tail_surface_tests -- --nocapture

commit message

ops: add focused night log tail command
