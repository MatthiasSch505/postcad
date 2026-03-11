# PostCAD Pilot — Operator Runbook

Local operator guide for the PostCAD pilot service. No Docker, no external services.

---

## Prerequisites

| Requirement | Notes |
|---|---|
| Rust toolchain (`cargo`) | `start_pilot.sh` builds the binary automatically if absent |
| `python3` on PATH | used by smoke test for JSON parsing |
| `curl` on PATH | used by smoke test |
| Port 8080 free | or override with `POSTCAD_ADDR=host:port` |

Quick check:
```bash
cargo --version && python3 --version && curl --version | head -1
```

---

## Exact sequence

### Step 1 — Reset (clean slate)

```bash
./release/reset_pilot_data.sh
```

Removes `data/cases/`, `data/receipts/`, `data/policies/`, `data/dispatch/`, `data/verification/`.
Does **not** touch source code, compiled artifacts, or canonical fixtures.

### Step 2 — Start the service (Terminal A)

```bash
./release/start_pilot.sh
```

Starts `postcad-service` in the foreground on `http://localhost:8080`.
Leave it running. Stop with `Ctrl-C`.

Overrides:
```bash
POSTCAD_ADDR=127.0.0.1:9000 ./release/start_pilot.sh
POSTCAD_DATA=/tmp/pilot_data  ./release/start_pilot.sh
```

### Step 3 — Smoke test (Terminal B, while service is running)

```bash
./release/smoke_test.sh
```

Runs a 7-step deterministic flow against the live service:

| Step | Method | Endpoint |
|------|--------|----------|
| 1 | GET | `/health` |
| 2 | POST | `/cases` |
| 3 | POST | `/cases/:id/route` |
| 4 | GET | `/receipts/:hash` |
| 5 | POST | `/dispatch/:hash` |
| 6 | POST | `/dispatch/:hash/verify` |
| 7 | GET | `/routes` |

Exits 0 on success. Prints `[FAIL]` and exits nonzero on the first assertion failure.

### Step 4 — Demo (optional, self-contained)

```bash
./demo/run_demo.sh
```

Starts its own service, runs an 8-step flow, then stops the service. No separate terminal needed.
See `docs/demo_run.md` for details.

### Step 5 — Generate evidence bundle (while service is still running)

```bash
./release/generate_evidence_bundle.sh
```

Captures the 7-step pilot flow as an inspectable folder at `release/evidence/current/`.
See `release/evidence/README.md` for the full folder structure and what to inspect.

---

## What success looks like

```
── 7. Route history ──────────────────────────────────
  [PASS] route history (1 route(s))

════════════════════════════════════════
  SMOKE TEST PASSED — all 7 steps OK
════════════════════════════════════════
```

---

## Where data lives

| Directory | Contents |
|---|---|
| `data/cases/` | Stored case JSON, keyed by `case_id` |
| `data/receipts/` | Routing receipts, keyed by `receipt_hash` |
| `data/policies/` | Derived policies, keyed by `receipt_hash` |
| `data/dispatch/` | Dispatch records, keyed by `receipt_hash` |
| `data/verification/` | Verification results, keyed by `receipt_hash` |

Directories are created on first write.
All removed by `reset_pilot_data.sh`.

---

## Common problems and direct fixes

| Symptom | Cause | Fix |
|---|---|---|
| `[FAIL] Cannot reach http://localhost:8080/health` | Service not started | Run `./release/start_pilot.sh` in Terminal A first |
| `cargo: command not found` | Rust not installed | Install from https://rustup.rs |
| `python3: command not found` | Python 3 not on PATH | Install Python 3 and ensure it is on PATH |
| Port 8080 in use | Another process bound to 8080 | `POSTCAD_ADDR=localhost:9090 ./release/start_pilot.sh` and `POSTCAD_ADDR=http://localhost:9090 ./release/smoke_test.sh` |
| `Binary not found — building...` on start | First run or clean rebuild | Normal — wait for the build to finish |
| Smoke test step 5 returns 409 | Receipt already dispatched from a previous run | Normal — the script accepts 409 as idempotent |
| Smoke test fails after partial run | Data from previous incomplete run | Run `./release/reset_pilot_data.sh` and start over |

---

## What this does NOT change

- Protocol version (`postcad-v1`)
- Routing kernel behavior
- Receipt schema or canonical hashes
- Endpoint response shapes
- Canonical fixtures in `examples/pilot/` or `tests/protocol_vectors/`

---

## Additional references

- `release/INDEX.md` — top-level index of every release surface with recommended paths
- `release/FREEZE_MANIFEST.md` — single-page freeze manifest with full surface listing and frozen boundaries
- `release/freeze/` — freeze bundle: grouped surface inventory, frozen boundaries, classification helper
- `release/selfcheck/` — read-only structural self-check of the whole release package
- `release/walkthrough/` — single-surface pilot walkthrough (sequence, commands, expected outputs, inspection points)
- `release/handoff/` — new operator/reviewer entry point (first-hour guide, handoff checklist, known-good state)
- `release/acceptance/` — pilot acceptance checklist, review worksheet, and acceptance pre-check
- `release/review/` — external review packet (system overview, operator flow, artifact guide, boundaries)
- `docs/local_service_run.md` — full curl reference for every endpoint
- `docs/demo_run.md` — single-command demo walkthrough
- `docs/pilot_maturity_check.md` — readiness assessment
- `cargo test --workspace` — full test suite (no service required)
