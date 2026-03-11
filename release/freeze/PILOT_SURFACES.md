# PostCAD Pilot Surfaces

Grouped inventory of every file in the local pilot package with classification and purpose. Matches the current repo state exactly.

Classifications:
- **executable** — shell script, must be run from repo root unless noted
- **read-only** — documentation; do not modify during a pilot run
- **runtime-generated** — produced by a script or service; excluded from git

---

## Top-level operator scripts

| Path | Classification | Purpose |
|---|---|---|
| `release/start_pilot.sh` | executable | Build (if needed) and start the HTTP service |
| `release/reset_pilot_data.sh` | executable | Remove all runtime data directories |
| `release/smoke_test.sh` | executable | 7-step deterministic smoke test (service must be running) |
| `release/generate_evidence_bundle.sh` | executable | Capture the full 7-step pilot flow to `release/evidence/current/` |
| `demo/run_demo.sh` | executable | Self-contained 8-step demo (manages its own service instance) |

---

## Release index and manifest files

| Path | Classification | Purpose |
|---|---|---|
| `release/README.md` | read-only | Operator runbook: prerequisites, exact sequence, troubleshooting |
| `release/INDEX.md` | read-only | Top-level reference table for the entire release/ tree |
| `release/FREEZE_MANIFEST.md` | read-only | Single-page freeze manifest with full surface listing |
| `release/print_release_index.sh` | executable | Read-only: prints all release surfaces and recommended paths |

---

## Evidence bundle

| Path | Classification | Purpose |
|---|---|---|
| `release/evidence/README.md` | read-only | Folder structure and inspection guide |
| `release/evidence/current/` | runtime-generated | Output from the last `generate_evidence_bundle.sh` run (gitignored) |
| `release/evidence/current/summary.txt` | runtime-generated | Human-readable pass/fail confirmation |
| `release/evidence/current/01_health.json` – `07_route_history.json` | runtime-generated | API responses from each pilot step |
| `release/evidence/current/04_receipt.json` | runtime-generated | Full routing receipt with all hash commitments |
| `release/evidence/current/06_verify.json` | runtime-generated | Verification result (`"result": "VERIFIED"`) |
| `release/evidence/current/inputs/` | runtime-generated | Copies of canonical fixture inputs used in the run |
| `release/evidence/current/data_artifacts/` | runtime-generated | Files written by the service under `data/` |
| `release/evidence/current/git_head.txt` | runtime-generated | Commit hash at time of capture |

---

## Walkthrough bundle

| Path | Classification | Purpose |
|---|---|---|
| `release/walkthrough/README.md` | read-only | Bundle overview and related packets |
| `release/walkthrough/PILOT_WALKTHROUGH.md` | read-only | 9-step narrative of the full local pilot path |
| `release/walkthrough/print_walkthrough.sh` | executable | Read-only: orientation script with file checks and sequence diagram |

---

## Review packet

| Path | Classification | Purpose |
|---|---|---|
| `release/review/README.md` | read-only | Entry point and reading order for external reviewers |
| `release/review/SYSTEM_OVERVIEW.md` | read-only | Four-layer architecture and current pilot scope |
| `release/review/OPERATOR_FLOW.md` | read-only | Operator sequence with expected output tables |
| `release/review/ARTIFACT_GUIDE.md` | read-only | Field-by-field guide to every evidence file |
| `release/review/BOUNDARIES.md` | read-only | What is frozen, what is not claimed, what is out of scope |

---

## Acceptance bundle

| Path | Classification | Purpose |
|---|---|---|
| `release/acceptance/README.md` | read-only | Entry point: what acceptance means, required inputs |
| `release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md` | read-only | 33-item checklist across 7 sections |
| `release/acceptance/REVIEW_WORKSHEET.md` | read-only | Fill-in worksheet for a formal review record |
| `release/acceptance/print_acceptance_summary.sh` | executable | Read-only: structural `[OK]`/`[--]` pre-check for acceptance inputs |

---

## Handoff packet

| Path | Classification | Purpose |
|---|---|---|
| `release/handoff/README.md` | read-only | Entry point for a new operator or reviewer |
| `release/handoff/FIRST_HOUR_GUIDE.md` | read-only | Exact first-hour sequence with commands and expected outputs |
| `release/handoff/KNOWN_GOOD_STATE.md` | read-only | How to verify the expected current state |
| `release/handoff/HANDOFF_CHECKLIST.md` | read-only | Practical transfer checklist |
| `release/handoff/print_handoff_index.sh` | executable | Read-only: handoff resource index with `[OK]`/`[--]` checks |

---

## Self-check bundle

| Path | Classification | Purpose |
|---|---|---|
| `release/selfcheck/README.md` | read-only | Entry point for the structural self-check |
| `release/selfcheck/SELFCHECK_SCOPE.md` | read-only | Complete scope definition: what is and is not checked |
| `release/selfcheck/run_release_selfcheck.sh` | executable | Read-only: structural file-presence check of the entire release package |

---

## Freeze bundle (this bundle)

| Path | Classification | Purpose |
|---|---|---|
| `release/freeze/README.md` | read-only | Entry point for the freeze bundle |
| `release/freeze/PILOT_SURFACES.md` | read-only | This file: grouped inventory with classifications |
| `release/freeze/FROZEN_BOUNDARIES.md` | read-only | Explicit statement of what is frozen |
| `release/freeze/print_freeze_manifest.sh` | executable | Read-only: prints freeze surfaces and classifications |

---

## Canonical fixtures (not under release/)

| Path | Classification | Purpose |
|---|---|---|
| `examples/pilot/case.json` | read-only | Canonical pilot case input (frozen) |
| `examples/pilot/registry_snapshot.json` | read-only | Manufacturer registry for the pilot flow (frozen) |
| `examples/pilot/config.json` | read-only | Routing config for the pilot flow (frozen) |

---

## Runtime-generated data paths (not in git)

| Path | Generator |
|---|---|
| `data/cases/` | service: `POST /cases` |
| `data/receipts/` | service: `POST /cases/:id/route` |
| `data/policies/` | service: `POST /cases/:id/route` |
| `data/dispatch/` | service: `POST /dispatch/:hash` |
| `data/verification/` | service: `POST /dispatch/:hash/verify` |

All cleared by `release/reset_pilot_data.sh`.
