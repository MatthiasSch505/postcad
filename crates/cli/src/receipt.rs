//! Public receipt contract for the `route-case` CLI command.
//!
//! This module defines the stable, external output schema. It has no dependency
//! on internal routing or audit domain types. Any change here is a breaking
//! change to the CLI surface; internal refactors that do not affect receipt
//! fields should not require touching this file.

use serde::{Deserialize, Serialize};

/// Stable, verification-ready routing receipt emitted by `route-case`.
///
/// Both routed and refused outcomes share the same top-level schema.
/// `selected_candidate_id` is `null` for refused outcomes; `refusal_code` is
/// `null` for routed outcomes. The optional `refusal` section is omitted from
/// serialized JSON when absent (routed outcome).
///
/// All hash fields are lowercase hex SHA-256 digests (64 characters).
///
/// Audit chain fields (`audit_seq`, `audit_entry_hash`, `audit_previous_hash`)
/// allow consumers to anchor this receipt to an append-only audit log. The
/// entry hash is SHA-256 of `{seq, event, previous_hash}` and is independently
/// verifiable without access to the routing engine.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoutingReceipt {
    /// `"routed"` or `"refused"`.
    pub outcome: String,
    /// SHA-256 of the canonical case payload.
    pub case_fingerprint: String,
    /// SHA-256 of the canonical policy configuration.
    pub policy_fingerprint: String,
    /// SHA-256 of the canonical routing decision fingerprint.
    pub routing_proof_hash: String,
    /// SHA-256 of the canonical candidate snapshot used at routing time.
    pub candidate_snapshot_hash: String,
    /// ID of the selected routing candidate. `null` when refused.
    pub selected_candidate_id: Option<String>,
    /// Machine-readable refusal code. `null` when routed.
    pub refusal_code: Option<String>,
    /// Sequence number of this entry in the audit log.
    pub audit_seq: u64,
    /// SHA-256 of `{audit_seq, event, audit_previous_hash}`.
    pub audit_entry_hash: String,
    /// Hash of the preceding audit entry (genesis zeros for the first entry).
    pub audit_previous_hash: String,
    /// Detailed refusal context. Present only when `outcome == "refused"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<RefusalDetail>,
}

/// Result of verifying a routing receipt against the original inputs.
///
/// `result` is `"valid"` when the recomputed receipt matches the provided one
/// field-for-field. `"mismatch"` when one or more fields differ; the
/// `mismatched_fields` array lists every field name that did not match.
#[derive(Debug, Serialize, PartialEq)]
pub struct ReceiptVerificationResult {
    /// `"valid"` or `"mismatch"`.
    pub result: String,
    /// Field names that differed. Absent when `result == "valid"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mismatched_fields: Option<Vec<String>>,
}

/// Detailed context for a refused routing outcome.
///
/// Present only when `outcome == "refused"`. Identifies which compliance or
/// policy constraint was not met and which candidate IDs were evaluated.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RefusalDetail {
    /// Human-readable explanation of the refusal.
    pub message: String,
    /// All candidate IDs that were evaluated before the refusal was issued.
    pub evaluated_candidate_ids: Vec<String>,
    /// The specific constraint gate that rejected the case.
    /// Stable values: `"compliance_gate"`, `"routing_policy"`,
    /// `"no_input_candidates"`, `"case_validation"`, `"unknown"`.
    pub failed_constraint: String,
}
