# PostCAD

Deterministic, rule-based routing platform for dental CAD manufacturing. Sits after
design and before production: verifies manufacturer certifications, checks regulatory
constraints by country, routes cases to eligible manufacturers, and records an
immutable audit trail.

Every routing decision produces a cryptographically verifiable receipt. Every receipt
can be independently verified against the original inputs without access to the routing
engine.

---

## Quickstart — verify a receipt in under 5 minutes

```bash
git clone https://github.com/MatthiasSch505/postcad.git
cd postcad
cargo build

# Verify a pre-built routed receipt against its original inputs:
cargo run -p postcad-cli -- verify-receipt --json \
  --receipt  examples/valid_routed_receipt.json \
  --case     fixtures/scenarios/routed_domestic_allowed/case.json \
  --policy   fixtures/scenarios/routed_domestic_allowed/policy.json \
  --candidates fixtures/scenarios/routed_domestic_allowed/candidates.json
```

Expected output:

```json
{"result":"VERIFIED"}
```

To see a verification failure, tamper any field in the receipt and re-run. The
response will include a stable machine-readable `code` identifying the failing check.

---

## Run a routing decision

```bash
cargo run -p postcad-cli -- route-case --json \
  --case       fixtures/scenarios/routed_domestic_allowed/case.json \
  --candidates fixtures/scenarios/routed_domestic_allowed/candidates.json \
  --snapshot   fixtures/scenarios/routed_domestic_allowed/snapshot.json
```

---

## Receipt structure

A routing receipt (`schema_version: "1"`) contains:

| Field | Description |
|---|---|
| `schema_version` | Always `"1"` for this pipeline. Checked before all other fields. |
| `outcome` | `"routed"` or `"refused"`. |
| `case_fingerprint` | SHA-256 of the canonical case payload. |
| `policy_fingerprint` | SHA-256 of the routing policy bundle. |
| `routing_proof_hash` | SHA-256 of the canonical routing decision. |
| `candidate_pool_hash` | SHA-256 of the full input candidate pool (sorted by id, order-independent). |
| `eligible_candidate_ids_hash` | SHA-256 of the sorted eligible candidate ID list after compliance and policy filtering. Order-independent. |
| `selection_input_candidate_ids_hash` | SHA-256 of the candidate ID list in the exact order presented to the deterministic selector. **Order-sensitive.** |
| `selected_candidate_id` | ID of the selected candidate. `null` on refused outcomes. |
| `refusal_code` | Machine-readable refusal reason. `null` on routed outcomes. |
| `audit_seq` | Sequence number of the audit log entry (0-indexed). |
| `audit_entry_hash` | SHA-256 of the audit log entry that records this decision. |
| `audit_previous_hash` | SHA-256 of the preceding audit entry (64 zeros for genesis). |
| `receipt_hash` | SHA-256 of all other receipt fields (canonical BTreeMap serialization). Computed last; verified first. |

See `examples/valid_routed_receipt.json` and `examples/valid_refusal_receipt.json`.

---

## Architecture

```
Case Input (JSON)
      │
      ▼
Compliance Engine          ← stateless rules: EU MDR, FDA 510k, MHLW, ISO 13485
      │  eligible candidates
      ▼
Routing Engine             ← deterministic selector (HighestPriority | DeterministicHash)
      │  routing decision
      ▼
Candidate Commitment       ← candidate_pool_hash, eligible_ids_hash, selection_input_hash
      │
      ▼
Append-Only Audit Chain    ← SHA-256 hash-chained log; genesis previous_hash = 64 zeros
      │  audit_entry_hash, audit_previous_hash
      ▼
Receipt Assembly           ← all fields serialized; receipt_hash computed last
      │
      ▼
verify-receipt             ← independent verifier; recomputes every hash from inputs
```

**Deterministic routing kernel.** Same case + same eligible candidates always
produces the same routing decision. No randomness, no timestamps, no AI.

**Routing receipt.** A self-contained JSON artifact binding the routing outcome to
its inputs via a chain of SHA-256 commitments. Portable: can be verified anywhere
without the routing engine.

**Append-only audit chain.** Each entry contains the previous entry's hash. Any
deletion or reordering of entries breaks the chain and is detectable by
`verify_chain()`.

**Independent verifier.** `verify-receipt` recomputes every hash from the original
inputs and compares against the receipt. It does not trust the receipt's own field
values. Failure produces a stable `code` identifying the specific broken commitment.

---

## Tests

```bash
cargo test --workspace   # 454 tests across 6 crates
```

---

## Workspace

```
postcad-core        shared domain types (Case, Decision, ReasonCode, …)
postcad-registry    manufacturer registry and certification structs
postcad-compliance  compliance rule engine (stateless, deterministic)
postcad-routing     routing engine with pluggable selector strategies
postcad-audit       hash-chained append-only audit log
postcad-cli         route-case and verify-receipt CLI commands
```
