# PostCAD Pilot Frozen Boundaries

Explicit statement of what is frozen for the current local pilot. No changes to any item in the "frozen" sections are permitted within the scope of this pilot package.

---

## Frozen: protocol behavior

- Protocol version: `postcad-v1`
- The protocol version string is embedded in every routing receipt and verified on every `verify_receipt` call.
- A receipt issued under `postcad-v1` is only verifiable against a `postcad-v1` verifier.
- No protocol version changes are in scope.

---

## Frozen: routing kernel

- Routing kernel version: `postcad-routing-v1`
- The kernel selects a manufacturer from an eligible candidate list using a deterministic strategy.
- The kernel behavior — selection logic, priority ordering, hash-based distribution — is frozen.
- No routing algorithm changes are in scope.

---

## Frozen: receipt schema

- Receipt schema version: `1`
- All field names, types, and presence requirements in the routing receipt are frozen.
- The set of 21 committed receipt fields is frozen.
- Required fields: `receipt_hash`, `routing_input_hash`, `routing_proof_hash`, and all other committed fields.
- No field additions, removals, or renames are in scope.

---

## Frozen: canonical hashing behavior

- All hash commitments use SHA-256.
- `receipt_hash` is the outermost commitment: it commits to all other hash fields.
- Verification replays the routing decision and recomputes all hashes.
- The deterministic receipt hash for the canonical pilot inputs is always:
  `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb`
- No changes to hash computation, field ordering, or commitment structure are in scope.

---

## Frozen: endpoint surface

The following HTTP endpoints are frozen in path, method, and response shape:

| Method | Path | Frozen behavior |
|---|---|---|
| `GET` | `/health` | Returns `{"status":"ok"}` |
| `POST` | `/cases` | Store a case; 201 (first), 200 (identical re-run), 409 (conflict) |
| `POST` | `/cases/:id/route` | Route a case; returns flat receipt fields |
| `GET` | `/receipts/:hash` | Retrieve a stored receipt |
| `POST` | `/dispatch/:hash` | Dispatch a routed case; 200 (first), 409 (idempotent re-run) |
| `POST` | `/dispatch/:hash/verify` | Verify a dispatched receipt; returns `{"result":"VERIFIED"}` |
| `GET` | `/routes` | Return route history |

No new endpoints, no path changes, no response shape changes are in scope.

---

## Frozen: canonical fixtures

The following files are frozen. They must not be modified:

- `examples/pilot/case.json`
- `examples/pilot/registry_snapshot.json`
- `examples/pilot/config.json`
- `tests/protocol_vectors/v01/` through `v05/` (all vector inputs and expected receipts)

---

## Frozen: release package intent

The `release/` tree is a packaging, orientation, and inspection layer. Its scripts invoke the frozen system — they do not modify it. The following release-layer behaviors are also frozen:

- The 7-step smoke test flow (`release/smoke_test.sh`) — steps, endpoint calls, assertion logic
- The 7-step evidence capture flow (`release/generate_evidence_bundle.sh`) — steps, file names, folder structure
- The canonical pilot case used in both flows: `case_id = f1000001-0000-0000-0000-000000000001`

---

## Not frozen: runtime-generated paths

The following are explicitly not frozen because they are produced at runtime and excluded from git:

| Path | Reason |
|---|---|
| `release/evidence/current/` | Generated fresh on each evidence run; gitignored |
| `data/cases/`, `data/receipts/`, `data/policies/`, `data/dispatch/`, `data/verification/` | Written by the service during a pilot run; cleared by `reset_pilot_data.sh`; gitignored |

The content of these paths is deterministic given the same inputs, but they are not committed and are not part of the frozen package.

---

## Not frozen: local git state

The current HEAD commit hash and branch state reflect the local repo at the time of inspection. They are not part of the frozen protocol or package definition. The expected state for a correctly set-up pilot is: branch `main`, clean working tree.

---

## What this document does not claim

- This document is a description of frozen scope, not a certification.
- It does not assert regulatory compliance, production readiness, or external audit status.
- It does not replace the acceptance checklist in `release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md`.
- It does not claim coverage beyond the single canonical pilot case.
