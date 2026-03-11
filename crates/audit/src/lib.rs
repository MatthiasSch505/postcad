pub mod canonical;
pub use canonical::to_canonical_json;

pub mod registry_snapshot;
pub use registry_snapshot::hash_registry_snapshots;

pub mod chain;
pub use chain::{AuditEntry, AuditEvent, AuditLog};

pub mod verify;
pub use verify::{verify_receipt, VerificationFailure, VerificationResult};

pub mod receipt;
pub use receipt::RoutingAuditReceipt;

pub mod trace;
pub use trace::DecisionTrace;

pub mod fingerprint;
pub use fingerprint::RoutingDecisionFingerprint;

pub mod proof;
pub use proof::RoutingProof;

pub mod service;
pub use service::{
    route_case_with_audit, route_case_with_compliance_audit,
    route_case_with_profile_compliance_audit, RoutingServiceResult,
};
