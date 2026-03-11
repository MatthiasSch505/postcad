//! Registry-backed routing entry point.
//!
//! [`route_case_from_registry_json`] replaces the ad hoc fixture-shaped
//! `CandidateEntry` + `SnapshotEntry` inputs with candidate derivation from
//! typed [`ManufacturerRecord`] data. The full routing kernel is reused without
//! change; only the input construction path differs.
//!
//! # Derivation rules
//!
//! Given a list of [`ManufacturerRecord`]s, a case, and a config:
//!
//! 1. **Jurisdiction filter** — keep only manufacturers whose
//!    `jurisdictions_served` contains the request jurisdiction.
//! 2. **Candidate derivation** — for each remaining record:
//!    - `id` = `manufacturer_id` (stable; no separate candidate ID layer)
//!    - `location` = `"domestic"` if the manufacturer's country matches the
//!      case's `manufacturer_country`, else `"cross_border"`
//!    - `accepts_case` = `capabilities` contains the required procedure AND
//!      `materials_supported` contains the required material
//!    - `eligibility` = `"eligible"` if all attestations are `Verified` AND
//!      the list is non-empty; otherwise `"ineligible"`
//! 3. **Snapshot derivation** — for each record:
//!    - `evidence_references` = one synthetic ref per attestation entry
//!      (guarantees the validator's structural invariants)
//!    - `attestation_statuses` = attestations converted to strings
//!    - `is_eligible` = active AND all attestations Verified
//! 4. **Deterministic ordering** — candidates are sorted by `manufacturer_id`
//!    before being passed to the kernel.
//!
//! The derived [`RoutingPolicyBundle`] is serialized to JSON and forwarded to
//! [`route_case_from_policy_json`], so all receipt commitments, hash
//! invariants, and verification semantics are preserved exactly.

use serde::Deserialize;

use crate::policy_bundle::{CandidateEntry, RoutingPolicyBundle, SnapshotEntry};
use crate::{route_case_from_policy_json, CaseInput, CliError, RoutingReceipt};
use postcad_registry::{
    AttestationStatus, ManufacturerCapability, ManufacturerCountry, ManufacturerMaterial,
    ManufacturerRecord,
};

// ── Public types ──────────────────────────────────────────────────────────────

/// Minimal routing configuration for the registry-backed path.
///
/// All fields are optional; absent values fall back to the case's own fields
/// or well-known defaults.
#[derive(Debug, Deserialize)]
pub struct RegistryRoutingConfig {
    /// Jurisdiction code used for filtering `jurisdictions_served`
    /// (e.g. `"DE"`, `"US"`). Falls back to the case's `jurisdiction` field.
    pub jurisdiction: Option<String>,
    /// Routing policy variant. Falls back to the case's `routing_policy` field.
    pub routing_policy: Option<String>,
    /// Optional policy version label committed into the receipt proof.
    pub policy_version: Option<String>,
}

