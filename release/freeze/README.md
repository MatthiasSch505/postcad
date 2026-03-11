# PostCAD Pilot Freeze Bundle

Entry point for the freeze documentation layer.

---

## What this is

A set of read-only documents that record, in one place, the complete inventory of pilot surfaces and the explicit boundaries that are frozen for the current local pilot. It does not add new behavior — it describes what already exists.

---

## Who should use it

- A reviewer who needs to understand what is and is not in scope for the pilot
- An operator who wants to confirm what files make up the release package
- Anyone who needs to distinguish frozen protocol behavior from packaging/orientation documents

---

## How it relates to other release surfaces

| Surface | Relationship |
|---|---|
| `release/INDEX.md` | Navigational reference table — start there for paths and recommended order |
| `release/FREEZE_MANIFEST.md` | Top-level freeze manifest — the single-page summary |
| `release/selfcheck/` | Structural file-presence check — run before inspecting |
| `release/review/` | Detailed system description for external reviewers |
| `release/acceptance/` | Acceptance checklist and worksheet — separate from freeze |
| `release/handoff/` | First-hour guide and transfer checklist for new operators |

---

## Recommended reading order inside this bundle

1. `PILOT_SURFACES.md` — complete grouped inventory with classifications
2. `FROZEN_BOUNDARIES.md` — what is frozen and what is out of scope
3. `print_freeze_manifest.sh` — optional: run to see the current state at a glance
