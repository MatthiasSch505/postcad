use postcad_core::{
    route_case_with_context, Case, CaseRefusal, DecisionContext, RefusalReason, RoutingCandidate,
    RoutingDecision, RoutingOutcome, RoutingPolicy, RoutingPolicyConfig,
};
use postcad_registry::attestation::EvidenceAttestation;
use postcad_registry::evidence::EligibilityEvidence;
use postcad_registry::profile::{manufacturer_satisfies_profile, RequiredEvidenceProfile};
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

    let compliant_ids =
        ComplianceGate::filter_compliant_manufacturers(&manufacturer_ids, snapshots);

    let compliant_candidates: Vec<RoutingCandidate> = candidates
        .iter()
        .filter(|c| compliant_ids.contains(&c.manufacturer_id.0))
        .cloned()
        .collect();

    // If candidates were provided but compliance removed all of them, emit an
    // explicit refusal rather than falling through to NoEligibleCandidate.
    if !candidates.is_empty() && compliant_candidates.is_empty() {
        let refusal = CaseRefusal::with_reason(case.id.clone(), RefusalReason::ComplianceExclusion);
        let context = DecisionContext::new(case.id.clone(), candidates.len(), 0);
        return RoutingOutcome {
            decision: RoutingDecision::Refused(refusal),
            context,
        };
    }

    route_case_with_context(case, policy, &compliant_candidates)
}

