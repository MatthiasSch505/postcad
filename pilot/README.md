# PostCAD v1 Pilot Handoff

This directory is the entry point for anyone evaluating or operating PostCAD Protocol v1.

**Time to first run: ~2 minutes.**

---

## What PostCAD does

PostCAD is a deterministic, rule-based routing kernel for dental manufacturing cases. Given a case description and a manufacturer registry, it:

1. Evaluates compliance constraints
2. Selects an eligible manufacturer — or records a structured refusal
3. Emits a tamper-evident receipt that any third party can independently verify

Every decision is deterministic: same inputs → same receipt hash, every time.

---

## Prerequisites

```
cargo (Rust toolchain)
```

Build once from the workspace root:

```bash
cargo build -p postcad-cli
```

The binary is at `target/debug/postcad-cli`.

---

## The three canonical flows

### 1. Routed flow — case is matched to a manufacturer

```bash
cargo run -p postcad-cli -- route-case \
  --case      fixtures/case.json \
  --candidates fixtures/candidates.json \
  --snapshot  fixtures/snapshot.json \
  --json
```

Expected key fields in output:

```
"outcome":              "routed"
"selected_candidate_id": "rc-de-01"
"receipt_hash":         "337f858244b2abb780a50a39774ba4ba44168571310785040b97977f59e7f036"
```

Full frozen output: `fixtures/expected_routed.json`

---

### 2. Verify flow — prove a receipt is authentic

Verification performs a full deterministic routing replay and checks every committed field.

```bash
cargo run -p postcad-cli -- verify-receipt \
  --receipt    fixtures/expected_routed.json \
  --case       fixtures/case.json \
  --policy     fixtures/policy.json \
  --candidates fixtures/candidates.json \
  --json
```

Expected output:

```json
{"result":"VERIFIED"}
```

Exit code: `0`

---

### 3. Refusal flow — case is refused with a reason code

```bash
cargo run -p postcad-cli -- route-case \
  --case      fixtures/case.json \
  --candidates fixtures/candidates.json \
  --snapshot  fixtures/snapshot_refusal.json \
  --json
```

Expected key fields:

```
"outcome":      "refused"
"refusal_code": "no_eligible_candidates"
"receipt_hash": "bd3b97dc5efddff25fd77f2bff35641710a923e383b612cad65defaaf81eb1b9"
```

Full frozen output: `fixtures/expected_refused.json`

Refusal receipts are also verifiable:

```bash
cargo run -p postcad-cli -- verify-receipt \
  --receipt    fixtures/expected_refused.json \
  --case       fixtures/case.json \
  --policy     fixtures/policy.json \
  --candidates fixtures/candidates.json \
  --json
```

---

### 4. Drift detection — tampered receipt is rejected

To prove tamper-evidence, change any field in a receipt (e.g. `selected_candidate_id`) and re-verify. The kernel replays the routing and detects the mismatch:

```bash
# Edit a copy of expected_routed.json, change selected_candidate_id to any other value, then:
cargo run -p postcad-cli -- verify-receipt \
  --receipt    /tmp/tampered.json \
  --case       fixtures/case.json \
  --policy     fixtures/policy.json \
  --candidates fixtures/candidates.json \
  --json
```

Expected output:

```json
{"result":"VERIFICATION FAILED","code":"routing_decision_hash_mismatch","reason":"..."}
```

Exit code: `1`

---

### 5. One-command demo (all three steps in sequence)

```bash
cargo run -p postcad-cli -- demo-run --json
```

Expected output:

```json
{
  "result": "VERIFIED",
  "protocol_version": "postcad-v1",
  "outcome": "routed",
  "selected_candidate_id": "rc-de-01",
  "receipt_hash": "337f858244b2abb780a50a39774ba4ba44168571310785040b97977f59e7f036"
}
```

---

## Protocol manifest

Machine-readable self-description of the protocol contract:

```bash
cargo run -p postcad-cli -- protocol-manifest --json
```

Key fields:

| Field | Value |
|---|---|
| `protocol_version` | `postcad-v1` |
| `routing_kernel_version` | `postcad-routing-v1` |
| `receipt_schema_version` | `1` |
| `verify_receipt_requires_replay` | `true` |
| `committed_receipt_fields` | 21 fields |
| `stable_error_codes` | 22 codes |

Full frozen output: `fixtures/expected_manifest.json`

---

## Registry-backed routing (production path)

The demo above uses hand-crafted policy bundles. The production path derives candidates directly from a typed manufacturer registry:

```bash
cargo run -p postcad-cli -- route-case-from-registry \
  --case     tests/protocol_vectors/v01_basic_routing/case.json \
  --registry tests/protocol_vectors/v01_basic_routing/registry_snapshot.json \
  --config   tests/protocol_vectors/v01_basic_routing/policy.json \
  --json
```

The command also self-verifies the receipt before printing it.

---

## Canonical inputs

| File | Contents |
|---|---|
| `fixtures/case.json` | Canonical demo case (crown/zirconia, DE jurisdiction) |
| `fixtures/candidates.json` | Single eligible candidate (`rc-de-01 / mfr-de-01`) |
| `fixtures/snapshot.json` | Valid compliance snapshot (attestation: verified) |
| `fixtures/snapshot_refusal.json` | Invalid snapshot (attestation: rejected) → refusal |
| `fixtures/policy.json` | RoutingPolicyBundle (DE, allow\_domestic\_and\_cross\_border) |
| `fixtures/expected_routed.json` | Frozen routed receipt (golden artifact) |
| `fixtures/expected_refused.json` | Frozen refused receipt (golden artifact) |
| `fixtures/expected_manifest.json` | Frozen protocol manifest (golden artifact) |

---

## Protocol conformance vectors

Five routing vectors and five verifier vectors are frozen at:

```
tests/protocol_vectors/          # v01–v05: routing scenarios
tests/protocol_verifier_vectors/ # v01–v05: tamper/drift scenarios
```

Run all conformance tests:

```bash
cargo test -p postcad-cli
```

---

## Protocol v1 specification

Full technical spec: `PROTOCOL_V1.md` at the workspace root.

Covers: accepted inputs, receipt schema, replay verification model, refusal codes, audit-chain behavior, deterministic guarantees, stable error codes, and explicit out-of-scope items.

---

## Stable error codes (quick reference)

Verification failures always carry a stable machine-readable `code`:

| Code | Meaning |
|---|---|
| `receipt_canonicalization_mismatch` | `receipt_hash` doesn't match recomputed value |
| `routing_decision_hash_mismatch` | Decision fields were tampered |
| `registry_snapshot_hash_mismatch` | Registry snapshot was changed |
| `candidate_pool_hash_mismatch` | Candidate list was changed |
| `routing_decision_replay_mismatch` | Replay produced a different outcome |
| `case_fingerprint_mismatch` | Case inputs were changed |
| `policy_fingerprint_mismatch` | Policy config was changed |
| `receipt_parse_failed` | Receipt JSON is malformed or missing required fields |

Full list of 22 stable codes: `PROTOCOL_V1.md` or `protocol-manifest --json`.
