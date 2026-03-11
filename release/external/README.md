# PostCAD Pilot — External Delivery Packet

Entry point for an external reviewer or operator receiving this local pilot package.

---

## What this packet is

A small, oriented subset of the local pilot package for external review. It points to the relevant existing surfaces in order and states explicitly what this delivery includes and does not claim.

This is a local pilot delivery surface. It is not a production system, a hosted service, or a regulatory submission.

---

## Who it is for

- An external reviewer evaluating the PostCAD local pilot
- A technical reviewer who was not involved in building the package
- Anyone receiving this repository for the first time

---

## Recommended reading order

1. **`EXTERNAL_DELIVERY_OVERVIEW.md`** (this bundle) — what you are receiving and what to inspect
2. **`EXTERNAL_REVIEW_PATH.md`** (this bundle) — concise step-by-step inspection path
3. **`EXTERNAL_BOUNDARIES.md`** (this bundle) — what this delivery does not claim
4. **`release/RELEASE_NOTES_PILOT.md`** — full list of included surfaces and frozen values
5. **`release/review/README.md`** — system description and reading order for the review packet

From there, follow `EXTERNAL_REVIEW_PATH.md` for the complete inspection sequence.

---

## Key existing surfaces for external reviewers

| Surface | Path | Purpose |
|---|---|---|
| Release notes | `release/RELEASE_NOTES_PILOT.md` | Full surface listing, frozen values, verification steps |
| System overview | `release/review/SYSTEM_OVERVIEW.md` | What the system does and what the pilot covers |
| Artifact guide | `release/review/ARTIFACT_GUIDE.md` | Every field in every evidence file |
| Frozen boundaries | `release/review/BOUNDARIES.md` | What is frozen, what is not claimed |
| Evidence guide | `release/evidence/README.md` | Evidence bundle structure and what to inspect |
| Acceptance checklist | `release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md` | 33-item criteria across 7 sections |
| Handoff guide | `release/handoff/FIRST_HOUR_GUIDE.md` | First-hour sequence for a new operator |
| Self-check | `release/selfcheck/run_release_selfcheck.sh` | Structural package check |

---

## Command

```bash
./release/external/print_external_packet.sh
```
