campaign name

ops: add night mode smoke note

files allowed to change

ops/README_night_mode.md

Claude prompt

Create a tiny lane-1 validation artifact at `ops/README_night_mode.md`.

Goal:
Prove that the non-interactive campaign runner can execute a fresh bounded edit without permission prompts.

Constraints:

* Stay strictly inside:
  * `ops/README_night_mode.md`
* Do not modify any other files.
* No network calls.
* Keep content short and deterministic.

Required behavior:

1. Create `ops/README_night_mode.md`.
2. The file must contain exactly these sections in this order:
   * `# PostCAD Night Mode`
   * `## Purpose`
   * `## Allowed Scope`
   * `## Stop Conditions`
3. Under `## Purpose`, state that night mode is for unattended lane-1 campaigns only.
4. Under `## Allowed Scope`, list exactly these paths:
   * `examples/pilot/**`
   * `ops/**`
   * `docs/**`
   * `crates/service/tests/*surface_tests.rs`
5. Under `## Stop Conditions`, list exactly these bullets:
   * forbidden file access
   * ambiguous instruction
   * failing test command
   * missing required fixture
6. Keep wording concise and operator-facing.

Definition of done:

* `ops/README_night_mode.md` exists
* content matches the required sections and scope
* no other files changed

test command

cd ~/projects/postcad && test -f ops/README_night_mode.md && grep -F "# PostCAD Night Mode" ops/README_night_mode.md && grep -F "examples/pilot/**" ops/README_night_mode.md && grep -F "forbidden file access" ops/README_night_mode.md

commit message

ops: add night mode smoke note
