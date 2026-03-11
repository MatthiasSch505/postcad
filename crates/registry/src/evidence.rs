#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EligibilityEvidence {
    pub manufacturer_id: String,
    pub evidence_type: String,
    pub evidence_reference: String,
}

impl EligibilityEvidence {
    pub fn new(
        manufacturer_id: impl Into<String>,
        evidence_type: impl Into<String>,
        evidence_reference: impl Into<String>,
    ) -> Self {
        Self {
            manufacturer_id: manufacturer_id.into(),
            evidence_type: evidence_type.into(),
            evidence_reference: evidence_reference.into(),
        }
    }
}

/// Returns true if `manufacturer_id` has at least one entry in `evidence`
/// with `evidence_type` matching `required_type`.
pub fn manufacturer_has_evidence(
    manufacturer_id: &str,
    evidence: &[EligibilityEvidence],
    required_type: &str,
) -> bool {
    evidence
        .iter()
        .any(|e| e.manufacturer_id == manufacturer_id && e.evidence_type == required_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manufacturer_with_matching_evidence_returns_true() {
        let evidence = vec![EligibilityEvidence::new(
            "mfr-01",
            "iso_cert",
            "ISO-9001-2024",
        )];
        assert!(manufacturer_has_evidence("mfr-01", &evidence, "iso_cert"));
    }

    #[test]
    fn manufacturer_without_matching_type_returns_false() {
        let evidence = vec![EligibilityEvidence::new(
            "mfr-01",
            "iso_cert",
            "ISO-9001-2024",
        )];
        assert!(!manufacturer_has_evidence(
            "mfr-01",
            &evidence,
            "regulatory_license"
        ));
    }

    #[test]
    fn evidence_for_different_manufacturer_is_ignored() {
        let evidence = vec![EligibilityEvidence::new(
            "mfr-02",
            "iso_cert",
            "ISO-9001-2024",
        )];
        assert!(!manufacturer_has_evidence("mfr-01", &evidence, "iso_cert"));
    }

    #[test]
    fn multiple_evidence_entries_handled_deterministically() {
        let evidence = vec![
            EligibilityEvidence::new("mfr-01", "milling_validation", "MILL-REF-001"),
            EligibilityEvidence::new("mfr-01", "iso_cert", "ISO-13485-2024"),
            EligibilityEvidence::new("mfr-02", "regulatory_license", "LIC-DE-2024"),
        ];
        assert!(manufacturer_has_evidence("mfr-01", &evidence, "iso_cert"));
        assert!(manufacturer_has_evidence(
            "mfr-01",
            &evidence,
            "milling_validation"
        ));
        assert!(!manufacturer_has_evidence(
            "mfr-01",
            &evidence,
            "regulatory_license"
        ));
        assert!(manufacturer_has_evidence(
            "mfr-02",
            &evidence,
            "regulatory_license"
        ));
        assert!(!manufacturer_has_evidence("mfr-02", &evidence, "iso_cert"));
    }

    #[test]
    fn empty_evidence_slice_returns_false() {
        assert!(!manufacturer_has_evidence("mfr-01", &[], "iso_cert"));
    }
}
