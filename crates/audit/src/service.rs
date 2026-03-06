use postcad_core::{Case, RoutingCandidate, RoutingPolicy, filter_candidates, route_case_with_context};

use crate::{DecisionTrace, RoutingAuditReceipt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingServiceResult {
    pub outcome: postcad_core::RoutingOutcome,
    pub audit_receipt: RoutingAuditReceipt,
    pub decision_trace: DecisionTrace,
}

/// Runs the deterministic routing pipeline and returns the outcome together
/// with derived audit artifacts. No persistence, timestamps, or I/O.
pub fn route_case_with_audit(
    case: &Case,
    jurisdiction: &str,
    policy: RoutingPolicy,
    candidates: &[RoutingCandidate],
    policy_version: Option<String>,
) -> RoutingServiceResult {
    // Capture filtered candidates before routing so DecisionTrace can
    // distinguish eligible from rejected without a second filter pass.
    let filtered = filter_candidates(policy.clone(), candidates);

    let outcome = route_case_with_context(case, policy, candidates);

    let audit_receipt = RoutingAuditReceipt::from_outcome(
        &outcome,
        jurisdiction,
        candidates,
        policy_version,
    );

    let decision_trace =
        DecisionTrace::from_outcome(&outcome, jurisdiction, candidates, &filtered);

    RoutingServiceResult {
        outcome,
        audit_receipt,
        decision_trace,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcad_core::{
        Case, Country, DentalCase, FileType, ManufacturerEligibility, ManufacturingLocation,
        Material, ProcedureType, RoutingCandidate, RoutingCandidateId, RoutingDecision,
        RoutingPolicy,
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
            file_type: FileType::Other(String::new()),
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

    #[test]
    fn successful_routing_returns_outcome_and_audit_artifacts() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-de-01")];

        let result = route_case_with_audit(
            &case,
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            None,
        );

        assert!(result.outcome.decision.is_selected());
        assert_eq!(
            result.audit_receipt.selected_manufacturer_id,
            Some("mfr-de-01".to_string())
        );
        assert_eq!(result.decision_trace.final_status, "selected");
        assert!(!result.audit_receipt.candidate_ids_considered.is_empty());
    }

    #[test]
    fn refusal_routing_populates_refusal_audit_fields() {
        let case = invalid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-de-01")];

        let result = route_case_with_audit(
            &case,
            "US",
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            None,
        );

        assert!(result.outcome.decision.is_refused());
        assert!(result.audit_receipt.refusal_code.is_some());
        assert!(result.audit_receipt.selected_manufacturer_id.is_none());
        assert_eq!(result.decision_trace.final_status, "refused");
    }

    #[test]
    fn audit_artifacts_match_case_id_and_jurisdiction() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];

        let result = route_case_with_audit(
            &case,
            "JP",
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            Some("v2".to_string()),
        );

        let case_id = case.id.to_string();
        assert_eq!(result.audit_receipt.case_id, case_id);
        assert_eq!(result.audit_receipt.jurisdiction, "JP");
        assert_eq!(result.decision_trace.case_id, case_id);
        assert_eq!(result.decision_trace.jurisdiction, "JP");
        assert_eq!(result.audit_receipt.policy_version, Some("v2".to_string()));
    }

    #[test]
    fn existing_route_case_with_context_unchanged() {
        // verify the original core function is unaffected
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        let outcome = route_case_with_context(&case, RoutingPolicy::AllowDomesticOnly, &candidates);
        assert!(matches!(outcome.decision, RoutingDecision::Selected(_)));
    }
}
