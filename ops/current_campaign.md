campaign name

reviewer run identity + artifact lineage clarity

goal

Strengthen operator confidence by making the reviewer shell explicitly communicate run identity and artifact lineage.

The operator must always be able to tell:

- which route run is currently active
- which artifacts belong to the active run
- when artifacts belong to a previous run

This prevents silent workflow confusion during pilot operation.

This campaign modifies only reviewer surface logic and does not touch protocol semantics.

frozen invariants

routing kernel semantics
refusal semantics
receipt schema
dispatch schema
OpenAPI surface
deterministic audit rules
liability boundaries

allowed files

crates/service/src/reviewer.rs
crates/service/tests/reviewer_workflow_smoke_test.rs

forbidden files

all routing kernel code
all refusal logic
all receipt schema code
all dispatch schema code
OpenAPI definitions
audit or replay logic
Cargo.toml
ops/

tasks

1. Introduce current run identity block

Add a compact reviewer UI section titled:

current run

This section must display a clear run identity summary derived from existing client state.

Show:

- route status
- receipt status
- verification status
- dispatch export status

All signals must represent only the currently active run.

2. Artifact lineage indicators

Each artifact panel must gain a small lineage indicator:

- current run
- previous run
- not generated

This must be derived from existing UI state signals.

Do not introduce new persistence.

Purpose:

Operator must instantly see whether a displayed artifact corresponds to the active route.

3. Reroute invalidation signals

When a new route is generated after artifacts existed for a prior route:

The reviewer shell must clearly indicate:

artifact belongs to previous run

Affected artifact panels:

- verification result
- dispatch export

These indicators must appear visually subtle but explicit.

4. Add run reset behavior

When a new route replaces an existing route:

UI must automatically downgrade artifact lineage status for:

- verification
- dispatch export

so the operator cannot mistake them as belonging to the new run.

This must rely purely on current client state logic.

5. Improve operator clarity messaging

Add short operator guidance messages depending on run state.

Examples:

verification belongs to previous run
run verification again for current route

dispatch export belongs to previous run
export dispatch packet again for current route

These messages should appear only when lineage mismatch occurs.

6. Align with existing surfaces

Ensure compatibility with:

- workflow timeline strip
- dispatch readiness panel
- empty state hardening
- next-action rail

The new lineage indicators must reinforce those components.

7. Smoke tests

Extend reviewer workflow smoke tests to validate:

- initial state shows no run artifacts
- route created shows current run state
- verification shows verification belonging to current run
- dispatch export shows export belonging to current run
- reroute causes verification/export to be marked previous run
- artifact lineage messaging appears

Implement the entire campaign in one pass.
Do not stop after the first subtask.
Run tests before returning.

completion criteria

Reviewer UI clearly communicates run identity and artifact lineage.

Operator cannot confuse artifacts from previous runs with artifacts of the active run.

No protocol, schema, routing, refusal, dispatch, OpenAPI, or audit behavior changes.

Reviewer smoke tests validate run lineage behavior.

All tests pass.

test command

cargo test -p postcad-service reviewer

commit message

reviewer: add run identity and artifact lineage indicators
