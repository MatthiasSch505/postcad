campaign name

ops: add frozen postcad constitution and campaign reference rule

files allowed to change

ops/POSTCAD_CONSTITUTION.md
ops/README.md
crates/service/tests/postcad_constitution_surface_tests.rs

Claude prompt

Create a new frozen governance artifact for the PostCAD project at `ops/POSTCAD_CONSTITUTION.md`.

Purpose:
Define the non-negotiable operating constitution for PostCAD so future campaigns can reference a single canonical source of truth for what may never drift.

Required sections in the constitution:
1. Mission
   - PostCAD is a deterministic post-CAD routing and audit infrastructure layer for regulated manufacturing.
2. Frozen kernel invariants
   - routing semantics are deterministic and rule-driven
   - refusal semantics must remain explicit
   - receipt/dispatch schema changes are controlled and not incidental
   - audit/integrity behavior must remain stable and reviewable
   - compliance and jurisdiction logic are constraints, not guesses
3. Liability boundary
   - PostCAD does not become manufacturer, clinician, or legal decision-maker
   - platform acts as routing/audit/compliance infrastructure only
4. Forbidden expansions
   - no chairside operational workflow in v1
   - no hidden AI heuristics in core routing
   - no silent schema drift
   - no convenience edits that cross into kernel truth
   - no ownership of clinical/manufacturing liability
5. Campaign lane model
   - core truth lane = kernel/protocol/schema/audit/refusal/routing/compliance semantics
   - surface lane = docs/helpers/operator UX/read-only observability/pilot presentation
   - core truth lane requires surgical scope and strict tests
   - surface lane may move faster but must not mutate core meaning
6. Campaign rule
   - every campaign must declare allowed files, forbidden layers, definition of done, and exact test command
7. Review doctrine
   - prefer refusal over cleverness
   - prefer boring determinism over speed if they conflict
   - treat ambiguous core edits as failures

Also update `ops/README.md` with a short section that tells operators and future campaigns to consult `ops/POSTCAD_CONSTITUTION.md` before creating or running campaigns.

Add a surface test file `crates/service/tests/postcad_constitution_surface_tests.rs` that verifies:
- `ops/POSTCAD_CONSTITUTION.md` exists
- it contains key anchor phrases:
  - `deterministic post-CAD routing and audit infrastructure layer`
  - `routing semantics are deterministic and rule-driven`
  - `refusal semantics must remain explicit`
  - `no hidden AI heuristics in core routing`
  - `every campaign must declare allowed files`
- `ops/README.md` references `POSTCAD_CONSTITUTION.md`

Constraints:
- Do not modify any kernel crates, protocol semantics, schemas, or routing logic.
- Stay strictly inside:
  - `ops/POSTCAD_CONSTITUTION.md`
  - `ops/README.md`
  - `crates/service/tests/postcad_constitution_surface_tests.rs`

test command

cd ~/projects/postcad && cargo test postcad_constitution_surface_tests -- --nocapture

commit message

ops: add frozen postcad constitution
