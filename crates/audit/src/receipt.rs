use postcad_core::{RoutingCandidate, RoutingDecision, RoutingOutcome};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoutingAuditReceipt {
    pub case_id: String,
    pub jurisdiction: String,
    pub selected_manufacturer_id: Option<String>,
    pub candidate_ids_considered: Vec<String>,
    pub refusal_code: Option<String>,
    pub refusal_message: Option<String>,
    pub policy_version: Option<String>,
    /// SHA-256 of the canonical registry snapshot used at routing time.
    /// `None` when routing ran without a compliance snapshot.
    pub registry_snapshot_hash: Option<String>,
    /// SHA-256 of the canonical case content (all DentalCase fields + case_id).
    /// `None` when not computed (legacy path). Matches the `input_case_hash` in
    /// the accompanying [`RoutingDecisionFingerprint`].
    pub input_case_hash: Option<String>,
}

impl RoutingAuditReceipt {
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
                    refusal_code: None,
                    refusal_message: None,
                    policy_version,
                    registry_snapshot_hash: None,
                    input_case_hash: None,
                }
            }

            RoutingDecision::Refused(refusal) => {
                let code = refusal
                    .reasons
                    .first()
                    .map(|r| r.code().to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let message = refusal
                    .reasons
                    .first()
                    .map(|r| r.message().to_string())
                    .unwrap_or_else(|| "Case refused with no specific reason".to_string());

                Self {
                    case_id,
                    jurisdiction,
                    selected_manufacturer_id: None,
                    candidate_ids_considered,
                    refusal_code: Some(code),
                    refusal_message: Some(message),
                    policy_version,
                    registry_snapshot_hash: None,
                    input_case_hash: None,
                }
            }

            RoutingDecision::NoEligibleCandidate => Self {
                case_id,
                jurisdiction,
                selected_manufacturer_id: None,
                candidate_ids_considered,
                refusal_code: Some("no_eligible_candidates".to_string()),
                refusal_message: Some("No eligible candidate found".to_string()),
                policy_version,
                registry_snapshot_hash: None,
                input_case_hash: None,
            },
        }
    }

    /// Sets the registry snapshot hash on this receipt, consuming `self`.
    ///
    /// Call after [`from_outcome`] when a compliance snapshot was used at
    /// routing time.
    pub fn with_registry_snapshot_hash(mut self, hash: Option<String>) -> Self {
        self.registry_snapshot_hash = hash;
        self
    }

    /// Sets the input case hash on this receipt, consuming `self`.
    ///
    /// Call after [`from_outcome`] with `Some(fingerprint_case(case))`.
    pub fn with_input_case_hash(mut self, hash: Option<String>) -> Self {
        self.input_case_hash = hash;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcad_core::{
        Case, Country, DecisionContext, DentalCase, FileType, ManufacturerEligibility,
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

    fn make_outcome(
        case: &Case,
        decision: RoutingDecision,
        candidate_count: usize,
    ) -> RoutingOutcome {
        RoutingOutcome {
            decision,
            context: DecisionContext::new(case.id.clone(), candidate_count, candidate_count),
        }
    }

    #[test]
    fn receipt_from_successful_outcome() {
        let case = make_case();
        let candidates = vec![
            make_candidate("rc-1", "mfr-de-01"),
            make_candidate("rc-2", "mfr-de-02"),
        ];
        let outcome = make_outcome(
            &case,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-1")),
            candidates.len(),
        );

        let receipt = RoutingAuditReceipt::from_outcome(&outcome, "DE", &candidates, None);

        assert_eq!(receipt.case_id, case.id.to_string());
        assert_eq!(receipt.jurisdiction, "DE");
        assert_eq!(
            receipt.selected_manufacturer_id,
            Some("mfr-de-01".to_string())
        );
        assert!(receipt.refusal_code.is_none());
        assert!(receipt.refusal_message.is_none());
        assert_eq!(receipt.candidate_ids_considered, vec!["rc-1", "rc-2"]);
        assert_eq!(receipt.policy_version, None);
    }

    #[test]
    fn receipt_from_refusal_outcome() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-de-01")];
        let refusal = postcad_core::CaseRefusal::with_reason(
            case.id.clone(),
            RefusalReason::ValidationFailed,
        );
        let outcome = make_outcome(&case, RoutingDecision::Refused(refusal), candidates.len());

        let receipt =
            RoutingAuditReceipt::from_outcome(&outcome, "US", &candidates, Some("v1".to_string()));

        assert_eq!(receipt.case_id, case.id.to_string());
        assert_eq!(receipt.jurisdiction, "US");
        assert!(receipt.selected_manufacturer_id.is_none());
        assert_eq!(receipt.refusal_code, Some("invalid_input".to_string()));
        assert!(receipt.refusal_message.is_some());
        assert_eq!(receipt.policy_version, Some("v1".to_string()));
    }

    #[test]
    fn receipt_from_no_eligible_candidate() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-01")];
        let outcome = make_outcome(&case, RoutingDecision::NoEligibleCandidate, 0);

        let receipt = RoutingAuditReceipt::from_outcome(&outcome, "JP", &candidates, None);

        assert!(receipt.selected_manufacturer_id.is_none());
        assert_eq!(
            receipt.refusal_code,
            Some("no_eligible_candidates".to_string())
        );
        assert!(receipt.refusal_message.is_some());
    }

    // ── canonical JSON ────────────────────────────────────────────────────────

    #[test]
    fn receipt_canonical_json_is_identical_for_same_input() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-de-01")];
        let outcome = make_outcome(
            &case,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-1")),
            candidates.len(),
        );
        let receipt = RoutingAuditReceipt::from_outcome(&outcome, "DE", &candidates, None);

        let json_a = crate::canonical::to_canonical_json(&receipt);
        let json_b = crate::canonical::to_canonical_json(&receipt);
        assert_eq!(json_a, json_b);
    }

    #[test]
    fn receipt_canonical_json_contains_case_id_and_jurisdiction() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-de-01")];
        let outcome = make_outcome(
            &case,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-1")),
            candidates.len(),
        );
        let receipt = RoutingAuditReceipt::from_outcome(&outcome, "DE", &candidates, None);
        let json = crate::canonical::to_canonical_json(&receipt);

        assert!(json.contains(&case.id.to_string()));
        assert!(json.contains("\"DE\""));
    }

    #[test]
    fn receipt_canonical_json_is_compact() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-de-01")];
        let outcome = make_outcome(
            &case,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-1")),
            candidates.len(),
        );
        let receipt = RoutingAuditReceipt::from_outcome(&outcome, "DE", &candidates, None);
        let json = crate::canonical::to_canonical_json(&receipt);

        assert!(!json.ends_with('\n'));
        assert!(!json.contains("\n  "));
    }

    #[test]
    fn receipt_refused_canonical_json_uses_stable_refusal_code() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-de-01")];
        let outcome = make_outcome(
            &case,
            RoutingDecision::NoEligibleCandidate,
            candidates.len(),
        );
        let receipt = RoutingAuditReceipt::from_outcome(&outcome, "DE", &candidates, None);
        let json = crate::canonical::to_canonical_json(&receipt);

        assert!(json.contains("\"no_eligible_candidates\""));
    }

    #[test]
    fn candidate_ids_are_preserved_in_order() {
        let case = make_case();
        let candidates = vec![
            make_candidate("rc-a", "mfr-1"),
            make_candidate("rc-b", "mfr-2"),
            make_candidate("rc-c", "mfr-3"),
        ];
        let outcome = make_outcome(&case, RoutingDecision::NoEligibleCandidate, 0);

        let receipt = RoutingAuditReceipt::from_outcome(&outcome, "DE", &candidates, None);

        assert_eq!(
            receipt.candidate_ids_considered,
            vec!["rc-a", "rc-b", "rc-c"]
        );
    }
}
