use crate::attestation::{evidence_attestation_status, EvidenceAttestation};
use crate::evidence::EligibilityEvidence;
use crate::profile::{manufacturer_satisfies_profile, RequiredEvidenceProfile};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManufacturerComplianceSnapshot {
    pub manufacturer_id: String,
    pub evidence_references: Vec<String>,
    pub attestation_statuses: Vec<String>,
    pub is_eligible: bool,
}

impl ManufacturerComplianceSnapshot {
    pub fn new(
        manufacturer_id: impl Into<String>,
        evidence_references: Vec<String>,
        attestation_statuses: Vec<String>,
        is_eligible: bool,
    ) -> Self {
        Self {
            manufacturer_id: manufacturer_id.into(),
            evidence_references,
            attestation_statuses,
            is_eligible,
        }
    }
}

pub fn build_compliance_snapshot(
    manufacturer_id: &str,
    evidence: &[EligibilityEvidence],
    attestations: &[EvidenceAttestation],
    required_evidence_type: &str,
) -> ManufacturerComplianceSnapshot {
    // Collect matching evidence references in slice order.
    let evidence_references: Vec<String> = evidence
        .iter()
        .filter(|e| {
            e.manufacturer_id == manufacturer_id && e.evidence_type == required_evidence_type
        })
        .map(|e| e.evidence_reference.clone())
        .collect();

    // For each reference, look up the attestation status if present.
    let attestation_statuses: Vec<String> = evidence_references
        .iter()
        .filter_map(|r| {
            evidence_attestation_status(manufacturer_id, r, attestations).map(|s| s.to_string())
        })
        .collect();

    let is_eligible =
        !evidence_references.is_empty() && attestation_statuses.iter().any(|s| s == "verified");

    ManufacturerComplianceSnapshot {
        manufacturer_id: manufacturer_id.to_string(),
        evidence_references,
        attestation_statuses,
        is_eligible,
    }
}

