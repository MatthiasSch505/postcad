# PostCAD Pilot Review Trace

Single deterministic review path through the local pilot package. Follow steps in order. Do not skip steps. Stop at any step that shows a mismatch or missing item — see `STOP_POINTS.md` for resolution guidance.

---

## Step 1 — Inspect the release index

**Path/command:**
```bash
cat release/INDEX.md
```

**What this answers:**
- What surfaces make up the local pilot package?
- Which files are executable vs read-only?
- What is the recommended operator and reviewer path?

**What "good" looks like:**
- `INDEX.md` is present and lists all major surfaces: operator scripts, evidence bundle, walkthrough, review, acceptance, handoff, selfcheck, freeze.
- Every path listed resolves when you check it manually or run the self-check.

**Where to go next:** Step 2.

**When not to proceed:** If `INDEX.md` is missing or references paths that do not exist. Run `release/selfcheck/run_release_selfcheck.sh` to identify what is missing before continuing.

---

## Step 2 — Inspect the freeze manifest

**Path/command:**
```bash
cat release/FREEZE_MANIFEST.md
```

**What this answers:**
- What is frozen in the current pilot?
- Which files are read-only, executable, or runtime-generated?
- What are the frozen protocol values (version, kernel, hash)?

**What "good" looks like:**
- Protocol version: `postcad-v1`
- Routing kernel: `postcad-routing-v1`
- Receipt schema version: `1`
- Deterministic receipt hash documented: `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb`
- `release/evidence/current/` is listed as runtime-generated (not frozen).

**Where to go next:** Step 3.

**When not to proceed:** If the freeze manifest is missing or contradicts the actual repo state (e.g., lists files that do not exist as frozen).

---

## Step 3 — Run the structural self-check

**Path/command:**
```bash
./release/selfcheck/run_release_selfcheck.sh
```

**What this answers:**
- Are all expected release files present?
- Are all operator scripts executable?
- Are the canonical fixtures in place?

**What "good" looks like:**
```
Structural self-check completed.
Missing items: 0 — package structure intact.
```
Every line shows `[OK]`. No `[--]` lines except the informational evidence note if `evidence/current/` has not been generated yet.

**Where to go next:** Step 4.

**When not to proceed:** Any `[--]` line for a non-evidence resource. Investigate and resolve before continuing.

---

## Step 4 — Inspect the walkthrough

**Path/command:**
```bash
cat release/walkthrough/PILOT_WALKTHROUGH.md
```

Or for a quicker orientation:
```bash
./release/walkthrough/print_walkthrough.sh
```

**What this answers:**
- What is the exact operator sequence for the local pilot?
- What are the expected outputs at each step?
- What are the failure modes?

**What "good" looks like:**
- 9 steps are described in order: inspect repo state, reset data, start service, smoke test, demo, generate evidence, inspect review, inspect acceptance, inspect handoff.
- Each step has an expected output and a "when not to proceed" note.
- Commands match the actual scripts in `release/`.

**Where to go next:** Step 5.

**When not to proceed:** If the walkthrough references commands or files that do not exist (caught by Step 3).

---

## Step 5 — Inspect the review packet

**Path/command:**
```bash
cat release/review/README.md
cat release/review/SYSTEM_OVERVIEW.md
cat release/review/OPERATOR_FLOW.md
cat release/review/ARTIFACT_GUIDE.md
cat release/review/BOUNDARIES.md
```

**What this answers:**
- What does the system do and what does the current pilot scope cover?
- How did the operator run the pilot?
- What does each artifact in the evidence bundle contain?
- What is intentionally out of scope?

**What "good" looks like:**
- `SYSTEM_OVERVIEW.md` describes a four-layer architecture (kernel, service, release/operator, evidence/review).
- `OPERATOR_FLOW.md` matches the walkthrough sequence.
- `ARTIFACT_GUIDE.md` lists every file in `evidence/current/` with field explanations.
- `BOUNDARIES.md` states explicitly what is frozen and what is not claimed.

