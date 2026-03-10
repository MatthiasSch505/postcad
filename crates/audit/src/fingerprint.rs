use postcad_core::{RoutingCandidate, RoutingDecision, RoutingOutcome};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingDecisionFingerprint {
    pub case_id: String,
    pub jurisdiction: String,
    pub selected_manufacturer_id: Option<String>,
    pub candidate_ids_considered: Vec<String>,
    pub final_status: String,
    pub refusal_code: Option<String>,
    pub policy_version: Option<String>,
    /// SHA-256 of the canonical registry snapshot used at routing time.
    /// `None` when routing ran without a compliance snapshot (e.g. plain
    /// `route_case_with_audit`). Covered by the proof hash.
    pub registry_snapshot_hash: Option<String>,
}

impl RoutingDecisionFingerprint {
    pub fn from_outcome(
        outcome: &RoutingOutcome,
        jurisdiction: impl Into<String>,
        candidates: &[RoutingCandidate],
        policy_version: Option<String>,
    ) -> Self {
        let case_id = outcome.context.case_id.to_string();
        let jurisdiction = jurisdiction.into();
        let candidate_ids_considered: Vec<String> =
            candidates.iter().map(|c| c.id.to_string()).collect();

        match &outcome.decision {
            RoutingDecision::Selected(candidate_id) => {
                let manufacturer_id = candidates
                    .iter()
                    .find(|c| &c.id == candidate_id)
                    .map(|c| c.manufacturer_id.0.clone());

                Self {
                    case_id,
                    jurisdiction,
                    selected_manufacturer_id: manufacturer_id,
                    candidate_ids_considered,
                    final_status: "selected".to_string(),
                    refusal_code: None,
                    policy_version,
                    registry_snapshot_hash: None,
                }
            }

            RoutingDecision::Refused(refusal) => {
                let refusal_code = refusal
                    .reasons
                    .first()
                    .map(|r| r.code().to_string());

                Self {
                    case_id,
                    jurisdiction,
                    selected_manufacturer_id: None,
                    candidate_ids_considered,
                    final_status: "refused".to_string(),
                    refusal_code,
                    policy_version,
                    registry_snapshot_hash: None,
                }
            }

            RoutingDecision::NoEligibleCandidate => Self {
                case_id,
                jurisdiction,
                selected_manufacturer_id: None,
                candidate_ids_considered,
                final_status: "refused".to_string(),
                refusal_code: Some("no_eligible_candidates".to_string()),
                policy_version,
                registry_snapshot_hash: None,
            },
        }
    }

    /// Sets the registry snapshot hash on this fingerprint, consuming `self`.
    ///
    /// Call after [`from_outcome`] when a compliance snapshot was used at
    /// routing time. The hash is then covered by [`RoutingProof`]'s SHA-256.
    pub fn with_registry_snapshot_hash(mut self, hash: Option<String>) -> Self {
        self.registry_snapshot_hash = hash;
        self
    }

    /// Returns the canonical JSON representation of this fingerprint.
    ///
    /// This is the byte string fed into SHA-256 to produce the proof hash.
    /// Field order follows struct declaration order; `None` serializes as `null`.
    pub fn canonical_string(&self) -> String {
        crate::to_canonical_json(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof::RoutingProof;
    use postcad_core::{
        Case, CaseRefusal, Country, DecisionContext, DentalCase, FileType, ManufacturerEligibility,
        ManufacturingLocation, Material, ProcedureType, RefusalReason, RoutingCandidate,
        RoutingCandidateId, RoutingDecision, RoutingOutcome,
    };

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

    #[test]
    fn successful_outcome_builds_selected_fingerprint() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-de-01")];
        let outcome = make_outcome(
            &case,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-1")),
            candidates.len(),
        );

        let fp = RoutingDecisionFingerprint::from_outcome(&outcome, "DE", &candidates, None);

        assert_eq!(fp.final_status, "selected");
        assert_eq!(fp.selected_manufacturer_id, Some("mfr-de-01".to_string()));
        assert!(fp.refusal_code.is_none());
        assert_eq!(fp.case_id, case.id.to_string());
        assert_eq!(fp.jurisdiction, "DE");
    }

    #[test]
    fn refusal_outcome_builds_refused_fingerprint() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-de-01")];
        let refusal = CaseRefusal::with_reason(case.id.clone(), RefusalReason::ValidationFailed);
        let outcome = make_outcome(
            &case,
            RoutingDecision::Refused(refusal),
            candidates.len(),
        );

        let fp = RoutingDecisionFingerprint::from_outcome(&outcome, "US", &candidates, None);

        assert_eq!(fp.final_status, "refused");
        assert!(fp.selected_manufacturer_id.is_none());
        assert_eq!(fp.refusal_code, Some("invalid_input".to_string()));
    }

