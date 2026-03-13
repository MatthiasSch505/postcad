# PostCAD

**Deterministic, verifiable routing and handoff infrastructure for dental CAD manufacturing.**

PostCAD sits between CAD design and production. It checks regulatory compliance by destination country, selects an eligible manufacturer via a deterministic kernel, records every decision in an append-only audit chain, and issues a cryptographically verifiable receipt. The receipt can be independently checked without the routing engine.

**No AI. No randomness. No timestamps in routing logic. Every decision carries a machine-readable reason code.**

---

## Protocol Flow

```
Case Input → Route → Receipt → Verify → Create Dispatch → Approve → Export Packet
```

1. **Route** — compliance rules evaluated (EU MDR, FDA 510k, MHLW, ISO 13485); eligible candidate selected deterministically
2. **Receipt** — 21 hash-committed fields; `receipt_hash` covers everything; portable and self-contained
3. **Verify** — replays routing from raw inputs; recomputes every hash; no stored state trusted
4. **Create Dispatch** — binds to the exact verified receipt; one dispatch per `receipt_hash`
5. **Approve** — operator identity and timestamp locked; routing fields become immutable
6. **Export** — deterministic canonical JSON packet; same bytes every time for the same approved dispatch

See [docs/protocol_diagram.md](docs/protocol_diagram.md) for a one-page visual.
See [docs/pilot_walkthrough.md](docs/pilot_walkthrough.md) for the step-by-step operator guide.
See [docs/pilot_run_example.md](docs/pilot_run_example.md) for a concrete trace with real values.

---

## Quickstart — verify a receipt in under 5 minutes

```bash
git clone https://github.com/MatthiasSch505/postcad.git
cd postcad
cargo build

cargo run -p postcad-cli -- verify-receipt --json \
  --receipt    examples/valid_routed_receipt.json \
  --case       fixtures/scenarios/routed_domestic_allowed/case.json \
  --policy     fixtures/scenarios/routed_domestic_allowed/policy.json \
  --candidates fixtures/scenarios/routed_domestic_allowed/candidates.json
```

Expected output:

```json
{"result":"VERIFIED"}
```

Tamper any field in the receipt and re-run — the response will include a stable `code`
identifying the exact failing check.

---

## Guarantees

| Invariant | Mechanism |
|---|---|
| Routing is deterministic | Same inputs → same receipt, always; kernel is stateless |
| Receipt integrity | 21 hash-committed fields; `receipt_hash` verified before any semantic check |
| Independent verifiability | `verify-receipt` recomputes every hash from raw inputs; does not trust stored state |
| Dispatch binds to exact receipt | `receipt_hash` is the chain-of-custody key; duplicate dispatch rejected (409) |
| Approval is final | `approved_by` + `approved_at` locked at approval; no further mutation |
| Audit trail is tamper-evident | SHA-256 hash-chained log; any deletion or reorder breaks `verify_chain()` |
| No silent decisions | All outcomes wrapped in `Decision<T>` with a `ReasonCode` |

Key verification failure codes: `receipt_hash_mismatch`, `routing_proof_hash_mismatch`,
`candidate_pool_hash_mismatch`, `eligible_candidate_ids_hash_mismatch`,
`selection_input_candidate_ids_hash_mismatch`, `audit_entry_hash_mismatch`,
`audit_previous_hash_mismatch`, `receipt_parse_failed`.

---

## Reviewer Shell

A thin local app over the real route/verify kernel. No mocked decisions.

```bash
cargo run -p postcad-service
# then open http://localhost:8080/reviewer
```

Loads pilot fixtures automatically. Three clicks: route → verify → tamper demo.
Expected receipt hash: `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb`

---

## Workspace

```
postcad-core        shared domain types (Case, Decision, ReasonCode, …)
postcad-registry    manufacturer registry and certification structs
postcad-compliance  compliance rule engine — stateless, deterministic
postcad-routing     routing engine with pluggable selector strategies
postcad-audit       hash-chained append-only audit log
postcad-cli         CLI: route-case, verify-receipt, protocol-manifest, …
postcad-service     axum HTTP service + reviewer shell + dispatch commitment layer
```

```bash
cargo test --workspace   # full suite
```

---

## Responsibility Boundary

PostCAD owns: routing decision · cryptographic proof · dispatch commitment · export packet.

PostCAD does **not**:
- produce or modify CAD geometry
- discover or manage manufacturer relationships
- make clinical decisions
- apply AI or probabilistic judgment

Liability for production and clinical outcomes sits outside PostCAD entirely.

---

## Further Reading

- [docs/protocol_diagram.md](docs/protocol_diagram.md) — one-page mental model
- [docs/pilot_walkthrough.md](docs/pilot_walkthrough.md) — step-by-step operator walkthrough
- [docs/pilot_run_example.md](docs/pilot_run_example.md) — canonical happy-path trace with real values
- [docs/local_service_run.md](docs/local_service_run.md) — local service run path
- [docs/development_bundle.md](docs/development_bundle.md) — development workflow and red lines
- [docs/operator_handoff.md](docs/operator_handoff.md) — pilot acceptance run path and checklist
