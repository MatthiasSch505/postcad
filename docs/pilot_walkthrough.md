# PostCAD Pilot Walkthrough

**Protocol:** `postcad-v1` · **Kernel:** `postcad-routing-v1`

---

## Purpose

This document walks through one complete PostCAD protocol flow:
route a dental case → verify the receipt → commit a dispatch → approve it → export the handoff packet.

It is written for a technical mentor, lab operator, or pilot partner who wants to understand what the system does at each step and what guarantees it provides.

---

## Input

| Field | Value |
|---|---|
| `case_id` | `a1b2c3d4-0000-0000-0000-000000000001` |
| `material` | `zirconia` |
| `procedure` | `crown` |
| `jurisdiction` | `DE` |
| `routing_policy` | `allow_domestic_and_cross_border` |
| Candidate pool | `rc-de-01` (backed by `mfr-de-01`) |
| Compliance snapshot | `is_eligible: true`, attestation verified |

---

## Step-by-Step Flow

### 1 — Route

The routing kernel receives the case and policy bundle, evaluates all compliance rules
(manufacturer active, capability, EU MDR certification, ISO 13485), and selects a candidate.
Every decision is wrapped in a `Decision<T>` carrying a `ReasonCode` and timestamp.
The result is a `RoutingReceipt` with 21 committed fields.

```
POST /cases/:id/route
→ outcome: "routed"
  selected_candidate_id: "rc-de-01"
  receipt_hash: "337f858244b2abb780a50a39774ba4ba44168571310785040b97977f59e7f036"
```

The receipt locks in: case fingerprint, policy fingerprint, registry snapshot hash,
candidate pool hash, eligible candidate IDs hash, and a chained audit entry hash.
The routing decision is deterministic — the same inputs always produce the same receipt.

---

### 2 — Verify

The operator replays the routing decision against the original inputs.
The service recomputes every hash in the receipt and checks all fields match.
No external state. No trust required.

```
POST /dispatch/:hash/verify   (legacy path)
→ result: "VERIFIED"
```

Verification fails if any input — case, policy, registry snapshot, or selected candidate — has changed.

---

### 3 — Create Dispatch

A dispatch record is created from the verified receipt. The service re-runs verification inline
before accepting the dispatch. If verification fails, the request is rejected (422).
Only one dispatch is allowed per `receipt_hash`.

```
POST /dispatch/create
  body: { receipt, case, policy }
→ dispatch_id: <uuid>
  status: "draft"
  receipt_hash: <bound to exact receipt>
  verification_passed: true
```

The `receipt_hash` binding is the chain of custody link between the dispatch and the exact
routing decision that was verified.

---

### 4 — Approve Dispatch

An operator approves the draft dispatch. The approval records operator identity and timestamp.
Once approved, the routing and receipt fields are immutable — only `status` advances.
A second approval attempt returns 409 (`dispatch_not_draft`).

```
POST /dispatch/:dispatch_id/approve
  body: { "approved_by": "operator-id" }
→ status: "approved"
  approved_by: "operator-id"
  approved_at: <ISO-8601>
```

---

### 5 — Export Dispatch Packet

The approved dispatch is exported as a deterministic canonical JSON packet.
The 10 fields are serialized in fixed alphabetical order — the same export byte-for-byte
every time for the same approved dispatch.
Attempting to export a draft returns 422 (`dispatch_not_approved`).

```
GET /dispatch/:dispatch_id/export
→ status: "exported"
  { approved_at, approved_by, case_id, created_at, dispatch_id,
    manufacturer_payload_json, receipt_hash, selected_candidate_id,
    status, verification_passed }
```

This packet is the handoff artifact. It carries the manufacturer identity, the receipt hash,
and the approval record. Nothing in it can be changed after export.

---

## What the System Proves

| Guarantee | Mechanism |
|---|---|
| Routing is deterministic | Same inputs → same receipt, always; kernel is stateless |
| Receipt integrity | 21 hash-committed fields; any tamper breaks `receipt_hash` |
| Independent verifiability | `verify_receipt` replays from raw inputs; no trust in stored state |
| Dispatch binds to exact receipt | `receipt_hash` field; duplicate dispatch rejected |
| Approval is recorded and immutable | `approved_by` + `approved_at` locked at approval; no further mutation |
| Audit trail | Hash-chained `AuditEntry` log; `verify_chain()` recomputes every link |
| Every decision carries a reason | All outcomes wrapped in `Decision<T>` with `ReasonCode` |

---

## Responsibility Boundary

PostCAD provides:
- deterministic routing against a compliance-checked candidate pool
- cryptographic proof that the receipt was not modified
- a dispatch commitment layer that binds approval to the exact verified receipt
- a deterministic export packet for handoff

PostCAD does **not**:
- manufacture anything
- make clinical decisions
- own downstream quality or liability
- apply AI or probabilistic judgment — every rule is stateless and deterministic

The manufacturer identified in the export packet is responsible for production.
The clinical team is responsible for the treatment plan.
PostCAD's role ends at the export step.
