# PostCAD — Protocol Walkthrough

---

## System overview

PostCAD is the execution layer after dental CAD design is complete. It takes a
case specification and a manufacturer registry, applies compliance and routing
rules deterministically, and produces a cryptographically committed routing
receipt. The receipt is the chain-of-custody record for the entire downstream
dispatch and verification workflow.

No AI. No randomness. Same inputs always produce the same receipt hash.

---

## Pilot flow

The canonical pilot uses five HTTP endpoints across seven logical steps.

---

### Step 1 — Normalized input

**Endpoint:** `POST /pilot/route-normalized`

The caller submits a 4-field normalized case plus the full manufacturer
registry and routing configuration:

```json
{
  "pilot_case": {
    "case_id":          "f1000001-0000-0000-0000-000000000001",
    "restoration_type": "crown",
    "material":         "zirconia",
    "jurisdiction":     "DE"
  },
  "registry_snapshot": { ... },
  "routing_config":    { "jurisdiction": "DE", "routing_policy": "allow_domestic_and_cross_border" }
}
```

The service normalizes this into the full routing input shape (8 fields) and
forwards it to the routing kernel.

**Artifact produced:** none yet — this is the entry point.
**Files involved:** `case.json`, `registry_snapshot.json`, `config.json`

---

### Step 2 — Routing decision

**Component:** routing kernel (`postcad-routing-v1`)

The kernel applies three sequential filters to the registry:

1. **Compliance filter** — drops manufacturers that are inactive, missing
   required jurisdiction coverage, or lack the required capability and material.
2. **Candidate pool** — the survivors form the eligible candidate set. For this
   pilot: all three manufacturers (`pilot-de-001`, `pilot-de-002`,
   `pilot-de-003`) pass compliance.
3. **Selection** — the `DeterministicHash` strategy hashes `case_id` to select
   one candidate without state. Result: `pilot-de-001` (Alpha Dental GmbH).

Every filter result is hashed and committed into the receipt.

**Artifact produced:** routing decision (internal, passed directly to step 3).
**Files involved:** `registry_snapshot.json`, `derived_policy.json`

---

### Step 3 — Receipt generation

**Component:** routing kernel

The kernel builds a routing receipt containing 21 committed fields:

| Field | What it commits |
|---|---|
| `routing_input` | Exact 8-field input used for routing |
| `routing_input_hash` | SHA-256 of the canonical routing input |
| `case_fingerprint` | SHA-256 of the case fields |
| `policy_fingerprint` | SHA-256 of the policy bundle |
| `registry_snapshot_hash` | SHA-256 of the registry snapshot |
| `candidate_pool_hash` | SHA-256 of all candidates before filtering |
| `eligible_candidate_ids_hash` | SHA-256 of the post-compliance candidate list |
| `selection_input_candidate_ids_hash` | SHA-256 of the list fed to the selector |
| `candidate_order_hash` | SHA-256 confirming sort order was stable |
| `routing_decision_hash` | SHA-256 covering the selection outcome |
| `routing_proof_hash` | SHA-256 of the full proof object |
| `audit_seq` / `audit_entry_hash` / `audit_previous_hash` | Append-only audit chain linkage |
| `receipt_hash` | SHA-256 of all preceding fields — the top-level integrity seal |

The `receipt_hash` for this pilot is always:

```
0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb
```

**Artifact produced:** `receipt` object returned in the HTTP response.
**Reference file:** `expected_routed.json`

---

### Step 4 — Dispatch creation

**Endpoint:** `POST /dispatch/create`

The caller submits the receipt, the original case, and the derived policy. The
service re-runs verification inline before creating any record. If verification
fails the request is rejected.

On success, a draft dispatch record is created keyed by `receipt_hash`:

```json
{
  "dispatch_id":         "<uuid>",
  "receipt_hash":        "0db54077...",
  "selected_candidate_id": "pilot-de-001",
  "verification_passed": true,
  "status":              "draft"
}
```

**Artifact produced:** dispatch record in `status: draft`.
**Files involved:** receipt from step 3, `case.json`, derived policy from step 1 response

