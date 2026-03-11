# PostCAD Release Self-Check Scope

Defines exactly what `run_release_selfcheck.sh` checks and does not check.

---

## What is checked

### Git state
- Current branch name
- Current HEAD commit hash
- Whether the working tree is clean or dirty

### Operator scripts
Existence and executability of:
- `release/start_pilot.sh`
- `release/reset_pilot_data.sh`
- `release/smoke_test.sh`
- `release/generate_evidence_bundle.sh`
- `demo/run_demo.sh`

### Release index and runbook
Existence of:
- `release/INDEX.md`
- `release/README.md`
- `release/print_release_index.sh`

### Walkthrough bundle (`release/walkthrough/`)
Existence of:
- `release/walkthrough/README.md`
- `release/walkthrough/PILOT_WALKTHROUGH.md`
- `release/walkthrough/print_walkthrough.sh` (and executability)

### Evidence bundle (`release/evidence/`)
Existence of:
- `release/evidence/README.md`

Presence (informational only, not required for pass) of:
- `release/evidence/current/` directory
- `release/evidence/current/summary.txt` (and whether it contains "All 7 steps passed")

### Review packet (`release/review/`)
Existence of:
- `release/review/README.md`
- `release/review/SYSTEM_OVERVIEW.md`
- `release/review/OPERATOR_FLOW.md`
- `release/review/ARTIFACT_GUIDE.md`
- `release/review/BOUNDARIES.md`

### Acceptance bundle (`release/acceptance/`)
Existence of:
- `release/acceptance/README.md`
- `release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md`
- `release/acceptance/REVIEW_WORKSHEET.md`
- `release/acceptance/print_acceptance_summary.sh` (and executability)

### Handoff packet (`release/handoff/`)
Existence of:
- `release/handoff/README.md`
- `release/handoff/FIRST_HOUR_GUIDE.md`
- `release/handoff/KNOWN_GOOD_STATE.md`
- `release/handoff/HANDOFF_CHECKLIST.md`
- `release/handoff/print_handoff_index.sh` (and executability)

### Self-check bundle (`release/selfcheck/`)
Existence of:
- `release/selfcheck/README.md`
- `release/selfcheck/SELFCHECK_SCOPE.md`

### Canonical fixtures
Existence of:
- `examples/pilot/case.json`
- `examples/pilot/registry_snapshot.json`
- `examples/pilot/config.json`

### Cross-references (simple path checks)
A sample of paths named in `release/INDEX.md` and `release/README.md` that must resolve:
- `demo/run_demo.sh` — referenced as self-contained demo
- `examples/pilot/` — referenced as canonical fixtures location

---

## What is NOT checked

- **Protocol correctness** — whether the routing kernel produces correct outputs
- **Routing behavior** — whether `selected_candidate_id`, `receipt_hash`, or any other field is correct
- **Endpoint behavior** — no HTTP calls are made; the service is not started
- **JSON artifact validity** — no JSON parsing; no field-level checks on evidence files
- **Acceptance decisions** — this script does not determine whether the pilot passed acceptance
- **Semantic consistency** — whether doc content accurately describes the system
- **Compilation or test passage** — no `cargo build` or `cargo test` is run
- **Service status** — whether the service is currently running on any port
- **Certification or compliance** — no claim of regulatory or standards compliance
- **Completeness of evidence** — missing `evidence/current/` is noted but not treated as a self-check failure; evidence is gitignored and must be generated fresh

---

## How to use alongside other release surfaces

| Surface | Purpose | When to use |
|---|---|---|
| `release/selfcheck/run_release_selfcheck.sh` | Structural file-presence check | First, before anything else |
| `release/INDEX.md` | Full reference table of every release surface | Orientation |
| `release/print_release_index.sh` | Release surface listing with recommended paths | Quick overview |
| `release/walkthrough/print_walkthrough.sh` | Walkthrough orientation with sequence diagram | Before first pilot run |
| `release/acceptance/print_acceptance_summary.sh` | Structural pre-check for acceptance inputs | After evidence is generated |
| `release/handoff/print_handoff_index.sh` | Handoff resource index | When handing off to a new operator |

The self-check is the fastest first pass. If anything shows `[--]`, investigate before proceeding. If all show `[OK]`, the release package structure is intact.
