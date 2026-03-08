use sha2::{Digest, Sha256};

use crate::fingerprint::RoutingDecisionFingerprint;
use crate::receipt::RoutingAuditReceipt;
use crate::proof::RoutingProof;

/// Outcome of verifying a routing audit receipt against its proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationResult {
    Valid,
    Invalid(VerificationFailure),
}

impl VerificationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, VerificationResult::Valid)
    }
}

/// The specific reason a receipt failed verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationFailure {
    /// SHA-256(canonical_payload) does not equal hash_hex.
    ProofHashMismatch,
    /// canonical_payload could not be parsed as a RoutingDecisionFingerprint.
    PayloadUnparseable,
    /// A fingerprint field does not match the corresponding receipt field.
    FieldMismatch { field: &'static str },
}

/// Verifies that a `RoutingAuditReceipt` is consistent with its `RoutingProof`.
///
/// Steps performed:
/// 1. Recompute SHA-256(proof.canonical_payload) and compare to proof.hash_hex.
/// 2. Parse proof.canonical_payload as a RoutingDecisionFingerprint.
/// 3. Cross-check every overlapping field between the fingerprint and the receipt.
///
/// Returns `VerificationResult::Valid` only if all checks pass.
/// The first failing check short-circuits and returns its specific failure.
pub fn verify_receipt(receipt: &RoutingAuditReceipt, proof: &RoutingProof) -> VerificationResult {
    // Step 1: recompute and compare the proof hash.
    if sha256_hex(&proof.canonical_payload) != proof.hash_hex {
        return VerificationResult::Invalid(VerificationFailure::ProofHashMismatch);
    }

    // Step 2: parse the canonical payload back into a fingerprint.
    let fp: RoutingDecisionFingerprint = match serde_json::from_str(&proof.canonical_payload) {
        Ok(f) => f,
        Err(_) => return VerificationResult::Invalid(VerificationFailure::PayloadUnparseable),
    };

    // Step 3: cross-check each field.
    if fp.case_id != receipt.case_id {
        return VerificationResult::Invalid(VerificationFailure::FieldMismatch {
            field: "case_id",
        });
    }

    if fp.jurisdiction != receipt.jurisdiction {
        return VerificationResult::Invalid(VerificationFailure::FieldMismatch {
            field: "jurisdiction",
        });
    }

    if fp.selected_manufacturer_id != receipt.selected_manufacturer_id {
        return VerificationResult::Invalid(VerificationFailure::FieldMismatch {
            field: "selected_manufacturer_id",
        });
    }

    if fp.candidate_ids_considered != receipt.candidate_ids_considered {
        return VerificationResult::Invalid(VerificationFailure::FieldMismatch {
            field: "candidate_ids_considered",
        });
    }

    if fp.refusal_code != receipt.refusal_code {
        return VerificationResult::Invalid(VerificationFailure::FieldMismatch {
            field: "refusal_code",
        });
    }

    // Derive expected final_status from the receipt and compare.
    let expected_status = if receipt.selected_manufacturer_id.is_some() {
        "selected"
    } else {
        "refused"
    };
    if fp.final_status != expected_status {
        return VerificationResult::Invalid(VerificationFailure::FieldMismatch {
            field: "final_status",
        });
    }

    VerificationResult::Valid
}

