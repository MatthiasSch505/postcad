# Pilot Acceptance Run

---

## Purpose

`scripts/pilot_acceptance.sh` is the one-command pilot acceptance runner. It starts the service, validates all locked pilot fixtures end-to-end, prints a PASS/FAIL summary, and exits nonzero on any failure.

---

## Exact command

```bash
make acceptance
```

Or directly:

```bash
scripts/pilot_acceptance.sh
```

To use a non-default port:

```bash
POSTCAD_ACCEPTANCE_PORT=9090 scripts/pilot_acceptance.sh
```

---

## What it does

| Step | Action |
|------|--------|
| 1 | Builds `postcad-service` if the binary is absent |
| 2 | Starts the service on `127.0.0.1:8080` (background) |
| 3 | Polls `GET /health` until `{"status":"ok"}` (20s timeout) |
| 4 | Calls `GET /version`; asserts `protocol_version`, `routing_kernel_version`, `service` |
| 5 | Calls `POST /route` with `examples/pilot/{case,registry_snapshot,config}.json`; asserts receipt matches `examples/pilot/expected_routed.json` value-for-value |
| 6 | Calls `POST /verify` with `examples/pilot/{expected_routed,case,derived_policy}.json`; asserts result matches `examples/pilot/expected_verify.json` |
| 7 | Checks `git diff examples/pilot/` is clean (locked fixtures were not modified) |
| 8 | Prints `RESULT: PASS` and check counts |

The service is stopped in a `trap EXIT` handler regardless of pass or fail.

---

## PASS condition

```
================================
  RESULT : PASS
  checks : 7 passed, 0 failed
================================
```

All 7 checks must pass. Exit code 0.

---

## FAIL meaning

Any `[FAIL]` line means a specific check did not pass. The runner exits nonzero immediately on the first failure. The service is still stopped by the cleanup trap.

Common causes:

| Symptom | Likely cause |
|---------|--------------|
| `/health` timeout | Port conflict or binary failed to start — check `service.stderr` in the temp dir (printed on failure) |
| `/version` field mismatch | Stale binary — run `cargo build --bin postcad-service` and retry |
| Route receipt mismatch | Receipt content changed — rebuild from source; do not edit `expected_routed.json` |
| Verify mismatch | `derived_policy.json` or `expected_routed.json` was modified — restore with `git checkout examples/pilot/` |
| Fixture integrity fail | `examples/pilot/` has uncommitted changes — restore with `git checkout examples/pilot/` |

---

## Temporary files

All intermediate outputs are written to a `mktemp -d` directory. The directory is deleted on exit (pass or fail). No files are written to the repository.

---

## Locked fixture paths

| File | Role |
|------|------|
| `examples/pilot/case.json` | Routing case input |
| `examples/pilot/registry_snapshot.json` | Registry input |
| `examples/pilot/config.json` | Routing config |
| `examples/pilot/derived_policy.json` | Policy bundle for verify |
| `examples/pilot/expected_routed.json` | Locked expected receipt |
| `examples/pilot/expected_verify.json` | Locked expected verify result |
