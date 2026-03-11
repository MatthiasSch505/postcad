# PostCAD Pilot Bundle

End-to-end runnable example of registry-backed routing and receipt verification using PostCAD Protocol v1.

To run the service locally with Docker, see [docs/local_service_run.md](../../docs/local_service_run.md).

---

## Files

| File | Description |
|------|-------------|
| `case.json` | Dental case input — procedure, material, jurisdiction, routing policy |
| `registry_snapshot.json` | Three active manufacturers in Germany, all capable of zirconia crowns |
| `config.json` | Routing configuration — jurisdiction `DE`, policy `allow_domestic_and_cross_border` |
| `derived_policy.json` | Full routing policy bundle derived from `registry_snapshot.json` — required for `verify-receipt` |
| `expected_routed.json` | Locked routing receipt — canonical output for these inputs |
| `expected_verify.json` | Locked verification result — `{"result":"VERIFIED"}` |
| `receipt.json` | Written by `run_pilot.sh` at runtime |

---

## Prerequisites

```bash
cargo build --bin postcad-cli
cargo build --bin postcad-service
```

Binaries are written to `target/debug/`.

---

## CLI: Route

Routes the pilot case and self-verifies the receipt in one step. Exits non-zero if self-verification fails.

```bash
./target/debug/postcad-cli route-case-from-registry --json \
  --case     examples/pilot/case.json \
  --registry examples/pilot/registry_snapshot.json \
  --config   examples/pilot/config.json
```

Expected output: contents of `expected_routed.json`.

`run_pilot.sh` wraps this command and writes the receipt to `receipt.json`:

```bash
./examples/pilot/run_pilot.sh
```

---

## CLI: Verify

Explicit receipt verification against the locked receipt and derived policy:

```bash
./target/debug/postcad-cli verify-receipt --json \
  --receipt    examples/pilot/expected_routed.json \
  --case       examples/pilot/case.json \
  --policy     examples/pilot/derived_policy.json \
  --candidates examples/pilot/derived_policy.json
```

Note: `derived_policy.json` contains both `snapshots` (used by `--policy`) and `candidates` (used by `--candidates`). Pass it for both arguments.

Expected exit code: 0. Expected stdout: `{"result":"VERIFIED"}`.

---

## Service: Run

```bash
./target/debug/postcad-service
# listening on 0.0.0.0:8080
```

Set a custom address with `POSTCAD_ADDR`:

```bash
POSTCAD_ADDR=127.0.0.1:9000 ./target/debug/postcad-service
```

---

## Service: /health

```bash
curl -s http://localhost:8080/health
```

Expected response:

```json
{"status":"ok"}
```

---

## Service: /version

```bash
curl -s http://localhost:8080/version
```

Expected response:

```json
{"protocol_version":"postcad-v1","routing_kernel_version":"postcad-routing-v1","service":"postcad-service"}
```

---

## Service: POST /route

```bash
curl -s -X POST http://localhost:8080/route \
  -H 'Content-Type: application/json' \
  -d "{
    \"case\":             $(cat examples/pilot/case.json),
    \"registry_snapshot\": $(cat examples/pilot/registry_snapshot.json),
    \"routing_config\":   $(cat examples/pilot/config.json)
  }"
```

Expected response shape:

```json
{
  "receipt": { ... },
  "derived_policy": { ... }
}
```

`receipt` matches `expected_routed.json`. `derived_policy` matches `derived_policy.json`.

---

## Service: POST /verify

```bash
curl -s -X POST http://localhost:8080/verify \
  -H 'Content-Type: application/json' \
  -d "{
    \"receipt\": $(cat examples/pilot/expected_routed.json),
    \"case\":    $(cat examples/pilot/case.json),
    \"policy\":  $(cat examples/pilot/derived_policy.json)
  }"
```

Expected response: contents of `expected_verify.json`.

```json
{"result":"VERIFIED"}
```

---

## Expected Output Files

`expected_routed.json` — locked routing receipt:
- `outcome`: `"routed"`
- `selected_candidate_id`: `"pilot-de-001"`
- `receipt_hash`: `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb`

`expected_verify.json`:
```json
{"result": "VERIFIED"}
```

Running the same inputs twice produces the same `receipt_hash`. The receipt will not change unless the inputs change.

---

## Smoke Test

The pilot bundle is covered by an in-process smoke test:

```bash
cargo test -p postcad-service --test pilot_bundle_smoke_test
```

The test routes using the pilot fixtures, compares the receipt to `expected_routed.json` value-for-value, then verifies and compares to `expected_verify.json`.
