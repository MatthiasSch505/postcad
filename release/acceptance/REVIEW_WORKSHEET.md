# PostCAD Pilot — Review Worksheet

Human-fillable worksheet for a local pilot review. Work through `PILOT_ACCEPTANCE_CHECKLIST.md` and record findings here.

---

## Review context

| Field | Value |
|---|---|
| Review date | _____________ |
| Reviewer | _____________ |
| Commit reviewed (`git_head.txt`) | _____________ |
| Evidence bundle location | `release/evidence/current/` |
| Review packet location | `release/review/` |

---

## Section 1 — Environment and Setup

| # | Item | Result | Notes |
|---|---|---|---|
| 1.1 | `cargo` available | PASS / FAIL | |
| 1.2 | `python3` available | PASS / FAIL | |
| 1.3 | `curl` available | PASS / FAIL | |
| 1.4 | `Cargo.toml` has `[workspace]` | PASS / FAIL | |
| 1.5 | `cargo test --workspace` passes | PASS / FAIL | |
| 1.6 | Service binary present or built | PASS / FAIL | |

Section 1 result: **PASS / FAIL**

---

## Section 2 — Operator Flow

| # | Item | Result | Notes |
|---|---|---|---|
| 2.1 | `reset_pilot_data.sh` exists with strict mode | PASS / FAIL | |
| 2.2 | `start_pilot.sh` exists with strict mode | PASS / FAIL | |
| 2.3 | `smoke_test.sh` exists with strict mode | PASS / FAIL | |
| 2.4 | `generate_evidence_bundle.sh` exists with strict mode | PASS / FAIL | |
| 2.5 | All four scripts are executable | PASS / FAIL | |
| 2.6 | `demo/run_demo.sh` exists and is executable | PASS / FAIL | |
| 2.7 | Canonical fixtures present in `examples/pilot/` | PASS / FAIL | |
| 2.8 | `release/README.md` documents all five steps | PASS / FAIL | |

Section 2 result: **PASS / FAIL**

---

## Section 3 — API Response Acceptance

| # | Item | Result | Notes |
|---|---|---|---|
| 3.1 | `01_health.json` — `status: ok` | PASS / FAIL | |
| 3.2 | `02_store_case.json` — correct `case_id` | PASS / FAIL | |
| 3.3 | `03_route_case.json` — correct `receipt_hash` and `selected_candidate_id` | PASS / FAIL | |
| 3.4 | `04_receipt.json` — all expected fields present and correct | PASS / FAIL | |
| 3.5 | `05_dispatch.json` — dispatched or idempotent note | PASS / FAIL | |
| 3.6 | `06_verify.json` — `result: VERIFIED` | PASS / FAIL | |
| 3.7 | `07_route_history.json` — ≥1 route entry | PASS / FAIL | |
| 3.8 | `receipt_hash` consistent across files 03–06 | PASS / FAIL | |

Section 3 result: **PASS / FAIL**

---

## Section 4 — Artifact Persistence Acceptance

| # | Item | Result | Notes |
|---|---|---|---|
| 4.1 | Case artifact present in `data_artifacts/cases/` | PASS / FAIL | |
| 4.2 | Receipt artifact present in `data_artifacts/receipts/` | PASS / FAIL | |
| 4.3 | Policy artifact present in `data_artifacts/policies/` | PASS / FAIL | |
| 4.4 | Dispatch artifact present in `data_artifacts/dispatch/` | PASS / FAIL | |
| 4.5 | Verification artifact present in `data_artifacts/verification/` | PASS / FAIL | |
| 4.6 | Stored receipt matches `receipt_hash` in `04_receipt.json` | PASS / FAIL | |

Section 4 result: **PASS / FAIL**

---

## Section 5 — Evidence Bundle Acceptance

| # | Item | Result | Notes |
|---|---|---|---|
| 5.1 | `release/evidence/current/` exists and non-empty | PASS / FAIL | |
| 5.2 | `summary.txt` ends with `All 7 steps passed.` | PASS / FAIL | |
| 5.3 | `commands.txt` lists all 7 HTTP calls | PASS / FAIL | |
| 5.4 | `git_head.txt` contains a 40-char hex hash | PASS / FAIL | |
| 5.5 | `inputs/case.json` — correct `case_id` | PASS / FAIL | |
| 5.6 | `inputs/registry_snapshot.json` — valid JSON | PASS / FAIL | |
| 5.7 | `inputs/config.json` — `jurisdiction: DE` | PASS / FAIL | |
| 5.8 | All seven numbered files `01`–`07` present | PASS / FAIL | |
| 5.9 | `release/evidence/current/` in `.gitignore` | PASS / FAIL | |

Section 5 result: **PASS / FAIL**

---

## Section 6 — Review Packet Consistency

| # | Item | Result | Notes |
|---|---|---|---|
| 6.1 | `release/review/README.md` exists | PASS / FAIL | |
| 6.2 | `release/review/SYSTEM_OVERVIEW.md` exists | PASS / FAIL | |
| 6.3 | `release/review/OPERATOR_FLOW.md` exists | PASS / FAIL | |
| 6.4 | `release/review/ARTIFACT_GUIDE.md` exists | PASS / FAIL | |
| 6.5 | `release/review/BOUNDARIES.md` exists | PASS / FAIL | |
| 6.6 | `OPERATOR_FLOW.md` matches smoke test's 7-endpoint flow | PASS / FAIL | |
| 6.7 | `ARTIFACT_GUIDE.md` states correct `receipt_hash` value | PASS / FAIL | |
| 6.8 | `BOUNDARIES.md` lists all seven pilot endpoints as frozen | PASS / FAIL | |
| 6.9 | No review doc references non-existent paths or endpoints | PASS / FAIL | |

Section 6 result: **PASS / FAIL**

---

## Section 7 — Frozen Boundary Acceptance

| # | Item | Result | Notes |
|---|---|---|---|
| 7.1 | No protocol/routing/schema/kernel changes were required | PASS / FAIL | |
| 7.2 | `examples/pilot/case.json` unchanged (`git diff` clean) | PASS / FAIL | |
| 7.3 | `examples/pilot/registry_snapshot.json` unchanged | PASS / FAIL | |
| 7.4 | `examples/pilot/config.json` unchanged | PASS / FAIL | |
| 7.5 | Receipt shows `routing_kernel_version: postcad-routing-v1` | PASS / FAIL | |
| 7.6 | Receipt shows `schema_version: 1` | PASS / FAIL | |
| 7.7 | Canonical `receipt_hash` matches value in `ARTIFACT_GUIDE.md` | PASS / FAIL | |

Section 7 result: **PASS / FAIL**

---

## Notes

_Record any deviations, observations, or items requiring follow-up:_

```
[notes here]
```

---

## Final outcome

Mark one:

- [ ] **ACCEPTED** — all 7 sections pass, no deviations
- [ ] **ACCEPTED WITH NOTES** — all mandatory items pass; notes recorded above
- [ ] **NOT ACCEPTED** — one or more items failed; see notes above

Reviewer signature / confirmation: _____________

Date: _____________
