#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoutingCandidateId(pub String);

impl RoutingCandidateId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for RoutingCandidateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManufacturingLocation {
    Domestic,
    CrossBorder,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManufacturerEligibility {
    Eligible,
    Ineligible,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingCandidate {
    pub id: RoutingCandidateId,
    pub manufacturer_id: String,
    pub location: ManufacturingLocation,
    pub supports_case: bool,
    pub eligibility: ManufacturerEligibility,
}

impl RoutingCandidate {
    pub fn new(
        id: RoutingCandidateId,
        manufacturer_id: impl Into<String>,
        location: ManufacturingLocation,
        supports_case: bool,
        eligibility: ManufacturerEligibility,
    ) -> Self {
        Self {
            id,
            manufacturer_id: manufacturer_id.into(),
            location,
            supports_case,
            eligibility,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_id_new() {
        let id = RoutingCandidateId::new("mfr-001");
        assert_eq!(id.0, "mfr-001");
    }

    #[test]
    fn candidate_id_display() {
        let id = RoutingCandidateId::new("mfr-042");
        assert_eq!(id.to_string(), "mfr-042");
    }

    #[test]
    fn candidate_id_equality() {
        let a = RoutingCandidateId::new("x");
        let b = RoutingCandidateId::new("x");
        let c = RoutingCandidateId::new("y");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn create_domestic_candidate() {
        let c = RoutingCandidate::new(
            RoutingCandidateId::new("rc-1"),
            "mfr-de-01",
            ManufacturingLocation::Domestic,
            true,
            ManufacturerEligibility::Eligible,
        );
        assert_eq!(c.location, ManufacturingLocation::Domestic);
        assert_eq!(c.manufacturer_id, "mfr-de-01");
        assert!(c.supports_case);
    }

    #[test]
    fn create_cross_border_candidate() {
        let c = RoutingCandidate::new(
            RoutingCandidateId::new("rc-2"),
            "mfr-us-07",
            ManufacturingLocation::CrossBorder,
            true,
            ManufacturerEligibility::Eligible,
        );
        assert_eq!(c.location, ManufacturingLocation::CrossBorder);
    }

    #[test]
    fn create_unknown_location_candidate() {
        let c = RoutingCandidate::new(
            RoutingCandidateId::new("rc-3"),
            "mfr-unknown",
            ManufacturingLocation::Unknown,
            false,
            ManufacturerEligibility::Unknown,
        );
        assert_eq!(c.location, ManufacturingLocation::Unknown);
        assert!(!c.supports_case);
    }

    #[test]
    fn supports_case_false() {
        let c = RoutingCandidate::new(
            RoutingCandidateId::new("rc-4"),
            "mfr-jp-02",
            ManufacturingLocation::Domestic,
            false,
            ManufacturerEligibility::Ineligible,
        );
        assert!(!c.supports_case);
    }

    #[test]
    fn eligible_candidate() {
        let c = RoutingCandidate::new(
            RoutingCandidateId::new("rc-e"),
            "mfr-01",
            ManufacturingLocation::Domestic,
            true,
            ManufacturerEligibility::Eligible,
        );
        assert_eq!(c.eligibility, ManufacturerEligibility::Eligible);
    }

    #[test]
    fn ineligible_candidate() {
        let c = RoutingCandidate::new(
            RoutingCandidateId::new("rc-i"),
            "mfr-02",
            ManufacturingLocation::Domestic,
            false,
            ManufacturerEligibility::Ineligible,
        );
        assert_eq!(c.eligibility, ManufacturerEligibility::Ineligible);
    }

    #[test]
    fn unknown_eligibility_candidate() {
        let c = RoutingCandidate::new(
            RoutingCandidateId::new("rc-u"),
            "mfr-03",
            ManufacturingLocation::Unknown,
            false,
            ManufacturerEligibility::Unknown,
        );
        assert_eq!(c.eligibility, ManufacturerEligibility::Unknown);
    }

    #[test]
    fn candidate_clone_is_equal() {
        let c = RoutingCandidate::new(
            RoutingCandidateId::new("rc-5"),
            "mfr-fr-03",
            ManufacturingLocation::CrossBorder,
            true,
            ManufacturerEligibility::Eligible,
        );
        assert_eq!(c.clone(), c);
    }
}
