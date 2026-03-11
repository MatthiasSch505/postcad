pub mod manufacturer;
pub use manufacturer::{
    canonical_hash as canonical_manufacturer_hash, eligible_records, filter_active,
    filter_by_attestation, filter_by_capability, filter_by_jurisdiction, filter_by_material,
    AttestationStatus, ManufacturerCapability, ManufacturerCountry, ManufacturerMaterial,
    ManufacturerRecord,
};

pub mod evidence;
pub use evidence::{manufacturer_has_evidence, EligibilityEvidence};

pub mod attestation;
pub use attestation::{evidence_attestation_status, evidence_is_attested, EvidenceAttestation};

pub mod snapshot;
pub use snapshot::{build_compliance_snapshot, build_compliance_snapshot_for_profile, ManufacturerComplianceSnapshot};

pub mod profile;
pub use profile::{manufacturer_satisfies_profile, RequiredEvidenceProfile};

pub mod snapshot_validator;
pub use snapshot_validator::{validate_snapshots, SnapshotValidationError};
