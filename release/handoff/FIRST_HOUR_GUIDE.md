# PostCAD Pilot — First Hour Guide

Exact sequence for a new operator or reviewer in the first hour. Follow in order. Stop and investigate if any step deviates from the expected result described.

---

## Step 1 — Confirm repo state (5 min)

```bash
git branch --show-current   # should output: main
git status                  # should output: nothing to commit, working tree clean
git log --oneline -5        # review recent commit history
```

Expected: branch is `main`, working tree is clean.

If `git status` shows unexpected changes, do not proceed until you understand what they are.

---

## Step 2 — Read the handoff entry point (5 min)

Read `release/handoff/README.md` — you are doing this now.

Then read `release/handoff/KNOWN_GOOD_STATE.md` to understand what state you should be in.

---

## Step 3 — Read the operator runbook (5 min)

```bash
cat release/README.md
```

This documents the full 5-step operator sequence, prerequisites, common problems, and what data directories look like. Read all sections before running anything.

---

## Step 4 — Read the review packet entry point (5 min)

```bash
cat release/review/README.md
```

Then skim `release/review/SYSTEM_OVERVIEW.md` for a description of each system layer, and `release/review/BOUNDARIES.md` for what is frozen.

Do not change anything in `release/review/` — it is documentation only.

---

## Step 5 — Inspect the evidence bundle (5 min)

Check whether evidence from a previous run exists:

```bash
ls release/evidence/current/ 2>/dev/null && echo "evidence present" || echo "no evidence yet"
```

If evidence exists, read the summary:

```bash
cat release/evidence/current/summary.txt
```

Expected last line: `All 7 steps passed.`

If evidence does not exist, you will generate it in Step 8.

---

## Step 6 — Run the acceptance pre-check (5 min)

```bash
./release/acceptance/print_acceptance_summary.sh
```

This prints a structural pre-check of all expected acceptance inputs. Every line should show `[OK]`. If any line shows `[--] MISSING`, note it before proceeding.

This script is read-only and does not modify anything.

---

## Step 7 — Run the full test suite (5 min)

```bash
cargo test --workspace
```

Expected: all test suites pass with 0 failures. This does not require a running service.

If any tests fail, do not proceed to the live run until you understand why.

---

## Step 8 — Optionally run the local pilot flow (30 min, two terminals)

If you want to verify the live service path yourself:

**Terminal A — start the service:**

```bash
./release/reset_pilot_data.sh
./release/start_pilot.sh
```

Leave Terminal A running.

**Terminal B — smoke test and evidence:**

```bash
./release/smoke_test.sh
```

Expected final line:
```
  SMOKE TEST PASSED — all 7 steps OK
```

Then generate the evidence bundle:

```bash
./release/generate_evidence_bundle.sh
```

Expected final output:
```
  EVIDENCE BUNDLE COMPLETE
  Output: .../release/evidence/current
```

Then verify:
```bash
cat release/evidence/current/summary.txt
```

Stop Terminal A with `Ctrl-C` when done.

---

## What NOT to change

During your first hour, do not modify:

- `examples/pilot/` — canonical frozen fixtures
- `tests/protocol_vectors/` — frozen test vectors
- Any file under `crates/` — kernel and service implementation
- Any file under `release/review/` — review packet documentation
- `release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md` — acceptance criteria

Safe to generate or regenerate:

- `release/evidence/current/` — runtime output, gitignored, safe to regenerate at any time
- `data/` directories — runtime state, gitignored, safe to reset at any time

---

## Where to stop if something is wrong

| Symptom | Stop and investigate |
|---|---|
| `git status` shows unexpected changes | Before Step 3 |
| `cargo test --workspace` fails | Before Step 8 |
| Acceptance pre-check shows `[--] MISSING` items | Before Step 8 |
| Smoke test fails a phase | Reset (`./release/reset_pilot_data.sh`) and retry once; if it fails again, investigate |
| `receipt_hash` in evidence does not match `0db54077cff0fbc4…` | Run `git diff examples/pilot/` to check for fixture drift |
| `result` in `06_verify.json` is not `VERIFIED` | Run `git diff` broadly; the receipt or policy may have been altered |
