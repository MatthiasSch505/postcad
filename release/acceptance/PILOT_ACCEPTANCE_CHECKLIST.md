# PostCAD Pilot Acceptance Checklist

Definitive checklist for a successful local pilot run. Each item is answerable pass/fail by inspecting existing files and artifacts. No item requires running the service.

---

## Section 1 — Environment and Setup

- [ ] **1.1** Rust toolchain (`cargo`) is available: `cargo --version` returns a version string
- [ ] **1.2** `python3` is available on PATH: `python3 --version` returns a version string
- [ ] **1.3** `curl` is available on PATH: `curl --version` returns a version string
- [ ] **1.4** The repo root contains `Cargo.toml` with a `[workspace]` declaration
- [ ] **1.5** `cargo test --workspace` passes with 0 failures (no service required)
- [ ] **1.6** The service binary exists at `target/debug/postcad-service` (or was built by `start_pilot.sh`)

---

## Section 2 — Operator Flow

- [ ] **2.1** `release/reset_pilot_data.sh` exists and contains `set -euo pipefail`
- [ ] **2.2** `release/start_pilot.sh` exists and contains `set -euo pipefail`
- [ ] **2.3** `release/smoke_test.sh` exists and contains `set -euo pipefail`
- [ ] **2.4** `release/generate_evidence_bundle.sh` exists and contains `set -euo pipefail`
- [ ] **2.5** All four scripts are executable (`ls -l release/*.sh` shows `-rwx` permissions)
- [ ] **2.6** `demo/run_demo.sh` exists and is executable
- [ ] **2.7** Canonical input fixtures are present:
  - `examples/pilot/case.json`
  - `examples/pilot/registry_snapshot.json`
  - `examples/pilot/config.json`
- [ ] **2.8** `release/README.md` documents all five operator steps (reset, start, smoke test, demo, evidence)

---

## Section 3 — API Response Acceptance

Inspect `release/evidence/current/` for each item.

- [ ] **3.1** `01_health.json` exists and contains `{"status": "ok"}`
- [ ] **3.2** `02_store_case.json` exists and contains `"case_id": "f1000001-0000-0000-0000-000000000001"`
- [ ] **3.3** `03_route_case.json` exists and contains:
  - `"receipt_hash": "0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb"`
  - `"selected_candidate_id": "pilot-de-001"`
- [ ] **3.4** `04_receipt.json` exists and contains:
  - `"outcome": "routed"`
  - `"selected_candidate_id": "pilot-de-001"`
  - `"receipt_hash": "0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb"`
  - `"schema_version": "1"`
  - `"routing_kernel_version": "postcad-routing-v1"`
  - `"audit_seq": 0`
  - `"audit_previous_hash": "0000000000000000000000000000000000000000000000000000000000000000"`
- [ ] **3.5** `05_dispatch.json` exists and contains either `"dispatched": true` or a `"note"` field explaining idempotent re-run
- [ ] **3.6** `06_verify.json` exists and contains `"result": "VERIFIED"`
- [ ] **3.7** `07_route_history.json` exists and contains at least one entry in `"routes"`
- [ ] **3.8** The `receipt_hash` value is identical in `03_route_case.json`, `04_receipt.json`, `05_dispatch.json` (when present), and `06_verify.json`

---

## Section 4 — Artifact Persistence Acceptance

Inspect `release/evidence/current/data_artifacts/` for each item.

- [ ] **4.1** `data_artifacts/cases/f1000001-0000-0000-0000-000000000001.json` exists
- [ ] **4.2** `data_artifacts/receipts/0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb.json` exists
- [ ] **4.3** `data_artifacts/policies/0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb.json` exists
- [ ] **4.4** `data_artifacts/dispatch/0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb.json` exists
- [ ] **4.5** `data_artifacts/verification/0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb.json` exists
- [ ] **4.6** The stored receipt in `data_artifacts/receipts/` is valid JSON and contains the same `receipt_hash` as in `04_receipt.json`

---

## Section 5 — Evidence Bundle Acceptance

- [ ] **5.1** `release/evidence/current/` exists and is non-empty
- [ ] **5.2** `summary.txt` exists and ends with `All 7 steps passed.`
- [ ] **5.3** `commands.txt` exists and lists all 7 HTTP calls
- [ ] **5.4** `git_head.txt` exists and contains a 40-character hex commit hash
- [ ] **5.5** `inputs/case.json` exists and contains `"case_id": "f1000001-0000-0000-0000-000000000001"`
- [ ] **5.6** `inputs/registry_snapshot.json` exists and is valid JSON
- [ ] **5.7** `inputs/config.json` exists and contains `"jurisdiction": "DE"`
- [ ] **5.8** All seven numbered response files (`01_health.json` through `07_route_history.json`) are present
- [ ] **5.9** `release/evidence/current/` is excluded from git (listed in `.gitignore`)

---

## Section 6 — Review Packet Consistency

- [ ] **6.1** `release/review/README.md` exists
- [ ] **6.2** `release/review/SYSTEM_OVERVIEW.md` exists
- [ ] **6.3** `release/review/OPERATOR_FLOW.md` exists
- [ ] **6.4** `release/review/ARTIFACT_GUIDE.md` exists
- [ ] **6.5** `release/review/BOUNDARIES.md` exists
- [ ] **6.6** `release/review/OPERATOR_FLOW.md` documents the same 7-endpoint flow as `release/smoke_test.sh`
- [ ] **6.7** `release/review/ARTIFACT_GUIDE.md` correctly states the deterministic `receipt_hash` value (`0db54077…`)
- [ ] **6.8** `release/review/BOUNDARIES.md` lists all seven pilot endpoints as frozen
- [ ] **6.9** No document in `release/review/` references an endpoint, file, or command that does not exist in the repo

---

## Section 7 — Frozen Boundary Acceptance

- [ ] **7.1** No checklist step required changing protocol behavior, routing logic, schemas, or kernel code to pass
- [ ] **7.2** `examples/pilot/case.json` is unchanged from the committed version (verify with `git diff examples/pilot/`)
- [ ] **7.3** `examples/pilot/registry_snapshot.json` is unchanged from the committed version
- [ ] **7.4** `examples/pilot/config.json` is unchanged from the committed version
- [ ] **7.5** `release/evidence/current/04_receipt.json` shows `"routing_kernel_version": "postcad-routing-v1"` (kernel version unchanged)
- [ ] **7.6** `release/evidence/current/04_receipt.json` shows `"schema_version": "1"` (schema unchanged)
- [ ] **7.7** The canonical `receipt_hash` in the evidence matches the value in `release/review/ARTIFACT_GUIDE.md` (`0db54077cff0fbc4…`)

---

## Overall result

All 7 sections pass → **ACCEPTED**

Any item fails → investigate before accepting. Record in `REVIEW_WORKSHEET.md`.
