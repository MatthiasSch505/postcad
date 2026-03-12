# External Pilot Smoke Run

One-command end-to-end validation of the PostCAD frozen pilot from a clean checkout.

---

## Prerequisites

- **Git** — repository cloned, working tree clean (`git status` shows nothing to commit)
- **Rust / cargo** — for building the service binary ([rustup.rs](https://rustup.rs))
- **curl** — HTTP requests
- **python3** — JSON parsing (standard library only, no pip installs)

No cloud access, no external services, no additional package installs.

---

## Run it

From the repo root:

```bash
scripts/external_pilot_smoke.sh
```

Optional: override the default port (8080):

```bash
POSTCAD_SMOKE_PORT=9090 scripts/external_pilot_smoke.sh
```

The script builds the service if the binary is missing, starts it in the background, runs the full pilot flow, and shuts it down on exit.

---

## What success looks like

```
══════════════════════════════════════════
  SMOKE RUN PASSED — 10 stages OK
══════════════════════════════════════════

  commit        : abc1234  (main)
  pilot label   : pilot-local-v1
  case_id       : f1000001-0000-0000-0000-000000000001
  receipt_hash  : 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb
  verify result : VERIFIED

  Fixtures used:
    examples/pilot/case.json
    examples/pilot/registry_snapshot.json
    examples/pilot/config.json
    examples/pilot/derived_policy.json

  Endpoints exercised:
    GET  http://127.0.0.1:8080/health
    POST http://127.0.0.1:8080/cases
    POST http://127.0.0.1:8080/cases/f1000001-.../route
    GET  http://127.0.0.1:8080/receipts/<hash>
    POST http://127.0.0.1:8080/dispatch/<hash>
    POST http://127.0.0.1:8080/dispatch/<hash>/verify
```

Exit code 0 = all stages passed. Any failure prints `[FAIL]` and exits nonzero.

---

## What the script validates

| Stage | What is checked |
|---|---|
| Preflight | `cargo`, `curl`, `python3` on PATH; working tree clean; fixtures present |
| Build | Service binary compiles (skipped if binary already exists) |
| Service start | `/health` responds within 20 seconds |
| 1/7 Health | `GET /health` → `{"status":"ok"}` |
| 2/7 Store case | `POST /cases` → 200 or 201; `case_id` matches canonical value |
| 3/7 Route | `POST /cases/:id/route` → receipt with non-empty `selected_candidate_id` |
| 4/7 Receipt hash | `receipt_hash` matches the frozen pilot value exactly |
| 5/7 Retrieve receipt | `GET /receipts/:hash` → `outcome=routed` |
| 6/7 Dispatch | `POST /dispatch/:hash` → 200 (or 409 on re-run) |
| 7/7 Verify | `POST /dispatch/:hash/verify` → `result=VERIFIED` |

The frozen receipt hash (`0db54077…`) is embedded in the script. Any deviation in routing output will fail stage 4/7.

---

## Where to find outputs

All output goes to stdout. No files are written by the script itself.

The service writes runtime data to `./data/` (auto-created, safe to delete). Clean it up with:

```bash
./release/reset_pilot_data.sh
```

---

## Common failure cases

| Symptom | Fix |
|---|---|
| `Working tree is not clean` | `git stash` or commit changes first |
| `Required tool not found: cargo` | Install Rust toolchain: `curl https://sh.rustup.rs -sSf \| sh` |
| `service did not respond ... after 20s` | Port 8080 may be in use — set `POSTCAD_SMOKE_PORT=9090` |
| `receipt_hash mismatch` | `examples/pilot/` fixtures were modified — restore with `git checkout examples/pilot/` |
| `expected VERIFIED, got FAILED` | Policy or registry fixture drifted — run `git diff examples/pilot/` |
| Build failure | Run `cargo build --bin postcad-service` to see full compiler output |
