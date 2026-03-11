# PostCAD v1 Pilot Input/Output Contract

This document defines exactly what a pilot partner provides to PostCAD v1 and what PostCAD returns. It covers both integration paths (registry-backed and policy-bundle), all three outcome types (routed, refused, verified), and explicit out-of-scope items.

All behavior described here is deterministic and frozen. Same inputs → same receipt hash, every time.

See also: `PROTOCOL_V1.md` (full protocol spec), `pilot/README.md` (runnable examples).

---

## Integration paths

PostCAD v1 exposes two integration shapes. Both produce identical receipt artifacts and verification behavior.

| Path | Inputs | Use when |
|---|---|---|
| **Registry-backed** | case + registry snapshot + routing config | Partner maintains a typed manufacturer registry |
| **Policy-bundle** | case + pre-built RoutingPolicyBundle | Partner manages candidate/snapshot construction externally |

The registry-backed path is preferred for pilot. It derives candidates and snapshots from typed manufacturer records; no hand-crafting required.

---

## Input contract

### 1. Case input

Provided as a JSON object. All fields required unless marked optional.

| Field | Type | Required | Description |
|---|---|---|---|
| `case_id` | UUID string | optional | Stable case identifier. Determines deterministic candidate selection. Generated if absent. |
| `jurisdiction` | string | optional | Routing jurisdiction code (e.g. `"DE"`, `"US"`). Falls back to `"global"` if absent. |
| `routing_policy` | string | optional | `"allow_domestic_only"` or `"allow_domestic_and_cross_border"`. Falls back to `"allow_domestic_only"`. |
| `patient_country` | string | **required** | Country of the clinic/patient. |
| `manufacturer_country` | string | **required** | Required manufacturing country. |
| `material` | string | **required** | Dental material. Accepted values: `zirconia`, `pmma`, `emax`, `cobalt_chrome`, `titanium`, `other:<name>`. |
| `procedure` | string | **required** | Dental procedure type. Accepted values: `crown`, `bridge`, `veneer`, `implant`, `denture`, `other:<name>`. |
| `file_type` | string | **required** | CAD file format. Accepted values: `stl`, `obj`, `ply`, `three_mf`, `other:<name>`. |

Country accepted values: `germany`, `united_states`, `france`, `japan`, `united_kingdom`, `other:<name>`.

**Canonical example** (`fixtures/case.json`):
```json
{
  "case_id": "a1b2c3d4-0000-0000-0000-000000000001",
  "jurisdiction": "DE",
  "routing_policy": "allow_domestic_and_cross_border",
  "patient_country": "germany",
  "manufacturer_country": "germany",
  "material": "zirconia",
  "procedure": "crown",
  "file_type": "stl"
}
```

---

### 2. Registry snapshot input (registry-backed path)

Provided as a JSON array of manufacturer records. All fields required unless marked optional.

| Field | Type | Required | Description |
|---|---|---|---|
| `manufacturer_id` | string | **required** | Stable unique identifier for this manufacturer. |
| `display_name` | string | optional | Human-readable name. Not committed into the receipt. |
| `country` | string | **required** | Manufacturer's country. Same accepted values as case country fields. |
| `is_active` | boolean | **required** | Whether this manufacturer is currently active. |
| `capabilities` | string array | **required** | Procedures this manufacturer supports (same values as `procedure`). |
| `materials_supported` | string array | **required** | Materials this manufacturer supports (same values as `material`). |
| `jurisdictions_served` | string array | **required** | Jurisdiction codes this manufacturer operates in (e.g. `["germany"]`). |
| `attestation_statuses` | string array | **required** | Attestation state per certification. Accepted values: `verified`, `expired`, `revoked`, `rejected`. Manufacturer is eligible only if all entries are `verified` and the list is non-empty. |
| `sla_days` | integer | optional | Indicative turnaround days. Stored but not used in routing decisions. |

**Canonical example** (`tests/protocol_vectors/v01_basic_routing/registry_snapshot.json`):
```json
[
  {
    "manufacturer_id": "mfr-de-001",
    "display_name": "Alpha Dental GmbH",
    "country": "germany",
    "is_active": true,
    "capabilities": ["crown"],
    "materials_supported": ["zirconia"],
    "jurisdictions_served": ["germany"],
    "attestation_statuses": ["verified"],
    "sla_days": 5
  }
]
```

---

### 3. Routing config input (registry-backed path)

Provided as a JSON object. All fields optional; absent values fall back to the case fields.

| Field | Type | Required | Description |
|---|---|---|---|
| `jurisdiction` | string | optional | Jurisdiction code for manufacturer filtering. Falls back to case `jurisdiction`. |
| `routing_policy` | string | optional | Policy variant. Falls back to case `routing_policy`. |
| `policy_version` | string | optional | Version label committed into the receipt proof (e.g. `"v1"`, `"2024-01"`). |