---

### Step 5 — Reviewer approval

**Endpoint:** `POST /dispatch/{dispatch_id}/approve`

```json
{ "approved_by": "reviewer" }
```

The service transitions the dispatch from `draft` → `approved`. The
`approved_by` identity and `approved_at` timestamp are locked onto the record.
No further mutations to the approval fields are possible after this point.

**Artifact produced:** dispatch record in `status: approved`.

---

### Step 6 — Dispatch export

**Endpoint:** `GET /dispatch/{dispatch_id}/export`

The service transitions the dispatch to `status: exported` and returns the
canonical handoff packet:

```json
{
  "dispatch_id":           "<uuid>",
  "receipt_hash":          "0db54077...",
  "selected_candidate_id": "pilot-de-001",
  "verification_passed":   true,
  "status":                "exported"
}
```

The `receipt_hash` in the export packet is the chain-of-custody key. Any
downstream system receiving this packet can independently verify the routing
decision by replaying step 7.

**Artifact produced:** `export_packet.json` (written by `demo.sh` to the bundle directory).

---

### Step 7 — Receipt verification

**Endpoint:** `POST /verify`

```json
{
  "receipt": { ... },
  "case":    { ... },
  "policy":  { ... }
}
```

The kernel replays the routing decision from raw inputs — no stored state is
read. It recomputes every hash field in the receipt and compares against the
submitted values. All 21 fields must match.

Expected response:

```json
{ "result": "VERIFIED" }
```

**Files involved:** `expected_routed.json`, `case.json`, `derived_policy.json`
**Reference:** `expected_verify.json`

---

## Artifact map

```
case.json + registry_snapshot.json + config.json
    │
    ▼ POST /pilot/route-normalized
    │
routing kernel (postcad-routing-v1)
    │  filters registry → selects pilot-de-001
    │  commits 21 fields
    ▼
receipt  (receipt_hash = 0db54077...)       ← expected_routed.json
    │
    ▼ POST /dispatch/create  (inline re-verify)
    │
dispatch record  status: draft
    │
    ▼ POST /dispatch/{id}/approve
    │
dispatch record  status: approved
    │
    ▼ GET /dispatch/{id}/export
    │
export_packet.json  (receipt_hash = 0db54077...)
    │
    ▼ POST /verify  (independent replay from raw inputs)
    │
{ "result": "VERIFIED" }                   ← expected_verify.json
```

---

## Verification

### Why verification exists

The routing decision is committed to a receipt at the moment it is made.
Verification provides an independent check that the decision recorded in the
receipt is the decision the kernel would make when re-run on the same inputs.
This means any party holding the case, the policy, and the receipt can confirm
the routing outcome without trusting the platform or its stored state.

### What invariants are checked

The verify endpoint recomputes and compares all 21 committed fields. Key checks:

| Check | What a mismatch means |
|---|---|
| `routing_input_hash` | The input committed in the receipt does not match the supplied case/policy |
| `case_fingerprint` | The case fields were altered after routing |
| `policy_fingerprint` | The policy bundle was altered after routing |
| `registry_snapshot_hash` | The registry snapshot used for routing differs from what was supplied |
| `eligible_candidate_ids_hash` | The compliance filter would produce a different candidate set |
| `routing_decision_hash` | The selection result differs from what the kernel would choose |
| `receipt_hash` | The top-level seal is broken — any field in the receipt was modified |

Checks run in this order. The first mismatch stops the replay and returns a
stable error code (e.g. `case_fingerprint_mismatch`,
`registry_snapshot_hash_mismatch`).

### Why altering the receipt fails

`receipt_hash` is a SHA-256 over all 21 preceding fields in canonical
serialization order. Changing any field — even a single byte — changes the
`receipt_hash`. If an attacker also updates `receipt_hash` to match the altered
fields, the kernel replay recomputes `routing_decision_hash` from the original
inputs and finds it no longer matches the value in the (forged) receipt. There
is no way to produce a consistent altered receipt without re-running the kernel
on the original inputs — which produces the original receipt.
