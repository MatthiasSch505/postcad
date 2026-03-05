# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

**Post-CAD Layer** — a deterministic, rule-based platform that sits after dental CAD design. It verifies manufacturer certifications, checks regulatory constraints by country, routes cases to eligible manufacturers, and records an immutable audit trail.

The platform does not own manufacturing or clinical liability. No AI decision-making. Every decision must carry a `ReasonCode`.

## Commands

```bash
# Build all crates
cargo build

# Run the demo workflow
cargo run -p postcad-cli

# Run all tests
cargo test

# Test a specific crate
cargo test -p postcad-compliance

# Run a single test by name
cargo test -p postcad-compliance eu_mdr_rule_fails_without_ce_mark

# Lint
cargo clippy --all-targets --all-features

# Format
cargo fmt
```

## Architecture

Cargo workspace with 6 crates (`crates/`). Dependency order (no cycles):

```
postcad-core
  └─ postcad-registry (uses core types)
       ├─ postcad-compliance (evaluates rules against registry + case)
       └─ postcad-routing   (selects manufacturer from eligible list)
postcad-audit (uses core types only)
postcad-cli   (depends on all above)
```

### `postcad-core`
Shared domain types: `Case`, `CaseId`, `ManufacturerId`, `Country`, `Material`, `ProcedureType`, `FileType`, `CertificationType`, `ReasonCode`, and the `Decision<T>` wrapper. Every pipeline decision must be wrapped in `Decision<T>` to carry a `ReasonCode` and timestamp.

### `postcad-registry`
`Manufacturer` and `Certification` structs. The `ManufacturerRegistry` trait with `InMemoryRegistry` impl. Key method: `find_capable(material, procedure)` — returns manufacturers that are active and support the required material+procedure combination. Certification validity checks: `is_valid_at(DateTime)` and `covers_country(Country)`.

### `postcad-compliance`
`ComplianceRule` trait — stateless, deterministic. `ComplianceEngine` holds a `Vec<Box<dyn ComplianceRule>>` and runs all rules whose `applies_to(case)` returns true. Built-in rules:
- `ManufacturerActiveRule` — always applies
- `CapabilityRule` — always applies
- `EuMdrCertificationRule` — applies to EU member countries (CE Mark required)
- `FdaClearanceRule` — applies to `Country::UnitedStates` (FDA 510k required)
- `MhlwApprovalRule` — applies to `Country::Japan`
- `Iso13485Rule` — always applies (baseline quality cert)

Add new country rules by implementing `ComplianceRule` and registering in `ComplianceEngine::default_rules()`.

### `postcad-routing`
`RoutingEngine` with pluggable `RoutingStrategy`. Takes a `Vec<&Manufacturer>` already filtered by compliance. All strategies sort by `(priority, id)` first for stable ordering before selection:
- `HighestPriority` — picks index 0 (lowest priority number)
- `DeterministicHash` — hashes `CaseId` to select index, distributing load without state

Returns `Decision<Option<RoutingDecision>>`. No manufacturer = `ReasonCode::RouteNoEligibleManufacturer`.

### `postcad-audit`
Hash-chained append-only log. Each `AuditEntry` contains: sequence number, timestamp, `AuditEvent`, `previous_hash`, and `hash` (SHA-256 of all fields). Genesis previous_hash is 64 zeros. `verify_chain()` recomputes every hash and checks linkage. `AuditEvent` is a tagged enum covering the full pipeline lifecycle.

## Key Invariants

- `ReasonCode` must be recorded for every decision — no silent pass/fail.
- Routing must be deterministic: same case + same eligible list = same manufacturer, always.
- Audit entries are never mutated; only appended.
- Compliance rules are stateless — they receive all context they need as arguments.
- `ComplianceEngine::check()` runs against one `(case, manufacturer)` pair at a time.
