# PostCAD Pilot Handoff Checklist

Practical transfer checklist for a new operator or reviewer. Each item is inspectable against the actual repo.

---

## Section 1 — Repo State

- [ ] **1.1** Repo is cloned and on branch `main`: `git branch --show-current` outputs `main`
- [ ] **1.2** Working tree is clean: `git status` shows `nothing to commit`
- [ ] **1.3** Recent commit history is visible: `git log --oneline -5` shows recognizable commit messages
- [ ] **1.4** `Cargo.toml` is present at the repo root and contains a `[workspace]` declaration
- [ ] **1.5** Full test suite passes from a clean state: `cargo test --workspace` exits 0 with 0 failures

---

## Section 2 — Release Folder Structure

- [ ] **2.1** `release/README.md` exists — operator runbook
- [ ] **2.2** `release/start_pilot.sh` exists and is executable
- [ ] **2.3** `release/reset_pilot_data.sh` exists and is executable
- [ ] **2.4** `release/smoke_test.sh` exists and is executable
- [ ] **2.5** `release/generate_evidence_bundle.sh` exists and is executable
- [ ] **2.6** `release/review/` exists with all five documents:
  - `README.md`, `SYSTEM_OVERVIEW.md`, `OPERATOR_FLOW.md`, `ARTIFACT_GUIDE.md`, `BOUNDARIES.md`
- [ ] **2.7** `release/acceptance/` exists with:
  - `README.md`, `PILOT_ACCEPTANCE_CHECKLIST.md`, `REVIEW_WORKSHEET.md`, `print_acceptance_summary.sh`
- [ ] **2.8** `release/handoff/` exists with:
  - `README.md`, `HANDOFF_CHECKLIST.md`, `FIRST_HOUR_GUIDE.md`, `KNOWN_GOOD_STATE.md`
- [ ] **2.9** `demo/run_demo.sh` exists and is executable

---

## Section 3 — Canonical Fixtures

- [ ] **3.1** `examples/pilot/case.json` exists and contains `"case_id": "f1000001-0000-0000-0000-000000000001"`
- [ ] **3.2** `examples/pilot/registry_snapshot.json` exists and is valid JSON
- [ ] **3.3** `examples/pilot/config.json` exists and contains `"jurisdiction": "DE"`
- [ ] **3.4** None of the above fixtures show as modified: `git diff examples/pilot/` is empty

---

## Section 4 — Evidence Folder

Either the evidence folder exists from a previous run, or you know how to generate it.

- [ ] **4.1a** `release/evidence/current/` exists (evidence from a previous run), **OR**
- [ ] **4.1b** You can generate it by starting the service and running `./release/generate_evidence_bundle.sh`
- [ ] **4.2** If `current/` exists: `release/evidence/current/summary.txt` ends with `All 7 steps passed.`
- [ ] **4.3** If `current/` exists: `release/evidence/current/04_receipt.json` contains `"receipt_hash": "0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb"`
- [ ] **4.4** `release/evidence/current/` is listed in `.gitignore` (runtime artifacts, not committed)

---

## Section 5 — Operator Sequence

Confirm you understand the sequence before running anything.

- [ ] **5.1** You have read `release/handoff/FIRST_HOUR_GUIDE.md`
- [ ] **5.2** You have read `release/README.md` (operator runbook)
- [ ] **5.3** You know the 5-step sequence: reset → start → smoke test → demo → generate evidence
- [ ] **5.4** You know that Terminal A runs the service and Terminal B runs scripts against it
- [ ] **5.5** You know that `reset_pilot_data.sh` removes only `data/` subdirectories and nothing else
- [ ] **5.6** You know that the smoke test expects the service to already be running on `localhost:8080`

---

## Section 6 — Frozen vs Editable

- [ ] **6.1** You have read `release/review/BOUNDARIES.md` and understand what is frozen
- [ ] **6.2** You understand that `examples/pilot/` fixtures are frozen and must not be modified
- [ ] **6.3** You understand that `tests/protocol_vectors/` is frozen
- [ ] **6.4** You understand that `data/` runtime directories are ephemeral and safe to reset
- [ ] **6.5** You understand that `release/evidence/current/` is gitignored and regenerated per run
- [ ] **6.6** You understand that the protocol version (`postcad-v1`) and kernel version (`postcad-routing-v1`) are frozen

---

## Section 7 — Where to Look on Failure

- [ ] **7.1** Smoke test fails step 1 (`/health` unreachable): service is not running — start it with `./release/start_pilot.sh`
- [ ] **7.2** Smoke test fails mid-flow: run `./release/reset_pilot_data.sh` then restart from Step 1
- [ ] **7.3** Evidence bundle shows wrong `receipt_hash`: canonical fixtures may have been modified — run `git diff examples/pilot/`
- [ ] **7.4** `cargo test --workspace` fails: check for uncommitted changes — run `git status` and `git diff`
- [ ] **7.5** You know the acceptance pre-check script: `./release/acceptance/print_acceptance_summary.sh`
- [ ] **7.6** You know the full acceptance checklist: `release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md`

---

## Handoff complete when

All sections above pass. The new operator/reviewer can independently run the full local pilot flow and verify the results.