/// Builds a compliance snapshot from a `RequiredEvidenceProfile`, collecting
/// all evidence entries whose type is required by the profile, and setting
/// `is_eligible` only if the manufacturer fully satisfies the profile.
pub fn build_compliance_snapshot_for_profile(
    manufacturer_id: &str,
    evidence: &[EligibilityEvidence],
    attestations: &[EvidenceAttestation],
    profile: &RequiredEvidenceProfile,
) -> ManufacturerComplianceSnapshot {
    // Collect all evidence references for this manufacturer that belong to any
    // required type, preserving deterministic slice order.
    let evidence_references: Vec<String> = evidence
        .iter()
        .filter(|e| e.manufacturer_id == manufacturer_id && profile.requires(&e.evidence_type))
        .map(|e| e.evidence_reference.clone())
        .collect();

    // For each reference, look up the attestation status if present.
    let attestation_statuses: Vec<String> = evidence_references
        .iter()
        .filter_map(|r| {
            evidence_attestation_status(manufacturer_id, r, attestations).map(|s| s.to_string())
        })
        .collect();

    let is_eligible =
        manufacturer_satisfies_profile(manufacturer_id, evidence, attestations, profile);

    ManufacturerComplianceSnapshot {
        manufacturer_id: manufacturer_id.to_string(),
        evidence_references,
        attestation_statuses,
        is_eligible,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attestation::EvidenceAttestation;
    use crate::evidence::EligibilityEvidence;
    use crate::profile::RequiredEvidenceProfile;

    fn iso_evidence(mfr: &str, reference: &str) -> EligibilityEvidence {
        EligibilityEvidence::new(mfr, "iso_cert", reference)
    }

    fn fda_evidence(mfr: &str, reference: &str) -> EligibilityEvidence {
        EligibilityEvidence::new(mfr, "fda_clearance", reference)
    }

    fn attested(mfr: &str, reference: &str, status: &str) -> EvidenceAttestation {
        EvidenceAttestation::new(mfr, reference, "registry-authority", status)
    }

    fn iso_profile() -> RequiredEvidenceProfile {
        RequiredEvidenceProfile::new("iso-only", vec!["iso_cert".to_string()])
    }

    fn dual_profile() -> RequiredEvidenceProfile {
        RequiredEvidenceProfile::new(
            "iso-and-fda",
            vec!["iso_cert".to_string(), "fda_clearance".to_string()],
        )
    }

    // ── existing build_compliance_snapshot tests ──────────────────────────────

    #[test]
    fn verified_evidence_produces_eligible_snapshot() {
        let evidence = vec![iso_evidence("mfr-01", "ISO-9001-2024")];
        let attestations = vec![attested("mfr-01", "ISO-9001-2024", "verified")];

        let snapshot = build_compliance_snapshot("mfr-01", &evidence, &attestations, "iso_cert");

        assert!(snapshot.is_eligible);
        assert_eq!(snapshot.evidence_references, vec!["ISO-9001-2024"]);
        assert_eq!(snapshot.attestation_statuses, vec!["verified"]);
    }

    #[test]
    fn evidence_without_attestation_produces_ineligible_snapshot() {
        let evidence = vec![iso_evidence("mfr-01", "ISO-9001-2024")];

        let snapshot = build_compliance_snapshot("mfr-01", &evidence, &[], "iso_cert");

        assert!(!snapshot.is_eligible);
        assert_eq!(snapshot.evidence_references, vec!["ISO-9001-2024"]);
        assert!(snapshot.attestation_statuses.is_empty());
    }

    #[test]
    fn rejected_attestation_produces_ineligible_snapshot() {
        let evidence = vec![iso_evidence("mfr-01", "ISO-9001-2024")];
        let attestations = vec![attested("mfr-01", "ISO-9001-2024", "rejected")];

        let snapshot = build_compliance_snapshot("mfr-01", &evidence, &attestations, "iso_cert");

        assert!(!snapshot.is_eligible);
        assert_eq!(snapshot.attestation_statuses, vec!["rejected"]);
    }

    #[test]
    fn wrong_manufacturer_data_is_ignored() {
        let evidence = vec![iso_evidence("mfr-02", "ISO-9001-2024")];
        let attestations = vec![attested("mfr-02", "ISO-9001-2024", "verified")];

        let snapshot = build_compliance_snapshot("mfr-01", &evidence, &attestations, "iso_cert");

        assert!(!snapshot.is_eligible);
        assert!(snapshot.evidence_references.is_empty());
        assert!(snapshot.attestation_statuses.is_empty());
    }

    #[test]
    fn non_matching_evidence_type_is_ignored() {
        let evidence = vec![EligibilityEvidence::new(
            "mfr-01",
            "milling_validation",
            "MILL-001",
        )];
        let attestations = vec![attested("mfr-01", "MILL-001", "verified")];

        let snapshot = build_compliance_snapshot("mfr-01", &evidence, &attestations, "iso_cert");

        assert!(!snapshot.is_eligible);
        assert!(snapshot.evidence_references.is_empty());
    }

    #[test]
    fn evidence_reference_ordering_is_preserved() {
        let evidence = vec![
            iso_evidence("mfr-01", "ISO-A"),
            iso_evidence("mfr-01", "ISO-B"),
            iso_evidence("mfr-01", "ISO-C"),
        ];
        let attestations = vec![attested("mfr-01", "ISO-A", "verified")];

        let snapshot = build_compliance_snapshot("mfr-01", &evidence, &attestations, "iso_cert");

        assert_eq!(
            snapshot.evidence_references,
            vec!["ISO-A", "ISO-B", "ISO-C"]
        );
    }

    #[test]
    fn attestation_status_ordering_is_preserved() {
        let evidence = vec![
            iso_evidence("mfr-01", "ISO-A"),
            iso_evidence("mfr-01", "ISO-B"),
        ];
        let attestations = vec![
            attested("mfr-01", "ISO-A", "expired"),
            attested("mfr-01", "ISO-B", "verified"),
        ];

        let snapshot = build_compliance_snapshot("mfr-01", &evidence, &attestations, "iso_cert");

        assert_eq!(snapshot.attestation_statuses, vec!["expired", "verified"]);
        // eligible because ISO-B is verified
        assert!(snapshot.is_eligible);
    }

    // ── build_compliance_snapshot_for_profile tests ───────────────────────────

    #[test]
    fn profile_snapshot_eligible_when_all_required_types_verified() {
        let evidence = vec![
            iso_evidence("mfr-01", "ISO-9001-2024"),
            fda_evidence("mfr-01", "FDA-510K-2024"),
        ];
        let attestations = vec![
            attested("mfr-01", "ISO-9001-2024", "verified"),
            attested("mfr-01", "FDA-510K-2024", "verified"),
        ];

        let snapshot = build_compliance_snapshot_for_profile(
            "mfr-01",
            &evidence,
            &attestations,
            &dual_profile(),
        );

        assert!(snapshot.is_eligible);
        assert_eq!(
            snapshot.evidence_references,
            vec!["ISO-9001-2024", "FDA-510K-2024"]
        );
        assert_eq!(snapshot.attestation_statuses, vec!["verified", "verified"]);
    }

    #[test]
    fn profile_snapshot_ineligible_when_one_required_type_missing() {
        // Only iso_cert — no fda_clearance.
        let evidence = vec![iso_evidence("mfr-01", "ISO-9001-2024")];
        let attestations = vec![attested("mfr-01", "ISO-9001-2024", "verified")];

        let snapshot = build_compliance_snapshot_for_profile(
            "mfr-01",
            &evidence,
            &attestations,
            &dual_profile(),
        );

        assert!(!snapshot.is_eligible);
        assert_eq!(snapshot.evidence_references, vec!["ISO-9001-2024"]);
    }

    #[test]
    fn profile_snapshot_ineligible_when_attestation_rejected_for_required_type() {
        let evidence = vec![
            iso_evidence("mfr-01", "ISO-9001-2024"),
            fda_evidence("mfr-01", "FDA-510K-2024"),
        ];
        let attestations = vec![
            attested("mfr-01", "ISO-9001-2024", "verified"),
            attested("mfr-01", "FDA-510K-2024", "rejected"),
        ];

        let snapshot = build_compliance_snapshot_for_profile(
            "mfr-01",
            &evidence,
            &attestations,
            &dual_profile(),
        );

        assert!(!snapshot.is_eligible);
    }

    #[test]
    fn profile_snapshot_ignores_non_required_evidence_types() {
        let evidence = vec![
            iso_evidence("mfr-01", "ISO-9001-2024"),
            EligibilityEvidence::new("mfr-01", "milling_validation", "MILL-001"),
        ];
        let attestations = vec![
            attested("mfr-01", "ISO-9001-2024", "verified"),
            attested("mfr-01", "MILL-001", "verified"),
        ];

        let snapshot = build_compliance_snapshot_for_profile(
            "mfr-01",
            &evidence,
            &attestations,
            &iso_profile(),
        );

        // milling_validation is not in iso_profile — should not appear
        assert_eq!(snapshot.evidence_references, vec!["ISO-9001-2024"]);
        assert!(snapshot.is_eligible);
    }

    #[test]
    fn profile_snapshot_ignores_wrong_manufacturer() {
        let evidence = vec![iso_evidence("mfr-02", "ISO-9001-2024")];
        let attestations = vec![attested("mfr-02", "ISO-9001-2024", "verified")];

        let snapshot = build_compliance_snapshot_for_profile(
            "mfr-01",
            &evidence,
            &attestations,
            &iso_profile(),
        );

        assert!(!snapshot.is_eligible);
        assert!(snapshot.evidence_references.is_empty());
    }

    #[test]
    fn profile_snapshot_preserves_evidence_reference_ordering() {
        let evidence = vec![
            iso_evidence("mfr-01", "ISO-A"),
            fda_evidence("mfr-01", "FDA-X"),
            iso_evidence("mfr-01", "ISO-B"),
        ];
        let attestations = vec![
            attested("mfr-01", "ISO-A", "verified"),
            attested("mfr-01", "FDA-X", "verified"),
            attested("mfr-01", "ISO-B", "verified"),
        ];

        let snapshot = build_compliance_snapshot_for_profile(
            "mfr-01",
            &evidence,
            &attestations,
            &dual_profile(),
        );

        assert_eq!(
            snapshot.evidence_references,
            vec!["ISO-A", "FDA-X", "ISO-B"]
        );
        assert!(snapshot.is_eligible);
    }

    #[test]
    fn profile_eligibility_can_differ_from_single_type_snapshot() {
        // mfr-01 has iso_cert but not fda_clearance
        let evidence = vec![iso_evidence("mfr-01", "ISO-9001-2024")];
        let attestations = vec![attested("mfr-01", "ISO-9001-2024", "verified")];

        let single_type_snapshot =
            build_compliance_snapshot("mfr-01", &evidence, &attestations, "iso_cert");
        let profile_snapshot = build_compliance_snapshot_for_profile(
            "mfr-01",
            &evidence,
            &attestations,
            &dual_profile(),
        );

        // eligible under iso-only check, not eligible under dual profile
        assert!(single_type_snapshot.is_eligible);
        assert!(!profile_snapshot.is_eligible);
    }
}
