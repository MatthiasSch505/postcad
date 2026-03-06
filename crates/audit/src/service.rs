use postcad_core::{
    Case, RoutingCandidate, RoutingPolicy, RoutingPolicyConfig,
    filter_candidates, fingerprint_policy, route_case_with_context,
};
use postcad_registry::attestation::EvidenceAttestation;
use postcad_registry::evidence::EligibilityEvidence;
use postcad_registry::profile::{manufacturer_satisfies_profile, RequiredEvidenceProfile};
use postcad_registry::snapshot::ManufacturerComplianceSnapshot;
use postcad_compliance::{ComplianceGate, route_case_with_profile_compliance};

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
    pub policy_fingerprint: String,
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

    let policy_config = RoutingPolicyConfig::new(policy.clone());
    let policy_fingerprint = fingerprint_policy(&policy_config);

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
        policy_fingerprint,
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

    let policy_config = RoutingPolicyConfig::new(policy.clone());
    let policy_fingerprint = fingerprint_policy(&policy_config);

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
        policy_fingerprint,
    }
}

/// Runs the profile-aware compliance routing pipeline and returns the outcome
/// together with derived audit artifacts.
///
/// The policy's `compliance_profile_name` drives pre-filtering: candidates
/// whose manufacturer does not satisfy the named `RequiredEvidenceProfile` are
/// removed before routing begins. If no profile name is set, standard routing
/// is used without compliance filtering. Audit artifacts are derived from the
/// compliance-filtered candidate view.
pub fn route_case_with_profile_compliance_audit(
    case: &Case,
    jurisdiction: &str,
    policy: RoutingPolicyConfig,
    candidates: &[RoutingCandidate],
    evidence: &[EligibilityEvidence],
    attestations: &[EvidenceAttestation],
    profiles: &[RequiredEvidenceProfile],
    policy_version: Option<String>,
) -> RoutingServiceResult {
    // Step 1: run profile compliance routing.
    let policy_fingerprint = fingerprint_policy(&policy);
    let outcome = route_case_with_profile_compliance(
        case,
        policy.clone(),
        candidates,
        evidence,
        attestations,
        profiles,
    );

    // Step 2: recompute compliant candidates for audit artifact derivation.
    let compliant_candidates: Vec<RoutingCandidate> =
        match policy.compliance_profile_name.as_deref() {
            None => candidates.to_vec(),
            Some(name) => match profiles.iter().find(|p| p.profile_name == name) {
                Some(profile) => candidates
                    .iter()
                    .filter(|c| {
                        manufacturer_satisfies_profile(
                            &c.manufacturer_id.0,
                            evidence,
                            attestations,
                            profile,
                        )
                    })
                    .cloned()
                    .collect(),
                None => Vec::new(),
            },
        };

    // Step 3: policy filter (captured for DecisionTrace only).
    let policy_filtered = filter_candidates(policy.routing_policy, &compliant_candidates);

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
        policy_fingerprint,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcad_core::{
        Case, Country, DentalCase, FileType, ManufacturerEligibility, ManufacturingLocation,
        Material, ProcedureType, RoutingCandidate, RoutingCandidateId, RoutingDecision,
        RoutingPolicy, RoutingPolicyConfig, fingerprint_policy,
    };
    use postcad_registry::attestation::EvidenceAttestation;
    use postcad_registry::evidence::EligibilityEvidence;
    use postcad_registry::profile::RequiredEvidenceProfile;

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

    // ── route_case_with_profile_compliance_audit tests ────────────────────────

    fn iso_evidence(mfr: &str, reference: &str) -> EligibilityEvidence {
        EligibilityEvidence::new(mfr, "iso_cert", reference)
    }

    fn verified_attestation(mfr: &str, reference: &str) -> EvidenceAttestation {
        EvidenceAttestation::new(mfr, reference, "registry-authority", "verified")
    }

    fn rejected_attestation(mfr: &str, reference: &str) -> EvidenceAttestation {
        EvidenceAttestation::new(mfr, reference, "registry-authority", "rejected")
    }

    fn iso_profile() -> RequiredEvidenceProfile {
        RequiredEvidenceProfile::new("iso_only_v1", vec!["iso_cert".to_string()])
    }

    #[test]
    fn profile_compliance_audit_verified_evidence_selects_and_proof_verifies() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        let evidence = vec![iso_evidence("mfr-01", "ISO-9001-2024")];
        let attestations = vec![verified_attestation("mfr-01", "ISO-9001-2024")];
        let profiles = vec![iso_profile()];
        let policy = RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly)
            .with_compliance_profile("iso_only_v1");

        let result = route_case_with_profile_compliance_audit(
            &case,
            "DE",
            policy,
            &candidates,
            &evidence,
            &attestations,
            &profiles,
            None,
        );

        assert!(result.outcome.decision.is_selected());
        assert!(result.proof.verify());
    }

    #[test]
    fn profile_compliance_audit_rejected_evidence_returns_compliance_refusal_and_proof_verifies() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        let evidence = vec![iso_evidence("mfr-01", "ISO-9001-2024")];
        let attestations = vec![rejected_attestation("mfr-01", "ISO-9001-2024")];
        let profiles = vec![iso_profile()];
        let policy = RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly)
            .with_compliance_profile("iso_only_v1");

        let result = route_case_with_profile_compliance_audit(
            &case,
            "DE",
            policy,
            &candidates,
            &evidence,
            &attestations,
            &profiles,
            None,
        );

        assert!(result.outcome.decision.is_refused());
        assert!(result.proof.verify());
    }

    #[test]
    fn profile_compliance_audit_no_profile_falls_back_to_normal_routing() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        let policy = RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly);

        let result = route_case_with_profile_compliance_audit(
            &case,
            "DE",
            policy,
            &candidates,
            &[],
            &[],
            &[],
            None,
        );

        assert!(result.outcome.decision.is_selected());
        assert!(result.proof.verify());
    }

    #[test]
    fn profile_compliance_audit_mixed_candidates_preserve_deterministic_ordering() {
        let case = valid_case();
        let candidates = vec![
            domestic_candidate("rc-1", "mfr-01"),
            domestic_candidate("rc-2", "mfr-02"),
            domestic_candidate("rc-3", "mfr-03"),
        ];
        let evidence = vec![
            iso_evidence("mfr-01", "ISO-A"),
            iso_evidence("mfr-02", "ISO-B"),
            iso_evidence("mfr-03", "ISO-C"),
        ];
        let attestations = vec![
            rejected_attestation("mfr-01", "ISO-A"),
            verified_attestation("mfr-02", "ISO-B"),
            verified_attestation("mfr-03", "ISO-C"),
        ];
        let profiles = vec![iso_profile()];
        let policy = RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly)
            .with_compliance_profile("iso_only_v1");

        let result = route_case_with_profile_compliance_audit(
            &case,
            "DE",
            policy,
            &candidates,
            &evidence,
            &attestations,
            &profiles,
            None,
        );

        // mfr-01 filtered; first remaining in original order is rc-2 / mfr-02.
        assert_eq!(
            result.outcome.decision,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-2"))
        );
        assert!(result.proof.verify());
    }

    #[test]
    fn profile_compliance_audit_proof_differs_when_filtering_changes_result() {
        let case = valid_case();
        let candidates = vec![
            domestic_candidate("rc-1", "mfr-01"),
            domestic_candidate("rc-2", "mfr-02"),
        ];
        let evidence = vec![
            iso_evidence("mfr-01", "ISO-A"),
            iso_evidence("mfr-02", "ISO-B"),
        ];
        // both verified
        let attestations_both = vec![
            verified_attestation("mfr-01", "ISO-A"),
            verified_attestation("mfr-02", "ISO-B"),
        ];
        // only mfr-02 verified
        let attestations_one = vec![
            rejected_attestation("mfr-01", "ISO-A"),
            verified_attestation("mfr-02", "ISO-B"),
        ];
        let profiles = vec![iso_profile()];

        let result_a = route_case_with_profile_compliance_audit(
            &case,
            "DE",
            RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly)
                .with_compliance_profile("iso_only_v1"),
            &candidates,
            &evidence,
            &attestations_both,
            &profiles,
            None,
        );

        let result_b = route_case_with_profile_compliance_audit(
            &case,
            "DE",
            RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly)
                .with_compliance_profile("iso_only_v1"),
            &candidates,
            &evidence,
            &attestations_one,
            &profiles,
            None,
        );

        assert_ne!(result_a.proof.hash_hex, result_b.proof.hash_hex);
    }

    // ── policy_fingerprint field tests ────────────────────────────────────────

    #[test]
    fn route_case_with_audit_policy_fingerprint_matches_direct_fingerprint() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        let policy = RoutingPolicy::AllowDomesticOnly;
        let expected = fingerprint_policy(&RoutingPolicyConfig::new(policy.clone()));

        let result = route_case_with_audit(&case, "DE", policy, &candidates, None);

        assert_eq!(result.policy_fingerprint, expected);
    }

    #[test]
    fn route_case_with_compliance_audit_policy_fingerprint_matches_direct_fingerprint() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        let snapshots = vec![eligible_snapshot("mfr-01")];
        let policy = RoutingPolicy::AllowDomesticOnly;
        let expected = fingerprint_policy(&RoutingPolicyConfig::new(policy.clone()));

        let result = route_case_with_compliance_audit(
            &case, "DE", policy, &candidates, &snapshots, None,
        );

        assert_eq!(result.policy_fingerprint, expected);
    }

    #[test]
    fn route_case_with_profile_compliance_audit_policy_fingerprint_matches_direct_fingerprint() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        let evidence = vec![iso_evidence("mfr-01", "ISO-9001-2024")];
        let attestations = vec![verified_attestation("mfr-01", "ISO-9001-2024")];
        let profiles = vec![iso_profile()];
        let policy = RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly)
            .with_compliance_profile("iso_only_v1");
        let expected = fingerprint_policy(&policy);

        let result = route_case_with_profile_compliance_audit(
            &case, "DE", policy, &candidates, &evidence, &attestations, &profiles, None,
        );

        assert_eq!(result.policy_fingerprint, expected);
    }

    #[test]
    fn different_routing_policies_produce_different_policy_fingerprints() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];

        let result_domestic = route_case_with_audit(
            &case, "DE", RoutingPolicy::AllowDomesticOnly, &candidates, None,
        );
        let result_cross = route_case_with_audit(
            &case, "DE", RoutingPolicy::AllowDomesticAndCrossBorder, &candidates, None,
        );

        assert_ne!(result_domestic.policy_fingerprint, result_cross.policy_fingerprint);
    }

    #[test]
    fn same_policy_produces_same_policy_fingerprint_across_calls() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];

        let result_a = route_case_with_audit(
            &case, "DE", RoutingPolicy::AllowDomesticOnly, &candidates, None,
        );
        let result_b = route_case_with_audit(
            &case, "DE", RoutingPolicy::AllowDomesticOnly, &candidates, None,
        );

        assert_eq!(result_a.policy_fingerprint, result_b.policy_fingerprint);
    }

    #[test]
    fn policy_fingerprint_is_64_hex_chars() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];

        let result = route_case_with_audit(
            &case, "DE", RoutingPolicy::AllowDomesticOnly, &candidates, None,
        );

        assert_eq!(result.policy_fingerprint.len(), 64);
        assert!(result.policy_fingerprint.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn profile_compliance_audit_with_and_without_profile_have_different_policy_fingerprints() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        let evidence = vec![iso_evidence("mfr-01", "ISO-9001-2024")];
        let attestations = vec![verified_attestation("mfr-01", "ISO-9001-2024")];
        let profiles = vec![iso_profile()];

        let with_profile = route_case_with_profile_compliance_audit(
            &case,
            "DE",
            RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly)
                .with_compliance_profile("iso_only_v1"),
            &candidates,
            &evidence,
            &attestations,
            &profiles,
            None,
        );
        let without_profile = route_case_with_profile_compliance_audit(
            &case,
            "DE",
            RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly),
            &candidates,
            &evidence,
            &attestations,
            &profiles,
            None,
        );

        assert_ne!(with_profile.policy_fingerprint, without_profile.policy_fingerprint);
    }
}