/// Runs the deterministic routing pipeline with a profile-aware compliance
/// pre-filter driven by the policy's `compliance_profile_name`.
///
/// - If the policy has no `compliance_profile_name`, falls back to the
///   standard routing path without any compliance filtering.
/// - If the policy names a profile that is not found in `profiles`, all
///   candidates are treated as non-compliant and the explicit compliance
///   exclusion refusal is returned (if any candidates were present).
/// - Otherwise, each candidate is evaluated against the named profile using
///   `manufacturer_satisfies_profile`, and only passing candidates are routed.
pub fn route_case_with_profile_compliance(
    case: &Case,
    policy: RoutingPolicyConfig,
    candidates: &[RoutingCandidate],
    evidence: &[EligibilityEvidence],
    attestations: &[EvidenceAttestation],
    profiles: &[RequiredEvidenceProfile],
) -> RoutingOutcome {
    // Extract both fields upfront to avoid borrow/move conflicts.
    let compliance_profile_name = policy.compliance_profile_name.clone();
    let routing_policy = policy.routing_policy;

    // No compliance profile configured — standard routing.
    let profile_name = match compliance_profile_name {
        None => return route_case_with_context(case, routing_policy, candidates),
        Some(name) => name,
    };

    // Find the named profile.
    let profile = profiles.iter().find(|p| p.profile_name == profile_name);

    // Filter candidates by profile compliance, preserving slice order.
    let compliant_candidates: Vec<RoutingCandidate> = match profile {
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
        // Profile name present but unknown — no candidates pass.
        None => Vec::new(),
    };

    // If candidates were present but all filtered out, emit explicit refusal.
    if !candidates.is_empty() && compliant_candidates.is_empty() {
        let refusal = CaseRefusal::with_reason(case.id.clone(), RefusalReason::ComplianceExclusion);
        let context = DecisionContext::new(case.id.clone(), candidates.len(), 0);
        return RoutingOutcome {
            decision: RoutingDecision::Refused(refusal),
            context,
        };
    }

    route_case_with_context(case, routing_policy, &compliant_candidates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcad_core::{
        Case, Country, DentalCase, FileType, ManufacturerEligibility, ManufacturingLocation,
        Material, ProcedureType, RoutingCandidate, RoutingCandidateId, RoutingDecision,
        RoutingPolicy, RoutingPolicyConfig,
    };
    use postcad_registry::attestation::EvidenceAttestation;
    use postcad_registry::evidence::EligibilityEvidence;
    use postcad_registry::profile::RequiredEvidenceProfile;
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

    // ── route_case_with_compliance tests ─────────────────────────────────────

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

    // ── route_case_with_profile_compliance tests ──────────────────────────────

    #[test]
    fn profile_policy_without_compliance_profile_falls_back_to_normal_routing() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        let policy = RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly);

        let outcome = route_case_with_profile_compliance(&case, policy, &candidates, &[], &[], &[]);

        // No compliance filtering — candidate selected normally.
        assert!(matches!(outcome.decision, RoutingDecision::Selected(_)));
    }

    #[test]
    fn profile_policy_with_verified_evidence_selects_candidate() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        let evidence = vec![iso_evidence("mfr-01", "ISO-9001-2024")];
        let attestations = vec![verified_attestation("mfr-01", "ISO-9001-2024")];
        let profiles = vec![iso_profile()];
        let policy = RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly)
            .with_compliance_profile("iso_only_v1");

        let outcome = route_case_with_profile_compliance(
            &case,
            policy,
            &candidates,
            &evidence,
            &attestations,
            &profiles,
        );

        assert!(matches!(outcome.decision, RoutingDecision::Selected(_)));
    }

    #[test]
    fn profile_policy_missing_required_evidence_type_filters_out_candidate() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        // no evidence at all
        let profiles = vec![iso_profile()];
        let policy = RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly)
            .with_compliance_profile("iso_only_v1");

        let outcome =
            route_case_with_profile_compliance(&case, policy, &candidates, &[], &[], &profiles);

        assert!(matches!(
            outcome.decision,
            RoutingDecision::Refused(ref r) if r.reasons.contains(&RefusalReason::ComplianceExclusion)
        ));
    }

    #[test]
    fn profile_policy_rejected_attestation_filters_out_candidate() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        let evidence = vec![iso_evidence("mfr-01", "ISO-9001-2024")];
        let attestations = vec![rejected_attestation("mfr-01", "ISO-9001-2024")];
        let profiles = vec![iso_profile()];
        let policy = RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly)
            .with_compliance_profile("iso_only_v1");

        let outcome = route_case_with_profile_compliance(
            &case,
            policy,
            &candidates,
            &evidence,
            &attestations,
            &profiles,
        );

        assert!(matches!(
            outcome.decision,
            RoutingDecision::Refused(ref r) if r.reasons.contains(&RefusalReason::ComplianceExclusion)
        ));
    }

    #[test]
    fn profile_policy_unknown_profile_name_returns_compliance_exclusion_refusal() {
        let case = valid_case();
        let candidates = vec![domestic_candidate("rc-1", "mfr-01")];
        let policy = RoutingPolicyConfig::new(RoutingPolicy::AllowDomesticOnly)
            .with_compliance_profile("nonexistent_profile");

        let outcome = route_case_with_profile_compliance(
            &case,
            policy,
            &candidates,
            &[],
            &[],
            &[], // no profiles
        );

        assert!(matches!(
            outcome.decision,
            RoutingDecision::Refused(ref r) if r.reasons.contains(&RefusalReason::ComplianceExclusion)
        ));
    }

    #[test]
    fn profile_policy_mixed_candidates_preserve_deterministic_ordering() {
        let case = valid_case();
        let candidates = vec![
            domestic_candidate("rc-1", "mfr-01"),
            domestic_candidate("rc-2", "mfr-02"),
            domestic_candidate("rc-3", "mfr-03"),
        ];
        let evidence = vec![
            iso_evidence("mfr-01", "ISO-A"), // ineligible — rejected below
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

        let outcome = route_case_with_profile_compliance(
            &case,
            policy,
            &candidates,
            &evidence,
            &attestations,
            &profiles,
        );

        // mfr-01 filtered; first remaining in original order is rc-2 / mfr-02.
        assert_eq!(
            outcome.decision,
            RoutingDecision::Selected(RoutingCandidateId::new("rc-2"))
        );
    }
}
