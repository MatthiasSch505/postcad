# PostCAD Pilot Review Trace

A single ordered path through the local pilot package for a reviewer or new operator.

---

## What this bundle is for

It tells you what to look at, in what order, what question each step answers, and when to stop. It does not add new information — it sequences the existing surfaces into one deterministic review path.

---

## Who should use it

- A reviewer who received the repo and wants to evaluate the pilot package end to end
- An operator who wants to confirm the package is complete before handing it off
- Anyone who needs to know when the package is "ready for external review"

---

## Recommended order inside this bundle

1. **`REVIEW_TRACE.md`** — the ordered review path, step by step
2. **`STOP_POINTS.md`** — explicit conditions that require stopping before proceeding
3. **`print_review_trace.sh`** — optional: run to see the review order and key checks at a glance

---

## Relation to other surfaces

| Surface | Role in review |
|---|---|
| `release/INDEX.md` | Reference table — consulted during trace step 1 |
| `release/FREEZE_MANIFEST.md` | Freeze scope — consulted during trace step 2 |
| `release/selfcheck/run_release_selfcheck.sh` | Structural check — run at trace step 3 |
| `release/walkthrough/PILOT_WALKTHROUGH.md` | Operator path — read at trace step 4 |
| `release/review/` | System description — inspected at trace step 5 |
| `release/evidence/current/` | Run artifacts — inspected at trace step 6 |
| `release/acceptance/` | Acceptance criteria — applied at trace step 7 |
| `release/handoff/` | Transfer materials — confirmed at trace step 8 |

---

## Command

```bash
./release/review-trace/print_review_trace.sh
```
