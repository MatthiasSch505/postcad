//! Protocol manifest — static machine-readable self-description of the current
//! PostCAD protocol contract.
//!
//! The manifest is the single authoritative source for:
//!   - protocol and kernel version identifiers
//!   - receipt schema version
//!   - canonical serialization rule used for all SHA-256 commitments
//!   - the complete list of committed receipt fields
//!   - schema hashes (receipt, proof, refusal code set)
//!   - manifest fingerprint
//!   - audit-chain mode
//!   - whether verify-receipt requires a full routing replay
//!   - the stable machine-readable error codes on the protocol surface
//!
//! Everything here is a compile-time constant; nothing is configured at
//! runtime.  Adding a field to the protocol requires editing this file and
//! regenerating `fixtures/expected_manifest.json`.

use hex::encode;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::receipt::RECEIPT_SCHEMA_VERSION;
use postcad_core::refusal_code_set_hash as core_refusal_code_set_hash;
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
    /// SHA-256 of (protocol_version ‖ receipt_schema_hash ‖ proof_schema_hash
    /// ‖ refusal_code_set_hash ‖ routing_kernel_version), joined with `\n`.
    /// Uniquely identifies this exact protocol configuration.
    pub manifest_fingerprint: String,
    /// SHA-256 of the canonical proof object field list (fields joined with `\n`).
    pub proof_schema_hash: String,
    /// Top-level protocol contract version.
    pub protocol_version: &'static str,
    /// SHA-256 of the committed receipt field list (fields joined with `\n`).
    pub receipt_schema_hash: String,
    /// JSON schema version of the `RoutingReceipt` artifact.
    pub receipt_schema_version: &'static str,
    /// SHA-256 of the canonical routing refusal code set (codes joined with `\n`).
    pub refusal_code_set_hash: String,
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
/// All values are deterministic; calling this function twice returns
/// identical results.
pub fn build_manifest() -> ProtocolManifest {
    let receipt_schema_hash = compute_receipt_schema_hash();
    let proof_schema_hash = compute_proof_schema_hash();
    let refusal_code_set_hash = core_refusal_code_set_hash();
    let manifest_fingerprint = compute_manifest_fingerprint(
        PROTOCOL_VERSION,
        &receipt_schema_hash,
        &proof_schema_hash,
        &refusal_code_set_hash,
        ROUTING_KERNEL_VERSION,
    );
    ProtocolManifest {
        audit_chain_mode: "sha256_hash_chained_append_only",
        canonical_serialization: "sha256(compact_json_sorted_keys_utf8)",
        committed_receipt_fields: COMMITTED_RECEIPT_FIELDS,
        manifest_fingerprint,
        proof_schema_hash,
        protocol_version: PROTOCOL_VERSION,
        receipt_schema_hash,
        receipt_schema_version: RECEIPT_SCHEMA_VERSION,
        refusal_code_set_hash,
        routing_kernel_version: ROUTING_KERNEL_VERSION,
        stable_error_codes: STABLE_ERROR_CODES,
        verify_receipt_requires_replay: true,
    }
}

// ── Schema hash helpers ───────────────────────────────────────────────────────

/// SHA-256 of the committed receipt field list, fields joined with `\n`.
pub fn compute_receipt_schema_hash() -> String {
    encode(Sha256::digest(COMMITTED_RECEIPT_FIELDS.join("\n").as_bytes()))
}

/// Canonical ordered proof object field names, sorted alphabetically.
///
/// Mirrors `RoutingProofObject` in `crates/cli/src/proof.rs`; must be kept
/// in sync when proof fields change.
const PROOF_FIELDS: &[&str] = &[
    "audit_entry_hash",
    "audit_previous_hash",
    "candidate_order_hash",
    "candidate_pool_hash",
    "protocol_version",
    "receipt_hash",
    "registry_snapshot_hash",
    "routing_decision_hash",
    "routing_kernel_version",
    "routing_input_hash",
    "selected_candidate_id",
];

/// SHA-256 of the proof object field list, fields joined with `\n`.
pub fn compute_proof_schema_hash() -> String {
    encode(Sha256::digest(PROOF_FIELDS.join("\n").as_bytes()))
}

/// SHA-256 of the five manifest inputs joined with `\n`.
fn compute_manifest_fingerprint(
    protocol_version: &str,
    receipt_schema_hash: &str,
    proof_schema_hash: &str,
    refusal_code_set_hash: &str,
    routing_kernel_version: &str,
) -> String {
    let input = [
        protocol_version,
        receipt_schema_hash,
        proof_schema_hash,
        refusal_code_set_hash,
        routing_kernel_version,
    ]
    .join("\n");
    encode(Sha256::digest(input.as_bytes()))
}

// ── Field lists ───────────────────────────────────────────────────────────────

/// All receipt fields covered by the protocol commitment, in alphabetical order.
pub(crate) const COMMITTED_RECEIPT_FIELDS: &[&str] = &[
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
pub(crate) const STABLE_ERROR_CODES: &[&str] = &[
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
    "protocol_version_mismatch",
    "receipt_canonicalization_mismatch",
    "receipt_parse_failed",
    "registry_snapshot_hash_mismatch",
    "routing_decision_hash_mismatch",
    "routing_decision_replay_mismatch",
    "routing_input_hash_mismatch",
    "routing_kernel_version_mismatch",
    "routing_proof_hash_mismatch",
    "selection_input_candidate_ids_hash_mismatch",
    "unknown_refusal_code",
    "unsupported_receipt_schema_version",
];
