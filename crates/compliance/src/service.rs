use postcad_core::{
    Case, CaseRefusal, DecisionContext, RefusalReason, RoutingCandidate, RoutingDecision,
    RoutingOutcome, RoutingPolicy, route_case_with_context,
};
use postcad_registry::snapshot::ManufacturerComplianceSnapshot;

use crate::ComplianceGate;

/// Runs the deterministic routing pipeline with a compliance pre-filter.
///
/// Candidates whose manufacturer id is not returned as compliant by the
/// `ComplianceGate` are removed before routing begins. The remaining
/// candidates are passed into the existing routing kernel unchanged.
///
/// If candidates were present but compliance filtering removed all of them,
/// a `RoutingDecision::Refused` with `RefusalReason::ComplianceExclusion`
/// is returned before falling through to the generic routing path.
pub fn route_case_with_compliance(
    case: &Case,
    policy: RoutingPolicy,
    candidates: &[RoutingCandidate],
    snapshots: &[ManufacturerComplianceSnapshot],
) -> RoutingOutcome {
    let manufacturer_ids: Vec<String> = candidates
        .iter()
        .map(|c| c.manufacturer_id.0.clone())
        .collect();

    let compliant_ids = ComplianceGate::filter_compliant_manufacturers(&manufacturer_ids, snapshots);

    let compliant_candidates: Vec<RoutingCandidate> = candidates
        .iter()
        .filter(|c| compliant_ids.contains(&c.manufacturer_id.0))
        .cloned()
        .collect();

    // If candidates were provided but compliance removed all of them, emit an
    // explicit refusal rather than falling through to NoEligibleCandidate.
    if !candidates.is_empty() && compliant_candidates.is_empty() {
        let refusal =
            CaseRefusal::with_reason(case.id.clone(), RefusalReason::ComplianceExclusion);
        let context = DecisionContext::new(case.id.clone(), candidates.len(), 0);
        return RoutingOutcome {
            decision: RoutingDecision::Refused(refusal),
            context,
        };
    }

    route_case_with_context(case, policy, &compliant_candidates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcad_core::{
        Case, Country, DentalCase, FileType, ManufacturerEligibility, ManufacturingLocation,
        Material, ProcedureType, RoutingCandidate, RoutingCandidateId, RoutingDecision,
        RoutingPolicy,
    };
    use postcad_registry::snapshot::ManufacturerComplianceSnapshot;

    fn valid_case() -> Case {
        Case::new(DentalCase {
            patient_country: Country::UnitedStates,
            manufacturer_country: Country::Germany,
            material: Material::Zirconia,
            procedure: ProcedureType::Crown,
            file_type: FileType::Stl,
        })
    }

    fn domestic_candidate(rc_id: &str, mfr_id: &str) -> RoutingCandidate {
        RoutingCandidate::new(
            RoutingCandidateId::new(rc_id),
            mfr_id,
            ManufacturingLocation::Domestic,
            true,
            ManufacturerEligibility::Eligible,
        )
    }

    fn eligible_snapshot(mfr_id: &str) -> ManufacturerComplianceSnapshot {
        ManufacturerComplianceSnapshot::new(
            mfr_id,
            vec!["REF-001".to_string()],
            vec!["verified".to_string()],
            true,
        )
    }

    fn ineligible_snapshot(mfr_id: &str) -> ManufacturerComplianceSnapshot {
        ManufacturerComplianceSnapshot::new(
            mfr_id,
            vec!["REF-001".to_string()],
            vec!["rejected".to_string()],
            false,
        )
    }

    #[test]
    fn eligible_manufacturer_survives_and_can_be_selected() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        let snapshots = vec![eligible_snapshot("mfr-01")];

        let outcome = route_case_with_compliance(
            &case,
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            &snapshots,
        );

        assert!(matches!(outcome.decision, RoutingDecision::Selected(_)));
    }

    #[test]
    fn ineligible_manufacturer_is_filtered_out() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        let snapshots = vec![ineligible_snapshot("mfr-01")];

        let outcome = route_case_with_compliance(
            &case,
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            &snapshots,
        );

        assert!(matches!(
            outcome.decision,
            RoutingDecision::Refused(ref r) if r.reasons.contains(&RefusalReason::ComplianceExclusion)
        ));
    }

    #[test]
    fn manufacturer_without_snapshot_is_filtered_out() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-99")];

        let outcome = route_case_with_compliance(
            &case,
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            &[], // no snapshots
        );

        assert!(matches!(
            outcome.decision,
            RoutingDecision::Refused(ref r) if r.reasons.contains(&RefusalReason::ComplianceExclusion)
        ));
    }

    #[test]
    fn mixed_candidates_preserve_deterministic_order_after_filtering() {
        let case = valid_case();
        let candidates = vec![
            domestic_candidate("rc-1", "mfr-01"),
            domestic_candidate("rc-2", "mfr-02"),
            domestic_candidate("rc-3", "mfr-03"),
        ];
        // only mfr-01 and mfr-03 are compliant
        let snapshots = vec![
            eligible_snapshot("mfr-01"),
            ineligible_snapshot("mfr-02"),
            eligible_snapshot("mfr-03"),
        ];

        let outcome = route_case_with_compliance(
            &case,
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            &snapshots,
        );

        // first compliant in original order is rc-1 / mfr-01
        assert_eq!(
            outcome.decision,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-1"))
        );
    }

    #[test]
    fn all_candidates_filtered_returns_compliance_exclusion_refusal() {
        let case = valid_case();
        let candidates = vec![
            domestic_candidate("rc-1", "mfr-01"),
            domestic_candidate("rc-2", "mfr-02"),
        ];
        let snapshots = vec![ineligible_snapshot("mfr-01"), ineligible_snapshot("mfr-02")];

        let outcome = route_case_with_compliance(
            &case,
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            &snapshots,
        );

        assert!(matches!(
            outcome.decision,
            RoutingDecision::Refused(ref r) if r.reasons.contains(&RefusalReason::ComplianceExclusion)
        ));
    }

    #[test]
    fn at_least_one_compliant_candidate_does_not_emit_compliance_refusal() {
        let case = valid_case();
        let candidates = vec![
            domestic_candidate("rc-1", "mfr-01"),
            domestic_candidate("rc-2", "mfr-02"),
        ];
        let snapshots = vec![ineligible_snapshot("mfr-01"), eligible_snapshot("mfr-02")];

        let outcome = route_case_with_compliance(
            &case,
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            &snapshots,
        );

        assert!(matches!(outcome.decision, RoutingDecision::Selected(_)));
    }
}
