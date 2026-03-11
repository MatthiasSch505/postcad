# PostCAD Protocol v1

**Version identifiers**

| Constant | Value |
|---|---|
| `protocol_version` | `postcad-v1` |
| `routing_kernel_version` | `postcad-routing-v1` |
| `receipt_schema_version` | `1` |

---

## Purpose

PostCAD Protocol v1 is a deterministic, rule-based routing protocol for dental manufacturing cases. It accepts a case description and a manufacturer registry, evaluates compliance constraints, selects a manufacturer (or records a refusal), and emits a tamper-evident receipt that any third party can independently verify.

Every decision carries a machine-readable reason code. No AI, no stochastic behaviour, no mutable state.

---

## Accepted inputs

### Case input (`case.json`)

| Field | Type | Notes |
|---|---|---|
| `case_id` | UUID string | Determines deterministic candidate selection |
| `jurisdiction` | string | Routing jurisdiction (e.g. `"DE"`) |
| `routing_policy` | string | Policy variant (`"allow_domestic_and_cross_border"`, …) |
| `patient_country` | string | Clinic/patient country |
| `manufacturer_country` | string | Required manufacturing country |
| `material` | string | Dental material (e.g. `"zirconia"`) |
| `procedure` | string | Dental procedure type (e.g. `"crown"`) |
| `file_type` | string | CAD file format (e.g. `"stl"`) |

### Registry snapshot (`registry_snapshot.json`)

Array of `ManufacturerRecord` objects. Each record carries: `manufacturer_id`, `country`, `is_active`, `capabilities`, `materials_supported`, `jurisdictions_served`, `attestation_statuses`, `sla_days`.

### Routing config (`registry_routing_config.json` / `policy.json`)

`RegistryRoutingConfig`: `{ "jurisdiction": "...", "routing_policy": "..." }`.

---

## Produced artifacts

### `RoutingReceipt` (schema version `"1"`)

Emitted by `route-case` and `route-case-from-registry`. Both routed and refused outcomes share the same schema.

**Committed fields** (all 21 are covered by `receipt_hash`):

```
audit_entry_hash            audit_previous_hash         audit_seq
candidate_order_hash        candidate_pool_hash         case_fingerprint
eligible_candidate_ids_hash outcome                     policy_fingerprint
policy_version              receipt_hash                refusal_code
registry_snapshot_hash      routing_decision_hash       routing_input
routing_input_hash          routing_kernel_version      routing_proof_hash
schema_version              selected_candidate_id       selection_input_candidate_ids_hash
```

`receipt_hash` = `SHA-256(compact_json_sorted_keys(receipt_without_receipt_hash))`.

### `RoutingPolicyBundle` (`policy.json` — verifier input)

Produced alongside every receipt. Contains the `candidates` list (with per-candidate `eligibility`), the `snapshots` list (compliance snapshots used during routing), and optional `refusal_reason_hint` and `policy_version`. Consumed by `verify-receipt`.

### `RoutingProofObject`

11-field projection of the receipt's committed hashes, produced by `build_routing_proof`. Fields: `audit_entry_hash`, `audit_previous_hash`, `candidate_order_hash`, `candidate_pool_hash`, `protocol_version`, `receipt_hash`, `registry_snapshot_hash`, `routing_decision_hash`, `routing_kernel_version`, `routing_input_hash`, `selected_candidate_id`.

### Protocol manifest (`protocol-manifest --json`)

Static, compile-time self-description of the protocol. Contains: `protocol_version`, `routing_kernel_version`, `receipt_schema_version`, `audit_chain_mode`, `canonical_serialization`, `committed_receipt_fields`, `stable_error_codes`, `verify_receipt_requires_replay`.

---

## Replay verification model

`verify-receipt` performs a full deterministic routing replay and checks every committed field in order:

