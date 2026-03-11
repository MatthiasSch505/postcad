//! Protocol version constants for PostCAD.
//!
//! `POSTCAD_PROTOCOL_VERSION` is the semantic versioning identifier for the
//! full verifiable-receipt protocol contract. It is independent of:
//!   - `routing_kernel_version` — the selection algorithm version
//!   - `receipt_schema_version` — the JSON schema shape version
//!   - the `"postcad-v1"` label used in proof objects and the manifest
//!
//! Bump this constant when the protocol contract changes in a way that makes
//! old receipts unverifiable with new software (breaking change).

/// Semantic version of the PostCAD verifiable-receipt protocol.
pub const POSTCAD_PROTOCOL_VERSION: &str = "1.0";
