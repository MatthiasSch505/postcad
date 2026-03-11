# PostCAD Local Pilot — Release Notes

**Pilot label:** `pilot-local-v1`
**Scope:** local pilot package only — no production deployment, no cloud infrastructure

---

## What this pilot release includes

A complete local pilot package for the PostCAD routing and verification system. All surfaces listed below are committed to the `main` branch and present in the `release/` tree.

### Operator surfaces (executable)

- `release/start_pilot.sh` — build and start the HTTP service
- `release/reset_pilot_data.sh` — remove runtime data
- `release/smoke_test.sh` — 7-step deterministic smoke test
- `release/generate_evidence_bundle.sh` — capture pilot run artifacts
- `demo/run_demo.sh` — self-contained 8-step demo

### Review and inspection surfaces (read-only)

- `release/review/` — system overview, operator flow, artifact guide, boundaries (5 docs)
- `release/walkthrough/` — 9-step pilot walkthrough narrative and orientation script
- `release/evidence/README.md` — evidence bundle structure guide
- `release/selfcheck/` — structural self-check scope and script
- `release/freeze/` — pilot surface inventory and frozen boundaries

### Acceptance and handoff surfaces

- `release/acceptance/` — 33-item checklist, review worksheet, structural pre-check script
- `release/handoff/` — first-hour guide, known-good state, handoff checklist, index script

### Navigation and summary surfaces

- `release/INDEX.md` — top-level reference table for the entire release/ tree
- `release/FREEZE_MANIFEST.md` — classified surface listing with frozen values
- `release/review-trace/` — 8-step ordered review path with 9 stop conditions
- `release/readiness/` — readiness snapshot, out-of-scope statement
- `release/version/` — this release notes file, version marker, version helper

### Canonical fixtures (frozen, not under release/)

- `examples/pilot/case.json`
- `examples/pilot/registry_snapshot.json`
- `examples/pilot/config.json`

---

## Frozen boundaries reaffirmed

| Item | Frozen value |
|---|---|
| Protocol version | `postcad-v1` |
| Routing kernel version | `postcad-routing-v1` |
| Receipt schema version | `1` |
| Deterministic receipt hash | `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb` |
| Canonical pilot case ID | `f1000001-0000-0000-0000-000000000001` |
| Expected verification result | `VERIFIED` |

No protocol, routing, schema, kernel, or hashing changes are included in this pilot release. See `release/freeze/FROZEN_BOUNDARIES.md` for the complete frozen scope.

---

## What is out of scope

- No regulatory or certification claims
- No production deployment
- No hosted or remote service operation
- No coverage beyond the single canonical pilot case
- No automated acceptance engine
- No protocol redesign

See `release/readiness/OUT_OF_SCOPE.md` for the complete exclusion list.

---

## How to verify the checked-out commit

```bash
# Confirm branch and commit
git branch --show-current    # expected: main
git log --oneline -1         # review HEAD

# Confirm tests pass
cargo test --workspace

# Confirm package structure
./release/selfcheck/run_release_selfcheck.sh

# Confirm evidence (requires service running)
./release/start_pilot.sh                  # Terminal A
./release/reset_pilot_data.sh             # Terminal B
./release/generate_evidence_bundle.sh     # Terminal B
cat release/evidence/current/summary.txt  # expected: All 7 steps passed.
```

See `release/version/PILOT_VERSION.md` for the optional git tag command to anchor this state.
