use crate::snapshot::ManufacturerComplianceSnapshot;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SnapshotValidationError {
    #[error("empty manufacturer_id at snapshot index {index}")]
    EmptyManufacturerId { index: usize },

    #[error("duplicate manufacturer_id: '{manufacturer_id}'")]
    DuplicateManufacturerId { manufacturer_id: String },

    #[error(
        "snapshot for '{manufacturer_id}' has {attestation_count} attestation status(es) \
        but only {evidence_count} evidence reference(s); counts cannot exceed references"
    )]
    ExcessAttestationStatuses {
        manufacturer_id: String,
        evidence_count: usize,
        attestation_count: usize,
    },

    #[error(
        "duplicate evidence reference '{reference}' in snapshot for '{manufacturer_id}'"
    )]
    DuplicateEvidenceReference {
        manufacturer_id: String,
        reference: String,
    },

    #[error(
        "snapshot for '{manufacturer_id}' is marked eligible but has no evidence references"
    )]
    EligibleWithNoEvidence { manufacturer_id: String },

    #[error(
        "snapshot for '{manufacturer_id}' is marked eligible but has no verified attestation"
    )]
    EligibleWithNoVerifiedAttestation { manufacturer_id: String },
}

/// Validates a slice of compliance snapshots before they are used for routing.
///
/// Returns the first error found. Checks are applied in the order listed.
pub fn validate_snapshots(
    snapshots: &[ManufacturerComplianceSnapshot],
) -> Result<(), SnapshotValidationError> {
    let mut seen_ids: HashSet<&str> = HashSet::new();

    for (index, snapshot) in snapshots.iter().enumerate() {
        // 1. manufacturer_id must be non-empty.
        if snapshot.manufacturer_id.is_empty() {
            return Err(SnapshotValidationError::EmptyManufacturerId { index });
        }

        // 2. manufacturer_id must be unique across the slice.
        if !seen_ids.insert(snapshot.manufacturer_id.as_str()) {
            return Err(SnapshotValidationError::DuplicateManufacturerId {
                manufacturer_id: snapshot.manufacturer_id.clone(),
            });
        }

        // 3. Attestation statuses must not outnumber evidence references.
        //    (Each status corresponds to one reference; more statuses than references
        //    means the data is structurally misaligned.)
        if snapshot.attestation_statuses.len() > snapshot.evidence_references.len() {
            return Err(SnapshotValidationError::ExcessAttestationStatuses {
                manufacturer_id: snapshot.manufacturer_id.clone(),
                evidence_count: snapshot.evidence_references.len(),
                attestation_count: snapshot.attestation_statuses.len(),
            });
        }

        // 4. Evidence references must be unique within a snapshot.
        let mut seen_refs: HashSet<&str> = HashSet::new();
        for reference in &snapshot.evidence_references {
            if !seen_refs.insert(reference.as_str()) {
                return Err(SnapshotValidationError::DuplicateEvidenceReference {
                    manufacturer_id: snapshot.manufacturer_id.clone(),
                    reference: reference.clone(),
                });
            }
        }

        // 5 & 6. Contradictory eligibility flag.
        if snapshot.is_eligible {
            if snapshot.evidence_references.is_empty() {
                return Err(SnapshotValidationError::EligibleWithNoEvidence {
                    manufacturer_id: snapshot.manufacturer_id.clone(),
                });
            }
            let has_verified = snapshot
                .attestation_statuses
                .iter()
                .any(|s| s == "verified");
            if !has_verified {
                return Err(SnapshotValidationError::EligibleWithNoVerifiedAttestation {
                    manufacturer_id: snapshot.manufacturer_id.clone(),
                });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::ManufacturerComplianceSnapshot;

    fn snap(
        manufacturer_id: &str,
        evidence_references: Vec<&str>,
        attestation_statuses: Vec<&str>,
        is_eligible: bool,
    ) -> ManufacturerComplianceSnapshot {
        ManufacturerComplianceSnapshot::new(
            manufacturer_id,
            evidence_references.into_iter().map(str::to_string).collect(),
            attestation_statuses.into_iter().map(str::to_string).collect(),
            is_eligible,
        )
    }

    // ── valid cases ───────────────────────────────────────────────────────────

    #[test]
    fn valid_eligible_snapshot_passes() {
        let snapshots = vec![snap("mfr-01", vec!["ISO-001"], vec!["verified"], true)];
        assert!(validate_snapshots(&snapshots).is_ok());
    }

    #[test]
    fn valid_ineligible_snapshot_passes() {
        let snapshots = vec![snap("mfr-01", vec!["ISO-001"], vec!["rejected"], false)];
        assert!(validate_snapshots(&snapshots).is_ok());
    }

    #[test]
    fn valid_ineligible_snapshot_no_attestation_passes() {
        // Evidence present but no attestation — ineligible, which is fine.
        let snapshots = vec![snap("mfr-01", vec!["ISO-001"], vec![], false)];
        assert!(validate_snapshots(&snapshots).is_ok());
    }

    #[test]
    fn valid_ineligible_snapshot_no_evidence_passes() {
        // No evidence at all — ineligible, which is fine.
        let snapshots = vec![snap("mfr-01", vec![], vec![], false)];
        assert!(validate_snapshots(&snapshots).is_ok());
    }

    #[test]
    fn valid_multiple_snapshots_pass() {
        let snapshots = vec![
            snap("mfr-01", vec!["ISO-001"], vec!["verified"], true),
            snap("mfr-02", vec!["ISO-002"], vec!["rejected"], false),
        ];
        assert!(validate_snapshots(&snapshots).is_ok());
    }

    #[test]
    fn empty_slice_passes() {
        assert!(validate_snapshots(&[]).is_ok());
    }

    #[test]
    fn valid_multiple_evidence_refs_fewer_attestations_passes() {
        // Two refs but only one attestation — allowed (second ref not yet attested).
        let snapshots = vec![snap(
            "mfr-01",
            vec!["ISO-A", "ISO-B"],
            vec!["verified"],
            true,
        )];
        assert!(validate_snapshots(&snapshots).is_ok());
    }

    // ── empty manufacturer_id ─────────────────────────────────────────────────

    #[test]
    fn empty_manufacturer_id_fails() {
        let snapshots = vec![snap("", vec![], vec![], false)];
        assert_eq!(
            validate_snapshots(&snapshots),
            Err(SnapshotValidationError::EmptyManufacturerId { index: 0 })
        );
    }

    #[test]
    fn empty_manufacturer_id_at_second_index_reported_correctly() {
        let snapshots = vec![
            snap("mfr-01", vec![], vec![], false),
            snap("", vec![], vec![], false),
        ];
        assert_eq!(
            validate_snapshots(&snapshots),
            Err(SnapshotValidationError::EmptyManufacturerId { index: 1 })
        );
    }

    // ── duplicate manufacturer_id ─────────────────────────────────────────────

    #[test]
    fn duplicate_manufacturer_id_fails() {
        let snapshots = vec![
            snap("mfr-01", vec!["ISO-001"], vec!["verified"], true),
            snap("mfr-01", vec!["ISO-002"], vec!["verified"], true),
        ];
        assert_eq!(
            validate_snapshots(&snapshots),
            Err(SnapshotValidationError::DuplicateManufacturerId {
                manufacturer_id: "mfr-01".to_string()
            })
        );
    }

    // ── excess attestation statuses ───────────────────────────────────────────

    #[test]
    fn more_attestations_than_evidence_fails() {
        let snapshots = vec![snap(
            "mfr-01",
            vec!["ISO-001"],
            vec!["verified", "verified"],
            true,
        )];
        assert_eq!(
            validate_snapshots(&snapshots),
            Err(SnapshotValidationError::ExcessAttestationStatuses {
                manufacturer_id: "mfr-01".to_string(),
                evidence_count: 1,
                attestation_count: 2,
            })
        );
    }

    #[test]
    fn attestations_with_no_evidence_fails() {
        let snapshots = vec![snap("mfr-01", vec![], vec!["verified"], false)];
        assert_eq!(
            validate_snapshots(&snapshots),
            Err(SnapshotValidationError::ExcessAttestationStatuses {
                manufacturer_id: "mfr-01".to_string(),
                evidence_count: 0,
                attestation_count: 1,
            })
        );
    }

    // ── duplicate evidence references ─────────────────────────────────────────

    #[test]
    fn duplicate_evidence_reference_fails() {
        let snapshots = vec![snap(
            "mfr-01",
            vec!["ISO-001", "ISO-001"],
            vec!["verified"],
            true,
        )];
        assert_eq!(
            validate_snapshots(&snapshots),
            Err(SnapshotValidationError::DuplicateEvidenceReference {
                manufacturer_id: "mfr-01".to_string(),
                reference: "ISO-001".to_string(),
            })
        );
    }

    // ── eligible with no evidence ─────────────────────────────────────────────

    #[test]
    fn eligible_with_no_evidence_fails() {
        let snapshots = vec![snap("mfr-01", vec![], vec![], true)];
        assert_eq!(
            validate_snapshots(&snapshots),
            Err(SnapshotValidationError::EligibleWithNoEvidence {
                manufacturer_id: "mfr-01".to_string()
            })
        );
    }

    // ── eligible with no verified attestation ─────────────────────────────────

    #[test]
    fn eligible_with_only_rejected_attestation_fails() {
        let snapshots = vec![snap("mfr-01", vec!["ISO-001"], vec!["rejected"], true)];
        assert_eq!(
            validate_snapshots(&snapshots),
            Err(SnapshotValidationError::EligibleWithNoVerifiedAttestation {
                manufacturer_id: "mfr-01".to_string()
            })
        );
    }

    #[test]
    fn eligible_with_no_attestation_at_all_fails() {
        let snapshots = vec![snap("mfr-01", vec!["ISO-001"], vec![], true)];
        assert_eq!(
            validate_snapshots(&snapshots),
            Err(SnapshotValidationError::EligibleWithNoVerifiedAttestation {
                manufacturer_id: "mfr-01".to_string()
            })
        );
    }

    #[test]
    fn eligible_with_mixed_attestations_passes_if_any_verified() {
        // One rejected, one verified — the verified one saves it.
        let snapshots = vec![snap(
            "mfr-01",
            vec!["ISO-OLD", "ISO-001"],
            vec!["rejected", "verified"],
            true,
        )];
        assert!(validate_snapshots(&snapshots).is_ok());
    }
}
