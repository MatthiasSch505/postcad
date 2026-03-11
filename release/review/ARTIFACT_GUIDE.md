# PostCAD — Artifact Guide

How to inspect the outputs of a local pilot run. All files described here are under `release/evidence/current/` after running `./release/generate_evidence_bundle.sh`.

---

## Full folder structure

```
release/evidence/current/
│
├── summary.txt                    human-readable pass/fail confirmation
├── commands.txt                   exact HTTP calls and fixture paths used
├── git_head.txt                   git commit hash at time of capture
│
├── inputs/                        input fixtures copied from examples/pilot/
│   ├── case.json                  the routing case submitted
│   ├── registry_snapshot.json     the manufacturer registry used
│   └── config.json                the routing config (jurisdiction + policy)
│
├── 01_health.json                 GET /health response
├── 02_store_case.json             POST /cases response
├── 03_route_case.json             POST /cases/:id/route response
├── 04_receipt.json                GET /receipts/:hash response (full receipt)
├── 05_dispatch.json               POST /dispatch/:hash response
├── 06_verify.json                 POST /dispatch/:hash/verify response
├── 07_route_history.json          GET /routes response
│
└── data_artifacts/
    ├── cases/{case_id}.json       stored case, as written by the service
    ├── receipts/{hash}.json       stored receipt, as written by the service
    ├── policies/{hash}.json       derived routing policy, as written by the service
    ├── dispatch/{hash}.json       dispatch record, as written by the service
    └── verification/{hash}.json   verification result, as written by the service
```

`data_artifacts/` is present only if the service used the default `data/` path.

---

## What to inspect first

### 1. `summary.txt`

Confirms the overall pass/fail state of the capture run. A clean run ends with:

```
All 7 steps passed.
```

Key fields:
```
selected  = pilot-de-001
result    = VERIFIED  (expected: VERIFIED)
routes    = 1 route(s) in history
```

### 2. `04_receipt.json` — the routing receipt

The central artifact. Contains all hash commitments for the routing decision. Key fields:

| Field | Meaning |
|---|---|
| `schema_version` | Always `"1"` for this protocol |
| `outcome` | `"routed"` or `"refused"` |
| `selected_candidate_id` | The selected manufacturer (`"pilot-de-001"`) |
| `receipt_hash` | SHA-256 of all other receipt fields; verified first by the verifier |
| `case_fingerprint` | SHA-256 of the canonical case payload |
| `policy_fingerprint` | SHA-256 of the routing policy bundle |
| `candidate_pool_hash` | SHA-256 of the full candidate pool (order-independent) |
| `eligible_candidate_ids_hash` | SHA-256 of eligible candidates after compliance filtering |
| `selection_input_candidate_ids_hash` | SHA-256 of candidates in order presented to the selector |
| `routing_decision_hash` | SHA-256 of the canonical routing decision |
| `routing_proof_hash` | SHA-256 of the routing proof object |
| `audit_entry_hash` | SHA-256 of the audit log entry for this decision |
| `audit_previous_hash` | SHA-256 of the preceding audit entry (64 zeros = genesis) |
| `routing_kernel_version` | `"postcad-routing-v1"` |

The receipt in `04_receipt.json` must have the same `receipt_hash` as shown in `03_route_case.json` and `06_verify.json`.

### 3. `06_verify.json` — verification result

Must contain exactly:
```json
{
  "receipt_hash": "0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb",
  "result": "VERIFIED"
}
```

`VERIFIED` means the service independently recomputed all hashes from the stored inputs and they all matched.

### 4. `inputs/` — what was submitted

- `case.json` — the canonical pilot case (jurisdiction DE, zirconia crown, `case_id: f1000001-0000-0000-0000-000000000001`)
- `registry_snapshot.json` — the manufacturer registry snapshot
- `config.json` — the routing config (`jurisdiction: DE`, `routing_policy: allow_domestic_and_cross_border`)

A reviewer can re-run verification independently using only these files and the CLI:

```bash
cargo run -p postcad-cli -- verify-receipt --json \
  --receipt  release/evidence/current/04_receipt.json \
  --case     examples/pilot/case.json \
  --policy   data/policies/<receipt_hash>.json
```

---

## File categories

| Category | Files | Notes |
|---|---|---|
| Context | `summary.txt`, `commands.txt`, `git_head.txt` | Capture metadata; not service artifacts |
| Inputs | `inputs/case.json`, `inputs/registry_snapshot.json`, `inputs/config.json` | Copied from `examples/pilot/`; frozen canonical fixtures |
| API responses | `01_` through `07_` | HTTP responses captured inline during the generator run |
| Stored artifacts | `data_artifacts/` | Files written to disk by the service during the run |

---

## What a clean run looks like

All of the following are true:

- `summary.txt` ends with `All 7 steps passed.`
- `04_receipt.json` has `"outcome": "routed"` and `"selected_candidate_id": "pilot-de-001"`
- `04_receipt.json` has `"receipt_hash": "0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb"`
- `06_verify.json` has `"result": "VERIFIED"`
- `07_route_history.json` contains at least one entry in `"routes"`
- The `receipt_hash` value is identical across `03_route_case.json`, `04_receipt.json`, `05_dispatch.json`, and `06_verify.json`
- `data_artifacts/receipts/` contains exactly one file matching the receipt hash

---

## Notes on determinism

- The `receipt_hash` is deterministic: `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb` for the canonical pilot inputs, every run.
- Re-running the generator replaces `current/` entirely; no timestamp-based naming.
- The pilot `case_id` is always `f1000001-0000-0000-0000-000000000001`.
- `05_dispatch.json` may contain a `"note"` field instead of `"dispatched": true` if the receipt was already dispatched in the same service session; this is normal and expected on re-runs.
