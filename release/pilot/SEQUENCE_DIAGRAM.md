# PostCAD — Pilot Sequence Diagram

End-to-end execution order for the canonical pilot flow.
See `PROTOCOL_WALKTHROUGH.md` for step-by-step explanation of each stage.

---

## Sequence

```
  Operator            Service / API         Routing Kernel        Dispatch Store        Reviewer Shell
     │                      │                      │                      │                      │
     │                      │                      │                      │                      │
     │  POST                │                      │                      │                      │
     │  /pilot/route-       │                      │                      │                      │
     │  normalized          │                      │                      │                      │
     │─────────────────────>│                      │                      │                      │
     │  {pilot_case,        │                      │                      │                      │
     │   registry_snapshot, │                      │                      │                      │
     │   routing_config}    │                      │                      │                      │
     │                      │  normalize to        │                      │                      │
     │                      │  8-field input       │                      │                      │
     │                      │─────────────────────>│                      │                      │
     │                      │                      │  apply compliance    │                      │
     │                      │                      │  rules to registry   │                      │
     │                      │                      │  → 3 eligible        │                      │
     │                      │                      │  candidates          │                      │
     │                      │                      │                      │                      │
     │                      │                      │  deterministic hash  │                      │
     │                      │                      │  select pilot-de-001 │                      │
     │                      │                      │                      │                      │
     │                      │                      │  commit 21 fields    │                      │
     │                      │                      │  seal receipt_hash   │                      │
     │                      │<─────────────────────│                      │                      │
     │                      │  {receipt,           │                      │                      │
     │<─────────────────────│   derived_policy}    │                      │                      │
     │                      │                      │                      │                      │
     │                      │                      │                      │                      │
     │  POST                │                      │                      │                      │
     │  /dispatch/create    │                      │                      │                      │
     │─────────────────────>│                      │                      │                      │
     │  {receipt,           │                      │                      │                      │
     │   case, policy}      │                      │                      │                      │
     │                      │  re-verify inline    │                      │                      │
     │                      │─────────────────────>│                      │                      │
     │                      │<─────────────────────│                      │                      │
     │                      │  VERIFIED            │                      │                      │
     │                      │                      │  create draft        │                      │
     │                      │─────────────────────────────────────────── >│                      │
     │                      │                      │                      │  record keyed by     │
     │                      │<──────────────────────────────────────────── │  receipt_hash        │
     │<─────────────────────│                      │                      │                      │
     │  {dispatch_id,       │                      │                      │                      │
     │   status: draft}     │                      │                      │                      │
     │                      │                      │                      │                      │
     │                      │                      │                      │                      │
     │                      │          GET /reviewer (load pending review) │                      │
     │                      │<──────────────────────────────────────────────────────────────────│
     │                      │  pending dispatches  │                      │                      │
     │                      │──────────────────────────────────────────────────────────────────>│
     │                      │                      │                      │   draft shown to     │
     │                      │                      │                      │   reviewer           │
     │                      │                      │                      │                      │
     │                      │  POST /dispatch/{id}/approve                │                      │
     │                      │<──────────────────────────────────────────────────────────────────│
     │                      │  {approved_by}       │                      │                      │
     │                      │                      │                      │  lock approved_by    │
     │                      │─────────────────────────────────────────── >│  + approved_at       │
     │                      │<──────────────────────────────────────────── │  status: approved    │
     │                      │  {status: approved}  │                      │                      │
     │                      │──────────────────────────────────────────────────────────────────>│
     │                      │                      │                      │                      │
     │                      │                      │                      │                      │
     │  GET                 │                      │                      │                      │
     │  /dispatch/{id}/     │                      │                      │                      │
     │  export              │                      │                      │                      │
     │─────────────────────>│                      │                      │                      │
     │                      │─────────────────────────────────────────── >│                      │
     │                      │<──────────────────────────────────────────── │  mark exported       │
     │<─────────────────────│                      │                      │                      │
     │  export_packet.json  │                      │                      │                      │
     │  {receipt_hash,      │                      │                      │                      │
     │   status: exported}  │                      │                      │                      │
     │                      │                      │                      │                      │
     │                      │                      │                      │                      │
     │  POST /verify        │                      │                      │                      │
     │  {receipt,           │                      │                      │                      │
     │   case, policy}      │                      │                      │                      │
     │─────────────────────>│                      │                      │                      │
     │                      │  replay from         │                      │                      │
     │                      │  raw inputs          │                      │                      │
     │                      │─────────────────────>│                      │                      │
     │                      │                      │  recompute all       │                      │
     │                      │                      │  21 hash fields      │                      │
     │                      │                      │  compare to receipt  │                      │
     │                      │<─────────────────────│                      │                      │
     │<─────────────────────│                      │                      │                      │
     │  {"result":          │                      │                      │                      │
     │   "VERIFIED"}        │                      │                      │                      │
     │                      │                      │                      │                      │
```

---

## Legend

| Actor | Role |
|---|---|
| **Operator** | Submits the pilot case, triggers export, runs independent verification |
| **Service / API** | HTTP layer; normalizes input, routes requests, enforces state transitions |
| **Routing Kernel** | `postcad-routing-v1`; stateless; applies compliance rules and selects candidate deterministically |
| **Dispatch Store** | Persistent dispatch records keyed by `receipt_hash`; enforces one dispatch per receipt |
| **Reviewer Shell** | Browser UI at `http://localhost:8080/reviewer`; human approval step |

---

## What each boundary proves

**What starts the flow:**
The operator submits a normalized 4-field case plus the manufacturer registry
and routing configuration to `POST /pilot/route-normalized`. Nothing executes
before this call.

**What the reviewer approves:**
A draft dispatch record that is already cryptographically bound to a verified
routing receipt. The reviewer approves the routing decision — not the input.
The receipt was verified twice before the reviewer sees it: once inside
`POST /pilot/route-normalized` (self-verify) and once inline during
`POST /dispatch/create`.

**What artifact is exported:**
A canonical JSON packet containing `dispatch_id`, `receipt_hash`,
`selected_candidate_id`, `verification_passed`, and `status: exported`. The
`receipt_hash` is the chain-of-custody key that links the export back to the
exact routing decision.

**What verification proves:**
`POST /verify` replays the routing decision from the original case, policy, and
registry snapshot — reading no stored state. It recomputes all 21 hash fields
and compares them against the submitted receipt. A `VERIFIED` result means the
routing decision recorded in the receipt is identical to what the kernel would
produce today on the same inputs.
