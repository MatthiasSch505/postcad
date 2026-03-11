# PostCAD Development Bundle

---

## Purpose

This document describes the current deterministic pilot stack and the canonical development workflow for PostCAD Protocol v1. It is the authoritative reference for a new engineer or operator joining the project.

Pilot readiness assessment: [`docs/pilot_maturity_check.md`](pilot_maturity_check.md) — machine-readable gap list: [`docs/pilot_gap_list.json`](pilot_gap_list.json).

---

## Locked Surfaces

The following surfaces are frozen. Changes require explicit approval and must not alter receipt hashes, verification behavior, or protocol artifact shapes.

| Surface | Location |
|---------|----------|
| Routing kernel | `crates/routing/` |
| Receipt / refusal artifact schema | `crates/cli/src/receipt.rs` |
| Verification path | `crates/cli/src/verifier.rs` |
| Registry snapshot export | `crates/cli/src/registry_export.rs` |
| HTTP pilot service | `crates/service/` |
| Pilot fixture bundle | `examples/pilot/` |
| Local deployment path | `docs/local_service_run.md` |

---

## Canonical Commands

### Run all tests

```bash
cargo test --workspace
```

### Run pilot HTTP endpoint tests

```bash
cargo test -p postcad-service --test pilot_http_tests
```

### Run pilot bundle smoke tests (value-for-value fixture comparison)

```bash
cargo test -p postcad-service --test pilot_bundle_smoke_test
```

### Export a registry snapshot

```bash
./target/debug/postcad-cli registry-export \
  --input  examples/pilot/registry_snapshot.json \
  --output /tmp/snapshot_export.json
```

### Route a case (CLI, pilot fixtures)

```bash
./target/debug/postcad-cli route-case-from-registry --json \
  --case     examples/pilot/case.json \
  --registry examples/pilot/registry_snapshot.json \
  --config   examples/pilot/config.json
```

Expected: receipt matching `examples/pilot/expected_routed.json`.

### Verify a receipt (CLI, pilot fixtures)

```bash
./target/debug/postcad-cli verify-receipt --json \
  --receipt    examples/pilot/expected_routed.json \
  --case       examples/pilot/case.json \
  --policy     examples/pilot/derived_policy.json \
  --candidates examples/pilot/derived_policy.json
```

Expected: `{"result":"VERIFIED"}`

### Start the service (binary)

```bash
cargo build --bin postcad-service
./target/debug/postcad-service
```

### Start the service (Docker Compose)

```bash
docker compose up
```

### Stop the service (Docker Compose)

```bash
docker compose down
```

---

## Repository Map

```
crates/
  core/           shared domain types: Case, Decision, ReasonCode, …
  registry/       ManufacturerRegistry trait and InMemoryRegistry impl
  compliance/     ComplianceRule trait + built-in country rules
  routing/        RoutingEngine + HighestPriority / DeterministicHash strategies
  audit/          hash-chained append-only audit log
  cli/
    src/lib.rs    all public library functions (route_case_from_*, verify_receipt_from_*, …)
    src/main.rs   CLI subcommand dispatcher
    src/bin/demo.rs  standalone demo runner
    tests/        golden, CLI artifact contract, protocol vector, fuzz, closure sweep tests
  service/
    src/lib.rs    app() router; exports all endpoints
    src/handlers.rs  per-endpoint handlers; no business logic
    tests/        service contract, pilot HTTP, pilot bundle smoke tests

examples/
  pilot/
    case.json             routing case input
    registry_snapshot.json  three DE manufacturers
    config.json           routing config (jurisdiction DE)
    derived_policy.json   RoutingPolicyBundle derived from registry (for verify-receipt)
    expected_routed.json  locked canonical receipt
    expected_verify.json  locked canonical verify response
    README.md             pilot bundle usage

tests/
  protocol_vectors/   frozen v01–v05 conformance inputs and expected receipts

fixtures/             canonical test fixtures used by golden and contract tests

docs/
  local_service_run.md   one-command Docker run path
  deployment.md          full deployment reference
  pilot_contract_v1.md   pilot input/output contract
  postcad_protocol_v1.md PostCAD Protocol v1 specification
  handoff_status.json    current phase and definition of done

Makefile               build / test / docker targets
docker/Dockerfile      multi-stage Rust build → debian:bookworm-slim runtime
docker-compose.yml     single-container local deployment
.env.example           POSTCAD_ADDR, RUST_LOG
```

---

## Red Lines

The following must not be changed without explicit protocol approval:

- **No protocol changes.** Receipt field list, hash algorithm, and canonical serialization are frozen.
- **No routing logic changes.** Candidate selection, ordering, and determinism invariants are frozen.
- **No schema changes.** Adding or removing receipt fields requires a full protocol version bump and fixture regeneration.
- **Preserve canonical ordering.** All JSON fields used in SHA-256 commitments must remain sorted.
- **Preserve deterministic JSON.** Do not introduce timestamps, UUIDs, or randomness into the routing or verification path.
- **Reuse existing library paths.** Do not duplicate `route_case_from_registry_json`, `verify_receipt_from_policy_json`, or any other library function in a parallel implementation.

---

## Safe Next Work Categories

The following categories are safe to pursue without protocol risk:

- **Docs** — clarifications, examples, operator guides
- **Packaging** — CI pipelines, release scripts, image tagging
- **Operator tooling** — dashboards that read receipts, audit log viewers
- **Test hardening** — additional edge-case inputs, property tests, negative tests
- **Observability** — structured logging, span IDs, metrics; only if they do not enter the canonical routing path
- **Productization layers** — auth, rate limiting, tenant isolation; only as wrappers that call the existing service, not replacements

---

## Unsafe Drift Categories

The following require explicit protocol-level review before any code is written:

- Auth redesign that changes receipt structure or routing inputs
- Database-first redesign (routing decisions must remain stateless)
- UI or dashboard that re-implements routing or verification logic client-side
- Async job system that introduces non-determinism into receipt generation
- Cloud infra expansion that changes how receipts are signed or stored
- Schema drift (any field addition, removal, or rename on a committed receipt)
- Protocol-version churn (bumping protocol version without a full conformance test sweep)
- Routing heuristic changes (priority overrides, ML-based selection, tie-breaking changes)
