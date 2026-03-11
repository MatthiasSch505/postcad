# PostCAD Pilot Walkthrough

Single-file narrative of the exact local pilot path, in order. All commands are repo-relative and run from the repo root. Two terminals are required for steps 3–6.

---

## Step 1 — Inspect repo state

**What it does:** confirms you are on the right branch with a clean working tree before touching anything.

```bash
git branch --show-current    # expected: main
git status                   # expected: nothing to commit, working tree clean
git log --oneline -3         # review recent commit history
cargo test --workspace       # expected: all suites pass, 0 failures
```

**Expected output:**
- Branch is `main`
- `git status` shows no modifications to tracked files
- `cargo test --workspace` exits 0

**If something is wrong:** do not proceed. Run `git diff` to identify unexpected changes. If `cargo test` fails, investigate before running the live flow.

**What not to change:** nothing — this step is inspection only.

---

## Step 2 — Reset pilot data

**What it does:** removes all runtime data directories so the pilot starts from a known empty state.

```bash
./release/reset_pilot_data.sh
```

**Expected output:**
```
[reset_pilot_data] REMOVED  .../data/cases  (N file(s))
[reset_pilot_data] REMOVED  .../data/receipts  (N file(s))
...
[reset_pilot_data] Done. N director(ies) removed, N skipped.
```

On a first run, all directories will be skipped (they do not exist yet). That is normal.

**Touches:** `data/cases/`, `data/receipts/`, `data/policies/`, `data/dispatch/`, `data/verification/`

**Does NOT touch:** source code, compiled binaries, canonical fixtures, or anything under `release/`, `examples/`, or `tests/`.

**If it fails:** check whether another process has files open in `data/`.

---

## Step 3 — Start the service (Terminal A)

**What it does:** builds `postcad-service` if needed, then starts the HTTP service in the foreground on `http://localhost:8080`.

```bash
./release/start_pilot.sh
```

**Expected output:**
```
══════════════════════════════════════════
  PostCAD Pilot — Local Service Startup
══════════════════════════════════════════
  Repo root : /path/to/postcad
  Base URL  : http://localhost:8080
  ...
[start_pilot] Service starting — press Ctrl-C to stop.
postcad-service listening on 0.0.0.0:8080
```

**Leave Terminal A running.** Open Terminal B for the next steps.

**Overrides:** `POSTCAD_ADDR=host:port` to change address, `POSTCAD_DATA=/path` to change data location.

**If it fails:**
- `Binary not found — building...` is normal on first run; wait for the build to finish.
- Port conflict: use `POSTCAD_ADDR=localhost:9090 ./release/start_pilot.sh`.

**What not to change:** do not modify service source, kernel code, or canonical fixtures while the service is running.

---

## Step 4 — Run the smoke test (Terminal B)

**What it does:** runs a 7-step deterministic flow against the live service using the canonical pilot fixture.

```bash
./release/smoke_test.sh
```

**Steps executed:**

| # | Call | Expected HTTP | Expected key field |
|---|---|---|---|
| 1 | `GET /health` | 200 | `"status": "ok"` |
| 2 | `POST /cases` | 201 (first) / 200 (re-run) | `"case_id": "f1000001-0000-0000-0000-000000000001"` |
| 3 | `POST /cases/:id/route` | 200 | `"selected_candidate_id": "pilot-de-001"` |
| 4 | `GET /receipts/:hash` | 200 | `"outcome": "routed"` |
| 5 | `POST /dispatch/:hash` | 200 (first) / 409 (re-run) | `"dispatched": true` |
| 6 | `POST /dispatch/:hash/verify` | 200 | `"result": "VERIFIED"` |
| 7 | `GET /routes` | 200 | ≥1 entry in `"routes"` |

**Expected final output:**
```
════════════════════════════════════════
  SMOKE TEST PASSED — all 7 steps OK
════════════════════════════════════════
```

**Inspect:** the smoke test prints each step's response inline.

**If it fails:** `[FAIL] Phase N: ...` identifies the failing step.
- Step 1 fails: service is not running — check Terminal A.
- Mid-flow failure: `./release/reset_pilot_data.sh` then retry from Step 2.

---

## Step 5 — Run the demo (Terminal B)

**What it does:** a self-contained 8-step demo that starts its own service instance, runs the full flow, and stops the service. Does not require the service from Step 3.

```bash
./demo/run_demo.sh
```

**Expected final output:**
```
════════════════════════════════════════
  DEMO COMPLETE — all 8 steps passed
════════════════════════════════════════
```

**Note:** this uses a temporary data directory (`mktemp -d`) and does not affect the `data/` directories used by Step 3.

