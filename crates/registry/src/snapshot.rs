use crate::attestation::{evidence_attestation_status, EvidenceAttestation};
use crate::evidence::EligibilityEvidence;

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
            evidence_attestation_status(manufacturer_id, r, attestations)
                .map(|s| s.to_string())
        })
        .collect();

    let is_eligible = !evidence_references.is_empty()
        && attestation_statuses.iter().any(|s| s == "verified");

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

    fn iso_evidence(mfr: &str, reference: &str) -> EligibilityEvidence {
        EligibilityEvidence::new(mfr, "iso_cert", reference)
    }

    fn attested(mfr: &str, reference: &str, status: &str) -> EvidenceAttestation {
        EvidenceAttestation::new(mfr, reference, "registry-authority", status)
    }

    #[test]
    fn verified_evidence_produces_eligible_snapshot() {
        let evidence = vec![iso_evidence("mfr-01", "ISO-9001-2024")];
        let attestations = vec![attested("mfr-01", "ISO-9001-2024", "verified")];

        let snapshot =
            build_compliance_snapshot("mfr-01", &evidence, &attestations, "iso_cert");

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

        let snapshot =
            build_compliance_snapshot("mfr-01", &evidence, &attestations, "iso_cert");

        assert!(!snapshot.is_eligible);
        assert_eq!(snapshot.attestation_statuses, vec!["rejected"]);
    }

    #[test]
    fn wrong_manufacturer_data_is_ignored() {
        let evidence = vec![iso_evidence("mfr-02", "ISO-9001-2024")];
        let attestations = vec![attested("mfr-02", "ISO-9001-2024", "verified")];

        let snapshot =
            build_compliance_snapshot("mfr-01", &evidence, &attestations, "iso_cert");

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

        let snapshot =
            build_compliance_snapshot("mfr-01", &evidence, &attestations, "iso_cert");

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

        let snapshot =
            build_compliance_snapshot("mfr-01", &evidence, &attestations, "iso_cert");

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

        let snapshot =
            build_compliance_snapshot("mfr-01", &evidence, &attestations, "iso_cert");

        assert_eq!(snapshot.attestation_statuses, vec!["expired", "verified"]);
        // eligible because ISO-B is verified
        assert!(snapshot.is_eligible);
    }
}
