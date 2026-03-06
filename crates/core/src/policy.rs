use crate::{ManufacturingLocation, RoutingCandidate};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutingPolicy {
    AllowDomesticOnly,
    AllowDomesticAndCrossBorder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JurisdictionPolicy {
    DomesticOnly,
    DomesticAndCrossBorder,
}

impl From<JurisdictionPolicy> for RoutingPolicy {
    fn from(jp: JurisdictionPolicy) -> Self {
        match jp {
            JurisdictionPolicy::DomesticOnly => RoutingPolicy::AllowDomesticOnly,
            JurisdictionPolicy::DomesticAndCrossBorder => RoutingPolicy::AllowDomesticAndCrossBorder,
        }
    }
}

/// A routing policy together with optional compliance metadata.
///
/// Wraps the existing `RoutingPolicy` and carries an optional
/// `compliance_profile_name` that downstream services can use to look up a
/// `RequiredEvidenceProfile` without changing routing semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingPolicyConfig {
    pub routing_policy: RoutingPolicy,
    pub compliance_profile_name: Option<String>,
}

impl RoutingPolicyConfig {
    pub fn new(routing_policy: RoutingPolicy) -> Self {
        Self {
            routing_policy,
            compliance_profile_name: None,
        }
    }

    pub fn with_compliance_profile(mut self, name: impl Into<String>) -> Self {
        self.compliance_profile_name = Some(name.into());
        self
    }

    pub fn compliance_profile_name(&self) -> Option<&str> {
        self.compliance_profile_name.as_deref()
    }
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

    #[test]
    fn domestic_only_jurisdiction_maps_to_allow_domestic_only() {
        assert_eq!(
            RoutingPolicy::from(JurisdictionPolicy::DomesticOnly),
            RoutingPolicy::AllowDomesticOnly
        );
    }

    #[test]
    fn domestic_and_cross_border_jurisdiction_maps_to_allow_both() {
        assert_eq!(
            RoutingPolicy::from(JurisdictionPolicy::DomesticAndCrossBorder),
            RoutingPolicy::AllowDomesticAndCrossBorder
        );
    }

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

    #[test]
    fn policy_config_with_compliance_profile_returns_profile_name() {
        let config = RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly)
            .with_compliance_profile("de_milling_v1");
        assert_eq!(config.compliance_profile_name(), Some("de_milling_v1"));
    }

    #[test]
    fn policy_config_without_compliance_profile_returns_none() {
        let config = RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly);
        assert_eq!(config.compliance_profile_name(), None);
    }

    #[test]
    fn policy_config_routing_policy_is_preserved() {
        let config = RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticAndCrossBorder)
            .with_compliance_profile("us_fda_v2");
        assert_eq!(config.routing_policy, RoutingPolicy::AllowDomesticAndCrossBorder);
        assert_eq!(config.compliance_profile_name(), Some("us_fda_v2"));
    }
}
