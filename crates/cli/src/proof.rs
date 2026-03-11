//! Routing proof object — a structured projection of the receipt commitments
//! needed by third parties to independently verify a routing decision.
//!
//! [`RoutingProofObject`] is derived deterministically from a [`RoutingReceipt`].
//! It exposes only the commitment fields required to anchor verification; it does
//! not duplicate the full receipt schema.
//!
//! # Typical usage
//!
//! ```no_run
//! use postcad_cli::{build_routing_proof, verify_routing_proof};
//! // let receipt = …;
//! // let proof = build_routing_proof(&receipt);
//! // verify_routing_proof(&proof, &receipt).expect("proof must match receipt");
//! ```
//!
//! # Field sources
//!
//! | Proof field               | Receipt source                    |
//! |---------------------------|-----------------------------------|
//! | `protocol_version`        | [`PROTOCOL_VERSION`] constant      |
//! | `routing_kernel_version`  | `receipt.routing_kernel_version`   |
//! | `routing_input_hash`      | `receipt.routing_input_hash`       |
//! | `registry_snapshot_hash`  | `receipt.registry_snapshot_hash`   |
//! | `candidate_pool_hash`     | `receipt.candidate_pool_hash`      |
//! | `candidate_order_hash`    | `receipt.candidate_order_hash`     |
//! | `routing_decision_hash`   | `receipt.routing_decision_hash`    |
//! | `selected_candidate_id`   | `receipt.selected_candidate_id`    |
//! | `receipt_hash`            | `receipt.receipt_hash`             |
//! | `audit_entry_hash`        | `receipt.audit_entry_hash`         |
//! | `audit_previous_hash`     | `receipt.audit_previous_hash`      |

use serde::{Deserialize, Serialize};

use crate::manifest::PROTOCOL_VERSION;
use crate::receipt::RoutingReceipt;

// ── Public types ──────────────────────────────────────────────────────────────

/// A structured proof object derived from the commitments in a [`RoutingReceipt`].
///
/// Contains the minimal set of hash commitments needed for a third party to
/// verify that a routing decision was produced deterministically from known
/// inputs, without access to the full receipt artifact.
///
/// Fields are declared in alphabetical order so `serde_json::to_value` produces
/// alphabetically sorted JSON keys, guaranteeing a canonical serial form.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoutingProofObject {
    /// SHA-256 of the audit chain entry for this routing event.
    pub audit_entry_hash: String,
    /// Previous audit chain entry hash (genesis zeros for the first entry).
    pub audit_previous_hash: String,
    /// SHA-256 of the deterministically ordered eligible candidate ID list.
    pub candidate_order_hash: String,
    /// SHA-256 of the canonical candidate pool committed at routing time.
    pub candidate_pool_hash: String,
    /// Top-level PostCAD protocol version that produced this proof.
    pub protocol_version: String,
    /// SHA-256 of the canonical receipt content — the top-level tamper seal.
    pub receipt_hash: String,
    /// SHA-256 of the canonical registry snapshot committed at routing time.
    pub registry_snapshot_hash: String,
    /// SHA-256 of the canonical routing decision outcome fields.
    pub routing_decision_hash: String,
    /// Identifier of the routing kernel algorithm that produced this decision.
    pub routing_kernel_version: String,
    /// SHA-256 of the canonical routing input envelope.
    pub routing_input_hash: String,
    /// ID of the selected routing candidate. `null` when the outcome is refused.
    pub selected_candidate_id: Option<String>,
}

/// A proof verification failure with a stable code and a human-readable message.
#[derive(Debug, PartialEq)]
pub struct ProofVerificationFailure {
    /// Stable machine-readable failure code.
    pub code: &'static str,
    /// Human-readable explanation.
    pub message: String,
}

impl ProofVerificationFailure {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }

    pub fn protocol_version_mismatch(proof: &str, expected: &str) -> Self {
        Self::new(
            "proof_protocol_version_mismatch",
            format!(
                "proof protocol_version {:?} does not match current protocol {:?}",
                proof, expected
            ),
        )
    }

    fn field_mismatch(field: &'static str, proof: &str, receipt: &str) -> Self {
        Self::new(
            "proof_field_mismatch",
            format!(
                "proof.{} {:?} does not match receipt.{} {:?}",
                field, proof, field, receipt
            ),
        )
    }
}

impl std::fmt::Display for ProofVerificationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

// ── Public functions ──────────────────────────────────────────────────────────

/// Builds a [`RoutingProofObject`] from a [`RoutingReceipt`].
///
/// All fields are copied directly from the receipt; no hashing is performed.
/// The proof is therefore deterministic: given the same receipt, `build_routing_proof`
/// always returns an identical object.
pub fn build_routing_proof(receipt: &RoutingReceipt) -> RoutingProofObject {
    RoutingProofObject {
        audit_entry_hash: receipt.audit_entry_hash.clone(),
        audit_previous_hash: receipt.audit_previous_hash.clone(),
        candidate_order_hash: receipt.candidate_order_hash.clone(),
        candidate_pool_hash: receipt.candidate_pool_hash.clone(),
        protocol_version: PROTOCOL_VERSION.to_string(),
        receipt_hash: receipt.receipt_hash.clone(),
        registry_snapshot_hash: receipt.registry_snapshot_hash.clone(),
        routing_decision_hash: receipt.routing_decision_hash.clone(),
        routing_kernel_version: receipt.routing_kernel_version.clone(),
        routing_input_hash: receipt.routing_input_hash.clone(),
        selected_candidate_id: receipt.selected_candidate_id.clone(),
    }
}

/// Verifies that every field in a [`RoutingProofObject`] is consistent with
/// the corresponding field in the provided [`RoutingReceipt`].
///
/// Checks:
/// 1. `protocol_version` matches the current [`PROTOCOL_VERSION`] constant.
/// 2. Each hash commitment field in the proof equals the corresponding receipt field.
///
/// Returns `Ok(())` when all checks pass; `Err(ProofVerificationFailure)` at
/// the first inconsistency, with a stable code and a human-readable message.
pub fn verify_routing_proof(
    proof: &RoutingProofObject,
    receipt: &RoutingReceipt,
) -> Result<(), ProofVerificationFailure> {
    // Check 1: protocol_version must match the current build.
    if proof.protocol_version != PROTOCOL_VERSION {
        return Err(ProofVerificationFailure::protocol_version_mismatch(
            &proof.protocol_version,
            PROTOCOL_VERSION,
        ));
    }

    // Check 2–11: each hash field must match the corresponding receipt field.
    macro_rules! check_field {
        ($field:ident) => {
            if proof.$field != receipt.$field {
                return Err(ProofVerificationFailure::field_mismatch(
                    stringify!($field),
                    &proof.$field,
                    &receipt.$field,
                ));
            }
        };
    }
    macro_rules! check_opt_field {
        ($field:ident) => {
            if proof.$field != receipt.$field {
                return Err(ProofVerificationFailure::new(
                    "proof_field_mismatch",
                    format!(
                        "proof.{} {:?} does not match receipt.{} {:?}",
                        stringify!($field),
                        proof.$field,
                        stringify!($field),
                        receipt.$field,
                    ),
                ));
            }
        };
    }

    check_field!(routing_kernel_version);
    check_field!(routing_input_hash);
    check_field!(registry_snapshot_hash);
    check_field!(candidate_pool_hash);
    check_field!(candidate_order_hash);
    check_field!(routing_decision_hash);
    check_opt_field!(selected_candidate_id);
    check_field!(receipt_hash);
    check_field!(audit_entry_hash);
    check_field!(audit_previous_hash);

    Ok(())
}
