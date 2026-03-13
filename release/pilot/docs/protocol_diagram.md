# PostCAD Protocol — One-Page Mental Model

**Protocol:** `postcad-v1` · **Kernel:** `postcad-routing-v1`

---

## Core Flow

```
  Case Input
  (case_id, material, procedure, jurisdiction, routing_policy)
        │
        ▼
  ┌─────────────┐
  │   ROUTE     │  compliance rules evaluated (EU MDR, FDA, ISO 13485, …)
  │             │  candidate pool filtered → winner selected deterministically
  └──────┬──────┘
         │  RoutingReceipt (21 hash-committed fields)
         │  receipt_hash = SHA-256 of full canonical receipt
         ▼
  ┌─────────────┐
  │   VERIFY    │  replay from raw inputs — no stored state trusted
  │             │  every hash recomputed; mismatch → hard failure
  └──────┬──────┘
         │  result: VERIFIED
         ▼
  ┌──────────────────┐
  │  CREATE DISPATCH │  verification re-run inline before record is written
  │  (status: draft) │  receipt_hash bound to dispatch — one dispatch per receipt
  └──────┬───────────┘
         │
         ▼
  ┌──────────────────┐
  │ APPROVE DISPATCH │  operator identity + timestamp recorded
  │ (status: approved│  routing/receipt fields become immutable
  └──────┬───────────┘
         │
         ▼
  ┌──────────────────┐
  │  EXPORT PACKET   │  deterministic canonical JSON (10 fields, fixed order)
  │ (status: exported│  handoff artifact — same bytes every time for same dispatch
  └──────────────────┘
         │
         ▼
    Manufacturer receives export packet
```

---

## Guarantees

| Invariant | How it holds |
|---|---|
| Routing is deterministic | Stateless kernel — same inputs always produce the same receipt |
| Receipt cannot be silently modified | `receipt_hash` covers all 21 fields; any change breaks verification |
| Verification requires no trust | Replays from raw inputs; does not read stored routing state |
| Dispatch binds to exact receipt | `receipt_hash` is the chain-of-custody key; duplicates rejected (409) |
| Approval is recorded and final | `approved_by` + `approved_at` locked on approval; no further mutation |
| Audit trail is tamper-evident | Hash-chained `AuditEntry` log; `verify_chain()` recomputes every link |
| Every decision carries a reason | All outcomes wrapped in `Decision<T>` with a `ReasonCode` — no silent pass/fail |

---

## Responsibility Boundary

```
  ┌────────────────────────────────────────────────────────┐
  │                    PostCAD                             │
  │  route · prove · verify · dispatch · approve · export  │
  └────────────────────────────────────────────────────────┘
            │                            │
            │                            │
  ┌─────────▼──────────┐      ┌──────────▼───────────┐
  │   Manufacturer     │      │   Clinical team       │
  │  (production)      │      │  (treatment plan)     │
  └────────────────────┘      └──────────────────────┘
```

PostCAD owns the routing decision, the cryptographic proof, and the handoff packet.
It does **not** manufacture, does **not** make clinical decisions, and does **not** apply
probabilistic or AI-based judgment. Every rule is stateless and deterministic.
Liability for production and clinical outcomes sits outside PostCAD entirely.
