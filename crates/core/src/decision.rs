use crate::{CaseId, CaseRefusal, RoutingCandidateId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutingDecision {
    Selected(RoutingCandidateId),
    Refused(CaseRefusal),
    NoEligibleCandidate,
}

impl RoutingDecision {
    pub fn is_selected(&self) -> bool {
        matches!(self, Self::Selected(_))
    }

    pub fn is_refused(&self) -> bool {
        matches!(self, Self::Refused(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionContext {
    pub case_id: CaseId,
    pub original_candidate_count: usize,
    pub filtered_candidate_count: usize,
}

impl DecisionContext {
    pub fn new(case_id: CaseId, original_candidate_count: usize, filtered_candidate_count: usize) -> Self {
        Self {
            case_id,
            original_candidate_count,
            filtered_candidate_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CaseId, RefusalReason, RoutingCandidateId};

    #[test]
    fn selected_decision() {
        let d = RoutingDecision::Selected(RoutingCandidateId::new("rc-1"));
        assert!(d.is_selected());
        assert!(!d.is_refused());
    }

    #[test]
    fn refused_decision() {
        let mut refusal = CaseRefusal::new(CaseId::new());
        refusal.add_reason(RefusalReason::ManufacturerNotEligible);
        let d = RoutingDecision::Refused(refusal);
        assert!(d.is_refused());
        assert!(!d.is_selected());
    }

    #[test]
    fn no_eligible_candidate_decision() {
        let d = RoutingDecision::NoEligibleCandidate;
        assert!(!d.is_selected());
        assert!(!d.is_refused());
    }

    #[test]
    fn decision_context_new() {
        let id = CaseId::new();
        let ctx = DecisionContext::new(id.clone(), 5, 3);
        assert_eq!(ctx.case_id, id);
        assert_eq!(ctx.original_candidate_count, 5);
        assert_eq!(ctx.filtered_candidate_count, 3);
    }

    #[test]
    fn decision_context_zero_candidates() {
        let ctx = DecisionContext::new(CaseId::new(), 0, 0);
        assert_eq!(ctx.original_candidate_count, 0);
        assert_eq!(ctx.filtered_candidate_count, 0);
    }

    #[test]
    fn selected_decision_preserves_candidate_id() {
        let id = RoutingCandidateId::new("rc-42");
        let d = RoutingDecision::Selected(id.clone());
        if let RoutingDecision::Selected(inner) = d {
            assert_eq!(inner, id);
        } else {
            panic!("expected Selected");
        }
    }

    #[test]
    fn refused_decision_preserves_refusal() {
        let case_id = CaseId::new();
        let mut refusal = CaseRefusal::new(case_id.clone());
        refusal.add_reason(RefusalReason::UnsupportedJurisdiction);
        let d = RoutingDecision::Refused(refusal);
        if let RoutingDecision::Refused(inner) = d {
            assert_eq!(inner.case_id, case_id);
            assert!(inner.reasons.contains(&RefusalReason::UnsupportedJurisdiction));
        } else {
            panic!("expected Refused");
        }
    }
}