| Step | Check | Error code on failure |
|---|---|---|
| 1a | `schema_version` present + equals `"1"` | `missing_receipt_schema_version` / `invalid_receipt_schema_version` / `unsupported_receipt_schema_version` |
| 1b | `receipt_hash` recomputed from receipt body | `receipt_canonicalization_mismatch` |
| 1c | `routing_input_hash` recomputed from `routing_input` | `routing_input_hash_mismatch` |
| 1d | `routing_kernel_version` equals kernel constant | `routing_kernel_version_mismatch` |
| 1e | `routing_decision_hash` recomputed from decision fields | `routing_decision_hash_mismatch` |
| 2 | `case_fingerprint` recomputed from case inputs | `case_fingerprint_mismatch` |
| 3 | `policy_fingerprint` + `policy_version` match policy config | `policy_fingerprint_mismatch` / `policy_version_mismatch` |
| 4a | `routing_proof_hash` recomputed from fingerprint | `routing_proof_hash_mismatch` |
| 4b | `eligible_candidate_ids_hash` + `selection_input_candidate_ids_hash` + `candidate_order_hash` | `eligible_candidate_ids_hash_mismatch` / `selection_input_candidate_ids_hash_mismatch` / `candidate_order_hash_mismatch` |
| 4c | `registry_snapshot_hash` recomputed from policy snapshots | `registry_snapshot_hash_mismatch` |
| 4d | `candidate_pool_hash` recomputed from policy candidates | `candidate_pool_hash_mismatch` |
| 5–6 | Full routing replay; outcome + selected candidate match | `routing_decision_replay_mismatch` |
| 7 | `audit_entry_hash` + `audit_previous_hash` recomputed | `audit_entry_hash_mismatch` / `audit_previous_hash_mismatch` |

Replay is mandatory (`verify_receipt_requires_replay: true`).

---

## Refusal behavior

When no manufacturer can be selected, the receipt carries `outcome: "refused"` and a stable `refusal_code`. Codes are derived from stepwise registry filtering:

| Code | Condition |
|---|---|
| `no_eligible_manufacturer` | Registry is empty |
| `no_active_manufacturer` | All records have `is_active: false` |
| `no_jurisdiction_match` | No record serves the required jurisdiction |
| `no_capability_match` | No record supports the required procedure |
| `no_material_match` | No record supports the required material |
| `attestation_failed` | All surviving records have invalid attestations |
| `no_eligible_candidates` | Catch-all (non-registry routing path) |

Refusal codes are committed into `routing_decision_hash` and verified during replay.

---

## Audit-chain behavior

Every routing call appends one entry to an in-memory SHA-256 hash-chained log (`audit_chain_mode: "sha256_hash_chained_append_only"`).

- Each entry: `{ seq, event, previous_hash }` → `audit_entry_hash = SHA-256(canonical_json(entry))`.
- First entry: `audit_previous_hash` = 64 zero characters.
- Receipt carries `audit_seq`, `audit_entry_hash`, `audit_previous_hash`; all three are committed into `receipt_hash`.
- `verify_chain()` recomputes every hash and checks linkage; any gap or mutation is detectable.

---

## Deterministic guarantees

1. Same `case_id` + same registry + same policy → same `selected_candidate_id`, same `receipt_hash`.
2. All hash commitments use `SHA-256(compact_json_sorted_keys_utf8)`.
3. Candidate selection uses `DeterministicHash` (hash of `case_id`) or `HighestPriority`; both sort by `(priority, id)` before selection.
4. Candidate pool and eligible IDs are sorted by `manufacturer_id` / lexicographic order before hashing — order of registry input does not affect output.
5. Candidate order hash commits to the exact sorted order presented to the selector, independently detectable from `candidate_pool_hash`.

---

## Protocol surfaces

### CLI (`postcad-cli`)

| Subcommand | Input | Output |
|---|---|---|
| `route-case` | `--case`, `--candidates`, `--snapshot`, `--policy` | `RoutingReceipt` (JSON) |
| `route-case-from-registry` | `--case`, `--registry`, `--config` | `RoutingReceipt` (JSON) |
| `verify-receipt` | `--receipt`, `--case`, `--policy-bundle` | `{"result":"VERIFIED"}` or exit 1 + error JSON |
| `protocol-manifest` | `--json` | `ProtocolManifest` (JSON) |

### HTTP service (`postcad-service`)

| Endpoint | Method | Notes |
|---|---|---|
| `POST /route-case` | POST | Body: `{case, candidates, snapshot, policy}` |
| `POST /route-case-from-registry` | POST | Body: `{case, registry, config}` |
| `POST /verify-receipt` | POST | Body: `{receipt, case, policy_bundle}` |