    #[test]
    fn compliance_exclusion_refusal_builds_refused_fingerprint() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-01")];
        let refusal =
            CaseRefusal::with_reason(case.id.clone(), RefusalReason::ComplianceExclusion);
        let outcome = make_outcome(
            &case,
            RoutingDecision::Refused(refusal),
            candidates.len(),
        );

        let fp = RoutingDecisionFingerprint::from_outcome(&outcome, "DE", &candidates, None);

        assert_eq!(fp.final_status, "refused");
        assert!(fp.selected_manufacturer_id.is_none());
        assert_eq!(fp.refusal_code, Some("compliance_failed".to_string()));
    }

    #[test]
    fn compliance_exclusion_refusal_code_appears_in_canonical_string() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-01")];
        let refusal =
            CaseRefusal::with_reason(case.id.clone(), RefusalReason::ComplianceExclusion);
        let outcome = make_outcome(
            &case,
            RoutingDecision::Refused(refusal),
            candidates.len(),
        );

        let fp = RoutingDecisionFingerprint::from_outcome(&outcome, "DE", &candidates, None);
        let s = fp.canonical_string();

        assert!(s.contains("\"refusal_code\":\"compliance_failed\""));
        assert!(s.contains("\"final_status\":\"refused\""));
    }

    #[test]
    fn different_refusal_codes_produce_different_proof_hashes() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-01")];

        let refusal_compliance =
            CaseRefusal::with_reason(case.id.clone(), RefusalReason::ComplianceExclusion);
        let outcome_compliance = make_outcome(
            &case,
            RoutingDecision::Refused(refusal_compliance),
            candidates.len(),
        );

        let refusal_validation =
            CaseRefusal::with_reason(case.id.clone(), RefusalReason::ValidationFailed);
        let outcome_validation = make_outcome(
            &case,
            RoutingDecision::Refused(refusal_validation),
            candidates.len(),
        );

        let fp_a =
            RoutingDecisionFingerprint::from_outcome(&outcome_compliance, "DE", &candidates, None);
        let fp_b =
            RoutingDecisionFingerprint::from_outcome(&outcome_validation, "DE", &candidates, None);

        let proof_a = RoutingProof::from_fingerprint(&fp_a);
        let proof_b = RoutingProof::from_fingerprint(&fp_b);

        assert_ne!(proof_a.hash_hex, proof_b.hash_hex);
    }

    #[test]
    fn canonical_string_is_deterministic_for_same_input() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-de-01")];
        let outcome = make_outcome(
            &case,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-1")),
            candidates.len(),
        );

        let fp = RoutingDecisionFingerprint::from_outcome(&outcome, "DE", &candidates, None);

        assert_eq!(fp.canonical_string(), fp.canonical_string());
    }

    #[test]
    fn candidate_ordering_is_preserved_in_canonical_string() {
        let case = make_case();
        let candidates = vec![
            make_candidate("rc-a", "mfr-1"),
            make_candidate("rc-b", "mfr-2"),
            make_candidate("rc-c", "mfr-3"),
        ];
        let outcome = make_outcome(&case, RoutingDecision::NoEligibleCandidate, candidates.len());

        let fp = RoutingDecisionFingerprint::from_outcome(&outcome, "JP", &candidates, None);
        let s = fp.canonical_string();

        // Candidates serialize as a JSON array in original order.
        let a_pos = s.find("\"rc-a\"").unwrap();
        let b_pos = s.find("\"rc-b\"").unwrap();
        let c_pos = s.find("\"rc-c\"").unwrap();
        assert!(a_pos < b_pos && b_pos < c_pos);
    }

    #[test]
    fn missing_optional_fields_serialize_as_null() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-01")];
        let outcome = make_outcome(&case, RoutingDecision::NoEligibleCandidate, candidates.len());

        let fp = RoutingDecisionFingerprint::from_outcome(&outcome, "DE", &candidates, None);
        let s = fp.canonical_string();

        assert!(s.contains("\"selected_manufacturer_id\":null"));
        assert!(s.contains("\"policy_version\":null"));
    }

    #[test]
    fn canonical_string_is_compact_json() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-de-01")];
        let outcome = make_outcome(
            &case,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-1")),
            candidates.len(),
        );
        let fp = RoutingDecisionFingerprint::from_outcome(&outcome, "DE", &candidates, None);
        let s = fp.canonical_string();

        assert!(s.starts_with('{'));
        assert!(s.ends_with('}'));
        assert!(!s.contains('\n'));
        assert!(!s.contains("  "));
    }
}
