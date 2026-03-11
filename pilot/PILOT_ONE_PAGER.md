# PostCAD v1 Pilot

**One-page technical brief for clinic, lab, and integration partners.**

---

## The problem

Dental manufacturing routing today is manual, opaque, and not auditable. Clinics select labs through informal processes. There is no machine-readable record of which manufacturer was selected, why, and under what regulatory constraints. Disputed cases have no independent evidence trail.

---

## What PostCAD v1 does

PostCAD is a deterministic, rule-based routing kernel. Given a dental case and a manufacturer registry, it:

1. Filters manufacturers by jurisdiction, capability, material, and attestation status
2. Selects a manufacturer using a deterministic algorithm — same inputs always produce the same selection
3. Records the decision in a tamper-evident receipt that any third party can independently verify

PostCAD does not own manufacturing, does not hold clinical liability, and does not use AI or probabilistic logic. Every decision is rule-based and carries a machine-readable reason code.

---

## What the pilot tests

The pilot answers three questions:

1. **Can PostCAD route real cases from a real registry?** — Given your manufacturer list and a representative set of dental cases, does the routing engine select correctly and refuse when it should?
2. **Are the receipts independently verifiable?** — Can a third party (clinic, regulator, audit system) verify a receipt without access to PostCAD's internals?
3. **Does the refusal behavior match your compliance expectations?** — Expired attestations, wrong jurisdiction, wrong capability — are these caught and coded correctly?

---

## What the partner provides

| Input | Format | Notes |
|---|---|---|
| Manufacturer registry | JSON array | One record per manufacturer. Fields: `manufacturer_id`, `country`, `is_active`, `capabilities`, `materials_supported`, `jurisdictions_served`, `attestation_statuses`. |
| Routing config | JSON object | `jurisdiction` + `routing_policy`. Two policy variants: `allow_domestic_only` or `allow_domestic_and_cross_border`. |
| Test cases | JSON objects | One object per case. Fields: `case_id`, `patient_country`, `manufacturer_country`, `material`, `procedure`, `file_type`. |

Full field definitions: `docs/pilot_contract_v1.md`.

---

## What PostCAD returns

**For every routed case:**

```json
{
  "outcome": "routed",
  "selected_candidate_id": "<manufacturer_id>",
  "receipt_hash": "<sha256>",
  "routing_decision_hash": "<sha256>",
  "registry_snapshot_hash": "<sha256>"
}
```

**For every refused case:**

```json
{
  "outcome": "refused",
  "refusal_code": "no_jurisdiction_match",
  "receipt_hash": "<sha256>"
}
```

Stable refusal codes: `no_active_manufacturer`, `no_jurisdiction_match`, `no_capability_match`, `no_material_match`, `attestation_failed`.

**For every verification call:**

```json
{"result": "VERIFIED"}
```

or exit 1 with a stable error code identifying exactly which field failed.

---

## Why the outputs are trustworthy

- **Deterministic:** same case + same registry = same `receipt_hash`, always. Two independent systems running PostCAD on the same inputs will produce byte-identical receipts.
- **Tamper-evident:** every committed field (21 fields including the full routing input, registry snapshot, candidate pool, and decision outcome) is covered by `receipt_hash`. Changing any field invalidates the hash.
- **Independently verifiable:** `verify-receipt` performs a full routing replay without trusting the receipt's content. A third party with the original inputs can verify any receipt without access to PostCAD's state.
- **Audit-chained:** every receipt carries `audit_seq`, `audit_entry_hash`, and `audit_previous_hash` — a SHA-256 hash-chained append-only log entry.
- **Frozen protocol:** the receipt schema (v1), kernel version (`postcad-routing-v1`), and all 22 stable error codes are locked. Receipts generated today will be verifiable by future versions.

Frozen conformance vectors with expected receipt hashes: `tests/protocol_vectors/` and `tests/protocol_verifier_vectors/`.

---

## What is out of scope in v1

- No persistent storage — receipts are returned in the response, not stored by PostCAD
- No authentication or access control
- No case lifecycle management (updates, cancellations, status tracking)
- No manufacturer onboarding API — registry is a read-only input per request
- No SLA enforcement — `sla_days` is stored but not used in routing decisions
- No batch routing — one case per call
- No AI or probabilistic logic of any kind

---

## What counts as pilot success

| Criterion | Pass condition |
|---|---|
| Routing accuracy | Routed cases select the expected manufacturer; refused cases carry the expected refusal code |
| Determinism | Running the same case twice produces the same `receipt_hash` |
| Verification | Every receipt returned by the pilot passes `verify-receipt` independently |
| Refusal coverage | All expected refusal scenarios (jurisdiction, capability, attestation) produce the correct stable code |
| Drift detection | A receipt verified against a modified registry snapshot fails with `registry_snapshot_hash_mismatch` |

---

## Technical entry points

| Surface | Command / endpoint |
|---|---|
| One-command demo | `cargo run -p postcad-cli -- demo-run --json` |
| Route a case (registry) | `POST /route-case-from-registry` |
| Verify a receipt | `POST /verify-receipt` |
| Protocol manifest | `GET /protocol-manifest` |
| Full pilot guide | `pilot/README.md` |
| Input/output contract | `docs/pilot_contract_v1.md` |
| Protocol v1 spec | `PROTOCOL_V1.md` |
