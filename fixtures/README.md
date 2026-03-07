# Fixtures

Deterministic test inputs and expected outputs for the `postcad-cli route-case` command.

## Root fixtures

Used by the original golden tests in `crates/cli/tests/golden.rs`.

| File | Purpose |
|------|---------|
| `case.json` | Base dental case (DE jurisdiction, fixed case_id) |
| `candidates.json` | Single domestic candidate |
| `snapshot.json` | Eligible compliance snapshot |
| `snapshot_refusal.json` | Ineligible compliance snapshot (rejected attestation) |
| `expected_routed.json` | Golden output for the routed path |
| `expected_refused.json` | Golden output for the refused path |

## Scenario corpus (`scenarios/`)

Each scenario is a self-contained directory with `case.json`, `candidates.json`,
`snapshot.json`, and `expected.json`. All `case_id` values are fixed UUIDs so
outputs are fully deterministic across runs.

| Scenario | Outcome | What it exercises |
|----------|---------|-------------------|
| `routed_domestic_allowed` | routed | Domestic candidate passes compliance, `allow_domestic_only` policy routes it |
| `refused_no_eligible_candidates` | refused (`no_eligible_candidates`) | Empty snapshot list — compliance gate filters everything |
| `refused_compliance_failed` | refused (`no_eligible_candidates`) | Snapshot present but `is_eligible: false` — candidate filtered by compliance |
| `refused_invalid_snapshot` | error (`invalid_snapshot`) | Snapshot marked eligible with no evidence — validator rejects before routing |
| `routed_cross_border_allowed` | routed | Cross-border candidate, `allow_domestic_and_cross_border` policy routes it |

All scenarios run through the real `route_case_from_json` path in `postcad-cli`.
No routing logic is duplicated in the tests.