**Canonical example** (`tests/protocol_vectors/v01_basic_routing/policy.json`):
```json
{
  "jurisdiction": "DE",
  "routing_policy": "allow_domestic_and_cross_border"
}
```

---

### 4. Policy bundle input (policy-bundle path)

Provided as a JSON object with pre-built candidate and snapshot arrays. Used when the caller manages candidate construction externally.

| Field | Type | Required | Description |
|---|---|---|---|
| `jurisdiction` | string | **required** | Routing jurisdiction code. |
| `routing_policy` | string | **required** | Policy variant. |
| `candidates` | array | **required** | List of routing candidates (see below). |
| `snapshots` | array | **required** | Compliance snapshots corresponding to candidates (see below). |
| `policy_version` | string | optional | Version label committed into the receipt proof. |

Each **candidate** entry:

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | string | **required** | Stable candidate identifier. |
| `manufacturer_id` | string | **required** | Linked manufacturer identifier. |
| `location` | string | **required** | `"domestic"` or `"cross_border"`. |
| `accepts_case` | boolean | **required** | Whether this candidate can handle the case procedure and material. |
| `eligibility` | string | **required** | `"eligible"` or `"ineligible"`. |

Each **snapshot** entry:

| Field | Type | Required | Description |
|---|---|---|---|
| `manufacturer_id` | string | **required** | Must match a candidate's `manufacturer_id`. |
| `evidence_references` | string array | **required** | Certification evidence identifiers (e.g. `["ISO-9001-2024"]`). |
| `attestation_statuses` | string array | **required** | Attestation state strings. |
| `is_eligible` | boolean | **required** | Composite eligibility flag. |

**Canonical example** (`fixtures/policy.json`): see `fixtures/policy.json`.

---

## Output contract

### Routed receipt

When a manufacturer is selected, PostCAD returns a `RoutingReceipt` JSON object with `"outcome": "routed"`.

Key output fields:

| Field | Type | Description |
|---|---|---|
| `schema_version` | string | Always `"1"` for v1. |
| `routing_kernel_version` | string | Always `"postcad-routing-v1"` for v1. |
| `outcome` | string | `"routed"` |
| `selected_candidate_id` | string | The ID of the selected manufacturer/candidate. |
| `refusal_code` | null | Always null for routed outcomes. |
| `receipt_hash` | string | SHA-256 of the canonical receipt body. Primary tamper-evidence field. |
| `routing_decision_hash` | string | Commits to outcome, selected_candidate_id, refusal_code, policy_version, kernel_version. |
| `registry_snapshot_hash` | string | Commits to the full registry snapshot used at routing time. |
| `candidate_pool_hash` | string | Commits to the complete candidate universe. |
| `case_fingerprint` | string | SHA-256 of the canonical case payload. |
| `policy_fingerprint` | string | SHA-256 of the canonical policy configuration. |
| `routing_input` | object | Full routing input envelope (all fields that determined this decision). |
| `routing_input_hash` | string | SHA-256 of `routing_input`. |
| `audit_seq` | integer | Sequence number in the in-process audit log. |
| `audit_entry_hash` | string | SHA-256 of this audit log entry. |
| `audit_previous_hash` | string | Hash of the preceding audit entry (genesis: 64 zeros). |

All hash fields are lowercase hex SHA-256 digests (64 characters).

**Canonical example** (`fixtures/expected_routed.json`):
```
"outcome":              "routed"
"selected_candidate_id": "rc-de-01"
"receipt_hash":         "337f858244b2abb780a50a39774ba4ba44168571310785040b97977f59e7f036"
```

---

### Refused artifact

When no manufacturer can be selected, PostCAD returns a `RoutingReceipt` with `"outcome": "refused"`. The refused receipt is structurally identical to the routed receipt; the additional fields are:

| Field | Type | Description |
|---|---|---|
| `outcome` | string | `"refused"` |
| `selected_candidate_id` | null | Always null for refused outcomes. |
| `refusal_code` | string | Machine-readable refusal reason. See refusal codes below. |
| `refusal` | object | Present only on refused outcomes. Contains `message`, `evaluated_candidate_ids`, `failed_constraint`. |

**Refusal codes** (stable, permanent identifiers):

| Code | Condition |
|---|---|
| `no_eligible_manufacturer` | Registry was empty |
| `no_active_manufacturer` | All records have `is_active: false` |
| `no_jurisdiction_match` | No record serves the required jurisdiction |
| `no_capability_match` | No record supports the required procedure |
| `no_material_match` | No record supports the required material |
| `attestation_failed` | All surviving records have invalid attestations |
| `no_eligible_candidates` | Compliance gate rejected all candidates (policy-bundle path) |

**Refused receipts are also verifiable.** Pass a refused receipt to `verify-receipt` with the same inputs; it must return `VERIFIED`.

