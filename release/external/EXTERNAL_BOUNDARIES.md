# PostCAD Pilot — External Boundaries

Explicit scope boundary for this external delivery. States what this package is and what it does not claim.

---

## What this package is

A deterministic local pilot package for the PostCAD routing and verification system. It demonstrates a complete case-intake → routing → receipt → dispatch → verification flow using fixed canonical inputs, running locally on a single machine.

---

## What this package is not

**Not a production deployment.**
The service runs on `localhost:8080` only. There is no hosted infrastructure, no remote endpoints, no network exposure beyond the local machine.

**Not a hosted or remote service.**
All operations are local. No external APIs are called. No cloud infrastructure is used.

**Not a certification or regulatory submission.**
This package makes no claim of CE mark, FDA clearance, MHLW approval, ISO certification, or any other regulatory or standards status.

**Not a protocol redesign.**
The protocol version is frozen at `postcad-v1`. No protocol changes are included or implied.

**Not a routing algorithm redesign.**
The routing kernel version is frozen at `postcad-routing-v1`. The selection logic is frozen.

**Not a schema change.**
The receipt schema version is `1`. All field names and types are frozen.

**Not a coverage claim beyond the canonical pilot case.**
This package covers exactly one canonical case: `case_id = f1000001-0000-0000-0000-000000000001`, jurisdiction DE, zirconia crown. Performance, load, and edge-case behavior are not claimed.

**Not an automated acceptance engine.**
The acceptance checklist (`release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md`) is a human-review instrument. No automated acceptance verdict is produced.

---

## Where to look for detailed scope

| Question | Where to look |
|---|---|
| What is frozen? | `release/freeze/FROZEN_BOUNDARIES.md` |
| What does the system do? | `release/review/SYSTEM_OVERVIEW.md` |
| What are the acceptance criteria? | `release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md` |
| What is out of scope? | `release/readiness/OUT_OF_SCOPE.md` |
| What surfaces are included? | `release/RELEASE_NOTES_PILOT.md` |
