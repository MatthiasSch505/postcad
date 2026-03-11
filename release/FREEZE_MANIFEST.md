# PostCAD Pilot Freeze Manifest

Top-level freeze manifest for the local pilot package. Lists every release surface, its classification, and what is frozen.

---

## What this manifest is for

A single inspectable document that describes the complete local pilot package layout — which files are executable, which are read-only, and which are runtime-generated. Intended for operators and reviewers who need to understand the package at a glance.

---

## Scope

This manifest covers the local pilot package only: the `release/` tree, the `demo/` script, and the canonical fixtures in `examples/pilot/`. No production deployment, no cloud infrastructure, no external services.

---

## Release surfaces

### Operator scripts

| Path | Type | Purpose |
|---|---|---|
| `release/start_pilot.sh` | executable | Build (if needed) and start the HTTP service on `localhost:8080` |
| `release/reset_pilot_data.sh` | executable | Remove all runtime data directories under `data/` |
| `release/smoke_test.sh` | executable | 7-step deterministic smoke test against the live service |
| `release/generate_evidence_bundle.sh` | executable | Capture the full pilot flow to `release/evidence/current/` |
| `demo/run_demo.sh` | executable | Self-contained 8-step demo (starts and stops its own service instance) |

### Release index and runbook

| Path | Type | Purpose |
|---|---|---|
| `release/README.md` | read-only | Operator runbook: prerequisites, exact sequence, troubleshooting |
| `release/INDEX.md` | read-only | Top-level reference table for the entire release/ tree |
| `release/FREEZE_MANIFEST.md` | read-only | This file |
| `release/print_release_index.sh` | executable | Read-only: prints all release surfaces and recommended paths |

### Evidence bundle

| Path | Type | Purpose |
|---|---|---|
| `release/evidence/README.md` | read-only | Folder structure and inspection guide for evidence bundles |
| `release/evidence/current/` | runtime-generated | Output from the last `generate_evidence_bundle.sh` run (gitignored) |

### Walkthrough bundle

| Path | Type | Purpose |
|---|---|---|
| `release/walkthrough/README.md` | read-only | Bundle overview and related packets |
| `release/walkthrough/PILOT_WALKTHROUGH.md` | read-only | 9-step narrative of the full local pilot path |
| `release/walkthrough/print_walkthrough.sh` | executable | Read-only: orientation script with file checks and sequence diagram |

### Review packet

| Path | Type | Purpose |
|---|---|---|
| `release/review/README.md` | read-only | Entry point and reading order for external reviewers |
| `release/review/SYSTEM_OVERVIEW.md` | read-only | Four-layer architecture and current pilot scope |
| `release/review/OPERATOR_FLOW.md` | read-only | Operator sequence with expected output tables |
| `release/review/ARTIFACT_GUIDE.md` | read-only | Field-by-field guide to every evidence file |
| `release/review/BOUNDARIES.md` | read-only | What is frozen, what is not claimed, what is out of scope |

### Acceptance bundle

| Path | Type | Purpose |
|---|---|---|
| `release/acceptance/README.md` | read-only | Entry point: what acceptance means, required inputs |
| `release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md` | read-only | 33-item checklist across 7 sections |
| `release/acceptance/REVIEW_WORKSHEET.md` | read-only | Fill-in worksheet for a formal review record |
| `release/acceptance/print_acceptance_summary.sh` | executable | Read-only: structural `[OK]`/`[--]` pre-check for acceptance inputs |

### Handoff packet

| Path | Type | Purpose |
|---|---|---|
| `release/handoff/README.md` | read-only | Entry point for a new operator or reviewer |
| `release/handoff/FIRST_HOUR_GUIDE.md` | read-only | Exact first-hour sequence with commands and expected outputs |
| `release/handoff/KNOWN_GOOD_STATE.md` | read-only | How to verify the expected current state |
| `release/handoff/HANDOFF_CHECKLIST.md` | read-only | Practical transfer checklist |
| `release/handoff/print_handoff_index.sh` | executable | Read-only: handoff resource index with `[OK]`/`[--]` checks |

### Self-check bundle

| Path | Type | Purpose |
|---|---|---|
| `release/selfcheck/README.md` | read-only | Entry point for the structural self-check |
| `release/selfcheck/SELFCHECK_SCOPE.md` | read-only | Complete scope definition: what is and is not checked |
| `release/selfcheck/run_release_selfcheck.sh` | executable | Read-only: structural file-presence check of the entire release package |

### Freeze bundle

| Path | Type | Purpose |
|---|---|---|
| `release/freeze/README.md` | read-only | Entry point for the freeze bundle |
| `release/freeze/PILOT_SURFACES.md` | read-only | Grouped inventory of every pilot surface with classifications |
| `release/freeze/FROZEN_BOUNDARIES.md` | read-only | Explicit statement of what is frozen |
| `release/freeze/print_freeze_manifest.sh` | executable | Read-only: prints freeze surfaces and classifications |

### Canonical fixtures (not under release/)

| Path | Type | Purpose |
|---|---|---|
| `examples/pilot/case.json` | read-only | Canonical pilot case input |
| `examples/pilot/registry_snapshot.json` | read-only | Manufacturer registry for the pilot flow |
| `examples/pilot/config.json` | read-only | Routing config (jurisdiction + policy) for the pilot flow |

---

## Runtime-generated paths

The following paths are produced at runtime and are excluded from git:

| Path | Generator | Notes |
|---|---|---|
| `release/evidence/current/` | `release/generate_evidence_bundle.sh` | Replaced entirely on each run |
| `data/cases/` | service (`POST /cases`) | Cleared by `reset_pilot_data.sh` |
| `data/receipts/` | service (`POST /cases/:id/route`) | Cleared by `reset_pilot_data.sh` |
| `data/policies/` | service (`POST /cases/:id/route`) | Cleared by `reset_pilot_data.sh` |
| `data/dispatch/` | service (`POST /dispatch/:hash`) | Cleared by `reset_pilot_data.sh` |
| `data/verification/` | service (`POST /dispatch/:hash/verify`) | Cleared by `reset_pilot_data.sh` |

---

## What is frozen

- Protocol version: `postcad-v1`
- Routing kernel version: `postcad-routing-v1`
- Receipt schema version: `1`
- All hash commitment fields and verification logic
- All endpoint paths and response shapes
- Canonical fixtures in `examples/pilot/`
- Protocol test vectors in `tests/protocol_vectors/`

No changes to any of the above are permitted within the scope of the local pilot package.

---

## What this manifest does not claim

- This manifest is a description, not a certification.
- It does not assert regulatory compliance or production readiness.
- It does not guarantee that the service is currently running.
- It does not replace the acceptance checklist (`release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md`).
- It does not claim coverage beyond the single canonical pilot case.
