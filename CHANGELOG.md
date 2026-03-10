# PostCAD Changelog

## v0.3-verifiable-routing-core

CLI routing receipt artifact and full verifier subprocess contract coverage.

### Completed

**Receipt commitments**
- `route-case --json` emits a canonical deterministic receipt with `schema_version`,
  `receipt_hash` (artifact-integrity hash covering all fields), and three candidate
  commitment hashes: `candidate_pool_hash`, `eligible_candidate_ids_hash`,
  `selection_input_candidate_ids_hash`

**Verifier CLI (`verify-receipt --json`)**
- Stable machine-readable `code` field on every `VERIFICATION FAILED` response
- Schema version enforcement: `missing_receipt_schema_version`,
  `invalid_receipt_schema_version`, `unsupported_receipt_schema_version`
- Artifact integrity check fires before all semantic checks: `receipt_hash_mismatch`
- Field-level tamper codes: `routing_proof_hash_mismatch`,
  `candidate_pool_hash_mismatch`, `eligible_candidate_ids_hash_mismatch`,
  `selection_input_candidate_ids_hash_mismatch`
- Audit-chain binding codes: `audit_entry_hash_mismatch`,
  `audit_previous_hash_mismatch`
- Structural parse failure on missing required field: `receipt_parse_failed`

**CLI subprocess contract tests (cli_errors.rs)**
- All four fixture scenarios covered: routed domestic, routed cross-border,
  refused compliance-failed, refused no-eligible-candidates
- Verified success envelope locked to exactly `{"result": "VERIFIED"}`
- Every verifier failure code covered at the subprocess boundary
- Parser tolerance locked: extra top-level fields, different key order,
  pretty-printed whitespace all pass verification
- Receipt hash determinism locked across separate subprocess invocations

---

## v0.1-foundation

Initial workspace: core domain types, compliance rule engine, deterministic
routing kernel, append-only hash-chained audit log.
