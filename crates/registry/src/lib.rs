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
