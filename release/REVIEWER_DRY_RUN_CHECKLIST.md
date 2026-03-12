# PostCAD Pilot — Reviewer Dry-Run Checklist

**Pilot label:** `pilot-local-v1`
**Purpose:** Structured evaluation aid for an external reviewer running the frozen local pilot.
**Note:** This is a local pilot only — not a production system, not a regulatory submission.

---

## Prerequisites

- Repository cloned from source, working tree clean (`git status` shows nothing to commit)
- Tools installed: `cargo` (Rust toolchain), `curl`, `python3`
- Starting point: repo root directory

---

## Section 1 — Package open / orientation

- [ ] Reviewer pack folder exists: `release/out/reviewer-pack-pilot-local-v1/`
- [ ] `README.md` is present in the pack and readable
- [ ] `HANDOFF_MESSAGE.txt` is present in the pack
- [ ] `PILOT_VERSION.md` is present and states label `pilot-local-v1`
- [ ] `RELEASE_NOTES_PILOT.md` is present and lists frozen protocol values

---

## Section 2 — Smoke run execution

Run from repo root:

```bash
scripts/external_pilot_smoke.sh
```

- [ ] Script is present and executable
- [ ] Service started without error
- [ ] All 7 pilot flow steps completed
- [ ] Final output line observed:

```
SMOKE RUN PASSED — 10 stages OK
```

---

## Section 3 — Frozen reference validation

After the smoke run completes, confirm the printed values match:

| Item | Expected value |
|---|---|
| Protocol version | `postcad-v1` |
| Routing kernel | `postcad-routing-v1` |
| Canonical case ID | `f1000001-0000-0000-0000-000000000001` |
| Receipt hash | `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb` |
| Verify result | `VERIFIED` |

- [ ] Protocol version matches
- [ ] Routing kernel version matches
- [ ] Canonical case ID matches
- [ ] Receipt hash matches frozen reference exactly
- [ ] Verify result is `VERIFIED`

---

## Section 4 — Reviewer understanding

- [ ] The purpose of the package is understandable from `README.md` alone
- [ ] The included files and their purposes are clear
- [ ] The single rerun command (`scripts/external_pilot_smoke.sh`) is the obvious entry point
- [ ] The path from "clone repo" to "pilot verified" is clear without insider guidance

---

## Section 5 — Out-of-scope acknowledgment

- [ ] This is a local pilot only — not a production deployment
- [ ] No external services, cloud infrastructure, or network dependencies are required or claimed
- [ ] No regulatory or certification status is claimed (no CE mark, FDA clearance, MHLW approval, etc.)
- [ ] Coverage is limited to one canonical pilot case (`f1000001-0000-0000-0000-000000000001`, DE, zirconia crown)
- [ ] The acceptance checklist (`release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md`) is a human-review instrument — no automated verdict

---

## Reviewer outcome

Complete after finishing the above sections.

- **Overall review result:** `PASS` / `PASS WITH QUESTIONS` / `FAIL`
- **Main issue encountered:** _(leave blank if none)_
- **Main question:** _(leave blank if none)_
- **Suggested next step:** _(leave blank if none)_
