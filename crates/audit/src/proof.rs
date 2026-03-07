use sha2::{Digest, Sha256};

use crate::fingerprint::RoutingDecisionFingerprint;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoutingProof {
    pub canonical_payload: String,
    pub hash_hex: String,
}

impl RoutingProof {
    pub fn from_fingerprint(fingerprint: &RoutingDecisionFingerprint) -> Self {
        let canonical_payload = fingerprint.canonical_string();
        let hash_hex = sha256_hex(&canonical_payload);
        Self {
            canonical_payload,
            hash_hex,
        }
    }

    pub fn verify(&self) -> bool {
        sha256_hex(&self.canonical_payload) == self.hash_hex
    }
}

fn sha256_hex(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    format!("{:x}", digest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcad_core::{
        Case, Country, DecisionContext, DentalCase, FileType, ManufacturerEligibility,
        ManufacturingLocation, Material, ProcedureType, RoutingCandidate, RoutingCandidateId,
        RoutingDecision, RoutingOutcome,
    };

    use crate::fingerprint::RoutingDecisionFingerprint;

    fn make_case() -> Case {
        Case::new(DentalCase {
            patient_country: Country::UnitedStates,
            manufacturer_country: Country::Germany,
            material: Material::Zirconia,
            procedure: ProcedureType::Crown,
            file_type: FileType::Stl,
        })
    }

    fn make_candidate(rc_id: &str, mfr_id: &str) -> RoutingCandidate {
        RoutingCandidate::new(
            RoutingCandidateId::new(rc_id),
            mfr_id,
            ManufacturingLocation::Domestic,
            true,
            ManufacturerEligibility::Eligible,
        )
    }

    fn make_outcome(case: &Case, decision: RoutingDecision, count: usize) -> RoutingOutcome {
        RoutingOutcome {
            decision,
            context: DecisionContext::new(case.id.clone(), count, count),
        }
    }

    fn selected_fingerprint() -> RoutingDecisionFingerprint {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-de-01")];
        let outcome = make_outcome(
            &case,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-1")),
            candidates.len(),
        );
        RoutingDecisionFingerprint::from_outcome(&outcome, "DE", &candidates, None)
    }

    fn refused_fingerprint() -> RoutingDecisionFingerprint {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-de-01")];
        let outcome = make_outcome(&case, RoutingDecision::NoEligibleCandidate, candidates.len());
        RoutingDecisionFingerprint::from_outcome(&outcome, "US", &candidates, None)
    }

    #[test]
    fn same_fingerprint_always_yields_same_hash() {
        let fp = selected_fingerprint();
        let proof_a = RoutingProof::from_fingerprint(&fp);
        let proof_b = RoutingProof::from_fingerprint(&fp);
        assert_eq!(proof_a.hash_hex, proof_b.hash_hex);
    }

    #[test]
    fn different_fingerprint_yields_different_hash() {
        let proof_a = RoutingProof::from_fingerprint(&selected_fingerprint());
        let proof_b = RoutingProof::from_fingerprint(&refused_fingerprint());
        assert_ne!(proof_a.hash_hex, proof_b.hash_hex);
    }

    #[test]
    fn verify_returns_true_for_untouched_proof() {
        let proof = RoutingProof::from_fingerprint(&selected_fingerprint());
        assert!(proof.verify());
    }

    #[test]
    fn verify_returns_false_if_canonical_payload_is_modified() {
        let mut proof = RoutingProof::from_fingerprint(&selected_fingerprint());
        proof.canonical_payload.push_str("\ntampered=true");
        assert!(!proof.verify());
    }

    #[test]
    fn verify_returns_false_if_hash_hex_is_modified() {
        let mut proof = RoutingProof::from_fingerprint(&selected_fingerprint());
        proof.hash_hex = "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        assert!(!proof.verify());
    }

    // ── canonical JSON ────────────────────────────────────────────────────────

    #[test]
    fn proof_canonical_json_is_identical_for_same_input() {
        let proof = RoutingProof::from_fingerprint(&selected_fingerprint());
        let json_a = crate::canonical::to_canonical_json(&proof);
        let json_b = crate::canonical::to_canonical_json(&proof);
        assert_eq!(json_a, json_b);
    }

    #[test]
    fn proof_canonical_json_is_compact() {
        let proof = RoutingProof::from_fingerprint(&selected_fingerprint());
        let json = crate::canonical::to_canonical_json(&proof);
        assert!(!json.ends_with('\n'));
        assert!(!json.contains("\n  "));
    }

    #[test]
    fn proof_canonical_json_contains_hash_hex() {
        let proof = RoutingProof::from_fingerprint(&selected_fingerprint());
        let json = crate::canonical::to_canonical_json(&proof);
        assert!(json.contains(&proof.hash_hex));
    }

    #[test]
    fn different_proofs_produce_different_canonical_json() {
        let proof_a = RoutingProof::from_fingerprint(&selected_fingerprint());
        let proof_b = RoutingProof::from_fingerprint(&refused_fingerprint());
        let json_a = crate::canonical::to_canonical_json(&proof_a);
        let json_b = crate::canonical::to_canonical_json(&proof_b);
        assert_ne!(json_a, json_b);
    }
}
