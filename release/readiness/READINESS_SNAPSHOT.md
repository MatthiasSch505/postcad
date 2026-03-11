# PostCAD Pilot Readiness Snapshot

Single-page summary of the current local pilot package state. This is a description only — not a certification claim.

---

## Release surfaces present

The following surfaces are committed and present in the repository:

| Surface | Path | Type |
|---|---|---|
| Operator runbook | `release/README.md` | read-only |
| Release index | `release/INDEX.md` | read-only |
| Freeze manifest | `release/FREEZE_MANIFEST.md` | read-only |
| Operator scripts | `release/start_pilot.sh`, `reset_pilot_data.sh`, `smoke_test.sh`, `generate_evidence_bundle.sh` | executable |
| Demo script | `demo/run_demo.sh` | executable |
| Evidence guide | `release/evidence/README.md` | read-only |
| Walkthrough bundle | `release/walkthrough/` (3 files) | read-only / executable |
| Review packet | `release/review/` (5 files) | read-only |
| Acceptance bundle | `release/acceptance/` (4 files) | read-only / executable |
| Handoff packet | `release/handoff/` (5 files) | read-only / executable |
| Self-check bundle | `release/selfcheck/` (3 files) | read-only / executable |
| Freeze bundle | `release/freeze/` (4 files) | read-only / executable |
| Review-trace bundle | `release/review-trace/` (4 files) | read-only / executable |
| Readiness bundle | `release/readiness/` (this bundle) | read-only / executable |
| Canonical fixtures | `examples/pilot/` (3 files) | read-only / frozen |

---

## Structural validation surfaces present

The following read-only helper scripts check the package structure without modifying anything:

| Script | What it checks |
|---|---|
| `release/selfcheck/run_release_selfcheck.sh` | All release files present and executable |
| `release/print_release_index.sh` | All release surfaces with recommended paths |
| `release/walkthrough/print_walkthrough.sh` | Key files, sequence orientation |
| `release/acceptance/print_acceptance_summary.sh` | Acceptance input structural pre-check |
| `release/handoff/print_handoff_index.sh` | Handoff resource completeness |
| `release/freeze/print_freeze_manifest.sh` | Surface classifications with protocol values |
| `release/review-trace/print_review_trace.sh` | Review order, stop conditions, trace resources |
| `release/readiness/print_readiness_snapshot.sh` | Readiness surfaces, review path, out-of-scope reminder |

---

## Recommended review path

Follow the 8-step trace in `release/review-trace/REVIEW_TRACE.md`:

```
Step 1  cat release/INDEX.md
Step 2  cat release/FREEZE_MANIFEST.md
Step 3  ./release/selfcheck/run_release_selfcheck.sh
Step 4  cat release/walkthrough/PILOT_WALKTHROUGH.md
Step 5  cat release/review/SYSTEM_OVERVIEW.md + OPERATOR_FLOW.md + ARTIFACT_GUIDE.md + BOUNDARIES.md
Step 6  cat release/evidence/current/summary.txt + 06_verify.json + 04_receipt.json
Step 7  ./release/acceptance/print_acceptance_summary.sh
Step 8  ./release/handoff/print_handoff_index.sh
```

Stop conditions for each step are defined in `release/review-trace/STOP_POINTS.md`.

---

## Frozen boundaries summary

| Item | Frozen value |
|---|---|
| Protocol version | `postcad-v1` |
| Routing kernel version | `postcad-routing-v1` |
| Receipt schema version | `1` |
| Deterministic receipt hash | `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb` |
| Canonical pilot case ID | `f1000001-0000-0000-0000-000000000001` |
| Expected verification result | `VERIFIED` |
| Endpoint surface | 7 endpoints frozen (see `release/freeze/FROZEN_BOUNDARIES.md`) |
| Canonical fixtures | `examples/pilot/case.json`, `registry_snapshot.json`, `config.json` |

---

## Local operator path summary

```bash
# Terminal A
./release/start_pilot.sh                        # start service

# Terminal B
./release/reset_pilot_data.sh                   # clean slate
./release/smoke_test.sh                         # 7-step smoke test
./demo/run_demo.sh                              # self-contained demo
./release/generate_evidence_bundle.sh           # capture evidence
cat release/evidence/current/summary.txt        # confirm: All 7 steps passed.
```

---

## Evidence and output summary

When the pilot has been run and evidence generated:

| Artifact | Expected value |
|---|---|
| `evidence/current/summary.txt` last line | `All 7 steps passed.` |
| `evidence/current/06_verify.json` | `{"receipt_hash":"...","result":"VERIFIED"}` |
| `evidence/current/04_receipt.json` `receipt_hash` | `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb` |

`release/evidence/current/` is runtime-generated and gitignored. It must be generated fresh using `generate_evidence_bundle.sh` with the service running.

---

## What "ready for external review" means here

The package is ready for external review when all of the following are true:

1. `release/selfcheck/run_release_selfcheck.sh` reports `Missing items: 0`
2. `release/evidence/current/summary.txt` ends with `All 7 steps passed.`
3. `release/evidence/current/06_verify.json` contains `"result": "VERIFIED"`
4. `release/evidence/current/04_receipt.json` contains the expected `receipt_hash`
5. `release/acceptance/print_acceptance_summary.sh` shows all `[OK]`
6. `release/handoff/print_handoff_index.sh` shows all `[OK]`

"Ready for external review" means the local package is structurally complete and the pilot run produced the expected deterministic outputs. It does not mean production readiness, regulatory approval, or any claim beyond the boundaries in `release/freeze/FROZEN_BOUNDARIES.md`.
