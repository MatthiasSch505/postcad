# PostCAD Pilot Release Index

Top-level entrypoint for the `release/` directory. Describes every surface, who it is for, and the recommended order to use it.

---

## What this directory contains

The `release/` tree is the local pilot package: scripts to run the service, check correctness, capture evidence, and orient a reviewer. It contains no production deployment config, no cloud infrastructure, and no external dependencies beyond the local Rust build.

---

## Who it is for

| Person | Where to start |
|---|---|
| New operator running for the first time | `release/handoff/FIRST_HOUR_GUIDE.md` |
| Reviewer or auditor evaluating the pilot | `release/walkthrough/PILOT_WALKTHROUGH.md` |
| Anyone who just landed in `release/` | This file, then see paths below |

---

## Release surfaces

### Operator scripts (executable)

| Path | Purpose | Action |
|---|---|---|
| `release/start_pilot.sh` | Build (if needed) and start the HTTP service | **run** |
| `release/reset_pilot_data.sh` | Remove all runtime data directories | **run** |
| `release/smoke_test.sh` | 7-step deterministic smoke test against live service | **run** |
| `release/generate_evidence_bundle.sh` | Capture the full pilot flow to `release/evidence/current/` | **run** |

### Operator documentation

| Path | Purpose | Action |
|---|---|---|
| `release/README.md` | Operator runbook: prerequisites, sequence, troubleshooting | read |

### Walkthrough bundle (`release/walkthrough/`)

| Path | Purpose | Action |
|---|---|---|
| `release/walkthrough/PILOT_WALKTHROUGH.md` | 9-step narrative of the full local pilot path | read |
| `release/walkthrough/README.md` | Bundle overview and related packets | read |
| `release/walkthrough/print_walkthrough.sh` | Read-only orientation: HEAD, file checks, sequence, expected outputs | **run** |

### Evidence bundle (`release/evidence/`)

| Path | Purpose | Action |
|---|---|---|
| `release/evidence/README.md` | Folder structure and inspection guide | read |
| `release/evidence/current/` | Output from the last evidence run (gitignored) | inspect |
| `release/evidence/current/summary.txt` | Human-readable pass/fail confirmation | inspect |

### Review packet (`release/review/`)

| Path | Purpose | Action |
|---|---|---|
| `release/review/README.md` | Entry point and reading order | read |
| `release/review/SYSTEM_OVERVIEW.md` | Four-layer architecture, current pilot scope | read |
| `release/review/OPERATOR_FLOW.md` | Operator sequence with expected output tables | read |
| `release/review/ARTIFACT_GUIDE.md` | Every field in every evidence file | read |
| `release/review/BOUNDARIES.md` | What is frozen, what is not claimed, what is out of scope | read |

### Acceptance bundle (`release/acceptance/`)

| Path | Purpose | Action |
|---|---|---|
| `release/acceptance/README.md` | Entry point: what acceptance means, required inputs | read |
| `release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md` | 33-item checklist across 7 sections | read |
| `release/acceptance/REVIEW_WORKSHEET.md` | Fill-in worksheet for a formal review record | read |
| `release/acceptance/print_acceptance_summary.sh` | Read-only pre-check: structural `[OK]`/`[--]` for all checklist inputs | **run** |

### Handoff packet (`release/handoff/`)

| Path | Purpose | Action |
|---|---|---|
| `release/handoff/README.md` | Entry point for a new operator or reviewer | read |
| `release/handoff/FIRST_HOUR_GUIDE.md` | Exact first-hour sequence with commands and expected outputs | read |
| `release/handoff/KNOWN_GOOD_STATE.md` | How to verify the expected current state | read |
| `release/handoff/HANDOFF_CHECKLIST.md` | Practical transfer checklist | read |
| `release/handoff/print_handoff_index.sh` | Read-only index: HEAD, script presence, evidence presence | **run** |

### Freeze bundle (`release/freeze/`)

| Path | Purpose | Action |
|---|---|---|
| `release/FREEZE_MANIFEST.md` | Single-page freeze manifest with full surface listing | read |
| `release/freeze/README.md` | Entry point for the freeze bundle | read |
| `release/freeze/PILOT_SURFACES.md` | Grouped inventory of every pilot surface with classifications | read |
| `release/freeze/FROZEN_BOUNDARIES.md` | Explicit statement of what is frozen | read |
| `release/freeze/print_freeze_manifest.sh` | Read-only: prints classified surface listing and protocol values | **run** |

### Self-check bundle (`release/selfcheck/`)

| Path | Purpose | Action |
|---|---|---|
| `release/selfcheck/README.md` | What the self-check covers and how to run it | read |
| `release/selfcheck/SELFCHECK_SCOPE.md` | Complete scope definition: checked vs not checked | read |
| `release/selfcheck/run_release_selfcheck.sh` | Read-only structural check of the entire release package | **run** |

---

## Minimal path

Fastest route to confirm the pilot runs correctly. Requires two terminals.

```bash
# Terminal A — start service (leave running)
./release/start_pilot.sh

# Terminal B
./release/reset_pilot_data.sh          # clean slate
./release/smoke_test.sh                # 7-step smoke test → "SMOKE TEST PASSED"
./demo/run_demo.sh                     # self-contained 8-step demo (no Terminal A needed)
./release/generate_evidence_bundle.sh  # capture evidence
cat release/evidence/current/summary.txt
```

Expected final line of summary.txt: `All 7 steps passed.`

---

## Full review path

For a reviewer or new operator who needs to understand and verify the whole package.

```bash
# Orient
./release/walkthrough/print_walkthrough.sh    # file checks + sequence diagram
cat release/walkthrough/PILOT_WALKTHROUGH.md  # full 9-step narrative

# Run the pilot (minimal path above)

# Inspect review packet
cat release/review/README.md
cat release/review/SYSTEM_OVERVIEW.md
cat release/review/ARTIFACT_GUIDE.md

# Acceptance pre-check
./release/acceptance/print_acceptance_summary.sh
cat release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md

# Handoff index
./release/handoff/print_handoff_index.sh
cat release/handoff/FIRST_HOUR_GUIDE.md
```

---

## Quick-reference: executable scripts

| Script | Purpose | Must be run from |
|---|---|---|
| `release/start_pilot.sh` | Start service | repo root |
| `release/reset_pilot_data.sh` | Clean runtime data | repo root |
| `release/smoke_test.sh` | Smoke test (service must be running) | repo root |
| `release/generate_evidence_bundle.sh` | Capture evidence (service must be running) | repo root |
| `demo/run_demo.sh` | Self-contained 8-step demo | repo root |
| `release/freeze/print_freeze_manifest.sh` | Classified surface listing with protocol values | repo root or `release/freeze/` |
| `release/selfcheck/run_release_selfcheck.sh` | Structural self-check of the whole release package | repo root or `release/selfcheck/` |
| `release/print_release_index.sh` | This index as a read-only script | repo root |
| `release/walkthrough/print_walkthrough.sh` | Walkthrough orientation | repo root or `release/walkthrough/` |
| `release/acceptance/print_acceptance_summary.sh` | Acceptance pre-check | repo root or `release/acceptance/` |
| `release/handoff/print_handoff_index.sh` | Handoff resource index | repo root or `release/handoff/` |
