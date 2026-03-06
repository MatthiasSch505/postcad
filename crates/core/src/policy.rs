use crate::{ManufacturingLocation, RoutingCandidate};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutingPolicy {
    AllowDomesticOnly,
    AllowDomesticAndCrossBorder,
}

pub fn filter_candidates(
    policy: RoutingPolicy,
    candidates: &[RoutingCandidate],
) -> Vec<RoutingCandidate> {
    candidates
        .iter()
        .filter(|c| match policy {
            RoutingPolicy::AllowDomesticOnly => c.location == ManufacturingLocation::Domestic,
            RoutingPolicy::AllowDomesticAndCrossBorder => {
                matches!(
                    c.location,
                    ManufacturingLocation::Domestic | ManufacturingLocation::CrossBorder
                )
            }
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ManufacturerEligibility, ManufacturingLocation, RoutingCandidate, RoutingCandidateId};

    fn domestic(id: &str) -> RoutingCandidate {
        RoutingCandidate::new(
            RoutingCandidateId::new(id),
            "mfr-domestic",
            ManufacturingLocation::Domestic,
            true,
            ManufacturerEligibility::Eligible,
        )
    }

    fn cross_border(id: &str) -> RoutingCandidate {
        RoutingCandidate::new(
            RoutingCandidateId::new(id),
            "mfr-cross",
            ManufacturingLocation::CrossBorder,
            true,
            ManufacturerEligibility::Eligible,
        )
    }

    fn unknown(id: &str) -> RoutingCandidate {
        RoutingCandidate::new(
            RoutingCandidateId::new(id),
            "mfr-unknown",
            ManufacturingLocation::Unknown,
            true,
            ManufacturerEligibility::Unknown,
        )
    }

    #[test]
    fn domestic_only_keeps_only_domestic() {
        let candidates = vec![domestic("a"), cross_border("b"), unknown("c")];
        let result = filter_candidates(RoutingPolicy::AllowDomesticOnly, &candidates);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, RoutingCandidateId::new("a"));
    }

    #[test]
    fn domestic_and_cross_border_keeps_both() {
        let candidates = vec![domestic("a"), cross_border("b"), unknown("c")];
        let result = filter_candidates(RoutingPolicy::AllowDomesticAndCrossBorder, &candidates);
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|c| c.id == RoutingCandidateId::new("a")));
        assert!(result.iter().any(|c| c.id == RoutingCandidateId::new("b")));
    }

    #[test]
    fn unknown_candidates_are_always_excluded() {
        let candidates = vec![unknown("x"), unknown("y")];
        let result_domestic = filter_candidates(RoutingPolicy::AllowDomesticOnly, &candidates);
        let result_both =
            filter_candidates(RoutingPolicy::AllowDomesticAndCrossBorder, &candidates);
        assert!(result_domestic.is_empty());
        assert!(result_both.is_empty());
    }

    #[test]
    fn empty_input_returns_empty() {
        let result = filter_candidates(RoutingPolicy::AllowDomesticOnly, &[]);
        assert!(result.is_empty());
        let result = filter_candidates(RoutingPolicy::AllowDomesticAndCrossBorder, &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn input_slice_is_not_mutated() {
        let candidates = vec![domestic("a"), cross_border("b")];
        let _ = filter_candidates(RoutingPolicy::AllowDomesticOnly, &candidates);
        // original still has both entries
        assert_eq!(candidates.len(), 2);
    }
}
