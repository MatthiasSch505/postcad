use crate::{CaseId, RoutingCandidate, RoutingDecision};

pub fn select_candidate(_case_id: CaseId, candidates: &[RoutingCandidate]) -> RoutingDecision {
    match candidates.first() {
        None => RoutingDecision::NoEligibleCandidate,
        Some(c) => RoutingDecision::Selected(c.id.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CaseId, ManufacturerEligibility, ManufacturingLocation, RoutingCandidate,
        RoutingCandidateId,
    };

    fn candidate(id: &str) -> RoutingCandidate {
        RoutingCandidate::new(
            RoutingCandidateId::new(id),
            "mfr-01",
            ManufacturingLocation::Domestic,
            true,
            ManufacturerEligibility::Eligible,
        )
    }

    #[test]
    fn empty_list_returns_no_eligible_candidate() {
        let result = select_candidate(CaseId::new(), &[]);
        assert_eq!(result, RoutingDecision::NoEligibleCandidate);
    }

    #[test]
    fn single_candidate_is_selected() {
        let candidates = vec![candidate("rc-1")];
        let result = select_candidate(CaseId::new(), &candidates);
        assert_eq!(
            result,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-1"))
        );
    }

    #[test]
    fn multiple_candidates_selects_first() {
        let candidates = vec![candidate("rc-1"), candidate("rc-2"), candidate("rc-3")];
        let result = select_candidate(CaseId::new(), &candidates);
        assert_eq!(
            result,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-1"))
        );
    }
}
