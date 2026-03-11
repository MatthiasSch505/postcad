# PostCAD Pilot Walkthrough Bundle

A single-surface orientation for running and inspecting the current local PostCAD pilot.

---

## What this is

A read-only walkthrough layer. It describes the exact local pilot sequence, commands, expected outputs, and inspection points — in one place, in order.

It does not replace the detailed docs elsewhere. It points to them at the right moments.

---

## Who it is for

Anyone who needs to orient quickly to the local pilot flow without reading all packets first:
- a new operator about to run the pilot for the first time
- a reviewer who wants to follow the path before inspecting artifacts
- anyone who needs a single ordered reference for the full local flow

---

## What it does and does not do

| Does | Does not |
|---|---|
| Print the exact sequence with commands | Start the service |
| Check whether key files exist | Run the smoke test |
| Show expected output for each step | Generate evidence |
| Point to inspection locations | Change any file |
| Orient a new operator in under 10 minutes | Declare acceptance |

---

## Contents

| File | Purpose |
|---|---|
| `PILOT_WALKTHROUGH.md` | Full narrative of the 9-step local pilot path |
| `print_walkthrough.sh` | Read-only script: prints sequence, checks key files, shows commands |

---

## Related packets

| Packet | Path |
|---|---|
| Operator runbook | `release/README.md` |
| Review packet | `release/review/` |
| Evidence bundle guide | `release/evidence/README.md` |
| Acceptance bundle | `release/acceptance/` |
| Handoff packet | `release/handoff/` |
