# PostCAD — Boundaries

What is frozen, what is not claimed, and what this pilot is not.

---

## What is frozen

The following are stable and will not change as part of this pilot:

### Protocol
- Protocol version: `postcad-v1`
- Receipt schema version: `"1"`
- All receipt field names and their types
- The canonical serialization used to compute hashes (BTreeMap, deterministic key order)
- The `receipt_hash` value for the canonical pilot inputs: `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb`

### Routing kernel
- Routing kernel version: `postcad-routing-v1`
- Compliance rules: EU MDR, FDA 510k, MHLW, ISO 13485
- Routing strategies: `HighestPriority`, `DeterministicHash`
- Determinism guarantee: same case + same eligible candidates → same routing decision, always
- All `ReasonCode` values (every decision carries a machine-readable reason code)

### Hashing behavior
- SHA-256 used throughout
- Genesis `audit_previous_hash` is 64 zero characters
- `receipt_hash` is computed last, covers all other receipt fields
- Hash chain linkage: each audit entry contains the previous entry's hash

### Endpoint contracts
- All seven endpoints used by the pilot flow are stable:
  - `GET /health`
  - `POST /cases`
  - `POST /cases/:id/route`
  - `GET /receipts/:hash`
  - `POST /dispatch/:hash`
  - `POST /dispatch/:hash/verify`
  - `GET /routes`
- Response shapes for all seven endpoints are frozen

### Canonical fixtures
- `examples/pilot/case.json` — frozen
- `examples/pilot/registry_snapshot.json` — frozen
- `examples/pilot/config.json` — frozen
- `tests/protocol_vectors/` — frozen test vectors for v01–v05

---

## What this pilot is

- A local, deterministic, self-contained demonstration of the routing and verification flow
- A working implementation of the `postcad-v1` protocol on a single test case
- A reproducible operator workflow (reset → start → smoke test → evidence bundle)
- A set of inspectable, stable artifacts that a reviewer can independently verify

---

## What this pilot is not

- A production deployment
- A multi-tenant system
- A system with external network dependencies
- A system with persistent storage beyond the local `data/` directory
- A CAD tool, lab marketplace, or manufacturing operator
- An AI-based system (routing is fully rule-based and deterministic)
- A system that claims to handle all jurisdictions, materials, procedures, or edge cases
- A complete regulatory submission artifact
- A specification for future features

---

## What this packet does not claim

- That the current implementation covers all pilot scenarios beyond the single canonical case
- That the local operator flow is the final production deployment model
- That the evidence bundle is equivalent to a formal audit log
- That passing `result = VERIFIED` constitutes regulatory approval of any kind
- That `selected_candidate_id = pilot-de-001` represents a real manufacturer or real capability check beyond the fixture data

---

## What is not included in this pilot

The following are explicitly out of scope and not present:

- Authentication or authorization
- Multi-user or multi-session state
- External API integrations
- Network communication outside `localhost`
- Database persistence (all state is flat JSON files in `data/`)
- Packaging, containerization, or deployment tooling beyond the local release scripts
- Automated CI triggers for the local operator flow
- Any form of cloud infrastructure
