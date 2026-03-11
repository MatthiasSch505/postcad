# PostCAD — Operator Flow

Exact local operator sequence. All commands are repo-relative and run from the repo root.

---

## Prerequisites

```bash
cargo --version   # Rust toolchain
python3 --version # for JSON parsing in scripts
curl --version    # for HTTP calls in scripts
```

Port 8080 must be free, or set `POSTCAD_ADDR=host:port` to override.

---

## Step 1 — Reset (clean slate)

```bash
./release/reset_pilot_data.sh
```

**What it does:** removes `data/cases/`, `data/receipts/`, `data/policies/`, `data/dispatch/`, `data/verification/`. Skips directories that do not exist.

**What it does NOT touch:** source code, compiled binaries, canonical fixtures (`examples/pilot/`, `tests/protocol_vectors/`).

**Expected output:**
```
[reset_pilot_data] REMOVED  .../data/cases  (N file(s))
...
[reset_pilot_data] Done. N director(ies) removed, N skipped.
```

**If it fails:** check that no process is holding files open in `data/`.

---

## Step 2 — Start the service (Terminal A)

```bash
./release/start_pilot.sh
```

**What it does:** builds `postcad-service` if the binary is absent, then starts the service in the foreground on `http://localhost:8080`. Runtime data is written to `data/`.

**Expected output:**
```
══════════════════════════════════════════
  PostCAD Pilot — Local Service Startup
══════════════════════════════════════════
  Repo root : /path/to/postcad
  Base URL  : http://localhost:8080
  ...
[start_pilot] Service starting — press Ctrl-C to stop.
```

Followed by the service's own listen message (e.g. `postcad-service listening on 0.0.0.0:8080`).

**Leave this terminal running.** Open a second terminal for the next steps.

**If it fails:**
- `Binary not found — building...` is normal on first run; wait for the build.
- Port conflict: use `POSTCAD_ADDR=localhost:9090 ./release/start_pilot.sh`.

---

## Step 3 — Smoke test (Terminal B)

```bash
./release/smoke_test.sh
```

**What it does:** runs a 7-step deterministic flow against the live service using the canonical pilot fixture.

| Step | Call | Expected |
|------|------|----------|
| 1 | `GET /health` | HTTP 200, `{"status":"ok"}` |
| 2 | `POST /cases` | HTTP 201 (first run) or 200 (re-run), `{"case_id":"f1000001…","stored":true}` |
| 3 | `POST /cases/:id/route` | HTTP 200, `{"receipt_hash":"0db54077…","selected_candidate_id":"pilot-de-001"}` |
| 4 | `GET /receipts/:hash` | HTTP 200, full receipt JSON |
| 5 | `POST /dispatch/:hash` | HTTP 200 (first run) or 409 (re-run, idempotent) |
| 6 | `POST /dispatch/:hash/verify` | HTTP 200, `{"result":"VERIFIED"}` |
| 7 | `GET /routes` | HTTP 200, route history with ≥1 entry |

**Expected final output:**
```
════════════════════════════════════════
  SMOKE TEST PASSED — all 7 steps OK
════════════════════════════════════════
```

**If it fails:** the script prints `[FAIL] Phase N: ...` identifying which step failed. Common fixes:
- Service not running: start it in Terminal A first.
- Partial data from a previous run: `./release/reset_pilot_data.sh` then re-run from Step 1.

---

## Step 4 — Demo (optional, self-contained)

```bash
./demo/run_demo.sh
```

**What it does:** starts its own service instance, runs an 8-step flow against it (same endpoints as the smoke test plus a longer route/verify chain), then stops the service. Does not require a running service.

**Expected final output:**
```
════════════════════════════════════════
  DEMO COMPLETE — all 8 steps passed
════════════════════════════════════════
```

**If it fails:** check that port 8080 is free before running.

---

## Step 5 — Generate evidence bundle (Terminal B, service still running)

```bash
./release/generate_evidence_bundle.sh
```

**What it does:** re-runs the 7-step pilot flow, saves each API response as a numbered JSON file, copies input fixtures and local data artifacts, writes a summary. Output: `release/evidence/current/`.

Replaces any existing `release/evidence/current/` cleanly each time.

**Expected final output:**
```
══════════════════════════════════════════
  EVIDENCE BUNDLE COMPLETE
  Output: .../release/evidence/current
══════════════════════════════════════════
```

**If it fails:** the script prints `[FAIL] Step NN ...` identifying which call failed. Ensure the service is still running from Step 2.

---

## Complete local run summary

```
Terminal A                          Terminal B
──────────                          ──────────
./release/start_pilot.sh
  (leave running)
                                    ./release/reset_pilot_data.sh
                                    ./release/smoke_test.sh
                                    ./release/generate_evidence_bundle.sh
  Ctrl-C to stop
```

After completion, inspect `release/evidence/current/summary.txt` to confirm all 7 steps passed.