fn sha256_hex(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    format!("{:x}", digest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcad_core::{
        Case, Country, DentalCase, FileType, ManufacturerEligibility, ManufacturingLocation,
        Material, ProcedureType, RoutingCandidate, RoutingCandidateId, RoutingPolicy,
    };

    use crate::service::route_case_with_audit;

    fn valid_case() -> Case {
        Case::new(DentalCase {
            patient_country: Country::UnitedStates,
            manufacturer_country: Country::Germany,
            material: Material::Zirconia,
            procedure: ProcedureType::Crown,
            file_type: FileType::Stl,
        })
    }

    fn invalid_case() -> Case {
        Case::new(DentalCase {
            patient_country: Country::UnitedStates,
            manufacturer_country: Country::Germany,
            material: Material::Zirconia,
            procedure: ProcedureType::Crown,
            file_type: FileType::Other(String::new()),
        })
    }

    fn domestic(rc_id: &str, mfr_id: &str) -> RoutingCandidate {
        RoutingCandidate::new(
            RoutingCandidateId::new(rc_id),
            mfr_id,
            ManufacturingLocation::Domestic,
            true,
            ManufacturerEligibility::Eligible,
        )
    }

    // ── happy path ────────────────────────────────────────────────────────────

    #[test]
    fn selected_receipt_and_proof_verify() {
        let result = route_case_with_audit(
            &valid_case(),
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &[domestic("rc-1", "mfr-01")],
            None,
        );
        assert_eq!(
            verify_receipt(&result.audit_receipt, &result.proof),
            VerificationResult::Valid
        );
    }

    #[test]
    fn refused_receipt_and_proof_verify() {
        let result = route_case_with_audit(
            &invalid_case(),
            "US",
            RoutingPolicy::AllowDomesticOnly,
            &[domestic("rc-1", "mfr-01")],
            None,
        );
        assert_eq!(
            verify_receipt(&result.audit_receipt, &result.proof),
            VerificationResult::Valid
        );
    }

    #[test]
    fn no_eligible_candidate_receipt_and_proof_verify() {
        let result = route_case_with_audit(
            &valid_case(),
            "JP",
            RoutingPolicy::AllowDomesticOnly,
            &[], // no candidates → NoEligibleCandidate
            None,
        );
        assert_eq!(
            verify_receipt(&result.audit_receipt, &result.proof),
            VerificationResult::Valid
        );
    }

    #[test]
    fn verify_is_deterministic_for_same_input() {
        let result = route_case_with_audit(
            &valid_case(),
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &[domestic("rc-1", "mfr-01")],
            None,
        );
        let a = verify_receipt(&result.audit_receipt, &result.proof);
        let b = verify_receipt(&result.audit_receipt, &result.proof);
        assert_eq!(a, b);
    }

    // ── proof hash tamper ─────────────────────────────────────────────────────

    #[test]
    fn tampered_hash_hex_returns_proof_hash_mismatch() {
        let result = route_case_with_audit(
            &valid_case(),
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &[domestic("rc-1", "mfr-01")],
            None,
        );
        let mut proof = result.proof.clone();
        proof.hash_hex =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        assert_eq!(
            verify_receipt(&result.audit_receipt, &proof),
            VerificationResult::Invalid(VerificationFailure::ProofHashMismatch)
        );
    }

    #[test]
    fn tampered_canonical_payload_returns_proof_hash_mismatch() {
        let result = route_case_with_audit(
            &valid_case(),
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &[domestic("rc-1", "mfr-01")],
            None,
        );
        let mut proof = result.proof.clone();
        proof.canonical_payload.push_str(",\"tampered\":true");
        assert_eq!(
            verify_receipt(&result.audit_receipt, &proof),
            VerificationResult::Invalid(VerificationFailure::ProofHashMismatch)
        );
    }

    // ── field mismatch ────────────────────────────────────────────────────────

    #[test]
    fn mismatched_case_id_in_receipt_returns_field_mismatch() {
        let result = route_case_with_audit(
            &valid_case(),
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &[domestic("rc-1", "mfr-01")],
            None,
        );
        let mut receipt = result.audit_receipt.clone();
        receipt.case_id = "00000000-0000-0000-0000-000000000000".to_string();
        assert_eq!(
            verify_receipt(&receipt, &result.proof),
            VerificationResult::Invalid(VerificationFailure::FieldMismatch { field: "case_id" })
        );
    }

    #[test]
    fn mismatched_jurisdiction_in_receipt_returns_field_mismatch() {
        let result = route_case_with_audit(
            &valid_case(),
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &[domestic("rc-1", "mfr-01")],
            None,
        );
        let mut receipt = result.audit_receipt.clone();
        receipt.jurisdiction = "JP".to_string();
        assert_eq!(
            verify_receipt(&receipt, &result.proof),
            VerificationResult::Invalid(VerificationFailure::FieldMismatch {
                field: "jurisdiction"
            })
        );
    }

    #[test]
    fn mismatched_selected_manufacturer_in_receipt_returns_field_mismatch() {
        let result = route_case_with_audit(
            &valid_case(),
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &[domestic("rc-1", "mfr-01")],
            None,
        );
        let mut receipt = result.audit_receipt.clone();
        receipt.selected_manufacturer_id = Some("mfr-tampered".to_string());
        assert_eq!(
            verify_receipt(&receipt, &result.proof),
            VerificationResult::Invalid(VerificationFailure::FieldMismatch {
                field: "selected_manufacturer_id"
            })
        );
    }

    #[test]
    fn mismatched_candidate_ids_in_receipt_returns_field_mismatch() {
        let result = route_case_with_audit(
            &valid_case(),
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &[domestic("rc-1", "mfr-01")],
            None,
        );
        let mut receipt = result.audit_receipt.clone();
        receipt.candidate_ids_considered = vec!["rc-tampered".to_string()];
        assert_eq!(
            verify_receipt(&receipt, &result.proof),
            VerificationResult::Invalid(VerificationFailure::FieldMismatch {
                field: "candidate_ids_considered"
            })
        );
    }

    #[test]
    fn mismatched_refusal_code_in_receipt_returns_field_mismatch() {
        let result = route_case_with_audit(
            &invalid_case(),
            "US",
            RoutingPolicy::AllowDomesticOnly,
            &[domestic("rc-1", "mfr-01")],
            None,
        );
        let mut receipt = result.audit_receipt.clone();
        receipt.refusal_code = Some("tampered_code".to_string());
        assert_eq!(
            verify_receipt(&receipt, &result.proof),
            VerificationResult::Invalid(VerificationFailure::FieldMismatch {
                field: "refusal_code"
            })
        );
    }

    #[test]
    fn clearing_refusal_code_in_receipt_returns_field_mismatch() {
        let result = route_case_with_audit(
            &invalid_case(),
            "US",
            RoutingPolicy::AllowDomesticOnly,
            &[domestic("rc-1", "mfr-01")],
            None,
        );
        let mut receipt = result.audit_receipt.clone();
        receipt.refusal_code = None;
        // Either refusal_code mismatch or final_status mismatch fires first.
        assert!(!verify_receipt(&receipt, &result.proof).is_valid());
    }

    #[test]
    fn promoting_refused_to_selected_in_receipt_returns_field_mismatch() {
        let result = route_case_with_audit(
            &invalid_case(),
            "US",
            RoutingPolicy::AllowDomesticOnly,
            &[domestic("rc-1", "mfr-01")],
            None,
        );
        let mut receipt = result.audit_receipt.clone();
        // Pretend the outcome was a selection.
        receipt.selected_manufacturer_id = Some("mfr-01".to_string());
        receipt.refusal_code = None;
        receipt.refusal_message = None;
        assert!(!verify_receipt(&receipt, &result.proof).is_valid());
    }

    // ── is_valid helper ───────────────────────────────────────────────────────

    #[test]
    fn is_valid_returns_true_for_valid_result() {
        let result = route_case_with_audit(
            &valid_case(),
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &[domestic("rc-1", "mfr-01")],
            None,
        );
        assert!(verify_receipt(&result.audit_receipt, &result.proof).is_valid());
    }

    #[test]
    fn is_valid_returns_false_for_invalid_result() {
        let result = route_case_with_audit(
            &valid_case(),
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &[domestic("rc-1", "mfr-01")],
            None,
        );
        let mut proof = result.proof.clone();
        proof.hash_hex =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        assert!(!verify_receipt(&result.audit_receipt, &proof).is_valid());
    }
}
