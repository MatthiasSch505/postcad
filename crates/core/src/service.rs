use crate::{
    filter_candidates, select_candidate, validate_case, Case, CaseRefusal, DecisionContext,
    RefusalReason, RoutingCandidate, RoutingDecision, RoutingPolicy,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingOutcome {
    pub decision: RoutingDecision,
    pub context: DecisionContext,
}

pub fn route_case_with_context(
    case: &Case,
    policy: RoutingPolicy,
    candidates: &[RoutingCandidate],
) -> RoutingOutcome {
    let original_count = candidates.len();

    if validate_case(case).is_err() {
        let mut refusal = CaseRefusal::new(case.id.clone());
        refusal.add_reason(RefusalReason::ValidationFailed);
        let context = DecisionContext::new(case.id.clone(), original_count, 0);
        return RoutingOutcome {
            decision: RoutingDecision::Refused(refusal),
            context,
        };
    }

    let filtered = filter_candidates(policy, candidates);
    let filtered_count = filtered.len();
    let decision = select_candidate(case.id.clone(), &filtered);
    let context = DecisionContext::new(case.id.clone(), original_count, filtered_count);
    RoutingOutcome { decision, context }
}

pub fn route_case(
    case: &Case,
    policy: RoutingPolicy,
    candidates: &[RoutingCandidate],
) -> RoutingDecision {
    if validate_case(case).is_err() {
        let mut refusal = CaseRefusal::new(case.id.clone());
        refusal.add_reason(RefusalReason::ValidationFailed);
        return RoutingDecision::Refused(refusal);
    }

    let filtered = filter_candidates(policy, candidates);
    select_candidate(case.id.clone(), &filtered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Case, Country, DentalCase, FileType, ManufacturerEligibility, ManufacturingLocation,
        Material, ProcedureType, RoutingCandidate, RoutingCandidateId, RoutingPolicy,
    };

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
            file_type: FileType::Other(String::new()), // triggers ValidationFailed
        })
    }

    fn domestic_candidate(id: &str) -> RoutingCandidate {
        RoutingCandidate::new(
            RoutingCandidateId::new(id),
            "mfr-01",
            ManufacturingLocation::Domestic,
            true,
            ManufacturerEligibility::Eligible,
        )
    }

    fn cross_border_candidate(id: &str) -> RoutingCandidate {
        RoutingCandidate::new(
            RoutingCandidateId::new(id),
            "mfr-02",
            ManufacturingLocation::CrossBorder,
            true,
            ManufacturerEligibility::Eligible,
        )
    }

    #[test]
    fn invalid_case_returns_refused_with_validation_failed() {
        let result = route_case(&invalid_case(), RoutingPolicy::AllowDomesticOnly, &[]);
        match result {
            RoutingDecision::Refused(refusal) => {
                assert!(refusal.reasons.contains(&RefusalReason::ValidationFailed));
            }
            other => panic!("expected Refused, got {:?}", other),
        }
    }

    #[test]
    fn valid_case_no_candidates_returns_no_eligible() {
        let result = route_case(&valid_case(), RoutingPolicy::AllowDomesticOnly, &[]);
        assert_eq!(result, RoutingDecision::NoEligibleCandidate);
    }

    #[test]
    fn valid_case_with_candidates_returns_selected() {
        let candidates = vec![domestic_candidate("rc-1")];
        let result = route_case(&valid_case(), RoutingPolicy::AllowDomesticOnly, &candidates);
        assert_eq!(
            result,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-1"))
        );
    }

    #[test]
    fn outcome_valid_case_has_correct_context() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1"), domestic_candidate("rc-2")];
        let outcome = route_case_with_context(&case, RoutingPolicy::AllowDomesticOnly, &candidates);
        assert_eq!(outcome.context.case_id, case.id);
        assert_eq!(outcome.context.original_candidate_count, 2);
        assert_eq!(outcome.context.filtered_candidate_count, 2);
        assert!(outcome.decision.is_selected());
    }

    #[test]
    fn outcome_invalid_case_returns_refused_with_correct_context() {
        let case = invalid_case();
        let candidates = vec![domestic_candidate("rc-1")];
        let outcome = route_case_with_context(&case, RoutingPolicy::AllowDomesticOnly, &candidates);
        assert_eq!(outcome.context.case_id, case.id);
        assert_eq!(outcome.context.original_candidate_count, 1);
        assert_eq!(outcome.context.filtered_candidate_count, 0);
        assert!(outcome.decision.is_refused());
    }

    #[test]
    fn outcome_empty_candidates_has_zero_count() {
        let outcome = route_case_with_context(&valid_case(), RoutingPolicy::AllowDomesticOnly, &[]);
        assert_eq!(outcome.context.original_candidate_count, 0);
        assert_eq!(outcome.context.filtered_candidate_count, 0);
        assert_eq!(outcome.decision, RoutingDecision::NoEligibleCandidate);
    }

    #[test]
    fn outcome_policy_reduces_filtered_count() {
        let case = valid_case();
        // 2 cross-border, 1 domestic — domestic-only should filter to 1
        let candidates = vec![
            cross_border_candidate("rc-cb1"),
            cross_border_candidate("rc-cb2"),
            domestic_candidate("rc-d1"),
        ];
        let outcome = route_case_with_context(&case, RoutingPolicy::AllowDomesticOnly, &candidates);
        assert_eq!(outcome.context.original_candidate_count, 3);
        assert_eq!(outcome.context.filtered_candidate_count, 1);
        assert!(outcome.decision.is_selected());
    }

    #[test]
    fn outcome_decision_matches_route_case() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1")];
        let direct = route_case(&case, RoutingPolicy::AllowDomesticOnly, &candidates);
        let outcome = route_case_with_context(&case, RoutingPolicy::AllowDomesticOnly, &candidates);
        assert_eq!(outcome.decision, direct);
    }

    #[test]
    fn policy_filtering_is_respected() {
        // only cross-border candidates available; domestic-only policy should exclude them
        let candidates = vec![cross_border_candidate("rc-cb")];
        let result = route_case(
            &valid_case(),
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
        );
        assert_eq!(result, RoutingDecision::NoEligibleCandidate);

        // same candidates pass with AllowDomesticAndCrossBorder
        let result2 = route_case(
            &valid_case(),
            RoutingPolicy::AllowDomesticAndCrossBorder,
            &candidates,
        );
        assert_eq!(
            result2,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-cb"))
        );
    }
}
