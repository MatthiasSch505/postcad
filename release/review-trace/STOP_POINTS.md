# PostCAD Pilot Review Trace — Stop Points

Explicit conditions that require stopping the review before proceeding. Each stop point is deterministic: if the condition is true, do not continue to the next trace step until it is resolved.

---

## S1 — Expected release path missing

**Condition:** `release/selfcheck/run_release_selfcheck.sh` reports one or more `[--]` lines for a non-evidence resource.

**Why it matters:** A missing release file means the package is incomplete. A reviewer cannot evaluate what is not present. Evidence absence alone is acceptable (it is gitignored and must be generated fresh), but any other missing item indicates an incomplete package.

**Where to resolve:** Run `git status` and `git log --oneline -5` to identify whether files were deleted or never committed. Restore from git history or re-create from the appropriate step.

---

## S2 — Self-check reports missing items

**Condition:** `./release/selfcheck/run_release_selfcheck.sh` exits with `Missing items: N` where N > 0, for non-evidence items.

**Why it matters:** The self-check is the minimum structural gate. If it fails, subsequent review steps will reference files that do not exist.

**Where to resolve:** See `release/selfcheck/SELFCHECK_SCOPE.md` for the complete list of checked items. Identify which items are missing and address each one before continuing.

---

## S3 — Evidence bundle absent or incomplete

**Condition:** `release/evidence/current/` does not exist, or exists but `summary.txt` does not end with `All 7 steps passed.`

**Why it matters:** The evidence bundle is the primary artifact of the pilot run. A reviewer cannot evaluate routing behavior, receipt correctness, or verification outcomes without it.

**Where to resolve:**
1. Start the service: `./release/start_pilot.sh` (Terminal A)
2. Reset data: `./release/reset_pilot_data.sh`
3. Generate evidence: `./release/generate_evidence_bundle.sh` (Terminal B)
4. Confirm: `cat release/evidence/current/summary.txt`

---

## S4 — Receipt hash mismatch

**Condition:** `release/evidence/current/04_receipt.json` contains a `receipt_hash` value other than `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb`.

**Why it matters:** The receipt hash is deterministic for the canonical pilot inputs. A different value means either the canonical fixtures were modified, the service behavior changed, or a non-canonical input was used.

**Where to resolve:**
1. Confirm fixtures are unmodified: `git diff examples/pilot/`
2. Confirm the service binary is built from the current source: rebuild with `cargo build -p postcad-service`
3. Reset and re-run: `./release/reset_pilot_data.sh` then regenerate evidence
4. If the hash still differs, investigate the routing kernel — do not proceed to acceptance review until the hash matches

---

## S5 — Verification result is not VERIFIED

**Condition:** `release/evidence/current/06_verify.json` does not contain `"result": "VERIFIED"`.

**Why it matters:** The verification step confirms that the routing receipt is internally consistent and the audit chain is intact. Any result other than `VERIFIED` means the receipt or policy did not pass replay verification.

**Where to resolve:**
1. Inspect `06_verify.json` for the actual result and error code
2. Reset data and regenerate evidence cleanly
3. If the error persists after a clean run, do not proceed — the system is not in a known-good state

---

## S6 — Review packet references non-existent paths

**Condition:** A document in `release/review/` references a file, command, or output that does not exist in the current repo.

**Why it matters:** A review packet that points to missing resources cannot be used to evaluate the system. It creates confusion for an external reviewer.

**Where to resolve:** Update the review packet document to match the actual current state. Do not modify referenced scripts or canonical files to match outdated docs.

---

## S7 — Acceptance pre-check shows structural failures

**Condition:** `./release/acceptance/print_acceptance_summary.sh` shows `[--]` lines.

**Why it matters:** The acceptance pre-check confirms that the structural inputs required for acceptance evaluation are in place. A failing pre-check means the acceptance checklist cannot be applied.

**Where to resolve:** Identify which acceptance inputs are missing (evidence bundle, review packet files, canonical fixtures). Resolve each missing item before proceeding to the acceptance checklist.

---

## S8 — Handoff index shows missing resources

**Condition:** `./release/handoff/print_handoff_index.sh` shows `[--]` lines for operator scripts, canonical fixtures, review packet files, acceptance docs, or handoff docs.

**Why it matters:** A handoff with missing resources cannot be completed. A new operator receiving an incomplete package will be unable to run or verify the pilot.

**Where to resolve:** Address each missing item identified by the handoff index. The handoff is only ready when all resources are present and the evidence bundle shows a passing run.

---

## S9 — Freeze manifest contradicted by another surface

**Condition:** A surface (review packet, acceptance checklist, walkthrough, or handoff guide) describes behavior, file paths, or expected values that contradict `release/FREEZE_MANIFEST.md` or `release/freeze/FROZEN_BOUNDARIES.md`.

**Why it matters:** Contradictions between surfaces indicate that documentation was not updated consistently after a change. A reviewer cannot determine which surface is authoritative.

**Where to resolve:** Identify the source of the contradiction. If the frozen behavior changed, update all affected surfaces. If only the docs diverged, correct the outdated doc to match the canonical definition in `FROZEN_BOUNDARIES.md`.

---

## Summary table

| ID | Condition | Blocks |
|---|---|---|
| S1 | Release path missing | All subsequent steps |
| S2 | Self-check `Missing items: N > 0` | All subsequent steps |
| S3 | Evidence absent or incomplete | Steps 6, 7, 8 |
| S4 | Receipt hash mismatch | Step 6, acceptance review |
| S5 | Verification not VERIFIED | Step 6, acceptance review |
| S6 | Review packet references missing paths | Step 5 |
| S7 | Acceptance pre-check `[--]` | Step 7 |
| S8 | Handoff index `[--]` | Step 8 |
| S9 | Freeze manifest contradicted | Steps 2 and 5 |
