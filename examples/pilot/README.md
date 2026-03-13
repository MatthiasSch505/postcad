# PostCAD Pilot Bundle

## Quick Start — Canonical Demo

One command runs the full pilot flow end-to-end against a live service:

```bash
cargo run -p postcad-service &          # start service on :8080
./examples/pilot/demo.sh                # route → dispatch → approve → export → verify
```

Expected final output:

```
  DEMO COMPLETE
  Candidate:    pilot-de-001
  Receipt hash: 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb
  Verification: VERIFIED
```

The export packet is written to `examples/pilot/export_packet.json`.
The reviewer shell is at `http://localhost:8080/reviewer`.

---

## What PostCAD Solves

After a dental CAD design is complete, someone still has to decide which manufacturer
gets the case. Today that step is manual, opaque, and relationship-driven: a coordinator
checks a spreadsheet, picks a lab, and hopes the choice was compliant. There is no
audit trail and no way for a third party to verify the decision after the fact.

PostCAD replaces that step with deterministic, rule-based routing. Every routing
decision is cryptographically committed to a receipt. The receipt can be independently
verified — by the clinic, the manufacturer, or a regulator — without trusting the
platform.

---

## Operator Demo Flow

### Step 1 — Route

Routes the pilot case against the manufacturer registry. Self-verification runs
automatically inside the routing step; the command exits non-zero if the receipt
fails to verify.

```bash
./examples/pilot/run_pilot.sh
```

Expected output:

```
Result:               routed
Selected candidate:   pilot-de-001
Receipt hash:         0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb
Kernel version:       postcad-routing-v1

Receipt written to:   examples/pilot/receipt.json

Verification: OK
```

### Step 2 — Verify

Independently verifies the receipt produced in Step 1 against the original inputs.
No routing is re-run. The kernel replays the decision deterministically and confirms
every hash field in the receipt.

```bash
./examples/pilot/verify.sh
```

Expected output:

```
VERIFIED
```

The same inputs always produce the same receipt hash. The receipt will not verify
if the case, the registry snapshot, or the routing policy has drifted from the
values committed at routing time.

---

## Files

| File | Description |
|------|-------------|
| `case.json` | Dental case input — procedure, material, jurisdiction, routing policy |
| `registry_snapshot.json` | Three active manufacturers in Germany, all capable of zirconia crowns |
| `config.json` | Routing configuration — jurisdiction `DE`, policy `allow_domestic_and_cross_border` |
| `derived_policy.json` | Full routing policy bundle derived from `registry_snapshot.json` — required for `verify-receipt` |
| `candidates.json` | Eligible candidate list extracted from `derived_policy.json` — required for `verify-receipt` |
| `expected_routed.json` | Locked routing receipt — canonical output for these inputs |
| `expected_verify.json` | Locked verification result — `{"result":"VERIFIED"}` |
| `receipt.json` | Written by `run_pilot.sh` at runtime |

---

## Prerequisites

```bash
cargo build --bin postcad-cli
```

Binary is written to `target/debug/postcad-cli`.

---

## CLI: Route (raw)

Routes the pilot case and self-verifies the receipt in one step. Exits non-zero if
self-verification fails.

```bash
./target/debug/postcad-cli route-case-from-registry --json \
  --case     examples/pilot/case.json \
  --registry examples/pilot/registry_snapshot.json \
  --config   examples/pilot/config.json
```

Expected output matches `expected_routed.json`.

---

## CLI: Verify (raw)

```bash
./target/debug/postcad-cli verify-receipt \
  --receipt    examples/pilot/receipt.json \
  --case       examples/pilot/case.json \
  --policy     examples/pilot/derived_policy.json \
  --candidates examples/pilot/candidates.json
```

Expected output: `VERIFIED`. Expected exit code: `0`.

Note: `--policy` takes `derived_policy.json` (full bundle including `snapshots`).
`--candidates` takes `candidates.json` (flat array). They are separate files.

---

## Service Flow

Start the service:

```bash
cargo run -p postcad-service
# listening on 0.0.0.0:8080
```

Run the canonical demo (route → dispatch → approve → export → verify):

```bash
./examples/pilot/demo.sh
```

Independent verify against the locked receipt:

```bash
curl -s -X POST http://localhost:8080/verify \
  -H 'Content-Type: application/json' \
  -d "{
    \"receipt\": $(cat examples/pilot/expected_routed.json),
    \"case\":    $(cat examples/pilot/case.json),
    \"policy\":  $(cat examples/pilot/derived_policy.json)
  }"
```

Expected response: `{"result":"VERIFIED"}`.

---

## Locked Outputs

`expected_routed.json`:
- `outcome`: `"routed"`
- `selected_candidate_id`: `"pilot-de-001"`
- `receipt_hash`: `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb`

`expected_verify.json`:
```json
{"result": "VERIFIED"}
```

Running the same inputs twice produces the same `receipt_hash`. The receipt will not
change unless the inputs change.

---

## Smoke Test

```bash
cargo test -p postcad-service --test pilot_bundle_smoke_test
```

The test routes using the pilot fixtures, compares the receipt to `expected_routed.json`
value-for-value, then verifies and compares to `expected_verify.json`.
