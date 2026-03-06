pub mod evidence;
pub use evidence::{manufacturer_has_evidence, EligibilityEvidence};

pub mod attestation;
pub use attestation::{evidence_attestation_status, evidence_is_attested, EvidenceAttestation};
