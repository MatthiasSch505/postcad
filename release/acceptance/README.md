# PostCAD Pilot Acceptance Bundle

This bundle provides a local pilot acceptance layer — a checklist and worksheet for determining whether a local PostCAD pilot run meets the current acceptance standard.

This is a local pilot acceptance layer, not a product certification layer.

---

## Who should use this

- An operator who has completed a local pilot run and wants to confirm it was successful
- A reviewer who was not present for the run and needs to evaluate the evidence
- Anyone auditing the local pilot artifacts against the stated scope

---

## Required inputs

Before using this bundle, the following must exist and be accessible:

| Input | Path | Purpose |
|---|---|---|
| Operator runbook | `release/README.md` | Confirms the pilot was run using the documented sequence |
| Evidence bundle | `release/evidence/current/` | Output artifacts from the pilot run |
| Evidence summary | `release/evidence/current/summary.txt` | Pass/fail confirmation from the evidence generator |
| Review packet | `release/review/` | System overview, operator flow, artifact guide, boundaries |
| Canonical fixtures | `examples/pilot/` | Frozen input fixtures used by the pilot flow |

---

## Recommended order of use

1. Read `release/review/SYSTEM_OVERVIEW.md` to understand what the pilot covers
2. Read `release/review/OPERATOR_FLOW.md` to understand how it was run
3. Inspect `release/evidence/current/summary.txt` for the top-level result
4. Work through `PILOT_ACCEPTANCE_CHECKLIST.md` item by item
5. Record findings in `REVIEW_WORKSHEET.md`
6. Optionally run `./release/acceptance/print_acceptance_summary.sh` for a structural pre-check

---

## What acceptance means here

A run is accepted if all checklist items pass. Acceptance means:

- The local pilot ran successfully using the documented operator sequence
- The evidence bundle is complete and structurally correct
- The key artifact values match expected deterministic outputs
- The review packet accurately describes the frozen scope

Acceptance does not mean:
- Production readiness
- Regulatory approval
- Coverage beyond the single canonical pilot case
- Any claim beyond the boundaries documented in `release/review/BOUNDARIES.md`
