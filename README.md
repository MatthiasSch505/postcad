# PostCAD

Deterministic, verifiable routing core for dental CAD manufacturing decisions.

PostCAD sits between CAD design and production. It evaluates regulatory compliance
by destination country, selects an eligible manufacturer via a deterministic kernel,
and records every decision in an append-only audit chain. The output is a
cryptographically verifiable receipt that can be independently checked without
access to the routing engine.

**No AI. No randomness. No timestamps in routing logic. Every decision carries a
machine-readable reason code.**

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

## Verification guarantees

`verify-receipt` recomputes every hash from the original inputs and compares against
the receipt. It does not trust any field value in the receipt itself.

| What is detected | Failure code |
|---|---|
| Any receipt field modified after issuance | `receipt_hash_mismatch` |
| Routing proof tampered or recomputed from different inputs | `routing_proof_hash_mismatch` |
| Candidate pool changed (manufacturer added, removed, or modified) | `candidate_pool_hash_mismatch` |
| Eligible candidate set changed after compliance filtering | `eligible_candidate_ids_hash_mismatch` |
| Order of candidates presented to the selector changed | `selection_input_candidate_ids_hash_mismatch` |
| Audit log entry content modified | `audit_entry_hash_mismatch` |
| Audit chain linkage broken (entry deleted or reordered) | `audit_previous_hash_mismatch` |
| Receipt missing required fields | `receipt_parse_failed` |
| Schema version absent, wrong type, or unsupported | `missing_receipt_schema_version` · `invalid_receipt_schema_version` · `unsupported_receipt_schema_version` |

Artifact integrity (`receipt_hash`) is verified before any semantic check. All
failure codes are stable across versions.

---

## Scope and non-goals

PostCAD is a **routing decision engine and audit infrastructure layer**. It is not:

- **A CAD tool.** PostCAD receives completed case files; it does not produce or
  modify design geometry.
- **A lab marketplace.** PostCAD selects from a pre-configured candidate pool; it
  does not discover, negotiate with, or manage manufacturer relationships.
- **An AI optimizer.** Routing is fully deterministic and rule-based. Given the same
  case and the same eligible candidates, PostCAD always produces the same decision.
- **A manufacturing operator.** PostCAD issues a routing decision. It does not
  dispatch jobs, track production, or communicate with lab systems.

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

---

## Reviewer shell — one-page local app

A thin reviewer shell over the real route/verify kernel. No mocked decisions.
No shell scripts. Fixed pilot fixtures are loaded from the backend at startup.

```bash
# From the repo root:
cargo run -p postcad-service
```

Then open: **http://localhost:8080/reviewer**

**Steps (under 30 seconds):**

1. The page auto-loads pilot fixtures from `examples/pilot/` — no input needed
2. Click **Execute Routing Kernel** — runs real `POST /route`, shows receipt
3. Click **Replay Verification** — runs real `POST /verify`, shows VERIFIED
4. Click **Tamper + Verify** — modifies `selected_candidate_id` client-side,
   submits to real `POST /verify`, shows the real verification failure

What you will see:

| Field | Value |
|---|---|
| outcome | `routed` |
| selected_candidate_id | `pilot-de-001` |
| receipt_hash | `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb` |
| verify result | `VERIFIED` |
| tamper result | `routing_decision_hash_mismatch` (real error from verifier) |

The receipt hash is deterministic — the same inputs always produce the same hash.
The tamper demo proves receipts are tamper-evident: one field change is caught immediately.

---

## Local service deployment

See [docs/local_service_run.md](docs/local_service_run.md) for the one-command Docker-based local run path.
See [docs/development_bundle.md](docs/development_bundle.md) for the canonical development workflow and red lines.
See [docs/operator_handoff.md](docs/operator_handoff.md) for the pilot acceptance run path and checklist.
