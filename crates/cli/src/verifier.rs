//! Structured verification result types for the `verify-receipt` command.
//!
//! [`VerificationFailure`] carries a **stable machine-readable code** alongside
//! a human-readable message. The code never changes once published; the message
//! may be refined over time for clarity.
//!
//! This module has no dependency on internal routing or audit domain types.

/// A verification failure with a stable code and a human-readable message.
///
/// The `code` field is a stable snake_case identifier that callers can
/// pattern-match on programmatically. `message` is for human display only.
#[derive(Debug, PartialEq)]
pub struct VerificationFailure {
    /// Stable machine-readable failure code.
    pub code: &'static str,
    /// Human-readable explanation, suitable for CLI output.
    pub message: String,
}

impl VerificationFailure {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }

    // ── Parse failures ────────────────────────────────────────────────────────

    pub fn receipt_parse_failed(detail: impl Into<String>) -> Self {
        Self::new("receipt_parse_failed", format!("receipt parse error: {}", detail.into()))
    }

    pub fn case_parse_failed(detail: impl Into<String>) -> Self {
        Self::new("case_parse_failed", format!("case parse error: {}", detail.into()))
    }

    pub fn policy_bundle_parse_failed(detail: impl Into<String>) -> Self {
        Self::new(
            "policy_bundle_parse_failed",
            format!("policy parse error: {}", detail.into()),
        )
    }

    // ── Fingerprint mismatches ────────────────────────────────────────────────

    pub fn case_fingerprint_mismatch(receipt: &str, computed: &str) -> Self {
        Self::new(
            "case_fingerprint_mismatch",
            format!(
                "case_fingerprint mismatch: receipt has {}, computed {}",
                receipt, computed
            ),
        )
    }

    pub fn policy_fingerprint_mismatch(receipt: &str, computed: &str) -> Self {
        Self::new(
            "policy_fingerprint_mismatch",
            format!(
                "policy_fingerprint mismatch: receipt has {}, computed {}",
                receipt, computed
            ),
        )
    }

    // ── Routing proof mismatch ────────────────────────────────────────────────

    pub fn routing_proof_hash_mismatch(receipt: &str, computed: &str) -> Self {
        Self::new(
            "routing_proof_hash_mismatch",
            format!(
                "routing_proof_hash mismatch: receipt has {}, computed {}",
                receipt, computed
            ),
        )
    }

    // ── Audit chain mismatches ────────────────────────────────────────────────

    pub fn audit_entry_hash_mismatch(receipt: &str, computed: &str) -> Self {
        Self::new(
            "audit_entry_hash_mismatch",
            format!(
                "audit_entry_hash mismatch: receipt has {}, computed {}",
                receipt, computed
            ),
        )
    }

    pub fn audit_previous_hash_mismatch(receipt: &str, computed: &str) -> Self {
        Self::new(
            "audit_previous_hash_mismatch",
            format!(
                "audit_previous_hash mismatch: receipt has {}, computed {}",
                receipt, computed
            ),
        )
    }
}

impl std::fmt::Display for VerificationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
