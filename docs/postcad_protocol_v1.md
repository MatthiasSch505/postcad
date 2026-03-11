# PostCAD Protocol v1 Specification

**Version:** 1.0
**Status:** Frozen Protocol
**Reference Implementation:** PostCAD
**Protocol Label:** `postcad-v1`
**Routing Kernel Label:** `postcad-routing-v1`
**Receipt Schema Version:** `1`

---

## Table of Contents

1. [Protocol Overview](#1-protocol-overview)
2. [System Model](#2-system-model)
3. [Routing Inputs](#3-routing-inputs)
4. [Registry Snapshot Format](#4-registry-snapshot-format)
5. [Routing Decision Semantics](#5-routing-decision-semantics)
6. [Receipt Structure](#6-receipt-structure)
7. [Proof Object](#7-proof-object)
8. [Refusal Semantics](#8-refusal-semantics)
9. [Verification Algorithm](#9-verification-algorithm)
10. [Protocol Manifest](#10-protocol-manifest)
11. [Stable Error Codes](#11-stable-error-codes)
12. [Determinism Guarantee](#12-determinism-guarantee)
13. [Security Model](#13-security-model)
14. [Versioning Rules](#14-versioning-rules)

---

## 1. Protocol Overview

PostCAD Protocol v1 is a **deterministic manufacturing routing protocol** for dental cases. Given a case, a registry snapshot, and a routing policy, the protocol selects an allowed manufacturing path and produces a cryptographically verifiable routing receipt. Any party holding the original inputs can independently replay the routing decision and confirm the receipt is authentic.

PostCAD **is**:

- A deterministic routing protocol that selects from an allowed candidate set.
- A verifiable-receipt protocol: every routing outcome is accompanied by a receipt that commits to all inputs and the decision.
- A replay verification protocol: receipts can be independently verified by re-running the routing kernel against the original inputs.

PostCAD **is not**:

- A marketplace. It does not negotiate prices or connect buyers to sellers.
- A pricing engine. Economic considerations are outside the protocol.
- An optimization engine. Routing occurs only within the allowed candidate set defined by policy and compliance. No global optimum is sought.

Routing only occurs inside the set of candidates that pass all applicable compliance and policy filters.

---

## 2. System Model

### 2.1 Actors

| Actor | Role |
|---|---|
| **Case Producer** | Submits a dental case with patient and manufacturing parameters. |
| **Registry Authority** | Maintains the registry snapshot: the list of manufacturing candidates with their capabilities, jurisdictions, and attestation statuses. |
| **Routing Engine** | Accepts case + registry snapshot + policy; applies compliance and policy filters; deterministically selects a candidate or produces a refusal; emits a receipt and proof object. |
| **Verifier** | Holds the original inputs and a receipt; replays the routing kernel and confirms the receipt is authentic. |

### 2.2 Data Flow

```
Case Input
Registry Snapshot    ──▶  Routing Engine  ──▶  Receipt + Proof Object
Policy Bundle

                                               │
                                               ▼
                                          Verifier
                                          (replay same inputs,
                                           confirm receipt is authentic)
```

Verification does not require network access or any state beyond the original three inputs and the receipt.

---

## 3. Routing Inputs

Every routing call requires exactly three inputs. All three are also required for verification.

### 3.1 Case Input

Represents a single dental case to be routed.

| Field | Type | Description |
|---|---|---|
| `case_id` | string (UUID) | Optional. A stable identifier for the case. Generated if absent. |
| `patient_country` | string | Country of the patient / clinic. |
| `manufacturer_country` | string | Requested manufacturing country. |
| `material` | string | Dental material (e.g. `zirconia`, `pmma`, `titanium`). |
| `procedure` | string | Dental procedure type (e.g. `crown`, `bridge`, `implant`). |
| `file_type` | string | CAD file format (e.g. `stl`, `obj`, `three_mf`). |
| `jurisdiction` | string | Optional. Routing jurisdiction (e.g. `DE`, `global`). Default: `global`. |
| `routing_policy` | string | Optional. Policy variant: `allow_domestic_only` or `allow_domestic_and_cross_border`. |

### 3.2 Registry Snapshot

A list of manufacturing candidates with their current attributes. See [Section 4](#4-registry-snapshot-format).

### 3.3 Policy Bundle

Routing configuration that controls which candidates are allowed.

| Field | Type | Description |
|---|---|---|
| `jurisdiction` | string | Jurisdiction code for compliance rule selection. |
| `routing_policy` | string | Policy variant controlling cross-border routing. |
| `policy_version` | string (optional) | Version label for the policy ruleset. Committed into the receipt. |
| `candidates` | array | Pre-derived candidate entries (used in the policy-bundle routing path). |
| `snapshots` | array | Compliance snapshots for each candidate. |
| `refusal_reason_hint` | string (optional) | Explicit refusal code override for the no-candidates case. |

---

## 4. Registry Snapshot Format

A registry snapshot is an ordered list of `ManufacturerRecord` objects. Each record describes a single manufacturing entity and its current capabilities and compliance state.

### 4.1 ManufacturerRecord Fields

| Field | Type | Description |
|---|---|---|
| `manufacturer_id` | string | Stable unique identifier for this manufacturer. |
| `display_name` | string | Human-readable name. |
| `country` | string | Country where manufacturing takes place. |
| `is_active` | bool | Whether this manufacturer is currently accepting cases. |
| `capabilities` | string[] | Procedure types this manufacturer can produce (e.g. `crown`, `implant`). |
| `materials_supported` | string[] | Materials this manufacturer can process. |
| `jurisdictions_served` | string[] | Countries / regions this manufacturer is authorized to serve. |
| `attestation_statuses` | string[] | Compliance attestation states (e.g. `verified`, `pending`). |
| `sla_days` | integer | Nominal turnaround time in business days. |

### 4.2 Candidate Derivation

When routing via the registry path (`route-case-from-registry`), the engine derives the candidate list and compliance snapshots directly from the registry snapshot. Each record becomes a routing candidate; `is_eligible` is set by the compliance gate based on the record's `attestation_statuses`.

The derived candidate list and snapshot are committed into the receipt via `registry_snapshot_hash` and `candidate_pool_hash`. A deterministic policy bundle (`derived_policy`) is also returned alongside the receipt, enabling verification without any separate policy file.

---

## 5. Routing Decision Semantics

### 5.1 Candidate Derivation

The routing engine accepts a candidate list and a set of compliance snapshots. Each candidate is paired with its snapshot. The engine evaluates each pair through the compliance gate.

### 5.2 Compliance Filtering

The compliance gate evaluates each candidate against its snapshot. A candidate is **eligible** if and only if its snapshot's `is_eligible` flag is `true`. Ineligible candidates are excluded from the routing pool. The set of eligible candidate IDs is recorded in the receipt.

### 5.3 Policy Filtering

After compliance filtering, the routing policy is applied:

- `allow_domestic_only`: only candidates whose location is `domestic` are admitted.
- `allow_domestic_and_cross_border`: all eligible candidates are admitted.

### 5.4 Deterministic Selection

After policy filtering, if one or more candidates remain, the routing engine applies the deterministic selection algorithm (`postcad-routing-v1`):

1. Sort candidates by `(priority, id)` in ascending order to produce a stable, canonical ordering.
2. Apply the configured `RoutingStrategy`:
   - `HighestPriority`: select the first candidate (index 0).
   - `DeterministicHash`: hash the `case_id` to derive an index, distributing load without state.

The selected candidate ID and the outcome (`"routed"`) are recorded in the receipt.

### 5.5 Deterministic Refusal

If no candidates remain after all filters, the routing engine emits a refusal outcome. The refusal code is assigned by stepwise inspection of the filter chain (see [Section 8](#8-refusal-semantics)). The refusal code is recorded in the receipt as `refusal_code`. `selected_candidate_id` is `null`.

The refusal is a valid protocol outcome, not an error. The receipt for a refusal carries the same commitment structure as a routed receipt and is fully verifiable.

---

## 6. Receipt Structure

A routing receipt is a JSON object produced by the routing engine after every routing call. It commits to all inputs and the decision via a chain of SHA-256 hashes.

### 6.1 Fields

| Field | Type | Description |
|---|---|---|
| `schema_version` | string | Receipt schema version. Always `"1"` for v1 receipts. |
| `routing_kernel_version` | string | Routing kernel label. Always `"postcad-routing-v1"` for v1. |
| `outcome` | string | `"routed"` or `"refused"`. |
| `selected_candidate_id` | string \| null | ID of the selected candidate. `null` when refused. |
| `refusal_code` | string \| null | Stable refusal code. `null` when routed. |
| `case_fingerprint` | string | SHA-256 of the canonical case payload. |
| `policy_fingerprint` | string | SHA-256 of the canonical policy configuration. |
| `policy_version` | string \| null | Version label from the policy bundle. `null` if not declared. |
| `routing_proof_hash` | string | SHA-256 of the canonical routing decision fingerprint (covers all decision inputs). |
| `registry_snapshot_hash` | string | SHA-256 of the canonical registry snapshot. |
| `candidate_pool_hash` | string | SHA-256 of the full input candidate universe, order-independent. |
| `eligible_candidate_ids_hash` | string | SHA-256 of eligible candidate IDs after compliance and policy filtering, sorted. |
| `selection_input_candidate_ids_hash` | string | SHA-256 of the ordered candidate ID list as presented to the selector (order-sensitive). |
| `candidate_order_hash` | string | SHA-256 of the eligible candidate IDs in canonical ascending sort order. |
| `routing_decision_hash` | string | SHA-256 of the canonical decision fields (`outcome`, `refusal_code`, `policy_version`, `routing_kernel_version`, `selected_candidate_id`). |
| `audit_seq` | integer | Sequence number of this entry in the append-only audit log. |
| `audit_entry_hash` | string | SHA-256 of `{audit_seq, event, audit_previous_hash}`. |
| `audit_previous_hash` | string | Hash of the preceding audit entry. Genesis entry uses 64 zero hex digits. |
| `routing_input` | object | Full routing input envelope (see below). Stored inline for verifier self-sufficiency. |
| `routing_input_hash` | string | SHA-256 of `canonical_json(routing_input)`. |
| `receipt_hash` | string | SHA-256 of the canonical receipt content (all fields except `receipt_hash` itself). |
| `refusal` | object \| absent | Detailed refusal context. Present only when `outcome == "refused"`. |

### 6.2 Routing Input Envelope

The `routing_input` sub-object is stored inline in the receipt. It captures the exact inputs that determined the routing decision.

| Field | Type |
|---|---|
| `case_id` | string |
| `file_type` | string |
| `jurisdiction` | string |
| `manufacturer_country` | string |
| `material` | string |
| `patient_country` | string |
| `procedure` | string |
| `routing_policy` | string |

### 6.3 Commitment Structure

All hash fields are lowercase hexadecimal SHA-256 digests (64 characters).

The canonical serialization rule for all hash inputs is:

> SHA-256 of the compact JSON representation with keys sorted alphabetically, UTF-8 encoded, no trailing whitespace.

`receipt_hash` covers all fields in the receipt except `receipt_hash` itself. It is computed last during receipt generation and verified first during verification (after `schema_version`).

### 6.4 Frozen Receipt Schema Hash

The SHA-256 of the committed receipt field list (alphabetically sorted field names, joined with `\n`) is:

```
receipt_schema_hash = 37a025b6cb167fbec020d6ea5e64ac9fcc2da7ca9e7aa5bf28272f3114c9fb49
```

This value is stable for Protocol v1. Any change to the committed field set requires a protocol version increment.

---

## 7. Proof Object

A routing proof object is a structured projection of the receipt's commitment fields. It is derived deterministically from a receipt and provides a compact artifact for third-party verification without requiring the full receipt JSON.

### 7.1 Proof Fields

| Field | Type | Description |
|---|---|---|
| `protocol_version` | string | Protocol label. Always `"postcad-v1"` for v1. |
| `routing_kernel_version` | string | Kernel label. Always `"postcad-routing-v1"` for v1. |
| `routing_input_hash` | string | From `receipt.routing_input_hash`. |
| `registry_snapshot_hash` | string | From `receipt.registry_snapshot_hash`. |
| `candidate_pool_hash` | string | From `receipt.candidate_pool_hash`. |
| `candidate_order_hash` | string | From `receipt.candidate_order_hash`. |
| `routing_decision_hash` | string | From `receipt.routing_decision_hash`. |
| `selected_candidate_id` | string \| null | From `receipt.selected_candidate_id`. |
| `receipt_hash` | string | From `receipt.receipt_hash`. Top-level tamper seal. |
| `audit_entry_hash` | string | From `receipt.audit_entry_hash`. |
| `audit_previous_hash` | string | From `receipt.audit_previous_hash`. |

### 7.2 Proof Verification

To verify a proof object against a receipt:

1. Assert `proof.protocol_version == "postcad-v1"`. Error: `proof_protocol_version_mismatch`.
2. For each hash field in the proof, assert `proof.<field> == receipt.<field>`. Error: `proof_field_mismatch`.

### 7.3 Frozen Proof Schema Hash

The SHA-256 of the proof object field list (alphabetically sorted field names, joined with `\n`) is:

```
proof_schema_hash = ebd5a82f9e64d3151d0e2df43585f244abaae664213a9df46a633277b52507e4
```

---

## 8. Refusal Semantics

When no candidate survives all routing filters, the engine emits a refusal receipt. The `refusal_code` is assigned by stepwise inspection of the filter pipeline, in priority order:

| Priority | Code | Condition |
|---|---|---|
| 1 | `no_eligible_manufacturer` | Registry is empty or cause is unknown. Fallback. |
| 2 | `no_active_manufacturer` | All records have `is_active == false`. |
| 3 | `no_jurisdiction_match` | No active record serves the requested jurisdiction. |
| 4 | `no_capability_match` | No active, jurisdiction-matched record has the required procedure capability. |
| 5 | `no_material_match` | No active, jurisdiction-matched, capability-matched record supports the required material. |
| 6 | `attestation_failed` | Records passed all structural filters but all have `is_eligible == false`. |
| — | `no_eligible_candidates` | Legacy fallback code retained for backwards compatibility. |

The first condition that applies determines the refusal code. The code is deterministic: the same inputs always produce the same refusal code.

Refusal codes are frozen for Protocol v1. The SHA-256 of the canonical refusal code set (codes joined with `\n`, alphabetically sorted, no trailing newline) is:

```
refusal_code_set_hash = 4ebf952c33509c9d67850dbd4e43a2fe9a8b0174f4d3009f5c247a4928cb50df
```

---

## 9. Verification Algorithm

Verification takes the same three inputs used at routing time (case, registry snapshot / policy bundle, and the receipt to verify), re-runs the routing kernel, and confirms the receipt is authentic.

**Input:** `receipt_json`, `case_json`, `policy_json`
**Output:** `Ok(())` or a `VerificationFailure` with a stable error code.

### Step 0 — Parse and Schema Check

1. Parse `receipt_json` as raw JSON. Error: `receipt_parse_failed`.
2. Check `schema_version` field. Errors: `missing_receipt_schema_version`, `invalid_receipt_schema_version`, `unsupported_receipt_schema_version`.

### Step 1 — Tamper-Seal Checks (self-contained)

**Step 1b — Receipt hash:**
Recompute `SHA-256(canonical_json(receipt_without_receipt_hash))` and compare to `receipt.receipt_hash`.
Error: `receipt_canonicalization_mismatch`.

**Step 1c — Routing input hash:**
Recompute `SHA-256(canonical_json(receipt.routing_input))` and compare to `receipt.routing_input_hash`.
Error: `routing_input_hash_mismatch`.

**Step 1d — Kernel version:**
Assert `receipt.routing_kernel_version == "postcad-routing-v1"`.
Error: `routing_kernel_version_mismatch`.

**Step 1e — Decision hash:**
Recompute `SHA-256(canonical_json({outcome, refusal_code, policy_version, routing_kernel_version, selected_candidate_id}))` and compare to `receipt.routing_decision_hash`.
Error: `routing_decision_hash_mismatch`.

### Step 2 — Case Fingerprint

Parse `case_json`. Recompute `case_fingerprint` and compare to `receipt.case_fingerprint`.
Error: `case_parse_failed`, `case_fingerprint_mismatch`.

### Step 3 — Policy Fingerprint and Registry Snapshot

Parse `policy_json`. Recompute `policy_fingerprint` and compare to `receipt.policy_fingerprint`.
Errors: `policy_bundle_parse_failed`, `policy_fingerprint_mismatch`.

**Step 3c — Policy version:**
Assert `policy_input.policy_version == receipt.policy_version`.
Error: `policy_version_mismatch`.

**Step 3d — Registry snapshot hash:**
Recompute `hash_registry_snapshots(snapshots)` from the provided policy bundle's snapshot data and compare to `receipt.registry_snapshot_hash`.
Error: `registry_snapshot_hash_mismatch`.

**Step 3e — Candidate pool hash:**
Recompute the compliance-gated candidate pool from the snapshot and compare to `receipt.candidate_pool_hash`.
Error: `candidate_pool_hash_mismatch`.

### Steps 4–6 — Routing Replay

Run the routing kernel against the parsed case, policy, candidates, and snapshots. Compare the replay result to the receipt:

| Check | Field compared | Error code |
|---|---|---|
| Routing proof hash | `routing_proof_hash` | `routing_proof_hash_mismatch` |
| Eligible candidates hash | `eligible_candidate_ids_hash` | `eligible_candidate_ids_hash_mismatch` |
| Selector input hash | `selection_input_candidate_ids_hash` | `selection_input_candidate_ids_hash_mismatch` |
| Candidate order hash | `candidate_order_hash` | `candidate_order_hash_mismatch` |
| Selected candidate / refusal | compared field-by-field | `routing_decision_replay_mismatch` |
| Audit entry hash | `audit_entry_hash` | `audit_entry_hash_mismatch` |
| Audit previous hash | `audit_previous_hash` | `audit_previous_hash_mismatch` |

Verification succeeds only if every check passes. The first failing check terminates verification immediately and returns the corresponding stable error code.

---

## 10. Protocol Manifest

The protocol manifest is a static self-description of the current protocol configuration. It is available from the reference implementation at `GET /protocol-manifest` and via `postcad-cli protocol-manifest`.

### 10.1 Manifest Fields

| Field | Value (v1) | Description |
|---|---|---|
| `protocol_version` | `"postcad-v1"` | Protocol label. |
| `routing_kernel_version` | `"postcad-routing-v1"` | Routing kernel label. |
| `receipt_schema_version` | `"1"` | Receipt JSON schema version. |
| `receipt_schema_hash` | `37a025b6...` | SHA-256 of committed receipt field names (see §6.4). |
| `proof_schema_hash` | `ebd5a82f...` | SHA-256 of proof object field names (see §7.3). |
| `refusal_code_set_hash` | `4ebf952c...` | SHA-256 of canonical refusal code set (see §8). |
| `manifest_fingerprint` | `a46b5519...` | SHA-256 of the five values above, joined with `\n` (see §10.2). |
| `audit_chain_mode` | `"sha256_hash_chained_append_only"` | Audit log algorithm. |
| `canonical_serialization` | `"sha256(compact_json_sorted_keys_utf8)"` | Hash input rule for all commitments. |
| `committed_receipt_fields` | (21-field array) | All receipt fields covered by the protocol commitment, sorted alphabetically. |
| `stable_error_codes` | (24-code array) | All stable verification error codes, sorted alphabetically. |
| `verify_receipt_requires_replay` | `true` | Whether verification requires a full routing replay. |

### 10.2 Manifest Fingerprint Computation

```
manifest_fingerprint = SHA-256(
    protocol_version
    + "\n" + receipt_schema_hash
    + "\n" + proof_schema_hash
    + "\n" + refusal_code_set_hash
    + "\n" + routing_kernel_version
)
```

The frozen v1 fingerprint is:

```
a46b5519ef5b4e3eda3e24666ec52442be6faa0853fd898ed6a8d1dae34df0fa
```

Any change to any of the five inputs produces a different fingerprint. A verifier can use this value to detect whether a manifest describes a different protocol configuration than expected.

### 10.3 Compact Protocol-Info Surface

The `postcad-cli protocol-info` command and the semver constants expose a compact view using semantic version numbers rather than label strings:

```json
{
  "protocol_version":        "1.0",
  "routing_kernel_version":  "1.0",
  "manifest_fingerprint":    "a46b5519ef5b4e3eda3e24666ec52442be6faa0853fd898ed6a8d1dae34df0fa",
  "receipt_schema_hash":     "37a025b6cb167fbec020d6ea5e64ac9fcc2da7ca9e7aa5bf28272f3114c9fb49",
  "proof_schema_hash":       "ebd5a82f9e64d3151d0e2df43585f244abaae664213a9df46a633277b52507e4",
  "refusal_code_set_hash":   "4ebf952c33509c9d67850dbd4e43a2fe9a8b0174f4d3009f5c247a4928cb50df"
}
```

---

## 11. Stable Error Codes

The following error codes are stable identifiers on the protocol surface. Codes are never removed or renamed in a minor version. Messages may be updated for clarity; only codes are stable.

| Code | Condition |
|---|---|
| `audit_entry_hash_mismatch` | Replay `audit_entry_hash` does not match receipt. |
| `audit_previous_hash_mismatch` | Replay `audit_previous_hash` does not match receipt. |
| `candidate_order_hash_mismatch` | Replay `candidate_order_hash` does not match receipt. |
| `candidate_pool_hash_mismatch` | Replay `candidate_pool_hash` does not match receipt. |
| `case_fingerprint_mismatch` | Recomputed `case_fingerprint` does not match receipt. |
| `case_parse_failed` | Case JSON could not be parsed. |
| `eligible_candidate_ids_hash_mismatch` | Replay eligible ID hash does not match receipt. |
| `invalid_receipt_schema_version` | `schema_version` field is not a string. |
| `missing_receipt_schema_version` | `schema_version` field is absent or null. |
| `policy_bundle_parse_failed` | Policy JSON could not be parsed. |
| `policy_fingerprint_mismatch` | Recomputed `policy_fingerprint` does not match receipt. |
| `policy_version_mismatch` | `policy_version` in policy bundle differs from receipt. |
| `protocol_version_mismatch` | Protocol version is not the expected value. |
| `receipt_canonicalization_mismatch` | `receipt_hash` does not match SHA-256 of the canonical receipt content. |
| `receipt_parse_failed` | Receipt JSON could not be parsed as a v1 receipt. |
| `registry_snapshot_hash_mismatch` | Recomputed registry snapshot hash does not match receipt. |
| `routing_decision_hash_mismatch` | Recomputed `routing_decision_hash` does not match receipt. |
| `routing_decision_replay_mismatch` | Routing replay produced a different decision than the receipt records. |
| `routing_input_hash_mismatch` | Recomputed `routing_input_hash` does not match receipt. |
| `routing_kernel_version_mismatch` | Receipt `routing_kernel_version` is not `"postcad-routing-v1"`. |
| `routing_proof_hash_mismatch` | Replay `routing_proof_hash` does not match receipt. |
| `selection_input_candidate_ids_hash_mismatch` | Replay selector-input hash does not match receipt. |
| `unknown_refusal_code` | Receipt `refusal_code` is not in the canonical refusal code set. |
| `unsupported_receipt_schema_version` | `schema_version` value is not `"1"`. |

---

## 12. Determinism Guarantee

**Property:** For any fixed tuple `(case, registry_snapshot, policy)`, the routing engine always produces the same receipt, including all hash fields.

This holds because:

1. All inputs are fully serialized before hashing. No timestamps, random seeds, or external state are incorporated into the routing decision or the hash commitments.
2. Hash inputs use a canonical serialization form (compact JSON, keys alphabetically sorted, UTF-8) that is independent of runtime state.
3. The candidate selection algorithms (`HighestPriority`, `DeterministicHash`) are purely functional: same inputs → same index → same candidate.
4. The audit log uses hash chaining but the genesis hash is defined (64 zero hex digits), so the first entry is also deterministic.

Any implementation claiming Protocol v1 compliance must satisfy this property.

---

## 13. Security Model

### 13.1 What PostCAD Guarantees

**Routing integrity:** The receipt commits to every input that determined the routing decision. Any modification to the case, registry snapshot, policy, or the decision fields causes at least one hash check to fail during verification. The commitment chain prevents undetected substitution of candidates, outputs, or inputs.

**Auditability:** Every routing event appends an entry to the append-only, hash-chained audit log. The chain can be independently verified by recomputing each entry hash and checking linkage. Log entries cannot be removed or reordered without breaking the chain.

**Deterministic verification:** Verification is a pure function of the original inputs and the receipt. It does not require access to any persistent state, database, or network resource. Any party holding the inputs can verify the receipt independently.

### 13.2 What PostCAD Does Not Guarantee

**Manufacturing quality:** PostCAD routes to a candidate that satisfies the stated compliance and policy constraints. It does not verify the quality of the manufactured output.

**Economic optimization:** The selected candidate is the first in canonical priority order, not necessarily the lowest price, fastest turnaround, or best fit by any economic metric.

**Execution enforcement:** PostCAD produces a verifiable routing decision. It does not enforce that the selected manufacturer accepts the case, executes the job, or delivers the result.

**Clinical liability:** PostCAD is not a clinical decision system. It does not own, assess, or guarantee any clinical outcome.

---

## 14. Versioning Rules

### 14.1 Protocol v1 Is Frozen

Protocol v1 is a frozen protocol. The following are immutable for the lifetime of v1:

- The 21 committed receipt fields and their semantics.
- The 11 proof object fields and their semantics.
- The 7 canonical refusal codes and their string values.
- The 24 stable error codes and their string values.
- The canonical serialization rule (`sha256(compact_json_sorted_keys_utf8)`).
- The routing kernel label `"postcad-routing-v1"`.
- The protocol label `"postcad-v1"`.
- The manifest fingerprint `a46b5519ef5b4e3eda3e24666ec52442be6faa0853fd898ed6a8d1dae34df0fa`.

### 14.2 Future Versions

Backwards-compatible additions (new optional fields, new refusal codes, new error codes) increment the minor version: `v1.1`.

Breaking changes (field removals, semantic changes to existing fields, new required fields) increment the major version: `v2`.

### 14.3 Long-Term Verifiability

Receipts generated under Protocol v1 must remain verifiable under Protocol v1 rules indefinitely. A v1 verifier must:

1. Accept any receipt with `schema_version == "1"` and `routing_kernel_version == "postcad-routing-v1"`.
2. Apply exactly the verification algorithm described in [Section 9](#9-verification-algorithm).
3. Reject receipts that claim v1 but carry fields or hash values inconsistent with the v1 commitment structure.

A future v2 verifier may optionally support v1 receipts by preserving the v1 verification path, but is not required to do so.
