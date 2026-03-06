use postcad_core::{Case, RoutingCandidate, RoutingPolicy, filter_candidates, route_case_with_context};
use postcad_registry::snapshot::ManufacturerComplianceSnapshot;
use postcad_compliance::ComplianceGate;

use crate::{
    DecisionTrace, RoutingAuditReceipt, RoutingDecisionFingerprint, RoutingProof,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingServiceResult {
    pub outcome: postcad_core::RoutingOutcome,
    pub audit_receipt: RoutingAuditReceipt,
    pub decision_trace: DecisionTrace,
    pub fingerprint: RoutingDecisionFingerprint,
    pub proof: RoutingProof,
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
        policy_version.clone(),
    );

    let decision_trace =
        DecisionTrace::from_outcome(&outcome, jurisdiction, candidates, &filtered);

    let fingerprint =
        RoutingDecisionFingerprint::from_outcome(&outcome, jurisdiction, candidates, policy_version);

    let proof = RoutingProof::from_fingerprint(&fingerprint);

    RoutingServiceResult {
        outcome,
        audit_receipt,
        decision_trace,
        fingerprint,
        proof,
    }
}

/// Runs the compliance-aware deterministic routing pipeline and returns the
/// outcome together with derived audit artifacts.
///
/// Candidates whose manufacturer id does not appear in `snapshots` with
/// `is_eligible == true` are removed before routing begins. The remaining
/// candidates follow the existing routing and audit derivation path.
pub fn route_case_with_compliance_audit(
    case: &Case,
    jurisdiction: &str,
    policy: RoutingPolicy,
    candidates: &[RoutingCandidate],
    snapshots: &[ManufacturerComplianceSnapshot],
    policy_version: Option<String>,
) -> RoutingServiceResult {
    // Step 1: compliance pre-filter.
    let manufacturer_ids: Vec<String> = candidates
        .iter()
        .map(|c| c.manufacturer_id.0.clone())
        .collect();
    let compliant_ids =
        ComplianceGate::filter_compliant_manufacturers(&manufacturer_ids, snapshots);
    let compliant_candidates: Vec<RoutingCandidate> = candidates
        .iter()
        .filter(|c| compliant_ids.contains(&c.manufacturer_id.0))
        .cloned()
        .collect();

    // Step 2: policy filter (captured for DecisionTrace only).
    let policy_filtered = filter_candidates(policy.clone(), &compliant_candidates);

    // Step 3: route against the compliance-filtered candidate set.
    let outcome = route_case_with_context(case, policy, &compliant_candidates);

    // Step 4: derive audit artifacts from the compliance-filtered view.
    let audit_receipt = RoutingAuditReceipt::from_outcome(
        &outcome,
        jurisdiction,
        &compliant_candidates,
        policy_version.clone(),
    );

    let decision_trace = DecisionTrace::from_outcome(
        &outcome,
        jurisdiction,
        &compliant_candidates,
        &policy_filtered,
    );

    let fingerprint = RoutingDecisionFingerprint::from_outcome(
        &outcome,
        jurisdiction,
        &compliant_candidates,
        policy_version,
    );

    let proof = RoutingProof::from_fingerprint(&fingerprint);

    RoutingServiceResult {
        outcome,
        audit_receipt,
        decision_trace,
        fingerprint,
        proof,
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

    // ── existing route_case_with_audit tests ─────────────────────────────────

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
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        let outcome = route_case_with_context(&case, RoutingPolicy::AllowDomesticOnly, &candidates);
        assert!(matches!(outcome.decision, RoutingDecision::Selected(_)));
    }

    #[test]
    fn successful_routing_returns_proof_that_verifies() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-de-01")];

        let result = route_case_with_audit(
            &case,
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            None,
        );

        assert!(result.proof.verify());
    }

    #[test]
    fn refusal_routing_returns_proof_that_verifies() {
        let case = invalid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-de-01")];

        let result = route_case_with_audit(
            &case,
            "US",
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            None,
        );

        assert!(result.proof.verify());
    }

    #[test]
    fn proof_canonical_payload_matches_fingerprint_canonical_string() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];

        let result = route_case_with_audit(
            &case,
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            None,
        );

        assert_eq!(
            result.proof.canonical_payload,
            result.fingerprint.canonical_string()
        );
    }

    #[test]
    fn proof_changes_when_routing_result_changes() {
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];

        let result_a = route_case_with_audit(
            &valid_case(),
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            None,
        );

        let result_b = route_case_with_audit(
            &invalid_case(),
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            None,
        );

        assert_ne!(result_a.proof.hash_hex, result_b.proof.hash_hex);
    }

    // ── route_case_with_compliance_audit tests ────────────────────────────────

    #[test]
    fn compliance_audit_eligible_manufacturer_returns_proof_that_verifies() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        let snapshots = vec![eligible_snapshot("mfr-01")];

        let result = route_case_with_compliance_audit(
            &case,
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            &snapshots,
            None,
        );

        assert!(result.outcome.decision.is_selected());
        assert!(result.proof.verify());
    }

    #[test]
    fn compliance_audit_ineligible_manufacturer_is_filtered_out() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        let snapshots = vec![ineligible_snapshot("mfr-01")];

        let result = route_case_with_compliance_audit(
            &case,
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            &snapshots,
            None,
        );

        assert_eq!(result.outcome.decision, RoutingDecision::NoEligibleCandidate);
        assert!(result.proof.verify());
    }

    #[test]
    fn compliance_audit_manufacturer_without_snapshot_is_filtered_out() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-99")];

        let result = route_case_with_compliance_audit(
            &case,
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            &[], // no snapshots
            None,
        );

        assert_eq!(result.outcome.decision, RoutingDecision::NoEligibleCandidate);
        assert!(result.proof.verify());
    }

    #[test]
    fn compliance_audit_mixed_candidates_preserve_deterministic_behavior() {
        let case = valid_case();
        let candidates = vec![
            domestic_candidate("rc-1", "mfr-01"),
            domestic_candidate("rc-2", "mfr-02"),
            domestic_candidate("rc-3", "mfr-03"),
        ];
        let snapshots = vec![
            ineligible_snapshot("mfr-01"),
            eligible_snapshot("mfr-02"),
            eligible_snapshot("mfr-03"),
        ];

        let result = route_case_with_compliance_audit(
            &case,
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            &snapshots,
            None,
        );

        // first compliant in original order is rc-2 / mfr-02
        assert_eq!(
            result.outcome.decision,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-2"))
        );
        assert!(result.proof.verify());
    }

    #[test]
    fn compliance_audit_proof_differs_when_compliance_changes_routing_result() {
        let case = valid_case();
        let candidates = vec![
            domestic_candidate("rc-1", "mfr-01"),
            domestic_candidate("rc-2", "mfr-02"),
        ];

        // both eligible
        let snapshots_both = vec![eligible_snapshot("mfr-01"), eligible_snapshot("mfr-02")];
        // only mfr-02 eligible
        let snapshots_one = vec![ineligible_snapshot("mfr-01"), eligible_snapshot("mfr-02")];

        let result_a = route_case_with_compliance_audit(
            &case,
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            &snapshots_both,
            None,
        );

        let result_b = route_case_with_compliance_audit(
            &case,
            "DE",
            RoutingPolicy::AllowDomesticOnly,
            &candidates,
            &snapshots_one,
            None,
        );

        assert_ne!(result_a.proof.hash_hex, result_b.proof.hash_hex);
    }
}
