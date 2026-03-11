# PostCAD Demo Run

---

## Purpose

`demo/run_demo.sh` is a self-contained, deterministic external demo that walks through the complete PostCAD 4-step dispatch flow in one command. It is the fastest way for a new operator or partner to see the full system end-to-end.

---

## Single command

```bash
./demo/run_demo.sh
```

The script builds the service binary if needed, starts the service, runs all 8 steps, and stops the service on exit.

---

## What it does

| Step | Endpoint | Description |
|------|----------|-------------|
| 1 | — | Start `postcad-service` on `localhost:8080` |
| 2 | `GET /health` | Wait until the service is accepting requests |
| 3 | `POST /cases` | Store `demo/case_demo.json` (zirconia crown, DE jurisdiction) |
| 4 | `POST /cases/:id/route` | Route the stored case against the pilot registry |
| 5 | `GET /receipts/:hash` | Retrieve and inspect the routing receipt |
| 6 | `POST /dispatch/:hash` | Dispatch the routed receipt |
| 7 | `POST /dispatch/:hash/verify` | Verify the dispatched receipt — expects `VERIFIED` |
| 8 | `GET /routes` | Show the full route history |

The script asserts the expected HTTP status code at each step and exits non-zero if any assertion fails.

---

## Expected output (abbreviated)

```
════════ STEP 2 — Waiting for /health ════════
  Service is up (attempt 2)
{"status": "ok"}

════════ STEP 3 — Store demo case ════════
{"case_id": "d0000001-0000-0000-0000-000000000001", "stored": true}
  Stored case_id: d0000001-0000-0000-0000-000000000001

════════ STEP 4 — Route stored case ════════
{
  "outcome": "routed",
  "selected_candidate_id": "pilot-de-001",
  "receipt_hash": "..."
}

...

════════ STEP 7 — Verify dispatch ════════
{"receipt_hash": "...", "result": "VERIFIED"}
  Verification: VERIFIED

════════ DEMO COMPLETE — all 8 steps passed ════════
```

---

## Prerequisites

- Rust toolchain (`cargo`) installed
- `python3` on `PATH` (used for JSON pretty-printing only)
- Port 8080 free

No Docker required. The script uses a temporary data directory for all runtime state and cleans up on exit.

---

## Demo case

`demo/case_demo.json` — a zirconia crown routed under DE jurisdiction against the pilot registry (`examples/pilot/registry_snapshot.json`). The case uses a distinct `case_id` (`d0000001-…`) so it does not conflict with pilot fixture cases.

---

## See also

- `release/README.md` — pilot release bundle: reset, start, smoke test
- `docs/local_service_run.md` — full curl reference for every endpoint
- `docs/pilot_maturity_check.md` — current readiness assessment
- `examples/pilot/README.md` — pilot fixture bundle
