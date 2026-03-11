# PostCAD Pilot — External Delivery Overview

**Pilot label:** `pilot-local-v1`

---

## What this package demonstrates

A complete deterministic routing and verification flow for a local dental CAD post-processing platform. The system verifies manufacturer certifications, checks regulatory constraints by jurisdiction, routes cases to eligible manufacturers, and records an immutable audit trail.

Key properties demonstrated:
- Deterministic routing: same inputs always produce the same routing decision
- Verifiable receipts: the routing receipt is self-verifiable without the routing engine
- Audit trail: each receipt is hash-chained and append-only
- Local operation only: no external services, no network dependencies beyond localhost

---

## What the package contains

All surfaces are committed to the `main` branch and described in `release/RELEASE_NOTES_PILOT.md`.

| Category | What is included |
|---|---|
| Operator scripts | Start service, reset data, smoke test, evidence capture |
| Demo | Self-contained 8-step demo (`demo/run_demo.sh`) |
| Review packet | System overview, operator flow, artifact guide, boundaries |
| Evidence bundle | Infrastructure to capture and inspect a full pilot run |
| Acceptance bundle | 33-item checklist, review worksheet, structural pre-check |
| Handoff packet | First-hour guide, known-good state, transfer checklist |
| Canonical fixtures | Frozen pilot inputs in `examples/pilot/` |

---

## Operator path (high level)

```bash
# Terminal A
./release/start_pilot.sh                       # start service on localhost:8080

# Terminal B
./release/reset_pilot_data.sh                  # clean slate
./release/smoke_test.sh                        # 7-step smoke test
./release/generate_evidence_bundle.sh          # capture evidence
cat release/evidence/current/summary.txt       # confirm: All 7 steps passed.
```

Full walkthrough: `release/walkthrough/PILOT_WALKTHROUGH.md`

---

## Evidence surfaces available for inspection

After running `generate_evidence_bundle.sh` with the service running:

| File | Contents |
|---|---|
| `release/evidence/current/summary.txt` | Pass/fail confirmation; last line: `All 7 steps passed.` |
| `release/evidence/current/06_verify.json` | Verification result: `{"result":"VERIFIED"}` |
| `release/evidence/current/04_receipt.json` | Full routing receipt with all hash commitments |
| `release/evidence/current/inputs/` | Exact fixture inputs used in the run |

---

## What a reviewer is expected to inspect

1. The structural self-check confirms the package is complete
2. The review packet describes what the system does and how it was run
3. The evidence bundle shows the actual pilot run outputs
4. The acceptance checklist provides the formal criteria for evaluation
5. The handoff packet confirms the package can be transferred and re-run

---

## How to verify the checked-out commit and package state

```bash
git branch --show-current                      # expected: main
git log --oneline -1                           # review HEAD
git status                                     # expected: clean
cargo test --workspace                         # all suites pass
./release/selfcheck/run_release_selfcheck.sh   # Missing items: 0
```

Frozen receipt hash for the canonical pilot inputs (deterministic across all runs):
`0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb`
