# PostCAD Pilot Release Bundle

This directory contains the operator-facing scripts for running, resetting, and smoke-testing the PostCAD pilot service locally. No external services, no Docker required.

---

## Prerequisites

- Rust toolchain (`cargo`) — the start script builds the binary automatically if absent
- `python3` on `PATH` — used by smoke test for JSON pretty-printing only
- `curl` — used by smoke test
- Port 8080 free

---

## Exact order of commands

### 1. Reset runtime data (clean slate)

```bash
./release/reset_pilot_data.sh
```

Removes `data/cases/`, `data/receipts/`, `data/policies/`, `data/dispatch/`, `data/verification/`.
Does **not** touch source code, compiled artifacts, or canonical fixtures.

### 2. Start the service

```bash
./release/start_pilot.sh
```

Starts `postcad-service` in the foreground on `localhost:8080`.
Leave it running in one terminal; use another for the smoke test.
Stop with `Ctrl-C`.

To override address or data location:

```bash
POSTCAD_ADDR=127.0.0.1:9000 ./release/start_pilot.sh
POSTCAD_DATA=/tmp/pilot_data  ./release/start_pilot.sh
```

### 3. Run the smoke test (separate terminal)

```bash
./release/smoke_test.sh
```

Runs a deterministic 7-step pilot flow against the running service:

| Step | Endpoint |
|------|----------|
| 1 | `GET /health` |
| 2 | `POST /cases` |
| 3 | `POST /cases/:id/route` |
| 4 | `GET /receipts/:hash` |
| 5 | `POST /dispatch/:hash` |
| 6 | `POST /dispatch/:hash/verify` |
| 7 | `GET /routes` |

Exits 0 on success. Exits nonzero and prints `[FAIL]` on the first assertion failure.

---

## Where outputs live

| Directory | Contents |
|-----------|----------|
| `data/cases/` | Stored case JSON files, keyed by `case_id` |
| `data/receipts/` | Routing receipts, keyed by `receipt_hash` |
| `data/policies/` | Derived policies, keyed by `receipt_hash` |
| `data/dispatch/` | Dispatch records, keyed by `receipt_hash` |
| `data/verification/` | Verification results, keyed by `receipt_hash` |

All directories are created lazily on first write.

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

## What this does NOT change

- Protocol version (`postcad-v1`)
- Routing kernel behavior
- Receipt schema or canonical hashes
- Any endpoint response shapes
- Canonical fixtures in `examples/pilot/` or `tests/protocol_vectors/`

---

## Additional commands

```bash
# Full test suite (no service required)
cargo test --workspace

# Operator UI (browser)
open http://localhost:8080/

# Full external demo (starts/stops service itself)
./demo/run_demo.sh
```

See also:
- `docs/local_service_run.md` — full curl reference for every endpoint
- `docs/pilot_maturity_check.md` — readiness assessment
- `docs/demo_run.md` — single-command external demo