### Rust library (`postcad-cli` as lib)

Public functions: `route_case_from_json`, `route_case_from_registry_json`, `verify_receipt_from_json`, `verify_receipt_from_policy_json`, `build_manifest`, `build_routing_proof`, `verify_routing_proof`.

---

## Conformance vectors

Located at `tests/protocol_vectors/` (5 routing vectors) and `tests/protocol_verifier_vectors/` (5 verifier vectors). All vectors are self-seeding: absent frozen files are generated on first run; subsequent runs compare against the frozen artifacts.

**Routing vectors** (`tests/protocol_vectors/`):

| Vector | Scenario | Expected outcome |
|---|---|---|
| `v01_basic_routing` | Single eligible domestic manufacturer | `routed`, `mfr-de-001` |
| `v02_multi_candidate` | Three eligible candidates; deterministic selection | `routed`, stable hash |
| `v03_jurisdiction_refusal` | US-only registry, DE case | `refused`, `no_jurisdiction_match` |
| `v04_capability_refusal` | Implant/bridge only, crown case | `refused`, `no_capability_match` |
| `v05_attestation_refusal` | Expired + revoked attestations | `refused`, `attestation_failed` |

**Verifier vectors** (`tests/protocol_verifier_vectors/`):

| Vector | Tamper type | Expected error code |
|---|---|---|
| `v01_valid_receipt` | None | — (Ok) |
| `v02_tampered_routing_decision_hash` | `routing_decision_hash` replaced; `receipt_hash` recomputed | `routing_decision_hash_mismatch` |
| `v03_tampered_registry_snapshot_hash` | Snapshot evidence reference drifted | `registry_snapshot_hash_mismatch` |
| `v04_tampered_candidate_pool_hash` | Phantom candidate injected into policy candidates | `candidate_pool_hash_mismatch` |
| `v05_tampered_receipt_hash` | `receipt_hash` replaced with wrong value | `receipt_canonicalization_mismatch` |

---

## Stable error codes

All 22 codes below are permanent protocol identifiers. Messages may change; codes will not:

```
audit_entry_hash_mismatch           audit_previous_hash_mismatch
candidate_order_hash_mismatch       candidate_pool_hash_mismatch
case_fingerprint_mismatch           case_parse_failed
eligible_candidate_ids_hash_mismatch  invalid_receipt_schema_version
missing_receipt_schema_version      policy_bundle_parse_failed
policy_fingerprint_mismatch         policy_version_mismatch
receipt_canonicalization_mismatch   receipt_parse_failed
registry_snapshot_hash_mismatch     routing_decision_hash_mismatch
routing_decision_replay_mismatch    routing_input_hash_mismatch
routing_kernel_version_mismatch     routing_proof_hash_mismatch
selection_input_candidate_ids_hash_mismatch  unsupported_receipt_schema_version
```

Note: `receipt_hash_mismatch` is defined in source but is unreachable in the current verification path; `receipt_canonicalization_mismatch` is fired instead. It is **not** a stable surface code.

---

## Out of scope for v1

The following are explicitly not part of PostCAD Protocol v1:

- **Persistent storage** — no database, no disk-backed audit log; the audit chain is in-memory per process lifetime.
- **Authentication / authorisation** — the HTTP service has no auth layer.
- **Multi-tenancy** — one registry, one policy per request; no tenant isolation.
- **Streaming or batched routing** — single case per call only.
- **Case lifecycle management** — no status tracking, no case updates, no cancellations.
- **Manufacturer onboarding / registry mutation** — the registry is read-only input; there is no write path.
- **SLA enforcement** — `sla_days` is stored but not used in routing decisions.
- **Human-readable audit reports** — the audit chain is machine-readable only.
- **Cross-version receipt migration** — receipts with `schema_version != "1"` are rejected, not migrated.
- **gRPC / GraphQL / other transport protocols** — HTTP JSON and CLI only.
- **SDKs / client libraries** — no generated clients.
- **Production deployment configuration** — no TLS, rate limiting, or infrastructure config.
- **AI or ML-based routing** — all decisions are rule-based and deterministic.
