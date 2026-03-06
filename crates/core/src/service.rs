use crate::{
    filter_candidates, select_candidate, validate_case, CaseRefusal, RefusalReason,
    RoutingCandidate, RoutingDecision, RoutingPolicy,
};
use crate::Case;

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
