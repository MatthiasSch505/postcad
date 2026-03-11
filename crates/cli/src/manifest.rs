//! Protocol manifest — static machine-readable self-description of the current
//! PostCAD protocol contract.
//!
//! The manifest is the single authoritative source for:
//!   - protocol and kernel version identifiers
//!   - receipt schema version
//!   - canonical serialization rule used for all SHA-256 commitments
//!   - the complete list of committed receipt fields
//!   - audit-chain mode
//!   - whether verify-receipt requires a full routing replay
//!   - the stable machine-readable error codes on the protocol surface
//!
//! Everything here is a compile-time constant; nothing is configured at
//! runtime.  Adding a field to the protocol requires editing this file and
//! regenerating `fixtures/expected_manifest.json`.

use serde::Serialize;

use crate::receipt::RECEIPT_SCHEMA_VERSION;
use postcad_routing::ROUTING_KERNEL_VERSION;

/// Top-level protocol version identifier.
///
/// Distinct from `routing_kernel_version` (which versions the selection
/// algorithm) and `receipt_schema_version` (which versions the JSON shape).
/// `protocol_version` versions the entire verifiable-receipt contract.
pub const PROTOCOL_VERSION: &str = "postcad-v1";

/// Machine-readable self-description of the PostCAD protocol contract.
///
/// Serialised with `serde_json::to_string_pretty`; field declaration order is
/// alphabetical so the JSON output is stable across Rust compiler versions.
#[derive(Debug, Serialize)]
pub struct ProtocolManifest {
    /// Audit-chain algorithm and mode.
    pub audit_chain_mode: &'static str,
    /// Rule used to compute all SHA-256 hash commitments in receipts.
    pub canonical_serialization: &'static str,
    /// Sorted list of all receipt fields covered by the protocol commitment.
    pub committed_receipt_fields: &'static [&'static str],
    /// Top-level protocol contract version.
    pub protocol_version: &'static str,
    /// JSON schema version of the `RoutingReceipt` artifact.
    pub receipt_schema_version: &'static str,
    /// Identifier of the routing selection algorithm.
    pub routing_kernel_version: &'static str,
    /// Sorted list of stable machine-readable error codes on the protocol
    /// surface.  Each code is a permanent identifier; messages may change.
    pub stable_error_codes: &'static [&'static str],
    /// Whether `verify-receipt` requires a deterministic routing replay.
    pub verify_receipt_requires_replay: bool,
}

/// Returns the static protocol manifest for the current build.
///
/// All values are compile-time constants; calling this function twice returns
/// identical results.
pub fn build_manifest() -> ProtocolManifest {
    ProtocolManifest {
        audit_chain_mode: "sha256_hash_chained_append_only",
        canonical_serialization: "sha256(compact_json_sorted_keys_utf8)",
        committed_receipt_fields: COMMITTED_RECEIPT_FIELDS,
        protocol_version: PROTOCOL_VERSION,
        receipt_schema_version: RECEIPT_SCHEMA_VERSION,
        routing_kernel_version: ROUTING_KERNEL_VERSION,
        stable_error_codes: STABLE_ERROR_CODES,
        verify_receipt_requires_replay: true,
    }
}

/// All receipt fields covered by the protocol commitment, in alphabetical order.
const COMMITTED_RECEIPT_FIELDS: &[&str] = &[
    "audit_entry_hash",
    "audit_previous_hash",
    "audit_seq",
    "candidate_order_hash",
    "candidate_pool_hash",
    "case_fingerprint",
    "eligible_candidate_ids_hash",
    "outcome",
    "policy_fingerprint",
    "policy_version",
    "receipt_hash",
    "refusal_code",
    "registry_snapshot_hash",
    "routing_decision_hash",
    "routing_input",
    "routing_input_hash",
    "routing_kernel_version",
    "routing_proof_hash",
    "schema_version",
    "selected_candidate_id",
    "selection_input_candidate_ids_hash",
];

/// Stable machine-readable error codes, in alphabetical order.
///
/// `receipt_hash_mismatch` is intentionally excluded: the code is defined in
/// the source but is unreachable in the current verification path
/// (`receipt_canonicalization_mismatch` is fired instead).
const STABLE_ERROR_CODES: &[&str] = &[
    "audit_entry_hash_mismatch",
    "audit_previous_hash_mismatch",
    "candidate_order_hash_mismatch",
    "candidate_pool_hash_mismatch",
    "case_fingerprint_mismatch",
    "case_parse_failed",
    "eligible_candidate_ids_hash_mismatch",
    "invalid_receipt_schema_version",
    "missing_receipt_schema_version",
    "policy_bundle_parse_failed",
    "policy_fingerprint_mismatch",
    "policy_version_mismatch",
    "receipt_canonicalization_mismatch",
    "receipt_parse_failed",
    "registry_snapshot_hash_mismatch",
    "routing_decision_hash_mismatch",
    "routing_decision_replay_mismatch",
    "routing_input_hash_mismatch",
    "routing_kernel_version_mismatch",
    "routing_proof_hash_mismatch",
    "selection_input_candidate_ids_hash_mismatch",
    "unsupported_receipt_schema_version",
];
