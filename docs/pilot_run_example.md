# PostCAD Pilot Run — Canonical Happy-Path Trace

**Protocol:** `postcad-v1` · **Kernel:** `postcad-routing-v1`

All values below are from the canonical pilot fixture (`examples/pilot/`).

---

## Input Case

```json
{
  "case_id":            "f1000001-0000-0000-0000-000000000001",
  "jurisdiction":       "DE",
  "material":           "zirconia",
  "procedure":          "crown",
  "file_type":          "stl",
  "patient_country":    "germany",
  "manufacturer_country": "germany",
  "routing_policy":     "allow_domestic_and_cross_border"
}
```

---

## Step 1 — Route Result

```
POST /cases/:id/route

outcome:               "routed"
selected_candidate_id: "pilot-de-001"
receipt_hash:          "0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb"
```

Key receipt commitments:

```
case_fingerprint:        169b48d0441c6acd7024710020886589dc9460c35c3ec47e278b4187641abefd
registry_snapshot_hash:  9b5e87ce43243821fc022048930f2f8d3d215c743a3fc1a3e33569df48ddf822
routing_decision_hash:   b97bb757d3f41b6bf764b9a17ff33faab56b59759f7e54df6bd5b44adb9952c5
audit_seq:               0
audit_entry_hash:        7eaa6b6742e98a2037000c039138a5091f5ed1490f6dad9bafbc83bdf9d644eb
audit_previous_hash:     0000000000000000000000000000000000000000000000000000000000000000
```

---

## Step 2 — Verification

```
POST /dispatch/:hash/verify

result: "VERIFIED"
```

The kernel replayed the routing decision from raw inputs and all 21 hash fields matched.

---

## Step 3 — Create Dispatch

```
POST /dispatch/create
  body: { receipt, case, policy }

dispatch_id:        <uuid generated at creation>
status:             "draft"
receipt_hash:       "0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb"
verification_passed: true
```

Verification is re-run inline. If it fails the request is rejected (422).
Only one dispatch is permitted per `receipt_hash`.

---

## Step 4 — Approve Dispatch

```
POST /dispatch/:dispatch_id/approve
  body: { "approved_by": "operator-de-01" }

status:      "approved"
approved_by: "operator-de-01"
approved_at: "2026-03-13T10:00:00Z"
```

Routing and receipt fields are now immutable. A second approval returns 409.

---

## Step 5 — Export Dispatch Packet

```
GET /dispatch/:dispatch_id/export

status: "exported"
```

Exported packet (all 10 fields, deterministic alphabetical order):

```json
{
  "approved_at":               "2026-03-13T10:00:00Z",
  "approved_by":               "operator-de-01",
  "case_id":                   "f1000001-0000-0000-0000-000000000001",
  "created_at":                "2026-03-13T09:59:00Z",
  "dispatch_id":               "<uuid>",
  "manufacturer_payload_json": null,
  "receipt_hash":              "0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb",
  "selected_candidate_id":     "pilot-de-001",
  "status":                    "exported",
  "verification_passed":       true
}
```

The `receipt_hash` in the export packet links back to the exact routing receipt that was
verified and approved. Same bytes every time for the same approved dispatch.

---

## What This Run Proves

| Claim | Evidence |
|---|---|
| Routing is deterministic | `receipt_hash` is `0db54077…` every time these inputs are routed |
| Receipt integrity holds | All 21 hash fields verified by replay — `result: "VERIFIED"` |
| Dispatch binds to exact receipt | `receipt_hash` in export packet matches routing receipt exactly |
| Approval is recorded | `approved_by` + `approved_at` locked in exported packet |
| Audit chain is intact | `audit_seq: 0`, genesis entry (`audit_previous_hash: 000…`), hash verified |
| No silent decisions | Every step carries a `ReasonCode` via `Decision<T>` |

This is a complete protocol execution: case in, cryptographic proof out, handoff packet exported.
No AI, no probabilistic judgment, no clinical decision. PostCAD's role ends at the export step.
