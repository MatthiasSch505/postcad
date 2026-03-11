# PostCAD Pilot — External Review Path

Smallest useful inspection path for an external reviewer. Follow steps in order. Each step answers one question and points to the next.

---

## Step 1 — Release notes

**Path:** `release/RELEASE_NOTES_PILOT.md`
**Type:** read-only

**What this answers:**
- What surfaces are included in this pilot release?
- What are the frozen protocol values?
- What is out of scope?

**What "good" looks like:**
- Protocol version `postcad-v1`, routing kernel `postcad-routing-v1`, schema `1` are listed.
- Deterministic receipt hash is documented.
- Out-of-scope section is present and clear.

**Where to go next:** Step 2.

---

## Step 2 — System overview

**Path:** `release/review/SYSTEM_OVERVIEW.md`
**Type:** read-only

**What this answers:**
- What does the system do?
- What are its four layers?
- What does the current pilot scope cover?

**What "good" looks like:**
- Four-layer architecture described: kernel, service, release/operator, evidence/review.
- Pilot scope is stated: one canonical case, local operation only.

**Where to go next:** Step 3.

---

## Step 3 — Structural self-check

**Path/command:** `./release/selfcheck/run_release_selfcheck.sh`
**Type:** executable (read-only, no modifications)

**What this answers:**
- Are all expected release files present?
- Are operator scripts executable?
- Are canonical fixtures in place?

**What "good" looks like:**
```
Structural self-check completed.
Missing items: 0 — package structure intact.
```

**Where to go next:** Step 4.
**When not to proceed:** Any `[--]` for a non-evidence resource.

---

## Step 4 — Artifact guide

**Path:** `release/review/ARTIFACT_GUIDE.md`
**Type:** read-only

**What this answers:**
- What does each file in the evidence bundle contain?
- What does each field in the routing receipt mean?
- What should the reviewer inspect first?

**What "good" looks like:**
- Every file in `evidence/current/` is listed with a description.
- Key fields (`receipt_hash`, `result`, `outcome`) are explained.

**Where to go next:** Step 5.

---

## Step 5 — Evidence bundle

**Path:** `release/evidence/current/summary.txt`, `06_verify.json`, `04_receipt.json`
**Type:** runtime-generated (inspect after generating; see `release/evidence/README.md`)

**What this answers:**
- Did the pilot run complete successfully?
- Is the verification result `VERIFIED`?
- Does the receipt hash match the expected deterministic value?

**What "good" looks like:**
- `summary.txt` ends with `All 7 steps passed.`
- `06_verify.json` contains `"result": "VERIFIED"`
- `04_receipt.json` contains `receipt_hash: 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb`

**Where to go next:** Step 6.
**If not yet generated:** `./release/start_pilot.sh` (Terminal A), then `./release/generate_evidence_bundle.sh` (Terminal B).

---

## Step 6 — Acceptance checklist

**Path:** `release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md`
**Type:** read-only

**What this answers:**
- What are the formal acceptance criteria for this pilot?
- How many items must pass, and across which sections?

**What "good" looks like:**
- 33 items across 7 sections: environment, operator flow, API responses, artifact persistence, evidence bundle, review packet consistency, frozen boundaries.
- All items are satisfiable given the evidence from Step 5.

**Also run:** `./release/acceptance/print_acceptance_summary.sh` — structural pre-check of acceptance inputs.

**Where to go next:** Step 7.

---

## Step 7 — Frozen boundaries

**Path:** `release/review/BOUNDARIES.md`
**Type:** read-only

**What this answers:**
- What is frozen and must not change?
- What is intentionally out of scope for this pilot?
- What does this package not claim?

**What "good" looks like:**
- Protocol, kernel, schema, hashing, and endpoint surface are all listed as frozen.
- Out-of-scope items are clearly stated.

**Where to go next:** Step 8 (if handoff is required), or end here.

---

## Step 8 — Handoff packet (if transferring to another operator)

**Path:** `release/handoff/README.md`, `release/handoff/FIRST_HOUR_GUIDE.md`
**Type:** read-only

**What this answers:**
- Is the package complete enough to hand to a new operator?
- What does a new operator need to do in the first hour?

**Also run:** `./release/handoff/print_handoff_index.sh` — confirms all resources are present.

---

## Full sequence at a glance

```bash
# Read
cat release/RELEASE_NOTES_PILOT.md
cat release/review/SYSTEM_OVERVIEW.md

# Check
./release/selfcheck/run_release_selfcheck.sh

# Read
cat release/review/ARTIFACT_GUIDE.md

# Inspect evidence (must be generated first)
cat release/evidence/current/summary.txt
cat release/evidence/current/06_verify.json

# Accept
cat release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md
./release/acceptance/print_acceptance_summary.sh

# Boundaries
cat release/review/BOUNDARIES.md
```
