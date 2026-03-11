//! Canonical routing refusal codes for the PostCAD protocol.
//!
//! These are the stable machine-readable codes emitted when no eligible
//! manufacturer can be selected. They are distinct from verification error
//! codes (which live in the CLI crate).
//!
//! The set is sorted alphabetically and frozen at protocol version 1.0.

use hex::encode;
use sha2::{Digest, Sha256};

/// Canonical routing refusal codes, sorted alphabetically.
///
/// Each code is a permanent identifier. New codes may be added in a backwards-
/// compatible way; existing codes must never be removed or renamed.
pub const REFUSAL_CODES: &[&str] = &[
    "attestation_failed",
    "no_active_manufacturer",
    "no_capability_match",
    "no_eligible_candidates",
    "no_eligible_manufacturer",
    "no_jurisdiction_match",
    "no_material_match",
];

/// Returns true if `code` is a known canonical refusal code.
pub fn is_known_refusal_code(code: &str) -> bool {
    REFUSAL_CODES.contains(&code)
}

/// SHA-256 of the canonical refusal code set (codes joined with `\n`, no trailing newline).
///
/// Stable across builds as long as `REFUSAL_CODES` does not change.
pub fn refusal_code_set_hash() -> String {
    let input = REFUSAL_CODES.join("\n");
    encode(Sha256::digest(input.as_bytes()))
}