/// The result of a registry-backed routing call.
///
/// Contains both the routing receipt and the derived policy bundle JSON so
/// callers can pass the same bundle to [`verify_receipt_from_policy_json`]
/// without having to re-derive it.
pub struct RegistryRoutingResult {
    /// The routing receipt returned by the kernel.
    pub receipt: RoutingReceipt,
    /// The derived `RoutingPolicyBundle` serialized to JSON.
    ///
    /// Pass this to `verify_receipt_from_policy_json` together with the
    /// original `case_json` to verify the receipt.
    pub derived_policy_json: String,
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Routes a case using typed [`ManufacturerRecord`] data as the registry source.
///
/// # Arguments
///
/// - `case_json` — JSON object matching [`CaseInput`].
/// - `registry_json` — JSON array of [`ManufacturerRecord`] objects.
/// - `config_json` — JSON object matching [`RegistryRoutingConfig`].
///
/// # Returns
///
/// A [`RegistryRoutingResult`] containing the receipt and the derived policy
/// bundle JSON (needed for verification).
pub fn route_case_from_registry_json(
    case_json: &str,
    registry_json: &str,
    config_json: &str,
) -> Result<RegistryRoutingResult, CliError> {
    let case_input: CaseInput = serde_json::from_str(case_json)?;
    let records: Vec<ManufacturerRecord> = serde_json::from_str(registry_json)?;
    let config: RegistryRoutingConfig = serde_json::from_str(config_json)?;

    // Resolve jurisdiction: config > case > default.
    let jurisdiction = config
        .jurisdiction
        .as_deref()
        .or(case_input.jurisdiction.as_deref())
        .unwrap_or("global");

    // Resolve routing_policy: config > case > default.
    let routing_policy = config
        .routing_policy
        .as_deref()
        .or(case_input.routing_policy.as_deref())
        .map(|s| s.to_string());

    // Resolve case fields for filtering.
    let case_capability = parse_mfr_capability(&case_input.procedure);
    let case_material = parse_mfr_material(&case_input.material);

    // 1. Filter by jurisdiction, active status, capability, and material.
    //    Applied stepwise so we can derive the most specific refusal reason.
    let jurisdiction_country = parse_jurisdiction_to_country(jurisdiction);
    let mut in_jurisdiction: Vec<&ManufacturerRecord> = records
        .iter()
        .filter(|r| {
            // Jurisdiction: must serve the request jurisdiction.
            let serves = jurisdiction_country
                .as_ref()
                .map_or(true, |target| r.jurisdictions_served.contains(target));
            // Capability: must support the required procedure.
            let has_capability = case_capability
                .as_ref()
                .map_or(true, |cap| r.capabilities.contains(cap));
            // Material: must support the required material.
            let has_material = case_material
                .as_ref()
                .map_or(true, |mat| r.materials_supported.contains(mat));
            serves && has_capability && has_material
        })
        .collect();

    // 2. Deterministic ordering: sort by manufacturer_id.
    in_jurisdiction.sort_by(|a, b| a.manufacturer_id.cmp(&b.manufacturer_id));

    // 3. Derive candidates and snapshots.
    let candidates = derive_candidates(&in_jurisdiction, &case_input);
    let snapshots = derive_snapshots(&in_jurisdiction);

    // 4. Compute the most specific refusal reason code for the case where
    //    no eligible candidates remain. Applied only when the candidate pool
    //    has no eligible entries; ignored for successful routing.
    let refusal_reason_hint = compute_refusal_hint(
        &records,
        &jurisdiction_country,
        &case_capability,
        &case_material,
        &candidates,
    );

    let bundle = RoutingPolicyBundle {
        jurisdiction: Some(jurisdiction.to_string()),
        routing_policy,
        compliance_profile: None,
        policy_version: config.policy_version,
        refusal_reason_hint,
        candidates,
        snapshots,
    };

    let derived_policy_json = serde_json::to_string(&bundle)?;
    let receipt = route_case_from_policy_json(case_json, &derived_policy_json)?;

    Ok(RegistryRoutingResult {
        receipt,
        derived_policy_json,
    })
}

// ── Refusal reason derivation ─────────────────────────────────────────────────

/// Derives the most specific stable refusal reason code from stepwise filtering facts.
///
/// Returns `None` when routing will succeed (at least one eligible candidate exists).
/// Returns `Some(code)` when no eligible candidates remain, where `code` is the
/// first filter step that emptied the candidate pool:
///
/// 1. `no_active_manufacturer`  — all records are inactive
/// 2. `no_jurisdiction_match`   — no active records serve the jurisdiction
/// 3. `no_capability_match`     — no active+jurisdiction records have the capability
/// 4. `no_material_match`       — no active+jurisdiction+capability records support the material
/// 5. `attestation_failed`      — records passed all structural filters but all are ineligible
/// 6. `no_eligible_manufacturer` — fallback (empty registry or unknown cause)
fn compute_refusal_hint(
    records: &[ManufacturerRecord],
    jurisdiction_country: &Option<ManufacturerCountry>,
    case_capability: &Option<ManufacturerCapability>,
    case_material: &Option<ManufacturerMaterial>,
    derived_candidates: &[crate::policy_bundle::CandidateEntry],
) -> Option<String> {
    // If at least one candidate is eligible, routing will succeed — no hint needed.
    if derived_candidates
        .iter()
        .any(|c| c.eligibility == "eligible")
    {
        return None;
    }

    if records.is_empty() {
        return Some("no_eligible_manufacturer".to_string());
    }

    // Step 1: active manufacturers.
    let active: Vec<&ManufacturerRecord> = records.iter().filter(|r| r.is_active).collect();
    if active.is_empty() {
        return Some("no_active_manufacturer".to_string());
    }

    // Step 2: jurisdiction filter.
    let in_juri: Vec<&&ManufacturerRecord> = active
        .iter()
        .filter(|r| {
            jurisdiction_country
                .as_ref()
                .map_or(true, |target| r.jurisdictions_served.contains(target))
        })
        .collect();
    if in_juri.is_empty() {
        return Some("no_jurisdiction_match".to_string());
    }

    // Step 3: capability filter.
    let with_cap: Vec<&&&ManufacturerRecord> = in_juri
        .iter()
        .filter(|r| {
            case_capability
                .as_ref()
                .map_or(true, |cap| r.capabilities.contains(cap))
        })
        .collect();
    if with_cap.is_empty() {
        return Some("no_capability_match".to_string());
    }

    // Step 4: material filter.
    let with_mat: Vec<&&&&ManufacturerRecord> = with_cap
        .iter()
        .filter(|r| {
            case_material
                .as_ref()
                .map_or(true, |mat| r.materials_supported.contains(mat))
        })
        .collect();
    if with_mat.is_empty() {
        return Some("no_material_match".to_string());
    }

    // Candidates exist but all failed attestation / active checks.
    Some("attestation_failed".to_string())
}

// ── Candidate + snapshot derivation ──────────────────────────────────────────

fn derive_candidates(records: &[&ManufacturerRecord], case: &CaseInput) -> Vec<CandidateEntry> {
    // Records reaching this point are already filtered by jurisdiction,
    // capability, and material — so accepts_case is always true here.
    let case_mfr_country = parse_mfr_country_str(&case.manufacturer_country);

    records
        .iter()
        .map(|r| {
            let location = case_mfr_country.as_ref().map_or("cross_border", |target| {
                if &r.country == target {
                    "domestic"
                } else {
                    "cross_border"
                }
            });

            let is_eligible_attestation = !r.attestation_statuses.is_empty()
                && r.attestation_statuses.iter().all(|s| s.is_compliant());
            let eligibility = if r.is_active && is_eligible_attestation {
                "eligible"
            } else {
                "ineligible"
            };

            CandidateEntry {
                id: r.manufacturer_id.clone(),
                manufacturer_id: r.manufacturer_id.clone(),
                location: location.to_string(),
                accepts_case: true,
                eligibility: eligibility.to_string(),
            }
        })
        .collect()
}

fn derive_snapshots(records: &[&ManufacturerRecord]) -> Vec<SnapshotEntry> {
    records
        .iter()
        .map(|r| {
            let statuses: Vec<String> = r
                .attestation_statuses
                .iter()
                .map(attestation_to_str)
                .collect();

            // Synthetic evidence references — one per attestation entry so the
            // snapshot validator's structural invariants are satisfied
            // (attestation_statuses.len() <= evidence_references.len()).
            let evidence_references: Vec<String> = statuses
                .iter()
                .enumerate()
                .map(|(i, _)| format!("registry::{}::{}", r.manufacturer_id, i))
                .collect();

            let is_eligible = r.is_active
                && !r.attestation_statuses.is_empty()
                && r.attestation_statuses.iter().all(|s| s.is_compliant());

            SnapshotEntry {
                manufacturer_id: r.manufacturer_id.clone(),
                evidence_references,
                attestation_statuses: statuses,
                is_eligible,
            }
        })
        .collect()
}

// ── String conversion helpers ─────────────────────────────────────────────────

fn attestation_to_str(s: &AttestationStatus) -> String {
    match s {
        AttestationStatus::Verified => "verified",
        AttestationStatus::Pending => "pending",
        AttestationStatus::Expired => "expired",
        AttestationStatus::Revoked => "revoked",
    }
    .to_string()
}

/// Maps a jurisdiction code or country name string to a [`ManufacturerCountry`].
/// Returns `None` for `"global"` and unrecognised strings (no filtering).
fn parse_jurisdiction_to_country(s: &str) -> Option<ManufacturerCountry> {
    parse_mfr_country_str(s)
}

/// Maps a country name or ISO-2 code string to [`ManufacturerCountry`].
fn parse_mfr_country_str(s: &str) -> Option<ManufacturerCountry> {
    match s.to_lowercase().as_str() {
        "de" | "germany" => Some(ManufacturerCountry::Germany),
        "us" | "united_states" | "unitedstates" => Some(ManufacturerCountry::UnitedStates),
        "fr" | "france" => Some(ManufacturerCountry::France),
        "jp" | "japan" => Some(ManufacturerCountry::Japan),
        "gb" | "uk" | "united_kingdom" | "unitedkingdom" => {
            Some(ManufacturerCountry::UnitedKingdom)
        }
        _ => None,
    }
}

fn parse_mfr_capability(s: &str) -> Option<ManufacturerCapability> {
    match s.to_lowercase().as_str() {
        "crown" => Some(ManufacturerCapability::Crown),
        "bridge" => Some(ManufacturerCapability::Bridge),
        "veneer" => Some(ManufacturerCapability::Veneer),
        "implant" => Some(ManufacturerCapability::Implant),
        "denture" => Some(ManufacturerCapability::Denture),
        _ => None,
    }
}

fn parse_mfr_material(s: &str) -> Option<ManufacturerMaterial> {
    match s.to_lowercase().as_str() {
        "zirconia" => Some(ManufacturerMaterial::Zirconia),
        "pmma" => Some(ManufacturerMaterial::Pmma),
        "emax" => Some(ManufacturerMaterial::Emax),
        "cobalt_chrome" | "cobaltchrome" => Some(ManufacturerMaterial::CobaltChrome),
        "titanium" => Some(ManufacturerMaterial::Titanium),
        _ => None,
    }
}