**Where to go next:** Step 6.

**When not to proceed:** If the review packet references paths or outputs that contradict the actual evidence (found in Step 6).

---

## Step 6 — Inspect the evidence bundle

**Path/command:**
```bash
cat release/evidence/current/summary.txt
cat release/evidence/current/06_verify.json
cat release/evidence/current/04_receipt.json
```

**What this answers:**
- Did the pilot run complete successfully?
- Is the verification result `VERIFIED`?
- Does the receipt hash match the expected deterministic value?

**What "good" looks like:**

`summary.txt` ends with:
```
outcome   = routed   (expected: routed)
result    = VERIFIED (expected: VERIFIED)
routes    = 1 route(s) in history

All 7 steps passed.
```

`06_verify.json` contains:
```json
{"receipt_hash": "...", "result": "VERIFIED"}
```

`04_receipt.json` contains `receipt_hash` field equal to:
`0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb`

**Where to go next:** Step 7.

**When not to proceed:**
- `summary.txt` does not end with `All 7 steps passed.`
- `06_verify.json` does not contain `"result": "VERIFIED"`
- `receipt_hash` does not match the expected deterministic value
- `evidence/current/` is absent — generate it with `./release/generate_evidence_bundle.sh` (requires service running)

---

## Step 7 — Inspect the acceptance bundle

**Path/command:**
```bash
./release/acceptance/print_acceptance_summary.sh
cat release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md
```

**What this answers:**
- Does the pilot run meet the stated acceptance criteria?
- Are all 33 checklist items satisfiable?

**What "good" looks like:**
- `print_acceptance_summary.sh` shows all `[OK]` lines.
- `PILOT_ACCEPTANCE_CHECKLIST.md` covers 7 sections: environment, operator flow, API responses, artifact persistence, evidence bundle, review packet consistency, frozen boundaries.
- A reviewer can work through the checklist with the evidence from Step 6 and mark all items as passing.

**Where to go next:** Step 8.

**When not to proceed:** If `print_acceptance_summary.sh` shows `[--]` lines for structural inputs, or if the evidence from Step 6 does not support the checklist items.

---

## Step 8 — Inspect the handoff packet

**Path/command:**
```bash
./release/handoff/print_handoff_index.sh
cat release/handoff/FIRST_HOUR_GUIDE.md
cat release/handoff/KNOWN_GOOD_STATE.md
```

**What this answers:**
- Is the handoff packet complete?
- Does `KNOWN_GOOD_STATE.md` match the current repo and evidence state?
- Is the package ready to hand to a new operator or external reviewer?

**What "good" looks like:**
- `print_handoff_index.sh` shows all `[OK]` lines.
- `KNOWN_GOOD_STATE.md` expected state matches what you observed in Steps 3 and 6.
- `FIRST_HOUR_GUIDE.md` commands match the actual scripts.

**When to stop (package ready):**
All 8 steps completed with no unresolved `[--]` items and no contradictions between surfaces. The package is ready for external review.

**When not to declare ready:**
- Any step above produced an unresolved mismatch.
- `evidence/current/` was not generated or does not pass.
- Any checklist item from Step 7 cannot be satisfied.

---

## Full sequence at a glance

```bash
# Read
cat release/INDEX.md
cat release/FREEZE_MANIFEST.md

# Check
./release/selfcheck/run_release_selfcheck.sh

# Read
cat release/walkthrough/PILOT_WALKTHROUGH.md
cat release/review/README.md
cat release/review/SYSTEM_OVERVIEW.md
cat release/review/BOUNDARIES.md

# Inspect evidence (must be generated first)
cat release/evidence/current/summary.txt
cat release/evidence/current/06_verify.json

# Accept
./release/acceptance/print_acceptance_summary.sh
cat release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md

# Handoff
./release/handoff/print_handoff_index.sh
cat release/handoff/KNOWN_GOOD_STATE.md
```