**Canonical example** (`fixtures/expected_refused.json`):
```
"outcome":      "refused"
"refusal_code": "no_eligible_candidates"
"receipt_hash": "bd3b97dc5efddff25fd77f2bff35641710a923e383b612cad65defaaf81eb1b9"
```

---

### Verify result

`verify-receipt` accepts a receipt + original inputs and performs a full routing replay. It checks every committed field in sequence (13 steps; see `PROTOCOL_V1.md` for the full table).

**Success**:
```json
{"result": "VERIFIED"}
```
Exit code: `0`

**Failure**:
```json
{
  "result": "VERIFICATION FAILED",
  "code": "<stable_error_code>",
  "reason": "<human-readable message>"
}
```
Exit code: `1`

The `code` field is always a stable identifier from the 22-code list in `PROTOCOL_V1.md`. The `reason` string is informational and may change.

---

### Proof object (`RoutingProofObject`)

A 11-field projection of the receipt's committed hashes, useful for third-party verification without access to the full kernel.

| Field | Description |
|---|---|
| `protocol_version` | `"postcad-v1"` |
| `receipt_hash` | Primary tamper-evidence field |
| `routing_decision_hash` | Commits to outcome + selected candidate |
| `registry_snapshot_hash` | Commits to registry state |
| `candidate_pool_hash` | Commits to candidate universe |
| `candidate_order_hash` | Commits to selector input ordering |
| `routing_input_hash` | Commits to all routing inputs |
| `routing_kernel_version` | Kernel algorithm identifier |
| `audit_entry_hash` | Audit log entry hash |
| `audit_previous_hash` | Preceding audit entry hash |
| `selected_candidate_id` | Null or selected ID |

Produced by `build_routing_proof(receipt)` via the Rust library.

---

## Canonical fixture mapping

| Scenario | Input files | Expected output |
|---|---|---|
| Routed | `fixtures/case.json` + `fixtures/candidates.json` + `fixtures/snapshot.json` | `fixtures/expected_routed.json` |
| Refused | `fixtures/case.json` + `fixtures/candidates.json` + `fixtures/snapshot_refusal.json` | `fixtures/expected_refused.json` |
| Verify routed | `fixtures/expected_routed.json` + `fixtures/case.json` + `fixtures/policy.json` + `fixtures/candidates.json` | `{"result":"VERIFIED"}` |
| Registry-routed | `tests/protocol_vectors/v01_basic_routing/case.json` + `registry_snapshot.json` + `policy.json` | routed, `mfr-de-001` selected |
| Registry-refused | `tests/protocol_vectors/v03_jurisdiction_refusal/…` | refused, `no_jurisdiction_match` |

---

## HTTP service contract

The service exposes the same contract over HTTP. All endpoints accept and return JSON.

| Endpoint | Method | Request body | Success response |
|---|---|---|---|
| `/route-case-from-registry` | POST | `{"case": {...}, "registry": [...], "config": {...}}` | `{"receipt": {...}, "derived_policy": {...}}` |
| `/route-case` | POST | `{"case": {...}, "policy": {...}}` | `{"receipt": {...}}` |
| `/verify-receipt` | POST | `{"receipt": {...}, "case": {...}, "policy": {...}}` | `{"result": "VERIFIED"}` |
| `/protocol-manifest` | GET | — | `ProtocolManifest` object |
| `/cases` | POST | case JSON object | `{"case_id": "...", "stored": true}` |
| `/cases` | GET | — | `{"case_ids": [...]}` |
| `/cases/{case_id}` | GET | — | stored case JSON object |

Error responses: HTTP 422, body `{"error": {"code": "...", "message": "..."}}` for routing errors; `{"result": "FAILED", "error": {"code": "...", "message": "..."}}` for verification failures.

### Case intake curl examples

```bash
# POST /cases — store a case
curl -s -X POST http://localhost:3000/cases \
  -H 'Content-Type: application/json' \
  -d @examples/pilot/case.json

# GET /cases — list stored case IDs
curl -s http://localhost:3000/cases

# GET /cases/{case_id} — retrieve a stored case
curl -s http://localhost:3000/cases/a1b2c3d4-0000-0000-0000-000000000001
```

---

## Out of scope for pilot v1

The following are explicitly not supported:

- **Persistent storage** — no database; audit log is in-memory per request.
- **Authentication / authorisation** — no auth layer on any endpoint.
- **Case status tracking** — no lifecycle management, no updates, no cancellations.
- **Manufacturer registry writes** — registry is read-only input per request; no onboarding API.
- **Multi-tenancy** — single registry, single policy per call.
- **SLA enforcement** — `sla_days` is stored in records but not used in routing decisions.
- **Batch routing** — one case per call only.
- **Receipt persistence** — receipts are returned in the response only; not stored by PostCAD.
- **Cross-version receipt migration** — receipts with `schema_version != "1"` are rejected outright.
- **Webhooks / async callbacks** — synchronous request/response only.
- **AI or probabilistic routing** — all decisions are rule-based and deterministic.
