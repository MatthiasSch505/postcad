# PostCAD Pilot Readiness Snapshot

Entry point for the readiness bundle — a single-page summary of the current local pilot package state.

---

## What this bundle is for

A top-level summary that consolidates what exists, what has been validated structurally, what the review path is, and what is intentionally outside the current pilot scope. It does not add new behavior. It describes the current state of the package.

---

## Who should use it

- A reviewer who wants a single-page orientation before starting the review trace
- An operator confirming the package is complete before handing it off
- Anyone who wants to understand at a glance what the local pilot package contains and what it does not claim

---

## Recommended reading order inside this bundle

1. **`READINESS_SNAPSHOT.md`** — current-state summary: surfaces present, review path, frozen values, output state
2. **`OUT_OF_SCOPE.md`** — explicit list of what this package does not claim
3. **`print_readiness_snapshot.sh`** — optional: run to see the snapshot at a glance with existence checks

---

## Relation to other surfaces

| Surface | Role |
|---|---|
| `release/INDEX.md` | Full reference table of every release surface |
| `release/FREEZE_MANIFEST.md` | Classified surface listing with frozen values |
| `release/selfcheck/run_release_selfcheck.sh` | Structural file-presence check |
| `release/review-trace/REVIEW_TRACE.md` | 8-step ordered review path with stop conditions |
| `release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md` | 33-item acceptance criteria |
| `release/handoff/FIRST_HOUR_GUIDE.md` | First-hour guide for a new operator |

---

## Command

```bash
./release/readiness/print_readiness_snapshot.sh
```
