//! Stable public artifact schema for the routing policy bundle.
//!
//! `RoutingPolicyBundle` is the canonical input document for the
//! `verify-receipt` command. It bundles everything the routing kernel needs
//! besides the case itself: jurisdiction, routing policy, optional compliance
//! profile, candidate list, and compliance snapshots.
//!
//! This module has no dependency on internal routing or audit domain types.
//! Any change to the field names or types here is a breaking change to the
//! CLI surface; internal refactors should not require touching this file.

use serde::Deserialize;

/// Stable input bundle for the `verify-receipt` command.
///
/// Parsed from `--policy policy.json`. All fields that have a sensible default
/// are `Option<String>` and documented with their default value.
#[derive(Debug, Deserialize)]
pub struct RoutingPolicyBundle {
    /// Jurisdiction code (e.g. `"DE"`, `"US"`). Defaults to `"global"`.
    pub jurisdiction: Option<String>,
    /// Routing policy variant. Defaults to `"allow_domestic_and_cross_border"`.
    /// Stable values: `"allow_domestic_only"`, `"allow_domestic_and_cross_border"`.
    pub routing_policy: Option<String>,
    /// Optional compliance profile name used for evidence-based filtering.
    pub compliance_profile: Option<String>,
    /// Routing candidates to evaluate.
    pub candidates: Vec<CandidateEntry>,
    /// Compliance snapshots for the manufacturers referenced in `candidates`.
    pub snapshots: Vec<SnapshotEntry>,
}

/// One routing candidate entry within a [`RoutingPolicyBundle`].
#[derive(Debug, Deserialize)]
pub struct CandidateEntry {
    /// Stable routing candidate identifier.
    pub id: String,
    /// Manufacturer this candidate represents.
    pub manufacturer_id: String,
    /// `"domestic"` | `"cross_border"` | `"unknown"`
    pub location: String,
    /// Whether the manufacturer has accepted this type of case.
    pub accepts_case: bool,
    /// `"eligible"` | `"ineligible"` | `"unknown"`
    pub eligibility: String,
}

/// One compliance snapshot entry within a [`RoutingPolicyBundle`].
#[derive(Debug, Deserialize)]
pub struct SnapshotEntry {
    /// Manufacturer this snapshot covers.
    pub manufacturer_id: String,
    /// Evidence document references (e.g. certification IDs).
    pub evidence_references: Vec<String>,
    /// Attestation status per evidence reference.
    pub attestation_statuses: Vec<String>,
    /// Whether this manufacturer is considered eligible under the snapshot.
    pub is_eligible: bool,
}