**If it fails:** port 8080 may be in use — check Terminal A (stop it first if needed, then re-run the demo).

---

## Step 6 — Generate evidence bundle (Terminal B, service from Step 3 still running)

**What it does:** re-runs the 7-step pilot flow and captures all API responses, input fixtures, and local data artifacts into `release/evidence/current/`.

```bash
./release/generate_evidence_bundle.sh
```

**Expected final output:**
```
══════════════════════════════════════════
  EVIDENCE BUNDLE COMPLETE
  Output: .../release/evidence/current
══════════════════════════════════════════
```

**Inspect:**
```bash
cat release/evidence/current/summary.txt
```
Expected last line: `All 7 steps passed.`

**Key output files:**

| File | Contents |
|---|---|
| `01_health.json` – `07_route_history.json` | API responses from each step |
| `04_receipt.json` | Full routing receipt with all hash commitments |
| `06_verify.json` | Verification result — must contain `"result": "VERIFIED"` |
| `inputs/` | Copied canonical input fixtures |
| `data_artifacts/` | Files written by the service under `data/` |
| `summary.txt` | Human-readable pass/fail confirmation |
| `git_head.txt` | Commit hash at time of capture |

The `receipt_hash` is deterministic: `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb` for the canonical pilot inputs, every run.

**If it fails:** `[FAIL] Step NN ...` identifies the failing call. Ensure the service from Step 3 is still running.

You can now stop Terminal A with `Ctrl-C`.

---

## Step 7 — Inspect the review packet

**What it does:** the review packet describes the system, operator flow, artifacts, and frozen boundaries for an external reviewer. It is documentation only — nothing to run.

```bash
ls release/review/
cat release/review/README.md
```

**Files:**

| File | Covers |
|---|---|
| `SYSTEM_OVERVIEW.md` | Four-layer architecture, current pilot scope |
| `OPERATOR_FLOW.md` | Same sequence as this walkthrough, with more detail |
| `ARTIFACT_GUIDE.md` | Every file in `evidence/current/`, what each field means |
| `BOUNDARIES.md` | What is frozen, what is not claimed, what is out of scope |

**What not to change:** review packet files are documentation; they must stay consistent with the scripts and artifacts they describe.

---

## Step 8 — Inspect the acceptance bundle

**What it does:** the acceptance bundle provides a checklist and worksheet for determining whether the pilot run meets the acceptance standard.

```bash
./release/acceptance/print_acceptance_summary.sh
```

Expected: every line shows `[OK]`.

Then open:
- `release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md` — 33-item checklist across 7 sections
- `release/acceptance/REVIEW_WORKSHEET.md` — fill in for a formal review record

**What not to change:** acceptance criteria; the checklist reflects the current frozen pilot scope.

---

## Step 9 — Inspect the handoff packet

**What it does:** the handoff packet orients a new operator or reviewer without requiring them to read all packets first.

```bash
./release/handoff/print_handoff_index.sh
```

Expected: all resources indexed as `[OK]`.

Then read:
- `release/handoff/FIRST_HOUR_GUIDE.md` — exact first-hour sequence
- `release/handoff/KNOWN_GOOD_STATE.md` — how to verify the expected current state
- `release/handoff/HANDOFF_CHECKLIST.md` — practical transfer checklist

---

## Full sequence at a glance

```
# Terminal A
./release/start_pilot.sh           # Step 3: start service (leave running)

# Terminal B
git status && cargo test --workspace   # Step 1: inspect state
./release/reset_pilot_data.sh          # Step 2: clean slate
./release/smoke_test.sh                # Step 4: 7-step smoke test
./demo/run_demo.sh                     # Step 5: self-contained demo
./release/generate_evidence_bundle.sh  # Step 6: capture evidence
cat release/evidence/current/summary.txt  # Step 6: confirm pass

# Inspection (no service required)
./release/acceptance/print_acceptance_summary.sh  # Step 8
./release/handoff/print_handoff_index.sh          # Step 9
```

---

## Key output locations

| Artifact | Location |
|---|---|
| Runtime data | `data/cases/`, `data/receipts/`, `data/policies/`, `data/dispatch/`, `data/verification/` |
| Evidence bundle | `release/evidence/current/` |
| Evidence summary | `release/evidence/current/summary.txt` |
| Routing receipt | `release/evidence/current/04_receipt.json` |
| Verification result | `release/evidence/current/06_verify.json` |
| Acceptance pre-check | `release/acceptance/print_acceptance_summary.sh` |
| Handoff index | `release/handoff/print_handoff_index.sh` |
