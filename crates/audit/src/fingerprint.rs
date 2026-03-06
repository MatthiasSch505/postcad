use postcad_core::{RoutingCandidate, RoutingDecision, RoutingOutcome};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingDecisionFingerprint {
    pub case_id: String,
    pub jurisdiction: String,
    pub selected_manufacturer_id: Option<String>,
    pub candidate_ids_considered: Vec<String>,
    pub final_status: String,
    pub refusal_code: Option<String>,
    pub policy_version: Option<String>,
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
                }
            }

            RoutingDecision::Refused(refusal) => {
                let refusal_code = refusal
                    .reasons
                    .first()
                    .map(|r| format!("{:?}", r));

                Self {
                    case_id,
                    jurisdiction,
                    selected_manufacturer_id: None,
                    candidate_ids_considered,
                    final_status: "refused".to_string(),
                    refusal_code,
                    policy_version,
                }
            }

            RoutingDecision::NoEligibleCandidate => Self {
                case_id,
                jurisdiction,
                selected_manufacturer_id: None,
                candidate_ids_considered,
                final_status: "refused".to_string(),
                refusal_code: Some("NoEligibleCandidate".to_string()),
                policy_version,
            },
        }
    }

    pub fn canonical_string(&self) -> String {
        let selected = self
            .selected_manufacturer_id
            .as_deref()
            .unwrap_or("");
        let candidate_ids = self.candidate_ids_considered.join(",");
        let refusal_code = self.refusal_code.as_deref().unwrap_or("");
        let policy_version = self.policy_version.as_deref().unwrap_or("");

        format!(
            "case_id={}\njurisdiction={}\nselected_manufacturer_id={}\ncandidate_ids={}\nfinal_status={}\nrefusal_code={}\npolicy_version={}",
            self.case_id,
            self.jurisdiction,
            selected,
            candidate_ids,
            self.final_status,
            refusal_code,
            policy_version,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcad_core::{
        Case, Country, DecisionContext, DentalCase, FileType, ManufacturerEligibility,
        ManufacturingLocation, Material, ProcedureType, RoutingCandidate, RoutingCandidateId,
        RoutingDecision, RoutingOutcome,
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
        let refusal = postcad_core::CaseRefusal::with_reason(
            case.id.clone(),
            postcad_core::RefusalReason::ValidationFailed,
        );
        let outcome = make_outcome(
            &case,
            RoutingDecision::Refused(refusal),
            candidates.len(),
        );

        let fp = RoutingDecisionFingerprint::from_outcome(&outcome, "US", &candidates, None);

        assert_eq!(fp.final_status, "refused");
        assert!(fp.selected_manufacturer_id.is_none());
        assert_eq!(fp.refusal_code, Some("ValidationFailed".to_string()));
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

        assert!(s.contains("candidate_ids=rc-a,rc-b,rc-c"));
    }

    #[test]
    fn missing_optional_fields_serialize_as_empty_strings() {
        let case = make_case();
        let candidates = vec![make_candidate("rc-1", "mfr-01")];
        let outcome = make_outcome(&case, RoutingDecision::NoEligibleCandidate, candidates.len());

        let fp = RoutingDecisionFingerprint::from_outcome(&outcome, "DE", &candidates, None);
        let s = fp.canonical_string();

        assert!(s.contains("selected_manufacturer_id=\n"));
        assert!(s.contains("policy_version="));
        // policy_version at end — check it ends with empty value
        assert!(s.ends_with("policy_version="));
    }
}
