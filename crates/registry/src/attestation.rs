#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceAttestation {
    pub manufacturer_id: String,
    pub evidence_reference: String,
    pub attested_by: String,
    pub status: String,
}

impl EvidenceAttestation {
    pub fn new(
        manufacturer_id: impl Into<String>,
        evidence_reference: impl Into<String>,
        attested_by: impl Into<String>,
        status: impl Into<String>,
    ) -> Self {
        Self {
            manufacturer_id: manufacturer_id.into(),
            evidence_reference: evidence_reference.into(),
            attested_by: attested_by.into(),
            status: status.into(),
        }
    }
}

fn find_attestation<'a>(
    manufacturer_id: &str,
    evidence_reference: &str,
    attestations: &'a [EvidenceAttestation],
) -> Option<&'a EvidenceAttestation> {
    attestations.iter().find(|a| {
        a.manufacturer_id == manufacturer_id && a.evidence_reference == evidence_reference
    })
}

/// Returns true only if a matching attestation exists with status `"verified"`.
pub fn evidence_is_attested(
    manufacturer_id: &str,
    evidence_reference: &str,
    attestations: &[EvidenceAttestation],
) -> bool {
    find_attestation(manufacturer_id, evidence_reference, attestations)
        .map(|a| a.status == "verified")
        .unwrap_or(false)
}

/// Returns the status of the first matching attestation, or `None` if not found.
pub fn evidence_attestation_status<'a>(
    manufacturer_id: &str,
    evidence_reference: &str,
    attestations: &'a [EvidenceAttestation],
) -> Option<&'a str> {
    find_attestation(manufacturer_id, evidence_reference, attestations).map(|a| a.status.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verified(mfr: &str, reference: &str) -> EvidenceAttestation {
        EvidenceAttestation::new(mfr, reference, "registry-authority", "verified")
    }

    fn rejected(mfr: &str, reference: &str) -> EvidenceAttestation {
        EvidenceAttestation::new(mfr, reference, "registry-authority", "rejected")
    }

    #[test]
    fn verified_attestation_returns_true() {
        let attestations = vec![verified("mfr-01", "ISO-9001-2024")];
        assert!(evidence_is_attested(
            "mfr-01",
            "ISO-9001-2024",
            &attestations
        ));
    }

    #[test]
    fn rejected_attestation_returns_false() {
        let attestations = vec![rejected("mfr-01", "ISO-9001-2024")];
        assert!(!evidence_is_attested(
            "mfr-01",
            "ISO-9001-2024",
            &attestations
        ));
    }

    #[test]
    fn wrong_manufacturer_is_ignored() {
        let attestations = vec![verified("mfr-02", "ISO-9001-2024")];
        assert!(!evidence_is_attested(
            "mfr-01",
            "ISO-9001-2024",
            &attestations
        ));
    }

    #[test]
    fn wrong_evidence_reference_is_ignored() {
        let attestations = vec![verified("mfr-01", "ISO-13485-2024")];
        assert!(!evidence_is_attested(
            "mfr-01",
            "ISO-9001-2024",
            &attestations
        ));
    }

    #[test]
    fn status_lookup_returns_some_verified_when_present() {
        let attestations = vec![verified("mfr-01", "ISO-9001-2024")];
        assert_eq!(
            evidence_attestation_status("mfr-01", "ISO-9001-2024", &attestations),
            Some("verified")
        );
    }

    #[test]
    fn status_lookup_returns_none_when_absent() {
        let attestations = vec![verified("mfr-01", "ISO-9001-2024")];
        assert_eq!(
            evidence_attestation_status("mfr-01", "MILL-REF-999", &attestations),
            None
        );
    }

    #[test]
    fn multiple_attestations_preserve_first_match_order() {
        let attestations = vec![
            rejected("mfr-01", "ISO-9001-2024"),
            verified("mfr-01", "ISO-9001-2024"),
        ];
        // first match is "rejected" — verified entry is ignored
        assert!(!evidence_is_attested(
            "mfr-01",
            "ISO-9001-2024",
            &attestations
        ));
        assert_eq!(
            evidence_attestation_status("mfr-01", "ISO-9001-2024", &attestations),
            Some("rejected")
        );
    }
}
