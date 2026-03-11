use crate::attestation::{evidence_is_attested, EvidenceAttestation};
use crate::evidence::EligibilityEvidence;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredEvidenceProfile {
    pub profile_name: String,
    pub required_evidence_types: Vec<String>,
}

impl RequiredEvidenceProfile {
    pub fn new(profile_name: impl Into<String>, required_evidence_types: Vec<String>) -> Self {
        Self {
            profile_name: profile_name.into(),
            required_evidence_types,
        }
    }

    pub fn requires(&self, evidence_type: &str) -> bool {
        self.required_evidence_types
            .iter()
            .any(|t| t == evidence_type)
    }
}

/// Returns true only if the manufacturer has at least one verified attestation
/// for at least one evidence entry of each type required by `profile`.
pub fn manufacturer_satisfies_profile(
    manufacturer_id: &str,
    evidence: &[EligibilityEvidence],
    attestations: &[EvidenceAttestation],
    profile: &RequiredEvidenceProfile,
) -> bool {
    profile.required_evidence_types.iter().all(|required_type| {
        // Collect all evidence references for this manufacturer + type.
        let references: Vec<&str> = evidence
            .iter()
            .filter(|e| e.manufacturer_id == manufacturer_id && &e.evidence_type == required_type)
            .map(|e| e.evidence_reference.as_str())
            .collect();

        // At least one reference must be verified.
        references
            .iter()
            .any(|r| evidence_is_attested(manufacturer_id, r, attestations))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attestation::EvidenceAttestation;
    use crate::evidence::EligibilityEvidence;

    fn iso_evidence(mfr: &str, reference: &str) -> EligibilityEvidence {
        EligibilityEvidence::new(mfr, "iso_cert", reference)
    }

    fn fda_evidence(mfr: &str, reference: &str) -> EligibilityEvidence {
        EligibilityEvidence::new(mfr, "fda_clearance", reference)
    }

    fn verified(mfr: &str, reference: &str) -> EvidenceAttestation {
        EvidenceAttestation::new(mfr, reference, "registry-authority", "verified")
    }

    fn rejected(mfr: &str, reference: &str) -> EvidenceAttestation {
        EvidenceAttestation::new(mfr, reference, "registry-authority", "rejected")
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

    #[test]
    fn requires_returns_true_for_exact_match() {
        let profile = iso_profile();
        assert!(profile.requires("iso_cert"));
    }

    #[test]
    fn requires_returns_false_for_non_matching_type() {
        let profile = iso_profile();
        assert!(!profile.requires("fda_clearance"));
    }

    #[test]
    fn requires_is_case_sensitive() {
        let profile = iso_profile();
        assert!(!profile.requires("ISO_CERT"));
        assert!(!profile.requires("Iso_Cert"));
    }

    #[test]
    fn manufacturer_satisfies_single_type_profile() {
        let evidence = vec![iso_evidence("mfr-01", "ISO-9001-2024")];
        let attestations = vec![verified("mfr-01", "ISO-9001-2024")];

        assert!(manufacturer_satisfies_profile(
            "mfr-01",
            &evidence,
            &attestations,
            &iso_profile()
        ));
    }

    #[test]
    fn manufacturer_fails_profile_when_evidence_type_missing() {
        // No iso_cert evidence at all.
        let evidence = vec![fda_evidence("mfr-01", "FDA-510K-2024")];
        let attestations = vec![verified("mfr-01", "FDA-510K-2024")];

        assert!(!manufacturer_satisfies_profile(
            "mfr-01",
            &evidence,
            &attestations,
            &iso_profile()
        ));
    }

    #[test]
    fn manufacturer_fails_profile_when_attestation_is_rejected() {
        let evidence = vec![iso_evidence("mfr-01", "ISO-9001-2024")];
        let attestations = vec![rejected("mfr-01", "ISO-9001-2024")];

        assert!(!manufacturer_satisfies_profile(
            "mfr-01",
            &evidence,
            &attestations,
            &iso_profile()
        ));
    }

    #[test]
    fn manufacturer_fails_profile_when_no_attestation_present() {
        let evidence = vec![iso_evidence("mfr-01", "ISO-9001-2024")];

        assert!(!manufacturer_satisfies_profile(
            "mfr-01",
            &evidence,
            &[],
            &iso_profile()
        ));
    }

    #[test]
    fn wrong_manufacturer_evidence_is_ignored() {
        let evidence = vec![iso_evidence("mfr-02", "ISO-9001-2024")];
        let attestations = vec![verified("mfr-02", "ISO-9001-2024")];

        // mfr-01 has no evidence — should fail
        assert!(!manufacturer_satisfies_profile(
            "mfr-01",
            &evidence,
            &attestations,
            &iso_profile()
        ));
    }

    #[test]
    fn manufacturer_satisfies_dual_type_profile_when_both_present_and_verified() {
        let evidence = vec![
            iso_evidence("mfr-01", "ISO-9001-2024"),
            fda_evidence("mfr-01", "FDA-510K-2024"),
        ];
        let attestations = vec![
            verified("mfr-01", "ISO-9001-2024"),
            verified("mfr-01", "FDA-510K-2024"),
        ];

        assert!(manufacturer_satisfies_profile(
            "mfr-01",
            &evidence,
            &attestations,
            &dual_profile()
        ));
    }

    #[test]
    fn manufacturer_fails_dual_type_profile_when_one_type_missing() {
        // Only iso_cert — no fda_clearance.
        let evidence = vec![iso_evidence("mfr-01", "ISO-9001-2024")];
        let attestations = vec![verified("mfr-01", "ISO-9001-2024")];

        assert!(!manufacturer_satisfies_profile(
            "mfr-01",
            &evidence,
            &attestations,
            &dual_profile()
        ));
    }

    #[test]
    fn multiple_evidence_entries_one_verified_satisfies_type() {
        // Two iso_cert entries — only one is verified.
        let evidence = vec![
            iso_evidence("mfr-01", "ISO-OLD-2020"),
            iso_evidence("mfr-01", "ISO-9001-2024"),
        ];
        let attestations = vec![
            rejected("mfr-01", "ISO-OLD-2020"),
            verified("mfr-01", "ISO-9001-2024"),
        ];

        assert!(manufacturer_satisfies_profile(
            "mfr-01",
            &evidence,
            &attestations,
            &iso_profile()
        ));
    }
}
